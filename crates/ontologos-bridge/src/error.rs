use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    #[error("bridge: {0}")]
    Bridge(String),
    #[error(transparent)]
    Reasonable(#[from] reasonable::error::ReasonableError),
}
