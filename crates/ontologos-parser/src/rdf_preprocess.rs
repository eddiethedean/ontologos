//! Expand XML internal entities in RDF/XML before parsing.

use std::collections::HashMap;

/// Expand `<!ENTITY ...>` declarations in RDF/XML prolog.
#[must_use]
pub fn expand_xml_entities(input: &str) -> String {
    if !input.contains("<!ENTITY") {
        return input.to_owned();
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
        return input.to_owned();
    }
    let mut out = input.to_owned();
    for _ in 0..8 {
        let before = out.clone();
        for (name, value) in &entities {
            out = out.replace(&format!("&{name};"), value);
        }
        if out == before {
            break;
        }
    }
    out
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
}
