use nusb::{list_devices, MaybeFuture};

static CAMERA: LazyLock<String> = LazyLock::new(|string_value| string_value.to_string());

pub fn scan_for_camera() {
    let usb_devices = list_devices();
    println("the devices")
}
