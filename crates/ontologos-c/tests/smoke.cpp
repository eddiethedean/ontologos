#include "ontologos.hpp"

#include <iostream>
#include <string>

int main() {
    if (ontologos::version() != "1.1.4") {
        std::cerr << "unexpected version\n";
        return 1;
    }

    ontologos::OntologyBuilder builder;
    builder.add_class("http://example.org/Pizza")
        .add_class("http://example.org/Food")
        .subclass_of("http://example.org/Pizza", "http://example.org/Food");
    ontologos::Ontology ontology = builder.build();
    ontologos::Reasoner reasoner(ontology, "el");
    const std::string report = reasoner.classify();
    if (report.find("\"status\":\"classified\"") == std::string::npos) {
        std::cerr << "classify report missing status: " << report << '\n';
        return 1;
    }

    std::cout << "C++ smoke test passed\n";
    return 0;
}
