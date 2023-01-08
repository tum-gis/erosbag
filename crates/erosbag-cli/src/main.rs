mod arguments;
mod commands;

use crate::arguments::{Arguments, Commands};
use clap::Parser;
use erosbag::transform::repair_rosbag;
use std::path::{Path, PathBuf};
use tracing::info;

fn main() {
    tracing_subscriber::fmt::init();
    let args = Arguments::parse();

    let x = match &args.command {
        Commands::Repair {
            rosbag_directory_path,
        } => {
            info!("Start reindexing");

            let rosbag_directory_path = Path::new(rosbag_directory_path).canonicalize().unwrap();
            repair_rosbag(rosbag_directory_path);
        }
        Commands::ExtractTransforms {
            rosbag_directory_path,
            output_ecoord_file_path,
        } => {
            info!("Start extracting transforms");
            let rosbag_directory_path = Path::new(rosbag_directory_path).canonicalize().unwrap();
            let output_ecoord_file_path = PathBuf::from(output_ecoord_file_path);
            info!("Rosbag path: {}", rosbag_directory_path.display());
            info!(
                "Output directory path: {}",
                output_ecoord_file_path.display()
            );

            commands::extract_transforms::run(rosbag_directory_path, output_ecoord_file_path);
        }
        Commands::ExtractPointClouds {
            rosbag_directory_path,
            output_directory_path,
        } => {
            info!("Start extracting point clouds");
            let rosbag_directory_path = Path::new(rosbag_directory_path).canonicalize().unwrap();
            let output_directory_path = PathBuf::from(output_directory_path);
            info!("Rosbag path: {}", rosbag_directory_path.display());
            info!("Output directory path: {}", output_directory_path.display());

            commands::extract_point_clouds::run(rosbag_directory_path, output_directory_path);

            info!("Completed.");
        }
        Commands::ExtractImages {
            rosbag_directory_path,
            output_directory_path,
        } => {
            info!("Start extracting images");

            let rosbag_directory_path = Path::new(rosbag_directory_path).canonicalize().unwrap();
            let output_directory_path = PathBuf::from(output_directory_path);

            info!("Rosbag path: {}", rosbag_directory_path.display());
            info!("Output directory path: {}", output_directory_path.display());

            commands::extract_image_data::run(rosbag_directory_path, output_directory_path);

            info!("Completed.");
        }
        Commands::CreateFromEcoord {
            reference_frames_directory_path,
            rosbag_directory_path,
        } => {
            info!("Start creation");
            let reference_frames_directory_path = PathBuf::from(reference_frames_directory_path);
            let rosbag_directory_path = PathBuf::from(rosbag_directory_path);

            info!(
                "Reference frames directory path: {}",
                reference_frames_directory_path.display()
            );
            info!("Rosbag path: {}", rosbag_directory_path.display());

            erosbag::transform::append_reference_frames(
                reference_frames_directory_path,
                rosbag_directory_path,
            );

            info!("Completed.");
        }
        Commands::Test {
            rosbag_directory_path,
        } => {
            let rosbag_directory_path = PathBuf::from(rosbag_directory_path);

            commands::test::run(rosbag_directory_path);
        }
    };
    x
}
