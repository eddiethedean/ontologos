//! HermiT `tableau.*` engine-internal tests — ported units + deferred inventory (Tier B3).

use ontologos_alc::{
    AnnotatedEquality, BlockingValidator, BranchingPoint, DependencySetFactory, DependencySetRef,
    DescriptionGraph, DescriptionGraphId, DlClauseEvaluator, DlObject, DlPredicate, Node,
    PermanentDependencySet, RoleRef, Tableau, TupleIndex, TupleIndexRetrieval, UnionDependencySet,
    blocking_test_annotated_equalities_clauses, blocking_test_one_invalid_block_clauses,
    do_iteration, graph_merge, run_calculus, test_helpers,
};
use std::rc::Rc;

/// HermiT cases ported to Rust unit tests (catalog `covered`).
const TABLEAU_MIGRATED_IDS: &[&str] = &[
    "tableau.BlockingValidatorTest.testInvalidBlockWithAnnotatedEqualities",
    "tableau.BlockingValidatorTest.testOneInvalidBlock",
    "tableau.DLClauseEvaluationTest.testEvaluator",
    "tableau.GraphTest.testGraph1",
    "tableau.GraphTest.testGraphMerging",
    "tableau.NIRuleTest.testContentingNIs",
    "tableau.NIRuleTest.testDeterministicRuleApplication",
    "tableau.NIRuleTest.testDisjunctionDerivation",
    "tableau.NIRuleTest.testDisjunctionsInTreePart",
    "tableau.NIRuleTest.testNIAndPruning",
    "tableau.NIRuleTest.testNIDoesNotPrune",
    "tableau.NIRuleTest.testNIPrunesOneNode",
    "tableau.NIRuleTest.testNIRuleDeterministic",
    "tableau.NIRuleTest.testNondeterministicEquality",
    "tableau.NIRuleTest.testRepeatedNIApplications",
    "tableau.DependencySetTest.testDependencySet1",
    "tableau.DependencySetTest.testDependencySet2",
    "tableau.DependencySetTest.testDependencySet3",
    "tableau.TupleIndexTest.testIndex1",
    "tableau.TupleIndexTest.testIndex2",
    "tableau.TupleTableFullIndexTest.testIndex",
    "tableau.TupleTableFullIndexTest.testLotsOfData",
    "tableau.MergeTest.testMergeAndBacktrack",
];

/// HermiT `tableau.*` cases still deferred (none after Phase 1 port).
const TABLEAU_INTERNAL_IDS: &[&str] = &[];

fn neg_a() -> DlPredicate {
    DlPredicate::AtomicNegationConcept("A")
}

fn exists_neg_a() -> DlPredicate {
    DlPredicate::AtLeastConcept {
        n: 1,
        role: RoleRef::Atomic("R"),
        filler: Box::new(neg_a()),
    }
}

fn role_obj(r: &'static str) -> DlObject {
    DlObject::Predicate(DlPredicate::AtomicRole(r))
}

fn node_obj(n: &Node) -> DlObject {
    DlObject::Node(n.id())
}

#[test]
fn hermit_tableau_internal_inventory() {
    #[derive(serde::Deserialize)]
    struct Case {
        id: String,
        status: String,
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/catalog/cases.json");
    let cases: Vec<Case> =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("cases.json")).expect("parse");
    for id in TABLEAU_INTERNAL_IDS {
        let case = cases
            .iter()
            .find(|c| c.id == *id)
            .unwrap_or_else(|| panic!("missing catalog entry {id}"));
        assert_eq!(
            case.status, "internal",
            "{id} should remain internal until ported"
        );
    }
    assert_eq!(TABLEAU_INTERNAL_IDS.len(), 0);
}

#[test]
fn hermit_tableau_migrated_catalog_status() {
    #[derive(serde::Deserialize)]
    struct Case {
        id: String,
        status: String,
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/catalog/cases.json");
    let cases: Vec<Case> =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("cases.json")).expect("parse");
    for id in TABLEAU_MIGRATED_IDS {
        let case = cases
            .iter()
            .find(|c| c.id == *id)
            .unwrap_or_else(|| panic!("missing catalog entry {id}"));
        assert_eq!(case.status, "covered", "{id} should be covered after port");
    }
}

