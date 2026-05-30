use crate::error::Result;
use std::path::Path;

pub fn write_export_file(path: &Path, content: &str) -> Result<u64> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(content.len() as u64)
}
