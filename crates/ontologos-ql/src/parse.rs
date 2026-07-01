//! Functional-syntax subset parser for OWL QL conjunctive queries.

use crate::query::{ConjunctiveQuery, QueryAtom};
use crate::{Error, Result};

/// Maximum conjunctive query string length.
pub const MAX_QUERY_LEN: usize = 4096;

/// OWL QL allows at most one query atom.
pub const MAX_QUERY_ATOMS: usize = 1;

/// Parse a conjunctive query string.
///
/// Supported atoms (AND-separated):
/// - `Type(?var, ClassIRI)`
/// - `SubClassOf(?var, ClassIRI)`
pub fn parse_conjunctive_query(input: &str) -> Result<ConjunctiveQuery> {
    if input.len() > MAX_QUERY_LEN {
        return Err(Error::Parse(format!(
            "query length exceeds maximum of {MAX_QUERY_LEN}"
        )));
    }
    let mut atoms = Vec::new();
    for part in input.split("AND").map(str::trim).filter(|s| !s.is_empty()) {
        atoms.push(parse_atom(part)?);
    }
    if atoms.len() > MAX_QUERY_ATOMS {
        return Err(Error::Parse(format!(
            "query atom count exceeds maximum of {MAX_QUERY_ATOMS}"
        )));
    }
    Ok(ConjunctiveQuery { atoms })
}

fn parse_atom(input: &str) -> Result<QueryAtom> {
    if let Some(inner) = input
        .strip_prefix("Type(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let (var, class) = split_two(inner)?;
        return Ok(QueryAtom::Type { var, class });
    }
    if let Some(inner) = input
        .strip_prefix("SubClassOf(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let (var, class) = split_two(inner)?;
        return Ok(QueryAtom::Subsumed {
            var,
            superclass: class,
        });
    }
    Err(Error::Parse(format!("unsupported query atom: {input}")))
}

fn split_two(input: &str) -> Result<(String, String)> {
    let mut parts = input.splitn(2, ',').map(str::trim);
    let a = parts
        .next()
        .ok_or_else(|| Error::Parse("missing first argument".into()))?
        .trim_matches('?')
        .to_owned();
    let b = parts
        .next()
        .ok_or_else(|| Error::Parse("missing second argument".into()))?
        .to_owned();
    Ok((a, b))
}
