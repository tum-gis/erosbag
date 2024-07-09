use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None, propagate_version = true)]
pub struct Arguments {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Reindex ROS2 bag
    Repair {
        /// Path to the ROS2 bag
        #[clap(long)]
        rosbag_directory_path: String,
    },

    /// Extract the transforms
    ExtractTransforms {
        /// Path to the ROS2 bag
        #[clap(long)]
        rosbag_directory_path: String,

        /// Path to the output ecoord file
        #[clap(long)]
        output_ecoord_path: String,
    },

    /// Extract the point clouds
    ExtractPointClouds {
        /// Path to the ROS2 bag
        #[clap(long)]
        rosbag_directory_path: String,

        /// Path to the ecoord document
        #[clap(long)]
        ecoord_file_path: Option<String>,

        /// Target frame id of extracted point cloud
        #[clap(long)]
        frame_id: Option<String>,

        /// Path to the output epoint file containing the extracted point clouds
        #[clap(long)]
        output_epoint_path: String,
    },

    /// Extract the images
    ExtractImages {
        /// Path to the ROS2 bag
        #[clap(long)]
        rosbag_directory_path: String,

        /// Path to output eimage file containing the extracted images
        #[clap(long)]
        output_eimage_path: String,
    },

    /// Append the reference frames to a ROS bag
    CreateFromEcoord {
        /// Path to the directory containing reference frames
        #[clap(long)]
        reference_frames_directory_path: String,

        /// Path to the ROS2 bag
        #[clap(long)]
        rosbag_directory_path: String,
    },

    /// Tests
    Test {
        /// Path to the ROS2 bag
        #[clap(long)]
        rosbag_directory_path: String,
    },
}
