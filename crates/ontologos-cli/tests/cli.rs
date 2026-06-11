use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ontologos-parser/tests/fixtures")
        .join(name)
}

#[test]
fn profile_succeeds_on_minimal_fixture() {
    let path = fixture("minimal_subclass.owl");
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["profile", path.to_str().expect("path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("detected profile:"));
}

#[test]
fn profile_json_includes_detected_field() {
    let path = fixture("minimal_subclass.owl");
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["--format", "json", "profile", path.to_str().expect("path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"detected\""));
}

#[test]
fn profile_pizza_when_data_present() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/pizza.owl");
    if !path.exists() {
        return;
    }
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["profile", path.to_str().expect("path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("detected profile: El"));
}

#[test]
fn profile_family_when_data_present() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/family.owl");
    if !path.exists() {
        return;
    }
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["profile", path.to_str().expect("path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("detected profile: Rl"));
}
