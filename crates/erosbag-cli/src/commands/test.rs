use std::path::Path;
use tracing::info;

pub fn run(_rosbag_directory_path: impl AsRef<Path>) {
    info!("Run some nice tests...");
}
