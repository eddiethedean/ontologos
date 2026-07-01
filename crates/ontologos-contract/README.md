# ontologos-contract

PR-blocking **user API contract** tests for OntoLogos. All semantic checks route through [`ontologos_facade`](../ontologos-facade).

HermiT parity burndown lives in [`ontologos-conformance`](../ontologos-conformance) (nightly / release only).

```bash
cargo test -p ontologos-contract --release
```

Catalog sample IDs: [`data/case_ids.txt`](data/case_ids.txt).
