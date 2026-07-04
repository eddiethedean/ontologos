namespace Ontologos;

/// <summary>OWL reasoner handle.</summary>
public sealed class Reasoner : IDisposable
{
    private long _handle;
    private bool _disposed;

    static Reasoner()
    {
        NativeLoader.EnsureLoaded();
    }

    private Reasoner(long handle)
    {
        _handle = NativeInterop.CheckHandle(handle, "reasoner");
    }

    public Reasoner(Ontology ontology, string? profile, bool incremental = false, long? budgetSecs = null)
        : this(
            NativeMethods.ReasonerNewNative(
                ontology.NativeHandle(),
                profile,
                incremental ? 1 : 0,
                NativeInterop.ToNativeOptional(budgetSecs)))
    {
    }

    public static Reasoner FromPath(
        string path,
        string? profile,
        bool incremental = false,
        long? budgetSecs = null,
        bool lenient = false) =>
        new(
            NativeMethods.ReasonerFromPathNative(
                path,
                profile,
                incremental ? 1 : 0,
                NativeInterop.ToNativeOptional(budgetSecs),
                lenient ? 1 : 0));

    public static Reasoner LoadIn(
        string baseDir,
        string path,
        string? profile,
        bool incremental = false,
        long? budgetSecs = null,
        bool lenient = false) =>
        new(
            NativeMethods.ReasonerLoadInNative(
                baseDir,
                path,
                profile,
                incremental ? 1 : 0,
                NativeInterop.ToNativeOptional(budgetSecs),
                lenient ? 1 : 0));

    public string Classify()
    {
        EnsureOpen();
        return NativeInterop.TakeString(NativeMethods.ReasonerClassifyNative(_handle));
    }

    public string Explain()
    {
        EnsureOpen();
        return NativeInterop.TakeString(NativeMethods.ReasonerExplainNative(_handle));
    }

    public string ParseMeta()
    {
        EnsureOpen();
        return NativeInterop.TakeString(NativeMethods.ReasonerParseMetaNative(_handle));
    }

    public string? Taxonomy()
    {
        EnsureOpen();
        return NativeInterop.TakeString(NativeMethods.ReasonerTaxonomyNative(_handle), allowNull: true);
    }

    public string CheckConsistency()
    {
        EnsureOpen();
        return NativeInterop.TakeString(NativeMethods.ReasonerCheckConsistencyNative(_handle));
    }

    public bool IsConsistent()
    {
        EnsureOpen();
        return NativeMethods.ReasonerIsConsistentNative(_handle) != 0;
    }

    public bool IsEntailed(EntailmentCheck check)
    {
        ArgumentNullException.ThrowIfNull(check);
        EnsureOpen();
        return NativeMethods.ReasonerIsEntailedNative(
            _handle,
            check.Sub,
            check.Sup,
            check.Individual,
            check.ClassIri,
            check.Subject,
            check.Property,
            check.Object) != 0;
    }

    public string Query(string query)
    {
        EnsureOpen();
        return NativeInterop.TakeString(NativeMethods.ReasonerQueryNative(_handle, query));
    }

    public Reasoner AddSubclassOf(string subclass, string superclass)
    {
        EnsureOpen();
        _handle = NativeMethods.ReasonerAddSubclassOfNative(_handle, subclass, superclass);
        return this;
    }

    public Reasoner RemoveSubclassOf(string subclass, string superclass)
    {
        EnsureOpen();
        _handle = NativeMethods.ReasonerRemoveSubclassOfNative(_handle, subclass, superclass);
        return this;
    }

    public Reasoner AddAxiomJson(string axiomJson)
    {
        EnsureOpen();
        _handle = NativeMethods.ReasonerAddAxiomJsonNative(_handle, axiomJson);
        return this;
    }

    public void Dispose()
    {
        if (!_disposed && _handle != 0)
        {
            NativeMethods.ReasonerCloseNative(_handle);
            _handle = 0;
            _disposed = true;
        }
    }

    private void EnsureOpen()
    {
        if (_disposed || _handle == 0)
        {
            throw new ObjectDisposedException(nameof(Reasoner));
        }
    }
}
