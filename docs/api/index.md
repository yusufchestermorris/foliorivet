# API Reference

This section provides authoritative reference documentation for all public symbols exported by the `foliorivet` package.

The public interface consists of three core functions, one custom exception class, and standard type annotations:

---

## Public Exports (`__all__`)

| Symbol | Kind | Description |
|:---|:---|:---|
| [`render_template`](rendering.md) | Function | Synchronously render a template and context data into a new staging DOCX file. |
| [`inspect_template`](inspection.md) | Function | Statically inspect a DOCX template and return declared tags, structural scopes, and diagnostics. |
| [`preflight_template`](preflight.md) | Function | Validate template targets, scalar types, limits, and image assets against context data before rendering. |
| [`TemplateError`](exceptions.md) | Exception | Domain exception raised for all template parsing, validation, resolution, and rendering errors. |

---

## Import Conventions

All public symbols should be imported directly from the top-level `foliorivet` namespace:

```python
from foliorivet import (
    render_template,
    inspect_template,
    preflight_template,
    TemplateError,
)
```

---

## Type Aliases

FolioRivet defines standard type annotations in [`foliorivet.types`](types.md):

```python
from os import PathLike

Path = str | PathLike[str]
```

All functions accepting file or directory paths accept either standard `str` paths or `os.PathLike[str]` objects (such as `pathlib.Path`).
