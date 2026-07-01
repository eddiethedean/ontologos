use super::*;

fn strip_xml_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut pos = 0usize;
    while pos < input.len() {
        let Some(rel) = input[pos..].find('<') else {
            out.push_str(&input[pos..]);
            break;
        };
        let start = pos + rel;
        out.push_str(&input[pos..start]);
        if input[start..].starts_with("<!--") || input[start..].starts_with("<!---") {
            if let Some(close) = input[start..].find("-->") {
                pos = start + close + 3;
                continue;
            }
        }
        if let Some(tag_end) = input[start..].find('>') {
            out.push_str(&input[start..start + tag_end + 1]);
            pos = start + tag_end + 1;
        } else {
            out.push_str(&input[start..]);
            break;
        }
    }
    out
}

#[test]
fn normalizes_numeric_rdf_ids() {
    let input = "<owl:Class rdf:ID=\"1.0\"/><owl:someValuesFrom rdf:resource=\"#1.0\"/>";
    let out = normalize_invalid_rdf_ids(input);
    assert!(out.contains("rdf:ID=\"_1_0\""));
    assert!(out.contains("rdf:resource=\"#_1_0\""));
    assert!(!out.contains("1.0"));
}

#[test]
fn expands_single_quoted_entity() {
    let input = r#"<?xml version="1.0"?>
<!DOCTYPE rdf:RDF [
    <!ENTITY owl 'http://www.w3.org/2002/07/owl#'>
]>
<rdf:RDF xmlns:owl="&owl;"><owl:Ontology rdf:about=""/></rdf:RDF>"#;
    let expanded = expand_xml_entities(input);
    assert!(!expanded.contains("<!ENTITY"));
    assert!(!expanded.contains("<!DOCTYPE"));
    assert!(expanded.contains("http://www.w3.org/2002/07/owl#"));
}

#[test]
fn expands_simple_entity() {
    let xml = r#"<!ENTITY ex "http://example.org/">"#;
    let input = format!(
        r#"<?xml version="1.0"?>
<!DOCTYPE rdf:RDF [{xml}]>
<rdf:RDF>&ex;a</rdf:RDF>"#
    );
    let expanded = expand_xml_entities(&input);
    assert!(expanded.contains("http://example.org/"));
    assert!(!expanded.contains("&ex;"));
}

#[test]
fn expansion_limit_rejects_blowup() {
    let xml = r#"<!ENTITY a "aaaaaaaaaa"><!ENTITY b "&a;&a;">"#;
    let input = format!(
        r#"<?xml version="1.0"?>
<!DOCTYPE rdf:RDF [{xml}]>
<rdf:RDF>&b;</rdf:RDF>"#
    );
    let err = expand_xml_entities_with_limit(&input, 64).expect_err("limit");
    assert!(err.to_string().contains("exceeds limit"));
}

#[test]
fn dedupe_removes_second_rdf_id_element() {
    let input = r##"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:owl="http://www.w3.org/2002/07/owl#">
  <owl:Class rdf:ID="Wine"><rdfs:label>first</rdfs:label></owl:Class>
  <owl:Class rdf:ID="Wine"><owl:equivalentClass rdf:resource="#Wine"/></owl:Class>
</rdf:RDF>"##;
    let out = dedupe_rdf_xml_ids(input);
    assert_eq!(out.matches("rdf:ID=\"Wine\"").count(), 1);
    assert!(out.contains("first"));
    assert!(!out.contains("equivalentClass"));
}

#[test]
fn expand_disjoint_classes_injects_pairwise_axioms() {
    let input = r#"<rdf:RDF xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <owl:AllDisjointClasses>
    <owl:members rdf:parseType="Collection">
      <rdf:Description rdf:about="http://ex.org#c1"/>
      <rdf:Description rdf:about="http://ex.org#c2"/>
    </owl:members>
  </owl:AllDisjointClasses>
</rdf:RDF>"#;
    let out = expand_all_disjoint_collections(input);
    assert!(out.contains("owl:disjointWith"));
    assert!(!out.contains("AllDisjointClasses"));
}

#[test]
fn normalize_class_intersection_wraps_direct_intersection_of() {
    let input = r#"<rdf:RDF xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
 <owl:Class rdf:about="http://ex.org#C">
  <owl:intersectionOf rdf:parseType="Collection">
   <owl:Class rdf:about="http://ex.org#A"/>
   <owl:Class rdf:about="http://ex.org#B"/>
  </owl:intersectionOf>
 </owl:Class>
</rdf:RDF>"#;
    let out = normalize_class_intersection_definitions(input);
    assert!(out.contains("<owl:equivalentClass>"));
    assert!(!out.contains("</owl:equivalentClass>\n  <owl:intersectionOf"));
}

#[test]
fn normalize_class_same_as_to_equivalent_class() {
    let input = r#"<rdf:RDF xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
 xml:base="http://www.w3.org/2002/03owlt/I4.6/premises003">
 <owl:Class rdf:ID="C1">
  <owl:sameAs>
   <owl:Class rdf:ID="C2"/>
  </owl:sameAs>
 </owl:Class>
</rdf:RDF>"#;
    let out = normalize_class_same_as(input);
    assert!(out.contains("<owl:equivalentClass>"));
    assert!(out.contains("rdf:about=\"http://www.w3.org/2002/03owlt/I4.6/premises003#C2\""));
    assert!(!out.contains("owl:sameAs"));
}

#[test]
fn materialize_typed_node_element_adds_class_assertion() {
    let input = r#"<rdf:RDF xmlns:oiled="http://oiled.man.example.net/test#"
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
 xml:base="http://ex.org/">
 <oiled:Unsatisfiable/>
</rdf:RDF>"#;
    let out = materialize_typed_node_elements(input);
    assert!(out.contains("rdf:about=\"http://ex.org#_:tn1\""));
    assert!(out.contains("rdf:resource=\"http://oiled.man.example.net/test#Unsatisfiable\""));
    assert!(!out.contains("<oiled:Unsatisfiable/>"));
}

#[test]
fn named_description_element_end_finds_outer_close() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/Datatype-2DFloat-2DDiscrete-2D001/premise.rdf");
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
    let injected = inject_rdf_based_punning_declarations(&expanded);
    let typed = materialize_typed_node_elements(&injected);
    let intersections = normalize_class_intersection_definitions(&typed);
    let start = intersections
        .find("<rdf:Description rdf:about=\"a\">")
        .unwrap_or_else(|| intersections.find("<rdf:Description").unwrap());
    let end = named_description_element_end(&intersections, start).expect("end");
    let block = &intersections[start..end];
    assert!(
        block.contains("</rdf:type>"),
        "block missing type close: {block}"
    );
    assert!(block.ends_with("</rdf:Description>"));
}

#[test]
fn float_discrete_horned_emits_class_assertion() {
    use crate::Format;
    use crate::limits::ParseLimits;
    use crate::map::map_to_core;
    use crate::read::read_horned_owl_from_reader;
    use std::io::Cursor;

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/Datatype-2DFloat-2DDiscrete-2D001/premise.rdf");
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
    let injected = inject_rdf_based_punning_declarations(&expanded);
    let typed = materialize_typed_node_elements(&injected);
    let intersections = normalize_class_intersection_definitions(&typed);
    let named = materialize_named_individual_descriptions(&intersections);
    let individuals = materialize_anonymous_individual_descriptions(&named);
    assert!(
        !individuals.contains("rdf:about=\"urn:ontologos:anon:"),
        "facet collection nodes must stay blank"
    );
    let parsed = read_horned_owl_from_reader(
        &mut Cursor::new(individuals.as_bytes()),
        Format::RdfXml,
        ParseLimits::default(),
    )
    .unwrap();
    let (ont, report) = map_to_core(&parsed, ParseLimits::default()).unwrap();
    assert!(
        ont.dl().axiom_count() > 0,
        "dl axioms skipped={}",
        report.meta.skipped_axiom_count
    );
}

