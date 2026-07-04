//! Node.js native bindings for OntoLogos.

#![deny(clippy::all)]

mod errors;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use ontologos_js::{JsOntology, JsOntologyBuilder, JsReasoner, VERSION, usize_to_u32};
use serde_json::Value;

use crate::errors::{map_err, u32_to_usize};

fn to_json_value(value: Value) -> Result<serde_json::Value> {
    Ok(value)
}

/// Package version string.
#[napi]
pub fn version() -> String {
    VERSION.to_owned()
}

/// In-memory ontology.
#[napi]
pub struct Ontology {
    inner: JsOntology,
}

#[napi]
impl Ontology {
    #[napi(factory, js_name = "fromJson")]
    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            inner: JsOntology::from_json(&json).map_err(map_err)?,
        })
    }

    #[napi(factory, js_name = "fromJsonWithLimits")]
    pub fn from_json_with_limits(
        json: String,
        max_json_bytes: Option<u32>,
        max_entities: Option<u32>,
        max_axioms: Option<u32>,
        max_iri_len: Option<u32>,
    ) -> Result<Self> {
        Ok(Self {
            inner: JsOntology::from_json_with_limits(
                &json,
                max_json_bytes.map(u32_to_usize),
                max_entities.map(u32_to_usize),
                max_axioms.map(u32_to_usize),
                max_iri_len.map(u32_to_usize),
            )
            .map_err(map_err)?,
        })
    }

    #[napi(factory, js_name = "fromObject")]
    pub fn from_object(value: serde_json::Value) -> Result<Self> {
        Ok(Self {
            inner: JsOntology::from_dict(&value).map_err(map_err)?,
        })
    }

    #[napi(factory, js_name = "fromObjectWithLimits")]
    pub fn from_object_with_limits(
        value: serde_json::Value,
        max_json_bytes: Option<u32>,
        max_entities: Option<u32>,
        max_axioms: Option<u32>,
        max_iri_len: Option<u32>,
    ) -> Result<Self> {
        Ok(Self {
            inner: JsOntology::from_dict_with_limits(
                &value,
                max_json_bytes.map(u32_to_usize),
                max_entities.map(u32_to_usize),
                max_axioms.map(u32_to_usize),
                max_iri_len.map(u32_to_usize),
            )
            .map_err(map_err)?,
        })
    }

    #[napi(factory, js_name = "fromBytes")]
    pub fn from_bytes(bytes: Buffer) -> Result<Self> {
        Ok(Self {
            inner: JsOntology::load_bytes(bytes.as_ref()).map_err(map_err)?,
        })
    }

    #[napi(factory, js_name = "fromBytesLenient")]
    pub fn from_bytes_lenient(bytes: Buffer) -> Result<Self> {
        Ok(Self {
            inner: JsOntology::load_bytes_lenient(bytes.as_ref()).map_err(map_err)?,
        })
    }

    #[napi(factory, js_name = "fromText")]
    pub fn from_text(text: String) -> Result<Self> {
        Ok(Self {
            inner: JsOntology::load_text(&text).map_err(map_err)?,
        })
    }

    #[napi(factory, js_name = "fromTextLenient")]
    pub fn from_text_lenient(text: String) -> Result<Self> {
        Ok(Self {
            inner: JsOntology::load_text_lenient(&text).map_err(map_err)?,
        })
    }

    /// Load from a trusted local path (lenient parse; no sandbox).
    #[napi(factory)]
    pub fn load(path: String) -> Result<Self> {
        Ok(Self {
            inner: JsOntology::load_path(&path).map_err(map_err)?,
        })
    }

    /// Sandboxed load constrained to `base` (strict; recommended for uploads).
    #[napi(factory, js_name = "loadIn")]
    pub fn load_in(base: String, path: String) -> Result<Self> {
        Ok(Self {
            inner: JsOntology::load_in(&base, &path).map_err(map_err)?,
        })
    }

    #[napi(js_name = "toJson")]
    pub fn to_json(&self) -> Result<String> {
        self.inner.to_json().map_err(map_err)
    }

    #[napi(js_name = "toObject")]
    pub fn to_object(&self) -> Result<serde_json::Value> {
        self.inner.to_value().map_err(map_err)
    }

    #[napi(getter, js_name = "axiomCount")]
    pub fn axiom_count(&self) -> Result<u32> {
        usize_to_u32(self.inner.axiom_count().map_err(map_err)?).map_err(map_err)
    }

    #[napi(getter, js_name = "entityCount")]
    pub fn entity_count(&self) -> Result<u32> {
        usize_to_u32(self.inner.entity_count().map_err(map_err)?).map_err(map_err)
    }
}

