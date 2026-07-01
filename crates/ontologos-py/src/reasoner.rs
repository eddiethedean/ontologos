//! Python `Reasoner` bindings.

use ontologos_core::{Axiom, ConsistencyResult, Ontology, OntologyRevision, ParseMetaSummary, Profile, Reasoner, ReasonerConfig};
use ontologos_facade::ClassifyOutcome;
use ontologos_explain::explain_with_profile;
use ontologos_parser::load_ontology;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};

use crate::convert::{
    entity_iri, find_subclass_axiom_id, parse_meta_dict, parse_profile, proof_graph_dict, py_err,
    rdfs_classify_dict, resolve_class, resolve_individual, resolve_object_property,
    rl_classify_dict, taxonomy_classify_dict,
};
use crate::exceptions::map_facade_py_err;
use crate::ontology::{PyOntology, SharedOntology};

/// Python wrapper around the OntoLogos reasoner.
#[pyclass(name = "Reasoner", unsendable)]
pub(crate) struct PyReasoner {
    pub(crate) reasoner: Reasoner,
    pub(crate) last_taxonomy: Option<ontologos_core::Taxonomy>,
    /// When constructed from `Ontology`, mutations sync back to this shared handle.
    shared_ontology: Option<SharedOntology>,
    /// Last synced ontology revision (avoids full clone when unchanged).
    shared_revision: Option<OntologyRevision>,
}

fn build_reasoner(
    ontology: ontologos_core::Ontology,
    profile: Option<&str>,
    incremental: bool,
    budget_secs: Option<u64>,
) -> PyResult<Reasoner> {
    Reasoner::builder()
        .profile(parse_profile(profile)?)
        .config(ReasonerConfig {
            incremental,
            budget_secs,
            ..ReasonerConfig::default()
        })
        .build(ontology)
        .map_err(py_err)
}

#[pymethods]
impl PyReasoner {
    #[new]
    #[pyo3(signature = (path=None, ontology=None, profile=None, incremental=false, budget_secs=None))]
    fn new(
        path: Option<&str>,
        ontology: Option<&PyOntology>,
        profile: Option<&str>,
        incremental: bool,
        budget_secs: Option<u64>,
    ) -> PyResult<Self> {
        let has_path = path.is_some();
        let has_ontology = ontology.is_some();
        if has_path == has_ontology {
            return Err(py_err(
                "Reasoner requires exactly one of `path` or `ontology`",
            ));
        }

        let (core_ontology, shared_ontology, shared_revision) = if let Some(path) = path {
            (
                load_ontology(std::path::Path::new(path)).map_err(py_err)?,
                None,
                None,
            )
        } else {
            let shared = ontology.expect("ontology checked above").inner.clone();
            let guard = shared
                .lock()
                .map_err(|e| py_err(format!("ontology lock poisoned: {e}")))?;
            let revision = guard.revision();
            let core_ontology = guard.clone();
            drop(guard);
            (core_ontology, Some(shared), Some(revision))
        };

        let reasoner = build_reasoner(
            core_ontology,
            profile,
            incremental,
            budget_secs,
        )?;
        Ok(Self {
            reasoner,
            last_taxonomy: None,
            shared_ontology,
            shared_revision,
        })
    }

    /// Parse metadata from the loaded ontology (warnings and axiom counts).
    #[getter]
    fn parse_meta(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let summary = self
            .reasoner
            .ontology()
            .parse_meta()
            .map(ParseMetaSummary::from)
            .ok_or_else(|| py_err("ontology has no parse metadata"))?;
        Ok(parse_meta_dict(py, &summary)?.into())
    }

    /// Taxonomy from the last EL classification (`None` for RDFS/RL runs).
    #[getter]
    fn taxonomy(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let Some(ref taxonomy) = self.last_taxonomy else {
            return Ok(py.None());
        };
        Ok(taxonomy_classify_dict(py, self.reasoner.ontology(), taxonomy)?.into())
    }

