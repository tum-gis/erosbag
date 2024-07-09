use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    RosbagError(#[from] erosbag_core::Error),
    #[error(transparent)]
    EcoordError(#[from] ecoord::Error),
}
