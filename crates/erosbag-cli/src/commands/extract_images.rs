use chrono::{DateTime, Duration, Utc};
use erosbag::Rosbag;
use std::fs;

use crate::error::Error;
use std::path::Path;
use tracing::info;

pub fn run(
    rosbag_directory_path: impl AsRef<Path>,
    output_eimage_path: impl AsRef<Path>,
) -> Result<(), Error> {
    info!("Start extracting images");
    info!("Rosbag path: {}", rosbag_directory_path.as_ref().display());
    info!(
        "Output eimage path: {}",
        output_eimage_path.as_ref().display()
    );

    let rosbag = Rosbag::new(rosbag_directory_path.as_ref())?;

    let start_date_time: DateTime<Utc> =
        rosbag.get_start_date_time()?.unwrap() + Duration::seconds(5);
    let end_date_time: DateTime<Utc> = start_date_time + Duration::seconds(1);

    let image_collection =
        rosbag.get_images(&Some(start_date_time), &Some(end_date_time), &None)?;

    fs::create_dir_all(output_eimage_path.as_ref().parent().expect("should exist"))?;
    info!("Extracted {} images.", image_collection.total_image_count());
    eimage::io::EimageWriter::from_path(output_eimage_path)?
        .with_compressed(false)
        .finish(image_collection)?;

    Ok(())
}
