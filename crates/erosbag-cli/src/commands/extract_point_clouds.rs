use chrono::{DateTime, TimeZone, Utc};
use epoint::io::EpointWriter;
use erosbag::RosbagOpenOptions;
use std::fs;
use std::path::Path;
use tracing::info;

pub fn run(rosbag_directory_path: impl AsRef<Path>, output_directory_path: impl AsRef<Path>) {
    let rosbag = RosbagOpenOptions::new()
        .read_write(true)
        .open(&rosbag_directory_path.as_ref().to_owned())
        .unwrap();

    let start_time: &Option<DateTime<Utc>> =
        &Some(Utc.timestamp_nanos(1579007185814586880 + 500000000));
    let stop_time: &Option<DateTime<Utc>> =
        &Some(Utc.timestamp_nanos(1579007185814586880 + 500000000 + 5000000000));
    let point_cloud = rosbag.get_point_clouds(start_time, stop_time).unwrap();
    info!(
        "Start writing to: {}",
        output_directory_path.as_ref().display()
    );

    fs::create_dir_all(&output_directory_path).unwrap();
    EpointWriter::new(output_directory_path)
        .with_compressed(false)
        .finish(&point_cloud)
        .unwrap();
}
