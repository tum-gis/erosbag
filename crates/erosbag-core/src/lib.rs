#[macro_use]
extern crate diesel;
extern crate core;

mod bagfile;
pub mod error;
pub mod ros_messages;
mod rosbag;
mod rosbag_metadata;
mod rosbag_open_options;
pub mod topics;

#[doc(inline)]
pub use error::Error;

#[doc(inline)]
pub use rosbag::Rosbag;

#[doc(inline)]
pub use rosbag_open_options::RosbagOpenOptions;

pub const SQLITE3_EXTENSION: &str = "db3";
