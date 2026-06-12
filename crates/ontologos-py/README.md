# ontologos (PyPI)

Python bindings for [OntoLogos](https://github.com/eddiethedean/ontologos) — a modular Rust ontology reasoner for OWL EL, RL, and RDFS.

**Alpha package (v0.5.0).** See the full guide: **[docs/guides/python.md](../../docs/guides/python.md)**.

> **Always pass `profile=`** — `Reasoner(path)` without a profile uses `"auto"`, which fails in v0.4.

| Capability | Rust v0.4 | Python v0.4 |
|------------|-----------|-------------|
| In-memory ontology model | Yes (`ontologos-core`) | No |
| OWL file loading | Yes (`ontologos-parser`) | Partial (`Reasoner(path)` loads only) |
| Profile detection | Yes (`ontologos-profile`) | No |
| RDFS materialization | Yes (`ontologos-rdfs`) | Partial (`Reasoner(path, profile="rdfs")`) |
| OWL RL saturation | Yes (`ontologos-rl`) | Partial (`Reasoner(path, profile="rl")`) |
| OWL EL taxonomy classification | No (v0.5) | No |
| Full Python API | — | v0.9 / 1.0 |

For working Rust APIs today, use [crates.io](https://crates.io/crates/ontologos-core) crates.

```bash
pip install ontologos
```

Pre-built wheels ship for Linux (x86_64, aarch64), macOS (Intel + Apple Silicon), and Windows (x64, ARM64). Python 3.10+ (`abi3` wheel).

```python
import ontologos

print(ontologos.__version__)

# Always set profile= — default "auto" fails in v0.4
reasoner = ontologos.Reasoner("ontology.owl", profile="rdfs")
reasoner.classify()

reasoner = ontologos.Reasoner("family.owl", profile="rl")
reasoner.classify()
```

See the [project README](https://github.com/eddiethedean/ontologos) and [ROADMAP](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md) for milestones.
