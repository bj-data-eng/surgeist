# Consolidated specification references

The [source catalog](SOURCE-CATALOG.md) links all 150 retained source snapshots and states their representations: 148 full documents/source witnesses and two metadata-only exceptions. Read the [conversion report](CONVERSION-REPORT.md) for checks, link policy and explicit limitations.

The references are format conversions, not summaries or replacement standards. Distinct specification editions remain separate. Eight redundant plain-text extractions were removed after comparison with the exact HTML editions already represented by structured Markdown. Source copyright/licensing notices, normative/informative classifications, literal code, definitions and published fragment IDs are retained where present.

## Source editions

[Source editions and bounded imports](SOURCE-EDITIONS.md) separates the pinned Snapshot bibliography from later implementation-selected editions and importer-specific witnesses. No implementation behavior or source-selection decision is changed by adding these documents.

## Coverage exceptions

Unicode Text Segmentation revision 47 is represented by a URL/hash/licensing entry only. Its full report is excluded from public distribution because the report's terms require permission. All links into its normative text remain external. This gap is explicit; the entry does not claim full public text coverage.

The separate [Unicode 17 grapheme implementation companion](unicode-17-grapheme-implementation-companion.md) includes licensed rule data, exact provenance and the complete Unicode License v3. Its engine-dependent notation and implicit start/end/default rules are explained in the companion. The official UAX 29 revision 47 remains the normative authority.

The [WebKit CSSProperties witness](webkit-cssproperties--73aa6c89e2cb--848d6e24bbd4.md) is also metadata-only because its file-specific redistribution terms were not established. Its pinned URL/hash are retained without redistributing its source.

## Supporting resources

The assets subdirectory contains 81 passive SVG figures: 53 extracted from the exact stored HTML sources and 28 railroad diagrams generated from the pinned CSS Syntax 3 Bikeshed source. The original railroad grammars remain beside the generated diagrams. Other source images, PNG equations, videos and interactive figures remain external resources. The bundle is not fully self-contained for those figures.

Six pinned Bikeshed sources are presented as readable, generated Markdown renderings, with the original source hashes and compiler/support-data provenance retained. They are not official publications or captured historical HTML renderings. The separate JSON source-history capture remains a supporting provenance witness.

## Link policy

An original fragment-only self-link stays local only when the original HTML base identifies that same captured source body and the exact emitted anchor exists. Cross-document local links require an audited exact snapshot identity and an existing emitted fragment. This creates no global alias for an undated/editor URL. Absolute undated/editor/latest links, genuinely absent fragments, and normative UAX 29 citations remain upstream.

## Legacy navigation

The [Nesting preview guide](push-test.md) preserves the earlier preview address and points to the canonical converted snapshot. It is tracked as a navigation guide, separately from source entries.
