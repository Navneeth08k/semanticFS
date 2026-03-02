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
                exported: t.starts_with("pub ")
                    || t.starts_with("pub(")
                    || t.starts_with("export "),
                scope: "top_level".to_string(),
            });
        }
    }

    out
}

fn parse_symbol_name(line: &str) -> Option<String> {
    for prefix in [
        "fn ",
        "async fn ",
        "pub fn ",
        "pub async fn ",
        "pub(crate) fn ",
        "pub(crate) async fn ",
        "pub(super) fn ",
        "pub(super) async fn ",
        "pub(self) fn ",
        "pub(self) async fn ",
        "public class ",
        "private class ",
        "protected class ",
        "class ",
        "abstract class ",
        "public interface ",
        "private interface ",
        "protected interface ",
        "interface ",
        "def ",
        "async def ",
        "pub struct ",
        "pub(crate) struct ",
        "pub(super) struct ",
        "pub(self) struct ",
        "struct ",
        "pub enum ",
        "pub(crate) enum ",
        "pub(super) enum ",
        "pub(self) enum ",
        "public enum ",
        "private enum ",
        "protected enum ",
        "enum ",
        "pub trait ",
        "pub(crate) trait ",
        "pub(super) trait ",
        "pub(self) trait ",
        "trait ",
        "const ",
        "let ",
        "export const ",
        "export let ",
        "function ",
        "export async function ",
        "export function ",
        "export default function ",
        "export default async function ",
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
    parse_typed_function_name(line)
}

fn parse_typed_function_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return None;
    }

    let Some(paren_idx) = trimmed.find('(') else {
        return None;
    };

    if let Some(eq_idx) = trimmed.find('=') {
        if eq_idx < paren_idx {
            return None;
        }
    }

    let looks_like_declaration = trimmed.ends_with('{')
        || trimmed.ends_with("=>")
        || trimmed.ends_with("=>;")
        || trimmed.ends_with(';');
    if !looks_like_declaration {
        return None;
    }

    let before_paren = trimmed[..paren_idx].trim_end();
    let tokens = before_paren.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 2 {
        return None;
    }

    let first = tokens[0];
    if matches!(
        first,
        "if" | "for" | "while" | "switch" | "catch" | "return" | "assert" | "throw" | "await"
    ) {
        return None;
    }

    let candidate = tokens[tokens.len() - 1]
        .trim_matches(|c: char| matches!(c, '*' | '&'))
        .trim();
    if candidate.is_empty() {
        return None;
    }
    if !candidate
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '?' | '$'))
    {
        return None;
    }
    Some(candidate.trim_end_matches('?').to_string())
}

fn classify_symbol_kind(line: &str) -> &'static str {
    if line.contains("class ") {
        return "class";
    }
    if line.contains("interface ") {
        return "trait";
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

#[cfg(test)]
mod tests {
    use super::parse_symbol_name;

    #[test]
    fn extracts_python_defs() {
        assert_eq!(
            parse_symbol_name("def _add_metrics(self, metrics):"),
            Some("_add_metrics".to_string())
        );
        assert_eq!(
            parse_symbol_name("async def create_dataset(config):"),
            Some("create_dataset".to_string())
        );
    }

    #[test]
    fn extracts_rust_async_fns() {
        assert_eq!(
            parse_symbol_name("pub async fn run_service() -> Result<()> {"),
            Some("run_service".to_string())
        );
        assert_eq!(
            parse_symbol_name("pub(crate) fn map_dir_entries(&self) -> Vec<String> {"),
            Some("map_dir_entries".to_string())
        );
        assert_eq!(
            parse_symbol_name("pub struct ResolvedPath {"),
            Some("ResolvedPath".to_string())
        );
    }

    #[test]
    fn extracts_function_keyword() {
        assert_eq!(
            parse_symbol_name("function buildModel(input) {"),
            Some("buildModel".to_string())
        );
    }

    #[test]
    fn extracts_export_async_function() {
        assert_eq!(
            parse_symbol_name("export async function DELETE(request: Request) {"),
            Some("DELETE".to_string())
        );
        assert_eq!(
            parse_symbol_name("export const useUser = () => {"),
            Some("useUser".to_string())
        );
        assert_eq!(
            parse_symbol_name("export let currentUser = null;"),
            Some("currentUser".to_string())
        );
    }

    #[test]
    fn extracts_java_class_declarations_with_modifiers() {
        assert_eq!(
            parse_symbol_name("public class ConceptVuforiaDriveToTargetWebcam extends LinearOpMode {"),
            Some("ConceptVuforiaDriveToTargetWebcam".to_string())
        );
        assert_eq!(
            parse_symbol_name("public interface CommandRunner {"),
            Some("CommandRunner".to_string())
        );
    }

    #[test]
    fn extracts_typed_method_declarations() {
        assert_eq!(
            parse_symbol_name("void _write(String text) {"),
            Some("_write".to_string())
        );
        assert_eq!(
            parse_symbol_name("String? _canRun(String path) {"),
            Some("_canRun".to_string())
        );
        assert_eq!(
            parse_symbol_name("void attemptToolExit() {"),
            Some("attemptToolExit".to_string())
        );
    }

    #[test]
    fn skips_plain_invocations() {
        assert_eq!(parse_symbol_name("attemptToolExit();"), None);
    }
}
