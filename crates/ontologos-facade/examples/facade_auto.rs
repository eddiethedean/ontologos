//! Load family.owl and classify with Profile::Auto — mirrors the getting-started guide.
//!
//! Run from repo root (after `curl` or with vendored family.owl):
//!   cargo run -p ontologos-facade --example facade_auto -- benchmarks/data/family.owl

use ontologos_core::{Profile, Reasoner};
use ontologos_facade::ClassifyOutcome;
use ontologos_parser::load_ontology;
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .map(|p| Path::new(&p).to_path_buf())
        .unwrap_or_else(|| Path::new("benchmarks/data/family.owl").to_path_buf());

    let ontology = load_ontology(&path)?;
    let mut reasoner = Reasoner::builder().profile(Profile::Auto).build(ontology)?;

    match ontologos_facade::classify(&mut reasoner)? {
        ClassifyOutcome::Taxonomy(t) => {
            println!("taxonomy: {} subsumptions", t.subsumption_count());
        }
        ClassifyOutcome::Rdfs(r) => {
            println!("RDFS: {} inferred axioms", r.inferred_total());
        }
        ClassifyOutcome::Rl(r) => {
            println!("RL: {} inferred axioms", r.inferred_total());
        }
        _ => println!("unexpected classify outcome variant"),
    }
    Ok(())
}
