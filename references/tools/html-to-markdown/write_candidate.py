from pathlib import Path
import json,sys

for work in [Path(value) for value in sys.argv[1:]]:
    r=json.loads((work/'conversion-record.json').read_text())
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

License: [{license2023}](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and exact self-fragment links as detailed in the [conversion report](CONVERSION-REPORT.md#cssom-planning-reference-refresh--2026-10-09).

# Source provenance

Title: {title}

Source snapshot: {r['base']}

Retrieved: {r['retrieved']}. The captured page states {status}. An undated editor URL can change; the retrieval date and exact byte hash identify this captured HTML.

Captured HTML SHA-256: {r['html_sha256']}

Captured HTML revision metadata: `{r['revision'][0]}`. This is the page’s declared revision; the byte hash identifies the captured rendering.

Representation notes:

- Full format conversion of the captured HTML body, including status, metadata, bibliography, indexes, examples, test references, and legal notice. Script and style elements are omitted and were not executed.
- All source body IDs are retained, including those inside preformatted blocks, which are relocated immediately before those blocks. Fragment-only links stay local only when their exact captured target exists; other links remain upstream.
- All {count} tables have readable Markdown layouts. {table_note} Synthetic Field/Definition or Column N headings are non-normative. Source row headers remain bold; native HTML header/accessibility semantics are not expressible in GFM.
- Preformatted examples retain literal text. Single-line table examples use semantic inline code. Compiler-generated hyperlinks inside fenced code blocks are omitted while their IDs and visible code text are retained; other source links are preserved. Small semantic code, variable, emphasis, subscript, and superscript HTML remains where Markdown notation would alter text.
- Conversion uses Pandoc 3.1.11.1 with focused Python/lxml preparation and a semantic Lua filter; the parsed GFM output is checked against the captured HTML. The [conversion report](CONVERSION-REPORT.md#cssom-planning-reference-refresh--2026-10-09) records provenance and check boundaries.
- This full edition is the selected planning reference for the new CSSOM work. The [source-edition mapping](SOURCE-EDITIONS.md#cssom-planning-sources) distinguishes these planning selections from historical editions consumed by completed CSS work.

---

'''
    target.write_text(prefix+marker+'\n\n'+body)
    print(target)
