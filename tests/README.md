# Integration Tests

Workspace integration tests live in individual crates:

| Crate | Integration tests |
|-------|-------------------|
| `ontologos-core` | `crates/ontologos-core/tests/` |
| `ontologos-parser` | `crates/ontologos-parser/tests/` (mapping fixtures, manifest corpora) |
| `ontologos-profile` | `crates/ontologos-profile/tests/` |
| `ontologos-cli` | `crates/ontologos-cli/tests/` |

Benchmark corpora (Pizza, Family) require `./benchmarks/scripts/download.sh` before running parser or CLI integration tests. See [benchmarks/README.md](../benchmarks/README.md) for the full testing guide.
