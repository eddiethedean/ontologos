# ontologos (PyPI)

Python bindings for [OntoLogos](https://github.com/eddiethedean/ontologos) — a modular Rust ontology reasoner for OWL EL, RL, and RDFS.

**Alpha package (v0.7.0).** See the full guide: **[docs/guides/python.md](../../docs/guides/python.md)**.

| Capability | Rust v0.7.0 | Python v0.7.0 |
|------------|-----------|-------------|
| In-memory ontology model | Yes (`ontologos-core`) | No |
| OWL file loading | Yes (`ontologos-parser`) | Partial (`Reasoner(path)` loads only) |
| Profile detection | Yes (`ontologos-profile`) | Via `"auto"` only |
| RDFS materialization | Yes (`ontologos-rdfs`) | Partial (`Reasoner(path, profile="rdfs")`) |
| OWL RL saturation | Yes (`ontologos-rl`) | Partial (`Reasoner(path, profile="rl")`) |
| OWL EL taxonomy classification | Yes (`ontologos-el`) | Partial (`Reasoner(path, profile="el")` or `"auto"`) |
| Full Python API | — | v0.9 / 1.0 |

For working Rust APIs today, use [crates.io](https://crates.io/crates/ontologos-core) crates.

```bash
pip install ontologos
```

Pre-built wheels ship for Linux (x86_64, aarch64), macOS (Intel + Apple Silicon), and Windows (x64, ARM64). Python 3.10+ (`abi3` wheel).

```python
import ontologos

print(ontologos.__version__)

reasoner = ontologos.Reasoner("pizza.owl", profile="auto")
taxonomy = reasoner.classify()

reasoner = ontologos.Reasoner("family.owl", profile="rl")
reasoner.classify()
```

See the [project README](https://github.com/eddiethedean/ontologos) and [ROADMAP](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md) for milestones.
