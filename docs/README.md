# Documentation

Published site: **[ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/)**

| Audience | Entry |
| --- | --- |
| New users | [Start here](https://ontologos.readthedocs.io/en/latest/guides/start-here.html) · [Prerequisites](https://ontologos.readthedocs.io/en/latest/guides/prerequisites.html) · [FAQ](https://ontologos.readthedocs.io/en/latest/project/faq.html) |
| Rust integrators | [Choosing an API](https://ontologos.readthedocs.io/en/latest/guides/choosing-an-api.html) · [Getting started](https://ontologos.readthedocs.io/en/latest/getting-started/) |
| Python users | [Python guide](https://ontologos.readthedocs.io/en/latest/guides/python.html) |
| Evaluators | [Evaluator playbook](https://ontologos.readthedocs.io/en/latest/guides/evaluator-playbook.html) · [Comparison](https://ontologos.readthedocs.io/en/latest/comparison.html) |
| Contributors | [Contributing](https://ontologos.readthedocs.io/en/latest/project/contributing.html) · [Read the Docs build](readthedocs.md) |

## Build locally

```bash
python3 -m venv .venv-docs
source .venv-docs/bin/activate
pip install -r docs/requirements.txt
./docs/serve-site.sh
```

Open [http://127.0.0.1:8000/](http://127.0.0.1:8000/) for live reload.

Static output (matches CI):

```bash
./docs/build-site.sh
```

Output: `site/` (gitignored).

## Source layout

- `index.md` — documentation home (hero + documentation map)
- `getting-started/` — tutorials
- `guides/` — how-tos (`start-here.md`, `prerequisites.md`, API guides)
- `reference/` — CLI, errors, constructs
- `migration/` — version upgrade guides
- `project/` — FAQ, contributing, release status (wrappers for root files where noted)
- `stylesheets/custom.css` — landing hero and card styling
- `internal/` — maintainer-only (excluded from published site)

Configuration: [`mkdocs.yml`](../mkdocs.yml) · [`.readthedocs.yaml`](../.readthedocs.yaml)

## Adding pages

1. Add or edit Markdown under `docs/`.
2. Register the page in `mkdocs.yml` `nav`.
3. Run `./docs/build-site.sh` before pushing doc-only changes.

Root-level files (`README.md`, `ROADMAP.md`, `FAQ.md`, …) are linked or included via wrappers—edit the canonical source at the repo root when updating release plans or FAQ.
