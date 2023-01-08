use ecoord::io::EcoordReader;
use erosbag_core::RosbagOpenOptions;
use std::path::PathBuf;
use tracing::{error, info};
use walkdir::WalkDir;

pub fn append_reference_frames(
    reference_frames_directory_path: PathBuf,
    rosbag_directory_path: PathBuf,
) {
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
        .filter(|p| p.is_file() && p.extension().unwrap() == "ecoo")
        .collect();
    if ecoord_paths.is_empty() {
        error!(
            "No ecoord documents found in {}",
            reference_frames_directory_path.display()
        );
        return;
    }

    for current_ecoord_path in ecoord_paths {
        info!("Read:{}", current_ecoord_path.display());

        let current_reference_frames = EcoordReader::new(current_ecoord_path).finish().unwrap();
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
}
