package dev.ontologos;

/** Resource limit exceeded. */
public final class ResourceLimitException extends OntologosException {
    public ResourceLimitException(String message) {
        super(message);
    }
}
