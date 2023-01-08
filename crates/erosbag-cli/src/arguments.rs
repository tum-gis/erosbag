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

    /// Extract the transforms to an ecoord document
    ExtractTransforms {
        /// Path to the ROS2 bag
        #[clap(long)]
        rosbag_directory_path: String,

        /// Path to the ecoord document
        #[clap(long)]
        output_ecoord_file_path: String,
    },

    /// Extract point clouds accumulated over time
    ExtractPointClouds {
        /// Path to the ROS2 bag
        #[clap(long)]
        rosbag_directory_path: String,

        /// Path to output directory of extracted point clouds
        #[clap(long)]
        output_directory_path: String,
    },

    /// Extract images
    ExtractImages {
        /// Path to the ROS2 bag
        #[clap(long)]
        rosbag_directory_path: String,

        /// Path to output directory of extracted images
        #[clap(long)]
        output_directory_path: String,
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
