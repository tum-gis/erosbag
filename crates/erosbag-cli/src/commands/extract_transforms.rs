use ecoord::io::EcoordWriter;
use erosbag::RosbagOpenOptions;
use std::path::Path;
use tracing::info;

pub fn run(rosbag_directory_path: impl AsRef<Path>, output_ecoord_path: impl AsRef<Path>) {
    info!("Start extracting transforms");
    info!("Rosbag path: {}", rosbag_directory_path.as_ref().display());
    info!(
        "Output ecoord path: {}",
        output_ecoord_path.as_ref().display()
    );

    let rosbag = RosbagOpenOptions::new()
        .read_write(true)
        .open(rosbag_directory_path.as_ref())
        .unwrap();

    let reference_frame = rosbag.get_transforms(&None, &None).unwrap();

    info!(
        "Start writing to: {}",
        output_ecoord_path.as_ref().display()
    );
    EcoordWriter::from_path(output_ecoord_path)
        .unwrap()
        .finish(&reference_frame)
        .expect("TODO: panic message");
}
