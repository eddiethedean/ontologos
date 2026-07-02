use ontologos_parser::load_ontology;

#[test]
fn pizza_has_no_percent23_entity_duplicates() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/pizza.owl");
    let ontology = load_ontology(&path).expect("load");
    let mut canon_to_ids = std::collections::BTreeMap::<String, Vec<u32>>::new();
    for (id, record) in ontology.entities().iter() {
        if record.kind.is_class() {
            let iri = ontology.resolve_iri(record.iri).unwrap();
            let canon = iri.replace("%23", "#");
            canon_to_ids.entry(canon).or_default().push(id.0);
        }
    }
    let dupes: Vec<_> = canon_to_ids
        .iter()
        .filter(|(_, ids)| ids.len() > 1)
        .collect();
    assert!(dupes.is_empty(), "duplicate class entities: {dupes:?}");
}
