"""Run using an installed wheel, preferably python -I from outside the source tree."""
from pathlib import Path
import hashlib
import importlib.machinery
import json
import platform
import posixpath
import subprocess
import sys
from datetime import datetime, timezone
from zipfile import ZipFile, ZIP_DEFLATED
from xml.etree import ElementTree as ET

import foliorivet
from foliorivet import _native, TemplateError

repo, evidence, cli = map(lambda p: Path(p).resolve(), sys.argv[1:])
run = evidence / datetime.now(timezone.utc).strftime("run-%Y%m%dT%H%M%S%fZ")
run.mkdir(parents=True)
template = repo / "docs/wordcraft/evidence/stage3/fixtures/scalar-template.docx"
original_hash = hashlib.sha256(template.read_bytes()).hexdigest()
assert any(_native.__file__.endswith(s) for s in importlib.machinery.EXTENSION_SUFFIXES)
assert "site-packages" in _native.__file__, _native.__file__
for notice in ["LICENSE-MIT", "LICENSE-APACHE", "NOTICE", "ADDED-DEPENDENCIES.json"]:
    assert (Path(foliorivet.__file__).parent / "licenses" / notice).is_file(), notice
inspection = foliorivet.inspect_template(template)
assert all(t["supported"] for t in inspection["targets"]), inspection
tags = [t["tag"] for t in inspection["targets"]]
assert "value:metadata.model_id" in tags and "value:labels.table_heading" in tags
assert any("header" in tag for tag in tags) and any("footer" in tag for tag in tags)
context = {"metadata": {"model_id": "模型 café & <ready>"},
           "labels": {tag.split(".", 1)[1]: "Review — Ω" for tag in tags if tag.startswith("value:labels.")}}
assert foliorivet.preflight_template(template, context)["scalar_count"] == len(tags)

def expect_error(call, contains):
    try:
        call()
    except TemplateError as exc:
        text = str(exc)
        assert contains in text and "scope=document" in text, text
        errors.append(text)
    else:
        raise AssertionError(f"Expected TemplateError containing {contains}")

errors = []
output = run / "native-output.docx"
# Native render cannot delegate to a CLI or any child process.
original_popen = subprocess.Popen
def denied(*a, **kw):
    raise AssertionError("Rendering attempted a child process")
subprocess.Popen = denied
try:
    assert foliorivet.render_template(template, context, "", output) == str(output)
finally:
    subprocess.Popen = original_popen

expect_error(lambda: foliorivet.render_template(template, context, "", template), "new staging path")
expect_error(lambda: foliorivet.render_template(template, context, "", output), "new staging path")
expect_error(lambda: foliorivet.preflight_template(template, "{"), "invalid JSON")
expect_error(lambda: foliorivet.preflight_template(template, {}), "missing required")
expect_error(lambda: foliorivet.preflight_template(template, dict(context, deleted="required")), "missing required target")
expect_error(lambda: foliorivet.preflight_template(template, dict(context, metadata={"model_id": None})), "expected a string")
expect_error(lambda: foliorivet.preflight_template(template, []), "expected a JSON object")
expect_error(lambda: foliorivet.preflight_template(template, dict(context, metadata={"model_id": "\x00"})), "unsupported control")
expect_error(lambda: foliorivet.render_template(template, context, "required-chart.png", run / "asset.docx"), "assets are unsupported")
expect_error(lambda: foliorivet.render_template(run / "absent.docx", context, "", run / "absent-output.docx"), "load")
expect_error(lambda: foliorivet.render_template(template, context, "", run / "no-parent/out.docx"), "save")
assert not (run / "absent-output.docx").exists()
assert not (run / "asset.docx").exists()
assert hashlib.sha256(template.read_bytes()).hexdigest() == original_hash

