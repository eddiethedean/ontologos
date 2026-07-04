namespace Ontologos;

/// <summary>In-memory ontology handle.</summary>
public sealed class Ontology : IDisposable
{
    private long _handle;
    private bool _disposed;

    static Ontology()
    {
        NativeLoader.EnsureLoaded();
    }

    internal Ontology(long handle)
    {
        _handle = NativeInterop.CheckHandle(handle, "ontology");
    }

    public static Ontology FromJson(string json) =>
        new(NativeMethods.OntologyFromJsonNative(json));

    public static Ontology FromJsonWithLimits(
        string json,
        long? maxJsonBytes = null,
        long? maxEntities = null,
        long? maxAxioms = null,
        long? maxIriLen = null) =>
        new(
            NativeMethods.OntologyFromJsonWithLimitsNative(
                json,
                NativeInterop.ToNativeOptional(maxJsonBytes),
                NativeInterop.ToNativeOptional(maxEntities),
                NativeInterop.ToNativeOptional(maxAxioms),
                NativeInterop.ToNativeOptional(maxIriLen)));

    public static Ontology FromBytes(byte[] bytes)
    {
        ArgumentNullException.ThrowIfNull(bytes);
        return new(NativeMethods.OntologyFromBytesNative(bytes, (nuint)bytes.Length));
    }

    public static Ontology FromBytesLenient(byte[] bytes)
    {
        ArgumentNullException.ThrowIfNull(bytes);
        return new(NativeMethods.OntologyFromBytesLenientNative(bytes, (nuint)bytes.Length));
    }

    public static Ontology FromText(string text) =>
        new(NativeMethods.OntologyFromTextNative(text));

    public static Ontology FromTextLenient(string text) =>
        new(NativeMethods.OntologyFromTextLenientNative(text));

    public static Ontology Load(string path, bool lenient = false) =>
        new(NativeMethods.OntologyLoadNative(path, lenient ? 1 : 0));

    public static Ontology LoadIn(string baseDir, string path, bool lenient = false) =>
        new(NativeMethods.OntologyLoadInNative(baseDir, path, lenient ? 1 : 0));

    public string ToJson()
    {
        EnsureOpen();
        return NativeInterop.TakeString(NativeMethods.OntologyToJsonNative(_handle));
    }

    public long AxiomCount
    {
        get
        {
            EnsureOpen();
            var count = NativeMethods.OntologyAxiomCountNative(_handle);
            if (count < 0)
            {
                NativeInterop.ThrowIfError();
            }

            return count;
        }
    }

    public long EntityCount
    {
        get
        {
            EnsureOpen();
            var count = NativeMethods.OntologyEntityCountNative(_handle);
            if (count < 0)
            {
                NativeInterop.ThrowIfError();
            }

            return count;
        }
    }

    internal long NativeHandle()
    {
        EnsureOpen();
        return _handle;
    }

    public void Dispose()
    {
        if (!_disposed && _handle != 0)
        {
            NativeMethods.OntologyCloseNative(_handle);
            _handle = 0;
            _disposed = true;
        }
    }

    private void EnsureOpen()
    {
        if (_disposed || _handle == 0)
        {
            throw new ObjectDisposedException(nameof(Ontology));
        }
    }
}
