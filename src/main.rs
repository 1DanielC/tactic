mod api;
mod camera_fs;
mod device_type;
mod json;
mod openspace;
mod storage;

use crate::openspace::upload_all_files::{upload_all_files, UploadEvent};
use crate::storage::clear_skipped_files;
use dioxus::prelude::*;
use dioxus_desktop::tao;
use std::collections::HashMap;
use std::sync::mpsc;

const MAIN_CSS: &str = include_str!("../assets/main.css");

fn main() {
    // Build a window configuration
    let window = tao::window::WindowBuilder::new()
        .with_inner_size(tao::dpi::LogicalSize::new(400.0, 600.0))
        .with_title("OpenSpace Desktop Sync")
        // optionally set min/max, resizable, etc.
        .with_min_inner_size(tao::dpi::LogicalSize::new(250.0, 250.0))
        .with_max_inner_size(tao::dpi::LogicalSize::new(400.0, 600.0))
        .with_resizable(true); // make it fixed size if you want

    LaunchBuilder::new()
        .with_cfg(
            dioxus_desktop::Config::new()
                .with_window(window)
                .with_custom_head(format!(r#"<style>{}</style>"#, MAIN_CSS)),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Hero {}
    }
}

#[derive(Clone, Debug)]
pub struct UploadStatus {
    pub filename: String,
    pub bytes_uploaded: i64,
    pub total_bytes: i64,
    pub percentage: f64,
    pub status: String, // "uploading", "completed", "skipped", "failed"
}

#[component]
pub fn Hero() -> Element {
    let device_id = use_signal(|| String::from("No camera connected"));
    let uploads = use_signal(|| HashMap::<String, UploadStatus>::new());
    let skipped_count = use_signal(|| 0usize);
    let is_uploading = use_signal(|| false);

    rsx! {
        div { id: "app",
            div { id: "header", span { "OpenSpace Desktop Sync" } }
            div { id: "content",
                { build_content(device_id, uploads, skipped_count, is_uploading) }
            }
            div { id: "footer",
                div { id: "footer-bar", p { "{device_id}" }}
                div { id: "footer-bar", p { "Le App is Updated" }}
            }
        }
    }
}

fn render_upload_item(filename: &str, upload: &UploadStatus) -> Element {
    let status_class = match upload.status.as_str() {
        "completed" => "status-completed",
        "skipped" => "status-skipped",
        _ if upload.status.starts_with("failed") => "status-failed",
        _ => "status-uploading",
    };

    rsx! {
        div {
            key: "{filename}",
            class: "upload-item",
            p { class: "upload-filename", "{upload.filename}" }
            p {
                class: "upload-status {status_class}",
                "Status: {upload.status}"
            }
            if upload.status == "uploading" {
                p { class: "upload-progress-text",
                    "{upload.bytes_uploaded} / {upload.total_bytes} bytes ({upload.percentage:.1}%)"
                }
                div { class: "progress-bar-container",
                    div {
                        class: "progress-bar-fill",
                        style: "width: {upload.percentage}%;",
                    }
                }
            }
        }
    }
}

fn handle_upload_event(
    event: UploadEvent,
    mut device_id: Signal<String>,
    mut uploads: Signal<HashMap<String, UploadStatus>>,
    mut skipped_count: Signal<usize>,
) {
    match event {
        UploadEvent::CameraFound(dev_id) => {
            device_id.set(dev_id);
        }
        UploadEvent::FileStarted { filename, total_bytes } => {
            let mut current_uploads = uploads();
            current_uploads.insert(filename.clone(), UploadStatus {
                filename: filename.clone(),
                bytes_uploaded: 0,
                total_bytes,
                percentage: 0.0,
                status: "uploading".to_string(),
            });
            uploads.set(current_uploads);
        }
        UploadEvent::FileProgress { filename, bytes_uploaded, total_bytes } => {
            let mut current_uploads = uploads();
            if let Some(upload) = current_uploads.get_mut(&filename) {
                upload.bytes_uploaded = bytes_uploaded;
                upload.percentage = (bytes_uploaded as f64 / total_bytes as f64) * 100.0;
            }
            uploads.set(current_uploads);
        }
        UploadEvent::FileSkipped { filename } => {
            let mut current_uploads = uploads();
            if let Some(upload) = current_uploads.get_mut(&filename) {
                upload.status = "skipped".to_string();
            }
            uploads.set(current_uploads);
            skipped_count.set(skipped_count() + 1);
        }
        UploadEvent::FileCompleted { filename } => {
            let mut current_uploads = uploads();
            if let Some(upload) = current_uploads.get_mut(&filename) {
                upload.status = "completed".to_string();
                upload.percentage = 100.0;
            }
            uploads.set(current_uploads);
        }
        UploadEvent::FileFailed { filename, error } => {
            let mut current_uploads = uploads();
            if let Some(upload) = current_uploads.get_mut(&filename) {
                upload.status = format!("failed: {}", error);
            }
            uploads.set(current_uploads);
        }
    }
}

async fn start_upload_process(
    mut device_id: Signal<String>,
    mut uploads: Signal<HashMap<String, UploadStatus>>,
    mut skipped_count: Signal<usize>,
    mut is_uploading: Signal<bool>,
) {
    is_uploading.set(true);
    uploads.set(HashMap::new());
    skipped_count.set(0);

    // Create channel for progress updates
    let (tx, rx) = mpsc::channel();

    // Spawn upload in background OS thread
    std::thread::spawn(move || {
        if let Err(e) = upload_all_files(Some(tx)) {
            eprintln!("Upload failed: {}", e);
        }
        // tx is dropped here when the thread exits, disconnecting the channel
    });

    // Process events from upload thread in async context
    loop {
        match rx.try_recv() {
            Ok(event) => {
                handle_upload_event(event, device_id, uploads, skipped_count);
            }
            Err(mpsc::TryRecvError::Empty) => {
                // No more events yet, wait a bit
                // Use tokio sleep since we're in an async context
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // Channel closed, upload finished
                break;
            }
        }
    }

    is_uploading.set(false);
}

fn build_content(
    mut device_id: Signal<String>,
    mut uploads: Signal<HashMap<String, UploadStatus>>,
    mut skipped_count: Signal<usize>,
    mut is_uploading: Signal<bool>,
) -> Element {
    rsx! {
        div { class: "content-container",
            // Upload list
            if !uploads().is_empty() {
                div { class: "upload-list-container",
                    for (filename, upload) in uploads().iter() {
                        { render_upload_item(filename, upload) }
                    }
                }
            }

            // Skipped files count
            if skipped_count() > 0 {
                p { class: "skipped-count", "Total skipped files: {skipped_count()}" }
            }

            // Upload button
            button {
                class: "button",
                disabled: is_uploading(),
                onclick: move |_| {
                    spawn(async move {
                        start_upload_process(
                            device_id,
                            uploads,
                            skipped_count,
                            is_uploading
                        ).await;
                    });
                },
                if is_uploading() { "Uploading..." } else { "Upload Files" }
            }

            // Clear cache button
            button {
                class: "button button-danger",
                onclick: move |_| {
                    if let Err(e) = clear_skipped_files() {
                        eprintln!("Failed to clear cache: {}", e);
                    } else {
                        println!("Cache cleared successfully");
                    }
                },
                "Clear Cache"
            }
        }
    }
}
