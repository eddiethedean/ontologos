package dev.ontologos;

/** Root exception for OntoLogos Java bindings. */
public class OntologosException extends RuntimeException {
    public OntologosException(String message) {
        super(message);
    }
}