#[test]
fn anonymous_individual_materialization_skips_facet_collections() {
    let input = r#"<rdf:RDF xml:base="http://ex.org/"
 xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
 xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
 xmlns:xsd="http://www.w3.org/2001/XMLSchema#">
 <owl:withRestrictions rdf:parseType="Collection">
  <rdf:Description>
   <xsd:minExclusive rdf:datatype="http://www.w3.org/2001/XMLSchema#float">0.0</xsd:minExclusive>
  </rdf:Description>
 </owl:withRestrictions>
 <rdf:Description>
  <rdf:type rdf:resource="http://ex.org/C"/>
 </rdf:Description>
</rdf:RDF>"#;
    let out = materialize_anonymous_individual_descriptions(input);
    assert!(out.contains("rdf:about=\"http://ex.org#_:1\""));
    assert!(out.contains("<rdf:Description>\n   <xsd:minExclusive"));
}

#[test]
fn tagged_element_end_self_close_with_iri_attr() {
    let s = r#"<owl:assertionProperty rdf:resource="http://www.example.org#p"/>"#;
    assert!(
        tagged_element_end(s, 0, "owl:assertionProperty").is_some(),
        "self-close with IRI attr"
    );
}

#[test]
fn collect_reified_data_npa_minimal() {
    let text = r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:ex="http://www.example.org#">
  <rdf:Description rdf:about="http://www.example.org#z">
    <owl:sourceIndividual>
      <rdf:Description rdf:about="http://www.example.org#s">
        <ex:p>data</ex:p>
      </rdf:Description>
    </owl:sourceIndividual>
    <owl:assertionProperty rdf:resource="http://www.example.org#p"/>
    <owl:targetValue>data</owl:targetValue>
  </rdf:Description>
</rdf:RDF>"#;
    let start = text.find("<rdf:Description").unwrap();
    let end =
        element_block_end(text, start, "rdf:Description", "</rdf:Description>").expect("block end");
    let block = &text[start..end];
    let open_end = block.find('>').unwrap();
    let close_idx = block.rfind("</rdf:Description>").unwrap();
    let inner = &block[open_end + 1..close_idx];
    assert!(inner.contains("owl:targetValue"), "inner={inner}");
    assert!(
        find_top_level_element_bounds(inner, "owl:sourceIndividual").is_some(),
        "sourceIndividual missing"
    );
    assert!(
        element_text_content(inner, "owl:targetValue").is_some(),
        "targetValue missing"
    );
    let collected = collect_reified_data_npas(text);
    assert_eq!(collected.len(), 1, "{collected:?}");
}

#[test]
fn misc203_dpa_supplement_ofn_parses() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Dmiscellaneous-2D203/premise.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let normalized_ids = normalize_invalid_rdf_ids(&deduped);
    let expanded = expand_xml_entities_with_limit(&normalized_ids, 1_000_000).unwrap();
    let relative_uris = normalize_relative_owl_uris(&expanded);
    let rdfs_classes = normalize_rdfs_class_elements(&relative_uris);
    let injected = inject_rdf_based_punning_declarations(&rdfs_classes);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let intersections = normalize_class_intersection_definitions(&typed_nodes);
    let same_as = normalize_class_same_as(&intersections);
    let named_individuals = materialize_named_individual_descriptions(&same_as);
    let individuals = materialize_anonymous_individual_descriptions(&named_individuals);
    let normalized = normalize_all_different_members(&individuals);
    let disjoint = expand_all_disjoint_collections(&normalized);
    let collected = collect_direct_data_literal_assertions(&disjoint);
    let dpa = &collected[0];
    let (lexical, datatype_iri) = if dpa.value_literal.contains("^^") {
        let mut parts = dpa.value_literal.splitn(2, "^^");
        let lex = parts.next().unwrap().trim_matches('"').to_string();
        let dt = parts.next().unwrap().trim_matches(|c| c == '<' || c == '>');
        (lex, dt.to_string())
    } else {
        panic!("unexpected literal");
    };
    let (extra_prefixes, lit, dt_decl) =
        qualify_typed_literal_for_supplement(&lexical, &datatype_iri);
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
    eprintln!("{ofn}");
    let ont = crate::load_ofn_from_str(&ofn).expect("supplement ofn");
    assert!(
        ont.dl()
            .axioms()
            .any(|a| { matches!(a, ontologos_core::DlAxiom::DataPropertyAssertion { .. }) })
    );
}

#[test]
fn misc203_full_load_has_data_assertions() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Dmiscellaneous-2D203/premise.rdf",
    );
    let ont = crate::load_ontology(&path).expect("load misc203");
    let dpas: Vec<_> = ont
        .dl()
        .axioms()
        .filter(|a| matches!(a, ontologos_core::DlAxiom::DataPropertyAssertion { .. }))
        .collect();
    eprintln!("dpas={dpas:?}");
    assert_eq!(
        dpas.len(),
        2,
        "all axioms: {:?}",
        ont.dl().axioms().collect::<Vec<_>>()
    );
}

#[test]
fn collect_direct_data_literals_from_misc203_after_full_preprocess() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Dmiscellaneous-2D203/premise.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let normalized_ids = normalize_invalid_rdf_ids(&deduped);
    let expanded = expand_xml_entities_with_limit(&normalized_ids, 1_000_000).unwrap();
    let relative_uris = normalize_relative_owl_uris(&expanded);
    let rdfs_classes = normalize_rdfs_class_elements(&relative_uris);
    let injected = inject_rdf_based_punning_declarations(&rdfs_classes);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let intersections = normalize_class_intersection_definitions(&typed_nodes);
    let same_as = normalize_class_same_as(&intersections);
    let named_individuals = materialize_named_individual_descriptions(&same_as);
    let individuals = materialize_anonymous_individual_descriptions(&named_individuals);
    let normalized = normalize_all_different_members(&individuals);
    let disjoint = expand_all_disjoint_collections(&normalized);
    let collected = collect_direct_data_literal_assertions(&disjoint);
    eprintln!("after full preprocess collected={collected:?}");
    assert_eq!(collected.len(), 2, "{collected:?}");
}

#[test]
fn collect_direct_data_literals_from_misc203() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Dmiscellaneous-2D203/premise.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let normalized_ids = normalize_invalid_rdf_ids(&deduped);
    let expanded = expand_xml_entities_with_limit(&normalized_ids, 1_000_000).unwrap();
    let relative_uris = normalize_relative_owl_uris(&expanded);
    let injected = inject_rdf_based_punning_declarations(&relative_uris);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let collected = collect_direct_data_literal_assertions(&typed_nodes);
    eprintln!("collected={collected:?}");
    assert_eq!(collected.len(), 2, "{collected:?}");
}

