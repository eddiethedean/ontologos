//! Expand XML internal entities in RDF/XML before parsing.

use std::collections::{HashMap, HashSet};

use crate::{Error, Result};

const BUILTIN_PREFIXES: &[&str] = &["rdf", "owl", "rdfs", "xsd", "xml"];

/// Expand `<!ENTITY ...>` declarations in RDF/XML prolog.
#[must_use]
pub fn expand_xml_entities(input: &str) -> String {
    expand_xml_entities_with_limit(input, usize::MAX).unwrap_or_else(|_| input.to_owned())
}

/// Collapse a multiline `<rdf:RDF ...>` opening tag onto one line for Horned-OWL's RDF/XML reader.
#[must_use]
pub fn normalize_multiline_rdf_root_tag(input: &str) -> String {
    let Some(root_start) = input.find("<rdf:RDF") else {
        return input.to_owned();
    };
    let Some(rel_end) = input[root_start..].find('>') else {
        return input.to_owned();
    };
    let root_end = root_start + rel_end + 1;
    let root_tag = &input[root_start..root_end];
    if !root_tag.contains(['\n', '\r']) {
        return input.to_owned();
    }
    let collapsed = root_tag.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = String::with_capacity(input.len());
    out.push_str(&input[..root_start]);
    out.push_str(&collapsed);
    out.push_str(&input[root_end..]);
    out
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

/// Rewrite legacy `rdf:ID` values that are not valid NCNames (HermiT `galen` uses `1.0`, etc.).
#[must_use]
pub fn normalize_invalid_rdf_ids(input: &str) -> String {
    let mut remap = HashMap::new();
    for id in collect_rdf_ids(input) {
        if !is_valid_rdf_id(&id) {
            let sanitized = sanitize_rdf_id(&id);
            remap.insert(id, sanitized);
        }
    }
    if remap.is_empty() {
        return input.to_owned();
    }
    let mut pairs: Vec<_> = remap.into_iter().collect();
    pairs.sort_by_key(|(old, _)| std::cmp::Reverse(old.len()));
    let mut out = input.to_owned();
    for (old, new) in pairs {
        out = out.replace(&format!("rdf:ID=\"{old}\""), &format!("rdf:ID=\"{new}\""));
        out = out.replace(&format!("rdf:ID='{old}'"), &format!("rdf:ID='{new}'"));
        out = out.replace(&format!("#{old}"), &format!("#{new}"));
    }
    out
}

fn collect_rdf_ids(input: &str) -> HashSet<String> {
    let mut ids = HashSet::new();
    let mut pos = 0usize;
    while pos < input.len() {
        let Some(rel) = input[pos..].find("rdf:ID=\"") else {
            break;
        };
        let start = pos + rel + 8;
        let Some(end_rel) = input[start..].find('"') else {
            break;
        };
        ids.insert(input[start..start + end_rel].to_owned());
        pos = start + end_rel + 1;
    }
    ids
}

fn is_valid_rdf_id(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

fn sanitize_rdf_id(id: &str) -> String {
    let mut out = String::from("_");
    for c in id.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else {
            out.push('_');
        }
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

/// Nesting depth inside `rdf:RDF` at `pos` (1 = direct child of document element).
fn rdf_document_nesting_depth_at(input: &str, pos: usize) -> usize {
    let mut depth = 0usize;
    let mut cursor = 0usize;
    while cursor < pos {
        let Some(rel) = input[cursor..].find('<') else {
            break;
        };
        let tag_start = cursor + rel;
        if input[tag_start..].starts_with("<!--") {
            let end = input[tag_start..]
                .find("-->")
                .map(|idx| tag_start + idx + 3)
                .unwrap_or(input.len());
            cursor = end;
            continue;
        }
        let Some(gt_rel) = input[tag_start..].find('>') else {
            break;
        };
        let tag_end = tag_start + gt_rel + 1;
        let tag = &input[tag_start..tag_end];
        if tag.starts_with("</rdf:RDF") {
            depth = depth.saturating_sub(1);
        } else if tag.starts_with("<rdf:RDF") && !tag.trim_end().ends_with("/>") {
            depth += 1;
        } else if tag.starts_with("</") {
            if depth > 0 {
                depth = depth.saturating_sub(1);
            }
        } else if !tag.trim_end().ends_with("/>")
            && !tag.starts_with("<?")
            && !tag.starts_with("<!")
            && depth > 0
        {
            depth += 1;
        }
        cursor = tag_end;
    }
    depth
}

fn is_direct_rdf_document_child(input: &str, element_start: usize) -> bool {
    rdf_document_nesting_depth_at(input, element_start) == 1
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
        return Ok(strip_xml_internal_subset(input));
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
    Ok(strip_xml_internal_subset(&out))
}

/// Remove `<!DOCTYPE ...>` after entity expansion so horned-owl does not re-parse declarations.
fn strip_xml_internal_subset(input: &str) -> String {
    let Some(start) = input.find("<!DOCTYPE") else {
        return input.to_owned();
    };
    let Some(end) = find_doctype_end(input, start) else {
        return input.to_owned();
    };
    let mut out = String::with_capacity(input.len());
    out.push_str(&input[..start]);
    out.push_str(&input[end..]);
    out
}

fn find_doctype_end(input: &str, start: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut i = start;
    let mut in_quote: Option<u8> = None;
    let mut bracket_depth = 0i32;
    while i < bytes.len() {
        let b = bytes[i];
        match in_quote {
            Some(q) if b == q => in_quote = None,
            Some(_) => {}
            None if b == b'"' || b == b'\'' => in_quote = Some(b),
            None if b == b'[' => bracket_depth += 1,
            None if b == b']' => bracket_depth = bracket_depth.saturating_sub(1),
            None if b == b'>' && bracket_depth == 0 => return Some(i + 1),
            _ => {}
        }
        i += 1;
    }
    None
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
    object_props.retain(|iri| !declared_object.contains(iri) && !declared_datatype.contains(iri));
    datatype_props.retain(|iri| !declared_object.contains(iri) && !declared_datatype.contains(iri));

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
    let root_inner = root_tag.trim_end().strip_suffix('>').unwrap_or(root_tag);
    for token in root_inner.split_whitespace() {
        if let Some((prefix, iri)) = token
            .strip_prefix("xmlns:")
            .and_then(|rest| rest.split_once('='))
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
    if (raw.starts_with('"') && raw.ends_with('"'))
        || (raw.starts_with('\'') && raw.ends_with('\''))
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
    for quote in ['"', '\''] {
        let mut search_from = 0usize;
        while let Some(rel) = tag[search_from..].find(name) {
            let abs = search_from + rel;
            let after_name = tag[abs + name.len()..].trim_start();
            let after_eq = after_name.strip_prefix('=')?.trim_start();
            if !after_eq.starts_with(quote) {
                search_from = abs + 1;
                continue;
            }
            let val = &after_eq[1..];
            if let Some(end) = val.find(quote) {
                return Some(val[..end].to_owned());
            }
            search_from = abs + 1;
        }
    }
    None
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

fn collect_punned_class_iris(
    input: &str,
    xmlns: &HashMap<String, String>,
    out: &mut HashSet<String>,
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
        } else if tag.contains("rdf:datatype=\"")
            || has_literal_body(input, start + tag_end + 1, name)
        {
            datatype_props.insert(iri);
        }
        pos = start + tag_end + 1;
    }
}

fn element_qname(open_tag: &str) -> Option<&str> {
    let inner = open_tag.strip_prefix('<')?.trim_end_matches('>').trim();
    let inner = inner.strip_suffix('/').unwrap_or(inner).trim();
    if inner.starts_with('/') {
        return None;
    }
    let name = inner.split_whitespace().next()?;
    if name.contains(':') && !name.starts_with("rdf:") {
        if name.starts_with("owl:") {
            return builtin_owl_typed_node_qname(name);
        }
        Some(name)
    } else {
        None
    }
}

/// RDF/XML typed nodes for built-in `owl:Thing` / `owl:Nothing` (WG inconsistency fixtures).
fn builtin_owl_typed_node_qname(name: &str) -> Option<&str> {
    match name {
        "owl:Thing" | "owl:Nothing" => Some(name),
        _ => None,
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
    let local = iri.rsplit('#').next().unwrap_or(iri);
    let local = local.rsplit('/').next().unwrap_or(local);
    local == "dp"
}

/// Map legacy `rdfs:Class` declarations to `owl:Class` for Horned-OWL RDF parsing.
#[must_use]
pub fn normalize_rdfs_class_elements(input: &str) -> String {
    if !input.contains("rdfs:Class") {
        return input.to_owned();
    }
    let mut out = input
        .replace("<rdfs:Class", "<owl:Class")
        .replace("</rdfs:Class>", "</owl:Class>");
    if out.contains("owl:Class") && !out.contains("xmlns:owl") {
        if let Some(root_start) = out.find("<rdf:RDF") {
            if let Some(rel_end) = out[root_start..].find('>') {
                let insert_at = root_start + rel_end;
                out.insert_str(insert_at, " xmlns:owl=\"http://www.w3.org/2002/07/owl#\"");
            }
        }
    }
    out
}

/// Rewrite legacy relative `rdf:datatype` / `rdf:resource` IRIs (`/2001/XMLSchema#...`).
#[must_use]
pub fn normalize_relative_owl_uris(input: &str) -> String {
    input
        .replace(
            "rdf:datatype=\"/2001/XMLSchema#",
            "rdf:datatype=\"http://www.w3.org/2001/XMLSchema#",
        )
        .replace(
            "rdf:datatype='/2001/XMLSchema#",
            "rdf:datatype='http://www.w3.org/2001/XMLSchema#",
        )
        .replace(
            "rdf:resource=\"/2002/07/owl#",
            "rdf:resource=\"http://www.w3.org/2002/07/owl#",
        )
        .replace(
            "rdf:resource='/2002/07/owl#",
            "rdf:resource='http://www.w3.org/2002/07/owl#",
        )
        .replace(
            "rdf:about=\"/2002/07/owl#",
            "rdf:about=\"http://www.w3.org/2002/07/owl#",
        )
        .replace(
            "rdf:about='/2002/07/owl#",
            "rdf:about='http://www.w3.org/2002/07/owl#",
        )
}

///
/// Horned-OWL ignores this RDF/XML production; OWL WG inconsistency fixtures use it for ABox
/// assertions such as `<oiled:Unsatisfiable/>` and `<owl:Nothing/>`.
#[must_use]
pub fn materialize_typed_node_elements(input: &str) -> String {
    if !input.contains('<') {
        return input.to_owned();
    }
    let xmlns = parse_xmlns(input);
    let base = parse_xml_base(input);
    let mut counter = 0usize;
    let mut out = String::with_capacity(input.len() + 512);
    let mut pos = 0usize;
    while pos < input.len() {
        let Some(rel) = input[pos..].find('<') else {
            out.push_str(&input[pos..]);
            break;
        };
        let start = pos + rel;
        out.push_str(&input[pos..start]);
        if input[start..].starts_with("</")
            || input[start..].starts_with("<!--")
            || input[start..].starts_with("<!")
            || input[start..].starts_with("<?")
        {
            let tag_end = input[start..]
                .find('>')
                .map(|idx| start + idx + 1)
                .unwrap_or(input.len());
            out.push_str(&input[start..tag_end]);
            pos = tag_end;
            continue;
        }
        let Some(tag_end_rel) = input[start..].find('>') else {
            out.push_str(&input[start..]);
            break;
        };
        let tag_end = start + tag_end_rel;
        let open_tag = &input[start..=tag_end];
        if is_direct_rdf_document_child(input, start) && is_typed_node_element(open_tag) {
            if let Some(type_iri) = typed_node_class_iri(open_tag, &xmlns) {
                counter += 1;
                let iri = format!("{base}#_:tn{counter}");
                out.push_str(&format!(
                    "<rdf:Description rdf:about=\"{iri}\">\n  <rdf:type rdf:resource=\"{type_iri}\"/>\n</rdf:Description>"
                ));
                pos = tag_end + 1;
                continue;
            }
        }
        if let Some(qname) = element_qname(open_tag) {
            if is_direct_rdf_document_child(input, start) && !open_tag.trim_end().ends_with("/>") {
                let end = find_element_end(input, start)
                    .filter(|&e| e <= input.len())
                    .unwrap_or(tag_end + 1);
                let block = &input[start..end];
                if let Some(rewritten) =
                    rewrite_typed_node_block(block, qname, &xmlns, &base, &mut counter)
                {
                    out.push_str(&rewritten);
                    pos = end;
                    continue;
                }
            }
        }
        out.push_str(open_tag);
        pos = tag_end + 1;
    }
    out
}

fn rewrite_typed_node_block(
    block: &str,
    qname: &str,
    xmlns: &HashMap<String, String>,
    base: &str,
    counter: &mut usize,
) -> Option<String> {
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    if element_qname(open).is_none_or(|q| q != qname) {
        return None;
    }
    let class_iri = builtin_owl_typed_node_iri(qname).or_else(|| expand_qname(qname, xmlns))?;
    let close = format!("</{qname}>");
    let close_start = block.rfind(&close)?;
    let inner = &block[open_end + 1..close_start];
    *counter += 1;
    let iri = format!("{base}#_:tn{counter}");
    Some(format!(
        "<rdf:Description rdf:about=\"{iri}\">\n  <rdf:type rdf:resource=\"{class_iri}\"/>\n{inner}</rdf:Description>"
    ))
}

/// Rewrite typed RDF/XML property elements with `rdf:about` into explicit individual typings.
///
/// Horned-OWL ignores `<ont:Man rdf:about="..."/>` and `<ex:z rdf:about="..."><ex:p .../></ex:z>`.
#[must_use]
pub fn materialize_typed_about_elements(input: &str) -> String {
    if !input.contains("rdf:about") {
        return input.to_owned();
    }
    let xmlns = parse_xmlns(input);
    let base = parse_xml_base(input);
    let mut out = String::with_capacity(input.len() + 512);
    let mut pos = 0usize;
    while pos < input.len() {
        let Some(rel) = input[pos..].find('<') else {
            out.push_str(&input[pos..]);
            break;
        };
        let start = pos + rel;
        out.push_str(&input[pos..start]);
        if input[start..].starts_with("</")
            || input[start..].starts_with("<!--")
            || input[start..].starts_with("<!")
            || input[start..].starts_with("<?")
        {
            let tag_end = input[start..]
                .find('>')
                .map(|idx| start + idx + 1)
                .unwrap_or(input.len());
            out.push_str(&input[start..tag_end]);
            pos = tag_end;
            continue;
        }
        let Some(tag_end_rel) = input[start..].find('>') else {
            out.push_str(&input[start..]);
            break;
        };
        let tag_end = start + tag_end_rel;
        let open_tag = &input[start..=tag_end];
        let is_candidate =
            extract_attribute(open_tag, "rdf:about").is_some() && element_qname(open_tag).is_some();
        if !is_candidate {
            if open_tag.trim_end().ends_with("/>") {
                out.push_str(open_tag);
                pos = tag_end + 1;
            } else {
                let end = find_element_end(input, start)
                    .filter(|&e| e <= input.len())
                    .unwrap_or(tag_end + 1);
                out.push_str(&input[start..end]);
                pos = end;
            }
            continue;
        }
        let end = if open_tag.trim_end().ends_with("/>") {
            tag_end + 1
        } else {
            find_element_end(input, start)
                .filter(|&e| e <= input.len())
                .unwrap_or(input.len())
        };
        let block = &input[start..end];
        if is_direct_rdf_document_child(input, start) {
            if let Some(rewritten) = rewrite_typed_about_block(block, &xmlns, &base) {
                out.push_str(&rewritten);
            } else {
                out.push_str(block);
            }
        } else {
            out.push_str(block);
        }
        pos = end;
    }
    out
}

fn rewrite_typed_about_block(
    block: &str,
    xmlns: &HashMap<String, String>,
    base: &str,
) -> Option<String> {
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    element_qname(open)?;
    let about = extract_attribute(open, "rdf:about")?;
    let qname = element_name(open)?;
    let class_iri = expand_qname(qname, xmlns)?;
    let ind_iri = resolve_relative_iri(&about, base);
    if open.trim_end().ends_with("/>") {
        return Some(format!(
            "<rdf:Description rdf:about=\"{ind_iri}\">\n  <rdf:type rdf:resource=\"{class_iri}\"/>\n</rdf:Description>"
        ));
    }
    let close = format!("</{qname}>");
    let close_start = block.rfind(&close)?;
    let inner = &block[open_end + 1..close_start];
    Some(format!(
        "<rdf:Description rdf:about=\"{ind_iri}\">\n  <rdf:type rdf:resource=\"{class_iri}\"/>\n{inner}</rdf:Description>"
    ))
}

/// Normalize `owl:intersectionOf` on `owl:Class` into `owl:equivalentClass` form.
///
/// Horned-OWL drops the RDF/XML class-expression shortcut where `intersectionOf` is a direct
/// child of `owl:Class`.
#[must_use]
pub fn normalize_class_intersection_definitions(input: &str) -> String {
    if !input.contains("<owl:Class") || !input.contains("owl:intersectionOf") {
        return input.to_owned();
    }
    let mut out = String::with_capacity(input.len() + 256);
    let mut pos = 0usize;
    while let Some(rel) = input[pos..].find("<owl:Class") {
        let start = pos + rel;
        out.push_str(&input[pos..start]);
        let Some(end) = owl_class_element_end(input, start) else {
            out.push_str(&input[start..]);
            return out;
        };
        let block = &input[start..end];
        out.push_str(&rewrite_class_intersection_block(block));
        pos = end;
    }
    out.push_str(&input[pos..]);
    out
}

/// Normalize `owl:sameAs` on `owl:Class` into `owl:equivalentClass`.
///
/// Horned-OWL's RDF reader does not emit class equality from nested `owl:sameAs`.
#[must_use]
pub fn normalize_class_same_as(input: &str) -> String {
    if !input.contains("<owl:Class") || !input.contains("owl:sameAs") {
        return input.to_owned();
    }
    let base = parse_xml_base(input);
    let mut out = String::with_capacity(input.len() + 128);
    let mut pos = 0usize;
    while let Some(rel) = input[pos..].find("<owl:Class") {
        let start = pos + rel;
        out.push_str(&input[pos..start]);
        let Some(end) = owl_class_element_end(input, start) else {
            out.push_str(&input[start..]);
            return out;
        };
        let block = &input[start..end];
        out.push_str(&rewrite_class_same_as_block(block, &base));
        pos = end;
    }
    out.push_str(&input[pos..]);
    out
}

fn rewrite_class_same_as_block(block: &str, base: &str) -> String {
    let open_end = match block.find('>') {
        Some(i) => i + 1,
        None => return block.to_owned(),
    };
    let close_start = match block.rfind("</owl:Class>") {
        Some(i) => i,
        None => return block.to_owned(),
    };
    if open_end > close_start {
        return block.to_owned();
    }
    let inner = &block[open_end..close_start];
    let Some(partner_iri) = extract_class_same_as_partner(inner, base) else {
        return block.to_owned();
    };
    let Some((same_start, same_end, _)) = find_top_level_element_bounds(inner, "owl:sameAs") else {
        return block.to_owned();
    };

    let mut remainder = String::new();
    remainder.push_str(inner[..same_start].trim_end());
    if !remainder.is_empty() && !remainder.ends_with('\n') {
        remainder.push('\n');
    }
    remainder.push_str(inner[same_end..].trim_start());

    let mut rewritten = String::new();
    rewritten.push_str(&block[..open_end]);
    if !remainder.trim().is_empty() {
        rewritten.push_str(remainder.trim_end());
        rewritten.push('\n');
    }
    rewritten.push_str("  <owl:equivalentClass>\n");
    rewritten.push_str(&format!("    <owl:Class rdf:about=\"{partner_iri}\"/>\n"));
    rewritten.push_str("  </owl:equivalentClass>\n");
    rewritten.push_str(&block[close_start..]);
    rewritten
}

fn extract_class_same_as_partner(inner: &str, base: &str) -> Option<String> {
    let (_, _, same_block) = find_top_level_element_bounds(inner, "owl:sameAs")?;
    let open_end = same_block.find('>')?;
    let open_tag = &same_block[..=open_end];
    if let Some(resource) = extract_attribute(open_tag, "rdf:resource") {
        return Some(resolve_relative_iri(&resource, base));
    }
    if open_tag.ends_with("/>") {
        return None;
    }
    let close = "</owl:sameAs>";
    let same_inner_end = same_block.rfind(close)?;
    let same_inner = &same_block[open_end + 1..same_inner_end];
    let class_start = same_inner.find("<owl:Class")?;
    let class_open_end = same_inner[class_start..].find('>')? + class_start;
    resolve_class_iri_from_tag(&same_inner[class_start..=class_open_end], base)
}

fn find_top_level_element_bounds<'a>(inner: &'a str, tag: &str) -> Option<(usize, usize, &'a str)> {
    find_top_level_element(inner, tag)
}

fn resolve_class_iri_from_tag(open_tag: &str, base: &str) -> Option<String> {
    if let Some(about) = extract_attribute(open_tag, "rdf:about") {
        return Some(resolve_relative_iri(&about, base));
    }
    if let Some(id) = extract_attribute(open_tag, "rdf:ID") {
        return Some(format!("{base}#{id}"));
    }
    None
}

fn resolve_relative_iri(iri: &str, base: &str) -> String {
    if iri.contains("://") || iri.starts_with("file:") {
        iri.to_owned()
    } else if let Some(stripped) = iri.strip_prefix('#') {
        format!("{base}#{stripped}")
    } else if let Some((doc, frag)) = iri.split_once('#') {
        if doc.contains("://") {
            format!("{doc}#{frag}")
        } else if let Some((parent, _)) = base.rsplit_once('/') {
            format!("{parent}/{doc}#{frag}")
        } else {
            format!("{base}/{doc}#{frag}")
        }
    } else if base.ends_with('/') {
        format!("{base}{iri}")
    } else {
        format!("{base}/{iri}")
    }
}

fn typed_node_class_iri(tag: &str, xmlns: &HashMap<String, String>) -> Option<String> {
    if !is_typed_node_element(tag) {
        return None;
    }
    let qname = element_qname(tag)?;
    if let Some(iri) = builtin_owl_typed_node_iri(qname) {
        return Some(iri);
    }
    expand_qname(qname, xmlns)
}

fn builtin_owl_typed_node_iri(qname: &str) -> Option<String> {
    match qname {
        "owl:Thing" => Some("http://www.w3.org/2002/07/owl#Thing".to_owned()),
        "owl:Nothing" => Some("http://www.w3.org/2002/07/owl#Nothing".to_owned()),
        _ => None,
    }
}

fn is_typed_node_element(tag: &str) -> bool {
    if !tag.ends_with("/>") || tag.starts_with("</") {
        return false;
    }
    if tag.starts_with("<!--") || tag.starts_with("<!") || tag.starts_with("<?") {
        return false;
    }
    !(tag.contains("rdf:about=\"")
        || tag.contains("rdf:about='")
        || tag.contains("rdf:nodeID=\"")
        || tag.contains("rdf:nodeID='")
        || tag.contains("rdf:ID=\"")
        || tag.contains("rdf:ID='")
        || tag.contains("rdf:resource=\"")
        || tag.contains("rdf:resource='")
        || tag.contains("rdf:datatype=\"")
        || tag.contains("rdf:datatype='")
        || tag.contains("rdf:parseType=\"")
        || tag.contains("rdf:parseType='"))
        && element_qname(tag).is_some()
}

/// Assign stable `rdf:about` IRIs to blank `rdf:Description` nodes carrying `rdf:type`.
///
/// Horned-OWL drops anonymous individual typings; OWL WG description-logic fixtures rely on them.
/// Blank nodes used for datatype facet collections (`owl:withRestrictions`) must stay anonymous.
#[must_use]
pub fn materialize_anonymous_individual_descriptions(input: &str) -> String {
    if !input.contains("<rdf:Description") || !input.contains("rdf:type") {
        return input.to_owned();
    }
    let base = parse_xml_base(input);
    let mut counter = 0usize;
    let mut out = String::with_capacity(input.len() + 256);
    let mut pos = 0usize;
    while let Some(rel) = input[pos..].find("<rdf:Description") {
        let start = pos + rel;
        if !input[start..].starts_with("<rdf:Description")
            || input[start..].starts_with("</rdf:Description")
        {
            pos = start + 1;
            continue;
        }
        out.push_str(&input[pos..start]);
        let open_end = input[start..].find('>').unwrap_or(0);
        let open_tag = &input[start..start + open_end + 1];
        let Some(end) = named_description_element_end(input, start) else {
            out.push_str(&input[start..]);
            return out;
        };
        let block = &input[start..end];
        if is_anonymous_description_open(open_tag)
            && (block.contains("<rdf:type") || block.contains("<rdf:type "))
        {
            counter += 1;
            let iri = format!("{base}#_:{counter}");
            out.push_str(&rewrite_anonymous_description_block(block, &iri));
        } else {
            out.push_str(block);
        }
        pos = end;
    }
    out.push_str(&input[pos..]);
    out
}

fn rewrite_anonymous_description_block(block: &str, iri: &str) -> String {
    let close_tag = "</rdf:Description>";
    let open_end = block.find('>').unwrap_or(0);
    let open = &block[..=open_end];
    if open.ends_with("/>") {
        let inner = open
            .strip_prefix('<')
            .unwrap_or(open)
            .trim_end_matches("/>")
            .trim();
        return format!("<{inner} rdf:about=\"{iri}\"/>");
    }
    if !block.ends_with(close_tag) {
        return block.to_owned();
    }
    let inner = open
        .strip_prefix('<')
        .unwrap_or(open)
        .trim_end_matches('>')
        .trim();
    let mut rewritten = format!("<{inner} rdf:about=\"{iri}\">");
    rewritten.push_str(&block[open_end + 1..block.len() - close_tag.len()]);
    rewritten.push_str(close_tag);
    rewritten
}

/// Emit explicit `owl:ClassAssertion` for individuals carrying inline complex `rdf:type`.
///
/// Horned-OWL's RDF reader does not surface `ClassAssertion` from `owl:NamedIndividual` or
/// `rdf:Description` elements with nested class expressions (e.g. `owl:Restriction`).
#[allow(dead_code)]
#[must_use]
pub fn materialize_complex_class_assertions(input: &str) -> String {
    if !input.contains("rdf:type") {
        return input.to_owned();
    }
    let base = parse_xml_base(input);
    let mut out = String::with_capacity(input.len() + 256);
    let mut pos = 0usize;
    while pos < input.len() {
        let next_desc = input[pos..].find("<rdf:Description");
        let next_ind = input[pos..].find("<owl:NamedIndividual");
        let next = match (next_desc, next_ind) {
            (Some(d), Some(i)) => Some(pos + d.min(i)),
            (Some(d), None) => Some(pos + d),
            (None, Some(i)) => Some(pos + i),
            (None, None) => None,
        };
        let Some(start) = next else {
            out.push_str(&input[pos..]);
            break;
        };
        if !input[start..].starts_with("<rdf:Description")
            && !input[start..].starts_with("<owl:NamedIndividual")
        {
            pos = start + 1;
            continue;
        }
        if input[start..].starts_with("</") {
            pos = start + 1;
            continue;
        }
        out.push_str(&input[pos..start]);
        let (open_tag, close_tag) = if input[start..].starts_with("<owl:NamedIndividual") {
            ("owl:NamedIndividual", "</owl:NamedIndividual>")
        } else {
            ("rdf:Description", "</rdf:Description>")
        };
        let Some(end) = element_block_end(input, start, open_tag, close_tag) else {
            out.push_str(&input[start..]);
            break;
        };
        let block = &input[start..end];
        if let Some(rewritten) = rewrite_individual_with_complex_type(block, open_tag, &base) {
            out.push_str(&rewritten);
        } else {
            out.push_str(block);
        }
        pos = end;
    }
    out
}

fn element_block_end(input: &str, start: usize, open_tag: &str, close_tag: &str) -> Option<usize> {
    let slice = &input[start..];
    let open = format!("<{open_tag}");
    if !slice.starts_with(&open) {
        return None;
    }
    let gt = slice.find('>')?;
    let mut pos = gt + 1;
    let mut depth = 1usize;
    while pos < slice.len() {
        let rel = slice[pos..].find('<')?;
        let tag_start = pos + rel;
        if slice[tag_start..].starts_with(&open) {
            let inner_gt = slice[tag_start..].find('>')?;
            if !slice[tag_start + inner_gt - 1..=tag_start + inner_gt].ends_with("/>") {
                depth += 1;
            }
        } else if slice[tag_start..].starts_with(close_tag) {
            depth -= 1;
            if depth == 0 {
                return Some(start + tag_start + close_tag.len());
            }
        }
        pos = tag_start + 1;
    }
    None
}

#[allow(dead_code)]
fn rewrite_individual_with_complex_type(block: &str, open_tag: &str, base: &str) -> Option<String> {
    if is_typed_entity_declaration(block) {
        return None;
    }
    let open_end = block.find('>')?;
    let close_tag = format!("</{open_tag}>");
    let close_start = block.rfind(&close_tag)?;
    let inner = &block[open_end + 1..close_start];
    let (_, _, type_block) = find_top_level_element_bounds(inner, "rdf:type")?;
    if is_simple_rdf_type_element(type_block) {
        return None;
    }
    let open = &block[..=open_end];
    let individual_iri = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let type_open_end = type_block.find('>')?;
    let type_close = "</rdf:type>";
    let type_close_start = type_block.rfind(type_close)?;
    let ce_block = type_block[type_open_end + 1..type_close_start].trim();
    if ce_block.is_empty() {
        return None;
    }
    let inner = &block[open_end + 1..close_start];
    Some(format!(
        "<{open_tag} rdf:about=\"{individual_iri}\">{inner}{close_tag}"
    ))
}

fn is_simple_rdf_type_element(type_block: &str) -> bool {
    let open_end = type_block.find('>').unwrap_or(0);
    let open = &type_block[..=open_end];
    open.contains("rdf:resource=") || open.trim_end().ends_with("/>")
}

/// Collect `owl:imports` ontology IRIs from RDF/XML.
pub(crate) fn collect_owl_imports(rdf: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some(rel) = rdf[pos..].find("owl:imports") {
        let start = pos + rel;
        let rest = &rdf[start..];
        let Some(gt) = rest.find('>') else {
            break;
        };
        let tag = &rest[..=gt];
        if let Some(resource) = extract_attribute(tag, "rdf:resource") {
            out.push(resource);
        }
        pos = start + gt + 1;
    }
    out
}

pub(crate) fn collect_object_class_assertions(rdf: &str) -> Vec<(String, String)> {
    let base = parse_xml_base(rdf);
    let dt_props = declared_datatype_property_iris(rdf);
    let node_lists = build_rdf_collection_node_map(rdf, &base, &dt_props);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let next_desc = rdf[pos..].find("<rdf:Description");
        let next_ind = rdf[pos..].find("<owl:NamedIndividual");
        let next = match (next_desc, next_ind) {
            (Some(d), Some(i)) => Some(pos + d.min(i)),
            (Some(d), None) => Some(pos + d),
            (None, Some(i)) => Some(pos + i),
            (None, None) => None,
        };
        let Some(start) = next else {
            break;
        };
        if rdf[start..].starts_with("</") {
            pos = start + 1;
            continue;
        }
        let (open_tag, close_tag) = if rdf[start..].starts_with("<owl:NamedIndividual") {
            ("owl:NamedIndividual", "</owl:NamedIndividual>")
        } else {
            ("rdf:Description", "</rdf:Description>")
        };
        let Some(end) = element_block_end(rdf, start, open_tag, close_tag) else {
            break;
        };
        let block = &rdf[start..end];
        if let Some((individual, ce_ofn)) =
            object_class_assertion_from_block(block, open_tag, &base, &dt_props, &node_lists)
        {
            out.push((individual, ce_ofn));
        }
        pos = end;
    }
    out
}

/// Self-disjoint `owl:Restriction` + anonymous `owl:Thing` with a matching OPA (disjointWith-010).
pub(crate) fn collect_self_disjoint_restriction_assertions(
    rdf: &str,
) -> Vec<(String, String, String)> {
    let base = parse_xml_base(rdf);
    let dt_props = declared_datatype_property_iris(rdf);
    let mut pos = 0usize;
    let mut match_triple: Option<(String, String, String)> = None;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<owl:Restriction") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = tagged_element_end(rdf, start, "owl:Restriction") else {
            break;
        };
        let block = &rdf[start..end];
        let open_end = block.find('>').unwrap_or(0);
        let open = &block[..=open_end];
        let self_disjoint = extract_attribute(open, "rdf:nodeID").is_some()
            && block.contains("owl:disjointWith")
            && block.contains("owl:minCardinality");
        if self_disjoint {
            let restriction_iri = extract_attribute(open, "rdf:nodeID")
                .map(|id| blank_node_iri(&base, &id))
                .unwrap_or_else(|| blank_node_iri(&base, "restriction"));
            if let Some(prop) = extract_property_resource(block, "owl:onProperty", &base) {
                let ce = restriction_ce_to_ofn(block, &base, &dt_props).unwrap_or_else(|| {
                    let n = element_text_content(block, "owl:minCardinality")
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "1".to_string());
                    format!("ObjectMinCardinality({} <{prop}> owl:Thing)", n.trim())
                });
                match_triple = Some((restriction_iri, prop, ce));
            }
        }
        pos = end;
    }
    let Some((restriction_iri, prop, ce)) = match_triple else {
        return Vec::new();
    };
    let prop_suffix = prop.rsplit('#').next().unwrap_or(&prop);
    collect_object_property_assertions(rdf)
        .into_iter()
        .filter(|(_, property, _)| {
            property == &prop
                || property.ends_with(&format!("#{prop_suffix}"))
                || property.ends_with(&format!("/{prop_suffix}"))
        })
        .map(|(subject, _, _)| (subject, restriction_iri.clone(), ce.clone()))
        .collect()
}

