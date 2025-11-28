use crate::error::Error;
use erosbag::Rosbag;
use std::path::Path;
use tracing::info;

pub fn run(rosbag_directory_path: impl AsRef<Path>) -> Result<(), Error> {
    info!("Run some nice tests...");

    let rosbag = Rosbag::new(rosbag_directory_path)?;
    let mcap_file = rosbag.mcap_files.get(&"test".into()).unwrap();

    let start_date_time = mcap_file.get_start_date_time()?;
    let end_date_time = mcap_file.get_end_date_time()?;
    info!("start_date_time: {:?}", start_date_time);
    info!("end_date_time: {:?}", end_date_time);

    // let channel_id = mcap_dir.get_channel_id("/fix")?;
    //let channel_id = mcap_dir.get_channel_id("/vehicle/sensor/fix")?;
    //let messages = mcap_file
    //    .get_nav_sat_fixes_of_channel(channel_id, &None, &None)?;

    // mcap_file.test(&start_date_time, &end_date_time)?;

    Ok(())
}
