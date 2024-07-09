use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BagFileError(#[from] crate::bagfile::error::BagFileError),

    #[error(transparent)]
    EcoordError(#[from] ecoord::Error),
    #[error(transparent)]
    EpointTransformError(#[from] epoint::transform::Error),

    #[error(transparent)]
    ConnectionError(#[from] diesel::ConnectionError),

    #[error(transparent)]
    DieselError(#[from] diesel::result::Error),

    #[error("Invalid combinations of open options")]
    InvalidInput,

    #[error("path is not a directory")]
    RosbagPathIsNoDirectory,

    #[error("path is not a directory")]
    ContainsNoRosbagFile,

    #[error("multiple bagfiles are currently not supported by erosbag")]
    MultipleBagfilesNotSupported,
}
