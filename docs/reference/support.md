# Support & Licensing

---

## 1. Public Preview Support Scope

FolioRivet 0.1.0 is an **experimental public preview** release.

- **Intended Scope**: Server-side batch generation and automated service workflows targeting CPython 3.12 x64 on Windows and Ubuntu 22.04 LTS.
- **Reporting Issues**: When reporting issues or unexpected generation behavior, provide:
  1. The exact FolioRivet wheel version and operating system.
  2. A minimal reproducible Python snippet.
  3. The template inspection output (`inspect_template(template)`).
  4. The complete error message string including the operation and scope.

---

## 2. Licensing & Attribution

FolioRivet is distributed under the dual license terms of:

- **MIT License** ([`LICENSE-MIT`](https://github.com/foliorivet/foliorivet/blob/main/LICENSE-MIT))
- **Apache License, Version 2.0** ([`LICENSE-APACHE`](https://github.com/foliorivet/foliorivet/blob/main/LICENSE-APACHE))

You may choose either license at your option.

### Upstream Attribution
FolioRivet is an independent fork based upon the WordCraft engine created by the ArtCraft team. Upstream copyright notices, license texts, and third-party attribution are preserved in the repository root:
- `NOTICE`
- `ATTRIBUTION.md`
- `docs/release/LICENSING.md`

All product-facing ArtCraft logo marks and brand assets are excluded from FolioRivet release artifacts.

---

## 3. Contributing

Contributions to FolioRivet are welcome:

1. **Bug Reports & Issues**: Open an issue detailing the unexpected behavior, host environment, and minimal reproducible DOCX fixture.
2. **Pull Requests**:
   - Ensure all code conforms to existing formatting (`rustfmt`, `pyproject.toml`).
   - All public Python APIs must have matching tests in `bindings/python/tests/`.
   - Never commit proprietary font binaries, confidential templates, or untested platform claims.
