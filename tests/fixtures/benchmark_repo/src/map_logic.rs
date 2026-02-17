pub fn directory_purpose(dir: &str) -> &'static str {
    match dir {
        "src" => "Core runtime and service entrypoints",
        "docs" => "Design notes and operator guidance",
        _ => "General project content",
    }
}