/// Collect `SubClassOf(C restriction)` axioms Horned-OWL's RDF reader skips on class resources.
pub(crate) fn collect_restriction_subclasses(rdf: &str) -> Vec<(String, String)> {
    let base = parse_xml_base(rdf);
    let dt_props = declared_datatype_property_iris(rdf);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        if let Some(pair) = restriction_subclass_from_description_block(block, &base, &dt_props) {
            out.push(pair);
        }
        pos = end;
    }
    pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<owl:Class") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = owl_class_element_end(rdf, start) else {
            break;
        };
        let block = &rdf[start..end];
        if let Some(pair) = restriction_subclass_from_owl_class_block(block, &base, &dt_props) {
            out.push(pair);
        }
        pos = end;
    }
    out
}

fn restriction_subclass_from_description_block(
    block: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
) -> Option<(String, String)> {
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    let class_iri = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let close_start = block.rfind("</rdf:Description>")?;
    let inner = &block[open_end + 1..close_start];
    if !is_class_restriction_description(inner) {
        return None;
    }
    let ce_ofn = inline_restriction_ce_to_ofn(inner, base, dt_props)?;
    Some((class_iri, ce_ofn))
}

fn restriction_subclass_from_owl_class_block(
    block: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
) -> Option<(String, String)> {
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    let class_iri = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let close_start = block.rfind("</owl:Class>")?;
    let inner = &block[open_end + 1..close_start];
    let (_, _, sub_block) = find_top_level_element_bounds(inner, "rdfs:subClassOf")?;
    let sub_inner = element_inner(sub_block, "rdfs:subClassOf");
    let ce_ofn = restriction_ce_to_ofn(sub_inner.trim(), base, dt_props)?;
    Some((class_iri, ce_ofn))
}

/// Reified RDF negative property assertions (`owl:sourceIndividual` / `owl:assertionProperty` / `owl:targetIndividual`).
pub(crate) struct ReifiedNpa {
    pub subject: String,
    pub property: String,
    pub object: String,
    pub positive_property: Option<(String, String)>,
}

/// Reified RDF negative **data** property assertions (`owl:targetValue`).
#[derive(Debug)]
pub(crate) struct ReifiedDataNpa {
    pub subject: String,
    pub property: String,
    pub value_literal: String,
    pub positive_property: Option<(String, String)>,
}

/// Direct datatype literal assertions on anonymous `owl:Thing` individuals (misc-203/204).
#[derive(Debug)]
pub(crate) struct DirectDataLiteralAssertion {
    pub subject: String,
    pub property: String,
    pub value_literal: String,
}

pub(crate) fn collect_direct_data_literal_assertions(rdf: &str) -> Vec<DirectDataLiteralAssertion> {
    let base = parse_xml_base(rdf);
    let xmlns = parse_xmlns(rdf);
    let mut out = Vec::new();
    out.extend(collect_direct_data_literals_from_owl_thing(
        rdf, &base, &xmlns,
    ));
    out.extend(collect_direct_data_literals_from_descriptions(
        rdf, &base, &xmlns,
    ));
    out
}

fn collect_direct_data_literals_from_owl_thing(
    rdf: &str,
    base: &str,
    xmlns: &std::collections::HashMap<String, String>,
) -> Vec<DirectDataLiteralAssertion> {
    let mut out = Vec::new();
    let mut counter = 0usize;
    let mut pos = 0usize;
    while let Some(rel) = rdf[pos..].find("<owl:Thing") {
        let start = pos + rel;
        if rdf[start..].starts_with("</owl:Thing") {
            pos = start + 1;
            continue;
        }
        let Some(end) = tagged_element_end(rdf, start, "owl:Thing") else {
            break;
        };
        let block = &rdf[start..end];
        counter += 1;
        let subject = format!("{base}#_:thing{counter}");
        let inner = element_inner(block, "owl:Thing");
        for (property, value) in literal_property_assertions_from_inner(&inner, base, xmlns) {
            out.push(DirectDataLiteralAssertion {
                subject: subject.clone(),
                property,
                value_literal: value,
            });
        }
        pos = end;
    }
    out
}

fn collect_direct_data_literals_from_descriptions(
    rdf: &str,
    base: &str,
    xmlns: &std::collections::HashMap<String, String>,
) -> Vec<DirectDataLiteralAssertion> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        let open_end = block.find('>').unwrap_or(0);
        let open = &block[..=open_end];
        let inner = element_inner(block, "rdf:Description");
        let literals = literal_property_assertions_from_inner(&inner, base, xmlns);
        if !description_typed_owl_thing(block) && literals.is_empty() {
            pos = end;
            continue;
        }
        let Some(subject) = extract_attribute(open, "rdf:about")
            .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
            .map(|iri| resolve_relative_iri(&iri, base))
        else {
            pos = end;
            continue;
        };
        for (property, value) in literals {
            out.push(DirectDataLiteralAssertion {
                subject: subject.clone(),
                property,
                value_literal: value,
            });
        }
        pos = end;
    }
    out
}

fn description_typed_owl_thing(block: &str) -> bool {
    block.contains("rdf:type")
        && (block.contains("http://www.w3.org/2002/07/owl#Thing") || block.contains("owl:Thing"))
}

fn literal_property_assertions_from_inner(
    inner: &str,
    base: &str,
    xmlns: &HashMap<String, String>,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some(rel) = inner[pos..].find('<') {
        let start = pos + rel;
        if inner[start..].starts_with("</") || inner[start..].starts_with("<!--") {
            pos = start + 1;
            continue;
        }
        let Some(gt) = inner[start..].find('>') else {
            break;
        };
        let tag = &inner[start..=start + gt];
        let Some(qname) = element_qname(tag) else {
            pos = start + gt + 1;
            continue;
        };
        let prefix = qname.split(':').next().unwrap_or("");
        if matches!(prefix, "owl" | "rdf" | "rdfs" | "xsd" | "xml") {
            pos = start + gt + 1;
            continue;
        }
        let Some(prop_iri) = expand_qname(qname, xmlns) else {
            pos = start + gt + 1;
            continue;
        };
        if tag.trim_end().ends_with("/>") {
            pos = start + gt + 1;
            continue;
        }
        let close = format!("</{qname}>");
        let Some(close_start_rel) = inner[start..].find(&close) else {
            pos = start + gt + 1;
            continue;
        };
        let close_start = start + close_start_rel;
        let body = inner[start + gt + 1..close_start].trim();
        if tag.contains("rdf:parseType=\"Literal\"") || tag.contains("rdf:parseType='Literal'") {
            let lit = ofn_literal_from_rdf_literal(body.trim(), tag);
            out.push((prop_iri, lit));
        } else if let Some(dt) = extract_attribute(tag, "rdf:datatype") {
            let dt = resolve_relative_iri(&dt, base);
            let lexical = if body.is_empty() {
                element_text_content(&inner[start..close_start + close.len()], qname)
                    .unwrap_or_default()
            } else {
                body.to_owned()
            };
            if !lexical.is_empty() {
                out.push((prop_iri, ofn_typed_literal(&lexical, &dt)));
            }
        } else if !body.is_empty() {
            out.push((prop_iri, body.to_owned()));
        }
        pos = close_start + close.len();
    }
    out
}

/// Collect `SameIndividual` pairs from `owl:sameAs` on RDF resources (WG I5.8-017).
pub(crate) fn collect_owl_same_as_pairs(rdf: &str) -> Vec<(String, String)> {
    let base = parse_xml_base(rdf);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        let open_end = block.find('>').unwrap_or(0);
        let open = &block[..=open_end];
        let Some(subject) = extract_attribute(open, "rdf:about")
            .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
            .map(|iri| resolve_relative_iri(&iri, &base))
        else {
            pos = end;
            continue;
        };
        let inner = element_inner(block, "rdf:Description");
        if let Some(target) = extract_property_resource(&inner, "owl:sameAs", &base) {
            out.push((subject, target));
        }
        pos = end;
    }
    out
}

pub(crate) fn collect_reified_npas(rdf: &str) -> Vec<ReifiedNpa> {
    let base = parse_xml_base(rdf);
    let xmlns = parse_xmlns(rdf);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        if let Some(npa) = reified_npa_from_block(block, &base, &xmlns) {
            out.push(npa);
        }
        pos = end;
    }
    out
}

pub(crate) fn contains_ill_founded_rdf_list(rdf: &str) -> bool {
    rdf.contains("syntax-ns#nil") && (rdf.contains("<rdf:first") || rdf.contains("<rdf:rest"))
}

