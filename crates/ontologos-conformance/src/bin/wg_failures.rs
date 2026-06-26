//! List active OWL WG catalog cases that fail semantic checks (for triage).
use std::collections::BTreeMap;

use ontologos_conformance::{ensure_concurrent_scan_defaults, scan_wg_failures, WgFailureBucket};

fn main() {
    ensure_concurrent_scan_defaults();
    let args: Vec<String> = std::env::args().collect();
    let json = args.iter().any(|a| a == "--json");
    let unpromoted_only = !args.iter().any(|a| a == "--all");

    let failures = scan_wg_failures(unpromoted_only);
    let mut by_bucket: BTreeMap<String, usize> = BTreeMap::new();
    for f in &failures {
        let key = format!("{:?}", f.bucket).to_lowercase();
        *by_bucket.entry(key).or_default() += 1;
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&failures).expect("json"));
        return;
    }

    let scope = if unpromoted_only {
        "unpromoted WG"
    } else {
        "active WG"
    };
    println!("{scope} failures: {}", failures.len());
    println!("by bucket:");
    for (bucket, count) in &by_bucket {
        println!("  {bucket}: {count}");
    }
    println!();
    for f in &failures {
        println!("{}", f.id);
        println!("  [{:?}] {}", f.bucket, f.detail);
    }
}

#[allow(dead_code)]
fn _bucket_names() {
    let _ = [
        WgFailureBucket::LoadError,
        WgFailureBucket::Timeout,
        WgFailureBucket::Consistency,
        WgFailureBucket::EntailmentPositive,
        WgFailureBucket::EntailmentNegative,
        WgFailureBucket::Other,
    ];
}
