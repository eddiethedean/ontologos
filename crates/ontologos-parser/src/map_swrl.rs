//! Map horned-owl SWRL rules into [`ontologos_core::SwrlRule`].

use horned_owl::model::{
    Atom, ClassExpression, DataProperty, DArgument, IArgument, Literal, ObjectProperty,
    ObjectPropertyExpression, RcStr, Rule, Variable,
};
use ontologos_core::{EntityId, EntityKind, SwrlAtom, SwrlDArg, SwrlIArg, SwrlRule};

use crate::map::Mapper;

impl Mapper<'_> {
    pub(crate) fn map_swrl_rule(&mut self, rule: &Rule<RcStr>) -> Option<SwrlRule> {
        let body = rule
            .body
            .iter()
            .filter_map(|atom| self.map_swrl_atom(atom))
            .collect::<Vec<_>>();
        let head = rule
            .head
            .iter()
            .filter_map(|atom| self.map_swrl_atom(atom))
            .collect::<Vec<_>>();
        if body.len() != rule.body.len() || head.len() != rule.head.len() {
            return None;
        }
        Some(SwrlRule { body, head })
    }

    fn map_swrl_atom(&mut self, atom: &Atom<RcStr>) -> Option<SwrlAtom> {
        match atom {
            Atom::ClassAtom { pred, arg } => {
                let class = self.named_class_entity(pred)?;
                let arg = self.map_iarg(arg)?;
                Some(SwrlAtom::Class { class, arg })
            }
            Atom::ObjectPropertyAtom { pred, args } => {
                let property = self.object_property_entity(pred)?;
                let subject = self.map_iarg(&args.0)?;
                let object = self.map_iarg(&args.1)?;
                Some(SwrlAtom::ObjectProperty {
                    property,
                    subject,
                    object,
                })
            }
            Atom::DataPropertyAtom { pred, args } => {
                let property = self.data_property_entity(pred)?;
                let subject = self.map_darg_as_individual(&args.0)?;
                let value = self.map_darg(&args.1)?;
                Some(SwrlAtom::DataProperty {
                    property,
                    subject,
                    value,
                })
            }
            Atom::SameIndividualAtom(a, b) => Some(SwrlAtom::SameIndividual(
                self.map_iarg(a)?,
                self.map_iarg(b)?,
            )),
            Atom::DifferentIndividualsAtom(a, b) => Some(SwrlAtom::DifferentIndividuals(
                self.map_iarg(a)?,
                self.map_iarg(b)?,
            )),
            Atom::BuiltInAtom { .. } | Atom::DataRangeAtom { .. } => None,
        }
    }

    fn map_iarg(&mut self, arg: &IArgument<RcStr>) -> Option<SwrlIArg> {
        match arg {
            IArgument::Individual(ind) => self
                .map_individual_entity(ind)
                .map(SwrlIArg::Individual),
            IArgument::Variable(var) => Some(SwrlIArg::Variable(variable_name(var))),
        }
    }

    fn map_darg(&mut self, arg: &DArgument<RcStr>) -> Option<SwrlDArg> {
        match arg {
            DArgument::Variable(var) => Some(SwrlDArg::Variable(variable_name(var))),
            DArgument::Literal(lit) => {
                let (lexical, datatype) = literal_parts(lit);
                let datatype = datatype
                    .and_then(|dt| self.register_or_warn_entity(dt, EntityKind::Datatype));
                Some(SwrlDArg::Literal { lexical, datatype })
            }
        }
    }

    fn map_darg_as_individual(&mut self, arg: &DArgument<RcStr>) -> Option<SwrlIArg> {
        match arg {
            DArgument::Variable(var) => Some(SwrlIArg::Variable(variable_name(var))),
            DArgument::Literal(_) => None,
        }
    }

    fn named_class_entity(&mut self, ce: &ClassExpression<RcStr>) -> Option<EntityId> {
        let ClassExpression::Class(class) = ce else {
            return None;
        };
        self.register_or_warn_entity(class.as_ref(), EntityKind::Class)
    }

    fn object_property_entity(&mut self, ope: &ObjectPropertyExpression<RcStr>) -> Option<EntityId> {
        let ObjectPropertyExpression::ObjectProperty(ObjectProperty(prop)) = ope else {
            return None;
        };
        self.register_or_warn_entity(prop.as_ref(), EntityKind::ObjectProperty)
    }

    fn data_property_entity(&mut self, dp: &DataProperty<RcStr>) -> Option<EntityId> {
        self.register_or_warn_entity(dp.as_ref(), EntityKind::DataProperty)
    }
}

fn variable_name(var: &Variable<RcStr>) -> String {
    local_name(var.as_ref())
}

fn local_name(iri: &str) -> String {
    iri.rsplit(['#', '/']).next().unwrap_or(iri).to_owned()
}

fn literal_parts(lit: &Literal<RcStr>) -> (String, Option<&str>) {
    match lit {
        Literal::Simple { literal } => (literal.clone(), None),
        Literal::Language { literal, .. } => (literal.clone(), None),
        Literal::Datatype {
            literal,
            datatype_iri,
        } => (literal.clone(), Some(datatype_iri.as_ref())),
    }
}
