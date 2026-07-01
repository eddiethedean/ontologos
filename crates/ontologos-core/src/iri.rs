use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::limits::Limits;

/// Stable identifier for an interned IRI string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IriId(NonZeroU32);

impl IriId {
    /// Construct an `IriId` from a one-based index.
    ///
    /// # Panics
    ///
    /// Panics if `index` is zero. Prefer [`try_from_index`](Self::try_from_index) on untrusted input.
    #[must_use]
    pub fn from_index(index: u32) -> Self {
        Self(NonZeroU32::new(index).expect("IriId index must be non-zero"))
    }

    /// Fallible constructor for untrusted indices.
    pub fn try_from_index(index: u32) -> Result<Self> {
        NonZeroU32::new(index)
            .map(Self)
            .ok_or_else(|| Error::InvalidIri("IriId index must be non-zero".into()))
    }

    /// One-based index into the intern pool.
    #[must_use]
    pub fn index(self) -> u32 {
        self.0.get()
    }
}

fn hash_iri(s: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Deduplicating pool of absolute IRI strings.
///
/// Each IRI is stored once in `strings`; `index` maps hash buckets to string indices
/// (collision chains compare full string equality).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InternPool {
    strings: Vec<Box<str>>,
    index: HashMap<u64, Vec<usize>>,
}

impl InternPool {
    /// Create an empty intern pool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of unique IRIs in the pool.
    #[must_use]
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Returns `true` if the pool contains no IRIs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Look up an already-interned IRI without inserting.
    #[must_use]
    pub fn get(&self, iri: &str) -> Option<IriId> {
        self.lookup(iri)
            .map(|idx| IriId::from_index((idx + 1) as u32))
    }

    fn lookup(&self, iri: &str) -> Option<usize> {
        let bucket = self.index.get(&hash_iri(iri))?;
        bucket
            .iter()
            .copied()
            .find(|&idx| self.strings[idx].as_ref() == iri)
    }

    /// Intern an absolute IRI, returning an existing id if already present.
    pub fn intern(&mut self, iri: &str) -> Result<IriId> {
        self.intern_with_limit(iri, Limits::default().max_iri_len)
    }

    /// Intern an IRI with a custom maximum length.
    pub fn intern_with_limit(&mut self, iri: &str, max_len: usize) -> Result<IriId> {
        validate_iri_with_max_len(iri, max_len)?;

        if let Some(idx) = self.lookup(iri) {
            return Ok(IriId::from_index((idx + 1) as u32));
        }

        let index = u32::try_from(self.strings.len())
            .map_err(|_| Error::InvalidIri("IRI pool capacity exceeded".into()))?
            + 1;
        let id = IriId(NonZeroU32::new(index).expect("index is non-zero"));
        let owned: Box<str> = iri.into();
        let idx = self.strings.len();
        self.strings.push(owned);
        self.index.entry(hash_iri(iri)).or_default().push(idx);
        Ok(id)
    }

    /// Resolve an interned IRI to its string value.
    pub fn resolve(&self, id: IriId) -> Result<&str> {
        let idx = usize::try_from(id.index())
            .map_err(|_| Error::InvalidIri(format!("invalid IriId index: {}", id.index())))?
            - 1;
        self.strings
            .get(idx)
            .map(AsRef::as_ref)
            .ok_or_else(|| Error::InvalidIri(format!("unknown IriId: {}", id.index())))
    }

    /// Iterate over all interned IRI strings in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.strings.iter().map(AsRef::as_ref)
    }
}

const ALLOWED_SCHEMES: &[&str] = &["http", "https", "urn", "file", "internal"];
const SNAPSHOT_ALLOWED_SCHEMES: &[&str] = &["http", "https", "urn"];

/// Validate that `iri` is an absolute IRI with an allowed scheme.
pub fn validate_iri(iri: &str) -> Result<()> {
    validate_iri_with_max_len(iri, Limits::default().max_iri_len)
}

/// Validate an IRI with a custom maximum length.
pub fn validate_iri_with_max_len(iri: &str, max_len: usize) -> Result<()> {
    validate_iri_schemes(iri, max_len, ALLOWED_SCHEMES)
}

/// Validate an IRI from an untrusted JSON snapshot (`http`, `https`, `urn` only).
pub fn validate_snapshot_iri(iri: &str) -> Result<()> {
    validate_snapshot_iri_with_max_len(iri, Limits::default().max_iri_len)
}

