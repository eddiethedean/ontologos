//! Expand XML internal entities in RDF/XML before parsing.

use std::collections::{HashMap, HashSet};

use crate::{Error, Result};

/// Expand `<!ENTITY ...>` declarations in RDF/XML prolog.
#[must_use]
pub fn expand_xml_entities(input: &str) -> String {
    expand_xml_entities_with_limit(input, usize::MAX).unwrap_or_else(|_| input.to_owned())
}

/// Remove duplicate `rdf:ID` element subtrees (legacy fixtures such as HermiT `wine.xml`).
#[must_use]
pub fn dedupe_rdf_xml_ids(input: &str) -> String {
    let mut seen = HashSet::new();
    let mut out = String::with_capacity(input.len());
    let mut pos = 0usize;
    while pos < input.len() {
        let Some(rel) = input[pos..].find('<') else {
            out.push_str(&input[pos..]);
            break;
        };
        let start = pos + rel;
        if start > pos {
            out.push_str(&input[pos..start]);
        }
        let Some(tag_end) = input[start..].find('>') else {
            out.push_str(&input[start..]);
            break;
        };
        let tag = &input[start..start + tag_end + 1];
        if tag.starts_with("<!--") || tag.starts_with("<!---") {
            if let Some(close) = input[start..].find("-->") {
                out.push_str(&input[start..start + close + 3]);
                pos = start + close + 3;
                continue;
            }
        }
        if let Some(id) = extract_rdf_id(tag) {
            if !seen.insert(id.to_owned()) {
                if let Some(end) = find_element_end(input, start) {
                    pos = end;
                    continue;
                }
            }
        }
        out.push_str(tag);
        pos = start + tag_end + 1;
    }
    out
}

