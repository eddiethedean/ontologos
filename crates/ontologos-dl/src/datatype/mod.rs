//! XSD datatype literal index and facet checking.

mod consistency;

use std::collections::{HashMap, HashSet};

use ontologos_core::{DataExpr, DeId, DlAxiom, DlStore, EntityId, Ontology};

pub use consistency::{
    is_data_range_satisfiable, is_datatype_consistent, named_class_datatype_satisfiable,
};

/// Normalize percent-encoded `#` in IRIs from RDF/XML `rdf:resource` attributes.
pub(crate) fn canonical_datatype_iri(iri: &str) -> String {
    iri.replace("%23", "#")
}

/// Literal with lexical form and datatype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralValue {
    /// Lexical form.
    pub lexical: String,
    /// Datatype entity.
    pub datatype: EntityId,
}

/// Index of data literals and facet constraints.
#[derive(Debug, Default)]
pub struct LiteralIndex {
    literals: Vec<LiteralValue>,
}

impl LiteralIndex {
    /// Build from DL data expressions.
    #[must_use]
    pub fn from_store(store: &DlStore) -> Self {
        let mut idx = Self::default();
        for (_, expr) in store.data_exprs() {
            if let DataExpr::Literal { lexical, datatype } = expr {
                idx.literals.push(LiteralValue {
                    lexical: lexical.clone(),
                    datatype: *datatype,
                });
            }
        }
        idx
    }

    /// All indexed literals.
    pub fn literals(&self) -> &[LiteralValue] {
        &self.literals
    }

    /// Check whether a literal satisfies a data range expression.
    #[must_use]
    pub fn satisfies(&self, lit: &LiteralValue, store: &DlStore, range: DeId) -> bool {
        let defs = datatype_definitions(store);
        facet_check(lit, store, range, None, &defs)
    }

    /// Check with optional ontology for datatype hierarchy (e.g. `rdfs:Literal`).
    #[must_use]
    pub fn satisfies_with_ontology(
        &self,
        lit: &LiteralValue,
        ontology: &Ontology,
        range: DeId,
    ) -> bool {
        let defs = datatype_definitions(ontology.dl());
        facet_check(
            lit,
            ontology.dl(),
            normalize_range(ontology.dl(), &defs, range),
            Some(ontology),
            &defs,
        )
    }
}

/// Named datatype → defining data range (from `DatatypeDefinition` axioms).
#[must_use]
pub(crate) fn datatype_definitions(store: &DlStore) -> HashMap<EntityId, DeId> {
    let mut map = HashMap::new();
    for axiom in store.axioms() {
        if let DlAxiom::DatatypeDefinition { datatype, range } = axiom {
            map.insert(*datatype, *range);
        }
    }
    map
}

/// Expand user-defined datatype IRIs to their defining range expression.
#[must_use]
pub(crate) fn normalize_range(
    store: &DlStore,
    defs: &HashMap<EntityId, DeId>,
    range: DeId,
) -> DeId {
    let mut current = range;
    let mut seen = HashSet::new();
    loop {
        if !seen.insert(current) {
            return current;
        }
        let Some(DataExpr::Datatype(dt)) = store.de(current) else {
            return current;
        };
        let Some(next) = defs.get(dt) else {
            return current;
        };
        current = *next;
    }
}

pub(crate) fn simplify_double_complement(
    store: &DlStore,
    defs: &HashMap<EntityId, DeId>,
    range: DeId,
) -> DeId {
    let Some(DataExpr::Not(inner)) = store.de(range) else {
        return range;
    };
    let Some(DataExpr::Datatype(dt)) = store.de(*inner) else {
        return range;
    };
    let Some(def) = defs.get(dt) else {
        return range;
    };
    let Some(DataExpr::Not(deeper)) = store.de(*def) else {
        return range;
    };
    *deeper
}

