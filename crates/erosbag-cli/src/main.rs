mod cli;
mod commands;
mod error;
mod util;

use crate::cli::{Cli, Commands};
use clap::Parser;

use anyhow::Result;
use ecoord::FrameId;
use erosbag::ChannelTopic;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::ExtractTransforms {
            rosbag_directory_path,
            output_ecoord_path,
        } => {
            commands::extract_transforms::run(
                rosbag_directory_path.canonicalize()?,
                output_ecoord_path,
            )?;
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
            let transform_channel_id: ChannelTopic = transform_channel_name.as_str().into();
            let target_frame_id = target_frame_id.as_ref().map(|x| FrameId::from(x.clone()));

            commands::extract_point_clouds::run(
                rosbag_directory_path.canonicalize()?,
                ecoord_file_path.as_ref().map(|x| x.canonicalize().unwrap()),
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
            commands::extract_images::run(
                rosbag_directory_path.canonicalize()?,
                output_eimage_path,
            )?;
        }
        Commands::CreateFromEcoord {
            reference_frames_directory_path,
            rosbag_directory_path,
        } => {

            /*erosbag::transform::append_reference_frames(
                reference_frames_directory_path,
                rosbag_directory_path,
            )?;*/
        }
        Commands::Test {
            rosbag_directory_path,
        } => {
            commands::test::run(rosbag_directory_path)?;
        }
    };

    Ok(())
}
