//! OWL and RDF syntax parsers for Ontologos.

mod error;

pub use error::{Error, Result};

/// Supported ontology serialization formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    OwlXml,
    RdfXml,
    Turtle,
    Functional,
}

/// Detect the most likely format from a file extension.
#[must_use]
pub fn detect_format(path: &std::path::Path) -> Option<Format> {
    match path.extension()?.to_str()? {
        "owl" | "xml" => Some(Format::OwlXml),
        "rdf" => Some(Format::RdfXml),
        "ttl" | "turtle" => Some(Format::Turtle),
        "ofn" | "func" => Some(Format::Functional),
        _ => None,
    }
}
