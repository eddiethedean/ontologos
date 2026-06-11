use std::path::{Component, Path, PathBuf};

use ontologos_core::Ontology;

use crate::limits::ParseLimits;
use crate::map::map_to_core;
use crate::read::{read_horned_owl, sniff_file_header};
use crate::{
    detect_format, detect_format_from_bytes, detect_turtle_from_bytes, Error, Format, Result,
};

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
    load_ontology_with_limits(path, ParseLimits::default())
}

/// Load an ontology with custom [`ParseLimits`].
pub fn load_ontology_with_limits(path: &Path, limits: ParseLimits) -> Result<Ontology> {
    let validated = validate_load_path(path, None)?;
    if !validated.is_file() {
        return Err(Error::Parse(format!("not a file: {}", validated.display())));
    }

    let format = detect_format_with_sniff(&validated)?;
    let set_ontology = read_horned_owl(&validated, format, limits)?;
    let (mut ontology, report) = map_to_core(&set_ontology, limits)?;
    ontology.set_parse_meta(report.into_meta());
    Ok(ontology)
}

fn detect_format_with_sniff(path: &Path) -> Result<Format> {
    if let Some(format) = detect_format(path) {
        return Ok(format);
    }

    let header = sniff_file_header(path, 4096)?;
    if detect_format_from_bytes(&header).is_some() {
        return detect_format_from_bytes(&header).ok_or_else(|| {
            Error::UnsupportedFormat(format!("unrecognized XML in {}", path.display()))
        });
    }
    if detect_turtle_from_bytes(&header) {
        return Ok(Format::Turtle);
    }

    Err(Error::UnsupportedFormat(format!(
        "could not detect OWL/RDF format for {}",
        path.display()
    )))
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
}
