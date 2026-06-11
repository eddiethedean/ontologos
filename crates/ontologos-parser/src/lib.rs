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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detect_format_by_extension() {
        assert_eq!(detect_format(Path::new("p.owl")), Some(Format::OwlXml));
        assert_eq!(detect_format(Path::new("p.xml")), Some(Format::OwlXml));
        assert_eq!(detect_format(Path::new("p.rdf")), Some(Format::RdfXml));
        assert_eq!(detect_format(Path::new("p.ttl")), Some(Format::Turtle));
        assert_eq!(detect_format(Path::new("p.turtle")), Some(Format::Turtle));
        assert_eq!(detect_format(Path::new("p.ofn")), Some(Format::Functional));
        assert_eq!(detect_format(Path::new("p.func")), Some(Format::Functional));
        assert_eq!(detect_format(Path::new("p.txt")), None);
        assert_eq!(detect_format(Path::new("noext")), None);
    }
}
