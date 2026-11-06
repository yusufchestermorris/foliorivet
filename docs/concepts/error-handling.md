# Error Handling & Diagnostics

FolioRivet provides structured, actionable error reporting to make debugging template and data issues straightforward.

All domain-specific errors raised by the native engine are instances of `foliorivet.TemplateError`.

---

## 1. Anatomy of a `TemplateError`

Every `TemplateError` raised by FolioRivet adheres to a consistent, parsable structure:

```text
<operation>: target=<target_name>, scope=<scope>: <actionable_message>
```

For example:

```text
preflight: target="value:recipient.email", scope=document: missing required value; supply this dotted field in context
```

### Components of the Error Message

- **`<operation>`**: The pipeline phase where the failure occurred. Common operations include:
  - `resolve`: Control discovery and AST integrity verification.
  - `preflight`: Schema, type, limit, or asset validation.
  - `load`: Template file opening or OpenXML unpacking.
  - `populate`: In-memory AST data replacement.
  - `save`: Destination file verification and disk write.
  - `serialize`: JSON context encoding.
- **`target`**: The specific template tag, field path, or filename that triggered the error (e.g. `value:recipient.email`, `rows:line_items`, `charts/plot.png`).
- **`scope`**: The document scope where the error was detected:
  - `document`: Top-level document body.
  - `repeat:[tag]` or `rows:[tag]`: Inside a specific repeated item prototype (e.g. `scope=rows:lines[2]`).
- **`<message>`**: An actionable explanation of what rule was violated and how to resolve it.

---

## 2. Common Error Scenarios and Causes

### 1. Missing Required Value (`preflight`)
```text
TemplateError: preflight: target="value:client.name", scope=document: missing required value; supply this dotted field in context
```
- **Cause**: The template declares `value:client.name`, but the context dictionary does not provide `context["client"]["name"]`.
- **Fix**: Supply the missing field in your context mapping, or remove the control from the template if it is no longer needed.

### 2. Unused Input Field (`preflight`)
```text
TemplateError: preflight: target="value:client.tax_id", scope=document: missing required target; add this tag to the template or remove the unused input field
```
- **Cause**: Your context dictionary contains `context["client"]["tax_id"]`, but no matching `value:client.tax_id` tag exists in the template.
- **Fix**: Remove the unused key from the input dictionary, or add the corresponding control to the template.

### 3. File Already Exists (`save`)
```text
TemplateError: save: target="staging/report.docx", scope=document: choose a new staging path with an existing parent directory: The file exists. (os error 80)
```
- **Cause**: FolioRivet refuses to overwrite existing files.
- **Fix**: Generate to a unique or temporary staging path, or delete the destination file before rendering.

### 4. Ambiguous Target (`resolve`)
```text
TemplateError: resolve: target="value:model_id", scope=document: ambiguous target; tags must be unique across document stories
```
- **Cause**: Two content controls at the top level of the document share the exact same tag `value:model_id`.
- **Fix**: Ensure all document-level tags are distinct. (If you need to repeat fields, use `rows:` or `repeat:`).

### 5. Multiline Text in Single-Line Control (`preflight`)
```text
TemplateError: preflight: target="value:notes", scope=document: enable multiline on this plain text control to insert line breaks
```
- **Cause**: The input string contains `\n`, but the Word Plain Text Content Control has multiline disabled.
- **Fix**: In Word, open Content Control Properties and check **Allow carriage returns**, or replace it with a Rich Text Content Control.

### 6. Missing or Invalid Image (`preflight`)
```text
TemplateError: preflight: target="image:chart", scope=document: required asset "charts/fig1.png" is missing or unreadable
```
- **Cause**: The relative image path does not exist inside `assets_dir`.
- **Fix**: Verify that the image was generated and that `assets_dir` points to the correct folder. If the image is optional, supply `{"path": null, "required": false, "unavailable": "..."}`.

---

## 3. Python Error Handling Patterns

### Basic Safe Rendering Pattern

```python
from pathlib import Path
import sys
from foliorivet import TemplateError, preflight_template, render_template

template = Path("template.docx")
output = Path("staging/output.docx")
data = {"user": "Alice"}

try:
    # 1. Validate data first
    preflight_template(template, data)

    # 2. Render to staging
    render_template(template, data, "", output)

except TemplateError as exc:
    # Handle domain-specific template or data errors
    print(f"Document generation failed: {exc}", file=sys.stderr)
    # Clean up incomplete staging file if one was created
    if output.exists():
        output.unlink(missing_ok=True)

except TypeError as exc:
    # Handle incorrect argument types (e.g. passing a number where a Path was expected)
    print(f"Invalid Python arguments: {exc}", file=sys.stderr)

except Exception as exc:
    # Handle unexpected system errors
    print(f"Unexpected error: {exc}", file=sys.stderr)
```

### Staging and Atomic Promotion Pattern

In automated environments, write to a temporary staging file and atomically promote it only after rendering succeeds:

```python
import uuid
from pathlib import Path
from foliorivet import render_template

def generate_document(template_path: Path, data: dict, final_path: Path) -> Path:
    staging_dir = final_path.parent / ".staging"
    staging_dir.mkdir(parents=True, exist_ok=True)
    temp_file = staging_dir / f"tmp-{uuid.uuid4().hex}.docx"

    try:
        render_template(template_path, data, "", temp_file)
        # Atomically replace final destination
        temp_file.replace(final_path)
        return final_path
    except Exception:
        # Guarantee no partial files linger
        temp_file.unlink(missing_ok=True)
        raise
```
