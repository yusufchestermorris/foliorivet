# Contributing to FolioRivet

FolioRivet is an independent WordCraft fork. Preserve upstream licenses, NOTICE,
asset attribution and the Rust engine layering. Keep PyO3 thin and business/report
calculations in the caller. Read AGENTS.md for implementation conventions.

## Build and test

See [build instructions](docs/release/BUILD.md) for Rust 1.95.0, CPython 3.12,
maturin 1.10.2 and the pinned craft-i18n checkout. Both Cargo lockfiles are tracked.
Run `cargo xtask ci` with the required language data and platform build libraries.
It checks formatting, clippy, workspace tests, localization, doctests, assets,
layering and WASM. The native binding has its own manifest and lockfile.

Install a locally built wheel in a fresh Python 3.12 environment, then run:

```text
python -I tools/release/smoke_installed.py --examples bindings/python/examples --cli PATH_TO_MATCHING_CLI --output NEW_RESULTS_DIRECTORY
```

All three public examples are committed, including templates, input, generators,
reference outputs and previews. This check uses only committed inputs, compares
native/CLI package members, verifies template preservation and refuses overwrite.
Rust template tests cover scalar, row/picture and block operations and hostile
inputs. Legacy qualification harnesses and private review evidence are local-only;
you do not need them to build or run the public checks.

## Report a bug or submit a change

Use https://github.com/yusufchestermorris/foliorivet/issues. Include version,
OS/architecture, Python version, exact command, expected/actual behavior, error
and a minimal redistributable synthetic template/input. Remove personal document
properties and private content. Do not submit a Word-resaved review copy as a fixture.
A behavior fix should include a regression test. Record checks actually run and
limits; do not claim Word compatibility from XML checks alone.

New assets need an ATTRIBUTION.md entry and appropriate redistribution terms.
Keep required notices with artifacts. Local plans, evidence, caches and build
outputs are ignored, not runtime dependencies. Add reusable tools and tests to Git.
