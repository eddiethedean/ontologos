//! Tier-B HermiT fixture loads (parser smoke).

use ontologos_conformance::vendored_hermit_test_path;
use ontologos_parser::load_ontology_lenient as load_ontology;

/// HermiT OWLLink eval files often use `encoding='ISO-8859-1'`; horned-owl RDF/XML is UTF-8 only.
fn is_utf8_owl_fixture(path: &std::path::Path) -> bool {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return false;
    };
    !text.contains("ISO-8859-1") && !text.contains("iso-8859-1")
}

/// Survey vendored UTF-8 OWLLink fixtures; reports load/skip stats (parser gaps are non-fatal).
#[test]
fn hermit_owllink_owl_fixtures_load() {
    let dir =
        vendored_hermit_test_path("reasoner/res/OWLLink").expect("vendored OWLLink directory");
    let mut loaded = 0_usize;
    let mut latin1_skipped = 0_usize;
    let mut parse_failed: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("read OWLLink dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("owl") {
            continue;
        }
        if !is_utf8_owl_fixture(&path) {
            latin1_skipped += 1;
            continue;
        }
        match load_ontology(&path) {
            Ok(ontology) => {
                if ontology.entity_count() > 0 || ontology.axiom_count() > 0 {
                    loaded += 1;
                } else {
                    parse_failed.push(format!("{}: empty ontology", path.display()));
                }
            }
            Err(e) => parse_failed.push(format!("{}: {e}", path.display())),
        }
    }
    eprintln!(
        "HermiT OWLLink survey: loaded={loaded} latin1_skipped={latin1_skipped} parse_failed={}",
        parse_failed.len()
    );
    for fail in &parse_failed {
        eprintln!("  parse gap: {fail}");
    }
    assert!(
        loaded > 0,
        "expected at least one vendored UTF-8 OWLLink fixture to load in {}",
        dir.display()
    );
}

#[test]
fn hermit_owllink_bob_corpus_loads() {
    for name in [
        "agent.owl",
        "test.owl",
        "agent-inst.owl",
        "situation.owl",
        "situation-inst.owl",
        "space.owl",
        "time.owl",
    ] {
        let path = vendored_hermit_test_path(&format!("reasoner/res/OWLLink/{name}"))
            .unwrap_or_else(|| panic!("missing vendored Bob fixture {name}"));
        let ontology = load_ontology(&path).unwrap_or_else(|e| {
            panic!("load {name}: {e}");
        });
        assert!(
            ontology.entity_count() > 0 || ontology.axiom_count() > 0,
            "{name} should not be empty"
        );
    }
}

#[test]
fn hermit_families_owl_loads() {
    let path = vendored_hermit_test_path("reasoner/res/families.owl").expect("families.owl");
    let ontology = load_ontology(&path).expect("load families.owl");
    assert!(ontology.axiom_count() > 0);
}
