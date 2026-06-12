# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.4.x   | Yes       |
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

- JSON snapshot limits and IRI validation
- OWL file parse limits and path sandboxing (`load_ontology_in`)
- Format v1 rejection for untrusted JSON

## Disclosure

We follow coordinated disclosure. Credit will be given in the advisory unless you prefer to remain anonymous.
