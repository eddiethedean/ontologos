use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Mutex;

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

/// Serializes low-level Horned-OWL reader entry points.
///
/// `horned-owl` parsers may panic or corrupt state when invoked concurrently; this mutex
/// pairs with the ontology-load lock for callers that read directly.
static HORNED_OWL_READ_LOCK: Mutex<()> = Mutex::new(());

/// Read a horned-owl ontology from disk after format detection and size checks.
///
/// Prefer [`crate::load_ontology`] or [`crate::load_ontology_in`] for sandboxed loads.
#[allow(dead_code)]
pub fn read_horned_owl(
    path: &Path,
    format: Format,
    limits: ParseLimits,
) -> Result<SetOntology<RcStr>> {
    let metadata = std::fs::metadata(path)?;
    check_file_size(metadata.len(), limits)?;
    let file = File::open(path)?;
    read_horned_owl_from_reader(BufReader::new(file), format, limits)
}

/// Parse ontology bytes from an already-open reader (single-fd load path).
///
/// `limits` are enforced during axiom mapping (`map_to_core`) and via a lightweight
/// pre-scan before horned-owl parsing (see `docs/security.md`).
pub fn read_horned_owl_from_reader<R: Read>(
    reader: R,
    format: Format,
    limits: ParseLimits,
) -> Result<SetOntology<RcStr>> {
    let bytes = read_bounded_bytes(reader, limits.max_file_bytes)?;
    prescan_axiom_estimate(&bytes, format, limits)?;
    read_horned_owl_from_bytes(&bytes, format)
}

fn read_bounded_bytes<R: Read>(reader: R, max_bytes: usize) -> Result<Vec<u8>> {
    let mut limited = reader.take(max_bytes.saturating_add(1) as u64);
    let mut bytes = Vec::new();
    limited
        .read_to_end(&mut bytes)
        .map_err(|e| Error::Parse(e.to_string()))?;
    if bytes.len() > max_bytes {
        return Err(Error::Parse(format!(
            "input size {} exceeds limit of {max_bytes} bytes",
            bytes.len()
        )));
    }
    Ok(bytes)
}

/// Lightweight axiom/component estimate before horned-owl allocates.
fn prescan_axiom_estimate(bytes: &[u8], format: Format, limits: ParseLimits) -> Result<()> {
    let estimate = match format {
        Format::Functional => count_ofn_axiom_markers(bytes),
        Format::OwlXml | Format::RdfXml => count_xml_axiom_markers(bytes),
        Format::Turtle => count_turtle_statement_markers(bytes),
    };
    if estimate > limits.max_axioms {
        return Err(Error::Parse(format!(
            "pre-scan estimate of {estimate} components exceeds axiom limit of {}",
            limits.max_axioms
        )));
    }
    Ok(())
}

fn count_ofn_axiom_markers(bytes: &[u8]) -> usize {
    const MARKERS: &[&str] = &[
        "SubClassOf(",
        "EquivalentClasses(",
        "DisjointClasses(",
        "ClassAssertion(",
        "ObjectPropertyAssertion(",
        "DataPropertyAssertion(",
        "Declaration(",
        "SubObjectPropertyOf(",
        "EquivalentObjectProperties(",
        "DisjointObjectProperties(",
        "SameIndividual(",
        "DifferentIndividuals(",
        "ObjectPropertyDomain(",
        "ObjectPropertyRange(",
        "NegativeObjectPropertyAssertion(",
        "NegativeDataPropertyAssertion(",
    ];
    let text = String::from_utf8_lossy(bytes);
    MARKERS
        .iter()
        .map(|marker| text.matches(marker).count())
        .sum()
}

fn count_xml_axiom_markers(bytes: &[u8]) -> usize {
    let text = String::from_utf8_lossy(bytes);
    const MARKERS: &[&str] = &[
        "owl:Class",
        "owl:ObjectProperty",
        "owl:DatatypeProperty",
        "owl:NamedIndividual",
        "owl:Restriction",
        "owl:equivalentClass",
        "owl:intersectionOf",
        "owl:unionOf",
        "owl:complementOf",
        "owl:disjointWith",
        "owl:someValuesFrom",
        "owl:allValuesFrom",
        "owl:hasValue",
        "owl:cardinality",
        "owl:minCardinality",
        "owl:maxCardinality",
        "rdf:type",
        "rdfs:subClassOf",
    ];
    MARKERS
        .iter()
        .map(|marker| text.matches(marker).count())
        .sum()
}

fn count_turtle_statement_markers(bytes: &[u8]) -> usize {
    let text = String::from_utf8_lossy(bytes);
    text.matches(" ;").count()
        + text.matches(" .").count()
        + text.matches('\n').map(|_| 1).sum::<usize>() / 4
}

fn read_horned_owl_from_bytes(bytes: &[u8], format: Format) -> Result<SetOntology<RcStr>> {
    let _guard = HORNED_OWL_READ_LOCK
        .lock()
        .map_err(|e| Error::Parse(format!("horned-owl read lock poisoned: {e}")))?;
    let config = parser_config(format);

    let (ontology, _prefixes) = match format {
        Format::OwlXml => guard_horned_parse(|| {
            owx_reader::read(&mut BufReader::new(Cursor::new(bytes)), config)
                .map_err(map_horned_error)
        })?,
        Format::RdfXml | Format::Turtle => guard_horned_parse(|| {
            let mut reader = BufReader::new(Cursor::new(bytes));
            let (concrete, incomplete) =
                rdf_reader::read(&mut reader, config).map_err(map_horned_error)?;
            if !incomplete.is_complete() {
                let mut tail = Vec::new();
                reader
                    .read_to_end(&mut tail)
                    .map_err(|e| Error::Parse(e.to_string()))?;
                if !tail.iter().all(|b| b.is_ascii_whitespace()) {
                    return Err(Error::Parse("incomplete RDF document".into()));
                }
            }
            Ok((concrete.into(), PrefixMapping::default()))
        })?,
        Format::Functional => guard_horned_parse(|| {
            ofn_reader::read(&mut BufReader::new(Cursor::new(bytes)), config)
                .map_err(map_horned_error)
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
    let mut file = File::open(path)?;
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
