use erosbag::RosbagOpenOptions;
use std::fs;
use std::path::Path;

use tracing::info;

pub fn run(rosbag_directory_path: impl AsRef<Path>, output_directory_path: impl AsRef<Path>) {
    fs::create_dir_all(output_directory_path).unwrap();

    let rosbag = RosbagOpenOptions::new()
        .read_write(true)
        .open(&rosbag_directory_path.as_ref().to_owned())
        .unwrap();

    rosbag.get_images();
}
