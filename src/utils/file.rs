use std::fs;
use std::path::Path;

use anyhow::Result;

pub fn create_dir_if_not_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}