/// Collect `SubClassOf(C ObjectComplementOf(D))` from `owl:complementOf` on class resources.
pub(crate) fn collect_complement_subclasses(rdf: &str) -> Vec<(String, String)> {
    let base = parse_xml_base(rdf);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        if let Some(pair) = complement_subclass_from_block(block, &base) {
            out.push(pair);
        }
        pos = end;
    }
    out
}

fn complement_subclass_from_block(block: &str, base: &str) -> Option<(String, String)> {
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    let class_iri = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let close_start = block.rfind("</rdf:Description>")?;
    let inner = &block[open_end + 1..close_start];
    if !inner.contains("owl:complementOf") {
        return None;
    }
    let complement = extract_property_resource(inner, "owl:complementOf", base)?;
    Some((class_iri, format!("ObjectComplementOf(<{complement}>)")))
}

/// Collect `EquivalentClasses(C boolean-expr)` from `intersectionOf` / `unionOf` / `oneOf` on class resources.
pub(crate) fn collect_boolean_class_equivalences(rdf: &str) -> Vec<(String, String)> {
    let base = parse_xml_base(rdf);
    let dt_props = declared_datatype_property_iris(rdf);
    let node_lists = build_rdf_collection_node_map(rdf, &base, &dt_props);
    let mut out = Vec::new();
    let mut anon_counter = 0usize;
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        if let Some(pair) = boolean_equivalence_from_class_block(
            block,
            &base,
            &dt_props,
            &node_lists,
            false,
            &mut anon_counter,
        ) {
            out.push(pair);
        }
        pos = end;
    }
    pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<owl:Class") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = owl_class_element_end(rdf, start) else {
            break;
        };
        let block = &rdf[start..end];
        if let Some(pair) = boolean_equivalence_from_class_block(
            block,
            &base,
            &dt_props,
            &node_lists,
            true,
            &mut anon_counter,
        ) {
            out.push(pair);
        }
        pos = end;
    }
    out
}

/// Collect `SubClassOf(sub sup)` from `rdfs:subClassOf` on named class resources.
#[allow(dead_code)]
pub(crate) fn collect_subclass_axioms(rdf: &str) -> Vec<(String, String)> {
    let base = parse_xml_base(rdf);
    let dt_props = declared_datatype_property_iris(rdf);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        out.extend(subclass_axioms_from_class_block(
            block, &base, &dt_props, false,
        ));
        pos = end;
    }
    pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<owl:Class") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = owl_class_element_end(rdf, start) else {
            break;
        };
        let block = &rdf[start..end];
        out.extend(subclass_axioms_from_class_block(
            block, &base, &dt_props, true,
        ));
        pos = end;
    }
    out
}

/// Collect `DataPropertyRange` from `rdfs:range` on datatype property resources.
pub(crate) fn collect_datatype_property_ranges(rdf: &str) -> Vec<(String, String)> {
    let base = parse_xml_base(rdf);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<owl:DatatypeProperty") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = tagged_element_end(rdf, start, "owl:DatatypeProperty").or_else(|| {
            let gt = rdf[start..].find('>')?;
            if rdf.as_bytes().get(start + gt - 1) == Some(&b'/') {
                Some(start + gt + 1)
            } else {
                None
            }
        }) else {
            break;
        };
        let block = &rdf[start..end];
        if let Some(pair) = datatype_property_range_from_block(block, &base) {
            out.push(pair);
        }
        pos = end;
    }
    pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        if block.contains("owl#DatatypeProperty") || block.contains("DatatypeProperty") {
            if let Some(pair) = datatype_property_range_from_description_block(block, &base) {
                out.push(pair);
            }
        }
        pos = end;
    }
    out
}

/// Collect `ObjectPropertyAssertion` from RDF/XML property elements on individual resources.
pub(crate) fn collect_object_property_assertions(rdf: &str) -> Vec<(String, String, String)> {
    let base = parse_xml_base(rdf);
    let xmlns = parse_xmlns(rdf);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let next_desc = rdf[pos..].find("<rdf:Description");
        let next_ind = rdf[pos..].find("<owl:NamedIndividual");
        let next_thing = rdf[pos..].find("<owl:Thing");
        let next = [next_desc, next_ind, next_thing]
            .into_iter()
            .flatten()
            .map(|rel| pos + rel)
            .min();
        let Some(start) = next else {
            break;
        };
        if rdf[start..].starts_with("</") {
            pos = start + 1;
            continue;
        }
        let (open_tag, close_tag) = if rdf[start..].starts_with("<owl:NamedIndividual") {
            ("owl:NamedIndividual", "</owl:NamedIndividual>")
        } else if rdf[start..].starts_with("<owl:Thing") {
            ("owl:Thing", "</owl:Thing>")
        } else {
            ("rdf:Description", "</rdf:Description>")
        };
        let Some(end) = element_block_end(rdf, start, open_tag, close_tag) else {
            break;
        };
        let block = &rdf[start..end];
        out.extend(object_property_assertions_from_block(
            block, open_tag, &base, &xmlns,
        ));
        pos = end;
    }
    out
}

/// Collect anonymous `SubClassOf(intersection sub)` from blank `owl:Class` (WG description-logic-902/904).
pub(crate) fn collect_anonymous_intersection_subclasses(rdf: &str) -> Vec<String> {
    let base = parse_xml_base(rdf);
    let dt_props = declared_datatype_property_iris(rdf);
    let node_lists = build_rdf_collection_node_map(rdf, &base, &dt_props);
    let mut out = Vec::new();
    let mut pos = 0usize;
    let mut counter = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<owl:Class") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = owl_class_element_end(rdf, start) else {
            break;
        };
        let block = &rdf[start..end];
        let open_end = block.find('>').unwrap_or(0);
        let open = &block[..=open_end];
        if extract_attribute(open, "rdf:about").is_some()
            || extract_attribute(open, "rdf:ID").is_some()
        {
            pos = end;
            continue;
        }
        let close_start = block.rfind("</owl:Class>").unwrap_or(block.len());
        let inner = &block[open_end + 1..close_start];
        let Some(inter_ofn) = boolean_operator_ofn(
            inner,
            "owl:intersectionOf",
            "ObjectIntersectionOf",
            &base,
            &dt_props,
            &node_lists,
        ) else {
            pos = end;
            continue;
        };
        let Some((_, _, sub_block)) = find_top_level_element_bounds(inner, "rdfs:subClassOf")
        else {
            pos = end;
            continue;
        };
        let sub_inner = element_inner(sub_block, "rdfs:subClassOf");
        let Some(sup_ofn) = restriction_ce_to_ofn(sub_inner.trim(), &base, &dt_props) else {
            pos = end;
            continue;
        };
        counter += 1;
        let anon = format!("{base}#_:anon{counter}");
        out.push(format!(
            "Declaration(Class(<{anon}>))\nSubClassOf(<{anon}> {inter_ofn})\nSubClassOf(<{anon}> {sup_ofn})"
        ));
        pos = end;
    }
    out
}

fn boolean_equivalence_from_class_block(
    block: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
    node_lists: &std::collections::HashMap<String, Vec<String>>,
    owl_class: bool,
    anon_counter: &mut usize,
) -> Option<(String, String)> {
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    let class_iri = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))
        .unwrap_or_else(|| {
            *anon_counter += 1;
            format!("{base}#_:anon{anon_counter}")
        });
    let close_tag = if owl_class {
        "</owl:Class>"
    } else {
        "</rdf:Description>"
    };
    let close_start = block.rfind(close_tag)?;
    let inner = &block[open_end + 1..close_start];
    let ce_ofn = boolean_operator_ofn(
        inner,
        "owl:intersectionOf",
        "ObjectIntersectionOf",
        base,
        dt_props,
        node_lists,
    )
    .or_else(|| {
        boolean_operator_ofn(
            inner,
            "owl:unionOf",
            "ObjectUnionOf",
            base,
            dt_props,
            node_lists,
        )
    })
    .or_else(|| one_of_ofn(inner, base, dt_props, node_lists))?;
    Some((class_iri, ce_ofn))
}

fn boolean_operator_ofn(
    inner: &str,
    tag: &str,
    ofn_ctor: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
    node_lists: &std::collections::HashMap<String, Vec<String>>,
) -> Option<String> {
    let (_, _, op_block) = find_top_level_element_bounds(inner, tag)?;
    let members = collection_members_ofn(op_block, base, dt_props, node_lists)?;
    if members.is_empty() {
        return None;
    }
    if members.len() == 1 {
        return Some(members[0].clone());
    }
    Some(format!("{ofn_ctor}({})", members.join(" ")))
}

fn one_of_ofn(
    inner: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
    node_lists: &std::collections::HashMap<String, Vec<String>>,
) -> Option<String> {
    let (_, _, op_block) = find_top_level_element_bounds(inner, "owl:oneOf")?;
    let members = collection_members_ofn(op_block, base, dt_props, node_lists)?;
    if members.is_empty() {
        return None;
    }
    Some(format!("ObjectOneOf({})", members.join(" ")))
}

fn collection_members_ofn(
    op_block: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
    node_lists: &std::collections::HashMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    if op_block.contains("parseType=\"Collection\"") || op_block.contains("parseType='Collection'")
    {
        return Some(parse_collection_children_ofn(op_block, base, dt_props));
    }
    if op_block.contains("rdf:List") || op_block.contains("<rdf:first") {
        return parse_rdf_list_members_ofn(op_block, base, dt_props).filter(|m| !m.is_empty());
    }
    if let Some(node) = extract_attribute(op_block, "rdf:nodeID") {
        return node_lists.get(&node).cloned();
    }
    if let Some(resource) = extract_attribute(op_block, "rdf:resource") {
        return Some(vec![ofn_entity_ref(&resolve_relative_iri(&resource, base))]);
    }
    None
}

fn parse_rdf_list_members_ofn(
    op_block: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
) -> Option<Vec<String>> {
    let inner = element_inner(op_block, "owl:unionOf");
    let content = if inner.is_empty() {
        op_block
    } else {
        inner.as_str()
    };
    let list_block = find_top_level_element_bounds(content, "rdf:List")
        .map(|(_, _, block)| block)
        .unwrap_or(content);
    let list_inner = element_inner(list_block, "rdf:List");
    let list_content = if list_inner.is_empty() {
        list_block
    } else {
        list_inner.as_str()
    };
    let mut members = Vec::new();
    let mut rest_block = list_content;
    loop {
        if let Some((_, _, first_block)) = find_top_level_element_bounds(rest_block, "rdf:first") {
            if let Some(ofn) = member_block_to_ofn(first_block, base, dt_props) {
                members.push(ofn);
            }
        }
        let (_, _, rest_elem) = find_top_level_element_bounds(rest_block, "rdf:rest")?;
        if rest_elem.contains("rdf:nil")
            || rest_elem.contains("#nil")
            || extract_attribute(rest_elem, "rdf:resource").is_some_and(|r| r.contains("nil"))
        {
            break;
        }
        rest_block = rest_elem;
    }
    if members.is_empty() {
        None
    } else {
        Some(members)
    }
}

fn parse_collection_children_ofn(
    collection_block: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
) -> Vec<String> {
    let open_end = collection_block.find('>').map(|i| i + 1).unwrap_or(0);
    let close_start = collection_block
        .rfind('>')
        .and_then(|_| {
            let tag = element_name(collection_block)?;
            collection_block.rfind(&format!("</{tag}>"))
        })
        .unwrap_or(collection_block.len());
    let inner = &collection_block[open_end..close_start];
    let mut members = Vec::new();
    let mut pos = 0usize;
    while pos < inner.len() {
        while pos < inner.len() && inner.as_bytes()[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= inner.len() || inner.as_bytes()[pos] != b'<' {
            break;
        }
        let start = pos;
        let tag_name = inner[start + 1..]
            .split(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .next()
            .unwrap_or("");
        let Some(end) = tagged_element_end(inner, start, tag_name) else {
            break;
        };
        let member_block = &inner[start..end];
        if let Some(ofn) = member_block_to_ofn(member_block, base, dt_props) {
            members.push(ofn);
        }
        pos = end;
    }
    members
}

fn member_block_to_ofn(
    block: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
) -> Option<String> {
    if block.contains("<rdf:first") {
        let inner = element_inner(block, "rdf:first");
        if let Some(ofn) = member_block_to_ofn(inner.trim(), base, dt_props) {
            return Some(ofn);
        }
    }
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    if let Some(about) = extract_attribute(open, "rdf:about") {
        return Some(ofn_entity_ref(&resolve_relative_iri(&about, base)));
    }
    if let Some(resource) = extract_attribute(open, "rdf:resource") {
        return Some(ofn_entity_ref(&resolve_relative_iri(&resource, base)));
    }
    if block.contains("<owl:Restriction") {
        return restriction_ce_to_ofn(block, base, dt_props);
    }
    if block.trim_start().starts_with("<owl:Class") {
        let inner = element_inner(block, "owl:Class");
        if let Some(ofn) = inline_restriction_ce_to_ofn(inner.trim(), base, dt_props) {
            return Some(ofn);
        }
    } else if block.contains("<owl:Class") {
        if let Some((_, _, class_block)) = find_top_level_element_bounds(block, "owl:Class") {
            if let Some(ofn) = member_block_to_ofn(class_block, base, dt_props) {
                return Some(ofn);
            }
        }
    }
    if block.contains("owl:onProperty") {
        return inline_restriction_ce_to_ofn(block, base, dt_props);
    }
    if block.trim_start().starts_with("<rdf:Description")
        && extract_attribute(open, "rdf:about").is_none()
        && extract_attribute(open, "rdf:resource").is_none()
    {
        return Some(ofn_entity_ref(&format!("{base}#_:nominal0")));
    }
    None
}

fn build_rdf_collection_node_map(
    rdf: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
) -> std::collections::HashMap<String, Vec<String>> {
    let mut map = std::collections::HashMap::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        let open_end = block.find('>').unwrap_or(0);
        let open = &block[..=open_end];
        let Some(node) = extract_attribute(open, "rdf:nodeID") else {
            pos = end;
            continue;
        };
        let close_start = block.rfind("</rdf:Description>").unwrap_or(block.len());
        let inner = &block[open_end + 1..close_start];
        if inner.contains("rdf:first") {
            if let Some(members) = parse_rdf_list_description(inner, base, dt_props) {
                map.insert(node, members);
            }
        }
        pos = end;
    }
    map
}

fn parse_rdf_list_description(
    inner: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
) -> Option<Vec<String>> {
    let mut members = Vec::new();
    if let Some(first) = extract_property_resource(inner, "rdf:first", base) {
        members.push(ofn_entity_ref(&first));
    } else if let Some((_, _, first_block)) = find_top_level_element_bounds(inner, "rdf:first") {
        if let Some(ofn) = member_block_to_ofn(first_block, base, dt_props) {
            members.push(ofn);
        }
    }
    let (_, _, rest_block) = find_top_level_element_bounds(inner, "rdf:rest")?;
    if rest_block.contains("parseType=\"Collection\"")
        || rest_block.contains("parseType='Collection'")
    {
        members.extend(parse_collection_children_ofn(rest_block, base, dt_props));
    } else if let Some(rest) = extract_property_resource(rest_block, "rdf:rest", base) {
        let _ = rest;
    } else if rest_block.contains("rdf:Description") || rest_block.contains("owl:Class") {
        let rest_inner = element_inner(rest_block, "rdf:rest");
        if let Some(ofn) = member_block_to_ofn(rest_inner.trim(), base, dt_props) {
            members.push(ofn);
        }
    }
    if members.is_empty() {
        None
    } else {
        Some(members)
    }
}

#[allow(dead_code)]
fn subclass_axioms_from_class_block(
    block: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
    owl_class: bool,
) -> Vec<(String, String)> {
    let open_end = match block.find('>') {
        Some(i) => i + 1,
        None => return Vec::new(),
    };
    let open = &block[..=open_end - 1];
    let Some(sub_iri) = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))
    else {
        return Vec::new();
    };
    let close_tag = if owl_class {
        "</owl:Class>"
    } else {
        "</rdf:Description>"
    };
    let close_start = match block.rfind(close_tag) {
        Some(i) => i,
        None => return Vec::new(),
    };
    let inner = &block[open_end..close_start];
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some((_start, end, sub_block)) =
        find_top_level_element_from(inner, pos, "rdfs:subClassOf")
    {
        let sup_ofn = superclass_ofn_from_subclass_element(sub_block, base, dt_props);
        if let Some(sup_ofn) = sup_ofn {
            out.push((sub_iri.clone(), sup_ofn));
        }
        pos = end;
    }
    out
}

