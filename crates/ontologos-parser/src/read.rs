use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::panic::{catch_unwind, AssertUnwindSafe};
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
///
/// Prefer [`crate::load_ontology`] or [`crate::load_ontology_in`] for sandboxed loads.
#[allow(dead_code)]
pub fn read_horned_owl(
    path: &Path,
    format: Format,
    limits: ParseLimits,
) -> Result<SetOntology<RcStr>> {
    let metadata = std::fs::metadata(path).map_err(|e| Error::Parse(e.to_string()))?;
    check_file_size(metadata.len(), limits)?;
    let file = File::open(path).map_err(|e| Error::Parse(e.to_string()))?;
    read_horned_owl_from_reader(BufReader::new(file), format, limits)
}

/// Parse ontology bytes from an already-open reader (single-fd load path).
///
/// `limits` are enforced during axiom mapping in [`crate::map`]; horned-owl itself
/// may allocate before mapping caps apply (see `docs/security.md`).
pub fn read_horned_owl_from_reader<R: Read>(
    reader: R,
    format: Format,
    _limits: ParseLimits,
) -> Result<SetOntology<RcStr>> {
    let config = parser_config(format);

    let (ontology, _prefixes) = match format {
        Format::OwlXml => guard_horned_parse(|| {
            owx_reader::read(&mut BufReader::new(reader), config).map_err(map_horned_error)
        })?,
        Format::RdfXml | Format::Turtle => guard_horned_parse(|| {
            let mut reader = BufReader::new(reader);
            let (concrete, incomplete) =
                rdf_reader::read(&mut reader, config).map_err(map_horned_error)?;
            let _ = incomplete.is_complete();
            Ok((concrete.into(), PrefixMapping::default()))
        })?,
        Format::Functional => guard_horned_parse(|| {
            let mut reader = BufReader::new(reader);
            ofn_reader::read(&mut reader, config).map_err(map_horned_error)
        })?,
    };

    Ok(ontology)
}

/// Horned-owl may panic on some malformed RDF/XML; convert to [`Error::Parse`] for callers.
fn guard_horned_parse<T, F>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(payload) => Err(Error::Parse(panic_payload_message(payload))),
    }
}

fn panic_payload_message(payload: Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| format!("parser internal error: {s}"))
        .or_else(|| {
            payload
                .downcast_ref::<String>()
                .map(|s| format!("parser internal error: {s}"))
        })
        .unwrap_or_else(|| "parser internal error (unknown panic)".into())
}

fn check_file_size(len: u64, limits: ParseLimits) -> Result<()> {
    if len as usize > limits.max_file_bytes {
        return Err(Error::Parse(format!(
            "file size {len} exceeds limit of {} bytes",
            limits.max_file_bytes
        )));
    }
    Ok(())
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
    sniff_reader(&mut file, max)
}

/// Read up to `max` bytes from a reader for format sniffing.
pub fn sniff_reader(reader: &mut impl Read, max: usize) -> Result<Vec<u8>> {
    let mut header = vec![0_u8; max];
    let read = reader
        .read(&mut header)
        .map_err(|e| Error::Parse(e.to_string()))?;
    header.truncate(read);
    Ok(header)
}

/// Sniff from a seekable reader and rewind to the start.
pub fn sniff_and_rewind(reader: &mut (impl Read + Seek), max: usize) -> Result<Vec<u8>> {
    let header = sniff_reader(reader, max)?;
    reader
        .seek(SeekFrom::Start(0))
        .map_err(|e| Error::Parse(e.to_string()))?;
    Ok(header)
}
