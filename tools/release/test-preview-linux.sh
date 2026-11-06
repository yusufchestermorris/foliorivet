#!/usr/bin/env bash
set -euo pipefail
workspace="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)"
repo="$workspace/foliorivet"
run=$(mktemp -d /tmp/foliorivet-preview-XXXXXX)
python="$workspace/.tooling/stage3/linux/build-env/bin/python"
"$python" -m venv "$run/venv"
python="$run/venv/bin/python"
cli="$repo/dist/preview-0.1.0a1-linux/foliorivet-cli"
"$python" -m pip install --no-index "$repo/dist/preview-0.1.0a1-linux/foliorivet-0.1.0a1-cp312-cp312-manylinux_2_35_x86_64.whl"
for script in installed_smoke stage4_smoke stage5_smoke; do
 "$python" -I "$repo/bindings/python/tests/$script.py" "$repo" "$run/$script" "$cli" > "$run/$script.log"
done
"$python" -I "$repo/tools/release/smoke_installed.py" --examples "$repo/bindings/python/examples" --cli "$cli" --output "$run/examples" > "$run/examples.log"
mkdir "$repo/dist/preview-tests-linux"
cp "$run/"*.log "$repo/dist/preview-tests-linux/"
printf '%s\n' "$run" > "$repo/dist/preview-tests-linux/run-path.txt"
echo PASS
