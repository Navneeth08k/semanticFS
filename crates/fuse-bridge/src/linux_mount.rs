#[cfg(target_os = "linux")]
use anyhow::{Context, Result};
#[cfg(target_os = "linux")]
use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
    ReplyOpen, Request,
};
#[cfg(target_os = "linux")]
use libc::{ENOENT, EROFS};
#[cfg(target_os = "linux")]
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};

#[cfg(target_os = "linux")]
use crate::{FuseBridge, FuseSessionMode, RawNodeKind};

#[cfg(target_os = "linux")]
const TTL: Duration = Duration::from_millis(300);
#[cfg(target_os = "linux")]
const HEALTH_PATH: &str = "/.well-known/health.json";
#[cfg(target_os = "linux")]
const SESSION_STATUS_PATH: &str = "/.well-known/session.json";
#[cfg(target_os = "linux")]
const SESSION_REFRESH_PATH: &str = "/.well-known/session.refresh";
#[cfg(target_os = "linux")]
const FOPEN_DIRECT_IO_FLAG: u32 = 1;

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct Node {
    path: String,
    kind: FileType,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SessionKey {
    uid: u32,
    pid: u32,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
struct SessionPin {
    snapshot_version: u64,
    pinned_at: SystemTime,
    last_access: SystemTime,
}

#[cfg(target_os = "linux")]
pub fn serve_mount(bridge: FuseBridge) -> Result<()> {
    let mountpoint = bridge_mountpoint(&bridge)?;
    let fs = SemanticFsFuse::new(Arc::new(bridge));

    let options = vec![
        MountOption::RO,
        MountOption::FSName("semanticfs".to_string()),
        MountOption::AutoUnmount,
    ];

    fuser::mount2(fs, mountpoint, &options).context("mount semanticfs fuse")
}

#[cfg(target_os = "linux")]
fn bridge_mountpoint(bridge: &FuseBridge) -> Result<PathBuf> {
    Ok(PathBuf::from(bridge.mount_point()))
}

#[cfg(target_os = "linux")]
struct SemanticFsFuse {
    bridge: Arc<FuseBridge>,
    inode_to_node: HashMap<u64, Node>,
    path_to_inode: HashMap<String, u64>,
    session_pins: HashMap<SessionKey, SessionPin>,
    session_mode: FuseSessionMode,
    max_session_entries: usize,
}

#[cfg(target_os = "linux")]
impl SemanticFsFuse {
    fn new(bridge: Arc<FuseBridge>) -> Self {
        let mut inode_to_node = HashMap::new();
        let mut path_to_inode = HashMap::new();
        let session_mode = bridge.fuse_session_mode();
        let max_session_entries = bridge.fuse_session_max_entries();

        for (path, kind) in [
            ("/".to_string(), FileType::Directory),
            ("/raw".to_string(), FileType::Directory),
            ("/search".to_string(), FileType::Directory),
            ("/map".to_string(), FileType::Directory),
            ("/.well-known".to_string(), FileType::Directory),
            (HEALTH_PATH.to_string(), FileType::RegularFile),
            (SESSION_STATUS_PATH.to_string(), FileType::RegularFile),
            (SESSION_REFRESH_PATH.to_string(), FileType::RegularFile),
        ] {
            let ino = if path == "/" {
                fuser::FUSE_ROOT_ID
            } else {
                hash_inode(&path)
            };
            inode_to_node.insert(
                ino,
                Node {
                    path: path.clone(),
                    kind,
                },
            );
            path_to_inode.insert(path, ino);
        }

        Self {
            bridge,
            inode_to_node,
            path_to_inode,
            session_pins: HashMap::new(),
            session_mode,
            max_session_entries,
        }
    }

    fn attr_for(&self, ino: u64) -> Option<FileAttr> {
        let node = self.inode_to_node.get(&ino)?;

        let now = SystemTime::now();
        let perm = match node.kind {
            FileType::Directory => 0o555,
            _ => 0o444,
        };

        let size = if node.kind == FileType::RegularFile {
            if let Some(raw_path) = node.path.strip_prefix("/raw/") {
                self.bridge
                    .raw_node_info(raw_path)
                    .map(|(_, size)| size)
                    .unwrap_or(0)
            } else {
                // Virtual files are rendered dynamically at read-time.
                // Advertise a non-zero size so clients issue reads instead of treating as EOF.
                4096
            }
        } else {
            0
        };

        Some(FileAttr {
            ino,
            size,
            blocks: 1,
            atime: now,
            mtime: now,
            ctime: now,
            crtime: now,
            kind: node.kind,
            perm,
            nlink: 1,
            uid: unsafe { libc::geteuid() },
            gid: unsafe { libc::getegid() },
            rdev: 0,
            blksize: 512,
            flags: 0,
        })
    }

    fn ensure_node(&mut self, parent: u64, name: &OsStr) -> Option<u64> {
        let parent_path = self.inode_to_node.get(&parent)?.path.clone();
        let name_str = name.to_string_lossy();

        let full = if parent_path == "/" {
            format!("/{}", name_str)
        } else {
            format!("{}/{}", parent_path, name_str)
        };

        if let Some(existing) = self.path_to_inode.get(&full) {
            return Some(*existing);
        }

        let kind = if parent_path.starts_with("/search") && name_str.ends_with(".md") {
            FileType::RegularFile
        } else if parent_path.starts_with("/map") {
            let active = self.bridge.active_version().unwrap_or(0);
            if name_str == "directory_overview.md" {
                let relative = parent_path
                    .trim_start_matches("/map")
                    .trim_start_matches('/');
                if self.bridge.map_has_overview(relative, active).ok()? {
                    FileType::RegularFile
                } else {
                    return None;
                }
            } else {
                let relative = full.trim_start_matches("/map").trim_start_matches('/');
                if self.bridge.map_dir_exists(relative, active).ok()? {
                    FileType::Directory
                } else {
                    return None;
                }
            }
        } else if parent_path.starts_with("/raw") {
            let relative = full.trim_start_matches("/raw/");
            let Ok((kind, _)) = self.bridge.raw_node_info(relative) else {
                return None;
            };
            match kind {
                RawNodeKind::Directory => FileType::Directory,
                RawNodeKind::File => FileType::RegularFile,
            }
        } else {
            return None;
        };

        let ino = hash_inode(&full);
        self.inode_to_node.insert(
            ino,
            Node {
                path: full.clone(),
                kind,
            },
        );
        self.path_to_inode.insert(full, ino);
        Some(ino)
    }

    fn session_key(req: &Request<'_>) -> SessionKey {
        SessionKey {
            uid: req.uid(),
            pid: req.pid(),
        }
    }

    fn prune_session_pins_if_needed(&mut self) {
        if self.session_pins.len() < self.max_session_entries {
            return;
        }
        if let Some((oldest_key, _)) = self
            .session_pins
            .iter()
            .min_by_key(|(_, pin)| pin.last_access)
            .map(|(key, pin)| (*key, pin.last_access))
        {
            self.session_pins.remove(&oldest_key);
        }
    }

    fn resolve_snapshot_versions(&mut self, req: &Request<'_>, refresh: bool) -> (u64, u64, bool) {
        let active = self.bridge.active_version().unwrap_or(0);
        if self.session_mode == FuseSessionMode::PerRequest {
            return (active, active, false);
        }

        let key = Self::session_key(req);
        let now = SystemTime::now();

        if refresh {
            self.prune_session_pins_if_needed();
            self.session_pins.insert(
                key,
                SessionPin {
                    snapshot_version: active,
                    pinned_at: now,
                    last_access: now,
                },
            );
            return (active, active, true);
        }

        let pin = self.session_pins.entry(key).or_insert_with(|| SessionPin {
            snapshot_version: active,
            pinned_at: now,
            last_access: now,
        });
        pin.last_access = now;
        (pin.snapshot_version, active, false)
    }

    fn resolve_status_snapshot_versions(&mut self, req: &Request<'_>) -> (u64, u64, bool) {
        let active = self.bridge.active_version().unwrap_or(0);
        if self.session_mode == FuseSessionMode::PerRequest {
            return (active, active, false);
        }

        let key = Self::session_key(req);
        if let Some(pin) = self.session_pins.get(&key) {
            return (pin.snapshot_version, active, false);
        }

        let now = SystemTime::now();
        self.prune_session_pins_if_needed();
        self.session_pins.insert(
            key,
            SessionPin {
                snapshot_version: active,
                pinned_at: now,
                last_access: now,
            },
        );
        (active, active, false)
    }

    fn render_session_status_json(&mut self, req: &Request<'_>, refresh: bool) -> Result<Vec<u8>> {
        let (snapshot, active, refreshed) = if refresh {
            self.resolve_snapshot_versions(req, true)
        } else {
            self.resolve_status_snapshot_versions(req)
        };
        let key = Self::session_key(req);
        let (pinned_at_ms, last_access_ms) = self
            .session_pins
            .get(&key)
            .map(|pin| {
                let pinned = unix_ms(pin.pinned_at);
                let last = unix_ms(pin.last_access);
                (pinned, last)
            })
            .unwrap_or((0, 0));

        Ok(encode_session_status_json(
            self.session_mode,
            key,
            snapshot,
            active,
            refreshed,
            pinned_at_ms,
            last_access_ms,
        ))
    }
}

#[cfg(target_os = "linux")]
fn unix_ms(ts: SystemTime) -> u64 {
    ts.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(target_os = "linux")]
fn session_mode_label(mode: FuseSessionMode) -> &'static str {
    match mode {
        FuseSessionMode::Pinned => "pinned",
        FuseSessionMode::PerRequest => "per_request",
    }
}

#[cfg(target_os = "linux")]
fn encode_session_status_json(
    mode: FuseSessionMode,
    key: SessionKey,
    snapshot: u64,
    active: u64,
    refreshed: bool,
    pinned_at_ms: u64,
    last_access_ms: u64,
) -> Vec<u8> {
    format!(
        "{{\"mode\":\"{}\",\"session\":{{\"uid\":{},\"pid\":{}}},\"snapshot_version\":{},\"active_version\":{},\"stale\":{},\"refreshed\":{},\"pinned_at_unix_ms\":{},\"last_access_unix_ms\":{}}}",
        session_mode_label(mode),
        key.uid,
        key.pid,
        snapshot,
        active,
        if snapshot != active { "true" } else { "false" },
        if refreshed { "true" } else { "false" },
        pinned_at_ms,
        last_access_ms
    )
    .into_bytes()
}

#[cfg(target_os = "linux")]
impl Filesystem for SemanticFsFuse {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if let Some(ino) = self.ensure_node(parent, name) {
            if let Some(attr) = self.attr_for(ino) {
                reply.entry(&TTL, &attr, 0);
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        match self.attr_for(ino) {
            Some(attr) => reply.attr(&TTL, &attr),
            None => reply.error(ENOENT),
        }
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let Some(node) = self.inode_to_node.get(&ino).cloned() else {
            reply.error(ENOENT);
            return;
        };
        let root_ino = *self.path_to_inode.get("/").unwrap_or(&ino);

        let mut entries: Vec<(u64, FileType, OsString)> = vec![
            (ino, FileType::Directory, OsString::from(".")),
            (root_ino, FileType::Directory, OsString::from("..")),
        ];

        match node.path.as_str() {
            "/" => {
                for child in ["raw", "search", "map", ".well-known"] {
                    let path = format!("/{}", child);
                    if let Some(inode) = self.path_to_inode.get(&path) {
                        entries.push((*inode, FileType::Directory, OsString::from(child)));
                    }
                }
            }
            "/.well-known" => {
                if let Some(inode) = self.path_to_inode.get(HEALTH_PATH) {
                    entries.push((*inode, FileType::RegularFile, OsString::from("health.json")));
                }
                if let Some(inode) = self.path_to_inode.get(SESSION_STATUS_PATH) {
                    entries.push((
                        *inode,
                        FileType::RegularFile,
                        OsString::from("session.json"),
                    ));
                }
                if let Some(inode) = self.path_to_inode.get(SESSION_REFRESH_PATH) {
                    entries.push((
                        *inode,
                        FileType::RegularFile,
                        OsString::from("session.refresh"),
                    ));
                }
            }
            p if p.starts_with("/raw") => {
                let rel = p.trim_start_matches("/raw").trim_start_matches('/');
                if let Ok(children) = self.bridge.raw_dir_entries(rel) {
                    for (child_name, child_kind) in children {
                        let name = OsString::from(&child_name);
                        let child_virtual = if p == "/raw" {
                            format!("/raw/{}", child_name)
                        } else {
                            format!("{}/{}", p, child_name)
                        };
                        let fuse_kind = match child_kind {
                            RawNodeKind::Directory => FileType::Directory,
                            RawNodeKind::File => FileType::RegularFile,
                        };

                        let child_ino = *self
                            .path_to_inode
                            .entry(child_virtual.clone())
                            .or_insert_with(|| {
                                let ino = hash_inode(&child_virtual);
                                self.inode_to_node.insert(
                                    ino,
                                    Node {
                                        path: child_virtual,
                                        kind: fuse_kind,
                                    },
                                );
                                ino
                            });

                        entries.push((child_ino, fuse_kind, name));
                    }
                }
            }
            p if p.starts_with("/map") => {
                let active = self.bridge.active_version().unwrap_or(0);
                let rel = p.trim_start_matches("/map").trim_start_matches('/');

                if self.bridge.map_has_overview(rel, active).unwrap_or(false) {
                    let candidate = format!("{}/directory_overview.md", p);
                    let inode = *self
                        .path_to_inode
                        .entry(candidate.clone())
                        .or_insert_with(|| {
                            let ino = hash_inode(&candidate);
                            self.inode_to_node.insert(
                                ino,
                                Node {
                                    path: candidate,
                                    kind: FileType::RegularFile,
                                },
                            );
                            ino
                        });
                    entries.push((
                        inode,
                        FileType::RegularFile,
                        OsString::from("directory_overview.md"),
                    ));
                }

                if let Ok(children) = self.bridge.map_dir_entries(rel, active) {
                    for child_name in children {
                        let name = OsString::from(&child_name);
                        let child_virtual = if p == "/map" {
                            format!("/map/{}", child_name)
                        } else {
                            format!("{}/{}", p, child_name)
                        };
                        let child_ino = *self
                            .path_to_inode
                            .entry(child_virtual.clone())
                            .or_insert_with(|| {
                                let ino = hash_inode(&child_virtual);
                                self.inode_to_node.insert(
                                    ino,
                                    Node {
                                        path: child_virtual,
                                        kind: FileType::Directory,
                                    },
                                );
                                ino
                            });
                        entries.push((child_ino, FileType::Directory, name));
                    }
                }
            }
            _ => {}
        }

        for (i, (entry_ino, kind, name)) in entries.into_iter().enumerate().skip(offset as usize) {
            if reply.add(entry_ino, (i + 1) as i64, kind, name) {
                break;
            }
        }

        reply.ok();
    }

    fn open(&mut self, _req: &Request<'_>, ino: u64, flags: i32, reply: ReplyOpen) {
        if flags & libc::O_ACCMODE != libc::O_RDONLY {
            reply.error(EROFS);
            return;
        }

        if let Some(node) = self.inode_to_node.get(&ino) {
            let open_flags = if node.path.starts_with("/raw/") {
                0
            } else {
                // Virtual files are synthesized per-read and can change between accesses.
                // Use direct I/O to avoid kernel page-cache size/content staleness.
                FOPEN_DIRECT_IO_FLAG
            };
            reply.opened(0, open_flags);
            return;
        }

        reply.error(ENOENT);
    }

    fn read(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        let Some(node) = self.inode_to_node.get(&ino) else {
            reply.error(ENOENT);
            return;
        };

        let node_kind = node.kind;
        let node_path = node.path.clone();

        if node_kind != FileType::RegularFile {
            reply.error(ENOENT);
            return;
        }

        let read_result = if node_path == SESSION_REFRESH_PATH {
            self.render_session_status_json(req, true)
        } else if node_path == SESSION_STATUS_PATH {
            self.render_session_status_json(req, false)
        } else if node_path == HEALTH_PATH {
            let active = self.bridge.active_version().unwrap_or(0);
            self.bridge.read_virtual(&node_path, active, active)
        } else {
            let (snapshot, active, _) = self.resolve_snapshot_versions(req, false);
            self.bridge.read_virtual(&node_path, snapshot, active)
        };

        match read_result {
            Ok(data) => {
                let start = offset.max(0) as usize;
                let end = (start + size as usize).min(data.len());
                if start >= data.len() {
                    reply.data(&[]);
                } else {
                    reply.data(&data[start..end]);
                }
            }
            Err(_) => reply.error(ENOENT),
        }
    }
}

#[cfg(target_os = "linux")]
fn hash_inode(key: &str) -> u64 {
    let mut hash: u64 = 1469598103934665603;
    for b in key.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::{encode_session_status_json, SessionKey};
    use crate::FuseSessionMode;

    fn as_str(bytes: Vec<u8>) -> String {
        String::from_utf8(bytes).expect("session status JSON should be utf8")
    }

    #[test]
    fn session_json_marks_stale_when_active_advances_without_refresh() {
        let body = as_str(encode_session_status_json(
            FuseSessionMode::Pinned,
            SessionKey { uid: 1000, pid: 42 },
            10,
            11,
            false,
            1700000000000,
            1700000000100,
        ));

        assert!(body.contains("\"mode\":\"pinned\""));
        assert!(body.contains("\"snapshot_version\":10"));
        assert!(body.contains("\"active_version\":11"));
        assert!(body.contains("\"stale\":true"));
        assert!(body.contains("\"refreshed\":false"));
    }

    #[test]
    fn session_refresh_marks_refreshed_and_clears_stale() {
        let body = as_str(encode_session_status_json(
            FuseSessionMode::Pinned,
            SessionKey { uid: 1000, pid: 42 },
            11,
            11,
            true,
            1700000000200,
            1700000000200,
        ));

        assert!(body.contains("\"snapshot_version\":11"));
        assert!(body.contains("\"active_version\":11"));
        assert!(body.contains("\"stale\":false"));
        assert!(body.contains("\"refreshed\":true"));
        assert!(body.contains("\"pinned_at_unix_ms\":1700000000200"));
        assert!(body.contains("\"last_access_unix_ms\":1700000000200"));
    }

    #[test]
    fn per_request_mode_label_is_exposed_in_status_json() {
        let body = as_str(encode_session_status_json(
            FuseSessionMode::PerRequest,
            SessionKey { uid: 501, pid: 777 },
            99,
            99,
            false,
            0,
            0,
        ));

        assert!(body.contains("\"mode\":\"per_request\""));
        assert!(body.contains("\"session\":{\"uid\":501,\"pid\":777}"));
        assert!(body.contains("\"stale\":false"));
    }
}
