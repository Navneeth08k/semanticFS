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
            || lower.ends_with(".ts")
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
