# ontologos (PyPI)

Python bindings for [OntoLogos](https://github.com/eddiethedean/ontologos) — a modular Rust ontology reasoner for OWL EL, RL, and RDFS.

**This is a pre-release placeholder (0.1.0).** Reasoning APIs are not yet functional:

- OWL file loading lands in v0.2
- Classification lands in v0.5
- Full Python ecosystem support is planned for v0.9 / 1.0

For the working Rust data model today, use [`ontologos-core` on crates.io](https://crates.io/crates/ontologos-core).

```bash
pip install ontologos
```

```python
import ontologos

print(ontologos.__version__)
# Reasoner and classify() will raise until parser and engines ship — see ROADMAP.md
```

See the [project README](https://github.com/eddiethedean/ontologos) and [ROADMAP](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md) for milestones.
