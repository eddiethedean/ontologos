use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// OWL 2 construct observed during parsing (for profile detection).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum OwlConstruct {
    /// `SubClassOf` between named classes.
    SubClassOfNamed,
    /// `SubClassOf` with `ObjectSomeValuesFrom` on the super class side.
    SubClassOfExistential,
    /// `SubClassOf` with `ObjectIntersectionOf`.
    SubClassOfIntersection,
    /// `ObjectIntersectionOf` class expression.
    ObjectIntersectionOf,
    /// `ObjectUnionOf` class expression.
    ObjectUnionOf,
    /// `ObjectComplementOf` class expression.
    ObjectComplementOf,
    /// `ObjectAllValuesFrom` (∀ restriction).
    ObjectAllValuesFrom,
    /// `ObjectSomeValuesFrom` class expression.
    ObjectSomeValuesFrom,
    /// `ObjectOneOf` / nominals.
    ObjectOneOf,
    /// Cardinality restrictions.
    ObjectCardinality,
    /// `ObjectHasValue` restriction.
    ObjectHasValue,
    /// `ObjectHasSelf`.
    ObjectHasSelf,
    /// `EquivalentClasses`.
    EquivalentClasses,
    /// `DisjointClasses`.
    DisjointClasses,
    /// `DisjointObjectProperties`.
    DisjointObjectProperties,
    /// `DisjointUnion`.
    DisjointUnion,
    /// `SubObjectPropertyOf`.
    SubObjectPropertyOf,
    /// Property chain in subproperty axiom.
    SubObjectPropertyChain,
    /// `EquivalentObjectProperties`.
    EquivalentObjectProperties,
    /// `InverseObjectProperties`.
    InverseObjectProperties,
    /// `TransitiveObjectProperty`.
    TransitiveObjectProperty,
    /// `SymmetricObjectProperty`.
    SymmetricObjectProperty,
    /// `AsymmetricObjectProperty`.
    AsymmetricObjectProperty,
    /// `ReflexiveObjectProperty`.
    ReflexiveObjectProperty,
    /// `IrreflexiveObjectProperty`.
    IrreflexiveObjectProperty,
    /// `FunctionalObjectProperty`.
    FunctionalObjectProperty,
    /// `InverseFunctionalObjectProperty`.
    InverseFunctionalObjectProperty,
    /// `ObjectPropertyDomain`.
    ObjectPropertyDomain,
    /// `ObjectPropertyRange`.
    ObjectPropertyRange,
    /// `HasKey`.
    HasKey,
    /// `SameIndividual` / `DifferentIndividuals`.
    IndividualEquality,
    /// `ClassAssertion`.
    ClassAssertion,
    /// `ObjectPropertyAssertion`.
    ObjectPropertyAssertion,
    /// `DataPropertyAssertion` or data properties.
    DataPropertyAssertion,
    /// `SubDataPropertyOf` / data property axioms.
    DataPropertyAxiom,
    /// `Datatype` / facet usage.
    Datatype,
    /// SWRL rule or other non-axiom component.
    SwrlRule,
    /// Annotation-only axiom (not used for profile escalation).
    Annotation,
    /// Import declaration.
    Import,
    /// Unmapped or unknown logical component.
    Unknown,
}

/// Metadata from OWL file parsing (not serialized in JSON v2 snapshots).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParseMeta {
    /// Constructs observed in the source ontology.
    pub constructs: BTreeSet<OwlConstruct>,
    /// Constructs from successfully mapped logical axioms (profile detection input).
    pub profile_constructs: BTreeSet<OwlConstruct>,
    /// Non-fatal parse warnings (skipped axioms, unsupported shapes).
    pub warnings: Vec<String>,
    /// Axioms successfully stored in the core model.
    pub mapped_axiom_count: usize,
    /// Logical axioms skipped during mapping.
    pub skipped_axiom_count: usize,
    /// Total logical components visited in the source (mapped + skipped).
    pub logical_axiom_count: usize,
}

impl ParseMeta {
    /// Maximum number of warnings retained from parsing.
    pub const MAX_WARNINGS: usize = 10_000;

    /// Record a construct flag.
    pub fn note_construct(&mut self, construct: OwlConstruct) {
        self.constructs.insert(construct);
    }

    /// Record a construct from a mapped logical axiom (profile detection).
    pub fn note_profile_construct(&mut self, construct: OwlConstruct) {
        self.profile_constructs.insert(construct.clone());
        self.constructs.insert(construct);
    }

    /// Append a warning message.
    pub fn warn(&mut self, message: impl Into<String>) {
        if self.warnings.len() >= Self::MAX_WARNINGS {
            return;
        }
        self.warnings.push(message.into());
    }
}

/// User-facing parse metadata for CLI and Python bindings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ParseMetaSummary {
    /// Non-fatal parse warnings (skipped axioms, unsupported shapes).
    pub warnings: Vec<String>,
    /// Axioms successfully stored in the core model.
    pub mapped_axiom_count: usize,
    /// Logical axioms skipped during mapping.
    pub skipped_axiom_count: usize,
    /// Total logical components visited in the source (mapped + skipped).
    pub logical_axiom_count: usize,
}

impl ParseMetaSummary {
    /// Returns true when there are skipped axioms or warning messages to report.
    pub fn has_issues(&self) -> bool {
        self.skipped_axiom_count > 0 || !self.warnings.is_empty()
    }

    /// Omit from JSON output when parsing produced no warnings or skipped axioms.
    pub fn omit_from_json(&self) -> bool {
        !self.has_issues()
    }

    /// Print parse summary and warnings to stderr (CLI text mode).
    pub fn emit_stderr(&self) {
        if !self.has_issues() {
            return;
        }
        eprintln!(
            "warning: parse skipped {} of {} logical axioms",
            self.skipped_axiom_count, self.logical_axiom_count
        );
        for warning in &self.warnings {
            eprintln!("warning: {warning}");
        }
    }
}

impl From<&ParseMeta> for ParseMetaSummary {
    fn from(meta: &ParseMeta) -> Self {
        Self {
            warnings: meta.warnings.clone(),
            mapped_axiom_count: meta.mapped_axiom_count,
            skipped_axiom_count: meta.skipped_axiom_count,
            logical_axiom_count: meta.logical_axiom_count,
        }
    }
}