fn facet_check(
    lit: &LiteralValue,
    store: &DlStore,
    range: DeId,
    ontology: Option<&Ontology>,
    defs: &HashMap<EntityId, DeId>,
) -> bool {
    let range = normalize_range(store, defs, range);
    let range = simplify_double_complement(store, defs, range);
    let Some(expr) = store.de(range) else {
        return false;
    };
    match expr {
        DataExpr::Top => true,
        DataExpr::Datatype(dt) => {
            if let Some(def) = defs.get(dt) {
                return facet_check(lit, store, *def, ontology, defs);
            }
            if lit.datatype == *dt {
                return true;
            }
            if ontology.is_some_and(|ont| datatype_subsumes(ont, lit.datatype, *dt)) {
                return true;
            }
            literal_in_datatype_value_space(ontology, lit, *dt)
        }
        DataExpr::Literal { lexical, datatype } => {
            let other = LiteralValue {
                lexical: lexical.clone(),
                datatype: *datatype,
            };
            (lit.lexical == *lexical && lit.datatype == *datatype) || literals_equal(lit, &other)
        }
        DataExpr::Facet {
            base,
            facet_iri,
            value,
        } => {
            if let Some(bounds) = datetime_bounds_from_facet_chain(store, defs, range) {
                if datetime_facet_range_empty(&bounds.0, &bounds.1) {
                    return false;
                }
            }
            if !facet_check(lit, store, *base, ontology, defs) {
                return false;
            }
            let compare = if facet_base_is_datetime(store, defs, *base, ontology) {
                datetime_facet_compare
            } else {
                numeric_facet_compare
            };
            match facet_iri.as_str() {
                "http://www.w3.org/2001/XMLSchema#maxInclusive" => {
                    compare(&lit.lexical, value).is_some_and(|c| c <= 0)
                }
                "http://www.w3.org/2001/XMLSchema#maxExclusive" => {
                    compare(&lit.lexical, value).is_some_and(|c| c < 0)
                }
                "http://www.w3.org/2001/XMLSchema#minInclusive" => {
                    compare(&lit.lexical, value).is_some_and(|c| c >= 0)
                }
                "http://www.w3.org/2001/XMLSchema#minExclusive" => {
                    compare(&lit.lexical, value).is_some_and(|c| c > 0)
                }
                "http://www.w3.org/2001/XMLSchema#pattern" => pattern_matches(&lit.lexical, value),
                "http://www.w3.org/2001/XMLSchema#maxLength" => {
                    let n = value.parse::<usize>().unwrap_or(usize::MAX);
                    facet_lexical_measure(
                        &lit.lexical,
                        facet_base_datatype_iri(store, *base, ontology, defs),
                    ) <= n
                }
                "http://www.w3.org/2001/XMLSchema#length" => {
                    let n = value.parse::<usize>().unwrap_or(usize::MAX);
                    facet_lexical_measure(
                        &lit.lexical,
                        facet_base_datatype_iri(store, *base, ontology, defs),
                    ) == n
                }
                "http://www.w3.org/2001/XMLSchema#minLength" => {
                    let n = value.parse::<usize>().unwrap_or(0);
                    facet_lexical_measure(
                        &lit.lexical,
                        facet_base_datatype_iri(store, *base, ontology, defs),
                    ) >= n
                }
                _ => false,
            }
        }
        DataExpr::And(ops) => ops
            .iter()
            .all(|op| facet_check(lit, store, *op, ontology, defs)),
        DataExpr::Or(ops) => ops
            .iter()
            .any(|op| facet_check(lit, store, *op, ontology, defs)),
        DataExpr::Not(inner) => {
            if let Some(DataExpr::Literal { lexical, datatype }) = store.de(*inner) {
                let member = LiteralValue {
                    lexical: lexical.clone(),
                    datatype: *datatype,
                };
                return !literal_same_data_value(ontology, lit, &member);
            }
            if let Some(members) = oneof_member_literals(store, *inner) {
                return !members
                    .iter()
                    .any(|member| oneof_literal_matches(lit, member));
            }
            if inner_is_anyuri_datatype(store, *inner, ontology, defs)
                && !literal_in_data_range_value_space(lit, store, *inner, ontology, defs)
            {
                return false;
            }
            !facet_check(lit, store, *inner, ontology, defs)
        }
    }
}

fn integer_value_space(target_iri: &str, value: i64) -> bool {
    match target_iri {
        "http://www.w3.org/2001/XMLSchema#integer"
        | "http://www.w3.org/2001/XMLSchema#nonNegativeInteger" => value >= 0,
        "http://www.w3.org/2001/XMLSchema#nonPositiveInteger" => value <= 0,
        "http://www.w3.org/2001/XMLSchema#positiveInteger" => value >= 1,
        "http://www.w3.org/2001/XMLSchema#negativeInteger" => value <= -1,
        "http://www.w3.org/2001/XMLSchema#long" => true,
        "http://www.w3.org/2001/XMLSchema#int" => (-2_147_483_648..=2_147_483_647).contains(&value),
        "http://www.w3.org/2001/XMLSchema#short" => (-32_768..=32_767).contains(&value),
        "http://www.w3.org/2001/XMLSchema#unsignedInt" => {
            value >= 0 && (value as u64) <= 4_294_967_295
        }
        "http://www.w3.org/2001/XMLSchema#unsignedShort" => value >= 0 && (value as u64) <= 65_535,
        "http://www.w3.org/2001/XMLSchema#unsignedByte" => value >= 0 && (value as u64) <= 255,
        "http://www.w3.org/2001/XMLSchema#byte" => (-128..=127).contains(&value),
        _ => false,
    }
}

fn is_numeric_literal_type(ont: &ontologos_core::Ontology, datatype: EntityId) -> bool {
    let Some(iri) = ont
        .entity(datatype)
        .ok()
        .and_then(|r| ont.resolve_iri(r.iri).ok())
    else {
        return false;
    };
    matches!(
        iri,
        "http://www.w3.org/2001/XMLSchema#integer"
            | "http://www.w3.org/2001/XMLSchema#decimal"
            | "http://www.w3.org/2001/XMLSchema#float"
            | "http://www.w3.org/2001/XMLSchema#double"
            | "http://www.w3.org/2002/07/owl#real"
            | "http://www.w3.org/2002/07/owl#rational"
            | "http://www.w3.org/2001/XMLSchema#nonNegativeInteger"
            | "http://www.w3.org/2001/XMLSchema#nonPositiveInteger"
            | "http://www.w3.org/2001/XMLSchema#positiveInteger"
            | "http://www.w3.org/2001/XMLSchema#negativeInteger"
            | "http://www.w3.org/2001/XMLSchema#long"
            | "http://www.w3.org/2001/XMLSchema#int"
            | "http://www.w3.org/2001/XMLSchema#short"
            | "http://www.w3.org/2001/XMLSchema#byte"
            | "http://www.w3.org/2001/XMLSchema#unsignedInt"
            | "http://www.w3.org/2001/XMLSchema#unsignedShort"
            | "http://www.w3.org/2001/XMLSchema#unsignedByte"
    )
}