#[test]
fn hermit_merge_test_merge_and_backtrack() {
    let tableau = Tableau::new_deterministic();
    let factory = DependencySetFactory::new();
    let empty = factory.empty_set();
    let ext = tableau.extension_manager();

    let a = tableau.create_new_ni_node(empty.clone());
    let b = tableau.create_new_ni_node(empty.clone());
    let a1 = tableau.create_new_tree_node(empty.clone(), &a);
    let a2 = tableau.create_new_tree_node(empty.clone(), &a);
    let a11 = tableau.create_new_tree_node(empty.clone(), &a1);
    let a12 = tableau.create_new_tree_node(empty.clone(), &a1);

    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a,
        Some(&a1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a,
        Some(&a2),
        empty.clone(),
        false,
    );

    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &a1, empty.clone(), false);
    ext.add_concept_assertion(exists_neg_a(), &a1, empty.clone(), false);

    ext.add_concept_assertion(neg_a(), &a2, empty.clone(), false);
    ext.add_concept_assertion(DlPredicate::AtomicConcept("B"), &a2, empty.clone(), false);
    ext.add_concept_assertion(DlPredicate::AtomicConcept("C"), &a2, empty.clone(), false);
    ext.add_concept_assertion(DlPredicate::AtomicConcept("D"), &a2, empty.clone(), false);

    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a1,
        Some(&a11),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a1,
        Some(&a12),
        empty.clone(),
        false,
    );

    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &a11, empty.clone(), false);
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &a12, empty.clone(), false);

    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a1,
        Some(&b),
        empty.clone(),
        false,
    );

    let bp = BranchingPoint::new(&tableau);
    tableau.push_branching_point(&bp);

    ext.add_assertion(DlPredicate::Equality, &a1, Some(&a2), empty.clone(), false);

    assert!(ext.contains_clash());
    tableau.assert_label(
        &a2,
        &[
            DlPredicate::AtomicConcept("A"),
            DlPredicate::AtomicConcept("B"),
            DlPredicate::AtomicConcept("C"),
            DlPredicate::AtomicConcept("D"),
            neg_a(),
            exists_neg_a(),
        ],
    );

    assert!(a1.is_merged());
    assert_eq!(tableau.canonical_node(&a1), tableau.canonical_node(&a2));
    assert!(!a11.is_active());
    assert!(!a12.is_active());

    test_helpers::assert_retrieval(
        &ext.ternary_extension_table(),
        &[Some(role_obj("R")), None, None],
        &[
            vec![role_obj("R"), node_obj(&a), node_obj(&a2)],
            vec![role_obj("R"), node_obj(&a2), node_obj(&b)],
        ],
    );
    test_helpers::assert_retrieval(
        &ext.binary_extension_table(),
        &[
            Some(DlObject::Predicate(DlPredicate::AtomicConcept("A"))),
            None,
        ],
        &[vec![
            DlObject::Predicate(DlPredicate::AtomicConcept("A")),
            node_obj(&a2),
        ]],
    );

    tableau.backtrack_to(bp.level());

    assert!(!ext.contains_clash());
    tableau.assert_label(
        &a2,
        &[
            DlPredicate::AtomicConcept("B"),
            DlPredicate::AtomicConcept("C"),
            DlPredicate::AtomicConcept("D"),
            neg_a(),
        ],
    );

    assert!(!a1.is_merged());
    assert_eq!(tableau.canonical_node(&a1), a1);
    assert!(a11.is_active());
    assert!(a12.is_active());

    test_helpers::assert_retrieval(
        &ext.ternary_extension_table(),
        &[Some(role_obj("R")), None, None],
        &[
            vec![role_obj("R"), node_obj(&a), node_obj(&a1)],
            vec![role_obj("R"), node_obj(&a1), node_obj(&a11)],
            vec![role_obj("R"), node_obj(&a1), node_obj(&a12)],
            vec![role_obj("R"), node_obj(&a1), node_obj(&b)],
            vec![role_obj("R"), node_obj(&a), node_obj(&a2)],
        ],
    );
    test_helpers::assert_retrieval(
        &ext.binary_extension_table(),
        &[
            Some(DlObject::Predicate(DlPredicate::AtomicConcept("A"))),
            None,
        ],
        &[
            vec![
                DlObject::Predicate(DlPredicate::AtomicConcept("A")),
                node_obj(&a1),
            ],
            vec![
                DlObject::Predicate(DlPredicate::AtomicConcept("A")),
                node_obj(&a11),
            ],
            vec![
                DlObject::Predicate(DlPredicate::AtomicConcept("A")),
                node_obj(&a12),
            ],
        ],
    );

    ext.add_assertion(
        DlPredicate::Inequality,
        &a11,
        Some(&a12),
        empty.clone(),
        false,
    );
    test_helpers::assert_retrieval(
        &ext.ternary_extension_table(),
        &[
            Some(DlObject::Predicate(DlPredicate::Inequality)),
            None,
            None,
        ],
        &[vec![
            DlObject::Predicate(DlPredicate::Inequality),
            node_obj(&a11),
            node_obj(&a12),
        ]],
    );

    ext.add_assertion(
        DlPredicate::Equality,
        &a11,
        Some(&a12),
        empty.clone(),
        false,
    );
    assert!(ext.contains_clash());

    tableau.backtrack_to(bp.level());

    assert!(!ext.contains_clash());
    test_helpers::assert_retrieval(
        &ext.ternary_extension_table(),
        &[
            Some(DlObject::Predicate(DlPredicate::Inequality)),
            None,
            None,
        ],
        &[],
    );
}

