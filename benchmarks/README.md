# Benchmarks

Benchmark ontology corpora for integration and conformance testing. Ontology files are **not** committed to the repository.

## Manifest

See [manifest.toml](manifest.toml) for the canonical list of ontologies, expected OWL profiles, source URLs, and licenses.

## Downloading

Create the data directory and download ontologies:

```bash
mkdir -p benchmarks/data

# Pizza (EL, ~800 axioms)
curl -L -o benchmarks/data/pizza.owl \
  "https://raw.githubusercontent.com/owlcs/pizza-ontology/master/pizza.owl"

# Family (RL, small)
curl -L -o benchmarks/data/family.owl \
  "https://raw.githubusercontent.com/owlcs/pizza-ontology/master/examples/family.owl"
```

GALEN, Gene Ontology, and SNOMED subsets require manual download or tooling (see manifest notes). A download script will be added in v0.2.

## Usage

Integration tests in v0.2+ will read `manifest.toml` and skip tests when `local_path` files are absent.