#[test]
fn xml_literal_data_property_supplement_loads() {
    let ofn = "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Prefix(rdf:=<http://www.w3.org/1999/02/22-rdf-syntax-ns#>)\n\
             Ontology(<http://example.org/xml-lit-test>\n\
               Declaration(NamedIndividual(<http://example.org/i>))\n\
               Declaration(DataProperty(<http://example.org/p>))\n\
               DataPropertyAssertion(<http://example.org/p> <http://example.org/i> \"<b>x</b>\"^^rdf:XMLLiteral)\n\
             )";
    let ont = crate::load_ofn_from_str(ofn).expect("load xml literal dpa");
    let has_dpa = ont
        .dl()
        .axioms()
        .any(|a| matches!(a, ontologos_core::DlAxiom::DataPropertyAssertion { .. }));
    assert!(
        has_dpa,
        "axioms: {:?}",
        ont.dl().axioms().collect::<Vec<_>>()
    );
}

#[test]
fn collect_reified_data_npa_from_fixture() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Dnpa-2Ddat-2Dfw/premise.rdf");
    let text = std::fs::read_to_string(&path).unwrap();
    let collected = collect_reified_data_npas(&text);
    assert_eq!(collected.len(), 1, "{collected:?}");
    assert_eq!(collected[0].value_literal, "data");
}

#[test]
fn named_description_skips_inverse_functional_property_typing() {
    let input = r#"<rdf:RDF xml:base="http://ex.org/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
 <rdf:Description rdf:about="http://ex.org/p">
  <rdf:type rdf:resource="http://www.w3.org/2002/07/owl#DatatypeProperty"/>
 </rdf:Description>
</rdf:RDF>"#;
    let out = materialize_named_individual_descriptions(input);
    assert!(out.contains("<rdf:Description rdf:about=\"http://ex.org/p\">"));
    assert!(!out.contains("NamedIndividual"));
}

#[test]
fn manual_rewrite_once_on_float() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/Datatype-2DFloat-2DDiscrete-2D001/premise.rdf");
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
    let injected = inject_rdf_based_punning_declarations(&expanded);
    let typed = materialize_typed_node_elements(&injected);
    let intersections = normalize_class_intersection_definitions(&typed);
    let start = intersections
        .find("<rdf:Description rdf:about=\"a\">")
        .unwrap();
    let end = named_description_element_end(&intersections, start).unwrap();
    let block = &intersections[start..end];
    assert!(
        block.ends_with("</rdf:Description>"),
        "block end: {:?}",
        &block[block.len().saturating_sub(40)..]
    );
    let rewritten = rewrite_description_to_named_individual(block);
    assert!(
        !rewritten.contains("rdf:ty</owl:NamedIndividual>"),
        "{rewritten}"
    );
}

#[test]
fn float_discrete_preprocess_produces_valid_xml() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/Datatype-2DFloat-2DDiscrete-2D001/premise.rdf");
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
    let injected = inject_rdf_based_punning_declarations(&expanded);
    let typed = materialize_typed_node_elements(&injected);
    let intersections = normalize_class_intersection_definitions(&typed);
    let named = materialize_named_individual_descriptions(&intersections);
    if named.contains("rdf:ty</owl:NamedIndividual>") {
        panic!("bad output:\n{named}");
    }
    assert!(named.contains("<owl:NamedIndividual"));
}

#[test]
fn materialize_named_description_to_individual() {
    let input = r#"<rdf:RDF xml:base="http://ex.org/ontology/"
 xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
 <rdf:Description rdf:about="a">
  <rdf:type>
   <owl:Restriction>
    <owl:onProperty rdf:resource="dp"/>
   </owl:Restriction>
  </rdf:type>
 </rdf:Description>
</rdf:RDF>"#;
    let out = materialize_named_individual_descriptions(input);
    assert!(out.contains("<owl:NamedIndividual"));
    assert!(!out.contains("<rdf:Description rdf:about=\"a\">"));
}

#[test]
fn horned_parses_object_restriction_via_equivalent_class() {
    use crate::Format;
    use crate::limits::ParseLimits;
    use crate::read::read_horned_owl_from_reader;
    use std::io::Cursor;

    let mini = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xml:base="http://example.org/">
<owl:Ontology/>
<owl:Class rdf:about="probe">
  <owl:equivalentClass>
    <owl:Restriction>
      <owl:onProperty rdf:resource="http://www.w3.org/2002/07/owl#bottomObjectProperty" />
      <owl:someValuesFrom rdf:resource="http://www.w3.org/2002/07/owl#Thing" />
    </owl:Restriction>
  </owl:equivalentClass>
</owl:Class>
</rdf:RDF>"#;
    let parsed = read_horned_owl_from_reader(
        &mut Cursor::new(mini.as_bytes()),
        Format::RdfXml,
        ParseLimits::default(),
    )
    .unwrap();
    for annotated in parsed.iter() {
        eprintln!("{:?}", annotated.component);
    }
}

#[test]
fn horned_components_bottom_vs_float() {
    use crate::Format;
    use crate::limits::ParseLimits;
    use crate::read::read_horned_owl_from_reader;
    use std::io::Cursor;

    for case in [
        "New-2DFeature-2DBottomObjectProperty-2D001",
        "Datatype-2DFloat-2DDiscrete-2D001",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg")
            .join(case)
            .join("premise.rdf");
        let text = std::fs::read_to_string(&path).unwrap();
        let deduped = dedupe_rdf_xml_ids(&text);
        let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
        let injected = inject_rdf_based_punning_declarations(&expanded);
        let typed = materialize_typed_node_elements(&injected);
        let intersections = normalize_class_intersection_definitions(&typed);
        let same_as = normalize_class_same_as(&intersections);
        let named = materialize_named_individual_descriptions(&same_as);
        let individuals = materialize_anonymous_individual_descriptions(&named);
        let parsed = read_horned_owl_from_reader(
            &mut Cursor::new(individuals.as_bytes()),
            Format::RdfXml,
            ParseLimits::default(),
        )
        .unwrap();
        eprintln!("\n=== {case} ===");
        for annotated in parsed.iter() {
            eprintln!("  {:?}", annotated.component);
        }
    }
}

#[test]
fn restriction_006_union_ofn_debug() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DRestriction-2D006/conclusion.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
    let injected = inject_rdf_based_punning_declarations(&expanded);
    let typed = materialize_typed_node_elements(&injected);
    let intersections = normalize_class_intersection_definitions(&typed);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    for (name, stage) in [
        ("raw", text.as_str()),
        ("deduped", deduped.as_str()),
        ("expanded", expanded.as_str()),
        ("injected", injected.as_str()),
        ("typed", typed.as_str()),
        ("intersections", intersections.as_str()),
        ("same_as", same_as.as_str()),
        ("named", named.as_str()),
    ] {
        let has_restriction = stage.contains("<owl:Restriction");
        let has_union_child = stage.contains("unionOf") && stage.contains("<owl:Class");
        eprintln!("{name}: restriction={has_restriction} union_child={has_union_child}");
    }
    let base = parse_xml_base(&text);
    let dt_props = declared_datatype_property_iris(&text);
    let node_lists = build_rdf_collection_node_map(&text, &base, &dt_props);
    let mut pos = 0usize;
    let mut ce_block = String::new();
    while pos < named.len() {
        let next_desc = named[pos..].find("<rdf:Description");
        let next_ind = named[pos..].find("<owl:NamedIndividual");
        let next = match (next_desc, next_ind) {
            (Some(d), Some(i)) => Some(pos + d.min(i)),
            (Some(d), None) => Some(pos + d),
            (None, Some(i)) => Some(pos + i),
            (None, None) => None,
        };
        let Some(start) = next else { break };
        if named[start..].starts_with("</") {
            pos = start + 1;
            continue;
        }
        let (open_tag, close_tag) = if named[start..].starts_with("<owl:NamedIndividual") {
            ("owl:NamedIndividual", "</owl:NamedIndividual>")
        } else {
            ("rdf:Description", "</rdf:Description>")
        };
        let Some(end) = element_block_end(&named, start, open_tag, close_tag) else {
            break;
        };
        let block = &named[start..end];
        if block.contains("premises006#x") {
            let open_end = block.find('>').unwrap();
            let close_start = block.rfind(close_tag).unwrap();
            let inner = &block[open_end + 1..close_start];
            if let Some((_, _, type_block)) = find_top_level_element_bounds(inner, "rdf:type") {
                ce_block =
                    unwrap_typed_class_expression_block(&element_inner(type_block, "rdf:type"));
            }
            break;
        }
        pos = end;
    }
    let union_ofn = boolean_operator_ofn(
        ce_block.trim(),
        "owl:unionOf",
        "ObjectUnionOf",
        &base,
        &dt_props,
        &node_lists,
    );
    assert!(
        union_ofn
            .as_deref()
            .is_some_and(|u| u.contains("ObjectUnionOf")),
        "union_ofn={union_ofn:?}"
    );
}

