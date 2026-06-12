# ontologos (PyPI)

Python bindings for [OntoLogos](https://github.com/eddiethedean/ontologos) — a modular Rust ontology reasoner for OWL EL, RL, and RDFS.

**This is an alpha placeholder (v0.2.0).** The package installs and reports its version. `Reasoner(path)` loads OWL files via the Rust parser; `classify()` is not implemented until v0.5.

| Capability | Rust v0.2 | Python |
|------------|-----------|--------|
| In-memory ontology model | Yes (`ontologos-core`) | No |
| OWL file loading | Yes (`ontologos-parser`) | Partial (`Reasoner(path)` loads only) |
| Profile detection | Yes (`ontologos-profile`) | No |
| Classification | No (v0.5) | No |
| Full Python API | — | v0.9 / 1.0 |

For working Rust APIs today, use [crates.io](https://crates.io/crates/ontologos-core) crates (`ontologos-core`, `ontologos-parser`, `ontologos-profile`).

```bash
pip install ontologos
```

```python
import ontologos

print(ontologos.__version__)
```

See the [project README](https://github.com/eddiethedean/ontologos) and [ROADMAP](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md) for milestones.
