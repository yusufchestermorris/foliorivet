"""Record added dependency provenance and carry upstream/new dependency notices."""
from pathlib import Path
import hashlib
import json
import shutil
import sys
import tomllib

repo = Path(sys.argv[1]).resolve()
binding = repo / "bindings/python"
metadata = json.loads((repo / "docs/wordcraft/evidence/stage3/binding-metadata.json").read_text(encoding="utf-8-sig"))
upstream = tomllib.loads((repo / "Cargo.lock").read_text())
old = {(p["name"], p["version"]) for p in upstream["package"]}
added = [p for p in metadata["packages"] if (p["name"], p["version"]) not in old and p["name"] != "foliorivet-python"]
notices = binding / "python/foliorivet/licenses"
notices.mkdir(exist_ok=True)
for name in ["LICENSE-MIT", "LICENSE-APACHE", "NOTICE"]:
    shutil.copyfile(repo / name, notices / name)
records = []
for pkg in added:
    root = Path(pkg["manifest_path"]).parent
    files = sorted({*root.glob("LICENSE*"), *root.glob("license*")})
    assert pkg["license"] in ["MIT OR Apache-2.0", "MIT", "Apache-2.0", "Unlicense", "Apache-2.0 OR MIT", "Apache-2.0 WITH LLVM-exception"], pkg
    copied = []
    for source in files:
        if not source.is_file():
            continue
        dest = notices / f'{pkg["name"]}-{pkg["version"]}-{source.name}'
        shutil.copyfile(source, dest)
        copied.append({"file": dest.name, "sha256": hashlib.sha256(dest.read_bytes()).hexdigest()})
    assert copied, pkg["name"]
    records.append({"name": pkg["name"], "version": pkg["version"], "source": pkg["source"],
                    "license": pkg["license"], "licenses": copied,
                    "disposition": "permissive addition accepted for local spike; license texts inspected; no distribution clearance"})
(notices / "ADDED-DEPENDENCIES.json").write_text(json.dumps(records, indent=2), encoding="utf-8")
(repo / "docs/wordcraft/evidence/stage3/added-dependencies.json").write_text(json.dumps(records, indent=2), encoding="utf-8")
print(json.dumps([{k: p[k] for k in ["name", "version", "license"]} for p in records], indent=2))
