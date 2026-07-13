#!/usr/bin/env bash
# Smoke-test Java bindings without Maven (JUnit tests use `mvn test` in CI).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
JAVA_ROOT="$ROOT/crates/ontologos-java/java"
SRC="$JAVA_ROOT/src/main/java"
OUT="$JAVA_ROOT/target/manual-classes"
NATIVE_DIR="$ROOT/target/release"

echo "Building native library..."
cargo build -p ontologos-jni --release --manifest-path "$ROOT/Cargo.toml"

mkdir -p "$OUT"
echo "Compiling Java sources..."
find "$SRC" -name '*.java' > "$JAVA_ROOT/target/sources.txt"
javac -d "$OUT" @"$JAVA_ROOT/target/sources.txt"

cat > "$OUT/dev/ontologos/ManualSmoke.java" <<'EOF'
package dev.ontologos;

public final class ManualSmoke {
    private ManualSmoke() {}

    public static void main(String[] args) {
        if (!"1.1.3".equals(Ontologos.version())) {
            throw new AssertionError("unexpected version: " + Ontologos.version());
        }
        try (OntologyBuilder builder = new OntologyBuilder()) {
            builder.addClass("http://example.org/Pizza");
            builder.addClass("http://example.org/Food");
            builder.subclassOf("http://example.org/Pizza", "http://example.org/Food");
            try (Ontology ontology = builder.build();
                    Reasoner reasoner = new Reasoner(ontology, "el")) {
                String report = reasoner.classify();
                if (!report.contains("\"status\":\"classified\"")) {
                    throw new AssertionError("classify report missing status: " + report);
                }
            }
        }
        System.out.println("Java smoke test passed");
    }
}
EOF

javac -d "$OUT" -cp "$OUT" "$OUT/dev/ontologos/ManualSmoke.java"
java -Djava.library.path="$NATIVE_DIR" -cp "$OUT" dev.ontologos.ManualSmoke
