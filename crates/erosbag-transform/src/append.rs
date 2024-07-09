use ecoord::io::EcoordReader;
use erosbag_core::RosbagOpenOptions;
use std::path::{Path, PathBuf};
use tracing::{error, info};
use walkdir::WalkDir;

pub fn append_reference_frames(
    reference_frames_directory_path: impl AsRef<Path>,
    rosbag_directory_path: impl AsRef<Path>,
) {
    info!("Start creation");
    info!(
        "Reference frames directory path: {}",
        reference_frames_directory_path.as_ref().display()
    );
    info!("Rosbag path: {}", rosbag_directory_path.as_ref().display());

    let mut rosbag = RosbagOpenOptions::new()
        .create_new(true)
        .open(&rosbag_directory_path)
        .unwrap();

    let paths: Vec<_> = WalkDir::new(&reference_frames_directory_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();
    //paths.sort_by_key(|dir| &dir.path());

    let ecoord_paths: Vec<PathBuf> = paths
        .into_iter()
        .map(|p| PathBuf::from(p.path()))
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .ends_with("ecoord.json")
        })
        .collect();
    if ecoord_paths.is_empty() {
        error!(
            "No ecoord documents found in {}",
            reference_frames_directory_path.as_ref().display()
        );
        return;
    }

    for current_ecoord_path in ecoord_paths {
        info!("Read:{}", current_ecoord_path.display());

        let current_reference_frames = EcoordReader::from_path(current_ecoord_path)
            .unwrap()
            .finish()
            .unwrap();
        rosbag.append_transform_messages(current_reference_frames);

        info!("read it");
    }

    rosbag.close();
    //    .for_each(|p| println!("{}", p.path().display()));
    /*{
        println!("{}", entry.path().display());
    }*/

    /*let ecoord_paths: Vec<PathBuf> = WalkDir::new(&reference_frames_directory_path)
    .into_iter()
    .map(|r| r.unwrap().path())
    //.filter(|p| p.is_file())
    //.filter(|p| p.file_name().unwrap() == "frames")
    .collect();*/

    info!("Completed.");
}