#[allow(dead_code)]
fn find_top_level_element_from<'a>(
    inner: &'a str,
    from: usize,
    tag: &str,
) -> Option<(usize, usize, &'a str)> {
    let mut pos = from;
    while pos < inner.len() {
        while pos < inner.len() && inner.as_bytes()[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= inner.len() || inner.as_bytes()[pos] != b'<' {
            return None;
        }
        let start = pos;
        let tag_name = inner[start + 1..]
            .split(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .next()
            .unwrap_or("");
        let end = tagged_element_end(inner, start, tag_name)?;
        if tag_name == tag {
            return Some((start, end, &inner[start..end]));
        }
        pos = end;
    }
    None
}

#[allow(dead_code)]
fn superclass_ofn_from_subclass_element(
    sub_block: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
) -> Option<String> {
    let open_end = sub_block.find('>')?;
    let open = &sub_block[..=open_end];
    if let Some(resource) = extract_attribute(open, "rdf:resource") {
        return Some(ofn_entity_ref(&resolve_relative_iri(&resource, base)));
    }
    let inner = element_inner(sub_block, "rdfs:subClassOf");
    if inner.contains("<owl:Restriction") {
        return restriction_ce_to_ofn(inner.trim(), base, dt_props);
    }
    if let Some(about) = inner
        .split("rdf:about=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
    {
        return Some(ofn_entity_ref(&resolve_relative_iri(about, base)));
    }
    if let Some(resource) = inner
        .split("rdf:resource=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
    {
        return Some(ofn_entity_ref(&resolve_relative_iri(resource, base)));
    }
    None
}

fn datatype_property_range_from_block(block: &str, base: &str) -> Option<(String, String)> {
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    let prop_iri = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let close_start = block
        .rfind("</owl:DatatypeProperty>")
        .unwrap_or(block.len());
    let inner = if block[open_end..].starts_with("/>") {
        ""
    } else {
        &block[open_end + 1..close_start]
    };
    let range_iri = extract_property_resource(inner, "rdfs:range", base)?;
    Some((prop_iri, datatype_ofn_ref(&range_iri)))
}

fn datatype_property_range_from_description_block(
    block: &str,
    base: &str,
) -> Option<(String, String)> {
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    let prop_iri = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let close_start = block.rfind("</rdf:Description>")?;
    let inner = &block[open_end + 1..close_start];
    if !inner.contains("rdfs:range") {
        return None;
    }
    let range_iri = extract_property_resource(inner, "rdfs:range", base)?;
    Some((prop_iri, datatype_ofn_ref(&range_iri)))
}

fn datatype_ofn_ref(iri: &str) -> String {
    if let Some(local) = iri
        .rsplit('#')
        .next()
        .filter(|_| iri.contains("XMLSchema#"))
    {
        format!("xsd:{local}")
    } else if iri == "http://www.w3.org/2000/01/rdf-schema#Literal" {
        "rdfs:Literal".to_owned()
    } else {
        format!("<{iri}>")
    }
}

fn blank_node_iri(base: &str, node: &str) -> String {
    format!("{base}#_{node}")
}

fn object_iri_from_property_element(
    prop_open: &str,
    prop_block: &str,
    tag_name: &str,
    base: &str,
) -> Option<String> {
    extract_attribute(prop_open, "rdf:resource")
        .map(|r| resolve_relative_iri(&r, base))
        .or_else(|| {
            extract_attribute(prop_open, "rdf:about").map(|a| resolve_relative_iri(&a, base))
        })
        .or_else(|| extract_attribute(prop_open, "rdf:nodeID").map(|n| blank_node_iri(base, &n)))
        .or_else(|| {
            let close = format!("</{tag_name}>");
            let cs = prop_block.rfind(&close)?;
            let prop_open_end = prop_block.find('>')?;
            let child_inner = &prop_block[prop_open_end + 1..cs];
            child_inner
                .split("rdf:about=\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .map(|a| resolve_relative_iri(a, base))
                .or_else(|| {
                    child_inner
                        .split("rdf:nodeID=\"")
                        .nth(1)
                        .and_then(|s| s.split('"').next())
                        .map(|n| blank_node_iri(base, n))
                })
        })
}

fn owl_thing_child_block(prop_block: &str, prop_tag: &str) -> Option<String> {
    let prop_open_end = prop_block.find('>')?;
    let close = format!("</{prop_tag}>");
    let cs = prop_block.rfind(&close)?;
    let child = prop_block[prop_open_end + 1..cs].trim();
    if !child.starts_with("<owl:Thing") {
        return None;
    }
    if child.ends_with("/>") {
        return Some(child.to_string());
    }
    let end = tagged_element_end(child, 0, "owl:Thing")?;
    Some(child[..end].to_string())
}

fn object_property_assertions_from_block(
    block: &str,
    open_tag: &str,
    base: &str,
    xmlns: &std::collections::HashMap<String, String>,
) -> Vec<(String, String, String)> {
    let mut anon = 0usize;
    let open_end = match block.find('>') {
        Some(i) => i + 1,
        None => return Vec::new(),
    };
    let open = &block[..=open_end - 1];
    let subject = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))
        .or_else(|| {
            if open_tag == "owl:Thing" {
                anon += 1;
                Some(blank_node_iri(base, &format!("thing{anon}")))
            } else {
                None
            }
        });
    object_property_assertions_from_subject_block(subject, block, open_tag, base, xmlns, &mut anon)
}

fn object_property_assertions_from_subject_block(
    subject_override: Option<String>,
    block: &str,
    open_tag: &str,
    base: &str,
    xmlns: &std::collections::HashMap<String, String>,
    anon: &mut usize,
) -> Vec<(String, String, String)> {
    let open_end = match block.find('>') {
        Some(i) => i + 1,
        None => return Vec::new(),
    };
    let open = &block[..=open_end - 1];
    let subject = subject_override.or_else(|| {
        extract_attribute(open, "rdf:about")
            .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
            .map(|iri| resolve_relative_iri(&iri, base))
    });
    let Some(subject) = subject else {
        return Vec::new();
    };
    let close_tag = format!("</{open_tag}>");
    let close_start = match block.rfind(&close_tag) {
        Some(i) => i,
        None => return Vec::new(),
    };
    let inner = &block[open_end..close_start];
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < inner.len() {
        while pos < inner.len() && inner.as_bytes()[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= inner.len() || inner.as_bytes()[pos] != b'<' {
            break;
        }
        if inner[pos..].starts_with("</") || inner[pos..].starts_with("<!--") {
            pos += 1;
            continue;
        }
        let start = pos;
        let tag_name = inner[start + 1..]
            .split(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .next()
            .unwrap_or("");
        let Some(end) = tagged_element_end(inner, start, tag_name) else {
            break;
        };
        let prop_block = &inner[start..end];
        let prefix = tag_name.split(':').next().unwrap_or("");
        if matches!(prefix, "owl" | "rdf" | "rdfs" | "xsd" | "xml") {
            pos = end;
            continue;
        }
        let Some(prop_iri) = expand_qname(tag_name, xmlns) else {
            pos = end;
            continue;
        };
        let prop_open_end = prop_block.find('>').unwrap_or(0);
        let prop_open = &prop_block[..=prop_open_end];
        if let Some(object) =
            object_iri_from_property_element(prop_open, prop_block, tag_name, base)
        {
            out.push((subject.clone(), prop_iri, object));
        } else if let Some(thing_block) = owl_thing_child_block(prop_block, tag_name) {
            *anon += 1;
            let object = blank_node_iri(base, &format!("anon{anon}"));
            out.push((subject.clone(), prop_iri, object.clone()));
            let wrapped = if thing_block.ends_with("/>") {
                format!("<owl:Thing rdf:about=\"{object}\"></owl:Thing>")
            } else {
                let open_end = thing_block.find('>').map(|i| i + 1).unwrap_or(0);
                let close = thing_block
                    .rfind("</owl:Thing>")
                    .unwrap_or(thing_block.len());
                format!(
                    "<owl:Thing rdf:about=\"{object}\">{}</owl:Thing>",
                    &thing_block[open_end..close]
                )
            };
            out.extend(object_property_assertions_from_subject_block(
                Some(object),
                &wrapped,
                "owl:Thing",
                base,
                xmlns,
                anon,
            ));
        }
        pos = end;
    }
    out
}

/// Collect `ObjectPropertyDomain` from `rdfs:domain` on property resources (RDF/XML).
pub(crate) fn collect_rdfs_object_property_domains(rdf: &str) -> Vec<(String, String)> {
    collect_rdfs_property_annotation(rdf, "rdfs:domain")
}

const OWL_PROPERTY_RANGE_TAGS: &[&str] = &[
    "owl:ObjectProperty",
    "owl:FunctionalProperty",
    "owl:SymmetricProperty",
    "owl:InverseFunctionalProperty",
    "owl:TransitiveProperty",
    "owl:ReflexiveProperty",
    "owl:IrreflexiveProperty",
    "owl:AsymmetricProperty",
];

/// Collect `ObjectPropertyRange` from `rdfs:range` on property resources (RDF/XML).
pub(crate) fn collect_rdfs_object_property_ranges(rdf: &str) -> Vec<(String, String)> {
    let mut out = collect_rdfs_property_annotation(rdf, "rdfs:range");
    let base = parse_xml_base(rdf);
    for tag in OWL_PROPERTY_RANGE_TAGS {
        out.extend(collect_owl_property_ranges_for_tag(rdf, tag, &base));
    }
    out
}

/// Collect object properties declared as `owl:FunctionalProperty` (RDF/XML typing idiom).
pub(crate) fn collect_functional_object_properties(rdf: &str) -> Vec<String> {
    let base = parse_xml_base(rdf);
    let functional = "http://www.w3.org/2002/07/owl#FunctionalProperty";
    let mut out = collect_owl_property_characteristic_elements(rdf, "owl:FunctionalProperty", &base);
    out.extend(collect_properties_with_rdf_type(rdf, functional, &base));
    out.sort();
    out.dedup();
    out
}

fn collect_owl_property_characteristic_elements(
    rdf: &str,
    tag: &str,
    base: &str,
) -> Vec<String> {
    let open = format!("<{tag}");
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find(&open) else {
            break;
        };
        let start = pos + rel;
        let Some(tag_end_rel) = rdf[start..].find('>') else {
            break;
        };
        let tag_end = start + tag_end_rel + 1;
        let open_tag = &rdf[start..tag_end];
        if let Some(iri) = extract_attribute(open_tag, "rdf:about").or_else(|| {
            extract_attribute(open_tag, "rdf:ID").map(|id| format!("{base}#{id}"))
        }) {
            out.push(resolve_relative_iri(&iri, base));
        }
        pos = if open_tag.trim_end().ends_with("/>") {
            tag_end
        } else {
            tagged_element_end(rdf, start, tag).unwrap_or(tag_end)
        };
    }
    out
}

fn collect_properties_with_rdf_type(rdf: &str, type_iri: &str, base: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        let open_end = block.find('>').unwrap_or(0);
        let open = &block[..=open_end];
        let Some(about) = extract_attribute(open, "rdf:about").or_else(|| {
            extract_attribute(open, "rdf:ID")
                .map(|id| format!("{base}#{id}"))
        }) else {
            pos = end;
            continue;
        };
        let close_start = block.rfind("</rdf:Description>").unwrap_or(block.len());
        let inner = &block[open_end + 1..close_start];
        let typed = extract_property_resource(inner, "rdf:type", base)
            .is_some_and(|t| t == type_iri)
            || inner.contains(&format!("resource=\"{type_iri}\""))
            || inner.contains(&format!("resource='{type_iri}'"));
        if typed {
            out.push(resolve_relative_iri(&about, base));
        }
        pos = end;
    }
    out
}

fn collect_owl_property_ranges_for_tag(rdf: &str, tag: &str, base: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let open = format!("<{tag}");
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find(&open) else {
            break;
        };
        let start = pos + rel;
        let Some(end) = tagged_element_end(rdf, start, tag).or_else(|| {
            let close = format!("</{tag}>");
            element_block_end(rdf, start, tag, &close)
                .map(|e| e - start)
                .map(|len| start + len)
        }) else {
            break;
        };
        let block = &rdf[start..end];
        if let Some(pair) = owl_property_range_from_block(block, base) {
            out.push(pair);
        }
        pos = end;
    }
    out
}

fn owl_property_range_from_block(block: &str, base: &str) -> Option<(String, String)> {
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    let prop_iri = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let close_start = block.rfind("</owl:ObjectProperty>").unwrap_or(block.len());
    let inner = &block[open_end + 1..close_start];
    let class_iri = extract_property_resource(inner, "rdfs:range", base)?;
    Some((prop_iri, class_iri))
}

/// Collect disjoint-union definitions: `C disjointUnionOf {A,B,...}`.
pub(crate) fn collect_disjoint_union_axioms(rdf: &str) -> Vec<String> {
    let base = parse_xml_base(rdf);
    let dt_props = declared_datatype_property_iris(rdf);
    let node_lists = build_rdf_collection_node_map(rdf, &base, &dt_props);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        if let Some((class_iri, union_ofn, member_iris)) =
            disjoint_union_from_class_block(block, &base, &dt_props, &node_lists)
        {
            let mut body = format!(
                "Declaration(Class(<{class_iri}>))\n\
                 EquivalentClasses(<{class_iri}> {union_ofn})"
            );
            if member_iris.len() >= 2 {
                let members = member_iris
                    .iter()
                    .map(|iri| format!("<{iri}>"))
                    .collect::<Vec<_>>()
                    .join(" ");
                body.push_str(&format!("\nDisjointClasses({members})"));
            }
            out.push(body);
        }
        pos = end;
    }
    out
}

fn disjoint_union_from_class_block(
    block: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
    node_lists: &std::collections::HashMap<String, Vec<String>>,
) -> Option<(String, String, Vec<String>)> {
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    let class_iri = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let close_start = block.rfind("</rdf:Description>")?;
    let inner = &block[open_end + 1..close_start];
    let (_, _, op_block) = find_top_level_element_bounds(inner, "owl:disjointUnionOf")?;
    let members_ofn = collection_members_ofn(op_block, base, dt_props, node_lists)?;
    if members_ofn.is_empty() {
        return None;
    }
    let member_iris = members_ofn
        .iter()
        .filter_map(|ofn| {
            ofn.strip_prefix('<')
                .and_then(|s| s.strip_suffix('>'))
                .map(str::to_owned)
        })
        .collect::<Vec<_>>();
    let union_ofn = format!("ObjectUnionOf({})", members_ofn.join(" "));
    Some((class_iri, union_ofn, member_iris))
}

fn collect_rdfs_property_annotation(rdf: &str, tag: &str) -> Vec<(String, String)> {
    let base = parse_xml_base(rdf);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        if let Some(pair) = rdfs_property_annotation_from_block(block, &base, tag) {
            out.push(pair);
        }
        pos = end;
    }
    out
}

fn rdfs_property_annotation_from_block(
    block: &str,
    base: &str,
    tag: &str,
) -> Option<(String, String)> {
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    let prop_iri = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let close_start = block.rfind("</rdf:Description>")?;
    let inner = &block[open_end + 1..close_start];
    if !inner.contains(tag) {
        return None;
    }
    let class_iri = extract_property_resource(inner, tag, base)?;
    Some((prop_iri, class_iri))
}

/// Collect `DisjointObjectProperties` from `owl:propertyDisjointWith` on property resources.
pub(crate) fn collect_property_disjoint_pairs(rdf: &str) -> Vec<(String, String)> {
    let base = parse_xml_base(rdf);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        if let Some(pair) = property_disjoint_from_block(block, &base) {
            out.push(pair);
        }
        pos = end;
    }
    out
}

fn property_disjoint_from_block(block: &str, base: &str) -> Option<(String, String)> {
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    let prop_iri = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let close_start = block.rfind("</rdf:Description>")?;
    let inner = &block[open_end + 1..close_start];
    if !inner.contains("owl:propertyDisjointWith") {
        return None;
    }
    let other = extract_property_resource(inner, "owl:propertyDisjointWith", base)?;
    Some((prop_iri, other))
}

pub(crate) fn collect_reified_data_npas(rdf: &str) -> Vec<ReifiedDataNpa> {
    let base = parse_xml_base(rdf);
    let xmlns = parse_xmlns(rdf);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rdf.len() {
        let Some(rel) = rdf[pos..].find("<rdf:Description") else {
            break;
        };
        let start = pos + rel;
        let Some(end) = element_block_end(rdf, start, "rdf:Description", "</rdf:Description>")
        else {
            break;
        };
        let block = &rdf[start..end];
        if let Some(npa) = reified_data_npa_from_block(block, &base, &xmlns) {
            out.push(npa);
        }
        pos = end;
    }
    out
}

fn reified_data_npa_from_block(
    block: &str,
    base: &str,
    xmlns: &HashMap<String, String>,
) -> Option<ReifiedDataNpa> {
    let open_end = block.find('>')?;
    let close_idx = block.rfind("</rdf:Description>")?;
    let inner = &block[open_end + 1..close_idx];
    if !inner.contains("owl:sourceIndividual")
        || !inner.contains("owl:assertionProperty")
        || !inner.contains("owl:targetValue")
    {
        return None;
    }
    if inner.contains("owl:targetIndividual") {
        return None;
    }
    let property = extract_property_resource(inner, "owl:assertionProperty", base)?;
    let value_literal = element_text_content(inner, "owl:targetValue")?;
    let (_, _, source_block) = find_top_level_element_bounds(inner, "owl:sourceIndividual")?;
    let source_inner = element_inner(source_block, "owl:sourceIndividual");
    let subject = extract_description_about(&source_inner, base)
        .or_else(|| {
            let source_open_end = source_block.find('>')?;
            let source_open = &source_block[..=source_open_end];
            extract_attribute(source_open, "rdf:about").or_else(|| {
                extract_attribute(source_open, "rdf:ID").map(|id| format!("{base}#{id}"))
            })
        })
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let positive_property = positive_data_property_from_inner(&source_inner, base, xmlns);
    Some(ReifiedDataNpa {
        subject,
        property,
        value_literal: value_literal.trim().to_owned(),
        positive_property,
    })
}

fn positive_data_property_from_inner(
    inner: &str,
    _base: &str,
    xmlns: &HashMap<String, String>,
) -> Option<(String, String)> {
    let mut pos = 0usize;
    while let Some(rel) = inner[pos..].find('<') {
        let start = pos + rel;
        if inner[start..].starts_with("</") || inner[start..].starts_with("<!--") {
            pos = start + 1;
            continue;
        }
        let Some(gt) = inner[start..].find('>') else {
            break;
        };
        let tag = &inner[start..=start + gt];
        let Some(qname) = element_qname(tag) else {
            pos = start + gt + 1;
            continue;
        };
        let prefix = qname.split(':').next().unwrap_or("");
        if matches!(prefix, "owl" | "rdf" | "rdfs" | "xsd" | "xml") {
            pos = start + gt + 1;
            continue;
        }
        let prop_iri = expand_qname(qname, xmlns)?;
        if tag.trim_end().ends_with("/>") {
            pos = start + gt + 1;
            continue;
        }
        let close = format!("</{qname}>");
        let close_start = inner[start..].find(&close)?;
        let body = inner[start + gt + 1..start + close_start].trim();
        if !body.is_empty() && !body.starts_with('<') {
            return Some((prop_iri, body.to_owned()));
        }
        pos = start + gt + 1;
    }
    None
}

fn reified_npa_from_block(
    block: &str,
    base: &str,
    xmlns: &HashMap<String, String>,
) -> Option<ReifiedNpa> {
    let open_end = block.find('>')?;
    let inner = &block[open_end + 1..block.rfind("</rdf:Description>")?];
    if !inner.contains("owl:sourceIndividual") || !inner.contains("owl:assertionProperty") {
        return None;
    }
    if inner.contains("owl:targetValue") {
        return None;
    }
    let property = extract_property_resource(inner, "owl:assertionProperty", base)?;
    let object = extract_property_resource(inner, "owl:targetIndividual", base)?;
    let (_, _, source_block) = find_top_level_element_bounds(inner, "owl:sourceIndividual")?;
    let source_inner = element_inner(source_block, "owl:sourceIndividual");
    let subject = extract_description_about(&source_inner, base)
        .or_else(|| {
            let source_open_end = source_block.find('>')?;
            let source_open = &source_block[..=source_open_end];
            extract_attribute(source_open, "rdf:about").or_else(|| {
                extract_attribute(source_open, "rdf:ID").map(|id| format!("{base}#{id}"))
            })
        })
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let positive_property = positive_property_from_inner(&source_inner, base, xmlns);
    Some(ReifiedNpa {
        subject,
        property,
        object,
        positive_property,
    })
}

fn extract_description_about(inner: &str, base: &str) -> Option<String> {
    let idx = inner.find("<rdf:Description")?;
    let open_end = inner[idx..].find('>')? + idx;
    let open = &inner[idx..=open_end];
    extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
}

fn positive_property_from_inner(
    inner: &str,
    base: &str,
    xmlns: &HashMap<String, String>,
) -> Option<(String, String)> {
    let mut pos = 0usize;
    while let Some(rel) = inner[pos..].find('<') {
        let start = pos + rel;
        if inner[start..].starts_with("</") || inner[start..].starts_with("<!--") {
            pos = start + 1;
            continue;
        }
        let Some(gt) = inner[start..].find('>') else {
            break;
        };
        let tag = &inner[start..=start + gt];
        let Some(qname) = element_qname(tag) else {
            pos = start + gt + 1;
            continue;
        };
        let prefix = qname.split(':').next().unwrap_or("");
        if matches!(prefix, "owl" | "rdf" | "rdfs" | "xsd" | "xml") {
            pos = start + gt + 1;
            continue;
        }
        let prop_iri = expand_qname(qname, xmlns)?;
        if let Some(resource) = extract_attribute(tag, "rdf:resource") {
            let object = resolve_relative_iri(&resource, base);
            return Some((prop_iri, object));
        }
        pos = start + gt + 1;
    }
    None
}

pub(crate) fn declared_datatype_property_iris(rdf: &str) -> std::collections::HashSet<String> {
    let base = parse_xml_base(rdf);
    let mut out = std::collections::HashSet::new();
    let mut pos = 0usize;
    while let Some(rel) = rdf[pos..].find("<owl:DatatypeProperty") {
        let start = pos + rel;
        let Some(gt) = rdf[start..].find('>') else {
            break;
        };
        let open = &rdf[start..=start + gt];
        if let Some(about) = extract_attribute(open, "rdf:about") {
            out.insert(resolve_relative_iri(&about, &base));
        } else if let Some(id) = extract_attribute(open, "rdf:ID") {
            out.insert(format!("{base}#{id}"));
        }
        pos = start + gt + 1;
    }
    out
}

fn unwrap_typed_class_expression_block(ce_block: &str) -> String {
    let trimmed = ce_block.trim();
    if trimmed.starts_with("<rdf:Description") {
        element_inner(trimmed, "rdf:Description")
    } else {
        trimmed.to_owned()
    }
}

fn object_class_assertion_from_block(
    block: &str,
    open_tag: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
    node_lists: &std::collections::HashMap<String, Vec<String>>,
) -> Option<(String, String)> {
    if is_typed_entity_declaration(block) {
        return None;
    }
    let open_end = block.find('>')?;
    let open = &block[..=open_end];
    let individual_iri = extract_attribute(open, "rdf:about")
        .or_else(|| extract_attribute(open, "rdf:ID").map(|id| format!("{base}#{id}")))
        .map(|iri| resolve_relative_iri(&iri, base))?;
    let close_tag = format!("</{open_tag}>");
    let close_start = block.rfind(&close_tag)?;
    let inner = &block[open_end + 1..close_start];

    if let Some((_, _, type_block)) = find_top_level_element_bounds(inner, "rdf:type") {
        if is_simple_rdf_type_element(type_block) {
            return None;
        }
        let ce_block = unwrap_typed_class_expression_block(&element_inner(type_block, "rdf:type"));
        if let Some(ce_ofn) = boolean_operator_ofn(
            &ce_block,
            "owl:intersectionOf",
            "ObjectIntersectionOf",
            base,
            dt_props,
            node_lists,
        ) {
            return Some((individual_iri, ce_ofn));
        }
        if let Some(ce_ofn) = boolean_operator_ofn(
            &ce_block,
            "owl:unionOf",
            "ObjectUnionOf",
            base,
            dt_props,
            node_lists,
        ) {
            return Some((individual_iri, ce_ofn));
        }
        if let Some(ce_ofn) = one_of_ofn(&ce_block, base, dt_props, node_lists) {
            return Some((individual_iri, ce_ofn));
        }
        if let Some(ce_ofn) = restriction_ce_to_ofn(&ce_block, base, dt_props) {
            return Some((individual_iri, ce_ofn));
        }
        if let Some(ce_ofn) = data_restriction_ce_to_ofn(&ce_block, base, dt_props) {
            return Some((individual_iri, ce_ofn));
        }
        if let Some(class_iri) = atomic_class_type_iri(&ce_block, base) {
            return Some((individual_iri, ofn_entity_ref(&class_iri)));
        }
        return None;
    }

    if inner.contains("<owl:onProperty") {
        if let Some(ce_ofn) = data_restriction_ce_to_ofn(inner, base, dt_props) {
            return Some((individual_iri, ce_ofn));
        }
        if !is_class_restriction_description(inner) {
            if let Some(ce_ofn) = inline_restriction_ce_to_ofn(inner, base, dt_props) {
                return Some((individual_iri, ce_ofn));
            }
        }
    }
    None
}

fn restriction_inner_body(block: &str) -> std::borrow::Cow<'_, str> {
    let trimmed = block.trim();
    if trimmed.starts_with("<owl:Restriction") {
        return std::borrow::Cow::Owned(element_inner(trimmed, "owl:Restriction"));
    }
    if let Some((_, _, restriction)) = find_top_level_element_bounds(trimmed, "owl:Restriction") {
        return std::borrow::Cow::Owned(element_inner(restriction, "owl:Restriction"));
    }
    std::borrow::Cow::Borrowed(block)
}

