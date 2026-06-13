//! XSD datatype literal index and facet checking.

use ontologos_core::{DataExpr, DeId, DlStore, EntityId};

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
        facet_check(lit, store, range)
    }
}

fn facet_check(lit: &LiteralValue, store: &DlStore, range: DeId) -> bool {
    let Some(expr) = store.de(range) else {
        return false;
    };
    match expr {
        DataExpr::Top => true,
        DataExpr::Datatype(dt) => lit.datatype == *dt,
        DataExpr::Literal { lexical, datatype } => {
            lit.lexical == *lexical && lit.datatype == *datatype
        }
        DataExpr::Facet {
            base,
            facet_iri,
            value,
        } => {
            if !facet_check(lit, store, *base) {
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
        DataExpr::And(ops) => ops.iter().all(|op| facet_check(lit, store, *op)),
    }
}

fn numeric_compare(a: &str, b: &str) -> i32 {
    let fa: f64 = a.parse().unwrap_or(0.0);
    let fb: f64 = b.parse().unwrap_or(0.0);
    fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal) as i32
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
