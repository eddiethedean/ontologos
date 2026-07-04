//! Reasoner handle for JavaScript bindings.

use std::path::Path;

use ontologos_core::{
    Axiom, ConsistencyResult, Ontology, OntologyRevision, ParseMetaSummary, Reasoner, ReasonerConfig,
};
use ontologos_explain::explain_with_profile;
use ontologos_facade::ClassifyOutcome;
use ontologos_parser::load_ontology_lenient;
use serde_json::{Value, json};

use crate::convert::{
    entity_iri, find_subclass_axiom_id, parse_meta_value, parse_profile, proof_graph_value,
    rdfs_classify_value, resolve_class, resolve_individual, resolve_object_property,
    rl_classify_value, taxonomy_classify_value,
};
use crate::error::{JsError, Result};
use crate::ontology::{JsOntology, SharedOntology};

/// OWL reasoner for JavaScript bindings.
pub struct JsReasoner {
    reasoner: Reasoner,
    last_taxonomy: Option<ontologos_core::Taxonomy>,
    shared_ontology: Option<SharedOntology>,
    shared_revision: Option<OntologyRevision>,
}

impl JsReasoner {
    pub fn from_path(
        path: &str,
        profile: Option<&str>,
        incremental: bool,
        budget_secs: Option<u64>,
    ) -> Result<Self> {
        let ontology = load_ontology_lenient(Path::new(path))?;
        Self::from_ontology_owned(ontology, profile, incremental, budget_secs, None)
    }

    pub fn load_in(
        base: &str,
        path: &str,
        profile: Option<&str>,
        incremental: bool,
        budget_secs: Option<u64>,
    ) -> Result<Self> {
        let ontology =
            ontologos_parser::load_ontology_in(Path::new(base), Path::new(path))?;
        Self::from_ontology_owned(ontology, profile, incremental, budget_secs, None)
    }

    pub fn from_ontology(
        ontology: &JsOntology,
        profile: Option<&str>,
        incremental: bool,
        budget_secs: Option<u64>,
    ) -> Result<Self> {
        let shared = ontology.inner.clone();
        let (core_ontology, revision) = {
            let guard = shared.borrow();
            (guard.clone(), guard.revision())
        };
        Self::from_ontology_owned(
            core_ontology,
            profile,
            incremental,
            budget_secs,
            Some((shared, revision)),
        )
    }

    fn from_ontology_owned(
        ontology: Ontology,
        profile: Option<&str>,
        incremental: bool,
        budget_secs: Option<u64>,
        shared: Option<(SharedOntology, OntologyRevision)>,
    ) -> Result<Self> {
        let reasoner = Reasoner::builder()
            .profile(parse_profile(profile)?)
            .config(ReasonerConfig {
                incremental,
                budget_secs,
                ..ReasonerConfig::default()
            })
            .build(ontology)?;
        let (shared_ontology, shared_revision) = match shared {
            Some((s, r)) => (Some(s), Some(r)),
            None => (None, None),
        };
        Ok(Self {
            reasoner,
            last_taxonomy: None,
            shared_ontology,
            shared_revision,
        })
    }

    pub fn parse_meta(&self) -> Result<Value> {
        let summary = self
            .reasoner
            .ontology()
            .parse_meta()
            .map(ParseMetaSummary::from)
            .ok_or_else(|| JsError::Other("ontology has no parse metadata".into()))?;
        Ok(parse_meta_value(&summary))
    }

    pub fn taxonomy(&self) -> Result<Option<Value>> {
        let Some(ref taxonomy) = self.last_taxonomy else {
            return Ok(None);
        };
        Ok(Some(taxonomy_classify_value(
            self.reasoner.ontology(),
            taxonomy,
        )?))
    }

    pub fn classify(&mut self) -> Result<Value> {
        self.sync_from_shared()?;
        let outcome = ontologos_facade::classify(&mut self.reasoner)?;

        let result = match outcome {
            ClassifyOutcome::Taxonomy(taxonomy) => {
                let value = taxonomy_classify_value(self.reasoner.ontology(), &taxonomy)?;
                self.last_taxonomy = Some(taxonomy);
                value
            }
            ClassifyOutcome::Rdfs(report) => {
                self.last_taxonomy = None;
                rdfs_classify_value(self.reasoner.ontology(), &report)?
            }
            ClassifyOutcome::Rl(report) => {
                self.last_taxonomy = None;
                rl_classify_value(self.reasoner.ontology(), &report)?
            }
            _ => {
                return Err(JsError::Other(
                    "unsupported ClassifyOutcome variant".into(),
                ));
            }
        };
        self.sync_to_shared()?;
        Ok(result)
    }

