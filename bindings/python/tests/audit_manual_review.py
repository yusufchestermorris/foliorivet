"""Check user-saved Word copies without claiming to observe Word's UI."""
from pathlib import Path
import ast
import hashlib
import json
import posixpath
import sys
from xml.etree import ElementTree as ET
from zipfile import ZipFile

repo = Path(sys.argv[1]).resolve()
base = Path(sys.argv[2]).resolve() if len(sys.argv) > 2 else repo / "docs/wordcraft/evidence/stage3"
W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
# Reuse the installed smoke test's structural comparison, without running its jobs.
source = Path(__file__).with_name("installed_smoke.py")
parsed = ast.parse(source.read_text(encoding="utf-8"))
namespace = {"ZipFile": ZipFile, "ET": ET, "hashlib": hashlib, "json": json,
             "posixpath": posixpath, "W": W}
definitions = [node for node in parsed.body if isinstance(node, ast.FunctionDef)
               and node.name in ["tree_signature", "structure"]]
exec(compile(ast.Module(body=definitions, type_ignores=[]), str(source), "exec"), namespace)
structure = namespace["structure"]
automation_file = "native-word-automation.json" if base.name == "style-repair" else "word-automation.json"
automation = json.loads((base / automation_file).read_text(encoding="utf-8-sig"))
checks = []
for original in automation:
    run = Path(original["input"]).parent
    manual = run / "manual-review-native-output.docx"
    assert manual.is_file(), manual
    actual = structure(manual)  # ZIP CRC, XML parsing and relationship destinations checked here.
    expected = structure(Path(original["saved"]))
    assert actual == expected, f"Manual copy differs structurally from automated Word-resaved copy: {manual}"
    with ZipFile(manual) as z:
        doc = ET.fromstring(z.read("word/document.xml"))
        table_count = len(list(doc.iter(W + "tbl")))
        section_count = len(list(doc.iter(W + "sectPr")))
        ids = []
        for name in z.namelist():
            if name.startswith("word/") and name.endswith(".xml"):
                for ctl in ET.fromstring(z.read(name)).iter(W + "sdtPr"):
                    ident = ctl.find(W + "id")
                    if ident is not None:
                        ids.append(ident.get(W + "val"))
        assert len(ids) == len(set(ids)), "Duplicate control identifiers"
    assert table_count == 3 and section_count == 3 and len(actual["controls"]) == 8
    assert hashlib.sha256(Path(original["input"]).read_bytes()).hexdigest() == original["input_sha256"].lower()
    checks.append({"platform": original["platform"], "file": str(manual.relative_to(repo)).replace("\\", "/"),
                   "sha256": hashlib.sha256(manual.read_bytes()).hexdigest(), "bytes": manual.stat().st_size,
                   "result": "PASS structural verification", "scalar_targets": len(actual["controls"]),
                   "tables": table_count, "sections": section_count, "control_ids_unique": True,
                   "matches_automated_word_resave": True, "generated_original_unchanged": True})
record = {"date": "2026-11-05", "procedure": "python bindings/python/tests/audit_manual_review.py REPO",
          "checks": checks, "user_report": "User supplied both saved-copy filenames; UI findings not yet confirmed",
          "interactive_review": "PENDING user confirmation of reopen, prompts and formatting",
          "word_build": "Not separately supplied for manual review; automated review used 16.0.20527",
          "gate": "PENDING interactive review and inherited style-limit disposition"}
confirmation = base / "manual-review-confirmation.json"
if confirmation.is_file():
    reported = json.loads(confirmation.read_text(encoding="utf-8"))
    record["user_report"] = reported["user_statement"]
    record["interactive_review"] = "PASS user-reported opening of both saved copies without repair prompts; other manual visual/editability findings not separately asserted"
    record["gate"] = "PENDING inherited style-limit disposition"
    if base.name == "style-repair":
        repair = json.loads((base / "repair-audit.json").read_text(encoding="utf-8"))
        assert repair["style_repair"].startswith("PASS"), "Style repair is not verified"
        record["gate"] = "PASS for the agreed representative scalar-template scope on Windows x64 and Ubuntu 22.04 WSL2 x64, CPython 3.12"
        record["procedure"] = "python bindings/python/tests/audit_manual_review.py REPO REPO/docs/wordcraft/evidence/stage3/style-repair"
(base / "manual-review-structural-audit.json").write_text(json.dumps(record, indent=2), encoding="utf-8")
print(json.dumps(record, indent=2))