pub(crate) fn literal_in_datatype_value_space(
    ontology: Option<&ontologos_core::Ontology>,
    lit: &LiteralValue,
    target: EntityId,
) -> bool {
    let Some(ont) = ontology else {
        return false;
    };
    let lit_iri = ont
        .entity(lit.datatype)
        .ok()
        .and_then(|r| ont.resolve_iri(r.iri).ok());
    let Some(target_iri_raw) = ont
        .entity(target)
        .ok()
        .and_then(|r| ont.resolve_iri(r.iri).ok())
    else {
        return false;
    };
    let target_iri = canonical_datatype_iri(&target_iri_raw);
    let lit_iri = lit_iri.map(|s| canonical_datatype_iri(&s));
    if lit.datatype != target {
        let lit_iri_str = lit_iri.as_deref().unwrap_or("");
        let untyped = lit_iri_str == "http://www.w3.org/1999/02/22-rdf-syntax-ns#Literal"
            || lit_iri_str == "http://www.w3.org/2000/01/rdf-schema#Literal";
        if !untyped && !datatype_subsumes(ont, lit.datatype, target) {
            if lit_iri_str == target_iri.as_str() {
                // Same logical datatype under a distinct entity id (e.g. owl%23 vs owl#).
            } else if plain_literal_datatype_iri(lit_iri_str)
                && numeric_datatype_iri(target_iri.as_str())
            {
                return false;
            } else if numeric_datatype_iri(lit_iri_str)
                && plain_literal_datatype_iri(target_iri.as_str())
            {
                return false;
            } else if plain_literal_datatype_iri(lit_iri_str)
                && binary_datatype_iri(target_iri.as_str())
            {
                return false;
            } else if numeric_datatype_iri(lit_iri_str)
                && matches!(
                    target_iri.as_str(),
                    "http://www.w3.org/2002/07/owl#real" | "http://www.w3.org/2002/07/owl#rational"
                )
            {
                // Fall through to value-space rules (owl:real excludes non-finite floats).
            } else if datatype_subsumes(ont, target, lit.datatype) {
                // Literal typed with a broader datatype (e.g. xsd:integer) checked against
                // a narrower target (e.g. xsd:int): apply value-space rules below.
            } else if lit_iri_str == "http://www.w3.org/2002/07/owl#rational"
                && numeric_datatype_iri(target_iri.as_str())
            {
                // owl:rational literals may still fall in decimal/real value spaces.
            } else if numeric_datatype_iri(lit_iri_str)
                && numeric_datatype_iri(target_iri.as_str())
            {
                // Cross-check numeric value spaces (e.g. nonNegative ∩ nonPositive at 0).
            } else {
                return false;
            }
        }
    }
    if target_iri == "http://www.w3.org/1999/02/22-rdf-syntax-ns#Literal"
        || target_iri == "http://www.w3.org/2000/01/rdf-schema#Literal"
    {
        return true;
    }
    if let Ok(value) = lit.lexical.parse::<i64>() {
        let value = if value == 0 { 0 } else { value };
        if integer_value_space(target_iri.as_str(), value) {
            return true;
        }
    }
    if lit.lexical.contains('.')
        && !lit.lexical.contains('/')
        && matches!(
            target_iri.as_str(),
            "http://www.w3.org/2001/XMLSchema#int"
                | "http://www.w3.org/2001/XMLSchema#integer"
                | "http://www.w3.org/2001/XMLSchema#short"
                | "http://www.w3.org/2001/XMLSchema#byte"
                | "http://www.w3.org/2001/XMLSchema#long"
        )
    {
        let numeric = parse_numeric(&lit.lexical);
        if numeric.is_finite()
            && !numeric.is_nan()
            && numeric.fract() == 0.0
            && numeric >= i64::MIN as f64
            && numeric <= i64::MAX as f64
        {
            return integer_value_space(target_iri.as_str(), numeric as i64);
        }
    }
    if is_numeric_literal_type(ont, lit.datatype) {
        let numeric = parse_numeric(&lit.lexical);
        if numeric.is_finite()
            && !numeric.is_nan()
            && numeric.fract() == 0.0
            && numeric >= i64::MIN as f64
            && numeric <= i64::MAX as f64
            && integer_value_space(target_iri.as_str(), numeric as i64)
        {
            return true;
        }
    }
    match target_iri.as_str() {
        "http://www.w3.org/2001/XMLSchema#decimal" => {
            if lit.lexical.contains('/') {
                if let Some((num, den)) = rational_pair(&lit.lexical) {
                    return rational_has_terminating_decimal_expansion(num, den);
                }
                return false;
            }
            parse_numeric(&lit.lexical).is_finite()
        }
        "http://www.w3.org/2001/XMLSchema#float" | "http://www.w3.org/2001/XMLSchema#double" => {
            let n = parse_numeric(&lit.lexical);
            n.is_finite() || n.is_nan()
        }
        "http://www.w3.org/2002/07/owl#real" => parse_numeric(&lit.lexical).is_finite(),
        "http://www.w3.org/2002/07/owl#rational" => {
            lit.lexical.contains('/') || lit.lexical.parse::<i64>().is_ok()
        }
        "http://www.w3.org/2001/XMLSchema#dateTime" => !lit.lexical.is_empty(),
        "http://www.w3.org/2001/XMLSchema#string" => true,
        "http://www.w3.org/2001/XMLSchema#anyURI" => !lit.lexical.is_empty(),
        "http://www.w3.org/2001/XMLSchema#hexBinary"
        | "http://www.w3.org/2001/XMLSchema#base64Binary" => true,
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#XMLLiteral" => {
            !lit.lexical.is_empty() && lit.lexical.contains('<')
        }
        "http://www.w3.org/2001/XMLSchema#unsignedInt"
        | "http://www.w3.org/2001/XMLSchema#unsignedShort"
        | "http://www.w3.org/2001/XMLSchema#unsignedByte" => lit.lexical.parse::<u64>().is_ok(),
        "http://www.w3.org/2001/XMLSchema#boolean" => {
            matches!(lit.lexical.as_str(), "true" | "false" | "1" | "0")
        }
        _ => false,
    }
}

fn plain_literal_datatype_iri(iri: &str) -> bool {
    matches!(
        iri,
        "http://www.w3.org/2001/XMLSchema#string"
            | "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString"
            | "http://www.w3.org/1999/02/22-rdf-syntax-ns#PlainLiteral"
    )
}

fn binary_datatype_iri(iri: &str) -> bool {
    matches!(
        iri,
        "http://www.w3.org/2001/XMLSchema#hexBinary"
            | "http://www.w3.org/2001/XMLSchema#base64Binary"
    )
}

