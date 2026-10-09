# Reference conversion and validation

The current collection contains **150 source identities: 148 full documents/source witnesses and two metadata-only exceptions**, with 81 passive SVG assets. The original 2026-10-03 verification record is preserved below; the 2026-10-09 additions have their own verification section at the end.

## Original collection scope and result

- 126 retained source identities in the public catalog: 119 captured-HTML conversions, five generated renderings of pinned Bikeshed sources, one JSON source-history witness, and one Unicode URL/hash-only exception
- Full source-based content for 125 entries; the normative Unicode report text is not distributed
- 80 passive SVG assets: 52 extracted from captured HTML and 28 railroad diagrams generated from the pinned CSS Syntax 3 source
- Eight redundant extracted-text copies were removed after whole-document comparison with their retained exact HTML counterparts. No distinct specification edition was removed
- Original source inputs and legal texts remain unchanged. Source scripts/JavaScript were not executed

Source identity counts are separate from rendering counts: generated diagrams, tables, presentation headings, the informative Unicode companion and navigation material are not additional source snapshots.

## Fidelity checks

For the 119 retained public captured-HTML documents, input-fidelity checks cover 386,732 source text nodes, 6,172 headings, 7,803 explicit definitions, 3,005 literal blocks, 8,150 inline-code runs, 8,911 semantic subscript/superscript/variable nodes, 1,031 source tables, 25,530 original cells, and 85,663 source anchor occurrences. These counts exclude the private full Unicode report and the removed extracted-text duplicates. The five generated Bikeshed renderings are checked and counted separately below; source counts are not rendered-output occurrence counts.

All source characters are preserved in the actual GFM AST under whitespace-only prose comparison. Code blocks preserve tabs, escapes, leading text and block boundaries; only the final block newline is excluded from equality checks. Inline HTML code is checked under HTML-collapsible whitespace normalization only, with exact characters/operators preserved; independent review confirmed the remaining multiline formula/grammar differences were source-rendering whitespace, not token loss. Script/variable structure and multiplicity are checked independently. Table row/column/header/span models are retained in the verification record; later table presentation changes are validated against those source models as described below. Full paragraph/list-entry comparisons are also reported; explicit conversion captions/structure labels can interrupt enclosing entries, so those long-entry checks are diagnostic rather than blanket acceptance gates. No Unicode normalization or punctuation folding is used to hide differences.

Pandoc's HTML writer adds text-presentation variation selectors to some arrows; the emitted Markdown AST preserves the exact source arrow. Character checks therefore inspect the actual AST, with rendered HTML checks separately covering code, anchors, scripts/variables and structures.

## Readable representations and explicit limitations

- Source Note, Example, Advisement, Issue and Warning classes become visible labels, including standalone examples and unresolved algorithm-list issues
- Tables use readable Markdown grids, explicit span expansions, and labeled field/case/code layouts. The 808 originally flattened table transcriptions have been replaced; no raw HTML table layouts or structured row/cell dumps remain. Original HTML header-scope and span accessibility semantics are not expressible in GFM; source structural models and explicit relationships are retained as described below
- 80 passive SVG assets are included: 52 extracted from captured HTML and 28 railroad diagrams generated from the pinned CSS Syntax 3 source. Source geometry, internal IDs, text labels, exact-case attributes and required passive railroad CSS are retained. XML/geometry/CSS checks pass; representative generated diagrams were raster-checked, but complete visual or browser-rendering equivalence is not certified
- All 17 MathML expressions have checked portable fenced TeX transcriptions. Independent round trips preserve mathematical tokens, matrix dimensions/ordering and role-preserving script/fraction/root/limit structures. Exact MathML remains in private verification records; visual equivalence is not certified
- Small semantic inline HTML anchors/emphasis/variables/subscripts/superscripts are retained where GFM otherwise corrupts delimiter or notation meaning. They are content semantics, not original website layout
- Escape-sensitive characters and the unstable Greek question-mark/combining-mark cases use numeric entities where needed. Adjacent literal code, code in tables and quoted/URL-valued link labels are explicitly handled
- The five pinned Bikeshed sources have readable generated Markdown renderings with headings, definitions, examples and eight native tables. Their source hashes are unchanged. Generated matter and external compiler-resolved references are disclosed separately below; no official publication or historical rendered-capture status is claimed
- The JSON source-history capture preserves its complete stored text in a fence. The eight extracted-text copies are no longer published because their complete text is already represented by the retained exact HTML editions; no missing HTML structure was invented
- Existing images, object-image fallbacks, videos and interactive figures retain resolved upstream resource links and available source descriptions. External assets are not downloaded or availability-tested. SVG 1.1 Implementation Notes has 14 PNG-only equations whose source images remain essential; their math is not invented from equation-number alt text
- UAX29 revision 47 has a checked private conversion but only the URL/hash/classification/licensing entry is public, due to its report-specific redistribution restriction
- Optional HTML link hover titles are omitted to avoid Pandoc's broken unescaped-quote title serialization; visible citation titles and original hrefs remain. Exact original href/title inventories are retained separately
- Checks establish structural/textual preservation at the stated scope, not standards conformance, mathematical proof, complete visual equivalence, or external link/asset availability

