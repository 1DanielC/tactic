use crate::api::http_client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PostGetOrCreateUploads {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    pub uploads: Vec<TictacUploadState>,
}

impl PostGetOrCreateUploads {
    pub fn new(device_id: String, uploads: Vec<TictacUploadState>) -> Self {
        Self { device_id, uploads }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TictacUploadState {
    pub md5: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub size: i64,
    #[serde(rename = "numParts")]
    pub num_parts: i64,
    #[serde(rename = "contentType")]
    pub content_type: String,
}

impl TictacUploadState {
    pub fn new(
        md5: String,
        file_name: String,
        size: i64,
        num_parts: i64,
        content_type: String,
    ) -> Self {
        Self {
            md5,
            file_name,
            size,
            num_parts,
            content_type,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOrCreateUploadResponse {
    pub uploads: Vec<UploadWithMd5>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadWithMd5 {
    #[serde(rename = "uploadId")]
    pub upload_id: String,
    pub md5: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUploadRequest {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "uploadId")]
    pub upload_id: String,
    #[serde(rename = "deviceFilename")]
    pub device_filename: String,
    pub tags: Option<Vec<String>>,
}

impl CreateUploadRequest {
    pub fn new(device_id: String, upload_id: String, device_filename: String) -> Self {
        Self {
            device_id,
            upload_id,
            device_filename,
            tags: None,
        }
    }
}

pub fn get_upload_state() -> reqwest::Result<String> {
    let device_id = "Insta360 OneX:sn:IXE04B8B3D123".to_string();
    let uploads = vec![TictacUploadState::new(
        "d38f2947-42d9-c417-1fc4-16eb7216c983".to_string(),
        "VID_20250929_201601_10_167.insv".to_string(),
        20973659,
        1,
        "video/insv".to_string(),
    )];

    let rq = PostGetOrCreateUploads::new(device_id, uploads);
    println!("{:?}", serde_json::to_string(&rq));
    let res = futures::executor::block_on(
        http_client()
            .post("http://localhost:8080/api/tictac/upload-state")
            .json(&rq)
            .send(),
    );

    futures::executor::block_on(res?.text())
}
