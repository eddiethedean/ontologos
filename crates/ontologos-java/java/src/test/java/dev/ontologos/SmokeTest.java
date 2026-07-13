package dev.ontologos;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class SmokeTest {
    @Test
    void versionMatchesRelease() {
        assertEquals("1.1.4", Ontologos.version());
    }

    @Test
    void builderClassifyEl() {
        try (OntologyBuilder builder = new OntologyBuilder()) {
            builder.addClass("http://example.org/Pizza");
            builder.addClass("http://example.org/Food");
            builder.subclassOf("http://example.org/Pizza", "http://example.org/Food");
            try (Ontology ontology = builder.build();
                    Reasoner reasoner = new Reasoner(ontology, "el")) {
                String report = reasoner.classify();
                assertTrue(report.contains("\"status\":\"classified\""));
                assertTrue(report.contains("subsumption_count"));
            }
        }
    }

    @Test
    void fromBytesStrictFunctionalSyntax() {
        String ofn =
                """
                Prefix(:=<http://example.org/>)
                Ontology(<http://example.org/o>
                  Declaration(Class(:A))
                  Declaration(Class(:B))
                  SubClassOf(:A :B)
                )""";
        try (Ontology ontology = Ontology.fromBytes(ofn.getBytes(java.nio.charset.StandardCharsets.UTF_8))) {
            assertTrue(ontology.getAxiomCount() >= 1L);
        }
    }

    @Test
    void sharedOntologyMutationSync() {
        try (OntologyBuilder builder = new OntologyBuilder()) {
            builder.addClass("http://example.org/A");
            builder.addClass("http://example.org/B");
            try (Ontology ontology = builder.build();
                    Reasoner reasoner = new Reasoner(ontology, "el")) {
                assertEquals(0L, ontology.getAxiomCount());
                reasoner.addSubclassOf("http://example.org/A", "http://example.org/B");
                assertEquals(1L, ontology.getAxiomCount());
            }
        }
    }
}
