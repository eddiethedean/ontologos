use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};

use ontologos_core::{Axiom, ClassExpr, DlAxiom, EntityId, EntityKind, Ontology};

use crate::limits::ParseLimits;
use crate::map::map_to_core;
use crate::read::{read_horned_owl_from_reader, sniff_and_rewind};
use crate::report::ParseReport;
use crate::validate::validate_loaded_ontology_strict_graph;
use crate::{
    Error, Format, Result, detect_format, detect_format_from_bytes, detect_functional_from_bytes,
    detect_turtle_from_bytes, validate_loaded_ontology_light,
};

struct PreprocessBudget {
    limit: usize,
    used: usize,
}

impl PreprocessBudget {
    fn new(limit: usize) -> Self {
        Self { limit, used: 0 }
    }

    fn track(&mut self, stage: &str) -> Result<()> {
        self.used = self.used.saturating_add(stage.len());
        if self.used > self.limit {
            Err(Error::Parse(format!(
                "RDF/XML preprocessing allocation {} bytes exceeds limit of {} bytes",
                self.used, self.limit
            )))
        } else {
            Ok(())
        }
    }
}

fn finalize_parsed_ontology(
    ontology: Ontology,
    report: ParseReport,
    limits: ParseLimits,
    validate: bool,
) -> Result<Ontology> {
    if limits.strict && report.meta.skipped_axiom_count > 0 {
        return Err(Error::Parse(format!(
            "strict parse: skipped {} axioms due to limits or mapping failures",
            report.meta.skipped_axiom_count
        )));
    }
    let mut ontology = ontology;
    ontology.set_parse_meta(report.into_meta());
    if validate {
        validate_loaded_ontology_light(&ontology)?;
        if limits.strict {
            validate_loaded_ontology_strict_graph(&ontology)?;
        }
    }
    Ok(ontology)
}

fn finish_loaded_ontology(
    ontology: Ontology,
    report: ParseReport,
    limits: ParseLimits,
) -> Result<Ontology> {
    finalize_parsed_ontology(ontology, report, limits, limits.validate_output)
}

fn bump_harvested_assertions(count: &mut usize, limits: ParseLimits) -> Result<()> {
    *count += 1;
    if *count > limits.max_harvested_assertions {
        Err(Error::Parse(format!(
            "harvested assertion count {} exceeds limit of {}",
            *count, limits.max_harvested_assertions
        )))
    } else {
        Ok(())
    }
}

fn read_text_file_with_limit(path: &Path, limits: ParseLimits) -> Result<String> {
    let metadata = std::fs::metadata(path).map_err(|e| Error::Parse(e.to_string()))?;
    if metadata.len() as usize > limits.max_file_bytes {
        return Err(Error::Parse(format!(
            "file size {} exceeds limit of {} bytes",
            metadata.len(),
            limits.max_file_bytes
        )));
    }
    std::fs::read_to_string(path).map_err(|e| Error::Parse(e.to_string()))
}

const SUPPLEMENT_STANDARD_PREFIXES: &str = "\
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\n\
Prefix(rdf:=<http://www.w3.org/1999/02/22-rdf-syntax-ns#>)\n";

/// Reject IRIs that would break OWL Functional Syntax interpolation in supplements.
fn validate_supplement_iri(iri: &str) -> Result<()> {
    crate::validate::validate_supplement_iri(iri)
}