#[test]
fn restriction_006_conclusion_emits_union_class_assertion() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DRestriction-2D006/conclusion.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
    let injected = inject_rdf_based_punning_declarations(&expanded);
    let typed = materialize_typed_node_elements(&injected);
    let intersections = normalize_class_intersection_definitions(&typed);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let collected = collect_object_class_assertions(&named);
    assert!(
        collected.iter().any(|(_, ce)| ce.contains("ObjectUnionOf")),
        "{collected:?}"
    );
}

#[test]
fn i5_1_premise_loads() {
    use crate::load_ontology;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DI5.1-2D001/premise.rdf");
    load_ontology(&path).expect("load I5.1 premise");
}

#[test]
fn transitive_property_conclusion_emits_class_assertion() {
    use crate::load_ontology;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DTransitiveProperty-2D002/conclusion.rdf",
        );
    let ont = load_ontology(&path).expect("load");
    assert!(
        ont.dl()
            .axioms()
            .any(|a| matches!(a, ontologos_core::DlAxiom::ClassAssertion { .. })),
        "expected DL ClassAssertion"
    );
}

#[test]
fn collect_object_class_assertion_restriction_with_nested_class_type() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DTransitiveProperty-2D002/conclusion.rdf",
        );
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
    let injected = inject_rdf_based_punning_declarations(&expanded);
    let typed = materialize_typed_node_elements(&injected);
    let intersections = normalize_class_intersection_definitions(&typed);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let collected = collect_object_class_assertions(&named);
    assert_eq!(collected.len(), 1, "{collected:?}");
    assert!(collected[0].1.contains("ObjectSomeValuesFrom"));
}

#[test]
fn collect_object_class_assertion_bottom_property() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/New-2DFeature-2DBottomObjectProperty-2D001/premise.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
    let injected = inject_rdf_based_punning_declarations(&expanded);
    let typed = materialize_typed_node_elements(&injected);
    let intersections = normalize_class_intersection_definitions(&typed);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let collected = collect_object_class_assertions(&named);
    assert_eq!(collected.len(), 1);
    assert!(collected[0].1.contains("bottomObjectProperty"));
}

#[test]
fn rational003_oneof_has_two_literals() {
    let avf = r#"<owl:Restriction>
      <owl:onProperty rdf:resource="dp" />
      <owl:allValuesFrom>
        <rdfs:Datatype>
          <owl:oneOf>
            <rdf:Description>
              <rdf:first rdf:datatype="http://www.w3.org/2001/XMLSchema#decimal">0.3333333333333333</rdf:first>
              <rdf:rest>
                <rdf:Description>
                  <rdf:first rdf:datatype="http://www.w3.org/2002/07/owl#rational">1/3</rdf:first>
                  <rdf:rest rdf:resource="http://www.w3.org/1999/02/22-rdf-syntax-ns#"/>
                </rdf:Description>
              </rdf:rest>
            </rdf:Description>
          </owl:oneOf>
        </rdfs:Datatype>
      </owl:allValuesFrom>
    </owl:Restriction>"#;
    let base = "http://example.org/";
    let mut dt = std::collections::HashSet::new();
    dt.insert("http://example.org/dp".to_string());
    let ofn = data_restriction_ce_to_ofn(avf, base, &dt).expect("ofn");
    assert!(ofn.contains("DataOneOf"), "{ofn}");
    assert!(ofn.contains("0.3333333333333333"), "{ofn}");
    assert!(ofn.contains("1/3"), "{ofn}");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/New-2DFeature-2DRational-2D003/premise.rdf");
    let rdf = std::fs::read_to_string(&path).expect("read");
    let deduped = dedupe_rdf_xml_ids(&rdf);
    let normalized_ids = normalize_invalid_rdf_ids(&deduped);
    let relative_uris = normalize_relative_owl_uris(&normalized_ids);
    let injected = inject_rdf_based_punning_declarations(&relative_uris);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let intersections = normalize_class_intersection_definitions(&typed_nodes);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let collected = collect_object_class_assertions(&named);
    assert_eq!(collected.len(), 2, "{collected:?}");
    assert!(
        collected
            .iter()
            .any(|(_, ce)| ce.contains("DataOneOf") && ce.contains("1/3")),
        "{collected:?}"
    );
    assert!(
        collected
            .iter()
            .any(|(_, ce)| ce.contains("DataMinCardinality")),
        "{collected:?}"
    );
}

#[test]
fn anonymous_restriction_subclass_axiom_ofn() {
    let rdf = r#"<rdf:RDF xml:base="file:/c:/temp/test.owl"
 xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
<owl:Class rdf:about="file:/c:/temp/test.owl#d"/>
<owl:ObjectProperty rdf:about="file:/c:/temp/test.owl#r"/>
<owl:ObjectProperty rdf:about="file:/c:/temp/test.owl#s"/>
<owl:Class rdf:about="file:/c:/temp/test.owl#c"/>
<owl:Restriction>
  <rdfs:subClassOf rdf:resource="file:/c:/temp/test.owl#d"/>
  <owl:onProperty rdf:resource="file:/c:/temp/test.owl#r"/>
  <owl:someValuesFrom>
    <owl:Restriction>
      <owl:onProperty rdf:resource="file:/c:/temp/test.owl#s"/>
      <owl:someValuesFrom rdf:resource="file:/c:/temp/test.owl#c"/>
    </owl:Restriction>
  </owl:someValuesFrom>
</owl:Restriction>
</rdf:RDF>"#;
    let axioms = collect_anonymous_restriction_subclass_axioms(rdf);
    assert_eq!(axioms.len(), 1);
    assert!(axioms[0].contains("SubClassOf("));
    assert!(axioms[0].contains("ObjectSomeValuesFrom"));
    assert!(axioms[0].contains("file:/c:/temp/test.owl#d"));
}

