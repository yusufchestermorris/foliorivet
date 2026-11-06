# 0.1.0a1 experimental preview status

Updated 6 November 2026. IN PREPARATION; publication HOLD.

Implemented: shared Rust scalar, row, inline-picture, repeat/conditional-block
operations; strict scoped diagnostics; synchronous native Python API and CLI.
Historical installed Windows/Linux and Word review evidence is recorded in the
canonical plan. Stage 7 found the bounded prototype feasible, not production-ready.

The current release work targets CPython 3.12, Windows x64 and Ubuntu 22.04 x64;
Python wheels and CLI only. No desktop or broader compatibility promise.

Pending acceptance:

- Final human legal/redistribution clearance of collected dependency and asset notices.
- Independent clean-machine installation and interactive Word repair-prompt review.
- PyPI release-account configuration and verified ownership (historical availability is not reservation).
- Reviewed source/history export; GitHub fork and package URLs are now configured.
- Final human Git commit, tagging, and publication approval.

Production-source access and opt-in shadow integration are deferred to adoption;
they do not gate this experimental preview. Clean-machine installation and final
independent Word review remain preview acceptance checks.

Historical evidence and original handoff artifacts are preserved privately during
cleanup. No commit, push, tag, public repository, package upload or production
cutover has occurred. Previous passing evidence retains its original scope;
changes made during release preparation require their own checks.

## Work executed on 6 November

- Replaced the upstream product README and exposed foliorivet-cli 0.1.0 while
  retaining the upstream binary target for existing tooling.
- Archived eight ArtCraft logo/mark files; preserved the separate brand license,
  upstream code licenses and NOTICE. Added modification notices to changed
  tracked source files. Upstream license/NOTICE contents match HEAD after normal
  CRLF normalization; no terms or attribution were removed.
- Collected exact notice texts for the 232-package binding and 226-package CLI
  Windows/Linux normal/build dependency supersets. The five objc2 packages and
  r-efi are absent from these target graphs. Missing alloc-stdlib/wuff texts were
  obtained from the exact Cargo-recorded upstream revisions. This closes source
  text detection for these graphs, not final binary-resource/legal clearance.
- Built both development-profile and optimized release-profile Windows and
  manylinux_2_35 Linux wheels and CLI bundles (`dist/release-windows`,
  `dist/release-linux`). Maturin accepted the Linux compatibility audit.
- Fresh installed scalar, row/picture and block suites pass on both platforms;
  block suite reports 16 combinations each. All three examples pass Python/CLI
  ZIP-member equality and existing-output refusal on both platforms. Their
  generated DOCX hashes also agree across platforms.
- Repaired Word review automation harness (`review-word.ps1`) by adding non-blocking
  display alert suppression. Word 16.0.20527 automation passed open/save/reopen/export
  on `training-evaluation` (1 page), `invoice` (1 page), and `project-status`.
- Isolated Linux timing test passes (13.92 ratio vs < 24.0 threshold). Intermittent
  aggregate failure investigated: identified as thread and allocator contention
  in virtualized WSL2 schedstat counters under parallel test runner. Assertion
  strictly preserved without relaxing thresholds.
- Rust formatting, Python tool compilation, asset attribution (49 active assets)
  and layering (19 crates) checks pass.
- Verified name availability: `foliorivet` is unregistered and available on PyPI
  (JSON API 404, 0 search results) and GitHub (0 search results).
- Maintained detailed preparation ledger in `docs/release/0.1.0-preparation.md`.
- Archived and verified 89 top-level scripts/logs and 2,625 non-cache evidence
  files. The entire evidence directory, including generated runs, is retained
  privately; the content manifest excludes generated runs/caches. An ignored
  local junction preserves historical paths. Original handoff packages remain.

Artifacts are under dist/ and recorded in candidate-manifest.json.
Private execution logs and archive manifests are under the workspace's
.private-archive/release-prep-20261106/. Current source includes later notice and
documentation additions; candidates remain local preparation artifacts, not a frozen
release. Final source-export review, example redistribution review, and public-release
approval are still required.

Current execution: [0.1.0a1 validation](PREVIEW-VALIDATION.md).