fn extract_rdf_id(tag: &str) -> Option<&str> {
    const MARKER: &str = "rdf:ID=\"";
    let idx = tag.find(MARKER)?;
    let rest = &tag[idx + MARKER.len()..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn find_element_end(input: &str, open_start: usize) -> Option<usize> {
    let open_tag_end = input[open_start..].find('>')? + open_start;
    let open_tag = &input[open_start..=open_tag_end];
    if open_tag.ends_with("/>") || open_tag.contains("</") {
        return Some(open_tag_end + 1);
    }
    let name = element_name(open_tag)?;
    let close = format!("</{name}>");
    let mut depth = 1usize;
    let mut search = open_tag_end + 1;
    while depth > 0 && search < input.len() {
        let abs = input[search..].find('<')? + search;
        let tag_end = input[abs..].find('>')? + abs;
        let tag = &input[abs..=tag_end];
        if tag.starts_with("</") {
            if let Some(close_name) = tag.strip_prefix("</").and_then(|t| t.strip_suffix('>')) {
                let close_name = close_name.split_whitespace().next().unwrap_or(close_name);
                if close_name == name {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(abs + tag_end + 1);
                    }
                }
            }
        } else if !tag.ends_with("/>")
            && !tag.starts_with("<?")
            && !tag.starts_with("<!")
            && element_name(tag) == Some(name)
        {
            depth += 1;
        }
        search = abs + tag_end + 1;
    }
    input[open_start..]
        .find(&close)
        .map(|idx| open_start + idx + close.len())
}

fn element_name(open_tag: &str) -> Option<&str> {
    let inner = open_tag.strip_prefix('<')?.trim_end_matches('>').trim();
    let name = inner.split_whitespace().next()?;
    Some(name.trim_end_matches('/'))
}

/// Expand entities with an output size cap for untrusted RDF/XML.
pub fn expand_xml_entities_with_limit(input: &str, max_bytes: usize) -> Result<String> {
    if !input.contains("<!ENTITY") {
        if input.len() > max_bytes {
            return Err(Error::Parse(format!(
                "expanded RDF/XML size {} exceeds limit of {max_bytes} bytes",
                input.len()
            )));
        }
        return Ok(input.to_owned());
    }
    let mut entities = HashMap::new();
    for line in input.lines() {
        if let Some(idx) = line.find("<!ENTITY") {
            let fragment = &line[idx..];
            if let Some((name, value)) = parse_entity_decl(fragment) {
                entities.insert(name, value);
            }
        }
    }
    if entities.is_empty() {
        if input.len() > max_bytes {
            return Err(Error::Parse(format!(
                "expanded RDF/XML size {} exceeds limit of {max_bytes} bytes",
                input.len()
            )));
        }
        return Ok(input.to_owned());
    }
    let mut out = input.to_owned();
    for _ in 0..8 {
        let before = out.clone();
        for (name, value) in &entities {
            out = out.replace(&format!("&{name};"), value);
            if out.len() > max_bytes {
                return Err(Error::Parse(format!(
                    "expanded RDF/XML size exceeds limit of {max_bytes} bytes during entity expansion"
                )));
            }
        }
        if out == before {
            break;
        }
    }
    Ok(out)
}

fn parse_entity_decl(rest: &str) -> Option<(String, String)> {
    let rest = rest.strip_prefix("<!ENTITY")?.trim();
    let (name, rest) = rest.split_once(|c: char| c.is_whitespace())?;
    let name = name.trim().to_owned();
    let value_start = rest.find('"')? + 1;
    let value_end = rest[value_start..].find('"')? + value_start;
    let value = rest[value_start..value_end].to_owned();
    Some((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip_xml_comments(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut pos = 0usize;
        while pos < input.len() {
            let Some(rel) = input[pos..].find('<') else {
                out.push_str(&input[pos..]);
                break;
            };
            let start = pos + rel;
            out.push_str(&input[pos..start]);
            if input[start..].starts_with("<!--") || input[start..].starts_with("<!---") {
                if let Some(close) = input[start..].find("-->") {
                    pos = start + close + 3;
                    continue;
                }
            }
            if let Some(tag_end) = input[start..].find('>') {
                out.push_str(&input[start..start + tag_end + 1]);
                pos = start + tag_end + 1;
            } else {
                out.push_str(&input[start..]);
                break;
            }
        }
        out
    }

    #[test]
    fn expands_simple_entity() {
        let xml = r#"<!ENTITY ex "http://example.org/">"#;
        let input = format!(
            r#"<?xml version="1.0"?>
<!DOCTYPE rdf:RDF [{xml}]>
<rdf:RDF>&ex;a</rdf:RDF>"#
        );
        let expanded = expand_xml_entities(&input);
        assert!(expanded.contains("http://example.org/"));
        assert!(!expanded.contains("&ex;"));
    }

    #[test]
    fn expansion_limit_rejects_blowup() {
        let xml = r#"<!ENTITY a "aaaaaaaaaa"><!ENTITY b "&a;&a;">"#;
        let input = format!(
            r#"<?xml version="1.0"?>
<!DOCTYPE rdf:RDF [{xml}]>
<rdf:RDF>&b;</rdf:RDF>"#
        );
        let err = expand_xml_entities_with_limit(&input, 64).expect_err("limit");
        assert!(err.to_string().contains("exceeds limit"));
    }

    #[test]
    fn dedupe_removes_second_rdf_id_element() {
        let input = r##"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:owl="http://www.w3.org/2002/07/owl#">
  <owl:Class rdf:ID="Wine"><rdfs:label>first</rdfs:label></owl:Class>
  <owl:Class rdf:ID="Wine"><owl:equivalentClass rdf:resource="#Wine"/></owl:Class>
</rdf:RDF>"##;
        let out = dedupe_rdf_xml_ids(input);
        assert_eq!(out.matches("rdf:ID=\"Wine\"").count(), 1);
        assert!(out.contains("first"));
        assert!(!out.contains("equivalentClass"));
    }

    #[test]
    fn wine_fixture_has_no_duplicate_ids_after_dedupe() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/reasoner/res/wine.xml");
        let input = std::fs::read_to_string(&path).expect("wine.xml");
        let deduped = dedupe_rdf_xml_ids(&input);
        let without_comments = strip_xml_comments(&deduped);
        let mut seen = HashSet::new();
        let mut dupes = Vec::new();
        for line in without_comments.lines() {
            if let Some(idx) = line.find("rdf:ID=\"") {
                let rest = &line[idx + 8..];
                if let Some(end) = rest.find('"') {
                    let id = &rest[..end];
                    if !seen.insert(id.to_owned()) {
                        dupes.push(id.to_owned());
                    }
                }
            }
        }
        assert!(dupes.is_empty(), "duplicate ids remain: {dupes:?}");
    }
}
