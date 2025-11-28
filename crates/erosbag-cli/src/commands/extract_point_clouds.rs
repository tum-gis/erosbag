use chrono::{DateTime, Duration, Utc};
use ecoord::merge;
use epoint::io::AutoWriter;
use erosbag::{ChannelTopic, Rosbag};
use std::collections::HashSet;

use crate::error::Error;
use std::path::Path;
use tracing::{info, warn};

pub fn run(
    rosbag_directory_path: impl AsRef<Path>,
    ecoord_file_path: Option<impl AsRef<Path>>,
    start_date_time: Option<DateTime<Utc>>,
    end_date_time: Option<DateTime<Utc>>,
    start_time_offset: Option<Duration>,
    total_duration: Option<Duration>,
    transform_channel_id: ChannelTopic,
    target_frame_id: Option<ecoord::FrameId>,
    output_path: impl AsRef<Path>,
) -> Result<(), Error> {
    let rosbag = Rosbag::new(rosbag_directory_path.as_ref())?;
    let rosbag_start_date_time = match rosbag.get_start_date_time()? {
        Some(date_time) => date_time,
        None => {
            panic!("Not able to retrieve start date time from Rosbag.")
        }
    };
    let rosbag_end_date_time = match rosbag.get_end_date_time()? {
        Some(date_time) => date_time,
        None => {
            panic!("Not able to retrieve end date time from Rosbag.")
        }
    };
    info!(
        "Rosbag times: {rosbag_start_date_time} - {rosbag_end_date_time} with a duration of {}",
        rosbag_end_date_time - rosbag_start_date_time
    );

    let start_date_time: DateTime<Utc> =
        start_date_time.unwrap_or(rosbag_start_date_time) + start_time_offset.unwrap_or_default();
    let end_date_time: DateTime<Utc> = match (total_duration, end_date_time) {
        (Some(_total_duration), Some(end_date_time)) => {
            warn!("Both end_date_time and total_duration defined. Using end_date_time");
            end_date_time
        }
        (Some(total_duration), None) => start_date_time + total_duration,
        (None, Some(end_date_time)) => end_date_time,
        _ => rosbag_end_date_time,
    };

    let start_date_time = if rosbag_start_date_time <= start_date_time {
        start_date_time
    } else {
        warn!(
            "Defined start_date_time ({}) is before rosbag's start date time ({})",
            start_date_time, rosbag_start_date_time
        );
        rosbag_start_date_time
    };
    let end_date_time = if end_date_time <= rosbag_end_date_time {
        end_date_time
    } else {
        warn!(
            "Defined end_date_time ({}) is after rosbag's end date time ({})",
            end_date_time, rosbag_end_date_time
        );
        rosbag_end_date_time
    };

    let mut point_cloud =
        rosbag.get_point_clouds(&Some(start_date_time), &Some(end_date_time), &None)?;
    info!("Read {} points", point_cloud.size());
    if rosbag.contains_channel(&transform_channel_id)? {
        let channel_ids = HashSet::from([transform_channel_id]);
        point_cloud.transform_tree = rosbag.get_transforms(&None, &None, &Some(channel_ids))?;
    }

    if let Some(ecoord_file_path) = ecoord_file_path {
        let additional_transform_tree =
            ecoord::io::EcoordReader::from_path(ecoord_file_path)?.finish()?;

        let original_transform_tree = point_cloud.transform_tree().clone();
        let merged_transform_tree = merge(&[original_transform_tree, additional_transform_tree])?;

        point_cloud.set_transform_tree(merged_transform_tree);
    }

    // point_cloud.point_data.add_sequential_id()?;
    // point_cloud.derive_spherical_points()?;
    if let Some(target_frame_id) = target_frame_id {
        point_cloud.resolve_to_frame(target_frame_id.clone())?;
        info!("Resolved to frame_id: {}", target_frame_id);
    }

    point_cloud = point_cloud.filter_by_beam_length(0.0, 30.0)?.unwrap();
    info!("Filtered to {} points", point_cloud.size());

    info!("Start writing to: {}", output_path.as_ref().display());
    AutoWriter::from_path(output_path)?.finish(point_cloud)?;

    info!("Completed.");

    Ok(())
}
