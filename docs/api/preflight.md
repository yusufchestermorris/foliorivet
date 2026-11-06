# `preflight_template`

```python
def preflight_template(
    template_path: Path,
    context_json: str | Mapping[str, Any],
    assets_dir: Path = ""
) -> dict[str, Any]
```

Validate template targets, input data types, structural resource budgets, and referenced external image assets prior to document generation.

Runs the complete in-memory preparation pipeline on a private document clone without writing output to disk or mutating the template.

---

## Parameters

### `template_path`
- **Type**: `str | os.PathLike[str]`
- **Required**: Yes
- **Description**: The path to an existing Microsoft Word template file. Must have a `.docx` extension.

### `context_json`
- **Type**: `str | Mapping[str, Any]`
- **Required**: Yes
- **Description**: The input data to validate against the template. Accepts either a Python mapping (`dict`) or a raw JSON string.

### `assets_dir`
- **Type**: `str | os.PathLike[str]`
- **Required**: No (defaults to `""`)
- **Description**: The directory where relative image paths referenced in `image:` targets are located. If no picture slots are present in the template, this can be omitted or passed as `""`.

---

## Return Value

- **Type**: `dict[str, Any]`
- **Structure**:
  ```python
  {
      "scalar_count": int,               # Number of validated scalar fields
      "row_count": int,                  # Number of rows to be expanded
      "image_count": int,                # Number of picture slots validated
      "repeated_items": int,             # (Optional) Count of repeated block items
      "included_optional_blocks": int,   # (Optional) Count of True conditional blocks
      "save_verified": False             # Always False during preflight
  }
  ```

---

## Exceptions

- **`foliorivet.TemplateError`**:
  Raised if any validation check fails:
  - Any required template target is missing from the context.
  - The context contains unused leaf keys not present in the template.
  - A scalar value is not a string, number, or boolean.
  - A single-line plain text control receives text containing `\n`.
  - A scalar value exceeds 64 KiB or contains invalid control/XML characters.
  - A table row prototype exceeds 1,000 rows or 16 MiB expansion budget.
  - An image file is missing, corrupt, outside `assets_dir`, or exceeds 16 MiB.
  - Image decoding exceeds the 64 MiB raster budget.
  - Context JSON exceeds 1 MiB or 32 levels of nesting.
- **`TypeError`**:
  Raised if argument types are invalid.

---

## Usage Example

```python
from pathlib import Path
from foliorivet import preflight_template, TemplateError

template = Path("templates/project-status.docx")
data = {
    "project_name": "Phoenix Migration",
    "workstreams": [
        {"name": "Security Audit", "progress": "Complete"},
        {"name": "Database Migration", "progress": "In Progress"}
    ],
    "has_review": True
}

try:
    summary = preflight_template(template, data)
    print("Preflight Successful!")
    print(f"  • Scalar fields: {summary['scalar_count']}")
    print(f"  • Repeated block items: {summary.get('repeated_items', 0)}")
    print(f"  • Included conditional sections: {summary.get('included_optional_blocks', 0)}")
except TemplateError as err:
    print(f"Preflight validation failed: {err}")
```

---

## Operational Notes

> [!NOTE]
> **Preflight Does Not Guarantee Disk Success**
> Preflight verifies that your data and assets are valid and that the document AST can be populated without error. It does not check destination disk permissions or available drive space. Always handle `TemplateError` during the final `render_template` call.
