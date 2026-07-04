package dev.ontologos;

/** Shared ontology was modified concurrently. */
public final class OntologyConflictException extends OntologosException {
    public OntologyConflictException(String message) {
        super(message);
    }
}
