//! OWL and RDF syntax parsers for OntoLogos.
//!
//! v0.2 loads OWL/XML, RDF/XML, Turtle, and OWL Functional Syntax into
//! [`ontologos_core::Ontology`] via [`load_ontology`].
//!
//! # Example
//!
//! ```no_run
//! use ontologos_parser::load_ontology;
//!
//! let ontology = load_ontology(std::path::Path::new("ontology.owl"))?;
//! println!("axioms: {}", ontology.axiom_count());
//! # Ok::<(), ontologos_parser::Error>(())
//! ```
//!
//! See [load guide](https://github.com/eddiethedean/ontologos/blob/main/docs/getting-started/load-owl-file.md).

#![warn(missing_docs)]

mod error;
mod limits;
mod load;
mod map;
mod map_dl;
mod map_swrl;
mod rdf_preprocess;
mod read;
mod report;
mod validate;

pub use error::{Error, Result};
pub use limits::ParseLimits;
pub use load::{
    load_ofn_from_str, load_ofn_from_str_with_limits, load_ofn_with_incremental,
    load_ofn_with_incremental_and_limits, load_ontology, load_ontology_from_bytes,
    load_ontology_from_bytes_lenient, load_ontology_from_bytes_with_limits,
    load_ontology_from_str, load_ontology_from_str_lenient, load_ontology_in,
    load_ontology_lenient, load_ontology_lenient_in, load_ontology_with_limits,
    load_ontology_with_limits_and_base, validate_load_path,
};
pub use rdf_preprocess::{expand_xml_entities, expand_xml_entities_with_limit};
pub use read::{detect_turtle_from_bytes, read_horned_owl_from_reader, sniff_file_header};
pub use validate::{validate_loaded_ontology, validate_loaded_ontology_light};

/// Supported ontology serialization formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// OWL/XML syntax.
    OwlXml,
    /// RDF/XML syntax.
    RdfXml,
    /// Turtle / RDF Turtle.
    Turtle,
    /// OWL Functional Syntax (`.ofn`, `.func`).
    Functional,
}

/// Detect format from file content bytes.
#[must_use]
pub fn detect_format_from_bytes(header: &[u8]) -> Option<Format> {
    if detect_functional_from_bytes(header) {
        return Some(Format::Functional);
    }
    let text = std::str::from_utf8(header).ok()?;
    let trimmed = text.trim_start();
    if trimmed.contains("rdf:RDF") || trimmed.contains("<rdf:RDF") {
        return Some(Format::RdfXml);
    }
    if trimmed.contains("<Ontology ") || trimmed.contains(":Ontology ") {
        return Some(Format::OwlXml);
    }
    if detect_turtle_from_bytes(header) {
        return Some(Format::Turtle);
    }
    None
}

/// Detect OWL Functional Syntax from a file header.
#[must_use]
pub fn detect_functional_from_bytes(header: &[u8]) -> bool {
    let text = match std::str::from_utf8(header) {
        Ok(t) => t.trim_start(),
        Err(_) => return false,
    };
    text.starts_with("Prefix(") || text.starts_with("Ontology(")
}

/// Detect the most likely format from a file path and optional content sniffing.
#[must_use]
pub fn detect_format(path: &std::path::Path) -> Option<Format> {
    match path.extension()?.to_str()? {
        "owl" => sniff_xml_format(path),
        "xml" => sniff_xml_format(path),
        "rdf" => Some(Format::RdfXml),
        "ttl" | "turtle" => Some(Format::Turtle),
        "ofn" | "func" => Some(Format::Functional),
        _ => None,
    }
}

fn sniff_xml_format(path: &std::path::Path) -> Option<Format> {
    let header = sniff_file_header(path, 4096).ok()?;
    detect_format_from_bytes(&header)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;

    #[test]
    fn detect_format_by_extension() {
        assert_eq!(detect_format(Path::new("p.rdf")), Some(Format::RdfXml));
        assert_eq!(detect_format(Path::new("p.ttl")), Some(Format::Turtle));
        assert_eq!(detect_format(Path::new("p.turtle")), Some(Format::Turtle));
        assert_eq!(detect_format(Path::new("p.ofn")), Some(Format::Functional));
        assert_eq!(detect_format(Path::new("p.func")), Some(Format::Functional));
        assert_eq!(detect_format(Path::new("p.txt")), None);
        assert_eq!(detect_format(Path::new("noext")), None);
    }

    #[test]
    fn detect_functional_from_bytes_header() {
        let header = b"Prefix(:=<http://example.org/>)\nOntology(<http://example.org/o>)";
        assert!(detect_functional_from_bytes(header));
        assert_eq!(detect_format_from_bytes(header), Some(Format::Functional));
    }

    #[test]
    fn detect_format_from_bytes_owl_xml() {
        let header = br#"<?xml version="1.0"?><Ontology xmlns="http://www.w3.org/2002/07/owl#"/>"#;
        assert_eq!(detect_format_from_bytes(header), Some(Format::OwlXml));
    }

    #[test]
    fn detect_format_from_bytes_rdf_xml() {
        let header = br#"<?xml version="1.0"?><rdf:RDF/>"#;
        assert_eq!(detect_format_from_bytes(header), Some(Format::RdfXml));
    }

    #[test]
    fn plain_xml_extension_without_sniff_returns_none() {
        let path = std::env::temp_dir().join(format!(
            "ontologos_parser_test_{}_{}.xml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        {
            let mut file = std::fs::File::create(&path).expect("create");
            file.write_all(b"<config><item/></config>").expect("write");
        }
        assert_eq!(detect_format(&path), None);
        let _ = std::fs::remove_file(&path);
    }
}