/// Validate a snapshot IRI with a custom maximum length.
pub fn validate_snapshot_iri_with_max_len(iri: &str, max_len: usize) -> Result<()> {
    validate_iri_schemes(iri, max_len, SNAPSHOT_ALLOWED_SCHEMES)
}

fn validate_iri_schemes(iri: &str, max_len: usize, allowed_schemes: &[&str]) -> Result<()> {
    if iri.is_empty() {
        return Err(Error::InvalidIri("IRI must not be empty".into()));
    }

    if iri.len() > max_len {
        return Err(Error::InvalidIri(format!(
            "IRI exceeds maximum length of {max_len} bytes"
        )));
    }

    if iri.chars().any(|c| c.is_ascii_control()) {
        return Err(Error::InvalidIri(format!(
            "IRI contains control characters: {iri}"
        )));
    }

    if iri.contains(' ') {
        return Err(Error::InvalidIri(format!("IRI contains whitespace: {iri}")));
    }

    let Some((scheme, rest)) = iri.split_once(':') else {
        return Err(Error::InvalidIri(format!(
            "IRI must be absolute (scheme:...): {iri}"
        )));
    };

    if scheme.is_empty() || rest.is_empty() {
        return Err(Error::InvalidIri(format!("invalid IRI scheme: {iri}")));
    }

    if !scheme
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
    {
        return Err(Error::InvalidIri(format!("invalid IRI scheme: {iri}")));
    }

    if !allowed_schemes.contains(&scheme) {
        return Err(Error::InvalidIri(format!(
            "IRI scheme '{scheme}' is not allowed (allowed: {})",
            allowed_schemes.join(", ")
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_deduplicates() {
        let mut pool = InternPool::new();
        let a = pool.intern("http://example.org/A").expect("intern");
        let b = pool.intern("http://example.org/A").expect("intern");
        let c = pool.intern("http://example.org/B").expect("intern");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn resolve_round_trip() {
        let mut pool = InternPool::new();
        let id = pool.intern("http://example.org/Test").expect("intern");
        assert_eq!(
            pool.resolve(id).expect("resolve"),
            "http://example.org/Test"
        );
    }

    #[test]
    fn rejects_relative_iri() {
        let mut pool = InternPool::new();
        assert!(pool.intern("relative/path").is_err());
    }

    #[test]
    fn accepts_urn() {
        let mut pool = InternPool::new();
        let id = pool.intern("urn:example:animal").expect("intern");
        assert_eq!(pool.resolve(id).expect("resolve"), "urn:example:animal");
    }

    #[test]
    fn rejects_empty_iri() {
        assert!(validate_iri("").is_err());
    }

    #[test]
    fn rejects_whitespace_iri() {
        assert!(validate_iri("http://example.org/a b").is_err());
    }

    #[test]
    fn rejects_javascript_scheme() {
        assert!(validate_iri("javascript:alert(1)").is_err());
    }

    #[test]
    fn snapshot_iri_rejects_file_scheme() {
        assert!(validate_snapshot_iri("file:///etc/passwd").is_err());
        assert!(validate_snapshot_iri("https://example.org/C").is_ok());
    }

    #[test]
    fn rejects_control_characters() {
        assert!(validate_iri("http://example.org/\u{0009}").is_err());
    }

    #[test]
    fn try_from_index_rejects_zero() {
        assert!(IriId::try_from_index(0).is_err());
    }

    #[test]
    fn get_returns_none_for_unknown() {
        let pool = InternPool::new();
        assert!(pool.get("http://example.org/missing").is_none());
    }

    #[test]
    fn resolve_unknown_id_errors() {
        let pool = InternPool::new();
        let err = pool.resolve(IriId::from_index(1)).expect_err("unknown id");
        assert!(matches!(err, Error::InvalidIri(_)));
    }

    #[test]
    fn single_storage_no_duplicate_bytes() {
        let mut pool = InternPool::new();
        pool.intern("http://example.org/A").expect("intern");
        // One string in vec; index holds only usize indices, not duplicate strings.
        assert_eq!(pool.strings.len(), 1);
        assert_eq!(pool.index.values().map(|v| v.len()).sum::<usize>(), 1);
    }
}
