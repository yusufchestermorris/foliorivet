"""Create bounded synthetic fixtures, keeping the accepted master unchanged.

Run with the managed artifact Python runtime (lxml, python-docx and Pillow).
Original code, text, chart and example data: MIT OR Apache-2.0.
"""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
from zipfile import ZipFile, ZIP_DEFLATED

from lxml import etree as E
from PIL import Image, ImageDraw
from docx import Document
from docx.shared import Inches, Pt, RGBColor

repo = Path(sys.argv[1]).resolve()
base = repo / "docs/wordcraft/evidence/stage4/fixtures"
example = repo / "bindings/python/examples/invoice"
base.mkdir(parents=True, exist_ok=True)
example.mkdir(parents=True, exist_ok=True)
W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
NS = {"w": W[1:-1]}
source = repo / "docs/wordcraft/evidence/stage2/representative-v2/representative-master.docx"

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def tag(control):
    element = control.find(W + "sdtPr/" + W + "tag")
    return element.get(W + "val") if element is not None else ""

def unwrap(control):
    parent = control.getparent()
    at = parent.index(control)
    for child in list(control.find(W + "sdtContent")):
        parent.insert(at, child)
        at += 1
    parent.remove(control)

def wrap_runs(paragraph, name, identifier):
    control = E.Element(W + "sdt")
    props = E.SubElement(control, W + "sdtPr")
    E.SubElement(props, W + "id").set(W + "val", str(identifier))
    E.SubElement(props, W + "tag").set(W + "val", name)
    E.SubElement(props, W + "richText")
    content = E.SubElement(control, W + "sdtContent")
    for child in list(paragraph):
        if child.tag != W + "pPr":
            content.append(child)
    paragraph.append(control)

def patch_package(src, dst, document):
    with ZipFile(src) as zin, ZipFile(dst, "x", ZIP_DEFLATED) as zout:
        for item in zin.infolist():
            data = E.tostring(document, xml_declaration=True, encoding="UTF-8", standalone=True) if item.filename == "word/document.xml" else zin.read(item.filename)
            zout.writestr(item, data)

before = digest(source)
with ZipFile(source) as z:
    root = E.fromstring(z.read("word/document.xml"))
for control in reversed(root.findall(".//" + W + "sdt")):
    if tag(control).startswith(("repeat:", "if:", "value:item.")):
        unwrap(control)
row_control = next(c for c in root.iter(W + "sdt") if tag(c) == "rows:class_distribution")
table = row_control.getparent()
header = table.find(W + "tr")
for child in list(table):
    if child.tag == W + "tr" and child is not header:
        table.remove(child)
for index, (cell, field) in enumerate(zip(row_control.findall("w:sdtContent/w:tr/w:tc", NS), ["class_name", "training", "evaluation", "note"], strict=True)):
    wrap_runs(cell.find(W + "p"), "value:item." + field, 8400 + index)
image_control = next(c for c in root.iter(W + "sdt") if tag(c) == "image:training_results")
props = image_control.find(W + "sdtPr")
for child in list(props):
    if child.tag in [W + "richText", W + "text"]:
        props.remove(child)
if props.find(W + "picture") is None:
    E.SubElement(props, W + "picture")
template = base / "rows-chart-template.docx"
patch_package(source, template, root)
assert digest(source) == before

# Existing, independently drawn chart asset; no runtime chart calculation in Rust.
chart = Image.new("RGB", (1000, 420), "white")
draw = ImageDraw.Draw(chart)
draw.text((35, 20), "Synthetic class training samples", fill="#172d42", font_size=28)
for i, (label, value) in enumerate([("Class Alpha", 120), ("Class Beta", 90), ("Class Gamma", 60)]):
    y = 90 + i * 95
    draw.text((35, y + 14), label, fill="#172d42", font_size=22)
    draw.rectangle((220, y, 220 + value * 5, y + 48), fill="#247c91")
    draw.text((240 + value * 5, y + 12), str(value), fill="#172d42", font_size=22)
chart.save(base / "training-chart.png")
context = {
    "metadata": {"model_id": "Stage 4 synthetic model 42"},
    "class_distribution": [
        {"class_name": name, "training": training, "evaluation": evaluation, "note": note}
        for name, training, evaluation, note in [
            ("Class Alpha", 120, 30, "First result"),
            ("Crème brûlée — Ω", 90, 22, "Unicode & <XML>\nSecond line"),
            ("Class Gamma with a longer descriptive label", 60, 15, "Editable result"),
        ]
    ],
    "training_results": {"path": "training-chart.png"},
}
(base / "context.json").write_text(json.dumps(context, ensure_ascii=False, indent=2), encoding="utf-8")

# Invoice example: new original layout, using precomputed/preformatted business values.
doc = Document()
# Do not carry python-docx's bundled preview image into an original example.
for rel in list(doc.part.package.rels.values()):
    if rel.reltype.endswith("/metadata/thumbnail"):
        del doc.part.package.rels[rel.rId]
doc.core_properties.title = "Synthetic quotation"
doc.core_properties.author = "FolioRivet synthetic example"
doc.core_properties.comments = "Original synthetic example; MIT OR Apache-2.0."
section = doc.sections[0]
section.page_width, section.page_height = Inches(8.27), Inches(11.69)
section.top_margin = section.bottom_margin = Inches(0.8)
section.left_margin = section.right_margin = Inches(0.8)
doc.styles["Normal"].font.name = "Liberation Sans"
doc.styles["Normal"].font.size = Pt(10)
for name in ["Title", "Title Char"]:
    style = doc.styles[name]
    style.font.color.rgb = RGBColor(0, 0, 0)
    style.font.underline = False
    for borders in list(style.element.iter(W + "pBdr")):
        borders.getparent().remove(borders)
