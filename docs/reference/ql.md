# OWL QL API Reference

Conjunctive query answering via [`ontologos-ql`](https://docs.rs/ontologos-ql/0.9.0).

Requires a classified ontology (`Taxonomy` from EL or DL classification).

## Parse a query

```rust
use ontologos_ql::parse_conjunctive_query;

let query = parse_conjunctive_query("Type(?x, http://ex.org/A)")?;
```

Limits: `MAX_QUERY_LEN`, `MAX_QUERY_ATOMS` (see docs.rs).

## Answer a query

```rust
use ontologos_ql::{answer_query, parse_conjunctive_query};

let query = parse_conjunctive_query("Type(?x, http://ex.org/Person)")?;
let answers = answer_query(&ontology, &taxonomy, &query)?;
```

Each `QueryAnswer` binds query variables to entity IDs.

## CLI

```bash
ontologos query --query 'Type(?x, http://ex.org/A)' ontology.owl
```

Requires prior classification or uses profile routing internally — see [CLI reference](cli.md).

## Errors

| Variant | Cause |
|---------|-------|
| `UnknownClass` | Class IRI not in ontology |
| `Parse` | Invalid query syntax |
| `Query` | Wrapped hierarchy navigation error |

## Related

- [Query API (hierarchy)](query.md)
- [CLI `query` subcommand](cli.md)