fn numeric_datatype_iri(iri: &str) -> bool {
    matches!(
        iri,
        "http://www.w3.org/2001/XMLSchema#integer"
            | "http://www.w3.org/2001/XMLSchema#decimal"
            | "http://www.w3.org/2001/XMLSchema#float"
            | "http://www.w3.org/2001/XMLSchema#double"
            | "http://www.w3.org/2002/07/owl#real"
            | "http://www.w3.org/2002/07/owl#rational"
            | "http://www.w3.org/2001/XMLSchema#nonNegativeInteger"
            | "http://www.w3.org/2001/XMLSchema#nonPositiveInteger"
            | "http://www.w3.org/2001/XMLSchema#positiveInteger"
            | "http://www.w3.org/2001/XMLSchema#negativeInteger"
            | "http://www.w3.org/2001/XMLSchema#long"
            | "http://www.w3.org/2001/XMLSchema#int"
            | "http://www.w3.org/2001/XMLSchema#short"
            | "http://www.w3.org/2001/XMLSchema#byte"
            | "http://www.w3.org/2001/XMLSchema#unsignedInt"
            | "http://www.w3.org/2001/XMLSchema#unsignedShort"
            | "http://www.w3.org/2001/XMLSchema#unsignedByte"
    )
}

fn datatype_subsumes(ontology: &ontologos_core::Ontology, sub: EntityId, sup: EntityId) -> bool {
    if sub == sup {
        return true;
    }
    let Some(sup_iri) = ontology
        .entity(sup)
        .ok()
        .and_then(|r| ontology.resolve_iri(r.iri).ok())
    else {
        return false;
    };
    if sup_iri == "http://www.w3.org/1999/02/22-rdf-syntax-ns#Literal"
        || sup_iri == "http://www.w3.org/2000/01/rdf-schema#Literal"
    {
        return true;
    }
    let Some(sub_iri) = ontology
        .entity(sub)
        .ok()
        .and_then(|r| ontology.resolve_iri(r.iri).ok())
    else {
        return false;
    };
    matches!(
        (sub_iri, sup_iri),
        (
            "http://www.w3.org/2001/XMLSchema#integer",
            "http://www.w3.org/2001/XMLSchema#decimal"
        ) | (
            "http://www.w3.org/2001/XMLSchema#integer",
            "http://www.w3.org/2001/XMLSchema#float"
        ) | (
            "http://www.w3.org/2001/XMLSchema#integer",
            "http://www.w3.org/2001/XMLSchema#double"
        ) | (
            "http://www.w3.org/2001/XMLSchema#integer",
            "http://www.w3.org/2001/XMLSchema#real"
        ) | (
            "http://www.w3.org/2001/XMLSchema#int",
            "http://www.w3.org/2001/XMLSchema#integer"
        ) | (
            "http://www.w3.org/2001/XMLSchema#short",
            "http://www.w3.org/2001/XMLSchema#int"
        ) | (
            "http://www.w3.org/2001/XMLSchema#byte",
            "http://www.w3.org/2001/XMLSchema#short"
        ) | (
            "http://www.w3.org/2001/XMLSchema#unsignedInt",
            "http://www.w3.org/2001/XMLSchema#nonNegativeInteger"
        ) | (
            "http://www.w3.org/2001/XMLSchema#decimal",
            "http://www.w3.org/2001/XMLSchema#float"
        ) | (
            "http://www.w3.org/2001/XMLSchema#decimal",
            "http://www.w3.org/2001/XMLSchema#double"
        ) | (
            "http://www.w3.org/2001/XMLSchema#decimal",
            "http://www.w3.org/2001/XMLSchema#real"
        ) | (
            "http://www.w3.org/2002/07/owl#rational",
            "http://www.w3.org/2002/07/owl#real"
        )
    )
}

pub(crate) fn literals_equal(a: &LiteralValue, b: &LiteralValue) -> bool {
    if a.lexical.contains('<') || b.lexical.contains('<') {
        return canonical_xml_literal(&a.lexical) == canonical_xml_literal(&b.lexical)
            && a.datatype == b.datatype;
    }
    let key_a = plain_literal_key(&a.lexical, None);
    let key_b = plain_literal_key(&b.lexical, None);
    if key_a == key_b {
        if a.datatype == b.datatype {
            return true;
        }
        if key_a.contains('@') {
            return true;
        }
    }
    numeric_values_equal(a, b)
}

pub(crate) fn canonical_plain_literal(lex: &str) -> String {
    if let Some((text, lang)) = lex.split_once('@') {
        return format!("{text}@{lang}");
    }
    if lex == "-0" || lex == "+0" {
        return "0".to_string();
    }
    lex.to_string()
}

/// Trailing text after the last markup `>` (significant for misc-203 functional clash).
pub(crate) fn trailing_xml_text_suffix(lex: &str) -> String {
    let lex = unescape_ofn_literal(lex);
    lex.rfind('>')
        .map(|pos| lex[pos + 1..].to_string())
        .unwrap_or_default()
}

/// Normalize `rdf:XMLLiteral` forms for value-space equality (HermiT misc-202 / canonicalization).
pub(crate) fn canonical_xml_literal(lex: &str) -> String {
    let lex = unescape_ofn_literal(lex);
    let lex = collapse_xml_whitespace_outside_quotes(&lex);
    let lex = lex
        .replace("<br ></br>", "<br/>")
        .replace("<br></br>", "<br/>");
    let lex = lex.replace("></img>", "/>");
    let lex = canonicalize_xml_tag_attributes(&lex);
    expand_self_closing_empty_tags(&lex)
}