    fn classify(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.sync_from_shared()?;
        let work = ClassifyWork {
            profile: self.reasoner.profile(),
            config: self.reasoner.config().clone(),
            ontology: self.reasoner.ontology().clone(),
        };
        let (outcome, ontology) = if self.shared_ontology.is_some() {
            run_classify_work(work)?
        } else {
            py.allow_threads(move || run_classify_work(work))?
        };
        *self.reasoner.ontology_mut() = ontology;
        let result = match outcome {
            ClassifyOutcome::Taxonomy(taxonomy) => {
                let dict = taxonomy_classify_dict(py, self.reasoner.ontology(), &taxonomy)?;
                self.last_taxonomy = Some(taxonomy);
                Ok(dict.into())
            }
            ClassifyOutcome::Rdfs(report) => {
                self.last_taxonomy = None;
                Ok(rdfs_classify_dict(py, &report)?.into())
            }
            ClassifyOutcome::Rl(report) => {
                self.last_taxonomy = None;
                Ok(rl_classify_dict(py, &report)?.into())
            }
        };
        self.sync_to_shared()?;
        result
    }

    fn explain(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.sync_from_shared()?;
        let graph = explain_with_profile(&mut self.reasoner).map_err(|e| py_err(e.to_string()))?;
        let summary = self
            .reasoner
            .ontology()
            .parse_meta()
            .map(ParseMetaSummary::from)
            .ok_or_else(|| py_err("ontology has no parse metadata"))?;
        self.sync_to_shared()?;
        Ok(proof_graph_dict(py, self.reasoner.ontology(), &graph, Some(&summary))?.into())
    }

    fn add_subclass_of(&mut self, subclass: &str, superclass: &str) -> PyResult<()> {
        self.sync_from_shared()?;
        let ontology = self.reasoner.ontology_mut();
        let sub = resolve_class(ontology, subclass)?;
        let sup = resolve_class(ontology, superclass)?;
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: sub,
                superclass: sup,
            })
            .map_err(py_err)?;
        self.sync_to_shared()?;
        Ok(())
    }

    fn remove_subclass_of(&mut self, subclass: &str, superclass: &str) -> PyResult<()> {
        self.sync_from_shared()?;
        let id = find_subclass_axiom_id(self.reasoner.ontology(), subclass, superclass)?
            .ok_or_else(|| py_err(format!("no SubClassOf axiom for {subclass} ⊑ {superclass}")))?;
        self.reasoner
            .ontology_mut()
            .remove_axiom(id)
            .map_err(py_err)?;
        self.sync_to_shared()?;
        Ok(())
    }

    fn add_axiom_json(&mut self, py: Python<'_>, axiom: &Bound<'_, PyAny>) -> PyResult<()> {
        self.sync_from_shared()?;
        let json_mod = PyModule::import(py, "json")?;
        let axiom_json: String = json_mod.call_method1("dumps", (axiom,))?.extract()?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&axiom_json).map_err(|e| py_err(e.to_string()))?;

        let ontology = self.reasoner.ontology_mut();
        apply_snapshot_axiom(ontology, &snapshot)?;
        self.sync_to_shared()?;
        Ok(())
    }

    /// Check ontology consistency; returns `{"consistent": bool, "complete": bool}`.
    fn check_consistency(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.sync_from_shared()?;
        let profile = self.reasoner.profile();
        let config = self.reasoner.config().clone();
        let ontology = self.reasoner.ontology().clone();
        let result = if self.shared_ontology.is_some() {
            run_consistency_check(profile, config, ontology)?
        } else {
            py.allow_threads(move || run_consistency_check(profile, config, ontology))?
        };
        self.sync_to_shared()?;
        let dict = PyDict::new(py);
        dict.set_item("consistent", result.consistent)?;
        dict.set_item("complete", result.complete)?;
        Ok(dict.into())
    }

    /// Check ontology consistency (OWLReasoner-style bool; raises if incomplete).
    fn is_consistent(&mut self, py: Python<'_>) -> PyResult<bool> {
        self.sync_from_shared()?;
        let profile = self.reasoner.profile();
        let config = self.reasoner.config().clone();
        let ontology = self.reasoner.ontology().clone();
        let consistent = if self.shared_ontology.is_some() {
            run_is_consistent_check(profile, config, ontology)?
        } else {
            py.allow_threads(move || run_is_consistent_check(profile, config, ontology))?
        };
        self.sync_to_shared()?;
        Ok(consistent)
    }

    /// Check entailment for `SubClassOf`, `ClassAssertion`, or `ObjectPropertyAssertion`.
    #[pyo3(signature = (sub=None, sup=None, *, individual=None, class_=None, subject=None, property=None, object=None))]
    #[allow(clippy::too_many_arguments)]
    fn is_entailed(
        &mut self,
        py: Python<'_>,
        sub: Option<&str>,
        sup: Option<&str>,
        individual: Option<&str>,
        class_: Option<&str>,
        subject: Option<&str>,
        property: Option<&str>,
        object: Option<&str>,
    ) -> PyResult<bool> {
        self.sync_from_shared()?;
        let check = ontologos_facade::parse_entailment_check(
            sub.map(str::to_owned),
            sup.map(str::to_owned),
            individual.map(str::to_owned),
            class_.map(str::to_owned),
            subject.map(str::to_owned),
            property.map(str::to_owned),
            object.map(str::to_owned),
        )
        .map_err(map_facade_py_err)?;
        let profile = self.reasoner.profile();
        let config = self.reasoner.config().clone();
        let ontology = self.reasoner.ontology().clone();
        let (entailed, ontology) = if self.shared_ontology.is_some() {
            let mut reasoner = Reasoner::builder()
                .profile(profile)
                .config(config)
                .build(ontology)
                .map_err(py_err)?;
            let entailed =
                ontologos_facade::is_entailed_axiom(&mut reasoner, check).map_err(map_facade_py_err)?;
            (entailed, reasoner.ontology().clone())
        } else {
            py.allow_threads(move || -> PyResult<(bool, Ontology)> {
                let mut reasoner = Reasoner::builder()
                    .profile(profile)
                    .config(config)
                    .build(ontology)
                    .map_err(py_err)?;
                let entailed =
                    ontologos_facade::is_entailed_axiom(&mut reasoner, check).map_err(map_facade_py_err)?;
                Ok((entailed, reasoner.ontology().clone()))
            })?
        };
        *self.reasoner.ontology_mut() = ontology;
        self.sync_to_shared()?;
        Ok(entailed)
    }

    /// Answer a conjunctive query after classification (e.g. `Type(?x, http://ex.org/A)`).
    fn query(&mut self, py: Python<'_>, query: &str) -> PyResult<Py<PyAny>> {
        self.sync_from_shared()?;
        if self.last_taxonomy.is_none() {
            self.classify(py)?;
        }
        let taxonomy = self
            .last_taxonomy
            .as_ref()
            .ok_or_else(|| py_err("query requires taxonomy classification outcome"))?;
        let cq = ontologos_ql::parse_conjunctive_query(query).map_err(py_err)?;
        let answers =
            ontologos_ql::answer_query(self.reasoner.ontology(), taxonomy, &cq).map_err(py_err)?;
        let list = pyo3::types::PyList::empty(py);
        for answer in answers {
            let dict = PyDict::new(py);
            for (var, id) in answer.bindings {
                dict.set_item(var, entity_iri(self.reasoner.ontology(), id)?)?;
            }
            list.append(dict)?;
        }
        self.sync_to_shared()?;
        Ok(list.into())
    }
}

