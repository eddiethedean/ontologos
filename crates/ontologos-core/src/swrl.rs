//! DLSafe SWRL rule atoms stored on an ontology.

use serde::{Deserialize, Serialize};

use crate::EntityId;

/// SWRL individual argument: named individual or variable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SwrlIArg {
    /// Named individual entity.
    Individual(EntityId),
    /// Rule variable name (local, without leading `:`).
    Variable(String),
}

/// SWRL data argument: literal or variable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SwrlDArg {
    /// Typed or plain literal value.
    Literal {
        /// Lexical form.
        lexical: String,
        /// XSD datatype entity when known.
        datatype: Option<EntityId>,
    },
    /// Rule variable name.
    Variable(String),
}

/// A single SWRL atom in rule body or head.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SwrlAtom {
    /// `C(x)` class atom.
    Class {
        /// Named class entity.
        class: EntityId,
        /// Individual or variable.
        arg: SwrlIArg,
    },
    /// `P(x, y)` object property atom.
    ObjectProperty {
        /// Object property entity.
        property: EntityId,
        /// Subject individual or variable.
        subject: SwrlIArg,
        /// Object individual or variable.
        object: SwrlIArg,
    },
    /// `P(x, v)` data property atom.
    DataProperty {
        /// Data property entity.
        property: EntityId,
        /// Subject individual or variable.
        subject: SwrlIArg,
        /// Literal or variable.
        value: SwrlDArg,
    },
    /// `sameAs(x, y)`.
    SameIndividual(SwrlIArg, SwrlIArg),
    /// `differentFrom(x, y)`.
    DifferentIndividuals(SwrlIArg, SwrlIArg),
}

/// Parsed DLSafe SWRL rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwrlRule {
    /// Body atoms (conjunction).
    pub body: Vec<SwrlAtom>,
    /// Head atoms (conjunction).
    pub head: Vec<SwrlAtom>,
}
