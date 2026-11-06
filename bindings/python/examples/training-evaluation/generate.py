"""Public API only; business decisions and precomputed values stay in Python."""
import json
from pathlib import Path
import sys
from foliorivet import preflight_template, render_template

here=Path(__file__).resolve().parent
context=json.loads((here/"input.json").read_text(encoding="utf-8"))
output=Path(sys.argv[1])
preflight_template(here/"template.docx",context)
print(render_template(here/"template.docx",context,"",output))
