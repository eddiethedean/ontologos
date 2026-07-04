using System.Runtime.InteropServices;

namespace Ontologos;

internal static class NativeInterop
{
    internal static string TakeString(IntPtr ptr, bool allowNull = false)
    {
        if (ptr == IntPtr.Zero)
        {
            if (!allowNull)
            {
                ThrowIfError();
            }

            return null!;
        }

        try
        {
            return Marshal.PtrToStringUTF8(ptr)!;
        }
        finally
        {
            NativeMethods.StringFree(ptr);
        }
    }

    internal static long CheckHandle(long handle, string name)
    {
        if (handle == 0)
        {
            ThrowIfError();
            throw new OntologosException($"failed to create {name}");
        }

        return handle;
    }

    internal static void ThrowIfError()
    {
        var codePtr = NativeMethods.LastErrorCode();
        if (codePtr == IntPtr.Zero)
        {
            return;
        }

        var code = Marshal.PtrToStringUTF8(codePtr) ?? "Error";
        var msgPtr = NativeMethods.LastErrorMessage();
        var message = msgPtr != IntPtr.Zero ? Marshal.PtrToStringUTF8(msgPtr)! : code;
        NativeMethods.ClearLastError();

        throw code switch
        {
            "ParseError" => new ParseException(message),
            "ResourceLimitError" => new ResourceLimitException(message),
            "IncompleteReasoningError" => new IncompleteReasoningException(message),
            "OntologyConflictError" => new OntologyConflictException(message),
            _ => new OntologosException(message),
        };
    }

    internal static long ToNativeOptional(long? value) => value ?? -1L;
}
