use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use horned_owl::curie::PrefixMapping;
use horned_owl::error::HornedError;
use horned_owl::io::ofn::reader as ofn_reader;
use horned_owl::io::owx::reader as owx_reader;
use horned_owl::io::rdf::reader as rdf_reader;
use horned_owl::io::{ParserConfiguration, RDFParserConfiguration};
use horned_owl::model::RcStr;
use horned_owl::ontology::set::SetOntology;
use oxrdfio::RdfFormat;

use crate::limits::ParseLimits;
use crate::{Error, Format, Result};

/// Read a horned-owl ontology from disk after format detection and size checks.
pub fn read_horned_owl(
    path: &Path,
    format: Format,
    limits: ParseLimits,
) -> Result<SetOntology<RcStr>> {
    let metadata = std::fs::metadata(path).map_err(|e| Error::Parse(e.to_string()))?;
    if metadata.len() as usize > limits.max_file_bytes {
        return Err(Error::Parse(format!(
            "file size {} exceeds limit of {} bytes",
            metadata.len(),
            limits.max_file_bytes
        )));
    }

    let file = File::open(path).map_err(|e| Error::Parse(e.to_string()))?;
    let config = parser_config(format);

    let (ontology, _prefixes) = match format {
        Format::OwlXml => {
            owx_reader::read(&mut BufReader::new(file), config).map_err(map_horned_error)?
        }
        Format::RdfXml | Format::Turtle => {
            let mut reader = BufReader::new(file);
            let (concrete, incomplete) =
                rdf_reader::read(&mut reader, config).map_err(map_horned_error)?;
            if !incomplete.is_complete() {
                return Err(Error::Parse(
                    "RDF parse incomplete: input truncated or malformed".into(),
                ));
            }
            (concrete.into(), PrefixMapping::default())
        }
        Format::Functional => {
            let mut reader = BufReader::new(file);
            ofn_reader::read(&mut reader, config).map_err(map_horned_error)?
        }
    };

    Ok(ontology)
}

fn parser_config(format: Format) -> ParserConfiguration {
    let rdf = match format {
        Format::Turtle => RDFParserConfiguration {
            format: Some(RdfFormat::Turtle),
            ..RDFParserConfiguration::default()
        },
        Format::RdfXml => RDFParserConfiguration {
            format: Some(RdfFormat::RdfXml),
            ..RDFParserConfiguration::default()
        },
        _ => RDFParserConfiguration::default(),
    };
    ParserConfiguration {
        rdf,
        ..ParserConfiguration::default()
    }
}

pub(crate) fn map_horned_error(err: HornedError) -> Error {
    Error::Parse(err.to_string())
}

/// Sniff the first bytes of a file for Turtle `@prefix` or `PREFIX` declarations.
pub fn detect_turtle_from_bytes(header: &[u8]) -> bool {
    let text = match std::str::from_utf8(header) {
        Ok(t) => strip_utf8_bom(t).trim_start(),
        Err(_) => return false,
    };
    text.starts_with("@prefix")
        || text.starts_with("@base")
        || text.to_ascii_lowercase().starts_with("prefix ")
        || text.contains("\n@prefix")
        || text.to_ascii_lowercase().contains("\nprefix ")
}

fn strip_utf8_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

/// Read up to `max` bytes from `path` for format sniffing.
pub fn sniff_file_header(path: &Path, max: usize) -> Result<Vec<u8>> {
    let mut file = File::open(path).map_err(|e| Error::Parse(e.to_string()))?;
    let mut header = vec![0_u8; max];
    let read = file
        .read(&mut header)
        .map_err(|e| Error::Parse(e.to_string()))?;
    header.truncate(read);
    Ok(header)
}