#[test]
fn rational002_allvaluesfrom_oneof_ofn() {
    let avf = r#"<owl:Restriction>
      <owl:onProperty rdf:resource="dp" />
      <owl:allValuesFrom>
        <rdfs:Datatype>
          <owl:oneOf>
            <rdf:Description>
              <rdf:first rdf:datatype="http://www.w3.org/2001/XMLSchema#decimal">0.5</rdf:first>
              <rdf:rest>
                <rdf:Description>
                  <rdf:first rdf:datatype="http://www.w3.org/2002/07/owl#rational">1/2</rdf:first>
                  <rdf:rest rdf:resource="http://www.w3.org/1999/02/22-rdf-syntax-ns#"/>
                </rdf:Description>
              </rdf:rest>
            </rdf:Description>
          </owl:oneOf>
        </rdfs:Datatype>
      </owl:allValuesFrom>
    </owl:Restriction>"#;
    let base = "http://example.org/";
    let mut dt = std::collections::HashSet::new();
    dt.insert("http://example.org/dp".to_string());
    let ofn = data_restriction_ce_to_ofn(avf, base, &dt);
    assert!(ofn.is_some_and(|s| s.contains("DataAllValuesFrom")));
}

#[test]
fn minimal_subclass_restriction_harvest_preprocessed() {
    let rdf = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/minimal_subclass.rdf"),
    )
    .expect("read");
    let deduped = dedupe_rdf_xml_ids(&rdf);
    let normalized_ids = normalize_invalid_rdf_ids(&deduped);
    let expanded = expand_xml_entities_with_limit(&normalized_ids, 50 * 1024 * 1024).unwrap();
    let relative_uris = normalize_relative_owl_uris(&expanded);
    let rdfs_classes = normalize_rdfs_class_elements(&relative_uris);
    let injected = inject_rdf_based_punning_declarations(&rdfs_classes);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let intersections = normalize_class_intersection_definitions(&typed_nodes);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let individuals = materialize_anonymous_individual_descriptions(&named);
    let normalized = normalize_all_different_members(&individuals);
    let disjoint = expand_all_disjoint_collections(&normalized);
    let property_usage = inject_object_property_declarations_from_usage(&disjoint);
    let preprocessed = normalize_property_same_as(&property_usage);
    let collected = collect_restriction_subclasses(&preprocessed);
    assert!(
        collected
            .iter()
            .any(|(_, ce)| ce.contains("ObjectSomeValuesFrom")),
        "expected someValuesFrom supplement on preprocessed RDF, got {collected:?}"
    );
}

#[test]
fn nominals2_max_cardinality_restriction_ofn() {
    let block = r#"<owl:Restriction>
                        <owl:onProperty rdf:resource="file:/c:/temp/test.owl#r"/>
                        <owl:onClass rdf:resource="file:/c:/temp/test.owl#d"/>
                        <owl:maxCardinality rdf:datatype="http://www.w3.org/2001/XMLSchema#nonNegativeInteger">2</owl:maxCardinality>
                    </owl:Restriction>"#;
    let base = "file:/c:/temp/test.owl";
    let dt = std::collections::HashSet::new();
    let ofn = restriction_ce_to_ofn(block.trim(), base, &dt);
    assert!(ofn.is_some_and(|s| s.contains("ObjectMaxCardinality")));
}

#[test]
fn nominals2_union_subclass_ofn_direct() {
    let sub_inner = r#"<owl:Class>
                <owl:unionOf rdf:parseType="Collection">
                    <rdf:Description rdf:about="file:/c:/temp/test.owl#e"/>
                    <owl:Restriction>
                        <owl:onProperty rdf:resource="file:/c:/temp/test.owl#r"/>
                        <owl:onClass rdf:resource="file:/c:/temp/test.owl#d"/>
                        <owl:maxCardinality rdf:datatype="http://www.w3.org/2001/XMLSchema#nonNegativeInteger">2</owl:maxCardinality>
                    </owl:Restriction>
                </owl:unionOf>
            </owl:Class>"#;
    let base = "file:/c:/temp/test.owl";
    let dt = std::collections::HashSet::new();
    let nl = std::collections::HashMap::new();
    let ofn = superclass_ce_ofn_from_subclass_inner(sub_inner.trim(), base, &dt, &nl);
    assert!(
        ofn.as_ref()
            .is_some_and(|s| s.contains("ObjectUnionOf") && s.contains("ObjectMaxCardinality")),
        "got {ofn:?}"
    );
}

#[test]
fn nominals2_union_subclass_loads() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/structural/nominals-2-input.xml");
    let rdf = std::fs::read_to_string(&path).expect("read");
    let deduped = dedupe_rdf_xml_ids(&rdf);
    let normalized_ids = normalize_invalid_rdf_ids(&deduped);
    let expanded = expand_xml_entities_with_limit(&normalized_ids, 50 * 1024 * 1024).unwrap();
    let relative_uris = normalize_relative_owl_uris(&expanded);
    let rdfs_classes = normalize_rdfs_class_elements(&relative_uris);
    let injected = inject_rdf_based_punning_declarations(&rdfs_classes);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let intersections = normalize_class_intersection_definitions(&typed_nodes);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let individuals = materialize_anonymous_individual_descriptions(&named);
    let normalized = normalize_all_different_members(&individuals);
    let disjoint = expand_all_disjoint_collections(&normalized);
    let property_usage = inject_object_property_declarations_from_usage(&disjoint);
    let preprocessed = normalize_property_same_as(&property_usage);
    let collected = collect_restriction_subclasses(&preprocessed);
    assert!(
        collected
            .iter()
            .any(|(_, ce)| ce.contains("ObjectUnionOf") && ce.contains("ObjectMaxCardinality")),
        "expected union supplement, got {collected:?}"
    );
}

#[test]
fn data_restriction_min_cardinality_ofn() {
    let ce = r#"<owl:Restriction>
    <owl:onProperty rdf:resource="dp"/>
    <owl:minCardinality>2</owl:minCardinality>
  </owl:Restriction>"#;
    let base = "http://example.org/";
    let mut dt = std::collections::HashSet::new();
    dt.insert("http://example.org/dp".to_string());
    let ofn = data_restriction_ce_to_ofn(ce, base, &dt);
    assert_eq!(
        ofn.as_deref(),
        Some("DataMinCardinality(2 <http://example.org/dp>)")
    );
}

#[test]
fn rational002_collects_both_data_restrictions() {
    let inline_min = r#"<rdf:RDF xml:base="http://example.org/"
 xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
<owl:DatatypeProperty rdf:about="dp"/>
<rdf:Description rdf:about="a">
  <owl:onProperty rdf:resource="dp"/>
  <owl:minCardinality>2</owl:minCardinality>
</rdf:Description>
</rdf:RDF>"#;
    let inline = collect_object_class_assertions(inline_min);
    assert_eq!(inline.len(), 1, "inline={inline:?}");

    let typed_min = r#"<rdf:RDF xml:base="http://example.org/"
 xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
<owl:DatatypeProperty rdf:about="dp"/>
<rdf:Description rdf:about="a">
  <rdf:type><owl:Restriction>
    <owl:onProperty rdf:resource="dp"/>
    <owl:minCardinality>2</owl:minCardinality>
  </owl:Restriction></rdf:type>
</rdf:Description>
</rdf:RDF>"#;
    let typed = collect_object_class_assertions(typed_min);
    assert_eq!(typed.len(), 1, "typed={typed:?}");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/New-2DFeature-2DRational-2D002/premise.rdf");
    let rdf = std::fs::read_to_string(&path).expect("read");
    let deduped = dedupe_rdf_xml_ids(&rdf);
    let normalized_ids = normalize_invalid_rdf_ids(&deduped);
    let relative_uris = normalize_relative_owl_uris(&normalized_ids);
    let injected = inject_rdf_based_punning_declarations(&relative_uris);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let intersections = normalize_class_intersection_definitions(&typed_nodes);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let collected = collect_object_class_assertions(&named);
    assert_eq!(collected.len(), 2, "{collected:?}");
    assert!(
        collected
            .iter()
            .any(|(_, ce)| ce.contains("DataAllValuesFrom")),
        "{collected:?}"
    );
    assert!(
        collected
            .iter()
            .any(|(_, ce)| ce.contains("DataMinCardinality")),
        "{collected:?}"
    );
}