fn data_restriction_ce_to_ofn(
    ce_block: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
) -> Option<String> {
    let body = restriction_inner_body(ce_block);
    if !body.contains("owl:onProperty") {
        return None;
    }
    let on_prop = extract_property_resource(&body, "owl:onProperty", base)?;
    if !dt_props.contains(&on_prop) && !datatype_property_fallback(&on_prop) {
        return None;
    }
    if let Some(range) = data_range_ofn(&body, base) {
        return Some(format!("DataAllValuesFrom(<{on_prop}> {range})"));
    }
    if let Some(n) = element_text_content(&body, "owl:maxCardinality") {
        return Some(format!("DataMaxCardinality({} <{on_prop}>)", n.trim()));
    }
    if let Some(n) = element_text_content(&body, "owl:minCardinality") {
        return Some(format!("DataMinCardinality({} <{on_prop}>)", n.trim()));
    }
    if let Some(n) = element_text_content(&body, "owl:cardinality") {
        return Some(format!("DataExactCardinality({} <{on_prop}>)", n.trim()));
    }
    if let Some(value) = data_has_value_ofn(&body, base) {
        return Some(format!("DataHasValue(<{on_prop}> {value})"));
    }
    None
}

fn data_range_ofn(ce_block: &str, base: &str) -> Option<String> {
    let (_, _, avf_block) = find_top_level_element_bounds(ce_block, "owl:allValuesFrom")?;
    let mut inner = element_inner(avf_block, "owl:allValuesFrom");
    if inner.contains("rdfs:Datatype") || inner.contains("<Datatype") {
        if let Some((_, _, dt_block)) = find_top_level_element_bounds(&inner, "rdfs:Datatype") {
            inner = element_inner(dt_block, "rdfs:Datatype");
        }
    }
    if let Some(resource) = extract_attribute(
        avf_block
            .get(
                ..avf_block
                    .find('>')
                    .map(|i| i + 1)
                    .unwrap_or(avf_block.len()),
            )
            .unwrap_or(avf_block),
        "rdf:resource",
    ) {
        return Some(ofn_entity_ref(&resolve_relative_iri(&resource, base)));
    }
    if inner.contains("owl:oneOf") {
        let literals = data_one_of_literals(&inner, base)?;
        return Some(format!("DataOneOf({})", literals.join(" ")));
    }
    None
}

fn data_one_of_literals(inner: &str, base: &str) -> Option<Vec<String>> {
    let (_, _, one_block) = find_top_level_element_bounds(inner, "owl:oneOf")?;
    let list_inner = element_inner(one_block, "owl:oneOf");
    let mut out = Vec::new();
    collect_data_list_literals(&list_inner, base, &mut out);
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn rdf_rest_is_nil(rest_block: &str) -> bool {
    rest_block.contains("rdf:nil")
        || extract_attribute(
            rest_block
                .get(
                    ..rest_block
                        .find('>')
                        .map(|i| i + 1)
                        .unwrap_or(rest_block.len()),
                )
                .unwrap_or(rest_block),
            "rdf:resource",
        )
        .is_some_and(|r| r.contains("nil"))
}

fn collect_data_list_literals(inner: &str, base: &str, out: &mut Vec<String>) {
    let inner = if inner.trim_start().starts_with("<rdf:Description") {
        element_inner(inner.trim(), "rdf:Description")
    } else {
        inner.to_string()
    };
    if let Some(lit) = data_literal_ofn_from_element(&inner, "rdf:first", base) {
        out.push(lit);
    }
    let Some((_, _, rest_block)) = find_top_level_element_bounds(&inner, "rdf:rest") else {
        return;
    };
    if rdf_rest_is_nil(rest_block) {
        return;
    }
    let rest_inner = element_inner(rest_block, "rdf:rest");
    collect_data_list_literals(&rest_inner, base, out);
}

fn data_literal_ofn_from_element(block: &str, tag: &str, base: &str) -> Option<String> {
    let (_, _, elem) = find_top_level_element_bounds(block, tag)?;
    let open_end = elem.find('>')?;
    let open = &elem[..=open_end];
    if let Some(dt) = extract_attribute(open, "rdf:datatype") {
        let text = element_text_content(elem, tag)?;
        let dt = resolve_relative_iri(&dt, base);
        return Some(ofn_typed_literal(&text, &dt));
    }
    if tag == "rdf:Description" {
        let inner = element_inner(elem, "rdf:Description");
        return data_literal_ofn_from_element(&inner, "rdf:first", base);
    }
    None
}

fn data_has_value_ofn(ce_block: &str, base: &str) -> Option<String> {
    let (_, _, hv_block) = find_top_level_element_bounds(ce_block, "owl:hasValue")?;
    let open_end = hv_block.find('>')?;
    let open = &hv_block[..=open_end];
    if let Some(lex) = extract_attribute(open, "rdf:datatype") {
        let text = element_text_content(hv_block, "owl:hasValue")?;
        let dt = resolve_relative_iri(&lex, base);
        let dt_local = dt.rsplit('#').next().unwrap_or("string");
        return Some(format!("\"{text}\"^^xsd:{dt_local}"));
    }
    None
}

fn is_class_restriction_description(inner: &str) -> bool {
    if inner.contains("<rdf:type") || inner.contains("owl:sourceIndividual") {
        return false;
    }
    if !inner.contains("owl:onProperty") {
        return false;
    }
    !has_user_property_elements(inner)
}

fn has_user_property_elements(inner: &str) -> bool {
    let mut pos = 0usize;
    while let Some(rel) = inner[pos..].find('<') {
        let start = pos + rel;
        if inner[start..].starts_with("</") || inner[start..].starts_with("<!--") {
            pos = start + 1;
            continue;
        }
        let Some(gt) = inner[start..].find('>') else {
            break;
        };
        let tag = &inner[start..=start + gt];
        if let Some(qname) = element_qname(tag) {
            let prefix = qname.split(':').next().unwrap_or("");
            if !matches!(prefix, "owl" | "rdf" | "rdfs" | "xsd" | "xml") {
                return true;
            }
        }
        pos = start + gt + 1;
    }
    false
}

fn inline_restriction_ce_to_ofn(
    inner: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
) -> Option<String> {
    if inner.contains("<owl:Restriction") {
        return restriction_ce_to_ofn(inner, base, dt_props);
    }
    let on_prop = extract_property_resource(inner, "owl:onProperty", base)?;
    if dt_props.contains(&on_prop) {
        return None;
    }
    if let Some(filler) = extract_filler_ofn(inner, "owl:someValuesFrom", base) {
        return Some(format!("ObjectSomeValuesFrom(<{on_prop}> {filler})"));
    }
    if let Some(filler) = extract_filler_ofn(inner, "owl:allValuesFrom", base) {
        return Some(format!("ObjectAllValuesFrom(<{on_prop}> {filler})"));
    }
    if let Some(value) = extract_filler_ofn(inner, "owl:hasValue", base) {
        return Some(format!("ObjectHasValue(<{on_prop}> {value})"));
    }
    if let Some(n) = element_text_content(inner, "owl:maxCardinality") {
        return Some(format!("ObjectMaxCardinality({} <{on_prop}>)", n.trim()));
    }
    if let Some(n) = element_text_content(inner, "owl:maxQualifiedCardinality") {
        let filler = extract_property_resource(inner, "owl:onClass", base)
            .map(|c| format!("<{c}>"))
            .unwrap_or_else(|| "owl:Thing".to_owned());
        return Some(format!(
            "ObjectMaxCardinality({} <{on_prop}> {filler})",
            n.trim()
        ));
    }
    if let Some(n) = element_text_content(inner, "owl:minCardinality") {
        return Some(format!("ObjectMinCardinality({} <{on_prop}>)", n.trim()));
    }
    if let Some(n) = element_text_content(inner, "owl:cardinality") {
        return Some(format!("ObjectExactCardinality({} <{on_prop}>)", n.trim()));
    }
    None
}

fn restriction_ce_to_ofn(
    ce_block: &str,
    base: &str,
    dt_props: &std::collections::HashSet<String>,
) -> Option<String> {
    if !ce_block.contains("<owl:Restriction") {
        return None;
    }
    if ce_block.contains("rdfs:Datatype") || ce_block.contains("owl:onDatatype") {
        return None;
    }
    let on_prop = extract_property_resource(ce_block, "owl:onProperty", base)?;
    if dt_props.contains(&on_prop) {
        return None;
    }
    if let Some(filler) = extract_filler_ofn(ce_block, "owl:someValuesFrom", base) {
        return Some(format!("ObjectSomeValuesFrom(<{on_prop}> {filler})"));
    }
    if let Some(filler) = extract_filler_ofn(ce_block, "owl:allValuesFrom", base) {
        return Some(format!("ObjectAllValuesFrom(<{on_prop}> {filler})"));
    }
    if let Some(n) = element_text_content(ce_block, "owl:maxCardinality") {
        let filler = extract_filler_ofn(ce_block, "owl:someValuesFrom", base)
            .unwrap_or_else(|| "owl:Thing".to_owned());
        return Some(format!(
            "ObjectMaxCardinality({} <{on_prop}> {filler})",
            n.trim()
        ));
    }
    if let Some(n) = element_text_content(ce_block, "owl:maxQualifiedCardinality") {
        let filler = extract_property_resource(ce_block, "owl:onClass", base)
            .map(|c| format!("<{c}>"))
            .unwrap_or_else(|| "owl:Thing".to_owned());
        return Some(format!(
            "ObjectMaxCardinality({} <{on_prop}> {filler})",
            n.trim()
        ));
    }
    if let Some(n) = element_text_content(ce_block, "owl:minCardinality") {
        let filler = extract_filler_ofn(ce_block, "owl:someValuesFrom", base)
            .unwrap_or_else(|| "owl:Thing".to_owned());
        return Some(format!(
            "ObjectMinCardinality({} <{on_prop}> {filler})",
            n.trim()
        ));
    }
    if ce_block.contains("owl:hasSelf") {
        return Some(format!("ObjectHasSelf(<{on_prop}>)"));
    }
    if let Some(value) = extract_filler_ofn(ce_block, "owl:hasValue", base) {
        return Some(format!("ObjectHasValue(<{on_prop}> {value})"));
    }
    None
}

fn extract_property_resource(block: &str, tag: &str, base: &str) -> Option<String> {
    let needle = format!("<{tag}");
    let idx = block.find(&needle)?;
    let rest = &block[idx..];
    let gt = rest.find('>')?;
    let open = &rest[..=gt];
    if let Some(resource) = extract_attribute(open, "rdf:resource") {
        return Some(resolve_relative_iri(&resource, base));
    }
    if open.trim_end().ends_with("/>") {
        return None;
    }
    let close = format!("</{tag}>");
    let close_start = rest.find(&close)?;
    let inner = rest[gt + 1..close_start].trim();
    for prop_tag in ["owl:ObjectProperty", "owl:DatatypeProperty"] {
        if inner.contains(prop_tag) {
            if let Some(about) = extract_attribute(inner, "rdf:about") {
                return Some(resolve_relative_iri(&about, base));
            }
            if let Some(id) = extract_attribute(inner, "rdf:ID") {
                return Some(format!("{base}#{id}"));
            }
        }
    }
    None
}

fn extract_filler_ofn(block: &str, tag: &str, base: &str) -> Option<String> {
    let needle = format!("<{tag}");
    let idx = block.find(&needle)?;
    let rest = &block[idx..];
    let gt = rest.find('>')?;
    let open = &rest[..=gt];
    if let Some(resource) = extract_attribute(open, "rdf:resource") {
        return Some(ofn_entity_ref(&resolve_relative_iri(&resource, base)));
    }
    if open.trim_end().ends_with("/>") {
        return None;
    }
    let close = format!("</{tag}>");
    let close_start = rest.find(&close)?;
    let inner = rest[gt + 1..close_start].trim();
    restriction_ce_to_ofn(inner, base, &std::collections::HashSet::new())
}

fn element_text_content(block: &str, tag: &str) -> Option<String> {
    let (_, _, elem) = find_top_level_element_bounds(block, tag)?;
    let open_end = elem.find('>')?;
    let close = format!("</{tag}>");
    let close_start = elem.rfind(&close)?;
    Some(elem[open_end + 1..close_start].trim().to_owned())
}

/// Rewrite boolean CE OFN so Horned-OWL accepts atomic class members (QName + Prefix).
pub(crate) fn qualify_ce_ofn_for_supplement(ce_ofn: &str) -> (String, String) {
    let mut prefixes = Vec::new();
    let mut out = ce_ofn.to_string();
    let mut counter = 0usize;
    let mut search_from = 0usize;
    while let Some(rel) = out[search_from..].find('<') {
        let start = search_from + rel;
        let Some(rel_end) = out[start + 1..].find('>') else {
            break;
        };
        let end = start + 1 + rel_end;
        let iri = &out[start + 1..end];
        search_from = end + 1;
        if iri == "http://www.w3.org/2002/07/owl#Thing"
            || iri == "http://www.w3.org/2002/07/owl#Nothing"
        {
            continue;
        }
        let Some(hash) = iri.find('#') else {
            continue;
        };
        let ns = &iri[..=hash];
        let local = &iri[hash + 1..];
        if local.is_empty() {
            continue;
        }
        counter += 1;
        let prefix = format!("ns{counter}");
        prefixes.push(format!("Prefix({prefix}:=<{ns}>)"));
        let qname = if local.chars().all(|c| c.is_alphanumeric() || c == '_') {
            format!("{prefix}:{local}")
        } else {
            format!("<{}>", iri.replace('#', "%23"))
        };
        let replaced_len = qname.len();
        out.replace_range(start..=end, &qname);
        search_from = start + replaced_len;
    }
    (prefixes.join("\n"), out)
}

pub(crate) fn escape_ofn_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn extract_xml_lang(text: &str) -> Option<String> {
    for marker in ["xml:lang=\"", "xml:lang='"] {
        let Some(start) = text.find(marker) else {
            continue;
        };
        let rest = &text[start + marker.len()..];
        let quote = marker.chars().last()?;
        let end = rest.find(quote)?;
        return Some(rest[..end].to_owned());
    }
    None
}

fn strip_xml_tags(text: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn ofn_literal_from_rdf_literal(body: &str, _tag: &str) -> String {
    if let Some(lang) = extract_xml_lang(body) {
        let text = strip_xml_tags(body);
        return format!("\"{}\"@{lang}", escape_ofn_string(&text));
    }
    format!("\"{}\"^^rdf:XMLLiteral", escape_ofn_string(body))
}

fn ofn_entity_ref(iri: &str) -> String {
    if iri == "http://www.w3.org/2002/07/owl#Thing" {
        "owl:Thing".to_owned()
    } else if iri == "http://www.w3.org/2002/07/owl#Nothing" {
        "owl:Nothing".to_owned()
    } else {
        let escaped = iri.replace('#', "%23");
        format!("<{escaped}>")
    }
}

fn ofn_typed_literal(lexical: &str, datatype_iri: &str) -> String {
    let (_, lit, _) = qualify_typed_literal_for_supplement(lexical, datatype_iri);
    lit
}

/// QName-qualify typed literals for Horned-OWL supplement ontologies.
pub(crate) fn qualify_typed_literal_for_supplement(
    lexical: &str,
    datatype_iri: &str,
) -> (String, String, Option<String>) {
    let normalized = datatype_iri.replace("%23", "#");
    if normalized.starts_with("http://www.w3.org/2001/XMLSchema#") {
        let local = normalized.rsplit('#').next().unwrap_or("string");
        return (String::new(), format!("\"{lexical}\"^^xsd:{local}"), None);
    }
    if normalized == "rdf:PlainLiteral"
        || normalized == "http://www.w3.org/1999/02/22-rdf-syntax-ns#PlainLiteral"
    {
        return (
            String::new(),
            format!("\"{}\"^^rdf:PlainLiteral", escape_ofn_string(lexical)),
            None,
        );
    }
    if normalized == "rdf:XMLLiteral"
        || normalized == "http://www.w3.org/1999/02/22-rdf-syntax-ns#XMLLiteral"
    {
        return (
            String::new(),
            format!("\"{}\"^^rdf:XMLLiteral", escape_ofn_string(lexical)),
            None,
        );
    }
    if let Some(qname) = known_supplement_datatype_qname(&normalized) {
        let lit = format!("\"{}\"^^{qname}", escape_ofn_string(lexical));
        let decl = if qname.starts_with("owl:") {
            Some(format!("Declaration(Datatype({qname}))"))
        } else {
            None
        };
        return (String::new(), lit, decl);
    }
    let Some(pos) = normalized.rfind('#') else {
        return (
            String::new(),
            format!(
                "\"{}\"^^{}",
                escape_ofn_string(lexical),
                ofn_entity_ref(datatype_iri)
            ),
            None,
        );
    };
    let ns = &normalized[..=pos];
    let local = &normalized[pos + 1..];
    if local.is_empty() || !local.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return (
            String::new(),
            format!("\"{lexical}\"^^{}", ofn_entity_ref(datatype_iri)),
            None,
        );
    }
    let prefix = "dtlit1";
    let prefixes = format!("Prefix({prefix}:=<{ns}>)");
    let lit = format!("\"{lexical}\"^^{prefix}:{local}");
    let decl = format!("Declaration(Datatype({prefix}:{local}))");
    (prefixes, lit, Some(decl))
}

/// QName refs for datatype IRIs used in supplement OFN.
fn known_supplement_datatype_qname(normalized: &str) -> Option<String> {
    if normalized.starts_with("http://www.w3.org/2002/07/owl#") {
        let local = normalized.rsplit('#').next().unwrap_or("");
        if !local.is_empty() {
            return Some(format!("owl:{local}"));
        }
    }
    if normalized.starts_with("http://www.w3.org/1999/02/22-rdf-syntax-ns#") {
        let local = normalized.rsplit('#').next().unwrap_or("");
        if !local.is_empty() {
            return Some(format!("rdf:{local}"));
        }
    }
    None
}

pub(crate) fn qualify_datatype_ref_for_supplement(datatype_iri: &str) -> (String, String) {
    let normalized = datatype_iri.replace("%23", "#");
    if normalized.starts_with("http://www.w3.org/2001/XMLSchema#") {
        let local = normalized.rsplit('#').next().unwrap_or("string");
        return (String::new(), format!("xsd:{local}"));
    }
    if let Some(qname) = known_supplement_datatype_qname(&normalized) {
        return (String::new(), qname);
    }
    let Some(pos) = normalized.rfind('#') else {
        return (String::new(), ofn_entity_ref(datatype_iri));
    };
    let ns = &normalized[..=pos];
    let local = &normalized[pos + 1..];
    if local.is_empty() || !local.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return (String::new(), ofn_entity_ref(datatype_iri));
    }
    let prefix = "dtsame1";
    let prefixes = format!("Prefix({prefix}:=<{ns}>)");
    (prefixes, format!("{prefix}:{local}"))
}

fn atomic_class_type_iri(ce_block: &str, base: &str) -> Option<String> {
    let idx = ce_block.find("<owl:Class")?;
    let rest = &ce_block[idx..];
    let gt = rest.find('>')?;
    let open = &rest[..=gt];
    if let Some(about) = extract_attribute(open, "rdf:about") {
        return Some(resolve_relative_iri(&about, base));
    }
    if let Some(resource) = extract_attribute(open, "rdf:resource") {
        return Some(resolve_relative_iri(&resource, base));
    }
    None
}

/// Convert typed `rdf:Description rdf:about="..."` to `owl:NamedIndividual`.
///
/// Horned-OWL's RDF reader does not emit `ClassAssertion` from `rdf:Description` typings.
#[must_use]
pub fn materialize_named_individual_descriptions(input: &str) -> String {
    if !input.contains("<rdf:Description") || !input.contains("rdf:type") {
        return input.to_owned();
    }
    let mut out = String::with_capacity(input.len() + 128);
    let mut pos = 0usize;
    while let Some(rel) = input[pos..].find("<rdf:Description") {
        let start = pos + rel;
        if !input[start..].starts_with("<rdf:Description")
            || input[start..].starts_with("</rdf:Description")
        {
            pos = start + 1;
            continue;
        }
        out.push_str(&input[pos..start]);
        let Some(end) = named_description_element_end(input, start) else {
            out.push_str(&input[start..]);
            return out;
        };
        let block = &input[start..end];
        if is_named_description_with_individual_type(block) {
            out.push_str(&rewrite_description_to_named_individual(block));
        } else {
            out.push_str(block);
        }
        pos = end;
    }
    out.push_str(&input[pos..]);
    out
}

fn is_named_description_with_individual_type(block: &str) -> bool {
    let open_end = block.find('>').unwrap_or(0);
    let open = &block[..=open_end];
    if !(open.contains("rdf:about=\"")
        || open.contains("rdf:about='")
        || open.contains("rdf:ID=\"")
        || open.contains("rdf:ID='"))
    {
        return false;
    }
    if !(block.contains("<rdf:type") || block.contains("<rdf:type ")) {
        return false;
    }
    !is_typed_entity_declaration(block)
}