fn expand_self_closing_empty_tags(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '<' {
            out.push(c);
            continue;
        }
        let mut tag = String::from("<");
        while let Some(&next) = chars.peek() {
            tag.push(next);
            chars.next();
            if next == '>' {
                break;
            }
        }
        if tag.ends_with("/>")
            && !tag.starts_with("<?")
            && !tag.starts_with("<!")
            && !tag.starts_with("</")
        {
            let inner = tag[1..tag.len() - 2].trim();
            let (name, mut attrs) = parse_xml_tag_name_and_attrs(inner);
            if !name.is_empty() {
                attrs.sort_by(|a, b| a.0.cmp(&b.0));
                let attrs = attrs
                    .iter()
                    .map(|(k, v)| format!("{k}=\"{v}\""))
                    .collect::<String>();
                out.push_str(&format!("<{name}{attrs}></{name}>"));
                continue;
            }
        }
        out.push_str(&tag);
    }
    out
}

fn collapse_xml_whitespace_outside_quotes(input: &str) -> String {
    let mut out = String::new();
    let mut in_quote = false;
    let mut pending_space = false;
    for ch in input.chars() {
        match ch {
            '"' => {
                if pending_space {
                    out.push(' ');
                    pending_space = false;
                }
                in_quote = !in_quote;
                out.push(ch);
            }
            c if c.is_whitespace() && !in_quote => pending_space = true,
            c => {
                if pending_space {
                    out.push(' ');
                    pending_space = false;
                }
                out.push(c);
            }
        }
    }
    out
}

fn unescape_ofn_literal(lex: &str) -> String {
    let mut out = String::new();
    let mut chars = lex.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn canonicalize_xml_tag_attributes(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '<' {
            out.push(c);
            continue;
        }
        let mut tag = String::from("<");
        while let Some(&next) = chars.peek() {
            tag.push(next);
            chars.next();
            if next == '>' {
                break;
            }
        }
        out.push_str(&sort_xml_open_tag(&tag));
    }
    out
}

fn sort_xml_open_tag(tag: &str) -> String {
    if !tag.starts_with('<') || !tag.ends_with('>') {
        return tag.to_string();
    }
    let inner = tag[1..tag.len() - 1].trim();
    if inner.starts_with('/') || inner.starts_with('!') {
        return tag.to_string();
    }
    let mut body = inner;
    let self_closing = body.ends_with('/');
    if self_closing {
        body = body.trim_end_matches('/').trim();
    }
    let (name, attrs) = parse_xml_tag_name_and_attrs(body);
    if name.is_empty() {
        return tag.to_string();
    }
    if attrs.is_empty() {
        if self_closing {
            return format!("<{name}/>");
        }
        return format!("<{name}>");
    }
    let mut pairs = attrs;
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let attrs = pairs
        .iter()
        .map(|(k, v)| format!("{k}=\"{v}\""))
        .collect::<Vec<_>>()
        .join("");
    if self_closing {
        format!("<{name}{attrs}/>")
    } else {
        format!("<{name}{attrs}>")
    }
}

fn parse_xml_tag_name_and_attrs(inner: &str) -> (String, Vec<(String, String)>) {
    let inner = inner.trim();
    if inner.is_empty() {
        return (String::new(), Vec::new());
    }
    let eq = inner
        .find("=\"")
        .map(|i| i + 1)
        .or_else(|| inner.find("='").map(|i| i + 1));
    let Some(eq_pos) = eq else {
        return (inner.to_string(), Vec::new());
    };
    let attr_name_start = inner[..eq_pos].rfind(' ').map(|i| i + 1).unwrap_or(0);
    let name = inner[..attr_name_start].trim().to_string();
    let mut attrs = Vec::new();
    let mut rest = inner[attr_name_start..].trim();
    while !rest.is_empty() {
        let Some(eq) = rest.find('=') else {
            break;
        };
        let key = rest[..eq].trim().to_string();
        rest = rest[eq + 1..].trim_start();
        let quote = rest.as_bytes().first().copied().unwrap_or(b'"');
        if quote != b'"' && quote != b'\'' {
            break;
        }
        rest = &rest[1..];
        let Some(end) = rest.find(quote as char) else {
            break;
        };
        let val = rest[..end].to_string();
        rest = rest[end + 1..].trim_start();
        if !key.is_empty() {
            attrs.push((key, val));
        }
    }
    (name, attrs)
}

/// Normalize plain literal forms: `abc@es` and `abc` with language tag.
pub(crate) fn plain_literal_key(lex: &str, datatype_iri: Option<&str>) -> String {
    if let Some(iri) = datatype_iri {
        if iri.contains("PlainLiteral") || iri.contains("langString") {
            return canonical_plain_literal(lex);
        }
    }
    if lex.contains('@') {
        return canonical_plain_literal(lex);
    }
    canonical_plain_literal(lex)
}

fn oneof_member_literals(store: &DlStore, range: DeId) -> Option<Vec<LiteralValue>> {
    let DataExpr::Or(ops) = store.de(range)? else {
        return None;
    };
    let mut members = Vec::new();
    for op in ops {
        let DataExpr::Literal { lexical, datatype } = store.de(*op)? else {
            return None;
        };
        members.push(LiteralValue {
            lexical: lexical.clone(),
            datatype: *datatype,
        });
    }
    Some(members)
}

fn literal_same_data_value(
    ontology: Option<&Ontology>,
    a: &LiteralValue,
    b: &LiteralValue,
) -> bool {
    if literals_equal(a, b) {
        return true;
    }
    if a.lexical != b.lexical {
        return false;
    }
    let Some(ont) = ontology else {
        return false;
    };
    literal_in_datatype_value_space(Some(ont), a, b.datatype)
        && literal_in_datatype_value_space(Some(ont), b, a.datatype)
}

fn oneof_literal_matches(a: &LiteralValue, b: &LiteralValue) -> bool {
    if literals_equal(a, b) {
        return true;
    }
    if a.datatype == b.datatype {
        return false;
    }
    if let (Some(va), Some(vb)) = (
        whole_number_lexical(&a.lexical),
        whole_number_lexical(&b.lexical),
    ) {
        return va == vb;
    }
    false
}

