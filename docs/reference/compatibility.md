# Compatibility Matrix

This document defines the verified platform and feature compatibility boundary for **FolioRivet 0.1.0**.

---

## 1. Operating System & Python Runtime

| Environment | Supported in 0.1.0 | Qualification Evidence | Notes |
|:---|:---:|:---|:---|
| **CPython 3.12 x64** | **YES** | Native wheel tests pass on Windows and Linux | Dedicated PyO3 native ABI wheel. |
| **Windows x64** | **YES** | Windows 10/11 & Server 2019+ qualification | Standard MSVC runtime environment. |
| **Ubuntu 22.04 LTS x64** | **YES** | Linux x64 candidate (`manylinux_2_35_x86_64`) | Requires `glibc >= 2.35`. |
| **CPython 3.11 or older** | NO | Not built or tested | 3.12-specific ABI bindings. |
| **CPython 3.13 or newer** | NO | Not built or tested | Targeted for future qualification. |
| **macOS (Apple Silicon / Intel)** | NO | Pending future qualification | Not provided in 0.1.0 preview. |
| **32-bit x86 Systems** | NO | Unsupported | Only 64-bit architectures supported. |
| **PyPy / Alternative Runtimes** | NO | Unsupported | Requires CPython C-API extensions. |

---

## 2. Microsoft Word Document Compatibility

FolioRivet generates standard OpenXML (`.docx`) archives conforming to ECMA-376 / ISO/IEC 29500:

| Feature / Element | Supported in 0.1.0 | Behavior & Preservation |
|:---|:---:|:---|
| **Standard Word Styles** | YES | Character and paragraph styles (`styles.xml`) are preserved without regression. |
| **Document Themes** | YES | Theme color and font scheme references (`theme1.xml`) remain intact. |
| **Headers & Footers** | YES | Header and footer stories (`header*.xml`, `footer*.xml`) support scalar targets. |
| **Tables (Static Layout)** | YES | Static tables, borders, cell margins, and horizontal spans (`gridSpan`) preserved. |
| **Repeated Table Rows** | YES | Single-row prototypes expanded via `rows:`. Prototype controls consumed. |
| **Inline Raster Pictures** | YES | PNG and JPEG images scaled to bounding box preserving aspect ratio. |
| **Repeating Body Blocks** | YES | Multi-paragraph and table sections repeated via `repeat:`. |
| **Conditional Sections** | YES | Blocks conditionally retained or cleanly removed via `if:`. |
| **Vertical Merges in Repeated Tables** | **NO** | `w:vMerge` anywhere in a `rows:` table causes a preflight error. |
| **Floating / Cropped Picture Slots** | **NO** | Only uncropped inline picture slots are supported for replacement. |
| **Nested Variable Blocks** | **NO** | `repeat:` inside `repeat:`, or `if:` inside `repeat:` is rejected. |
| **Legacy `.doc` Formats** | **NO** | Binary Word 97-2003 formats are rejected. |
| **Macro-Enabled `.docm`** | **NO** | Only standard `.docx` files are accepted. |

---

## 3. Visual Compatibility & Word Repair Review

> [!IMPORTANT]
> **Generation vs Visual Review**
> Generating a valid OpenXML ZIP package does not automatically guarantee visual perfection in every third-party viewer.
> FolioRivet's qualification suite includes automated Microsoft Word 16.0 open/save/re-open and export testing on generated reference examples to verify that Word opens generated documents **without repair warnings or corruption prompts**.
> When developing new custom templates, always perform an independent visual review in Microsoft Word on your target viewing platforms.
