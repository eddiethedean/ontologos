using System.Runtime.InteropServices;

namespace Ontologos;

internal static partial class NativeMethods
{
    internal const string LibraryName = "ontologos_dotnet";

    [DllImport(LibraryName, EntryPoint = "ontologos_version")]
    internal static extern IntPtr VersionNative();

    [DllImport(LibraryName, EntryPoint = "ontologos_error_code_from_message")]
    internal static extern IntPtr ErrorCodeFromMessageNative(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string message);

    [DllImport(LibraryName, EntryPoint = "ontologos_last_error_code")]
    internal static extern IntPtr LastErrorCode();

    [DllImport(LibraryName, EntryPoint = "ontologos_last_error_message")]
    internal static extern IntPtr LastErrorMessage();

    [DllImport(LibraryName, EntryPoint = "ontologos_clear_last_error")]
    internal static extern void ClearLastError();

    [DllImport(LibraryName, EntryPoint = "ontologos_string_free")]
    internal static extern void StringFree(IntPtr value);

    [DllImport(LibraryName, EntryPoint = "ontologos_ontology_from_json")]
    internal static extern long OntologyFromJsonNative(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string json);

    [DllImport(LibraryName, EntryPoint = "ontologos_ontology_from_json_with_limits")]
    internal static extern long OntologyFromJsonWithLimitsNative(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string json,
        long maxJsonBytes,
        long maxEntities,
        long maxAxioms,
        long maxIriLen);

    [DllImport(LibraryName, EntryPoint = "ontologos_ontology_from_bytes")]
    internal static extern long OntologyFromBytesNative(byte[] data, nuint len);

    [DllImport(LibraryName, EntryPoint = "ontologos_ontology_from_bytes_lenient")]
    internal static extern long OntologyFromBytesLenientNative(byte[] data, nuint len);

    [DllImport(LibraryName, EntryPoint = "ontologos_ontology_from_text")]
    internal static extern long OntologyFromTextNative(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string text);

    [DllImport(LibraryName, EntryPoint = "ontologos_ontology_from_text_lenient")]
    internal static extern long OntologyFromTextLenientNative(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string text);

    [DllImport(LibraryName, EntryPoint = "ontologos_ontology_load")]
    internal static extern long OntologyLoadNative(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
        int lenient);

    [DllImport(LibraryName, EntryPoint = "ontologos_ontology_load_in")]
    internal static extern long OntologyLoadInNative(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string baseDir,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
        int lenient);

    [DllImport(LibraryName, EntryPoint = "ontologos_ontology_to_json")]
    internal static extern IntPtr OntologyToJsonNative(long handle);

    [DllImport(LibraryName, EntryPoint = "ontologos_ontology_axiom_count")]
    internal static extern long OntologyAxiomCountNative(long handle);

    [DllImport(LibraryName, EntryPoint = "ontologos_ontology_entity_count")]
    internal static extern long OntologyEntityCountNative(long handle);

    [DllImport(LibraryName, EntryPoint = "ontologos_ontology_close")]
    internal static extern void OntologyCloseNative(long handle);

    [DllImport(LibraryName, EntryPoint = "ontologos_builder_new")]
    internal static extern long BuilderNewNative();

    [DllImport(LibraryName, EntryPoint = "ontologos_builder_add_class")]
    internal static extern long BuilderAddClassNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string iri);

    [DllImport(LibraryName, EntryPoint = "ontologos_builder_individual")]
    internal static extern long BuilderIndividualNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string iri);

    [DllImport(LibraryName, EntryPoint = "ontologos_builder_object_property")]
    internal static extern long BuilderObjectPropertyNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string iri);

    [DllImport(LibraryName, EntryPoint = "ontologos_builder_subclass_of")]
    internal static extern long BuilderSubclassOfNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string subclass,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string superclass);

    [DllImport(LibraryName, EntryPoint = "ontologos_builder_subproperty_of")]
    internal static extern long BuilderSubpropertyOfNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string sub,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string sup);

    [DllImport(LibraryName, EntryPoint = "ontologos_builder_property_domain")]
    internal static extern long BuilderPropertyDomainNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string property,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string domain);

    [DllImport(LibraryName, EntryPoint = "ontologos_builder_property_range")]
    internal static extern long BuilderPropertyRangeNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string property,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string range);

    [DllImport(LibraryName, EntryPoint = "ontologos_builder_class_assertion")]
    internal static extern long BuilderClassAssertionNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string individual,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string classIri);

    [DllImport(LibraryName, EntryPoint = "ontologos_builder_object_property_assertion")]
    internal static extern long BuilderObjectPropertyAssertionNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string subject,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string property,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string obj);

    [DllImport(LibraryName, EntryPoint = "ontologos_builder_build")]
    internal static extern long BuilderBuildNative(long handle);

    [DllImport(LibraryName, EntryPoint = "ontologos_builder_close")]
    internal static extern void BuilderCloseNative(long handle);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_new")]
    internal static extern long ReasonerNewNative(
        long ontologyHandle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? profile,
        int incremental,
        long budgetSecs);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_from_path")]
    internal static extern long ReasonerFromPathNative(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? profile,
        int incremental,
        long budgetSecs,
        int lenient);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_load_in")]
    internal static extern long ReasonerLoadInNative(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string baseDir,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? profile,
        int incremental,
        long budgetSecs,
        int lenient);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_parse_meta")]
    internal static extern IntPtr ReasonerParseMetaNative(long handle);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_taxonomy")]
    internal static extern IntPtr ReasonerTaxonomyNative(long handle);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_classify")]
    internal static extern IntPtr ReasonerClassifyNative(long handle);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_explain")]
    internal static extern IntPtr ReasonerExplainNative(long handle);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_check_consistency")]
    internal static extern IntPtr ReasonerCheckConsistencyNative(long handle);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_is_consistent")]
    internal static extern int ReasonerIsConsistentNative(long handle);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_is_entailed")]
    internal static extern int ReasonerIsEntailedNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? sub,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? sup,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? individual,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? classIri,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? subject,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? property,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? obj);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_query")]
    internal static extern IntPtr ReasonerQueryNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string query);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_add_subclass_of")]
    internal static extern long ReasonerAddSubclassOfNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string subclass,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string superclass);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_remove_subclass_of")]
    internal static extern long ReasonerRemoveSubclassOfNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string subclass,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string superclass);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_add_axiom_json")]
    internal static extern long ReasonerAddAxiomJsonNative(
        long handle,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string axiomJson);

    [DllImport(LibraryName, EntryPoint = "ontologos_reasoner_close")]
    internal static extern void ReasonerCloseNative(long handle);
}
