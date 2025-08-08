use crate::identifier::ChannelId;
use crate::ros_messages::RosMessageType;
use crate::{ChannelTopic, ChunkId, FileName};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BagFileError(#[from] crate::bagfile::error::BagFileError),

    #[error(transparent)]
    EcoordError(#[from] ecoord::Error),
    #[error(transparent)]
    EpointError(#[from] epoint::Error),
    #[error(transparent)]
    EpointTransformError(#[from] epoint::transform::Error),
    #[error(transparent)]
    EimageError(#[from] eimage::Error),

    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
    #[error(transparent)]
    CdrError(#[from] cdr::Error),
    #[error(transparent)]
    McapError(#[from] mcap::McapError),

    #[error("Invalid combinations of open options")]
    InvalidInput,

    #[error("path is not a directory")]
    RosbagPathIsNoDirectory,

    #[error("path is not a directory")]
    ContainsNoRosbagFile,

    #[error("directory path contains no mcap file")]
    ContainsNoMcapFile,
    #[error("MCAP contains no file with name `{0}`")]
    ContainsNoMcapFileWithName(FileName),
    #[error("channel with topic `{0}` does not exist")]
    ChannelWithTopicDoesNotExist(ChannelTopic),
    #[error("channel with id `{0}` does not exist")]
    ChannelWithIdDoesNotExist(ChannelId),
    #[error("channel with id `{0}` does not have a schema")]
    ChannelWithoutSchema(ChannelTopic),
    #[error("channel with id `{0}` has the schema `{1}`")]
    ChannelWithInvalidSchema(ChannelTopic, String),
    #[error("channel with name `{0}` exists multiple times")]
    MultipleChannelsWithNameExist(String),

    #[error("chunk with id `{0}` not found")]
    ChunkIdNotFound(ChunkId),

    #[error("channel with id `{0}` does not hold messages of type `{1}`")]
    ChannelDoesNotHold(ChannelTopic, RosMessageType),

    #[error("multiple bagfiles are currently not supported by erosbag")]
    MultipleBagfilesNotSupported,
}
