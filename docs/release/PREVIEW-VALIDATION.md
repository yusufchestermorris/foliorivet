# 0.1.0a1 local preview validation

6 November 2026. Local uncommitted candidates only; publication HOLD.

## Executed and passed

- Optimized Windows x64 CLI and CPython 3.12 wheel build, locked/offline inputs.
- Optimized Linux x64 CLI and CPython 3.12 manylinux_2_35 wheel build through WSL,
  locked/offline inputs and maturin compatibility audit.
- Fresh installed scalar, row/picture and block suites on both platforms;
  block suite reports 16 cases per platform.
- All three public examples on both platforms: Python/CLI ZIP-member agreement,
  template preservation and existing-output refusal. Example hashes agree across OSes.
- Twine 7.0.0 strict metadata validation for both wheels.
- Artifact assembly verifies candidate hashes, release profiles, notice inventories
  and bundled wheel notice contents. Negative check rejects stale 0.1.0 candidates
  before creating output. Two wheels, two CLI ZIPs and SHA256SUMS.txt assembled.
- Rust formatting, Git whitespace check, release-tool Python syntax, Python/Cargo
  version-lock consistency and inherited release workflow disabled checks.
- Review source export includes public documentation and excludes private evidence,
  caches and executable workflows. This is an exclusion check, not content clearance.

Commands and locations are in PREVIEW.md. Candidate build logs and manifests:
`dist/preview-0.1.0a1-windows/` and `dist/preview-0.1.0a1-linux/`.
Installed results: `dist/preview-tests-windows/` and `dist/preview-tests-linux/`.
Review artifacts: `dist/preview-upload/`. No upload command was executed.

## Limitations and remaining gates

No full Rust workspace suite, clippy, WASM, asset/layer gates, independent Word
review or clean-host acceptance was rerun in this change. Historical results retain
only their recorded scope. WSL is not an independent clean Linux machine. Windows
checks do not establish operation on a machine without Office.

Source/history privacy review, final legal/redistribution and resource clearance,
GitHub owner/project URLs, PyPI ownership, final reviewed commit/tag and final-source
rebuild remain pending. Public source links to private historical evidence still
need review/disposition. The source ZIP is for local review only.

WSL initially failed under the sandbox; approved escalated local execution worked.
The Linux wrapper needed LF line endings. Twine was installed in the disposable
Windows test environment after approved network access. No global configuration,
commits, remote changes, tags or publication were performed.

## Pre-commit review — 6 November 2026

User explicitly requested review and a local commit, overriding earlier local
no-commit instructions for this preparation snapshot. No push or publication.

Removed staged Python bytecode caches and added ignore rules. Updated fork URLs
and manual instructions. Corrected unit/help integration expectations for the
FolioRivet CLI identity, including the Unix-only invalid-Unicode expectation.
Windows CLI unit tests pass for both binaries (32 each). Unix-only assertion
change was inspected but not executed on Linux in this review.

Final `cargo xtask ci` on Windows, Rust 1.95.0, offline cached inputs and pinned
required craft-i18n: all nine steps PASS (fmt, clippy, workspace test,
i18n-report, language, doctest, assets, layers, wasm). WASM passes with unused
import/variable warnings. Full log retained privately in
`dist/checks-20261106-155426/ci-review-final.log`.

Staged Python/TOML syntax and credential-pattern checks pass (not a full secrets
audit). First-party whitespace passes. Exact copied dependency/license/data
texts retain upstream whitespace diagnostics; do not normalize those notices.

This is a preparation source commit, not a release-cleared snapshot. Example
binaries and other unreviewed handoff material remain untracked and excluded.
Some local test/tool workflows therefore still require the preserved private
fixtures or excluded example files. Final public examples, documentation-link
review, privacy/history and redistribution clearance, clean-host/independent Word
acceptance and final wheel rebuild remain required. Existing wheels predate the
GitHub URL metadata update. No new installed-wheel or Linux qualification is
claimed by this review.
