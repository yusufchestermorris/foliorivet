"""Run with the installed native wheel; all amounts are supplied by Python input."""
from pathlib import Path
import json
import sys
from foliorivet import preflight_template, render_template

base = Path(__file__).resolve().parent
context = json.loads((base / "input.json").read_text(encoding="utf-8"))
output = Path(sys.argv[1]) if len(sys.argv) > 1 else base / "generated.docx"
print(preflight_template(base / "template.docx", context))
print(render_template(base / "template.docx", context, "", output))