#[test]
fn rational002_full_load_parses() {
    use crate::load_ontology;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/New-2DFeature-2DRational-2D002/premise.rdf");
    let ont = load_ontology(&path).expect("load rational002");
    assert!(ont.dl().axiom_count() > 0);
}

#[test]
fn bottom_property_emits_class_assertion() {
    use crate::load_ontology;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/New-2DFeature-2DBottomObjectProperty-2D001/premise.rdf",
    );
    let ont = load_ontology(&path).expect("load");
    assert!(ont.dl().axiom_count() > 0);
}

#[test]
fn materialize_named_description_skips_thing_subclass() {
    let input = r#"<rdf:RDF xmlns:owl="http://www.w3.org/2002/07/owl#"
 xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
 xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#">
 <rdf:Description rdf:about="http://www.w3.org/2002/07/owl#Thing">
  <rdfs:subClassOf>
   <owl:Restriction>
    <owl:onProperty rdf:resource="http://ex.org#f"/>
   </owl:Restriction>
  </rdfs:subClassOf>
 </rdf:Description>
</rdf:RDF>"#;
    let out = materialize_named_individual_descriptions(input);
    assert!(out.contains("<rdf:Description"));
    assert!(!out.contains("<owl:NamedIndividual"));
}

#[test]
fn materialize_anonymous_description_adds_about() {
    let input = r#"<rdf:RDF xml:base="http://ex.org/">
  <rdf:Description>
    <rdf:type rdf:resource="http://ex.org#C"/>
  </rdf:Description>
</rdf:RDF>"#;
    let out = materialize_anonymous_individual_descriptions(input);
    assert!(out.contains("rdf:about=\"http://ex.org#_:1\""));
}

#[test]
fn description_element_end_parses_member_descriptions() {
    let inner = r#"
      <rdf:Description rdf:about="http://ex.org#w1">
        <owl:sameAs rdf:resource="http://ex.org#w2"/>
      </rdf:Description>
      <rdf:Description rdf:about="http://ex.org#w2"/>
    "#;
    let start = inner.find("<rdf:Description").unwrap();
    let end = super::description_element_end(inner, start).expect("first desc end");
    assert!(inner[start..end].contains("sameAs"));
    let start2 = inner[end..].find("<rdf:Description").unwrap() + end;
    let end2 = super::description_element_end(inner, start2).expect("second desc end");
    assert!(inner[start2..end2].contains("w2"));
}

#[test]
fn normalize_all_different_members_tag() {
    let input = r#"<rdf:RDF xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <owl:AllDifferent rdf:about="http://ex.org#z">
    <owl:members rdf:parseType="Collection">
      <rdf:Description rdf:about="http://ex.org#w1">
        <owl:sameAs rdf:resource="http://ex.org#w2"/>
      </rdf:Description>
      <rdf:Description rdf:about="http://ex.org#w2"/>
    </owl:members>
  </owl:AllDifferent>
</rdf:RDF>"#;
    let out = normalize_all_different_members(input);
    assert!(out.contains("owl:distinctMembers"));
    assert!(out.contains("<owl:NamedIndividual rdf:about=\"http://ex.org#w1\"/>"));
    assert!(!out.contains("AllDifferent rdf:about"));
    assert!(out.contains("owl:sameAs"));
}

#[test]
fn injects_nested_class_declarations_for_subst_fixture() {
    let input = include_str!(
        "../../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Deqdis-2Deqclass-2Dsubst/premise.rdf"
    );
    let out = inject_rdf_based_punning_declarations(input);
    assert!(out.contains("<owl:Class rdf:about=\"http://www.example.org#c1\"/>"));
    assert!(out.contains("<owl:Class rdf:about=\"http://www.example.org#d1\"/>"));
}

#[test]
fn injects_class_declarations_for_rdf_based_equivalent_class() {
    let input = include_str!(
        "../../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Deqdis-2Deqclass-2Dinst/premise.rdf"
    );
    let out = inject_rdf_based_punning_declarations(input);
    assert!(out.contains("<owl:Class rdf:about=\"http://www.example.org#c1\"/>"));
    assert!(out.contains("<owl:Class rdf:about=\"http://www.example.org#c2\"/>"));
}

#[test]
fn injects_property_declarations_for_rdf_based_equivalent_property() {
    let input = include_str!(
        "../../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Deqdis-2Deqprop-2Dinst/premise.rdf"
    );
    let out = inject_rdf_based_punning_declarations(input);
    assert!(out.contains("<owl:ObjectProperty rdf:about=\"http://www.example.org#p1\"/>"));
    assert!(out.contains("<owl:ObjectProperty rdf:about=\"http://www.example.org#p2\"/>"));
}

#[test]
fn injects_datatype_property_for_rdf_based_rflxv_conclusion() {
    let input = include_str!(
        "../../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Deqdis-2Deqprop-2Drflxv/conclusion.rdf"
    );
    let out = inject_rdf_based_punning_declarations(input);
    assert!(out.contains("<owl:DatatypeProperty rdf:about=\"http://www.example.org#dp\"/>"));
    assert!(out.contains("<owl:ObjectProperty rdf:about=\"http://www.example.org#op\"/>"));
}

#[test]
fn injects_datatype_peer_for_equivalent_property_target() {
    let input = r#"<rdf:RDF xml:base="http://example.com/owl/families/"
  xmlns="http://example.com/owl/families/"
  xmlns:owl="http://www.w3.org/2002/07/owl#"
  xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <owl:DatatypeProperty rdf:about="hasAge">
    <owl:equivalentProperty rdf:resource="http://example.org/otherOntologies/families/age"/>
  </owl:DatatypeProperty>
</rdf:RDF>"#;
    let out = inject_rdf_based_punning_declarations(input);
    assert!(
        out.contains(
            "<owl:DatatypeProperty rdf:about=\"http://example.org/otherOntologies/families/age\"/>"
        ),
        "expected imported age peer as datatype property, got:\n{out}"
    );
    assert!(
        !out.contains(
            "<owl:ObjectProperty rdf:about=\"http://example.org/otherOntologies/families/age\"/>"
        ),
        "must not inject object property for datatype equivalent peer:\n{out}"
    );
}

#[test]
fn wine_fixture_has_no_duplicate_ids_after_dedupe() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/reasoner/res/wine.xml");
    let input = std::fs::read_to_string(&path).expect("wine.xml");
    let deduped = dedupe_rdf_xml_ids(&input);
    let without_comments = strip_xml_comments(&deduped);
    let mut seen = HashSet::new();
    let mut dupes = Vec::new();
    for line in without_comments.lines() {
        if let Some(idx) = line.find("rdf:ID=\"") {
            let rest = &line[idx + 8..];
            if let Some(end) = rest.find('"') {
                let id = &rest[..end];
                if !seen.insert(id.to_owned()) {
                    dupes.push(id.to_owned());
                }
            }
        }
    }
    assert!(dupes.is_empty(), "duplicate ids remain: {dupes:?}");
}

