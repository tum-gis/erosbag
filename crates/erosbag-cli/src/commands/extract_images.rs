use chrono::{DateTime, Duration, TimeZone, Utc};
use ecoord::{FrameId, ReferenceFrames};
use eimage::{ImageCollection, ImageSeries};
use erosbag::RosbagOpenOptions;
use std::collections::HashMap;

use std::path::Path;
use tracing::info;

pub fn run(rosbag_directory_path: impl AsRef<Path>, output_eimage_path: impl AsRef<Path>) {
    info!("Start extracting images");
    info!("Rosbag path: {}", rosbag_directory_path.as_ref().display());
    info!(
        "Output eimage path: {}",
        output_eimage_path.as_ref().display()
    );

    // fs::create_dir_all(output_file_path).unwrap();

    let rosbag = RosbagOpenOptions::new()
        .read_write(true)
        .open(rosbag_directory_path.as_ref())
        .unwrap();

    let start_time: Option<DateTime<Utc>> =
        Some(Utc.timestamp_nanos(1579007185814586880 + 1000000000) + Duration::seconds(5));
    let stop_time: Option<DateTime<Utc>> = start_time.map(|x| x + Duration::seconds(5));

    let image_series = rosbag
        .get_images(
            &"/camera_side_left/rgb/image_rect_color".to_string(),
            &start_time,
            &stop_time,
        )
        .unwrap();
    info!("Extracted {} images.", image_series.len());
    let mut image_series_map: HashMap<FrameId, ImageSeries> = HashMap::new();
    image_series_map.insert(FrameId::from("camera_side_left"), image_series);
    let image_collection =
        ImageCollection::new(image_series_map, ReferenceFrames::default()).unwrap();
    eimage::io::EimageWriter::from_path(output_eimage_path)
        .unwrap()
        .finish(image_collection)
        .unwrap();
}
