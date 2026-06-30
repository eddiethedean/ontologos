//! Thread-safe CE satisfiability probes against HermiT OFN TBoxes.

use ontologos_alc::{DlOntology, TableauSeed};
use ontologos_core::DlAxiom;
use ontologos_parser::load_ontology;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static PROBE_SEQ: AtomicU64 = AtomicU64::new(0);

fn ax(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms")
        .join(name)
}

/// Whether `ce_ofn` is satisfiable under the TBox in `ofn` (empty ABox + one class assertion).
///
/// Each call uses a unique temp file and ontology IRI so parallel `cargo test` runs do not race.
pub fn ce_sat(ofn: &str, ce_ofn: &str) -> bool {
    let seq = PROBE_SEQ.fetch_add(1, Ordering::Relaxed);
    // HermiT OFN TBoxes use `file:/c/test.owl#`; probe entities must share that IRI namespace.
    let ontology_iri = "file:/c/test.owl#";
    let assertion = format!("ClassAssertion({ce_ofn} :__probe__)");
    let body = format!(
        "Prefix(:=<{ontology_iri}>)\n\
         Prefix(a:=<{ontology_iri}>)\n\
         Prefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)\n\
         Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
         Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\n\
         Ontology(<{ontology_iri}>\n{assertion}\n)"
    );
    let temp = std::env::temp_dir().join(format!("ian-ce-probe-{seq}.ofn"));
    std::fs::write(&temp, &body).unwrap();
    let probe = load_ontology(&temp).unwrap();
    let base = load_ontology(&ax(ofn)).unwrap();
    let mut merged = base.clone();
    for (_, record) in probe.entities().iter() {
        let Ok(iri) = probe.resolve_iri(record.iri) else {
            continue;
        };
        if merged.lookup_entity(iri).is_none() {
            let _ = merged.entity_id(iri, record.kind);
        }
    }
    let entity_map: HashMap<_, _> = probe
        .entities()
        .iter()
        .filter_map(|(id, record)| {
            let iri = probe.resolve_iri(record.iri).ok()?;
            Some((id, merged.lookup_entity(iri)?))
        })
        .collect();
    merged
        .dl_mut()
        .import_axioms_from(probe.dl(), |id| entity_map.get(&id).copied().unwrap());
    let dl = DlOntology::from_ontology(&merged).unwrap();
    let ce_id = merged
        .dl()
        .axioms()
        .filter_map(|axiom| {
            let DlAxiom::ClassAssertion { class, .. } = axiom else {
                return None;
            };
            Some(*class)
        })
        .last()
        .unwrap();
    let sat =
        ontologos_alc::is_ce_satisfiable_with_seed(&dl, ce_id, &TableauSeed::default()).unwrap();
    let _ = std::fs::remove_file(&temp);
    sat
}
