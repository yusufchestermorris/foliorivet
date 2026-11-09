# Experimental alpha status

FolioRivet 0.1.0a1 is published on [PyPI](https://pypi.org/project/foliorivet/0.1.0a1/).
Source fork: https://github.com/yusufchestermorris/foliorivet.
The published wheels were built from ad194fb5 (WordCraft-based Rust engine plus
FolioRivet native templating changes). This source cleanup does not replace those
artifacts or move the release tag.

CPython 3.12 x64: Windows and Linux glibc 2.35+ (initial Linux qualification on
Ubuntu 22.04/WSL2). No macOS, ARM, desktop, abi3 or sdist promise.

Recorded checks: all nine Windows cargo xtask ci steps passed. Rebuilt Windows/Linux
wheels passed installed regressions and three Python/CLI examples. TestPyPI and
production PyPI Linux installations passed those examples, with matching hashes.
WSL is not an independent clean host. Known historical Linux full-workspace UI and
timing failures are not represented as a passing Linux full-CI result.

Clean-host/Windows-without-Office qualification, independent Word review, measured
onboarding and production integration are deferred to adoption. APIs and behavior
may change. New templates need their own structural and visual review.
Private execution logs and historical plans are retained locally, not required by
public builds. See BUILD.md, COMPATIBILITY.md and LICENSING.md.

## Public source cleanup checks

All nine Windows CI steps passed again for the cleanup. The staged public-only
source passed Windows and Linux installed native/CLI example checks, strict
documentation build, README executable example and link checks, exact notice-hash
checks and local artifact assembly. A changed-notice negative check rejects the
artifact before output creation. Example ZIP/XML properties, relationship targets,
JSON records, PNG metadata and document-only previews were checked for personal
content; local paths were removed from the public compatibility record.
These checks do not claim a new Linux full-CI run or clean-host qualification.

## Integration with the fork's newer main branch

The alpha branch incorporates main at 729cd0a5, retaining its 85 newer WordCraft
commits. README identity, contributor instructions and DOCX test registration
were resolved together. The isolated binding follows the updated PDF pin; its
lockfile and dependency/language-data notices match the merged build inputs.

Windows integration checks: all nine cargo xtask ci steps PASS. The rebuilt
optimized native wheel and matching CLI pass all three public examples. Initial
checks exposed Windows socket-reset test portability and a temporary paragraph
size regression; both are covered by regressions without relaxing limits or
production authentication. Paragraph assembly now reuses finished block allocations.
WASM checks pass with upstream unused-code warnings. Linux was not rebuilt for
this integration. The existing alpha tag and PyPI wheels remain at their original
source revision and are not replaced by this development merge.


## Unreleased alpha feedback changes — 9 November 2026

The development checkout adds conditional native bullets, conservative image
allocation diagnostics, opt-in scalar fan-out, outer-condition/immediate-repeat
composition and non-rendering context analysis. Published wheels are unchanged.
Shared document budgets cover independent and nested operations. Legacy Rust and
raw-context command entry points retain strict defaults.

Windows full CI passes all nine steps. Fresh optimized CPython 3.12 Windows/Linux
x64 wheels pass bounded installed API/CLI checks and the three public examples;
eight cross-platform DOCX package pairs match. Five synthetic outputs also pass
automated Word save/reopen, without establishing interactive repair-prompt or
visual fidelity findings. Full Linux CI and broader acceptance/Word/adoption
qualification are not claimed. See the API README and reference limitations for
the development support matrix and decoder allocation-audit limitations.


Follow-up reassessment fixes context-analysis parity for active conditional
consumers and unused fields after invalid context keys. Added real shared-budget
boundary tests and a complete original progressive-JPEG decode/round-trip fixture.
Both platforms pass 35 template tests, nine image tests and refreshed installed
wheel/CLI checks; all nine Windows CI steps pass. No new Word interactive/visual
or full Linux CI qualification is claimed.