impl PyReasoner {
    fn sync_from_shared(&mut self) -> PyResult<()> {
        if let Some(shared) = &self.shared_ontology {
            let mut guard = shared
                .lock()
                .map_err(|e| py_err(format!("ontology lock poisoned: {e}")))?;
            let current = guard.revision();
            if self.shared_revision != Some(current) {
                std::mem::swap(self.reasoner.ontology_mut(), &mut *guard);
                self.shared_revision = Some(self.reasoner.ontology().revision());
            }
        }
        Ok(())
    }

    fn sync_to_shared(&mut self) -> PyResult<()> {
        if let Some(shared) = &self.shared_ontology {
            let reasoner_rev = self.reasoner.ontology().revision();
            if self.shared_revision != Some(reasoner_rev) {
                let mut guard = shared
                    .lock()
                    .map_err(|e| py_err(format!("ontology lock poisoned: {e}")))?;
                std::mem::swap(self.reasoner.ontology_mut(), &mut *guard);
                self.shared_revision = Some(reasoner_rev);
            }
        }
        Ok(())
    }
}

fn run_consistency_check(
    profile: Profile,
    config: ReasonerConfig,
    ontology: Ontology,
) -> PyResult<ConsistencyResult> {
    let reasoner = Reasoner::builder()
        .profile(profile)
        .config(config)
        .build(ontology)
        .map_err(py_err)?;
    ontologos_facade::check_consistency(&reasoner).map_err(map_facade_py_err)
}

