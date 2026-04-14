//! Scratchpad — append-only daily files, cross-session continuity.

use chrono::Utc;
use std::path::{Path, PathBuf};
use std::io::Write;

pub struct Scratchpad {
    dir: PathBuf,
    today: String,
}

impl Scratchpad {
    pub fn new(root_dir: &Path, mind_id: &str) -> Self {
        let dir = root_dir.join("scratchpads").join(mind_id);
        std::fs::create_dir_all(&dir).ok();
        Self {
            dir,
            today: Utc::now().format("%Y-%m-%d").to_string(),
        }
    }

    fn today_file(&self) -> PathBuf {
        self.dir.join(format!("{}.md", self.today))
    }

    pub fn read(&self) -> String {
        let path = self.today_file();
        std::fs::read_to_string(&path).unwrap_or_default()
    }

    pub fn read_recent(&self, bytes: usize) -> String {
        let path = self.today_file();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let start = content.len().saturating_sub(bytes);
        content[start..].to_string()
    }

    pub fn append(&self, text: &str) {
        let path = self.today_file();
        let timestamp = Utc::now().format("%H:%M:%S");
        let entry = format!("\n## [{timestamp}]\n\n{text}\n");
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();
        writeln!(f, "{entry}").ok();
    }

    pub fn write(&self, text: &str) {
        let path = self.today_file();
        std::fs::write(&path, text).ok();
    }
}