fn assert_ds_equals(set: &PermanentDependencySet, expected: &[i32]) {
    let actual: Vec<i32> = set.branching_points().collect();
    assert_eq!(actual, expected);
}

fn perm(set: Rc<PermanentDependencySet>) -> DependencySetRef {
    DependencySetRef::Permanent(set)
}

#[test]
fn hermit_dependency_set_test_1() {
    let factory = DependencySetFactory::new();
    let mut set = factory.empty_set();
    assert_ds_equals(&set, &[]);
    assert!(set.is_empty());

    set = factory.add_branching_point(&perm(set), 32);
    assert!(!set.is_empty());
    assert_ds_equals(&set, &[32]);

    set = factory.add_branching_point(&perm(set), 0);
    assert_ds_equals(&set, &[32, 0]);

    let set2 = factory.add_branching_point(&perm(factory.empty_set()), 0);
    assert!(!Rc::ptr_eq(&set, &set2));
    let merged = factory.add_branching_point(&perm(set2.clone()), 32);
    assert!(Rc::ptr_eq(&set, &merged));

    let mid = factory.add_branching_point(&perm(set2.clone()), 15);
    let mid = factory.add_branching_point(&perm(mid), 17);
    set = factory.union_with(&perm(set), &perm(mid));
    assert_ds_equals(&set, &[32, 17, 15, 0]);

    let empty_union = factory.union_with(&perm(set.clone()), &perm(factory.empty_set()));
    assert!(Rc::ptr_eq(&set, &empty_union));
    let dup = factory.add_branching_point(&perm(empty_union.clone()), 17);
    assert!(Rc::ptr_eq(&set, &dup));

    let unchanged = factory.remove_branching_point(&perm(set.clone()), 13);
    assert!(Rc::ptr_eq(&set, &unchanged));

    set = factory.remove_branching_point(&perm(set), 17);
    assert_ds_equals(&set, &[32, 15, 0]);
    set = factory.remove_branching_point(&perm(set), 15);
    assert_ds_equals(&set, &[32, 0]);
    set = factory.remove_branching_point(&perm(set), 32);
    assert_ds_equals(&set, &[0]);
}

#[test]
fn hermit_dependency_set_test_2() {
    let factory = DependencySetFactory::new();
    let mut set1 = factory.empty_set();
    set1 = factory.add_branching_point(&perm(set1), 10);
    set1 = factory.add_branching_point(&perm(set1), 3);
    set1 = factory.add_branching_point(&perm(set1), 1);
    set1 = factory.add_branching_point(&perm(set1), 14);
    assert_ds_equals(&set1, &[14, 10, 3, 1]);

    let mut set2 = factory.empty_set();
    set2 = factory.add_branching_point(&perm(set2), 15);
    set2 = factory.add_branching_point(&perm(set2), 10);
    set2 = factory.add_branching_point(&perm(set2), 1);
    set2 = factory.add_branching_point(&perm(set2), 17);
    assert_ds_equals(&set2, &[17, 15, 10, 1]);

    let set3 = factory.union_with(&perm(set1), &perm(set2));
    assert_ds_equals(&set3, &[17, 15, 14, 10, 3, 1]);
}

#[test]
fn hermit_dependency_set_test_3() {
    let factory = DependencySetFactory::new();
    let mut set1 = factory.empty_set();
    set1 = factory.add_branching_point(&perm(set1), 10);
    set1 = factory.add_branching_point(&perm(set1), 3);
    set1 = factory.add_branching_point(&perm(set1), 1);
    assert_ds_equals(&set1, &[10, 3, 1]);

    let mut set2 = factory.empty_set();
    set2 = factory.add_branching_point(&perm(set2), 14);
    set2 = factory.add_branching_point(&perm(set2), 3);
    set2 = factory.add_branching_point(&perm(set2), 1);
    set2 = factory.add_branching_point(&perm(set2), 17);
    assert_ds_equals(&set2, &[17, 14, 3, 1]);

    let mut set3 = factory.empty_set();
    set3 = factory.add_branching_point(&perm(set3), 14);
    set3 = factory.add_branching_point(&perm(set3), 3);
    set3 = factory.add_branching_point(&perm(set3), 2);
    set3 = factory.add_branching_point(&perm(set3), 18);
    assert_ds_equals(&set3, &[18, 14, 3, 2]);

    let union = UnionDependencySet {
        constituents: vec![
            DependencySetRef::Permanent(set1),
            DependencySetRef::Permanent(set2),
            DependencySetRef::Permanent(set3),
        ],
    };
    let set4 = factory.get_permanent_union(&union);
    assert_ds_equals(&set4, &[18, 17, 14, 10, 3, 2, 1]);
}

