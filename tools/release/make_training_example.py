"""Build an original minimal OOXML training fixture; no imported template/assets."""
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED
from xml.sax.saxutils import escape
import json

base = Path(__file__).resolve().parents[2] / 'bindings/python/examples/training-evaluation'

def text(value):
    return '<w:r><w:t xml:space="preserve">'+escape(value)+'</w:t></w:r>'

def scalar(tag, ident):
    return f'<w:sdt><w:sdtPr><w:id w:val="{ident}"/><w:tag w:val="value:{tag}"/><w:text/></w:sdtPr><w:sdtContent>'+text('Template value')+'</w:sdtContent></w:sdt>'

def para(content, style='Normal'):
    return f'<w:p><w:pPr><w:pStyle w:val="{style}"/></w:pPr>{content}</w:p>'

def block(tag, ident, content):
    return f'<w:sdt><w:sdtPr><w:id w:val="{ident}"/><w:tag w:val="{tag}"/></w:sdtPr><w:sdtContent>{content}</w:sdtContent></w:sdt>'

body = para(text('Training and evaluation summary'), 'Title')
body += para(text('Synthetic results demonstrate editable native template output. All metrics are supplied by Python.'))
body += para(text('Model: ')+scalar('model',1))
body += para(text('Training results'),'Heading1')
body += block('repeat:classes',2,para(scalar('item.name',3),'Heading2')+para(text('Precision: ')+scalar('item.precision',4))+para(text('Reviewer notes: ')))
body += block('if:has_evaluation',5,para(text('Evaluation'),'Heading1')+para(scalar('evaluation',6)))
body += '<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720"/></w:sectPr>'
ns='http://schemas.openxmlformats.org/wordprocessingml/2006/main'
parts={
 '[Content_Types].xml':'<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/></Types>',
 '_rels/.rels':'<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>',
 'word/document.xml':f'<w:document xmlns:w="{ns}"><w:body>{body}</w:body></w:document>',
 'word/_rels/document.xml.rels':'<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>',
 'word/styles.xml':f'<w:styles xmlns:w="{ns}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Liberation Sans" w:hAnsi="Liberation Sans"/><w:sz w:val="22"/><w:color w:val="000000"/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:pPr><w:spacing w:after="160"/></w:pPr></w:style><w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:rPr><w:b/><w:sz w:val="36"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:pPr><w:keepNext/></w:pPr><w:rPr><w:b/><w:sz w:val="28"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:pPr><w:keepNext/></w:pPr><w:rPr><w:b/></w:rPr></w:style></w:styles>'}
base.mkdir(parents=True,exist_ok=True)
with ZipFile(base/'template.docx','x',ZIP_DEFLATED) as z:
    for name,value in parts.items():z.writestr(name,'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'+value)
(base/'input.json').write_text(json.dumps({'model':'Synthetic classifier 01','classes':[{'name':'Class A','precision':'94.0%'},{'name':'Class B','precision':'91.5%'}],'has_evaluation':True,'evaluation':'Held-out accuracy: 92.8%. Illustrative data only.'},indent=2)+'\n',encoding='utf-8')
