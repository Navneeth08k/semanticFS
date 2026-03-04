#[derive(Debug, Clone)]
pub enum FileType {
    Code(String),
    Markdown,
    Text,
    Json,
    Yaml,
    Toml,
    Pdf,
    Binary,
}

impl FileType {
    pub fn from_path(path: &str) -> Self {
        let lower = path.to_lowercase();
        if lower.ends_with(".rs")
            || lower.ends_with(".py")
            || lower.ends_with(".go")
            || lower.ends_with(".js")
            || lower.ends_with(".jsx")
            || lower.ends_with(".ts")
            || lower.ends_with(".tsx")
            || lower.ends_with(".java")
            || lower.ends_with(".c")
            || lower.ends_with(".cpp")
            || lower.ends_with(".h")
            || lower.ends_with(".hpp")
            || lower.ends_with(".cs")
            || lower.ends_with(".dart")
        {
            return FileType::Code("code".to_string());
        }
        if lower.ends_with(".md") {
            return FileType::Markdown;
        }
        if lower.ends_with(".txt") {
            return FileType::Text;
        }
        if lower.ends_with(".json") {
            return FileType::Json;
        }
        if lower.ends_with(".yaml") || lower.ends_with(".yml") {
            return FileType::Yaml;
        }
        if lower.ends_with(".toml") {
            return FileType::Toml;
        }
        if lower.ends_with(".pdf") {
            return FileType::Pdf;
        }
        if lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".gif")
            || lower.ends_with(".mp4")
        {
            return FileType::Binary;
        }
        FileType::Text
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::Code(_) => "code",
            FileType::Markdown => "markdown",
            FileType::Text => "text",
            FileType::Json => "json",
            FileType::Yaml => "yaml",
            FileType::Toml => "toml",
            FileType::Pdf => "pdf",
            FileType::Binary => "binary",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FileType;

    #[test]
    fn classifies_extended_code_extensions() {
        assert!(matches!(
            FileType::from_path("components/app-header.tsx"),
            FileType::Code(_)
        ));
        assert!(matches!(
            FileType::from_path("src/Main.java"),
            FileType::Code(_)
        ));
        assert!(matches!(
            FileType::from_path("lib/src/android/android_console.dart"),
            FileType::Code(_)
        ));
    }

    #[test]
    fn leaves_sql_as_non_code_for_now() {
        assert!(matches!(
            FileType::from_path("supabase/quick_setup.sql"),
            FileType::Text
        ));
    }
}
