use crate::camera_fs::sys_profiler_usb::{UsbNode, UsbRoot};
use crate::device_type::{DeviceType, VendorType};
use nusb::{list_devices, DeviceInfo, MaybeFuture};
use std::ops::Deref;
use std::path::PathBuf;
use std::process::Command;

fn scan_for_camera_or_none() -> Option<DeviceInfo> {
    list_devices()
        .wait()
        .expect("Failed to scan devices for camera")
        .find(|dev| {
            VendorType::from_vendor_id(dev.vendor_id()).is_some()
                && DeviceType::from_product_id(dev.product_id()).is_some()
        })
}

pub fn scan_for_camera_fs() -> Option<PathBuf> {
    let os = std::env::consts::OS;

    match os {
        "linux" => {
            println!("Linux");
            scan_for_camera_fs_linux()
        }
        "windows" => {
            println!("Windows");
            scan_for_camera_fs_windows()
        }
        "macos" => {
            println!("MacOS");
            scan_for_camera_fs_macos()
        }
        _ => panic!("Unsupported OS"),
    }
}

fn scan_for_camera_fs_linux() -> Option<PathBuf>{
    eprintln!("Linux not supported yet");
    None
}

fn scan_for_camera_fs_windows() -> Option<PathBuf> {
    eprintln!("Windows not supported yet");
    None
}

fn scan_for_camera_fs_macos() -> Option<PathBuf> {
    let out = Command::new("system_profiler")
        .arg("SPUSBDataType")
        .arg("-json")
        .output()
        .expect("Failed to run system_profiler");

    let json_output: String = String::from_utf8_lossy(&out.stdout).to_string();
    let usb_root: UsbRoot = serde_json::from_str(&json_output).unwrap();

    // Get First Camera Node
    let Some(camera_node) = usb_root
        .spusb_data_type
        .iter()
        .flat_map(|bus| bus.items.iter())
        .find(|n| {
            VendorType::from_vendor_id(n.vendor_id.unwrap_or_default()).is_some()
                && DeviceType::from_product_id(n.product_id.unwrap_or_default()).is_some()
        })
    else {
        println!("No camera found");
        return None;
    };

    println!("Found Camera: {}", camera_node.name);

    // Get First Volume with a mount point
    if let Some(mount_point) = camera_node
        .media
        .iter()
        .flat_map(|m| m.iter())
        .flat_map(|m| m.volumes.iter())
        .flat_map(|v| v.iter())
        .find(|v| v.mount_point.is_some())
        .map(|v| v.mount_point.as_deref())
    {
        if let Some(m) = mount_point {
            println!("Found Volume: {}", m);

            Some(PathBuf::from(m))
        } else {
            None
        }
    } else {
        None
    }
}
