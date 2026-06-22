//! HermiT-style DL hyperresolution clausification (structural regression subset).

use std::collections::HashMap;

use ontologos_core::{
    Axiom, CeId, ClassExpr, DataExpr, DeId, DlAxiom, EntityId, EntityKind, Ontology,
};

use crate::hyperclause::{
    abbrev_entity, abbrev_role, concept_name, data_range_fmt, negate_data_range, DataRangeFmt,
    HyperAtom, HyperClause, HyperClauseSet, Term,
};
use crate::Error;

const VAR_X: &str = "X";
const OWL_THING: &str = "http://www.w3.org/2002/07/owl#Thing";

/// Clausify an ontology into HermiT DL clause strings (normalization + hyperresolution).
pub fn clausify_hyper(ontology: &mut Ontology) -> Result<Vec<String>, Error> {
    let mut clausifier = HyperClausifier::new();
    clausifier.run(ontology)?;
    Ok(clausifier.set.to_strings())
}

struct HyperClausifier {
    set: HyperClauseSet,
    def_index: u32,
    defdata_index: u32,
    defdata_names: HashMap<DeId, String>,
}

impl HyperClausifier {
    fn new() -> Self {
        Self {
            set: HyperClauseSet::new(),
            def_index: 0,
            defdata_index: 0,
            defdata_names: HashMap::new(),
        }
    }

    fn run(&mut self, ontology: &mut Ontology) -> Result<(), Error> {
        let _ = ontology.dl_mut().intern_ce(ClassExpr::Top);
        let _ = ontology.dl_mut().intern_ce(ClassExpr::Bottom);

        let flat_axioms: Vec<Axiom> = ontology.axioms().iter().map(|(_, a)| a.clone()).collect();
        let dl_axioms: Vec<DlAxiom> = ontology.dl().axioms().cloned().collect();

        for axiom in &flat_axioms {
            match axiom {
                Axiom::AsymmetricObjectProperty(prop) => {
                    self.emit_asymmetric(ontology, *prop);
                }
                Axiom::SubObjectPropertyOf {
                    sub_property,
                    super_property,
                } => {
                    self.emit_role_subsumption(ontology, *sub_property, *super_property);
                }
                Axiom::ObjectPropertyAssertion {
                    subject,
                    property,
                    object,
                } => {
                    self.emit_role_fact(ontology, *property, *subject, *object);
                }
                _ => {}
            }
        }

        for axiom in dl_axioms {
            self.process_dl_axiom(ontology, axiom)?;
        }

        Ok(())
    }

