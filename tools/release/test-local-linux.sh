#!/usr/bin/env bash
set -euo pipefail
workspace="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)"
repo="$workspace/foliorivet"
run=$(mktemp -d /tmp/foliorivet-release-010-XXXXXX)
echo "$run"
"$workspace/.tooling/stage3/linux/build-env/bin/python" -m venv "$run/venv"
"$run/venv/bin/python" -m pip install --no-index "$repo/dist/linux/foliorivet-0.1.0-cp312-cp312-manylinux_2_35_x86_64.whl"
for script in installed_smoke stage4_smoke stage5_smoke; do
 "$run/venv/bin/python" -I "$repo/bindings/python/tests/$script.py" "$repo" "$run/$script" "$workspace/.tooling/linux/target/debug/foliorivet-cli" > "$run/$script.log"
done
"$run/venv/bin/python" -I "$repo/bindings/python/examples/training-evaluation/generate.py" "$run/training.docx"
mkdir -p "$workspace/.private-archive/release-prep-20261106/linux-test"
cp "$run/"*.log "$run/training.docx" "$workspace/.private-archive/release-prep-20261106/linux-test/"
printf '%s\n' "$run" > "$workspace/.private-archive/release-prep-20261106/linux-test/run-path.txt"
echo PASS
