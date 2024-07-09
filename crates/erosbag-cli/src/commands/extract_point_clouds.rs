use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use ecoord::merge;
use epoint::io::{EpointWriter, XyzWriter};
use erosbag::RosbagOpenOptions;

use std::path::{Path, PathBuf};
use tracing::info;

pub fn run(
    rosbag_directory_path: impl AsRef<Path>,
    ecoord_file_path: Option<impl AsRef<Path>>,
    _frame_id: Option<ecoord::FrameId>,
    output_epoint_path: impl AsRef<Path>,
) {
    info!("Start extracting point clouds");
    info!("Rosbag path: {}", rosbag_directory_path.as_ref().display());
    info!(
        "Output epoint path: {}",
        output_epoint_path.as_ref().display()
    );

    let rosbag = RosbagOpenOptions::new()
        .read_write(true)
        .open(rosbag_directory_path.as_ref())
        .unwrap();

    //let start_time: &Option<DateTime<Utc>> =
    //    &Some(Utc.timestamp_nanos(1579007185814586880 + 500000000));
    let start_time: &Option<DateTime<Utc>> = &Some(Utc.from_utc_datetime(&NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2020, 11, 18).unwrap(),
        NaiveTime::from_hms_opt(12, 36, 02).unwrap(),
    )));
    let stop_time: &Option<DateTime<Utc>> = &start_time.map(|x| x + Duration::milliseconds(100));
    //&Some(Utc.timestamp_nanos(1579007185814586880 + 500000000 + 50000000000));
    let mut point_cloud = rosbag.get_point_clouds(start_time, stop_time).unwrap();
    if let Some(ecoord_file_path) = ecoord_file_path {
        let additional_reference_frames = ecoord::io::EcoordReader::from_path(ecoord_file_path)
            .unwrap()
            .finish()
            .unwrap();

        let original_reference_frames = point_cloud.reference_frames().clone();
        let merged_reference_frames =
            merge(&[original_reference_frames, additional_reference_frames]).unwrap();

        point_cloud.set_reference_frames(merged_reference_frames);
    }

    point_cloud.point_data.add_sequential_id().expect("");
    point_cloud.derive_spherical_points().expect("should work");
    // point_cloud.resolve_to_frame(frame_id.clone()).unwrap();

    info!(
        "Start writing to: {}",
        output_epoint_path.as_ref().display()
    );

    // fs::create_dir_all(&output_directory_path).unwrap();
    EpointWriter::from_path(&output_epoint_path)
        .unwrap()
        .with_compressed(false)
        .finish(point_cloud.clone())
        .unwrap();

    let mut xyz_path: PathBuf = output_epoint_path.as_ref().to_owned().clone();
    xyz_path.set_extension("xyz");
    XyzWriter::new(xyz_path).finish(&point_cloud).unwrap();

    info!("Completed.");
}
