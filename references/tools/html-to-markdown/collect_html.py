from pathlib import Path
from lxml import html, etree
import json,hashlib,sys
work=Path(sys.argv[1]); base=sys.argv[2]; shortname=sys.argv[3]; retrieved=sys.argv[4]
source=work/'source.html'
data=source.read_bytes();root=html.fromstring(data);body=root.find('body')
for node in body.xpath('.//script|.//style'):node.drop_tree()
sha=hashlib.sha256(data).hexdigest()
filename=f'{shortname}--editor-capture-{retrieved.replace('-', '')}--{sha[:12]}.md'
ids=body.xpath('//@id');assert len(ids)==len(set(ids))
pre=[];tables=[];links=0
for p in body.xpath('.//pre'):
    count=len(p.xpath('.//a[@href]'));links+=count
    pre.append({'text':p.text_content(),'ids':p.xpath('.//@id'),'code_links':count})
for t in body.xpath('.//table'):
    rows=t.xpath('./thead/tr|./tbody/tr|./tfoot/tr|./tr')
    cells=t.xpath('.//th|.//td')
    tables.append({'cells':[c.text_content() for c in cells], 'rows':len(rows),'row_columns':[len(row) for row in rows],'rowspans':t.xpath('.//@rowspan'),'colspans':t.xpath('.//@colspan')})
record={'retrieved':retrieved,'filename':filename,'base':base,'html_sha256':sha,'source_bytes':len(data),'ids':ids,'preformatted':pre,'tables':tables,'omitted_code_link_count':links,'title':root.find('head/title').text_content(),'revision':root.xpath('//meta[@name="revision"]/@content'),'source_status':body.xpath('.//*[@id="w3c-state"]')[0].text_content()}
(work/'capture-body.expected.html').write_text(etree.tostring(body,method='html',encoding='unicode'))
(work/'conversion-record.json').write_text(json.dumps(record,ensure_ascii=False,indent=2))
print(json.dumps({k:record[k] for k in ['filename','base','html_sha256','source_bytes','title','source_status']},ensure_ascii=False,indent=2))
print(json.dumps({'tables':[{k:v for k,v in t.items() if k!='cells'} for t in tables],'ids':len(ids),'pres':len(pre),'omitted_code_links':links}))