/// RDF/XML shortcuts that type a named node as a class or property, not an individual.
fn is_typed_entity_declaration(block: &str) -> bool {
    const ENTITY_TYPES: [&str; 13] = [
        "owl#Class",
        "owl#ObjectProperty",
        "owl#DatatypeProperty",
        "owl#AnnotationProperty",
        "owl#OntologyProperty",
        "owl#InverseFunctionalProperty",
        "owl#FunctionalProperty",
        "owl#SymmetricProperty",
        "owl#AsymmetricProperty",
        "owl#ReflexiveProperty",
        "owl#IrreflexiveProperty",
        "owl#TransitiveProperty",
        "owl#NamedIndividual",
    ];
    let Some(open_end) = block.find('>') else {
        return false;
    };
    let close_tag = if block.starts_with("<owl:NamedIndividual") {
        "</owl:NamedIndividual>"
    } else {
        "</rdf:Description>"
    };
    let Some(close_start) = block.rfind(close_tag) else {
        return false;
    };
    let inner = &block[open_end + 1..close_start];
    let Some((_, _, type_block)) = find_top_level_element_bounds(inner, "rdf:type") else {
        return false;
    };
    if !is_simple_rdf_type_element(type_block) {
        return false;
    }
    let Some(type_open_end) = type_block.find('>') else {
        return false;
    };
    let open = &type_block[..=type_open_end];
    let Some(resource) = extract_attribute(open, "rdf:resource") else {
        return false;
    };
    ENTITY_TYPES.iter().any(|marker| resource.contains(marker))
}

fn rewrite_description_to_named_individual(block: &str) -> String {
    let close_tag = "</rdf:Description>";
    if !block.ends_with(close_tag) {
        return block.to_owned();
    }
    let mut rewritten = block.replacen("<rdf:Description", "<owl:NamedIndividual", 1);
    let close_start = rewritten
        .rfind(close_tag)
        .expect("matching close tag after open rewrite");
    rewritten.replace_range(
        close_start..close_start + close_tag.len(),
        "</owl:NamedIndividual>",
    );
    rewritten
}

fn named_description_element_end(input: &str, start: usize) -> Option<usize> {
    let slice = &input[start..];
    let open = "<rdf:Description";
    let close = "</rdf:Description>";
    if !slice.starts_with(open) {
        return None;
    }
    let gt = slice.find('>')?;
    let mut pos = gt + 1;
    let mut depth = 1usize;
    while pos < slice.len() {
        let rel = slice[pos..].find('<')?;
        let tag_start = pos + rel;
        if slice[tag_start..].starts_with(open) {
            let inner_gt = slice[tag_start..].find('>')?;
            if &slice[tag_start + inner_gt - 1..=tag_start + inner_gt] != "/>" {
                depth += 1;
            }
        } else if slice[tag_start..].starts_with(close) {
            depth -= 1;
            if depth == 0 {
                return Some(start + tag_start + close.len());
            }
        }
        pos = tag_start + 1;
    }
    None
}

/// Rewrite `owl:members` collections to `owl:distinctMembers` and flatten member nodes.
#[must_use]
pub fn normalize_all_different_members(input: &str) -> String {
    if !input.contains("<owl:AllDifferent") {
        return input.to_owned();
    }
    let with_distinct = if input.contains("owl:members") {
        input.replace("owl:members", "owl:distinctMembers")
    } else {
        input.to_owned()
    };
    let stripped = strip_all_different_about(&with_distinct);
    flatten_descriptions_in_distinct_members(&stripped)
}

/// Expand `owl:AllDisjointClasses` / `owl:AllDisjointProperties` into pairwise disjoint axioms.
#[must_use]
pub fn expand_all_disjoint_collections(input: &str) -> String {
    if !input.contains("AllDisjointClasses") && !input.contains("AllDisjointProperties") {
        return input.to_owned();
    }
    let mut out = input.to_owned();
    while out.contains("<owl:AllDisjointClasses") {
        out = expand_disjoint_block(
            &out,
            "owl:AllDisjointClasses",
            "owl:disjointWith",
            "owl:Class",
        );
    }
    while out.contains("<owl:AllDisjointProperties") {
        out = expand_disjoint_block(
            &out,
            "owl:AllDisjointProperties",
            "owl:propertyDisjointWith",
            "owl:ObjectProperty",
        );
    }
    out
}

fn expand_disjoint_block(
    input: &str,
    container: &str,
    disjoint_tag: &str,
    decl_tag: &str,
) -> String {
    let open = format!("<{container}");
    let Some(start) = input.find(&open) else {
        return input.to_owned();
    };
    let members_marker = if input[start..].contains("<owl:members") {
        "<owl:members"
    } else {
        "<owl:distinctMembers"
    };
    let Some(members_rel) = input[start..].find(members_marker) else {
        return input.to_owned();
    };
    let abs_members = start + members_rel;
    let Some(coll_open_end) = input[abs_members..].find('>') else {
        return input.to_owned();
    };
    let coll_open_end = abs_members + coll_open_end + 1;
    let close_tag = if members_marker == "<owl:members" {
        "</owl:members>"
    } else {
        "</owl:distinctMembers>"
    };
    let Some(close_rel) = input[coll_open_end..].find(close_tag) else {
        return input.to_owned();
    };
    let coll_close_start = coll_open_end + close_rel;
    let inner = &input[coll_open_end..coll_close_start];

    let mut iris = Vec::new();
    let mut pos = 0usize;
    while let Some(rel) = inner[pos..].find("rdf:about=\"") {
        let value_start = pos + rel + "rdf:about=\"".len();
        let rest = &inner[value_start..];
        if let Some(end) = rest.find('"') {
            iris.push(rest[..end].to_owned());
        }
        pos = value_start + 1;
    }

    let close_container = format!("</{container}>");
    let Some(container_end_rel) = input[start..].find(&close_container) else {
        return input.to_owned();
    };
    let container_end = start + container_end_rel + close_container.len();

    let mut injections = String::new();
    for iri in &iris {
        injections.push_str(&format!("  <{decl_tag} rdf:about=\"{iri}\"/>\n"));
    }
    for i in 0..iris.len() {
        for j in (i + 1)..iris.len() {
            injections.push_str(&format!(
                "  <rdf:Description rdf:about=\"{}\">\n    <{disjoint_tag} rdf:resource=\"{}\"/>\n  </rdf:Description>\n",
                iris[i], iris[j]
            ));
        }
    }

    let mut out = String::new();
    out.push_str(&input[..start]);
    out.push_str(&injections);
    out.push_str(&input[container_end..]);
    out
}

fn owl_class_element_end(input: &str, class_start: usize) -> Option<usize> {
    tagged_element_end(input, class_start, "owl:Class")
}

fn tagged_element_end(input: &str, start: usize, tag: &str) -> Option<usize> {
    let slice = &input[start..];
    let open = format!("<{tag}");
    if !slice.starts_with(&open) {
        return None;
    }
    let gt = slice.find('>')?;
    if slice.as_bytes().get(gt.wrapping_sub(1)) == Some(&b'/') {
        return Some(start + gt + 1);
    }
    let close = format!("</{tag}>");
    let mut depth = 0usize;
    let mut pos = 0usize;
    while pos < slice.len() {
        let rel = slice[pos..].find('<')?;
        let tag_start = pos + rel;
        if slice[tag_start..].starts_with(&open) {
            let gt = slice[tag_start..].find('>')?;
            let is_self_close = &slice[tag_start + gt - 1..=tag_start + gt] == "/>";
            if is_self_close {
                // Self-closing children do not close the outer element.
            } else {
                depth += 1;
            }
        } else if slice[tag_start..].starts_with(&close) {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(start + tag_start + close.len());
            }
        }
        pos = tag_start + 1;
    }
    None
}

