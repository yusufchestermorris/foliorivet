#!/usr/bin/env bash
set -euo pipefail
workspace="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)"
export RUSTUP_HOME="$workspace/.tooling/linux/rustup"
export CARGO_HOME="$workspace/.tooling/linux/cargo"
export CARGO_TARGET_DIR="$workspace/.tooling/linux/target"
export RUSTUP_TOOLCHAIN=1.95.0
export PATH="$CARGO_HOME/bin:/usr/local/bin:/usr/bin:/bin"
export CRAFT_I18N_DIR="$workspace/.tooling/craft-i18n"
export CRAFT_I18N_REQUIRED=1
export PYO3_PYTHON="$workspace/.tooling/stage3/linux/build-env/bin/python"
cd "$workspace/foliorivet"
if [ "${1:-}" = "--release" ]; then
    "$PYO3_PYTHON" tools/release/build_candidate.py --i18n "$CRAFT_I18N_DIR" --output "${2:-dist/preview-0.1.0a1-linux}" --offline --profile release
    exit $?
fi
cargo build --locked --offline -p wordcraft-cli --bin foliorivet-cli
"$PYO3_PYTHON" -m maturin build --locked --offline --manifest-path bindings/python/Cargo.toml --interpreter "$PYO3_PYTHON" --compatibility manylinux_2_35 --out dist/linux
cargo test --locked --offline -p wordcraft-doc --lib tests::note_conversion::conversion_work_scales_with_reference_and_formatting_runs -- --exact --nocapture
