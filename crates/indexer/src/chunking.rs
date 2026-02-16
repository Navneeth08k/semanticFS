use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub chunk_id: String,
    pub start_line: u32,
    pub end_line: u32,
    pub content: String,
    pub language: String,
    pub symbol: Option<String>,
}

pub fn chunk_content(
    content: &str,
    file_type: &crate::filetype::FileType,
    max_lines: usize,
) -> Vec<ChunkRecord> {
    match file_type {
        crate::filetype::FileType::Markdown | crate::filetype::FileType::Text => {
            chunk_text(content)
        }
        crate::filetype::FileType::Json
        | crate::filetype::FileType::Yaml
        | crate::filetype::FileType::Toml => chunk_config(content),
        crate::filetype::FileType::Code(_) => chunk_lines(content, max_lines.max(20)),
        _ => Vec::new(),
    }
}

fn chunk_lines(content: &str, max_lines: usize) -> Vec<ChunkRecord> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    let mut start = 0usize;

    while start < lines.len() {
        let end = (start + max_lines).min(lines.len());
        let body = lines[start..end].join("\n");
        out.push(ChunkRecord {
            chunk_id: format!("chunk:{}:{}", start + 1, end),
            start_line: (start + 1) as u32,
            end_line: end as u32,
            content: body,
            language: "code".to_string(),
            symbol: None,
        });
        start = end;
    }

    out
}

fn chunk_text(content: &str) -> Vec<ChunkRecord> {
    let mut out = Vec::new();
    for (idx, para) in content.split("\n\n").enumerate() {
        let trimmed = para.trim();
        if trimmed.is_empty() {
            continue;
        }
        out.push(ChunkRecord {
            chunk_id: format!("txt:{}", idx + 1),
            start_line: 0,
            end_line: 0,
            content: trimmed.to_string(),
            language: "text".to_string(),
            symbol: None,
        });
    }
    out
}

fn chunk_config(content: &str) -> Vec<ChunkRecord> {
    content
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            Some(ChunkRecord {
                chunk_id: format!("cfg:{}", idx + 1),
                start_line: (idx + 1) as u32,
                end_line: (idx + 1) as u32,
                content: trimmed.to_string(),
                language: "config".to_string(),
                symbol: parse_config_key(trimmed),
            })
        })
        .collect()
}

fn parse_config_key(line: &str) -> Option<String> {
    if let Some((key, _)) = line.split_once('=') {
        return Some(key.trim().to_string());
    }
    if let Some((key, _)) = line.split_once(':') {
        return Some(key.trim().to_string());
    }
    None
}