## Size

The original source bytes represented by the 125 full public source entries total 41,572,855 bytes. The metadata-only Unicode entry identifies an excluded 138,801-byte report input; its full text is not part of that total or the public bundle. The eight removed extracted-text inputs totaled 547,383 bytes. These are input byte counts, not current Markdown size or measured model-token counts. Source identity is determined by the original hashes, independently of rendering size.

## Source identity and link policy

See [Source catalog](SOURCE-CATALOG.md). Exact byte copies share one output and are recorded as aliases; distinct specification editions remain separate. The removed flattened-text representations are not counted as additional public source snapshots. Trusted catalog/hash identities determine canonical paths. Fragment-only self-links are local only when source provenance/base and retained target anchors prove that identity. Cross-document links may localize only when an audited exact source identity and an emitted anchor both exist. Undated/editor/latest/bibliographic or absent-fragment targets remain upstream.

The final combined collection is checked for local Markdown/SVG targets and fragments, including attribution and asset links. Source provenance, external bibliography/current-version links, and normative Unicode citations remain upstream. Genuinely absent source fragments are not invented. External network availability is not tested.

## Table readability update

The 1,031 tables in the retained public HTML inputs comprise 223 tables already rendered readably and 808 originally flattened transcriptions. Of those 808, 692 are ordinary tables (two previously published fixes plus 690 new conversions) and 116 need complex layouts. The private Unicode report's 26 tables are excluded. The eight tables generated from pinned Bikeshed sources are a separate rendering count.

The 690 new ordinary conversions cover 17,904 original cells across 73 documents. The full 692 ordinary-table set comprises 544 Field/Definition layouts, 83 grids with column and row labels, 45 column-header tables, 17 labeled grids, and three blank-corner grids. Source column labels are used where possible; synthetic Field/Definition or Column N headings and empty corner presentation headings are non-normative. Source row-header labels remain visible in bold, but native HTML th/scope accessibility semantics cannot be reproduced in GFM.

The 116 complex tables cover 3,438 source cells across 30 documents: 39 span/ragged/hierarchical layouts, 53 semantic field/list/code/case layouts, and 24 color-value matrices. Merged header paths and span relationships are written explicitly; repeated source values express their original applicability without duplicating source anchors. Long lists, code, demonstrations and matrices use labeled sections or cases rather than flattened per-cell dumps. Source rowspan/colspan/th-scope semantics remain in the verification model, not native GFM accessibility structure.

The 24 color tables recover 1,224 exact CSS background-color values that existed only in source styles; they do not claim equivalent swatch appearance or colorimetry. Six live CSS demonstration tables preserve static source HTML/CSS and text, with their demonstration links pointing to authoritative pinned source pages. No runtime or browser-appearance equivalence is claimed. Five sample-display elements follow HTML's incidental-whitespace collapse while retaining their exact source markup; true preformatted/code blocks remain literal. External image resources retain their source URLs and available descriptions but are not availability-tested.

The ordinary and complex layers are composed as 806 disjoint exact table-span replacements against the same published baseline, affecting 75 documents. The two earlier fixes remain intact. Every byte outside those table spans is preserved until the separately recorded representation-note updates. Those note edits preserve every other byte and all attribution/provenance prefixes. One source-backed correctness recovery accompanies the table changes: CSS 2.1 errata Table 2 restores a previously collapsed space in literal code. It is not an editorial rewrite of the source.

