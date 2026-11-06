# Quick Start

In this 5-minute tutorial, you will create a complete Python script to inspect, validate, and render a Microsoft Word document using FolioRivet.

---

## What We Are Building

We will generate a personalised welcome letter from a Word template. The template contains two scalar fields:
- `value:recipient.name`
- `value:recipient.role`

---

## 1. Project Directory Structure

Create a clean working folder with the following layout:

```text
hello-foliorivet/
├── templates/
│   └── welcome.docx       # Word template with Content Controls
├── generate.py            # Document generator script
└── staging/               # Target output directory (created by script)
```

---

## 2. The Python Generator Script (`generate.py`)

Create `generate.py` with the following code:

```python
"""FolioRivet Quickstart: Inspect, Preflight, and Render a DOCX Template."""

from pathlib import Path
import sys
from foliorivet import (
    TemplateError,
    inspect_template,
    preflight_template,
    render_template,
)


def main() -> None:
    # 1. Define paths using standard pathlib.Path objects
    base_dir = Path(__file__).resolve().parent
    template_path = base_dir / "templates" / "welcome.docx"
    staging_dir = base_dir / "staging"
    output_path = staging_dir / "welcome-alice.docx"

    # Ensure parent output directory exists (FolioRivet requires this)
    staging_dir.mkdir(parents=True, exist_ok=True)

    print(f"Loading template: {template_path.name}")

    # 2. Inspect the template to discover declared targets
    inspection = inspect_template(template_path)
    print("\n--- Template Inspection ---")
    for target in inspection.get("targets", []):
        tag = target["tag"]
        kind = target["kind"]
        supported = target["supported"]
        print(f"  • Found target: {tag:<30} [{kind}] (Supported: {supported})")

    # 3. Construct input context matching the template's required keys
    context = {
        "recipient": {
            "name": "Alice Morgan",
            "role": "Senior Research Scientist",
        }
    }

    # 4. Preflight data against template rules before making any changes
    print("\n--- Running Preflight Validation ---")
    try:
        summary = preflight_template(template_path, context)
        print(f"  ✓ Preflight passed! Validated {summary['scalar_count']} scalar fields.")
    except TemplateError as err:
        print(f"  ✗ Preflight failed: {err}", file=sys.stderr)
        sys.exit(1)

    # 5. Render the populated document into a new staging file
    print("\n--- Rendering Document ---")
    try:
        # Note: Pass "" for assets_dir when no images are required
        result_path = render_template(
            template_path=template_path,
            context_json=context,
            assets_dir="",
            output_path=output_path,
        )
        print(f"  ✓ Successfully rendered document to: {result_path}")
    except TemplateError as err:
        print(f"  ✗ Render failed: {err}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
```

---

## 3. Running the Generator

Execute the script inside your virtual environment:

```powershell
python generate.py
```

Expected output:

```text
Loading template: welcome.docx

--- Template Inspection ---
  • Found target: value:recipient.name           [plainText] (Supported: True)
  • Found target: value:recipient.role           [plainText] (Supported: True)

--- Running Preflight Validation ---
  ✓ Preflight passed! Validated 2 scalar fields.

--- Rendering Document ---
  ✓ Successfully rendered document to: .../staging/welcome-alice.docx
```

Open `staging/welcome-alice.docx` in Microsoft Word or any DOCX viewer. Notice that:
1. The placeholder text in the controls has been replaced with `"Alice Morgan"` and `"Senior Research Scientist"`.
2. All original font formatting, bold styling, paragraph spacing, and margins have been preserved.

---

## 4. Understanding Error Guarantees

FolioRivet is designed to fail loudly and safely:

### Overwrite Refusal
Try running `python generate.py` a second time immediately without deleting `staging/welcome-alice.docx`:

```text
  ✗ Render failed: save: target=".../staging/welcome-alice.docx", scope=document: choose a new staging path with an existing parent directory: The file exists. (os error 80)
```

FolioRivet **refuses to overwrite existing files**. This prevents race conditions and accidental data loss in concurrent automated environments.

### Missing Field Protection
If you omit `"role"` from the context:

```python
context = {"recipient": {"name": "Alice Morgan"}}
```

Running preflight immediately raises:

```text
TemplateError: preflight: target="value:recipient.role", scope=document: missing required value; supply this dotted field in context
```

Every declared template target is strictly required unless explicitly optional.

---

## Next Steps

Now that you've generated your first document, learn how to build the template yourself in [Your First Word Template](first-template.md).
