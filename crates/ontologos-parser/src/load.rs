use std::path::{Component, Path, PathBuf};

use ontologos_core::Ontology;

use crate::{detect_format, Error, Result};

/// Resolve and validate a path before loading an ontology file.
pub fn validate_load_path(path: &Path, base: Option<&Path>) -> Result<PathBuf> {
    let normalized = normalize_path(path)?;

    if let Some(base) = base {
        let base_normalized = normalize_path(base)?;
        if !normalized.starts_with(&base_normalized) {
            return Err(Error::Parse(format!(
                "path {} escapes allowed base {}",
                normalized.display(),
                base_normalized.display()
            )));
        }
    }

    Ok(normalized)
}

/// Load an ontology from a validated file path.
pub fn load_ontology(path: &Path) -> Result<Ontology> {
    let validated = validate_load_path(path, None)?;
    let _format = detect_format(&validated);
    Ontology::from_file(&validated).map_err(Error::from)
}

fn normalize_path(path: &Path) -> Result<PathBuf> {
    let base = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir().map_err(|e| Error::Parse(e.to_string()))?
    };

    let mut normalized = base;
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(Error::Parse("path escapes beyond filesystem root".into()));
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }

    if normalized.exists() {
        normalized = normalized
            .canonicalize()
            .map_err(|e| Error::Parse(e.to_string()))?;
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn rejects_path_traversal_outside_base() {
        let base = std::env::current_dir().expect("cwd");
        let err = validate_load_path(Path::new("../../../etc/passwd"), Some(&base))
            .expect_err("traversal");
        assert!(matches!(err, Error::Parse(_)));
    }

    #[test]
    fn load_ontology_returns_parse_not_available() {
        let err = load_ontology(Path::new("test.owl")).expect_err("stub");
        assert!(matches!(
            err,
            Error::Core(ontologos_core::Error::ParseNotAvailable)
        ));
    }
}
