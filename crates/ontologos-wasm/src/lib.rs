//! WebAssembly bindings for OntoLogos.

use ontologos_js::{JsOntology, JsOntologyBuilder, JsReasoner, VERSION, usize_to_u32};
use serde_json::Value;
use wasm_bindgen::prelude::*;

fn map_err(error: ontologos_js::JsError) -> JsValue {
    let err = js_sys::Error::new(&error.to_string());
    let _ = js_sys::Reflect::set(
        &err,
        &JsValue::from_str("name"),
        &JsValue::from_str(error.code()),
    );
    let _ = js_sys::Reflect::set(
        &err,
        &JsValue::from_str("code"),
        &JsValue::from_str(error.code()),
    );
    err.into()
}

fn to_js_value(value: Value) -> Result<JsValue, JsValue> {
    let json = serde_json::to_string(&value).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_sys::JSON::parse(&json).map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

/// Package version string.
#[wasm_bindgen(js_name = version)]
pub fn package_version() -> String {
    VERSION.to_owned()
}

/// In-memory ontology for browser use.
#[wasm_bindgen]
pub struct Ontology {
    inner: JsOntology,
}

#[wasm_bindgen]
impl Ontology {
    /// Parse an ontology from a JSON snapshot string.
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<Ontology, JsValue> {
        Ok(Self {
            inner: JsOntology::from_json(json).map_err(map_err)?,
        })
    }

    /// Parse an ontology from a JSON snapshot with custom resource limits.
    #[wasm_bindgen(js_name = fromJsonWithLimits)]
    pub fn from_json_with_limits(
        json: &str,
        max_json_bytes: Option<u32>,
        max_entities: Option<u32>,
        max_axioms: Option<u32>,
        max_iri_len: Option<u32>,
    ) -> Result<Ontology, JsValue> {
        Ok(Self {
            inner: JsOntology::from_json_with_limits(
                json,
                max_json_bytes.map(|n| n as usize),
                max_entities.map(|n| n as usize),
                max_axioms.map(|n| n as usize),
                max_iri_len.map(|n| n as usize),
            )
            .map_err(map_err)?,
        })
    }

    /// Parse an ontology from a plain JavaScript object (JSON snapshot v3).
    #[wasm_bindgen(js_name = fromObject)]
    pub fn from_object(value: JsValue) -> Result<Ontology, JsValue> {
        let data: Value =
            serde_wasm_bindgen::from_value(value).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self {
            inner: JsOntology::from_dict(&data).map_err(map_err)?,
        })
    }

    /// Parse an ontology object with custom resource limits.
    #[wasm_bindgen(js_name = fromObjectWithLimits)]
    pub fn from_object_with_limits(
        value: JsValue,
        max_json_bytes: Option<u32>,
        max_entities: Option<u32>,
        max_axioms: Option<u32>,
        max_iri_len: Option<u32>,
    ) -> Result<Ontology, JsValue> {
        let data: Value =
            serde_wasm_bindgen::from_value(value).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self {
            inner: JsOntology::from_dict_with_limits(
                &data,
                max_json_bytes.map(|n| n as usize),
                max_entities.map(|n| n as usize),
                max_axioms.map(|n| n as usize),
                max_iri_len.map(|n| n as usize),
            )
            .map_err(map_err)?,
        })
    }

    /// Load an ontology from OWL/RDF/Turtle/Functional Syntax bytes (strict; untrusted input).
    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<Ontology, JsValue> {
        Ok(Self {
            inner: JsOntology::load_bytes(bytes).map_err(map_err)?,
        })
    }

    /// Lenient byte load for trusted corpora only.
    #[wasm_bindgen(js_name = fromBytesLenient)]
    pub fn from_bytes_lenient(bytes: &[u8]) -> Result<Ontology, JsValue> {
        Ok(Self {
            inner: JsOntology::load_bytes_lenient(bytes).map_err(map_err)?,
        })
    }

    /// Load from UTF-8 text (strict; untrusted input).
    #[wasm_bindgen(js_name = fromText)]
    pub fn from_text(text: &str) -> Result<Ontology, JsValue> {
        Ok(Self {
            inner: JsOntology::load_text(text).map_err(map_err)?,
        })
    }

    /// Lenient text load for trusted corpora only.
    #[wasm_bindgen(js_name = fromTextLenient)]
    pub fn from_text_lenient(text: &str) -> Result<Ontology, JsValue> {
        Ok(Self {
            inner: JsOntology::load_text_lenient(text).map_err(map_err)?,
        })
    }

    /// Serialize to a JSON snapshot string.
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        self.inner.to_json().map_err(map_err)
    }

    /// Serialize to a plain JavaScript object.
    #[wasm_bindgen(js_name = toObject)]
    pub fn to_object(&self) -> Result<JsValue, JsValue> {
        to_js_value(self.inner.to_value().map_err(map_err)?)
    }

    /// Number of axioms in the ontology.
    #[wasm_bindgen(getter, js_name = axiomCount)]
    pub fn axiom_count(&self) -> Result<u32, JsValue> {
        usize_to_u32(self.inner.axiom_count().map_err(map_err)?).map_err(map_err)
    }

    /// Number of entities in the ontology.
    #[wasm_bindgen(getter, js_name = entityCount)]
    pub fn entity_count(&self) -> Result<u32, JsValue> {
        usize_to_u32(self.inner.entity_count().map_err(map_err)?).map_err(map_err)
    }
}

