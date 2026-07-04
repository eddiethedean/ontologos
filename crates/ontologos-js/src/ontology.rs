//! In-memory ontology handle for JavaScript bindings.

use std::path::Path;
use std::sync::{Arc, Mutex};

use ontologos_core::{Limits, Ontology, OntologyBuilder};
use ontologos_parser::{
    ParseLimits, load_ontology, load_ontology_from_bytes, load_ontology_from_bytes_lenient,
    load_ontology_from_bytes_with_limits, load_ontology_from_str, load_ontology_from_str_lenient,
    load_ontology_in, load_ontology_lenient,
};
use serde_json::Value;

use crate::error::{JsError, Result};

/// Shared ontology reference used by [`JsReasoner`](crate::reasoner::JsReasoner).
pub type SharedOntology = Arc<Mutex<Ontology>>;

fn finalize_loaded(mut ontology: Ontology, limits: Limits) -> Ontology {
    ontology.set_enforce_limits(limits);
    ontology
}

fn limits_from_parse(limits: ParseLimits) -> Limits {
    limits.into()
}

/// In-memory ontology for JavaScript bindings.
pub struct JsOntology {
    pub(crate) inner: SharedOntology,
}

impl JsOntology {
    pub fn from_owned(ontology: Ontology) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ontology)),
        }
    }

    fn from_owned_with_limits(ontology: Ontology, limits: Limits) -> Self {
        Self::from_owned(finalize_loaded(ontology, limits))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let limits = Limits::default();
        let ontology = Ontology::from_json_with_limits(json, limits)?;
        Ok(Self::from_owned_with_limits(ontology, limits))
    }

    pub fn from_json_with_limits(
        json: &str,
        max_json_bytes: Option<usize>,
        max_entities: Option<usize>,
        max_axioms: Option<usize>,
        max_iri_len: Option<usize>,
    ) -> Result<Self> {
        let mut limits = Limits::default();
        if let Some(n) = max_json_bytes {
            limits.max_json_bytes = n;
        }
        if let Some(n) = max_entities {
            limits.max_entities = n;
        }
        if let Some(n) = max_axioms {
            limits.max_axioms = n;
        }
        if let Some(n) = max_iri_len {
            limits.max_iri_len = n;
        }
        let ontology = Ontology::from_json_with_limits(json, limits)?;
        Ok(Self::from_owned_with_limits(ontology, limits))
    }

    pub fn from_dict(value: &Value) -> Result<Self> {
        Self::from_dict_with_limits(value, None, None, None, None)
    }

    pub fn from_dict_with_limits(
        value: &Value,
        max_json_bytes: Option<usize>,
        max_entities: Option<usize>,
        max_axioms: Option<usize>,
        max_iri_len: Option<usize>,
    ) -> Result<Self> {
        let limits = Limits::default();
        let max_json = max_json_bytes.unwrap_or(limits.max_json_bytes);
        let json = serde_json::to_string(value).map_err(|e| JsError::Other(e.to_string()))?;
        if json.len() > max_json {
            return Err(JsError::ResourceLimit(format!(
                "ontology object JSON size {} exceeds limit of {max_json} bytes",
                json.len()
            )));
        }
        Self::from_json_with_limits(&json, Some(max_json), max_entities, max_axioms, max_iri_len)
    }

    /// Load from a trusted local path (strict parse by default).
    pub fn load_path(path: &str, lenient: bool) -> Result<Self> {
        let path = Path::new(path);
        let parse_limits = if lenient {
            ParseLimits::lenient()
        } else {
            ParseLimits {
                merge_imports: true,
                ..ParseLimits::default()
            }
        };
        let ontology = if lenient {
            load_ontology_lenient(path)?
        } else {
            load_ontology(path)?
        };
        Ok(Self::from_owned_with_limits(
            ontology,
            limits_from_parse(parse_limits),
        ))
    }

    /// Sandboxed load constrained to `base` (strict parse by default).
    pub fn load_in(base: &str, path: &str, lenient: bool) -> Result<Self> {
        let base = Path::new(base);
        let path = Path::new(path);
        let parse_limits = if lenient {
            ParseLimits::lenient()
        } else {
            ParseLimits {
                merge_imports: true,
                ..ParseLimits::default()
            }
        };
        let ontology = if lenient {
            ontologos_parser::load_ontology_lenient_in(base, path)?
        } else {
            load_ontology_in(base, path)?
        };
        Ok(Self::from_owned_with_limits(
            ontology,
            limits_from_parse(parse_limits),
        ))
    }

    /// Parse in-memory bytes with strict defaults (recommended for untrusted input).
    pub fn load_bytes(bytes: &[u8]) -> Result<Self> {
        let parse_limits = ParseLimits::default();
        let ontology = load_ontology_from_bytes(bytes)?;
        Ok(Self::from_owned_with_limits(
            ontology,
            limits_from_parse(parse_limits),
        ))
    }

    /// Parse in-memory bytes leniently (trusted corpora only).
    pub fn load_bytes_lenient(bytes: &[u8]) -> Result<Self> {
        let parse_limits = ParseLimits::lenient();
        let ontology = load_ontology_from_bytes_lenient(bytes)?;
        Ok(Self::from_owned_with_limits(
            ontology,
            limits_from_parse(parse_limits),
        ))
    }

    /// Parse in-memory bytes with custom [`ParseLimits`].
    pub fn load_bytes_with_limits(bytes: &[u8], limits: ParseLimits) -> Result<Self> {
        let validate = limits.validate_output;
        let ontology = load_ontology_from_bytes_with_limits(bytes, limits, validate)?;
        Ok(Self::from_owned_with_limits(
            ontology,
            limits_from_parse(limits),
        ))
    }

    /// Parse UTF-8 text with strict defaults (recommended for untrusted input).
    pub fn load_text(text: &str) -> Result<Self> {
        let parse_limits = ParseLimits::default();
        let ontology = load_ontology_from_str(text)?;
        Ok(Self::from_owned_with_limits(
            ontology,
            limits_from_parse(parse_limits),
        ))
    }

    /// Parse UTF-8 text leniently (trusted corpora only).
    pub fn load_text_lenient(text: &str) -> Result<Self> {
        let parse_limits = ParseLimits::lenient();
        let ontology = load_ontology_from_str_lenient(text)?;
        Ok(Self::from_owned_with_limits(
            ontology,
            limits_from_parse(parse_limits),
        ))
    }

    pub fn to_json(&self) -> Result<String> {
        self.inner
            .lock()
            .map_err(|e| JsError::Other(format!("ontology lock poisoned: {e}")))?
            .to_json()
            .map_err(JsError::from)
    }

    pub fn to_value(&self) -> Result<Value> {
        let json = self.to_json()?;
        serde_json::from_str(&json).map_err(|e| JsError::Other(e.to_string()))
    }

    pub fn axiom_count(&self) -> Result<usize> {
        Ok(self
            .inner
            .lock()
            .map_err(|e| JsError::Other(format!("ontology lock poisoned: {e}")))?
            .axiom_count())
    }

    pub fn entity_count(&self) -> Result<usize> {
        Ok(self
            .inner
            .lock()
            .map_err(|e| JsError::Other(format!("ontology lock poisoned: {e}")))?
            .entity_count())
    }
}

