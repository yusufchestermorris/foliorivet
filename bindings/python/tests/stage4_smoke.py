"""Real installed-extension and CLI qualification; stdlib only, run with -I."""
from copy import deepcopy
from datetime import datetime, timezone
import hashlib
import importlib.machinery
import json
from pathlib import Path
import platform
import posixpath
import subprocess
import sys
from xml.etree import ElementTree as E
from zipfile import ZipFile, ZIP_DEFLATED

import foliorivet
from foliorivet import _native, TemplateError

repo, evidence, cli = (Path(p).resolve() for p in sys.argv[1:])
run = evidence / datetime.now(timezone.utc).strftime("run-%Y%m%dT%H%M%S%fZ")
run.mkdir(parents=True)
fixtures = repo / "docs/wordcraft/evidence/stage4/fixtures"
template = fixtures / "rows-chart-template.docx"
context = json.loads((fixtures / "context.json").read_text(encoding="utf-8"))
W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
A = "{http://schemas.openxmlformats.org/drawingml/2006/main}"
WP = "{http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing}"
R = "{http://schemas.openxmlformats.org/officeDocument/2006/relationships}"

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def error(call, message):
    try:
        call()
    except TemplateError as exc:
        assert message in str(exc), (message, str(exc))
        assert "target=" in str(exc) and "scope=" in str(exc), str(exc)
    else:
        raise AssertionError("Expected error: " + message)

def package(path):
    with ZipFile(path) as z:
        assert z.testzip() is None
        names = set(z.namelist())
        for name in names:
            if name.endswith((".xml", ".rels")):
                part_root = E.fromstring(z.read(name))
                if name.endswith(".xml"):
                    rel_name = posixpath.join(posixpath.dirname(name), "_rels", posixpath.basename(name) + ".rels")
                    rel_ids = {r.get("Id") for r in E.fromstring(z.read(rel_name))} if rel_name in names else set()
                    for node in part_root.iter():
                        for key, value in node.attrib.items():
                            if key in [R + "id", R + "embed", R + "link"]:
                                assert value in rel_ids, (name, key, value)
            if name.endswith(".rels"):
                folder = posixpath.dirname(posixpath.dirname(name))
                ids = []
                for rel in E.fromstring(z.read(name)):
                    ids.append(rel.get("Id"))
                    if rel.get("TargetMode") != "External":
                        target = rel.get("Target")
                        target = target.lstrip("/") if target.startswith("/") else posixpath.normpath(posixpath.join(folder, target))
                        assert target in names, (name, target)
                assert len(ids) == len(set(ids)), name
        root = E.fromstring(z.read("word/document.xml"))
        for identifier in [W + "id", WP + "docPr"]:
            values = [n.get(W + "val") if identifier == W + "id" else n.get("id") for n in root.iter(identifier)]
            assert len(values) == len(set(values)), (identifier, values)
        return root, {n: z.read(n) for n in names}

def text(node):
    return "".join((n.text or "") if n.tag == W + "t" else "\n" if n.tag == W + "br" else "\t" if n.tag == W + "tab" else "" for n in node.iter())

