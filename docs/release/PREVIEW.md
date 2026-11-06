# Manual experimental preview: 0.1.0a1

Publication HOLD. Nothing has been committed, tagged, pushed or uploaded.

## Prepared source

Python metadata: 0.1.0a1; isolated Rust binding: 0.1.0-alpha.1;
CLI display: 0.1.0a1. Upstream engine version remains unchanged.
The inherited WordCraft release workflow is retained as release.yml.disabled.
Review all other workflows before enabling Actions on a new repository.
No Python API change. CPython 3.12 Windows x64 and Linux x64 glibc 2.35+
(initially Ubuntu 22.04) only. No sdist, desktop, ARM or macOS release.

## Owner decisions and release gates

- GitHub fork: https://github.com/yusufchestermorris/foliorivet. Project URLs are
  set in Python metadata; existing wheels require rebuilding to include them.
- Review source AND history for private reports, evidence, secrets and personal data.
  Ignoring files does not erase history. Do not publish the workspace or handoff ZIPs.
- Complete legal/redistribution and binary-resource review, including all example
  DOCX/PNG files. Collected notices do not constitute clearance.
- Verify clean-host installation, Windows generation without Office, independent
  Word repair-prompt review, and user-followed quick start on final candidates.
- Record remaining full-workspace failures as limitations, never as passing checks.
- Production-source access and shadow integration are deferred to adoption.
- Human review, commit and publication approval remain required by workspace rules.

## Build and test

Run from the repository with the prepared toolchains. Output directories must be new:

```powershell
./tools/release/build-local-windows.ps1 -Release
```

```sh
bash tools/release/build-local-linux.sh --release
```

These use dist/preview-0.1.0a1-windows and dist/preview-0.1.0a1-linux.
Use fresh CPython 3.12 environments to install each matching wheel with
`python -m pip install --no-index <wheel>`; run installed_smoke.py,
stage4_smoke.py and stage5_smoke.py with REPO NEW_RESULTS MATCHING_CLI arguments.
Then run:

```text
python -I tools/release/smoke_installed.py --examples bindings/python/examples --cli <matching-cli> --output <new-results>
python tools/release/prepare_preview.py --windows dist/preview-0.1.0a1-windows --linux dist/preview-0.1.0a1-linux --output dist/preview-upload
python -m twine check --strict dist/preview-upload/pypi/*.whl
```

The assembler rejects missing candidates, development profiles, changed hashes,
missing notice inventory entries and mismatched wheel notice contents. It writes
CLI bundles, a wheel-only pypi directory and SHA256SUMS.txt. It grants no clearance.
The legacy package_local_cli.py is historical tooling; do not use it for this preview.

## Manual publication after gates pass

1. Use the GitHub fork yusufchestermorris/foliorivet of storytold/wordcraft, with
   Actions disabled. Review and commit the intended source, then push only the
   approved branch. Audit its complete history before publishing local changes.
2. Freeze the release commit and rebuild/test from exactly that source. The current
   uncommitted candidates cannot establish final tagged-source provenance.
3. Create separate PyPI/TestPyPI accounts, verify email and configure account security.
   Verify project ownership/name availability; earlier availability checks reserve nothing.
4. Upload only the two wheels to TestPyPI:
   `python -m twine upload --repository testpypi dist/preview-upload/pypi/*.whl`.
   Enter username __token__ and the TestPyPI token at the prompt, never in source.
5. On both targets, install with
   `python -m pip install --index-url https://test.pypi.org/simple/ --only-binary=:all: --no-deps foliorivet==0.1.0a1`
   and rerun the examples. Current metadata declares no Python runtime dependencies.
6. After final approval, publish the reviewed branch to the GitHub fork, tag the frozen
   commit v0.1.0a1, and push that tag explicitly. Create a draft prerelease with
   both wheels, CLI bundles, checksums and PREVIEW-NOTES.md text.
7. Upload the identical tested wheels using the separate PyPI token:
   `python -m twine upload dist/preview-upload/pypi/*.whl`.
8. Verify fresh installation on both targets:
   `python -m pip install --index-url https://pypi.org/simple/ --only-binary=:all: foliorivet==0.1.0a1`.
9. Publish the GitHub draft marked prerelease. Add actual repository and PyPI links.
10. Replace bootstrap tokens with project-scoped tokens. Publish fixes as 0.1.0a2;
    yank materially broken PyPI releases and annotate GitHub notes.

Official references: [TestPyPI](https://packaging.python.org/en/latest/guides/using-testpypi/),
[GitHub releases](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository).
Trusted Publishing is recommended for later automation; this first release is manual.

Current execution: [0.1.0a1 validation](PREVIEW-VALIDATION.md).