Per-table checks compare cell text and code, original links/images, anchors, and the explicit source row/cell/span models. Ordinary table characters are checked in the parsed GFM table structure; complex layouts retain source-to-output mappings for every original cell. Representative offline rendering checks were performed, but GitHub/browser pixel equivalence is not certified. The frozen source eligibility classifier, ordinary table policy, span/block/color generators, exact-span manifests and representation-note patches are required to reproduce the final presentation.

## Pinned Bikeshed renderings

Five exact pinned source files were rendered offline on 2026-10-03 with Bikeshed 7.1.3, then converted to Markdown. The compiler used its bundled support-data manifest dated 2026-09-14T19:19:55.382036+00:00 without updating it (SHA-256 `e70f976a413257fcd3c1406f5b7097ae7d95f9c78de5ed652a332290902e7211`). Original source hashes, commit URLs and attribution prefixes remain unchanged. These are generated renderings of those sources, not official publications or captured historical HTML.

Source-authored metadata, prose, definitions, examples, explicit anchors, property-definition fields, 43 source WPT-reference lists containing 721 paths, and all 28 railroad grammars are retained. Compiler-inserted default property rows and generated current publication/copyright/status/index/TOC boilerplate are omitted. Generated bibliography descriptions and external automatic link targets come from the stated support data and do not establish historical versions of those external documents. Ambiguous automatic references remain visible without guessed destinations. Source text issues are retained rather than silently edited.

The renderings contain eight native Markdown tables: five source-metadata tables, two property-definition tables and the CSS Syntax 3 12-by-13 token grid. Synthetic Field/Definition headings are non-normative; bold source row-header labels do not reproduce native HTML th/scope accessibility semantics. Twenty-eight passive railroad SVGs are compiler-generated from the exact CSS Syntax 3 source, with their original grammar text adjacent and source/hash/compiler/license attribution embedded in each asset. Equivalent HSL colors are serialized as RGB percentages for portable raster rendering; diagram geometry and text remain unchanged.

Source-to-generated checks cover compiler-parsed authored text, explicit anchors and literal preformatted blocks. Generated-to-Markdown checks cover text, headings, definitions, code, table cells and 4,282 retained generated anchor occurrences, with zero broken local targets. A second build/finalization reproduced the outputs byte-for-byte. Representative diagrams 1, 18 and 28 passed raster inspection. GitHub/browser rendering and external asset availability are not certified. The frozen compiler version, dependency lock, support-data hash and preparation/finalization scripts are required to reproduce these renderings; a current live build is not an equivalent input.

## Duplicate cleanup verification

The eight removed text captures match the corresponding exact HTML documents after whitespace-only whole-document text comparison. Their original filenames, converted filenames and SHA-256 values have no references in the source ledger. The 126 retained catalog entries each have a local target; the per-document W3C notice inventory now covers 123 retained W3C source-based references. Original source inputs and the legal texts remain unchanged.

## Reproducibility

The offline converter is convert_collection.py, with semantic_boxes.lua and svg_support.py; run_collection.py consumes the audited source-identity map. The environment uses Pandoc 3.1.11.1, Python 3.12 and lxml. Per-document cleaned/rendered intermediates, exact source hashes, tables, anchors, structure checks and diagnostics are retained in the local work directory. Reproduction requires the same source bytes and identity map, not a live-page substitution. The original stored snapshots and acquired pins are hash-verified.

## Informative Unicode companion and legacy navigation

The bundle also includes a separately licensed Unicode 17 grapheme implementation rule-data companion, with immutable data/source hashes and the complete Unicode License v3. Its source-data excerpt is exact, but its engine-dependent notation and implicit start/end/default behavior are explicitly disclosed; it does not replace or reproduce the normative UAX 29 report. This informative companion is separate from the 126 retained source snapshot entries. A small `push-test.md` navigation guide preserves the earlier Nesting preview address without duplicating the complete conversion.

## Exact-edition supplement — 2026-10-09

