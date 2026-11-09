<!-- Modified for FolioRivet: release/source API and qualification synchronization; see docs/release/STATUS.md (2026-11-09). -->
# FolioRivet

**Published: 0.1.0a1 experimental alpha. Development checkout: unreleased improvements.**
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
> **FolioRivet 0.1.0a1 is an experimental public preview.**
> The published alpha is available on PyPI. The development checkout adds scalar fan-out,
> context analysis, native conditional bullets and outer-condition/immediate-repeat composition.
> These additions are implemented and locally qualified; they have not been published.

Development qualification passes all nine Windows CI steps, 35 template and nine image
regressions per platform, and installed CPython 3.12 wheel/CLI checks on Windows x64
and Ubuntu 22.04 x64. Scope and remaining Word/visual, hostile-input and full Linux CI
limitations are recorded in [release status](docs/release/STATUS.md).

---

## Features

The following describes the current checkout; unreleased additions are marked explicitly.

- **Scalar Values (`value:`)**: In-place text replacement preserving surrounding typography, bold/italic runs, and paragraph styling.
- **Table Row Repetition (`rows:`)**: Duplicates a prototype row for each item in a data list, maintaining cell formatting, borders, and column widths.
- **Picture Replacement (`image:`)**: Proportions raster images into an uncropped inline bounding box, maintaining aspect ratio, with optional fallback text.
- **Repeating Blocks (`repeat:`)**: Repeats entire multi-paragraph and multi-table document sections.
- **Conditional Sections (`if:`)**: Keeps complete wrapped sections when `True` and removes their content, headings and spacing paragraphs when `False`. **Unreleased:** supports native bullets and immediate `repeat:` children.
- **Scalar Fan-out (unreleased)**: Opt in with keyword-only `scalar_fanout=True` to fill repeated `value:` destinations in each effective scope, preserving each destination’s formatting. Strict uniqueness remains the default.
- **Context Analysis (`analyze_context`, unreleased)**: Reports active required, inactive, missing and unused paths plus scoped diagnostics, without decoding assets or writing output.
- **Static Template Inspection (`inspect_template`)**: Lists declared targets, locations, structural scopes and diagnostics without requiring data, including each scalar occurrence when fan-out is enabled.
- **Preflight Validation (`preflight_template`)**: Validates data, structural limits and required image assets without modifying the template or writing output.
- **Atomic Staging Safety**: Refuses to overwrite existing files (`create_new`).
- **Zero Server Office Dependency**: Native Rust core parses and writes standard OpenXML (`.docx`) headlessly.
- **Separation of Concerns**: Numerical totals, currency symbols, and business logic stay in your Python application; the engine stays strictly declarative.

---

## Installation

The published alpha provides native wheels for **CPython 3.12 x64** on Windows
and Linux with glibc 2.35+ (qualified on Ubuntu 22.04 LTS).

Inside your CPython 3.12 virtual environment, install the published version:

```bash
python -m pip install --only-binary=:all: foliorivet==0.1.0a1
```

To use the **unreleased features** documented below, build and install a wheel from
this checkout using the [source build instructions](docs/release/BUILD.md).
Installing the PyPI command above supplies the earlier alpha behavior.

Verify which installation is being imported:

```bash
python -c "import foliorivet; print('FolioRivet loaded from:', foliorivet.__file__)"
```

For platform prerequisites and troubleshooting, see the
[Installation Guide](docs/getting-started/installation.md).

---

## Quick Start

Use the included commercial quotation template with precomputed Python values:

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

The **development checkout** exports five public symbols. The published 0.1.0a1
has the first three functions and `TemplateError`; it does not include the new
`scalar_fanout` parameters or `analyze_context`.

