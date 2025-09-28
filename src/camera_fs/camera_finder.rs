use crate::camera_fs::sys_profiler_usb::{UsbNode, UsbRoot};
use crate::device_type::{DeviceType, VendorType};
use nusb::{list_devices, DeviceInfo, MaybeFuture};
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

pub fn scan_for_camera_fs() {
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

fn scan_for_camera_fs_linux() {
    let camera_path = PathBuf::from("/dev/video0");
    println!("Camera path: {:?}", camera_path);
}

fn scan_for_camera_fs_windows() {
    let camera_path = PathBuf::from("\\\\.\\Global\\");
}

fn scan_for_camera_fs_macos() {
    let out = Command::new("system_profiler")
        .arg("SPUSBDataType")
        .arg("-json")
        .output()
        .expect("Failed to run system_profiler");

    let json_output: String = String::from_utf8_lossy(&out.stdout).to_string();
    let usb_root: UsbRoot = serde_json::from_str(&json_output).unwrap();
    let camera_usb_nodes = usb_root
        .spusb_data_type
        .iter()
        .flat_map(|usb_bus| usb_bus.items.iter())
        .find(|usb_node| {
            VendorType::from_vendor_id(usb_node.vendor_id.unwrap_or_default()).is_some()
                && DeviceType::from_product_id(usb_node.product_id.unwrap_or_default()).is_some()
        })
        ;

    // just grabbing the first camera we find
    if(camera_usb_nodes.iter().size_hint().0 == 0) {
        println!("No camera found");
        return;
    }

    let camera_node = camera_usb_nodes.into_iter().next().unwrap();
    let media_volumes = camera_node
        .media
        .iter()
        .flat_map(|m| m.iter())
        .flat_map(|m| m.volumes.iter())
        .flat_map(|v| v.iter())
        ;

    for volume in media_volumes {
        let mountpoint = volume.mount_point.as_deref().unwrap_or_default();
        println!("Mountpoint: {:?}", mountpoint);
    }
}
