use std::collections::HashSet;
use std::path::Path;

use ontologos_bridge::core_to_triples;
use ontologos_parser::load_ontology;
use ontologos_rl::RlEngine;
use oxrdf::{NamedOrBlankNode, Term, Triple};
use reasonable::reasoner::ReasonerBuilder;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const OWL_THING: &str = "http://www.w3.org/2002/07/owl#Thing";
const OWL_CLASS: &str = "http://www.w3.org/2002/07/owl#Class";

fn triple_key(triple: &Triple) -> String {
    let sub = match &triple.subject {
        NamedOrBlankNode::NamedNode(n) => n.as_str(),
        NamedOrBlankNode::BlankNode(n) => n.as_str(),
    };
    let obj = match &triple.object {
        Term::NamedNode(n) => n.as_str().to_string(),
        Term::BlankNode(n) => n.as_str().to_string(),
        Term::Literal(l) => l.to_string(),
    };
    format!("{sub}|{}|{obj}", triple.predicate.as_str())
}

/// Triples reasonable emits but ontologos-core does not store (Thing typing seeds).
fn mergeable_triple_key(triple: &Triple) -> Option<String> {
    let key = triple_key(triple);
    if key.is_empty() {
        return None;
    }
    if key.ends_with(&format!("|{RDF_TYPE}|{OWL_THING}")) {
        return None;
    }
    if key.starts_with(&format!("{OWL_THING}|{RDF_TYPE}|")) {
        return None;
    }
    if key.ends_with(&format!("|{RDF_TYPE}|{OWL_CLASS}"))
        && !key.starts_with("http://a.com/")
        && !key.starts_with("http://ex.org/")
    {
        // owl:Nothing and other seed class declarations
        if key.starts_with("http://www.w3.org/2002/07/owl#") {
            return None;
        }
    }
    Some(key)
}

fn reasonable_closure(triples: Vec<Triple>) -> HashSet<String> {
    let mut reasoner = ReasonerBuilder::new()
        .with_triples(triples)
        .build()
        .expect("reasonable");
    reasoner.reason_full();
    reasoner
        .view_output()
        .iter()
        .filter_map(mergeable_triple_key)
        .collect()
}

/// Adapter smoke test: brick subset saturates without error.
#[test]
fn brick_subset_saturates() {
    let path = repo_root().join("benchmarks/data/brick-subset.ttl");
    if !path.exists() {
        eprintln!("skip: missing {}", path.display());
        return;
    }
    let mut ontology = load_ontology(&path).expect("load brick subset");
    let initial = ontology.axiom_count();
    let report = RlEngine::new(1).saturate(&mut ontology).expect("saturate");
    assert!(report.final_axiom_count >= initial);
}

/// CI conformance gate: ontologos-rl closure matches reasonable on mapped triples.
#[test]
fn family_rl_closure_matches_reasonable() {
    let path = repo_root().join("benchmarks/data/family.owl");
    assert!(path.exists(), "missing {}", path.display());

    let mut ontology = load_ontology(&path).expect("load family");
    let input = core_to_triples(&ontology).expect("export");
    let expected = reasonable_closure(input);

    RlEngine::new(1).saturate(&mut ontology).expect("saturate");
    let actual: HashSet<_> = core_to_triples(&ontology)
        .expect("re-export")
        .iter()
        .filter_map(mergeable_triple_key)
        .collect();

    let missing: Vec<_> = expected.difference(&actual).take(10).collect();
    assert!(
        missing.is_empty(),
        "ontologos-rl missing {} reasonable triples (sample: {missing:?})",
        expected.difference(&actual).count()
    );
}