/// Fluent builder for constructing ontologies.
#[napi(js_name = "OntologyBuilder")]
pub struct JsOntologyBuilderWrap {
    inner: JsOntologyBuilder,
}

impl Default for JsOntologyBuilderWrap {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl JsOntologyBuilderWrap {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: JsOntologyBuilder::new(),
        }
    }

    #[napi(js_name = "addClass")]
    pub fn add_class(&mut self, iri: String) -> Result<()> {
        self.inner.add_class(&iri).map_err(map_err)
    }

    #[napi]
    pub fn individual(&mut self, iri: String) -> Result<()> {
        self.inner.individual(&iri).map_err(map_err)
    }

    #[napi(js_name = "objectProperty")]
    pub fn object_property(&mut self, iri: String) -> Result<()> {
        self.inner.object_property(&iri).map_err(map_err)
    }

    #[napi(js_name = "subclassOf")]
    pub fn subclass_of(&mut self, subclass: String, superclass: String) -> Result<()> {
        self.inner
            .subclass_of(&subclass, &superclass)
            .map_err(map_err)
    }

    #[napi(js_name = "subpropertyOf")]
    pub fn subproperty_of(&mut self, sub: String, sup: String) -> Result<()> {
        self.inner.subproperty_of(&sub, &sup).map_err(map_err)
    }

    #[napi(js_name = "propertyDomain")]
    pub fn property_domain(&mut self, property: String, domain: String) -> Result<()> {
        self.inner
            .property_domain(&property, &domain)
            .map_err(map_err)
    }

    #[napi(js_name = "propertyRange")]
    pub fn property_range(&mut self, property: String, range: String) -> Result<()> {
        self.inner.property_range(&property, &range).map_err(map_err)
    }

    #[napi(js_name = "classAssertion")]
    pub fn class_assertion(&mut self, individual: String, class: String) -> Result<()> {
        self.inner
            .class_assertion(&individual, &class)
            .map_err(map_err)
    }

    #[napi(js_name = "objectPropertyAssertion")]
    pub fn object_property_assertion(
        &mut self,
        subject: String,
        property: String,
        object: String,
    ) -> Result<()> {
        self.inner
            .object_property_assertion(&subject, &property, &object)
            .map_err(map_err)
    }

    #[napi]
    pub fn build(&mut self) -> Result<Ontology> {
        Ok(Ontology {
            inner: self.inner.build().map_err(map_err)?,
        })
    }
}

/// Entailment check input (exactly one axiom shape).
#[napi(object)]
pub struct EntailmentCheck {
    pub sub: Option<String>,
    pub sup: Option<String>,
    pub individual: Option<String>,
    #[napi(js_name = "class")]
    pub class_name: Option<String>,
    pub subject: Option<String>,
    pub property: Option<String>,
    pub object: Option<String>,
}

/// OWL reasoner.
#[napi]
pub struct Reasoner {
    inner: JsReasoner,
}

#[napi]
impl Reasoner {
    #[napi(constructor)]
    pub fn new(
        ontology: &Ontology,
        profile: Option<String>,
        incremental: Option<bool>,
        budget_secs: Option<u32>,
    ) -> Result<Self> {
        Ok(Self {
            inner: JsReasoner::from_ontology(
                &ontology.inner,
                profile.as_deref(),
                incremental.unwrap_or(false),
                budget_secs.map(u64::from),
            )
            .map_err(map_err)?,
        })
    }

