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
    let dir = std::env::temp_dir().join("ontologos_parser_load_errors");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("ontology.txt");
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
fn legacy_wine_fixture_returns_parse_error_not_panic() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/reasoner/res/wine.xml");
    assert!(
        path.exists(),
        "missing vendored wine.xml at {}",
        path.display()
    );

    let err = load_ontology(&path).expect_err("wine.xml should fail to parse");
    assert!(matches!(err, Error::Parse(_)));
    assert!(
        err.to_string().contains("rdf:ID") || err.to_string().contains("parser internal error"),
        "unexpected message: {err}"
    );
}
