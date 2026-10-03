# Reference conversion and validation

## Scope and result

- 129 original stored files, plus five exact acquired snapshots and one explicitly pinned supporting JSON capture
- 138 materialized source-file witnesses, deduplicated to 134 exact byte snapshots; four additional paths are exact-copy aliases
- 120 HTML snapshots, 5 uncompiled Bikeshed snapshots, 8 extracted-text witnesses, and 1 JSON acquisition-chain witness
- All 134 private conversions pass the final automated gates; failed intermediate candidates were quarantined and do not define acceptance
- Public tree: 133 full source conversions/witnesses and one Unicode identity-only exception, plus 52 passive SVG diagram assets
- Original inputs remain unchanged. Source scripts/JavaScript were never executed. No remote publication was performed by the converter

## Fidelity checks

The final checks cover 388,510 source text nodes, 6,204 headings, 7,803 explicit definitions, 3,005 literal blocks, 8,180 inline-code runs, 8,930 semantic subscript/superscript/variable nodes, 1,057 source tables, 26,162 original cells, and 85,828 retained source anchor occurrences.

All source characters are preserved in the actual GFM AST under whitespace-only prose comparison. Code blocks preserve tabs, escapes, leading text and block boundaries; only the final block newline is excluded from equality checks. Inline HTML code is checked under HTML-collapsible whitespace normalization only, with exact characters/operators preserved; independent review confirmed the remaining multiline formula/grammar differences were source-rendering whitespace, not token loss. Script/variable structure and multiplicity are checked independently. Table row/column/header/span models are retained in the verification record. Full paragraph/list-entry comparisons are also reported; explicit conversion captions/structure labels can interrupt enclosing entries, so those long-entry checks are diagnostic rather than blanket acceptance gates. No Unicode normalization or punctuation folding is used to hide differences.

Pandoc's HTML writer adds text-presentation variation selectors to some arrows; the emitted Markdown AST preserves the exact source arrow. Character checks therefore inspect the actual AST, with rendered HTML checks separately covering code, anchors, scripts/variables and structures.

## Readable representations and explicit limitations

- Source Note, Example, Advisement, Issue and Warning classes become visible labels, including standalone examples and unresolved algorithm-list issues
- Complex/multi-paragraph tables are structured Markdown cell blocks with original row/column coordinates, header/data roles, scope, and row/column spans; no raw HTML tables remain
- 52 inline SVG diagrams retain geometry, internal IDs, source text labels, exact-case SVG attributes and required passive source railroad CSS. XML/geometry/CSS checks pass; full visual equivalence is not certified
- All 17 MathML expressions have checked portable fenced TeX transcriptions. Independent round trips preserve mathematical tokens, matrix dimensions/ordering and role-preserving script/fraction/root/limit structures. Exact MathML remains in private verification records; visual equivalence is not certified
- Small semantic inline HTML anchors/emphasis/variables/subscripts/superscripts are retained where GFM otherwise corrupts delimiter or notation meaning. They are content semantics, not original website layout
- Escape-sensitive characters and the unstable Greek question-mark/combining-mark cases use numeric entities where needed. Adjacent literal code, code in tables and quoted/URL-valued link labels are explicitly handled
- The five Bikeshed inputs remain clearly labelled uncompiled source witnesses in exact fenced blocks. They are not falsely presented as newly generated published specifications
- Eight extracted-text witnesses and the JSON capture preserve their complete stored text in fences. Missing source HTML structure is not reconstructed or invented
- Existing images, object-image fallbacks, videos and interactive figures retain resolved upstream resource links and available source descriptions. External assets are not downloaded or availability-tested. SVG 1.1 Implementation Notes has 14 PNG-only equations whose source images remain essential; their math is not invented from equation-number alt text
- UAX29 revision 47 has a checked private conversion but only the URL/hash/classification/licensing entry is public, due to its report-specific redistribution restriction
- Optional HTML link hover titles are omitted to avoid Pandoc's broken unescaped-quote title serialization; visible citation titles and original hrefs remain. Exact original href/title inventories are retained separately
- Checks establish structural/textual preservation at the stated scope, not standards conformance, mathematical proof, complete visual equivalence, or external link/asset availability

## Size

Unique source snapshots: 42,259,039 bytes. Checked private Markdown: 21,307,679 bytes. Public Markdown reference entries after cross-link rewriting: 21,108,098 bytes. These are measured UTF-8 byte counts, not measured model-token counts. Public/private differences include the Unicode licensing exception.

## Source identity and link policy

See [Source catalog](SOURCE-CATALOG.md). Exact byte copies share one output and are recorded as aliases; differing hashes/editions remain distinct. Trusted catalog/hash identities determine canonical paths. Fragment-only self-links are local only when source provenance/base and retained target anchors prove that identity. Cross-document links may localize only when an audited exact source identity and an emitted anchor both exist. Undated/editor/latest/bibliographic or absent-fragment targets remain upstream.

The completed cross-link pass localized 3,053 exact destination occurrences across 42 reference entries. All 134 entry ASTs remained identical except link destinations. It verified 45,505 local link occurrences and 52 local image occurrences, including SVG href/xlink fragments, with zero broken local targets and zero malformed external destinations. All expected localization-eligible bodies were present. Ten occurrences (nine distinct URLs) with genuinely absent source fragments remain external; no invented anchors were added. Source provenance, external bibliography/current-version links, and normative Unicode citations remain upstream. External network availability was not tested.

## Reproducibility

The offline converter is convert_collection.py, with semantic_boxes.lua and svg_support.py; run_collection.py consumes the audited source-identity map. The environment uses Pandoc 3.1.11.1, Python 3.12 and lxml. Per-document cleaned/rendered intermediates, exact source hashes, tables, anchors, structure checks and diagnostics are retained in the local work directory. Reproduction requires the same source bytes and identity map, not a live-page substitution. The original stored snapshots and acquired pins are hash-verified.

## Informative Unicode companion and legacy navigation

The bundle also includes a separately licensed Unicode 17 grapheme implementation rule-data companion, with immutable data/source hashes and the complete Unicode License v3. Its source-data excerpt is exact, but its engine-dependent notation and implicit start/end/default behavior are explicitly disclosed; it does not replace or reproduce the normative UAX 29 report. This informative companion is separate from the 134 source snapshot entries. A small `push-test.md` navigation guide preserves the earlier Nesting preview address without duplicating the complete conversion.
