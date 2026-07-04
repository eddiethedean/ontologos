package dev.ontologos;

import java.util.Objects;

/** In-memory ontology handle. */
public final class Ontology implements AutoCloseable {
    static {
        NativeLoader.load();
    }

    private long handle;
    private boolean closed;

    Ontology(long handle) {
        if (handle == 0L) {
            throw new OntologosException("failed to create ontology");
        }
        this.handle = handle;
    }

    public static Ontology fromJson(String json) {
        return new Ontology(nativeFromJson(Objects.requireNonNull(json)));
    }

    public static Ontology fromJsonWithLimits(
            String json, Long maxJsonBytes, Long maxEntities, Long maxAxioms, Long maxIriLen) {
        return new Ontology(
                nativeFromJsonWithLimits(
                        Objects.requireNonNull(json),
                        toNativeOptional(maxJsonBytes),
                        toNativeOptional(maxEntities),
                        toNativeOptional(maxAxioms),
                        toNativeOptional(maxIriLen)));
    }

    public static Ontology fromBytes(byte[] bytes) {
        return new Ontology(nativeFromBytes(Objects.requireNonNull(bytes)));
    }

    public static Ontology fromBytesLenient(byte[] bytes) {
        return new Ontology(nativeFromBytesLenient(Objects.requireNonNull(bytes)));
    }

    public static Ontology fromText(String text) {
        return new Ontology(nativeFromText(Objects.requireNonNull(text)));
    }

    public static Ontology fromTextLenient(String text) {
        return new Ontology(nativeFromTextLenient(Objects.requireNonNull(text)));
    }

    /** Load from a trusted local path (strict by default). */
    public static Ontology load(String path) {
        return load(path, false);
    }

    /** Load from a local path; set {@code lenient} for trusted corpora only. */
    public static Ontology load(String path, boolean lenient) {
        return new Ontology(nativeLoad(Objects.requireNonNull(path), lenient));
    }

    /** Sandboxed load constrained to {@code base} (recommended for uploads). */
    public static Ontology loadIn(String base, String path) {
        return loadIn(base, path, false);
    }

    public static Ontology loadIn(String base, String path, boolean lenient) {
        return new Ontology(
                nativeLoadIn(
                        Objects.requireNonNull(base), Objects.requireNonNull(path), lenient));
    }

    public String toJson() {
        ensureOpen();
        return nativeToJson(handle);
    }

    public long getAxiomCount() {
        ensureOpen();
        return nativeAxiomCount(handle);
    }

    public long getEntityCount() {
        ensureOpen();
        return nativeEntityCount(handle);
    }

    long nativeHandle() {
        ensureOpen();
        return handle;
    }

    @Override
    public void close() {
        if (!closed && handle != 0L) {
            nativeClose(handle);
            handle = 0L;
            closed = true;
        }
    }

    private void ensureOpen() {
        if (closed || handle == 0L) {
            throw new IllegalStateException("ontology handle is closed");
        }
    }

    private static long toNativeOptional(Long value) {
        return value == null ? -1L : value;
    }

    private static native long nativeFromJson(String json);

    private static native long nativeFromJsonWithLimits(
            String json, long maxJsonBytes, long maxEntities, long maxAxioms, long maxIriLen);

    private static native long nativeFromBytes(byte[] bytes);

    private static native long nativeFromBytesLenient(byte[] bytes);

    private static native long nativeFromText(String text);

    private static native long nativeFromTextLenient(String text);

    private static native long nativeLoad(String path, boolean lenient);

    private static native long nativeLoadIn(String base, String path, boolean lenient);

    private static native String nativeToJson(long handle);

    private static native long nativeAxiomCount(long handle);

    private static native long nativeEntityCount(long handle);

    private static native void nativeClose(long handle);
}
