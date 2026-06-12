"""Rewrite repository links for the MkDocs site (Read the Docs)."""

from __future__ import annotations

import re

_REPO = "https://github.com/eddiethedean/ontologos"

# Repo-root markdown files included under docs/project/.
_ROOT_SIBLING = {
    "ROADMAP.md": "roadmap.md",
    "SPEC.md": "spec.md",
    "CHANGELOG.md": "changelog.md",
    "CONTRIBUTING.md": "contributing.md",
    "FAQ.md": "faq.md",
    "SECURITY.md": "security-policy.md",
    "CODE_OF_CONDUCT.md": "code-of-conduct.md",
    "README.md": "overview.md",
    "PLAN.md": f"{_REPO}/blob/main/PLAN.md",
    "benchmarks/README.md": "benchmarks.md",
}

# Paths that should stay as GitHub links when referenced from project/ pages.
_GITHUB_TREE_PREFIXES = (
    "benchmarks/",
    "tests/",
    "crates/",
    ".github/",
)


def _github_blob(path: str) -> str:
    return f"{_REPO}/blob/main/{path}"


def _github_tree(path: str) -> str:
    return f"{_REPO}/tree/main/{path.rstrip('/')}"


def _rewrite_root_path(path: str) -> str:
    if path in _ROOT_SIBLING:
        target = _ROOT_SIBLING[path]
        if target.startswith("http"):
            return target
        return target
    for prefix in _GITHUB_TREE_PREFIXES:
        if path.startswith(prefix):
            if path.endswith("/"):
                return _github_tree(path)
            return _github_blob(path)
    if path == "Cargo.toml":
        return _github_blob(path)
    return _github_blob(path)


def _rewrite_research_links(markdown: str) -> str:
    """Research notes are excluded from the site; link to GitHub instead."""

    def to_blob(match: re.Match[str]) -> str:
        rel = match.group(1)
        return f"]({_github_blob(f'docs/internal/research/{rel}')})"

    return re.sub(
        r"\]\((?:\.\./)*(?:docs/)?internal/research/([^)]+)\)",
        to_blob,
        markdown,
    )


def _rewrite_project_page(markdown: str) -> str:
    # docs/foo.md → ../foo.md (relative from docs/project/)
    markdown = re.sub(r"\]\(docs/", "](../", markdown)

    # ../../path from repo root (included files)
    markdown = re.sub(
        r"\]\(\.\./\.\./([^)]+)\)",
        lambda m: f"]({_rewrite_root_path(m.group(1))})",
        markdown,
    )

    # Root-level sibling links without prefix
    for source, target in _ROOT_SIBLING.items():
        markdown = markdown.replace(f"]({source})", f"]({target})")

    # LICENSE files at repo root
    markdown = markdown.replace(
        "](LICENSE-APACHE)", f"]({_github_blob('LICENSE-APACHE')})"
    )
    markdown = markdown.replace("](LICENSE-MIT)", f"]({_github_blob('LICENSE-MIT')})")

    return markdown


def on_page_markdown(markdown: str, *, page, config, files) -> str:  # noqa: ARG001
    markdown = _rewrite_research_links(markdown)

    if page.file.src_uri.startswith("project/"):
        markdown = _rewrite_project_page(markdown)

    return markdown
