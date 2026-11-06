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
