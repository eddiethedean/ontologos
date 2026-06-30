//! HermiT burndown dashboard — fast catalog metrics and optional targeted scans.
use std::collections::BTreeMap;

use ontologos_conformance::{
    audit_planned_backlog_with, ensure_concurrent_scan_defaults, parity_metrics,
    scan_planned_engine_failures, scan_wg_failures, AuditOptions, WgFailureBucket,
};

fn usage() -> &'static str {
    "usage: parity_status [--json] [--scan] [--scan-full] [--audit] [--audit-fast]\n\
     \n\
       (default)     catalog parity metrics only (<1s)\n\
       --scan        WG failures for unpromoted cases only (daily triage)\n\
       --scan-full   WG failures for all active cases\n\
       --audit-fast  planned-backlog metadata triage (no engine checks)\n\
       --audit       full planned-backlog audit (slow)\n\
       --json        machine-readable output"
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{}", usage());
        return;
    }

    let json = args.iter().any(|a| a == "--json");
    let scan_full = args.iter().any(|a| a == "--scan-full");
    let scan = scan_full || args.iter().any(|a| a == "--scan");
    let audit = args.iter().any(|a| a == "--audit");
    let audit_fast = args.iter().any(|a| a == "--audit-fast");

    let metrics = parity_metrics();

    if json && !scan && !audit && !audit_fast {
        println!("{}", serde_json::to_string_pretty(&metrics).expect("json"));
        return;
    }

    if !json {
        print_text_header(&metrics);
    }

    if audit_fast {
        let audit = audit_planned_backlog_with(AuditOptions {
            run_engine_checks: false,
        });
        if json {
            println!("{}", serde_json::to_string_pretty(&audit).expect("json"));
        } else {
            print_audit_summary("planned backlog (fast)", &audit.summary);
        }
    } else if audit {
        ensure_concurrent_scan_defaults();
        let audit = audit_planned_backlog_with(AuditOptions {
            run_engine_checks: true,
        });
        if json {
            println!("{}", serde_json::to_string_pretty(&audit).expect("json"));
        } else {
            print_audit_summary("planned backlog (full)", &audit.summary);
            let engine_failures = scan_planned_engine_failures();
            println!("planned engine failures: {}", engine_failures.len());
            for (id, err) in engine_failures.iter().take(10) {
                println!("  {id}: {}", truncate(err, 100));
            }
            if engine_failures.len() > 10 {
                println!("  … and {} more", engine_failures.len() - 10);
            }
        }
    }

    if scan {
        ensure_concurrent_scan_defaults();
        let unpromoted_only = !scan_full;
        let failures = scan_wg_failures(unpromoted_only);
        if json {
            println!("{}", serde_json::to_string_pretty(&failures).expect("json"));
        } else {
            let scope = if unpromoted_only {
                "unpromoted WG"
            } else {
                "all active WG"
            };
            println!();
            println!("{scope} failures: {}", failures.len());
            let mut by_bucket: BTreeMap<String, usize> = BTreeMap::new();
            for f in &failures {
                let key = format!("{:?}", f.bucket).to_lowercase();
                *by_bucket.entry(key).or_default() += 1;
            }
            for (bucket, count) in &by_bucket {
                println!("  {bucket}: {count}");
            }
            for f in failures.iter().take(15) {
                println!("  {} [{:?}] {}", f.id, f.bucket, truncate(&f.detail, 80));
            }
            if failures.len() > 15 {
                println!(
                    "  … and {} more (use wg_failures for full list)",
                    failures.len() - 15
                );
            }
        }
    }

    if json && (audit || audit_fast || scan) {
        return;
    }

    if !scan && !audit && !audit_fast {
        print_next_steps(&metrics);
    }
}

fn print_text_header(metrics: &ontologos_conformance::ParityMetrics) {
    println!("HermiT burndown status");
    println!("  parity_pct:      {:.1}%", metrics.parity_pct);
    println!("  in_scope_total:  {}", metrics.in_scope_total);
    println!(
        "  backlog:         {} (java {} + wg {})",
        metrics.backlog, metrics.java_planned, metrics.wg_planned
    );
    println!(
        "  promoted:        axiom {} / wg {} of {} active",
        metrics.promoted_axiom, metrics.promoted_wg, metrics.active_wg
    );
    if metrics.unpromoted_wg > 0 {
        println!(
            "  unpromoted WG:   {} cases to burn down",
            metrics.unpromoted_wg
        );
    }
    println!("  runnable Java:   {}", metrics.runnable_java);
    println!(
        "  literal catalog: {:.1}% harness ({}/{} tests active, {} ignored)",
        metrics.literal_catalog_pct,
        metrics.conformance_active,
        metrics.literal_catalog_total,
        metrics.conformance_ignored
    );
    println!(
        "  literal green:   {:.1}% (catalog status + ADR-waived covered/excluded)",
        metrics.literal_green_pct
    );
    if metrics.java_out_of_scope > 0 {
        println!(
            "  java out-scope:  {} (excluded/internal/migrated/covered)",
            metrics.java_out_of_scope
        );
    }
    println!(
        "  taxonomy strict: {:.1}% (Tier C HermiT --max-extra 0)",
        metrics.taxonomy_strict_pct
    );
    println!(
        "  perf gate:       {:.1}% (ROADMAP DL targets)",
        metrics.perf_gate_pct
    );
    println!(
        "  internal ports:  {:.1}% (tableau/graph → alc unit tests)",
        metrics.internal_port_pct
    );
    println!(
        "  rules test:      {:.1}% (RulesTest swrl active / catalog)",
        metrics.rules_test_pct
    );
    println!("  activatable #[ignore]: {}", metrics.activatable_ignored);
    println!(
        "  true parity:     {:.1}% (min of sub-metrics; target 100%)",
        metrics.true_parity_pct
    );
}

fn print_audit_summary(title: &str, summary: &ontologos_conformance::PlannedBacklogSummary) {
    println!();
    println!("{title}");
    println!("  java planned: {}", summary.java_total);
    for (k, v) in &summary.java_by_category {
        println!("    {k}: {v}");
    }
    if summary.wg_total > 0 {
        println!("  wg planned: {}", summary.wg_total);
        for (k, v) in &summary.wg_by_category {
            println!("    {k}: {v}");
        }
    }
}

fn print_next_steps(metrics: &ontologos_conformance::ParityMetrics) {
    println!();
    println!("Next steps:");
    if metrics.unpromoted_wg > 0 {
        println!(
            "  parity_status --scan          # triage {} unpromoted WG failures",
            metrics.unpromoted_wg
        );
        println!("  bash benchmarks/scripts/hermit-burndown.sh promote");
    }
    if metrics.java_planned > 0 {
        println!(
            "  parity_status --audit-fast    # classify {} Java planned cases",
            metrics.java_planned
        );
    }
    if metrics.activatable_ignored > 0 {
        println!(
            "  cargo test -p ontologos-conformance -- --ignored   # {} activatable B4 tests",
            metrics.activatable_ignored
        );
        println!("  bash benchmarks/scripts/report-ignored-buckets.sh");
    }
    if metrics.backlog == 0 && metrics.unpromoted_wg == 0 {
        println!("  catalog parity 100% — run check-hermit-parity-phases.sh");
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

#[allow(dead_code)]
fn _buckets() {
    let _ = [
        WgFailureBucket::LoadError,
        WgFailureBucket::Timeout,
        WgFailureBucket::Consistency,
        WgFailureBucket::EntailmentPositive,
        WgFailureBucket::EntailmentNegative,
        WgFailureBucket::Other,
    ];
}
