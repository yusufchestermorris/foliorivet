"""Check alpha feedback through an installed wheel and matching CLI; synthetic DOCX only."""
import argparse
import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path
from xml.etree import ElementTree as ET
from xml.sax.saxutils import escape
from zipfile import ZipFile

import foliorivet as fr

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"


def scalar(tag, bold=False, multiline=True):
    kind = '' if multiline else '<w:text/>'
    return f'<w:sdt><w:sdtPr><w:tag w:val="{escape(tag)}"/>{kind}</w:sdtPr><w:sdtContent><w:r><w:rPr>{"<w:b/>" if bold else "<w:i/>"}</w:rPr><w:t>placeholder</w:t></w:r></w:sdtContent></w:sdt>'


def paragraph(content):
    return f'<w:p>{content}</w:p>'


def block(tag, content):
    return f'<w:sdt><w:sdtPr><w:tag w:val="{tag}"/></w:sdtPr><w:sdtContent>{content}</w:sdtContent></w:sdt>'


def package(path, body):
    types = '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>'
    rels = '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>'
    with ZipFile(path, 'w') as out:
        out.writestr('[Content_Types].xml', types)
        out.writestr('_rels/.rels', rels)
        out.writestr('word/document.xml', f'<w:document xmlns:w="{W}"><w:body>{body}<w:sectPr/></w:body></w:document>')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--cli', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    output = args.output.resolve()
    cases = []

    def rejected(call):
        try:
            call()
        except fr.TemplateError:
            return
        raise AssertionError('Expected TemplateError')

    def cli(template, command, params, name):
        job = output / (name + '.json')
        job.write_text(json.dumps({'commands': [{'command': command, 'params': params}]}), encoding='utf-8')
        result = subprocess.run([str(args.cli.resolve()), 'run', '--file', str(template), '--script', str(job), '--print'], text=True, capture_output=True)
        (output / (name + '.log')).write_text(result.stdout + result.stderr, encoding='utf-8')
        return result

    template = output / 'fanout.docx'
    package(template, paragraph(scalar('value:name', True) + scalar('value:name')) + paragraph(scalar('value:name')))
    original = hashlib.sha256(template.read_bytes()).hexdigest()
    rejected(lambda: fr.inspect_template(template))
    targets = fr.inspect_template(template, scalar_fanout=True)['targets']
    assert len(targets) == 3 and all('story' in t and 'path' in t and 'scope' in t for t in targets)
    assert fr.preflight_template(template, {'name': 'Alpha'}, scalar_fanout=True)['scalar_count'] == 3
    native = output / 'fanout-python.docx'
    fr.render_template(template, {'name': 'Alpha'}, '', native, scalar_fanout=True)
    rejected(lambda: fr.render_template(template, {'name': 'Alpha'}, '', native, scalar_fanout=True))
    cli_output = output / 'fanout-cli.docx'
    result = cli(template, 'template.render', {'template_path': str(template), 'context': {'name': 'Alpha'}, 'assets_dir': '', 'output_path': str(cli_output), 'scalar_fanout': True}, 'fanout-render')
    assert result.returncode == 0, result.stderr
    with ZipFile(native) as a, ZipFile(cli_output) as b:
        assert set(a.namelist()) == set(b.namelist())
        assert all(a.read(n) == b.read(n) for n in a.namelist())
        root = ET.fromstring(a.read('word/document.xml'))
        assert [n.text for n in root.iter(f'{{{W}}}t')] == ['Alpha'] * 3
        controls = list(root.iter(f'{{{W}}}sdt'))
        assert len(controls) == 3
        assert controls[0].find(f'.//{{{W}}}b') is not None
        assert controls[1].find(f'.//{{{W}}}i') is not None
    assert hashlib.sha256(template.read_bytes()).hexdigest() == original
    cases.append('fanout, formatting, retained controls, strict default, no overwrite, native/CLI package equality')

    invalid = output / 'invalid-last.docx'
    package(invalid, paragraph(scalar('value:name') + scalar('value:name', multiline=False)))
    failed_output = output / 'must-not-exist.docx'
    rejected(lambda: fr.render_template(invalid, {'name': 'one\ntwo'}, '', failed_output, scalar_fanout=True))
    assert not failed_output.exists()
    cases.append('invalid-last multiline destination is atomic')

    nested = output / 'nested.docx'
    package(nested, block('if:show', paragraph('<w:r><w:t>Heading</w:t></w:r>') + block('repeat:items', paragraph(scalar('value:item.name') + scalar('value:item.name')))))
    hidden = fr.analyze_context(nested, {'show': False}, scalar_fanout=True)
    assert hidden['missing_paths'] == [] and 'items' in hidden['inactive_paths']
    unknown = fr.analyze_context(nested, {'items': [{'name': 'A'}]}, scalar_fanout=True)
    assert unknown['missing_paths'] == ['show'] and unknown['inactive_paths'] == []
    invalid_condition = fr.analyze_context(nested, {'show': 'false'}, scalar_fanout=True)
    assert invalid_condition['diagnostics'] and invalid_condition['inactive_paths'] == []
    context = {'show': True, 'items': [{'name': 'A', 'extra': 1}, {'name': 'B'}]}
    active = fr.analyze_context(nested, context, scalar_fanout=True)
    assert active['unused_supplied_paths'] == ['items[0].extra']
    assert 'items[1].name' in active['active_required_paths']
    result = cli(nested, 'template.analyzeContext', {'context': context, 'scalar_fanout': True}, 'analysis')
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout.strip().split(': ', 1)[1]) == active
    rejected(lambda: fr.preflight_template(nested, {'show': False, 'items': []}, scalar_fanout=True))
    for count in (0, 1, 3):
        context = {'show': True, 'items': [{'name': str(i)} for i in range(count)]}
        fr.render_template(nested, context, '', output / f'nested-{count}.docx', scalar_fanout=True)
    fr.render_template(nested, {'show': False}, '', output / 'hidden.docx', scalar_fanout=True)
    rejected(lambda: fr.analyze_context(nested, '{', scalar_fanout=True))
    cases.append('conditional/repeat zero-one-many, inactive data, undetermined conditions, indexed analysis and CLI')

    conditional = output / 'conditional-consumers.docx'
    package(conditional, block('if:first', paragraph(scalar('value:name'))) + block('if:second', paragraph(scalar('value:name'))))
    context = {'first': True, 'second': True, 'name': 'Example'}
    ambiguous = fr.analyze_context(conditional, context)
    assert ambiguous['diagnostics'] and ambiguous['diagnostics'][0]['target'] == 'name'
    rejected(lambda: fr.preflight_template(conditional, context))
    assert fr.analyze_context(conditional, context, scalar_fanout=True)['diagnostics'] == []
    fr.preflight_template(conditional, context, scalar_fanout=True)
    context['second'] = False
    assert fr.analyze_context(conditional, context)['diagnostics'] == []
    fr.preflight_template(conditional, context)
    context['second'] = True
    result = cli(conditional, 'template.analyzeContext', {'context': context}, 'conditional-consumers')
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout.strip().split(': ', 1)[1]) == ambiguous

    invalid_context = output / 'invalid-context.docx'
    package(invalid_context, paragraph(scalar('value:name')))
    context = {'!invalid': 1, 'name': 'Example', 'z_unused': 2}
    findings = fr.analyze_context(invalid_context, context)
    assert findings['unused_supplied_paths'] == ['!invalid', 'z_unused']
    assert findings['diagnostics'][0]['target'] == '!invalid'
    result = cli(invalid_context, 'template.analyzeContext', {'context': context}, 'invalid-context')
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout.strip().split(': ', 1)[1]) == findings

    consumed = output / 'consumed-condition.docx'
    package(consumed, paragraph(scalar('value:show')) + block('if:show', paragraph('<w:r><w:t>Details</w:t></w:r>')))
    for fanout in (False, True):
        assert fr.analyze_context(consumed, {'show': True}, scalar_fanout=fanout)['diagnostics'][0]['target'] == 'show'
        rejected(lambda: fr.preflight_template(consumed, {'show': True}, scalar_fanout=fanout))
    cases.append('analysis/preflight parity for conditional consumers; invalid keys preserve later unused fields')

    report = {'status': 'PASS', 'python': sys.version, 'platform': platform.platform(), 'module': fr.__file__, 'cases': cases,
              'word_qualification': 'PENDING; structural checks do not establish Word compatibility'}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
