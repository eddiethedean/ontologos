use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ontologos-parser/tests/fixtures")
        .join(name)
}

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn profile_succeeds_on_minimal_fixture() {
    let path = fixture("minimal_subclass.owl");
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["profile", path.to_str().expect("path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("detected profile: El"));
}

#[test]
fn profile_json_includes_detected_field() {
    let path = fixture("minimal_subclass.owl");
    let output = Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["--format", "json", "profile", path.to_str().expect("path")])
        .assert()
        .success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).expect("utf8");
    assert!(stdout.contains("\"detected\":\"EL\"") || stdout.contains("\"detected\": \"EL\""));
}

#[test]
fn profile_pizza_corpus() {
    let path = repo_root().join("benchmarks/data/pizza.owl");
    assert!(
        path.exists(),
        "missing pizza corpus at {} (run ./benchmarks/scripts/download.sh)",
        path.display()
    );
    let output = Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["profile", path.to_str().expect("path")])
        .assert()
        .success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).expect("utf8");
    assert!(
        stdout.contains("detected profile: Dl"),
        "pizza profile should be Dl (mapped EL+RL mix): {stdout}"
    );
    assert!(stdout.contains("diagnostics:"));
    assert!(
        !stdout.contains("diagnostics: none"),
        "pizza classified as Dl should report mapped-construct diagnostics: {stdout}"
    );
}