# Repeated independent jobs do not retain scalar state.
for name, value in [("empty", ""), ("long", "Long label é " * 100), ("numeric", 42), ("boolean", False)]:
    changed = json.loads(json.dumps(context))
    changed["metadata"]["model_id"] = value
    foliorivet.render_template(template, changed, "", run / f"{name}.docx")
changed = json.loads(json.dumps(context))
changed["labels"]["table_heading"] = "First\nSecond\tΩ"
foliorivet.render_template(template, changed, "", run / "multiline.docx")

# Full representative requires explicit input for its block/optional targets.
full = repo / "docs/wordcraft/evidence/stage2/representative-v2/representative-master.docx"
expect_error(lambda: foliorivet.preflight_template(full, context), "missing required value")

# Exercise ambiguity and invalid required target via actual installed native preflight.
for label, new_tag, message in [("duplicate", "value:metadata.model_id", "ambiguous"),
                               ("malformed", "value:bad..field", "dotted ASCII")]:
    variant = run / f"{label}-template.docx"
    with ZipFile(template) as src, ZipFile(variant, "x", ZIP_DEFLATED) as dst:
        for item in src.infolist():
            data = src.read(item.filename)
            if item.filename == "word/document.xml":
                data = data.replace(b"value:labels.table_heading", new_tag.encode())
            dst.writestr(item, data)
    expect_error(lambda: foliorivet.preflight_template(variant, context), message)

job = {"commands": [{"command": "template.render", "params": {
    "template_path": str(template), "context": context, "assets_dir": "", "output_path": str(run / "cli-output.docx")}}]}
(run / "job.json").write_text(json.dumps(job, ensure_ascii=False), encoding="utf-8")
result = subprocess.run([str(cli), "run", "--script", str(run / "job.json")], capture_output=True, text=True)
(run / "cli.log").write_text(result.stdout + result.stderr, encoding="utf-8")
assert result.returncode == 0, result.stderr
baseline_result = subprocess.run([str(cli), "convert", str(template), str(run / "unpopulated-roundtrip.docx")], capture_output=True, text=True)
assert baseline_result.returncode == 0, baseline_result.stderr
W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"

def tree_signature(element):
    # Revision-session identifiers are provenance, not document formatting.
    attributes = {k: v for k, v in element.attrib.items() if not k.removeprefix(W).startswith("rsid")}
    if element.tag == W + "cols":
        attributes.setdefault(W + "num", "1")
    children = [tree_signature(c) for c in element if c.tag != W + "rsid"
                and not (c.tag in [W + "pPr", W + "rPr"] and not len(c) and not c.attrib)]
    if element.tag == W + "sectPr":
        if not any(c.tag == W + "type" for c in element):
            children.append((W + "type", [(W + "val", "nextPage")], "", []))
        children.sort(key=repr)
    return (element.tag, sorted(attributes.items()), element.text or "", children)

