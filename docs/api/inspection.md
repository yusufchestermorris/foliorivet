# `inspect_template`

```python
def inspect_template(template_path: Path) -> dict[str, Any]
```

Statically inspect a Microsoft Word template and discover all declared tags, structural scopes, control types, and diagnostic warnings using the engine's native resolver.

Does not require input context data, does not resolve external image assets, and does not alter the template.

---

## Parameters

### `template_path`
- **Type**: `str | os.PathLike[str]`
- **Required**: Yes
- **Description**: The path to an existing Microsoft Word template file. Must have a `.docx` extension.

---

## Return Value

- **Type**: `dict[str, Any]`
- **Structure**:
  ```python
  {
      "scope": "document",
      "targets": [
          {
              "tag": str,              # e.g. "value:customer_name"
              "kind": str,             # "plainText", "richText", "picture", "row", "repeat", "optional"
              "scope": str,            # "document" or parent tag name
              "story": str,            # "body" or header/footer part reference
              "path": list[int],       # AST structural path
              "supported": bool,       # True if structurally valid
              "diagnostic": str | None # Error message if supported is False
          },
          ...
      ]
  }
  ```

---

## Exceptions

- **`foliorivet.TemplateError`**:
  Raised if:
  - The template file does not exist or cannot be read.
  - The file does not have a `.docx` extension.
  - The `.docx` archive is corrupted or contains invalid OpenXML.
  - The document contains unbalanced content controls (e.g. mismatched opening and closing tags).
- **`TypeError`**:
  Raised if `template_path` is not a string or `os.PathLike` object.

---

## Usage Example

```python
from pathlib import Path
from foliorivet import inspect_template

template = Path("templates/training-evaluation.docx")

inspection = inspect_template(template)

print(f"Template inspection scope: {inspection['scope']}")
print(f"Total declared targets: {len(inspection['targets'])}\n")

for target in inspection["targets"]:
    tag = target["tag"]
    kind = target["kind"]
    supported = target["supported"]
    scope = target.get("scope", "document")
    diagnostic = target.get("diagnostic")

    status = "✓ OK" if supported else f"✗ ERROR: {diagnostic}"
    print(f"  • {tag:<30} [{kind:<10}] Scope: {scope:<20} {status}")
```

---

## Use Cases

1. **Automated Schema Generation**: Inspect templates at runtime to build dynamic UI forms or JSON Schema validators.
2. **CI/CD Quality Gates**: Validate designer-submitted Word templates in pull requests to catch invalid structures before deployment.
3. **Template Discovery**: Allow end-users to see all configurable fields in a template library.
