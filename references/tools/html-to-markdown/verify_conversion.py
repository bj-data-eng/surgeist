from pathlib import Path
from lxml import html
from collections import Counter
from urllib.parse import unquote, urljoin, urlsplit
import json, re, difflib,sys
import hashlib
work=Path(sys.argv[1]) if len(sys.argv)>1 else Path('tmp/work/cssom-reference-conversion')
r=json.loads((work/'conversion-record.json').read_text())
source=html.fromstring((work/'capture-body.expected.html').read_text())
result=html.fromstring((work/'rendered-body.html').read_text())
whole=html.fromstring((work/'rendered.html').read_text())
norm=lambda x:re.sub(r'\s+',' ',x).strip()
def visible(n):
    # HTML block boundaries and line breaks render as whitespace, independently
    # of whether the DOM contains an explicit inter-element text node.
    v=n.text or ''
    for c in n:
        block=c.tag in {'p','h1','h2','h3','h4','h5','h6','div','main','nav','details','summary','dt','dd','li','ol','ul','dl','table','tr','td','th','pre','br','hr'}
        v+=(' ' if block else '')+visible(c)+(' ' if block else '')+(c.tail or '')
    return v
errors=[]
actual=Counter(result.xpath('//@id'))
if actual!=Counter(r['ids']): errors.append({'ids_missing':list((Counter(r['ids'])-actual).elements()),'ids_extra':list((actual-Counter(r['ids'])).elements())})
source_pres=source.xpath('.//pre')
result_pres=result.xpath('.//pre')
table_codes=source.xpath('.//table//pre')
expected_pres=[p for p in source_pres if p not in table_codes]
if len(expected_pres)!=len(result_pres):errors.append({'pres':(len(expected_pres),len(result_pres))})
for index,(a,b) in enumerate(zip(expected_pres,result_pres)):
    # The Markdown code fence contributes one final separator newline.
    if a.text_content().rstrip('\n')!=b.text_content().rstrip('\n'):
        errors.append({'pre_mismatch':index,'diff':list(difflib.unified_diff(a.text_content().splitlines(),b.text_content().splitlines()))[:12]})
models=json.loads((work/'table-models.json').read_text()) if (work/'table-models.json').exists() else None
source_tables=source.xpath('.//table');result_tables=result.xpath('.//table')
if len(source_tables)!=len(result_tables):errors.append({'table_count_mismatch':(len(source_tables),len(result_tables))})
for index,(a,b) in enumerate(zip(source_tables,result_tables)):
    ac=[norm(c.text_content()) for c in a.xpath('.//th|.//td')]
    bc=[norm(c.text_content()) for c in b.xpath('.//th|.//td')]
    if models:
        model=models[index]
        ac=model['expected_cells']
        output_rows=b.xpath('./thead/tr|./tbody/tr|./tfoot/tr|./tr')
        if len(output_rows)!=model['output_rows'] or any(len(row)!=model['source_columns'] for row in output_rows):errors.append({'table_layout_dimensions':index})
        source_cells=a.xpath('.//th|.//td')
        if [norm(c.text_content()) for c in source_cells]!=[norm(c['text']) for c in model['source_cells'].values()]:errors.append({'source_cell_model_mismatch':index})
        if model['complex']:
            # Each original cell's applicability is explicit in the grid; check
            # its span rectangle, all header paths, and every output body cell.
            grid=model['source_grid'];headers=model['source_header_rows']
            for rid,cell in model['source_cells'].items():
                coords=[(rr,cc) for rr,row in enumerate(grid) for cc,value in enumerate(row) if value==int(rid)]
                expected=[(rr,cc) for rr in range(cell['row'],cell['row']+cell['rowspan']) for cc in range(cell['column'],cell['column']+cell['colspan'])]
                if coords!=expected:errors.append({'invalid_source_span':(index,rid)})
            for cc,cell in enumerate(output_rows[0]):
                path=[]
                for rr in range(headers):
                    rid=grid[rr][cc]
                    if rid is not None and rid not in path:path.append(rid)
                text=' / '.join(norm(source_cells[rid].text_content()) for rid in path)
                if norm(cell.text_content())!=text:errors.append({'header_path_mismatch':(index,cc)})
            for rr,row in enumerate(output_rows[1:],headers):
                for cc,cell in enumerate(row):
                    rid=grid[rr][cc]
                    text='' if rid is None else norm(source_cells[rid].text_content())
                    if norm(cell.text_content())!=text:errors.append({'span_applicability_mismatch':(index,rr,cc)})
    if ac!=bc:errors.append({'table_mismatch':index,'source':ac,'result':bc})