/// Fluent builder for constructing ontologies in the browser.
#[wasm_bindgen(js_name = OntologyBuilder)]
pub struct JsOntologyBuilderWrap {
    inner: JsOntologyBuilder,
}

impl Default for JsOntologyBuilderWrap {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = OntologyBuilder)]
impl JsOntologyBuilderWrap {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: JsOntologyBuilder::new(),
        }
    }

    #[wasm_bindgen(js_name = addClass)]
    pub fn add_class(&mut self, iri: &str) -> Result<(), JsValue> {
        self.inner.add_class(iri).map_err(map_err)
    }

    #[wasm_bindgen]
    pub fn individual(&mut self, iri: &str) -> Result<(), JsValue> {
        self.inner.individual(iri).map_err(map_err)
    }

    #[wasm_bindgen(js_name = objectProperty)]
    pub fn object_property(&mut self, iri: &str) -> Result<(), JsValue> {
        self.inner.object_property(iri).map_err(map_err)
    }

    #[wasm_bindgen(js_name = subclassOf)]
    pub fn subclass_of(&mut self, subclass: &str, superclass: &str) -> Result<(), JsValue> {
        self.inner
            .subclass_of(subclass, superclass)
            .map_err(map_err)
    }

    #[wasm_bindgen(js_name = subpropertyOf)]
    pub fn subproperty_of(&mut self, sub: &str, sup: &str) -> Result<(), JsValue> {
        self.inner.subproperty_of(sub, sup).map_err(map_err)
    }

    #[wasm_bindgen(js_name = propertyDomain)]
    pub fn property_domain(&mut self, property: &str, domain: &str) -> Result<(), JsValue> {
        self.inner
            .property_domain(property, domain)
            .map_err(map_err)
    }

    #[wasm_bindgen(js_name = propertyRange)]
    pub fn property_range(&mut self, property: &str, range: &str) -> Result<(), JsValue> {
        self.inner.property_range(property, range).map_err(map_err)
    }

    #[wasm_bindgen(js_name = classAssertion)]
    pub fn class_assertion(&mut self, individual: &str, class: &str) -> Result<(), JsValue> {
        self.inner
            .class_assertion(individual, class)
            .map_err(map_err)
    }

    #[wasm_bindgen(js_name = objectPropertyAssertion)]
    pub fn object_property_assertion(
        &mut self,
        subject: &str,
        property: &str,
        object: &str,
    ) -> Result<(), JsValue> {
        self.inner
            .object_property_assertion(subject, property, object)
            .map_err(map_err)
    }

    #[wasm_bindgen]
    pub fn build(&mut self) -> Result<Ontology, JsValue> {
        Ok(Ontology {
            inner: self.inner.build().map_err(map_err)?,
        })
    }
}

/// OWL reasoner for browser use.
#[wasm_bindgen]
pub struct Reasoner {
    inner: JsReasoner,
}