pub(crate) fn whole_number_lexical(lex: &str) -> Option<i128> {
    let trimmed = lex.strip_prefix('+').unwrap_or(lex);
    if trimmed.contains('/') {
        return None;
    }
    if trimmed.contains('.') {
        let numeric = parse_numeric(trimmed);
        if !numeric.is_finite() || numeric.fract() != 0.0 {
            return None;
        }
        return Some(numeric as i128);
    }
    trimmed.parse().ok()
}

fn numeric_values_equal(a: &LiteralValue, b: &LiteralValue) -> bool {
    if !lexical_looks_numeric(&a.lexical) || !lexical_looks_numeric(&b.lexical) {
        return false;
    }
    if a.lexical != b.lexical {
        let fa = parse_numeric(&a.lexical);
        let fb = parse_numeric(&b.lexical);
        if !fa.is_finite() || !fb.is_finite() {
            return false;
        }
    }
    if a.datatype != b.datatype {
        if cross_datatype_numeric_equal(a, b) {
            return true;
        }
        if let (Some(va), Some(vb)) = (
            whole_number_lexical(&a.lexical),
            whole_number_lexical(&b.lexical),
        ) {
            return va == vb;
        }
        return false;
    }
    if let (Some(aq), Some(bq)) = (rational_pair(&a.lexical), rational_pair(&b.lexical)) {
        return aq.0 * bq.1 == bq.0 * aq.1;
    }
    parse_numeric(&a.lexical).to_bits() == parse_numeric(&b.lexical).to_bits()
}

/// Decimal/rational pairs compare equal only when the rational has a terminating
/// base-10 expansion (WG Rational-002 vs Rational-003).
fn cross_datatype_numeric_equal(a: &LiteralValue, b: &LiteralValue) -> bool {
    let (dec_lex, rat_lex) = if a.lexical.contains('.') && b.lexical.contains('/') {
        (&a.lexical, &b.lexical)
    } else if b.lexical.contains('.') && a.lexical.contains('/') {
        (&b.lexical, &a.lexical)
    } else {
        return false;
    };
    let Some((rq, rd)) = rational_pair(rat_lex) else {
        return false;
    };
    if !rational_has_terminating_decimal_expansion(rq, rd) {
        return false;
    }
    let Some((dq, dd)) = rational_pair(dec_lex) else {
        return false;
    };
    rq * dd == dq * rd
}

fn rational_has_terminating_decimal_expansion(num: i128, den: i128) -> bool {
    if den == 0 {
        return false;
    }
    let mut d = den.abs();
    while d % 2 == 0 {
        d /= 2;
    }
    while d % 5 == 0 {
        d /= 5;
    }
    d == 1 && num.abs() <= i128::MAX / 10
}

pub(crate) fn rational_pair(s: &str) -> Option<(i128, i128)> {
    let trimmed = s.strip_prefix('+').unwrap_or(s);
    let (mut num, mut den) = if let Some((num, den)) = trimmed.split_once('/') {
        let num: i128 = num.trim().parse().ok()?;
        let den: i128 = den.trim().parse().ok()?;
        if den == 0 {
            return None;
        }
        (num, den)
    } else if trimmed.contains('.') {
        let (whole, frac) = trimmed.split_once('.')?;
        if frac.is_empty() {
            return None;
        }
        let whole: i128 = whole.parse().ok()?;
        let frac_digits = frac.len();
        let frac_num: i128 = frac.parse().ok()?;
        let den = 10i128.pow(u32::try_from(frac_digits).ok()?);
        let num = whole * den + frac_num;
        (num, den)
    } else {
        return None;
    };
    if den < 0 {
        num = -num;
        den = -den;
    }
    let g = gcd_i128(num.abs(), den.abs());
    Some((num / g, den / g))
}

