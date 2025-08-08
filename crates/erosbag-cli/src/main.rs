mod cli;
mod commands;
mod error;
mod util;

use crate::cli::{Cli, Commands};
use clap::Parser;

use anyhow::Result;
use ecoord::FrameId;
use erosbag::ChannelTopic;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::ExtractTransforms {
            rosbag_directory_path,
            output_ecoord_path,
        } => {
            let rosbag_directory_path = Path::new(rosbag_directory_path).canonicalize()?;
            let output_ecoord_path = PathBuf::from(output_ecoord_path);

            commands::extract_transforms::run(rosbag_directory_path, output_ecoord_path)?;
        }
        Commands::ExtractPointClouds {
            rosbag_directory_path,
            ecoord_file_path,
            start_date_time,
            stop_date_time,
            start_time_offset,
            total_duration,
            transform_channel_name,
            target_frame_id,
            output_path,
        } => {
            let rosbag_directory_path = Path::new(rosbag_directory_path).canonicalize()?;
            let ecoord_file_path = ecoord_file_path
                .as_ref()
                .map(|x| Path::new(x).canonicalize().unwrap());
            let transform_channel_id: ChannelTopic = transform_channel_name.as_str().into();
            let target_frame_id = target_frame_id.as_ref().map(|x| FrameId::from(x.clone()));
            let output_path = PathBuf::from(output_path);

            commands::extract_point_clouds::run(
                rosbag_directory_path,
                ecoord_file_path,
                *start_date_time,
                *stop_date_time,
                *start_time_offset,
                *total_duration,
                transform_channel_id,
                target_frame_id,
                output_path,
            )?;
        }
        Commands::ExtractImages {
            rosbag_directory_path,
            output_eimage_path,
        } => {
            let rosbag_directory_path = Path::new(rosbag_directory_path).canonicalize()?;
            let output_eimage_path = PathBuf::from(output_eimage_path);

            commands::extract_images::run(rosbag_directory_path, output_eimage_path)?;
        }
        Commands::CreateFromEcoord {
            reference_frames_directory_path,
            rosbag_directory_path,
        } => {
            let reference_frames_directory_path = PathBuf::from(reference_frames_directory_path);
            let rosbag_directory_path = PathBuf::from(rosbag_directory_path);

            /*erosbag::transform::append_reference_frames(
                reference_frames_directory_path,
                rosbag_directory_path,
            )?;*/
        }
        Commands::Test {
            rosbag_directory_path,
        } => {
            let rosbag_directory_path = PathBuf::from(rosbag_directory_path);

            commands::test::run(rosbag_directory_path)?;
        }
    };

    Ok(())
}
