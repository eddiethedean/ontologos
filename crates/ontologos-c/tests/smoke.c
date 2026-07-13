#include "ontologos.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void die_on_error(void) {
    const char *code = ontologos_last_error_code();
    if (!code) {
        return;
    }
    const char *message = ontologos_last_error_message();
    fprintf(stderr, "%s: %s\n", code, message ? message : code);
    ontologos_clear_last_error();
    exit(1);
}

static char *take_string(char *ptr, int allow_null) {
    if (!ptr) {
        if (!allow_null) {
            die_on_error();
        }
        return NULL;
    }
    char *copy = strdup(ptr);
    ontologos_string_free(ptr);
    return copy;
}

static ontologos_handle_t require_handle(ontologos_handle_t handle, const char *what) {
    if (handle == 0) {
        die_on_error();
        fprintf(stderr, "failed to create %s\n", what);
        exit(1);
    }
    return handle;
}

int main(void) {
    char *version = take_string(ontologos_version(), 0);
    if (strcmp(version, "1.1.4") != 0) {
        fprintf(stderr, "unexpected version: %s\n", version);
        return 1;
    }
    free(version);

    ontologos_handle_t builder = require_handle(ontologos_builder_new(), "builder");
    builder = ontologos_builder_add_class(builder, "http://example.org/Pizza");
    builder = ontologos_builder_add_class(builder, "http://example.org/Food");
    builder = ontologos_builder_subclass_of(
        builder, "http://example.org/Pizza", "http://example.org/Food");
    ontologos_handle_t ontology = require_handle(ontologos_builder_build(builder), "ontology");

    ontologos_handle_t reasoner =
        require_handle(ontologos_reasoner_new(ontology, "el", 0, -1), "reasoner");
    char *report = take_string(ontologos_reasoner_classify(reasoner), 0);
    if (strstr(report, "\"status\":\"classified\"") == NULL) {
        fprintf(stderr, "classify report missing status: %s\n", report);
        return 1;
    }
    free(report);

    ontologos_reasoner_close(reasoner);
    ontologos_ontology_close(ontology);

    puts("C smoke test passed");
    return 0;
}