def structure(path):
    with ZipFile(path) as z:
        assert z.testzip() is None
        roots = {n: ET.fromstring(z.read(n)) for n in z.namelist() if n.endswith(".xml") or n.endswith(".rels")}
        xml = {n: r for n, r in roots.items() if n.startswith("word/")}
        relns = "{http://schemas.openxmlformats.org/package/2006/relationships}"
        rid = "{http://schemas.openxmlformats.org/officeDocument/2006/relationships}id"
        names = set(z.namelist())
        for name, root in roots.items():
            if not name.endswith(".rels"):
                continue
            folder = posixpath.dirname(posixpath.dirname(name))
            for rel in root:
                if rel.get("TargetMode") != "External":
                    target = posixpath.normpath(posixpath.join(folder, rel.get("Target", ""))).lstrip("/")
                    assert target in names, (name, target)
        # Compare section header/footer associations by their semantic target,
        # since relationship numbers and part filenames are regenerated on save.
        rels = {e.get("Id"): e.get("Target") for e in roots["word/_rels/document.xml.rels"]}
        for element in xml["word/document.xml"].iter():
            if element.tag in [W + "headerReference", W + "footerReference"]:
                target = posixpath.normpath(posixpath.join("word", rels[element.get(rid)]))
                part = xml[target]
                tagged = [e.get(W + "val") for e in part.iter(W + "tag")]
                identity = tagged or ["".join(e.text or "" for e in part.iter(W + "t"))]
                element.set(rid, json.dumps(identity, ensure_ascii=False))
        controls = {}
        for name, root in xml.items():
            for sdt in root.iter(W + "sdt"):
                tag = sdt.find("./" + W + "sdtPr/" + W + "tag")
                if tag is not None:
                    key = tag.get(W + "val")
                    content = sdt.find(W + "sdtContent")
                    text = "".join("\n" if e.tag == W + "br" else "\t" if e.tag == W + "tab" else e.text or "" for e in content.iter() if e.tag in [W + "t", W + "br", W + "tab"])
                    controls[key] = {"text": text, "run_properties": [tree_signature(e) for e in content.iter(W + "rPr")]}
        return {"controls": controls,
                "geometry": {n: [tree_signature(e) for e in r.iter() if e.tag in [W + p for p in ["tblPr", "tblGrid", "trPr", "tcPr", "sectPr"]]] for n, r in xml.items() if n == "word/document.xml"},
                "styles": tree_signature(xml["word/styles.xml"]),
                "media": {n: hashlib.sha256(z.read(n)).hexdigest() for n in z.namelist() if n.startswith("word/media/")}}

native = structure(output)
command_line = structure(run / "cli-output.docx")
assert native == command_line, "CLI and binding differ semantically"
baseline = structure(template)
unpopulated = structure(run / "unpopulated-roundtrip.docx")
assert native["geometry"] == baseline["geometry"], "Template table/section properties changed"
assert native["styles"] == unpopulated["styles"], "Scalar population changed styles beyond an engine roundtrip"
# Repaired theme references, gallery flags and level-only numbering must match the source.
# Only harmless revision IDs and package representation differences are normalized.
assert native["styles"][3] == baseline["styles"][3], "Source style properties changed"
def style_catalog(signature):
    return [(dict(e[1]).get(W + "styleId"), [c for c in e[3] if c[0] == W + "name"]) for e in signature[3] if e[0] == W + "style"]
assert style_catalog(native["styles"]) == style_catalog(baseline["styles"]), "Named style identities changed"
assert native["media"] == baseline["media"], "Media changed"
for tag in tags:
    key = tag.removeprefix("value:").split(".")
    expected = context[key[0]][key[1]]
    assert native["controls"][tag]["text"] == expected, (tag, native["controls"][tag])
    assert native["controls"][tag]["run_properties"] == baseline["controls"][tag]["run_properties"], tag
for name, expected in [("empty", ""), ("long", "Long label é " * 100), ("numeric", "42"), ("boolean", "false")]:
    assert structure(run / f"{name}.docx")["controls"]["value:metadata.model_id"]["text"] == expected
assert structure(run / "multiline.docx")["controls"]["value:labels.table_heading"]["text"] == "First\nSecond\tΩ"
record = {"result": "PASS", "python": sys.version, "platform": platform.platform(), "module": _native.__file__,
          "template_sha256": original_hash, "scalar_targets": len(tags), "errors": errors,
          "native_output": str(output), "cli_output": str(run / "cli-output.docx"),
          "structure": native, "word_review": "PENDING", "no_compiler_runtime_requirement": "wheel installed with --no-index; generation uses installed native module"}
record["inherited_style_normalization"] = baseline["styles"][3] != unpopulated["styles"][3]
record["style_root_namespace_metadata_differs"] = baseline["styles"][:3] != unpopulated["styles"][:3]
(run / "result.json").write_text(json.dumps(record, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps({k: v for k, v in record.items() if k != "structure"}, ensure_ascii=False, indent=2))
