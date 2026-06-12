//! Tier-B HermiT fixture loads (parser smoke). Skipped when `HermiT/` is absent.

use ontologos_conformance::hermit_test_path;
use ontologos_parser::load_ontology;

fn require_hermit() -> std::path::PathBuf {
    ontologos_conformance::hermit_root().expect(
        "HermiT source not found — clone to HermiT/ or set ONTOLOGOS_HERMIT_ROOT (see tests/hermit/README.md)",
    )
}

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

/// Survey UTF-8 OWLLink fixtures; reports load/skip stats (parser gaps are non-fatal).
#[test]
#[ignore = "requires local HermiT/ checkout"]
fn hermit_owllink_owl_fixtures_load() {
    let _root = require_hermit();
    let dir = hermit_test_path("reasoner/res/OWLLink").expect("owllink dir");
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
        loaded > 0 || latin1_skipped > 0,
        "no OWLLink fixtures found in {}",
        dir.display()
    );
}

#[test]
#[ignore = "requires local HermiT/ checkout"]
fn hermit_families_owl_loads() {
    let _root = require_hermit();
    let path = hermit_test_path("reasoner/res/families.owl").expect("families.owl");
    let ontology = load_ontology(&path).expect("load families.owl");
    assert!(ontology.axiom_count() > 0);
}
