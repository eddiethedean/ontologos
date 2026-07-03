# Read the Docs

Published documentation: **[ontologos.readthedocs.io](https://ontologos.readthedocs.io/)**

The site is built with [MkDocs Material](https://squidfunk.github.io/mkdocs-material/) from the [`docs/`](https://github.com/eddiethedean/ontologos/tree/main/docs) directory. Configuration lives at the repository root:

| File | Purpose |
|------|---------|
| [`.readthedocs.yaml`](https://github.com/eddiethedean/ontologos/blob/main/.readthedocs.yaml) | Read the Docs build config (MkDocs + Python 3.11) |
| [`mkdocs.yml`](https://github.com/eddiethedean/ontologos/blob/main/mkdocs.yml) | Site navigation, theme, plugins |
| [`docs/requirements.txt`](requirements.txt) | Python packages installed on RTD |

## Import the project on Read the Docs (maintainers)

1. Sign in at [readthedocs.org](https://readthedocs.org/) with GitHub.
2. **Import a Project** → select `eddiethedean/ontologos`.
3. RTD detects `.readthedocs.yaml` automatically.
4. Set the default branch to `main` and enable **Build pull requests** (optional).
5. After the first build, the site is live at `https://ontologos.readthedocs.io/`.

No secrets are required for a standard public MkDocs build.

## Build locally

```bash
python3 -m venv .venv-docs
source .venv-docs/bin/activate
pip install -r docs/requirements.txt
./docs/serve-site.sh
```

Open [http://127.0.0.1:8000/](http://127.0.0.1:8000/) for live reload.

Static output (matches CI — strict MkDocs, no warnings, version pins):

```bash
chmod +x docs/build-site.sh
./docs/build-site.sh
```

`build-site.sh` runs `mkdocs build --strict`, fails on any `WARNING` line, and runs `docs/scripts/check-doc-versions.sh` and `check-doc-snippets.sh`.

Read the Docs (`.readthedocs.yaml`) runs the same `./docs/build-site.sh` as CI (strict MkDocs, warning grep, version and snippet checks). The Rust facade snippet compile check is skipped on RTD (`SKIP_DOC_SNIPPET_CARGO=1` when `READTHEDOCS` is set).

Output directory: `site/` (gitignored).

## Adding pages

1. Add or edit Markdown under `docs/`.
2. Register the page in [`mkdocs.yml`](https://github.com/eddiethedean/ontologos/blob/main/mkdocs.yml) `nav` (required for discoverability).
3. Prefer links relative to `docs/` (e.g. `guides/python.md`), not `../` paths to the repository root.
4. Root-level files (`README.md`, `ROADMAP.md`, …) are included via wrappers or linked from [Roadmap summary](project/roadmap-summary.md) — edit the source file at the repo root when updating release plans.

## CI

GitHub Actions runs `./docs/build-site.sh` (strict validation, no Material banner) on every push/PR to validate the docs site builds cleanly.
