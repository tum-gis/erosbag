use erosbag_core::topics::qos_profile::QualityOfServiceProfile;
use erosbag_core::RosbagOpenOptions;
use std::fs;
use std::path::PathBuf;
use tracing::info;

pub fn repair_rosbag(rosbag_directory_path: PathBuf) {
    info!("Rosbag: {}", rosbag_directory_path.display());

    normalize_rosbag_files(rosbag_directory_path.clone());

    let mut rosbag = RosbagOpenOptions::new()
        .read_write(true)
        .open(&rosbag_directory_path)
        .unwrap();

    let quality_of_service_profile = QualityOfServiceProfile::new_for_static_tf_topic();
    rosbag.set_qos_profile(&"/tf_static".to_string(), &quality_of_service_profile);
    let quality_of_service_profile = QualityOfServiceProfile::new_for_tf_topic();
    rosbag.set_qos_profile(
        &"/tf_custom/slam_map_to_base_link/slam".to_string(),
        &quality_of_service_profile,
    );

    rosbag.close();

    info!("ok done");
}

fn normalize_rosbag_files(rosbag_directory_path: PathBuf) {
    let rosbag_directory_name = rosbag_directory_path.file_name().unwrap().to_str().unwrap();

    let direct_filename =
        PathBuf::from(rosbag_directory_name).with_extension(erosbag_core::SQLITE3_EXTENSION);
    let main_path = rosbag_directory_path.join(direct_filename);

    if main_path.exists() {
        let indexed_filename = PathBuf::from(rosbag_directory_name.to_string() + "_0")
            .with_extension(erosbag_core::SQLITE3_EXTENSION);
        let indexed_path = rosbag_directory_path.join(indexed_filename);
        fs::rename(&main_path, indexed_path).expect("Renaming failed");
    }
}