Added 22 exact dated HTML editions, one readable rendering of the live Values 4 Bikeshed pin, and one WebKit metadata-only witness. All pre-existing specification bodies and source identities are retained. The unrelated Page 3 2023 discovery was excluded: the relevant Cascade bibliographies select Page 3 2018. The [edition mapping](SOURCE-EDITIONS.md) distinguishes Snapshot baseline, implementation-selected versions and bounded imports. Reference additions do not implement missing grammar or resolve source contradictions.

### Acquisition and source identity

Dated HTML was retrieved directly from the authoritative source URL; response identity, This Version, title, byte count and SHA-256 were checked. These hashes identify the retrieved official dated bytes, not unavailable audit-archive bytes or a claim that every dated publication is historically byte-immutable. Exact source hashes appear in the source catalog and each document. Source scripts were not executed. The WebKit witness remains metadata-only because file-specific redistribution terms could not be established.

### Conversion and fidelity

The earlier repaired converter, semantic-label filter, SVG handling, and ordinary/complex table methods were reused in an isolated new-source workspace with Pandoc 3.1.11.1, Python 3.12.14 and lxml 6.1.1. The 22 HTML inputs contribute 112,293 source text nodes, 1,614 headings, 2,625 definitions, 1,009 literal blocks, 1,002 inline-code runs, 1,930 script/variable nodes, 333 tables, 10,726 cells and 25,209 anchors. Source character checks use the actual GFM AST with whitespace-only prose comparison and exact code checks; coverage checks are not a proof of parser conformance or full rendering equivalence.

Independent ordinary-table checks cover 285 HTML tables and 8,408 ordered source cells. Complex tables retain source-cell mappings, explicit spans/header relationships and literal color values; no flattened row/cell dumps or raw HTML table layouts remain. Four new MathML expressions pass independent token/matrix/script/fraction/root round trips and are bound to the actual emitted TeX. One new Images 4 SVG passes source geometry/attribute/text and six-label checks. The corresponding source and generated-output hashes are retained with the conversion evidence.

Forty-eight ordinary-table link hover-title attributes are omitted while link targets and visible text remain; literal-code protection relocates some empty anchor markers within their original cells. All such source IDs remain once in the same cell, but original intra-cell ID order is not claimed. The new additions contain no malformed GFM autolinks. A separate scan identified 55 inherited malformed autolinks in older documents; the follow-up below records their source-backed repair. GFM does not reproduce native HTML table accessibility semantics. Source image/video links remain external, and neither external availability nor browser/pixel equivalence is certified.

### Additional pinned Bikeshed rendering

The original Values 4 source at `720ea2863696971ea6a6744e0f23acbb3e6936bd` is rendered with the recovered Bikeshed 7.1.3 environment and its unchanged support-data manifest SHA-256 `e70f976a413257fcd3c1406f5b7097ae7d95f9c78de5ed652a332290902e7211`. Source-authored metadata, all 25 WPT reference lists, link-defaults and ignored-specs configuration are retained. Five ambiguous automatic links remain unlinked; four compiler-generated CanIUse compatibility panels are omitted while the source configuration is retained. Generated bibliography and automatic reference resolution are compiler output, not historical publication evidence. The original source hash, not generated HTML, is the catalog identity; the pinned CSSWG license declaration matches the bundled declaration byte-for-byte.

This supplement remains a source-reference maintenance change. The full CSS Snapshot 2026 language target and the separately identified implementation gaps are not narrowed or marked complete by these conversions.

## Inherited autolink repair — 2026-10-09

After the exact-edition supplement, 55 older converted documents received one bounded status-paragraph formatting correction each. GFM had interpreted a visible source URL next to a closing semantic HTML tag as an additional link whose destination incorrectly contained that tag. Numeric entities now shield the visible URL colon and domain dots from autolinking; the original visible URL, trailing punctuation and actual source href remain unchanged.

Each repair is bound to the prior Markdown hash and exact original source hash. Source status prose, literal code, existing href attributes and anchor sequence are verified unchanged; all bytes outside the recorded replacement span are identical. All 55 repaired candidates have no malformed GFM links under the same AST check. The final combined reference collection is checked for local paths/fragments and malformed autolinks; upstream network availability and browser/pixel equivalence remain outside these checks. This corrects formatting only and changes no source edition, normative requirement, implementation or conformance claim.
