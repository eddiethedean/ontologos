//! HermiT-style DL hyperresolution clause formatting.

use std::cmp::Ordering;

use ontologos_core::{CeId, ClassExpr, DataExpr, DeId, EntityId, Ontology};

const HERMIT_NS: &str = "file:/c/test.owl#";
const OWL_THING: &str = "http://www.w3.org/2002/07/owl#Thing";
const RDFS_LITERAL: &str = "http://www.w3.org/2000/01/rdf-schema#Literal";

/// Variable or individual term in a DL atom.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Term {
    /// DL variable (`X`, `Y`, `Y1`, …).
    Var(String),
    /// Named individual (`:a`).
    Ind(String),
}

impl Term {
    fn fmt(&self) -> String {
        match self {
            Self::Var(v) => v.clone(),
            Self::Ind(i) => i.clone(),
        }
    }
}

/// Formatted data range for HermiT clause text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataRangeFmt {
    /// Named datatype (`xsd:integer`).
    Datatype(String),
    /// Enumeration (`{ "a" "b"^^xsd:int }`).
    OneOf(Vec<LiteralFmt>),
    /// Negated range (`not(xsd:string)` or `not({ ... })`).
    Not(Box<DataRangeFmt>),
    /// Internal normalized datatype (`defdata:0`).
    Internal(String),
}

/// Literal in a data enumeration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LiteralFmt {
    /// Lexical form.
    pub lexical: String,
    /// Optional datatype suffix (`^^xsd:int`).
    pub datatype: Option<String>,
}

/// Atom in a HermiT DL clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HyperAtom {
    /// Concept atom `:A(X)`.
    Concept { name: String, term: Term },
    /// Data-range atom `xsd:integer(Y)`.
    DataRange { range: DataRangeFmt, term: Term },
    /// `atLeast(n :dp range)(X)`.
    AtLeastData {
        n: u32,
        role: String,
        range: DataRangeFmt,
        term: Term,
    },
    /// `atLeast(n role concept)(X)` for object restrictions.
    AtLeastObject {
        n: u32,
        role: String,
        concept: String,
        term: Term,
    },
    /// Object/data role `:dp(X,Y)`.
    Role {
        role: String,
        subject: Term,
        object: Term,
    },
    /// Equality `Y1 == Y2`.
    Equality { left: Term, right: Term },
    /// `atMost(n role concept)(X)` with equality annotation in head.
    AtMostAnnotated {
        n: u32,
        role: String,
        concept: String,
        term: Term,
        eq_left: Term,
        eq_right: Term,
    },
    /// Node ordering `Y1 <= Y2`.
    NodeLe { left: Term, right: Term },
    /// `NodeIDsAscendingOrEqual(Y1,Y2,…)`.
    NodeIDsAscendingOrEqual { vars: Vec<Term> },
    /// Negated concept fact `not def:1(:a)`.
    NotConcept { name: String, term: Term },
}

/// Hyperresolution clause (`head v … :- body, …`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperClause {
    /// Head disjuncts (may be empty).
    pub head: Vec<HyperAtom>,
    /// Body conjuncts.
    pub body: Vec<HyperAtom>,
}

/// Clause set with positive facts.
#[derive(Debug, Clone, Default)]
pub struct HyperClauseSet {
    clauses: Vec<HyperClause>,
    facts: Vec<HyperAtom>,
}

impl HyperClauseSet {
    /// Empty set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a clause.
    pub fn push_clause(&mut self, clause: HyperClause) {
        self.clauses.push(clause);
    }

    /// Append a ground fact.
    pub fn push_fact(&mut self, fact: HyperAtom) {
        self.facts.push(fact);
    }

    /// Serialize to HermiT test strings (sorted multiset).
    #[must_use]
    pub fn to_strings(&self) -> Vec<String> {
        let mut out: Vec<String> = self.clauses.iter().map(format_hyper_clause).collect();
        out.extend(self.facts.iter().map(format_hyper_atom));
        out.sort();
        out
    }
}

/// Format a clause set for an ontology (stable HermiT abbreviations).
pub fn format_hyper_clauses(ontology: &Ontology, set: &HyperClauseSet) -> Vec<String> {
    let _ = ontology;
    set.to_strings()
}

