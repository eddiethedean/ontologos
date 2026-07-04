package dev.ontologos;

/** Parse or serialization failure. */
public final class ParseException extends OntologosException {
    public ParseException(String message) {
        super(message);
    }
}
