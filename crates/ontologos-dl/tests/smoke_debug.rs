use ontologos_alc::is_consistent as alc_consistent;
use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn ax(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms")
        .join(name)
}

#[test]
fn debug_bottom_wg() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/New-2DFeature-2DBottomObjectProperty-2D001/premise.rdf",
    );
    let ont = load_ontology(&path).unwrap();
    eprintln!("entities {}", ont.entity_count());
    eprintln!("dl axioms {}", ont.dl().axiom_count());
    eprintln!("core axioms {}", ont.axiom_count());
    for (_, ax) in ont.axioms().iter() {
        eprintln!("  core: {ax:?}");
    }
    for ax in ont.dl().axioms() {
        eprintln!("  dl: {ax:?}");
    }
    for (id, ce) in ont.dl().expressions() {
        eprintln!("  CeId({}): {ce:?}", id.0);
    }
    eprintln!("consistent={:?}", is_consistent(&ont));
}

#[test]
fn smoke_kb_consistency() {
    for (name, ofn) in [
        (
            "nominals3",
            "hermit_reasoner_reasonertest_testnominals3.ofn",
        ),
        (
            "exists_self2",
            "hermit_reasoner_reasonertest_testexistsself2.ofn",
        ),
    ] {
        let ont = load_ontology(&ax(ofn)).unwrap();
        let alc = alc_consistent(&ont);
        let full = is_consistent(&ont);
        eprintln!("{name}: alc={alc:?} dl={full:?}");
    }
}

#[test]
fn debug_domain_cond() {
    let prem = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Drdfs-2Ddomain-2Dcond/premise.rdf");
    let conc = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Drdfs-2Ddomain-2Dcond/conclusion.rdf",
    );
    for (label, path) in [("prem", &prem), ("conc", &conc)] {
        let ont = load_ontology(path).unwrap();
        eprintln!(
            "{label} dl={} core={}",
            ont.dl().axiom_count(),
            ont.axiom_count()
        );
        for (id, ce) in ont.dl().expressions() {
            eprintln!("  CeId({}): {ce:?}", id.0);
        }
        for (id, rec) in ont.entities().iter() {
            eprintln!("  ent {:?} {:?}", id, ont.resolve_iri(rec.iri));
        }
        for ax in ont.dl().axioms() {
            eprintln!("  dl {ax:?}");
        }
        for (_, ax) in ont.axioms().iter() {
            eprintln!("  core {ax:?}");
        }
    }
}

#[test]
fn debug_bool_intersection() {
    use ontologos_parser::load_ontology;
    let prem = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Dbool-2Dintersection-2Dinst-2Dcomp/premise.rdf");
    let conc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/Rdfbased-2Dsem-2Dbool-2Dintersection-2Dinst-2Dcomp/conclusion.rdf");
    for (label, path) in [("prem", &prem), ("conc", &conc)] {
        let ont = load_ontology(path).unwrap();
        eprintln!(
            "{label} dl={} core={}",
            ont.dl().axiom_count(),
            ont.axiom_count()
        );
        for (id, ce) in ont.dl().expressions() {
            eprintln!("  CeId({}): {ce:?}", id.0);
        }
        for ax in ont.dl().axioms() {
            eprintln!("  dl {ax:?}");
        }
        for (_, ax) in ont.axioms().iter() {
            eprintln!("  core {ax:?}");
        }
    }
}
