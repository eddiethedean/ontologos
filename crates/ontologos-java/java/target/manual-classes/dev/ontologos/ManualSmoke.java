package dev.ontologos;

public final class ManualSmoke {
    private ManualSmoke() {}

    public static void main(String[] args) {
        if (!"1.0.1".equals(Ontologos.version())) {
            throw new AssertionError("unexpected version: " + Ontologos.version());
        }
        try (OntologyBuilder builder = new OntologyBuilder()) {
            builder.addClass("http://example.org/Pizza");
            builder.addClass("http://example.org/Food");
            builder.subclassOf("http://example.org/Pizza", "http://example.org/Food");
            try (Ontology ontology = builder.build();
                    Reasoner reasoner = new Reasoner(ontology, "el")) {
                String report = reasoner.classify();
                if (!report.contains("\"status\":\"classified\"")) {
                    throw new AssertionError("classify report missing status: " + report);
                }
            }
        }
        System.out.println("Java smoke test passed");
    }
}