#[test]
fn profile_family_corpus() {
    let path = repo_root().join("benchmarks/data/family.owl");
    assert!(path.exists(), "missing family corpus at {}", path.display());
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["profile", path.to_str().expect("path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("detected profile: Rl"));
}

#[test]
fn profile_missing_file_fails() {
    let path = fixture("missing.owl");
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["profile", path.to_str().expect("path")])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

#[test]
fn help_succeeds() {
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("profile"));
}

#[test]
fn classify_el_pizza_corpus() {
    let path = repo_root().join("benchmarks/data/pizza.owl");
    assert!(
        path.exists(),
        "missing pizza corpus at {} (run ./benchmarks/scripts/download.sh)",
        path.display()
    );
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["--profile", "el", "classify", path.to_str().expect("path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("subsumption_count:"));
}

#[test]
fn classify_rdfs_materializes_family_corpus() {
    let path = repo_root().join("benchmarks/data/family.owl");
    assert!(path.exists(), "missing family corpus at {}", path.display());
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args([
            "--profile",
            "rdfs",
            "classify",
            path.to_str().expect("path"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("status: classified"))
        .stdout(predicate::str::contains("inferred_axioms:"));
}

#[test]
fn materialize_family_corpus_succeeds() {
    let path = repo_root().join("benchmarks/data/family.owl");
    assert!(path.exists(), "missing family corpus at {}", path.display());
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["materialize", path.to_str().expect("path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("status: materialized"))
        .stdout(predicate::str::contains("inferred_axioms:"));
}

#[test]
fn classify_json_includes_report_fields() {
    let path = repo_root().join("benchmarks/data/family.owl");
    assert!(path.exists(), "missing family corpus at {}", path.display());
    let output = Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["--format", "json", "classify", path.to_str().expect("path")])
        .assert()
        .success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).expect("utf8");
    assert!(stdout.contains("\"status\""));
    assert!(stdout.contains("\"classified\"") || stdout.contains("classified"));
    assert!(stdout.contains("initial_axiom_count"));
    assert!(stdout.contains("final_axiom_count"));
    assert!(stdout.contains("inferred_by_rule"));
    assert!(stdout.contains("inferred_axioms"));
}

#[test]
fn materialize_json_includes_report_fields() {
    let path = repo_root().join("benchmarks/data/family.owl");
    assert!(path.exists(), "missing family corpus at {}", path.display());
    let output = Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args([
            "--format",
            "json",
            "materialize",
            path.to_str().expect("path"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).expect("utf8");
    assert!(stdout.contains("\"status\""));
    assert!(stdout.contains("initial_axiom_count"));
    assert!(stdout.contains("final_axiom_count"));
    assert!(stdout.contains("inferred_by_rule"));
    assert!(stdout.contains("inferred_axioms"));
}

#[test]
fn materialize_minimal_fixture_succeeds() {
    let path = fixture("minimal_subclass.owl");
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["materialize", path.to_str().expect("path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("status: materialized"));
}

#[test]
fn explain_stub_exits_not_implemented() {
    let path = repo_root().join("benchmarks/data/family.owl");
    assert!(path.exists(), "missing family corpus at {}", path.display());
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["explain", path.to_str().expect("path")])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "explanation generation not yet implemented",
        ));
}

#[test]
fn profile_surfaces_parse_meta_warnings_on_stderr() {
    let path = fixture("class_individual_kind_clash.ttl");
    let output = Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["profile", path.to_str().expect("path")])
        .assert()
        .success();
    let stderr = String::from_utf8(output.get_output().stderr.clone()).expect("utf8");
    assert!(
        stderr.contains("warning: parse skipped 1 of 1 logical axioms"),
        "expected parse summary on stderr, got: {stderr}"
    );
    assert!(
        stderr.contains("entity kind mismatch"),
        "expected kind mismatch warning on stderr, got: {stderr}"
    );
}

#[test]
fn profile_json_includes_parse_meta() {
    let path = fixture("class_individual_kind_clash.ttl");
    let output = Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["--format", "json", "profile", path.to_str().expect("path")])
        .assert()
        .success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).expect("utf8");
    assert!(stdout.contains("\"parse_meta\""));
    assert!(stdout.contains("\"skipped_axiom_count\""));
    assert!(stdout.contains("\"warnings\""));
    assert!(stdout.contains("entity kind mismatch"));
}

#[test]
fn classify_json_includes_parse_meta() {
    let path = fixture("class_individual_kind_clash.ttl");
    let output = Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args([
            "--format",
            "json",
            "--profile",
            "rdfs",
            "classify",
            path.to_str().expect("path"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).expect("utf8");
    assert!(stdout.contains("\"parse_meta\""));
    assert!(stdout.contains("\"skipped_axiom_count\": 1"));
}

#[test]
fn materialize_surfaces_parse_meta_warnings_on_stderr() {
    let path = fixture("class_individual_kind_clash.ttl");
    let output = Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["materialize", path.to_str().expect("path")])
        .assert()
        .success();
    let stderr = String::from_utf8(output.get_output().stderr.clone()).expect("utf8");
    assert!(
        stderr.contains("warning: parse skipped 1 of 1 logical axioms"),
        "expected parse summary on stderr, got: {stderr}"
    );
    assert!(
        stderr.contains("entity kind mismatch"),
        "expected kind mismatch warning on stderr, got: {stderr}"
    );
}

#[test]
fn materialize_json_includes_parse_meta() {
    let path = fixture("class_individual_kind_clash.ttl");
    let output = Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args([
            "--format",
            "json",
            "materialize",
            path.to_str().expect("path"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).expect("utf8");
    assert!(stdout.contains("\"parse_meta\""));
    assert!(stdout.contains("\"skipped_axiom_count\": 1"));
}

#[test]
fn profile_json_omits_parse_meta_for_clean_fixture() {
    let path = fixture("minimal_subclass.owl");
    let output = Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["--format", "json", "profile", path.to_str().expect("path")])
        .assert()
        .success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).expect("utf8");
    assert!(
        !stdout.contains("\"parse_meta\""),
        "clean fixture should omit parse_meta from JSON: {stdout}"
    );
}

#[test]
fn explain_surfaces_parse_meta_warnings_on_stderr_before_failure() {
    let path = fixture("class_individual_kind_clash.ttl");
    let output = Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["explain", path.to_str().expect("path")])
        .assert()
        .failure();
    let stderr = String::from_utf8(output.get_output().stderr.clone()).expect("utf8");
    assert!(
        stderr.contains("warning: parse skipped 1 of 1 logical axioms"),
        "expected parse summary on stderr, got: {stderr}"
    );
    assert!(
        stderr.contains("entity kind mismatch"),
        "expected kind mismatch warning on stderr, got: {stderr}"
    );
    assert!(
        stderr.contains("explanation generation not yet implemented"),
        "expected explain stub error after parse warnings, got: {stderr}"
    );
}