    #[napi(factory, js_name = "fromPath")]
    pub fn from_path(
        path: String,
        profile: Option<String>,
        incremental: Option<bool>,
        budget_secs: Option<u32>,
    ) -> Result<Self> {
        Ok(Self {
            inner: JsReasoner::from_path(
                &path,
                profile.as_deref(),
                incremental.unwrap_or(false),
                budget_secs.map(u64::from),
            )
            .map_err(map_err)?,
        })
    }

    #[napi(factory, js_name = "loadIn")]
    pub fn load_in(
        base: String,
        path: String,
        profile: Option<String>,
        incremental: Option<bool>,
        budget_secs: Option<u32>,
    ) -> Result<Self> {
        Ok(Self {
            inner: JsReasoner::load_in(
                &base,
                &path,
                profile.as_deref(),
                incremental.unwrap_or(false),
                budget_secs.map(u64::from),
            )
            .map_err(map_err)?,
        })
    }

    #[napi(getter, js_name = "parseMeta")]
    pub fn parse_meta(&self) -> Result<serde_json::Value> {
        to_json_value(self.inner.parse_meta().map_err(map_err)?)
    }

    #[napi(getter)]
    pub fn taxonomy(&self) -> Result<Option<serde_json::Value>> {
        match self.inner.taxonomy().map_err(map_err)? {
            Some(value) => Ok(Some(to_json_value(value)?)),
            None => Ok(None),
        }
    }

    #[napi]
    pub fn classify(&mut self) -> Result<serde_json::Value> {
        to_json_value(self.inner.classify().map_err(map_err)?)
    }

    #[napi]
    pub fn explain(&mut self) -> Result<serde_json::Value> {
        to_json_value(self.inner.explain().map_err(map_err)?)
    }

    #[napi(js_name = "checkConsistency")]
    pub fn check_consistency(&mut self) -> Result<serde_json::Value> {
        to_json_value(self.inner.check_consistency().map_err(map_err)?)
    }

    #[napi(js_name = "isConsistent")]
    pub fn is_consistent(&mut self) -> Result<bool> {
        self.inner.is_consistent().map_err(map_err)
    }

    #[napi(js_name = "isEntailed")]
    pub fn is_entailed(&mut self, check: EntailmentCheck) -> Result<bool> {
        self.inner
            .is_entailed(
                check.sub.as_deref(),
                check.sup.as_deref(),
                check.individual.as_deref(),
                check.class_name.as_deref(),
                check.subject.as_deref(),
                check.property.as_deref(),
                check.object.as_deref(),
            )
            .map_err(map_err)
    }

    #[napi]
    pub fn query(&mut self, query: String) -> Result<Vec<serde_json::Value>> {
        let value = self.inner.query(&query).map_err(map_err)?;
        match value {
            Value::Array(items) => Ok(items),
            other => Err(Error::new(
                Status::GenericFailure,
                format!("unexpected query result shape: {other}"),
            )),
        }
    }

    #[napi(js_name = "addSubclassOf")]
    pub fn add_subclass_of(&mut self, subclass: String, superclass: String) -> Result<()> {
        self.inner
            .add_subclass_of(&subclass, &superclass)
            .map_err(map_err)
    }

    #[napi(js_name = "removeSubclassOf")]
    pub fn remove_subclass_of(&mut self, subclass: String, superclass: String) -> Result<()> {
        self.inner
            .remove_subclass_of(&subclass, &superclass)
            .map_err(map_err)
    }

    #[napi(js_name = "addAxiomJson")]
    pub fn add_axiom_json(&mut self, axiom: serde_json::Value) -> Result<()> {
        self.inner.add_axiom_json(&axiom).map_err(map_err)
    }
}

pub use errors::error_code_from_message;
