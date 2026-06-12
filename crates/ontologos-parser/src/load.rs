use std::path::{Component, Path, PathBuf};

use ontologos_core::Ontology;

use crate::limits::ParseLimits;
use crate::map::map_to_core;
use crate::read::{read_horned_owl, sniff_file_header};
use crate::{
    detect_format, detect_format_from_bytes, detect_functional_from_bytes,
    detect_turtle_from_bytes, Error, Format, Result,
};

/// Resolve and validate a path before loading an ontology file.
pub fn validate_load_path(path: &Path, base: Option<&Path>) -> Result<PathBuf> {
    let normalized = normalize_path(path)?;

    if let Some(base) = base {
        let base_normalized = normalize_path(base)?;
        if !path_is_under_base(&normalized, &base_normalized) {
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

/// Load an ontology constrained to stay under `base` (untrusted uploads).
pub fn load_ontology_in(base: &Path, path: &Path) -> Result<Ontology> {
    load_ontology_with_limits_and_base(path, ParseLimits::default(), Some(base))
}

/// Load an ontology with custom [`ParseLimits`].
pub fn load_ontology_with_limits(path: &Path, limits: ParseLimits) -> Result<Ontology> {
    load_ontology_with_limits_and_base(path, limits, None)
}

/// Load an ontology with custom limits and optional sandbox base directory.
pub fn load_ontology_with_limits_and_base(
    path: &Path,
    limits: ParseLimits,
    base: Option<&Path>,
) -> Result<Ontology> {
    let validated = validate_load_path(path, base)?;
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
    if let Some(format) = detect_format_from_bytes(&header) {
        return Ok(format);
    }
    if detect_turtle_from_bytes(&header) {
        return Ok(Format::Turtle);
    }
    if detect_functional_from_bytes(&header) {
        return Ok(Format::Functional);
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

/// True when `path` is the same as or nested under `base` (path-component wise).
fn path_is_under_base(path: &Path, base: &Path) -> bool {
    let mut path_iter = path.components();
    for base_comp in base.components() {
        match path_iter.next() {
            Some(path_comp) if path_comp == base_comp => {}
            _ => return false,
        }
    }
    true
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
    fn rejects_path_prefix_bypass() {
        let parent = std::env::temp_dir();
        let base = parent.join("ontologos_uploads_base");
        let evil = parent.join("ontologos_uploads_base_evil");
        std::fs::create_dir_all(&base).expect("create base");
        std::fs::create_dir_all(&evil).expect("create evil sibling");
        let file = evil.join("secret.owl");
        std::fs::write(&file, b"<rdf:RDF/>").expect("write file");

        let err = validate_load_path(&file, Some(&base)).expect_err("prefix bypass");
        assert!(matches!(err, Error::Parse(_)));

        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_dir(&evil);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn path_is_under_base_accepts_nested_file() {
        let parent = std::env::temp_dir();
        let base = parent.join("ontologos_nested_base");
        let nested = base.join("nested");
        std::fs::create_dir_all(&nested).expect("create nested");
        let file = nested.join("ontology.owl");
        std::fs::write(&file, b"<rdf:RDF/>").expect("write file");

        let validated = validate_load_path(&file, Some(&base)).expect("nested file under base");
        assert!(path_is_under_base(
            &validated,
            &base.canonicalize().expect("canonicalize base")
        ));

        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_dir(&nested);
        let _ = std::fs::remove_dir(&base);
    }
}
