use crate::camera_fs::camera_finder::scan_for_camera_fs;
use crate::api::http_client;
use crate::storage::{add_skipped_file, is_file_skipped, SkippedFile};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use walkdir::WalkDir;
use crate::openspace::model::{TicTacUploadRequest, GetOrCreateUploadResponse};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const API_BASE_URL: &str = "http://localhost:8080/api";
const CHUNK_SIZE: i64 = 8 * 1024 * 1024; // 8MB chunks

#[derive(Debug)]
struct FileToUpload {
    path: PathBuf,
    size: i64,
}

#[derive(Debug)]
enum UploadResult {
    Completed,
    Skipped,
}

#[derive(Debug, Clone)]
pub enum UploadEvent {
    CameraFound(String),
    FileStarted { filename: String, total_bytes: i64 },
    FileProgress { filename: String, bytes_uploaded: i64, total_bytes: i64 },
    FileSkipped { filename: String },
    FileCompleted { filename: String },
    FileFailed { filename: String, error: String },
}

pub fn upload_all_files(progress_tx: Option<Sender<UploadEvent>>) -> Result<(), Box<dyn std::error::Error>> {
    let camera_info = match scan_for_camera_fs() {
        Some(info) => info,
        None => {
            println!("No camera volume found");
            return Ok(()); // exit the function cleanly
        }
    };

    println!("Found camera volume: {:?}", camera_info.mount_point);
    println!("Device ID: {}", camera_info.device_id);

    // Notify UI that camera was found
    if let Some(ref tx) = progress_tx {
        let _ = tx.send(UploadEvent::CameraFound(camera_info.device_id.clone()));
    }

    // Step 1: Find all .insv files (filtered by skipped files cache)
    let insv_files: Vec<PathBuf> = collect_insv_files(camera_info.mount_point, &camera_info.device_id, progress_tx.as_ref())?;
    println!("Found {} .insv files to upload", insv_files.len());

    if insv_files.is_empty() {
        println!("No files to upload");
        return Ok(());
    }

    // Create a Tokio runtime for async operations
    let runtime = tokio::runtime::Runtime::new()?;

    // Step 2: Upload each file
    for file in insv_files {
        let filename = file.file_name().unwrap().to_str().unwrap().to_string();
        let file_size = file.metadata()?.len() as i64;

        let request = TicTacUploadRequest::new(
            camera_info.device_id.clone(),
            filename.clone(),
            "video/insv".to_string(),
            file_size,
            1,
        );

        // Notify UI that file upload is starting
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(UploadEvent::FileStarted {
                filename: filename.clone(),
                total_bytes: file_size,
            });
        }

        match runtime.block_on(upload_file(&file, request, progress_tx.clone(), &camera_info.device_id)) {
            Ok(UploadResult::Completed) => {
                println!("Successfully uploaded: {:?}", filename);
                if let Some(ref tx) = progress_tx {
                    let _ = tx.send(UploadEvent::FileCompleted { filename });
                }
            }
            Ok(UploadResult::Skipped) => {
                println!("File already exists on server, skipping: {:?}", filename);
                // Add to skipped files cache
                let skipped = SkippedFile::new(filename.clone(), file_size, camera_info.device_id.clone());
                if let Err(e) = add_skipped_file(skipped) {
                    eprintln!("Failed to cache skipped file: {}", e);
                }
                if let Some(ref tx) = progress_tx {
                    let _ = tx.send(UploadEvent::FileSkipped { filename });
                }
            }
            Err(e) => {
                eprintln!("Failed to upload {:?}: {}", filename, e);
                if let Some(ref tx) = progress_tx {
                    let _ = tx.send(UploadEvent::FileFailed {
                        filename,
                        error: e.to_string(),
                    });
                }
            }
        }
    }

    println!("Upload process completed");
    Ok(())
}

fn collect_insv_files(
    volume: PathBuf,
    device_id: &str,
    progress_tx: Option<&Sender<UploadEvent>>,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut insv_files = Vec::new();

    for entry in WalkDir::new(volume) {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if ext.eq_ignore_ascii_case("insv") {
                            let path = entry.path().to_path_buf();

                            // Check if this file is in the skipped cache
                            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                                if let Ok(metadata) = entry.metadata() {
                                    let size = metadata.len() as i64;

                                    // Skip if already in cache
                                    if is_file_skipped(filename, size, device_id) {
                                        println!("Skipping cached file: {}", filename);

                                        // Send event for cached skipped file
                                        if let Some(tx) = progress_tx {
                                            let _ = tx.send(UploadEvent::FileSkipped {
                                                filename: filename.to_string(),
                                            });
                                        }
                                        continue;
                                    }
                                }
                            }

                            insv_files.push(path);
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error reading directory entry: {}", e),
        }
    }

    Ok(insv_files)
}

async fn upload_file(
    file: &PathBuf,
    req: TicTacUploadRequest,
    progress_tx: Option<Sender<UploadEvent>>,
    _device_id: &str,
) -> Result<UploadResult, Box<dyn std::error::Error>> {
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
            return Ok(UploadResult::Skipped);
        }
    };

    // Step 3: Upload the file in chunks
    let mut file_handle = File::open(file)?;
    let file_size = req.size;
    let filename = req.device_filename.clone();
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

        // Send progress update
        if let Some(ref tx) = progress_tx {
            let bytes_uploaded = end + 1;
            let _ = tx.send(UploadEvent::FileProgress {
                filename: filename.clone(),
                bytes_uploaded,
                total_bytes: file_size,
            });
        }
    }

    Ok(UploadResult::Completed)
}
