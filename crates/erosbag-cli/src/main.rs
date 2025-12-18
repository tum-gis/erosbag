mod cli;
mod commands;
mod error;
mod util;

use crate::cli::{Cli, Commands};
use clap::Parser;

use anyhow::Result;
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
            end_date_time,
            start_time_offset,
            total_duration,
            transform_channel_name,
            target_frame_id,
            output_path,
        } => {
            let transform_channel_id: ChannelTopic = transform_channel_name.as_str().into();

            commands::extract_point_clouds::run(
                rosbag_directory_path.canonicalize()?,
                ecoord_file_path.as_ref().map(|x| x.canonicalize().unwrap()),
                *start_date_time,
                *end_date_time,
                *start_time_offset,
                *total_duration,
                transform_channel_id,
                target_frame_id.clone(),
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
            transform_tree_directory_path: _,
            rosbag_directory_path: _,
        } => {

            /*erosbag::transform::append_transform_tree(
                transform_tree_directory_path,
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
