# Protégé vs OntoLogos Axiom Counts

Evaluators often compare `ontology.axiom_count()` to Protégé's logical axiom count and conclude the parser is broken. **It is not** — OntoLogos stores a **subset** of OWL axioms in its core model.

## What each count measures

| Source | Count | Meaning |
|--------|-------|---------|
| Protégé | Logical axiom count | All OWL logical axioms in the loaded ontology |
| OntoLogos `axiom_count()` | Mapped axioms | Axioms **stored in core** after parser mapping |
| `parse_meta.logical_axiom_count` | Mapped + skipped | Logical components the parser recognized |
| `parse_meta.skipped_axiom_count` | Skipped only | Recognized but not stored (complex expressions, etc.) |

Relationship: `logical_axiom_count ≈ mapped_axiom_count + skipped_axiom_count` (per load).

## Example: Family ontology

After loading `benchmarks/data/family.owl`:

```rust
use ontologos_parser::load_ontology;

let ontology = load_ontology(path)?;
println!("axiom_count(): {}", ontology.axiom_count());

if let Some(meta) = ontology.parse_meta() {
    println!("mapped: {}", meta.mapped_axiom_count);
    println!("skipped: {}", meta.skipped_axiom_count);
    println!("logical (mapped+skipped): {}", meta.logical_axiom_count);
    for warning in meta.warnings.iter().take(3) {
        println!("warning: {warning}");
    }
}
```

Typical output (approximate):

```text
axiom_count(): 57
mapped: 57
skipped: 12
logical (mapped+skipped): 69
warning: skipped SubClassOf with complex superclass: ObjectIntersectionOf(...)
...
```

Protégé may report a higher logical axiom count because it counts axioms OntoLogos **scans** for profile detection but does **not map** into core.

## Example: Pizza ontology

Pizza is a stress corpus. After `./benchmarks/scripts/download.sh`:

```text
axiom_count(): 658        # mapper output (manifest target)
skipped_axiom_count: ...   # complex class expressions, data properties, etc.
detected profile: DL      # mapped axioms mix EL and RL-forbidden shapes
```

Manifest `axiom_count_approx` values in [`benchmarks/manifest.toml`](https://github.com/eddiethedean/ontologos/blob/main/benchmarks/manifest.toml) are **mapper output**, not Protégé totals.

## What is mapped vs skipped

See [Supported constructs](../reference/supported-constructs.md). Common skips:

- `ObjectIntersectionOf`, `ObjectUnionOf`, `ObjectComplementOf`
- `ObjectAllValuesFrom`, cardinalities, nominals
- Data property assertions and most datatype axioms
- SWRL rules and annotations (neutral for reasoning)

Named ABox axioms (`ClassAssertion`, `ObjectPropertyAssertion`, `SameIndividual`, `DifferentIndividuals`) **are** mapped in v0.4.

## What to do

1. Compare **`mapped_axiom_count`** to benchmark manifest targets, not Protégé.
2. Read **`parse_meta.warnings`** for skipped shapes.
3. Use **`logical_axiom_count`** to see how much the parser recognized vs stored.
4. For profile detection, trust **`profile_constructs`** (mapped TBox only).

## Related

- [Troubleshooting](troubleshooting.md)
- [Supported constructs](../reference/supported-constructs.md)
- [Load an OWL file](../getting-started/load-owl-file.md)
