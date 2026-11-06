# `TemplateError`

```python
class TemplateError(Exception):
    """Domain exception raised for all FolioRivet template operations."""
```

`TemplateError` is the single domain exception raised by FolioRivet for all template parsing, control resolution, data validation, asset decoding, and staging write failures.

It inherits directly from Python's built-in `Exception` (via PyO3's native exception mapping), allowing it to be caught with standard Python exception handling.

---

## Error Message Structure

Every `TemplateError` instance formats its error message with three standard diagnostic fields:

```text
<operation>: target=<target_identifier>, scope=<scope>: <message>
```

### Attributes and Message Fields

While `TemplateError` provides its diagnostic information via `str(err)`, the components represent:

| Field | Meaning | Example |
|:---|:---|:---|
| **`operation`** | The pipeline stage where failure occurred (`resolve`, `preflight`, `load`, `populate`, `save`, `serialize`). | `preflight` |
| **`target`** | The tag name, context path, or filename associated with the failure. | `value:item.amount` |
| **`scope`** | The document structural context (`document` or parent prototype scope). | `rows:line_items[1]` |
| **`message`** | Actionable text explaining the exact rule violation and remediation. | `missing required value` |

---

## Exception Hierarchy

```text
builtins.BaseException
 └── builtins.Exception
      └── foliorivet.TemplateError
```

Because `TemplateError` is a subclass of `Exception`:
- You can catch it specifically without intercepting unexpected system-level crashes (`KeyboardInterrupt`, `SystemExit`).
- Standard Python `TypeError` and `ValueError` remain distinct and indicate invalid Python calling arguments rather than template/data domain errors.

---

## Safe Handling Pattern

```python
from pathlib import Path
from foliorivet import TemplateError, render_template

template = Path("templates/invoice.docx")
output = Path("staging/output.docx")

try:
    render_template(template, {"customer": "Acme"}, "", output)

except TemplateError as err:
    # 1. Log domain-specific diagnostic message
    print(f"Template processing error: {err}")

    # 2. Extract error details if needed
    message = str(err)
    if "missing required value" in message:
        print("Action required: Check your input data dictionary.")
    elif "choose a new staging path" in message:
        print("Action required: Staging file already exists; choose a new path.")

    # 3. Clean up staging file if an incomplete write occurred
    if output.exists():
        output.unlink(missing_ok=True)

except TypeError as err:
    # Handle incorrect Python types passed to the function
    print(f"Programming error (wrong argument type): {err}")
```
