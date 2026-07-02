use serde::Serialize;
use thiserror::Error;

/// Result type for facade operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Facade routing errors.
#[derive(Debug, Error)]
pub enum Error {
    /// EL engine error.
    #[error(transparent)]
    El(#[from] ontologos_el::Error),
    /// DL engine error.
    #[error(transparent)]
    Dl(#[from] ontologos_dl::Error),
    /// RL/ABox engine error.
    #[error(transparent)]
    Rl(#[from] ontologos_rl::Error),
    /// Core error (e.g. incomplete consistency folded to message).
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
}

/// Axiom-shaped entailment checks for [`crate::is_entailed_axiom`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum EntailmentCheck {
    /// Named class subsumption `SubClassOf(sub, sup)`.
    SubClassOf {
        /// Subclass IRI.
        sub: String,
        /// Superclass IRI.
        sup: String,
    },
    /// `ClassAssertion(individual, class)` with named classes.
    ClassAssertion {
        /// Individual IRI.
        individual: String,
        /// Class IRI.
        class: String,
    },
    /// `ObjectPropertyAssertion(subject, property, object)`.
    ObjectPropertyAssertion {
        /// Subject individual IRI.
        subject: String,
        /// Object property IRI.
        property: String,
        /// Object individual IRI.
        object: String,
    },
}

/// Parse CLI-style entailment flags into a single [`EntailmentCheck`].
pub fn parse_entailment_check(
    sub: Option<String>,
    sup: Option<String>,
    individual: Option<String>,
    class: Option<String>,
    subject: Option<String>,
    property: Option<String>,
    object: Option<String>,
) -> Result<EntailmentCheck> {
    let subclass = sub.is_some() || sup.is_some();
    let class_assertion = individual.is_some() || class.is_some();
    let property_assertion = subject.is_some() || property.is_some() || object.is_some();
    let count =
        usize::from(subclass) + usize::from(class_assertion) + usize::from(property_assertion);
    if count != 1 {
        return Err(Error::Core(ontologos_core::Error::Message(
            "entail requires exactly one of: (--sub and --sup), (--individual and --class), or (--subject, --property, and --object)".into(),
        )));
    }
    if subclass {
        let sub = sub
            .ok_or_else(|| Error::Core(ontologos_core::Error::Message("--sub required".into())))?;
        let sup = sup
            .ok_or_else(|| Error::Core(ontologos_core::Error::Message("--sup required".into())))?;
        return Ok(EntailmentCheck::SubClassOf { sub, sup });
    }
    if class_assertion {
        let individual = individual.ok_or_else(|| {
            Error::Core(ontologos_core::Error::Message(
                "--individual required".into(),
            ))
        })?;
        let class = class.ok_or_else(|| {
            Error::Core(ontologos_core::Error::Message("--class required".into()))
        })?;
        return Ok(EntailmentCheck::ClassAssertion { individual, class });
    }
    let subject = subject
        .ok_or_else(|| Error::Core(ontologos_core::Error::Message("--subject required".into())))?;
    let property = property
        .ok_or_else(|| Error::Core(ontologos_core::Error::Message("--property required".into())))?;
    let object = object
        .ok_or_else(|| Error::Core(ontologos_core::Error::Message("--object required".into())))?;
    Ok(EntailmentCheck::ObjectPropertyAssertion {
        subject,
        property,
        object,
    })
}
