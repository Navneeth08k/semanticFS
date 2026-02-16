use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorySummary {
    pub dir_path: String,
    pub summary_markdown: String,
}
