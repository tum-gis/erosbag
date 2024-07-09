mod arguments;
mod commands;

use crate::arguments::{Arguments, Commands};
use clap::Parser;

use ecoord::FrameId;
use std::path::{Path, PathBuf};

fn main() {
    tracing_subscriber::fmt::init();
    let arguments = Arguments::parse();

    match &arguments.command {
        Commands::Repair {
            rosbag_directory_path,
        } => {
            let rosbag_directory_path = Path::new(rosbag_directory_path).canonicalize().unwrap();

            commands::repair::run(rosbag_directory_path);
        }
        Commands::ExtractTransforms {
            rosbag_directory_path,
            output_ecoord_path,
        } => {
            let rosbag_directory_path = Path::new(rosbag_directory_path).canonicalize().unwrap();
            let output_ecoord_path = PathBuf::from(output_ecoord_path);

            commands::extract_transforms::run(rosbag_directory_path, output_ecoord_path);
        }
        Commands::ExtractPointClouds {
            rosbag_directory_path,
            ecoord_file_path,
            frame_id,
            output_epoint_path,
        } => {
            let rosbag_directory_path = Path::new(rosbag_directory_path).canonicalize().unwrap();
            let ecoord_file_path = ecoord_file_path
                .as_ref()
                .map(|x| Path::new(x).canonicalize().unwrap());
            let frame_id = frame_id.as_ref().map(|x| FrameId::from(x.clone()));
            let output_epoint_path = PathBuf::from(output_epoint_path);

            commands::extract_point_clouds::run(
                rosbag_directory_path,
                ecoord_file_path,
                frame_id,
                output_epoint_path,
            );
        }
        Commands::ExtractImages {
            rosbag_directory_path,
            output_eimage_path,
        } => {
            let rosbag_directory_path = Path::new(rosbag_directory_path).canonicalize().unwrap();
            let output_eimage_path = PathBuf::from(output_eimage_path);

            commands::extract_images::run(rosbag_directory_path, output_eimage_path);
        }
        Commands::CreateFromEcoord {
            reference_frames_directory_path,
            rosbag_directory_path,
        } => {
            let reference_frames_directory_path = PathBuf::from(reference_frames_directory_path);
            let rosbag_directory_path = PathBuf::from(rosbag_directory_path);

            erosbag::transform::append_reference_frames(
                reference_frames_directory_path,
                rosbag_directory_path,
            );
        }
        Commands::Test {
            rosbag_directory_path,
        } => {
            let rosbag_directory_path = PathBuf::from(rosbag_directory_path);

            commands::test::run(rosbag_directory_path);
        }
    };
}
