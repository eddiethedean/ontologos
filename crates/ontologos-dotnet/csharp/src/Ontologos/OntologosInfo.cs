namespace Ontologos;

/// <summary>Package metadata and helpers.</summary>
public static class OntologosInfo
{
    static OntologosInfo()
    {
        NativeLoader.EnsureLoaded();
    }

    /// <summary>Returns the OntoLogos package version.</summary>
    public static string Version() =>
        NativeInterop.TakeString(NativeMethods.VersionNative());

    /// <summary>Returns a typed error code prefix from an exception message, if present.</summary>
    public static string? ErrorCodeFromMessage(string message)
    {
        ArgumentNullException.ThrowIfNull(message);
        return NativeInterop.TakeString(
            NativeMethods.ErrorCodeFromMessageNative(message),
            allowNull: true);
    }
}
