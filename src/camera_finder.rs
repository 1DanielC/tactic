use std::path::PathBuf;
use crate::DeviceType::{DeviceType, VendorType};
use nusb::{list_devices, MaybeFuture};
use rusb::{Device, UsbContext};

pub fn scan_for_camera() {
    let camera = list_devices()
        .wait()
        .expect("Failed to scan devices for camera")
        .find(|dev| {
            VendorType::from_vendor_id(dev.vendor_id()).is_some()
                && DeviceType::from_product_id(dev.product_id()).is_some()
        });

    if let Some(camera) = camera {
        println!("Found camera: {:?}", camera);
        let device_info = camera.open().wait().unwrap();
        let interface = device_info.claim_interface(0).wait();
        println!("{:?}", interface);
    }
}

// TODO Rackon with aligning the mountpoints to the camera
pub fn scan_for_camera_2() {
    let rusb = rusb::Context::new().unwrap();
    let device = rusb
        .devices()
        .expect("Failed to scan devices for camera")
        .iter()
        .find(|some_device| {
            let description = some_device.device_descriptor();
            if !description.is_ok() {
                return false;
            }
            let desc = description.unwrap();

            return VendorType::from_vendor_id(desc.vendor_id()).is_some()
                && DeviceType::from_product_id(desc.product_id()).is_some();
        });

    let mountpoints: Vec<PathBuf> = mountpoints::mountpaths().expect("Failed to get mountpoints");

    if let Some(device) = device {
        println!("Found camera: {:?}", device);
        let desc = device.device_descriptor().unwrap();
        let vendor_id = desc.vendor_id();
        let product_id = desc.product_id();
        let interface =
            rusb::open_device_with_vid_pid(vendor_id, product_id).expect("Failed to open device");
        println!("{:?}", interface);
    }
}
