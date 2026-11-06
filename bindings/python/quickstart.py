"""Run with an installed wheel: python -I quickstart.py TEMPLATE NEW_OUTPUT."""
from pathlib import Path
import sys
from foliorivet import inspect_template, preflight_template, render_template

template, output = map(Path, sys.argv[1:])
inspection = inspect_template(template)
context = {
    "metadata": {"model_id": "Example 42 — café"},
    "labels": {
        target["tag"].removeprefix("value:labels."): "Review — Ω"
        for target in inspection["targets"] if target["tag"].startswith("value:labels.")
    },
}
print(preflight_template(template, context))
output.parent.mkdir(parents=True, exist_ok=True)
print(render_template(template, context, "", output))
