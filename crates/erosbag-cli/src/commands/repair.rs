use erosbag::transform::repair_rosbag;
use std::path::Path;
use tracing::info;

pub fn run(rosbag_directory_path: impl AsRef<Path>) {
    info!("Start reindexing");

    repair_rosbag(rosbag_directory_path);
}
