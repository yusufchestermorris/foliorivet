"""Audit supplied saved copies without editing any DOCX or generated original."""
import ast
import hashlib
import json
import posixpath
from pathlib import Path
import sys
from xml.etree import ElementTree as ET
from zipfile import ZipFile

repo = Path(sys.argv[1]).resolve()
base = repo / "docs/wordcraft/evidence/stage5"
W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
WP = "{http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing}"
R = "{http://schemas.openxmlformats.org/officeDocument/2006/relationships}"
source = Path(__file__).with_name("installed_smoke.py")
parsed = ast.parse(source.read_text(encoding="utf-8"))
namespace = {"ZipFile": ZipFile, "ET": ET, "hashlib": hashlib, "json": json, "posixpath": posixpath, "W": W}
definitions = [n for n in parsed.body if isinstance(n, ast.FunctionDef) and n.name in ["tree_signature", "structure"]]
exec(compile(ast.Module(body=definitions, type_ignores=[]), str(source), "exec"), namespace)
structure = namespace["structure"]
records = json.loads((base / "word-automation.json").read_text(encoding="utf-8-sig"))
candidates = json.loads((base / "manual-review-pending.json").read_text(encoding="utf-8"))["candidates"]
hashes = json.loads((base / "artifact-hashes.json").read_text(encoding="utf-8"))
checks = []
for candidate in candidates:
    original, manual = Path(candidate["input"]), Path(candidate["save_as"])
    record = next(r for r in records if r["platform"] == candidate["platform"] and r["case"] == "native-output")
    expected = Path(record["saved"])
    assert manual.is_file(), manual
    assert structure(manual) == structure(expected), f"Structure differs from automated Word save: {manual}"
    with ZipFile(manual) as z:
        assert z.testzip() is None
        roots = {n: ET.fromstring(z.read(n)) for n in z.namelist() if n.endswith((".xml", ".rels"))}
        for name, root in roots.items():
            if name.endswith(".rels"):
                ids = []
                for rel in root:
                    ids.append(rel.get("Id"))
                    if rel.get("TargetMode") != "External":
                        target = rel.get("Target")
                        target = target.lstrip("/") if target.startswith("/") else posixpath.normpath(posixpath.join(posixpath.dirname(posixpath.dirname(name)), target))
                        assert target in z.namelist(), (name, target)
                assert len(ids) == len(set(ids)), name
            else:
                relname = posixpath.join(posixpath.dirname(name), "_rels", posixpath.basename(name) + ".rels")
                ids = {r.get("Id") for r in roots[relname]} if relname in roots else set()
                for node in root.iter():
                    for key, value in node.attrib.items():
                        if key in [R + "id", R + "embed", R + "link"]: assert value in ids, (name, key, value)
        document = roots["word/document.xml"]
        with ZipFile(expected) as e:
            expected_root = ET.fromstring(e.read("word/document.xml"))
        def paragraphs(root):
            return ["".join(n.text or "" if n.tag == W + "t" else "\n" if n.tag == W + "br" else "\t" for n in p.iter() if n.tag in [W + "t", W + "br", W + "tab"]) for p in root.iter(W + "p")]
        assert paragraphs(document) == paragraphs(expected_root), "Paragraph content changed"
        tables = list(document.iter(W + "tbl"))
        row_counts = [len(t.findall(W + "tr")) for t in tables]
        assert row_counts == [4, 2, 2, 2, 5], row_counts
        assert len(list(document.iter(W + "sectPr"))) == 3
        ids = [n.get(W + "val") for name, root in roots.items() if name.startswith("word/") for props in root.iter(W + "sdtPr") for n in props.findall(W + "id")]
        drawings = [n.get("id") for n in document.iter(WP + "docPr")]
        assert len(ids) == len(set(ids)) and len(drawings) == len(set(drawings)) == 3
        tags = [n.get(W + "val", "") for n in document.iter(W + "tag")]
        assert not any(t.startswith(("repeat:", "if:", "rows:", "image:", "value:item.")) for t in tags)
        assert (base / "fixtures/training-chart.png").read_bytes() in [z.read(n) for n in z.namelist() if n.startswith("word/media/")]
    original_hash = hashlib.sha256(original.read_bytes()).hexdigest()
    assert original_hash == record["input_sha256"].lower() == hashes[original.relative_to(repo).as_posix()]
    checks.append({"platform": candidate["platform"], "file": manual.relative_to(repo).as_posix(), "sha256": hashlib.sha256(manual.read_bytes()).hexdigest(), "bytes": manual.stat().st_size, "result": "PASS", "matches_automated_word_resave": True, "table_row_counts": row_counts, "sections": 3, "drawings": 3, "paragraph_text_styles_geometry_relationships_media": "PASS", "unique_control_and_drawing_ids": True, "generated_original_unchanged": True})
assert {c["platform"] for c in checks} == {"windows", "linux"}
result = {"date": "2026-11-05", "procedure": "python bindings/python/tests/stage5_manual_review.py REPO", "checks": checks, "user_statement": "windows: manual-review-native-output.docx; linux: manual-review-native-output.docx; no repair prompts", "independent_review": "PASS user-reported no repair prompts for both supplied saved copies; other manual appearance/editability findings not separately asserted", "manual_word_build": "Not separately supplied; preceding automation used Word 16.0 build 16.0.20527", "gate": "PASS for the bounded representative block scope on Windows x64 and Ubuntu 22.04 WSL2 x64 with CPython 3.12; inherited limits remain", "next_stage": "Stage 6 eligible; not started or authorized by this confirmation"}
(base / "manual-review-structural-audit.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, indent=2))
