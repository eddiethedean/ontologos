# Integration Tests

Workspace integration tests live in individual crates:

| Crate | Integration tests |
|-------|-------------------|
| `ontologos-core` | `crates/ontologos-core/tests/` |
| `ontologos-parser` | `crates/ontologos-parser/tests/` (mapping fixtures, manifest corpora) |
| `ontologos-profile` | `crates/ontologos-profile/tests/` |
| `ontologos-rdfs` | `crates/ontologos-rdfs/tests/` |
| `ontologos-cli` | `crates/ontologos-cli/tests/` |
| `ontologos-conformance` | HermiT-ported tests — [tests/hermit/](hermit/) |

Benchmark corpora (Pizza, Family) require `./benchmarks/scripts/download.sh` before running parser or CLI integration tests.

HermiT Tier-B tests require a local `HermiT/` checkout (gitignored). See [tests/hermit/README.md](hermit/README.md).

See [benchmarks/README.md](../benchmarks/README.md) for the full testing guide.
