# Example 3: Project Status Report

This tutorial walks through the **Project Status Report** example in `bindings/python/examples/project-status/`. It demonstrates combining repeated multi-block sections with conditional sections.

---

## Business Scenario

An enterprise engineering organisation generates executive status reports across technical initiatives:

- **Executive Header**: Project title and generation date.
- **Repeating Workstreams**: A section repeated for each initiative, containing a workstream title, status table, progress notes, and owner commentary.
- **Conditional Review Section**: An executive sign-off and review section that is included or omitted based on a boolean flag.

---

## 1. Directory Structure

Located at `bindings/python/examples/project-status/`:

```text
project-status/
├── template.docx         # Template containing repeat: and if: blocks
├── input.json            # Synthetic project status data
├── generate.py           # Generation script
├── README.md             # Execution notes
├── compatibility.json    # Verified compatibility audit record
└── generated.docx        # Pre-generated reference output
```

---

## 2. Template Structure in Microsoft Word

```text
===========================================================
PROJECT STATUS REPORT
===========================================================
Project: [value:project_name]

-----------------------------------------------------------
[repeat:workstreams]
  Workstream: [value:item.name]
  Status:     [value:item.progress]
  Owner Commentary: [Static text run remains editable]
[/repeat:workstreams]
-----------------------------------------------------------

[if:has_review]
  =========================================================
  EXECUTIVE REVIEW & SIGN-OFF
  This report has undergone formal peer and QA review.
  =========================================================
[/if:has_review]
===========================================================
```

### Control Elements

1. `value:project_name`: Top-level scalar control for the overall project name.
2. `repeat:workstreams`: Rich Text Content Control wrapping paragraphs 3 through 5.
   - `value:item.name`: Scalar Plain Text control inside `repeat:workstreams`.
   - `value:item.progress`: Scalar Plain Text control inside `repeat:workstreams`.
3. `if:has_review`: Rich Text Content Control wrapping paragraphs 6 through 7 (executive review heading and confirmation statement).

---

## 3. Input Data (`input.json`)

```json
{
  "project_name": "Example migration project",
  "workstreams": [
    {
      "name": "Template verification",
      "progress": "Complete for the tested scope"
    },
    {
      "name": "Native packaging",
      "progress": "Windows and Linux checks in progress"
    }
  ],
  "has_review": true
}
```

---

## 4. The Python Generator (`generate.py`)

```python
"""Generate Project Status Report."""

import json
from pathlib import Path
import sys
from foliorivet import preflight_template, render_template

here = Path(__file__).resolve().parent
template_path = here / "template.docx"
input_path = here / "input.json"
output_path = Path(sys.argv[1]) if len(sys.argv) > 1 else here / "output.docx"

context = json.loads(input_path.read_text(encoding="utf-8"))

# Preflight: validates repeated items and conditional inclusion
summary = preflight_template(template_path, context)
print("Preflight passed:", summary)

# Render: atomic generation
output_path.parent.mkdir(parents=True, exist_ok=True)
rendered = render_template(
    template_path=template_path,
    context_json=context,
    assets_dir="",
    output_path=output_path
)
print(f"Generated: {rendered}")
```

---

## 5. Execution and Verification

Run the generator:

```powershell
python bindings/python/examples/project-status/generate.py staging/status-report.docx
```

Expected output:

```text
Preflight passed: {'image_count': 0, 'included_optional_blocks': 1, 'repeated_items': 2, 'row_count': 0, 'save_verified': False, 'scalar_count': 5}
Generated: .../staging/status-report.docx
```

### Verification Highlights

Open `staging/status-report.docx` in Microsoft Word:

1. **Title**: Displays `"Example migration project"`.
2. **Duplicated Workstream Blocks**: Two distinct workstream sections are present:
   - Section 1: *Template verification* (*Complete for the tested scope*).
   - Section 2: *Native packaging* (*Windows and Linux checks in progress*).
3. **Editable Areas**: Surrounding manual commentary sections remain intact and editable by reviewers in Microsoft Word.
4. **Conditional Inclusion**: The *Executive Review & Sign-Off* block is included.
5. **No Wrapper Artifacts**: Word's content control bounding boxes have been cleanly removed, leaving standard Word headings and paragraphs.

---

## 6. Structural Rules Demonstrated

- Repeating blocks (`repeat:`) and conditional blocks (`if:`) can be positioned sequentially in the same document.
- Nested variable blocks (e.g. putting `if:` inside `repeat:workstreams`) are **not supported** in 0.1.0; they must be top-level body siblings.
- Workstream fields must use the scoped `value:item.*` syntax.
