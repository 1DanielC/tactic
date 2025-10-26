use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

const STORAGE_DIR: &str = ".openspace_sync";
const SKIPPED_FILES_FILE: &str = "skipped_files.json";

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct SkippedFile {
    pub filename: String,
    pub size: i64,
    pub device_id: String,
}

impl SkippedFile {
    pub fn new(filename: String, size: i64, device_id: String) -> Self {
        Self {
            filename,
            size,
            device_id,
        }
    }
}

fn get_storage_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let storage_dir = home.join(STORAGE_DIR);

    if !storage_dir.exists() {
        fs::create_dir_all(&storage_dir)?;
    }

    Ok(storage_dir.join(SKIPPED_FILES_FILE))
}

pub fn load_skipped_files() -> Result<HashSet<SkippedFile>, Box<dyn std::error::Error>> {
    let storage_path = get_storage_path()?;

    if !storage_path.exists() {
        return Ok(HashSet::new());
    }

    let content = fs::read_to_string(storage_path)?;
    let skipped: HashSet<SkippedFile> = serde_json::from_str(&content)?;

    Ok(skipped)
}

pub fn save_skipped_files(skipped: &HashSet<SkippedFile>) -> Result<(), Box<dyn std::error::Error>> {
    let storage_path = get_storage_path()?;
    let content = serde_json::to_string_pretty(skipped)?;

    fs::write(storage_path, content)?;

    Ok(())
}

pub fn add_skipped_file(skipped_file: SkippedFile) -> Result<(), Box<dyn std::error::Error>> {
    let mut skipped = load_skipped_files()?;
    skipped.insert(skipped_file);
    save_skipped_files(&skipped)?;

    Ok(())
}

pub fn clear_skipped_files() -> Result<(), Box<dyn std::error::Error>> {
    let storage_path = get_storage_path()?;

    if storage_path.exists() {
        fs::remove_file(storage_path)?;
    }

    Ok(())
}

pub fn is_file_skipped(filename: &str, size: i64, device_id: &str) -> bool {
    match load_skipped_files() {
        Ok(skipped) => {
            let file = SkippedFile::new(filename.to_string(), size, device_id.to_string());
            skipped.contains(&file)
        }
        Err(_) => false,
    }
}