#[wasm_bindgen]
impl Reasoner {
    /// Create a reasoner from an in-memory ontology.
    #[wasm_bindgen(constructor)]
    pub fn new(
        ontology: &Ontology,
        profile: Option<String>,
        incremental: Option<bool>,
        budget_secs: Option<u32>,
    ) -> Result<Reasoner, JsValue> {
        let profile_ref = profile.as_deref();
        Ok(Self {
            inner: JsReasoner::from_ontology(
                &ontology.inner,
                profile_ref,
                incremental.unwrap_or(false),
                budget_secs.map(u64::from),
            )
            .map_err(map_err)?,
        })
    }

    /// Parser metadata (warnings and axiom counts), when available.
    #[wasm_bindgen(getter, js_name = parseMeta)]
    pub fn parse_meta(&self) -> Result<JsValue, JsValue> {
        to_js_value(self.inner.parse_meta().map_err(map_err)?)
    }

    /// Taxonomy from the last EL/DL classification (`null` for RDFS/RL runs).
    #[wasm_bindgen(getter)]
    pub fn taxonomy(&mut self) -> Result<JsValue, JsValue> {
        match self.inner.taxonomy().map_err(map_err)? {
            Some(value) => to_js_value(value),
            None => Ok(JsValue::NULL),
        }
    }

    /// Classify or materialize according to the configured profile.
    #[wasm_bindgen]
    pub fn classify(&mut self) -> Result<JsValue, JsValue> {
        to_js_value(self.inner.classify().map_err(map_err)?)
    }

    /// Build an explanation proof graph for the last classification.
    #[wasm_bindgen]
    pub fn explain(&mut self) -> Result<JsValue, JsValue> {
        to_js_value(self.inner.explain().map_err(map_err)?)
    }

    /// Check consistency; returns `{ consistent, complete }`.
    #[wasm_bindgen(js_name = checkConsistency)]
    pub fn check_consistency(&mut self) -> Result<JsValue, JsValue> {
        to_js_value(self.inner.check_consistency().map_err(map_err)?)
    }

    /// Check consistency (boolean); throws when reasoning is incomplete.
    #[wasm_bindgen(js_name = isConsistent)]
    pub fn is_consistent(&mut self) -> Result<bool, JsValue> {
        self.inner.is_consistent().map_err(map_err)
    }

    /// Check entailment for SubClassOf, ClassAssertion, or ObjectPropertyAssertion.
    #[wasm_bindgen(js_name = isEntailed)]
    pub fn is_entailed(&mut self, check: JsValue) -> Result<bool, JsValue> {
        let value: Value =
            serde_wasm_bindgen::from_value(check).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let entailed = self
            .inner
            .is_entailed(
                value.get("sub").and_then(Value::as_str),
                value.get("sup").and_then(Value::as_str),
                value.get("individual").and_then(Value::as_str),
                value.get("class").and_then(Value::as_str),
                value.get("subject").and_then(Value::as_str),
                value.get("property").and_then(Value::as_str),
                value.get("object").and_then(Value::as_str),
            )
            .map_err(map_err)?;
        Ok(entailed)
    }

    /// Answer a conjunctive query after classification.
    #[wasm_bindgen]
    pub fn query(&mut self, query: &str) -> Result<JsValue, JsValue> {
        to_js_value(self.inner.query(query).map_err(map_err)?)
    }

    /// Add a SubClassOf axiom incrementally.
    #[wasm_bindgen(js_name = addSubclassOf)]
    pub fn add_subclass_of(&mut self, subclass: &str, superclass: &str) -> Result<(), JsValue> {
        self.inner
            .add_subclass_of(subclass, superclass)
            .map_err(map_err)
    }

    /// Remove a matching asserted SubClassOf axiom.
    #[wasm_bindgen(js_name = removeSubclassOf)]
    pub fn remove_subclass_of(&mut self, subclass: &str, superclass: &str) -> Result<(), JsValue> {
        self.inner
            .remove_subclass_of(subclass, superclass)
            .map_err(map_err)
    }

    /// Add an axiom from a JSON snapshot v2 axiom object.
    #[wasm_bindgen(js_name = addAxiomJson)]
    pub fn add_axiom_json(&mut self, axiom: JsValue) -> Result<(), JsValue> {
        let value: Value =
            serde_wasm_bindgen::from_value(axiom).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner.add_axiom_json(&value).map_err(map_err)
    }
}