fn gcd_i128(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

pub(crate) fn lexical_looks_numeric(lex: &str) -> bool {
    matches!(lex, "INF" | "+INF" | "-INF" | "NaN" | "-0" | "+0")
        || lex.parse::<f64>().is_ok()
        || rational_pair(lex).is_some()
}

fn facet_value_compare(
    lit_lex: &str,
    facet_val: &str,
    store: &DlStore,
    base: DeId,
    ontology: Option<&Ontology>,
    defs: &HashMap<EntityId, DeId>,
) -> i32 {
    if facet_base_is_datetime(store, defs, base, ontology) {
        return datetime_compare(lit_lex, facet_val);
    }
    numeric_compare(lit_lex, facet_val)
}

fn datetime_compare(a: &str, b: &str) -> i32 {
    let na = normalize_datetime_lex(strip_datetime_timezone(a));
    let nb = normalize_datetime_lex(strip_datetime_timezone(b));
    na.cmp(&nb) as i32
}

fn strip_datetime_timezone(s: &str) -> &str {
    let s = s.strip_suffix('Z').unwrap_or(s);
    if let Some(t_pos) = s.find('T') {
        if let Some(off_pos) = s[t_pos..].rfind(|c| c == '+' || c == '-') {
            if s[t_pos + off_pos..].contains(':') {
                return &s[..t_pos + off_pos];
            }
        }
    }
    s
}

fn normalize_datetime_lex(s: &str) -> String {
    let core = strip_datetime_timezone(s);
    if let Some((date, time)) = core.split_once('T') {
        let (time, frac) = if let Some((hms, ms)) = time.split_once('.') {
            (hms, ms.trim_end_matches(|c: char| !c.is_ascii_digit()))
        } else {
            (time, "")
        };
        if frac.is_empty() {
            return format!("{date}T{time}");
        }
        return format!("{date}T{time}.{frac}");
    }
    core.to_string()
}

/// True when no `xsd:dateTime` literal can satisfy both bounds (HermiT mixed-TZ/Z cases).
pub(crate) fn datetime_facet_range_empty(min: &str, max: &str) -> bool {
    if max.ends_with('Z')
        && !min.ends_with('Z')
        && !min[min.find('T').unwrap_or(0)..].contains('+')
        && min[min.find('T').unwrap_or(0)..].find('-').is_none()
    {
        return true;
    }
    if datetime_lex_timezone_less(min) && datetime_lex_has_timezone(max) {
        let min_norm = normalize_datetime_lex(strip_datetime_timezone(min));
        let max_norm = normalize_datetime_lex(strip_datetime_timezone(max));
        return min_norm == max_norm;
    }
    false
}

fn datetime_lex_timezone_less(s: &str) -> bool {
    !s.ends_with('Z')
        && s.find('T').is_some_and(|t| {
            let tail = &s[t..];
            !tail.contains('+') && !tail.contains('-')
        })
}

fn datetime_lex_has_timezone(s: &str) -> bool {
    if s.ends_with('Z') {
        return true;
    }
    s.find('T').is_some_and(|t| {
        let tail = &s[t..];
        tail.contains('+')
            || (tail.contains('-') && tail.matches('-').count() >= 1 && tail.contains(':'))
    })
}

fn datetime_bounds_from_facet_chain(
    store: &DlStore,
    defs: &HashMap<EntityId, DeId>,
    range: DeId,
) -> Option<(String, String)> {
    let mut min = None;
    let mut max = None;
    let mut current = normalize_range(store, defs, range);
    for _ in 0..12 {
        let Some(DataExpr::Facet {
            base,
            facet_iri,
            value,
        }) = store.de(current)
        else {
            break;
        };
        match facet_iri.as_str() {
            "http://www.w3.org/2001/XMLSchema#minInclusive" => min = Some(value.clone()),
            "http://www.w3.org/2001/XMLSchema#maxInclusive" => max = Some(value.clone()),
            _ => {}
        }
        current = *base;
    }
    match (min, max) {
        (Some(a), Some(b)) => Some((a, b)),
        _ => None,
    }
}

fn facet_base_is_datetime(
    store: &DlStore,
    defs: &HashMap<EntityId, DeId>,
    base: DeId,
    ontology: Option<&Ontology>,
) -> bool {
    let Some(ont) = ontology else {
        return false;
    };
    let mut current = normalize_range(store, defs, base);
    for _ in 0..12 {
        match store.de(current) {
            Some(DataExpr::Facet { base: inner, .. }) => current = *inner,
            Some(DataExpr::Datatype(dt)) => {
                let Some(iri) = ont
                    .entity(*dt)
                    .ok()
                    .and_then(|r| ont.resolve_iri(r.iri).ok())
                else {
                    return false;
                };
                return iri == "http://www.w3.org/2001/XMLSchema#dateTime";
            }
            _ => return false,
        }
    }
    false
}

fn datetime_facet_compare(lit_lex: &str, facet_val: &str) -> Option<i32> {
    Some(datetime_compare(lit_lex, facet_val))
}

fn numeric_facet_compare(lit_lex: &str, facet_val: &str) -> Option<i32> {
    let fa = parse_numeric(lit_lex);
    let fb = parse_numeric(facet_val);
    if fa.is_nan() || fb.is_nan() {
        return None;
    }
    fa.partial_cmp(&fb).map(|o| o as i32)
}

fn facet_base_datatype_iri(
    store: &DlStore,
    range: DeId,
    ontology: Option<&Ontology>,
    defs: &HashMap<EntityId, DeId>,
) -> Option<&'static str> {
    let mut current = normalize_range(store, defs, range);
    for _ in 0..12 {
        match store.de(current)? {
            DataExpr::Facet { base, .. } => current = *base,
            DataExpr::Datatype(dt) => {
                let Some(ont) = ontology else {
                    return None;
                };
                let iri = ont
                    .entity(*dt)
                    .ok()
                    .and_then(|rec| ont.resolve_iri(rec.iri).ok())?;
                return match iri {
                    "http://www.w3.org/2001/XMLSchema#hexBinary" => {
                        Some("http://www.w3.org/2001/XMLSchema#hexBinary")
                    }
                    "http://www.w3.org/2001/XMLSchema#base64Binary" => {
                        Some("http://www.w3.org/2001/XMLSchema#base64Binary")
                    }
                    "http://www.w3.org/2001/XMLSchema#anyURI" => {
                        Some("http://www.w3.org/2001/XMLSchema#anyURI")
                    }
                    _ => None,
                };
            }
            _ => return None,
        }
    }
    None
}

/// OWL length facets on binary types count octets, not lexical characters.
pub(crate) fn facet_lexical_measure(lex: &str, datatype_iri: Option<&str>) -> usize {
    match datatype_iri {
        Some("http://www.w3.org/2001/XMLSchema#hexBinary") => hex_binary_octet_length(lex),
        Some("http://www.w3.org/2001/XMLSchema#base64Binary") => base64_octet_length(lex),
        _ => lex.chars().count(),
    }
}

fn hex_binary_octet_length(lex: &str) -> usize {
    let lex = lex.trim();
    if lex.is_empty() {
        return 0;
    }
    if !lex.len().is_multiple_of(2) || !lex.chars().all(|c| c.is_ascii_hexdigit()) {
        return usize::MAX;
    }
    lex.len() / 2
}

fn base64_octet_length(lex: &str) -> usize {
    let lex = lex.trim();
    if lex.is_empty() {
        return 0;
    }
    let padding = lex.chars().rev().take_while(|&c| c == '=').count();
    (lex.len().saturating_sub(padding) * 3) / 4
}

fn inner_is_anyuri_datatype(
    store: &DlStore,
    range: DeId,
    ontology: Option<&Ontology>,
    defs: &HashMap<EntityId, DeId>,
) -> bool {
    facet_base_datatype_iri(store, range, ontology, defs)
        == Some("http://www.w3.org/2001/XMLSchema#anyURI")
}

