use crate::error::Error;
use ecoord::io::EcoordWriter;
use erosbag::Rosbag;
use std::path::Path;
use tracing::info;

pub fn run(
    rosbag_directory_path: impl AsRef<Path>,
    output_ecoord_path: impl AsRef<Path>,
) -> Result<(), Error> {
    info!("Start extracting transforms");
    info!("Rosbag path: {}", rosbag_directory_path.as_ref().display());
    info!(
        "Output ecoord path: {}",
        output_ecoord_path.as_ref().display()
    );

    let rosbag = Rosbag::new(rosbag_directory_path.as_ref())?;
    let o = rosbag.get_overview()?;

    let transform_tree = rosbag.get_transforms(&None, &None, &None)?;

    info!(
        "Start writing to: {}",
        output_ecoord_path.as_ref().display()
    );
    EcoordWriter::from_path(output_ecoord_path)?
        .with_pretty(true)
        .finish(&transform_tree)?;

    Ok(())
}