fn format_hyper_clause(clause: &HyperClause) -> String {
    let mut head: Vec<String> = clause.head.iter().map(format_hyper_atom).collect();
    head.sort();
    let body: Vec<String> = if clause
        .body
        .iter()
        .any(|a| matches!(a, HyperAtom::NodeIDsAscendingOrEqual { .. }))
    {
        clause.body.iter().map(format_hyper_atom).collect()
    } else {
        let mut body: Vec<String> = clause.body.iter().map(format_hyper_atom).collect();
        body.sort();
        body
    };
    let body_s = body.join(", ");
    if head.is_empty() {
        return format!(":- {body_s}");
    }
    let head_s = head.join(" v ");
    format!("{head_s} :- {body_s}")
}

fn format_hyper_atom(atom: &HyperAtom) -> String {
    match atom {
        HyperAtom::Concept { name, term } => format!("{name}({})", term.fmt()),
        HyperAtom::DataRange { range, term } => {
            format!("{}({})", format_data_range(range), term.fmt())
        }
        HyperAtom::AtLeastData {
            n,
            role,
            range,
            term,
        } => format!(
            "atLeast({n} {role} {})({})",
            format_data_range(range),
            term.fmt()
        ),
        HyperAtom::AtLeastObject {
            n,
            role,
            concept,
            term,
        } => format!("atLeast({n} {role} {concept})({})", term.fmt()),
        HyperAtom::Role {
            role,
            subject,
            object,
        } => {
            format!("{role}({},{})", subject.fmt(), object.fmt())
        }
        HyperAtom::Equality { left, right } => format!("{} == {}", left.fmt(), right.fmt()),
        HyperAtom::AtMostAnnotated {
            n,
            role,
            concept,
            term,
            eq_left,
            eq_right,
        } => format!(
            "[{} == {}]@atMost({n} {role} {concept})({})",
            eq_left.fmt(),
            eq_right.fmt(),
            term.fmt()
        ),
        HyperAtom::NodeLe { left, right } => format!("{} <= {}", left.fmt(), right.fmt()),
        HyperAtom::NodeIDsAscendingOrEqual { vars } => {
            let args: Vec<String> = vars.iter().map(Term::fmt).collect();
            format!("NodeIDsAscendingOrEqual({})", args.join(","))
        }
        HyperAtom::NotConcept { name, term } => format!("not {name}({})", term.fmt()),
    }
}

fn format_data_range(range: &DataRangeFmt) -> String {
    match range {
        DataRangeFmt::Datatype(dt) => dt.clone(),
        DataRangeFmt::Internal(name) => name.clone(),
        DataRangeFmt::OneOf(lits) => {
            let parts: Vec<String> = lits.iter().map(format_literal).collect();
            format!("{{ {} }}", parts.join(" "))
        }
        DataRangeFmt::Not(inner) => format!("not({})", format_data_range(inner)),
    }
}

pub fn format_literal(lit: &LiteralFmt) -> String {
    match &lit.datatype {
        Some(dt) => format!("\"{}\"^^{}", lit.lexical, dt),
        None => format!("\"{}\"", lit.lexical),
    }
}

fn atom_sort_key(atom: &HyperAtom) -> (String, usize, Vec<String>) {
    let s = format_hyper_atom(atom);
    // HermiT sorts by predicate then arity then arguments; formatted string is a workable proxy.
    (s.clone(), s.matches('(').count(), vec![s])
}

impl PartialOrd for HyperAtom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HyperAtom {
    fn cmp(&self, other: &Self) -> Ordering {
        atom_sort_key(self).cmp(&atom_sort_key(other))
    }
}

/// Build a HermiT abbreviation for an entity IRI.
pub fn abbrev_entity_iri(iri: &str) -> String {
    let iri = iri.replace("%23", "#");
    if let Some(local) = iri.strip_prefix(HERMIT_NS) {
        return format!(":{local}");
    }
    if let Some(idx) = iri.rfind('#') {
        let local = &iri[idx + 1..];
        if !local.is_empty() {
            return format!(":{local}");
        }
    }
    if iri == OWL_THING {
        return "owl:Thing".into();
    }
    if let Some(n) = iri.strip_prefix("internal:def#") {
        return format!("def:{n}");
    }
    if let Some(n) = iri.strip_prefix("internal:defdata#") {
        return format!("defdata:{n}");
    }
    if let Some(n) = iri.strip_prefix("internal:all#") {
        return format!("all:{n}");
    }
    if let Some(n) = iri.strip_prefix("internal:nom#") {
        return format!("nom:{n}");
    }
    datatype_shorthand(&iri).unwrap_or_else(|| format!("<{iri}>"))
}

