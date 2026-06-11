//! Load a benchmark ontology and print profile detection results.
//!
//! Run from repo root (after `./benchmarks/scripts/download.sh`):
//!
//! ```bash
//! cargo run -p ontologos-parser --example load_and_profile
//! ```

use std::path::PathBuf;

use ontologos_parser::load_ontology;
use ontologos_profile::detect_profile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let pizza = manifest_dir
        .join("../../benchmarks/data/pizza.owl")
        .canonicalize()
        .map_err(|_| {
            eprintln!("Run ./benchmarks/scripts/download.sh first");
            std::io::Error::other("missing pizza.owl")
        })?;

    let ontology = load_ontology(&pizza)?;
    println!("loaded: {}", pizza.display());
    println!("entities: {}", ontology.entity_count());
    println!("axioms (mapped): {}", ontology.axiom_count());

    if let Some(meta) = ontology.parse_meta() {
        println!(
            "parse: mapped={} skipped={} logical={}",
            meta.mapped_axiom_count, meta.skipped_axiom_count, meta.logical_axiom_count
        );
    }

    let report = detect_profile(&ontology)?;
    println!("detected profile: {:?}", report.detected);
    println!("diagnostics: {}", report.diagnostics.len());
    for diag in report.diagnostics.iter().take(5) {
        println!("  - {}: {}", diag.construct, diag.message);
    }
    if report.diagnostics.len() > 5 {
        println!("  ... and {} more", report.diagnostics.len() - 5);
    }

    Ok(())
}
