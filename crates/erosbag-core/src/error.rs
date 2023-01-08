use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid combinations of open options")]
    InvalidInput,

    #[error("path is not a directory")]
    RosbagPathIsNoDirectory,

    #[error("path is not a directory")]
    ContainsNoRosbagFile,

    #[error("unknown data store error")]
    Unknown,

    #[error(transparent)]
    ConnectionError(#[from] diesel::ConnectionError),

    #[error(transparent)]
    DieselError(#[from] diesel::result::Error),

    #[error(transparent)]
    EpointTransformError(#[from] epoint::transform::Error),
}
