mod api;
mod camera_finder;
mod DeviceType;

use dioxus::prelude::*;
use dioxus_desktop::tao;
use reqwest::{Client, Response};
use crate::api::http_client;
use crate::camera_finder::{scan_for_camera, scan_for_camera_2};
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
        .with_cfg(dioxus_desktop::Config::new()
            .with_window(window)
            .with_custom_head(format!(r#"<style>{}</style>"#, MAIN_CSS))
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

#[component]
pub fn Hero() -> Element {
    let http_client = http_client();
    rsx! {
        div { id: "app",
            div { id: "header", span { "OpenSpace Desktop Sync" } }
            div { id: "content",
                { build_content(http_client) }
            }
            div { id: "footer",
                div { id: "footer-bar", p { "Le Camera is disconnected" }}
                div { id: "footer-bar", p { "Le App is Updated" }}
            }
        }
    }
}

fn build_content(http_client: Client) -> Element {
    do_http_stuff(http_client);
    rsx! {
        button { class: "button", onclick: move |_| async move { upload_file() }, "Upload" },
        button { class: "button", onclick: move |_| async move { scan_for_camera_2() }, "Camera?" },
    }
}

async fn do_http_stuff(http_client: Client) {
    let response_body = http_client.get("https://example/com").send().await.unwrap();
    let body = response_body.text().await.unwrap();
    println!("{}", body);
}

fn upload_file() {
    if let Some(path) = rfd::FileDialog::new().pick_file() {
        println!("Selected file: {:?}", path);
    }
}