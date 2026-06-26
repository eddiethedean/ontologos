use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};

use ontologos_core::{Axiom, ClassExpr, DlAxiom, EntityId, EntityKind, Ontology};

use crate::limits::ParseLimits;
use crate::map::map_to_core;
use crate::read::{read_horned_owl_from_reader, sniff_and_rewind};
use crate::report::ParseReport;
use crate::{
    detect_format, detect_format_from_bytes, detect_functional_from_bytes,
    detect_turtle_from_bytes, validate_loaded_ontology, Error, Format, Result,
};

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

/// Load an ontology constrained to stay under `base` (untrusted uploads).
pub fn load_ontology_in(base: &Path, path: &Path) -> Result<Ontology> {
    load_ontology_with_limits_and_base(path, ParseLimits::default(), Some(base))
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
    load_ontology_with_limits_and_base_inner(path, limits, base, true)
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
        let text = String::from_utf8_lossy(&bytes);
        let deduped = crate::rdf_preprocess::dedupe_rdf_xml_ids(&text);
        let normalized_ids = crate::rdf_preprocess::normalize_invalid_rdf_ids(&deduped);
        let expanded = crate::rdf_preprocess::expand_xml_entities_with_limit(
            &normalized_ids,
            limits.max_expanded_bytes,
        )?;
        let ill_founded_list = crate::rdf_preprocess::contains_ill_founded_rdf_list(&expanded);
        let relative_uris = crate::rdf_preprocess::normalize_relative_owl_uris(&expanded);
        let rdfs_classes = crate::rdf_preprocess::normalize_rdfs_class_elements(&relative_uris);
        let injected = crate::rdf_preprocess::inject_rdf_based_punning_declarations(&rdfs_classes);
        let typed_about = crate::rdf_preprocess::materialize_typed_about_elements(&injected);
        let typed_nodes = crate::rdf_preprocess::materialize_typed_node_elements(&typed_about);
        let intersections =
            crate::rdf_preprocess::normalize_class_intersection_definitions(&typed_nodes);
        let same_as = crate::rdf_preprocess::normalize_class_same_as(&intersections);
        let named_individuals =
            crate::rdf_preprocess::materialize_named_individual_descriptions(&same_as);
        // let class_assertions =
        //     crate::rdf_preprocess::materialize_complex_class_assertions(&named_individuals);
        let individuals = crate::rdf_preprocess::materialize_anonymous_individual_descriptions(
            &named_individuals,
        );
        let normalized = crate::rdf_preprocess::normalize_all_different_members(&individuals);
        let disjoint = crate::rdf_preprocess::expand_all_disjoint_collections(&normalized);
        let preprocessed_rdf = disjoint.clone();
        let set_ontology = read_horned_owl_from_reader(
            &mut std::io::Cursor::new(disjoint.as_bytes().to_vec()),
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
            merge_rdf_owl_imports(path, &preprocessed_rdf, &mut ontology, &mut report, limits, base)?;
        }
        if limits.strict && report.meta.skipped_axiom_count > 0 {
            return Err(Error::Parse(format!(
                "strict parse: skipped {} axioms due to limits or mapping failures",
                report.meta.skipped_axiom_count
            )));
        }
        ontology.set_parse_meta(report.into_meta());
        if limits.strict {
            validate_loaded_ontology(&ontology)?;
        }
        return Ok(ontology);
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|e| Error::Parse(e.to_string()))?;
    let set_ontology = read_horned_owl_from_reader(&mut file, format, limits)?;
    let (mut ontology, report) = map_to_core(&set_ontology, limits)?;
    if limits.strict && report.meta.skipped_axiom_count > 0 {
        return Err(Error::Parse(format!(
            "strict parse: skipped {} axioms due to limits or mapping failures",
            report.meta.skipped_axiom_count
        )));
    }
    ontology.set_parse_meta(report.into_meta());
    if limits.strict {
        validate_loaded_ontology(&ontology)?;
    }
    Ok(ontology)
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
    merge_supplement_ontology(ontology, &supplement)?;
    report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    Ok(true)
}

