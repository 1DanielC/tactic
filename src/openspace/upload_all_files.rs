use crate::camera_fs::camera_finder::scan_for_camera_fs;
use crate::api::http_client;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::openspace::get_upload_state::{TicTacUploadRequest, GetOrCreateUploadResponse};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

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

        match futures::executor::block_on(upload_file(&file, request)) {
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

async fn upload_file(file: &PathBuf, req: TicTacUploadRequest) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Create the upload on the backend
    let client = http_client();
    let create_url = format!("{}/tictac/uploads", API_BASE_URL);

    let response = client
        .post(&create_url)
        .json(&req)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Failed to create upload: {}", response.status()).into());
    }

    let create_response: GetOrCreateUploadResponse = response.json().await?;

    // Step 2: Check if we need to upload (uploadId will be None if file already exists)
    let upload_id = match create_response.upload_id {
        Some(id) => id,
        None => {
            println!("File already exists on server, skipping upload");
            return Ok(());
        }
    };

    // Step 3: Upload the file in chunks
    let mut file_handle = File::open(file)?;
    let file_size = req.size;
    let num_parts = req.num_parts.max(1); // Ensure at least 1 part
    let chunk_size = if num_parts == 1 {
        file_size
    } else {
        (file_size as f64 / num_parts as f64).ceil() as i64
    };

    for part in 0..num_parts {
        let start = part as i64 * chunk_size;
        let end = ((part + 1) as i64 * chunk_size).min(file_size) - 1;
        let chunk_len = (end - start + 1) as usize;

        // Read chunk from file
        file_handle.seek(SeekFrom::Start(start as u64))?;
        let mut buffer = vec![0u8; chunk_len];
        file_handle.read_exact(&mut buffer)?;

        // Upload chunk with Content-Range header
        let upload_url = format!("{}/tictac/uploads/{}", API_BASE_URL, upload_id);
        let content_range = format!("bytes {}-{}/{}", start, end, file_size);

        let response = client
            .put(&upload_url)
            .header("Content-Range", content_range)
            .header("Content-Type", "application/octet-stream")
            .body(buffer)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to upload chunk {}: {}", part, response.status()).into());
        }

        println!("Uploaded chunk {}/{} (bytes {}-{})", part + 1, num_parts, start, end);
    }

    Ok(())
}