fn assert_tuple_retrieval(index: &TupleIndex<String>, selection: &[&str], expected: &[i32]) {
    let bindings: Vec<String> = selection.iter().map(|s| (*s).to_string()).collect();
    let selection_indices: Vec<usize> = (0..bindings.len()).collect();
    let mut retrieval = TupleIndexRetrieval::new(index, &bindings, &selection_indices);
    retrieval.open();
    let mut used = vec![false; expected.len()];
    while !retrieval.after_last() {
        let tuple_index = retrieval.get_current_tuple_index();
        let found = expected.iter().enumerate().any(|(i, &exp)| {
            if tuple_index == exp && !used[i] {
                used[i] = true;
                true
            } else {
                false
            }
        });
        assert!(
            found,
            "tuple index {tuple_index} not found in expected set {expected:?}"
        );
        retrieval.next();
    }
    for (i, &u) in used.iter().enumerate() {
        assert!(
            u,
            "expected tuple index {} not found in retrieval",
            expected[i]
        );
    }
}

fn tuple3(a: &str, b: &str, c: &str) -> [String; 3] {
    [a.to_string(), b.to_string(), c.to_string()]
}

#[test]
fn hermit_tuple_index_test_1() {
    let mut index = TupleIndex::new(&[0, 1, 2]);

    assert_tuple_retrieval(&index, &[], &[]);

    index.add_tuple(&tuple3("a", "b", "c"), 1);
    assert_tuple_retrieval(&index, &["a"], &[1]);
    assert_tuple_retrieval(&index, &["a", "b"], &[1]);

    index.add_tuple(&tuple3("a", "b", "d"), 2);
    assert_tuple_retrieval(&index, &["a"], &[1, 2]);
    assert_tuple_retrieval(&index, &["a", "b"], &[1, 2]);

    index.add_tuple(&tuple3("a", "b", "c"), 3);
    assert_tuple_retrieval(&index, &["a"], &[2, 1]);
    assert_tuple_retrieval(&index, &["a", "b"], &[2, 1]);
    assert_tuple_retrieval(&index, &["a", "b", "c"], &[1]);

    index.add_tuple(&tuple3("c", "b", "d"), 4);
    assert_tuple_retrieval(&index, &[], &[2, 1, 4]);
    assert_tuple_retrieval(&index, &["a"], &[2, 1]);
    assert_tuple_retrieval(&index, &["a", "b"], &[2, 1]);
    assert_tuple_retrieval(&index, &["a", "b", "c"], &[1]);
    assert_tuple_retrieval(&index, &["f"], &[]);

    index.remove_tuple(&tuple3("a", "b", "d"));
    assert_tuple_retrieval(&index, &[], &[1, 4]);

    index.remove_tuple(&tuple3("a", "b", "c"));
    assert_tuple_retrieval(&index, &[], &[4]);

    index.remove_tuple(&tuple3("c", "b", "d"));
    assert_tuple_retrieval(&index, &[], &[]);
}

#[test]
fn hermit_tuple_index_test_2() {
    let mut index = TupleIndex::new(&[0, 1, 2]);
    let mut tuples: Vec<[String; 3]> = Vec::with_capacity(10_000);
    let mut tuple_indexes = Vec::with_capacity(10_000);

    for i in 0..10_000 {
        tuples.push([(i % 300).to_string(), (i % 3000).to_string(), i.to_string()]);
        tuple_indexes.push(i);
    }

    for (i, tuple) in tuples.iter().enumerate() {
        index.add_tuple(tuple, i as i32);
    }
    assert_tuple_retrieval(&index, &[], &tuple_indexes);

    for (i, tuple) in tuples.iter().enumerate() {
        assert_eq!(index.remove_tuple(tuple), i as i32);
    }
    assert_tuple_retrieval(&index, &[], &[]);
}

#[test]
fn hermit_tuple_table_full_index_test_1() {
    use ontologos_alc::TupleTableFullIndex;

    struct Harness {
        index: TupleTableFullIndex<String>,
    }

    impl Harness {
        fn new() -> Self {
            Self {
                index: TupleTableFullIndex::new(2, 2),
            }
        }

        fn add(&mut self, a: &str, b: &str) -> i32 {
            let tuple = [a.to_string(), b.to_string()];
            let tentative = self.index.tuple_table().first_free_tuple_index();
            let result = self.index.add_tuple(&tuple, tentative);
            if result == tentative {
                self.index.tuple_table_mut().add_tuple(&tuple);
            }
            result
        }

        fn get(&self, a: &str, b: &str) -> i32 {
            self.index.get_tuple_index(&[a.to_string(), b.to_string()])
        }
    }

    let mut h = Harness::new();
    assert_eq!(h.add("a", "b"), 0);
    assert_eq!(h.add("b", "c"), 1);
    assert_eq!(h.add("c", "d"), 2);
    assert_eq!(h.add("a", "b"), 0);
    assert_eq!(h.get("a", "b"), 0);
    assert_eq!(h.get("b", "c"), 1);
    assert_eq!(h.get("c", "d"), 2);
    assert!(h.index.remove_tuple(1));
    assert_eq!(h.get("a", "b"), 0);
    assert_eq!(h.get("b", "c"), -1);
    assert_eq!(h.get("c", "d"), 2);
    assert_eq!(h.add("e", "f"), 3);
    assert_eq!(h.get("e", "f"), 3);
    assert_eq!(h.add("g", "h"), 4);
    assert_eq!(h.get("g", "h"), 4);
}

