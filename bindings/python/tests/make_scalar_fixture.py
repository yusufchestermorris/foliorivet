"""Derive a scalar-only fixture from our synthetic Word-authored Stage 2 master.

Remove later-stage control wrappers, retaining their contents. No Word-produced
document is committed; all evidence remains local/uncommitted under workspace rules.
"""
from pathlib import Path
import hashlib
import json
import sys
from xml.dom import minidom
from zipfile import ZipFile, ZIP_DEFLATED

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
repo = Path(sys.argv[1]).resolve()
base = repo / "docs/wordcraft/evidence"
source = base / "stage2/representative-v2/representative-master.docx"
dest = base / "stage3/fixtures/scalar-template.docx"
dest.parent.mkdir(parents=True, exist_ok=True)
if dest.exists():
    raise SystemExit(f"Preserve existing fixture: {dest}")
with ZipFile(source) as original, ZipFile(dest, "x", ZIP_DEFLATED) as out:
    for item in original.infolist():
        data = original.read(item.filename)
        if item.filename.startswith("word/") and item.filename.endswith(".xml"):
            dom = minidom.parseString(data)
            for sdt in reversed(list(dom.getElementsByTagNameNS(W, "sdt"))):
                tags = sdt.getElementsByTagNameNS(W, "tag")
                tag = tags[0].getAttributeNS(W, "val") if tags else ""
                if tag == "value:metadata.model_id":
                    continue
                content = sdt.getElementsByTagNameNS(W, "sdtContent")[0]
                parent = sdt.parentNode
                for child in list(content.childNodes):
                    parent.insertBefore(child, sdt)
                parent.removeChild(sdt)
            # Wrap ordinary text in each header/footer, and one table-cell text run.
            candidate = None
            tag = None
            if item.filename.startswith(("word/header", "word/footer")):
                texts = dom.getElementsByTagNameNS(W, "t")
                if texts:
                    candidate = texts[0].parentNode
                    tag = "value:labels." + Path(item.filename).stem
            elif item.filename == "word/document.xml":
                cells = dom.getElementsByTagNameNS(W, "tc")
                if cells:
                    texts = cells[0].getElementsByTagNameNS(W, "t")
                    candidate = texts[0].parentNode
                    tag = "value:labels.table_heading"
            if candidate is not None:
                wrapper = dom.createElementNS(W, "w:sdt")
                props = dom.createElementNS(W, "w:sdtPr")
                tag_node = dom.createElementNS(W, "w:tag")
                tag_node.setAttributeNS(W, "w:val", tag)
                props.appendChild(tag_node)
                wrapper.appendChild(props)
                content = dom.createElementNS(W, "w:sdtContent")
                wrapper.appendChild(content)
                candidate.parentNode.replaceChild(wrapper, candidate)
                content.appendChild(candidate)
            data = dom.toxml(encoding="UTF-8")
        out.writestr(item, data)
record = {"source": str(source.relative_to(repo)), "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
          "fixture_sha256": hashlib.sha256(dest.read_bytes()).hexdigest(),
          "provenance": "FolioRivet synthetic Stage 2 master; locally derived by make_scalar_fixture.py; original unchanged; MIT OR Apache-2.0 for independently authored fixture content"}
(dest.parent / "provenance.json").write_text(json.dumps(record, indent=2), encoding="utf-8")
print(json.dumps(record, indent=2))
