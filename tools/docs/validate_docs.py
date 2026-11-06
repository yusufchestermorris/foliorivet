#!/usr/bin/env python3
"""Validate FolioRivet documentation examples, README, and build health.

Runs all documented Python workflows against the installed foliorivet candidate:
1. Public symbol introspection and docstrings.
2. Official example suites (training-evaluation, invoice, project-status).
3. Preflight and inspection verification.
4. Atomic overwrite refusal.
5. README links and executable code snippet verification.
6. Bidirectional validation rules (missing vs unused fields).
7. MkDocs documentation build verification.
"""

from __future__ import annotations

import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile

import foliorivet
from foliorivet import (
    TemplateError,
    inspect_template,
    preflight_template,
    render_template,
)


def log(msg: str) -> None:
    print(f"[docs-validator] {msg}")


def test_public_api() -> None:
    log("Verifying public API exports and signatures...")
    expected_all = ["TemplateError", "render_template", "inspect_template", "preflight_template"]
    assert foliorivet.__all__ == expected_all, f"Unexpected __all__: {foliorivet.__all__}"
    assert issubclass(TemplateError, Exception), "TemplateError must inherit from Exception"
    assert callable(render_template)
    assert callable(inspect_template)
    assert callable(preflight_template)
    log("  [OK] Public symbols verified successfully.")


def test_official_examples(repo_root: Path, staging_dir: Path) -> None:
    log("Testing official example suites against installed package...")
    examples_dir = repo_root / "bindings" / "python" / "examples"

    for example_name in ["training-evaluation", "invoice", "project-status"]:
        log(f"  Testing example: {example_name}")
        ex_dir = examples_dir / example_name
        template_path = ex_dir / "template.docx"
        input_path = ex_dir / "input.json"

        assert template_path.is_file(), f"Missing template: {template_path}"
        assert input_path.is_file(), f"Missing input data: {input_path}"

        # 1. Inspection
        inspection = inspect_template(template_path)
        assert inspection["scope"] == "document"
        assert len(inspection["targets"]) > 0
        assert all(t["supported"] for t in inspection["targets"])

        # 2. Preflight
        context = json.loads(input_path.read_text(encoding="utf-8"))
        preflight = preflight_template(template_path, context)
        assert preflight["save_verified"] is False
        assert preflight["scalar_count"] > 0

        # 3. Render
        output_file = staging_dir / f"{example_name}.docx"
        if output_file.exists():
            output_file.unlink()

        res = render_template(
            template_path=template_path,
            context_json=context,
            assets_dir="",
            output_path=output_file
        )
        assert Path(res).is_file(), f"Output not created: {res}"
        assert output_file.stat().st_size > 1000, "Output file too small"

        # 4. Overwrite refusal check
        try:
            render_template(template_path, context, "", output_file)
        except TemplateError as err:
            assert "choose a new staging path" in str(err)
        else:
            raise AssertionError(f"Expected overwrite refusal for {output_file}")

        log(f"    [OK] {example_name} passed inspection, preflight, render, and overwrite refusal.")


def test_readme_links_and_snippet(repo_root: Path, staging_dir: Path) -> None:
    log("Validating root README.md links and quick-start snippet...")
    readme_path = repo_root / "README.md"
    assert readme_path.is_file(), "README.md missing"
    content = readme_path.read_text(encoding="utf-8")

    # 1. Verify all relative Markdown links
    link_pattern = re.compile(r"\[.*?\]\((?!http)(.*?)\)")
    relative_links = link_pattern.findall(content)
    log(f"  Found {len(relative_links)} relative links in README.md")

    for link in relative_links:
        # Strip anchor if present
        target_path_str = link.split("#")[0]
        if not target_path_str:
            continue
        target_path = repo_root / target_path_str
        assert target_path.exists(), f"Broken README link: {link} -> {target_path} not found"

    log("  [OK] All relative links in README.md verified.")

    # 2. Extract and execute the quickstart python snippet
    py_pattern = re.compile(r"```python\n(.*?)\n```", re.DOTALL)
    matches = py_pattern.findall(content)
    assert matches, "No python code block found in README.md"
    snippet = matches[0]

    # Replace output path to write into staging_dir
    test_output_path = staging_dir / "readme_quickstart_output.docx"
    adapted_snippet = snippet.replace('Path("staging/quotation.docx")', f'Path(r"{test_output_path}")')

    # Execute snippet in isolated namespace
    namespace: dict = {}
    subprocess.run([sys.executable, "-I", "-c", adapted_snippet], cwd=repo_root, check=True)
    assert test_output_path.is_file(), "README snippet did not create expected output file"
    assert test_output_path.stat().st_size > 1000, "README snippet generated file too small"
    log("  [OK] README quickstart Python snippet executed and verified.")


def test_bidirectional_validation(repo_root: Path) -> None:
    log("Testing strict bidirectional field matching...")
    ex_dir = repo_root / "bindings" / "python" / "examples" / "invoice"
    template_path = ex_dir / "template.docx"
    valid_context = json.loads((ex_dir / "input.json").read_text(encoding="utf-8"))

    # Test missing required field
    missing_context = dict(valid_context)
    del missing_context["customer"]
    try:
        preflight_template(template_path, missing_context)
    except TemplateError as err:
        assert "missing required value" in str(err)
        log("  [OK] Correctly rejected missing required field.")
    else:
        raise AssertionError("Expected preflight error for missing field")

    # Test unexpected extra field
    extra_context = dict(valid_context)
    extra_context["unknown_field_123"] = "invalid"
    try:
        preflight_template(template_path, extra_context)
    except TemplateError as err:
        assert "missing required target" in str(err)
        log("  [OK] Correctly rejected unused extra field.")
    else:
        raise AssertionError("Expected preflight error for unused field")


def test_mkdocs_build(repo_root: Path) -> None:
    log("Verifying MkDocs documentation build with --strict...")
    result = subprocess.run(
        [sys.executable, "-m", "mkdocs", "build", "--strict"],
        cwd=str(repo_root),
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(result.stdout)
        print(result.stderr, file=sys.stderr)
        raise AssertionError(f"MkDocs build failed with exit code {result.returncode}")

    site_index = repo_root / "site" / "index.html"
    assert site_index.is_file(), "site/index.html was not generated"
    log("  [OK] MkDocs strict build succeeded; site/index.html verified.")


def main() -> None:
    repo_root = Path(__file__).resolve().parent.parent.parent
    log(f"Running documentation validation on repository: {repo_root}")

    with tempfile.TemporaryDirectory(prefix="foliorivet-docs-test-") as temp_dir:
        staging_dir = Path(temp_dir)
        test_public_api()
        test_official_examples(repo_root, staging_dir)
        test_readme_links_and_snippet(repo_root, staging_dir)
        test_bidirectional_validation(repo_root)
        test_mkdocs_build(repo_root)

    log("ALL DOCUMENTATION VALIDATION CHECKS PASSED SUCCESSFULLY!")


if __name__ == "__main__":
    main()