fn run_is_consistent_check(
    profile: Profile,
    config: ReasonerConfig,
    ontology: Ontology,
) -> PyResult<bool> {
    let reasoner = Reasoner::builder()
        .profile(profile)
        .config(config)
        .build(ontology)
        .map_err(py_err)?;
    ontologos_facade::is_consistent(&reasoner).map_err(map_facade_py_err)
}

struct ClassifyWork {
    profile: Profile,
    config: ReasonerConfig,
    ontology: Ontology,
}

fn run_classify_work(work: ClassifyWork) -> PyResult<(ClassifyOutcome, Ontology)> {
    let mut reasoner = Reasoner::builder()
        .profile(work.profile)
        .config(work.config)
        .build(work.ontology)
        .map_err(py_err)?;
    let outcome = ontologos_facade::classify(&mut reasoner).map_err(map_facade_py_err)?;
    Ok((outcome, reasoner.ontology().clone()))
}

fn apply_snapshot_axiom(
    ontology: &mut ontologos_core::Ontology,
    value: &serde_json::Value,
) -> PyResult<()> {
    if let Some(inner) = value.get("SubClassOf") {
        let subclass = inner
            .get("subclass")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("SubClassOf missing subclass"))?;
        let superclass = inner
            .get("superclass")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("SubClassOf missing superclass"))?;
        let sub = resolve_class(ontology, subclass)?;
        let sup = resolve_class(ontology, superclass)?;
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: sub,
                superclass: sup,
            })
            .map_err(py_err)?;
        return Ok(());
    }

    if let Some(inner) = value.get("SubObjectPropertyOf") {
        let sub = inner
            .get("sub_property")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("SubObjectPropertyOf missing sub_property"))?;
        let sup = inner
            .get("super_property")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("SubObjectPropertyOf missing super_property"))?;
        let sub_id = resolve_object_property(ontology, sub)?;
        let sup_id = resolve_object_property(ontology, sup)?;
        ontology
            .add_axiom(Axiom::SubObjectPropertyOf {
                sub_property: sub_id,
                super_property: sup_id,
            })
            .map_err(py_err)?;
        return Ok(());
    }

    if let Some(inner) = value.get("ObjectPropertyDomain") {
        let property = inner
            .get("property")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyDomain missing property"))?;
        let domain = inner
            .get("domain")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyDomain missing domain"))?;
        let property_id = resolve_object_property(ontology, property)?;
        let domain_id = resolve_class(ontology, domain)?;
        ontology
            .add_axiom(Axiom::ObjectPropertyDomain {
                property: property_id,
                domain: domain_id,
            })
            .map_err(py_err)?;
        return Ok(());
    }

    if let Some(inner) = value.get("ObjectPropertyRange") {
        let property = inner
            .get("property")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyRange missing property"))?;
        let range = inner
            .get("range")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyRange missing range"))?;
        let property_id = resolve_object_property(ontology, property)?;
        let range_id = resolve_class(ontology, range)?;
        ontology
            .add_axiom(Axiom::ObjectPropertyRange {
                property: property_id,
                range: range_id,
            })
            .map_err(py_err)?;
        return Ok(());
    }

    if let Some(inner) = value.get("ClassAssertion") {
        let individual = inner
            .get("individual")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ClassAssertion missing individual"))?;
        let class = inner
            .get("class")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ClassAssertion missing class"))?;
        let individual_id = resolve_individual(ontology, individual)?;
        let class_id = resolve_class(ontology, class)?;
        ontology
            .add_axiom(Axiom::ClassAssertion {
                individual: individual_id,
                class: class_id,
            })
            .map_err(py_err)?;
        return Ok(());
    }

    if let Some(inner) = value.get("ObjectPropertyAssertion") {
        let subject = inner
            .get("subject")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyAssertion missing subject"))?;
        let property = inner
            .get("property")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyAssertion missing property"))?;
        let object = inner
            .get("object")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyAssertion missing object"))?;
        let subject_id = resolve_individual(ontology, subject)?;
        let property_id = resolve_object_property(ontology, property)?;
        let object_id = resolve_individual(ontology, object)?;
        ontology
            .add_axiom(Axiom::ObjectPropertyAssertion {
                subject: subject_id,
                property: property_id,
                object: object_id,
            })
            .map_err(py_err)?;
        return Ok(());
    }

    Err(py_err(
        "unsupported axiom JSON; use format v2 axiom objects (e.g. {\"SubClassOf\": {...}})",
    ))
}

