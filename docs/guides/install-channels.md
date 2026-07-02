# Install and release channels

Single source of truth for **what you can install today** vs **what requires building from `main`**.

--8<-- "snippets/channel-banner.md"

## Quick decision

| I want to… | Install | Profiles for production |
|------------|---------|-------------------------|
| Embed in Rust (crates.io) | `ontologos-* = "0.9.0"` in `Cargo.toml` | **EL, RL, RDFS** |
| Use from Python (PyPI) | `pip install ontologos` | **EL, RL, RDFS** |
| OWL 2 DL or SWRL | Build from [`main`](https://github.com/eddiethedean/ontologos) with `"1.0.0"` pins | **DL, SWRL** (workspace; publish pending) |
| Run the CLI | `cargo install --git https://github.com/eddiethedean/ontologos ontologos-cli` or clone + build | Matches workspace version on `main` |
| Contribute / conformance | Clone + `./benchmarks/scripts/download.sh` | Full engine set |

**Default recommendation:** use **crates.io / PyPI 0.9.0** unless you explicitly need DL or SWRL.

## Published channel (0.9.0)

| Surface | Version | Install |
|---------|---------|---------|
| **crates.io** | 0.9.0 | `cargo add ontologos-core@0.9.0` (+ parser, facade, profile crates as needed) |
| **PyPI** | 0.9.0 | `pip install ontologos` |
| **docs.rs** | 0.9.0 | Links in [Reference](../reference/facade.md) reflect this channel |
| **Read the Docs** | latest | [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/) |

**Production-ready on this channel:** RDFS materialization, OWL RL saturation, OWL EL taxonomy, profile detection, explanations (EL full), incremental sessions.

**Not production-supported on PyPI 0.9.0:** `profile="dl"`, `profile="swrl"`, `profile="alc"`, `profile="dl-preview"` — may error or differ from `main`. See [Profile stability matrix](profile-stability.md).

## Workspace channel (`main`, 1.0.0)

| Surface | Version | Notes |
|---------|---------|-------|
| **Git workspace** | 1.0.0 (pre-release tag) | `Cargo.toml` at repo root; **v1.0.0 git tag not cut yet** |
| **HermiT gates** | Green on `main` | Gated conformance corpora only — see [Evaluator scope](evaluator-scope.md) |
| **PyPI / crates.io** | Still 0.9.0 | Publish pending — [Release checklist](../project/release-1.0-checklist.md) |

**Build from source:**

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh   # optional for Family; required for Pizza/HermiT
cargo build -p ontologos-cli --release
```

Pin all `ontologos-*` crates to `"1.0.0"` when depending on path or git.

**Adds over 0.9.0:** stable OWL 2 DL (`ontologos-dl`), DLSafe SWRL, full facade routing for DL/SWRL, JSON snapshot v3 writers, OWL QL CLI `query`.

## CLI install (not on crates.io)

The `ontologos-cli` binary is **not published** to crates.io. Install options:

```bash
# One-liner (requires Rust 1.88+)
cargo install --git https://github.com/eddiethedean/ontologos ontologos-cli

# Or from a clone
cargo build -p ontologos-cli --release
./target/release/ontologos --help
```

See [CLI reference](../reference/cli.md) and [Troubleshooting](troubleshooting.md#command-not-found-ontologos).

## API documentation by channel

| Channel | Rust API docs |
|---------|---------------|
| **0.9.0 published** | [docs.rs](https://docs.rs/ontologos-core/0.9.0) (linked from this site) |
| **`main` / 1.0.0** | `cargo doc --open -p ontologos-facade` from a clone |

Site reference pages (facade, errors, CLI) describe both channels where they differ.

## After v1.0.0 is published

When the annotated **v1.0.0** tag ships to crates.io and PyPI, follow [Post-1.0.0 documentation update](../project/post-1.0-doc-update.md) to sync version pins, banners, and docs.rs links.

## Related

- [Release status](../project/release-status.md) — live metrics and publish state
- [Profile stability matrix](profile-stability.md) — per-profile production guidance
- [Known limitations](known-limitations.md) — imports, mapping, axiom counts
- [Migration hub](../migration/index.md) — upgrade paths
- [Prerequisites](prerequisites.md) — Rust 1.88+, Python 3.10+
