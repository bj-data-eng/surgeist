from pathlib import Path
from lxml import html, etree
import json, re, sys
work=Path(sys.argv[1]) if len(sys.argv)>1 else Path('tmp/work/cssom-reference-conversion')
r=json.loads((work/'conversion-record.json').read_text())
base=r.get('base','https://drafts.csswg.org/cssom/')
body=html.fromstring((work/'capture-body.expected.html').read_text())
from table_layouts import prepare_tables
prepare_tables(body,work)
for caption in list(body.xpath('.//table/caption')):
    caption.tag='p'
    table=caption.getparent();table.remove(caption);table.addprevious(caption)
# Preformatted text is literal; formatting links are compiler decorations.
for pre in body.xpath('.//pre'):
    value=pre.text_content()
    anchors=[x.get('id') for x in pre.iter() if x.get('id')]
    for attr in list(pre.attrib): del pre.attrib[attr]
    pre.set('class','source-literal-block')
    for child in list(pre): pre.remove(child)
    pre.text=('surgeist-preserve-leading-newline'+value) if value.startswith('\n') else value
    if pre.xpath('ancestor::table'):
        assert '\n' not in value, 'Multiline literal table cell needs an explicit case layout'
        pre.tag='code'
        for aid in anchors:
            a=etree.Element('span',id=aid); pre.addprevious(a)
    else:
        holder=etree.Element('div')
        for aid in anchors: etree.SubElement(holder,'span',id=aid)
        pre.addprevious(holder)
# Normalize presentation attributes while preserving source-defined IDs.
for n in list(body.iter()):
    if not isinstance(n.tag,str): continue
    classes=n.get('class','').split()
    if n.tag=='var': n.tag='span'; n.set('class','source-var')
    elif n.tag=='dfn': n.tag='strong'
    elif n.tag=='code' and n.xpath('.//a|.//var'):
        n.tag='span';n.set('class','source-linked-code')
    for label in ['example','note','issue','advisement','warning']:
        if label in classes and not n.text_content().lstrip().lower().startswith(label+':'):
            p=etree.Element('p');etree.SubElement(p,'strong').text=label.title()+':'
            n.insert(0,p)
            break
    for attr in list(n.attrib):
        if attr not in {'id','href','src','alt','rowspan','colspan','start'} and not (attr=='class' and n.get(attr) in {'source-var','source-literal-block','source-linked-code'}):
            del n.attrib[attr]
    if n.get('href'):
        from urllib.parse import urljoin, unquote
        value=n.get('href')
        if not value.startswith('#') or unquote(value[1:]) not in set(r['ids']): value=urljoin(base,value)
        n.set('href',value)
    if n.get('src'):
        from urllib.parse import urljoin
        n.set('src',urljoin(base,n.get('src')))
    if n.tag=='summary': n.tag='p'
    elif n.tag=='details': n.tag='div'
    elif n.tag=='nav': n.tag='div'
    elif n.tag=='main': n.tag='div'
# Read source definition lists as explicit term paragraphs and definition
# blocks. This also retains the source's irregular text/siblings between a
# closed dt and its dd, which Pandoc's definition-list reader would omit.
for n in list(body.iter()):
    if n.tag=='dl': n.tag='div'
    elif n.tag=='dd': n.tag='div'
    elif n.tag=='dt':
        n.tag='p'
        strong=etree.Element('strong');strong.text=n.text;n.text=None
        for c in list(n): n.remove(c);strong.append(c)
        n.append(strong)
# Pandoc drops IDs on paragraphs, definition terms, and strong elements. Move
# each ID into a semantic empty span at the same source position before reading.
for n in list(body.iter()):
    aid=n.get('id')
    if not aid or n.tag=='span': continue
    del n.attrib['id']
    marker=etree.Element('span',id=aid)
    if n.tag in {'p','h1','h2','h3','h4','h5','h6','dt','dd','li','div'}:
        marker.tail=n.text;n.text=None;n.insert(0,marker)
    else:
        n.addprevious(marker)
(work/'pandoc-input.html').write_text('<!doctype html><meta charset="utf-8">'+etree.tostring(body,method='html',encoding='unicode'))
print('Prepared exact text/IDs; scripts and styles omitted; pre code links flattened.')
