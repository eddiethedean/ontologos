//! Triage planned HermiT Java and OWL WG catalog cases.
use std::env;
use std::path::PathBuf;

use ontologos_conformance::audit_planned_backlog;

fn main() {
    let audit = audit_planned_backlog();
    let json = serde_json::to_string_pretty(&audit).expect("serialize audit");
    println!("{json}");

    let out = env::var("PLANNED_BACKLOG_OUT").ok().map(PathBuf::from);
    if let Some(path) = out {
        let mut body = json;
        body.push('\n');
        std::fs::write(&path, body).expect("write PLANNED_BACKLOG_OUT");
        eprintln!("wrote {}", path.display());
    }

    let s = &audit.summary;
    eprintln!("planned backlog: java={} wg={}", s.java_total, s.wg_total);
    for (k, v) in &s.java_by_category {
        eprintln!("  java {k}: {v}");
    }
    for (k, v) in &s.wg_by_category {
        eprintln!("  wg {k}: {v}");
    }
}
