"""Installed CPython native and CLI Stage 5 qualification; stdlib only, run -I."""
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

repo,evidence,cli=(Path(p).resolve() for p in sys.argv[1:])
run=evidence/datetime.now(timezone.utc).strftime("run-%Y%m%dT%H%M%S%fZ")
run.mkdir(parents=True)
fixtures=repo/"docs/wordcraft/evidence/stage5/fixtures"
template=fixtures/"report-template.docx"
context=json.loads((fixtures/"context.json").read_text(encoding="utf-8"))
W="{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
R="{http://schemas.openxmlformats.org/officeDocument/2006/relationships}"
WP="{http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing}"
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def text(n): return "".join((e.text or "") if e.tag==W+"t" else "\n" if e.tag==W+"br" else "" for e in n.iter())
def package(p):
    with ZipFile(p) as z:
        assert z.testzip() is None
        parts={n:z.read(n) for n in z.namelist()}
    for name,data in parts.items():
        if name.endswith((".xml",".rels")):
            root=E.fromstring(data)
            if name.endswith(".rels"):
                ids=[]
                for rel in root:
                    ids.append(rel.get("Id"))
                    if rel.get("TargetMode")!="External":
                        target=rel.get("Target")
                        target=target.lstrip("/") if target.startswith("/") else posixpath.normpath(posixpath.join(posixpath.dirname(posixpath.dirname(name)),target))
                        assert target in parts,(name,target)
                assert len(ids)==len(set(ids))
            else:
                relname=posixpath.join(posixpath.dirname(name),"_rels",posixpath.basename(name)+".rels")
                ids={r.get("Id") for r in E.fromstring(parts[relname])} if relname in parts else set()
                for node in root.iter():
                    for k,v in node.attrib.items():
                        if k in [R+"id",R+"embed",R+"link"]: assert v in ids,(name,v)
    root=E.fromstring(parts["word/document.xml"])
    for kind,attr in [(W+"id",W+"val"),(WP+"docPr","id")]:
        ids=[n.get(attr) for n in root.iter(kind)]
        assert len(ids)==len(set(ids)),(kind,ids)
    return root,parts
def failure(fn,message):
    try: fn()
    except TemplateError as e:
        assert message in str(e),(message,str(e))
        assert "target=" in str(e) and "scope=" in str(e)
    else: raise AssertionError("Expected failure: "+message)

def sections(root,parts):
    # Removing every drawing legitimately renumbers relationships. Compare each
    # section's resolved header/footer content as well as all section properties.
    rels={r.get("Id"):r.get("Target") for r in E.fromstring(parts["word/_rels/document.xml.rels"])}
    result=[]
    for section in root.iter(W+"sectPr"):
        section=deepcopy(section)
        for n in section.iter():
            if R+"id" in n.attrib:
                target=posixpath.normpath(posixpath.join("word",rels[n.get(R+"id")]))
                n.set(R+"id",hashlib.sha256(parts[target]).hexdigest())
        result.append(E.tostring(section))
    return result