#[test]
fn hermit_tuple_table_full_index_test_2() {
    use ontologos_alc::TupleTableFullIndex;

    let mut index = TupleTableFullIndex::<String>::new(2, 2);
    let tuples: Vec<[String; 2]> = (0..40_000)
        .map(|i| [format!("a{i}"), format!("b{i}")])
        .collect();
    for (i, tuple) in tuples.iter().enumerate() {
        assert_eq!(index.add_tuple(tuple, i as i32), i as i32);
        index.tuple_table_mut().add_tuple(tuple);
    }
    for (i, tuple) in tuples.iter().enumerate() {
        assert_eq!(index.get_tuple_index(tuple), i as i32);
    }
    assert_eq!(index.get_tuple_index(&["e".into(), "f".into()]), -1);
}

fn empty() -> Rc<PermanentDependencySet> {
    DependencySetFactory::new().empty_set()
}

fn eq_one_r_a() -> AnnotatedEquality {
    AnnotatedEquality::new(1, "R", "A")
}

fn eq_two_r_a() -> AnnotatedEquality {
    AnnotatedEquality::new(2, "R", "A")
}

fn eq_one_s_a() -> AnnotatedEquality {
    AnnotatedEquality::new(1, "S", "A")
}

fn ni_fixture_chain(
    tableau: &Tableau,
    ext: &ontologos_alc::ExtensionManagerRef,
    empty: &Rc<PermanentDependencySet>,
) -> (Node, Node, Node, Node, Node) {
    let a = tableau.create_new_ni_node(empty.clone());
    let b = tableau.create_new_ni_node(empty.clone());
    let b1 = tableau.create_new_tree_node(empty.clone(), &b);
    let b11 = tableau.create_new_tree_node(empty.clone(), &b1);
    let b111 = tableau.create_new_tree_node(empty.clone(), &b11);
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &b,
        Some(&b1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &b1,
        Some(&b11),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &b11,
        Some(&b111),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a,
        Some(&b11),
        empty.clone(),
        false,
    );
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &b11, empty.clone(), false);
    (a, b, b1, b11, b111)
}

#[test]
fn hermit_ni_rule_test_deterministic() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let ni = tableau.ni_manager();
    let (a, _b, b1, b11, b111) = ni_fixture_chain(&tableau, &ext, &empty);
    assert_eq!(ni.pending_annotated_equalities(), 0);
    ni.add_annotated_equality(eq_one_r_a(), &b11, &b11, &a, empty.clone());
    assert_eq!(ni.pending_annotated_equalities(), 0);
    let new_root = ni.root_node_for(&a, eq_one_r_a(), 1).expect("new root");
    assert!(new_root.is_active());
    assert!(!b11.is_active());
    assert_eq!(
        tableau.canonical_node(&b11),
        tableau.canonical_node(&new_root)
    );
    assert!(!b111.is_active());
    assert!(!ext.contains_assertion("S", &b11, &b111));
    assert!(ext.contains_assertion("S", &b1, &new_root));
    assert!(ext.contains_assertion("R", &a, &new_root));
}

#[test]
fn hermit_ni_rule_test_ni_prunes_one_node() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let ni = tableau.ni_manager();
    let (a, _b, b1, b11, b111) = ni_fixture_chain(&tableau, &ext, &empty);
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a,
        Some(&b1),
        empty.clone(),
        false,
    );
    ni.add_annotated_equality(eq_one_r_a(), &b1, &b11, &a, empty.clone());
    let new_root = ni.root_node_for(&a, eq_one_r_a(), 1).expect("new root");
    assert!(b1.is_merged());
    assert_eq!(
        tableau.canonical_node(&b1),
        tableau.canonical_node(&new_root)
    );
    assert!(b11.is_pruned());
    assert!(b111.is_pruned());
    assert!(!ext.has_concept("A", &new_root));
    assert!(ext.contains_assertion("R", &a, &new_root));
}

