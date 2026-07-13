//! Shared helpers for contract / corpus fixture tests.

use std::path::{Path, PathBuf};

/// Resolve a benchmark data file relative to the repo root.
pub fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data")
        .join(name)
}

/// Require a vendored benchmark file — fail in CI instead of silently skipping.
pub fn require_data_file(name: &str) -> PathBuf {
    let path = data_path(name);
    assert!(
        path.is_file(),
        "missing benchmark fixture {} — run ./benchmarks/scripts/download.sh",
        path.display()
    );
    path
}

/// Require any existing path (used for parser fixtures).
#[allow(dead_code)]
pub fn require_file(path: &Path) -> &Path {
    assert!(
        path.is_file(),
        "missing required fixture at {}",
        path.display()
    );
    path
}
