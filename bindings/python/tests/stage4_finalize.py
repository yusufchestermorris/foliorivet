"""Audit actual Word artifacts, wheels and asset provenance; never promote gates."""
import hashlib
import json
from pathlib import Path
import posixpath
import subprocess
import sys
from xml.etree import ElementTree as E
from zipfile import ZipFile
from PIL import Image, ImageChops

repo = Path(sys.argv[1]).resolve()
base = repo / "docs/wordcraft/evidence/stage4"
W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
WP = "{http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing}"

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def root(path):
    with ZipFile(path) as z:
        assert z.testzip() is None
        names = set(z.namelist())
        for name in names:
            if name.endswith((".xml", ".rels")):
                E.fromstring(z.read(name))
            if name.endswith(".rels"):
                parent = posixpath.dirname(posixpath.dirname(name))
                for rel in E.fromstring(z.read(name)):
                    if rel.get("TargetMode") == "External":
                        continue
                    target = rel.get("Target")
                    target = target.lstrip("/") if target.startswith("/") else posixpath.normpath(posixpath.join(parent, target))
                    assert target in names, (path, name, target)
        return E.fromstring(z.read("word/document.xml"))

def content(element):
    return "".join(n.text or "" for n in element.iter(W + "t"))

records = json.loads((base / "word-automation.json").read_text(encoding="utf-8-sig"))
audits = []
for item in records:
    original, saved = Path(item["input"]), Path(item["saved"])
    assert sha(original) == item["input_sha256"].lower()
    before, after = root(original), root(saved)
    assert content(before) == content(after), (item["platform"], item["case"], "text")
    for kind in ["tbl", "sectPr"]:
        assert len(list(before.iter(W + kind))) == len(list(after.iter(W + kind)))
    tables = list(after.iter(W + "tbl"))
    for old, new in zip(before.iter(W + "tbl"), tables, strict=True):
        assert len(list(old.iter(W + "tr"))) == len(list(new.iter(W + "tr")))
        assert [n.get(W + "w") for n in old.findall(W + "tblGrid/" + W + "gridCol")] == [n.get(W + "w") for n in new.findall(W + "tblGrid/" + W + "gridCol")]
        assert [dict(n.attrib) for n in old.iter(W + "tcW")] == [dict(n.attrib) for n in new.iter(W + "tcW")]
        assert [dict(n.attrib) for n in old.iter(W + "tblHeader")] == [dict(n.attrib) for n in new.iter(W + "tblHeader")]
    for kind, attr in [(W + "id", W + "val"), (WP + "docPr", "id")]:
        ids = [n.get(attr) for n in after.iter(kind)]
        assert len(ids) == len(set(ids))
    if item["case"] not in ["optional-absent", "invoice"]:
        chart_bytes = (base / "fixtures/training-chart.png").read_bytes()
        with ZipFile(saved) as z:
            assert chart_bytes in [z.read(p) for p in z.namelist() if p.startswith("word/media/")]
    audits.append({"platform": item["platform"], "case": item["case"], "word_open_save_reopen": "PASS automation", "original_unchanged": True, "text_grid_cell_widths_headers": "PASS", "relationship_targets_and_chart_media": "PASS", "saved_sha256": sha(saved), "independent_prompt_review": "PENDING"})
(base / "word-structural-audit.json").write_text(json.dumps(audits, indent=2), encoding="utf-8")

comparisons = []
for case in ["rows-0", "rows-1", "native-output", "rows-120", "optional-absent", "invoice"]:
    selected = [next(r for r in records if r["case"] == case and r["platform"] == p) for p in ["windows", "linux"]]
    folders = [Path(r["saved"]).parent / (case + "-render") for r in selected]
    pages = [sorted(folder.glob("page-*.png")) for folder in folders]
    assert pages[0] and len(pages[0]) == len(pages[1])
    for a, b in zip(*pages, strict=True):
        with Image.open(a) as ai, Image.open(b) as bi:
            same = ai.size == bi.size and ImageChops.difference(ai.convert("RGB"), bi.convert("RGB")).getbbox() is None
        comparisons.append({"case": case, "page": a.name, "identical_pixels": same})
    assert all(c["identical_pixels"] for c in comparisons if c["case"] == case)
(base / "pixel-comparison.json").write_text(json.dumps(comparisons, indent=2), encoding="utf-8")

wheels = []
for path in sorted(base.glob("*/wheels/*.whl")):
    with ZipFile(path) as z:
        members = z.namelist()
        assert any(p.endswith((".pyd", ".so")) for p in members)
        for notice in ["NOTICE", "LICENSE-MIT", "LICENSE-APACHE", "ADDED-DEPENDENCIES.json"]:
            assert "foliorivet/licenses/" + notice in members
    wheels.append({"path": str(path.relative_to(repo)), "sha256": sha(path), "bytes": path.stat().st_size, "members": members, "publication": "NOT AUTHORIZED"})
(base / "wheel-audit.json").write_text(json.dumps(wheels, indent=2), encoding="utf-8")

files = subprocess.run(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], cwd=repo, capture_output=True, check=True).stdout.decode().split("\0")
asset_ext = {"png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "ico", "icns", "svg", "pdf", "ai", "eps", "psd", "icc", "icm", "ttf", "otf", "woff", "woff2", "ase", "indd", "idml", "aco", "abr", "docx", "odt", "rtf", "mp4", "wav", "mp3"}
assets = [p for p in files if p and (p.startswith("docs/wordcraft/evidence/stage4/") or p.startswith("bindings/python/examples/invoice/")) and Path(p).suffix.lstrip(".").lower() in asset_ext]
attribution = repo / "ATTRIBUTION.md"
existing = attribution.read_text(encoding="utf-8")
added = []
for path in sorted(assets):
    if f"`{path}`" not in existing:
        added.append(f"| `{path}` | FolioRivet synthetic Stage 4 generation | Original fixture/example generator, native/CLI or Word/Poppler derivative; docs/wordcraft/evidence/STAGE4.md | Original code, text, chart and invoice layout MIT OR Apache-2.0; inherited template/theme metadata and PDF font subsets retain applicable terms; local QA only | Uncommitted evidence; no distribution authorized |")
if added:
    with attribution.open("a", encoding="utf-8") as out:
        out.write("\n" + "\n".join(added) + "\n")
hashes = {p: sha(repo / p) for p in sorted(assets)}
(base / "artifact-hashes.json").write_text(json.dumps(hashes, indent=2), encoding="utf-8")
print(json.dumps({"word_cases": len(audits), "cross_platform_pages": len(comparisons), "wheels": len(wheels), "assets": len(assets), "attribution_rows_added": len(added)}, indent=2))
