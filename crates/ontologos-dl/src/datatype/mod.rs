//! XSD datatype literal index and facet checking.

mod consistency;

use std::collections::{HashMap, HashSet};

use ontologos_core::{DataExpr, DeId, DlAxiom, DlStore, EntityId, Ontology};

pub use consistency::{is_datatype_consistent, named_class_datatype_satisfiable};

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
            if !facet_check(lit, store, *base, ontology, defs) {
                return false;
            }
            match facet_iri.as_str() {
                "http://www.w3.org/2001/XMLSchema#maxInclusive" => {
                    facet_value_compare(&lit.lexical, value, store, *base, ontology, defs) <= 0
                }
                "http://www.w3.org/2001/XMLSchema#maxExclusive" => {
                    facet_value_compare(&lit.lexical, value, store, *base, ontology, defs) < 0
                }
                "http://www.w3.org/2001/XMLSchema#minInclusive" => {
                    facet_value_compare(&lit.lexical, value, store, *base, ontology, defs) >= 0
                }
                "http://www.w3.org/2001/XMLSchema#minExclusive" => {
                    facet_value_compare(&lit.lexical, value, store, *base, ontology, defs) > 0
                }
                "http://www.w3.org/2001/XMLSchema#pattern" => pattern_matches(&lit.lexical, value),
                "http://www.w3.org/2001/XMLSchema#maxLength"
                | "http://www.w3.org/2001/XMLSchema#length" => {
                    lit.lexical.len() <= value.parse::<usize>().unwrap_or(usize::MAX)
                }
                "http://www.w3.org/2001/XMLSchema#minLength" => {
                    lit.lexical.len() >= value.parse::<usize>().unwrap_or(0)
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
        DataExpr::Not(inner) => !facet_check(lit, store, *inner, ontology, defs),
    }
}

fn integer_value_space(target_iri: &str, value: i64) -> bool {
    match target_iri {
        "http://www.w3.org/2001/XMLSchema#integer"
        | "http://www.w3.org/2001/XMLSchema#nonNegativeInteger" => value >= 0,
        "http://www.w3.org/2001/XMLSchema#nonPositiveInteger" => value <= 0,
        "http://www.w3.org/2001/XMLSchema#positiveInteger" => value >= 1,
        "http://www.w3.org/2001/XMLSchema#negativeInteger" => value <= -1,
        "http://www.w3.org/2001/XMLSchema#long"
        | "http://www.w3.org/2001/XMLSchema#int"
        | "http://www.w3.org/2001/XMLSchema#short"
        | "http://www.w3.org/2001/XMLSchema#unsignedInt"
        | "http://www.w3.org/2001/XMLSchema#unsignedShort"
        | "http://www.w3.org/2001/XMLSchema#unsignedByte" => value >= 0,
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

fn literal_in_datatype_value_space(
    ontology: Option<&ontologos_core::Ontology>,
    lit: &LiteralValue,
    target: EntityId,
) -> bool {
    let Some(ont) = ontology else {
        return false;
    };
    let Some(target_iri) = ont
        .entity(target)
        .ok()
        .and_then(|r| ont.resolve_iri(r.iri).ok())
    else {
        return false;
    };
    if target_iri == "http://www.w3.org/1999/02/22-rdf-syntax-ns#Literal"
        || target_iri == "http://www.w3.org/2000/01/rdf-schema#Literal"
    {
        return true;
    }
    if let Ok(value) = lit.lexical.parse::<i64>() {
        let value = if value == 0 { 0 } else { value };
        return integer_value_space(target_iri, value);
    }
    if is_numeric_literal_type(ont, lit.datatype) {
        let numeric = parse_numeric(&lit.lexical);
        if numeric.is_finite()
            && !numeric.is_nan()
            && numeric.fract() == 0.0
            && numeric >= i64::MIN as f64
            && numeric <= i64::MAX as f64
        {
            return integer_value_space(target_iri, numeric as i64);
        }
    }
    match target_iri {
        "http://www.w3.org/2001/XMLSchema#decimal"
        | "http://www.w3.org/2001/XMLSchema#float"
        | "http://www.w3.org/2001/XMLSchema#double"
        | "http://www.w3.org/2002/07/owl#real" => parse_numeric(&lit.lexical).is_finite(),
        "http://www.w3.org/2002/07/owl#rational" => {
            lit.lexical.contains('/') || lit.lexical.parse::<i64>().is_ok()
        }
        "http://www.w3.org/2001/XMLSchema#dateTime" => !lit.lexical.is_empty(),
        "http://www.w3.org/2001/XMLSchema#unsignedInt"
        | "http://www.w3.org/2001/XMLSchema#unsignedShort"
        | "http://www.w3.org/2001/XMLSchema#unsignedByte" => lit.lexical.parse::<u64>().is_ok(),
        "http://www.w3.org/2001/XMLSchema#boolean" => {
            matches!(lit.lexical.as_str(), "true" | "false" | "1" | "0")
        }
        _ => false,
    }
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

fn numeric_values_equal(a: &LiteralValue, b: &LiteralValue) -> bool {
    if !lexical_looks_numeric(&a.lexical) || !lexical_looks_numeric(&b.lexical) {
        return false;
    }
    if a.datatype != b.datatype {
        return cross_datatype_numeric_equal(a, b);
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
        || lex.contains('/')
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
        return lit_lex.cmp(facet_val) as i32;
    }
    numeric_compare(lit_lex, facet_val)
}

fn facet_base_is_datetime(
    store: &DlStore,
    defs: &HashMap<EntityId, DeId>,
    base: DeId,
    ontology: Option<&Ontology>,
) -> bool {
    let base = normalize_range(store, defs, base);
    let Some(DataExpr::Datatype(dt)) = store.de(base) else {
        return false;
    };
    let Some(ont) = ontology else {
        return false;
    };
    let Some(iri) = ont
        .entity(*dt)
        .ok()
        .and_then(|r| ont.resolve_iri(r.iri).ok())
    else {
        return false;
    };
    iri == "http://www.w3.org/2001/XMLSchema#dateTime"
}

fn numeric_compare(a: &str, b: &str) -> i32 {
    let fa = parse_numeric(a);
    let fb = parse_numeric(b);
    fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal) as i32
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

fn pattern_matches(lexical: &str, pattern: &str) -> bool {
    let body = pattern
        .strip_prefix('^')
        .and_then(|p| p.strip_suffix('$'))
        .unwrap_or(pattern);
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
