"""Run public examples through an installed wheel and matching CLI on a trial host."""
import argparse,hashlib,json,platform,subprocess,sys
from pathlib import Path
from zipfile import ZipFile
import foliorivet


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--examples',type=Path,required=True)
    parser.add_argument('--cli',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=False)
    results=[]
    for name in ('invoice','project-status','training-evaluation'):
        source=args.examples.resolve()/name
        template=source/'template.docx'
        before=hashlib.sha256(template.read_bytes()).hexdigest()
        context=json.loads((source/'input.json').read_text(encoding='utf-8-sig'))
        native=args.output.resolve()/(name+'-python.docx')
        cli_output=args.output.resolve()/(name+'-cli.docx')
        foliorivet.inspect_template(template)
        foliorivet.preflight_template(template,context)
        # A source checkout must exercise the installed API's failure paths too.
        for bad_context in ('{', {}):
            try:
                foliorivet.preflight_template(template,bad_context)
            except foliorivet.TemplateError:
                pass
            else:
                raise RuntimeError(name+': invalid or missing input was accepted')
        foliorivet.render_template(template,context,'',native)
        try:
            foliorivet.render_template(template,context,'',native)
        except foliorivet.TemplateError:
            pass
        else:
            raise RuntimeError('Existing output was not refused')
        job=args.output/(name+'-job.json')
        job.write_text(json.dumps({'commands':[{'command':'template.render','params':{'template_path':str(template),'context':context,'assets_dir':'','output_path':str(cli_output)}}]}),encoding='utf-8')
        completed=subprocess.run([str(args.cli.resolve()),'run','--script',str(job.resolve())],capture_output=True,text=True,check=True)
        (args.output/(name+'-cli.log')).write_text(completed.stdout+completed.stderr,encoding='utf-8')
        with ZipFile(native) as left,ZipFile(cli_output) as right:
            if set(left.namelist())!=set(right.namelist()) or any(left.read(n)!=right.read(n) for n in left.namelist()):
                raise RuntimeError(name+': Python/CLI package members differ')
        if hashlib.sha256(template.read_bytes()).hexdigest()!=before:
            raise RuntimeError(name+': template changed')
        results.append({'example':name,'template_sha256':before,'output_sha256':hashlib.sha256(native.read_bytes()).hexdigest(),'native_cli_agreement':'PASS','existing_output_refused':True})
    report={'status':'PASS','python':sys.version,'platform':platform.platform(),'module':foliorivet.__file__,'examples':results,'word_review':'deferred adoption check; not an alpha gate','production_pipeline':'not exercised'}
    (args.output/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))

if __name__=='__main__':main()
