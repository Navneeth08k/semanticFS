use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub symbol_name: String,
    pub symbol_kind: String,
    pub path: String,
    pub line_start: u32,
    pub line_end: u32,
    pub language: String,
    pub exported: bool,
    pub scope: String,
}

pub fn extract_symbols(
    content: &str,
    file_type: &crate::filetype::FileType,
    path: &str,
) -> Vec<SymbolRecord> {
    let mut out = Vec::new();
    let is_code = matches!(file_type, crate::filetype::FileType::Code(_));
    if !is_code {
        return out;
    }

    for (idx, line) in content.lines().enumerate() {
        let t = line.trim();
        if let Some(name) = parse_symbol_name(t) {
            out.push(SymbolRecord {
                symbol_name: name,
                symbol_kind: classify_symbol_kind(t).to_string(),
                path: path.to_string(),
                line_start: (idx + 1) as u32,
                line_end: (idx + 1) as u32,
                language: "code".to_string(),
                exported: t.starts_with("pub ") || t.starts_with("export "),
                scope: "top_level".to_string(),
            });
        }
    }

    out
}

fn parse_symbol_name(line: &str) -> Option<String> {
    for prefix in [
        "fn ",
        "pub fn ",
        "class ",
        "struct ",
        "enum ",
        "trait ",
        "const ",
        "let ",
        "export function ",
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name = rest
                .split(['(', '{', ':', ' ', '='])
                .next()
                .unwrap_or("")
                .trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

fn classify_symbol_kind(line: &str) -> &'static str {
    if line.contains("class ") {
        return "class";
    }
    if line.contains("struct ") {
        return "struct";
    }
    if line.contains("enum ") {
        return "enum";
    }
    if line.contains("trait ") {
        return "trait";
    }
    if line.contains("const ") {
        return "const";
    }
    "function"
}
