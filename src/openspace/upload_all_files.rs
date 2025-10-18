use crate::camera_fs::camera_finder::scan_for_camera_fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::openspace::get_upload_state::TicTacUploadRequest;

const API_BASE_URL: &str = "http://localhost:8080/api";
const CHUNK_SIZE: i64 = 8 * 1024 * 1024; // 8MB chunks

#[derive(Debug)]
struct FileToUpload {
    path: PathBuf,
    size: i64,
}

pub fn upload_all_files() -> Result<(), Box<dyn std::error::Error>> {
    let volume = match scan_for_camera_fs() {
        Some(v) => v,
        None => {
            println!("No camera volume found");
            return Ok(()); // exit the function cleanly
        }
    };

    println!("Found camera volume: {:?}", volume);

    // Step 1: Find all .insv files
    let insv_files: Vec<PathBuf> = collect_insv_files(volume)?;
    println!("Found {} .insv files", insv_files.len());

    if insv_files.is_empty() {
        println!("No files to upload");
        return Ok(());
    }

    // Step 4: Upload each file
    for file in insv_files {
        let request = TicTacUploadRequest::new(
            "Insta360 OneX2:sn:IXSE09DN9FBSPU".to_string(),
            file.file_name().unwrap().to_str().unwrap().to_string(),
            "video/insv".to_string(),
            file.metadata()?.len() as i64,
            1,
        );

        println!("Uploading file: {:?}", request.device_filename);
        match upload_file(&file, request) {
            Ok(_) => println!("Successfully uploaded: {:?}", &file.file_name().unwrap()),
            Err(e) => eprintln!("Failed to upload {:?}: {}", file, e),
        }
    }

    println!("Upload process completed");
    Ok(())
}

fn collect_insv_files(volume: PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut insv_files = Vec::new();

    for entry in WalkDir::new(volume) {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if ext.eq_ignore_ascii_case("insv") {
                            insv_files.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error reading directory entry: {}", e),
        }
    }

    Ok(insv_files)
}

fn upload_file(file: &PathBuf, req: TicTacUploadRequest) -> Result<(), Box<dyn std::error::Error>> {
    println!("I'm uploading!");
    Ok(())
}
