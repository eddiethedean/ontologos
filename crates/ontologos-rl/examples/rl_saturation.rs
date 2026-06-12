//! Load the Family benchmark ontology and run OWL RL saturation.
//!
//! Run from repo root (Family corpus is vendored):
//!
//! ```bash
//! cargo run -p ontologos-rl --example rl_saturation
//! ```

use std::path::PathBuf;

use ontologos_parser::load_ontology;
use ontologos_rl::RlEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let family = manifest_dir
        .join("../../benchmarks/data/family.owl")
        .canonicalize()
        .map_err(|_| {
            eprintln!("Expected benchmarks/data/family.owl in the repository");
            std::io::Error::other("missing family.owl")
        })?;

    let mut ontology = load_ontology(&family)?;
    let initial = ontology.axiom_count();

    if let Some(meta) = ontology.parse_meta() {
        println!(
            "loaded: {} (mapped={}, skipped={})",
            family.display(),
            meta.mapped_axiom_count,
            meta.skipped_axiom_count
        );
    } else {
        println!("loaded: {}", family.display());
    }

    let report = RlEngine::new(1).saturate(&mut ontology)?;

    println!("initial axioms: {initial}");
    println!("final axioms: {}", report.final_axiom_count);
    println!("inferred total: {}", report.inferred_total());
    println!("RDFS inferred: {}", report.rdfs_inferred);

    if report.inferred_by_rule.is_empty() {
        println!("RL rules fired: none");
    } else {
        println!("RL rules fired:");
        for (rule, count) in &report.inferred_by_rule {
            println!("  {}: {count}", rule.as_str());
        }
    }

    for clash in &report.clashes {
        println!("clash: {clash}");
    }

    Ok(())
}
