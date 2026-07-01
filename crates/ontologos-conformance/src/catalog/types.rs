//! HermiT / WG catalog case types.

use serde::Deserialize;

/// HermiT test case from `benchmarks/data/hermit/catalog/cases.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct HermitCase {
    pub id: String,
    pub java_class: String,
    pub java_method: String,
    pub java_file: String,
    pub engine: String,
    pub status: String,
    pub tier: String,
    pub ignore_reason: Option<String>,
    pub fixture: Option<String>,
    pub golden: Option<String>,
    pub axiom_ofn: Option<String>,
    #[serde(default)]
    pub subsumptions: Vec<SubsumptionExpectation>,
    #[serde(default)]
    pub property_subsumptions: Vec<SubsumptionExpectation>,
    #[serde(default)]
    pub property_characteristics: Vec<PropertyCharacteristicExpectation>,
    #[serde(default)]
    pub consistent: Option<bool>,
    #[serde(default)]
    pub class_satisfiability: Vec<ClassSatisfiabilityExpectation>,
    pub conclusion_ofn: Option<String>,
    pub expected_entailment: Option<bool>,
    pub incremental_ofn: Option<String>,
    #[serde(default)]
    pub individual_types: Vec<IndividualTypeExpectation>,
    #[serde(default)]
    pub individual_instances: Vec<IndividualInstancesExpectation>,
    #[serde(default)]
    pub data_property_subsumptions: Vec<SubsumptionExpectation>,
    #[serde(default)]
    pub datalog_queries: Vec<DatalogQueryExpectation>,
    #[serde(default)]
    pub load_error_expected: bool,
    #[serde(default)]
    pub ce_instance_checks: Vec<CeInstanceCheck>,
    #[serde(default)]
    pub ce_satisfiability: Vec<CeSatisfiabilityCheck>,
    #[serde(default)]
    pub ria_regular: Option<RiaRegularExpectation>,
    #[serde(default)]
    pub role_simple: Option<RoleSimpleExpectation>,
    pub rust_test: Option<String>,
    #[serde(default)]
    pub hand_written: bool,
}

/// OWL WG parameterized test case.
#[derive(Debug, Clone, Deserialize)]
pub struct WgCase {
    pub id: String,
    pub test_type: String,
    pub status: String,
    pub engine: String,
    pub premise_ofn: Option<String>,
    pub conclusion_ofn: Option<String>,
    pub expected_entailment: Option<bool>,
    pub expected_consistent: Option<bool>,
    pub ignore_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubsumptionExpectation {
    pub sub: String,
    pub sup: String,
    pub expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PropertyCharacteristicExpectation {
    pub property: String,
    pub kind: String,
    pub expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClassSatisfiabilityExpectation {
    pub class: String,
    pub expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndividualTypeExpectation {
    pub individual: String,
    pub class: String,
    pub expected: bool,
    #[serde(default)]
    pub direct: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CeSatisfiabilityCheck {
    pub ce_ofn: String,
    pub expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RiaRegularExpectation {
    pub axioms: String,
    pub expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleSimpleExpectation {
    pub axioms: String,
    pub expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CeInstanceCheck {
    pub individual: String,
    pub ce_ofn: String,
    pub expected: bool,
    #[serde(default)]
    pub direct: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndividualInstancesExpectation {
    pub class: String,
    #[serde(default)]
    pub expected_individuals: Vec<String>,
    #[serde(default)]
    pub direct: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatalogAtomExpectation {
    pub kind: String,
    #[serde(default)]
    pub class: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub variable: Option<String>,
    #[serde(default)]
    pub variable2: Option<String>,
    #[serde(default)]
    pub individual: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatalogQueryExpectation {
    pub atoms: Vec<DatalogAtomExpectation>,
    #[serde(default)]
    pub answers: Vec<String>,
}
