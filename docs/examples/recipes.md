# Practical Recipes

This page collects practical, production-ready recipes for common document automation tasks with FolioRivet.

---

## Recipe 1: Atomic File Generation with Overwrite Protection

Because FolioRivet refuses to overwrite existing files, production services should write to a temporary staging file and atomically replace the destination path:

```python
import uuid
from pathlib import Path
from foliorivet import render_template, TemplateError

def safe_render(
    template: Path,
    context: dict,
    assets_dir: Path | str,
    target_output: Path
) -> Path:
    """Render a template atomically, safely replacing target_output if it exists."""
    staging_dir = target_output.parent / ".staging"
    staging_dir.mkdir(parents=True, exist_ok=True)

    # Use a unique temporary filename
    temp_path = staging_dir / f"tmp-{uuid.uuid4().hex}.docx"

    try:
        render_template(
            template_path=template,
            context_json=context,
            assets_dir=assets_dir,
            output_path=temp_path
        )
        # Atomically move temp file to destination (replaces existing file safely)
        temp_path.replace(target_output)
        return target_output
    except Exception:
        # Guarantee no orphaned staging files remain
        temp_path.unlink(missing_ok=True)
        raise
```

---

## Recipe 2: Discovering Required Context Keys Dynamically

Use `inspect_template` to extract all required context keys from a Word template:

```python
from pathlib import Path
from foliorivet import inspect_template

def get_required_fields(template_path: Path) -> dict[str, list[str]]:
    """Return top-level and scoped required field paths from a template."""
    inspection = inspect_template(template_path)
    fields = {"top_level": [], "repeating": []}

    for target in inspection.get("targets", []):
        tag = target["tag"]
        scope = target.get("scope", "document")

        if scope == "document":
            fields["top_level"].append(tag)
        else:
            fields["repeating"].append(f"{scope} -> {tag}")

    return fields

# Example usage:
discovered = get_required_fields(Path("templates/invoice.docx"))
print("Discovered Fields:")
print("  Top Level:", discovered["top_level"])
print("  Repeating:", discovered["repeating"])
```

---

## Recipe 3: Handling Optional Charts Gracefully

Handle scenarios where a chart image may or may not be available:

```python
from pathlib import Path
from foliorivet import render_template

def build_chart_payload(chart_filename: str | None, assets_folder: Path) -> dict:
    """Construct an image payload, providing fallback text if absent."""
    if chart_filename and (assets_folder / chart_filename).is_file():
        return {
            "path": chart_filename,
            "required": True
        }
    else:
        return {
            "path": None,
            "required": False,
            "unavailable": "Audit chart was not generated for this reporting period."
        }

# Usage:
assets_dir = Path("reports/assets")
chart_field = build_chart_payload("monthly_trend.png", assets_dir)

context = {
    "report_title": "Q3 Performance",
    "trend_chart": chart_field
}
```

---

## Recipe 4: Batch Generation with Error Isolation

Generate hundreds of personalised certificates or statements in a batch loop without letting one bad record crash the entire job:

```python
from pathlib import Path
import logging
from foliorivet import preflight_template, render_template, TemplateError

logger = logging.getLogger(__name__)

def process_batch(
    template: Path,
    records: list[dict],
    output_dir: Path
) -> tuple[int, int]:
    """Generate documents for multiple records, isolating individual failures."""
    output_dir.mkdir(parents=True, exist_ok=True)
    success_count = 0
    failure_count = 0

    for index, record in enumerate(records):
        doc_id = record.get("id", f"record-{index:04d}")
        output_file = output_dir / f"statement-{doc_id}.docx"

        # Skip already generated documents
        if output_file.exists():
            logger.info("Skipping already generated file: %s", output_file.name)
            success_count += 1
            continue

        try:
            # 1. Preflight individual record
            preflight_template(template, record)

            # 2. Render to unique path
            render_template(template, record, "", output_file)
            success_count += 1

        except TemplateError as err:
            logger.error("Failed generating %s: %s", doc_id, err)
            output_file.unlink(missing_ok=True)
            failure_count += 1

    return success_count, failure_count
```

---

## Recipe 5: Loading Context Safely from JSON Files

A robust pattern for loading external JSON files with strict error handling:

```python
import json
from pathlib import Path
from foliorivet import TemplateError, render_template

def render_from_json(template_path: Path, json_path: Path, output_path: Path) -> Path:
    """Render a DOCX template using an external JSON data file."""
    try:
        raw_text = json_path.read_text(encoding="utf-8")
        # Validate that JSON is an object
        parsed = json.loads(raw_text)
        if not isinstance(parsed, dict):
            raise ValueError(f"JSON root must be an object, got {type(parsed).__name__}")
    except Exception as exc:
        raise TemplateError(f"serialize: target='{json_path}', scope=document: {exc}") from exc

    output_path.parent.mkdir(parents=True, exist_ok=True)
    render_template(template_path, parsed, "", output_path)
    return output_path
```

---

## Recipe 6: Pre-validating Uploaded Templates in Web Applications

If users upload custom Word templates to your web application, validate them before saving them to your database:

```python
from pathlib import Path
from foliorivet import inspect_template, TemplateError

def validate_uploaded_template(temp_docx_path: Path) -> dict:
    """Validate that an uploaded DOCX file conforms to FolioRivet rules."""
    try:
        inspection = inspect_template(temp_docx_path)
    except TemplateError as err:
        return {"valid": False, "error": f"Invalid template: {err}"}

    # Check for unsupported controls
    unsupported = [t for t in inspection["targets"] if not t["supported"]]
    if unsupported:
        diagnostics = [f"{t['tag']}: {t.get('diagnostic')}" for t in unsupported]
        return {
            "valid": False,
            "error": "Template contains unsupported controls:\n" + "\n".join(diagnostics)
        }

    # Success: return valid target tags
    tags = [t["tag"] for t in inspection["targets"]]
    return {"valid": True, "tags": tags}
```