doc.add_paragraph("Synthetic quotation", "Title")
doc.add_paragraph("This editable example demonstrates repeated line items and supplied totals. All names and amounts are synthetic.")
for label, field in [("Quotation", "quote_id"), ("Customer", "customer")]:
    p = doc.add_paragraph(label + ": ")
    r = E.SubElement(p._p, W + "r")
    E.SubElement(r, W + "t").text = "Placeholder"
    # Keep the label outside the value target.
    control = E.Element(W + "sdt")
    pr = E.SubElement(control, W + "sdtPr")
    E.SubElement(pr, W + "id").set(W + "val", str(8500 + len(doc.paragraphs)))
    E.SubElement(pr, W + "tag").set(W + "val", "value:" + field)
    content = E.SubElement(control, W + "sdtContent")
    content.append(r)
    p._p.append(control)
t = doc.add_table(rows=2, cols=4)
t.style = "Table Grid"
t.autofit = False
for column, width in zip(t.columns, [3.3, 0.75, 1.25, 1.35], strict=True):
    column.width = Inches(width)
for row in t.rows:
    for cell, width in zip(row.cells, [3.3, 0.75, 1.25, 1.35], strict=True):
        cell.width = Inches(width)
        props = cell._tc.get_or_add_tcPr()
        margins = E.SubElement(props, W + "tcMar")
        for side in ["top", "bottom", "left", "right"]:
            entry = E.SubElement(margins, W + side)
            entry.set(W + "w", "80")
            entry.set(W + "type", "dxa")
for cell, text in zip(t.rows[0].cells, ["Description", "Quantity", "Unit price", "Amount"], strict=True):
    cell.text = text
    for run in cell.paragraphs[0].runs:
        run.bold = True
E.SubElement(t.rows[0]._tr.get_or_add_trPr(), W + "tblHeader")
for i, (cell, field) in enumerate(zip(t.rows[1].cells, ["description", "quantity", "unit_price", "amount"], strict=True)):
    cell.text = "Placeholder"
    wrap_runs(cell.paragraphs[0]._p, "value:item." + field, 8520 + i)
wrapper = E.Element(W + "sdt")
pr = E.SubElement(wrapper, W + "sdtPr")
E.SubElement(pr, W + "id").set(W + "val", "8510")
E.SubElement(pr, W + "tag").set(W + "val", "rows:lines")
content = E.SubElement(wrapper, W + "sdtContent")
row = t.rows[1]._tr
at = t._tbl.index(row)
content.append(row)
t._tbl.insert(at, wrapper)
for label, field in [("Subtotal", "subtotal"), ("Tax", "tax"), ("Total", "total")]:
    p = doc.add_paragraph()
    p.paragraph_format.space_before = Pt(8)
    p.add_run(label + ": ").bold = True
    p.add_run("Supplied amount")
    run = p._p[-1]
    c = E.Element(W + "sdt")
    pr = E.SubElement(c, W + "sdtPr")
    E.SubElement(pr, W + "id").set(W + "val", str(8600 + len(doc.paragraphs)))
    E.SubElement(pr, W + "tag").set(W + "val", "value:" + field)
    E.SubElement(c, W + "sdtContent").append(run)
    p._p.append(c)
doc.add_paragraph("Manual comments", "Heading 2")
doc.add_paragraph("Add editable notes here after generation.")
invoice = example / "template.docx"
if invoice.exists():
    raise FileExistsError(invoice)
doc.save(invoice)
invoice_context = {
    "quote_id": "SYN-0042", "customer": "Example Research Studio",
    "lines": [
        {"description": "Template preparation", "quantity": "2", "unit_price": "GBP 75.00", "amount": "GBP 150.00"},
        {"description": "Compatibility review — café", "quantity": "1", "unit_price": "GBP 120.00", "amount": "GBP 120.00"},
        {"description": "Documentation\nEditable handoff", "quantity": "1", "unit_price": "GBP 30.00", "amount": "GBP 30.00"},
    ],
    "subtotal": "GBP 300.00", "tax": "GBP 60.00", "total": "GBP 360.00",
}
(example / "input.json").write_text(json.dumps(invoice_context, ensure_ascii=False, indent=2), encoding="utf-8")
record = {
    "source": str(source.relative_to(repo)), "source_sha256": before,
    "generator": str(Path(__file__).relative_to(repo)),
    "license": "Original generator, synthetic content/chart and invoice layout MIT OR Apache-2.0; accepted master retains inherited template/theme metadata; local QA only, publication unauthorized.",
    "changes": ["Unwrap Stage 5 repeat/if/item controls in derivative only", "Retain header and one prototype in target table", "Wrap four prototype cell texts with value:item fields", "Explicit picture kind on existing inline chart control", "Preserve all non-document.xml package entries byte-for-byte"],
    "files": {str(p.relative_to(repo)): digest(p) for p in [template, base / "training-chart.png", base / "context.json", invoice, example / "input.json"]},
}
(base / "provenance.json").write_text(json.dumps(record, indent=2), encoding="utf-8")
(base / "artifact.md").write_text(
    "# Stage 4 fixture contract\n\nReference: " + str(source) + "\n\nSHA-256: " + before +
    "\n\nThe accepted six-page three-section master controls styles, typography, page geometry, headers/footers, borders and media. Only the recorded target-table and control-wrapper changes are intentional. Stage 2 reference renders remain in representative-v2/master-render. Repeated data rows retain their prototype formatting. The standalone invoice is original synthetic content with an editable comments area. Neither fixture authorizes distribution of inherited third-party assets.\n",
    encoding="utf-8",
)
print(json.dumps(record, indent=2))
