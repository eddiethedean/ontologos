//! Expand XML internal entities in RDF/XML before parsing.

use std::collections::{HashMap, HashSet};

use crate::{Error, Result};

const BUILTIN_PREFIXES: &[&str] = &["rdf", "owl", "rdfs", "xsd", "xml"];

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

/// Inject explicit `owl:Class` / property declarations for RDF-Based Semantics punning.
///
/// Horned-OWL requires typed entities before `equivalentClass`, `disjointWith`, and
/// `equivalentProperty` axioms; OWL WG RDF-based fixtures often rely on punning only.
#[must_use]
pub fn inject_rdf_based_punning_declarations(input: &str) -> String {
    if !input.contains("equivalentClass")
        && !input.contains("equivalentProperty")
        && !input.contains("propertyDisjointWith")
        && !input.contains("disjointWith")
    {
        return input.to_owned();
    }

    let Some(insert_at) = find_rdf_open_body_start(input) else {
        return input.to_owned();
    };

    let xmlns = parse_xmlns(input);
    let declared_classes = declared_iris(input, "owl:Class");
    let declared_object = declared_iris(input, "owl:ObjectProperty");
    let declared_datatype = declared_iris(input, "owl:DatatypeProperty");

    let mut classes = HashSet::new();
    let mut object_props = HashSet::new();
    let mut datatype_props = HashSet::new();

    collect_class_axiom_iris(input, &mut classes);
    collect_property_axiom_iris(input, &mut object_props);
    collect_punned_class_iris(input, &xmlns, &mut classes);
    collect_punned_property_iris(input, &xmlns, &mut object_props, &mut datatype_props);

    classes.retain(|iri| !declared_classes.contains(iri));
    object_props.retain(|iri| {
        !declared_object.contains(iri) && !declared_datatype.contains(iri)
    });
    datatype_props.retain(|iri| {
        !declared_object.contains(iri) && !declared_datatype.contains(iri)
    });

    // Property axioms without usage clues default to object properties, except WG `dp`.
    for iri in object_props.clone() {
        if datatype_property_fallback(&iri) {
            object_props.remove(&iri);
            datatype_props.insert(iri);
        }
    }

    if classes.is_empty() && object_props.is_empty() && datatype_props.is_empty() {
        return input.to_owned();
    }

    let mut injections = String::new();
    let mut class_list: Vec<_> = classes.into_iter().collect();
    class_list.sort();
    for iri in class_list {
        injections.push_str(&format!("  <owl:Class rdf:about=\"{iri}\"/>\n"));
    }
    let mut object_list: Vec<_> = object_props.into_iter().collect();
    object_list.sort();
    for iri in object_list {
        injections.push_str(&format!("  <owl:ObjectProperty rdf:about=\"{iri}\"/>\n"));
    }
    let mut datatype_list: Vec<_> = datatype_props.into_iter().collect();
    datatype_list.sort();
    for iri in datatype_list {
        injections.push_str(&format!("  <owl:DatatypeProperty rdf:about=\"{iri}\"/>\n"));
    }

    let mut out = String::with_capacity(input.len() + injections.len());
    out.push_str(&input[..insert_at]);
    out.push_str(&injections);
    out.push_str(&input[insert_at..]);
    out
}

fn find_rdf_open_body_start(input: &str) -> Option<usize> {
    let root_start = input.find("<rdf:RDF")?;
    let root_end = input[root_start..].find('>')? + root_start + 1;
    Some(root_end)
}

fn parse_xmlns(input: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Some(root_start) = input.find("<rdf:RDF") else {
        return map;
    };
    let Some(root_end) = input[root_start..].find('>') else {
        return map;
    };
    let root_tag = &input[root_start..root_start + root_end + 1];
    for token in root_tag.split_whitespace() {
        if let Some((prefix, iri)) = token.strip_prefix("xmlns:").and_then(|rest| rest.split_once('='))
        {
            if let Some(iri) = trim_xml_attr_value(iri) {
                map.insert(prefix.to_owned(), iri);
            }
        } else if let Some(iri) = token.strip_prefix("xmlns=").and_then(trim_xml_attr_value) {
            map.insert(String::new(), iri);
        }
    }
    map
}

fn trim_xml_attr_value(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if (raw.starts_with('"') && raw.ends_with('"')) || (raw.starts_with('\'') && raw.ends_with('\''))
    {
        Some(raw[1..raw.len() - 1].to_owned())
    } else {
        None
    }
}

