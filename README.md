<!-- Modified for FolioRivet: native template fork changes; see docs/release/STATUS.md (2026-11-06). -->
# FolioRivet

**0.1.0a1 experimental preview — in preparation, not yet published.**
APIs and output behavior may change. Qualification covers documented fixtures only;
new templates need independent review. Production migration is not approved.
See [the manual release checklist](docs/release/PREVIEW.md).

**Headless Microsoft Word (DOCX) document generation from structured templates using Python or the command line.**

FolioRivet is an independent document-generation engine powered by a high-performance Rust core with native Python 3.12 bindings, based on WordCraft by the ArtCraft team.

Instead of writing fragile regex replacers (`{{ name }}` tags that corrupt when Word splits internal text runs) or automating desktop office suites (COM automation, LibreOffice background daemons), FolioRivet populates native Microsoft Word templates authored with standard **Content Controls**.

Business users and designers format templates directly in Microsoft Word, while developers populate them with clean JSON or Python dictionaries—without requiring Microsoft Word installed on the server.

---

## Release Status

> [!IMPORTANT]
> **FolioRivet 0.1.0 is an experimental public preview.**
> Supported template operations and platform matrices are deliberately bounded. Candidate wheels are prepared in `dist/` for qualification; public PyPI publication is pending final release clearance.

---

## Features

- **Scalar Values (`value:`)**: In-place text replacement preserving surrounding typography, bold/italic runs, and paragraph styling.
- **Table Row Repetition (`rows:`)**: Duplicates a prototype row for each item in a data list, maintaining cell formatting, borders, and column widths.
- **Picture Replacement (`image:`)**: Proportions raster images into an uncropped inline bounding box, maintaining aspect ratio, with optional fallback text.
- **Repeating Blocks (`repeat:`)**: Repeats entire multi-paragraph and multi-table document sections.
- **Conditional Sections (`if:`)**: Retains complete sections when `True`; excises sections cleanly without gaps when `False`.
- **Static Template Inspection (`inspect_template`)**: Discovers declared targets, structural scopes, and diagnostics without requiring data.
- **Preflight Validation (`preflight_template`)**: Validates data schemas, value limits, and image assets before touching disk.
- **Atomic Staging Safety**: Refuses to overwrite existing files (`create_new`), eliminating accidental data destruction.
- **Zero Server Office Dependency**: Native Rust core parses and writes standard OpenXML (`.docx`) headlessly.
- **Separation of Concerns**: Numerical totals, currency symbols, and business logic stay in your Python application; the engine stays strictly declarative.

---

## Installation

FolioRivet 0.1.0 provides precompiled native wheels for **CPython 3.12 x64** on Windows and Ubuntu 22.04 LTS.

Install from a qualified local wheel package:

=== "Windows (PowerShell)"

    ```powershell
    # Inside your CPython 3.12 virtual environment:
    python -m pip install --no-index dist/release-windows/foliorivet-0.1.0-cp312-cp312-win_amd64.whl
    ```

=== "Ubuntu / Linux (bash)"

    ```bash
    # Requires glibc >= 2.35 (e.g. Ubuntu 22.04 LTS):
    python3 -m pip install --no-index dist/release-linux/foliorivet-0.1.0-cp312-cp312-manylinux_2_35_x86_64.whl
    ```

Verify the installation:

```bash
python -c "import foliorivet; print('FolioRivet exports:', foliorivet.__all__)"
```

Expected output:
```text
FolioRivet exports: ['TemplateError', 'render_template', 'inspect_template', 'preflight_template']
```

> [!NOTE]
> When public distribution is officially enabled on PyPI, installation will simply be `pip install foliorivet`.
> For platform prerequisites and troubleshooting, see the [Installation Guide](docs/getting-started/installation.md).

---

## Quick Start

Generate a complete document in five lines of Python using the included commercial quotation template:

```python
from pathlib import Path
from foliorivet import preflight_template, render_template

template = Path("bindings/python/examples/invoice/template.docx")
output = Path("staging/quotation.docx")

context = {
    "quote_id": "SYN-0042",
    "customer": "Acme Innovations Ltd",
    "lines": [
        {
            "description": "Template Engineering",
            "quantity": "2",
            "unit_price": "GBP 75.00",
            "amount": "GBP 150.00"
        },
        {
            "description": "Documentation & Review",
            "quantity": "1",
            "unit_price": "GBP 120.00",
            "amount": "GBP 120.00"
        }
    ],
    "subtotal": "GBP 270.00",
    "tax": "GBP 54.00",
    "total": "GBP 324.00"
}

# 1. Validate data against template targets
preflight = preflight_template(template, context)
print(f"Preflight passed: {preflight['row_count']} lines validated.")

# 2. Render atomically to a new staging file
output.parent.mkdir(parents=True, exist_ok=True)
result = render_template(
    template_path=template,
    context_json=context,
    assets_dir="",
    output_path=output
)
print(f"Generated: {result}")
```

