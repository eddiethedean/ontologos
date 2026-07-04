package dev.ontologos;

/** Package metadata and helpers. */
public final class Ontologos {
    static {
        NativeLoader.load();
    }

    private Ontologos() {}

    /** Returns the OntoLogos package version. */
    public static String version() {
        return nativeVersion();
    }

    /**
     * Returns a typed error code prefix from an exception message, if present.
     *
     * @return one of {@code ParseError}, {@code ResourceLimitError}, {@code IncompleteReasoningError},
     *         {@code OntologyConflictError}, or {@code null}
     */
    public static String errorCodeFromMessage(String message) {
        return nativeErrorCodeFromMessage(message);
    }

    private static native String nativeVersion();

    private static native String nativeErrorCodeFromMessage(String message);
}