fn expand_qname(qname: &str, xmlns: &HashMap<String, String>) -> Option<String> {
    let (prefix, local) = qname.split_once(':')?;
    if BUILTIN_PREFIXES.contains(&prefix) {
        return None;
    }
    let base = xmlns.get(prefix)?;
    Some(format!("{base}{local}"))
}

fn declared_iris(input: &str, element: &str) -> HashSet<String> {
    let open = format!("<{element}");
    let mut out = HashSet::new();
    let mut pos = 0usize;
    while let Some(rel) = input[pos..].find(&open) {
        let start = pos + rel;
        let Some(tag_end) = input[start..].find('>') else {
            break;
        };
        let tag = &input[start..start + tag_end + 1];
        if let Some(iri) = extract_attribute(tag, "rdf:about") {
            out.insert(iri);
        }
        pos = start + tag_end + 1;
    }
    out
}

fn extract_attribute(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let idx = tag.find(&needle)?;
    let rest = &tag[idx + needle.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

fn collect_class_axiom_iris(input: &str, out: &mut HashSet<String>) {
    for tag in ["owl:equivalentClass", "owl:disjointWith"] {
        collect_axiom_endpoint_iris(input, tag, out);
    }
}

fn collect_property_axiom_iris(input: &str, out: &mut HashSet<String>) {
    for tag in ["owl:equivalentProperty", "owl:propertyDisjointWith"] {
        collect_axiom_endpoint_iris(input, tag, out);
    }
}

fn collect_axiom_endpoint_iris(input: &str, tag: &str, out: &mut HashSet<String>) {
    let open = format!("<{tag}");
    let mut pos = 0usize;
    while let Some(rel) = input[pos..].find(&open) {
        let start = pos + rel;
        let Some(tag_end) = input[start..].find('>') else {
            break;
        };
        let end = start + tag_end + 1;
        let fragment = &input[start..end];
        if let Some(iri) = extract_attribute(fragment, "rdf:resource") {
            out.insert(iri);
        }
        if let Some(iri) = find_enclosing_rdf_about(input, start) {
            out.insert(iri);
        }
        pos = end;
    }
}

fn find_enclosing_rdf_about(input: &str, from: usize) -> Option<String> {
    let head = &input[..from];
    let desc = "<rdf:Description";
    let mut search = head.len();
    while search > 0 {
        let Some(rel) = head[..search].rfind(desc) else {
            break;
        };
        let start = rel;
        let Some(tag_end) = head[start..].find('>') else {
            search = start;
            continue;
        };
        let tag = &head[start..start + tag_end + 1];
        let Some(about) = extract_attribute(tag, "rdf:about") else {
            search = start;
            continue;
        };
        if description_encloses_position(input, start, from) {
            return Some(about);
        }
        search = start;
    }
    None
}

fn description_encloses_position(input: &str, desc_start: usize, pos: usize) -> bool {
    let Some(open_end) = input[desc_start..].find('>') else {
        return false;
    };
    let open_tag = &input[desc_start..desc_start + open_end + 1];
    if !open_tag.starts_with("<rdf:Description") {
        return false;
    }
    let mut depth = 1usize;
    let mut search = desc_start + open_end + 1;
    while depth > 0 && search < input.len() {
        let Some(rel) = input[search..].find('<') else {
            break;
        };
        let abs = search + rel;
        let Some(tag_end) = input[abs..].find('>') else {
            break;
        };
        let tag = &input[abs..abs + tag_end + 1];
        if tag.starts_with("</rdf:Description") {
            depth -= 1;
            if depth == 0 {
                return pos < abs + tag_end + 1;
            }
        } else if tag.starts_with("<rdf:Description") && !tag.ends_with("/>") {
            depth += 1;
        }
        search = abs + tag_end + 1;
    }
    false
}

fn collect_punned_class_iris(input: &str, xmlns: &HashMap<String, String>, out: &mut HashSet<String>) {
    let mut pos = 0usize;
    while pos < input.len() {
        let Some(rel) = input[pos..].find('<') else {
            break;
        };
        let start = pos + rel;
        let Some(tag_end) = input[start..].find('>') else {
            break;
        };
        let tag = &input[start..start + tag_end + 1];
        if tag.starts_with("<!--") || tag.starts_with("<!") || tag.starts_with("<?") {
            pos = start + tag_end + 1;
            continue;
        }
        if let Some(name) = element_qname(tag) {
            if let Some(iri) = expand_qname(name, xmlns) {
                if tag.contains("rdf:about=\"") {
                    out.insert(iri);
                }
            }
        }
        pos = start + tag_end + 1;
    }
}

fn collect_punned_property_iris(
    input: &str,
    xmlns: &HashMap<String, String>,
    object_props: &mut HashSet<String>,
    datatype_props: &mut HashSet<String>,
) {
    let mut pos = 0usize;
    while pos < input.len() {
        let Some(rel) = input[pos..].find('<') else {
            break;
        };
        let start = pos + rel;
        let Some(tag_end) = input[start..].find('>') else {
            break;
        };
        let tag = &input[start..start + tag_end + 1];
        if tag.starts_with("<!--") || tag.starts_with("<!") || tag.starts_with("<?") {
            pos = start + tag_end + 1;
            continue;
        }
        let Some(name) = element_qname(tag) else {
            pos = start + tag_end + 1;
            continue;
        };
        let Some(iri) = expand_qname(name, xmlns) else {
            pos = start + tag_end + 1;
            continue;
        };
        if tag.contains("rdf:resource=\"") {
            object_props.insert(iri);
        } else if tag.contains("rdf:datatype=\"") || has_literal_body(input, start + tag_end + 1, name)
        {
            datatype_props.insert(iri);
        }
        pos = start + tag_end + 1;
    }
}

fn element_qname(open_tag: &str) -> Option<&str> {
    let inner = open_tag.strip_prefix('<')?.trim_end_matches('>').trim();
    if inner.starts_with('/') {
        return None;
    }
    let name = inner.split_whitespace().next()?;
    if name.contains(':') && !name.starts_with("rdf:") && !name.starts_with("owl:") {
        Some(name)
    } else {
        None
    }
}

fn has_literal_body(input: &str, body_start: usize, qname: &str) -> bool {
    let close = format!("</{qname}>");
    let Some(rel) = input[body_start..].find(&close) else {
        return false;
    };
    let body = input[body_start..body_start + rel].trim();
    !body.is_empty() && !body.starts_with('<')
}

fn datatype_property_fallback(iri: &str) -> bool {
    iri.rsplit('#').next().is_some_and(|local| local == "dp")
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
    fn injects_nested_class_declarations_for_subst_fixture() {
        let input = include_str!(
            "../../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Deqdis-2Deqclass-2Dsubst/premise.rdf"
        );
        let out = inject_rdf_based_punning_declarations(input);
        assert!(out.contains("<owl:Class rdf:about=\"http://www.example.org#c1\"/>"));
        assert!(out.contains("<owl:Class rdf:about=\"http://www.example.org#d1\"/>"));
    }

    #[test]
    fn injects_class_declarations_for_rdf_based_equivalent_class() {
        let input = include_str!(
            "../../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Deqdis-2Deqclass-2Dinst/premise.rdf"
        );
        let out = inject_rdf_based_punning_declarations(input);
        assert!(out.contains("<owl:Class rdf:about=\"http://www.example.org#c1\"/>"));
        assert!(out.contains("<owl:Class rdf:about=\"http://www.example.org#c2\"/>"));
    }

    #[test]
    fn injects_property_declarations_for_rdf_based_equivalent_property() {
        let input = include_str!(
            "../../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Deqdis-2Deqprop-2Dinst/premise.rdf"
        );
        let out = inject_rdf_based_punning_declarations(input);
        assert!(out.contains("<owl:ObjectProperty rdf:about=\"http://www.example.org#p1\"/>"));
        assert!(out.contains("<owl:ObjectProperty rdf:about=\"http://www.example.org#p2\"/>"));
    }

    #[test]
    fn injects_datatype_property_for_rdf_based_rflxv_conclusion() {
        let input = include_str!(
            "../../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Deqdis-2Deqprop-2Drflxv/conclusion.rdf"
        );
        let out = inject_rdf_based_punning_declarations(input);
        assert!(out.contains("<owl:DatatypeProperty rdf:about=\"http://www.example.org#dp\"/>"));
        assert!(out.contains("<owl:ObjectProperty rdf:about=\"http://www.example.org#op\"/>"));
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
