# Build and test from source

Use Rust 1.95.0, CPython 3.12 x64 and maturin 1.10.2. Keep both Cargo lockfiles.
Windows needs an x64 Visual Studio C++ development shell. Linux needs a native
linker, pkg-config and development libraries for the dependencies being built;
full workspace/desktop tests additionally need their UI system libraries and font
inputs. The existing CJK indent fixture requires an installed CJK font.
No private .tooling directory, reports or historical evidence are build inputs.

## Language data and tools

From the repository root, clone the public language-data input into ignored .deps:

```sh
git clone https://github.com/storytold/craft-i18n.git .deps/craft-i18n
git -C .deps/craft-i18n checkout "$(cat craft-i18n.rev)"
python3.12 -m venv .venv
. .venv/bin/activate
python -m pip install maturin==1.10.2
export RUSTUP_TOOLCHAIN=1.95.0
export CRAFT_I18N_DIR="$PWD/.deps/craft-i18n"
export CRAFT_I18N_REQUIRED=1
```

PowerShell equivalents: use `py -3.12 -m venv .venv`, activate
`.venv/Scripts/Activate.ps1`, checkout the pin from `Get-Content craft-i18n.rev`,
and set `$env:RUSTUP_TOOLCHAIN`, `$env:CRAFT_I18N_DIR` (absolute path) and
`$env:CRAFT_I18N_REQUIRED`. Install Rust's wasm32-unknown-unknown target for CI.
The language-data checkout must be at the exact pin and unmodified.

## Build and verify

```text
python tools/release/build_candidate.py --i18n ABSOLUTE_LANGUAGE_DATA_PATH --output NEW_CANDIDATE_DIRECTORY
cargo xtask ci
```

The builder uses locked inputs and an optimized release profile by default.
`--offline` needs cached dependencies; `--profile dev` is for development only.
Linux wheels use the audited manylinux_2_35 tag. Never relabel a failed wheel.
No sdist is qualified. Binding-only build:
`python -m maturin build --locked --release --manifest-path bindings/python/Cargo.toml`.
CLI-only build: `cargo build --locked --release -p wordcraft-cli --bin foliorivet-cli`.

Install the wheel into a fresh CPython 3.12 environment, then from this source tree:

```text
python -m pip install --no-index PATH_TO_WHEEL
python -I tools/release/smoke_installed.py --examples bindings/python/examples --cli PATH_TO_CLI --output NEW_RESULTS_DIRECTORY
```

This uses committed examples, not private fixtures. On Linux, give the extracted
CLI execute permission with `chmod +x foliorivet-cli` if needed. The check verifies
three native/CLI outputs, input preservation, overwrite refusal and error handling.
It does not prove production suitability or visual fidelity of arbitrary templates.
See CONTRIBUTING.md and PREVIEW.md for release assembly and workflow.
