use std::io::Write;
use std::path::Path;

use ontologos_parser::{load_ontology, load_ontology_with_limits, Error, ParseLimits};

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn missing_file_returns_parse_error() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/missing.owl");
    let err = load_ontology(&path).expect_err("missing file");
    assert!(matches!(err, Error::Parse(_)));
    let message = err.to_string();
    assert!(
        message.contains("not a file") || message.contains("No such file"),
        "unexpected message: {message}"
    );
}

#[test]
fn unsupported_extension_returns_unsupported_format() {
    let path = std::env::temp_dir().join(format!(
        "ontologos_parser_load_errors_{}_{}.txt",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    {
        let mut file = std::fs::File::create(&path).expect("create temp file");
        writeln!(file, "not an ontology").expect("write");
    }

    let err = load_ontology(&path).expect_err("unsupported extension");
    assert!(matches!(err, Error::UnsupportedFormat(_)));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn file_size_limit_returns_parse_error() {
    let path = fixture("minimal_subclass.owl");
    let limits = ParseLimits {
        max_file_bytes: 1,
        ..ParseLimits::default()
    };
    let err = load_ontology_with_limits(&path, limits).expect_err("size limit");
    assert!(matches!(err, Error::Parse(_)));
    assert!(
        err.to_string().contains("exceeds limit"),
        "unexpected message: {err}"
    );
}

#[test]
fn legacy_galen_fixture_loads_after_entity_expansion() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/reasoner/res/galen-ians-full-undoctored.xml");
    assert!(path.exists(), "missing galen fixture at {}", path.display());

    let ontology = load_ontology(&path).expect("galen.xml should load after entity expansion");
    assert!(
        ontology.axiom_count() > 0,
        "expected mapped axioms from galen.xml"
    );
}

#[test]
fn legacy_propreo_fixture_loads_after_entity_expansion() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/reasoner/res/propreo.xml");
    assert!(
        path.exists(),
        "missing propreo fixture at {}",
        path.display()
    );

    let ontology = load_ontology(&path).expect("propreo.xml should load after entity expansion");
    assert!(
        ontology.axiom_count() > 0,
        "expected mapped axioms from propreo.xml"
    );
}

#[test]
fn legacy_wine_fixture_loads_after_duplicate_rdf_id_dedup() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/reasoner/res/wine.xml");
    assert!(
        path.exists(),
        "missing vendored wine.xml at {}",
        path.display()
    );

    let ontology = load_ontology(&path).expect("wine.xml should load after rdf:ID dedup");
    assert!(
        ontology.axiom_count() > 0,
        "expected mapped axioms from wine.xml"
    );
}

#[test]
fn parse_limits_merge_imports_defaults_true() {
    let limits = ParseLimits::default();
    assert!(limits.merge_imports);
}