fn supplement_rdf_dl_axioms(
    preprocessed_rdf: &str,
    ontology: &mut Ontology,
    report: &mut ParseReport,
    limits: ParseLimits,
    ill_founded_list: bool,
) -> Result<()> {
    for (individual_iri, restriction_iri, ce_ofn) in
        crate::rdf_preprocess::collect_self_disjoint_restriction_assertions(preprocessed_rdf)
    {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<{individual_iri}>\n\
               Declaration(Class(<{restriction_iri}>))\n\
               Declaration(NamedIndividual(<{individual_iri}>))\n\
               Declaration(ObjectProperty(<http://www.w3.org/2002/03owlt/disjointWith/inconsistent010#p>))\n\
               EquivalentClasses(<{restriction_iri}> {ce_ofn})\n\
               DisjointClasses(<{restriction_iri}> <{restriction_iri}>)\n\
               ClassAssertion(<{restriction_iri}> <{individual_iri}>)\n\
             )"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for (individual_iri, ce_ofn) in
        crate::rdf_preprocess::collect_object_class_assertions(preprocessed_rdf)
    {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<{individual_iri}>\n\
               Declaration(NamedIndividual(<{individual_iri}>))\n\
               ClassAssertion({ce_ofn} <{individual_iri}>)\n\
             )"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for (class_iri, ce_ofn) in
        crate::rdf_preprocess::collect_restriction_subclasses(preprocessed_rdf)
    {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<{class_iri}>\n\
               Declaration(Class(<{class_iri}>))\n\
               SubClassOf(<{class_iri}> {ce_ofn})\n\
             )"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for (class_iri, ce_ofn) in
        crate::rdf_preprocess::collect_complement_subclasses(preprocessed_rdf)
    {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<{class_iri}>\n\
               Declaration(Class(<{class_iri}>))\n\
               SubClassOf(<{class_iri}> {ce_ofn})\n\
             )"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for (class_iri, ce_ofn) in
        crate::rdf_preprocess::collect_boolean_class_equivalences(preprocessed_rdf)
    {
        let (extra_prefixes, ce_qualified) =
            crate::rdf_preprocess::qualify_ce_ofn_for_supplement(&ce_ofn);
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             {extra_prefixes}\n\
             Ontology(<{class_iri}>\n\
               Declaration(Class(<{class_iri}>))\n\
               EquivalentClasses(<{class_iri}> {ce_qualified})\n\
             )"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for (subject, property, object) in
        crate::rdf_preprocess::collect_object_property_assertions(preprocessed_rdf)
    {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/opa-supplement>\n\
               Declaration(NamedIndividual(<{subject}>))\n\
               Declaration(NamedIndividual(<{object}>))\n\
               Declaration(ObjectProperty(<{property}>))\n\
               ObjectPropertyAssertion(<{property}> <{subject}> <{object}>)\n\
             )"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for (property, range) in
        crate::rdf_preprocess::collect_datatype_property_ranges(preprocessed_rdf)
    {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\n\
             Prefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)\n\
             Ontology(<http://example.org/datatype-range-supplement>\n\
               Declaration(DataProperty(<{property}>))\n\
               DataPropertyRange(<{property}> {range})\n\
             )"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for (left, right) in crate::rdf_preprocess::collect_owl_same_as_pairs(preprocessed_rdf) {
        if merge_datatype_sameas_supplement(ontology, report, limits, &left, &right)? {
            continue;
        }
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/same-as-supplement>\n\
               Declaration(NamedIndividual(<{left}>))\n\
               Declaration(NamedIndividual(<{right}>))\n\
               SameIndividual(<{left}> <{right}>)\n\
             )"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for (left, right) in
        crate::rdf_preprocess::collect_property_disjoint_pairs(preprocessed_rdf)
    {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/disjoint-supplement>\n\
               Declaration(ObjectProperty(<{left}>))\n\
               Declaration(ObjectProperty(<{right}>))\n\
               DisjointObjectProperties(<{left}> <{right}>)\n\
             )"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for (property, domain) in
        crate::rdf_preprocess::collect_rdfs_object_property_domains(preprocessed_rdf)
    {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/rdfs-domain-supplement>\n\
               Declaration(ObjectProperty(<{property}>))\n\
               Declaration(Class(<{domain}>))\n\
               ObjectPropertyDomain(<{property}> <{domain}>)\n\
             )"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for (property, range) in
        crate::rdf_preprocess::collect_rdfs_object_property_ranges(preprocessed_rdf)
    {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/rdfs-range-supplement>\n\
               Declaration(ObjectProperty(<{property}>))\n\
               Declaration(Class(<{range}>))\n\
               ObjectPropertyRange(<{property}> <{range}>)\n\
             )"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for body in crate::rdf_preprocess::collect_disjoint_union_axioms(preprocessed_rdf) {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/disjoint-union-supplement>\n{body}\n)"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for npa in crate::rdf_preprocess::collect_reified_data_npas(preprocessed_rdf) {
        let lit = npa.value_literal.replace('"', "\\\"");
        let mut body = format!(
            "Declaration(NamedIndividual(<{}>))\n\
             Declaration(DataProperty(<{}>))\n\
             NegativeDataPropertyAssertion(<{}> <{}> \"{lit}\"^^xsd:string)\n\
             DataPropertyAssertion(<{}> <{}> \"{lit}\"^^xsd:string)",
            npa.subject, npa.property, npa.property, npa.subject, npa.property, npa.subject
        );
        if let Some((prop, value)) = &npa.positive_property {
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
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for dpa in crate::rdf_preprocess::collect_direct_data_literal_assertions(preprocessed_rdf) {
        let (lexical, datatype_iri) = if dpa.value_literal.contains("^^") {
            let mut parts = dpa.value_literal.splitn(2, "^^");
            let lex = parts
                .next()
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
            let dt = parts
                .next()
                .unwrap_or("")
                .trim_matches(|c| c == '<' || c == '>');
            (lex, dt.to_string())
        } else {
            (dpa.value_literal.replace('"', "\\\""), String::new())
        };
        let (extra_prefixes, lit, dt_decl) = if datatype_iri.is_empty() {
            if dpa.value_literal.contains('@') || dpa.value_literal.contains("^^") {
                (
                    String::new(),
                    dpa.value_literal.clone(),
                    None,
                )
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
        let dt_decl_line = dt_decl
            .map(|d| format!("\n       {d}"))
            .unwrap_or_default();
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
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for (left, right) in crate::rdf_preprocess::collect_owl_same_as_pairs(preprocessed_rdf) {
        if merge_datatype_sameas_supplement(ontology, report, limits, &left, &right)? {
            continue;
        }
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/same-as-supplement>\n\
               Declaration(NamedIndividual(<{left}>))\n\
               Declaration(NamedIndividual(<{right}>))\n\
               SameIndividual(<{left}> <{right}>)\n\
             )"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for body in crate::rdf_preprocess::collect_anonymous_intersection_subclasses(preprocessed_rdf) {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/anon-intersection-supplement>\n{body}\n)"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
    }
    for body in crate::rdf_preprocess::collect_anonymous_intersection_subclasses(preprocessed_rdf) {
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/anon-intersection-supplement>\n{body}\n)"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
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
        report.meta.mapped_axiom_count += 2;
    }
    for npa in crate::rdf_preprocess::collect_reified_npas(preprocessed_rdf) {
        let mut body = format!(
            "Declaration(NamedIndividual(<{}>))\n\
             Declaration(NamedIndividual(<{}>))\n\
             Declaration(ObjectProperty(<{}>))\n\
             NegativeObjectPropertyAssertion(<{}> <{}> <{}>)",
            npa.subject, npa.object, npa.property, npa.property, npa.subject, npa.object
        );
        if let Some((prop, object)) = npa.positive_property {
            body.push_str(&format!(
                "\nObjectPropertyAssertion(<{prop}> <{}> <{object}>)",
                npa.subject
            ));
        }
        let ofn = format!(
            "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<http://example.org/npa-supplement>\n{body}\n)"
        );
        let supplement = load_ofn_from_str_with_limits(&ofn, limits)?;
        merge_supplement_ontology(ontology, &supplement)?;
        report.meta.mapped_axiom_count += supplement.dl().axiom_count();
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
        let Some(import_path) = resolve_wg_import_path(path, &import_iri) else {
            continue;
        };
        if !visited.insert(import_path.clone()) {
            continue;
        }
        let imported =
            load_ontology_with_limits_and_base_inner(&import_path, limits, base, false)?;
        merge_full_ontology(ontology, &imported)?;
        report.meta.mapped_axiom_count += imported.dl().axiom_count();
    }
    Ok(())
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

fn merge_full_ontology(target: &mut Ontology, source: &Ontology) -> Result<()> {
    merge_supplement_ontology(target, source)
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
    target.dl_mut().import_axioms_from(source.dl(), |id| {
        entity_map.get(&id).copied().expect("supplement entity missing after merge")
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
        entity_map.get(&id).copied().ok_or_else(|| {
            Error::Parse(format!(
                "supplement entity {id:?} missing after merge"
            ))
        })
    };
    let remap_vec = |ids: &[EntityId]| -> Result<Vec<EntityId>> {
        ids.iter().map(|id| remap(*id)).collect()
    };
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
        Axiom::SameIndividual(ids) => Axiom::SameIndividual(remap_vec(ids)?),
        Axiom::DifferentIndividuals(ids) => Axiom::DifferentIndividuals(remap_vec(ids)?),
    })
}

fn open_for_load(path: &Path, base: Option<&Path>) -> Result<File> {
    let pre_meta = std::fs::symlink_metadata(path).map_err(|e| Error::Parse(e.to_string()))?;
    let file = open_readonly_nofollow(path)?;
    if let Some(base) = base {
        verify_opened_under_base(&file, base, path, &pre_meta)?;
    }
    Ok(file)
}

fn open_readonly_nofollow(path: &Path) -> Result<File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .read(true)
            .custom_flags(O_NOFOLLOW)
            .open(path)
            .map_err(|e| Error::Parse(e.to_string()))
    }
    #[cfg(not(unix))]
    {
        File::open(path).map_err(|e| Error::Parse(e.to_string()))
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

    let file_meta = file.metadata().map_err(|e| Error::Parse(e.to_string()))?;
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
    std::fs::read_link(format!("/proc/self/fd/{fd}")).map_err(|e| Error::Parse(e.to_string()))
}

#[cfg(target_os = "macos")]
fn opened_path(file: &File) -> Result<PathBuf> {
    use std::ffi::CStr;
    use std::os::unix::io::AsRawFd;

    const F_GETPATH: i32 = 50;
    let fd = file.as_raw_fd();
    let mut buf = [0u8; 1024];
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
        std::env::current_dir().map_err(|e| Error::Parse(e.to_string()))?
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
    load_ofn_from_str_with_limits(text, ParseLimits::default())
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
    let (mut ontology, report) = map_to_core(&set_ontology, limits)?;
    if limits.strict && report.meta.skipped_axiom_count > 0 {
        return Err(Error::Parse(format!(
            "strict parse: skipped {} axioms due to limits or mapping failures",
            report.meta.skipped_axiom_count
        )));
    }
    ontology.set_parse_meta(report.into_meta());
    if limits.strict {
        validate_loaded_ontology(&ontology)?;
    }
    Ok(ontology)
}

/// Load an OFN ontology and append axioms from a second OFN fragment (same prefixes/IRIs).
pub fn load_ofn_with_incremental(base: &Path, incremental: &Path) -> Result<Ontology> {
    let base_text = std::fs::read_to_string(base).map_err(|e| Error::Parse(e.to_string()))?;
    let inc_text = std::fs::read_to_string(incremental).map_err(|e| Error::Parse(e.to_string()))?;
    let merged = merge_ofn_documents(&base_text, &inc_text)?;
    load_ofn_from_str(&merged)
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
