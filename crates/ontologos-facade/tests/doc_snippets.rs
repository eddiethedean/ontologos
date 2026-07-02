//! Compile-check for canonical documentation examples (production facade pattern).

use ontologos_core::{Profile, Reasoner};
use ontologos_facade::{ClassifyOutcome, check_consistency, classify};

#[test]
fn production_facade_pattern_compiles() {
    fn run() -> ontologos_facade::Result<()> {
        let ontology = ontologos_core::Ontology::builder()
            .class("http://example.org/A")?
            .class("http://example.org/B")?
            .subclass_of("http://example.org/A", "http://example.org/B")?
            .build()?;

        let mut reasoner = Reasoner::builder().profile(Profile::El).build(ontology)?;

        let consistency = check_consistency(&reasoner)?;
        assert!(consistency.complete);
        assert!(consistency.consistent);

        match classify(&mut reasoner)? {
            ClassifyOutcome::Taxonomy(t) => {
                assert!(t.subsumption_count() >= 1);
            }
            ClassifyOutcome::Rdfs(_) | ClassifyOutcome::Rl(_) => {
                panic!("EL profile should yield taxonomy");
            }
        }
        Ok(())
    }
    run().expect("doc snippet pattern should succeed");
}