Open `staging/quotation.docx` in Microsoft Word to inspect the populated document.

For a full step-by-step walkthrough, see the [Quick Start Tutorial](docs/getting-started/quickstart.md).

---

## Microsoft Word Template Syntax

Templates are authored directly in Microsoft Word using native **Content Controls** located on Word's **Developer** tab.

FolioRivet matches targets strictly against the **Tag** property of each content control:

| Operation | Target Prefix | Purpose | Word Control Type |
|:---|:---|:---|:---|
| **Scalar Values** | `value:` | Replace a scalar text target | Plain Text or Rich Text |
| **Repeating Rows** | `rows:` | Duplicate a table data row | Rich Text wrapping a single row |
| **Pictures** | `image:` | Replace an inline picture slot | Picture Content Control |
| **Repeating Blocks** | `repeat:` | Repeat multi-block sections | Rich Text wrapping paragraphs/tables |
| **Conditional Blocks** | `if:` | Include or omit content | Rich Text wrapping paragraphs/tables |

### Authoring Rules at a Glance

1. **Tag, Not Title**: Enter your target (e.g. `value:customer`) in the control's **Tag** property. Word's Title is purely visual and ignored by the resolver.
2. **Unlock Contents**: Ensure **Contents cannot be edited** is unchecked so FolioRivet can replace the text.
3. **Scoped Addressing**: Inside repeating rows (`rows:`) and repeating blocks (`repeat:`), all scalar controls must use the `value:item.` prefix (e.g. `value:item.description`).
4. **Clean Dotted Paths**: Field keys must be non-empty dotted ASCII identifiers (`a-z`, `A-Z`, `0-9`, `_`).

For complete authoring instructions with UI diagrams, see the [Word Template Authoring Guide](docs/getting-started/first-template.md) and [Template Syntax Specification](docs/concepts/template-syntax.md).

---

## Python API

The `foliorivet` package exports four public symbols:

- [`render_template(template_path, context_json, assets_dir, output_path) -> str`](docs/api/rendering.md): Synchronously renders a template and context into a new staging DOCX file.
- [`inspect_template(template_path) -> dict[str, Any]`](docs/api/inspection.md): Statically inspects a template, returning declared targets, structural scopes, and diagnostic health.
- [`preflight_template(template_path, context_json, assets_dir="") -> dict[str, Any]`](docs/api/preflight.md): Validates context data, structural budgets, and image assets without modifying the template or writing output.
- [`TemplateError`](docs/api/exceptions.md): Domain exception providing actionable diagnostic messages formatted as `<operation>: target=<target>, scope=<scope>: <message>`.

Type definitions and Pydantic integration examples are documented in [Types & Annotations](docs/api/types.md).

---

## Examples

FolioRivet includes three complete, redistributable public-preview examples:

### 1. Training and Evaluation Report
Demonstrates machine learning evaluation summaries combining scalar model details, repeated class performance sections, and an optional benchmark appendix.
- **Tutorial**: [Training & Evaluation Guide](docs/examples/training-evaluation.md)
- **Source Files**: [`bindings/python/examples/training-evaluation/`](bindings/python/examples/training-evaluation)

### 2. Commercial Quotation & Invoice
Demonstrates quotation metadata, dynamic table row expansion, multiline cell descriptions, and Python-computed financial totals.
- **Tutorial**: [Invoice & Quotation Guide](docs/examples/invoice.md)
- **Source Files**: [`bindings/python/examples/invoice/`](bindings/python/examples/invoice)

### 3. Project Status Report
Demonstrates executive progress reporting combining repeated workstream multi-block sections with conditional review and sign-off blocks.
- **Tutorial**: [Project Status Guide](docs/examples/project-status.md)
- **Source Files**: [`bindings/python/examples/project-status/`](bindings/python/examples/project-status)

Looking for production patterns (atomic staging, optional charts, batch processing)? See [Practical Recipes](docs/examples/recipes.md).

---

## Documentation

Full documentation is available in the `docs/` directory:

- **Getting Started**:
  - [Overview](docs/getting-started/index.md)
  - [Installation Guide](docs/getting-started/installation.md)
  - [Quick Start Tutorial](docs/getting-started/quickstart.md)
  - [Word Template Authoring Guide](docs/getting-started/first-template.md)
