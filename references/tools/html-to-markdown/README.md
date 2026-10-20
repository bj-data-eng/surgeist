# Captured HTML conversion

These scripts reproduce the three [CSSOM planning captures](../../SOURCE-EDITIONS.md#cssom-planning-sources) and the [CSS Syntax 3 edition](../../SOURCE-EDITIONS.md#css-syntax-3-edition) capture from their exact HTML bytes. They acquire no software or source material and execute no source scripts. Run from the repository root with the same input hashes recorded in the catalog; a newly fetched editor page is not an equivalent input.

The verified environment uses Pandoc 3.1.11.1, Python 3.12.14, lxml 6.1.1, and the bundled Node `marked` GFM renderer; the CSS Syntax 3 capture used Python 3.14.6 with the same Pandoc and lxml. Pandoc was extracted locally from the approved official [arm64 macOS release archive](https://github.com/jgm/pandoc/releases/download/3.1.11.1/pandoc-3.1.11.1-arm64-macOS.zip), SHA-256 `fa38ad91d8f1f09549ae16830ade3a26650b03cb9a29c68b41b55ea7fab0aa2d`. No system installation is required. Dependency installation is outside this workflow.

Place the original input at `WORK/source.html`, where `WORK` is an owned working directory. Use the corresponding source URL, short name and retrieval date, then run:

```sh
python3 references/tools/html-to-markdown/collect_html.py WORK https://drafts.csswg.org/cssom/ cssom-1 2026-10-09
python3 references/tools/html-to-markdown/prepare_pandoc.py WORK
pandoc WORK/pandoc-input.html --from=html --to=gfm --wrap=none --lua-filter=references/tools/html-to-markdown/semantic_cssom.lua -o WORK/pandoc-body.md
python3 references/tools/html-to-markdown/write_candidate.py WORK
MARKED_MODULE=/absolute/path/to/marked/lib/marked.esm.js node references/tools/html-to-markdown/render_markdown.mjs WORK
python3 references/tools/html-to-markdown/verify_conversion.py WORK
```

A dated publication passes its edition label as a fifth collection argument, for example `CRD-css-syntax-3-20261001`, and is named `<short name>--<label>--<hash>.md`; an undated editor page is named by its retrieval date. A capture other than the CSSOM planning captures places `candidate-options.json` in `WORK` with `report_anchor`, `stability_note` and `selection_note`, which replace the CSSOM defaults in the attribution prefix. Inline SVG diagrams are written to `WORK/assets/` during preparation, copied to `references/assets/` by the candidate writer, and checked against their source SVG by verification. Pandoc 3.1.11.1 mangles a backslash before ASCII punctuation, so the Lua filter writes such text as character entities.

The candidate writer writes `references/<recorded filename>`. The date and hash determine its identity. It emits the attribution and planning-selection prefix used for these three captures; it is not a general source-selection or license classifier. For Counter Styles use `https://drafts.csswg.org/css-counter-styles-3/` and `css-counter-styles-3`; for Conditional 5 use `https://drafts.csswg.org/css-conditional-5/` and `css-conditional-5`.

Collection writes the source-body and provenance record; preparation writes the source table grid/cell/span models and cleaned Pandoc input. Spanning tables become readable column-header paths, with spanning values repeated only in columns to which the source cell applies. Literal code and IDs survive conversion; generated hyperlinks inside fenced blocks are omitted and counted. Table captions retain their source position. Exact initial newlines are protected, and semantic inline HTML prevents Markdown from interpreting literal delimiters or inventing email links.

Verification compares parsed GFM with the original source body: IDs, heading levels and text, literal blocks, inline code, variables, resolved hyperlink destinations, local targets, ordinary table cell order, complex header paths and span applicability. Synthetic headers and expanded complex tables are excluded from the separate whole-prose comparison because those tables have their own complete cell checks. This does not certify browser appearance, native table accessibility, external availability, or implementation conformance.

The active task retains original captures and per-document conversion/verification records under `tmp/work/cssom-reference-conversion/`, with extension inputs under `counter-styles/` and `conditional-5/`. These ignored inputs remain local evidence. Reproduction after their removal requires an archive of those exact byte hashes; live URLs alone cannot supply historical capture identity. The reusable scripts are tracked here so reproduction does not depend on discarded temporary code.
