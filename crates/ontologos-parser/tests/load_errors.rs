use std::io::Write;
use std::path::{Path, PathBuf};

use ontologos_parser::{
    Error, ParseLimits, load_ontology, load_ontology_lenient, load_ontology_with_limits,
};

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

    let ontology =
        load_ontology_lenient(&path).expect("galen.xml should load after entity expansion");
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

    let ontology =
        load_ontology_lenient(&path).expect("propreo.xml should load after entity expansion");
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

    let ontology = load_ontology_lenient(&path).expect("wine.xml should load after rdf:ID dedup");
    assert!(
        ontology.axiom_count() > 0,
        "expected mapped axioms from wine.xml"
    );
}

#[test]
fn owllink_primer_loads_with_families_import() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/reasoner/res/primer.owl");
    assert!(
        path.is_file(),
        "missing vendored primer.owl at {}",
        path.display()
    );

    let ontology =
        load_ontology_lenient(&path).expect("primer.owl should load with families import");
    assert!(
        ontology.axiom_count() > 0,
        "expected axioms from primer.owl"
    );
    assert!(
        ontology
            .lookup_entity("http://example.com/owl/families/hasParent")
            .is_some(),
        "expected hasParent from primer corpus"
    );
}

#[test]
fn parse_limits_merge_imports_defaults_false() {
    let limits = ParseLimits::default();
    assert!(!limits.merge_imports);
    assert!(limits.strict);
}

#[test]
fn parse_limits_lenient_disables_strict() {
    let limits = ParseLimits::lenient();
    assert!(!limits.strict);
}

#[test]
fn rejects_invalid_utf8_rdf_xml() {
    let path = std::env::temp_dir().join(format!(
        "ontologos_invalid_utf8_{}_{}.rdf",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    std::fs::write(&path, b"<rdf:RDF>\xff</rdf:RDF>").expect("write");
    let err = load_ontology(&path).expect_err("invalid utf8");
    assert!(matches!(err, Error::Parse(_)));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn rejects_incompatible_declarations_in_strict_mode() {
    let path = fixture("subclass_data_property_decl_first.ofn");
    let err = load_ontology(&path).expect_err("incompatible declarations");
    assert!(matches!(err, Error::Parse(_)));
}

#[test]
fn rejects_oversized_preprocess_budget() {
    let path = fixture("minimal_subclass.rdf");
    let limits = ParseLimits {
        max_preprocess_bytes: 32,
        ..ParseLimits::default()
    };
    let err = load_ontology_with_limits(&path, limits).expect_err("preprocess budget");
    assert!(matches!(err, Error::Parse(_)));
    assert!(
        err.to_string().contains("preprocessing allocation"),
        "unexpected: {err}"
    );
}
