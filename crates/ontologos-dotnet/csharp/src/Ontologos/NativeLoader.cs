using System.Runtime.InteropServices;

namespace Ontologos;

internal static class NativeLoader
{
    private static readonly object Gate = new();
    private static IntPtr _libraryHandle;
    private static bool _loaded;

    internal static void EnsureLoaded()
    {
        if (_loaded)
        {
            return;
        }

        lock (Gate)
        {
            if (_loaded)
            {
                return;
            }

            var path = ResolveLibraryPath()
                ?? throw new DllNotFoundException(
                    "OntoLogos native library not found; build with "
                    + "`cargo build -p ontologos-dotnet --release` "
                    + "or set ONTOLOGOS_NATIVE_PATH");

            _libraryHandle = NativeLibrary.Load(path);
            NativeLibrary.SetDllImportResolver(
                typeof(NativeMethods).Assembly,
                (_, _, _) => _libraryHandle);
            _loaded = true;
        }
    }

    private static string? ResolveLibraryPath()
    {
        var overridePath = Environment.GetEnvironmentVariable("ONTOLOGOS_NATIVE_PATH");
        if (!string.IsNullOrWhiteSpace(overridePath))
        {
            return Path.GetFullPath(overridePath);
        }

        return FindWorkspaceLibrary();
    }

    private static string? FindWorkspaceLibrary()
    {
        var fileName = OperatingSystem.IsMacOS()
            ? "libontologos_dotnet.dylib"
            : OperatingSystem.IsLinux()
                ? "libontologos_dotnet.so"
                : OperatingSystem.IsWindows()
                    ? "ontologos_dotnet.dll"
                    : null;

        if (fileName is null)
        {
            return null;
        }

        foreach (var root in CandidateRoots())
        {
            if (string.IsNullOrWhiteSpace(root))
            {
                continue;
            }

            var candidate = Path.GetFullPath(Path.Combine(root, "target", "release", fileName));
            if (File.Exists(candidate))
            {
                return candidate;
            }
        }

        return null;
    }

    private static IEnumerable<string> CandidateRoots()
    {
        yield return Environment.GetEnvironmentVariable("ONTOLOGOS_REPO_ROOT") ?? string.Empty;

        var current = AppContext.BaseDirectory;
        for (var i = 0; i < 10 && !string.IsNullOrEmpty(current); i++)
        {
            yield return current;
            current = Directory.GetParent(current)?.FullName ?? string.Empty;
        }

        yield return Directory.GetCurrentDirectory();
    }
}