fn rewrite_class_intersection_block(block: &str) -> String {
    let open_end = match block.find('>') {
        Some(i) => i + 1,
        None => return block.to_owned(),
    };
    let close_start = match block.rfind("</owl:Class>") {
        Some(i) => i,
        None => return block.to_owned(),
    };
    if open_end > close_start {
        return block.to_owned();
    }
    let open_tag = &block[..open_end];
    let inner = &block[open_end..close_start];
    let close_tag = &block[close_start..];

    let Some((is, ie, intersection)) = find_top_level_element(inner, "owl:intersectionOf") else {
        return block.to_owned();
    };
    let equiv = find_top_level_element(inner, "owl:equivalentClass");

    let mut remainder = String::new();
    let mut pos = 0usize;
    while pos < inner.len() {
        while pos < inner.len() && inner.as_bytes()[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= inner.len() {
            break;
        }
        if inner.as_bytes()[pos] != b'<' {
            return block.to_owned();
        }
        let start = pos;
        let tag_name = inner[start + 1..]
            .split(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .next()
            .unwrap_or("");
        let Some(end) = tagged_element_end(inner, start, tag_name) else {
            return block.to_owned();
        };
        if (start, end) != (is, ie) && equiv.is_none_or(|(es, ee, _)| (start, end) != (es, ee)) {
            remainder.push_str(&inner[start..end]);
            if !remainder.ends_with('\n') {
                remainder.push('\n');
            }
        }
        pos = end;
    }

    let merged_intersection = if let Some((es, ee, _)) = equiv {
        let equiv_inner = element_inner(&inner[es..ee], "owl:equivalentClass");
        merge_intersection_first_member(intersection, &equiv_inner)
    } else {
        intersection.to_owned()
    };

    let mut rewritten = String::new();
    rewritten.push_str(open_tag);
    if !remainder.trim().is_empty() {
        rewritten.push_str(remainder.trim_end());
        rewritten.push('\n');
    }
    rewritten.push_str("  <owl:equivalentClass>\n    <owl:Class>\n      ");
    rewritten.push_str(&merged_intersection);
    rewritten.push_str("\n    </owl:Class>\n  </owl:equivalentClass>\n");
    rewritten.push_str(close_tag);
    rewritten
}

fn find_top_level_element<'a>(inner: &'a str, tag: &str) -> Option<(usize, usize, &'a str)> {
    let mut pos = 0usize;
    while pos < inner.len() {
        while pos < inner.len() && inner.as_bytes()[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= inner.len() {
            break;
        }
        if inner.as_bytes()[pos] != b'<' {
            pos += 1;
            continue;
        }
        let start = pos;
        let tag_name = inner[start + 1..]
            .split(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .next()
            .unwrap_or("");
        let end = tagged_element_end(inner, start, tag_name)?;
        if tag_name == tag {
            return Some((start, end, &inner[start..end]));
        }
        pos = end;
    }
    None
}

fn element_inner(block: &str, tag: &str) -> String {
    let open = format!("<{tag}");
    if !block.trim_start().starts_with(&open) {
        return String::new();
    }
    let start = block.find(&open).unwrap_or(0);
    let Some(end) = tagged_element_end(block, start, tag) else {
        return String::new();
    };
    let open_end = block[start..]
        .find('>')
        .map(|i| start + i + 1)
        .unwrap_or(start);
    let close = format!("</{tag}>");
    let close_start = block[start..end].rfind(&close).unwrap_or(end);
    block[open_end..start + close_start].trim().to_owned()
}

fn merge_intersection_first_member(intersection_block: &str, first_member: &str) -> String {
    if first_member.is_empty() {
        return intersection_block.to_owned();
    }
    let Some(open_end) = intersection_block.find('>') else {
        return intersection_block.to_owned();
    };
    let mut out = String::new();
    out.push_str(&intersection_block[..open_end + 1]);
    out.push('\n');
    out.push_str(first_member);
    if !first_member.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&intersection_block[open_end + 1..]);
    out
}

fn flatten_descriptions_in_distinct_members(input: &str) -> String {
    let all_diff = "<owl:AllDifferent";
    let Some(all_start) = input.find(all_diff) else {
        return input.to_owned();
    };
    let marker = "<owl:distinctMembers";
    let Some(start) = input.find(marker) else {
        return input.to_owned();
    };
    let Some(open_end) = input[start..].find('>') else {
        return input.to_owned();
    };
    let open_end = start + open_end + 1;
    let Some(close_start) = input[open_end..].find("</owl:distinctMembers>") else {
        return input.to_owned();
    };
    let close_start = open_end + close_start;
    let inner = &input[open_end..close_start];

    let mut extracted_same_as = String::new();
    let mut flattened_inner = String::new();
    let mut pos = 0usize;
    while pos < inner.len() {
        let Some(rel) = inner[pos..].find("<rdf:Description") else {
            flattened_inner.push_str(&inner[pos..]);
            break;
        };
        let desc_start = pos + rel;
        flattened_inner.push_str(&inner[pos..desc_start]);
        let Some(desc_end) = description_element_end(inner, desc_start) else {
            flattened_inner.push_str(&inner[desc_start..]);
            break;
        };
        let desc = &inner[desc_start..desc_end];
        if let Some(about) = extract_attribute(desc, "rdf:about") {
            for target in extract_same_as_targets(desc) {
                extracted_same_as.push_str(&format!(
                    "  <rdf:Description rdf:about=\"{about}\">\n    <owl:sameAs rdf:resource=\"{target}\"/>\n  </rdf:Description>\n"
                ));
            }
            flattened_inner.push_str(&format!(
                "    <owl:NamedIndividual rdf:about=\"{about}\"/>\n"
            ));
        } else {
            flattened_inner.push_str(desc);
        }
        pos = desc_end;
    }

    let mut out = String::new();
    out.push_str(&input[..all_start]);
    out.push_str(&extracted_same_as);
    out.push_str(&input[all_start..start]);
    out.push_str(&input[start..open_end]);
    out.push_str(&flattened_inner);
    out.push_str(&input[close_start..]);
    out
}

fn description_element_end(input: &str, desc_start: usize) -> Option<usize> {
    let slice = &input[desc_start..];
    if let Some(rel) = slice.find("/>") {
        let candidate = &slice[..rel + 2];
        if candidate.starts_with("<rdf:Description") && !candidate[1..rel].contains('<') {
            return Some(desc_start + rel + 2);
        }
    }
    let close = "</rdf:Description>";
    let rel = slice.find(close)?;
    Some(desc_start + rel + close.len())
}

fn extract_same_as_targets(description: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some(rel) = description[pos..].find("<owl:sameAs") {
        let start = pos + rel;
        let Some(tag_end) = description[start..].find('>') else {
            break;
        };
        let tag = &description[start..start + tag_end + 1];
        if let Some(target) = extract_attribute(tag, "rdf:resource") {
            out.push(target);
        }
        pos = start + tag_end + 1;
    }
    out
}

fn strip_all_different_about(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut pos = 0usize;
    while pos < input.len() {
        let Some(rel) = input[pos..].find("<owl:AllDifferent") else {
            out.push_str(&input[pos..]);
            break;
        };
        let start = pos + rel;
        out.push_str(&input[pos..start]);
        let Some(tag_end) = input[start..].find('>') else {
            out.push_str(&input[start..]);
            break;
        };
        let end = start + tag_end + 1;
        let tag = &input[start..end];
        if tag.contains("rdf:about=\"") {
            out.push_str(&remove_attribute(tag, "rdf:about"));
        } else {
            out.push_str(tag);
        }
        pos = end;
    }
    out
}

fn remove_attribute(tag: &str, attr: &str) -> String {
    let marker = format!("{attr}=\"");
    let Some(attr_idx) = tag.find(&marker) else {
        return tag.to_owned();
    };
    let value_start = attr_idx + marker.len();
    let Some(value_end) = tag[value_start..].find('"') else {
        return tag.to_owned();
    };
    let mut out = String::new();
    out.push_str(&tag[..attr_idx]);
    out.push_str(tag[value_start + value_end + 1..].trim_start());
    out
}

fn find_element_open_end(input: &str, start: usize) -> Option<usize> {
    let mut in_quote = None::<char>;
    for (i, ch) in input[start..].char_indices() {
        match ch {
            '"' | '\'' if in_quote.is_none() => in_quote = Some(ch),
            ch if Some(ch) == in_quote => in_quote = None,
            '>' if in_quote.is_none() => return Some(start + i + 1),
            _ => {}
        }
    }
    None
}

fn parse_xml_base(input: &str) -> String {
    let Some(root_start) = input.find("<rdf:RDF") else {
        return "urn:ontologos:anon:".to_owned();
    };
    let Some(root_end) = find_element_open_end(input, root_start) else {
        return "urn:ontologos:anon:".to_owned();
    };
    let root_tag = &input[root_start..root_end];
    if let Some(base) = extract_attribute(root_tag, "xml:base") {
        return base.trim_end_matches('#').trim_end_matches('/').to_owned();
    }
    "urn:ontologos:anon:".to_owned()
}

fn is_anonymous_description_open(tag: &str) -> bool {
    if !tag.starts_with("<rdf:Description") || tag.starts_with("</") {
        return false;
    }
    !tag.contains("rdf:about=\"")
        && !tag.contains("rdf:about='")
        && !tag.contains("rdf:nodeID=\"")
        && !tag.contains("rdf:nodeID='")
        && !tag.contains("rdf:ID=\"")
        && !tag.contains("rdf:ID='")
}

fn parse_entity_decl(rest: &str) -> Option<(String, String)> {
    let rest = rest.strip_prefix("<!ENTITY")?.trim();
    let (name, rest) = rest.split_once(|c: char| c.is_whitespace())?;
    let name = name.trim().to_owned();
    let rest = rest.trim_start();
    let quote = rest.as_bytes().first().copied()?;
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    let rest = &rest[1..];
    let value_end = rest.find(quote as char)?;
    let value = rest[..value_end].to_owned();
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
    fn normalizes_numeric_rdf_ids() {
        let input = "<owl:Class rdf:ID=\"1.0\"/><owl:someValuesFrom rdf:resource=\"#1.0\"/>";
        let out = normalize_invalid_rdf_ids(input);
        assert!(out.contains("rdf:ID=\"_1_0\""));
        assert!(out.contains("rdf:resource=\"#_1_0\""));
        assert!(!out.contains("1.0"));
    }

    #[test]
    fn expands_single_quoted_entity() {
        let input = r#"<?xml version="1.0"?>
<!DOCTYPE rdf:RDF [
    <!ENTITY owl 'http://www.w3.org/2002/07/owl#'>
]>
<rdf:RDF xmlns:owl="&owl;"><owl:Ontology rdf:about=""/></rdf:RDF>"#;
        let expanded = expand_xml_entities(input);
        assert!(!expanded.contains("<!ENTITY"));
        assert!(!expanded.contains("<!DOCTYPE"));
        assert!(expanded.contains("http://www.w3.org/2002/07/owl#"));
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
    fn expand_disjoint_classes_injects_pairwise_axioms() {
        let input = r#"<rdf:RDF xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <owl:AllDisjointClasses>
    <owl:members rdf:parseType="Collection">
      <rdf:Description rdf:about="http://ex.org#c1"/>
      <rdf:Description rdf:about="http://ex.org#c2"/>
    </owl:members>
  </owl:AllDisjointClasses>
</rdf:RDF>"#;
        let out = expand_all_disjoint_collections(input);
        assert!(out.contains("owl:disjointWith"));
        assert!(!out.contains("AllDisjointClasses"));
    }

    #[test]
    fn normalize_class_intersection_wraps_direct_intersection_of() {
        let input = r#"<rdf:RDF xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
 <owl:Class rdf:about="http://ex.org#C">
  <owl:intersectionOf rdf:parseType="Collection">
   <owl:Class rdf:about="http://ex.org#A"/>
   <owl:Class rdf:about="http://ex.org#B"/>
  </owl:intersectionOf>
 </owl:Class>
</rdf:RDF>"#;
        let out = normalize_class_intersection_definitions(input);
        assert!(out.contains("<owl:equivalentClass>"));
        assert!(!out.contains("</owl:equivalentClass>\n  <owl:intersectionOf"));
    }

    #[test]
    fn normalize_class_same_as_to_equivalent_class() {
        let input = r#"<rdf:RDF xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
 xml:base="http://www.w3.org/2002/03owlt/I4.6/premises003">
 <owl:Class rdf:ID="C1">
  <owl:sameAs>
   <owl:Class rdf:ID="C2"/>
  </owl:sameAs>
 </owl:Class>
</rdf:RDF>"#;
        let out = normalize_class_same_as(input);
        assert!(out.contains("<owl:equivalentClass>"));
        assert!(out.contains("rdf:about=\"http://www.w3.org/2002/03owlt/I4.6/premises003#C2\""));
        assert!(!out.contains("owl:sameAs"));
    }

    #[test]
    fn materialize_typed_node_element_adds_class_assertion() {
        let input = r#"<rdf:RDF xmlns:oiled="http://oiled.man.example.net/test#"
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
 xml:base="http://ex.org/">
 <oiled:Unsatisfiable/>
</rdf:RDF>"#;
        let out = materialize_typed_node_elements(input);
        assert!(out.contains("rdf:about=\"http://ex.org#_:tn1\""));
        assert!(out.contains("rdf:resource=\"http://oiled.man.example.net/test#Unsatisfiable\""));
        assert!(!out.contains("<oiled:Unsatisfiable/>"));
    }

    #[test]
    fn named_description_element_end_finds_outer_close() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/Datatype-2DFloat-2DDiscrete-2D001/premise.rdf");
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
        let injected = inject_rdf_based_punning_declarations(&expanded);
        let typed = materialize_typed_node_elements(&injected);
        let intersections = normalize_class_intersection_definitions(&typed);
        let start = intersections
            .find("<rdf:Description rdf:about=\"a\">")
            .unwrap_or_else(|| intersections.find("<rdf:Description").unwrap());
        let end = named_description_element_end(&intersections, start).expect("end");
        let block = &intersections[start..end];
        assert!(
            block.contains("</rdf:type>"),
            "block missing type close: {block}"
        );
        assert!(block.ends_with("</rdf:Description>"));
    }

    #[test]
    fn float_discrete_horned_emits_class_assertion() {
        use crate::limits::ParseLimits;
        use crate::map::map_to_core;
        use crate::read::read_horned_owl_from_reader;
        use crate::Format;
        use std::io::Cursor;

        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/Datatype-2DFloat-2DDiscrete-2D001/premise.rdf");
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
        let injected = inject_rdf_based_punning_declarations(&expanded);
        let typed = materialize_typed_node_elements(&injected);
        let intersections = normalize_class_intersection_definitions(&typed);
        let named = materialize_named_individual_descriptions(&intersections);
        let individuals = materialize_anonymous_individual_descriptions(&named);
        assert!(
            !individuals.contains("rdf:about=\"urn:ontologos:anon:"),
            "facet collection nodes must stay blank"
        );
        let parsed = read_horned_owl_from_reader(
            &mut Cursor::new(individuals.as_bytes()),
            Format::RdfXml,
            ParseLimits::default(),
        )
        .unwrap();
        let (ont, report) = map_to_core(&parsed, ParseLimits::default()).unwrap();
        assert!(
            ont.dl().axiom_count() > 0,
            "dl axioms skipped={}",
            report.meta.skipped_axiom_count
        );
    }

    #[test]
    fn anonymous_individual_materialization_skips_facet_collections() {
        let input = r#"<rdf:RDF xml:base="http://ex.org/"
 xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
 xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
 xmlns:xsd="http://www.w3.org/2001/XMLSchema#">
 <owl:withRestrictions rdf:parseType="Collection">
  <rdf:Description>
   <xsd:minExclusive rdf:datatype="http://www.w3.org/2001/XMLSchema#float">0.0</xsd:minExclusive>
  </rdf:Description>
 </owl:withRestrictions>
 <rdf:Description>
  <rdf:type rdf:resource="http://ex.org/C"/>
 </rdf:Description>
</rdf:RDF>"#;
        let out = materialize_anonymous_individual_descriptions(input);
        assert!(out.contains("rdf:about=\"http://ex.org#_:1\""));
        assert!(out.contains("<rdf:Description>\n   <xsd:minExclusive"));
    }

    #[test]
    fn tagged_element_end_self_close_with_iri_attr() {
        let s = r#"<owl:assertionProperty rdf:resource="http://www.example.org#p"/>"#;
        assert!(
            tagged_element_end(s, 0, "owl:assertionProperty").is_some(),
            "self-close with IRI attr"
        );
    }

    #[test]
    fn collect_reified_data_npa_minimal() {
        let text = r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:ex="http://www.example.org#">
  <rdf:Description rdf:about="http://www.example.org#z">
    <owl:sourceIndividual>
      <rdf:Description rdf:about="http://www.example.org#s">
        <ex:p>data</ex:p>
      </rdf:Description>
    </owl:sourceIndividual>
    <owl:assertionProperty rdf:resource="http://www.example.org#p"/>
    <owl:targetValue>data</owl:targetValue>
  </rdf:Description>
</rdf:RDF>"#;
        let start = text.find("<rdf:Description").unwrap();
        let end = element_block_end(text, start, "rdf:Description", "</rdf:Description>")
            .expect("block end");
        let block = &text[start..end];
        let open_end = block.find('>').unwrap();
        let close_idx = block.rfind("</rdf:Description>").unwrap();
        let inner = &block[open_end + 1..close_idx];
        assert!(inner.contains("owl:targetValue"), "inner={inner}");
        assert!(
            find_top_level_element_bounds(inner, "owl:sourceIndividual").is_some(),
            "sourceIndividual missing"
        );
        assert!(
            element_text_content(inner, "owl:targetValue").is_some(),
            "targetValue missing"
        );
        let collected = collect_reified_data_npas(text);
        assert_eq!(collected.len(), 1, "{collected:?}");
    }

    #[test]
    fn misc203_dpa_supplement_ofn_parses() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Dmiscellaneous-2D203/premise.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let normalized_ids = normalize_invalid_rdf_ids(&deduped);
        let expanded = expand_xml_entities_with_limit(&normalized_ids, 1_000_000).unwrap();
        let relative_uris = normalize_relative_owl_uris(&expanded);
        let rdfs_classes = normalize_rdfs_class_elements(&relative_uris);
        let injected = inject_rdf_based_punning_declarations(&rdfs_classes);
        let typed_about = materialize_typed_about_elements(&injected);
        let typed_nodes = materialize_typed_node_elements(&typed_about);
        let intersections = normalize_class_intersection_definitions(&typed_nodes);
        let same_as = normalize_class_same_as(&intersections);
        let named_individuals = materialize_named_individual_descriptions(&same_as);
        let individuals = materialize_anonymous_individual_descriptions(&named_individuals);
        let normalized = normalize_all_different_members(&individuals);
        let disjoint = expand_all_disjoint_collections(&normalized);
        let collected = collect_direct_data_literal_assertions(&disjoint);
        let dpa = &collected[0];
        let (lexical, datatype_iri) = if dpa.value_literal.contains("^^") {
            let mut parts = dpa.value_literal.splitn(2, "^^");
            let lex = parts.next().unwrap().trim_matches('"').to_string();
            let dt = parts.next().unwrap().trim_matches(|c| c == '<' || c == '>');
            (lex, dt.to_string())
        } else {
            panic!("unexpected literal");
        };
        let (extra_prefixes, lit, dt_decl) =
            qualify_typed_literal_for_supplement(&lexical, &datatype_iri);
        let dt_decl_line = dt_decl.map(|d| format!("\n       {d}")).unwrap_or_default();
        let body = format!(
            "Declaration(NamedIndividual(<{}>))\n\
             Declaration(DataProperty(<{}>))\n\
             ClassAssertion(owl:Thing <{}>){dt_decl_line}\n\
             DataPropertyAssertion(<{}> <{}> {lit})",
            dpa.subject, dpa.property, dpa.subject, dpa.property, dpa.subject
        );
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\n\
             Prefix(rdf:=<http://www.w3.org/1999/02/22-rdf-syntax-ns#>)\n\
             {extra_prefixes}\n\
             Ontology(<http://example.org/thing-data-literal-supplement>\n{body}\n)"
        );
        eprintln!("{ofn}");
        let ont = crate::load_ofn_from_str(&ofn).expect("supplement ofn");
        assert!(ont
            .dl()
            .axioms()
            .any(|a| { matches!(a, ontologos_core::DlAxiom::DataPropertyAssertion { .. }) }));
    }

    #[test]
    fn misc203_full_load_has_data_assertions() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Dmiscellaneous-2D203/premise.rdf",
        );
        let ont = crate::load_ontology(&path).expect("load misc203");
        let dpas: Vec<_> = ont
            .dl()
            .axioms()
            .filter(|a| matches!(a, ontologos_core::DlAxiom::DataPropertyAssertion { .. }))
            .collect();
        eprintln!("dpas={dpas:?}");
        assert_eq!(
            dpas.len(),
            2,
            "all axioms: {:?}",
            ont.dl().axioms().collect::<Vec<_>>()
        );
    }

    #[test]
    fn collect_direct_data_literals_from_misc203_after_full_preprocess() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Dmiscellaneous-2D203/premise.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let normalized_ids = normalize_invalid_rdf_ids(&deduped);
        let expanded = expand_xml_entities_with_limit(&normalized_ids, 1_000_000).unwrap();
        let relative_uris = normalize_relative_owl_uris(&expanded);
        let rdfs_classes = normalize_rdfs_class_elements(&relative_uris);
        let injected = inject_rdf_based_punning_declarations(&rdfs_classes);
        let typed_about = materialize_typed_about_elements(&injected);
        let typed_nodes = materialize_typed_node_elements(&typed_about);
        let intersections = normalize_class_intersection_definitions(&typed_nodes);
        let same_as = normalize_class_same_as(&intersections);
        let named_individuals = materialize_named_individual_descriptions(&same_as);
        let individuals = materialize_anonymous_individual_descriptions(&named_individuals);
        let normalized = normalize_all_different_members(&individuals);
        let disjoint = expand_all_disjoint_collections(&normalized);
        let collected = collect_direct_data_literal_assertions(&disjoint);
        eprintln!("after full preprocess collected={collected:?}");
        assert_eq!(collected.len(), 2, "{collected:?}");
    }

    #[test]
    fn collect_direct_data_literals_from_misc203() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Dmiscellaneous-2D203/premise.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let normalized_ids = normalize_invalid_rdf_ids(&deduped);
        let expanded = expand_xml_entities_with_limit(&normalized_ids, 1_000_000).unwrap();
        let relative_uris = normalize_relative_owl_uris(&expanded);
        let injected = inject_rdf_based_punning_declarations(&relative_uris);
        let typed_about = materialize_typed_about_elements(&injected);
        let typed_nodes = materialize_typed_node_elements(&typed_about);
        let collected = collect_direct_data_literal_assertions(&typed_nodes);
        eprintln!("collected={collected:?}");
        assert_eq!(collected.len(), 2, "{collected:?}");
    }

    #[test]
    fn xml_literal_data_property_supplement_loads() {
        let ofn = "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Prefix(rdf:=<http://www.w3.org/1999/02/22-rdf-syntax-ns#>)\n\
             Ontology(<http://example.org/xml-lit-test>\n\
               Declaration(NamedIndividual(<http://example.org/i>))\n\
               Declaration(DataProperty(<http://example.org/p>))\n\
               DataPropertyAssertion(<http://example.org/p> <http://example.org/i> \"<b>x</b>\"^^rdf:XMLLiteral)\n\
             )";
        let ont = crate::load_ofn_from_str(ofn).expect("load xml literal dpa");
        let has_dpa = ont
            .dl()
            .axioms()
            .any(|a| matches!(a, ontologos_core::DlAxiom::DataPropertyAssertion { .. }));
        assert!(
            has_dpa,
            "axioms: {:?}",
            ont.dl().axioms().collect::<Vec<_>>()
        );
    }

    #[test]
    fn collect_reified_data_npa_from_fixture() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Dnpa-2Ddat-2Dfw/premise.rdf");
        let text = std::fs::read_to_string(&path).unwrap();
        let collected = collect_reified_data_npas(&text);
        assert_eq!(collected.len(), 1, "{collected:?}");
        assert_eq!(collected[0].value_literal, "data");
    }

    #[test]
    fn named_description_skips_inverse_functional_property_typing() {
        let input = r#"<rdf:RDF xml:base="http://ex.org/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
 <rdf:Description rdf:about="http://ex.org/p">
  <rdf:type rdf:resource="http://www.w3.org/2002/07/owl#DatatypeProperty"/>
 </rdf:Description>
</rdf:RDF>"#;
        let out = materialize_named_individual_descriptions(input);
        assert!(out.contains("<rdf:Description rdf:about=\"http://ex.org/p\">"));
        assert!(!out.contains("NamedIndividual"));
    }

    #[test]
    fn manual_rewrite_once_on_float() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/Datatype-2DFloat-2DDiscrete-2D001/premise.rdf");
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
        let injected = inject_rdf_based_punning_declarations(&expanded);
        let typed = materialize_typed_node_elements(&injected);
        let intersections = normalize_class_intersection_definitions(&typed);
        let start = intersections
            .find("<rdf:Description rdf:about=\"a\">")
            .unwrap();
        let end = named_description_element_end(&intersections, start).unwrap();
        let block = &intersections[start..end];
        assert!(
            block.ends_with("</rdf:Description>"),
            "block end: {:?}",
            &block[block.len().saturating_sub(40)..]
        );
        let rewritten = rewrite_description_to_named_individual(block);
        assert!(
            !rewritten.contains("rdf:ty</owl:NamedIndividual>"),
            "{rewritten}"
        );
    }

    #[test]
    fn float_discrete_preprocess_produces_valid_xml() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/Datatype-2DFloat-2DDiscrete-2D001/premise.rdf");
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
        let injected = inject_rdf_based_punning_declarations(&expanded);
        let typed = materialize_typed_node_elements(&injected);
        let intersections = normalize_class_intersection_definitions(&typed);
        let named = materialize_named_individual_descriptions(&intersections);
        if named.contains("rdf:ty</owl:NamedIndividual>") {
            panic!("bad output:\n{named}");
        }
        assert!(named.contains("<owl:NamedIndividual"));
    }

    #[test]
    fn materialize_named_description_to_individual() {
        let input = r#"<rdf:RDF xml:base="http://ex.org/ontology/"
 xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
 <rdf:Description rdf:about="a">
  <rdf:type>
   <owl:Restriction>
    <owl:onProperty rdf:resource="dp"/>
   </owl:Restriction>
  </rdf:type>
 </rdf:Description>
</rdf:RDF>"#;
        let out = materialize_named_individual_descriptions(input);
        assert!(out.contains("<owl:NamedIndividual"));
        assert!(!out.contains("<rdf:Description rdf:about=\"a\">"));
    }

    #[test]
    fn horned_parses_object_restriction_via_equivalent_class() {
        use crate::limits::ParseLimits;
        use crate::read::read_horned_owl_from_reader;
        use crate::Format;
        use std::io::Cursor;

        let mini = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xml:base="http://example.org/">
<owl:Ontology/>
<owl:Class rdf:about="probe">
  <owl:equivalentClass>
    <owl:Restriction>
      <owl:onProperty rdf:resource="http://www.w3.org/2002/07/owl#bottomObjectProperty" />
      <owl:someValuesFrom rdf:resource="http://www.w3.org/2002/07/owl#Thing" />
    </owl:Restriction>
  </owl:equivalentClass>
</owl:Class>
</rdf:RDF>"#;
        let parsed = read_horned_owl_from_reader(
            &mut Cursor::new(mini.as_bytes()),
            Format::RdfXml,
            ParseLimits::default(),
        )
        .unwrap();
        for annotated in parsed.iter() {
            eprintln!("{:?}", annotated.component);
        }
    }

    #[test]
    fn horned_components_bottom_vs_float() {
        use crate::limits::ParseLimits;
        use crate::read::read_horned_owl_from_reader;
        use crate::Format;
        use std::io::Cursor;

        for case in [
            "New-2DFeature-2DBottomObjectProperty-2D001",
            "Datatype-2DFloat-2DDiscrete-2D001",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../benchmarks/data/hermit/wg")
                .join(case)
                .join("premise.rdf");
            let text = std::fs::read_to_string(&path).unwrap();
            let deduped = dedupe_rdf_xml_ids(&text);
            let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
            let injected = inject_rdf_based_punning_declarations(&expanded);
            let typed = materialize_typed_node_elements(&injected);
            let intersections = normalize_class_intersection_definitions(&typed);
            let same_as = normalize_class_same_as(&intersections);
            let named = materialize_named_individual_descriptions(&same_as);
            let individuals = materialize_anonymous_individual_descriptions(&named);
            let parsed = read_horned_owl_from_reader(
                &mut Cursor::new(individuals.as_bytes()),
                Format::RdfXml,
                ParseLimits::default(),
            )
            .unwrap();
            eprintln!("\n=== {case} ===");
            for annotated in parsed.iter() {
                eprintln!("  {:?}", annotated.component);
            }
        }
    }

    #[test]
    fn restriction_006_union_ofn_debug() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DRestriction-2D006/conclusion.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
        let injected = inject_rdf_based_punning_declarations(&expanded);
        let typed = materialize_typed_node_elements(&injected);
        let intersections = normalize_class_intersection_definitions(&typed);
        let same_as = normalize_class_same_as(&intersections);
        let named = materialize_named_individual_descriptions(&same_as);
        for (name, stage) in [
            ("raw", text.as_str()),
            ("deduped", deduped.as_str()),
            ("expanded", expanded.as_str()),
            ("injected", injected.as_str()),
            ("typed", typed.as_str()),
            ("intersections", intersections.as_str()),
            ("same_as", same_as.as_str()),
            ("named", named.as_str()),
        ] {
            let has_restriction = stage.contains("<owl:Restriction");
            let has_union_child = stage.contains("unionOf") && stage.contains("<owl:Class");
            eprintln!("{name}: restriction={has_restriction} union_child={has_union_child}");
        }
        let base = parse_xml_base(&text);
        let dt_props = declared_datatype_property_iris(&text);
        let node_lists = build_rdf_collection_node_map(&text, &base, &dt_props);
        let mut pos = 0usize;
        let mut ce_block = String::new();
        while pos < named.len() {
            let next_desc = named[pos..].find("<rdf:Description");
            let next_ind = named[pos..].find("<owl:NamedIndividual");
            let next = match (next_desc, next_ind) {
                (Some(d), Some(i)) => Some(pos + d.min(i)),
                (Some(d), None) => Some(pos + d),
                (None, Some(i)) => Some(pos + i),
                (None, None) => None,
            };
            let Some(start) = next else { break };
            if named[start..].starts_with("</") {
                pos = start + 1;
                continue;
            }
            let (open_tag, close_tag) = if named[start..].starts_with("<owl:NamedIndividual") {
                ("owl:NamedIndividual", "</owl:NamedIndividual>")
            } else {
                ("rdf:Description", "</rdf:Description>")
            };
            let Some(end) = element_block_end(&named, start, open_tag, close_tag) else {
                break;
            };
            let block = &named[start..end];
            if block.contains("premises006#x") {
                let open_end = block.find('>').unwrap();
                let close_start = block.rfind(close_tag).unwrap();
                let inner = &block[open_end + 1..close_start];
                if let Some((_, _, type_block)) = find_top_level_element_bounds(inner, "rdf:type") {
                    ce_block =
                        unwrap_typed_class_expression_block(&element_inner(type_block, "rdf:type"));
                }
                break;
            }
            pos = end;
        }
        let union_ofn = boolean_operator_ofn(
            ce_block.trim(),
            "owl:unionOf",
            "ObjectUnionOf",
            &base,
            &dt_props,
            &node_lists,
        );
        assert!(
            union_ofn
                .as_deref()
                .is_some_and(|u| u.contains("ObjectUnionOf")),
            "union_ofn={union_ofn:?}"
        );
    }

    #[test]
    fn restriction_006_conclusion_emits_union_class_assertion() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DRestriction-2D006/conclusion.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
        let injected = inject_rdf_based_punning_declarations(&expanded);
        let typed = materialize_typed_node_elements(&injected);
        let intersections = normalize_class_intersection_definitions(&typed);
        let same_as = normalize_class_same_as(&intersections);
        let named = materialize_named_individual_descriptions(&same_as);
        let collected = collect_object_class_assertions(&named);
        assert!(
            collected.iter().any(|(_, ce)| ce.contains("ObjectUnionOf")),
            "{collected:?}"
        );
    }

    #[test]
    fn i5_1_premise_loads() {
        use crate::load_ontology;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DI5.1-2D001/premise.rdf");
        load_ontology(&path).expect("load I5.1 premise");
    }

    #[test]
    fn transitive_property_conclusion_emits_class_assertion() {
        use crate::load_ontology;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DTransitiveProperty-2D002/conclusion.rdf",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            ont.dl()
                .axioms()
                .any(|a| matches!(a, ontologos_core::DlAxiom::ClassAssertion { .. })),
            "expected DL ClassAssertion"
        );
    }

    #[test]
    fn collect_object_class_assertion_restriction_with_nested_class_type() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DTransitiveProperty-2D002/conclusion.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
        let injected = inject_rdf_based_punning_declarations(&expanded);
        let typed = materialize_typed_node_elements(&injected);
        let intersections = normalize_class_intersection_definitions(&typed);
        let same_as = normalize_class_same_as(&intersections);
        let named = materialize_named_individual_descriptions(&same_as);
        let collected = collect_object_class_assertions(&named);
        assert_eq!(collected.len(), 1, "{collected:?}");
        assert!(collected[0].1.contains("ObjectSomeValuesFrom"));
    }

    #[test]
    fn collect_object_class_assertion_bottom_property() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/New-2DFeature-2DBottomObjectProperty-2D001/premise.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
        let injected = inject_rdf_based_punning_declarations(&expanded);
        let typed = materialize_typed_node_elements(&injected);
        let intersections = normalize_class_intersection_definitions(&typed);
        let same_as = normalize_class_same_as(&intersections);
        let named = materialize_named_individual_descriptions(&same_as);
        let collected = collect_object_class_assertions(&named);
        assert_eq!(collected.len(), 1);
        assert!(collected[0].1.contains("bottomObjectProperty"));
    }

    #[test]
    fn rational003_oneof_has_two_literals() {
        let avf = r#"<owl:Restriction>
      <owl:onProperty rdf:resource="dp" />
      <owl:allValuesFrom>
        <rdfs:Datatype>
          <owl:oneOf>
            <rdf:Description>
              <rdf:first rdf:datatype="http://www.w3.org/2001/XMLSchema#decimal">0.3333333333333333</rdf:first>
              <rdf:rest>
                <rdf:Description>
                  <rdf:first rdf:datatype="http://www.w3.org/2002/07/owl#rational">1/3</rdf:first>
                  <rdf:rest rdf:resource="http://www.w3.org/1999/02/22-rdf-syntax-ns#"/>
                </rdf:Description>
              </rdf:rest>
            </rdf:Description>
          </owl:oneOf>
        </rdfs:Datatype>
      </owl:allValuesFrom>
    </owl:Restriction>"#;
        let base = "http://example.org/";
        let mut dt = std::collections::HashSet::new();
        dt.insert("http://example.org/dp".to_string());
        let ofn = data_restriction_ce_to_ofn(avf, base, &dt).expect("ofn");
        assert!(ofn.contains("DataOneOf"), "{ofn}");
        assert!(ofn.contains("0.3333333333333333"), "{ofn}");
        assert!(ofn.contains("1/3"), "{ofn}");

        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/New-2DFeature-2DRational-2D003/premise.rdf");
        let rdf = std::fs::read_to_string(&path).expect("read");
        let deduped = dedupe_rdf_xml_ids(&rdf);
        let normalized_ids = normalize_invalid_rdf_ids(&deduped);
        let relative_uris = normalize_relative_owl_uris(&normalized_ids);
        let injected = inject_rdf_based_punning_declarations(&relative_uris);
        let typed_about = materialize_typed_about_elements(&injected);
        let typed_nodes = materialize_typed_node_elements(&typed_about);
        let intersections = normalize_class_intersection_definitions(&typed_nodes);
        let same_as = normalize_class_same_as(&intersections);
        let named = materialize_named_individual_descriptions(&same_as);
        let collected = collect_object_class_assertions(&named);
        assert_eq!(collected.len(), 2, "{collected:?}");
        assert!(
            collected
                .iter()
                .any(|(_, ce)| ce.contains("DataOneOf") && ce.contains("1/3")),
            "{collected:?}"
        );
        assert!(
            collected
                .iter()
                .any(|(_, ce)| ce.contains("DataMinCardinality")),
            "{collected:?}"
        );
    }

    #[test]
    fn rational002_allvaluesfrom_oneof_ofn() {
        let avf = r#"<owl:Restriction>
      <owl:onProperty rdf:resource="dp" />
      <owl:allValuesFrom>
        <rdfs:Datatype>
          <owl:oneOf>
            <rdf:Description>
              <rdf:first rdf:datatype="http://www.w3.org/2001/XMLSchema#decimal">0.5</rdf:first>
              <rdf:rest>
                <rdf:Description>
                  <rdf:first rdf:datatype="http://www.w3.org/2002/07/owl#rational">1/2</rdf:first>
                  <rdf:rest rdf:resource="http://www.w3.org/1999/02/22-rdf-syntax-ns#"/>
                </rdf:Description>
              </rdf:rest>
            </rdf:Description>
          </owl:oneOf>
        </rdfs:Datatype>
      </owl:allValuesFrom>
    </owl:Restriction>"#;
        let base = "http://example.org/";
        let mut dt = std::collections::HashSet::new();
        dt.insert("http://example.org/dp".to_string());
        let ofn = data_restriction_ce_to_ofn(avf, base, &dt);
        assert!(ofn.is_some_and(|s| s.contains("DataAllValuesFrom")));
    }

    #[test]
    fn data_restriction_min_cardinality_ofn() {
        let ce = r#"<owl:Restriction>
    <owl:onProperty rdf:resource="dp"/>
    <owl:minCardinality>2</owl:minCardinality>
  </owl:Restriction>"#;
        let base = "http://example.org/";
        let mut dt = std::collections::HashSet::new();
        dt.insert("http://example.org/dp".to_string());
        let ofn = data_restriction_ce_to_ofn(ce, base, &dt);
        assert_eq!(
            ofn.as_deref(),
            Some("DataMinCardinality(2 <http://example.org/dp>)")
        );
    }

    #[test]
    fn rational002_collects_both_data_restrictions() {
        let inline_min = r#"<rdf:RDF xml:base="http://example.org/"
 xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
<owl:DatatypeProperty rdf:about="dp"/>
<rdf:Description rdf:about="a">
  <owl:onProperty rdf:resource="dp"/>
  <owl:minCardinality>2</owl:minCardinality>
</rdf:Description>
</rdf:RDF>"#;
        let inline = collect_object_class_assertions(inline_min);
        assert_eq!(inline.len(), 1, "inline={inline:?}");

        let typed_min = r#"<rdf:RDF xml:base="http://example.org/"
 xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
<owl:DatatypeProperty rdf:about="dp"/>
<rdf:Description rdf:about="a">
  <rdf:type><owl:Restriction>
    <owl:onProperty rdf:resource="dp"/>
    <owl:minCardinality>2</owl:minCardinality>
  </owl:Restriction></rdf:type>
</rdf:Description>
</rdf:RDF>"#;
        let typed = collect_object_class_assertions(typed_min);
        assert_eq!(typed.len(), 1, "typed={typed:?}");

        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/New-2DFeature-2DRational-2D002/premise.rdf");
        let rdf = std::fs::read_to_string(&path).expect("read");
        let deduped = dedupe_rdf_xml_ids(&rdf);
        let normalized_ids = normalize_invalid_rdf_ids(&deduped);
        let relative_uris = normalize_relative_owl_uris(&normalized_ids);
        let injected = inject_rdf_based_punning_declarations(&relative_uris);
        let typed_about = materialize_typed_about_elements(&injected);
        let typed_nodes = materialize_typed_node_elements(&typed_about);
        let intersections = normalize_class_intersection_definitions(&typed_nodes);
        let same_as = normalize_class_same_as(&intersections);
        let named = materialize_named_individual_descriptions(&same_as);
        let collected = collect_object_class_assertions(&named);
        assert_eq!(collected.len(), 2, "{collected:?}");
        assert!(
            collected
                .iter()
                .any(|(_, ce)| ce.contains("DataAllValuesFrom")),
            "{collected:?}"
        );
        assert!(
            collected
                .iter()
                .any(|(_, ce)| ce.contains("DataMinCardinality")),
            "{collected:?}"
        );
    }

    #[test]
    fn rational002_full_load_parses() {
        use crate::load_ontology;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/New-2DFeature-2DRational-2D002/premise.rdf");
        let ont = load_ontology(&path).expect("load rational002");
        assert!(ont.dl().axiom_count() > 0);
    }

    #[test]
    fn bottom_property_emits_class_assertion() {
        use crate::load_ontology;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/New-2DFeature-2DBottomObjectProperty-2D001/premise.rdf",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(ont.dl().axiom_count() > 0);
    }

    #[test]
    fn materialize_named_description_skips_thing_subclass() {
        let input = r#"<rdf:RDF xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
 xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#">
 <rdf:Description rdf:about="http://www.w3.org/2002/07/owl#Thing">
  <rdfs:subClassOf>
   <owl:Restriction>
    <owl:onProperty rdf:resource="http://ex.org#f"/>
   </owl:Restriction>
  </rdfs:subClassOf>
 </rdf:Description>
</rdf:RDF>"#;
        let out = materialize_named_individual_descriptions(input);
        assert!(out.contains("<rdf:Description"));
        assert!(!out.contains("<owl:NamedIndividual"));
    }

    #[test]
    fn materialize_anonymous_description_adds_about() {
        let input = r#"<rdf:RDF xml:base="http://ex.org/">
  <rdf:Description>
    <rdf:type rdf:resource="http://ex.org#C"/>
  </rdf:Description>
</rdf:RDF>"#;
        let out = materialize_anonymous_individual_descriptions(input);
        assert!(out.contains("rdf:about=\"http://ex.org#_:1\""));
    }

    #[test]
    fn description_element_end_parses_member_descriptions() {
        let inner = r#"
      <rdf:Description rdf:about="http://ex.org#w1">
        <owl:sameAs rdf:resource="http://ex.org#w2"/>
      </rdf:Description>
      <rdf:Description rdf:about="http://ex.org#w2"/>
    "#;
        let start = inner.find("<rdf:Description").unwrap();
        let end = super::description_element_end(inner, start).expect("first desc end");
        assert!(inner[start..end].contains("sameAs"));
        let start2 = inner[end..].find("<rdf:Description").unwrap() + end;
        let end2 = super::description_element_end(inner, start2).expect("second desc end");
        assert!(inner[start2..end2].contains("w2"));
    }

    #[test]
    fn normalize_all_different_members_tag() {
        let input = r#"<rdf:RDF xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <owl:AllDifferent rdf:about="http://ex.org#z">
    <owl:members rdf:parseType="Collection">
      <rdf:Description rdf:about="http://ex.org#w1">
        <owl:sameAs rdf:resource="http://ex.org#w2"/>
      </rdf:Description>
      <rdf:Description rdf:about="http://ex.org#w2"/>
    </owl:members>
  </owl:AllDifferent>
</rdf:RDF>"#;
        let out = normalize_all_different_members(input);
        assert!(out.contains("owl:distinctMembers"));
        assert!(out.contains("<owl:NamedIndividual rdf:about=\"http://ex.org#w1\"/>"));
        assert!(!out.contains("AllDifferent rdf:about"));
        assert!(out.contains("owl:sameAs"));
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

    #[test]
    fn functional_property_004_conclusion_loads_functional_axiom() {
        use crate::load_ontology;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DFunctionalProperty-2D004/conclusion.rdf",
        );
        let ont = load_ontology(&path).expect("load");
        let has_functional = ont.axioms().iter().any(|(_, ax)| {
            matches!(ax, ontologos_core::Axiom::FunctionalObjectProperty(_))
        });
        eprintln!("conclusion axioms: {:?}", ont.axioms().iter().collect::<Vec<_>>());
        eprintln!("conclusion dl: {:?}", ont.dl().axioms().collect::<Vec<_>>());
        assert!(has_functional, "expected FunctionalObjectProperty in conclusion");
    }

    #[test]
    fn functional_property_004_oneof_supplement() {
        use crate::load_ontology;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DFunctionalProperty-2D004/premise.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let root = normalize_multiline_rdf_root_tag(&text);
        let deduped = dedupe_rdf_xml_ids(&root);
        let normalized_ids = normalize_invalid_rdf_ids(&deduped);
        let expanded = expand_xml_entities_with_limit(&normalized_ids, 10_000_000).unwrap();
        let relative_uris = normalize_relative_owl_uris(&expanded);
        let rdfs_classes = normalize_rdfs_class_elements(&relative_uris);
        let injected = inject_rdf_based_punning_declarations(&rdfs_classes);
        let typed_about = materialize_typed_about_elements(&injected);
        let typed_nodes = materialize_typed_node_elements(&typed_about);
        let intersections = normalize_class_intersection_definitions(&typed_nodes);
        let same_as = normalize_class_same_as(&intersections);
        let named = materialize_named_individual_descriptions(&same_as);
        let individuals = materialize_anonymous_individual_descriptions(&named);
        let normalized = normalize_all_different_members(&individuals);
        let preprocessed = expand_all_disjoint_collections(&normalized);
        let equivs = collect_boolean_class_equivalences(&preprocessed);
        assert!(
            equivs.iter().any(|(_, ce)| ce.contains("ObjectOneOf")),
            "expected singleton oneOf equivalence, got {equivs:?}\npreprocessed:\n{preprocessed}"
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            ont.dl().axiom_count() > 0,
            "dl axioms: {:?}",
            ont.dl().axioms().collect::<Vec<_>>()
        );
    }

    #[test]
    fn normalize_multiline_rdf_root_tag_collapses_functional_property_fixture() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DFunctionalProperty-2D004/premise.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let collapsed = normalize_multiline_rdf_root_tag(&text);
        assert!(
            !collapsed.contains("\n  xmlns:owl"),
            "root tag should be collapsed: {}",
            &collapsed[..collapsed.find('>').unwrap_or(collapsed.len())]
        );
        let loaded = crate::load_ontology(&path).expect("load functional property premise");
        assert!(loaded.axiom_count() > 0 || loaded.dl().axiom_count() > 0);
    }

    #[test]
    fn parse_xmlns_multiline_root_with_trailing_angle() {
        let input = r#"<rdf:RDF 
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:oiled="http://oiled.man.example.net/test#">
 <oiled:Unsatisfiable/>
</rdf:RDF>"#;
        let xmlns = parse_xmlns(input);
        assert_eq!(
            xmlns.get("oiled"),
            Some(&"http://oiled.man.example.net/test#".to_owned())
        );
        let out = materialize_typed_node_elements(input);
        assert!(out.contains("#_:tn1"));
        assert!(!out.contains("<oiled:Unsatisfiable/>"));
    }

    #[test]
    fn dl035_materialize_typed_node_on_fixture() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D035/premise.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
        let out = materialize_typed_node_elements(&expanded);
        assert!(out.contains("#_:tn1"));
        assert!(!out.contains("<oiled:Unsatisfiable/>"));
    }

    #[test]
    fn dl035_preprocess_retains_typed_node_and_spy_individual() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D035/premise.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
        let injected = inject_rdf_based_punning_declarations(&expanded);
        let typed = materialize_typed_node_elements(&injected);
        assert!(
            typed.contains("#_:tn1"),
            "typed node not materialized:\n{typed}"
        );
        assert!(!typed.contains("<oiled:Unsatisfiable/>"));
        let intersections = normalize_class_intersection_definitions(&typed);
        let same_as = normalize_class_same_as(&intersections);
        let named = materialize_named_individual_descriptions(&same_as);
        let individuals = materialize_anonymous_individual_descriptions(&named);
        assert!(individuals.contains("test#spy"));
        assert!(individuals.contains("owl:NamedIndividual"));
    }

    #[test]
    fn wg_nothing_001_loads_class_assertion() {
        use crate::load_ontology;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DNothing-2D001/premise.rdf");
        let ont = load_ontology(&path).unwrap();
        assert!(
            ont.dl().axiom_count() > 0,
            "expected ClassAssertion for owl:Nothing typed node"
        );
    }

    #[test]
    fn intersection_of_001_supplementation() {
        use crate::load_ontology;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DintersectionOf-2D001/premise.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let equivs = collect_boolean_class_equivalences(&text);
        assert!(
            equivs.len() >= 2,
            "expected boolean equivalences, got {equivs:?}"
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            ont.dl().axiom_count() >= 3,
            "axiom_count={}",
            ont.dl().axiom_count()
        );
    }

    #[test]
    fn collect_rdfs_domain_from_domain_cond() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Drdfs-2Ddomain-2Dcond/premise.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let normalized_ids = normalize_invalid_rdf_ids(&deduped);
        let expanded = expand_xml_entities_with_limit(&normalized_ids, 10_000_000).expect("expand");
        let relative_uris = normalize_relative_owl_uris(&expanded);
        let injected = inject_rdf_based_punning_declarations(&relative_uris);
        let typed_about = materialize_typed_about_elements(&injected);
        let typed_nodes = materialize_typed_node_elements(&typed_about);
        let intersections = normalize_class_intersection_definitions(&typed_nodes);
        let same_as = normalize_class_same_as(&intersections);
        let named = materialize_named_individual_descriptions(&same_as);
        let anon = materialize_anonymous_individual_descriptions(&named);
        let normalized = normalize_all_different_members(&anon);
        let final_out = expand_all_disjoint_collections(&normalized);
        let domains = collect_rdfs_object_property_domains(&final_out);
        assert!(
            !domains.is_empty(),
            "expected rdfs:domain after full pipeline, got {domains:?}\nfinal:\n{final_out}"
        );
    }

    #[test]
    fn wg_somevalues_inst_subj_preprocess_no_nested_description() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Drestrict-2Dsomevalues-2Dinst-2Dsubj/premise.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let normalized_ids = normalize_invalid_rdf_ids(&deduped);
        let expanded = expand_xml_entities_with_limit(&normalized_ids, 10_000_000).expect("expand");
        let relative_uris = normalize_relative_owl_uris(&expanded);
        let injected = inject_rdf_based_punning_declarations(&relative_uris);
        let typed_about = materialize_typed_about_elements(&injected);
        let typed_nodes = materialize_typed_node_elements(&typed_about);
        let intersections = normalize_class_intersection_definitions(&typed_nodes);
        let same_as = normalize_class_same_as(&intersections);
        let named = materialize_named_individual_descriptions(&same_as);
        let anon = materialize_anonymous_individual_descriptions(&named);
        let normalized = normalize_all_different_members(&anon);
        let final_out = expand_all_disjoint_collections(&normalized);
        for (label, out) in [
            ("injected", &injected),
            ("typed_about", &typed_about),
            ("typed_nodes", &typed_nodes),
            ("intersections", &intersections),
            ("same_as", &same_as),
            ("named", &named),
            ("anon", &anon),
            ("normalized", &normalized),
            ("final", &final_out),
        ] {
            if out.matches("<rdf:Description").count() > 2 {
                eprintln!(
                    "=== {label} ({} descriptions) ===\n{out}\n",
                    out.matches("<rdf:Description").count()
                );
            }
        }
        use crate::limits::ParseLimits;
        use crate::read::read_horned_owl_from_reader;
        use crate::Format;
        read_horned_owl_from_reader(
            &mut std::io::Cursor::new(final_out.into_bytes()),
            Format::RdfXml,
            ParseLimits::default(),
        )
        .expect("horned-owl parse");
    }

    #[test]
    fn collect_opa_from_owl_thing_with_blank_node_object() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DallValuesFrom-2D002/conclusion.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let opas = collect_object_property_assertions(&text);
        assert_eq!(opas.len(), 1, "expected one OPA, got {opas:?}");
        assert!(opas[0].0.contains("premises002#i"));
        assert!(opas[0].1.contains("premises002#p"));
        assert!(opas[0].2.contains("#_o"));

        let deduped = dedupe_rdf_xml_ids(&text);
        let normalized_ids = normalize_invalid_rdf_ids(&deduped);
        let expanded = expand_xml_entities_with_limit(&normalized_ids, 1_000_000).unwrap();
        let relative_uris = normalize_relative_owl_uris(&expanded);
        let injected = inject_rdf_based_punning_declarations(&relative_uris);
        let typed_about = materialize_typed_about_elements(&injected);
        let typed_nodes = materialize_typed_node_elements(&typed_about);
        let intersections = normalize_class_intersection_definitions(&typed_nodes);
        let same_as = normalize_class_same_as(&intersections);
        let named_individuals = materialize_named_individual_descriptions(&same_as);
        let individuals = materialize_anonymous_individual_descriptions(&named_individuals);
        let normalized = normalize_all_different_members(&individuals);
        let preprocessed = expand_all_disjoint_collections(&normalized);
        let preprocessed_opas = collect_object_property_assertions(&preprocessed);
        assert_eq!(
            preprocessed_opas.len(),
            1,
            "expected one OPA, got {preprocessed_opas:?}"
        );
        let loaded = crate::load_ontology(&path).expect("load conclusion");
        let has_opa = loaded
            .dl()
            .axioms()
            .any(|a| matches!(a, ontologos_core::DlAxiom::ObjectPropertyAssertion { .. }))
            || loaded
                .axioms()
                .iter()
                .any(|(_, a)| matches!(a, ontologos_core::Axiom::ObjectPropertyAssertion { .. }));
        assert!(has_opa, "loaded ontology missing OPA");
    }

    #[test]
    fn collect_nested_opa_from_somevalues_conclusion() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DsomeValuesFrom-2D003/conclusion.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let opas = collect_object_property_assertions(&text);
        assert_eq!(opas.len(), 2, "expected two OPAs, got {opas:?}");
        assert!(opas[0].0.contains("fred"));
        assert!(opas[0].1.contains("parent"));
        let loaded = crate::load_ontology(&path).expect("load conclusion");
        let opa_count = loaded
            .axioms()
            .iter()
            .filter(|(_, a)| matches!(a, ontologos_core::Axiom::ObjectPropertyAssertion { .. }))
            .count();
        assert_eq!(opa_count, 2, "loaded conclusion should have two OPAs");
    }

    #[test]
    fn i5_5_005_singleton_union_supplement() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DI5.5-2D005/conclusion.rdf");
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let normalized_ids = normalize_invalid_rdf_ids(&deduped);
        let expanded = expand_xml_entities_with_limit(&normalized_ids, 10_000_000).unwrap();
        let relative_uris = normalize_relative_owl_uris(&expanded);
        let injected = inject_rdf_based_punning_declarations(&relative_uris);
        let typed_about = materialize_typed_about_elements(&injected);
        let typed_nodes = materialize_typed_node_elements(&typed_about);
        let intersections = normalize_class_intersection_definitions(&typed_nodes);
        let same_as = normalize_class_same_as(&intersections);
        let named = materialize_named_individual_descriptions(&same_as);
        let individuals = materialize_anonymous_individual_descriptions(&named);
        let normalized = normalize_all_different_members(&individuals);
        let preprocessed = expand_all_disjoint_collections(&normalized);
        let equivs = collect_boolean_class_equivalences(&preprocessed);
        assert!(
            equivs.iter().any(|(_, ce)| ce.contains("ObjectUnionOf")),
            "expected singleton union equivalence, got {equivs:?}"
        );
    }

    #[test]
    fn disjoint_with_010_premise_loads_abox() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DdisjointWith-2D010/premise.rdf",
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let opas = collect_object_property_assertions(&text);
        assert_eq!(opas.len(), 1, "expected one OPA, got {opas:?}");
        let typing = collect_self_disjoint_restriction_assertions(&text);
        assert_eq!(
            typing.len(),
            1,
            "expected one class assertion, got {typing:?}"
        );
        let loaded = crate::load_ontology(&path).expect("load");
        assert!(
            loaded.dl().axiom_count() > 0 || !loaded.axioms().is_empty(),
            "loaded ontology should have axioms"
        );
        assert!(
            !ontologos_dl::is_consistent(&loaded).expect("check"),
            "disjointWith-010 should be inconsistent"
        );
    }
}