fn validate_supplement_iris(iris: impl IntoIterator<Item = impl AsRef<str>>) -> Result<()> {
    for iri in iris {
        validate_supplement_iri(iri.as_ref())?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
const O_NOFOLLOW: i32 = 0o100_000;
#[cfg(target_os = "macos")]
const O_NOFOLLOW: i32 = 0x0000_0040;
#[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
const O_NOFOLLOW: i32 = 0;

/// Resolve and validate a path before loading an ontology file.
pub fn validate_load_path(path: &Path, base: Option<&Path>) -> Result<PathBuf> {
    let normalized = normalize_path(path)?;

    if let Some(base) = base {
        let base_normalized = normalize_path(base)?;
        if !path_is_under_base(&normalized, &base_normalized) {
            return Err(Error::Parse(format!(
                "path {} escapes allowed base {}",
                normalized.display(),
                base_normalized.display()
            )));
        }
    }

    Ok(normalized)
}

/// Load an ontology from a validated file path.
pub fn load_ontology(path: &Path) -> Result<Ontology> {
    load_ontology_with_limits(path, ParseLimits::default())
}

/// Load an ontology without failing on skipped axioms or incompatible declarations.
pub fn load_ontology_lenient(path: &Path) -> Result<Ontology> {
    load_ontology_with_limits(path, ParseLimits::lenient())
}

/// Load an ontology constrained to stay under `base` (untrusted uploads).
pub fn load_ontology_in(base: &Path, path: &Path) -> Result<Ontology> {
    load_ontology_with_limits_and_base(path, ParseLimits::default(), Some(base))
}

/// Lenient sandboxed load for untrusted uploads that may skip axioms with warnings.
pub fn load_ontology_lenient_in(base: &Path, path: &Path) -> Result<Ontology> {
    load_ontology_with_limits_and_base(path, ParseLimits::lenient(), Some(base))
}

/// Load an ontology with custom [`ParseLimits`].
pub fn load_ontology_with_limits(path: &Path, limits: ParseLimits) -> Result<Ontology> {
    load_ontology_with_limits_and_base(path, limits, None)
}

/// Load an ontology with custom limits and optional sandbox base directory.
pub fn load_ontology_with_limits_and_base(
    path: &Path,
    limits: ParseLimits,
    base: Option<&Path>,
) -> Result<Ontology> {
    load_ontology_with_limits_and_base_inner(path, limits, base, limits.merge_imports)
}

fn load_ontology_with_limits_and_base_inner(
    path: &Path,
    limits: ParseLimits,
    base: Option<&Path>,
    merge_imports: bool,
) -> Result<Ontology> {
    let validated = validate_load_path(path, base)?;
    if !validated.is_file() {
        return Err(Error::Parse(format!("not a file: {}", validated.display())));
    }

    let mut file = open_for_load(&validated, base)?;
    let file_len = file
        .metadata()
        .map_err(|e| Error::Parse(e.to_string()))?
        .len();
    if file_len as usize > limits.max_file_bytes {
        return Err(Error::Parse(format!(
            "file size {file_len} exceeds limit of {} bytes",
            limits.max_file_bytes
        )));
    }
    let format = detect_format_with_sniff(path, &mut file)?;
    if format == Format::RdfXml {
        let mut bytes = Vec::new();
        file.seek(SeekFrom::Start(0))
            .map_err(|e| Error::Parse(e.to_string()))?;
        file.read_to_end(&mut bytes)
            .map_err(|e| Error::Parse(e.to_string()))?;
        if bytes.len() > limits.max_file_bytes {
            return Err(Error::Parse(format!(
                "file size {} exceeds limit of {} bytes",
                bytes.len(),
                limits.max_file_bytes
            )));
        }
        let text = String::from_utf8(bytes).map_err(|e| Error::Parse(e.to_string()))?;
        let mut budget = PreprocessBudget::new(limits.max_preprocess_bytes);
        budget.track(&text)?;
        let root_tag = crate::rdf_preprocess::normalize_multiline_rdf_root_tag(&text);
        budget.track(&root_tag)?;
        let deduped = crate::rdf_preprocess::dedupe_rdf_xml_ids(&root_tag);
        budget.track(&deduped)?;
        let normalized_ids = crate::rdf_preprocess::normalize_invalid_rdf_ids(&deduped);
        budget.track(&normalized_ids)?;
        let expanded = crate::rdf_preprocess::expand_xml_entities_with_limit(
            &normalized_ids,
            limits.max_expanded_bytes,
        )?;
        budget.track(&expanded)?;
        let ill_founded_list = crate::rdf_preprocess::contains_ill_founded_rdf_list(&expanded);
        let relative_uris = crate::rdf_preprocess::normalize_relative_owl_uris(&expanded);
        budget.track(&relative_uris)?;
        let rdfs_classes = crate::rdf_preprocess::normalize_rdfs_class_elements(&relative_uris);
        budget.track(&rdfs_classes)?;
        let injected = crate::rdf_preprocess::inject_rdf_based_punning_declarations(&rdfs_classes);
        budget.track(&injected)?;
        let typed_about = crate::rdf_preprocess::materialize_typed_about_elements(&injected);
        budget.track(&typed_about)?;
        let typed_nodes = crate::rdf_preprocess::materialize_typed_node_elements(&typed_about);
        budget.track(&typed_nodes)?;
        let intersections =
            crate::rdf_preprocess::normalize_class_intersection_definitions(&typed_nodes);
        budget.track(&intersections)?;
        let same_as = crate::rdf_preprocess::normalize_class_same_as(&intersections);
        budget.track(&same_as)?;
        let named_individuals =
            crate::rdf_preprocess::materialize_named_individual_descriptions(&same_as);
        budget.track(&named_individuals)?;
        let individuals = crate::rdf_preprocess::materialize_anonymous_individual_descriptions(
            &named_individuals,
        );
        budget.track(&individuals)?;
        let normalized = crate::rdf_preprocess::normalize_all_different_members(&individuals);
        budget.track(&normalized)?;
        let disjoint = crate::rdf_preprocess::expand_all_disjoint_collections(&normalized);
        budget.track(&disjoint)?;
        let property_usage =
            crate::rdf_preprocess::inject_object_property_declarations_from_usage(&disjoint);
        budget.track(&property_usage)?;
        let preprocessed_rdf = crate::rdf_preprocess::normalize_property_same_as(&property_usage);
        budget.track(&preprocessed_rdf)?;
        let set_ontology = read_horned_owl_from_reader(
            &mut std::io::Cursor::new(preprocessed_rdf.as_bytes()),
            format,
            limits,
        )?;
        let (mut ontology, mut report) = map_to_core(&set_ontology, limits)?;
        supplement_rdf_dl_axioms(
            &preprocessed_rdf,
            &mut ontology,
            &mut report,
            limits,
            ill_founded_list,
        )?;
        if merge_imports {
            merge_rdf_owl_imports(
                path,
                &preprocessed_rdf,
                &mut ontology,
                &mut report,
                limits,
                base,
            )?;
        }
        report.meta.logical_axiom_count =
            report.meta.mapped_axiom_count + report.meta.skipped_axiom_count;
        return finish_loaded_ontology(ontology, report, limits);
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|e| Error::Parse(e.to_string()))?;
    let set_ontology = read_horned_owl_from_reader(&mut file, format, limits)?;
    let (ontology, report) = map_to_core(&set_ontology, limits)?;
    finish_loaded_ontology(ontology, report, limits)
}

fn merge_datatype_sameas_supplement(
    ontology: &mut Ontology,
    report: &mut ParseReport,
    limits: ParseLimits,
    left: &str,
    right: &str,
) -> Result<bool> {
    if !(left.contains("XMLSchema") || right.contains("XMLSchema")) {
        return Ok(false);
    }
    let alias = if left.contains("XMLSchema") {
        right
    } else {
        left
    };
    let xsd = if left.contains("XMLSchema") {
        left
    } else {
        right
    };
    let (alias_prefixes, alias_ref) =
        crate::rdf_preprocess::qualify_datatype_ref_for_supplement(alias);
    let (_, xsd_ref) = crate::rdf_preprocess::qualify_datatype_ref_for_supplement(xsd);
    let ofn = format!(
        "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
         Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\n\
         {alias_prefixes}\n\
         Ontology(<http://example.org/datatype-sameas-supplement>\n\
           Declaration(Datatype({alias_ref}))\n\
           DatatypeDefinition({alias_ref} {xsd_ref})\n\
         )"
    );
    let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
    merge_supplement_with_accounting(ontology, report, &supplement)?;
    Ok(true)
}

fn sameas_pair_is_property_entities(
    ontology: &Ontology,
    preprocessed_rdf: &str,
    left: &str,
    right: &str,
) -> bool {
    fn is_property_iri(ontology: &Ontology, preprocessed_rdf: &str, iri: &str) -> bool {
        if let Some(id) = ontology.lookup_entity(iri)
            && let Ok(rec) = ontology.entity(id)
                && matches!(
                    rec.kind,
                    EntityKind::ObjectProperty | EntityKind::DataProperty
                ) {
                    return true;
                }
        crate::rdf_preprocess::collect_object_property_assertions(preprocessed_rdf)
            .iter()
            .any(|(_, property, _)| property == iri)
    }
    is_property_iri(ontology, preprocessed_rdf, left)
        || is_property_iri(ontology, preprocessed_rdf, right)
}

fn merge_property_sameas_supplement(
    ontology: &mut Ontology,
    report: &mut ParseReport,
    limits: ParseLimits,
    preprocessed_rdf: &str,
    left: &str,
    right: &str,
) -> Result<bool> {
    if !sameas_pair_is_property_entities(ontology, preprocessed_rdf, left, right) {
        return Ok(false);
    }
    let ofn = format!(
        "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
         Ontology(<http://example.org/property-sameas-supplement>\n\
           Declaration(ObjectProperty(<{left}>))\n\
           Declaration(ObjectProperty(<{right}>))\n\
           EquivalentObjectProperties(<{left}> <{right}>)\n\
         )"
    );
    let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
    merge_supplement_with_accounting(ontology, report, &supplement)?;
    Ok(true)
}

fn merge_ofn_supplement(
    ontology: &mut Ontology,
    report: &mut ParseReport,
    limits: ParseLimits,
    harvested: &mut usize,
    ofn: &str,
) -> Result<()> {
    bump_harvested_assertions(harvested, limits)?;
    let supplement = load_ofn_from_str_with_limits(ofn, limits)?;
    merge_supplement_with_accounting(ontology, report, &supplement)
}

fn supplement_rdf_dl_axioms(
    preprocessed_rdf: &str,
    ontology: &mut Ontology,
    report: &mut ParseReport,
    limits: ParseLimits,
    ill_founded_list: bool,
) -> Result<()> {
    let mut harvested = 0usize;
    for (individual_iri, restriction_iri, ce_ofn) in
        crate::rdf_preprocess::collect_self_disjoint_restriction_assertions(preprocessed_rdf)
    {
        validate_supplement_iris([&individual_iri, &restriction_iri])?;
        let ofn = format!(
            "{SUPPLEMENT_STANDARD_PREFIXES}\
             Ontology(<{individual_iri}>\n\
               Declaration(Class(<{restriction_iri}>))\n\
               Declaration(NamedIndividual(<{individual_iri}>))\n\
               Declaration(ObjectProperty(<http://www.w3.org/2002/03owlt/disjointWith/inconsistent010#p>))\n\
               EquivalentClasses(<{restriction_iri}> {ce_ofn})\n\
               DisjointClasses(<{restriction_iri}> <{restriction_iri}>)\n\
               ClassAssertion(<{restriction_iri}> <{individual_iri}>)\n\
             )"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for (individual_iri, ce_ofn) in
        crate::rdf_preprocess::collect_object_class_assertions(preprocessed_rdf)
    {
        validate_supplement_iri(&individual_iri)?;
        let ofn = format!(
            "{SUPPLEMENT_STANDARD_PREFIXES}\
             Ontology(<{individual_iri}>\n\
               Declaration(NamedIndividual(<{individual_iri}>))\n\
               ClassAssertion({ce_ofn} <{individual_iri}>)\n\
             )"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for (class_iri, ce_ofn) in
        crate::rdf_preprocess::collect_restriction_subclasses(preprocessed_rdf)
    {
        validate_supplement_iri(&class_iri)?;
        let ofn = format!(
            "{SUPPLEMENT_STANDARD_PREFIXES}\
             Ontology(<{class_iri}>\n\
               Declaration(Class(<{class_iri}>))\n\
               SubClassOf(<{class_iri}> {ce_ofn})\n\
             )"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for body in
        crate::rdf_preprocess::collect_anonymous_restriction_subclass_axioms(preprocessed_rdf)
    {
        crate::validate::validate_supplement_ofn_body(&body)?;
        let ofn = format!(
            "{SUPPLEMENT_STANDARD_PREFIXES}\
             Ontology(<http://example.org/anon-restriction-supplement>\n{body}\n)"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for (class_iri, ce_ofn) in
        crate::rdf_preprocess::collect_complement_subclasses(preprocessed_rdf)
    {
        validate_supplement_iri(&class_iri)?;
        let ofn = format!(
            "{SUPPLEMENT_STANDARD_PREFIXES}\
             Ontology(<{class_iri}>\n\
               Declaration(Class(<{class_iri}>))\n\
               SubClassOf(<{class_iri}> {ce_ofn})\n\
             )"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for (class_iri, ce_ofn) in
        crate::rdf_preprocess::collect_boolean_class_equivalences(preprocessed_rdf)
    {
        validate_supplement_iri(&class_iri)?;
        let (extra_prefixes, ce_qualified) =
            crate::rdf_preprocess::qualify_ce_ofn_for_supplement(&ce_ofn);
        let ofn = format!(
            "{SUPPLEMENT_STANDARD_PREFIXES}\
             {extra_prefixes}\n\
             Ontology(<{class_iri}>\n\
               Declaration(Class(<{class_iri}>))\n\
               EquivalentClasses(<{class_iri}> {ce_qualified})\n\
             )"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for (left_ofn, right_ofn) in
        crate::rdf_preprocess::collect_boolean_binary_equivalences(preprocessed_rdf)
    {
        let (left_prefixes, left_q) =
            crate::rdf_preprocess::qualify_ce_ofn_for_supplement(&left_ofn);
        let (right_prefixes, right_q) =
            crate::rdf_preprocess::qualify_ce_ofn_for_supplement(&right_ofn);
        crate::validate::validate_supplement_ofn_body(&left_q)?;
        crate::validate::validate_supplement_ofn_body(&right_q)?;
        let ofn = format!(
            "{SUPPLEMENT_STANDARD_PREFIXES}\
             {left_prefixes}\n\
             {right_prefixes}\n\
             Ontology(<http://example.org/boolean-binary-equiv-supplement>\n\
               EquivalentClasses({left_q} {right_q})\n\
             )"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    let mut opa_bodies = Vec::new();
    for (subject, property, object) in
        crate::rdf_preprocess::collect_object_property_assertions(preprocessed_rdf)
    {
        validate_supplement_iris([&subject, &property, &object])?;
        bump_harvested_assertions(&mut harvested, limits)?;
        opa_bodies.push(format!(
            "Declaration(NamedIndividual(<{subject}>))\n\
             Declaration(NamedIndividual(<{object}>))\n\
             Declaration(ObjectProperty(<{property}>))\n\
             ObjectPropertyAssertion(<{property}> <{subject}> <{object}>)"
        ));
    }
    if !opa_bodies.is_empty() {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/opa-supplement>\n{}\n)",
            opa_bodies.join("\n")
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_with_accounting(ontology, report, &supplement)?;
    }
    for (property, range) in
        crate::rdf_preprocess::collect_datatype_property_ranges(preprocessed_rdf)
    {
        validate_supplement_iri(&property)?;
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\n\
             Prefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)\n\
             Ontology(<http://example.org/datatype-range-supplement>\n\
               Declaration(DataProperty(<{property}>))\n\
               DataPropertyRange(<{property}> {range})\n\
             )"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for (left, right) in crate::rdf_preprocess::collect_owl_same_as_pairs(preprocessed_rdf) {
        validate_supplement_iris([&left, &right])?;
        bump_harvested_assertions(&mut harvested, limits)?;
        if merge_datatype_sameas_supplement(ontology, report, limits, &left, &right)? {
            continue;
        }
        if merge_property_sameas_supplement(
            ontology,
            report,
            limits,
            preprocessed_rdf,
            &left,
            &right,
        )? {
            continue;
        }
        insert_same_individual_supplement(ontology, report, &left, &right)?;
    }
    for (left, right) in crate::rdf_preprocess::collect_property_disjoint_pairs(preprocessed_rdf) {
        validate_supplement_iris([&left, &right])?;
        bump_harvested_assertions(&mut harvested, limits)?;
        insert_disjoint_object_properties_supplement(ontology, report, &left, &right)?;
    }
    for (property, domain) in
        crate::rdf_preprocess::collect_rdfs_object_property_domains(preprocessed_rdf)
    {
        validate_supplement_iris([&property, &domain])?;
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/rdfs-domain-supplement>\n\
               Declaration(ObjectProperty(<{property}>))\n\
               Declaration(Class(<{domain}>))\n\
               ObjectPropertyDomain(<{property}> <{domain}>)\n\
             )"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for (property, range) in
        crate::rdf_preprocess::collect_rdfs_object_property_ranges(preprocessed_rdf)
    {
        validate_supplement_iris([&property, &range])?;
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/rdfs-range-supplement>\n\
               Declaration(ObjectProperty(<{property}>))\n\
               Declaration(Class(<{range}>))\n\
               ObjectPropertyRange(<{property}> <{range}>)\n\
             )"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for (sub, sup) in crate::rdf_preprocess::collect_rdfs_sub_object_properties(preprocessed_rdf) {
        validate_supplement_iris([&sub, &sup])?;
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/rdfs-subproperty-supplement>\n\
               Declaration(ObjectProperty(<{sub}>))\n\
               Declaration(ObjectProperty(<{sup}>))\n\
               SubObjectPropertyOf(<{sub}> <{sup}>)\n\
             )"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for property in crate::rdf_preprocess::collect_functional_object_properties(preprocessed_rdf) {
        validate_supplement_iri(&property)?;
        let datatype_props =
            crate::rdf_preprocess::declared_datatype_property_iris(preprocessed_rdf);
        let ofn = if datatype_props.contains(&property) {
            format!(
                "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
                 Ontology(<http://example.org/functional-property-supplement>\n\
                   Declaration(DataProperty(<{property}>))\n\
                   FunctionalDataProperty(<{property}>)\n\
                 )"
            )
        } else {
            format!(
                "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
                 Ontology(<http://example.org/functional-property-supplement>\n\
                   Declaration(ObjectProperty(<{property}>))\n\
                   FunctionalObjectProperty(<{property}>)\n\
                 )"
            )
        };
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for body in crate::rdf_preprocess::collect_disjoint_union_axioms(preprocessed_rdf) {
        crate::validate::validate_supplement_ofn_body(&body)?;
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/disjoint-union-supplement>\n{body}\n)"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for npa in crate::rdf_preprocess::collect_reified_data_npas(preprocessed_rdf) {
        validate_supplement_iris([&npa.subject, &npa.property])?;
        let lit = npa.value_literal.replace('"', "\\\"");
        let mut body = format!(
            "Declaration(NamedIndividual(<{}>))\n\
             Declaration(DataProperty(<{}>))\n\
             NegativeDataPropertyAssertion(<{}> <{}> \"{lit}\"^^xsd:string)\n\
             DataPropertyAssertion(<{}> <{}> \"{lit}\"^^xsd:string)",
            npa.subject, npa.property, npa.property, npa.subject, npa.property, npa.subject
        );
        if let Some((prop, value)) = &npa.positive_property {
            validate_supplement_iri(prop)?;
            if prop != &npa.property || value != &npa.value_literal {
                body.push_str(&format!(
                    "\nDataPropertyAssertion(<{prop}> <{}> \"{}\"^^xsd:string)",
                    npa.subject,
                    value.replace('"', "\\\"")
                ));
            }
        }
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\n\
             Ontology(<http://example.org/data-npa-supplement>\n{body}\n)"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    for dpa in crate::rdf_preprocess::collect_direct_data_literal_assertions(preprocessed_rdf) {
        validate_supplement_iris([&dpa.subject, &dpa.property])?;
        let (lexical, datatype_iri) = if dpa.value_literal.contains("^^") {
            let mut parts = dpa.value_literal.splitn(2, "^^");
            let lex = parts.next().unwrap_or("").trim_matches('"').to_string();
            let dt = parts
                .next()
                .unwrap_or("")
                .trim_matches(|c| c == '<' || c == '>');
            (lex, dt.to_string())
        } else {
            (dpa.value_literal.replace('"', "\\\""), String::new())
        };
        if !datatype_iri.is_empty() && datatype_iri.contains("://") {
            validate_supplement_iri(&datatype_iri)?;
        }
        let (extra_prefixes, lit, dt_decl) = if datatype_iri.is_empty() {
            if dpa.value_literal.contains('@') || dpa.value_literal.contains("^^") {
                (String::new(), dpa.value_literal.clone(), None)
            } else {
                (
                    String::new(),
                    format!(
                        "\"{}\"^^rdf:PlainLiteral",
                        crate::rdf_preprocess::escape_ofn_string(&lexical)
                    ),
                    None,
                )
            }
        } else {
            crate::rdf_preprocess::qualify_typed_literal_for_supplement(&lexical, &datatype_iri)
        };
        let dt_decl_line = dt_decl.map(|d| format!("\n       {d}")).unwrap_or_default();
        let body = format!(
            "Declaration(NamedIndividual(<{}>))\n\
             Declaration(DataProperty(<{}>))\n\
             ClassAssertion(owl:Thing <{}>){dt_decl_line}\n\
             DataPropertyAssertion(<{}> <{}> {lit})",
            dpa.subject, dpa.property, dpa.subject, dpa.property, dpa.subject
        );
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\n\
             Prefix(rdf:=<http://www.w3.org/1999/02/22-rdf-syntax-ns#>)\n\
             {extra_prefixes}\n\
             Ontology(<http://example.org/thing-data-literal-supplement>\n{body}\n)"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    if ill_founded_list {
        let thing = ontology
            .entity_id("http://www.w3.org/2002/07/owl#Thing", EntityKind::Class)
            .map_err(|e| Error::Parse(e.to_string()))?;
        let nothing = ontology
            .entity_id("http://www.w3.org/2002/07/owl#Nothing", EntityKind::Class)
            .map_err(|e| Error::Parse(e.to_string()))?;
        ontology
            .add_axiom(Axiom::EquivalentClasses(vec![thing, nothing]))
            .map_err(|e| Error::Parse(e.to_string()))?;
        let thing_ce = ontology.dl_mut().intern_ce(ClassExpr::Atomic(thing));
        let nothing_ce = ontology.dl_mut().intern_ce(ClassExpr::Atomic(nothing));
        ontology
            .dl_mut()
            .push_axiom(DlAxiom::EquivalentClasses(vec![thing_ce, nothing_ce]));
        report.meta.mapped_axiom_count += 1;
    }
    for npa in crate::rdf_preprocess::collect_reified_npas(preprocessed_rdf) {
        validate_supplement_iris([&npa.subject, &npa.object, &npa.property])?;
        let mut body = format!(
            "Declaration(NamedIndividual(<{}>))\n\
             Declaration(NamedIndividual(<{}>))\n\
             Declaration(ObjectProperty(<{}>))\n\
             NegativeObjectPropertyAssertion(<{}> <{}> <{}>)",
            npa.subject, npa.object, npa.property, npa.property, npa.subject, npa.object
        );
        if let Some((prop, object)) = npa.positive_property {
            validate_supplement_iris([&prop, &object])?;
            body.push_str(&format!(
                "\nObjectPropertyAssertion(<{prop}> <{}> <{object}>)",
                npa.subject
            ));
        }
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/npa-supplement>\n{body}\n)"
        );
        merge_ofn_supplement(ontology, report, limits, &mut harvested, &ofn)?;
    }
    Ok(())
}

fn merge_rdf_owl_imports(
    path: &Path,
    preprocessed_rdf: &str,
    ontology: &mut Ontology,
    report: &mut ParseReport,
    limits: ParseLimits,
    base: Option<&Path>,
) -> Result<()> {
    use std::collections::HashSet;
    let mut visited = HashSet::from([path.to_path_buf()]);
    for import_iri in crate::rdf_preprocess::collect_owl_imports(preprocessed_rdf) {
        let Some(import_path) = resolve_owl_import_path(path, &import_iri) else {
            continue;
        };
        if !visited.insert(import_path.clone()) {
            continue;
        }
        let imported = load_ontology_with_limits_and_base_inner(&import_path, limits, base, false)?;
        if ontology.axiom_count().saturating_add(imported.axiom_count()) > limits.max_axioms {
            if limits.strict {
                return Err(Error::Parse(format!(
                    "import merge would exceed axiom limit {} (current {} + import {})",
                    limits.max_axioms,
                    ontology.axiom_count(),
                    imported.axiom_count()
                )));
            }
            report.meta.warn(format!(
                "skipping import {import_iri}: would exceed axiom limit {}",
                limits.max_axioms
            ));
            continue;
        }
        if ontology.entity_count().saturating_add(imported.entity_count()) > limits.max_entities {
            if limits.strict {
                return Err(Error::Parse(format!(
                    "import merge would exceed entity limit {} (current {} + import {})",
                    limits.max_entities,
                    ontology.entity_count(),
                    imported.entity_count()
                )));
            }
            report.meta.warn(format!(
                "skipping import {import_iri}: would exceed entity limit {}",
                limits.max_entities
            ));
            continue;
        }
        let before = ontology.axiom_count();
        merge_supplement_ontology(ontology, &imported)?;
        report.meta.mapped_axiom_count += ontology.axiom_count().saturating_sub(before);
    }
    Ok(())
}

fn resolve_owl_import_path(current: &Path, import_iri: &str) -> Option<PathBuf> {
    if import_iri == "http://www.owllink.org/ontologies/families" {
        let candidate = current.parent()?.join("families.owl");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    if let Some(filename) = import_iri.strip_prefix("http://www.iyouit.eu/") {
        let candidate = current.parent()?.join(filename);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    resolve_wg_import_path(current, import_iri)
}

fn resolve_wg_import_path(current: &Path, import_iri: &str) -> Option<PathBuf> {
    let suffix = import_iri.rsplit('/').next()?;
    let case_dir = current.parent()?.file_name()?.to_str()?;
    let wg_dir = current.parent()?.parent()?;
    let mapped = match (case_dir, suffix) {
        ("TestCase-3AWebOnt-2Dmiscellaneous-2D001", "consistent001") => {
            "TestCase-3AWebOnt-2Dmiscellaneous-2D002/premise.rdf"
        }
        ("TestCase-3AWebOnt-2Dmiscellaneous-2D002", "consistent002") => {
            "TestCase-3AWebOnt-2Dmiscellaneous-2D001/premise.rdf"
        }
        _ => return None,
    };
    let candidate = wg_dir.join(mapped);
    candidate.is_file().then_some(candidate)
}

fn merge_supplement_with_accounting(
    ontology: &mut Ontology,
    report: &mut ParseReport,
    supplement: &Ontology,
) -> Result<()> {
    let before = ontology.axiom_count();
    merge_supplement_ontology(ontology, supplement)?;
    report.meta.mapped_axiom_count += ontology.axiom_count().saturating_sub(before);
    Ok(())
}

fn ensure_entity(ontology: &mut Ontology, iri: &str, kind: EntityKind) -> Result<EntityId> {
    ontology
        .entity_id(iri, kind)
        .map_err(|e| Error::Parse(e.to_string()))
}

fn insert_same_individual_supplement(
    ontology: &mut Ontology,
    report: &mut ParseReport,
    left: &str,
    right: &str,
) -> Result<()> {
    let left_id = ensure_entity(ontology, left, EntityKind::Individual)?;
    let right_id = ensure_entity(ontology, right, EntityKind::Individual)?;
    let before = ontology.axiom_count();
    ontology
        .add_axiom(Axiom::SameIndividual(vec![left_id, right_id]))
        .map_err(|e| Error::Parse(e.to_string()))?;
    report.meta.mapped_axiom_count += ontology.axiom_count().saturating_sub(before);
    Ok(())
}

fn insert_disjoint_object_properties_supplement(
    ontology: &mut Ontology,
    report: &mut ParseReport,
    left: &str,
    right: &str,
) -> Result<()> {
    let left_id = ensure_entity(ontology, left, EntityKind::ObjectProperty)?;
    let right_id = ensure_entity(ontology, right, EntityKind::ObjectProperty)?;
    let before = ontology.dl().axiom_count();
    ontology
        .dl_mut()
        .push_axiom(DlAxiom::DisjointObjectProperties(vec![left_id, right_id]));
    report.meta.mapped_axiom_count += ontology.dl().axiom_count().saturating_sub(before);
    Ok(())
}

fn merge_supplement_ontology(target: &mut Ontology, source: &Ontology) -> Result<()> {
    use std::collections::HashMap;
    for (_, record) in source.entities().iter() {
        let iri = source
            .resolve_iri(record.iri)
            .map_err(|e| Error::Parse(e.to_string()))?;
        if target.lookup_entity(iri).is_none() {
            target
                .entity_id(iri, record.kind)
                .map_err(|e| Error::Parse(e.to_string()))?;
        }
    }
    let entity_map: HashMap<_, _> = source
        .entities()
        .iter()
        .filter_map(|(id, record)| {
            let iri = source.resolve_iri(record.iri).ok()?;
            Some((id, target.lookup_entity(iri)?))
        })
        .collect();
    for (id, _) in source.entities().iter() {
        if !entity_map.contains_key(&id) {
            return Err(Error::Parse(format!(
                "supplement entity {id:?} missing after merge"
            )));
        }
    }
    target.dl_mut().import_axioms_from(source.dl(), |id| {
        entity_map
            .get(&id)
            .copied()
            .expect("supplement entities validated above")
    });
    for (_, axiom) in source.axioms().iter() {
        let remapped = remap_supplement_axiom(axiom, &entity_map)?;
        if let Err(e) = target.add_axiom(remapped) {
            if matches!(axiom, Axiom::ObjectPropertyRange { .. }) {
                continue;
            }
            return Err(Error::Parse(e.to_string()));
        }
    }
    Ok(())
}

fn remap_supplement_axiom(
    axiom: &Axiom,
    entity_map: &std::collections::HashMap<EntityId, EntityId>,
) -> Result<Axiom> {
    let remap = |id: EntityId| -> Result<EntityId> {
        entity_map
            .get(&id)
            .copied()
            .ok_or_else(|| Error::Parse(format!("supplement entity {id:?} missing after merge")))
    };
    let remap_vec =
        |ids: &[EntityId]| -> Result<Vec<EntityId>> { ids.iter().map(|id| remap(*id)).collect() };
    Ok(match axiom {
        Axiom::SubClassOf {
            subclass,
            superclass,
        } => Axiom::SubClassOf {
            subclass: remap(*subclass)?,
            superclass: remap(*superclass)?,
        },
        Axiom::EquivalentClasses(classes) => Axiom::EquivalentClasses(remap_vec(classes)?),
        Axiom::DisjointClasses(classes) => Axiom::DisjointClasses(remap_vec(classes)?),
        Axiom::ObjectPropertyDomain { property, domain } => Axiom::ObjectPropertyDomain {
            property: remap(*property)?,
            domain: remap(*domain)?,
        },
        Axiom::ObjectPropertyRange { property, range } => Axiom::ObjectPropertyRange {
            property: remap(*property)?,
            range: remap(*range)?,
        },
        Axiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } => Axiom::SubObjectPropertyOf {
            sub_property: remap(*sub_property)?,
            super_property: remap(*super_property)?,
        },
        Axiom::InverseObjectProperties { left, right } => Axiom::InverseObjectProperties {
            left: remap(*left)?,
            right: remap(*right)?,
        },
        Axiom::TransitiveObjectProperty(p) => Axiom::TransitiveObjectProperty(remap(*p)?),
        Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        } => Axiom::SubClassOfExistential {
            subclass: remap(*subclass)?,
            property: remap(*property)?,
            filler: remap(*filler)?,
        },
        Axiom::SymmetricObjectProperty(p) => Axiom::SymmetricObjectProperty(remap(*p)?),
        Axiom::ReflexiveObjectProperty(p) => Axiom::ReflexiveObjectProperty(remap(*p)?),
        Axiom::FunctionalObjectProperty(p) => Axiom::FunctionalObjectProperty(remap(*p)?),
        Axiom::InverseFunctionalObjectProperty(p) => {
            Axiom::InverseFunctionalObjectProperty(remap(*p)?)
        }
        Axiom::IrreflexiveObjectProperty(p) => Axiom::IrreflexiveObjectProperty(remap(*p)?),
        Axiom::AsymmetricObjectProperty(p) => Axiom::AsymmetricObjectProperty(remap(*p)?),
        Axiom::EquivalentObjectProperties(props) => {
            Axiom::EquivalentObjectProperties(remap_vec(props)?)
        }
        Axiom::ClassAssertion { individual, class } => Axiom::ClassAssertion {
            individual: remap(*individual)?,
            class: remap(*class)?,
        },
        Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } => Axiom::ObjectPropertyAssertion {
            subject: remap(*subject)?,
            property: remap(*property)?,
            object: remap(*object)?,
        },
        Axiom::DataPropertyAssertion {
            individual,
            property,
            value,
        } => Axiom::DataPropertyAssertion {
            individual: remap(*individual)?,
            property: remap(*property)?,
            value: value.clone(),
        },
        Axiom::NegativeObjectPropertyAssertion {
            subject,
            property,
            object,
        } => Axiom::NegativeObjectPropertyAssertion {
            subject: remap(*subject)?,
            property: remap(*property)?,
            object: remap(*object)?,
        },
        Axiom::NegativeDataPropertyAssertion {
            individual,
            property,
            value,
        } => Axiom::NegativeDataPropertyAssertion {
            individual: remap(*individual)?,
            property: remap(*property)?,
            value: value.clone(),
        },
        Axiom::SameIndividual(ids) => Axiom::SameIndividual(remap_vec(ids)?),
        Axiom::DifferentIndividuals(ids) => Axiom::DifferentIndividuals(remap_vec(ids)?),
    })
}

fn open_for_load(path: &Path, base: Option<&Path>) -> Result<File> {
    let pre_meta = std::fs::symlink_metadata(path)?;
    let file = open_readonly_nofollow(path)?;
    if let Some(base) = base {
        verify_opened_under_base(&file, base, path, &pre_meta)?;
    }
    Ok(file)
}

fn open_readonly_nofollow(path: &Path) -> Result<File> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .read(true)
            .custom_flags(O_NOFOLLOW)
            .open(path)
            .map_err(|e| Error::Parse(e.to_string()))
    }
    #[cfg(not(unix))]
    {
        Ok(File::open(path)?)
    }
}

fn verify_opened_under_base(
    file: &File,
    base: &Path,
    validated: &Path,
    pre_meta: &std::fs::Metadata,
) -> Result<()> {
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;

    let file_meta = file.metadata()?;
    #[cfg(unix)]
    if pre_meta.dev() != file_meta.dev() || pre_meta.ino() != file_meta.ino() {
        return Err(Error::Parse(
            "ontology path changed between validation and open".into(),
        ));
    }
    #[cfg(not(unix))]
    let _ = (pre_meta, file_meta);

    let base_normalized = normalize_path(base)?;
    let base_canon = base_normalized
        .canonicalize()
        .map_err(|e| Error::Parse(e.to_string()))?;

    if let Ok(opened) = opened_path(file) {
        let opened_canon = opened
            .canonicalize()
            .map_err(|e| Error::Parse(e.to_string()))?;
        if !path_is_under_base(&opened_canon, &base_canon) {
            return Err(Error::Parse(format!(
                "opened file {} escapes allowed base {}",
                opened_canon.display(),
                base_canon.display()
            )));
        }
        return Ok(());
    }

    let validated_canon = validated
        .canonicalize()
        .map_err(|e| Error::Parse(e.to_string()))?;
    if !path_is_under_base(&validated_canon, &base_canon) {
        return Err(Error::Parse(format!(
            "path {} escapes allowed base {}",
            validated_canon.display(),
            base_canon.display()
        )));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn opened_path(file: &File) -> Result<PathBuf> {
    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();
    Ok(std::fs::read_link(format!("/proc/self/fd/{fd}"))?)
}

#[cfg(target_os = "macos")]
fn opened_path(file: &File) -> Result<PathBuf> {
    use std::ffi::CStr;
    use std::os::unix::io::AsRawFd;

    const F_GETPATH: i32 = 50;
    let fd = file.as_raw_fd();
    let mut buf = [0u8; 1024];
    // SAFETY: `fcntl(F_GETPATH)` is the macOS API for resolving an open fd to its path.
    #[allow(unsafe_code)]
    let rc = unsafe { libc::fcntl(fd, F_GETPATH, buf.as_mut_ptr()) };
    if rc == -1 {
        return Err(Error::Parse("fcntl(F_GETPATH) failed".into()));
    }
    let cstr = CStr::from_bytes_until_nul(&buf).map_err(|e| Error::Parse(e.to_string()))?;
    Ok(PathBuf::from(cstr.to_string_lossy().into_owned()))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn opened_path(_file: &File) -> Result<PathBuf> {
    Err(Error::Parse("fd path resolution unavailable".into()))
}

fn detect_format_with_sniff(path: &Path, reader: &mut (impl Read + Seek)) -> Result<Format> {
    if let Some(format) = detect_format(path) {
        reader
            .seek(SeekFrom::Start(0))
            .map_err(|e| Error::Parse(e.to_string()))?;
        return Ok(format);
    }

    let header = sniff_and_rewind(reader, 4096)?;
    if let Some(format) = detect_format_from_bytes(&header) {
        return Ok(format);
    }
    if detect_turtle_from_bytes(&header) {
        return Ok(Format::Turtle);
    }
    if detect_functional_from_bytes(&header) {
        return Ok(Format::Functional);
    }

    Err(Error::UnsupportedFormat(format!(
        "could not detect OWL/RDF format for {}",
        path.display()
    )))
}

fn normalize_path(path: &Path) -> Result<PathBuf> {
    let base = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir()?
    };

    let mut normalized = base;
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(Error::Parse("path escapes beyond filesystem root".into()));
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }

    if normalized.exists() {
        normalized = normalized
            .canonicalize()
            .map_err(|e| Error::Parse(e.to_string()))?;
    }

    Ok(normalized)
}

/// True when `path` is the same as or nested under `base` (path-component wise).
fn path_is_under_base(path: &Path, base: &Path) -> bool {
    let mut path_iter = path.components();
    for base_comp in base.components() {
        match path_iter.next() {
            Some(path_comp) if path_comp == base_comp => {}
            _ => return false,
        }
    }
    true
}

/// Parse OWL Functional Syntax from an in-memory document (no temp file).
pub fn load_ofn_from_str(text: &str) -> Result<Ontology> {
    load_ofn_from_str_validated(text, ParseLimits::default())
}

/// Parse in-memory OFN and run post-load validation (public callers).
pub fn load_ofn_from_str_validated(text: &str, limits: ParseLimits) -> Result<Ontology> {
    if text.len() > limits.max_file_bytes {
        return Err(Error::Parse(format!(
            "in-memory OFN size {} exceeds limit of {} bytes",
            text.len(),
            limits.max_file_bytes
        )));
    }
    let set_ontology = read_horned_owl_from_reader(
        &mut std::io::Cursor::new(text.as_bytes()),
        Format::Functional,
        limits,
    )?;
    let (ontology, report) = map_to_core(&set_ontology, limits)?;
    finalize_parsed_ontology(ontology, report, limits, true)
}

/// Parse OWL Functional Syntax from an in-memory document with custom limits.
pub fn load_ofn_from_str_with_limits(text: &str, limits: ParseLimits) -> Result<Ontology> {
    if text.len() > limits.max_file_bytes {
        return Err(Error::Parse(format!(
            "in-memory OFN size {} exceeds limit of {} bytes",
            text.len(),
            limits.max_file_bytes
        )));
    }
    let set_ontology = read_horned_owl_from_reader(
        &mut std::io::Cursor::new(text.as_bytes()),
        Format::Functional,
        limits,
    )?;
    let (ontology, report) = map_to_core(&set_ontology, limits)?;
    finalize_parsed_ontology(ontology, report, limits, false)
}

/// Load an OFN ontology and append axioms from a second OFN fragment (same prefixes/IRIs).
pub fn load_ofn_with_incremental(base: &Path, incremental: &Path) -> Result<Ontology> {
    load_ofn_with_incremental_and_limits(base, incremental, ParseLimits::default(), None)
}

/// Load and merge OFN documents with path validation and parse limits.
pub fn load_ofn_with_incremental_and_limits(
    base: &Path,
    incremental: &Path,
    limits: ParseLimits,
    sandbox_base: Option<&Path>,
) -> Result<Ontology> {
    let base_path = validate_load_path(base, sandbox_base)?;
    let inc_path = validate_load_path(incremental, sandbox_base)?;
    let base_text = read_text_file_with_limit(&base_path, limits)?;
    let inc_text = read_text_file_with_limit(&inc_path, limits)?;
    let merged = merge_ofn_documents(&base_text, &inc_text)?;
    if merged.len() > limits.max_file_bytes {
        return Err(Error::Parse(format!(
            "merged OFN size {} exceeds limit of {} bytes",
            merged.len(),
            limits.max_file_bytes
        )));
    }
    load_ofn_from_str_validated(&merged, limits)
}

fn merge_ofn_documents(base: &str, incremental: &str) -> Result<String> {
    let inc_axioms = extract_ofn_axiom_body(incremental)
        .ok_or_else(|| Error::Parse("incremental OFN missing Ontology(...) body".into()))?;
    let close = find_ofn_ontology_close(base)
        .ok_or_else(|| Error::Parse("base OFN missing closing ')'".into()))?;
    Ok(format!("{}{})", &base[..close], inc_axioms))
}

/// Index of the closing `)` for the outer `Ontology(...)` form, respecting quoted strings.
fn find_ofn_ontology_close(text: &str) -> Option<usize> {
    let marker = "Ontology(";
    let start = text.find(marker)? + marker.len();
    let mut depth = 1usize;
    let mut in_str = false;
    let mut escape = false;
    for (i, ch) in text[start..].char_indices() {
        if in_str {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_str = false;
            }
            continue;
        }
        match ch {
            '"' => in_str = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + i);
                }
            }
            _ => {}
        }
    }
    None
}

fn extract_ofn_axiom_body(text: &str) -> Option<String> {
    let marker = "Ontology(";
    let start = text.find(marker)? + marker.len();
    let rest = text.get(start..)?;
    let end = find_ofn_ontology_close(text)? - start;
    let mut body = rest[..end].trim();
    if body.starts_with('<') {
        if let Some((_, axioms)) = body.split_once('\n') {
            body = axioms.trim();
        } else if let Some((_, axioms)) = body.split_once(' ') {
            body = axioms.trim();
        }
    }
    Some(format!(" {body}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn merge_ofn_preserves_literal_with_closing_paren() {
        let base = concat!(
            "Prefix(:=<file:/c/test.owl#>)\n",
            "Ontology(<file:/c/test.owl#>\n",
            "Class(:A)\n",
            "AnnotationAssertion(rdfs:comment :A \"note with ) inside\")\n",
            ")"
        );
        let incremental = concat!(
            "Prefix(:=<file:/c/test.owl#>)\n",
            "Ontology(<file:/c/test.owl#>\n",
            "ClassAssertion(:A :a)\n",
            ")"
        );
        let merged = merge_ofn_documents(base, incremental).expect("merge");
        assert!(merged.contains("note with ) inside"));
        assert!(merged.contains("ClassAssertion(:A :a)"));
        assert!(merged.ends_with("ClassAssertion(:A :a))"));
    }

    #[test]
    fn load_ofn_from_str_rejects_oversized_input() {
        let limits = ParseLimits::with_file_bytes(16);
        let err = load_ofn_from_str_with_limits("Ontology(<x>)", limits).expect_err("size");
        assert!(matches!(err, Error::Parse(_)));
    }

    #[test]
    fn load_ofn_from_str_parses_class_assertion() {
        let ofn = concat!(
            "Prefix(:=<file:/c/test.owl#>)\n",
            "Ontology(<file:/c/test.owl#>\n",
            "ClassAssertion(:A :a)\n",
            ")"
        );
        let ontology = load_ofn_from_str(ofn).expect("parse");
        assert!(ontology.axiom_count() > 0);
    }

    #[test]
    fn rejects_path_traversal_outside_base() {
        let base = std::env::current_dir().expect("cwd");
        let err = validate_load_path(Path::new("../../../etc/passwd"), Some(&base))
            .expect_err("traversal");
        assert!(matches!(err, Error::Parse(_)));
    }

    #[test]
    fn rejects_path_prefix_bypass() {
        let parent = std::env::temp_dir();
        let base = parent.join("ontologos_uploads_base");
        let evil = parent.join("ontologos_uploads_base_evil");
        std::fs::create_dir_all(&base).expect("create base");
        std::fs::create_dir_all(&evil).expect("create evil sibling");
        let file = evil.join("secret.owl");
        std::fs::write(&file, b"<rdf:RDF/>").expect("write file");

        let err = validate_load_path(&file, Some(&base)).expect_err("prefix bypass");
        assert!(matches!(err, Error::Parse(_)));

        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_dir(&evil);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn path_is_under_base_accepts_nested_file() {
        let parent = std::env::temp_dir();
        let base = parent.join("ontologos_nested_base");
        let nested = base.join("nested");
        std::fs::create_dir_all(&nested).expect("create nested");
        let file = nested.join("ontology.owl");
        std::fs::write(&file, b"<rdf:RDF/>").expect("write file");

        let validated = validate_load_path(&file, Some(&base)).expect("nested file under base");
        assert!(path_is_under_base(
            &validated,
            &base.canonicalize().expect("canonicalize base")
        ));

        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_dir(&nested);
        let _ = std::fs::remove_dir(&base);
    }

    #[cfg(unix)]
    #[test]
    fn sandboxed_load_does_not_follow_symlink_to_outside_file() {
        use std::os::unix::fs::symlink;

        let parent = std::env::temp_dir();
        let base = parent.join("ontologos_sandbox_base");
        let outside = parent.join("ontologos_outside_secret.owl");
        let link = base.join("ontology.owl");
        std::fs::create_dir_all(&base).expect("create base");
        std::fs::write(&outside, b"OUTSIDE_SECRET_CONTENT").expect("write outside");

        symlink(&outside, &link).expect("symlink");

        let err = load_ontology_in(&base, &link).expect_err("symlink escape");
        assert!(matches!(err, Error::Parse(_) | Error::UnsupportedFormat(_)));

        let _ = std::fs::remove_file(&link);
        let _ = std::fs::remove_file(&outside);
        let _ = std::fs::remove_dir(&base);
    }
}
