use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};

use ontologos_core::Ontology;

use crate::limits::ParseLimits;
use crate::map::map_to_core;
use crate::read::{read_horned_owl_from_reader, sniff_and_rewind};
use crate::{
    detect_format, detect_format_from_bytes, detect_functional_from_bytes,
    detect_turtle_from_bytes, Error, Format, Result,
};

#[cfg(target_os = "linux")]
const O_NOFOLLOW: i32 = 0o100_000;
#[cfg(target_os = "macos")]
const O_NOFOLLOW: i32 = 0x0000_0040;
#[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
const O_NOFOLLOW: i32 = 0;

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

    let mut file = open_for_load(&validated, base)?;
    let file_len = file
        .metadata()
        .map_err(|e| Error::Parse(e.to_string()))?
        .len();
    if file_len as usize > limits.max_file_bytes {
        return Err(Error::Parse(format!(
            "file size {file_len} exceeds limit of {} bytes",
            limits.max_file_bytes
        )));
    }
    let format = detect_format_with_sniff(path, &mut file)?;
    let set_ontology = read_horned_owl_from_reader(&mut file, format, limits)?;
    let (mut ontology, report) = map_to_core(&set_ontology, limits)?;
    ontology.set_parse_meta(report.into_meta());
    Ok(ontology)
}

fn open_for_load(path: &Path, base: Option<&Path>) -> Result<File> {
    let pre_meta = std::fs::symlink_metadata(path).map_err(|e| Error::Parse(e.to_string()))?;
    let file = open_readonly_nofollow(path)?;
    if let Some(base) = base {
        verify_opened_under_base(&file, base, path, &pre_meta)?;
    }
    Ok(file)
}

fn open_readonly_nofollow(path: &Path) -> Result<File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .read(true)
            .custom_flags(O_NOFOLLOW)
            .open(path)
            .map_err(|e| Error::Parse(e.to_string()))
    }
    #[cfg(not(unix))]
    {
        File::open(path).map_err(|e| Error::Parse(e.to_string()))
    }
}

fn verify_opened_under_base(
    file: &File,
    base: &Path,
    validated: &Path,
    pre_meta: &std::fs::Metadata,
) -> Result<()> {
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;

    let file_meta = file.metadata().map_err(|e| Error::Parse(e.to_string()))?;
    #[cfg(unix)]
    if pre_meta.dev() != file_meta.dev() || pre_meta.ino() != file_meta.ino() {
        return Err(Error::Parse(
            "ontology path changed between validation and open".into(),
        ));
    }
    #[cfg(not(unix))]
    let _ = (pre_meta, file_meta);

    let base_normalized = normalize_path(base)?;
    let base_canon = base_normalized
        .canonicalize()
        .map_err(|e| Error::Parse(e.to_string()))?;

    if let Ok(opened) = opened_path(file) {
        let opened_canon = opened
            .canonicalize()
            .map_err(|e| Error::Parse(e.to_string()))?;
        if !path_is_under_base(&opened_canon, &base_canon) {
            return Err(Error::Parse(format!(
                "opened file {} escapes allowed base {}",
                opened_canon.display(),
                base_canon.display()
            )));
        }
        return Ok(());
    }

    let validated_canon = validated
        .canonicalize()
        .map_err(|e| Error::Parse(e.to_string()))?;
    if !path_is_under_base(&validated_canon, &base_canon) {
        return Err(Error::Parse(format!(
            "path {} escapes allowed base {}",
            validated_canon.display(),
            base_canon.display()
        )));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn opened_path(file: &File) -> Result<PathBuf> {
    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();
    std::fs::read_link(format!("/proc/self/fd/{fd}")).map_err(|e| Error::Parse(e.to_string()))
}

#[cfg(target_os = "macos")]
fn opened_path(file: &File) -> Result<PathBuf> {
    use std::ffi::CStr;
    use std::os::unix::io::AsRawFd;

    const F_GETPATH: i32 = 50;
    let fd = file.as_raw_fd();
    let mut buf = [0u8; 1024];
    let rc = unsafe { libc::fcntl(fd, F_GETPATH, buf.as_mut_ptr()) };
    if rc == -1 {
        return Err(Error::Parse("fcntl(F_GETPATH) failed".into()));
    }
    let cstr = CStr::from_bytes_until_nul(&buf).map_err(|e| Error::Parse(e.to_string()))?;
    Ok(PathBuf::from(cstr.to_string_lossy().into_owned()))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn opened_path(_file: &File) -> Result<PathBuf> {
    Err(Error::Parse("fd path resolution unavailable".into()))
}

fn detect_format_with_sniff(path: &Path, reader: &mut (impl Read + Seek)) -> Result<Format> {
    if let Some(format) = detect_format(path) {
        reader
            .seek(SeekFrom::Start(0))
            .map_err(|e| Error::Parse(e.to_string()))?;
        return Ok(format);
    }

    let header = sniff_and_rewind(reader, 4096)?;
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

    #[cfg(unix)]
    #[test]
    fn sandboxed_load_does_not_follow_symlink_to_outside_file() {
        use std::os::unix::fs::symlink;

        let parent = std::env::temp_dir();
        let base = parent.join("ontologos_sandbox_base");
        let outside = parent.join("ontologos_outside_secret.owl");
        let link = base.join("ontology.owl");
        std::fs::create_dir_all(&base).expect("create base");
        std::fs::write(&outside, b"OUTSIDE_SECRET_CONTENT").expect("write outside");

        symlink(&outside, &link).expect("symlink");

        let err = load_ontology_in(&base, &link).expect_err("symlink escape");
        assert!(matches!(err, Error::Parse(_) | Error::UnsupportedFormat(_)));

        let _ = std::fs::remove_file(&link);
        let _ = std::fs::remove_file(&outside);
        let _ = std::fs::remove_dir(&base);
    }
}
