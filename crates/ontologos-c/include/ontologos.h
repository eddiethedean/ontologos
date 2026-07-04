#ifndef ONTOLOGOS_H
#define ONTOLOGOS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Opaque handle for in-memory ontologies, builders, and reasoners. */
typedef int64_t ontologos_handle_t;

/** Package metadata and helpers. */
char *ontologos_version(void);
char *ontologos_error_code_from_message(const char *message);

/** Thread-local error state (valid until the next API call). */
const char *ontologos_last_error_code(void);
const char *ontologos_last_error_message(void);
void ontologos_clear_last_error(void);

/** Free strings returned by this library. */
void ontologos_string_free(char *value);

/** Ontology loaders and accessors. */
ontologos_handle_t ontologos_ontology_from_json(const char *json);
ontologos_handle_t ontologos_ontology_from_json_with_limits(
    const char *json,
    int64_t max_json_bytes,
    int64_t max_entities,
    int64_t max_axioms,
    int64_t max_iri_len);
ontologos_handle_t ontologos_ontology_from_bytes(const uint8_t *data, size_t len);
ontologos_handle_t ontologos_ontology_from_bytes_lenient(const uint8_t *data, size_t len);
ontologos_handle_t ontologos_ontology_from_text(const char *text);
ontologos_handle_t ontologos_ontology_from_text_lenient(const char *text);
ontologos_handle_t ontologos_ontology_load(const char *path, int lenient);
ontologos_handle_t ontologos_ontology_load_in(const char *base, const char *path, int lenient);
char *ontologos_ontology_to_json(ontologos_handle_t handle);
int64_t ontologos_ontology_axiom_count(ontologos_handle_t handle);
int64_t ontologos_ontology_entity_count(ontologos_handle_t handle);
void ontologos_ontology_close(ontologos_handle_t handle);

/** Ontology builder. */
ontologos_handle_t ontologos_builder_new(void);
ontologos_handle_t ontologos_builder_add_class(ontologos_handle_t handle, const char *iri);
ontologos_handle_t ontologos_builder_individual(ontologos_handle_t handle, const char *iri);
ontologos_handle_t ontologos_builder_object_property(ontologos_handle_t handle, const char *iri);
ontologos_handle_t ontologos_builder_subclass_of(
    ontologos_handle_t handle,
    const char *subclass,
    const char *superclass);
ontologos_handle_t ontologos_builder_subproperty_of(
    ontologos_handle_t handle,
    const char *sub,
    const char *sup);
ontologos_handle_t ontologos_builder_property_domain(
    ontologos_handle_t handle,
    const char *property,
    const char *domain);
ontologos_handle_t ontologos_builder_property_range(
    ontologos_handle_t handle,
    const char *property,
    const char *range);
ontologos_handle_t ontologos_builder_class_assertion(
    ontologos_handle_t handle,
    const char *individual,
    const char *class_iri);
ontologos_handle_t ontologos_builder_object_property_assertion(
    ontologos_handle_t handle,
    const char *subject,
    const char *property,
    const char *object);
ontologos_handle_t ontologos_builder_build(ontologos_handle_t handle);
void ontologos_builder_close(ontologos_handle_t handle);

/** Reasoner. Pass -1 for optional numeric limits; NULL for optional strings. */
ontologos_handle_t ontologos_reasoner_new(
    ontologos_handle_t ontology_handle,
    const char *profile,
    int incremental,
    int64_t budget_secs);
ontologos_handle_t ontologos_reasoner_from_path(
    const char *path,
    const char *profile,
    int incremental,
    int64_t budget_secs,
    int lenient);
ontologos_handle_t ontologos_reasoner_load_in(
    const char *base,
    const char *path,
    const char *profile,
    int incremental,
    int64_t budget_secs,
    int lenient);
char *ontologos_reasoner_parse_meta(ontologos_handle_t handle);
char *ontologos_reasoner_taxonomy(ontologos_handle_t handle);
char *ontologos_reasoner_classify(ontologos_handle_t handle);
char *ontologos_reasoner_explain(ontologos_handle_t handle);
char *ontologos_reasoner_check_consistency(ontologos_handle_t handle);
int ontologos_reasoner_is_consistent(ontologos_handle_t handle);
int ontologos_reasoner_is_entailed(
    ontologos_handle_t handle,
    const char *sub,
    const char *sup,
    const char *individual,
    const char *class_iri,
    const char *subject,
    const char *property,
    const char *object);
char *ontologos_reasoner_query(ontologos_handle_t handle, const char *query);
ontologos_handle_t ontologos_reasoner_add_subclass_of(
    ontologos_handle_t handle,
    const char *subclass,
    const char *superclass);
ontologos_handle_t ontologos_reasoner_remove_subclass_of(
    ontologos_handle_t handle,
    const char *subclass,
    const char *superclass);
ontologos_handle_t ontologos_reasoner_add_axiom_json(
    ontologos_handle_t handle,
    const char *axiom_json);
void ontologos_reasoner_close(ontologos_handle_t handle);

#ifdef __cplusplus
}
#endif

#endif /* ONTOLOGOS_H */
