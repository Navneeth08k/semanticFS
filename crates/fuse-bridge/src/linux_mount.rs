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
use crate::FuseBridge;

#[cfg(target_os = "linux")]
const TTL: Duration = Duration::from_millis(300);

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct Node {
    path: String,
    kind: FileType,
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
}

#[cfg(target_os = "linux")]
impl SemanticFsFuse {
    fn new(bridge: Arc<FuseBridge>) -> Self {
        let mut inode_to_node = HashMap::new();
        let mut path_to_inode = HashMap::new();

        for (path, kind) in [
            ("/".to_string(), FileType::Directory),
            ("/raw".to_string(), FileType::Directory),
            ("/search".to_string(), FileType::Directory),
            ("/map".to_string(), FileType::Directory),
            ("/.well-known".to_string(), FileType::Directory),
            (
                "/.well-known/health.json".to_string(),
                FileType::RegularFile,
            ),
        ] {
            let ino = hash_inode(&path);
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
            self.bridge
                .read_virtual_current(&node.path)
                .map(|v| v.len() as u64)
                .unwrap_or(0)
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
            if name_str == "directory_overview.md" {
                FileType::RegularFile
            } else {
                FileType::Directory
            }
        } else if parent_path.starts_with("/raw") {
            let relative = full.trim_start_matches("/raw/");
            let mut disk = PathBuf::from(self.bridge.repo_root());
            disk.push(relative);
            if let Ok(meta) = fs::metadata(disk) {
                if meta.is_dir() {
                    FileType::Directory
                } else {
                    FileType::RegularFile
                }
            } else {
                return None;
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

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
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
                if let Some(inode) = self.path_to_inode.get("/.well-known/health.json") {
                    entries.push((*inode, FileType::RegularFile, OsString::from("health.json")));
                }
            }
            p if p.starts_with("/raw") => {
                let rel = p.trim_start_matches("/raw").trim_start_matches('/');
                let mut disk = PathBuf::from(self.bridge.repo_root());
                if !rel.is_empty() {
                    disk.push(rel);
                }
                if let Ok(iter) = fs::read_dir(disk) {
                    for child in iter.flatten() {
                        let name = child.file_name();
                        let name_str = name.to_string_lossy();
                        let child_virtual = if p == "/raw" {
                            format!("/raw/{}", name_str)
                        } else {
                            format!("{}/{}", p, name_str)
                        };

                        let child_kind = child
                            .metadata()
                            .map(|m| {
                                if m.is_dir() {
                                    FileType::Directory
                                } else {
                                    FileType::RegularFile
                                }
                            })
                            .unwrap_or(FileType::RegularFile);

                        let child_ino = *self
                            .path_to_inode
                            .entry(child_virtual.clone())
                            .or_insert_with(|| {
                                let ino = hash_inode(&child_virtual);
                                self.inode_to_node.insert(
                                    ino,
                                    Node {
                                        path: child_virtual,
                                        kind: child_kind,
                                    },
                                );
                                ino
                            });

                        entries.push((child_ino, child_kind, name));
                    }
                }
            }
            p if p.starts_with("/map") => {
                if p != "/map" {
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

        if self.inode_to_node.contains_key(&ino) {
            reply.opened(0, 0);
            return;
        }

        reply.error(ENOENT);
    }

    fn read(
        &mut self,
        _req: &Request<'_>,
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

        if node.kind != FileType::RegularFile {
            reply.error(ENOENT);
            return;
        }

        match self.bridge.read_virtual_current(&node.path) {
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
