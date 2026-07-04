namespace Ontologos;

/// <summary>Fluent builder for in-memory ontologies.</summary>
public sealed class OntologyBuilder : IDisposable
{
    private long _handle;
    private bool _disposed;

    static OntologyBuilder()
    {
        NativeLoader.EnsureLoaded();
    }

    public OntologyBuilder()
    {
        _handle = NativeInterop.CheckHandle(NativeMethods.BuilderNewNative(), "ontology builder");
    }

    public OntologyBuilder AddClass(string iri)
    {
        EnsureOpen();
        _handle = NativeMethods.BuilderAddClassNative(_handle, iri);
        return this;
    }

    public OntologyBuilder Individual(string iri)
    {
        EnsureOpen();
        _handle = NativeMethods.BuilderIndividualNative(_handle, iri);
        return this;
    }

    public OntologyBuilder ObjectProperty(string iri)
    {
        EnsureOpen();
        _handle = NativeMethods.BuilderObjectPropertyNative(_handle, iri);
        return this;
    }

    public OntologyBuilder SubclassOf(string subclass, string superclass)
    {
        EnsureOpen();
        _handle = NativeMethods.BuilderSubclassOfNative(_handle, subclass, superclass);
        return this;
    }

    public OntologyBuilder SubpropertyOf(string sub, string sup)
    {
        EnsureOpen();
        _handle = NativeMethods.BuilderSubpropertyOfNative(_handle, sub, sup);
        return this;
    }

    public OntologyBuilder PropertyDomain(string property, string domain)
    {
        EnsureOpen();
        _handle = NativeMethods.BuilderPropertyDomainNative(_handle, property, domain);
        return this;
    }

    public OntologyBuilder PropertyRange(string property, string range)
    {
        EnsureOpen();
        _handle = NativeMethods.BuilderPropertyRangeNative(_handle, property, range);
        return this;
    }

    public OntologyBuilder ClassAssertion(string individual, string classIri)
    {
        EnsureOpen();
        _handle = NativeMethods.BuilderClassAssertionNative(_handle, individual, classIri);
        return this;
    }

    public OntologyBuilder ObjectPropertyAssertion(string subject, string property, string obj)
    {
        EnsureOpen();
        _handle = NativeMethods.BuilderObjectPropertyAssertionNative(_handle, subject, property, obj);
        return this;
    }

    public Ontology Build()
    {
        EnsureOpen();
        var ontologyHandle = NativeMethods.BuilderBuildNative(_handle);
        _handle = 0;
        _disposed = true;
        return new Ontology(ontologyHandle);
    }

    public void Dispose()
    {
        if (!_disposed && _handle != 0)
        {
            NativeMethods.BuilderCloseNative(_handle);
            _handle = 0;
            _disposed = true;
        }
    }

    private void EnsureOpen()
    {
        if (_disposed || _handle == 0)
        {
            throw new ObjectDisposedException(nameof(OntologyBuilder));
        }
    }
}
