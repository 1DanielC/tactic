mod api;
mod camera_fs;
mod device_type;
mod json;
mod openspace;

use crate::openspace::upload_all_files::upload_all_files;
use dioxus::prelude::*;
use dioxus_desktop::tao;

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

#[component]
pub fn Hero() -> Element {
    rsx! {
        div { id: "app",
            div { id: "header", span { "OpenSpace Desktop Sync" } }
            div { id: "content", { build_content() } }
            div { id: "footer",
                div { id: "footer-bar", p { "Le Camera is disconnected" }}
                div { id: "footer-bar", p { "Le App is Updated" }}
            }
        }
    }
}

fn build_content() -> Element {
    rsx! {
        button { class: "button", onclick: move |_| async move {
            if let Err(e) = upload_all_files() { eprintln!("Upload failed: {}", e) }
        }, "Upload Files?" },
    }
}
