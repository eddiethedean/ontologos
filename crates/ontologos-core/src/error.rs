use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("ontology not loaded")]
    OntologyNotLoaded,
    #[error("reasoning not yet implemented")]
    NotImplemented,
    #[error("{0}")]
    Message(String),
}
