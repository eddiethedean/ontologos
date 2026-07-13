# 30-minute evaluator playbook

Evaluate OntoLogos against ELK/reasonable without reading the whole codebase. Allow **30 minutes** plus download time.

## Step 1 — Honest scope check (5 min)

Read these **before** running commands — they explain what pass/fail means:

1. [Evaluator scope](evaluator-scope.md) — what `parity_pct = 100%` measures (and does not)
2. [When not to use OntoLogos](when-not-to-use.md)
3. [Supported constructs](../reference/supported-constructs.md)
4. [Protégé axiom counts](protege-axiom-counts.md) — count mismatches are expected

## Prerequisites

- Rust **1.88+** (for CLI) or Python **3.10+**
- Clone optional; Family corpus is vendored on GitHub

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh   # Pizza; Family is vendored
cargo build -p ontologos-cli --release
CLI=./target/release/ontologos
```

Or install CLI from git: see [CLI installation](../getting-started/cli-install.md).

## Step 2 — Profile detection (2 min)

```bash
$CLI profile benchmarks/data/family.owl
```

**Expected:** `detected: RL` (text) or `"detected": "RL"` (JSON).

Download Pizza first (not bundled in pip/crates.io installs):

--8<-- "snippets/pizza-owl-download.md"

```bash
$CLI profile pizza.owl
```

**Expected:** `detected: DL` with diagnostics (Pizza mixes EL shapes with DL constructs).

## Step 3 — OWL RL saturation on Family (5 min)

```bash
$CLI --format json classify --profile rl benchmarks/data/family.owl
```

**Expected JSON keys:** `status: "classified"`, `initial_axiom_count`, `final_axiom_count`, `inferred_axioms` > 0.

Family is the RL golden corpus — compare inferred counts with [benchmarks](../project/benchmarks.md) if curious.

## Step 4 — OWL EL taxonomy on Pizza (5 min)

```bash
$CLI --format json classify --profile el pizza.owl
```

**Expected:** `status: "classified"`, `subsumption_count` > 0, `subsumptions` array of IRI pairs.

Golden reference: `crates/ontologos-conformance/golden/pizza-el-subsumptions.json` (maintainer CI).

## Step 5 — RDFS materialization (3 min)

```bash
$CLI materialize benchmarks/data/family.owl
```

**Expected:** materialization report with inferred axioms; same engine as `classify --profile rdfs`.

## Step 6 — OWL DL classification (5 min)

```bash
$CLI classify --profile dl --budget-secs 30 pizza.owl
```

**Expected:** taxonomy with `subsumption_count` > 0. For preview gating behavior only, use `--profile dl-preview` — see [Preview profiles](preview-profiles.md).

## Step 7 — Python parity (5 min)

```bash
pip install ontologos
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
python - <<'PY'
from ontologos import Reasoner

r = Reasoner(path="family.owl", profile="rl")
report = r.classify()
assert report["inferred_axioms"] > 0
print("RL OK:", report["inferred_axioms"], "inferences")
PY
```

For EL Python test (after Pizza download above):

```python
from ontologos import Reasoner
tax = Reasoner(path="pizza.owl", profile="el").classify()
assert tax["subsumption_count"] > 0
print("EL OK:", tax["subsumption_count"], "subsumptions")
```

## Step 8 — DL consistency and entailment (optional, 5 min)

```bash
export ONTOLOGOS_DL_BUDGET_SECS=30
$CLI consistent --profile dl --budget-secs 30 pizza.owl
```

Python (PyPI **1.1.3**):

```python
from ontologos import Reasoner

r = Reasoner(path="pizza.owl", profile="dl", budget_secs=30)
c = r.check_consistency()
assert c["complete"] and c["consistent"]
r.classify()
```

Contract tests (facade API surface, contributors):

```bash
cargo test -p ontologos-contract --release
```

See [Contract tests](../examples/contract-tests.md).

## Pass / fail criteria

| Check | Pass |
|-------|------|
| Family RL infers axioms | `inferred_axioms > 0` |
| Pizza EL subsumptions | `subsumption_count > 0` |
| Profile detection | Family → RL, Pizza → DL |
| Python RL matches CLI | Same Family report shape |
| DL stable on Pizza | `subsumption_count > 0` with `--profile dl` |

## Related

- [Comparison](../comparison.md)
- [Preview profiles](preview-profiles.md)
- [Troubleshooting](troubleshooting.md)
- [Benchmarks](../project/benchmarks.md)
