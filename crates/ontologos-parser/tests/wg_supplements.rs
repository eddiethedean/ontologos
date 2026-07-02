use ontologos_core::DlAxiom;
use ontologos_parser::load_ontology_lenient;
use std::path::PathBuf;

fn wg_fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(rel)
}

#[test]
fn disjoint_data_properties_premise_maps_data_property_disjointness() {
    let path = wg_fixture("wg/New-2DFeature-2DDisjointDataProperties-2D001/premise.rdf");
    let ontology = load_ontology_lenient(&path).expect("load premise");
    let has_name = ontology
        .lookup_entity("http://example.org/hasName")
        .expect("hasName");
    let has_address = ontology
        .lookup_entity("http://example.org/hasAddress")
        .expect("hasAddress");
    let disjoint = ontology.dl().axioms().any(|axiom| {
        matches!(
            axiom,
            DlAxiom::DisjointDataProperties(props)
                if props.contains(&has_name) && props.contains(&has_address)
        )
    });
    assert!(
        disjoint,
        "expected DisjointDataProperties(hasName, hasAddress)"
    );
}

#[test]
fn reflexive_sameas_conclusion_loads_without_axioms() {
    let path = wg_fixture("wg/Rdfbased-2Dsem-2Deqdis-2Dsameas-2Drflxv/conclusion.rdf");
    let ontology = load_ontology_lenient(&path).expect("load conclusion");
    assert_eq!(ontology.axiom_count(), 0);
    assert_eq!(ontology.dl().axiom_count(), 0);
}
