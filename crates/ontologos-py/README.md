# ontologos (PyPI)

Python bindings for [OntoLogos](https://github.com/eddiethedean/ontologos) — a modular Rust ontology reasoner for OWL EL, RL, and RDFS.

**This is an alpha package (v0.4.0).** The package installs and reports its version. `Reasoner(path)` loads OWL files via the Rust parser. Pass `profile="rdfs"` for RDFS materialization or `profile="rl"` for OWL RL saturation via `classify()`. The default profile returns not-implemented until OWL EL taxonomy classification ships in v0.5.

| Capability | Rust v0.4 | Python |
|------------|-----------|--------|
| In-memory ontology model | Yes (`ontologos-core`) | No |
| OWL file loading | Yes (`ontologos-parser`) | Partial (`Reasoner(path)` loads only) |
| Profile detection | Yes (`ontologos-profile`) | No |
| RDFS materialization | Yes (`ontologos-rdfs`) | Partial (`Reasoner(path, profile="rdfs")`) |
| OWL RL saturation | Yes (`ontologos-rl`) | Partial (`Reasoner(path, profile="rl")`) |
| OWL EL taxonomy classification | No (v0.5) | No |
| Full Python API | — | v0.9 / 1.0 |

For working Rust APIs today, use [crates.io](https://crates.io/crates/ontologos-core) crates (`ontologos-core`, `ontologos-parser`, `ontologos-profile`, `ontologos-rdfs`, `ontologos-rl`).

```bash
pip install ontologos
```

```python
import ontologos

print(ontologos.__version__)

reasoner = ontologos.Reasoner("ontology.owl", profile="rdfs")
reasoner.classify()

reasoner = ontologos.Reasoner("family.owl", profile="rl")
reasoner.classify()
```

See the [project README](https://github.com/eddiethedean/ontologos) and [ROADMAP](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md) for milestones.
