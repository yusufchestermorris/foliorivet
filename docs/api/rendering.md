# `render_template`

```python
def render_template(
    template_path: Path,
    context_json: str | Mapping[str, Any],
    assets_dir: Path,
    output_path: Path
) -> str
```

Populate supported scalar, row, picture, repeat, and conditional tags into a new staging DOCX document.

Every call executes synchronously inside a fresh, isolated Rust engine session. The original template is treated as strictly read-only, and the engine refuses to overwrite an existing destination file.

---

## Parameters

### `template_path`
- **Type**: `str | os.PathLike[str]`
- **Required**: Yes
- **Description**: The path to an existing Microsoft Word template file. Must have a `.docx` extension.

### `context_json`
- **Type**: `str | Mapping[str, Any]`
- **Required**: Yes
- **Description**: The input data used to populate template targets. Accepts either a standard Python mapping (such as a `dict`) or a pre-serialised JSON string. If a mapping is passed, it is encoded using `json.dumps(..., ensure_ascii=False, allow_nan=False)`.

### `assets_dir`
- **Type**: `str | os.PathLike[str]`
- **Required**: Yes
- **Description**: The root directory where relative image assets referenced by `image:` targets are resolved. If your template does not use `image:` controls, pass an empty string `""` or `Path("")`.

### `output_path`
- **Type**: `str | os.PathLike[str]`
- **Required**: Yes
- **Description**: The destination file path where the generated document will be written. Must have a `.docx` extension. The parent folder must already exist. The output file itself **must not already exist**.

---

## Return Value

- **Type**: `str`
- **Description**: Returns the destination path string as written.

---

## Exceptions

- **`foliorivet.TemplateError`**:
  Raised if:
  - `template_path` does not exist or is not a valid `.docx` file.
  - `context_json` exceeds 1 MiB or contains invalid JSON.
  - Any required target is missing from the context.
  - Context contains unused leaf keys not present in the template.
  - A scalar value exceeds 64 KiB or contains unsupported XML/control characters.
  - A multiline string is passed to a single-line plain text control.
  - An image asset is missing, corrupted, or exceeds size limits.
  - `output_path` already exists on disk.
  - `output_path`'s parent directory does not exist.
  - Disk write fails due to OS permission restrictions.
- **`TypeError`**:
  Raised if argument types are invalid (for example, passing an integer for `template_path`).

---

## Usage Example

```python
from pathlib import Path
from foliorivet import render_template, TemplateError

template = Path("templates/invoice.docx")
assets = Path("templates/assets")
output = Path("staging/invoice-101.docx")

data = {
    "customer": "Global Media Corp",
    "quote_id": "INV-2026-101",
    "lines": [
        {"description": "Content Licensing", "quantity": 1, "unit_price": "£1,200", "amount": "£1,200"}
    ],
    "subtotal": "£1,200",
    "tax": "£240",
    "total": "£1,440"
}

# Ensure staging directory exists
output.parent.mkdir(parents=True, exist_ok=True)

try:
    written_path = render_template(
        template_path=template,
        context_json=data,
        assets_dir=assets,
        output_path=output
    )
    print(f"Generated document at: {written_path}")
except TemplateError as err:
    print(f"Rendering failed: {err}")
    # Discard any incomplete staging file
    output.unlink(missing_ok=True)
```

---

## Operational Notes

> [!IMPORTANT]
> **Staging Output Safety**
> FolioRivet enforces `create_new(true)` semantics. If `output_path` already exists, rendering will fail immediately. If a disk write fails midway (e.g. disk full), an incomplete staging file may remain on disk; callers should catch `TemplateError` and delete the incomplete file.
