# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 1.0.x   | Yes       |
| 0.9.x   | Yes (security fixes only; upgrade to 1.0.x recommended) |
| 0.4.x – 0.8.x | No |
| < 0.4   | No        |

## Reporting a vulnerability

**Do not open public GitHub issues for security vulnerabilities.**

Report security issues privately via:

1. [GitHub Security Advisories](https://github.com/eddiethedean/ontologos/security/advisories/new) (preferred), or
2. Email the maintainer listed in crate metadata: odosmatthews@gmail.com

Include a description, reproduction steps, and impact assessment. You should receive a response within a reasonable timeframe.

## Security documentation

Input validation, default limits, and recommended practices for untrusted JSON and OWL files:

**[docs/security.md](docs/security.md)** · **[ontologos.readthedocs.io/security](https://ontologos.readthedocs.io/en/latest/security/)**

Topics covered:

- JSON snapshot limits and IRI validation (`max_literal_bytes`, entity/axiom caps)
- OWL file parse limits and path sandboxing (`load_ontology_in`)
- Parser single-thread contract (horned-owl mutex) for server embedders
- Conformance harness environment variables — do not set in production
- Format v1 rejection for untrusted JSON

## Disclosure

We follow coordinated disclosure. Credit will be given in the advisory unless you prefer to remain anonymous.
