# Types & Annotations

FolioRivet provides PEP 561 compliance (`py.typed`) and standard type annotations across all public functions.

---

## 1. Type Aliases

### `Path`

```python
from os import PathLike

Path = str | PathLike[str]
```

All functions accepting template paths, asset directories, or destination files accept standard `str` paths or any object implementing the `os.PathLike[str]` protocol (such as `pathlib.Path`).

Example:

```python
from pathlib import Path
from foliorivet import render_template

# Both are fully supported and type-checked:
render_template("template.docx", {}, "", "output.docx")
render_template(Path("template.docx"), {}, Path(""), Path("output.docx"))
```

---

## 2. Context Input Contract

The `context_json` parameter across `render_template` and `preflight_template` is typed as:

```python
str | Mapping[str, Any]
```

- **`Mapping[str, Any]`**: A standard Python dictionary or immutable mapping. Nested values must be JSON-serialisable primitives (`str`, `int`, `float`, `bool`, `list`, or `dict`).
- **`str`**: A pre-encoded JSON text string. Useful when receiving raw JSON payloads directly from HTTP requests or message queues.

---

## 3. Integration with Pydantic

Because FolioRivet accepts standard mappings or JSON text strings, integrating with [Pydantic](https://docs.pydantic.dev/) models is seamless:

```python
from pathlib import Path
from pydantic import BaseModel, Field
from foliorivet import render_template

class LineItem(BaseModel):
    description: str
    quantity: int
    unit_price: str
    amount: str

class InvoiceData(BaseModel):
    customer: str
    quote_id: str
    lines: list[LineItem]
    subtotal: str
    tax: str
    total: str

# Instantiate validated Pydantic model
invoice = InvoiceData(
    customer="Apex Research",
    quote_id="INV-2026-042",
    lines=[
        LineItem(description="Analysis", quantity=1, unit_price="£500", amount="£500")
    ],
    subtotal="£500",
    tax="£100",
    total="£600"
)

# Option A: Pass as mapping using model_dump()
render_template(
    Path("template.docx"),
    invoice.model_dump(),
    "",
    Path("staging/invoice.docx")
)

# Option B: Pass pre-serialised JSON using model_dump_json()
render_template(
    Path("template.docx"),
    invoice.model_dump_json(),
    "",
    Path("staging/invoice.docx")
)
```

---

## 4. Static Type Checking with Mypy or Pyright

FolioRivet includes a `py.typed` marker file in its distribution, enabling static type checkers (such as `mypy` or `pyright`) to verify function arguments automatically:

```bash
mypy generate.py
```

Mypy will verify that:
- You pass valid `Path` and `Mapping` arguments.
- Return values from `inspect_template` and `preflight_template` are treated as `dict[str, Any]`.
- Return value from `render_template` is treated as `str`.
