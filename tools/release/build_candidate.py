"""Build local candidates with an already prepared Rust/Python toolchain; never upload."""
import argparse,hashlib,json,os,platform,shutil,subprocess,sys
from pathlib import Path


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--i18n',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--profile',choices=('dev','release'),default='release')
    p.add_argument('--offline',action='store_true')
    a=p.parse_args();repo=Path(__file__).resolve().parents[2]
    if sys.version_info[:2]!=(3,12):raise SystemExit('CPython 3.12 required')
    if platform.machine().lower() not in ('amd64','x86_64'):raise SystemExit('Only x64 is qualified')
    if sys.platform not in ('win32','linux'):raise SystemExit('Only Windows/Linux candidates supported')
    i18n=a.i18n.resolve();pin=(repo/'craft-i18n.rev').read_text().strip()
    git=['git','-c','core.autocrlf=true','-c','core.filemode=false','-c',f'safe.directory={i18n.as_posix()}','-C',str(i18n)]
    if subprocess.check_output(git+['rev-parse','HEAD'],text=True).strip()!=pin:raise SystemExit('Language-data revision mismatch')
    if subprocess.check_output(git+['status','--porcelain','--untracked-files=all'],text=True).strip():raise SystemExit('Language-data checkout is modified')
    out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    env=dict(os.environ,CRAFT_I18N_DIR=str(i18n),CRAFT_I18N_REQUIRED='1',PYO3_PYTHON=sys.executable)
    env['RUSTFLAGS']=(env.get('RUSTFLAGS','')+' --remap-path-prefix='+str(repo.parent)+'=/foliorivet-workspace').strip()
    records=[]
    def run(name,command):
        with (out/(name+'.log')).open('w',encoding='utf-8') as log:
            proc=subprocess.run(command,cwd=repo,env=env,stdout=log,stderr=subprocess.STDOUT)
        records.append({'name':name,'command':command,'exit':proc.returncode})
        (out/'commands.json').write_text(json.dumps(records,indent=2),encoding='utf-8')
        if proc.returncode:raise SystemExit(f'{name} failed; see retained log')
    options=['--locked']+(['--offline'] if a.offline else [])+(['--release'] if a.profile=='release' else [])
    run('rust-version',['rustc','-Vv']);run('maturin-version',[sys.executable,'-m','maturin','--version'])
    run('cli-build',['cargo','build',*options,'-p','wordcraft-cli','--bin','foliorivet-cli'])
    target=Path(env.get('CARGO_TARGET_DIR',str(repo/'target')))
    if not target.is_absolute():target=repo/target
    cli=target/('release' if a.profile=='release' else 'debug')/('foliorivet-cli.exe' if os.name=='nt' else 'foliorivet-cli')
    shutil.copy2(cli,out/cli.name)
    command=[sys.executable,'-m','maturin','build',*options,'--manifest-path','bindings/python/Cargo.toml','--interpreter',sys.executable,'--out',str(out)]
    if sys.platform=='linux':command+=['--compatibility','manylinux_2_35']
    run('wheel-build',command)
    manifest={'status':'LOCAL CANDIDATE; release acceptance still required','profile':a.profile,'python':sys.version,'platform':platform.platform(),'i18n_revision':pin,'files':{f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in out.iterdir() if f.is_file()}}
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print(out)

if __name__=='__main__':main()