- **Concepts**:
  - [Engine Architecture & Limits](docs/concepts/how-it-works.md)
  - [Template Syntax Grammar](docs/concepts/template-syntax.md)
  - [Scalar Values](docs/concepts/scalar-values.md)
  - [Table Rows](docs/concepts/table-rows.md)
  - [Pictures & Images](docs/concepts/pictures.md)
  - [Repeating Blocks](docs/concepts/repeating-blocks.md)
  - [Conditional Blocks](docs/concepts/conditional-blocks.md)
  - [Template Inspection](docs/concepts/inspection.md)
  - [Preflight Validation](docs/concepts/preflight.md)
  - [Error Handling](docs/concepts/error-handling.md)
- **API Reference**:
  - [API Overview](docs/api/index.md)
  - [`render_template`](docs/api/rendering.md)
  - [`inspect_template`](docs/api/inspection.md)
  - [`preflight_template`](docs/api/preflight.md)
  - [`TemplateError`](docs/api/exceptions.md)
  - [Types & Pydantic Integration](docs/api/types.md)
- **Reference**:
  - [Platform Compatibility Matrix](docs/reference/compatibility.md)
  - [Known Limitations](docs/reference/limitations.md)
  - [Troubleshooting & Diagnostics](docs/reference/troubleshooting.md)
  - [Frequently Asked Questions (FAQ)](docs/reference/faq.md)
  - [Support Policy](docs/reference/support.md)

### Building the Documentation Locally

To build and preview the documentation site locally with live reload:

```bash
pip install -r docs-requirements.txt
mkdocs serve
```

Then open `http://127.0.0.1:8000/` in your browser.

---

## Supported Platforms

| Component | Intended 0.1.0 Scope | Notes |
|:---|:---|:---|
| **Python Runtime** | CPython 3.12 x64 | 64-bit only; Python 3.11 and 3.13 not supported in 0.1.0 |
| **Operating System** | Windows x64 | Windows 10/11, Server 2019+ (MSVC runtime) |
| **Operating System** | Ubuntu 22.04 LTS x64 | Requires `glibc >= 2.35` (`manylinux_2_35_x86_64`) |
| **Interfaces** | Python bindings & CLI | Synchronous native extension & `foliorivet-cli` |
| **Document Format** | Microsoft Word DOCX | Standard OpenXML (ECMA-376 / ISO/IEC 29500) |
| **Microsoft Word** | Not required for generation | Server-side generation is 100% headless |

> [!NOTE]
> Generating OpenXML headlessly does not require Microsoft Word. However, independent visual review of newly authored templates should be conducted in Microsoft Word. See the [Compatibility Matrix](docs/reference/compatibility.md).

---

## Limitations

FolioRivet 0.1.0 enforces bounded structural guardrails to guarantee deterministic document generation:

- **No Nested Variable Blocks**: A `repeat:` block cannot contain an inner `repeat:` or `if:` block.
- **No Vertical Merges in Repeated Tables**: Vertical cell merges (`w:vMerge`) anywhere in a table targeted by `rows:` are unsupported.
- **Uncropped Inline Images Only**: Picture replacement slots must use inline text wrapping and be uncropped. Floating images, SVG, and OLE objects are not supported in 0.1.0 replacement slots.
- **Overwrite Protection**: `render_template` refuses to overwrite existing files. Output must target a new path in an existing directory.
- **Resource Bounds**: 1 MiB context JSON, 64 KiB per scalar, 1,000 table rows, 1,000 repeated items, and 16 MiB expansion budgets.
- **Declarative Only**: FolioRivet does not evaluate mathematical expressions or format dates/currencies. Perform all data transformations in Python.

For the exhaustive list of bounded constraints, see [Known Limitations](docs/reference/limitations.md).

---

## License and Attribution

FolioRivet is open-source software dual-licensed under:

- **MIT License** ([`LICENSE-MIT`](LICENSE-MIT))
- **Apache License, Version 2.0** ([`LICENSE-APACHE`](LICENSE-APACHE))

You may select either license at your option.

### Upstream Attribution
FolioRivet is an independent fork based on the WordCraft engine created by the ArtCraft team. Upstream copyright notices, licenses, and attribution obligations are preserved in:
- [`NOTICE`](NOTICE)
- [`ATTRIBUTION.md`](ATTRIBUTION.md)
- [`docs/release/STATUS.md`](docs/release/STATUS.md)

ArtCraft logo marks and product branding are subject to separate terms and are excluded from FolioRivet release packages. No endorsement by the upstream authors is implied.

---

## Contributing and Support

- **Issue Reporting**: See [Reporting a Preview Issue](docs/release/CONTRIBUTING.md) and [Support Policy](docs/reference/support.md).
- **Security Inquiries**: See [`SECURITY.md`](SECURITY.md).