/// Fluent builder for constructing ontologies in memory.
pub struct JsOntologyBuilder {
    builder: OntologyBuilder,
}

impl Default for JsOntologyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl JsOntologyBuilder {
    pub fn new() -> Self {
        Self {
            builder: OntologyBuilder::new(),
        }
    }

    fn apply<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(OntologyBuilder) -> ontologos_core::Result<OntologyBuilder>,
    {
        let next = std::mem::take(&mut self.builder);
        self.builder = f(next)?;
        Ok(())
    }

    pub fn add_class(&mut self, iri: &str) -> Result<()> {
        self.apply(|b| b.class(iri))
    }

    pub fn individual(&mut self, iri: &str) -> Result<()> {
        self.apply(|b| b.individual(iri))
    }

    pub fn object_property(&mut self, iri: &str) -> Result<()> {
        self.apply(|b| b.object_property(iri))
    }

    pub fn subclass_of(&mut self, subclass: &str, superclass: &str) -> Result<()> {
        self.apply(|b| b.subclass_of(subclass, superclass))
    }

    pub fn subproperty_of(&mut self, sub: &str, sup: &str) -> Result<()> {
        self.apply(|b| b.subproperty_of(sub, sup))
    }

    pub fn property_domain(&mut self, property: &str, domain: &str) -> Result<()> {
        self.apply(|b| b.property_domain(property, domain))
    }

    pub fn property_range(&mut self, property: &str, range: &str) -> Result<()> {
        self.apply(|b| b.property_range(property, range))
    }

    pub fn class_assertion(&mut self, individual: &str, class: &str) -> Result<()> {
        self.apply(|b| b.class_assertion(individual, class))
    }

    pub fn object_property_assertion(
        &mut self,
        subject: &str,
        property: &str,
        object: &str,
    ) -> Result<()> {
        self.apply(|b| b.object_property_assertion(subject, property, object))
    }

    pub fn build(&mut self) -> Result<JsOntology> {
        let inner = std::mem::take(&mut self.builder).build()?;
        Ok(JsOntology::from_owned(inner))
    }
}