assert "site-packages" in _native.__file__, _native.__file__
assert any(_native.__file__.endswith(s) for s in importlib.machinery.EXTENSION_SUFFIXES)
original = digest(template)
inspection = foliorivet.inspect_template(template)
assert all(t["supported"] for t in inspection["targets"]), inspection
assert {t["tag"] for t in inspection["targets"]} == {
    "value:metadata.model_id", "rows:class_distribution", "image:training_results",
    "value:item.class_name", "value:item.training", "value:item.evaluation", "value:item.note",
}
baseline = run / "unpopulated-roundtrip.docx"
convert = subprocess.run([str(cli), "convert", str(template), str(baseline)], capture_output=True, text=True)
(run / "baseline.log").write_text(convert.stdout + convert.stderr, encoding="utf-8")
assert convert.returncode == 0, convert.stderr
baseline_root, baseline_parts = package(baseline)
source_root, _ = package(template)
source_tables = list(source_root.iter(W + "tbl"))
baseline_tables = list(baseline_root.iter(W + "tbl"))
target_index = next(i for i, t in enumerate(source_tables) if any(n.get(W + "val") == "rows:class_distribution" for n in t.iter(W + "tag")))
target_baseline = baseline_tables[target_index]
prototype = target_baseline.find(W + "sdt/" + W + "sdtContent/" + W + "tr")
assert prototype is not None
records = {}
for count in [0, 1, 3, 120]:
    job = deepcopy(context)
    job["class_distribution"] = [deepcopy(context["class_distribution"][i % 3]) for i in range(count)]
    for i, row in enumerate(job["class_distribution"]):
        row["class_name"] = f"{i + 1:03d} " + row["class_name"]
    if count == 120:
        job["class_distribution"][8]["class_name"] = "009 " + "Long class label café " * 12
    preflight = foliorivet.preflight_template(template, job, fixtures)
    assert preflight["row_count"] == count and preflight["save_verified"] is False
    output = run / ("native-output.docx" if count == 3 else f"rows-{count}.docx")
    foliorivet.render_template(template, job, fixtures, output)
    root, parts = package(output)
    table = list(root.iter(W + "tbl"))[target_index]
    rows = table.findall(W + "tr")
    assert len(rows) == count + 1
    assert E.tostring(table.find(W + "tblPr")) == E.tostring(target_baseline.find(W + "tblPr"))
    assert E.tostring(table.find(W + "tblGrid")) == E.tostring(target_baseline.find(W + "tblGrid"))
    assert rows[0].find(W + "trPr/" + W + "tblHeader") is not None
    for row, item in zip(rows[1:], job["class_distribution"], strict=True):
        assert [text(c) for c in row.findall(W + "tc")] == [str(item[k]) for k in ["class_name", "training", "evaluation", "note"]]
        for cell, original_cell in zip(row.findall(W + "tc"), prototype.findall(W + "tc"), strict=True):
            assert E.tostring(cell.find(W + "tcPr")) == E.tostring(original_cell.find(W + "tcPr"))
            assert E.tostring(cell.find(W + "p/" + W + "pPr")) == E.tostring(original_cell.find(W + "p/" + W + "pPr"))
    assert parts["word/styles.xml"] == baseline_parts["word/styles.xml"]
    assert [E.tostring(n) for n in root.iter(W + "sectPr")] == [E.tostring(n) for n in baseline_root.iter(W + "sectPr")]
    tags = [n.get(W + "val") for n in root.iter(W + "tag")]
    assert not any(t.startswith(("rows:", "value:item.", "image:")) for t in tags)
    assert (fixtures / "training-chart.png").read_bytes() in parts.values()
    chart_control = next(c for c in root.iter(W + "sdt") if c.find(W + "sdtPr/" + W + "alias") is not None and c.find(W + "sdtPr/" + W + "alias").get(W + "val") == "image:training_results")
    old_chart = next(c for c in source_root.iter(W + "sdt") if any(t.get(W + "val") == "image:training_results" for t in c.findall(W + "sdtPr/" + W + "tag")))
    extent = next(chart_control.iter(WP + "extent"))
    original_extent = next(old_chart.iter(WP + "extent"))
    assert int(extent.get("cx")) <= int(original_extent.get("cx")) + 1
    assert int(extent.get("cy")) <= int(original_extent.get("cy")) + 1
    assert abs(int(extent.get("cx")) / int(extent.get("cy")) - 1000 / 420) < 0.001
    output_hash = digest(output)
    error(lambda: foliorivet.render_template(template, job, fixtures, output), "save")
    assert digest(output) == output_hash
    records[str(count)] = {"path": str(output), "sha256": output_hash, "preflight": preflight}
    if count == 3:
        script = {"commands": [{"command": "template.render", "params": {"template_path": str(template), "context": job, "assets_dir": str(fixtures), "output_path": str(run / "cli-output.docx")}}]}
        script_path = run / "job.json"
        script_path.write_text(json.dumps(script, ensure_ascii=False), encoding="utf-8")
        result = subprocess.run([str(cli), "run", "--script", str(script_path)], capture_output=True, text=True)
        (run / "cli.log").write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stderr
        cli_root, cli_parts = package(run / "cli-output.docx")
        assert E.tostring(root) == E.tostring(cli_root)
        assert parts["word/styles.xml"] == cli_parts["word/styles.xml"]
        assert sorted(hashlib.sha256(v).hexdigest() for k, v in parts.items() if k.startswith("word/media/")) == sorted(hashlib.sha256(v).hexdigest() for k, v in cli_parts.items() if k.startswith("word/media/"))

bad_cases = [
    (dict(context, class_distribution=None), "expected an array"),
    (dict(context, class_distribution=[{"class_name": "missing fields"}]), "missing required"),
    (dict(context, training_results={"path": "missing.png"}), "required asset"),
    (dict(context, training_results={"path": "../training-chart.png"}), "relative asset"),
    (dict(context, training_results={"path": None}), "required chart"),
    (dict(context, class_distribution=[context["class_distribution"][0]] * 1001), "1000 total rows"),
]
for index, (job, message) in enumerate(bad_cases):
    destination = run / f"failure-{index}.docx"
    error(lambda: foliorivet.preflight_template(template, job, fixtures), message)
    error(lambda: foliorivet.render_template(template, job, fixtures, destination), message)
    assert not destination.exists()
optional = dict(context, training_results={"path": None, "required": False, "unavailable": "Chart unavailable — synthetic fixture"})
foliorivet.render_template(template, optional, "", run / "optional-absent.docx")
optional_root, _ = package(run / "optional-absent.docx")
assert "Chart unavailable — synthetic fixture" in text(optional_root)
invoice = repo / "bindings/python/examples/invoice"
example_result = subprocess.run([sys.executable, "-I", str(invoice / "generate.py"), str(run / "invoice.docx")], capture_output=True, text=True)
(run / "invoice.log").write_text(example_result.stdout + example_result.stderr, encoding="utf-8")
assert example_result.returncode == 0, example_result.stderr
invoice_root, _ = package(run / "invoice.docx")
assert "GBP 360.00" in text(invoice_root)
assert len(next(invoice_root.iter(W + "tbl")).findall(W + "tr")) == 4
assert digest(template) == original
result = {"platform": platform.platform(), "python": sys.version, "native": _native.__file__, "template_sha256": original, "row_cases": records, "cli_equivalence": True, "failure_cases": len(bad_cases), "optional_absent": True, "invoice": str(run / "invoice.docx"), "word_review": "PENDING"}
(run / "result.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
print(json.dumps(result, indent=2))
