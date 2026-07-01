## Summary

<!-- One paragraph: what changed and why -->

## Release channel

- [ ] Docs / examples only (no behavior change)
- [ ] Rust library change
- [ ] Python bindings
- [ ] CLI
- [ ] Conformance / benchmarks

## Checklist

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` (or `cargo clippy -p <crate>` for scoped PRs)
- [ ] Tests added or updated for behavior changes
- [ ] `./docs/scripts/check-doc-versions.sh` if user-facing docs or version pins changed

### Light PR path (docs or single-crate fix)

For documentation-only or isolated crate changes, these are usually enough:

```bash
cargo fmt --all -- --check
cargo test -p <affected-crate>
./docs/scripts/check-doc-versions.sh   # if docs/ or README changed
```

### Full CI parity (engine / conformance changes)

```bash
./benchmarks/scripts/download.sh
cargo test --workspace --locked
cargo test -p ontologos-conformance --release --locked
```

See [CONTRIBUTING.md](../CONTRIBUTING.md) for the complete maintainer checklist.

## Related issues

<!-- Fixes #123 -->
