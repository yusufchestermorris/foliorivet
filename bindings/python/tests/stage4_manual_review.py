"""Read-only audit of the user-saved representative Stage 4 outputs."""
import ast
import hashlib
import json
import posixpath
from pathlib import Path
import sys
from xml.etree import ElementTree as ET
from zipfile import ZipFile

repo = Path(sys.argv[1]).resolve()
base = repo / "docs/wordcraft/evidence/stage4"
W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
WP = "{http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing}"
R = "{http://schemas.openxmlformats.org/officeDocument/2006/relationships}"
source = Path(__file__).with_name("installed_smoke.py")
parsed = ast.parse(source.read_text(encoding="utf-8"))
namespace = {"ZipFile": ZipFile, "ET": ET, "hashlib": hashlib, "json": json, "posixpath": posixpath, "W": W}
definitions = [node for node in parsed.body if isinstance(node, ast.FunctionDef) and node.name in ["tree_signature", "structure"]]
exec(compile(ast.Module(body=definitions, type_ignores=[]), str(source), "exec"), namespace)
structure = namespace["structure"]

def paragraphs(path):
    with ZipFile(path) as z:
        root = ET.fromstring(z.read("word/document.xml"))
        return ["".join(n.text or "" if n.tag == W + "t" else "\n" if n.tag == W + "br" else "\t" for n in p.iter() if n.tag in [W + "t", W + "br", W + "tab"]) for p in root.iter(W + "p")]

records = json.loads((base / "word-automation.json").read_text(encoding="utf-8-sig"))
checks = []
for record in records:
    if record["case"] != "native-output":
        continue
    original = Path(record["input"])
    manual = original.with_name("manual-review-native-output.docx")
    expected = Path(record["saved"])
    assert manual.is_file(), manual
    assert structure(manual) == structure(expected), f"Saved copy differs from automated Word-resaved structure: {manual}"
    assert paragraphs(manual) == paragraphs(expected), f"Paragraph content changed: {manual}"
    with ZipFile(manual) as z:
        roots = {n: ET.fromstring(z.read(n)) for n in z.namelist() if n.endswith((".xml", ".rels"))}
        for name, root in roots.items():
            if not name.endswith(".xml"):
                continue
            relname = posixpath.join(posixpath.dirname(name), "_rels", posixpath.basename(name) + ".rels")
            relids = {r.get("Id") for r in roots[relname]} if relname in roots else set()
            for node in root.iter():
                for key, value in node.attrib.items():
                    if key in [R + "id", R + "embed", R + "link"]:
                        assert value in relids, (name, key, value)
        document = roots["word/document.xml"]
        tables = list(document.iter(W + "tbl"))
        assert len(tables) == 3 and len(list(document.iter(W + "sectPr"))) == 3
        assert len(tables[0].findall(W + "tr")) == 4
        assert tables[0].find(W + "tr/" + W + "trPr/" + W + "tblHeader") is not None
        ids = [node.get(W + "val") for name, root in roots.items() if name.startswith("word/") for props in root.iter(W + "sdtPr") for node in props.findall(W + "id")]
        assert len(ids) == len(set(ids))
        drawing_ids = [n.get("id") for n in document.iter(WP + "docPr")]
        assert len(drawing_ids) == len(set(drawing_ids))
        chart = (base / "fixtures/training-chart.png").read_bytes()
        assert chart in [z.read(n) for n in z.namelist() if n.startswith("word/media/")]
    assert hashlib.sha256(original.read_bytes()).hexdigest() == record["input_sha256"].lower()
    checks.append({"platform": record["platform"], "file": manual.relative_to(repo).as_posix(), "sha256": hashlib.sha256(manual.read_bytes()).hexdigest(), "bytes": manual.stat().st_size, "result": "PASS", "matches_automated_word_resave": True, "table_row_counts": [len(t.findall(W + "tr")) for t in tables], "sections": 3, "paragraph_text_and_structure": "PASS", "styles_geometry_relationships_media": "PASS", "unique_control_and_drawing_ids": True, "generated_original_unchanged": True})
assert {c["platform"] for c in checks} == {"windows", "linux"}
result = {"date": "2026-11-05", "procedure": "python bindings/python/tests/stage4_manual_review.py REPO", "checks": checks, "user_statement": "windows: manual-review-native-output.docx; linux: manual-review-native-output.docx; no repair prompts", "independent_review": "PASS user-reported no repair prompts for both saved copies; other manual appearance/editability findings not separately asserted", "manual_word_build": "Not separately supplied; preceding automation used Word 16.0 build 16.0.20527", "gate": "PASS for the agreed representative row/chart scope on Windows x64 and Ubuntu 22.04 WSL2 x64 with CPython 3.12; inherited qualification limits remain", "next_stage": "Stage 5 eligible; not authorized by this review confirmation"}
(base / "manual-review-structural-audit.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
print(json.dumps(result, indent=2))
