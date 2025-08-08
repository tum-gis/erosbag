mod bagfile;
pub mod dto;
pub mod error;
pub mod identifier;
mod mcap;
mod mcap_file;
pub mod ros_messages;
pub mod topics;

#[doc(inline)]
pub use error::Error;

#[doc(inline)]
pub use identifier::TopicId;

#[doc(inline)]
pub use identifier::MessageId;

#[doc(inline)]
pub use identifier::FileName;

#[doc(inline)]
pub use identifier::ChunkId;

#[doc(inline)]
pub use identifier::ChannelId;

#[doc(inline)]
pub use identifier::ChannelTopic;

#[doc(inline)]
pub use bagfile::point_cloud_extensions::RosPointDataColumnType;

#[doc(inline)]
pub use mcap::Rosbag;

#[doc(inline)]
pub use mcap_file::McapFile;

pub const MCAP_EXTENSION: &str = "mcap";
pub const SQLITE3_EXTENSION: &str = "db3";
