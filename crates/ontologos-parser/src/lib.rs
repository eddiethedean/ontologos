//! OWL and RDF syntax parsers for Ontologos.

mod error;
mod load;

pub use error::{Error, Result};
pub use load::{load_ontology, validate_load_path};

/// Supported ontology serialization formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    OwlXml,
    RdfXml,
    Turtle,
    Functional,
}

/// Detect format from file content bytes.
#[must_use]
pub fn detect_format_from_bytes(header: &[u8]) -> Option<Format> {
    let text = std::str::from_utf8(header).ok()?;
    let trimmed = text.trim_start();
    if trimmed.contains("owl:Ontology") || trimmed.contains("<owl:Ontology") {
        return Some(Format::OwlXml);
    }
    if trimmed.contains("rdf:RDF") || trimmed.contains("<rdf:RDF") {
        return Some(Format::RdfXml);
    }
    None
}

/// Detect the most likely format from a file path and optional content sniffing.
#[must_use]
pub fn detect_format(path: &std::path::Path) -> Option<Format> {
    match path.extension()?.to_str()? {
        "owl" => Some(Format::OwlXml),
        "xml" => sniff_xml_format(path),
        "rdf" => Some(Format::RdfXml),
        "ttl" | "turtle" => Some(Format::Turtle),
        "ofn" | "func" => Some(Format::Functional),
        _ => None,
    }
}

fn sniff_xml_format(path: &std::path::Path) -> Option<Format> {
    const SNIFF_BYTES: usize = 4096;
    let mut header = vec![0_u8; SNIFF_BYTES];
    let read = std::fs::File::open(path)
        .and_then(|mut file| {
            use std::io::Read;
            file.read(&mut header)
        })
        .ok()?;
    detect_format_from_bytes(&header[..read])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;

    #[test]
    fn detect_format_by_extension() {
        assert_eq!(detect_format(Path::new("p.owl")), Some(Format::OwlXml));
        assert_eq!(detect_format(Path::new("p.rdf")), Some(Format::RdfXml));
        assert_eq!(detect_format(Path::new("p.ttl")), Some(Format::Turtle));
        assert_eq!(detect_format(Path::new("p.turtle")), Some(Format::Turtle));
        assert_eq!(detect_format(Path::new("p.ofn")), Some(Format::Functional));
        assert_eq!(detect_format(Path::new("p.func")), Some(Format::Functional));
        assert_eq!(detect_format(Path::new("p.txt")), None);
        assert_eq!(detect_format(Path::new("noext")), None);
    }

    #[test]
    fn detect_format_from_bytes_owl_xml() {
        let header = br#"<?xml version="1.0"?><owl:Ontology/>"#;
        assert_eq!(detect_format_from_bytes(header), Some(Format::OwlXml));
    }

    #[test]
    fn detect_format_from_bytes_rdf_xml() {
        let header = br#"<?xml version="1.0"?><rdf:RDF/>"#;
        assert_eq!(detect_format_from_bytes(header), Some(Format::RdfXml));
    }

    #[test]
    fn plain_xml_extension_without_sniff_returns_none() {
        let dir = std::env::temp_dir().join("ontologos_parser_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("config.xml");
        {
            let mut file = std::fs::File::create(&path).expect("create");
            file.write_all(b"<config><item/></config>").expect("write");
        }
        assert_eq!(detect_format(&path), None);
        let _ = std::fs::remove_file(&path);
    }
}
