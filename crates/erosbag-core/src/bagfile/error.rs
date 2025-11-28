use chrono::Utc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BagFileError {
    #[error(transparent)]
    EcoordError(#[from] ecoord::Error),

    #[error(transparent)]
    EpointTransformError(#[from] epoint::transform::Error),

    #[error(transparent)]
    EimageError(#[from] eimage::Error),

    #[error(transparent)]
    CdrError(#[from] cdr::Error),

    #[error("topic with name `{0}` does not exist")]
    TopicWithNameDoesNotExist(String),

    #[error("topic with id `{0}` does not exist")]
    TopicWithIdDoesNotExist(i32),

    #[error("topic `{0}` already exists")]
    TopicAlreadyExists(String),

    #[error("topic `{0}` already exists")]
    InvalidMessageType(&'static str),

    #[error("unknown data store error")]
    RequestedTimeInvalid {
        requested_time: chrono::DateTime<Utc>,
        bag_start_time: chrono::DateTime<Utc>,
        bag_end_time: chrono::DateTime<Utc>,
    },

    #[error("path is not a directory")]
    ContainsNoPointCloud2Topics,

    #[error("path is not a directory")]
    ContainsNoMessages,

    #[error("path is not a directory")]
    ContainsNoTimestamp,

    #[error("message type is not supported")]
    UnsupportedMessageType(),
}
