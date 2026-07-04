package dev.ontologos;

/** Reasoning did not complete within configured limits. */
public final class IncompleteReasoningException extends OntologosException {
    public IncompleteReasoningException(String message) {
        super(message);
    }
}