source_headings=[(h.tag,norm(h.text_content())) for h in source.xpath('.//h1|.//h2|.//h3|.//h4|.//h5|.//h6')]
result_headings=[(h.tag,norm(h.text_content())) for h in result.xpath('.//h1|.//h2|.//h3|.//h4|.//h5|.//h6')]
if source_headings!=result_headings:errors.append({'heading_mismatch':(source_headings,result_headings)})
source_inline=source.xpath('.//code[not(ancestor::pre)]|.//table//pre')
result_inline=result.xpath('.//code[not(ancestor::pre)]')
if [norm(n.text_content()) for n in source_inline]!=[norm(n.text_content()) for n in result_inline]:errors.append({'inline_code_mismatch':True,'counts':(len(source_inline),len(result_inline))})
source_vars=source.xpath('.//var[not(ancestor::pre)]')
result_vars=result.xpath('.//var[not(ancestor::pre)]')
if [norm(n.text_content()) for n in source_vars]!=[norm(n.text_content()) for n in result_vars]:errors.append({'variable_mismatch':True,'counts':(len(source_vars),len(result_vars))})
source_images=[(urljoin(r['base'],n.get('src','')),n.get('alt','')) for n in source.xpath('.//img')]
result_images=[(n.get('src',''),n.get('alt','')) for n in result.xpath('.//img')]
if source_images!=result_images:errors.append({'image_source_or_description_mismatch':True})
for a in whole.xpath('.//a[@href]'):
    href=a.get('href')
    if href.startswith('#'):
        if unquote(href[1:]) not in whole.xpath('//@id'): errors.append({'missing_fragment':href})
    elif not urlsplit(href).scheme:
        path,sep,fragment=href.partition('#')
        target=Path('references',unquote(path))
        if not target.exists():errors.append({'missing_path':str(target)})
        elif sep:
            target_text=target.read_text()
            heading_ids=[]
            for heading in re.findall(r'^#+\s+(.+)$',target_text,flags=re.M):
                heading=re.sub(r'<[^>]+>','',heading).lower()
                heading=re.sub(r'[^\w\- ]','',heading).replace(' ','-')
                heading_ids.append(heading)
            if not re.search(r'id=["\']'+re.escape(unquote(fragment))+r'["\']',target_text) and unquote(fragment) not in heading_ids:errors.append({'missing_external_local_fragment':href})
    if re.search(r'</?(?:a|em|code|var|strong)',href):errors.append({'malformed_href':href})
# Strip added visible semantic box labels only; source body remains in order.
for strong in result.xpath('.//strong'):
    if strong.text_content() in {'Example:','Note:','Issue:','Advisement:','Warning:'}:
        strong.drop_tree()
# Table-cell ordering/spans are verified separately above. Remove expanded
# complex tables and synthetic headers for the rest-of-document prose check.
if models:
    for model,a,b in zip(models,source_tables,result_tables):
        if model['complex']:
            a.drop_tree();b.drop_tree()
        elif model['synthetic_header']:
            b.find('thead').drop_tree()
sv=norm(visible(source))
rv=norm(visible(result))
if sv!=rv:
    matcher=difflib.SequenceMatcher(None,sv,rv,autojunk=True)
    issues=[]
    for tag,i,j,k,l in matcher.get_opcodes():
        if tag!='equal':issues.append({'kind':tag,'source':sv[max(0,i-35):min(len(sv),j+35)],'result':rv[max(0,k-35):min(len(rv),l+35)]})
        if len(issues)>25:break
    errors.append({'whole_text_differences':issues,'source_text_length':len(sv),'result_text_length':len(rv)})
# Original hyperlinks outside preformatted blocks retain their resolved destinations.
canonical=lambda u:unquote(urljoin(r.get('base','https://drafts.csswg.org/cssom/'),u))
sl=Counter(canonical(a.get('href')) for a in source.xpath('.//a[@href]') if not a.xpath('ancestor::pre'))
rl=Counter(canonical(a.get('href')) for a in result.xpath('.//a[@href]'))
if sl!=rl:errors.append({'links_missing':list((sl-rl).elements())[:30],'links_extra':list((rl-sl).elements())[:30],'counts':(sum(sl.values()),sum(rl.values()))})
report={'errors':errors,'body_ids':len(r['ids']),'literal_blocks':len(expected_pres),'table_literal_cells':len(table_codes),'tables':len(r['tables']),'table_cells':sum(len(t['cells']) for t in r['tables']),'headings':len(source_headings),'inline_code':len(source_inline),'variables':len(source_vars),'source_visible_text_characters':len(sv),'retained_links':sum(sl.values()),'html_sha256':r['html_sha256']}
report['table_structures']=[{'rows':len(t.xpath('.//tr')),'columns':len(t.xpath('.//tr')[0]),'header_cells':len(t.xpath('.//th')),'data_cells':len(t.xpath('.//td')),'rowspans':t.xpath('.//@rowspan'),'colspans':t.xpath('.//@colspan')} for t in source_tables]
(work/'verification.json').write_text(json.dumps(report,ensure_ascii=False,indent=2))
print(json.dumps({'error_count':len(errors),'errors':[{k:(v[:5] if isinstance(v,list) else v) for k,v in e.items()} for e in errors[:12]], 'body_ids':len(r['ids']), 'literal_blocks':len(expected_pres),'tables':len(r['tables'])},ensure_ascii=False,indent=2))
raise SystemExit(bool(errors))
