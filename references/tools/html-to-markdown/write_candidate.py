from pathlib import Path
import json,shutil,sys

# Defaults reproduce the CSSOM planning captures. Another capture supplies
# `candidate-options.json` in its work directory to state its own selection,
# report section and source stability.
DEFAULTS={
    'report_anchor':'cssom-planning-reference-refresh--2026-10-09',
    'stability_note':'An undated editor URL can change; the retrieval date and exact byte hash identify this captured HTML.',
    'selection_note':'This full edition is the selected planning reference for the new CSSOM work. The [source-edition mapping](SOURCE-EDITIONS.md#cssom-planning-sources) distinguishes these planning selections from historical editions consumed by completed CSS work.',
}

for work in [Path(value) for value in sys.argv[1:]]:
    r=json.loads((work/'conversion-record.json').read_text())
    options=work/'candidate-options.json'
    o={**DEFAULTS,**(json.loads(options.read_text()) if options.exists() else {})}
    svgs=json.loads((work/'svg-assets.json').read_text()) if (work/'svg-assets.json').exists() else []
    for name in svgs: shutil.copyfile(work/'assets'/name,Path('references','assets',name))
    svg_note=(f'- {len(svgs)} inline SVG diagrams are retained as local passive SVG assets with their source geometry, exact-case names and visible text; railroad diagrams carry the capture\u2019s railroad stylesheet with light-theme values resolved. Each image is followed by its diagram text. Supporting assets are not reference documents.\n' if svgs else '')
    target=Path('references',r['filename'])
    marker='<!-- captured-body-start -->'
    body=(work/'pandoc-body.md').read_text()
    count=len(r['tables'])
    models=json.loads((work/'table-models.json').read_text())
    complex_count=sum(m['complex'] for m in models)
    license2023='W3C Software and Document License, 2023 version'
    status=' '.join(r['source_status'].split())
    table_note=(f'{complex_count} tables use explicit merged header paths and repeat spanning values in their applicable columns.' if complex_count else 'All tables have ordinary row/column layouts.')
    title=r['title']
    prefix=f'''Attribution and reformatting notice added for Surgeist on {r['retrieved']}

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [{title}]({r['base']}).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply. The capture’s full original notice and links remain below.

License: [{license2023}](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and exact self-fragment links as detailed in the [conversion report](CONVERSION-REPORT.md#{o['report_anchor']}).

# Source provenance

Title: {title}

Source snapshot: {r['base']}

Retrieved: {r['retrieved']}. The captured page states {status}. {o['stability_note']}

Captured HTML SHA-256: {r['html_sha256']}

Captured HTML revision metadata: `{r['revision'][0]}`. This is the page’s declared revision; the byte hash identifies the captured rendering.

Representation notes:

- Full format conversion of the captured HTML body, including status, metadata, bibliography, indexes, examples, test references, and legal notice. Script and style elements are omitted and were not executed.
- All source body IDs are retained, including those inside preformatted blocks, which are relocated immediately before those blocks. Fragment-only links stay local only when their exact captured target exists; other links remain upstream.
- All {count} tables have readable Markdown layouts. {table_note} Synthetic Field/Definition or Column N headings are non-normative. Source row headers remain bold; native HTML header/accessibility semantics are not expressible in GFM.
- Preformatted examples retain literal text. Single-line table examples use semantic inline code. Compiler-generated hyperlinks inside fenced code blocks are omitted while their IDs and visible code text are retained; other source links are preserved. Small semantic code, variable, emphasis, subscript, and superscript HTML remains where Markdown notation would alter text.
{svg_note}- Conversion uses Pandoc 3.1.11.1 with focused Python/lxml preparation and a semantic Lua filter; the parsed GFM output is checked against the captured HTML. The [conversion report](CONVERSION-REPORT.md#{o['report_anchor']}) records provenance and check boundaries.
- {o['selection_note']}

---

'''
    target.write_text(prefix+marker+'\n\n'+body)
    print(target)