#[test]
fn hermit_ni_rule_test_ni_does_not_prune() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let ni = tableau.ni_manager();
    let a = tableau.create_new_ni_node(empty.clone());
    let b = tableau.create_new_ni_node(empty.clone());
    let b1 = tableau.create_new_tree_node(empty.clone(), &b);
    let b11 = tableau.create_new_tree_node(empty.clone(), &b1);
    let c = tableau.create_new_ni_node(empty.clone());
    let c1 = tableau.create_new_tree_node(empty.clone(), &c);
    let c11 = tableau.create_new_tree_node(empty.clone(), &c1);
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &b,
        Some(&b1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &b1,
        Some(&b11),
        empty.clone(),
        false,
    );
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &b1, empty.clone(), false);
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a,
        Some(&b1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("T"),
        &c,
        Some(&c1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("T"),
        &c1,
        Some(&c11),
        empty.clone(),
        false,
    );
    ext.add_concept_assertion(DlPredicate::AtomicConcept("B"), &c1, empty.clone(), false);
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a,
        Some(&c1),
        empty.clone(),
        false,
    );
    ni.add_annotated_equality(eq_one_r_a(), &b1, &c1, &a, empty.clone());
    let new_root = ni.root_node_for(&a, eq_one_r_a(), 1).expect("new root");
    assert!(b1.is_merged());
    assert!(c11.is_pruned());
    assert!(ext.has_concept("A", &new_root));
    assert!(ext.has_concept("B", &new_root));
}

#[test]
fn hermit_ni_rule_test_deterministic_rule_application() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let a = tableau.create_new_ni_node(empty.clone());
    let a1 = tableau.create_new_tree_node(empty.clone(), &a);
    let b = tableau.create_new_ni_node(empty.clone());
    let b1 = tableau.create_new_tree_node(empty.clone(), &b);
    let c = tableau.create_new_ni_node(empty.clone());
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &a,
        Some(&a1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &c,
        Some(&a1),
        empty.clone(),
        false,
    );
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &a1, empty.clone(), false);
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &b,
        Some(&b1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &c,
        Some(&b1),
        empty.clone(),
        false,
    );
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &b1, empty.clone(), false);
    ext.add_concept_assertion(
        DlPredicate::AtomicConcept("AT_MOST_ONE_R_A"),
        &c,
        empty.clone(),
        false,
    );
    let ni = tableau.ni_manager();
    assert!(ni.add_annotated_equality(eq_one_r_a(), &a1, &b1, &c, empty.clone()));
    let c_n1 = ni.root_node_for(&c, eq_one_r_a(), 1).expect("c_n1");
    assert_eq!(tableau.canonical_node(&a1), tableau.canonical_node(&c_n1));
    assert_eq!(tableau.canonical_node(&b1), tableau.canonical_node(&c_n1));
}

#[test]
fn hermit_ni_rule_test_ni_and_pruning() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let ni = tableau.ni_manager();
    let a = tableau.create_new_ni_node(empty.clone());
    let a1 = tableau.create_new_tree_node(empty.clone(), &a);
    let b = tableau.create_new_ni_node(empty.clone());
    let b1 = tableau.create_new_tree_node(empty.clone(), &b);
    let b11 = tableau.create_new_tree_node(empty.clone(), &b1);
    let c = tableau.create_new_ni_node(empty.clone());
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &a,
        Some(&a1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &c,
        Some(&a1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("T"),
        &b,
        Some(&b1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("T"),
        &b1,
        Some(&b11),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &c,
        Some(&b11),
        empty.clone(),
        false,
    );
    ni.add_annotated_equality(eq_two_r_a(), &a1, &b11, &c, empty.clone());
    ext.add_assertion(DlPredicate::Equality, &b1, Some(&c), empty.clone(), false);
    assert!(b11.is_pruned());
    assert!(do_iteration(&tableau, &[]));
    assert!(a1.is_active());
    assert_eq!(ni.pending_annotated_equalities(), 1);
}

#[test]
fn hermit_dl_clause_evaluation_test_evaluator() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let a = tableau.create_new_ni_node(empty.clone());
    let b = tableau.create_new_ni_node(empty.clone());
    let c = tableau.create_new_ni_node(empty.clone());
    let d = tableau.create_new_ni_node(empty.clone());
    let e = tableau.create_new_ni_node(empty.clone());
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a,
        Some(&b),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a,
        Some(&c),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &b,
        Some(&d),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("T"),
        &e,
        Some(&e),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("T"),
        &c,
        Some(&d),
        empty.clone(),
        false,
    );
    let evaluators = [DlClauseEvaluator::new(
        ontologos_alc::dl_clause_evaluation_test_clause(),
    )];
    assert!(run_calculus(&tableau, &evaluators));
    test_helpers::assert_retrieval(
        &ext.ternary_extension_table(),
        &[
            Some(DlObject::Predicate(DlPredicate::AtomicRole("U"))),
            None,
            None,
        ],
        &[test_helpers::t(vec![
            DlObject::Predicate(DlPredicate::AtomicRole("U")),
            DlObject::Node(d.id()),
            DlObject::Node(e.id()),
        ])],
    );
}

