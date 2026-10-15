# Reference conversion and validation

The current collection contains **372 source identities: 260 full documents/source witnesses, 98 partial witnesses and 14 metadata-only records**, with 81 passive SVG assets. The original 2026-10-03 verification record and subsequent dated addition records are preserved below.

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
- Tables use readable Markdown grids, explicit span expansions, and labeled field/case/code layouts. The 808 originally flattened table transcriptions have been replaced; no raw HTML table layouts or structured row/cell dumps remain in that original collection. The later DOM captures preserve complex semantic HTML tables as disclosed below. Original HTML header-scope and span accessibility semantics are not expressible in GFM; source structural models and explicit relationships are retained as described below
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

## CSSOM planning reference refresh — 2026-10-09

Three complete captured HTML documents supply the operative CSSOM planning editions. Their source identities and dates appear in the [edition mapping](SOURCE-EDITIONS.md#cssom-planning-sources) and [catalog](SOURCE-CATALOG.md). CSSOM is 1,057,190 bytes; Counter Styles 3 is 638,504 bytes; Conditional 5 is 494,912 bytes. Each undated URL capture is identified by its complete byte hash and page-declared revision. Older editions remain for completed CSS consumers. No correction excerpt is layered over the planning editions.

Conversion uses the approved official Pandoc 3.1.11.1 arm64 macOS archive, extracted task-locally without a system installation. Archive SHA-256 is `fa38ad91d8f1f09549ae16830ade3a26650b03cb9a29c68b41b55ea7fab0aa2d`. Python/lxml preparation preserves source IDs, irregular definition-list text, literal code, and table relationships; a semantic Lua filter retains code, variable, emphasis, subscript and superscript content. Script/style elements are omitted and were not executed.

CSSOM’s 157,628 visible body characters pass whole-document comparison under whitespace-only prose normalization with HTML block boundaries treated as whitespace. All 78 heading levels/texts, 1,939 body IDs, 568 ordered inline-code/example runs, 649 semantic variables, 41 literal fenced blocks, and four literal table examples pass source-to-parsed-GFM checks. Fenced text is compared exactly except for the Markdown fence’s final separator newline. All 2,212 source hyperlinks outside preformatted blocks retain equivalent resolved destinations. The 595 generated code-block hyperlink decorations are omitted while every visible literal character and source ID remains.

All three CSSOM tables carry a source `complex data` class but have ordinary two-column structures: 3, 14, and 12 rows, each with one two-cell header row and no rowspans or colspans. All 58 source cells pass ordered text and literal-code comparisons in the parsed GFM structure. Source headers, row/column relationships, cell links, and examples remain intact. Native HTML header/accessibility semantics and browser appearance are not certified.

Counter Styles contributes 68 heading levels/texts, 845 IDs, 35 literal blocks, 32 inline-code runs and 86 variables. All 17 tables and 575 source cells pass comparison. Twelve tables have ordinary layouts; five have multi-row headers and spanning cells. Their source grids preserve every cell’s row/column position and span rectangle. Output checks cover each combined header path and every repeated body value’s original applicability. Repetition expresses spans rather than adding new source values. Links and source IDs are not duplicated by the complex layouts.

Conditional 5 contributes 64 heading levels/texts, 1,249 IDs, 37 literal blocks, 59 inline-code runs and 30 variables. All 16 tables and 228 cells are ordinary and pass ordered cell checks with synthetic headings removed from the prose comparison. Its container-unit table caption remains before its rows. Counter Styles’ initial blank lines in three examples are retained exactly; one unlinked email address is shielded from GFM autolinking while its visible text remains unchanged.

Every document passes whole-prose comparison under whitespace-only normalization, with table expansions checked separately. The comparison covers all CSSOM and Conditional 5 text and all Counter Styles text outside its five expanded tables; those five tables pass complete source-cell/header/span mappings. All 1,591 Counter Styles and 1,107 Conditional 5 hyperlinks outside preformatted blocks retain their resolved destinations. The 79 and 160 generated hyperlink decorations inside their fenced examples are omitted and disclosed. No prose or inline-code links are omitted.

The [tracked preparation, filter and verification scripts](tools/html-to-markdown/README.md) reproduce these captures from the exact retained input bytes. Python 3.12.14 and lxml 6.1.1 supply the parser; the bundled Node `marked` renderer supplies parsed GFM output. The active task retains raw HTML, source models and verification records locally. All source self-fragments and local attribution/provenance links are checked against actual targets; upstream availability and edition equivalence are outside the check boundary. Native table accessibility, browser/pixel equivalence and implementation conformance are not certified.

## WebKit container-query source retention — 2026-10-09

The [implementation witness](webkit-cssom-container-query--73aa6c89e2cb.md) retains four complete files obtained using `git show` from the existing WebKit checkout at `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Each file has its exact source path, byte hash, complete Apple copyright, two-clause redistribution conditions and disclaimer. No checkout mutation or software acquisition is involved. Added headings and an evidence map do not change the source. Parsed Markdown code-block text is verified byte-for-byte against the immutable file bytes; links and local anchors are checked. This supports only the stated container-query source boundaries and does not establish whole-engine behavior or normative standards consensus.

## Core CSSOM browser source retention — 2026-10-09

The [WebKit core witness](webkit-cssom-core-rules--73aa6c89e2cb.md) retains five complete source files from the existing checkout at `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. The [Blink Page/margin witness](blink-cssom-page-rules--7984f9d11800.md) retains six complete files from official immutable Gitiles TEXT responses at `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Base64 transport is decoded before hashing; no source code is executed. Per-file paths/hashes and exact relied-on line boundaries are retained with each readable source block. Complete source files retain every original copyright and file-specific license header; applicable full legal texts accompany them. Project BSD terms are not substituted for LGPL files. These are bounded implementation sources, not normative editions or whole-engine behavior claims. Parsed GFM source blocks are checked byte-for-byte, including final blank lines, and all local legal/section links are checked. No negative HTTP or font search bundle is retained.

The final collection check covers all 167 Markdown files in the reference tree and root/project notices. All 64,996 local link occurrences resolve to existing paths and applicable fragments, including SVG targets; no malformed HTML-tag autolinks remain. The catalog contains exactly 168 source identities: 166 full representations and two metadata-only exceptions. Markdown validation and `git diff --check` pass. Original captures, browser source bytes and the approved task-local Pandoc archive remain retained; no source evidence was removed during final verification. External network availability and browser appearance remain outside this check.

## Bounded Page page-orientation witness — 2026-10-10

The [partial witness](css-page-3-page-orientation--editor-capture-20261010--48ec99cd3a58.md) retains complete §1.1 and §7.1.2 from the Page 3 editor capture displaying 3 March 2026. The full captured HTML is 407,670 bytes, SHA-256 `48ec99cd3a581f90b588c6d578be8fd97102c01d6482b1d75ebf39108924dbfb`. Other source sections are not represented by this witness. It supplies only the named `page-orientation` grammar and common CSS-wide statement; other Page/margin requirements keep their own sources.

Clause text matches after whitespace normalization, excluding documented editorial table headers and informative-note labels. All 17 original fragment IDs and 18 resolved link destinations retain source order and identity; all five local targets resolve. Both source note blocks remain informative and literal CSS examples preserve their characters. The original copyright and license notice is retained with the unchanged shared local 2023 W3C license. Source-generated ambiguous hyperlink tooltips are omitted, as disclosed in the witness. No source scripts were executed.

This is a partial-clause fidelity check, not a full-document conversion, browser-appearance check, or implementation-conformance verdict. The source catalog now contains 193 identities: 190 full representations, one partial witness and two metadata-only exceptions. Earlier dated verification totals describe their original scope.

## Text, font and effects source resolution — 2026-10-10

Ten new references add 44 source identities: five partial standards captures, 23 full browser representations (including two newer byte-identical font identities), three partial effects files and thirteen partial registry files. The current catalog totals are 237 identities: 213 full, 22 partial and two metadata-only. Earlier dated totals describe their original check scope. Original source hashes in the catalog identify complete HTML/files; bounded-input and Markdown hashes remain separate in each reference’s provenance. Historical editions and both metadata-only exceptions remain intact.

### Standards clause conversions

Official drafts were inspected first; complete retrieved HTML and response identities supply each version anchor. Bounded extraction retains the original source head/status/legal material and every descendant in each named section through the next same-or-higher heading. Omitted sections are disclosed and remain linked upstream. Conversion uses installed Pandoc 3.1.11.1 and the repository [HTML preparation, semantic filter and checks](tools/html-to-markdown/README.md); source scripts/styles are not executed. No partial conversion is described as whole-document coverage.

| Capture | Verified IDs | Literal blocks | Tables | Distinct representation details |
| --- | --- | --- | --- | --- |
| [Fonts 4 width matching](css-fonts-4-width-matching--editor-capture-20261010--72f4c49ea2ec.md) | 132 | 2 | 2 | Complete §§2.3 (including 2.3.1) and 5.2; 38 ordered source cells. |
| [Text Decoration 4 controls](css-text-decor-4-position-controls--editor-capture-20261010--8820547faa34.md) | 202 | 6 | 7 | Complete §§2.4–2.9 (including 2.8.1); 112 ordered source cells. |
| [Text 4 whitespace/alignment/hyphenation](css-text-4-whitespace-alignment-hyphenation--editor-capture-20261010--2b0d3e0d5174.md) | 347 | 17 | 5 | Complete §§4.1, 4.3, 6.3.5, 6.4, 7.1 and 7.2; one tab expanded by Pandoc restored to exact literal source. |
| [Filter Effects 1 clauses](filter-effects-1-effects-clauses--editor-capture-20261010--b9a791956ef1.md) | 375 | 11 | 4 | Six MathML expressions represented by exact source StarMath 5.0 annotations; raw MathML/annotation comparison retained. |
| [Transforms 2 clauses](css-transforms-2-effects-clauses--editor-capture-20261010--ec252486c8c9.md) | 108 | 18 | 1 | Nine literal TeX matrix/formula paragraphs remain literal blocks; one tab-sensitive block uses pre/numeric tab entities. |

The checks compare IDs, headings, whitespace-normalized prose, literal code, variables, resolved links and table cells against each bounded source input; all five report no conversion errors. The 19 retained tables have ordinary grids without source spans; their actual row/column/cell relationships are checked, not inferred from a flattened dump. Original copyright, license and informative/normative classification remain. Formula-image and other figures remain upstream links. The conflicting Filter Effects constants are retained rather than harmonized. Markdown fidelity is not standards implementation conformance, native table accessibility or browser/pixel equivalence.

### Browser and registry source retention

Browser witnesses use immutable WebKit objects from the existing checkout and official immutable Blink Gitiles TEXT responses decoded before hashing. Complete bodies preserve exact original bytes, including final blank lines and file-specific license headers. The two newer FontSelection identities reuse byte-identical complete preferred bodies. The partial effects witness retains complete isBackFaceVisible and calculateFillLayerImageGeometryImpl functions and explicitly bounded paintLayerContents ranges; omitted intervening content is disclosed. Exact line ranges, full-file hashes and segment hashes remain beside each excerpt. License text is linked per source; project BSD terms are not substituted for LGPL or file-specific alternatives. CSSProperties remains metadata-only and is not newly redistributed.

The Parley/provider witness was captured from already installed registry sources without software acquisition or execution. Thirteen complete original file hashes anchor partial declarations/functions and exact selected line ranges. Original headers accompany the excerpts; installed Parley MIT/Apache texts were compared byte-for-byte with the local license copies. No Git commit is invented for registry identities, and no copied source is compiled into Surgeist by retaining these references.

Fidelity records cover the actual retained browser bodies/segments and registry excerpts, not omitted implementation paths. The catalog records 44 distinct added source URLs/version identities without re-adding an existing identity; duplicate hashes for the two newer font files are intentional and their shared representation is disclosed. The [edition mapping](SOURCE-EDITIONS.md#text-font-and-effects-source-resolution) separates standards text, browser precedent and adopted product policy. The [owning source issues](SOURCE-EDITIONS.md#operative-source-decisions-by-issue) record the feature determinations; the [text/layout contract](../docs/text-layout-contract.md) owns shared mechanisms. Source conversions do not silently amend standards or mark product implementation complete.

## Layout sizing, fragmentation and overflow source-resolution — 2026-10-10

Twelve references add 22 source identities (six partial standards, seven full browser and nine partial browser identities). The catalog now contains 259 identities: 220 full, 37 partial and two metadata-only exceptions. No passive SVG asset was added. The partial Grid 2 rendering has the same full HTML hash as the already retained 2025-03-26 CRD, and the overflow TransformationMatrix.cpp excerpts share the existing preferred-pin identity. Both extend representation without adding identities. The published Grid 3 research capture likewise matches the already retained 2026-01-21 WD and adds no new reference or identity.

The standards conversions retain the complete selected clauses, document head, status and original legal notice. Full HTML hashes anchor editions; bounded input hashes identify conversion scope. Original fragment IDs, visible text, links, literal blocks, code/variables and table cells and spans were checked against each bounded HTML input. Scripts and styles were omitted without execution; image references remain upstream links. Partial conversions do not supply omitted sections or resolve the standards’ open issues.

| Bounded standards representation | Complete source SHA-256 | Bounded HTML SHA-256 | Complete selected scope |
| --- | --- | --- | --- |
| [CSS Tables 3 sizing](css-tables-3-sizing--editor-capture-20261010--e0ddc09996f1.md) | f5adcb6c9ca724aad1dd1a04e20c2dab2695b33707af6da2acae30225ec4e381 | e0ddc09996f1f860a83fd67a72b8fd8148e79ee22c5c6e1f0b0172b879007505 | complete sections 3.1, 3.5.1, 3.8, 3.9, 3.10 and 9, including descendants; head/status/legal notice |
| [CSS Flexbox 1 intrinsic sizing and fragmentation](css-flexbox-1-intrinsic-fragmentation--editor-capture-20261010--cc15b06af9bf.md) | cc15b06af9bfc9b875db1072d5ae8c2e153017efbc07aa405d6aca5e3dc2a7bd | 9e189415a389d3757a6828cb0fe8f3d9918dd597eab3eafb8914a583595d1845 | complete section 9.9 and section 10, including informative section 10.1; head/status/legal notice |
| [CSS Grid Layout Module Level 3](css-grid-3-layout-decisions--editor-capture-20261010--54fd3c34f1ce.md) | 3d85385289b72096aa6ce0af877211e42401573304821b5cb71f3f2f85ee121f | 54fd3c34f1ce1b5ff1937a46dc1ac1975012b01c045577ab2b2c2926df580fa3 | complete selected heading sections `grid-lanes-model`, `grid-lanes-track-templates`, `grid-lanes-track-placement`, `intrinsic-sizes`, `alignment`, `pagination`, `abspos`; head/status/legal notice |
| [CSS Fragmentation Module Level 3](css-break-3-layout-decisions--editor-capture-20261010--debb8c3e11e2.md) | 854ffd2e0e97b08f218c80bcfcbf2d5b4fa9711ae2f695750d13d98925e00dc2 | debb8c3e11e2bd1a6a2fb82f5d93e3a4a2f2432e05f23a3c624f101f0c23831a | complete selected heading sections `fragmentation-model`, `breaking-controls`, `breaking-rules`, `breaking-boxes`; head/status/legal notice |
| [CSS Grid Layout Module Level 2](css-grid-2-fragmentation-clauses--editor-capture-20261010--444e85eb609a.md) | 05aa64853c4428146973943b35caf121e44c1076bdf5b8c29f8896dba9b778e2 | 444e85eb609a7e27708f04240dd60df1e104c651b398f6f519488d24d608a601 | complete selected heading sections `pagination`; head/status/legal notice |
| [CSS Content 3 leaders](css-content-3-leaders-clauses--editor-capture-20261010--586d408a8de7.md) | a0db7cbeb488725fe33adf99a6dde5adbca4afe14ed45fa5bdc7f44ef864dc94 | 586d408a8de7bd160fa01009331ef04ec7d283e7e71278a0dfe9ad63c0b9f9c0 | complete content-property (including accessibility), strings and leaders sections; head/status/legal notice |
| [CSS Overflow 3 geometry](css-overflow-3-geometry-clauses--editor-capture-20261010--86c09a636507.md) | 4635826dcd37bbf938b4043c08359ccf5c9a0befe88c6c8a33480bae523cd979 | 86c09a636507eabbbe39c1a9507d79602e9c75451170c90077c1b4c6cb207ed5 | complete sections 2 and 3, including all subsections; head/status/legal notice |

Four WebKit table files, one GridLanesLayout.cpp file and two Blink flex files retain complete original UTF-8 source bodies. Other browser records explicitly delimit excerpts and retain each complete original file-level copyright/license header. Their full-file hashes, exact ranges and omission boundaries remain in the references. Preferred and newer FlexFormattingContext.cpp complete-file hashes differ; only the retained header and complete placeFlexItems function are byte-identical and share one displayed body. Leader evidence is limited to the named content model/parser front doors; it does not establish engine-wide absence. StyleAdjuster.cpp retains only the identified adjustment branch. Overflow witnesses do not define clipping, transform or margin policies beyond the retained evidence.

Exact source/legal comparisons and bounded-conversion checks passed for the final author inventories. The independent integration review is recorded separately; these checks establish reference fidelity rather than product behavior. The [source-edition mapping](SOURCE-EDITIONS.md#layout-sizing-fragmentation-and-overflow-source-resolution) owns selection, and the [W3C](../licenses/w3c/NOTICE.md#layout-source-resolution-clause-captures), [WebKit](../licenses/webkit/NOTICE.md#layout-sizing-fragmentation-leaders-and-overflow-witnesses) and [Chromium](../licenses/chromium/NOTICE.md#flex-fragmentation-and-content-parser-witnesses) notices retain per-source legal coverage. Existing historical entries and their verification records remain unchanged.

## HTML parser source witnesses

Captured 2026-10-10: 13 complete files and 2 partial files from pinned html5ever and html5gum checkouts. Markdown, TOML and Rust sources are preserved in four-backtick fences with complete-file SHA-256, immutable upstream URLs and original line ranges for excerpts. No HTML conversion or Pandoc was needed; no tables were reconstructed. Original licenses and copyright notices accompany the witnesses. These are bounded implementation evidence, not a converted HTML standard or a whole-repository snapshot.

Byte comparison verified all fifteen retained file bodies/excerpts and complete-file hashes against the pinned checkouts. The upstream html5ever README contains one trailing space retained verbatim inside its source fence; this intentional source-fidelity exception is reported by `git diff --check`. Added prose and generated wrappers have no whitespace findings.

## DOM, selector execution and bounded style source additions — 2026-10-10/11

These additions contribute 40 distinct source identities: 14 full witnesses, 25 partial witnesses and one ordinary metadata-only minutes provenance record. Conditional 5 and Content 3 each have two bounded Markdown renderings sharing a complete original HTML hash. Blink css_properties.json5 adds a bounded rendering of its existing complete source identity. The source catalog counts identities, not Markdown files or extraction hashes.

Five complete selector/cascade/scoping/pseudo HTML captures and the bounded standards captures use the approved Pandoc 3.1.11.1 conversion with source headers, original status/notices, definition IDs, normative/informative labels, literal/code/variable structure and original links retained. Original scripts/styles are omitted without execution. No compiler was acquired for these additions. Displayed live-page revision metadata and separately acquired repository sources are provenance, not additional distributed source bodies.

The six DOM captures pass source-to-parsed-GFM checks for 6,705 IDs, 69 headings, 41 literal blocks, five tables, 113 cells and 5,326 retained links, with no conversion errors. Complex node-type and focus tables remain literal semantic HTML: original row/column spans, cells, anchors and inline semantics are preserved instead of flattened. Literal table code uses numeric newline/tab entities; these preserve source text under parsing. This exception to the original collection’s grid-table presentation policy is deliberate source fidelity, not a claim of browser-rendering equivalence.

| Verified conversion group | Captures | IDs | Headings | Literal blocks | Tables | Table cells | Retained links |
| --- | --- | --- | --- | --- | --- | --- | --- |
| selectors | 5 | 3950 | 345 | 175 | 1 | 220 | 7617 |
| style | 6 | 660 | 32 | 23 | 12 | 169 | 1604 |
| specialized | 7 | 706 | 34 | 31 | 1 | 14 | 1039 |
| specialized677 | 1 | 165 | 7 | 7 | 1 | 18 | 210 |

Every listed conversion reports zero errors. Counts cover retained source sections, not omitted document bodies, and are not conformance tests. Browser excerpts retain exact original line ranges, complete file-level headers and source-byte hashes; complete files retain their full bytes. Resolution witnesses retain two brief attributed comment quotations plus response/body hashes. Dated minutes have URL/hash provenance only and are therefore metadata-only without a rights exclusion. Captured upstream links are provenance, not a claim that external network availability was tested. Legal header copies are verified byte-for-byte against the selected source; existing license files are reused only where exact text matches.

Final integration checks confirm 314 catalog rows, the full/partial/metadata totals above, all 40 added original-source hashes against retained raw evidence, and 13 exact legal-header copies. Conditional 5, Content 3, RenderElement.cpp and Blink css_properties.json5 each occupy one catalog row; the full RenderElement witness upgrades its earlier bounded representation without adding another identity. Parsed Markdown/SVG link checks cover 226 reference/notice files and 73,018 local link occurrences with no missing path or fragment. The owned catalog/notice edits pass `git diff --check`; quoted source-fidelity whitespace exceptions in earlier captures remain as previously disclosed.

## Additional source-question references — 2026-10-10/11

These additions contribute 27 distinct source identities: 7 full witnesses, 18 partial witnesses and 2 ordinary metadata-only provenance records. 6 additional representations reuse existing source identities; existing complete witnesses remain complete. Historical captures and rights/coverage exceptions are unchanged. Full original document/file/response hashes remain separate from bounded input and Markdown hashes.

| Verified bounded conversion group | Captures | IDs | Headings | Literal blocks | Tables | Cells | Retained links |
| --- | --- | --- | --- | --- | --- | --- | --- |
| additional-style | 4 | 669 | 29 | 19 | 4 | 42 | 690 |
| additional-selectors | 1 | 168 | 12 | 13 | 0 | 0 | 116 |
| additional-content-host | 2 | 181 | 17 | 8 | 4 | 74 | 253 |
| additional-media | 3 | 321 | 31 | 5 | 8 | 64 | 325 |
| additional-animation-host | 1 | 414 | 11 | 10 | 0 | 0 | 400 |

All conversion records report zero errors against their retained sections. The approved Pandoc 3.1.11.1 workflow preserves original status/notices, source IDs, literal code, links and table cell/span models; scripts/styles are omitted without execution. Browser excerpts preserve exact declared line ranges plus complete file-level legal headers. Four brief comment quotations retain response/body hashes and edit times; two additional comments retain metadata only. The source catalog counts those nonrestricted provenance records separately from the two rights/coverage exceptions. Checks concern source fidelity and identity, not implementation conformance, complete browser behavior, visual equivalence or external link availability.

Final integration checks confirm 341 catalog rows and the 254 full, 82 partial and five metadata-only totals. All 33 added/reused representation records identify matching retained original source bytes; 14 accompanying legal headers match their source blocks byte-for-byte. Six representation additions reuse source identities, including the two new Web Animations renderings sharing one original document. Parsed link checks cover 243 reference/notice files and 73,732 local link occurrences without missing paths or fragments. Blink page.cc preserves its file-specific GNU Library GPL version 2-or-later header and uses the existing complete Blink license-credit text, rather than a generic Chromium BSD declaration. Owned catalog/notice edits pass `git diff --check`.

## Pinned HTML language snapshot — 2026-10-11

Ten bounded conversions share one immutable WHATWG HTML snapshot, revision `877d6146487f15b15bc8dd0e3c625063a37b456a`, complete original SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f` (14,930,614 bytes). The full original source hash identifies the edition; each filename separately identifies its complete-section extraction. All chapter 4 element subsections and descendant clauses are retained across the captures. The [edition map](SOURCE-EDITIONS.md#pinned-html-language-baseline) explicitly bounds omitted browser runtime chapters and historical selections.

Approved Pandoc 3.1.11.1, the existing repository preparation and semantic filter preserve source headings/IDs, literal code, linked code, variables, prose, original link destinations, images and complex table cell/span structure. Adaptations prevent Markdown automatic linking and punctuation reinterpretation, preserve explicit line breaks in literal examples and retain `samp` semantics. Tables retain semantic HTML. Six original inline SVG diagrams retain their parsed structure with formatting whitespace normalized; GitHub/native rendering equivalence is not claimed. Three embedded iframe demonstrations are retained as exact inert literal markup before the captured body, rather than fetched, executed or recursively inserted into the standard text. Other source images/media retain their original external resource links; those resources are not pinned or bundled, and no media was downloaded.

| Verified capture group | Captures | IDs | Headings | Literal blocks | Tables | Table cells | Retained links |
| --- | --- | --- | --- | --- | --- | --- | --- |
| HTML language | 10 | 45205 | 713 | 872 | 80 | 11854 | 39302 |

Every conversion reports zero fidelity errors after 8629 same-snapshot crosslinks are rewritten to retained companion captures. Original omitted-target links continue to resolve to the exact immutable snapshot. Complete §4 coverage is additionally checked by inclusion of every element subsection and all 27456 source IDs beneath that chapter. These checks concern source preservation, not parser or browser conformance.

The supplemental html5ever witness adds three partial source identities at `d7232d746d3112f08926d169591576bd12fe2fdf`. All complete-file hashes and nine selected line ranges match original upstream files; original MIT OR Apache-2.0 notices are retained verbatim. One full HTML source identity plus three engine source identities increases the catalog to 345: 254 full, 86 partial and five metadata-only records. Existing legal files are reused after exact byte comparison; WHATWG prose remains Creative Commons Attribution 4.0 International, with BSD scope only for incorporation into code. Source selection does not itself add a production dependency, implement HTML or rewrite historical conformance evidence.

Final integration checks cover 345 catalog rows, 255 reference/notice files and 101,454 local link occurrences without missing paths or fragments. Catalog/legal Markdown passes `git diff --check`. Eight trailing-whitespace lines remain inside verbatim source examples in the forms/tables, scripting and obsolete captures; their preservation is intentional and the literal-source comparisons pass.

## HTML dimension-list cursor determination — 2026-10-11

The [cursor witness](html-dimension-list-cursor--webkit-73aa6c89e2cb-blink-7984f9d11800.md)
adds two partial browser source identities. Both full original source hashes
match the retained raw files. All four literal blocks match exact source bytes:
WebKit's 25-line original header and lines 543–684, and Blink's 29-line original
header and lines 43–143. Accompanying license-header files match those originals
byte-for-byte. Added headings and the explicit bounded interpretation are
documentation, separate from unchanged source excerpts. No upstream erratum or
full algorithm parity is claimed. The catalog now contains 347 identities:
254 full, 88 partial and five metadata-only records.

## Bounded Referrer Policy vocabulary — 2026-10-11

One partial source identity is added at W3C editor-source commit `cc435b05ca4a94f7f1a139be5074b168d20014db`. The complete 49,334-byte original matches SHA-256 `a277e78e06418ec60afc6f499aee79d4f3a3a067e0fd97e9886a2d9e3de95eb1`. The bounded Markdown conversion retains original lines 216–236 (vocabulary paragraph and exact IDL apart from uniform indentation), line 249 (default sentence), and lines 518–522 (first empty-string paragraph). Source title, Editor's Draft status and editor identities are retained in its wrapper. Literal vocabulary and IDL are checked against the complete original; prose changes remove Bikeshed link/definition markup while preserving the selected text. There are no selected tables, images, diagrams or complex layout; no compiler, supporting-data download or network algorithm is introduced.

The repository license declaration is copied byte-for-byte. Existing full W3C document/software license texts supply the applicable prose and marked Web IDL terms. The wrapper identifies the absence of an original copyright statement and includes license-prescribed source-year and derivative notices; these are separate from original text. Counts become 348 identities: 254 full, 89 partial and five metadata-only. These checks establish bounded source fidelity and local attribution, without claiming whole Referrer Policy or HTML conformance.

## HTML list participation definition — 2026-10-11

The [being-rendered excerpt](html-being-rendered--877d6146487f.md) converts original
lines 99611–99622 of the already retained HTML snapshot: one definition and two
notes. All three paragraphs match the complete original after inline-markup
removal and layout-whitespace folding. The complete 14,930,614-byte source still
matches SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
The excerpt’s scope explanation is separate from its retained original text.
The existing WHATWG prose license and source identity apply; no code, diagrams,
tables, styles or browser implementation are incorporated. Catalog counts remain
348 identities: 254 full, 89 partial and five metadata-only.

## HTML language, directionality and modal sources — 2026-10-11

The complete Unicode 18.0.0 DerivedBidiClass data and its exact Unicode License
v3 text are retained in literal blocks. Both blocks match their raw sources
byte-for-byte; the data source SHA-256 is
`d9e23222522551348ea1ccfbb4f62efbf98982afb95840f8959c08ed992c5607`.
The separately retained generator input matches the same complete original.
The 1991–2026 legal text differs from the older Unicode companion's license,
so it has a separate versioned legal directory. This full data witness includes
all explicit ranges and general/range-specific missing defaults.

UAX #9 revision 52 and RFC 5646 add metadata-only source identities. The UAX
report's publication restriction does not apply the data-file license to report
prose or tables. RFC excerpt permission provenance remains unresolved under its
pre-November-2008 notice and historic IETF terms. Their technical text remains
outside the public collection; edition, external citations and complete source
hashes remain discoverable. The RFC's original legal legend is retained
verbatim separately. A private draft replaced eight form-feed characters while
wrapping RFC excerpts, so it is also excluded from any claim of byte-exact
public conversion.

The modal inertness determination adds only source metadata for the existing
WebKit StyleAdjuster.cpp and Document.cpp identities. Raw original headers and
selected ranges were compared exactly during research, but are not reproduced
in the new public record: the supplied licensing basis did not establish this
documentation-excerpt distribution. The independent determination, immutable
URLs, full-file hashes and line ranges remain. This does not expand the scope
of older source witnesses or claim a blanket restriction on all excerpts.

The catalog contains 351 distinct source identities: 255 full witnesses,
89 partial witnesses and seven metadata-only records. Counts concern original
identities, not additional Markdown renderings. Local links and the new source
anchors are checked; historical conversion records remain unchanged. These
checks establish provenance and publication handling rather than HTML behavior
or whole-parser conformance.

## HTML editability source selection — 2026-10-11

The [HTML editability capture](html-editability--877d6146487f.md) retains complete
§§6.8.1, 6.8.2, 6.8.4 and 6.8.5 from the existing immutable HTML snapshot.
It adds a bounded representation of that existing source identity. The
[delegated editable definition](execcommand-editability--85de9ff4a981.md) adds one
partial source identity from w3c/editing revision
`85de9ff4a9816139d06fc2dc39d5dacdc456fddc`, complete original SHA-256
`da222a4ea70349eba9d9036a20708f1fa63690118f6c657da3f7f06c56fa78cf`.
Original notice, exact repository license declaration and linked full W3C
Software and Document License are retained. Selected paragraphs were converted
with Pandoc 3.1.11.1, retaining authored unresolved Bikeshed tokens and absolute
source links. Source scripts were not executed. Scope selects HTML-delegated
editability only; command execution and editing UI are outside it.

## HTML autocomplete credential determination — 2026-10-11

The [WebAuthn edge determination](html-autocomplete-webauthn--877d6146487f.md)
contains independent analysis and source metadata, without browser code or
upstream issue prose. The pinned HTML author grammar and processing branches
were compared with the existing retained original. A scoped source review also
confirmed the browser file hash, cited lines and dated upstream status/merge
links. The WebKit witness corroborates two lowercase outcomes only; its broader
case normalization is not selected. Autofill.cpp adds one metadata-only file
identity, giving 353 identities: 255 full, 90 partial and eight metadata-only.
HTML's existing identity is reused, and linked issues/PRs without retained
response snapshots do not add identities. This records provenance and the
selected recovery disposition rather than autofill or credential execution.

## JavaScript MIME essence and HTML checkpoint sources — 2026-10-11

The [JavaScript essence definitions](mimesniff-javascript-essence--39aa53511b13.md)
add one partial source identity: MIME Sniffing revision
`39aa53511b13953d84fef8d4131d6f61d0ccbde6`, displayed 17 July 2026,
complete original HTML SHA-256
`11caac49505995bcef51153a827c6fe96464111c8914e2c399d9e8fe9a467cd7`.
The bounded conversion retains original lines 838–880: the JavaScript MIME type
definition, all 16 distinct essence strings, the ASCII case-insensitive string
match and attached HTML hook note. Independent comparison of the GFM rendering
with the retained original preserves 528 non-layout text characters, all eight
ordered links and nine retained IDs. Every same-edition target resolves in the
original snapshot. The original footer declares CC BY 4.0 for prose and BSD
3-Clause for portions incorporated into code; the existing full WHATWG license
is reused without claiming an acquired MIME repository LICENSE byte comparison.

The [microtask queue and checkpoint](html-microtask-checkpoint--877d6146487f.md)
adds a bounded representation of the existing HTML877d identity. Its queue and
reentrancy definitions and complete eight-step checkpoint retain all explanatory
notes and promise, IndexedDB, ClearKeptObjects and timing hooks. Independent
comparison preserves 1,399 non-layout text characters, all 36 ordered links and
40 retained IDs; every same-edition target resolves. The two prepared extracts
are retained byte-for-byte. Their Markdown SHA-256 values are
`72aabeed12109427769c58c3b2b9d1d4cdaaa6a539edefe307abcebfbb7730bb`
and `0d8b8d8d533d1a62a3f9ddbc91fb44831d8e4f458a1a63b3069cc12710594815`.
Conversion used Pandoc 3.1.11.1 with absolute links and no source-script execution.
These checks establish selected text/anchor fidelity, not browser execution,
complete document conversion or universal external-link availability.

Long Animation Frames is an explanatory live citation inspected on 2026-10-11;
no original page bytes/content hash or verified commit/page association was
retained. It adds no source identity or selected pin. HTML's retained checkpoint
establishes the hook's existence. The standalone absent-runtime-domain reduction
is an explicit architecture capability specialization, not execution of that
external algorithm or a normative null-script-return rule.

The catalog now contains **354 identities: 255 full, 91 partial and eight
metadata-only**, with **81 passive SVG assets** unchanged. Historical counts
above remain dated records. No production dependency is introduced by retaining
these source references.

## HTML sandboxing source representation — 2026-10-11

The [complete sandboxing section](html-sandboxing--877d6146487f.md) adds a
representation of the existing HTML877d identity, complete original HTML
SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
It retains §7.1.5, original lines 65599–65796, immediately before §7.1.6:
sixteen flag definitions, the complete three-step directive algorithm with
sixteen entries, and the popup, iframe, active, CSP-derived and creation flag
clauses. Independent source-to-Markdown comparison preserves 7,976 non-layout
characters, 117 ordered links, 141 unique ordered IDs and three informative
notes; all 102 same-edition targets resolve in the retained original.
Pandoc 3.1.11.1 conversion uses absolute links and inert wrappers to preserve
code/paragraph IDs, note classes and the original semantic heading. No source
scripts execute. Sixteen Markdown hard breaks are serialized with backslash-newline instead
of trailing spaces, preserving the same GFM break semantics. The retained
Markdown SHA-256 is
`ce19566ea24ff77f990e496b76b98a642166a55bbfe68eba7c0da7dfbedca830`.
The existing exact WHATWG license applies: CC BY 4.0 for reference prose,
BSD 3-Clause only for portions incorporated into code. This representation
does not claim browser navigation, CSP evaluation or sandbox enforcement.
The source counts remain 354 identities: 255 full, 91 partial and eight
metadata-only; 81 passive SVG assets and historical dated counts are unchanged.

## Srcset density overflow source retention — 2026-10-11

The complete [HTMLParserIdioms.cpp](webkit-html-parser-idioms--73aa6c89e2cb.md)
upgrades its existing partial identity to full source coverage. Its complete
original SHA-256 is
`99107062a7eaec7499838aaa5fab77045d6d92348c62b260cff4daf9cbef226d`; the
previous dimension-cursor excerpt remains linked as an additional representation.
The complete [HTMLSrcsetParser.cpp](webkit-html-srcset-parser--73aa6c89e2cb.md)
adds one full identity, complete original SHA-256
`1905a2a03797d7851859f1555b2d5b3caff72f1cec18c307013391001ec0194c`.
Both originals are retained UTF-8 source at WebKit revision
`73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Complete fenced bodies compare
byte-for-byte with the retained originals, including their original headers.
The existing Idioms two-clause header and newly retained Srcset three-clause
header compare byte-for-byte with their respective original file headers.
Added source/legal links resolve locally; documentation uses UTF-8 and LF.
No source scripts execute and no additional acquisition is involved.

The [edition determination](SOURCE-EDITIONS.md#srcset-density-conversion-overflow)
selects only candidate-local omission after density conversion overflow. HTML's
existing normative identity and unchanged captures remain authoritative for
other grammar and recovery. This conversion does not establish complete
WebKit behavior, source-set normalization, URL validation or image selection.
The catalog now contains **355 identities: 257 full, 90 partial and eight
metadata-only**. Historical dated counts and **81 passive SVG assets** remain
unchanged.

## Absent SVG document-title metadata — 2026-10-12

The [bounded WebKit witness](webkit-document-title--73aa6c89e2cb-metadata.md)
retains original file URLs, SHA-256 identities and inspected ranges at revision
`73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Document.cpp's complete original
hash is `c3f64c3d98f23601ffa74b2e63b82f30aaeb4035bb537c657475a8c00095c55c`;
the title evidence concerns original lines 2462–2475, 2524–2527 and 2550–2561.
Document.h's complete original hash is
`d834dcea6916301c50febc380c74c28f0162c72c75b510b34d4b3feb409fa542`;
its getter evidence is original line 1203. Retained private originals match
those hashes. No original source excerpt, body, converted browser code or
new license text is redistributed in this metadata addition.

Document.cpp reuses its existing partial identity and legal-header witness;
its earlier coverage is not upgraded. Document.h adds exactly one metadata
identity. Existing pinned URL/hash identities remain unchanged, including
byte-identical bodies at distinct historical revisions. Local source and
heading links resolve, text uses UTF-8/LF and whitespace checks pass. The
[edition determination](SOURCE-EDITIONS.md#absent-svg-document-title) fills
only the absent direct SVG title case; it does not certify WebKit parity or
adopt its other title processing. The catalog now contains **356 identities:
257 full, 90 partial and nine metadata-only**. Current overview counts match;
dated historical records and **81 passive SVG assets** remain unchanged.

## Foreign popover target metadata — 2026-10-12

The [bounded metadata witness](webkit-popover-target--73aa6c89e2cb-metadata.md)
identifies HTMLFormControlElement.cpp at WebKit
`73aa6c89e2cb77c46184a81aec944e4ab99d114d`, complete original SHA-256
`84bb969f25feec272fb4b764e80c13cdaf9bbbe0a4ee811ef5872a55cc498740`.
Inspected original lines 335–362 include the interface guard at 348. The
retained private original matches that identity. This addition redistributes
no original body, source excerpt or converted browser code; GNU Library GPL
notices remain with the private original. The
[edition determination](SOURCE-EDITIONS.md#foreign-popover-target-interface-guard)
adopts only foreign-target rejection after actual generic association.

Exactly one previously absent URL/hash identity adds a metadata-only record.
The current catalog contains **357 identities: 257 full, 90 partial and ten
metadata-only**. Historical dated counts and **81 passive SVG assets** remain
unchanged. Local source/heading links, UTF-8/LF and whitespace are checked;
these provenance checks do not establish browser parity or live effects.

## HTML-delegated SVG script sources — 2026-10-12

The [Tiny processing reference](svg-tiny12-script-processing--REC-20081222--2f36eb76440c.md) retains selected
edition-index ranges 15–49 and 74–146 and script-chapter range 91–198 from the dated
2008 Recommendation. The complete original chapter and index have separate
catalog identities. Conversion preserves original prose, four-step order,
notification qualification, later modification rules, notice and complete status;
only bounded selection, Markdown presentation, heading IDs, absolute dated links
and inter-element whitespace change. All 74 source link occurrences and selected
text/IDs were checked against retained raw captures. The original copyright is
©2008; the separately labelled ©2015 derivative notice and local document-license
terms cover software-support conversion under the official 2015 update, which
extends permission to existing publications. This is not a replacement technical
specification or an edited offline algorithm.

The [SVG2 observations](svg-script-observations--capture-20261012-metadata.md) add three metadata-only
URL/hash identities. Their conformance-chapter bodies differ at publication
header raw line 62; relied-on raw 91–161 clauses match. That bounded comparison
establishes no global edition equivalence. No scripts, stylesheets, MathJax or
linked resource was run or acquired. The
[WebKit clone witness](webkit-svg-script-clone--73aa6c89e2cb-metadata.md) adds one metadata-only
identity, retaining original URL/hash and inspected 117–120 scope. Its original
GNU Library GPL source stays private; no browser code/header excerpt is shipped.
The existing DOM clone reference supplies its acknowledged standards gap.

The [source-edition mapping](SOURCE-EDITIONS.md#offline-disabled-svg-script-and-clone-sources)
separates normative delegation, offline capability policy, dated observations
and the bounded clone correction. Reference retention is not implementation
acceptance. The catalog contains **363 identities: 257 full, 92 partial and
fourteen metadata-only**. Historical dated counts and **81 passive SVG assets**
remain unchanged. Original captures and conversion-fidelity evidence remain
available for the active parser implementation and review.

## HTML-delegated Fetch destinations — 2026-10-12

The [bounded Fetch witness](fetch-script-like-destinations--e9460d1b22ea.md)
retains the complete script-like definition and adjacent warning from immutable
revision `e9460d1b22ead1c6a72eeac37fb2ec0dc01aeb1d`, rendered lines 2820–2828.
The complete original is 1,932,351 bytes, SHA-256
`639d437c000a870bf161804a92f9e0ce1b22dae61f39d8975a7b8429302534eb`.
Hand conversion changes heading/anchor presentation, quotes/typography and links;
the exact six definition keywords and excluded xslt warning remain. Explicit
Surgeist selection commentary is separate from the source paragraphs. There
are no tables or diagrams in the selection and no source scripts are executed.
The exact upstream LICENSE matches the existing complete local WHATWG license;
prose attribution and modification notice are retained.

This adds one partial identity. The current collection is **364 identities:
257 full, 93 partial and 14 metadata-only**, with all prior 363 identities and
81 passive SVG assets preserved. Earlier dated counts remain historical.

## DOM-delegated XML pseudo-attribute grammar — 2026-10-12

Two distinct partial identities retain the
[whole XML-stylesheet §3](xml-stylesheet-pseudo-attributes--REC-xml-stylesheet-20101028--a0e1a5198a21.md#pseudo-attributes)
and [bounded XML grammar closure](xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md).
Complete originals are respectively 34,841 and 219,926 bytes, SHA-256
`a0e1a5198a21c2aa4b623fc35b3c890d6e51c6c14ee126aed71cf3b93157515a`
and `f58a4b9e1e5b8bac13fb55c4e5c7c8c5e2c176aa45ee68e2f1ab2498511927ec`.
The latter retains its own disclosed 2013 in-place RFC-link correction.

The stylesheet selection covers raw 35–43 (title/editors/notice), 45–88
(complete original STATUS) and 164–196 (entire §3). XML covers raw 39–53 and
59–95 for its corresponding original header/complete STATUS; 306–320 for the
complete Char table and following encoding paragraph/discouraged-character note;
322–331 for complete S context/table/note; 331–357 for Name context and exactly
the first three complete production rows at 358; 1141–1150 for complete CharRef,
constraint and interpretation; and 1724–1780 for complete §6 Notation.
Unselected Names/Nmtoken/Nmtokens rows and the rest of both documents are not
represented as converted text.

Selected original text is equal under HTML-collapsible whitespace comparison;
inline code, grammar operators/ranges, ordered anchors and table row/cell models
were checked directly against the retained originals. Headings/paragraphs use
Markdown; grammar tables, ordered and definition lists retain semantic HTML.
Explicit source anchors remain. Original-relative and unselected-fragment links
become dated absolute references; only the selected XML dependency fragments
are linked locally. Inline layout styles and inter-element whitespace change.
No source script, linked resource, XML parser or stylesheet loader is executed.

Original ©2008/©2010 notices, editor material and full original STATUS remain.
Each reference adds the prescribed separate ©2015 implementation-support
derivative notice and links complete local 2002/2015 document terms, supported
by the official 2015 existing-publications update. These references accompany
software/support material and are not replacement technical specifications.
No generic software-license substitution or pending erratum is introduced.

The current collection is **366 identities: 257 full, 95 partial and
14 metadata-only**; all prior 364 identities, dated historical counts and
**81 passive SVG assets** remain unchanged. Source retention does not certify
ordinary-template implementation or whole-parser completion.

## HTML spin-event-loop reference — 2026-10-12

The [bounded spin reference](html-spin-event-loop--877d6146487f.md#spin-the-event-loop)
retains the entire algorithm wrapper at full-original raw 81620–81655: seven
outer steps, two nested two-step lists and four Note paragraphs. The complete
HTML877d original is 14,930,614 bytes, SHA-256
877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f.
The byte-exact contiguous raw wrapper is 3,016 bytes, SHA-256
a74d317d6c0fe82edf2496cb8e5c998be863cb04b2181618e4e0231b2b9fe028.
The normalized bounded HTML input is 4,215 bytes, SHA-256
a9ae6309c8c229a9953f02c9e01d5e826d638ae826a06f454f46154be410a6be.
Its quoted attributes, explicit closing list tags and absolute links are
presentation conversions, not original-byte identity.

The supplied Pandoc 3.1.11.1 conversion is preserved with an expanded provenance
wrapper, the source algorithm marker and explicit Note labels. Selected display
text is equal under HTML-collapsible whitespace and paragraph-boundary comparison;
the complete list structure, four note texts, 12 variable occurrences, 15 links
and 16 ordered published IDs were checked against the full raw and normalized
input. No source script, task scheduler, resource, code example, table or diagram
is executed or acquired. Variables use inline-code presentation, with exact token
text; original link labels and source-qualified destinations remain intact.

Existing WHATWG attribution and complete local license apply. This is an
additional partial rendering of the already cataloged full HTML source identity:
366 identities remain 257 full, 95 partial and 14 metadata-only; 81 passive SVG assets
and historical dated counts are unchanged. The
[source mapping](SOURCE-EDITIONS.md#fresh-standalone-eof-lifecycle-sources)
distinguishes the normative algorithm from the separately selected restricted
construction completion. Reference retention does not certify its implementation
or provide a browser event loop.

## ContrastColor source witnesses — 2026-10-13

The [combined WebKit reference](webkit-contrast-color--73aa6c89e2cb--6592a4862701.md)
retains three complete files at revision
`73aa6c89e2cb77c46184a81aec944e4ab99d114d`, rather than another standards
edition. Its three fenced bodies were checked byte-for-byte against the
already retained source files and their declared full-file SHA-256 values:
CSSContrastColorResolver.cpp `660886c44d4d6b1c6bc6e1dacedf733bf9eb596e0bb9cb123d34c2cffe3830c0`,
ColorLuminance.h `794f5f1a3edc3241422cc73fcb9d523be1ba613a03a1fcbb2829f465cc00201c`,
and ColorLuminance.cpp `915b775c2184892eeb4f4d93c86ca601648083e8709a15ab8c1ab6ad76cee722`.
The combined Markdown SHA-256 is
`6592a4862701d7056d12f2591c3cdc3151c7acbcd6e22d513afd7824fd4dc045`.
Only headings and provenance labels are added; no original body is changed or
omitted. Both distinct BSD notices have
[exact local header companions](../licenses/webkit/NOTICE.md#contrast-color-source-witnesses).

The [edition map](SOURCE-EDITIONS.md#html-color-delegation-and-intrinsic-contrast-policy)
records HTML877d's bounded delegation to the existing selected Color 4 and
the explicit #805 contrast policy, preserving normative versus browser versus
product distinctions. It does not implement the operation, select a new
standard edition or ingest browser code.

This addition takes the collection from 366 to 369 identities: 260 full,
95 partial and 14 metadata-only. Earlier dated addition counts and the 81
passive SVG figures remain unchanged. Whole witness bodies, legal companion
equality, UTF-8/LF/whitespace and affected local links/fragments were checked.
No source acquisition, browser-source execution or new media/diagram conversion occurred.

## ECMAScript Number serialization source retention — 2026-10-13

The [bounded reference](ecmascript-number-serialization--2026--cd65f2eaa201.md)
adds three partial original-page identities, taking the live collection from
369 (260 full/95 partial/14 metadata) to 372 (260 full/98 partial/14 metadata).
All earlier URL/hash pairs, historical counts and 81 passive SVG assets remain.

Complete §7.1.17 and §6.1.6.1.20, including Notes 1–3, are retained. The Number
introductory body ends after its Number-value-for-infinities paragraph, excluding
unselected arithmetic operations. §5.2.6 retains complete numeric-kind, suffix,
integer, operator, explicit-conversion, abs and interval paragraphs/lists;
unrelated mathematical functions are omitted explicitly. The exact front
title/date, complete About/Contributing block and introduction edition paragraph
supply provenance. The source literally says Draft ECMA-262 / March 31, 2026
and separately identifies the seventeenth edition; neither fact is normalized.

Original page hashes and byte lengths appear in the reference; its converted
Markdown hash is representation identity only. Ignored raw captures, selected
HTML, Pandoc preparation/body/render outputs and fidelity.json remain under
tmp/html-1134/ecma-number-sources. The existing approved Pandoc 3.1.11.1 and
semantic filter were used, with a bounded link-label adapter to preserve
variables inside links and prevent accidental nested URL autolinks.
Whitespace-only text comparison passes from original selection through prepared
HTML and rendered GFM. Superscript/subscript/variable text order, ordered-list
and item counts and resolved link-destination order are checked independently.
All source notes and nested formatting cases remain complete. The same checks
cover the complete original legal section; LICENSE-2026.txt preserves its exact
extracted text with only boundary/line-end whitespace cleanup.

The full alternative copyright license grants copying/modification/distribution
with the complete notice, disclaimers and modification notice retained. Those
and the software/BSD/patent caveats accompany the conversion in
[licenses/ecma262](../licenses/ecma262/NOTICE.md). No linked dtoa software or
script was downloaded or executed. Conversion checks establish bounded text
and structure fidelity, not formatter correctness, ECMA/HTML conformance,
visual equivalence or source-integrated implementation.

The converted reference SHA-256 is
`d9a172cec01000e5abcced21ee4f9bade1f9b6266eb59a5eb1877c54e1b91bf5`.
This representation hash does not replace any complete original-page hash.

## Intrinsic HTML supplements — 2026-10-13

The [nonce attributes](html-nonce--877d6146487f.md) and
[microdata encoding](html-microdata--877d6146487f.md) references are complete
heading-section extractions from the already retained HTML877d source, SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
Nonce ends before `lazy-loading-attributes`; microdata ends before `mdvocabs`.
No new edition, software or source scripts were acquired or executed.

The existing preparation/filter, approved Pandoc 3.1.11.1 and bundled parsed-GFM
renderer preserve exact text order, headings, IDs, inline code, variables,
literal examples and resolved links. Two nonce and five microdata MDN
compatibility annotations were omitted as presentation material. No normative
clause or source note is omitted. Whitespace-normalized rendered prose and the
independent structural comparisons passed without errors.

| Capture | Normalized extraction bytes / SHA-256 | Verified structure | Markdown SHA-256 |
| --- | --- | --- | --- |
| Nonce | 7,794 / `1ce61900b19cdbad79076014e3a03b4ec1401ceae153bdedd08f43adb594fb5a` | 40 IDs, 1 heading, 16 inline-code values, 28 variables, 39 links; no literal blocks or tables | `ec148b281a9e1c6270a9ab1b62a5857d37e708d5fb080b6fb7db2dc7fc82d19b` |
| Microdata | 42,687 / `35d3d012166bfb6f977e81a154a28259467433df799b1647e3c9f7ef19abb2f0` | 209 IDs, 7 headings, 87 inline-code values, 28 variables, 191 links, 9 literal blocks; no tables | `ae6859843e3d08b7258f0d8c2e58380dd945b41a6c382fd726d75f38a16c6418` |

The original, bounded extraction, preparation, rendered GFM and verification
records are retained under `tmp/html-final-integration/reference-retention/`
while the contribution's review consumes them. The exact existing
[WHATWG terms and supplemental attribution](../licenses/whatwg/NOTICE.md#intrinsic-html-source-supplements--2026-10-13)
remain linked. These checks establish source fidelity and navigation, not
implementation correctness, browser behavior, external fact truth or conformance.

## Hosted Document activity source retention — 2026-10-15

The [bounded reference](html-hosted-document-activity--877d6146487f--f8564ea4758e.md)
retains complete selected clauses from the already retained HTML revision
`877d6146487f15b15bc8dd0e3c625063a37b456a`: active Document at raw70877–70880
with its70876 opening algorithm wrapper; Document browsing-context definition
and complete createDocument/removed-iframe note71636–71642; fully-active heading/
definition and child-navigable interaction restriction71919–71932. Surrounding
chapters and the following example remain outside the bounded selection.

Complete original: 14,930,614 bytes, SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
The three original fragments, preserving their LF endings and joined by one
extra LF between fragments, have SHA-256
`f8564ea4758e8af4bdd7ee661325dfe6565013d25796121e84e924efe8e225eb`.
This bounded-input hash supplies the filename suffix. Final Markdown SHA-256 is
`81ea7b382ff9b9f6137c4185afd3e6c0bedc8ffe69b9a76ab733a6798fe1bb99`;
it identifies the representation, not an original page or different edition.
Existing original-source identities and historical counts are unchanged.

Conversion on 2026-10-15 uses literal semantic HTML inside Markdown, adding only
editorial headings/provenance and rebasing internal fragment hrefs to the same
immutable snapshot. Exact reversal of that href transformation recovers every
selected body byte. Independent HTML-parser token comparison preserves text,
links, IDs, code/variables/definitions, algorithm wrappers and note class.
No words or operative branches are repaired or paraphrased. All four owning
Markdown documents passed validation and LF/trailing-whitespace checks; local
license and attribution links resolve. No software/source acquisition, source scripts,
browser implementation, Cargo or documentation build is involved.

The existing exact WHATWG LICENSE hash was checked. Copyright, CC BY 4.0 prose
terms, BSD source-code distinction and disclosed changes are recorded in the
[supplemental notice](../licenses/whatwg/NOTICE.md#hosted-document-activity-clauses--2026-10-15)
and reference. These checks establish bounded source fidelity and navigation,
not hosted lifecycle implementation or conformance.
