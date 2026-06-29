# 30-minute evaluator playbook

Evaluate OntoLogos against ELK/reasonable without reading the whole codebase. Allow **30 minutes** plus download time.

## Prerequisites

- Rust **1.88+** (for CLI) or Python **3.10+**
- Clone optional; Family corpus is vendored on GitHub

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh   # Pizza only; Family is vendored
cargo build -p ontologos-cli --release
CLI=./target/release/ontologos
```

Or install CLI from git: `cargo install --git https://github.com/eddiethedean/ontologos ontologos-cli`

## Step 1 — Profile detection (2 min)

```bash
$CLI profile benchmarks/data/family.owl
```

**Expected:** `detected: RL` (text) or `"detected": "RL"` (JSON).

```bash
$CLI profile benchmarks/data/pizza.owl
```

**Expected:** `detected: DL` with diagnostics (Pizza mixes EL shapes with DL constructs).

## Step 2 — OWL RL saturation on Family (5 min)

```bash
$CLI --format json classify --profile rl benchmarks/data/family.owl
```

**Expected JSON keys:** `status: "materialized"`, `initial_axiom_count`, `final_axiom_count`, `inferred_axioms` > 0.

Family is the RL golden corpus — compare inferred counts with [benchmarks](../project/benchmarks.md) if curious.

## Step 3 — OWL EL taxonomy on Pizza (5 min)

Requires `./benchmarks/scripts/download.sh` first.

```bash
$CLI --format json classify --profile el benchmarks/data/pizza.owl
```

**Expected:** `status: "classified"`, `subsumption_count` > 0, `subsumptions` array of IRI pairs.

Golden reference: `crates/ontologos-conformance/golden/pizza-el-subsumptions.json` (maintainer CI).

## Step 4 — RDFS materialization (3 min)

```bash
$CLI materialize benchmarks/data/family.owl
```

**Expected:** materialization report with inferred axioms; same engine as `classify --profile rdfs`.

## Step 5 — DL preview smoke (5 min, optional)

```bash
$CLI classify --profile dl-preview benchmarks/data/family.owl
```

**Expected:** taxonomy output + preview warning on stderr. May hit `PreviewLimit` or `ResourceLimit` on larger DL ontologies — see [Preview profiles](preview-profiles.md).

## Step 6 — Python parity (5 min)

```bash
pip install ontologos
python - <<'PY'
from ontologos import Reasoner

r = Reasoner(path="benchmarks/data/family.owl", profile="rl")
report = r.classify()
assert report["inferred_axioms"] > 0
print("RL OK:", report["inferred_axioms"], "inferences")

r = Reasoner(path="benchmarks/data/family.owl", profile="el")
# Family is RL — force EL may error or return sparse taxonomy; use pizza for EL:
PY
```

For EL Python test, run from clone with Pizza downloaded:

```python
from ontologos import Reasoner
tax = Reasoner(path="benchmarks/data/pizza.owl", profile="el").classify()
assert tax["subsumption_count"] > 0
print("EL OK:", tax["subsumption_count"], "subsumptions")
```

## Step 7 — Honest scope check (5 min)

Read these before filing "missing feature" issues:

1. [Supported constructs](../reference/supported-constructs.md)
2. [Comparison](../comparison.md) — not HermiT replacement on arbitrary ontologies
3. [HermiT parity assessment](../internal/hermit-parity-honest-assessment.md) — what `parity_pct = 100%` measures
4. [Protégé axiom counts](protege-axiom-counts.md) — count mismatches are expected

## Pass / fail criteria

| Check | Pass |
|-------|------|
| Family RL infers axioms | `inferred_axioms > 0` |
| Pizza EL subsumptions | `subsumption_count > 0` |
| Profile detection | Family → RL, Pizza → DL |
| Python RL matches CLI | Same Family report shape |
| DL preview | Runs or fails with documented error type |

## Related

- [Comparison](../comparison.md)
- [Preview profiles](preview-profiles.md)
- [Troubleshooting](troubleshooting.md)
- [Benchmarks](../project/benchmarks.md)
