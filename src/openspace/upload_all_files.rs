use crate::api::http_client;
use crate::camera_fs::camera_finder::scan_for_camera_fs;
use crate::openspace::get_upload_state::{
    CreateUploadRequest, GetOrCreateUploadResponse, PostGetOrCreateUploads, TictacUploadState,
};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use walkdir::WalkDir;

const API_BASE_URL: &str = "http://localhost:8080/api";
const DEVICE_ID: &str = "Insta360 OneX:sn:IXE04B8B3D123";
const CHUNK_SIZE: usize = 5 * 1024 * 1024; // 5MB chunks

#[derive(Debug)]
struct FileToUpload {
    path: PathBuf,
    md5: String,
    size: i64,
}

pub fn upload_all_files() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref volume) = scan_for_camera_fs() {
        println!("Found camera volume: {:?}", volume);

        // Step 1: Find all .insv files
        let insv_files = collect_insv_files(volume)?;
        println!("Found {} .insv files", insv_files.len());

        if insv_files.is_empty() {
            println!("No files to upload");
            return Ok(());
        }

        // Step 2: Calculate MD5 checksums
        let files_with_md5 = calculate_md5_checksums(insv_files)?;
        println!("Calculated checksums for {} files", files_with_md5.len());

        // Step 3: Check upload state with backend
        let files_to_upload = check_upload_state(&files_with_md5)?;
        println!("Need to upload {} files", files_to_upload.len());

        // Step 4: Upload each file
        for (file, upload_id) in files_to_upload {
            println!("Uploading file: {:?}", file.path);
            match upload_file(&file, &upload_id) {
                Ok(_) => println!("Successfully uploaded: {:?}", file.path),
                Err(e) => eprintln!("Failed to upload {:?}: {}", file.path, e),
            }
        }

        println!("Upload process completed");
    } else {
        println!("No camera volume found");
    }

    Ok(())
}

fn collect_insv_files(volume: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
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

fn calculate_md5_checksums(
    files: Vec<PathBuf>,
) -> Result<Vec<FileToUpload>, Box<dyn std::error::Error>> {
    let mut files_with_md5 = Vec::new();

    for path in files {
        match calculate_file_md5(&path) {
            Ok((md5, size)) => {
                files_with_md5.push(FileToUpload { path, md5, size });
            }
            Err(e) => eprintln!("Failed to calculate MD5 for {:?}: {}", path, e),
        }
    }

    Ok(files_with_md5)
}

fn calculate_file_md5(path: &PathBuf) -> Result<(String, i64), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut context = md5::Context::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        context.consume(&buffer[..bytes_read]);
    }

    let digest = context.compute();
    let md5_hex = format!("{:x}", digest);

    // Get file size
    let size = file.metadata()?.len() as i64;

    Ok((md5_hex, size))
}

fn check_upload_state(
    files: &[FileToUpload],
) -> Result<Vec<(FileToUpload, String)>, Box<dyn std::error::Error>> {
    // Build request
    let upload_states: Vec<TictacUploadState> = files
        .iter()
        .map(|f| {
            let file_name = f
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.insv")
                .to_string();

            // Calculate number of parts (5MB chunks)
            let num_parts = ((f.size as f64) / (CHUNK_SIZE as f64)).ceil() as i64;

            TictacUploadState::new(
                f.md5.clone(),
                file_name,
                f.size,
                num_parts,
                "video/insv".to_string(),
            )
        })
        .collect();

    let request = PostGetOrCreateUploads::new(DEVICE_ID.to_string(), upload_states);

    // Make API call
    let response = futures::executor::block_on(
        http_client()
            .post(format!("{}/tictac/upload-state", API_BASE_URL))
            .json(&request)
            .send(),
    )?;

    let upload_response: GetOrCreateUploadResponse =
        futures::executor::block_on(response.json())?;

    // Match files with upload IDs
    let mut files_to_upload = Vec::new();
    for upload_info in upload_response.uploads {
        if let Some(file) = files.iter().find(|f| f.md5 == upload_info.md5) {
            files_to_upload.push((
                FileToUpload {
                    path: file.path.clone(),
                    md5: file.md5.clone(),
                    size: file.size,
                },
                upload_info.upload_id,
            ));
        }
    }

    Ok(files_to_upload)
}

fn upload_file(file: &FileToUpload, upload_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Create upload
    let file_name = file
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.insv")
        .to_string();

    let create_request = CreateUploadRequest::new(
        DEVICE_ID.to_string(),
        upload_id.to_string(),
        file_name.clone(),
    );

    println!("Creating upload for {}", file_name);
    futures::executor::block_on(
        http_client()
            .post(format!("{}/tictac/uploads/{}", API_BASE_URL, upload_id))
            .json(&create_request)
            .send(),
    )?;

    // Step 2: Upload file content
    println!("Uploading file content for {}", file_name);
    let file_content = std::fs::read(&file.path)?;

    futures::executor::block_on(
        http_client()
            .put(format!("{}/tictac/uploads/{}", API_BASE_URL, upload_id))
            .header("Content-Type", "application/octet-stream")
            .header(
                "Content-Range",
                format!("bytes 0-{}/{}", file_content.len() - 1, file_content.len()),
            )
            .body(file_content)
            .send(),
    )?;

    Ok(())
}
