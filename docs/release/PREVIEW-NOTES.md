# FolioRivet 0.1.0a1 — experimental public preview

DRAFT: publish only after PREVIEW.md gates pass.

FolioRivet is an experimental native DOCX template engine based on WordCraft by
the ArtCraft team. It supports bounded scalar, table-row, inline-picture and
conditional/repeating-block operations through Python and a separate CLI.
APIs and output behavior may change. New templates need independent structural
and Word review. This release is not approved for production migration.

Install with CPython 3.12 x64:

```text
python -m pip install --only-binary=:all: foliorivet==0.1.0a1
```

Windows x64 and Linux x64 glibc 2.35+ (initially Ubuntu 22.04) only.
Download and extract the separate CLI bundle from this release; run
`foliorivet-cli --help`. No desktop, macOS, ARM, abi3 or sdist support is promised.
Nested variable blocks, arbitrary expressions, floating/cropped picture slots
and reference-bearing repeat prototypes are unsupported.
Generation uses native code; matching prebuilt wheels need no Rust compiler.
Runtime/font requirements and final target-machine findings must be recorded
before publishing this draft. Report issues with platform/Python versions,
a minimal redistributable template, input and error message; omit private reports.

See COMPATIBILITY.md for limits and the included licenses for redistribution notices.