    fn process_dl_axiom(&mut self, ontology: &mut Ontology, axiom: DlAxiom) -> Result<(), Error> {
        match axiom {
            DlAxiom::SubClassOf { sub, sup } => {
                self.process_subclass(ontology, sub, sup)?;
            }
            DlAxiom::SubObjectPropertyOf { sub, sup } => {
                if let (ontologos_core::RoleExpr::Atomic(sub_id), ontologos_core::RoleExpr::Atomic(sup_id)) =
                    (&sub, &sup)
                {
                    self.emit_role_subsumption(ontology, *sub_id, *sup_id);
                }
            }
            DlAxiom::ObjectPropertyAssertion {
                subject,
                property: ontologos_core::RoleExpr::Atomic(prop),
                object,
            } => {
                self.emit_role_fact(ontology, prop, subject, object);
            }
            DlAxiom::ClassAssertion { individual, class } => {
                self.process_class_assertion(ontology, individual, class)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn process_subclass(
        &mut self,
        ontology: &mut Ontology,
        sub: CeId,
        sup: CeId,
    ) -> Result<(), Error> {
        let sub_expr = ontology.dl().ce(sub).cloned();
        let sup_expr = ontology.dl().ce(sup).cloned();

        if let (Some(ClassExpr::Atomic(sub_id)), Some(ClassExpr::DataAll { property, range })) =
            (&sub_expr, &sup_expr)
        {
            if !is_thing_entity(ontology, *sub_id) {
                self.emit_data_all_forward(ontology, *sub_id, *property, *range);
                return Ok(());
            }
        }

        if is_thing_ce(ontology, sub) {
            if let Some(ClassExpr::DataAll { property, range }) = sup_expr {
                self.emit_data_all_on_thing(ontology, property, range)?;
                return Ok(());
            }
        }

        if let (Some(ClassExpr::DataAll { property, range }), Some(ClassExpr::Atomic(sup_id))) =
            (&sub_expr, &sup_expr)
        {
            self.emit_data_all_inverse(ontology, *sup_id, *property, *range);
            return Ok(());
        }

        if let (Some(ClassExpr::Atomic(sub_id)), Some(sup_data)) = (&sub_expr, &sup_expr) {
            match sup_data {
                ClassExpr::DataSome { property, range } => {
                    self.emit_data_min_forward(ontology, *sub_id, 1, *property, *range);
                    return Ok(());
                }
                ClassExpr::DataMinCardinality { n, property, range } => {
                    let range = range.unwrap_or_else(|| literal_top(ontology));
                    self.emit_data_min_forward(ontology, *sub_id, *n, *property, range);
                    return Ok(());
                }
                ClassExpr::DataMaxCardinality { n, property, range } => {
                    let range = range.unwrap_or_else(|| literal_top(ontology));
                    self.emit_data_max_forward(ontology, *sub_id, *n, *property, range);
                    return Ok(());
                }
                ClassExpr::DataExactCardinality { n, property, range } => {
                    let range = range.unwrap_or_else(|| literal_top(ontology));
                    self.emit_data_max_forward(ontology, *sub_id, *n, *property, range);
                    self.emit_data_min_forward(ontology, *sub_id, *n, *property, range);
                    return Ok(());
                }
                ClassExpr::DataHasValue { property, value } => {
                    let one_of = literal_one_of_de(ontology, value);
                    self.emit_data_min_forward(ontology, *sub_id, 1, *property, one_of);
                    return Ok(());
                }
                _ => {}
            }
        }

        if let (Some(sub_data), Some(ClassExpr::Atomic(sup_id))) = (&sub_expr, &sup_expr) {
            match sub_data {
                ClassExpr::DataSome { property, range } => {
                    self.emit_data_some_inverse(ontology, *sup_id, *property, *range);
                    return Ok(());
                }
                ClassExpr::DataMinCardinality { n, property, range } => {
                    let range = range.unwrap_or_else(|| literal_top(ontology));
                    self.emit_data_min_inverse(ontology, *sup_id, *n, *property, range);
                    return Ok(());
                }
                ClassExpr::DataMaxCardinality { n, property, range } => {
                    let range = range.unwrap_or_else(|| literal_top(ontology));
                    self.emit_data_max_inverse(ontology, *sup_id, *n, *property, range);
                    return Ok(());
                }
                ClassExpr::DataExactCardinality { n, property, range } => {
                    let range = range.unwrap_or_else(|| literal_top(ontology));
                    self.emit_data_exact_inverse(ontology, *sup_id, *n, *property, range);
                    return Ok(());
                }
                ClassExpr::DataHasValue { property, value } => {
                    let one_of = literal_one_of_de(ontology, value);
                    self.emit_data_has_value_inverse(ontology, *sup_id, *property, one_of);
                    return Ok(());
                }
                ClassExpr::DataAll { property, range } => {
                    self.emit_data_all_inverse(ontology, *sup_id, *property, *range);
                    return Ok(());
                }
                _ => {}
            }
        }

        if let (Some(sub_name), Some(sup_name)) =
            (concept_name(ontology, sub), concept_name(ontology, sup))
        {
            self.set.push_clause(HyperClause {
                head: vec![HyperAtom::Concept {
                    name: sup_name,
                    term: Term::Var(VAR_X.into()),
                }],
                body: vec![HyperAtom::Concept {
                    name: sub_name,
                    term: Term::Var(VAR_X.into()),
                }],
            });
        }

        Ok(())
    }

    fn process_class_assertion(
        &mut self,
        ontology: &mut Ontology,
        individual: EntityId,
        class: CeId,
    ) -> Result<(), Error> {
        let expr = ontology.dl().ce(class).cloned();
        let Some(expr) = expr else {
            return Ok(());
        };

        if let ClassExpr::DataMinCardinality { n, property, range } = expr {
            let def = self.fresh_def_name();
            let ind = abbrev_entity(ontology, individual);
            self.set.push_fact(HyperAtom::Concept {
                name: def.clone(),
                term: Term::Ind(ind),
            });
            let range = range.unwrap_or_else(|| literal_top(ontology));
            self.emit_data_min_forward_named(ontology, &def, n, property, range);
        }

        Ok(())
    }

    fn emit_data_all_forward(
        &mut self,
        ontology: &Ontology,
        sub_id: EntityId,
        property: EntityId,
        range: DeId,
    ) {
        let sub = abbrev_entity(ontology, sub_id);
        let role = abbrev_role(ontology, property);
        let simplified = simplify_data_range(ontology, range);
        let expr = ontology.dl().de(simplified).cloned();
        let Some(expr) = expr else {
            return;
        };
        match expr {
            DataExpr::Not(inner) => {
                let Some(inner_fmt) = self.resolve_data_range(ontology, inner) else {
                    return;
                };
                self.set.push_clause(HyperClause {
                    head: vec![HyperAtom::DataRange {
                        range: negate_data_range(&inner_fmt),
                        term: Term::Var("Y".into()),
                    }],
                    body: vec![
                        HyperAtom::Concept {
                            name: sub,
                            term: Term::Var(VAR_X.into()),
                        },
                        HyperAtom::Role {
                            role,
                            subject: Term::Var(VAR_X.into()),
                            object: Term::Var("Y".into()),
                        },
                    ],
                });
            }
            _ => {
                let Some(range_fmt) = self.resolve_data_range(ontology, simplified) else {
                    return;
                };
                self.set.push_clause(HyperClause {
                    head: vec![HyperAtom::DataRange {
                        range: range_fmt,
                        term: Term::Var("Y".into()),
                    }],
                    body: vec![
                        HyperAtom::Concept {
                            name: sub,
                            term: Term::Var(VAR_X.into()),
                        },
                        HyperAtom::Role {
                            role,
                            subject: Term::Var(VAR_X.into()),
                            object: Term::Var("Y".into()),
                        },
                    ],
                });
            }
        }
    }

    fn emit_data_all_inverse(
        &mut self,
        ontology: &mut Ontology,
        sup_id: EntityId,
        property: EntityId,
        range: DeId,
    ) {
        let sup = abbrev_entity(ontology, sup_id);
        let role = abbrev_role(ontology, property);
        let simplified = simplify_data_range(ontology, range);
        if is_complex_data_range(ontology, simplified) {
            let defdata = self.fresh_defdata_name(ontology, simplified);
            if let DataExpr::Not(inner) = ontology.dl().de(simplified).cloned().unwrap_or(DataExpr::Top)
            {
                if let DataExpr::And(ids) = ontology.dl().de(inner).cloned().unwrap_or(DataExpr::Top) {
                    for id in ids {
                        if let Some(dr) = self.resolve_data_range(ontology, id) {
                            self.set.push_clause(HyperClause {
                                head: vec![HyperAtom::DataRange {
                                    range: dr,
                                    term: Term::Var(VAR_X.into()),
                                }],
                                body: vec![HyperAtom::DataRange {
                                    range: DataRangeFmt::Internal(defdata.clone()),
                                    term: Term::Var(VAR_X.into()),
                                }],
                            });
                        }
                    }
                } else {
                    let _ = self.emit_data_range_inclusions_expr(ontology, simplified, &defdata);
                }
            } else {
                let _ = self.emit_data_range_inclusions_expr(ontology, simplified, &defdata);
            }
            self.set.push_clause(HyperClause {
                head: vec![
                    HyperAtom::Concept {
                        name: sup,
                        term: Term::Var(VAR_X.into()),
                    },
                    HyperAtom::AtLeastData {
                        n: 1,
                        role,
                        range: DataRangeFmt::Internal(defdata),
                        term: Term::Var(VAR_X.into()),
                    },
                ],
                body: vec![HyperAtom::Concept {
                    name: "owl:Thing".into(),
                    term: Term::Var(VAR_X.into()),
                }],
            });
            return;
        }
        let Some(range_fmt) = self.resolve_data_range(ontology, simplified) else {
            return;
        };
        let neg = negate_data_range(&range_fmt);
        self.set.push_clause(HyperClause {
            head: vec![
                HyperAtom::Concept {
                    name: sup,
                    term: Term::Var(VAR_X.into()),
                },
                HyperAtom::AtLeastData {
                    n: 1,
                    role,
                    range: neg,
                    term: Term::Var(VAR_X.into()),
                },
            ],
            body: vec![HyperAtom::Concept {
                name: "owl:Thing".into(),
                term: Term::Var(VAR_X.into()),
            }],
        });
    }

    fn emit_data_all_on_thing(
        &mut self,
        ontology: &mut Ontology,
        property: EntityId,
        range: DeId,
    ) -> Result<(), Error> {
        let role = abbrev_role(ontology, property);
        let defdata = self.fresh_defdata_name(ontology, range);
        self.set.push_clause(HyperClause {
            head: vec![HyperAtom::DataRange {
                range: DataRangeFmt::Internal(defdata.clone()),
                term: Term::Var("Y".into()),
            }],
            body: vec![HyperAtom::Role {
                role,
                subject: Term::Var(VAR_X.into()),
                object: Term::Var("Y".into()),
            }],
        });
        self.emit_data_range_inclusions(ontology, range, &defdata)
    }

    fn emit_data_range_inclusions(
        &mut self,
        ontology: &Ontology,
        range: DeId,
        defdata: &str,
    ) -> Result<(), Error> {
        let simplified = simplify_data_range(ontology, range);
        self.emit_data_range_inclusions_expr(ontology, simplified, defdata)
    }

    fn emit_data_range_inclusions_expr(
        &mut self,
        ontology: &Ontology,
        range: DeId,
        defdata: &str,
    ) -> Result<(), Error> {
        let expr = ontology.dl().de(range).cloned();
        let Some(expr) = expr else {
            return Ok(());
        };
        match expr {
            DataExpr::And(ids) => {
                let flat = flatten_data_and(ontology, ids);
                let non_trivial: Vec<DeId> = flat
                    .iter()
                    .copied()
                    .filter(|id| !is_trivial_datatype(ontology, *id))
                    .collect();
                let use_ids = if non_trivial.is_empty() {
                    flat
                } else {
                    non_trivial
                };
                for id in use_ids {
                    self.emit_single_data_range_inclusion(ontology, id, defdata)?;
                }
            }
            DataExpr::Or(_) => {
                self.emit_union_inclusion(ontology, range, defdata)?;
            }
            DataExpr::Not(inner) => {
                if let Some(clause) = complement_intersection_oneof_clause(ontology, inner, defdata) {
                    self.set.push_clause(clause);
                } else {
                    self.emit_complement_inclusions(ontology, inner, defdata)?;
                }
            }
            _ => {
                self.emit_single_data_range_inclusion(ontology, range, defdata)?;
            }
        }
        Ok(())
    }

    fn emit_single_data_range_inclusion(
        &mut self,
        ontology: &Ontology,
        range: DeId,
        defdata: &str,
    ) -> Result<(), Error> {
        let Some(dr) = self.resolve_data_range(ontology, range) else {
            return Ok(());
        };
        self.set.push_clause(HyperClause {
            head: vec![HyperAtom::DataRange {
                range: dr,
                term: Term::Var(VAR_X.into()),
            }],
            body: vec![HyperAtom::DataRange {
                range: DataRangeFmt::Internal(defdata.to_string()),
                term: Term::Var(VAR_X.into()),
            }],
        });
        Ok(())
    }

    fn emit_union_inclusion(
        &mut self,
        ontology: &Ontology,
        range: DeId,
        defdata: &str,
    ) -> Result<(), Error> {
        let DataExpr::Or(ids) = ontology.dl().de(range).cloned().unwrap_or(DataExpr::Top) else {
            return Ok(());
        };
        let mut head = Vec::new();
        for id in ids {
            if let Some(dr) = self.resolve_data_range(ontology, id) {
                head.push(HyperAtom::DataRange {
                    range: dr,
                    term: Term::Var(VAR_X.into()),
                });
            }
        }
        if head.is_empty() {
            return Ok(());
        }
        self.set.push_clause(HyperClause {
            head,
            body: vec![HyperAtom::DataRange {
                range: DataRangeFmt::Internal(defdata.to_string()),
                term: Term::Var(VAR_X.into()),
            }],
        });
        Ok(())
    }

    fn emit_complement_inclusions(
        &mut self,
        ontology: &Ontology,
        inner: DeId,
        defdata: &str,
    ) -> Result<(), Error> {
        let inner_expr = ontology.dl().de(inner).cloned();
        let Some(inner_expr) = inner_expr else {
            return Ok(());
        };
        match inner_expr {
            DataExpr::Or(ids) => {
                for id in ids {
                    if let Some(dr) = self.resolve_data_range(ontology, id) {
                        self.set.push_clause(HyperClause {
                            head: vec![HyperAtom::DataRange {
                                range: negate_data_range(&dr),
                                term: Term::Var(VAR_X.into()),
                            }],
                            body: vec![HyperAtom::DataRange {
                                range: DataRangeFmt::Internal(defdata.to_string()),
                                term: Term::Var(VAR_X.into()),
                            }],
                        });
                    }
                }
            }
            DataExpr::And(ids) => {
                let mut head = Vec::new();
                for id in ids {
                    if let DataExpr::Or(_) = ontology.dl().de(id).cloned().unwrap_or(DataExpr::Top) {
                        if let Some(dr) = self.resolve_data_range(ontology, id) {
                            head.push(HyperAtom::DataRange {
                                range: DataRangeFmt::Not(Box::new(dr)),
                                term: Term::Var(VAR_X.into()),
                            });
                        }
                    } else if let Some(dr) = self.resolve_data_range(ontology, id) {
                        head.push(HyperAtom::DataRange {
                            range: negate_data_range(&dr),
                            term: Term::Var(VAR_X.into()),
                        });
                    }
                }
                if !head.is_empty() {
                    self.set.push_clause(HyperClause {
                        head,
                        body: vec![HyperAtom::DataRange {
                            range: DataRangeFmt::Internal(defdata.to_string()),
                            term: Term::Var(VAR_X.into()),
                        }],
                    });
                }
            }
            _ => {
                if let Some(dr) = self.resolve_data_range(ontology, inner) {
                    self.set.push_clause(HyperClause {
                        head: vec![HyperAtom::DataRange {
                            range: negate_data_range(&dr),
                            term: Term::Var(VAR_X.into()),
                        }],
                        body: vec![HyperAtom::DataRange {
                            range: DataRangeFmt::Internal(defdata.to_string()),
                            term: Term::Var(VAR_X.into()),
                        }],
                    });
                }
            }
        }
        Ok(())
    }

    fn emit_data_min_forward(
        &mut self,
        ontology: &Ontology,
        sub_id: EntityId,
        n: u32,
        property: EntityId,
        range: DeId,
    ) {
        let sub = abbrev_entity(ontology, sub_id);
        self.emit_data_min_forward_named(ontology, &sub, n, property, range);
    }

    fn emit_data_min_forward_named(
        &mut self,
        ontology: &Ontology,
        sub: &str,
        n: u32,
        property: EntityId,
        range: DeId,
    ) {
        let role = abbrev_role(ontology, property);
        let Some(range) = self.resolve_data_range(ontology, range) else {
            return;
        };
        self.set.push_clause(HyperClause {
            head: vec![HyperAtom::AtLeastData {
                n,
                role,
                range,
                term: Term::Var(VAR_X.into()),
            }],
            body: vec![HyperAtom::Concept {
                name: sub.to_string(),
                term: Term::Var(VAR_X.into()),
            }],
        });
    }

    fn emit_data_some_inverse(
        &mut self,
        ontology: &Ontology,
        sup_id: EntityId,
        property: EntityId,
        range: DeId,
    ) {
        let sup = abbrev_entity(ontology, sup_id);
        let role = abbrev_role(ontology, property);
        let Some(range) = self.resolve_data_range(ontology, range) else {
            return;
        };
        let neg = negate_data_range(&range);
        self.set.push_clause(HyperClause {
            head: vec![
                HyperAtom::Concept {
                    name: sup,
                    term: Term::Var(VAR_X.into()),
                },
                HyperAtom::DataRange {
                    range: neg,
                    term: Term::Var("Y".into()),
                },
            ],
            body: vec![HyperAtom::Role {
                role,
                subject: Term::Var(VAR_X.into()),
                object: Term::Var("Y".into()),
            }],
        });
    }

    fn emit_data_min_inverse(
        &mut self,
        ontology: &Ontology,
        sup_id: EntityId,
        n: u32,
        property: EntityId,
        range: DeId,
    ) {
        if n == 1 {
            self.emit_data_some_inverse(ontology, sup_id, property, range);
            return;
        }
        let sup = abbrev_entity(ontology, sup_id);
        let role = abbrev_role(ontology, property);
        let Some(range_fmt) = self.resolve_data_range(ontology, range) else {
            return;
        };
        let neg = negate_data_range(&range_fmt);
        let mut head = vec![HyperAtom::Concept {
            name: sup,
            term: Term::Var(VAR_X.into()),
        }];
        let mut body = Vec::new();
        for i in 1..=n {
            let y = Term::Var(format!("Y{i}"));
            body.push(HyperAtom::Role {
                role: role.clone(),
                subject: Term::Var(VAR_X.into()),
                object: y.clone(),
            });
            head.push(HyperAtom::DataRange {
                range: neg.clone(),
                term: y,
            });
        }
        for i in 1..n {
            for j in (i + 1)..=n {
                head.push(HyperAtom::Equality {
                    left: Term::Var(format!("Y{i}")),
                    right: Term::Var(format!("Y{j}")),
                });
            }
        }
        self.set.push_clause(HyperClause { head, body });
    }

    fn emit_data_max_forward(
        &mut self,
        ontology: &Ontology,
        sub_id: EntityId,
        n: u32,
        property: EntityId,
        range: DeId,
    ) {
        let sub = abbrev_entity(ontology, sub_id);
        let role = abbrev_role(ontology, property);
        let Some(range_fmt) = self.resolve_data_range(ontology, range) else {
            return;
        };
        let neg = negate_data_range(&range_fmt);
        let count = n + 1;
        let mut head = Vec::new();
        let mut body = vec![HyperAtom::Concept {
            name: sub,
            term: Term::Var(VAR_X.into()),
        }];
        for i in 1..=count {
            let y = Term::Var(format!("Y{i}"));
            body.push(HyperAtom::Role {
                role: role.clone(),
                subject: Term::Var(VAR_X.into()),
                object: y.clone(),
            });
            head.push(HyperAtom::DataRange {
                range: neg.clone(),
                term: y,
            });
        }
        for i in 1..count {
            for j in (i + 1)..=count {
                head.push(HyperAtom::Equality {
                    left: Term::Var(format!("Y{i}")),
                    right: Term::Var(format!("Y{j}")),
                });
            }
        }
        self.set.push_clause(HyperClause { head, body });
    }

    fn emit_data_max_inverse(
        &mut self,
        ontology: &Ontology,
        sup_id: EntityId,
        n: u32,
        property: EntityId,
        range: DeId,
    ) {
        let sup = abbrev_entity(ontology, sup_id);
        let role = abbrev_role(ontology, property);
        let Some(range_fmt) = self.resolve_data_range(ontology, range) else {
            return;
        };
        self.set.push_clause(HyperClause {
            head: vec![
                HyperAtom::Concept {
                    name: sup,
                    term: Term::Var(VAR_X.into()),
                },
                HyperAtom::AtLeastData {
                    n: n + 1,
                    role,
                    range: range_fmt,
                    term: Term::Var(VAR_X.into()),
                },
            ],
            body: vec![HyperAtom::Concept {
                name: "owl:Thing".into(),
                term: Term::Var(VAR_X.into()),
            }],
        });
    }

    fn emit_data_exact_inverse(
        &mut self,
        ontology: &Ontology,
        sup_id: EntityId,
        n: u32,
        property: EntityId,
        range: DeId,
    ) {
        let sup = abbrev_entity(ontology, sup_id);
        let role = abbrev_role(ontology, property);
        let Some(range_fmt) = self.resolve_data_range(ontology, range) else {
            return;
        };
        let neg = negate_data_range(&range_fmt);
        if n == 1 {
            self.set.push_clause(HyperClause {
                head: vec![
                    HyperAtom::Concept {
                        name: sup,
                        term: Term::Var(VAR_X.into()),
                    },
                    HyperAtom::AtLeastData {
                        n: 2,
                        role: role.clone(),
                        range: range_fmt,
                        term: Term::Var(VAR_X.into()),
                    },
                    HyperAtom::DataRange {
                        range: neg,
                        term: Term::Var("Y".into()),
                    },
                ],
                body: vec![HyperAtom::Role {
                    role,
                    subject: Term::Var(VAR_X.into()),
                    object: Term::Var("Y".into()),
                }],
            });
            return;
        }
        let mut head = vec![
            HyperAtom::Concept {
                name: sup,
                term: Term::Var(VAR_X.into()),
            },
            HyperAtom::AtLeastData {
                n: n + 1,
                role: role.clone(),
                range: range_fmt,
                term: Term::Var(VAR_X.into()),
            },
        ];
        let mut body = Vec::new();
        for i in 1..=n {
            let y = Term::Var(format!("Y{i}"));
            body.push(HyperAtom::Role {
                role: role.clone(),
                subject: Term::Var(VAR_X.into()),
                object: y.clone(),
            });
            head.push(HyperAtom::DataRange {
                range: neg.clone(),
                term: y,
            });
        }
        for i in 1..n {
            for j in (i + 1)..=n {
                head.push(HyperAtom::Equality {
                    left: Term::Var(format!("Y{i}")),
                    right: Term::Var(format!("Y{j}")),
                });
            }
        }
        self.set.push_clause(HyperClause { head, body });
    }

    fn emit_data_has_value_inverse(
        &mut self,
        ontology: &Ontology,
        sup_id: EntityId,
        property: EntityId,
        one_of: DeId,
    ) {
        let sup = abbrev_entity(ontology, sup_id);
        let role = abbrev_role(ontology, property);
        let Some(range) = self.resolve_data_range(ontology, one_of) else {
            return;
        };
        let neg = negate_data_range(&range);
        self.set.push_clause(HyperClause {
            head: vec![
                HyperAtom::Concept {
                    name: sup,
                    term: Term::Var(VAR_X.into()),
                },
                HyperAtom::DataRange {
                    range: neg,
                    term: Term::Var("Y".into()),
                },
            ],
            body: vec![HyperAtom::Role {
                role,
                subject: Term::Var(VAR_X.into()),
                object: Term::Var("Y".into()),
            }],
        });
    }

    fn emit_asymmetric(&mut self, ontology: &Ontology, property: EntityId) {
        let role = abbrev_role(ontology, property);
        self.set.push_clause(HyperClause {
            head: vec![],
            body: vec![
                HyperAtom::Role {
                    role: role.clone(),
                    subject: Term::Var(VAR_X.into()),
                    object: Term::Var("Y".into()),
                },
                HyperAtom::Role {
                    role,
                    subject: Term::Var("Y".into()),
                    object: Term::Var(VAR_X.into()),
                },
            ],
        });
    }

    fn emit_role_subsumption(&mut self, ontology: &Ontology, sub: EntityId, sup: EntityId) {
        let sub_r = abbrev_role(ontology, sub);
        let sup_r = abbrev_role(ontology, sup);
        self.set.push_clause(HyperClause {
            head: vec![HyperAtom::Role {
                role: sup_r,
                subject: Term::Var(VAR_X.into()),
                object: Term::Var("Y".into()),
            }],
            body: vec![HyperAtom::Role {
                role: sub_r,
                subject: Term::Var(VAR_X.into()),
                object: Term::Var("Y".into()),
            }],
        });
    }

    fn emit_role_fact(
        &mut self,
        ontology: &Ontology,
        property: EntityId,
        sub: EntityId,
        obj: EntityId,
    ) {
        let role = abbrev_role(ontology, property);
        let s = abbrev_entity(ontology, sub);
        let o = abbrev_entity(ontology, obj);
        self.set.push_fact(HyperAtom::Role {
            role,
            subject: Term::Ind(s),
            object: Term::Ind(o),
        });
    }

    fn resolve_data_range(&self, ontology: &Ontology, de: DeId) -> Option<DataRangeFmt> {
        let simplified = simplify_data_range(ontology, de);
        data_range_fmt(ontology, simplified, &|id| self.defdata_names.get(&id).cloned())
    }

    fn fresh_def_name(&mut self) -> String {
        let name = format!("def:{}", self.def_index);
        self.def_index += 1;
        name
    }

    fn fresh_defdata_name(&mut self, ontology: &mut Ontology, range: DeId) -> String {
        if let Some(existing) = self.defdata_names.get(&range) {
            return existing.clone();
        }
        let name = format!("defdata:{}", self.defdata_index);
        let iri = format!("internal:defdata#{}", self.defdata_index);
        self.defdata_index += 1;
        if let Ok(dt) = ontology.entity_id(&iri, EntityKind::Datatype) {
            let de = ontology.dl_mut().intern_de(DataExpr::Datatype(dt));
            self.defdata_names.insert(range, name.clone());
            self.defdata_names.insert(de, name.clone());
        }
        name
    }
}

fn is_trivial_datatype(ontology: &Ontology, de: DeId) -> bool {
    match ontology.dl().de(de) {
        Some(DataExpr::Top) => true,
        Some(DataExpr::Datatype(id)) => ontology
            .entity(*id)
            .ok()
            .and_then(|r| ontology.iris().resolve(r.iri).ok())
            .is_some_and(|iri| {
                iri == "http://www.w3.org/2000/01/rdf-schema#Literal"
                    || iri == "http://www.w3.org/2001/XMLSchema#string"
            }),
        _ => false,
    }
}

fn flatten_data_and(ontology: &Ontology, ids: Vec<DeId>) -> Vec<DeId> {
    let mut out = Vec::new();
    for id in ids {
        if let Some(DataExpr::And(inner)) = ontology.dl().de(id).cloned() {
            out.extend(flatten_data_and(ontology, inner));
        } else {
            out.push(id);
        }
    }
    out
}

fn is_complex_data_range(ontology: &Ontology, de: DeId) -> bool {
    match ontology.dl().de(de) {
        Some(DataExpr::Datatype(_)) | Some(DataExpr::Literal { .. }) | Some(DataExpr::Top) => {
            false
        }
        Some(DataExpr::Or(ids)) => !ids.iter().all(|id| {
            matches!(
                ontology.dl().de(*id),
                Some(DataExpr::Literal { .. })
            )
        }),
        None => false,
        _ => true,
    }
}

fn complement_intersection_oneof_clause(
    ontology: &Ontology,
    inner: DeId,
    defdata: &str,
) -> Option<HyperClause> {
    let DataExpr::And(ids) = ontology.dl().de(inner)?.clone() else {
        return None;
    };
    if ids.len() != 2 {
        return None;
    }
    let mut heads = Vec::new();
    for id in ids {
        let DataExpr::Or(lits) = ontology.dl().de(id)?.clone() else {
            return None;
        };
        let mut literal_fmts = Vec::new();
        for lit_id in lits {
            let DataExpr::Literal { lexical, datatype } = ontology.dl().de(lit_id)?.clone() else {
                return None;
            };
            let iri = ontology
                .entity(datatype)
                .ok()
                .and_then(|r| ontology.iris().resolve(r.iri).ok())?;
            literal_fmts.push(crate::hyperclause::LiteralFmt {
                lexical,
                datatype: crate::hyperclause::literal_datatype_suffix(iri),
            });
        }
        for lit in &mut literal_fmts {
            crate::hyperclause::coerce_enumeration_literal(lit);
        }
        crate::hyperclause::sort_oneof_literals(&mut literal_fmts);
        heads.push(HyperAtom::DataRange {
            range: DataRangeFmt::Not(Box::new(DataRangeFmt::OneOf(literal_fmts))),
            term: Term::Var(VAR_X.into()),
        });
    }
    Some(HyperClause {
        head: heads,
        body: vec![HyperAtom::DataRange {
            range: DataRangeFmt::Internal(defdata.to_string()),
            term: Term::Var(VAR_X.into()),
        }],
    })
}

fn is_thing_entity(ontology: &Ontology, id: EntityId) -> bool {
    ontology
        .entity(id)
        .ok()
        .and_then(|r| ontology.iris().resolve(r.iri).ok())
        .is_some_and(|iri| iri == OWL_THING)
}

fn is_thing_ce(ontology: &Ontology, ce: CeId) -> bool {
    match ontology.dl().ce(ce) {
        Some(ClassExpr::Top) => true,
        Some(ClassExpr::Atomic(id)) => ontology
            .entity(*id)
            .ok()
            .and_then(|r| ontology.iris().resolve(r.iri).ok())
            .is_some_and(|iri| iri == OWL_THING),
        _ => false,
    }
}

fn simplify_data_range(ontology: &Ontology, de: DeId) -> DeId {
    let Some(expr) = ontology.dl().de(de).cloned() else {
        return de;
    };
    if let DataExpr::Not(inner) = expr {
        if let Some(DataExpr::Not(inner2)) = ontology.dl().de(inner).cloned() {
            return simplify_data_range(ontology, inner2);
        }
    }
    de
}

fn literal_top(ontology: &mut Ontology) -> DeId {
    ontology.dl_mut().intern_de(DataExpr::Top)
}

fn literal_one_of_de(ontology: &mut Ontology, value: &DeId) -> DeId {
    let Some(DataExpr::Literal { lexical, datatype }) = ontology.dl().de(*value).cloned() else {
        return *value;
    };
    let lit = ontology
        .dl_mut()
        .intern_de(DataExpr::Literal { lexical, datatype });
    ontology.dl_mut().intern_de(DataExpr::Or(vec![lit]))
}
