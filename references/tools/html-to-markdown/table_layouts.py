from lxml import etree
from copy import deepcopy
import json,re

def normalize(value):return re.sub(r'\s+',' ',value).strip()

def prepare_tables(body,work):
    models=[]
    for index,table in enumerate(body.xpath('.//table')):
        rows=table.xpath('./thead/tr|./tbody/tr|./tfoot/tr|./tr')
        cells=[];grid=[];places={}
        for row_index,row in enumerate(rows):
            while len(grid)<=row_index:grid.append([])
            column=0
            for cell in row:
                while column<len(grid[row_index]) and grid[row_index][column] is not None:column+=1
                rid=len(cells);cells.append(cell)
                rs=int(cell.get('rowspan','1'));cs=int(cell.get('colspan','1'))
                places[rid]={'row':row_index,'column':column,'rowspan':rs,'colspan':cs,'tag':cell.tag,'text':cell.text_content()}
                for rr in range(row_index,row_index+rs):
                    while len(grid)<=rr:grid.append([])
                    while len(grid[rr])<column+cs:grid[rr].append(None)
                    for cc in range(column,column+cs):
                        assert grid[rr][cc] is None
                        grid[rr][cc]=rid
                column+=cs
        width=max(len(row) for row in grid)
        for row in grid:row.extend([None]*(width-len(row)))
        complex=any(c.get('rowspan') or c.get('colspan') for c in cells)
        header_rows=len(table.xpath('./thead/tr'))
        model={'table':index,'complex':complex,'source_rows':len(rows),'source_columns':width,'source_cells':places,'source_grid':grid,'source_header_rows':header_rows,'synthetic_header':not header_rows,'expected_cells':[]}
        if complex:
            assert header_rows>0
            assert not table.xpath('.//a[@href]|.//code|.//var|.//*[@id]'), 'Complex content needs a separate semantic layout'
            seen=set()
            def clone(rid,tag):
                if rid is None:return etree.Element(tag)
                old=cells[rid];new=deepcopy(old);new.tag=tag
                new.attrib.pop('rowspan',None);new.attrib.pop('colspan',None)
                if rid in seen:
                    for node in new.iter():node.attrib.pop('id',None)
                seen.add(rid)
                return new
            new=etree.Element('table');new.attrib.update(table.attrib)
            head=etree.SubElement(new,'thead');tr=etree.SubElement(head,'tr')
            for column in range(width):
                path=[]
                for row in range(header_rows):
                    rid=grid[row][column]
                    if rid is not None and rid not in path:path.append(rid)
                th=etree.SubElement(tr,'th')
                for offset,rid in enumerate(path):
                    part=clone(rid,'span');part.tail=None
                    if offset:
                        if len(th):th[-1].tail=' / '
                        else:th.text=' / '
                    th.append(part)
                model['expected_cells'].append(normalize(''.join(th.itertext())))
            tbody=etree.SubElement(new,'tbody')
            for row_index in range(header_rows,len(rows)):
                tr=etree.SubElement(tbody,'tr')
                for column in range(width):
                    rid=grid[row_index][column]
                    cell=clone(rid,'td')
                    if rid is not None and cells[rid].tag=='th':
                        strong=etree.Element('strong');strong.text=cell.text;cell.text=None
                        for child in list(cell):cell.remove(child);strong.append(child)
                        cell.append(strong)
                    tr.append(cell)
                    model['expected_cells'].append(normalize(''.join(cell.itertext())))
            assert set(places)==seen
            new.tail=table.tail;table.getparent().replace(table,new)
            model['output_rows']=1+len(rows)-header_rows
        else:
            if not header_rows:
                head=etree.Element('thead');tr=etree.SubElement(head,'tr')
                labels=['Field','Definition'] if width==2 else [f'Column {i+1}' for i in range(width)]
                for label in labels:etree.SubElement(tr,'th').text=label
                table.insert(0,head)
                model['expected_cells'].extend(labels)
            model['expected_cells'].extend(normalize(c.text_content()) for c in cells)
            for row in rows:
                if row.getparent().tag=='thead':continue
                for cell in row:
                    if cell.tag=='th':
                        strong=etree.Element('strong');strong.text=cell.text;cell.text=None
                        for child in list(cell):cell.remove(child);strong.append(child)
                        cell.append(strong)
            model['output_rows']=len(rows)+(not header_rows)
        models.append(model)
    (work/'table-models.json').write_text(json.dumps(models,ensure_ascii=False,indent=2))
    return models