#[test]
fn hermit_graph_test_graph_merging() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let graph = DescriptionGraph::test_graph(
        1,
        vec!["A", "B", "C"],
        vec![
            DescriptionGraph::edge("R", 0, 1),
            DescriptionGraph::edge("R", 1, 2),
        ],
        vec!["A", "B", "C"],
    );
    let n1 = tableau.create_new_ni_node(empty.clone());
    let n2 = tableau.create_new_ni_node(empty.clone());
    let n3 = tableau.create_new_ni_node(empty.clone());
    let n4 = tableau.create_new_ni_node(empty.clone());
    let n5 = tableau.create_new_ni_node(empty.clone());
    let n6 = tableau.create_new_ni_node(empty.clone());
    ext.add_quaternary_tuple(graph.id(), &n1, &n2, &n3, empty.clone(), false);
    ext.add_quaternary_tuple(graph.id(), &n4, &n5, &n6, empty.clone(), false);
    ext.add_concept_assertion(DlPredicate::AtomicConcept("R"), &n1, empty.clone(), false);
    ext.add_concept_assertion(DlPredicate::AtomicConcept("S"), &n6, empty.clone(), false);
    let n7 = tableau.create_new_ni_node(empty.clone());
    ext.add_quaternary_tuple(graph.id(), &n1, &n7, &n6, empty.clone(), false);
    assert!(ext.contains_quaternary_tuple(graph.id(), &n1, &n2, &n3));
    assert!(tableau.saturate_description_graphs());
    graph_merge::assert_graph_merging_canonicals(&tableau, &n1, &n2, &n3, &n4, &n5, &n6, &n7);
    assert!(ext.contains_quaternary_tuple(graph.id(), &n1, &n7, &n6));
}

#[test]
fn hermit_graph_test_graph1_fixture() {
    let graph = DescriptionGraph::test_graph(
        2,
        vec!["A", "B", "C", "A"],
        vec![
            DescriptionGraph::edge("R", 0, 1),
            DescriptionGraph::edge("R", 3, 2),
        ],
        vec!["A"],
    );
    assert_eq!(graph.number_of_vertices(), 4);
    assert_eq!(graph.id(), DescriptionGraphId(2));
}

#[test]
fn hermit_ni_rule_test_nondeterministic_equality() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let ni = tableau.ni_manager();
    let (a, _b, _b1, b11, _b111) = ni_fixture_chain(&tableau, &ext, &empty);
    ni.add_annotated_equality(eq_two_r_a(), &b11, &b11, &a, empty.clone());
    assert_eq!(ni.pending_annotated_equalities(), 1);
    assert!(ni.process_annotated_equalities());
    let new_root1 = ni.root_node_for(&a, eq_two_r_a(), 1).expect("root1");
    assert!(!b11.is_active());
    assert_eq!(
        tableau.canonical_node(&b11),
        tableau.canonical_node(&new_root1)
    );
}

#[test]
fn hermit_ni_rule_test_repeated_ni_applications() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let ni = tableau.ni_manager();
    let a = tableau.create_new_ni_node(empty.clone());
    let b = tableau.create_new_ni_node(empty.clone());
    let b1 = tableau.create_new_tree_node(empty.clone(), &b);
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &b,
        Some(&b1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a,
        Some(&b1),
        empty.clone(),
        false,
    );
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &b1, empty.clone(), false);
    ni.add_annotated_equality(eq_two_r_a(), &b1, &b1, &a, empty.clone());
    assert!(ni.process_annotated_equalities());
    let a_n1 = ni.root_node_for(&a, eq_two_r_a(), 1).expect("a_n1");
    assert_eq!(tableau.canonical_node(&b1), tableau.canonical_node(&a_n1));
}

#[test]
fn hermit_ni_rule_test_contenting_nis() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let ni = tableau.ni_manager();
    let a = tableau.create_new_ni_node(empty.clone());
    let b = tableau.create_new_ni_node(empty.clone());
    let c = tableau.create_new_ni_node(empty.clone());
    let c1 = tableau.create_new_tree_node(empty.clone(), &c);
    let d = tableau.create_new_ni_node(empty.clone());
    let d1 = tableau.create_new_tree_node(empty.clone(), &d);
    ext.add_assertion(
        DlPredicate::AtomicRole("T"),
        &c,
        Some(&c1),
        empty.clone(),
        false,
    );
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &c1, empty.clone(), false);
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &c1,
        Some(&a),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &c1,
        Some(&b),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("T"),
        &d,
        Some(&d1),
        empty.clone(),
        false,
    );
    ext.add_concept_assertion(DlPredicate::AtomicConcept("B"), &d1, empty.clone(), false);
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &d1,
        Some(&a),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &d1,
        Some(&b),
        empty.clone(),
        false,
    );
    ni.add_annotated_equality(eq_two_r_a(), &c1, &d1, &a, empty.clone());
    assert_eq!(ni.pending_annotated_equalities(), 1);
    ni.add_annotated_equality(eq_one_s_a(), &d1, &d1, &b, empty.clone());
    let b_n1 = ni.root_node_for(&b, eq_one_s_a(), 1).expect("b_n1");
    assert_eq!(tableau.canonical_node(&d1), tableau.canonical_node(&b_n1));
    assert!(ni.process_annotated_equalities());
    assert_eq!(tableau.canonical_node(&c1), tableau.canonical_node(&b_n1));
}