/// Map XSD / RDFS datatype IRIs to HermiT prefixes.
pub fn datatype_shorthand(iri: &str) -> Option<String> {
    match iri {
        "http://www.w3.org/2001/XMLSchema#integer" => Some("xsd:integer".into()),
        "http://www.w3.org/2001/XMLSchema#int" => Some("xsd:int".into()),
        "http://www.w3.org/2001/XMLSchema#string" => Some("xsd:string".into()),
        "http://www.w3.org/2001/XMLSchema#double" => Some("xsd:double".into()),
        "http://www.w3.org/2001/XMLSchema#decimal" => Some("xsd:decimal".into()),
        "http://www.w3.org/2001/XMLSchema#nonNegativeInteger" => {
            Some("xsd:nonNegativeInteger".into())
        }
        "http://www.w3.org/2001/XMLSchema#nonPositiveInteger" => {
            Some("xsd:nonPositiveInteger".into())
        }
        RDFS_LITERAL => Some("rdfs:Literal".into()),
        _ => None,
    }
}

/// Literal datatype suffix for enumerations (`xsd:int` for integer literals).
pub fn literal_datatype_suffix(iri: &str) -> Option<String> {
    match iri {
        "http://www.w3.org/2001/XMLSchema#integer" | "http://www.w3.org/2001/XMLSchema#int" => {
            Some("xsd:int".into())
        }
        "http://www.w3.org/2001/XMLSchema#string" => None,
        "http://www.w3.org/2001/XMLSchema#double" => Some("xsd:double".into()),
        "http://www.w3.org/2001/XMLSchema#decimal" => Some("xsd:decimal".into()),
        "http://www.w3.org/2001/XMLSchema#nonNegativeInteger" => {
            Some("xsd:nonNegativeInteger".into())
        }
        _ => datatype_shorthand(iri),
    }
}

/// Coerce literals for HermiT enumeration display (e.g. `nonNegativeInteger` `5` → `xsd:int`).
pub fn coerce_enumeration_literal(lit: &mut LiteralFmt) {
    if lit.datatype.as_deref() == Some("xsd:nonNegativeInteger")
        && lit.lexical.chars().all(|c| c.is_ascii_digit())
    {
        lit.datatype = Some("xsd:int".into());
    }
    if lit.datatype.as_deref() == Some("xsd:integer") {
        lit.datatype = Some("xsd:int".into());
    }
}

pub(crate) fn sort_oneof_literals(lits: &mut [LiteralFmt]) {
    if lits.len() <= 1 {
        return;
    }
    let all_int = lits
        .iter()
        .all(|l| matches!(l.datatype.as_deref(), Some("xsd:int")));
    if all_int {
        lits.sort_by_key(|l| std::cmp::Reverse(format_literal(l)));
        return;
    }
    let has_double = lits.iter().any(|l| {
        matches!(
            l.datatype.as_deref(),
            Some("xsd:double") | Some("xsd:decimal")
        )
    });
    let has_int = lits
        .iter()
        .any(|l| matches!(l.datatype.as_deref(), Some("xsd:int") | Some("xsd:integer")));
    if has_double && has_int {
        lits.sort_by_key(|l| match l.datatype.as_deref() {
            Some("xsd:double") | Some("xsd:decimal") => 0,
            _ => 1,
        });
    }
}

/// Canonical IRI for signature dedupe (percent-encoded `#` normalized).
pub(crate) fn entity_canonical_iri(ontology: &Ontology, id: EntityId) -> String {
    ontology
        .entity(id)
        .ok()
        .and_then(|r| ontology.iris().resolve(r.iri).ok())
        .map(|iri| iri.replace("%23", "#"))
        .unwrap_or_default()
}

/// Entity abbreviation from ontology registry.
pub fn abbrev_entity(ontology: &Ontology, id: EntityId) -> String {
    let iri = ontology
        .entity(id)
        .ok()
        .and_then(|r| ontology.iris().resolve(r.iri).ok())
        .unwrap_or("<?>");
    abbrev_entity_iri(iri)
}

