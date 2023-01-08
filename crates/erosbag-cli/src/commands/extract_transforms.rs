use ecoord::io::EcoordWriter;
use erosbag::RosbagOpenOptions;
use std::path::Path;
use tracing::info;

pub fn run(rosbag_directory_path: impl AsRef<Path>, output_ecoord_file_path: impl AsRef<Path>) {
    let rosbag = RosbagOpenOptions::new()
        .read_write(true)
        .open(&rosbag_directory_path.as_ref().to_owned())
        .unwrap();

    let reference_frame = rosbag.get_transforms(&None, &None).unwrap();

    info!(
        "Start writing to: {}",
        output_ecoord_file_path.as_ref().display()
    );
    EcoordWriter::new(output_ecoord_file_path)
        .finish(&reference_frame)
        .expect("TODO: panic message");
}
