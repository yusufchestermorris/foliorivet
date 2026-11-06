"""Export a review-only source candidate from an explicit source allowlist."""
import argparse,hashlib,json,subprocess,tomllib
from pathlib import Path
from zipfile import ZipFile,ZIP_DEFLATED

p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);a=p.parse_args()
repo=Path(__file__).resolve().parents[2]
if a.output.exists():raise SystemExit('Choose a new output path')
git=['git','-c',f'safe.directory={repo.as_posix()}','-C',str(repo)]
tracked=subprocess.check_output(git+['ls-files','-z']).decode().split('\0')
files={n for n in tracked if n and (repo/n).is_file()}
for prefix in ['bindings/python','tools/release','docs/release','crates/engine/src/template']:
 for f in (repo/prefix).rglob('*'):
  if f.is_file():files.add(f.relative_to(repo).as_posix())
files.update(n for n in ['CHANGELOG.md','mkdocs.yml','docs-requirements.txt'] if (repo/n).is_file())
for prefix in ['docs/api','docs/concepts','docs/examples','docs/getting-started','docs/reference']:
 for f in (repo/prefix).rglob('*'):
  if f.is_file():files.add(f.relative_to(repo).as_posix())
files.add('docs/index.md')
version=tomllib.loads((repo/'bindings/python/pyproject.toml').read_text())['project']['version']
for n in ['crates/engine/src/template.rs','crates/doc/src/props/theme.rs','crates/docx/tests/it/fidelity_preservation.rs','crates/docx/tests/it/style_fidelity.rs','crates/engine/src/tests/style_fidelity.rs']:
 if (repo/n).is_file():files.add(n)
def excluded(n):
 parts=Path(n).parts
 return any(x in parts for x in ['target','dist','__pycache__','.git','.tooling','.private-archive']) or n.startswith(('.github/workflows/','docs/wordcraft/','bindings/python/tests/')) or n.startswith('docs/brand/artcraft-') or n.endswith(('.pyc','.log'))
files=sorted(n for n in files if not excluded(n))
manifest=[]
a.output.parent.mkdir(parents=True,exist_ok=True)
with ZipFile(a.output,'x',ZIP_DEFLATED) as z:
 for n in files:
  data=(repo/n).read_bytes();z.writestr(f'foliorivet-{version}/'+n,data)
  manifest.append({'path':n,'sha256':hashlib.sha256(data).hexdigest()})
 z.writestr(f'foliorivet-{version}/SOURCE-MANIFEST.json',json.dumps({'upstream':subprocess.check_output(git+['rev-parse','HEAD'],text=True).strip(),'status':'REVIEW ONLY; not release-cleared','files':manifest},indent=2))
print(json.dumps({'source':str(a.output),'files':len(files),'sha256':hashlib.sha256(a.output.read_bytes()).hexdigest(),'status':'review candidate; not clearance'}))