/// Format a class expression id as a concept name (atomic only).
pub fn concept_name(ontology: &Ontology, ce: CeId) -> Option<String> {
    match ontology.dl().ce(ce)? {
        ClassExpr::Atomic(id) => Some(abbrev_entity(ontology, *id)),
        ClassExpr::Top => Some("owl:Thing".into()),
        ClassExpr::Bottom => Some("owl:Nothing".into()),
        _ => None,
    }
}

/// Convert a stored data expression to HermiT range format (after NNF / simplification).
pub fn data_range_fmt(
    ontology: &Ontology,
    de: DeId,
    internal: &dyn Fn(DeId) -> Option<String>,
) -> Option<DataRangeFmt> {
    let expr = ontology.dl().de(de)?.clone();
    data_range_fmt_expr(ontology, de, &expr, internal)
}

fn data_range_fmt_expr(
    ontology: &Ontology,
    de: DeId,
    expr: &DataExpr,
    internal: &dyn Fn(DeId) -> Option<String>,
) -> Option<DataRangeFmt> {
    match expr {
        DataExpr::Top => Some(DataRangeFmt::Datatype("rdfs:Literal".into())),
        DataExpr::Datatype(id) => {
            let iri = ontology
                .entity(*id)
                .ok()
                .and_then(|r| ontology.iris().resolve(r.iri).ok())?;
            if iri.starts_with("internal:defdata#") {
                let n = iri.rsplit('#').next().unwrap_or("0");
                return Some(DataRangeFmt::Internal(format!("defdata:{n}")));
            }
            datatype_shorthand(iri).map(DataRangeFmt::Datatype)
        }
        DataExpr::Literal { lexical, datatype } => {
            let iri = ontology
                .entity(*datatype)
                .ok()
                .and_then(|r| ontology.iris().resolve(r.iri).ok())?;
            Some(DataRangeFmt::OneOf(vec![LiteralFmt {
                lexical: lexical.clone(),
                datatype: literal_datatype_suffix(iri),
            }]))
        }
        DataExpr::Or(ids) => {
            let mut lits: Vec<LiteralFmt> = ids
                .iter()
                .filter_map(|id| {
                    let DataExpr::Literal { lexical, datatype } = ontology.dl().de(*id)? else {
                        return None;
                    };
                    let iri = ontology
                        .entity(*datatype)
                        .ok()
                        .and_then(|r| ontology.iris().resolve(r.iri).ok())?;
                    Some(LiteralFmt {
                        lexical: lexical.clone(),
                        datatype: literal_datatype_suffix(iri),
                    })
                })
                .collect();
            if lits.len() == ids.len() {
                for lit in &mut lits {
                    coerce_enumeration_literal(lit);
                }
                sort_oneof_literals(&mut lits);
                Some(DataRangeFmt::OneOf(lits))
            } else {
                None
            }
        }
        DataExpr::Not(inner) => {
            let inner_fmt = data_range_fmt(ontology, *inner, internal)?;
            Some(DataRangeFmt::Not(Box::new(inner_fmt)))
        }
        DataExpr::And(ids) => {
            if ids.len() == 1 {
                data_range_fmt(ontology, ids[0], internal)
            } else {
                internal(de).map(DataRangeFmt::Internal)
            }
        }
        DataExpr::Facet { base, .. } => data_range_fmt(ontology, *base, internal),
    }
}

/// Negate a data range format (for max-cardinality / inverted some).
pub fn negate_data_range(range: &DataRangeFmt) -> DataRangeFmt {
    DataRangeFmt::Not(Box::new(range.clone()))
}

/// Role abbreviation.
pub fn abbrev_role(ontology: &Ontology, property: EntityId) -> String {
    abbrev_entity(ontology, property)
}

/// Build a role atom, swapping arguments for inverse roles.
pub fn role_atom(
    ontology: &Ontology,
    property: &ontologos_core::RoleExpr,
    subject: Term,
    object: Term,
) -> HyperAtom {
    match property {
        ontologos_core::RoleExpr::Atomic(id) => HyperAtom::Role {
            role: abbrev_role(ontology, *id),
            subject,
            object,
        },
        ontologos_core::RoleExpr::Inverse(id) => HyperAtom::Role {
            role: abbrev_role(ontology, *id),
            subject: object,
            object: subject,
        },
    }
}