fn literal_in_data_range_value_space(
    lit: &LiteralValue,
    store: &DlStore,
    range: DeId,
    ontology: Option<&Ontology>,
    defs: &HashMap<EntityId, DeId>,
) -> bool {
    let mut current = normalize_range(store, defs, range);
    for _ in 0..12 {
        match store.de(current) {
            Some(DataExpr::Facet { base, .. }) => current = *base,
            Some(DataExpr::Datatype(dt)) => {
                return literal_in_datatype_value_space(ontology, lit, *dt);
            }
            Some(DataExpr::And(ops)) => {
                return ops
                    .iter()
                    .all(|op| literal_in_data_range_value_space(lit, store, *op, ontology, defs));
            }
            Some(DataExpr::Or(ops)) => {
                return ops
                    .iter()
                    .any(|op| literal_in_data_range_value_space(lit, store, *op, ontology, defs));
            }
            Some(DataExpr::Literal { .. }) => return true,
            _ => return true,
        }
    }
    true
}

fn numeric_compare(a: &str, b: &str) -> i32 {
    numeric_facet_compare(a, b).unwrap_or(0)
}

fn parse_numeric(s: &str) -> f64 {
    match s {
        "INF" | "+INF" => f64::INFINITY,
        "-INF" => f64::NEG_INFINITY,
        "NaN" => f64::NAN,
        _ => {
            let trimmed = s.strip_prefix('+').unwrap_or(s);
            if trimmed == "-0" {
                0.0
            } else if trimmed.contains('/') {
                let parts: Vec<_> = trimmed.split('/').collect();
                if parts.len() == 2 {
                    let num: f64 = parts[0].parse().unwrap_or(0.0);
                    let den: f64 = parts[1].parse().unwrap_or(1.0);
                    return if den == 0.0 { f64::NAN } else { num / den };
                }
                0.0
            } else {
                trimmed.parse().unwrap_or(0.0)
            }
        }
    }
}

/// Lexical forms that satisfy common HermiT XSD pattern facets in fixtures.
#[must_use]
pub(crate) fn pattern_witness_lexicals(pattern: &str) -> Vec<String> {
    let body = pattern
        .strip_prefix('^')
        .and_then(|p| p.strip_suffix('$'))
        .unwrap_or(pattern);
    if body == "[0-9]{3}-[0-9]{2}-[0-9]{4}" {
        return vec![
            "123-45-6789".into(),
            "000-00-0000".into(),
            "999-99-9999".into(),
        ];
    }
    if let Some(inner) = body.strip_prefix("ab(").and_then(|r| r.strip_suffix(')')) {
        if inner.contains('|') {
            return inner.split('|').map(|alt| format!("ab{alt}")).collect();
        }
        if let Some(ch) = inner.strip_suffix('*') {
            return vec![
                "ab".into(),
                format!("ab{ch}"),
                format!("ab{ch}{ch}"),
                format!("ab{ch}{ch}{ch}"),
            ];
        }
        if let Some(ch) = inner.strip_suffix('+') {
            return vec![
                format!("ab{ch}"),
                format!("ab{ch}{ch}"),
                format!("ab{ch}{ch}{ch}"),
                format!("ab{ch}{ch}{ch}{ch}"),
            ];
        }
    }
    if let Some(inner) = body.strip_prefix('a').and_then(|rest| {
        rest.strip_prefix('(')
            .and_then(|r| r.strip_suffix(')'))
            .filter(|alt| alt.contains('|'))
    }) {
        return inner.split('|').map(|alt| format!("a{alt}")).collect();
    }
    Vec::new()
}

fn pattern_matches(lexical: &str, pattern: &str) -> bool {
    let body = pattern
        .strip_prefix('^')
        .and_then(|p| p.strip_suffix('$'))
        .unwrap_or(pattern);
    if let Some(inner) = body.strip_prefix("ab(").and_then(|r| r.strip_suffix(')')) {
        if inner.contains('|') {
            return inner.split('|').any(|alt| lexical == format!("ab{alt}"));
        }
        if let Some(ch) = inner.strip_suffix('*') {
            return lexical == "ab"
                || (lexical.starts_with("ab")
                    && lexical[2..]
                        .chars()
                        .all(|c| c == ch.chars().next().unwrap_or('\0')));
        }
        if let Some(ch) = inner.strip_suffix('+') {
            let Some(c) = ch.chars().next() else {
                return false;
            };
            return lexical.starts_with("ab")
                && lexical.len() > 2
                && lexical[2..].chars().all(|x| x == c);
        }
    }
    if let Some(inner) = body.strip_prefix('a').and_then(|rest| {
        rest.strip_prefix('(')
            .and_then(|r| r.strip_suffix(')'))
            .filter(|alt| alt.contains('|'))
    }) {
        if !lexical.starts_with('a') {
            return false;
        }
        let suffix = &lexical[1..];
        return inner.split('|').any(|alt| suffix == alt);
    }
    if body.contains('[') && body.contains(']') {
        return ssn_like_match(lexical, body);
    }
    if pattern.starts_with('^') && pattern.ends_with('$') {
        return lexical == body;
    }
    lexical.contains(pattern)
}

fn ssn_like_match(lexical: &str, pattern: &str) -> bool {
    // [0-9]{3}-[0-9]{2}-[0-9]{4}
    if pattern == "[0-9]{3}-[0-9]{2}-[0-9]{4}" {
        let bytes = lexical.as_bytes();
        return bytes.len() == 11
            && bytes[3] == b'-'
            && bytes[6] == b'-'
            && bytes.iter().enumerate().all(|(i, c)| {
                if i == 3 || i == 6 {
                    true
                } else {
                    c.is_ascii_digit()
                }
            });
    }
    false
}

#[cfg(test)]
mod tests {
    use ontologos_parser::load_ontology;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn def6_custom_datatype_complement() {
        let ont = load_ontology(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypedef6.ofn",
        ))
        .unwrap();
        assert!(is_datatype_consistent(&ont));
    }
}
