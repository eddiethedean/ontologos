use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn profile_requires_parser() {
    Command::cargo_bin("ontologos")
        .expect("ontologos binary")
        .args(["profile", "test.owl"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not available until v0.2"));
}
