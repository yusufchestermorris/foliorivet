# Template Inspection (`inspect_template`)

FolioRivet's `inspect_template` function allows you to inspect any Microsoft Word document template statically, discovering all declared tags, structural scopes, and control properties before passing any input data.

---

## 1. Why Inspect Templates?

Static template inspection enables several powerful development workflows:

- **Schema Discovery**: Dynamically discover what fields a Word template requires without opening Microsoft Word.
- **Automated CI/CD Validation**: Run automated tests against template repositories to ensure designers haven't introduced unsupported controls, duplicate tags, or broken structures.
- **Dynamic Form Generation**: Build web UI form fields or API input validators dynamically based on the tags discovered in an uploaded Word template.

---

## 2. Using `inspect_template`

```python
from pathlib import Path
import json
from foliorivet import inspect_template

template_path = Path("templates/training-evaluation.docx")
inspection = inspect_template(template_path)

print(json.dumps(inspection, indent=2))
```

### Example Inspection Output

For a template containing scalar fields, a repeating block, and a conditional block:

```json
{
  "scope": "document",
  "targets": [
    {
      "tag": "value:model",
      "kind": "plainText",
      "story": "body",
      "path": [2],
      "supported": true,
      "diagnostic": null
    },
    {
      "tag": "repeat:classes",
      "kind": "repeat",
      "scope": "document",
      "first_block": 4,
      "last_block": 6,
      "supported": true
    },
    {
      "tag": "value:item.name",
      "kind": "plainText",
      "scope": "repeat:classes",
      "story": "body",
      "path": [0],
      "supported": true,
      "diagnostic": null
    },
    {
      "tag": "value:item.precision",
      "kind": "plainText",
      "scope": "repeat:classes",
      "story": "body",
      "path": [1],
      "supported": true,
      "diagnostic": null
    },
    {
      "tag": "if:has_evaluation",
      "kind": "optional",
      "scope": "document",
      "first_block": 7,
      "last_block": 8,
      "supported": true
    },
    {
      "tag": "value:evaluation",
      "kind": "plainText",
      "scope": "if:has_evaluation",
      "story": "body",
      "path": [1],
      "supported": true,
      "diagnostic": null
    }
  ]
}
```

---

## 3. Understanding the Inspection Fields

Every entry in the `targets` array provides structured metadata about a detected control:

| Field | Type | Description |
|:---|:---|:---|
| `tag` | `str` | The exact tag string read from the control's properties. |
| `kind` | `str` | Control type: `"plainText"`, `"richText"`, `"picture"`, `"row"`, `"repeat"`, or `"optional"`. |
| `scope` | `str` | Scope hierarchy: `"document"` for top-level targets, or the parent tag name (e.g. `"repeat:classes"` or `"rows:lines"`). |
| `story` | `str` | Document story location: `"body"` or part reference for headers/footers. |
| `path` | `list[int]` | Structural index path locating the block/inline in the engine's AST. |
| `supported` | `bool` | `True` if the control meets all structural rules; `False` if it violates a constraint. |
| `diagnostic` | `str \| null` | When `supported` is `False`, contains an actionable message explaining the violation. |

---

## 4. Catching Unsupported Controls in CI/CD

If a template designer accidentally creates an invalid control—such as an edit-locked control, a control spanning multiple paragraphs, or an unsupported control type (e.g. date picker or dropdown list)—`supported` evaluates to `False` and `diagnostic` explains how to fix it:

```python
inspection = inspect_template(Path("templates/draft.docx"))

unsupported = [t for t in inspection["targets"] if not t["supported"]]
if unsupported:
    print(f"Found {len(unsupported)} invalid template controls:")
    for target in unsupported:
        print(f"  • {target['tag']}: {target['diagnostic']}")
```

Example diagnostic output:

```text
Found 1 invalid template controls:
  • value:summary: nested or multi-paragraph scalar targets are unsupported
```

---

## 5. What Inspection Does NOT Guarantee

While `inspect_template` validates template syntax and AST structure:

- It **does not validate input values**: it does not verify whether your Python context dictionary has provided the required keys. (Use `preflight_template` for this).
- It **does not check asset files**: it cannot verify whether image files exist on disk without knowing the context.
- It **does not guarantee layout or page counts**: Word visual rendering and pagination are determined by the consumer's font layout engine.
