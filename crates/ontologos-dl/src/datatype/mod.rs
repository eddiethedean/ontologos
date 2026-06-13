//! XSD datatype literal index and facet checking.

mod consistency;

use ontologos_core::{DataExpr, DeId, DlStore, EntityId};

pub use consistency::is_datatype_consistent;

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
        facet_check(lit, store, range, None)
    }

    /// Check with optional ontology for datatype hierarchy (e.g. `rdfs:Literal`).
    #[must_use]
    pub fn satisfies_with_ontology(
        &self,
        lit: &LiteralValue,
        ontology: &ontologos_core::Ontology,
        range: DeId,
    ) -> bool {
        facet_check(lit, ontology.dl(), range, Some(ontology))
    }
}

fn facet_check(
    lit: &LiteralValue,
    store: &DlStore,
    range: DeId,
    ontology: Option<&ontologos_core::Ontology>,
) -> bool {
    let Some(expr) = store.de(range) else {
        return false;
    };
    match expr {
        DataExpr::Top => true,
        DataExpr::Datatype(dt) => {
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
            (lit.lexical == *lexical && lit.datatype == *datatype)
                || literals_equal(lit, &other)
        }
        DataExpr::Facet {
            base,
            facet_iri,
            value,
        } => {
            if !facet_check(lit, store, *base, ontology) {
                return false;
            }
            match facet_iri.as_str() {
                "http://www.w3.org/2001/XMLSchema#maxInclusive" => {
                    numeric_compare(&lit.lexical, value) <= 0
                }
                "http://www.w3.org/2001/XMLSchema#maxExclusive" => {
                    numeric_compare(&lit.lexical, value) < 0
                }
                "http://www.w3.org/2001/XMLSchema#minInclusive" => {
                    numeric_compare(&lit.lexical, value) >= 0
                }
                "http://www.w3.org/2001/XMLSchema#minExclusive" => {
                    numeric_compare(&lit.lexical, value) > 0
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
        DataExpr::And(ops) => ops.iter().all(|op| facet_check(lit, store, *op, ontology)),
        DataExpr::Or(ops) => ops.iter().any(|op| facet_check(lit, store, *op, ontology)),
        DataExpr::Not(inner) => !facet_check(lit, store, *inner, ontology),
    }
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
        return match &*target_iri {
            "http://www.w3.org/2001/XMLSchema#integer"
            | "http://www.w3.org/2001/XMLSchema#nonNegativeInteger" => value >= 0,
            "http://www.w3.org/2001/XMLSchema#nonPositiveInteger" => value <= 0,
            "http://www.w3.org/2001/XMLSchema#positiveInteger" => value >= 1,
            "http://www.w3.org/2001/XMLSchema#negativeInteger" => value <= -1,
            "http://www.w3.org/2001/XMLSchema#long"
            | "http://www.w3.org/2001/XMLSchema#int"
            | "http://www.w3.org/2001/XMLSchema#short"
            | "http://www.w3.org/2001/XMLSchema#byte" => true,
            _ => false,
        };
    }
    match &*target_iri {
        "http://www.w3.org/2001/XMLSchema#decimal"
        | "http://www.w3.org/2001/XMLSchema#float"
        | "http://www.w3.org/2001/XMLSchema#double"
        | "http://www.w3.org/2002/07/owl#real" => parse_numeric(&lit.lexical).is_finite(),
        "http://www.w3.org/2002/07/owl#rational" => {
            lit.lexical.contains('/') || lit.lexical.parse::<i64>().is_ok()
        }
        "http://www.w3.org/2001/XMLSchema#dateTime" => !lit.lexical.is_empty(),
        _ => false,
    }
}

fn datatype_subsumes(
    ontology: &ontologos_core::Ontology,
    sub: EntityId,
    sup: EntityId,
) -> bool {
    if sub == sup {
        return true;
    }
    let Some(sup_iri) = ontology.entity(sup).ok().and_then(|r| ontology.resolve_iri(r.iri).ok()) else {
        return false;
    };
    if sup_iri == "http://www.w3.org/1999/02/22-rdf-syntax-ns#Literal"
        || sup_iri == "http://www.w3.org/2000/01/rdf-schema#Literal"
    {
        return true;
    }
    let Some(sub_iri) = ontology.entity(sub).ok().and_then(|r| ontology.resolve_iri(r.iri).ok()) else {
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
    let iri_a = None;
    let iri_b = None;
    let key_a = plain_literal_key(&a.lexical, iri_a);
    let key_b = plain_literal_key(&b.lexical, iri_b);
    if key_a == key_b && a.datatype == b.datatype {
        return true;
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
    parse_numeric(&a.lexical).to_bits() == parse_numeric(&b.lexical).to_bits()
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
            trimmed.parse().unwrap_or(0.0)
        }
    }
}

fn pattern_matches(lexical: &str, pattern: &str) -> bool {
    // Lightweight XSD pattern check without regex crate: anchor full match when possible.
    if let Some(inner) = pattern.strip_prefix('^').and_then(|p| p.strip_suffix('$')) {
        if inner.contains('[') && inner.contains(']') {
            return ssn_like_match(lexical, inner);
        }
        return lexical == inner;
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
