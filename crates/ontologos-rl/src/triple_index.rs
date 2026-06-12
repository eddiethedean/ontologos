use std::collections::{HashMap, HashSet};

use ontologos_core::{Axiom, EntityId, Ontology};

/// Rule-matching index seeded from an ontology and updated as RL rules fire.
#[derive(Debug, Default)]
pub struct TripleIndex {
    transitive_properties: HashSet<EntityId>,
    symmetric_properties: HashSet<EntityId>,
    reflexive_properties: HashSet<EntityId>,
    functional_properties: HashSet<EntityId>,
    asymmetric_properties: HashSet<EntityId>,
    class_assertions: HashMap<EntityId, HashSet<EntityId>>,
    property_assertions: HashSet<(EntityId, EntityId, EntityId)>,
}

impl TripleIndex {
    #[must_use]
    pub fn from_ontology(ontology: &Ontology) -> Self {
        let mut index = Self::default();
        index.sync_from_ontology(ontology);
        index
    }

    pub fn sync_from_ontology(&mut self, ontology: &Ontology) {
        self.transitive_properties = ontology.index().transitive_properties().clone();
        self.symmetric_properties = ontology.index().symmetric_properties().clone();
        self.reflexive_properties = ontology.index().reflexive_properties().clone();
        self.functional_properties = ontology.index().functional_properties().clone();
        self.asymmetric_properties = ontology.index().asymmetric_properties().clone();
        self.class_assertions.clear();
        self.property_assertions.clear();

        for (individual, record) in ontology.entities().iter() {
            if record.kind != ontologos_core::EntityKind::Individual {
                continue;
            }
            for class in ontology.classes_of(individual) {
                self.class_assertions
                    .entry(individual)
                    .or_default()
                    .insert(*class);
            }
            for &(property, object) in ontology.object_assertions_of(individual) {
                self.property_assertions
                    .insert((individual, property, object));
            }
        }
    }

    pub fn on_axiom_added(&mut self, ontology: &Ontology, axiom: &Axiom) {
        match axiom {
            Axiom::TransitiveObjectProperty(property) => {
                self.transitive_properties.insert(*property);
            }
            Axiom::SymmetricObjectProperty(property) => {
                self.symmetric_properties.insert(*property);
            }
            Axiom::ReflexiveObjectProperty(property) => {
                self.reflexive_properties.insert(*property);
            }
            Axiom::FunctionalObjectProperty(property) => {
                self.functional_properties.insert(*property);
            }
            Axiom::AsymmetricObjectProperty(property) => {
                self.asymmetric_properties.insert(*property);
            }
            Axiom::ClassAssertion { individual, class } => {
                self.class_assertions
                    .entry(*individual)
                    .or_default()
                    .insert(*class);
            }
            Axiom::ObjectPropertyAssertion {
                subject,
                property,
                object,
            } => {
                self.property_assertions
                    .insert((*subject, *property, *object));
            }
            _ => {
                let _ = ontology;
            }
        }
    }

    #[must_use]
    pub fn transitive_properties(&self) -> &HashSet<EntityId> {
        &self.transitive_properties
    }

    #[must_use]
    pub fn symmetric_properties(&self) -> &HashSet<EntityId> {
        &self.symmetric_properties
    }

    #[must_use]
    pub fn reflexive_properties(&self) -> &HashSet<EntityId> {
        &self.reflexive_properties
    }

    #[must_use]
    pub fn class_assertions(&self, individual: EntityId) -> Option<&HashSet<EntityId>> {
        self.class_assertions.get(&individual)
    }

    #[must_use]
    pub fn has_property_assertion(
        &self,
        subject: EntityId,
        property: EntityId,
        object: EntityId,
    ) -> bool {
        self.property_assertions
            .contains(&(subject, property, object))
    }

    pub fn property_assertions(&self) -> &HashSet<(EntityId, EntityId, EntityId)> {
        &self.property_assertions
    }
}