assert "site-packages" in _native.__file__
assert any(_native.__file__.endswith(s) for s in importlib.machinery.EXTENSION_SUFFIXES)
original=digest(template)
inspection=foliorivet.inspect_template(template)
assert all(t["supported"] for t in inspection["targets"]),inspection
(run/"inspection.json").write_text(json.dumps(inspection,indent=2),encoding="utf-8")
baseline=run/"baseline.docx"
p=subprocess.run([str(cli),"convert",str(template),str(baseline)],capture_output=True,text=True)
assert p.returncode==0,p.stderr
base_root,base_parts=package(baseline)
prototype_table=list(base_root.iter(W+"tbl"))[1]
cases={}
for count in [0,1,3,15]:
    for evaluation in [False,True]:
        for chart in [False,True]:
            job=deepcopy(context)
            job["has_evaluation"]=evaluation
            job["class_training_results"]=[deepcopy(context["class_training_results"][i%3]) for i in range(count)]
            for i,item in enumerate(job["class_training_results"]):
                item["class_name"]=f"{i+1:03d} "+item["class_name"]+(" — long label café "*7 if count==15 and i==7 else "")
                if not chart: item["training_results"]={"path":None,"required":False,"unavailable":"Chart unavailable for this synthetic class"}
            name=f"sections-{count}-evaluation-{int(evaluation)}-chart-{int(chart)}"
            out=run/(name+".docx")
            checked=foliorivet.preflight_template(template,job,fixtures)
            assert checked["repeated_items"]==count
            assert checked["included_optional_blocks"]==int(evaluation)
            foliorivet.render_template(template,job,fixtures,out)
            root,parts=package(out)
            tags=[n.get(W+"val") for n in root.iter(W+"tag")]
            assert not any(t.startswith(("repeat:","if:","rows:","image:","value:item.")) for t in tags)
            assert len(list(root.iter(W+"tbl")))==2+count
            for table,item in zip(list(root.iter(W+"tbl"))[1:1+count],job["class_training_results"],strict=True):
                assert text(table.findall(W+"tr")[1].findall(W+"tc")[1])==item["precision"]
                for node in ["tblPr","tblGrid"]:
                    assert E.tostring(table.find(W+node))==E.tostring(prototype_table.find(W+node))
                for cell,old in zip(table.iter(W+"tc"),prototype_table.iter(W+"tc"),strict=True):
                    assert E.tostring(cell.find(W+"tcPr"))==E.tostring(old.find(W+"tcPr"))
                    assert E.tostring(cell.find(W+"p/"+W+"pPr"))==E.tostring(old.find(W+"p/"+W+"pPr"))
            assert len(list(root.iter(WP+"docPr")))==(count if chart else 0)
            alltext=text(root)
            assert alltext.count("Manual class comments remain editable")==count
            assert alltext.count("Class training results")==count
            eval_text="".join(text(s) for s in E.fromstring(ZipFile(template).read("word/document.xml")).iter(W+"sdt") if any(n.get(W+"val")=="if:has_evaluation" for n in s.findall(W+"sdtPr/"+W+"tag")))
            assert bool(eval_text in alltext)==evaluation,(name,eval_text)
            cursor=0
            for item in job["class_training_results"]:
                cursor=alltext.index(item["class_name"],cursor)+len(item["class_name"])
                assert item["precision"] in alltext
            assert parts["word/styles.xml"]==base_parts["word/styles.xml"]
            assert sections(root,parts)==sections(base_root,base_parts)
            cases[name]={"path":str(out),"sha256":digest(out),"preflight":checked}
            if count==3 and evaluation and chart:
                (run/"native-output.docx").write_bytes(out.read_bytes())
                script={"commands":[{"command":"template.render","params":{"template_path":str(template),"context":job,"assets_dir":str(fixtures),"output_path":str(run/"cli-output.docx")}}]}
                scriptpath=run/"cli-job.json"; scriptpath.write_text(json.dumps(script),encoding="utf-8")
                result=subprocess.run([str(cli),"run","--script",str(scriptpath)],capture_output=True,text=True)
                (run/"cli.log").write_text(result.stdout+result.stderr,encoding="utf-8")
                assert result.returncode==0,result.stderr
                cli_root,cli_parts=package(run/"cli-output.docx")
                assert parts["word/document.xml"]==cli_parts["word/document.xml"]
                assert {n:b for n,b in parts.items() if n.startswith("word/media/")}=={n:b for n,b in cli_parts.items() if n.startswith("word/media/")}

bad=[(dict(context,has_evaluation="false"),"explicit boolean"),(dict(context,class_training_results=[{}]),"missing required"),(dict(context,class_training_results=[dict(context["class_training_results"][0],training_results={"path":"missing.png"})]),"required asset"),(dict(context,class_training_results=[context["class_training_results"][0]]*1001),"1000 items"),(dict(context,unused=42),"missing required target")]
for i,(job,message) in enumerate(bad):
    out=run/f"failure-{i}.docx"
    failure(lambda:foliorivet.preflight_template(template,job,fixtures),message)
    failure(lambda:foliorivet.render_template(template,job,fixtures,out),message)
    assert not out.exists()
for variant in ["missing","ambiguous"]:
    with ZipFile(template) as z: parts={n:z.read(n) for n in z.namelist()}
    root=E.fromstring(parts["word/document.xml"])
    parent={c:p for p in root.iter() for c in p}
    target=next(s for s in root.iter(W+"sdt") if any(n.get(W+"val")=="value:item.class_name" for n in s.findall(W+"sdtPr/"+W+"tag")))
    container=parent[target]
    if variant=="missing":
        index=list(container).index(target)
        container.remove(target)
        for child in list(target.find(W+"sdtContent")): container.insert(index,child); index+=1
    else: container.insert(list(container).index(target)+1,deepcopy(target))
    parts["word/document.xml"]=E.tostring(root)
    bad_template=run/(variant+"-target.docx")
    with ZipFile(bad_template,"w",ZIP_DEFLATED) as z:
        for n,b in parts.items(): z.writestr(n,b)
    destination=run/(variant+"-failure.docx")
    failure(lambda:foliorivet.render_template(bad_template,context,fixtures,destination),"missing required target" if variant=="missing" else "ambiguous")
    assert not destination.exists()
output=run/"native-output.docx"; before=digest(output)
failure(lambda:foliorivet.render_template(template,context,fixtures,output),"save")
assert digest(output)==before and digest(template)==original
example=repo/"bindings/python/examples/project-status"
job=json.loads((example/"input.json").read_text())
foliorivet.render_template(example/"template.docx",job,"",run/"project-status.docx")
project,_=package(run/"project-status.docx")
assert len(list(project.iter(W+"tbl")))==2
assert "Review actions" in text(project)
result={"platform":platform.platform(),"python":sys.version,"native_module":_native.__file__,"template_sha256":original,"block_cases":cases,"negative_cases":len(bad)+2,"cli_equivalent":True,"project_status":str(run/"project-status.docx")}
(run/"result.json").write_text(json.dumps(result,indent=2),encoding="utf-8")
print(json.dumps({"run":str(run),"cases":len(cases),"result":"PASS"}))
