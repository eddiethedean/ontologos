//! Phase 8 expressivity exit gate — v1.5–v1.9 track smoke tests.

#[test]
fn phase8_alc_exit_tests_exist() {
    // Compiled via `cargo test -p ontologos-alc --test alc_exit`
    assert!(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../ontologos-alc/tests/alc_exit.rs")
            .is_file()
    );
}

#[test]
fn phase8_abox_family_exit_exists() {
    assert!(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../ontologos-abox/tests/family_exit.rs")
            .is_file()
    );
}

#[test]
fn phase8_ql_w3c_subset_exists() {
    assert!(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../ontologos-ql/tests/w3c_ql_subset.rs")
            .is_file()
    );
}

#[test]
fn phase8_alc_boundary_doc_exists() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/internal/design/alc-boundary.md");
    assert!(path.is_file(), "alc-boundary.md must exist");
}

#[test]
fn phase8_dl_dependency_index_compiles() {
    use ontologos_dl::DependencyIndex;
    let ont = ontologos_core::Ontology::new();
    let _index = DependencyIndex::from_ontology(&ont);
}
