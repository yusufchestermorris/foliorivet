# Manual experimental release workflow

The first alpha is on PyPI. Use a new version for subsequent releases; never
replace previously uploaded wheel contents or move an existing public tag.

1. Review the source, tests, public examples, licenses and notices. Ignore local
   plans, evidence, build outputs and machine-specific convenience scripts.
2. Build and run the checks in BUILD.md on the supported platforms. Record failures
   and qualification limits. Clean-host, independent Word, measured onboarding and
   production integration are future adoption work, not alpha release gates.
3. Review and commit changes on a branch, open a PR into this fork's main branch,
   review and merge before releasing. Preserve upstream history and notices.
4. Build matching candidates from the reviewed source. Assemble using:

   ```text
   python tools/release/prepare_preview.py --windows WINDOWS_CANDIDATE --linux LINUX_CANDIDATE --output NEW_UPLOAD_DIRECTORY
   python -m twine check --strict NEW_UPLOAD_DIRECTORY/pypi/*.whl
   ```

   The assembler uses committed notice inputs and checks release profiles, wheel
   tags and candidate hashes. It emits two wheels, CLI bundles and SHA256SUMS.txt.
5. Upload the selected wheels to TestPyPI with Twine. Enter its token at the hidden
   prompt. In a fresh environment, install with the TestPyPI index, an exact version,
   `--only-binary=:all: --no-deps --no-cache-dir`, and run the public smoke check.
6. Upload the identical tested wheels to PyPI using its separate token. Verify a
   fresh installation from the production index and repeat the smoke check.
7. Tag the tested source commit once it is reachable from main. Create a GitHub
   prerelease in yusufchestermorris/foliorivet with wheels, CLI bundles, checksums,
   scope, known limits and the PyPI link. Keep inherited upstream release automation
   disabled until a dedicated FolioRivet workflow has been reviewed.

Official references: [TestPyPI](https://packaging.python.org/en/latest/guides/using-testpypi/),
[GitHub releases](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository).
Do not put tokens in source, shell arguments or issue reports. For fixes, publish
an incremented alpha version; yank materially broken releases rather than replacing
existing files. See STATUS.md for the actual first-alpha qualification scope.
