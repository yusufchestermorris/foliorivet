"""Audit saved artifacts and record reproducible local evidence without gate promotion."""
from pathlib import Path
import hashlib
import json
import sys
import tomllib
from xml.etree import ElementTree as ET
from zipfile import ZipFile

repo = Path(sys.argv[1]).resolve()
base = repo / "docs/wordcraft/evidence/stage3"
W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

records = json.loads((base / "word-automation.json").read_text(encoding="utf-8-sig"))
audits = []
for record in records:
    run = Path(record["saved"]).parent.parent
    result = json.loads((run / "result.json").read_text(encoding="utf-8"))
    values = {}
    with ZipFile(record["saved"]) as saved:
        assert saved.testzip() is None
        for name in saved.namelist():
            if name.startswith("word/") and name.endswith(".xml"):
                root = ET.fromstring(saved.read(name))
                for sdt in root.iter(W + "sdt"):
                    tag = sdt.find("./" + W + "sdtPr/" + W + "tag")
                    if tag is not None:
                        key = tag.get(W + "val")
                        assert key not in values, key
                        values[key] = "".join(e.text or "" for e in sdt.iter(W + "t"))
        document = ET.fromstring(saved.read("word/document.xml"))
        assert len(list(document.iter(W + "tbl"))) == 3
        assert len(list(document.iter(W + "sectPr"))) == 3
    assert values == {tag: data["text"] for tag, data in result["structure"]["controls"].items()}
    assert sha(Path(record["input"])) == record["input_sha256"].lower()
    audits.append({"platform": record["platform"], "word_resave_scalar_values": "PASS", "scalar_count": len(values), "tables": 3, "sections": 3})
locks = [tomllib.loads((repo / p).read_text()) for p in ["Cargo.lock", "bindings/python/Cargo.lock"]]
# Some package names have multiple locked versions; verify exact package tuples.
old_packages = {(p["name"], p["version"], p.get("source"), p.get("checksum")) for p in locks[0]["package"]}
new_packages = {(p["name"], p["version"], p.get("source"), p.get("checksum")) for p in locks[1]["package"]}
added_names = {p["name"] for p in json.loads((base / "added-dependencies.json").read_text())} | {"foliorivet-python"}
assert all(p in old_packages or p[0] in added_names for p in new_packages)
wheels = []
for wheel in sorted(base.glob("*/wheels/*.whl")):
    with ZipFile(wheel) as z:
        files = z.namelist()
        assert any(n.endswith((".pyd", ".so")) for n in files)
        assert all(f"foliorivet/licenses/{name}" in files for name in ["NOTICE", "LICENSE-MIT", "LICENSE-APACHE", "ADDED-DEPENDENCIES.json"])
        wheels.append({"path": str(wheel.relative_to(repo)), "sha256": sha(wheel), "bytes": wheel.stat().st_size,
                       "members": files, "distribution_review": "NOT CLEARED; local spike only"})
latest = {p: json.loads(sorted((base / p).glob("run-*/result.json"))[-1].read_text()) for p in ["windows", "linux"]}
assert latest["windows"]["structure"] == latest["linux"]["structure"]
hashes = {str(p.relative_to(repo)).replace('\\', '/'): sha(p) for p in sorted(base.rglob('*')) if p.is_file() and p.name != 'artifact-audit.json'}
hashes.update({p: sha(repo / p) for p in ['Cargo.lock', 'bindings/python/Cargo.lock', 'crates/engine/src/template.rs', 'crates/engine/src/io.rs', 'bindings/python/src/lib.rs']})
output = {"gate": "PENDING independent Word UI review and documented fidelity disposition", "audits": audits,
          "native_cli_cross_platform_semantics": "PASS", "existing_dependency_versions_retained": True,
          "wheels": wheels, "sha256": hashes}
manual = base / "manual-review-structural-audit.json"
if manual.is_file():
    review = json.loads(manual.read_text(encoding="utf-8"))
    output["manual_review"] = review
    output["gate"] = review["gate"]
(base / 'artifact-audit.json').write_text(json.dumps(output, indent=2), encoding='utf-8')
attribution = repo / 'ATTRIBUTION.md'
text = attribution.read_text(encoding='utf-8')
assets = [p for p in sorted(base.rglob('*')) if p.suffix.lower() in ['.docx', '.pdf', '.png']]
rows = []
for p in assets:
    rel = str(p.relative_to(repo)).replace('\\', '/')
    if f'`{rel}`' not in text:
        rows.append(f'| `{rel}` | FolioRivet synthetic fixture generation | Stage 3 scalar fixture, native/CLI output or Word/Poppler derivative; evidence/STAGE3.md | Original content MIT OR Apache-2.0; inherited template/theme metadata and PDF font subsets retain applicable terms; internal QA only | Local uncommitted evidence; includes deliberate invalid-input fixtures |')
if rows:
    with attribution.open('a', encoding='utf-8') as f:
        f.write('\n### Stage 3 local scalar qualification artifacts\n\n| Asset | Author | Source | License / status | Purpose |\n|---|---|---|---|---|\n' + '\n'.join(rows) + '\n')
print(json.dumps({'audits': audits, 'wheel_count': len(wheels), 'attribution_rows_added': len(rows)}, indent=2))