    pub fn explain(&mut self) -> Result<Value> {
        self.sync_from_shared()?;
        let graph = explain_with_profile(&mut self.reasoner)
            .map_err(|e| JsError::Other(e.to_string()))?;
        let parse_meta = self
            .reasoner
            .ontology()
            .parse_meta()
            .map(ParseMetaSummary::from);
        let value = proof_graph_value(
            self.reasoner.ontology(),
            &graph,
            parse_meta.as_ref(),
        )?;
        self.sync_to_shared()?;
        Ok(value)
    }

    pub fn check_consistency(&mut self) -> Result<Value> {
        self.sync_from_shared()?;
        let result = ontologos_facade::check_consistency(&self.reasoner)?;
        self.sync_to_shared()?;
        Ok(consistency_value(&result))
    }

    pub fn is_consistent(&mut self) -> Result<bool> {
        self.sync_from_shared()?;
        let consistent = ontologos_facade::is_consistent(&self.reasoner)?;
        self.sync_to_shared()?;
        Ok(consistent)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn is_entailed(
        &mut self,
        sub: Option<&str>,
        sup: Option<&str>,
        individual: Option<&str>,
        class: Option<&str>,
        subject: Option<&str>,
        property: Option<&str>,
        object: Option<&str>,
    ) -> Result<bool> {
        self.sync_from_shared()?;
        let check = ontologos_facade::parse_entailment_check(
            sub.map(str::to_owned),
            sup.map(str::to_owned),
            individual.map(str::to_owned),
            class.map(str::to_owned),
            subject.map(str::to_owned),
            property.map(str::to_owned),
            object.map(str::to_owned),
        )?;
        let entailed = ontologos_facade::is_entailed_axiom(&mut self.reasoner, check)?;
        self.invalidate_taxonomy_cache();
        self.sync_to_shared()?;
        Ok(entailed)
    }

    pub fn query(&mut self, query: &str) -> Result<Value> {
        self.sync_from_shared()?;
        if self.last_taxonomy.is_none() {
            self.classify()?;
        }
        let taxonomy = self
            .last_taxonomy
            .as_ref()
            .ok_or_else(|| JsError::Other("query requires taxonomy classification outcome".into()))?;
        let cq = ontologos_ql::parse_conjunctive_query(query).map_err(|e| JsError::Other(e.to_string()))?;
        let answers =
            ontologos_ql::answer_query(self.reasoner.ontology(), taxonomy, &cq).map_err(|e| JsError::Other(e.to_string()))?;
        let bindings: Vec<Value> = answers
            .into_iter()
            .map(|answer| {
                let mut map = serde_json::Map::new();
                for (var, id) in answer.bindings {
                    map.insert(var, Value::String(entity_iri(self.reasoner.ontology(), id)?));
                }
                Ok(Value::Object(map))
            })
            .collect::<Result<Vec<_>>>()?;
        self.sync_to_shared()?;
        Ok(Value::Array(bindings))
    }

    pub fn add_subclass_of(&mut self, subclass: &str, superclass: &str) -> Result<()> {
        self.sync_from_shared()?;
        let ontology = self.reasoner.ontology_mut();
        let sub = resolve_class(ontology, subclass)?;
        let sup = resolve_class(ontology, superclass)?;
        ontology.add_axiom(Axiom::SubClassOf {
            subclass: sub,
            superclass: sup,
        })?;
        self.invalidate_taxonomy_cache();
        self.sync_to_shared()?;
        Ok(())
    }

    pub fn remove_subclass_of(&mut self, subclass: &str, superclass: &str) -> Result<()> {
        self.sync_from_shared()?;
        let id = find_subclass_axiom_id(self.reasoner.ontology(), subclass, superclass)?
            .ok_or_else(|| {
                JsError::Other(format!("no SubClassOf axiom for {subclass} ⊑ {superclass}"))
            })?;
        self.reasoner.ontology_mut().remove_axiom(id)?;
        self.invalidate_taxonomy_cache();
        self.sync_to_shared()?;
        Ok(())
    }

    pub fn add_axiom_json(&mut self, axiom: &Value) -> Result<()> {
        self.sync_from_shared()?;
        let limits = ontologos_core::Limits::default();
        let axiom_json = serde_json::to_string(axiom).map_err(|e| JsError::Other(e.to_string()))?;
        if axiom_json.len() > limits.max_json_bytes {
            return Err(JsError::ResourceLimit(format!(
                "axiom JSON exceeds maximum size of {} bytes",
                limits.max_json_bytes
            )));
        }
        let ontology = self.reasoner.ontology_mut();
        apply_snapshot_axiom(ontology, axiom)?;
        self.invalidate_taxonomy_cache();
        self.sync_to_shared()?;
        Ok(())
    }

    fn invalidate_taxonomy_cache(&mut self) {
        self.last_taxonomy = None;
    }

    fn sync_from_shared(&mut self) -> Result<()> {
        if let Some(shared) = &self.shared_ontology {
            let guard = shared.borrow();
            let current = guard.revision();
            let revision_changed = self.shared_revision != Some(current);
            if revision_changed {
                *self.reasoner.ontology_mut() = guard.clone();
                self.shared_revision = Some(current);
            }
            drop(guard);
            if revision_changed {
                self.invalidate_taxonomy_cache();
            }
        }
        Ok(())
    }

    fn sync_to_shared(&mut self) -> Result<()> {
        if let Some(shared) = &self.shared_ontology {
            let reasoner_rev = self.reasoner.ontology().revision();
            if self.shared_revision != Some(reasoner_rev) {
                *shared.borrow_mut() = self.reasoner.ontology().clone();
                self.shared_revision = Some(reasoner_rev);
            }
        }
        Ok(())
    }
}

fn consistency_value(result: &ConsistencyResult) -> Value {
    json!({
        "consistent": result.consistent,
        "complete": result.complete,
    })
}

fn apply_snapshot_axiom(ontology: &mut Ontology, value: &Value) -> Result<()> {
    if let Some(inner) = value.get("SubClassOf") {
        let subclass = inner
            .get("subclass")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("SubClassOf missing subclass".into()))?;
        let superclass = inner
            .get("superclass")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("SubClassOf missing superclass".into()))?;
        let sub = resolve_class(ontology, subclass)?;
        let sup = resolve_class(ontology, superclass)?;
        ontology.add_axiom(Axiom::SubClassOf {
            subclass: sub,
            superclass: sup,
        })?;
        return Ok(());
    }

    if let Some(inner) = value.get("SubObjectPropertyOf") {
        let sub = inner
            .get("sub_property")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("SubObjectPropertyOf missing sub_property".into()))?;
        let sup = inner
            .get("super_property")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("SubObjectPropertyOf missing super_property".into()))?;
        let sub_id = resolve_object_property(ontology, sub)?;
        let sup_id = resolve_object_property(ontology, sup)?;
        ontology.add_axiom(Axiom::SubObjectPropertyOf {
            sub_property: sub_id,
            super_property: sup_id,
        })?;
        return Ok(());
    }

    if let Some(inner) = value.get("ObjectPropertyDomain") {
        let property = inner
            .get("property")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("ObjectPropertyDomain missing property".into()))?;
        let domain = inner
            .get("domain")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("ObjectPropertyDomain missing domain".into()))?;
        let property_id = resolve_object_property(ontology, property)?;
        let domain_id = resolve_class(ontology, domain)?;
        ontology.add_axiom(Axiom::ObjectPropertyDomain {
            property: property_id,
            domain: domain_id,
        })?;
        return Ok(());
    }

    if let Some(inner) = value.get("ObjectPropertyRange") {
        let property = inner
            .get("property")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("ObjectPropertyRange missing property".into()))?;
        let range = inner
            .get("range")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("ObjectPropertyRange missing range".into()))?;
        let property_id = resolve_object_property(ontology, property)?;
        let range_id = resolve_class(ontology, range)?;
        ontology.add_axiom(Axiom::ObjectPropertyRange {
            property: property_id,
            range: range_id,
        })?;
        return Ok(());
    }

    if let Some(inner) = value.get("ClassAssertion") {
        let individual = inner
            .get("individual")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("ClassAssertion missing individual".into()))?;
        let class = inner
            .get("class")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("ClassAssertion missing class".into()))?;
        let individual_id = resolve_individual(ontology, individual)?;
        let class_id = resolve_class(ontology, class)?;
        ontology.add_axiom(Axiom::ClassAssertion {
            individual: individual_id,
            class: class_id,
        })?;
        return Ok(());
    }

    if let Some(inner) = value.get("ObjectPropertyAssertion") {
        let subject = inner
            .get("subject")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("ObjectPropertyAssertion missing subject".into()))?;
        let property = inner
            .get("property")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("ObjectPropertyAssertion missing property".into()))?;
        let object = inner
            .get("object")
            .and_then(Value::as_str)
            .ok_or_else(|| JsError::Other("ObjectPropertyAssertion missing object".into()))?;
        let subject_id = resolve_individual(ontology, subject)?;
        let property_id = resolve_object_property(ontology, property)?;
        let object_id = resolve_individual(ontology, object)?;
        ontology.add_axiom(Axiom::ObjectPropertyAssertion {
            subject: subject_id,
            property: property_id,
            object: object_id,
        })?;
        return Ok(());
    }

    Err(JsError::Other(
        "unsupported axiom JSON; use format v2 axiom objects (e.g. {\"SubClassOf\": {...}})".into(),
    ))
}
