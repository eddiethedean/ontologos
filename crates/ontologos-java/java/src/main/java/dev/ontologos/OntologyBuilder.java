package dev.ontologos;

import java.util.Objects;

/** Fluent builder for in-memory ontologies. */
public final class OntologyBuilder implements AutoCloseable {
    static {
        NativeLoader.load();
    }

    private long handle;
    private boolean closed;

    public OntologyBuilder() {
        handle = nativeNew();
        if (handle == 0L) {
            throw new OntologosException("failed to create ontology builder");
        }
    }

    public OntologyBuilder addClass(String iri) {
        ensureOpen();
        handle = nativeAddClass(handle, Objects.requireNonNull(iri));
        return this;
    }

    public OntologyBuilder individual(String iri) {
        ensureOpen();
        handle = nativeIndividual(handle, Objects.requireNonNull(iri));
        return this;
    }

    public OntologyBuilder objectProperty(String iri) {
        ensureOpen();
        handle = nativeObjectProperty(handle, Objects.requireNonNull(iri));
        return this;
    }

    public OntologyBuilder subclassOf(String subclass, String superclass) {
        ensureOpen();
        handle =
                nativeSubclassOf(
                        handle,
                        Objects.requireNonNull(subclass),
                        Objects.requireNonNull(superclass));
        return this;
    }

    public OntologyBuilder subpropertyOf(String sub, String sup) {
        ensureOpen();
        handle = nativeSubpropertyOf(handle, Objects.requireNonNull(sub), Objects.requireNonNull(sup));
        return this;
    }

    public OntologyBuilder propertyDomain(String property, String domain) {
        ensureOpen();
        handle =
                nativePropertyDomain(
                        handle,
                        Objects.requireNonNull(property),
                        Objects.requireNonNull(domain));
        return this;
    }

    public OntologyBuilder propertyRange(String property, String range) {
        ensureOpen();
        handle =
                nativePropertyRange(
                        handle, Objects.requireNonNull(property), Objects.requireNonNull(range));
        return this;
    }

    public OntologyBuilder classAssertion(String individual, String classIri) {
        ensureOpen();
        handle =
                nativeClassAssertion(
                        handle,
                        Objects.requireNonNull(individual),
                        Objects.requireNonNull(classIri));
        return this;
    }

    public OntologyBuilder objectPropertyAssertion(String subject, String property, String object) {
        ensureOpen();
        handle =
                nativeObjectPropertyAssertion(
                        handle,
                        Objects.requireNonNull(subject),
                        Objects.requireNonNull(property),
                        Objects.requireNonNull(object));
        return this;
    }

    public Ontology build() {
        ensureOpen();
        long ontologyHandle = nativeBuild(handle);
        handle = 0L;
        closed = true;
        return new Ontology(ontologyHandle);
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
            throw new IllegalStateException("ontology builder handle is closed");
        }
    }

    private static native long nativeNew();

    private static native long nativeAddClass(long handle, String iri);

    private static native long nativeIndividual(long handle, String iri);

    private static native long nativeObjectProperty(long handle, String iri);

    private static native long nativeSubclassOf(long handle, String subclass, String superclass);

    private static native long nativeSubpropertyOf(long handle, String sub, String sup);

    private static native long nativePropertyDomain(long handle, String property, String domain);

    private static native long nativePropertyRange(long handle, String property, String range);

    private static native long nativeClassAssertion(long handle, String individual, String classIri);

    private static native long nativeObjectPropertyAssertion(
            long handle, String subject, String property, String object);

    private static native long nativeBuild(long handle);

    private static native void nativeClose(long handle);
}
