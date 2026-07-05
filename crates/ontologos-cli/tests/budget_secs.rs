use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ontologos-parser/tests/fixtures")
        .join(name)
}

#[test]
fn classify_dl_budget_secs_zero_is_unlimited() {
    let path = fixture("minimal_subclass.owl");
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args([
            "--profile",
            "dl",
            "--budget-secs",
            "0",
            "classify",
            path.to_str().expect("path"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("status: classified"));
}
