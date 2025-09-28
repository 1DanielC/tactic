use crate::camera_fs::camera_finder::scan_for_camera_fs;
use walkdir::WalkDir;

pub fn upload_all_files() {
    if let Some(volume) = scan_for_camera_fs() {
        for entry in WalkDir::new(volume) {
            match entry {
                Err(e) => println!("{:?}", e),
                Ok(entry) => println!("{:?}", entry),
            }
        }
    }
}
