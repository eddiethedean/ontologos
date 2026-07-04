package dev.ontologos;

import java.util.Objects;

/** OWL reasoner handle. */
public final class Reasoner implements AutoCloseable {
    static {
        NativeLoader.load();
    }

    private long handle;
    private boolean closed;

    private Reasoner(long handle) {
        if (handle == 0L) {
            throw new OntologosException("failed to create reasoner");
        }
        this.handle = handle;
    }

    public Reasoner(Ontology ontology, String profile) {
        this(ontology, profile, false, null);
    }

    public Reasoner(Ontology ontology, String profile, boolean incremental, Long budgetSecs) {
        this(
                nativeNew(
                        Objects.requireNonNull(ontology).nativeHandle(),
                        profile,
                        incremental,
                        toNativeOptional(budgetSecs)));
    }

    public static Reasoner fromPath(String path, String profile) {
        return fromPath(path, profile, false, null, false);
    }

    public static Reasoner fromPath(
            String path, String profile, boolean incremental, Long budgetSecs, boolean lenient) {
        return new Reasoner(
                nativeFromPath(
                        Objects.requireNonNull(path),
                        profile,
                        incremental,
                        toNativeOptional(budgetSecs),
                        lenient));
    }

    public static Reasoner loadIn(String base, String path, String profile) {
        return loadIn(base, path, profile, false, null, false);
    }

    public static Reasoner loadIn(
            String base,
            String path,
            String profile,
            boolean incremental,
            Long budgetSecs,
            boolean lenient) {
        return new Reasoner(
                nativeLoadIn(
                        Objects.requireNonNull(base),
                        Objects.requireNonNull(path),
                        profile,
                        incremental,
                        toNativeOptional(budgetSecs),
                        lenient));
    }

    /** Returns classify/materialize report JSON. */
    public String classify() {
        ensureOpen();
        return nativeClassify(handle);
    }

    /** Returns explain result JSON. */
    public String explain() {
        ensureOpen();
        return nativeExplain(handle);
    }

    /** Returns parse metadata JSON (empty object when absent). */
    public String parseMeta() {
        ensureOpen();
        return nativeParseMeta(handle);
    }

    /** Returns taxonomy JSON after EL/DL classify, or {@code null}. */
    public String taxonomy() {
        ensureOpen();
        return nativeTaxonomy(handle);
    }

    /** Returns consistency check JSON. */
    public String checkConsistency() {
        ensureOpen();
        return nativeCheckConsistency(handle);
    }

    public boolean isConsistent() {
        ensureOpen();
        return nativeIsConsistent(handle);
    }

    public boolean isEntailed(EntailmentCheck check) {
        ensureOpen();
        Objects.requireNonNull(check);
        return nativeIsEntailed(
                handle,
                check.sub(),
                check.sup(),
                check.individual(),
                check.classIri(),
                check.subject(),
                check.property(),
                check.object());
    }

    /** Returns query bindings JSON array. */
    public String query(String query) {
        ensureOpen();
        return nativeQuery(handle, Objects.requireNonNull(query));
    }

    public Reasoner addSubclassOf(String subclass, String superclass) {
        ensureOpen();
        handle =
                nativeAddSubclassOf(
                        handle,
                        Objects.requireNonNull(subclass),
                        Objects.requireNonNull(superclass));
        return this;
    }

    public Reasoner removeSubclassOf(String subclass, String superclass) {
        ensureOpen();
        handle =
                nativeRemoveSubclassOf(
                        handle,
                        Objects.requireNonNull(subclass),
                        Objects.requireNonNull(superclass));
        return this;
    }

    public Reasoner addAxiomJson(String axiomJson) {
        ensureOpen();
        handle = nativeAddAxiomJson(handle, Objects.requireNonNull(axiomJson));
        return this;
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
            throw new IllegalStateException("reasoner handle is closed");
        }
    }

    private static long toNativeOptional(Long value) {
        return value == null ? -1L : value;
    }

    private static native long nativeNew(
            long ontologyHandle, String profile, boolean incremental, long budgetSecs);

    private static native long nativeFromPath(
            String path, String profile, boolean incremental, long budgetSecs, boolean lenient);

    private static native long nativeLoadIn(
            String base,
            String path,
            String profile,
            boolean incremental,
            long budgetSecs,
            boolean lenient);

    private static native String nativeParseMeta(long handle);

    private static native String nativeTaxonomy(long handle);

    private static native String nativeClassify(long handle);

    private static native String nativeExplain(long handle);

    private static native String nativeCheckConsistency(long handle);

    private static native boolean nativeIsConsistent(long handle);

    private static native boolean nativeIsEntailed(
            long handle,
            String sub,
            String sup,
            String individual,
            String classIri,
            String subject,
            String property,
            String object);

    private static native String nativeQuery(long handle, String query);

    private static native long nativeAddSubclassOf(long handle, String subclass, String superclass);

    private static native long nativeRemoveSubclassOf(long handle, String subclass, String superclass);

    private static native long nativeAddAxiomJson(long handle, String axiomJson);

    private static native void nativeClose(long handle);
}