#[test]
fn functional_property_004_conclusion_loads_functional_axiom() {
    use crate::load_ontology;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DFunctionalProperty-2D004/conclusion.rdf",
        );
    let ont = load_ontology(&path).expect("load");
    let has_functional = ont
        .axioms()
        .iter()
        .any(|(_, ax)| matches!(ax, ontologos_core::Axiom::FunctionalObjectProperty(_)));
    eprintln!(
        "conclusion axioms: {:?}",
        ont.axioms().iter().collect::<Vec<_>>()
    );
    eprintln!("conclusion dl: {:?}", ont.dl().axioms().collect::<Vec<_>>());
    assert!(
        has_functional,
        "expected FunctionalObjectProperty in conclusion"
    );
}

#[test]
fn functional_property_004_oneof_supplement() {
    use crate::load_ontology;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DFunctionalProperty-2D004/premise.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let root = normalize_multiline_rdf_root_tag(&text);
    let deduped = dedupe_rdf_xml_ids(&root);
    let normalized_ids = normalize_invalid_rdf_ids(&deduped);
    let expanded = expand_xml_entities_with_limit(&normalized_ids, 10_000_000).unwrap();
    let relative_uris = normalize_relative_owl_uris(&expanded);
    let rdfs_classes = normalize_rdfs_class_elements(&relative_uris);
    let injected = inject_rdf_based_punning_declarations(&rdfs_classes);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let intersections = normalize_class_intersection_definitions(&typed_nodes);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let individuals = materialize_anonymous_individual_descriptions(&named);
    let normalized = normalize_all_different_members(&individuals);
    let preprocessed = expand_all_disjoint_collections(&normalized);
    let equivs = collect_boolean_class_equivalences(&preprocessed);
    assert!(
        equivs.iter().any(|(_, ce)| ce.contains("ObjectOneOf")),
        "expected singleton oneOf equivalence, got {equivs:?}\npreprocessed:\n{preprocessed}"
    );
    let ont = load_ontology(&path).expect("load");
    assert!(
        ont.dl().axiom_count() > 0,
        "dl axioms: {:?}",
        ont.dl().axioms().collect::<Vec<_>>()
    );
}

#[test]
fn normalize_multiline_rdf_root_tag_collapses_functional_property_fixture() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DFunctionalProperty-2D004/premise.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let collapsed = normalize_multiline_rdf_root_tag(&text);
    assert!(
        !collapsed.contains("\n  xmlns:owl"),
        "root tag should be collapsed: {}",
        &collapsed[..collapsed.find('>').unwrap_or(collapsed.len())]
    );
    let loaded = crate::load_ontology(&path).expect("load functional property premise");
    assert!(loaded.axiom_count() > 0 || loaded.dl().axiom_count() > 0);
}

#[test]
fn parse_xmlns_multiline_root_with_trailing_angle() {
    let input = r#"<rdf:RDF 
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:oiled="http://oiled.man.example.net/test#">
 <oiled:Unsatisfiable/>
</rdf:RDF>"#;
    let xmlns = parse_xmlns(input);
    assert_eq!(
        xmlns.get("oiled"),
        Some(&"http://oiled.man.example.net/test#".to_owned())
    );
    let out = materialize_typed_node_elements(input);
    assert!(out.contains("#_:tn1"));
    assert!(!out.contains("<oiled:Unsatisfiable/>"));
}

#[test]
fn dl035_materialize_typed_node_on_fixture() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D035/premise.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
    let out = materialize_typed_node_elements(&expanded);
    assert!(out.contains("#_:tn1"));
    assert!(!out.contains("<oiled:Unsatisfiable/>"));
}

#[test]
fn dl035_preprocess_retains_typed_node_and_spy_individual() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D035/premise.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
    let injected = inject_rdf_based_punning_declarations(&expanded);
    let typed = materialize_typed_node_elements(&injected);
    assert!(
        typed.contains("#_:tn1"),
        "typed node not materialized:\n{typed}"
    );
    assert!(!typed.contains("<oiled:Unsatisfiable/>"));
    let intersections = normalize_class_intersection_definitions(&typed);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let individuals = materialize_anonymous_individual_descriptions(&named);
    assert!(individuals.contains("test#spy"));
    assert!(individuals.contains("owl:NamedIndividual"));
}

#[test]
fn equivalent_class_007_binary_equivalence_ofn() {
    use crate::load_ontology;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DequivalentClass-2D007/conclusion.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let expanded = expand_xml_entities_with_limit(&deduped, 1_000_000).unwrap();
    let relative = normalize_relative_owl_uris(&expanded);
    let rdfs = normalize_rdfs_class_elements(&relative);
    let injected = inject_rdf_based_punning_declarations(&rdfs);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let intersections = normalize_class_intersection_definitions(&typed_nodes);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let individuals = materialize_anonymous_individual_descriptions(&named);
    let normalized = normalize_all_different_members(&individuals);
    let disjoint = expand_all_disjoint_collections(&normalized);
    let preprocessed = normalize_property_same_as(&disjoint);
    let pairs = collect_boolean_binary_equivalences(&preprocessed);
    assert_eq!(pairs.len(), 1, "pairs={pairs:?}");
    let left = &pairs[0].0;
    let right = &pairs[0].1;
    assert!(left.contains("ObjectComplementOf"), "left={left}");
    assert!(right.contains("ObjectComplementOf"), "right={right}");
    assert!(left.contains("ObjectIntersectionOf"), "left={left}");
    assert!(right.contains("ObjectUnionOf"), "right={right}");
    let conc = load_ontology(&path).unwrap();
    assert!(
        conc.dl()
            .axioms()
            .any(|a| matches!(a, ontologos_core::DlAxiom::EquivalentClasses(_))),
        "expected EquivalentClasses in DL"
    );
}

#[test]
fn wg_nothing_001_loads_class_assertion() {
    use crate::load_ontology;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DNothing-2D001/premise.rdf");
    let ont = load_ontology(&path).unwrap();
    assert!(
        ont.dl().axiom_count() > 0,
        "expected ClassAssertion for owl:Nothing typed node"
    );
}

#[test]
fn intersection_of_001_supplementation() {
    use crate::load_ontology;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DintersectionOf-2D001/premise.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let equivs = collect_boolean_class_equivalences(&text);
    assert!(
        equivs.len() >= 2,
        "expected boolean equivalences, got {equivs:?}"
    );
    let ont = load_ontology(&path).expect("load");
    assert!(
        ont.dl().axiom_count() >= 3,
        "axiom_count={}",
        ont.dl().axiom_count()
    );
}

#[test]
fn collect_rdfs_domain_from_domain_cond() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Drdfs-2Ddomain-2Dcond/premise.rdf");
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let normalized_ids = normalize_invalid_rdf_ids(&deduped);
    let expanded = expand_xml_entities_with_limit(&normalized_ids, 10_000_000).expect("expand");
    let relative_uris = normalize_relative_owl_uris(&expanded);
    let injected = inject_rdf_based_punning_declarations(&relative_uris);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let intersections = normalize_class_intersection_definitions(&typed_nodes);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let anon = materialize_anonymous_individual_descriptions(&named);
    let normalized = normalize_all_different_members(&anon);
    let final_out = expand_all_disjoint_collections(&normalized);
    let domains = collect_rdfs_object_property_domains(&final_out);
    assert!(
        !domains.is_empty(),
        "expected rdfs:domain after full pipeline, got {domains:?}\nfinal:\n{final_out}"
    );
}

