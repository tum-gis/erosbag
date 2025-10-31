use crate::util::parse_duration;
use crate::util::parse_timestamp;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueHint};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about, long_about = None, propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Extract the transforms
    ExtractTransforms {
        /// Path to the ROS2 bag
        #[clap(long, value_hint = ValueHint::DirPath)]
        rosbag_directory_path: PathBuf,

        /// Path to the output ecoord file
        #[clap(long, value_hint = ValueHint::FilePath)]
        output_ecoord_path: PathBuf,
    },

    /// Extract the point clouds
    ExtractPointClouds {
        /// Path to the ROS2 bag
        #[clap(long, value_hint = ValueHint::DirPath)]
        rosbag_directory_path: PathBuf,

        /// Path to the ecoord document
        #[clap(long, value_hint = ValueHint::FilePath)]
        ecoord_file_path: Option<PathBuf>,

        /// The start time of the import in UTC.
        /// Example: 2020-04-12 22:10:57.123456789 +00:00
        /// If not provided, the import starts from the beginning.
        #[clap(long, value_parser = parse_timestamp)]
        start_date_time: Option<DateTime<Utc>>,

        /// The stop time of the import in UTC.
        /// Example: 2020-04-12 22:10:57.123456789 +00:00
        /// If not provided, the import runs until the end of the available data.
        #[clap(long, value_parser = parse_timestamp)]
        stop_date_time: Option<DateTime<Utc>>,

        /// The time offset applied to the rosbag import.
        /// Example: "5s" (5 seconds), "2m" (2 minutes).
        /// If not provided, no offset is applied.
        #[clap(long, value_parser = parse_duration)]
        start_time_offset: Option<chrono::Duration>,

        /// The total duration of the rosbag import.
        /// Example: "30s" (30 seconds), "1h" (1 hour).
        /// If not provided, the import runs until the stop time or the end of the data.
        #[clap(long, value_parser = parse_duration)]
        total_duration: Option<chrono::Duration>,

        /// Name of the channel providing the transform messages
        #[clap(long, default_value_t = String::from("/tf_static"))]
        transform_channel_name: String,

        /// Target frame id of extracted point cloud
        #[clap(long)]
        target_frame_id: Option<String>,

        /// Path to the output epoint file containing the extracted point clouds
        #[clap(long, value_hint = ValueHint::FilePath)]
        output_path: PathBuf,
    },

    /// Extract the images
    ExtractImages {
        /// Path to the ROS2 bag
        #[clap(long, value_hint = ValueHint::DirPath)]
        rosbag_directory_path: PathBuf,

        /// Path to output eimage file containing the extracted images
        #[clap(long, value_hint = ValueHint::FilePath)]
        output_eimage_path: PathBuf,
    },

    /// Append the reference frames to a ROS bag
    CreateFromEcoord {
        /// Path to the directory containing reference frames
        #[clap(long, value_hint = ValueHint::FilePath)]
        reference_frames_directory_path: PathBuf,

        /// Path to the ROS2 bag
        #[clap(long, value_hint = ValueHint::DirPath)]
        rosbag_directory_path: PathBuf,
    },

    /// Tests
    Test {
        /// Path to the ROS2 bag
        #[clap(long, value_hint = ValueHint::DirPath)]
        rosbag_directory_path: PathBuf,
    },
}
