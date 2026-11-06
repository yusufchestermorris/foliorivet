"""Execute a revised invoice example with the installed wheel, retaining prior output."""
import json
from pathlib import Path
import subprocess
import sys

repo, evidence = (Path(p).resolve() for p in sys.argv[1:])
results = [(p, json.loads(p.read_text(encoding="utf-8"))) for p in sorted(evidence.glob("run-*/result.json"))]
result, data = [pair for pair in results if "row_cases" in pair[1]][-1]
output = result.parent / "invoice-final.docx"
subprocess.run([sys.executable, "-I", str(repo / "bindings/python/examples/invoice/generate.py"), str(output)], check=True)
print(output)