#[test]
fn wg_somevalues_inst_subj_preprocess_no_nested_description() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Drestrict-2Dsomevalues-2Dinst-2Dsubj/premise.rdf",
        );
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let normalized_ids = normalize_invalid_rdf_ids(&deduped);
    let expanded = expand_xml_entities_with_limit(&normalized_ids, 10_000_000).expect("expand");
    let relative_uris = normalize_relative_owl_uris(&expanded);
    let injected = inject_rdf_based_punning_declarations(&relative_uris);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let intersections = normalize_class_intersection_definitions(&typed_nodes);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let anon = materialize_anonymous_individual_descriptions(&named);
    let normalized = normalize_all_different_members(&anon);
    let final_out = expand_all_disjoint_collections(&normalized);
    for (label, out) in [
        ("injected", &injected),
        ("typed_about", &typed_about),
        ("typed_nodes", &typed_nodes),
        ("intersections", &intersections),
        ("same_as", &same_as),
        ("named", &named),
        ("anon", &anon),
        ("normalized", &normalized),
        ("final", &final_out),
    ] {
        if out.matches("<rdf:Description").count() > 2 {
            eprintln!(
                "=== {label} ({} descriptions) ===\n{out}\n",
                out.matches("<rdf:Description").count()
            );
        }
    }
    use crate::Format;
    use crate::limits::ParseLimits;
    use crate::read::read_horned_owl_from_reader;
    read_horned_owl_from_reader(
        &mut std::io::Cursor::new(final_out.into_bytes()),
        Format::RdfXml,
        ParseLimits::default(),
    )
    .expect("horned-owl parse");
}

#[test]
fn collect_opa_from_owl_thing_with_blank_node_object() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DallValuesFrom-2D002/conclusion.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let opas = collect_object_property_assertions(&text);
    assert_eq!(opas.len(), 1, "expected one OPA, got {opas:?}");
    assert!(opas[0].0.contains("premises002#i"));
    assert!(opas[0].1.contains("premises002#p"));
    assert!(opas[0].2.contains("#_o"));

    let deduped = dedupe_rdf_xml_ids(&text);
    let normalized_ids = normalize_invalid_rdf_ids(&deduped);
    let expanded = expand_xml_entities_with_limit(&normalized_ids, 1_000_000).unwrap();
    let relative_uris = normalize_relative_owl_uris(&expanded);
    let injected = inject_rdf_based_punning_declarations(&relative_uris);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let intersections = normalize_class_intersection_definitions(&typed_nodes);
    let same_as = normalize_class_same_as(&intersections);
    let named_individuals = materialize_named_individual_descriptions(&same_as);
    let individuals = materialize_anonymous_individual_descriptions(&named_individuals);
    let normalized = normalize_all_different_members(&individuals);
    let preprocessed = expand_all_disjoint_collections(&normalized);
    let preprocessed_opas = collect_object_property_assertions(&preprocessed);
    assert_eq!(
        preprocessed_opas.len(),
        1,
        "expected one OPA, got {preprocessed_opas:?}"
    );
    let loaded = crate::load_ontology(&path).expect("load conclusion");
    let has_opa = loaded
        .dl()
        .axioms()
        .any(|a| matches!(a, ontologos_core::DlAxiom::ObjectPropertyAssertion { .. }))
        || loaded
            .axioms()
            .iter()
            .any(|(_, a)| matches!(a, ontologos_core::Axiom::ObjectPropertyAssertion { .. }));
    assert!(has_opa, "loaded ontology missing OPA");
}

#[test]
fn collect_nested_opa_from_somevalues_conclusion() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DsomeValuesFrom-2D003/conclusion.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let opas = collect_object_property_assertions(&text);
    assert_eq!(opas.len(), 2, "expected two OPAs, got {opas:?}");
    assert!(opas[0].0.contains("fred"));
    assert!(opas[0].1.contains("parent"));
    let loaded = crate::load_ontology(&path).expect("load conclusion");
    let opa_count = loaded
        .axioms()
        .iter()
        .filter(|(_, a)| matches!(a, ontologos_core::Axiom::ObjectPropertyAssertion { .. }))
        .count();
    assert_eq!(opa_count, 2, "loaded conclusion should have two OPAs");
}

#[test]
fn qualify_ce_ofn_percent23_fragment_uses_hash_namespace() {
    let ce = "ObjectUnionOf(<http://www.w3.org/2002/03owlt/I5.5/premises005%23a>)";
    let (prefixes, qualified) = qualify_ce_ofn_for_supplement(ce);
    assert!(prefixes.contains("premises005#>"));
    assert_eq!(qualified, "ns1:a");
    let normalized = normalize_supplement_boolean_ce(ce);
    assert_eq!(
        normalized,
        "<http://www.w3.org/2002/03owlt/I5.5/premises005%23a>"
    );
}

#[test]
fn i5_5_005_singleton_union_supplement() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DI5.5-2D005/conclusion.rdf");
    let text = std::fs::read_to_string(&path).unwrap();
    let deduped = dedupe_rdf_xml_ids(&text);
    let normalized_ids = normalize_invalid_rdf_ids(&deduped);
    let expanded = expand_xml_entities_with_limit(&normalized_ids, 10_000_000).unwrap();
    let relative_uris = normalize_relative_owl_uris(&expanded);
    let injected = inject_rdf_based_punning_declarations(&relative_uris);
    let typed_about = materialize_typed_about_elements(&injected);
    let typed_nodes = materialize_typed_node_elements(&typed_about);
    let intersections = normalize_class_intersection_definitions(&typed_nodes);
    let same_as = normalize_class_same_as(&intersections);
    let named = materialize_named_individual_descriptions(&same_as);
    let individuals = materialize_anonymous_individual_descriptions(&named);
    let normalized = normalize_all_different_members(&individuals);
    let preprocessed = expand_all_disjoint_collections(&normalized);
    let equivs = collect_boolean_class_equivalences(&preprocessed);
    assert!(
        equivs.iter().any(|(_, ce)| ce.contains("ObjectUnionOf")),
        "expected singleton union equivalence, got {equivs:?}"
    );
}

#[test]
fn disjoint_with_010_premise_loads_abox() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DdisjointWith-2D010/premise.rdf");
    let text = std::fs::read_to_string(&path).unwrap();
    let opas = collect_object_property_assertions(&text);
    assert_eq!(opas.len(), 1, "expected one OPA, got {opas:?}");
    let typing = collect_self_disjoint_restriction_assertions(&text);
    assert_eq!(
        typing.len(),
        1,
        "expected one class assertion, got {typing:?}"
    );
    let loaded = crate::load_ontology(&path).expect("load");
    assert!(
        loaded.dl().axiom_count() > 0 || !loaded.axioms().is_empty(),
        "loaded ontology should have axioms"
    );
    assert!(
        !ontologos_dl::is_consistent(&loaded).expect("check"),
        "disjointWith-010 should be inconsistent"
    );
}

#[test]
fn sameas_subst_premise_collects_nested_opa_and_loads() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Deqdis-2Dsameas-2Dsubst/premise.rdf",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let opas = collect_object_property_assertions(&text);
    assert!(
        opas.iter().any(|(_, prop, _)| prop.contains("#p1")),
        "expected nested sameAs OPA on p1, got {opas:?}"
    );
    let loaded = crate::load_ontology(&path).expect("load premise");
    let p1 = "http://www.example.org#p1";
    let id = loaded.lookup_entity(p1).expect("p1 entity");
    let rec = loaded.entity(id).expect("p1 record");
    assert_eq!(
        rec.kind,
        ontologos_core::EntityKind::ObjectProperty,
        "p1 should be object property, got {:?}",
        rec.kind
    );
}
