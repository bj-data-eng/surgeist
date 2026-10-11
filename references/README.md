# Consolidated specification references

The [source catalog](SOURCE-CATALOG.md) links all 347 retained source identities and states their representations: 254 full documents/source witnesses, 88 partial witnesses, and 5 metadata-only records. Read the [conversion report](CONVERSION-REPORT.md) for checks, link policy and explicit limitations.

The references are format conversions, not summaries or replacement standards. Distinct specification editions remain separate. Eight redundant plain-text extractions were removed after comparison with the exact HTML editions already represented by structured Markdown. Source copyright/licensing notices, normative/informative classifications, literal code, definitions and published fragment IDs are retained where present.

## Source editions

[Source editions and bounded imports](SOURCE-EDITIONS.md) separates the pinned Snapshot bibliography from later implementation-selected editions and importer-specific witnesses. Source-selection boundaries are recorded there; captures provide evidence without implementing the selected behavior.

The [CSSOM planning sources](SOURCE-EDITIONS.md#cssom-planning-sources) select complete current CSSOM, Counter Styles 3, and Conditional 5 captures for the new CSSOM work. Historical editions remain available for completed CSS consumers; each planning contribution uses one operative edition.

The [bounded Page3 supplement](SOURCE-EDITIONS.md#bounded-page-page-orientation-supplement) retains complete §1.1 and §7.1.2 from one hashed editor capture for the named `page-orientation` grammar. It explicitly has partial document coverage and adds no font-model selection; other Page/margin domains retain their own requirements and source selections. The historical Page3 witness remains dated evidence, and the adopted Blink Page/margin serialization format is unchanged.

## Text, font and effects sources

The [text, font and effects edition mapping](SOURCE-EDITIONS.md#text-font-and-effects-source-resolution) identifies five partial current-draft captures, pinned WebKit/Blink evidence and the installed Parley/provider source excerpts. Each partial witness names its complete sections or exact line ranges and identifies the full original source by hash. These ten reference files add 44 source identities without replacing historical editions.

Standards gaps remain visible in the captures. Browser algorithms supply bounded implementation evidence. The [seven-issue source-decision mapping](SOURCE-EDITIONS.md#operative-source-decisions-by-issue) links the owning feature determinations and distinguishes normative interpretation, pinned browser compatibility and authorized product policy. The [text/layout contract](../docs/text-layout-contract.md) separately owns the shared preparation, metric and admission mechanisms. Reference retention does not claim that the planned text, layout or effects behavior has shipped.

## Coverage exceptions

Unicode Text Segmentation revision 47 is represented by a URL/hash/licensing entry only. Its full report is excluded from public distribution because the report's terms require permission. All links into its normative text remain external. This gap is explicit; the entry does not claim full public text coverage.

The separate [Unicode 17 grapheme implementation companion](unicode-17-grapheme-implementation-companion.md) includes licensed rule data, exact provenance and the complete Unicode License v3. Its engine-dependent notation and implicit start/end/default rules are explained in the companion. The official UAX 29 revision 47 remains the normative authority.

The [WebKit CSSProperties witness](webkit-cssproperties--73aa6c89e2cb--848d6e24bbd4.md) is also metadata-only because its file-specific redistribution terms were not established. Its pinned URL/hash are retained without redistributing its source.

The [WebKit container-query source witness](webkit-cssom-container-query--73aa6c89e2cb.md) retains four complete files with their original two-clause license notices. Its bounded implementation evidence is distinct from normative specification selections and from the earlier metadata-only CSSProperties witness.

The [WebKit core CSSOM witness](webkit-cssom-core-rules--73aa6c89e2cb.md) and [Blink Page/margin witness](blink-cssom-page-rules--7984f9d11800.md) retain the eleven additional complete files used for bounded planning evidence, with their file-specific LGPL or BSD legal material.

The [WebKit wrapper and dispatch witness](webkit-cssom-wrapper-rules--73aa6c89e2cb.md) and [Blink wrapper deficiency witness](blink-cssom-wrapper-gaps--7984f9d11800.md) retain twenty additional complete files and four newer WebKit identities whose bytes equal retained preferred-revision bodies. They document wrapper branches, getter/storage deficiencies, and bounded dispatch absence without selecting product behavior.

## Supporting resources

The assets subdirectory contains 81 passive SVG figures: 53 extracted from the exact stored HTML sources and 28 railroad diagrams generated from the pinned CSS Syntax 3 Bikeshed source. The original railroad grammars remain beside the generated diagrams. Other source images, PNG equations, videos and interactive figures remain external resources. The bundle is not fully self-contained for those figures. The pinned HTML embedded-content capture additionally preserves six inline SVG diagrams as semantic source markup; these are separate from the 81 asset files.

Six pinned Bikeshed sources are presented as readable, generated Markdown renderings, with the original source hashes and compiler/support-data provenance retained. They are not official publications or captured historical HTML renderings. The separate JSON source-history capture remains a supporting provenance witness.

## Link policy

An original fragment-only self-link stays local only when the original HTML base identifies that same captured source body and the exact emitted anchor exists. Cross-document local links require an audited exact snapshot identity and an existing emitted fragment. This creates no global alias for an undated/editor URL. Absolute undated/editor/latest links, genuinely absent fragments, and normative UAX 29 citations remain upstream.

## Legacy navigation

The [Nesting preview guide](push-test.md) preserves the earlier preview address and points to the canonical converted snapshot. It is tracked as a navigation guide, separately from source entries.

## Layout source-resolution witnesses

The [layout sizing, fragmentation and overflow edition mapping](SOURCE-EDITIONS.md#layout-sizing-fragmentation-and-overflow-source-resolution) selects twelve additional references for tables, flex, grid lanes, fragmentation, generated-content leaders and overflow geometry. Six new standards identities have partial clause coverage; browser witnesses distinguish seven complete files from nine partial identities, including preferred and newer FlexFormattingContext.cpp revisions with identical retained segments but different complete-file hashes. The catalog also links complementary Grid 2 and TransformationMatrix.cpp witnesses under their existing identities. Historical references and completed acceptance remain intact. Source evidence, browser behavior and Surgeist policy are stated separately; retaining these witnesses does not implement the selected contracts.

## HTML parser research

The [parser suitability assessment](../docs/html-parser-suitability.md) uses pinned html5ever and html5gum implementation witnesses, listed in the [catalog](SOURCE-CATALOG.md#html-parser-research-sources). These retain selected complete files and bounded source excerpts, not complete repository copies.

## DOM, selectors and bounded style determinations

The [new operative edition mapping](SOURCE-EDITIONS.md#dom-selector-execution-and-bounded-style-determinations) identifies current DOM/HTML/Web IDL/Infra foundation clauses, selector and scope execution sources, and bounded style gap determinations. Native Rust identity, admission, atomicity and host contracts remain explicit product choices. Historical authored-CSS editions are retained; a current capture does not silently replace every consumer’s source selection.

The [addition verification record](CONVERSION-REPORT.md#dom-selector-execution-and-bounded-style-source-additions--2026-10-1011) distinguishes complete documents, complete selected sections, exact browser excerpts and brief resolution quotations. Complex DOM tables preserve semantic HTML rather than flattening. Conditional 5 and Content 3 renderings share complete original source identities; the Blink speak entry shares its existing complete registry witness. Dated CSSWG minutes are an ordinary metadata-only corroborating record, separate from the two rights/coverage exceptions above.

## Additional source questions

The [additional edition mapping](SOURCE-EDITIONS.md#additional-source-question-determinations) retains bounded color/profile, animation/keyframe, generalized-conditional, marker/pseudo-object, bookmark/accessibility and assigned media/preference/border evidence. Reused full source identities gain representation links without duplicate counting. Current published AccName 1.2 and separately pinned browser evidence remain explicitly scoped. The [verification record](CONVERSION-REPORT.md#additional-source-question-references--2026-10-1011) states the conversion checks and metadata-only discussion provenance.

## Pinned HTML language sources

The [dimension-list cursor witness](html-dimension-list-cursor--webkit-73aa6c89e2cb-blink-7984f9d11800.md)
retains bounded WebKit and Blink excerpts, complete original file headers and
full-source hashes for the narrow decimal-cursor determination under #1133.
Its selected correction and unadopted browser differences remain explicit in
the [edition map](SOURCE-EDITIONS.md#pinned-html-language-baseline).

The [HTML language edition map](SOURCE-EDITIONS.md#pinned-html-language-baseline) selects one immutable WHATWG HTML snapshot, split into ten complete-section captures covering all chapter 4 element definitions and the retained language/parser/host clauses. The [supplemental html5ever witness](html5ever-parser-supplemental-contracts--d7232d746d31.md) preserves exact char-reference, tree-builder and buffer excerpts at the independently selected ingestion pin. Historical live HTML captures remain available as proof for their original determinations. Selection does not assert that the current engine already implements the newer standard.