- [`render_template(template_path, context_json, assets_dir, output_path, *, scalar_fanout=False) -> str`](docs/api/rendering.md): Render to a new staging DOCX path.
- [`inspect_template(template_path, *, scalar_fanout=False) -> dict[str, Any]`](docs/api/inspection.md): List target occurrences, locations, scopes and diagnostics.
- [`preflight_template(template_path, context_json, assets_dir="", *, scalar_fanout=False) -> dict[str, Any]`](docs/api/preflight.md): Validate context, structure, budgets and required image assets without writing output.
- [`analyze_context(template_path, context_json, *, scalar_fanout=False) -> dict[str, Any]`](bindings/python/README.md#development-checkout-alpha-feedback-options): Return `active_required_paths`, `inactive_paths`, `missing_paths`, `unused_supplied_paths` and scoped `diagnostics`. Malformed JSON or invalid template structure raises `TemplateError`; ordinary context findings are returned.
- [`TemplateError`](docs/api/exceptions.md): Template parsing, validation, resolution and rendering exception with operation, target and scope details.

With a development wheel installed, reuse `template` and `context` from the Quick Start:

```python
from foliorivet import analyze_context, inspect_template

inspection = inspect_template(template, scalar_fanout=True)
findings = analyze_context(template, context, scalar_fanout=True)
print("Missing:", findings["missing_paths"])
print("Unused:", findings["unused_supplied_paths"])
```

Fan-out applies only to scalar destinations. Images and variable wrappers retain
unique-target requirements; row and repeat-item fields stay local to each instance.
Missing or invalid conditions leave descendant activity undetermined. Analysis
reports active consumer conflicts and continues reporting unused fields after an
invalid context key. Preflight and rendering remain strict.

The command registry exposes `template.analyzeContext` and equivalent options on
structured inspect/preflight/population/render commands. Legacy raw-context
`template.preflight` and `template.populateScalars` stay strict. See the
[binding README](bindings/python/README.md#development-checkout-alpha-feedback-options)
for the current Python/Rust/CLI contract and [Types & Annotations](docs/api/types.md)
for context typing examples.

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
  - [`analyze_context` and development options](bindings/python/README.md#development-checkout-alpha-feedback-options)
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

| Component | Intended 0.1.0a1 Scope | Notes |
|:---|:---|:---|
| **Python Runtime** | CPython 3.12 x64 | 64-bit only; Python 3.11 and 3.13 not supported in 0.1.0a1 |
| **Operating System** | Windows x64 | Windows 10/11, Server 2019+ (MSVC runtime) |
| **Operating System** | Ubuntu 22.04 LTS x64 | Requires `glibc >= 2.35` (`manylinux_2_35_x86_64`) |
| **Interfaces** | Python bindings & CLI | Synchronous native extension & `foliorivet-cli` |
| **Document Format** | Microsoft Word DOCX | Standard OpenXML (ECMA-376 / ISO/IEC 29500) |
| **Microsoft Word** | Not required for generation | Server-side generation is 100% headless |

> [!NOTE]
> Generating OpenXML headlessly does not require Microsoft Word. However, independent visual review of newly authored templates should be conducted in Microsoft Word. See the [Compatibility Matrix](docs/reference/compatibility.md).

---

## Limitations

The development checkout supports the following bounded scope:

- **Bounded Nesting**: A top-level `if:` may contain immediate sibling `repeat:` children. Nested conditionals and repeats containing `if:` or `repeat:` remain unsupported. The published 0.1.0a1 rejects all nested variable blocks.
- **Lists**: Valid direct or style-inherited native bullets are supported in conditionals. Numbered/unresolved lists, and all numbering in repeats or repeated rows, remain unsupported. The published alpha also rejects conditional bullets.
- **No Vertical Merges in Repeated Tables**: Vertical cell merges (`w:vMerge`) anywhere in a table targeted by `rows:` are unsupported.
- **Uncropped Inline Images Only**: Replacement slots accept supported PNG/JPEG assets. Floating/cropped images, SVG and OLE slots are unsupported. Original media bytes and aspect ratio are preserved.
- **Overwrite Protection**: `render_template` requires a new output path in an existing directory. A failed filesystem write may leave a new incomplete staging file; retry using a fresh path.
- **Document-wide Bounds**: 1 MiB context JSON, 64 KiB per scalar and 1,024 template target occurrences. Separate ceilings of 1,000 repeated items and 1,000 data rows aggregate across independent and nested operations. Prototype-copy and expanded-output reservations share 16 MiB across the document; counters do not reset per item or conditional.
- **Image Reservation**: Encoded assets are limited to 16 MiB each. The 64 MiB conservative pixel/decode/conversion reservation is an allocation estimate, not measured process memory. A 3571 × 2366 image has an eight-byte-per-pixel floor of 67,591,888 bytes, above the 67,108,864-byte limit. Images are not resized or recompressed automatically.
- **Control Retention**: Document and conditional scalar controls may remain in the output. Variable wrappers and repeated-item controls are consumed.
- **Declarative Only**: FolioRivet does not evaluate expressions or format dates/currencies. Perform those transformations in Python.

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

- **Contributing**: See [CONTRIBUTING.md](CONTRIBUTING.md).
- **Issue Reporting**: See [Reporting a Preview Issue](docs/release/CONTRIBUTING.md) and [Support Policy](docs/reference/support.md).
- **Security Inquiries**: See [`SECURITY.md`](SECURITY.md).
