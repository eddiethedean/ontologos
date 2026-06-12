//! Shared helpers for HermiT-ported conformance tests.
//!
//! HermiT source is expected at `HermiT/` in the repo root (gitignored) or at
//! `ONTOLOGOS_HERMIT_ROOT`.

use std::path::{Path, PathBuf};

/// Default namespace used by HermiT's `AbstractOntologyTest` (`file:/c/test.owl#`).
pub const HERMIT_DEFAULT_NS: &str = "file:/c/test.owl#";

/// OntoLogos-style namespace for inlined Tier-A ports.
pub const PORT_NS: &str = "http://example.org/hermit-port#";

/// Resolve the HermiT source tree, if present.
#[must_use]
pub fn hermit_root() -> Option<PathBuf> {
    if let Ok(env) = std::env::var("ONTOLOGOS_HERMIT_ROOT") {
        let path = PathBuf::from(env);
        if is_hermit_tree(&path) {
            return path.canonicalize().ok().or(Some(path));
        }
    }

    let default = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../HermiT");
    if is_hermit_tree(&default) {
        return default.canonicalize().ok().or(Some(default));
    }
    None
}

fn is_hermit_tree(path: &Path) -> bool {
    path.join("project/test/org/semanticweb/HermiT").is_dir()
}

/// Path under `HermiT/project/test/org/semanticweb/HermiT/`.
#[must_use]
pub fn hermit_test_path(relative: &str) -> Option<PathBuf> {
    hermit_root().map(|root| {
        root.join("project/test/org/semanticweb/HermiT")
            .join(relative)
    })
}

/// Returns true when optional Tier-B HermiT fixture tests should run.
#[must_use]
pub fn hermit_available() -> bool {
    hermit_root().is_some()
}

/// Assert `subclass` has a direct or indirect super-class `superclass` after RDFS materialization.
pub fn assert_subsumed(
    ontology: &ontologos_core::Ontology,
    subclass: &str,
    superclass: &str,
) -> bool {
    let Some(sub) = ontology.lookup_entity(subclass) else {
        return false;
    };
    let Some(sup) = ontology.lookup_entity(superclass) else {
        return false;
    };
    if ontology.direct_superclasses(sub).contains(&sup) {
        return true;
    }
    transitive_superclasses(ontology, sub, sup)
}

fn transitive_superclasses(
    ontology: &ontologos_core::Ontology,
    subclass: ontologos_core::EntityId,
    target: ontologos_core::EntityId,
) -> bool {
    let mut stack: Vec<ontologos_core::EntityId> = ontology.direct_superclasses(subclass).to_vec();
    let mut seen = std::collections::HashSet::new();
    while let Some(current) = stack.pop() {
        if !seen.insert(current) {
            continue;
        }
        if current == target {
            return true;
        }
        stack.extend_from_slice(ontology.direct_superclasses(current));
    }
    false
}