#[test]
fn hermit_ni_rule_test_disjunction_derivation() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let ni = tableau.ni_manager();
    let a = tableau.create_new_ni_node(empty.clone());
    let a1 = tableau.create_new_tree_node(empty.clone(), &a);
    let b = tableau.create_new_ni_node(empty.clone());
    let b1 = tableau.create_new_tree_node(empty.clone(), &b);
    let c = tableau.create_new_ni_node(empty.clone());
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &a,
        Some(&a1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &c,
        Some(&a1),
        empty.clone(),
        false,
    );
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &a1, empty.clone(), false);
    ext.add_assertion(
        DlPredicate::AtomicRole("S"),
        &b,
        Some(&b1),
        empty.clone(),
        false,
    );
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &c,
        Some(&b1),
        empty.clone(),
        false,
    );
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &b1, empty.clone(), false);
    ext.add_concept_assertion(
        DlPredicate::AtomicConcept("AT_MOST_TWO_R_A"),
        &c,
        empty.clone(),
        false,
    );
    ni.add_annotated_equality(eq_two_r_a(), &a1, &b1, &c, empty.clone());
    assert!(ni.process_annotated_equalities());
    let c_n1 = ni.root_node_for(&c, eq_two_r_a(), 1).expect("c_n1");
    assert_eq!(tableau.canonical_node(&a1), tableau.canonical_node(&c_n1));
}

#[test]
fn hermit_ni_rule_test_disjunctions_in_tree_part() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let a = tableau.create_new_ni_node(empty.clone());
    let a1 = tableau.create_new_tree_node(empty.clone(), &a);
    let a11 = tableau.create_new_tree_node(empty.clone(), &a1);
    let a12 = tableau.create_new_tree_node(empty.clone(), &a1);
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a1,
        Some(&a11),
        empty.clone(),
        false,
    );
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &a11, empty.clone(), false);
    ext.add_assertion(
        DlPredicate::AtomicRole("R"),
        &a1,
        Some(&a12),
        empty.clone(),
        false,
    );
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &a12, empty.clone(), false);
    ext.add_concept_assertion(
        DlPredicate::AtomicConcept("AT_MOST_TWO_R_A"),
        &a1,
        empty.clone(),
        false,
    );
    assert!(do_iteration(&tableau, &[]));
    assert_eq!(tableau.ni_manager().pending_annotated_equalities(), 0);
}

#[test]
fn hermit_blocking_validator_test_one_invalid_block() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let a = tableau.create_new_ni_node(empty.clone());
    let a1 = tableau.create_new_tree_node(empty.clone(), &a);
    let a2 = tableau.create_new_tree_node(empty.clone(), &a);
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &a1, empty.clone(), true);
    ext.add_concept_assertion(DlPredicate::AtomicConcept("A"), &a2, empty.clone(), true);
    a2.set_directly_blocked(a1.id(), &ext);
    assert!(a2.is_directly_blocked(&ext));
    let validator = BlockingValidator::new(blocking_test_one_invalid_block_clauses());
    assert!(validator.is_block_valid(&ext, &a2));
}

#[test]
fn hermit_blocking_validator_test_invalid_block_with_annotated_equalities() {
    let tableau = Tableau::new_deterministic();
    let ext = tableau.extension_manager();
    let empty = empty();
    let a = tableau.create_new_ni_node(empty.clone());
    let a1 = tableau.create_new_tree_node(empty.clone(), &a);
    let a2 = tableau.create_new_tree_node(empty.clone(), &a);
    ext.add_concept_assertion(DlPredicate::AtomicConcept("B"), &a1, empty.clone(), true);
    ext.add_concept_assertion(DlPredicate::AtomicConcept("B"), &a2, empty.clone(), true);
    ext.add_concept_assertion(DlPredicate::AtomicConcept("C"), &a, empty.clone(), false);
    a2.set_directly_blocked(a1.id(), &ext);
    let validator = BlockingValidator::new(blocking_test_annotated_equalities_clauses());
    assert!(!validator.is_block_valid(&ext, &a2));
}
