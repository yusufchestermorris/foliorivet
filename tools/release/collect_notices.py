"""Collect exact source notice texts. Inventory is not automatic legal clearance."""
import argparse,hashlib,json,urllib.request,urllib.error
from pathlib import Path

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('metadata',type=Path);p.add_argument('output',type=Path);p.add_argument('--fetch-missing',action='store_true');p.add_argument('--supplements',type=Path);a=p.parse_args()
 m=json.loads(a.metadata.read_text(encoding='utf-8')); a.output.mkdir(parents=True,exist_ok=True);rows=[]
 for pkg in m['packages']:
  root=Path(pkg['manifest_path']).parent; dest=a.output/(pkg['name']+'-'+pkg['version']);dest.mkdir(exist_ok=True)
  candidates=[x for x in root.iterdir() if x.is_file() and x.name.lower().startswith(('license','licence','copying','copyright','notice'))]
  if pkg.get('license_file'):
   f=root/pkg['license_file']
   if f.is_file() and f not in candidates:candidates.append(f)
  if not candidates and (pkg['source'] is None or pkg['source'].startswith('git+')):
   for parent in root.parents:
    if (parent/'.git').exists():
     candidates=[x for x in parent.iterdir() if x.is_file() and x.name.lower().startswith(('license','notice'))]
     for directory in (parent/'licenses').glob('**/'+pkg['name']) if (parent/'licenses').is_dir() else []:
      candidates += [x for x in directory.iterdir() if x.is_file() and x.name.lower().startswith(('license','notice','copying'))]
     break
  notices=[]
  for i,f in enumerate(candidates):
   name=f'{i:02d}-'+f.name
   data=f.read_bytes();(dest/name).write_bytes(data);notices.append({'file':name,'sha256':hashlib.sha256(data).hexdigest(),'source':pkg.get('source') or 'workspace source','source_file':f.name})
  if not notices and a.supplements:
   for f in a.supplements.glob(pkg['name']+'-LICENSE*'):
    data=f.read_bytes();name=f.name.removeprefix(pkg['name']+'-');(dest/name).write_bytes(data)
    notices.append({'file':name,'sha256':hashlib.sha256(data).hexdigest(),'source':'pinned upstream supplemental text; see supplements provenance'})
  vcs=root/'.cargo_vcs_info.json'
  if not notices and a.fetch_missing and vcs.is_file() and (pkg.get('repository') or '').startswith('https://github.com/'):
   v=json.loads(vcs.read_text());repo=pkg['repository'].removeprefix('https://github.com/').removesuffix('.git');rev=v['git']['sha1'];sub=v.get('path_in_vcs','')
   prefixes=[sub,''] if sub else ['']
   for prefix in prefixes:
    for name in ['LICENSE','LICENSE.md','LICENSE-MIT','LICENSE-APACHE','LICENSE-ZLIB','COPYING']:
     rel='/'.join(x for x in [prefix,name] if x);url=f'https://raw.githubusercontent.com/{repo}/{rev}/{rel}'
     try:
      with urllib.request.urlopen(url,timeout=20) as response:data=response.read()
     except urllib.error.HTTPError as e:
      if e.code==404:continue
      raise
     (dest/name).write_bytes(data);notices.append({'file':name,'sha256':hashlib.sha256(data).hexdigest(),'source':url})
    if notices:break
  rows.append({'name':pkg['name'],'version':pkg['version'],'declared_license':pkg['license'],'notices':notices,'review':'texts collected; inclusion and obligations require review' if notices else 'HOLD: no source text'})
 result={'scope':'Input metadata package set; build dependencies may be included; not a binary SBOM','packages':rows,'missing':[r['name'] for r in rows if not r['notices']]}
 (a.output/'inventory.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8');print(json.dumps({'packages':len(rows),'missing':result['missing']}));return bool(result['missing'])
if __name__=='__main__':raise SystemExit(main())
