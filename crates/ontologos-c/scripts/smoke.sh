#!/usr/bin/env bash
# Smoke-test C/C++ bindings. Uses CMake when available; otherwise compiles directly.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
CRATE="$ROOT/crates/ontologos-c"
INCLUDE="$CRATE/include"
NATIVE_DIR="$ROOT/target/release"

echo "Building native library..."
cargo build -p ontologos-c --release --manifest-path "$ROOT/Cargo.toml"

if command -v cmake >/dev/null 2>&1; then
    BUILD_DIR="$CRATE/build"
    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"
    cmake .. -DREPO_ROOT="$ROOT"
    cmake --build .
    ctest --output-on-failure
    exit 0
fi

echo "CMake not found; running manual smoke tests..."

if [[ "$(uname -s)" == "Darwin" ]]; then
    LIB_NAME="libontologos_c.dylib"
    LINK_FLAGS=(-Wl,-rpath,"$NATIVE_DIR")
elif [[ "$(uname -s)" == "Linux" ]]; then
    LIB_NAME="libontologos_c.so"
    LINK_FLAGS=(-Wl,-rpath,"$NATIVE_DIR")
else
    echo "manual smoke unsupported on this OS without CMake" >&2
    exit 1
fi

LIB_PATH="$NATIVE_DIR/$LIB_NAME"
if [[ ! -f "$LIB_PATH" ]]; then
    echo "native library not found: $LIB_PATH" >&2
    exit 1
fi

CC="${CC:-cc}"
CXX="${CXX:-c++}"
OUT="$CRATE/target/manual"

mkdir -p "$OUT"
"$CC" -std=c11 -I"$INCLUDE" "$CRATE/tests/smoke.c" -L"$NATIVE_DIR" -lontologos_c "${LINK_FLAGS[@]}" -o "$OUT/smoke_c"
"$OUT/smoke_c"

"$CXX" -std=c++17 -I"$INCLUDE" "$CRATE/tests/smoke.cpp" -L"$NATIVE_DIR" -lontologos_c "${LINK_FLAGS[@]}" -o "$OUT/smoke_cpp"
"$OUT/smoke_cpp"

echo "C/C++ smoke tests passed"
