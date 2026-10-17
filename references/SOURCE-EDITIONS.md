# CSS source editions and bounded imports

The reference collection records the full selected source scope of [CSS Snapshot 2026, 22 June 2026](https://www.w3.org/TR/2026/NOTE-css-2026-20260622/): all four module tiers and its section 4 features. The Snapshot bibliography is the reproducible audit baseline. Existing implementation references and required imports are retained alongside it. Adding a faithful source copy does not upgrade scope, implement missing grammar, or establish conformance.

This mapping describes the repository at `104bff510075d1acb59c23bb9491c7a368b7d05a`. The [catalog](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/specs/catalog.json) selects editions under its existing 2026-09-10 publication-cutoff policy; [conformance metadata](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs) records source attribution and declared subsets. They are separate evidence: catalog membership and a registry entry do not prove implementation. The full pinned Snapshot scope remains the target even where the catalog or implementation has no corresponding production. The [source catalog](SOURCE-CATALOG.md) identifies each retained text or metadata-only representation.

## Edition differences

Rows below distinguish the Snapshot bibliography from current selected/provenance editions. Unlisted same-edition documents retain their exact identities; overlapping definitions and feature-scoped imports still need their own boundaries.

| Source | Snapshot baseline | Existing selected edition | Implementation provenance and evidence |
| --- | --- | --- | --- |
| UI 3 | [REC-css-ui-3-20260407](https://www.w3.org/TR/2026/REC-css-ui-3-20260407/); [local copy](css-ui-3--REC-css-ui-3-20260407--6dfbf85a9d9c.md) | [REC-css-ui-3-20260407](https://www.w3.org/TR/2026/REC-css-ui-3-20260407/); [local copy](css-ui-3--REC-css-ui-3-20260407--6dfbf85a9d9c.md) | O_UI3 records the selected 2026 REC in registry, obsolete-property, downstream-behavior and informative-audit metadata. Its [§6.3.1](https://www.w3.org/TR/2026/REC-css-ui-3-20260407/#input-method-editor) obsoletes `ime-mode`; the [2018 REC local copy](css-ui-3--REC-css-ui-3-20180621--7f8a182b5285.md#input-method-editor) remains a separately identified historical witness. UI property dispatch retains its existing UI4 and shared-module owners; source attribution does not change accepted authored grammar or canonical order. [source](../crates/surgeist-css/src/conformance.rs), [implemented reference](../crates/surgeist-css/docs/reference.md#remaining-authored-ui4-values).
| Display 3 | [CR-css-display-3-20230330](https://www.w3.org/TR/2023/CR-css-display-3-20230330/); [local copy](css-display-3--CR-css-display-3-20230330--e4e7ee86ee9b.md) | [CRD-css-display-3-20260605](https://www.w3.org/TR/2026/CRD-css-display-3-20260605/); [local copy](css-display-3--CRD-css-display-3-20260605--872cdd7c49fd.md) | S_DISPLAY3 and the catalog select 2026-06-05; display/visibility dispatch names that source. [source](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L896-L904), [dispatch](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L2380). |
| Sizing 3 | [WD-css-sizing-3-20211217](https://www.w3.org/TR/2021/WD-css-sizing-3-20211217/); [local copy](css-sizing-3--WD-css-sizing-3-20211217--88e7fb4fefc1.md) | [WD-css-sizing-3-20260904](https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/); [local copy](css-sizing-3--WD-css-sizing-3-20260904--ca0f1db3d5ad.md) | I_SIZING3_20260904 owns sizing-property provenance; the 2021 I_SIZING3 record is retained without a live feature use. [dispatch](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L2419-L2433). |
| Sizing 4 | [WD-css-sizing-4-20210520](https://www.w3.org/TR/2021/WD-css-sizing-4-20210520/); [local copy](css-sizing-4--WD-css-sizing-4-20210520--febb0814cb6d.md) | [WD-css-sizing-4-20260904](https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/); [local copy](css-sizing-4--WD-css-sizing-4-20260904--faaa86589559.md) | Snapshot section 4 selects named features; the existing catalog separately selects the extension. Aspect-ratio and newer sizing provenance use X_SIZING4_20260904; the 2021 source is retained. [dispatch](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L2434-L2441), [aspect-ratio](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L2712). |
| Color 4 | [CRD-css-color-4-20260502](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/); [local copy](css-color-4--CRD-css-color-4-20260502--cfd58f190b6f.md) | [CRD-css-color-4-20260908](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/); [local copy](css-color-4--CRD-css-color-4-20260908--bada647312a7.md) | O_COLOR4 and the catalog select 2026-09-08. Retain May as the audit baseline. [source](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L734-L741). |
| Color 5 | [WD-css-color-5-20260507](https://www.w3.org/TR/2026/WD-css-color-5-20260507/); [local copy](css-color-5--WD-css-color-5-20260507--08bdb8062820.md) | [WD-css-color-5-20260908](https://www.w3.org/TR/2026/WD-css-color-5-20260908/); [local copy](css-color-5--WD-css-color-5-20260908--f749cb75ec1c.md) | The catalog selects September; live feature records retain both June 18 (relative colors/mix) and September 8 (newer definitions). Neither older witness is an alias for September. [feature provenance](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L3484-L3584). |
| Fonts 4 | [WD-css-fonts-4-20260422](https://www.w3.org/TR/2026/WD-css-fonts-4-20260422/); [local copy](css-fonts-4--WD-css-fonts-4-20260422--fbcd4e298b4c.md) | [WD-css-fonts-4-20260907](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/); [local copy](css-fonts-4--WD-css-fonts-4-20260907--03626a0c8565.md) | Most font-property dispatch, including font-palette, names I_FONTS4_20260907; I_FONTS4 still occurs in a descriptor record. Preserve both editions. [dispatch](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L2565-L2591), [older record](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L4389-L4400). |
| Text 3 | [CRD-css-text-3-20240930](https://www.w3.org/TR/2024/CRD-css-text-3-20240930/); [local copy](css-text-3--CRD-css-text-3-20240930--61d133c34716.md) | [CRD-css-text-3-20260814](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/); [local copy](css-text-3--CRD-css-text-3-20260814--17c26f1b4145.md) | S_TEXT3 and the catalog select August 2026; overflow-wrap dispatch names S_TEXT3. [source](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L974-L982), [dispatch](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L2642). |
| Text 4 | [WD-css-text-4-20240529](https://www.w3.org/TR/2024/WD-css-text-4-20240529/); [local copy](css-text-4--WD-css-text-4-20240529--be6e92f95bd4.md) | [WD-css-text-4-20260814](https://www.w3.org/TR/2026/WD-css-text-4-20260814/); [local copy](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md) | The Snapshot section 4 edition and current extension edition remain distinct; X_TEXT4 names August 2026. [dispatch](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L2643-L2675). |

Three implemented choices must remain visible beside those edition rows:

- `palette-mix()` follows [Fonts 4, 2026-09-07 §9.1](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-palette-prop), beyond the April Snapshot grammar. [The implemented reference](../crates/surgeist-css/docs/reference.md#authored-font-palette) records the same edition.
- Extended `text-shadow` is attributed to [Text Decoration 4, 2022-05-04](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/#text-shadow-property), alongside the Snapshot's Text Decoration 3 baseline. [The dispatch](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L2676-L2694) and [parser](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/parser/text_decoration.rs#L213-L239) are evidence of this module difference.
- `grid-auto-flow` has a symbolic `normal` initial rather than the frozen [Grid 2 `row` initial](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#grid-auto-flow-property). The existing [implementation reference](../crates/surgeist-css/docs/reference.md#authored-grid-repetition-and-keyframe-structure) expressly cites [frozen WebKit](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSProperties.json#L11479) and the unresolved [Grid 3 orientation issue](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#grid-lanes-orientation). Keep Grid 1/2 baseline, Grid 3 source, and browser witness separate; this is a recorded product/source difference.

## Importer-selected witnesses

These documents support reachable definitions, not new whole-module selections. The importer bibliography establishes the dated witness; any existing catalog-selected replacement is identified separately.

| Importer | Exact imported witness / retained copy | Bounded definition and reason |
| --- | --- | --- |
| [Transitions 1, 2026-01-08](https://www.w3.org/TR/2026/WD-css-transitions-1-20260108/#biblio-css-easing-2) | [WD-css-easing-2-20240829](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/); [local copy](css-easing-2--WD-css-easing-2-20240829--570d12249ade.md) | `<easing-function>`, including [linear()](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/#linear-easing-function). The source import remains required even though the separate static audit found a missing authored linear() branch. |
| [Font Loading 3, 2023-04-06](https://www.w3.org/TR/2023/WD-css-font-loading-3-20230406/#biblio-css-fonts-5) | [WD-css-fonts-5-20211221](https://www.w3.org/TR/2021/WD-css-fonts-5-20211221/); [local copy](css-fonts-5--WD-css-fonts-5-20211221--6dfbf10b25f1.md) | Referenced font-face/src/metrics definitions only. Keep its printed `techn()`/feature spellings and removed variant-descriptor conflict visible beside newer Fonts 4; a source copy does not reconcile those conflicts. |
| [Motion 1, 2024-11-05](https://www.w3.org/TR/2024/WD-motion-1-20241105/#biblio-css-values-5) | [WD-css-values-5-20240917](https://www.w3.org/TR/2024/WD-css-values-5-20240917/); [local copy](css-values-5--WD-css-values-5-20240917--9b60eb26cb8f.md) | The imported [position](https://www.w3.org/TR/2024/WD-css-values-5-20240917/#position) production. Later [Images 4](https://www.w3.org/TR/2025/WD-css-images-4-20250930/#biblio-css-values-5) and the existing catalog use November 11 for their own bounded Values 5 definitions; do not collapse the September witness into it. |
| [Lists 3, 2020-11-17](https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#biblio-css-content-3) | [WD-css-content-3-20190802](https://www.w3.org/TR/2019/WD-css-content-3-20190802/); [local copy](css-content-3--WD-css-content-3-20190802--58d280c1fc9f.md) | Referenced [content-list](https://www.w3.org/TR/2019/WD-css-content-3-20190802/#typedef-content-content-list) definitions; distinct from the existing selected [Content 3, 2025-12-04](https://www.w3.org/TR/2025/WD-css-content-3-20251204/). |
| [Lists 3](https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#biblio-css-values-4) | [WD-css-values-4-20190131](https://www.w3.org/TR/2019/WD-css-values-4-20190131/); [local copy](css-values-4--WD-css-values-4-20190131--403391ef96a7.md) | Required value-definition/comma-combinator witness. [§2.1](https://www.w3.org/TR/2019/WD-css-values-4-20190131/#comb-comma) requires omission of the comma when its following optional value is omitted. |
| [Counter Styles 3](https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/#biblio-css-values-4) | [WD-css-values-4-20210715](https://www.w3.org/TR/2021/WD-css-values-4-20210715/); [local copy](css-values-4--WD-css-values-4-20210715--23e1c33c648c.md) | Importer-specific Values 4 witness for applicable numeric/grammar definitions, including [integers](https://www.w3.org/TR/2021/WD-css-values-4-20210715/#integers); not a full Values 4 scope expansion. |
| [Font Loading 3](https://www.w3.org/TR/2023/WD-css-font-loading-3-20230406/#biblio-css-values-4) | [WD-css-values-4-20221019](https://www.w3.org/TR/2022/WD-css-values-4-20221019/); [local copy](css-values-4--WD-css-values-4-20221019--b7ccf53eec03.md) | Importer-specific Values 4 witness. The current catalog selects [2024-03-12](https://www.w3.org/TR/2024/WD-css-values-4-20240312/) separately; retain the historical import identity. |
| [Cascade 4](https://www.w3.org/TR/2022/CR-css-cascade-4-20220113/#informative) / [Cascade 5](https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#informative) | [WD-css-page-3-20181018](https://www.w3.org/TR/2018/WD-css-page-3-20181018/); [local copy](css-page-3--WD-css-page-3-20181018--0c654ec792d0.md) | Informative referral for Page-context behavior, not a normative whole-Page3 import or authority to supersede CSS2's explicit Page `em`/`ex` restriction. The bibliographies name October 18, 2018. Values 3/4 separately refer to [Page 3, 2023-09-14](https://www.w3.org/TR/2023/WD-css-page-3-20230914/) for bounded paged viewport/page-area definitions; that dependency does not select its complete Page font model. |

Other existing bounded imports are recorded by the [catalog](../crates/surgeist-css/specs/catalog.json) and [implemented reference](../crates/surgeist-css/docs/reference.md): UI 4 cursor→Images 4 image-set; Shapes→SVG 1.1 path grammar/parsing and Values 5 position; selector categories→Pseudo/Scoping/Shadow/Shadow Parts; Nesting/Conditional→pinned Syntax block-contents and Scope definitions; Values 4→Typed OM numeric typing; Content→Values 5 attr(); Conditional→Values 5 tree-counting and extension-name; Sizing→Values 5 calc-size(); Overflow/Position→Box 4 visual-box/four-side rules; Color 5→pinned Values 5 mix normalization. Preserve their named fragments and immutable witnesses. None selects every property in its defining document. Repeating-conic is not added merely because conic/interpolation definitions are selected.

## CSSOM planning sources

The new CSSOM work selects the following complete captures as its operative editions. Each contribution has one selected edition, rather than a historical edition plus correction excerpts. Retrieval hashes identify the exact captured HTML; the pages declare revision `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`. These are captured current drafts, not immutable dated publications or regenerated source pins.

| Contribution | Operative planning edition | Retrieval identity |
| --- | --- | --- |
| CSS Object Model (CSSOM) Module Level 1 | [Editor’s Draft, 31 August 2026](cssom-1--editor-capture-20261009--d42e145ec395.md) | 2026-10-09; SHA-256 `d42e145ec395103f592001e5683cbeb63084ea530b76ec2923ecbcf80c7e74f9` |
| CSS Counter Styles Module Level 3 | [Editor’s Draft,
    8 October 2026](css-counter-styles-3--editor-capture-20261009--ef29d0a06a13.md) | 2026-10-09; SHA-256 `ef29d0a06a13683cbb3244bebc6f36d359f62fa107c17f1250111c9b0e4b0f43` |
| CSS Conditional Rules Module Level 5 | [W3C Working Draft,
    8 October 2026](css-conditional-5--editor-capture-20261009--75e15be8c8b4.md) | 2026-10-09; SHA-256 `75e15be8c8b41c6c34de2fa861801dbc6b77a06e6aba79d397bad82608e0aac1` |

The Conditional 5 page advertises a dated 8 October 2026 Working Draft. That dated TR URL returned HTTP 404 during acquisition, so the retained reference identifies the full drafts URL capture rather than claiming dated-publication bytes.

The [CSSOM 2021](cssom-1--WD-cssom-1-20210826--24d77393e4ea.md), [Counter Styles 2021](css-counter-styles-3--CR-css-counter-styles-3-20210727--7d697581c3eb.md), and [Conditional 5 2025](css-conditional-5--WD-css-conditional-5-20251030--272548f04685.md) copies remain historical sources for completed CSS work. The [implemented CSS reference](../crates/surgeist-css/docs/reference.md) and [source conformance records](../crates/surgeist-css/src/conformance.rs) still consume those editions. Planning selection does not rewrite completed implementation provenance or assert implementation conformance to the newer editions.

### Bounded Page page-orientation supplement

For the Page CSSOM contribution, the [bounded Page 3 witness](css-page-3-page-orientation--editor-capture-20261010--48ec99cd3a58.md) selects only the authored `page-orientation` descriptor grammar in §7.1.2 and §1.1’s common CSS-wide acceptance statement. The Editor’s Draft displays 3 March 2026; the HTML was captured on 10 October 2026 from `https://drafts.csswg.org/css-page-3/`, SHA-256 `48ec99cd3a581f90b588c6d578be8fd97102c01d6482b1d75ebf39108924dbfb`, 407,670 bytes, declaring source revision `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`. The witness retains both selected sections in full and explicitly has partial document coverage. The grammar is `upright | rotate-left | rotate-right`, with initial `upright`; §1.1 supplies common CSS-wide acceptance without repeating the keywords in the descriptor definition.

This is a named supplement for that descriptor, not a whole-Page3 edition selection. Keep the [18 October 2018 Page3 historical witness](css-page-3--WD-css-page-3-20181018--0c654ec792d0.md) as dated evidence and preserve the adopted [Blink Page/margin serialization evidence](blink-cssom-page-rules--7984f9d11800.md). This supplement alone adds no font-model selection; other Page/margin domains retain their own requirements and source selections. Page rotation, layout and painting remain downstream.

The [WebKit container-query source witness](webkit-cssom-container-query--73aa6c89e2cb.md) supplies separate, bounded implementation evidence for omitted-query serialization where the selected Conditional 5 prose lacks the complete rule. Its four complete files are identified by immutable revision and per-file hashes. This browser-source evidence is explicitly distinct from the operative normative edition.

## Retained registry history and representation limits

### Existing registry witnesses

At this revision, `R_MEDIA4`, `R_GRID1`, `S_TEXTDECOR3`, `I_SIZING3`, `X_SIZING4`, `X_CONTAIN3`, `X_BORDERS4_20251216`, and `X_FULLSCREEN` occur only in their source declaration and registry. They retain source history; where they are Snapshot members, membership also remains intact. They do not establish live consumption. [The declaration/registry evidence](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L831-L1408) is separate from current feature records and catalog supersession.

The old `X_VALUES4` repository pin is different: [ext.supports.general-enclosed still cites it](https://github.com/bj-data-eng/surgeist/blob/104bff510075d1acb59c23bb9491c7a368b7d05a/crates/surgeist-css/src/conformance.rs#L7489-L7495), while the catalog selects dated Values 4. Preserve [commit 720ea286… source identity](https://github.com/w3c/csswg-drafts/blob/720ea2863696971ea6a6744e0f23acbb3e6936bd/css-values-4/Overview.bs) as a legacy live provenance witness. The [readable generated source rendering](css-values-4--720ea2863696971ea6a6744e0f23acbb3e6936bd--Overview.bs--593d3bb00def.md) preserves that pin; compiler-generated links and bibliography remain explicitly qualified.

The [WebKit glyph witness](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSProperties.json#L5350-L5367) is bounded operational compatibility evidence, not a CSS or SVG whole-module selection or standards consensus. Its file-specific redistribution license remains unverified; metadata-only representation is appropriate until resolved. The pre-existing Unicode UAX 29 revision 47 metadata-only exception also remains explicit. Missing readable text is a representation gap; missing authored grammar is a separate implementation gap. This reference update fixes neither by assertion.

## Text, font and effects source resolution

The following Editor’s Draft captures were retrieved on 10 October 2026 and declare source revision `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`. They supplement only the named domains; historical complete editions remain available and are not silently superseded. A displayed draft date is separate from retrieval identity. The complete original HTML hash is the version anchor even where the Markdown filename identifies the bounded conversion input.

| Contribution | Bounded retained clauses | Complete original HTML SHA-256 |
| --- | --- | --- |
| [CSS Fonts 4 width matching](css-fonts-4-width-matching--editor-capture-20261010--72f4c49ea2ec.md) | complete sections 2.3 (including 2.3.1) and 5.2, plus document head/status/legal notice | 72f4c49ea2ec6b3e417f2c3cb8e140d4106ffba9f2610b9b74219231424dddb7 |
| [Text Decoration 4 position controls](css-text-decor-4-position-controls--editor-capture-20261010--8820547faa34.md) | complete sections 2.4–2.9 (including 2.8.1), plus document head/status/legal notice | 8820547faa34362e96dd4542039da8374f416634dc36f5e58f7a03346e7a4277 |
| [CSS Text 4 whitespace, alignment and hyphenation](css-text-4-whitespace-alignment-hyphenation--editor-capture-20261010--2b0d3e0d5174.md) | complete sections 4.1, 4.3, 6.3.5, 6.4, 7.1 and 7.2, plus document head/status/legal notice | 84295064951605c856935d2216b8952060726da6794412c8c4ab35a86a298f64 |
| [Filter Effects 1 effects clauses](filter-effects-1-effects-clauses--editor-capture-20261010--b9a791956ef1.md) | complete sections placement, FilterProperty, FilterPrimitivesOverviewIntro, feColorMatrixElement, feDisplacementMapElement, ColorInterpolationFiltersProperty, huerotateEquivalent, tainted-filter-primitives, fedisplacemnentmap-restrictions, plus document head/status/legal notice | 2ead3581d431bf3663167ada7e410793e0e561812b772081bc286da489826e29 |
| [CSS Transforms 2 effects clauses](css-transforms-2-effects-clauses--editor-capture-20261010--ec252486c8c9.md) | complete sections transform-rendering, ctm, backface-visibility-property, mathematical-description, plus document head/status/legal notice | e3de1ccf32db61406fcb5cbb8feb0791b8375844f6f6eebf1cb016e73167f2fe |

The Text 4 published URL still returned the existing 14 August 2026 edition (`106c5cd086ab23b0edea5aa3ff6876b61cc623154f728aa269b74f0527dba19d`). The newer editor capture preserves the open discard/preserve-spaces, character-alignment and emergency-hyphenation questions. Filter Effects preserves its conflicting hue-rotate matrix/formula and taint questions; Transforms preserves its backface/noninvertible-matrix question. Those omissions/conflicts are not repaired inside the source conversions. Fonts and Text Decoration captures preserve their selected width-matching and decoration controls without widening a reference addition into whole-module implementation.

[Font/decoration browser evidence](browser-font-width-decoration--pinned-20261010.md), [WebKit text evidence](webkit-text-source-gaps-hyphenation--73aa6c89e2cb.md), [Blink text evidence](blink-text-source-gaps--7984f9d11800.md), and [WebKit effects evidence](webkit-filter-backface-witnesses--73aa6c89e2cb--5a2ab9f34073.md) are separately identified implementation witnesses. WebKit uses preferred pin `73aa6c89e2cb77c46184a81aec944e4ab99d114d`; two FontSelection files at `437139e30888c5e1b4c35f889f32e1d0c39d018e` are verified byte-identical and reuse the complete preferred bodies. Blink uses `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. The catalog distinguishes all 23 full browser identities and three partial effects files; partial functions/ranges are not whole-engine witnesses. CSSProperties remains the existing metadata-only exception.

The [Parley 0.9.0 witness](parley-0.9-text-contract-witness.md) retains bounded declarations/functions from twelve exact-version installed registry files, plus `skrifa` 0.42.1 metrics excerpts. Full-file hashes and selected line ranges identify thirteen partial witnesses; the normalized Cargo manifest is partial even though its selected range starts at line 1. Source pages identify exact registry versions rather than invented Git revisions. This evidence supports provider/private-module coupling and selected-line ordering; no production ingestion or exhaustive conformance is claimed.

### Operative source decisions by issue

Each question below selects the named current capture for its operative clause evidence. Earlier dated editions remain historical evidence. The selected result is recorded in the linked owning issue; a browser precedent does not become normative CSS. This mapping records documented determinations and consumer allocation, not completed feature implementation.

| Owning source issue | Operative clauses and implementation evidence | Selected result and category | Open implementation consumers |
| --- | --- | --- | --- |
| [#74 whitespace gaps](https://github.com/bj-data-eng/surgeist/issues/74) | [Text 4 §§4.1/4.3](css-text-4-whitespace-alignment-hyphenation--editor-capture-20261010--2b0d3e0d5174.md#white-space-collapsing), including [Phase II](css-text-4-whitespace-alignment-hyphenation--editor-capture-20261010--2b0d3e0d5174.md#white-space-phase-2); [WebKit](webkit-text-source-gaps-hyphenation--73aa6c89e2cb.md) and [Blink](blink-text-source-gaps--7984f9d11800.md) bounded value absence | **Authorized product policy:** discard removes its own opportunities; preserve-spaces uses preserve-equivalent hanging with wrapping, conditional forced/end hanging and nowrap bypass, retaining advances. | #64/#65/#71 |
| [#76 character-alignment gaps](https://github.com/bj-data-eng/surgeist/issues/76) | [Text 4 §7.2](css-text-4-whitespace-alignment-hyphenation--editor-capture-20261010--2b0d3e0d5174.md#character-alignment); pinned [WebKit](webkit-text-source-gaps-hyphenation--73aa6c89e2cb.md) and [Blink](blink-text-source-gaps--7984f9d11800.md) string-alignment absence | **Authorized product policy:** shaped advance midpoint; first logical match in eligible in-flow text whose containing block is the cell; measurement-only virtual character uses the cell font context. Defined fallbacks and #48’s separate questions remain. | #68 |
| [#77 emergency-hyphenation interaction](https://github.com/bj-data-eng/surgeist/issues/77) | [Text 4 §6.3.5](css-text-4-whitespace-alignment-hyphenation--editor-capture-20261010--2b0d3e0d5174.md#hyphenate-line-limits); [WebKit committed-line state](webkit-text-source-gaps-hyphenation--73aa6c89e2cb.md#inlinelayoutstateh) and complete breaker chain | **Pinned browser compatibility precedent**, completed by **authorized product policy** for uncovered exceptional endings: committed hyphenation endings increment the successive counter, other endings reset; arbitrary unhyphenated emergency wraps re-enable automatic opportunities; trial candidates are pure. | #66 |
| [#415 decoration ambiguity](https://github.com/bj-data-eng/surgeist/issues/415) | [Text Decoration 4 §2.5](css-text-decor-4-position-controls--editor-capture-20261010--8820547faa34.md#line-position), [§2.4 thickness](css-text-decor-4-position-controls--editor-capture-20261010--8820547faa34.md#text-decoration-thickness-property) and [§2.8 offset](css-text-decor-4-position-controls--editor-capture-20261010--8820547faa34.md#underline-offset); [browser decorating metrics](browser-font-width-decoration--pinned-20261010.md); [Skrifa metric excerpt](parley-0.9-text-contract-witness.md#supporting-provider-skrifa-0421-metricsrs) | **Normative interpretation:** compute base position/adjustments, then local offset; originating controls select thickness. **Authorized product policy:** an empty considered-text fragment uses its decorating box’s resolved strikeout center; provider normalization/fallback follows the named metric contract. | #407/#408 and text-metric provider work |
| [#765 exact 100% width boundary](https://github.com/bj-data-eng/surgeist/issues/765) | [Fonts 4 §5.2](css-fonts-4-width-matching--editor-capture-20261010--72f4c49ea2ec.md#font-style-matching), selected over the conflicting [§2.3 summary](css-fonts-4-width-matching--editor-capture-20261010--72f4c49ea2ec.md#font-width-prop); [preferred/newer WebKit matcher](browser-font-width-decoration--pinned-20261010.md#fontselectionalgorithm-cpp) | **Normative interpretation corroborated by pinned browser evidence:** exact 100% is narrower-first when no candidate range contains it; below remains narrower-first and above wider-first. Preserve range matching, width/style/weight filtering and the synthesis restriction. The source conflict remains visible. | #761 |
| [#356 Filter Effects execution gaps](https://github.com/bj-data-eng/surgeist/issues/356) | Current [FilterProperty](filter-effects-1-effects-clauses--editor-capture-20261010--b9a791956ef1.md#FilterProperty), [feDisplacementMapElement](filter-effects-1-effects-clauses--editor-capture-20261010--b9a791956ef1.md#feDisplacementMapElement) and [feColorMatrixElement](filter-effects-1-effects-clauses--editor-capture-20261010--b9a791956ef1.md#feColorMatrixElement); [pinned WebKit kernels/paint](webkit-filter-backface-witnesses--73aa6c89e2cb--5a2ab9f34073.md) | **Pinned browser compatibility:** filter includes fixed-background pixels while preserving attachment geometry; the software displacement profile uses one-source-pixel integer sampling truncated toward zero; hueRotate selects the .213/.715/.072 matrix. Defined color, clipping and taint rules remain; the conflicting expanded expression remains a documented source conflict, not another execution branch. | #348/#349/#350/#888 |
| [#563 perspective-aware backface predicate](https://github.com/bj-data-eng/surgeist/issues/563) | Current [Transforms 2 backface clause](css-transforms-2-effects-clauses--editor-capture-20261010--ec252486c8c9.md#backface-visibility-property); [pinned WebKit inverse-normal function](webkit-filter-backface-witnesses--73aa6c89e2cb--5a2ab9f34073.md) | **Pinned browser compatibility predicate:** cull hidden backfaces iff the accumulated inverse-normal component is negative. **Authorized product numeric policy:** evaluate robust signs, keep exact singularity and horizon/negative-w clipping separate, and do not copy WebKit’s non-normal-determinant early return as a front-facing rule. | #559 |

The [text/layout contract](../docs/text-layout-contract.md) owns the shared preparation, selected-line, font-instance, metric and renderer-admission mechanisms; it does not settle every separate CSS feature question above. Feature determinations remain directly discoverable in their owning issues. Source retention neither resolves a standards question by assertion nor grants an exemption from defined requirements or closes implementation work.

## Layout sizing, fragmentation and overflow source resolution

The captures below were retrieved on 10 October 2026. Each retains complete named clauses, document status and legal material, and identifies the complete original HTML separately from its bounded conversion input. They are operative only for the named issue scopes. Earlier editions remain historical evidence for completed work; neither their old omissions nor a browser implementation overrides a newer defined requirement. Remaining choices are recorded as authorized Surgeist behavior in the owning issues, rather than inserted into the source text.

| Owning issue | Operative evidence | Determination and implementation owner |
| --- | --- | --- |
| [#48 character-aligned table sizing](https://github.com/bj-data-eng/surgeist/issues/48) and [#953 table dimensions](https://github.com/bj-data-eng/surgeist/issues/953) | [Current Tables 3 sizing clauses](css-tables-3-sizing--editor-capture-20261010--e0ddc09996f1.md), existing [Text 4 character alignment](css-text-4-whitespace-alignment-hyphenation--editor-capture-20261010--2b0d3e0d5174.md#character-alignment), and [pinned WebKit table implementation](webkit-table-sizing--73aa6c89e2cb.md) | Defined CSS2 constraints remain; #953 explicitly adopts bounded newer draft sizing mechanisms for gaps, preserving the draft’s warnings. The source issue selects product policies for intrinsic splits, spanning cells and constrained alignment; #953 owns table sizing and #68 consumes character alignment. The previously selected character-search and metric contract in #76 remains unchanged. |
| [#246 Flexbox intrinsic sizing and fragmentation](https://github.com/bj-data-eng/surgeist/issues/246) | [Current Flexbox §§9.9–10](css-flexbox-1-intrinsic-fragmentation--editor-capture-20261010--cc15b06af9bf.md) | **Upstream resolution:** §9.9.1.2 now defines the web-compatible intrinsic-main algorithm, consumed by #243. Fragmentation still leaves algorithm choices; #244 records the selected continuation policy and consumes shared break selection from #14. The informative sample remains informative. |
| [#678 generated leaders](https://github.com/bj-data-eng/surgeist/issues/678) | [Current Content 3 leader clauses](css-content-3-leaders-clauses--editor-capture-20261010--586d408a8de7.md) and [pinned browser entry points](browser-leader-support--webkit-73aa6c89e2cb-blink-7984f9d11800.md) | **Authorized product policy:** isolated pattern units, deterministic alignment and progressing overflow handling, and finite table intrinsic contributions. The four source questions remain visible in the source; #672 owns implementation and behavioral verification. Inspected browser entry points provide no leader algorithm. |
| [#206 scrollable overflow](https://github.com/bj-data-eng/surgeist/issues/206), [#205 clip geometry](https://github.com/bj-data-eng/surgeist/issues/205) and [#199 computed overflow](https://github.com/bj-data-eng/surgeist/issues/199) | [Current Overflow 3 §§2–3](css-overflow-3-geometry-clauses--editor-capture-20261010--86c09a636507.md) | **Upstream clarification:** overflow/paint-containment clipping affects scrollable overflow; clip, clip-path and masking do not. Current signed clip-margin and mixed-axis rules supersede older clauses only for these planned consumers. Projection and other remaining latitude receive explicit bounded product policies in their owners. Completed #198 retains its historical authored grammar; adopting the newer signed grammar is separate implementation work. |
| [#997 lanes orientation/repetition/subgrid](https://github.com/bj-data-eng/surgeist/issues/997), [#998 cursor/static positioning](https://github.com/bj-data-eng/surgeist/issues/998) and [#1000 balance](https://github.com/bj-data-eng/surgeist/issues/1000) | [Current Grid 3 selected clauses](css-grid-3-layout-decisions--editor-capture-20261010--54fd3c34f1ce.md) and [pinned WebKit lanes evidence](webkit-grid-lanes-witnesses--73aa6c89e2cb--5b433979a8aa.md) | Named determinations preserve the completed authored contracts in #977/#978/#999. Layout consumers #986/#988/#990/#993/#994/#996 own execution; no provisional property spelling is invented to fill an editorial placeholder. |
| [#14 shared fragmentation](https://github.com/bj-data-eng/surgeist/issues/14) and [#991 grid fragmentation](https://github.com/bj-data-eng/surgeist/issues/991) | [Current Break 3 selected clauses](css-break-3-layout-decisions--editor-capture-20261010--debb8c3e11e2.md), [Grid 2 fragmentation excerpt](css-grid-2-fragmentation-clauses--editor-capture-20261010--444e85eb609a.md), and [current Flexbox fragmentation](css-flexbox-1-intrinsic-fragmentation--editor-capture-20261010--cc15b06af9bf.md) | #14 owns one deterministic shared break-selection/progress policy, preserving forced-break precedence and required relaxation. The current Break 3 wording corrects cloned-edge truncation to block-end then block-start. #244/#991/#996 retain their domain-specific requirements and existing genuine prerequisite on #14. These remain implementation issues. |

The shared [text/layout contract](../docs/text-layout-contract.md) remains authoritative for prepared text, selected-line metrics, font identity and publication. Feature-specific decisions stay in the linked issues. Resolving source/design questions does not certify implementation, browser parity, or completion of the enclosing contribution.

## DOM, selector execution and bounded style determinations

The 2026-10-10/11 sources below select bounded clauses for unstarted DOM, selector, cascade and style execution. Earlier dated editions remain historical evidence and continue to govern already selected authored grammar unless an owning issue expressly changes it. Reference retention does not claim implementation or full DOM/HTML/browser conformance. Source gaps and contradictions remain visible; the issue determinations distinguish standards interpretation, browser precedent and chosen Surgeist policy.

| Domain and owning issues | Selected source and bounded effect |
| --- | --- |
| DOM foundation #1074/#1124 | [DOM foundation](dom-foundation--editor-capture-20261010--08dff0a4827f.md) selects tree/name/node-type, node-document, adoption/import, hierarchy/mutation and slot algorithms. Native Rust store ownership, generation-qualified identities, explicit limits, atomic publication and cross-store adoption rejection are chosen API policy. Detachment preserves identity and node-document; adoption within a store preserves identity; import creates fresh identity. |
| DOM strings #1074/#1124 | [Web IDL DOMString/USVString](dom-webidl-string-types--editor-capture-20261010--53a256058148.md) and [Infra code points/strings](dom-infra-codepoints-strings--editor-capture-20261010--a140d8f6286f.md) establish UTF-16 strings, including isolated surrogates. Lossless DomString and fallible UTF-8 conversion are Rust policy; CSS preprocessing belongs at the consuming adapter. |
| DOM host facts #1124/#1125 | [HTML language/direction](dom-html-language-direction--editor-capture-20261010--494ac341fe2b.md), [interaction](dom-html-host-interaction--editor-capture-20261010--8a01a8043e7c.md) and [selector facts](dom-html-host-selector-facts--editor-capture-20261010--07d421b207b3.md) select only captured clauses. Host observations require explicit coherent DOM/host revisions. Privacy treatment of links as unvisited and supported/unsupported/not-applicable facts are product policy, not complete HTML runtime behavior. |
| Shadow relations #375/#1124 | [current CSS Scoping 1](selectors-execution-css-shadow-1--editor-capture-20261010--b02c47a6b167.md) supplies shadow matching and flat-tree/inheritance semantics alongside DOM slots. DOM parent, shadow-including parent, direct assigned-slot relations, flattened slottables and flat-tree parent remain distinct. Assigned nodes inherit through the slot; selector traversal does not indiscriminately use the flat tree. |
| Selector execution #1123 | [Selectors 4](selectors-execution-4--editor-capture-20261010--7dddba84e895.md), [Selectors 5](selectors-execution-5--editor-capture-20261010--46c47dc5e452.md) and [Pseudo-Elements 4](selectors-execution-css-pseudo-4--editor-capture-20261010--46f88407070a.md) select execution/validation facts without replacing the existing authored CSS grammar. Column membership is a document-language fact independent of layout geometry; pseudo identity and host capability limits are explicit contracts. |
| Scope cascade #317 | [current Cascade 6](css-cascade-scope-6--editor-capture-20261010--9f61dd29dd2a.md) plus [CSSWG resolutions](css-cascade-scope-resolutions--capture-20261010--123a5243ef9f.md) select specificity before scope proximity, then order of appearance, and a single innermost-scope proximity. Conflicting historical/current draft prose is retained; accepted resolutions adjudicate the conflict without an outer-distance vector. |
| Speech #41/#37 | [Speech 1 speak clause](style-source-determinations-css-speech-1--editor-capture-20261010--2721be0ed51a.md) has a surviving table/prose contradiction. Choosing per-value prose is Surgeist policy: own display:none computes auto to never; otherwise auto remains computed, and used auto follows visibility. Legacy Blink grammar is evidence only. |
| Containment #125/#151 | [Containment 2](style-source-determinations-css-contain-2--editor-capture-20261010--ac9dcde44315.md) now defines style containment in the computed table. Retaining inline-size despite the remaining table omission and expanded strict/content computed serialization are selected policy; authored aliases remain distinct. |
| Scheme/emulation #154/#151 | [Color Adjustment 1](style-source-determinations-css-color-adjust-1--editor-capture-20261010--a703bef833db.md) selects the newer complete page/element resolving domain. Early algorithm default returns take precedence over broader override prose. Explicit host lookup context corrects the still-invalid enum/global-object getter model as Surgeist policy. |
| Environment/substitution #260/#262 | [Environment Variables 1](style-source-determinations-css-env-1--editor-capture-20261010--dcc621f98ef6.md) and [pinned deferred-substitution evidence](style-source-determinations-browser--pinned-20261010.md) support style-time substitution. Recursive evaluation of selected fallback branches, immutable host environment context and grammar validation after substitution are explicit policy for the draft timing gap. |
| Container queries #650/#654 | [Conditional 5 type/eligibility/units](style-source-determinations-css-conditional-5--editor-capture-20261010--b381cdbfd039.md) retains the layout-containment contradiction. Principal-box and effective queried-axis size containment, without an added layout-containment flag, follow pinned browser precedent and chosen interpretation; declared type/name selection remains separate from feature evaluation. |
| Device CMYK #822/#810 | [Color 5 device-CMYK clauses](style-source-determinations-css-color-5--editor-capture-20261010--179c744eea38.md) retain the computed Lab/sRGB versus device-specific serialization conflict. Computed CMYK channels plus a separate context-dependent profile/naive colorimetric conversion are Surgeist policy, not standards consensus. |
| Line-boundary owner #75 | [Text 4 line-break details](style-specialized-determinations-css-text-4--editor-capture-20261011--2bb0ce6fbd6f.md) leaves boundary ownership unresolved. The previously retained WebKit ordinary-boundary mechanism and selected emergency completion remain explicitly bounded policy. |
| Generated-content retrieval #679 | [Content 3 target/content functions](style-specialized-determinations-css-content-3--editor-capture-20261011--edd86c947a65.md) selects retrieval clauses; local-fragment-only targets, failure to empty string plus diagnostics, decimal omitted style and content omitted selector are explicit product completion. |
| Generated content model #677 | [Content 3 content/string/element clauses](style-specialized677-determinations-css-content-3--editor-capture-20261011--2afa2007d798.md) is an additional bounded representation of the same full source identity. The owning issue selects native replacement/content behavior and retains parser evidence separately; reference acquisition alone does not resolve or ship the domain. |
| View transitions #392 | [View Transitions 1 lifecycle clauses](style-specialized-determinations-css-view-transitions-1--editor-capture-20261011--af337b0548f0.md) supplies newer size-change InvalidStateError clauses. Paired ordinary-rule retention and native host/runtime effects are separate product contracts. |
| Page contextual lengths #949 | [Page 3 progression/first-page clauses](style-specialized-determinations-css-page-3--editor-capture-20261011--c44d437cd37a.md) and [Conditional 5 container lengths](style-specialized-determinations-css-conditional-5--editor-capture-20261011--bc340aca45c3.md) select only those clauses. An independent reference page-area snapshot for no-container fallback is chosen policy; existing em/ex restrictions and approved page-local variables remain. |
| Table overlap placement #952 | [Tables 3 row/column grid](style-specialized-determinations-css-tables-3--editor-capture-20261011--f213e96cb8a9.md) and [HTML table processing](style-specialized-determinations-html-tables--editor-capture-20261011--7e90e654e94f.md) select first-free starting slots, complete spans and overlap reporting. Generic host spans, layout sizing and placement are separately owned; no HTML parser conformance is implied. |

The Conditional 5 type/eligibility capture and page-length capture share full HTML hash `3b4a4b4ef336edeaca211cdb90481bfcf9cda1955c1cabce485b98e2fcb497e3`. The two Content 3 captures share `9c640ae017c7f848bb11171b96cca9d6b3b01f78f33faafc9bb094607c833945`. These are multiple representations of one source each. Blink css_properties.json5 retains its earlier complete witness and adds only a bounded speak-entry link. Displayed CSSWG revision `50337328484f7fcdd9ab6fe9ded6011815b1707e` is provenance; full captured HTML hashes remain the source identities. New browser witnesses use immutable source URLs and preserve exact file-specific legal headers.

### Native Rust event dispatch

For #840's Style load/error and #1124's Details toggle delivery, select the
complete bounded [DOM event source](dom-native-events--3071e5f26944--332564d9519c.md)
from the existing DOM3071 living capture and the
[ToggleEvent interface](html-toggle-event--877d6146487f--eeb465762458.md)
from the existing HTML877d snapshot. DOM original SHA-256 is
`034b308db8482bddb37070849e9d4ab40bcb2d10e6494cffad14678a252a58b1`;
HTML original SHA-256 is
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
These add representations of already cataloged identities, not newer editions.

Runtime owns one typed listener/path dispatcher and feature task servicing;
DOM owns canonical identities and stored tree/slot relations, and HTML owns
payload and notification rules. Nonbubbling events still invoke ancestor capture
listeners. Event paths use actual assigned-slot and ShadowRoot parent rules,
with retargeting and closed-tree exposure. Listener snapshots are per invocation;
real synchronous Rust callbacks may change the tree and registry. Closed native
Documents have no Window event parent. The selected scope does not add JavaScript,
full WebIDL exposure, arbitrary event construction or a generic event loop.
The owning issues retain admission, retirement, reentry and actual delivery
acceptance; these references do not establish implementation completion.

## Additional source-question determinations

These bounded additions address #823, #629/#630, #602/#604/#605, #648, #378/#379, #681/#682 and the assigned media/preference/border questions. Existing authored grammar and unrelated completed domains keep their source selections. The issue determinations distinguish normative requirements from chosen Rust contracts, compatibility evidence and unresolved standards text. Source acquisition itself does not implement these features.

| Selected domain | Operative source boundary |
| --- | --- |
| CSS Color Module Level 5 | [bounded capture](style-source-determinations-additional-css-color-5--editor-capture-20261010--8758c9a76695.md); complete sections `relative-color-function`, `at-profile`. Full original SHA-256 `a53d07682aa1de30f93de499e7e29131d6460f393e3559d669ac663075b6b26b`. |
| CSS Animations Module Level 1 | [bounded capture](style-source-determinations-additional-css-animations-1--editor-capture-20261010--bc6a6dd8f7b1.md); complete sections `animations`, `keyframes`, `other-open-issues`, `wg-resolutions-pending`. Full original SHA-256 `a6ef2cb48b0d502cbf3f7ad7636e48279b330b10f4451700e25741f326cb5be0`. |
| CSS Animations Module Level 2 | [bounded capture](style-source-determinations-additional-css-animations-2--editor-capture-20261010--d03de1340087.md); complete sections `delta`, `animation-composite-order`, `keyframe-processing`, `animation-timing-function`, `animation-composition`. Full original SHA-256 `f99950324f4253347cd9481a211edd23ae7b7db90f866782d87f84098c347b6c`. |
| Web Animations Module Level 1 | [bounded capture](style-source-determinations-additional-web-animations-1--editor-capture-20261010--92e1d5e27678.md); complete sections `animating-properties`, `keyframe-effects`, `combining-effects`. Full original SHA-256 `13e0307c3c72626740baca4c9d38e30848674a2c6904d5f513e19c70ec6bd58f`. |
| Media Queries Level 5 | [bounded capture](style-media-determinations-mediaqueries-5--editor-capture-20261011--8ff761c2dff2.md); complete sections `environment-blending`, `mf-user-preferences`, `script-control-user-prefs`. Full original SHA-256 `6985bff472a17f32b4eefe21273a689f7fcbe9f6334cefc6541c9b5d472cd626`. |
| CSS Backgrounds and Borders Module Level 3 | [bounded capture](style-media-determinations-css-backgrounds-3--editor-capture-20261011--11afcd91a294.md); complete sections `border-image-tables`. Full original SHA-256 `ebf9643d9bf7228b590b57480841ea839930b603acf4ea97ef0644c2240e46f6`. |
| CSS Borders and Box Decorations Module Level 4 | [bounded capture](style-media-determinations-css-borders-4--editor-capture-20261011--3d13a172cd34.md); complete sections `border-image-tables`. Full original SHA-256 `b592227c041d8681a2de4f11503a995ea1fc57ec17975fb251b5201d3ac32f7e`. |
| Web Animations 1: native timeline/lifecycle host | [bounded capture](style-animation-host-determinations-web-animations-1--editor-capture-20261011--032c03645e8f.md); complete sections `document-timelines`, `sorting-animation-events`, `the-animation-interface`, `changes-since-last-publication`, `model-liveness`. Full original SHA-256 `13e0307c3c72626740baca4c9d38e30848674a2c6904d5f513e19c70ec6bd58f`. |
| CSS Conditional Rules 5: supports/generalized conditionals | [bounded capture](additional-selectors-css-conditional-5--editor-capture-20261011--d1b76ceb44e1.md); complete sections `at-supports-ext`, `when-rule`, `else-rule`, `supports-condition-rule`, `the-csssupportscondition-interface`. Full original SHA-256 `3b4a4b4ef336edeaca211cdb90481bfcf9cda1955c1cabce485b98e2fcb497e3`. |
| CSS Generated Content Module Level 3 | [bounded capture](style-host-bookmarks-css-content-3--editor-capture-20261011--b3028754056d.md); complete sections `bookmark-generation`. Full original SHA-256 `9c640ae017c7f848bb11171b96cca9d6b3b01f78f33faafc9bb094607c833945`. |
| Accessible Name and Description Computation 1.2 | [bounded capture](style-host-generated-accname-1.2--WD-accname-1.2-20261002--86c08e5f10b6.md); complete sections `terms`, `conformance`, `mapping_additional_nd`, `accessible-name-and-description-mapping`. Full original SHA-256 `3658a43482510f5fa0083069c9dcb0321089acc903a2177a63091930da038082`. |

The [additional CSSWG resolution witness](additional-selectors-resolutions--capture-20261011--b321c7ac6475.md) retains four brief accepted-resolution quotations for marker inclusion and pseudo-object identity/parsing. Two additional comments are metadata-only provenance. Existing Pseudo-Elements 4 remains the operative source for the related execution clauses. Exact edit times and response/body hashes remain in the witness; GitHub comment IDs alone are not immutable source identity.

Pinned browser evidence supplies bounded at-rule recognition, keyframe selection/interpolation, accessibility projection, preferences/media interpretation and table painting/geometry facts. It is not whole-browser parity or a replacement specification. Color 5, Conditional 5 and Content 3 additional clauses share existing original hashes; preferred at-rule catalog files remain complete witnesses. Displayed CSSWG revision metadata remains provenance rather than proof of live HTML byte identity. The AccName 1.2 capture is the current published 2 October 2026 Working Draft, not the older editor witness; only its named complete sections are retained.

The #823 profile-name collision remains a standards gap: selecting the first exactly matching decoded channel name while preserving ordinal coordinates is Surgeist policy. #629/#630 select the captured Animations 2 assembly and Web Animations property-track/effect-stack algorithms; same-offset frames group by computed easing and composition, and distinct groups retain stable overlap. The last contributing replace effect is selected per property; already selected non-replace operations retain their composition behavior. These bounded changes do not import a new animation API or every newer timeline family.

The #648 generalized-supports body remains a normative gap with no support in the inspected browser recognition tables. Last-definition document order follows the captured source. Conjunction over retained recursive candidates, an empty clean body evaluating true, activation dependencies and cycle handling are explicit product policy. Marker inclusion follows the accepted CSSWG resolution: inside markers participate in first-line, outside markers do not, and first-letter excludes both. Native pseudo lookup preserves full-selector identity independently of generated existence; the current interface supplies no EventTarget inheritance. These selections preserve the existing authored grammar.

For #604, the current Web Animations clauses select document clocks with fixed origins and remove the incompatible conditional resume adjustment. #602 adopts the retained display:contents assumption as explicit rendered eligibility, with its guards and the recorded WebKit renderer divergence. #605 repairs the animation-origin wrapper signature and unresolved-result propagation while keeping timeline-origin conversion separate. Native host contracts own these choices; no JavaScript API is imported.

The owning issue determinations record the operative choices beyond the captured clause boundaries: [#460 subtractive output](https://github.com/bj-data-eng/surgeist/issues/460), [#461 preference overrides](https://github.com/bj-data-eng/surgeist/issues/461), [#884 border-image applicability](https://github.com/bj-data-eng/surgeist/issues/884), [#892 nine-slice painting](https://github.com/bj-data-eng/surgeist/issues/892) and [#954 table composition](https://github.com/bj-data-eng/surgeist/issues/954). The bookmark selection and host projection are owned by [#681](https://github.com/bj-data-eng/surgeist/issues/681) and [#675](https://github.com/bj-data-eng/surgeist/issues/675); accessible generated-content projection and its consumer contract are owned by [#682](https://github.com/bj-data-eng/surgeist/issues/682) and [#671](https://github.com/bj-data-eng/surgeist/issues/671). These issue contracts identify the selected requirements, browser divergences and explicit product completion without silently adopting an entire newer module.

## HTML Color delegation and intrinsic contrast policy

For [#805](https://github.com/bj-data-eng/surgeist/issues/805), the Architecture
Coordinator selects CSS-owned standalone source parsing, contextual resolution
and source/fact qualification, with numerical conversion, interpolation,
transformations and resolved serialization owned by
[`surgeist-color`](../crates/surgeist-color/README.md). The current
[`surgeist-html-parser` adapter](../crates/surgeist-html-parser/src/construction/input_color.rs)
consumes both directly and owns its original controller, fallback, transformation
order and publication. Each consuming crate owns its adapter; the root facade
supplies optional composition. Style owns contextual property/profile facts and
runtime owns resource retrieval. Source-based and checked-authored standalone
resolution, controlled numerical continuations and HTML Color construction are
implemented. HTML values, parsing and semantics have separate crate owners. The
owning issue retains its separate contextual CSSOM and
quirky-color/CSS.supports acceptance.

HTML remains at `877d6146487f15b15bc8dd0e3c625063a37b456a`. Its
[color-well update](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#update-a-color-well-control-color)
and [serialization](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#serialize-a-color-well-control-color)
delegate bounded parsing, resolution, conversion and serialization to the
already selected [Color 4 CRD of 8 September 2026](css-color-4--CRD-css-color-4-20260908--bada647312a7.md).
The original Color 4 document's HTML capture SHA-256 is
`bada647312a73c4cd4484dc177d67d22cf70df6c451b0e8c5dd5efc29c3415ba`;
the converted Markdown file SHA-256 is
`31b9210596843b84bdc1a61d82687b3d1278f885a1559ec0c64264e3642076fe`.
These identify different representations, not different operative editions.

The selected boundaries are
[parse-a-css-color-value](css-color-4--CRD-css-color-4-20260908--bada647312a7.md#parse-a-css-color-value),
[resolving-color-values](css-color-4--CRD-css-color-4-20260908--bada647312a7.md#resolving-color-values),
[color-conversion](css-color-4--CRD-css-color-4-20260908--bada647312a7.md#color-conversion)
and [serializing-color-values](css-color-4--CRD-css-color-4-20260908--bada647312a7.md#serializing-color-values),
including [conditional HTML-compatible sRGB serialization](css-color-4--CRD-css-color-4-20260908--bada647312a7.md#HTML-compatible-serialization-of-srgb).
HTML supplies no context element. Required property values use their selected
initial values; [initial color is CanvasText](css-color-4--CRD-css-color-4-20260908--bada647312a7.md#propdef-color),
which does not itself supply a system palette. HTML owns dirty/default/current
selection, alpha/colorspace policy, the request for numerical quantization and
source-positioned updates.
Its positive-infinity tie rounding retains
[Values 4 of 12 March 2024](css-values-4--WD-css-values-4-20240312--7b6b68c34d00.md#css-round-to-the-nearest-integer).
No HTML pin, accepted authored grammar or global gamut policy is replaced.

Existing selected Color 5 editions and explicitly scoped later determinations
remain operative for their admitted syntax. The [#822 device-CMYK selection](https://github.com/bj-data-eng/surgeist/issues/822)
and [#823 profile-channel selection](https://github.com/bj-data-eng/surgeist/issues/823)
remain distinct product determinations where reached. This delegation does not
select every later editor-draft clause.

### ContrastColor metric and bounded browser evidence

[Color 5 of 8 September 2026 §8](css-color-5--WD-css-color-5-20260908--f749cb75ec1c.md#contrast-color)
requires the black/white choice with maximum contrast under a UA-defined metric
and white on equal contrast. It advises against simply using the WCAG 2.1
ratio algorithm. The selected [#805 policy](https://github.com/bj-data-eng/surgeist/issues/805)
explicitly considers that warning and chooses the relative-luminance ratio
comparison as Surgeist's deterministic intrinsic operation. The metric is
product policy, not a normative CSS requirement or an accessibility
certification.

The [complete pinned WebKit witness](webkit-contrast-color--73aa6c89e2cb--6592a4862701.md)
retains CSSContrastColorResolver.cpp, ColorLuminance.h and ColorLuminance.cpp at
`73aa6c89e2cb77c46184a81aec944e4ab99d114d`, with original notices and full
per-file hashes. Its helper converts to XYZ D65 Y and compares luminance ratios.
The browser resolver uses strict `>` and therefore black on an exact tie;
that branch is not selected.

CSS resolves the input using the source-qualified used-color producer and only
its reached facts, then delegates the numerical black/white operation to Color.
For the solid-background analysis it ignores alpha, resolves missing components
through the selected conversion rules, and uses Color's XYZ D65 conversion's Y.
Finite Y is locally clamped to [0,1] only for this comparison. This clamp is
Surgeist policy; the cited helper does not establish it. The actual input color and alpha remain intact, with no invented
canvas compositing, global gamut mapping or sRGB byte quantization.

Under the producer's binary64 contract, black scores `(Y + 0.05) / 0.05` and
white scores `1.05 / (Y + 0.05)`. White is selected when its computed score is
greater than or equal to black's; otherwise black is selected. The result is
opaque sRGB. No epsilon or rounded crossover threshold is selected. Required
conversion/numerical failures and unavailable influencing facts retain their
typed outcomes; they do not become a successful black fallback. Contrast itself
requires no external service.

The owning issue supplies the detailed capability and test contract. This
reference checkpoint records its exact source selection and the
[file-specific attribution](../licenses/webkit/NOTICE.md#contrast-color-source-witnesses);
this reference ingests no WebKit code, implements no algorithm and claims no
browser parity. The implemented numerical owner preserves that source selection
and policy.

## Pinned HTML language baseline

The unstarted native HTML language contracts select WHATWG HTML commit snapshot `877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026), independently of the html5ever implementation ingestion baseline. The complete original source SHA-256 is `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Companion captures retain complete selected sections, all chapter 4 element definitions, syntax/parsing, language foundations, host facts, DOM parsing and obsolete behavior. These bounded source references do not claim whole browser implementation or assign JavaScript runtime APIs to the HTML crate.

| Retained complete sections | Pinned local capture |
| --- | --- |
| `syntax` | [syntax-parser](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md) |
| `terminology`, `common-microsyntaxes`, `urls`, `common-dom-interfaces`, `dom` | [foundations](html-language-foundations--877d6146487f--b55e73c31978.md) |
| `the-root-element`, `document-metadata`, `elements-3`, `element-content-categories`, `attributes-3` | [elements-attributes](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md) |
| `tables`, `forms` | [forms-tables-templates](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md) |
| `sections`, `grouping-content`, `text-level-semantics`, `links`, `edits` | [elements-text-links](html-language-elements-text-links--877d6146487f--8833a85fd242.md) |
| `embedded-content` | [elements-embedded](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md) |
| `interactive-elements`, `scripting-3`, `custom-elements`, `common-idioms` | [elements-interactive-scripting](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md) |
| `the-hidden-attribute`, `inert-subtrees`, `user-activation-data-model`, `focus`, `disabled-elements`, `selectors` | [host-facts](html-language-host-facts--877d6146487f--2dd822df9d5e.md) |
| `dom-parsing-and-serialization`, `enabling-and-disabling-scripting` | [dom-parsing](html-language-dom-parsing--877d6146487f--c076ab7a56b9.md) |
| `obsolete` | [obsolete](html-language-obsolete--877d6146487f--66870e9cdabe.md) |
| `sandboxing` | [flag definitions and directive parsing](html-sandboxing--877d6146487f.md) |

The [being-rendered definition and two notes](html-being-rendered--877d6146487f.md)
add a bounded selection from the same snapshot’s rendering introduction.
For HTML list-owner and ordinal interpretation, supplied participation means
associated CSS/SVG/equivalent layout boxes; viewport intersection, painted
visibility, positive geometry and `hidden` alone do not establish it. CSS-box
production is a separate supplied fact. The complete retained grouping-content
clauses define list ownership, starting values and ordinal numbering. This
selection adopts no layout geometry or browser rendering engine into HTML.

All §4 element definitions are retained across the root/metadata, text/link, embedded, forms/tables, interactive/scripting and host-fact captures. Common microsyntaxes, global attributes and content models are retained in foundations; complete syntax/parser recovery, named references, foreign parsing and fragment algorithms are retained in syntax/parser. Required obsolete parsing behavior is retained separately from author-conformance restrictions. Forms, table processing, template contents, foreign element declarations, optional end tags and void-element rules therefore have pinned source definitions without imposing a stricter Surgeist language dialect. Output plus diagnostics remains the intended parse result for malformed authored source. Author conformance, parser recovery, unsupported browser runtime capabilities, and cancellation/admission failure are separate contracts.

The [complete sandboxing section](html-sandboxing--877d6146487f.md) retains all
sixteen flag definitions and the directive parsing algorithm from this same
edition. The existing embedded-elements capture owns iframe attribute grammar
and its author-only redundancy rules. Intrinsic iframe interpretation projects
only the content directive: absent contributes no local restrictions; present
empty retains all sixteen. Unknown, repeated and redundant tokens do not
suppress recognized contributions. The active Document policy, ancestor/CSP
and popup composition, navigation and enforcement remain external operations.
This additional representation adds no source identity or edition change.

The [hosted Document activity clauses](html-hosted-document-activity--877d6146487f--f8564ea4758e.md)
retain the complete active-document definition (original70877–70880, including
its70876 wrapper), Document browsing-context definition and createDocument/
removed-iframe note (71636–71642), and fully-active definition plus child-navigable
interaction restriction (71919–71932). They use the same HTML877d original above;
bounded original-fragment SHA-256 is
`f8564ea4758e8af4bdd7ee661325dfe6565013d25796121e84e924efe8e225eb`.
For #136/#1124 this supplies the selected active-entry/Document/container relation
and preserves closed native createDocument semantics. It retains no surrounding
navigation chapter or subsequent example and adds no newer edition, browser
override or hosted lifecycle implementation.

The [element-task association clauses](html-element-task-association--877d6146487f.md)
retain task fields and runnability plus queue-task, global-task and element-task
algorithms from original lines81133–81144 and81195–81232 of the same HTML877d
snapshot. The original element's relevant global determines the queued task's
associated Document through the global-task algorithm; later nodeDocument changes
do not replace that recorded association. The bounded excerpt includes the
wrapper recommendation and adds no newer edition or whole event-loop selection.
Explicit Rust identity retirement remains the selected product policy in #136,
separately from these task source clauses.

The collection omits unrelated full browser runtime chapters (loading/navigation, browsing contexts, web application scripting APIs, workers, worklets, storage, rendering, and XML syntax) except the explicitly retained host facts, DOM parsing, scripting-state, bounded hosted Document activity and [native hosted lifecycle clauses](#native-hosted-lifecycle-and-clock). The supplemental [microdata encoding clauses](#remaining-intrinsic-html-source-clauses) retain section 5.2; section 5.3 vocabulary examples remain outside that capture. Original links resolve to the immutable snapshot. Complete element chapters necessarily include JavaScript interfaces and algorithms for script execution, fetch/media behavior and custom elements; their source retention does not require native bindings or runtime implementation in surgeist-html. Named external DOM, Infra, URL, Encoding, MathML, SVG and Web IDL specifications remain separate sources rather than being replaced by an HTML-only conformance assertion.

The previous live [language/direction](dom-html-language-direction--editor-capture-20261010--494ac341fe2b.md), [interaction](dom-html-host-interaction--editor-capture-20261010--8a01a8043e7c.md), [HTML selector facts](dom-html-host-selector-facts--editor-capture-20261010--07d421b207b3.md) and [table processing](style-specialized-determinations-html-tables--editor-capture-20261011--7e90e654e94f.md) captures remain historical source identities and original decision evidence. For unstarted HTML language and HTML-supplied host facts, this pinned baseline supersedes those bounded live source selections. This changes source selection, not completed implementation provenance or the separately selected WHATWG DOM foundation. The repository LICENSE pin previously associated with those live captures did not identify their served chapter bytes; the new immutable full snapshot does identify this retained HTML edition.

The implementation ingestion baseline remains [html5ever 0.40.1 at d7232d746d3112f08926d169591576bd12fe2fdf](html5ever-parser-contracts--d7232d746d31.md), with [supplemental source witnesses](html5ever-parser-supplemental-contracts--d7232d746d31.md). Engine ingestion must be reconciled with the selected HTML language rules, including processing-instruction syntax/tokenization; differences are implementation work, not permission to narrow the standard. Prototype measurements and differential samples remain bounded evidence rather than exhaustive conformance.

For [#1133](https://github.com/bj-data-eng/surgeist/issues/1133), the
[dimension-list cursor witness](html-dimension-list-cursor--webkit-73aa6c89e2cb-blink-7984f9d11800.md)
selects one bounded correction to §2.3.4.7: advance past a recognized decimal
point before collecting fractional digits and whitespace. The pinned WebKit
operation supplies the decisive implementation evidence; Blink corroborates
integer-prefixed decimals. Their overflow policies and other differences are
not adopted. This is a source-qualified Surgeist determination, without an
established upstream erratum. The normative capture retains its original text.

HTML's §2.3.6 legacy-color algorithm uses the 148 opaque named-color entries
from the retained [CSS Color 4 publication of 8 September 2026](css-color-4--CRD-css-color-4-20260908--bada647312a7.md#named-colors).
The existing external `cssparser` 0.37.0 named-color helper supplies that exact
name/RGB domain; HTML owns its surrounding legacy recovery steps. This selects
the bounded external fact table, without adding CSS color grammar validation or
changing the color-input state's separate CSS ownership.

### Srcset density conversion overflow

For [#1133](https://github.com/bj-data-eng/surgeist/issues/1133), HTML
`877d6146487f15b15bc8dd0e3c625063a37b456a` remains authoritative for
[srcset author grammar and processing](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md).
Its complete original lines 22673–22675 apply the floating-point conversion and
compare the result with zero, without specifying the conversion-error branch;
the [common conversion rules](html-language-foundations--877d6146487f--b55e73c31978.md)
at original lines 3325–3329 can return an error on overflow. The normative
captures retain that text unchanged.

The Architecture Coordinator's selected disposition uses WebKit revision
`73aa6c89e2cb77c46184a81aec944e4ab99d114d`: complete
[HTMLParserIdioms.cpp](webkit-html-parser-idioms--73aa6c89e2cb.md), original lines
219–241 (finite, complete conversion at 229–231), and complete
[HTMLSrcsetParser.cpp](webkit-html-srcset-parser--73aa6c89e2cb.md), original lines
134–140 (failed conversion rejected at 137–139) and 194–205 (candidate skipped
at 200–201). A failed density conversion is a recoverable candidate-local parse
error: omit that candidate, retain a diagnostic and continue. Thus
`a 1e999x, b 2x` retains only `b` at density 2; a lone overflow candidate
produces an empty candidate list with a diagnostic. No infinity, clamp, default
1x or descriptor salvage is selected. Processing still accepts density zero
and underflow to zero. Strict author positivity and density equality
use the mathematical decimal values independently of binary64 recovery.

This determination fills only that source omission. Other WebKit descriptor
grammar, integer limits, source-set normalization and image selection remain
unadopted. HTMLParserIdioms.cpp upgrades the existing partial identity to full
coverage while preserving its dimension-cursor representation; only
HTMLSrcsetParser.cpp adds an identity. Complete original headers and the
[file-specific legal notice](../licenses/webkit/NOTICE.md#srcset-density-conversion-witnesses)
accompany both bodies. This is not an upstream erratum or whole-engine parity
claim.

### Absent SVG document title

For [#1134](https://github.com/bj-data-eng/surgeist/issues/1134), HTML
`877d6146487f15b15bc8dd0e3c625063a37b456a` remains authoritative for title
selection, direct-child text and ASCII whitespace strip/collapse. Its
`document.title` getter's SVG branch does not specify the null-title case.
The [selected determination](https://github.com/bj-data-eng/surgeist/issues/1134#issuecomment-6117688267)
uses bounded [WebKit evidence](webkit-document-title--73aa6c89e2cb-metadata.md)
at `73aa6c89e2cb77c46184a81aec944e4ab99d114d`: a complete SVG `svg` document
element with no direct SVG `title` returns empty text without a selecting
element. Unknown required children remain missing information; an existing
empty title retains its identity. Neither deeper SVG descendants nor HTML
fallback titles are selected in this branch.

The witness retains exact original file hashes and inspected line ranges as
metadata. Document.cpp shares its already retained partial source identity;
Document.h adds one metadata-only identity. This determination adopts no
WebKit descendant-text extraction, canonicalization, mutation, scheduling or
notification behavior. It fills this source omission without claiming an
upstream erratum, complete SVG behavior or a different HTML edition.

### Foreign popover target interface guard

For [#1134](https://github.com/bj-data-eng/surgeist/issues/1134), HTML
`877d6146487f15b15bc8dd0e3c625063a37b456a` governs generic reflected Element
association, pointer scope and ordinary-tree first-ID selection. Its guarded
popover-target operation reaches an interface-specific step after selecting
an Element. The
[selected determination](https://github.com/bj-data-eng/surgeist/issues/1134#issuecomment-6117820267)
uses [bounded WebKit metadata](webkit-popover-target--73aa6c89e2cb-metadata.md)
at `73aa6c89e2cb77c46184a81aec944e4ab99d114d`: a selected non-HTML Element
returns null before popover-state inspection. The generic association retains
that Element; no later matching HTML ID is searched.

The witness identifies HTMLFormControlElement.cpp's complete original hash
and original lines 335–362, with the interface downcast at 348. Its private
original carries GNU Library GPL notices; this public record contains only
source identity and independent behavioral summary, with no code excerpt.
The selection adopts no other browser target, activation, form or popover
behavior and does not claim an upstream erratum or whole-engine parity.

### Bounded referrer-policy vocabulary

For [#1133](https://github.com/bj-data-eng/surgeist/issues/1133), the [Referrer Policy vocabulary](referrer-policy-vocabulary--cc435b05ca4a.md) selects the finite enumeration from W3C editor-source revision `cc435b05ca4a94f7f1a139be5074b168d20014db` (20 March 2026). The complete original source SHA-256 is `a277e78e06418ec60afc6f499aee79d4f3a3a067e0fd97e9886a2d9e3de95eb1`. Eight nonempty keywords and the empty state are retained. Pinned HTML §2.5.5 supplies ASCII-insensitive whole-value matching without trimming and missing/invalid empty-state defaults. Empty remains unresolved; the implementation does not substitute the external draft's default policy. This selection adopts no network, Fetch, referrer-generation, policy-container, URL or header-list algorithm. The older 2017 CR's surrounding processing model is not mixed into this selection.

### HTML language and directionality sources

For [#1134](https://github.com/bj-data-eng/surgeist/issues/1134), pinned HTML
§3.2.6.2 selects the effective language tag. The
[RFC 5646 boundary reference](bcp47-html-language-boundary--rfc5646.md) identifies
the distinctions between syntax, registry-qualified validity and private
interpretation. Selecting and preserving the exact nonempty tag does not claim
language recognition, validity, installed locale data or service support. An
empty selection represents explicitly unknown language; no universal recognized
language enumeration or registry filter is required for this operation.

Pinned HTML §3.2.6.4 supplies directionality, including first-strong tests on
text, current control values and assigned slot content. The selected property
edition is final Unicode 18.0.0, UAX #9 revision 52 (1 September 2026), with the
complete [DerivedBidiClass data](unicode-derived-bidi-class--18.0.0.md), SHA-256
`d9e23222522551348ea1ccfbb4f62efbf98982afb95840f8959c08ed992c5607`.
Both general and range-specific `@missing` defaults apply; isolated surrogate
code points have class L. HTML requires only L, R, AL and the remaining classes
for its first-strong decision. This selection introduces no full bidirectional
layout algorithm or text-service dependency. The
[UAX #9 record](uax9--18.0.0--rev52-metadata.md) is metadata only because the
report's publication terms do not authorize public reproduction of its text.
The separately licensed data remains available for offline generation and
verification.

### HTML inertness and DOM flat-tree facts

Pinned HTML §6.3.1–6.3.2 owns inertness. DOM supplies qualified flat-tree facts
under [CSS Shadow §4.1](selectors-execution-css-shadow-1--editor-capture-20261010--b02c47a6b167.md#flattening),
source revision `50337328484f7fcdd9ab6fe9ded6011815b1707e`; HTML does not derive
flattening from ordinary parent links. The
[pinned WebKit modal witness](html-modal-inert-escape--webkit-73aa6c89e2cb.md)
resolves one bounded question: the current blocking/topmost modal subject
resets inherited attribute inertness. A lower modal dialog does not receive an
independent escape boundary. The blocking subject's own applicable `inert`
attribute still establishes inertness.

The selected document flat-tree relation excludes detached nodes. That known
exclusion does not establish empty inherited inert ancestry. Until a separately
qualified producer supplies detached ancestry, a query requiring it reports
unavailable context. An own applicable `inert` attribute can establish true
without that ancestry. Disconnection removes document modal blocking, not all
inertness causes. This is a producer capability limit, not a new HTML flattening
rule or a successful false result.

### Bounded HTML editability and spellcheck readback

For [#1134](https://github.com/bj-data-eng/surgeist/issues/1134), the
[complete selected editing/design-mode/spellcheck sections](html-editability--877d6146487f.md)
retain the existing HTML revision. HTML owns editing-host classification and
spellcheck IDL readback. The delegated editable definition selects only the
[bounded execCommand definitions](execcommand-editability--85de9ff4a981.md) at
w3c/editing `85de9ff4a9816139d06fc2dc39d5dacdc456fddc`, complete source SHA-256
`da222a4ea70349eba9d9036a20708f1fa63690118f6c657da3f7f06c56fa78cf`.
An editing host is distinct from editable; isContentEditable is their union.
Actual captured document design mode and established per-element spellcheck
defaults are supplied facts, never inferred from scripting state or text
preferences. Ordinary parent relations govern editability; source-defined
heading-offset traversal separately crosses a shadow root to its captured host.
This source selection implements no editing command, selection mutation,
spelling service or JavaScript runtime.

For [#1124](https://github.com/bj-data-eng/surgeist/issues/1124), the
[nullable active-range reference](designmode-active-range--877d6146487f.md)
retains the delegated nullable definition and DOM boundary-point/live-range
definitions from the existing pinned editions. Its WebKit witness is behavioral
metadata only. The issue records the explicit Surgeist null-range policy
separately from those source definitions; the source does not specify that
branch. Non-null ranges retain their actual identity and live-update obligation.

### Initial hosted Permissions Policy creation

For [#1218](https://github.com/bj-data-eng/surgeist/issues/1218), select
Permissions Policy at W3C commit
`29398046f1e0cb45c091adda853df74925dc740a` (22 September 2026), complete
`index.bs` SHA-256
`51a87c5f4008510e501b5a4fe553baf8271e0e3af6ce6e8feece987639794c9a`.
The [bounded creation reference](permissions-policy-native-creation--29398046f1e0.md)
retains policy records, allowlist matching, declared origin, iframe attributes
and initial inheritance algorithms, plus opaque-origin equality from the
existing HTML edition. The selected native feature roster and opaque-origin
projection are recorded separately in #1218. This does not select response
policies, reporting, arbitrary-origin introspection or general navigation.
The existing pinned Fullscreen definition supplies its feature and default.
For #1225's deliberate focus-policy extension, the same bounded reference also
retains HTML `877d6146487f` raw line 3029: `focus-without-user-activation` has
default allowlist `'self'`. Its allowed-to-use and allow-focus algorithms remain
in the existing embedded-element and host-fact conversions. This adds a clause
from the existing HTML pin, not a new edition or a broader policy profile by
source conversion alone; the consuming implementation owns its selected roster.

### Native hosted lifecycle and clock

For [#136](https://github.com/bj-data-eng/surgeist/issues/136) and
[#1124](https://github.com/bj-data-eng/surgeist/issues/1124), the
[navigable lifecycle reference](html-hosted-navigable-lifecycle--877d6146487f--3a9658155866.md)
and [initial Document/global/agent/settings reference](html-hosted-global-agent-settings--877d6146487f--03a314e0aa55.md)
reuse the existing complete HTML877d source identity. They retain complete
selected creation, destruction-caller, history-record, make-active and
environment definitions and algorithms, including JavaScript-only steps.
Their raw ranges and original-fragment hashes are recorded in each reference;
literal HTML bodies preserve source wording and structure. They do not establish
transitive source closure or implement every referenced service.

The [initial teardown clauses](html-hosted-initial-teardown--877d6146487f--c2dd632ac56d.md)
reuse that same HTML identity for actual Document destruction, abort, unloading
cleanup, related navigable collections, initial Navigation state and task-source
definitions. The source's raw75916 singular wait-variable discrepancy is kept
literally; #136 separately qualifies the native predicate as the captured plural
child-list size, including zero. A satisfied predicate does not skip the parent
global task or cleanup. Unreached child-history application and the complete
browser event-loop processing model remain outside this capture.

The [native clock clauses](hr-time-native-clock--1f0b9faf793d--5f2276cce0af.md)
select W3C High Resolution Time commit
`1f0b9faf793db3b06c8e85462448e8a812d00fb9` (1 September 2026), preceding the
HTML pin. Complete `index.bs` SHA-256:
`e928551146aa839cb2d6da490b4c2b07921b596d92514c8a37c6062eac6bddde`.
Four complete selected source units cover clocks/moments/durations, time origin,
coarsening, and relative/current/shared time, with literal Bikeshed source.
Both isolation branches, optional jitter and Unix-aligned exposure definitions
remain visible even where they lie outside the native API.

The native Rust projection is recorded separately in #136 under “Native hosted
execution: Global, settings and agent ownership”. It requires actual native
Globals, agents/event loops, settings, canonical Document associations and an
Instant-backed shared epoch with selected 100-microsecond flooring, while
excluding JavaScript realm execution and Performance/Unix exposure. Retained
source algorithms are not rewritten to make those exclusions normative no-ops.
This source selection certifies neither implementation nor browser conformance.

### Native hosted entry UUID generation

The [random UUID reference](webcrypto-random-uuid--811c24c69eb2--1c697c6b1d48.md) selects W3C Web Cryptography commit
`811c24c69eb22d477af5f1678cb70dbc611c7c40` (11 August 2026), preceding HTML877d.
Complete `spec/Overview.html` SHA-256:
`00a094ac4298f0c7a5a4afe599281827c98fefbe72b09756f1f5859746f52c97`.
It supplies the named “generate a random UUID” algorithm reached by the retained
[HTML session-history-entry definition](html-hosted-navigable-lifecycle--877d6146487f--3a9658155866.md#complete-session-history-entry-items-and-document-getter).
The complete Crypto IDL, entropy notes, randomUUID method, generation algorithm
and hexadecimal definition remain literal source, including the exposed
method's `[SecureContext]` attribute and original RFC4122/RFC2141 citations.

This selects a bounded external algorithm, not full Web Crypto, JavaScript
execution, a new general secure-context guard or a newer UUID RFC. The native
entry-identifier owner, actual random-source failure, Pending custody and
no-collision-retry contract are determined separately in
[#136](https://github.com/bj-data-eng/surgeist/issues/136).
The exact repository declaration selects the W3C Document License 2023 for
prose; marked IDL has the separate Code Components terms described in the
[W3C notice](../licenses/w3c/NOTICE.md#native-hosted-random-uuid-clauses).
Source retention does not certify an implementation or a cryptographic sample.

### Child navigable history and traversal support

The [child history/traversal reference](html-child-navigable-history--877d6146487f--a9f635b298df.md) reuses HTML877d,
complete original SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
Eleven fragments retain complete History/latest-entry definitions, Navigation
aborting and child-destruction notification bodies, traversal queue and append/
runner definitions, creation/destruction history update, full apply-history-step
and its lookup helpers. Actual queued/global-task execution, History writes,
completion waits and source order remain obligations; retaining JavaScript-only
branches does not turn them into native no-ops.

The existing [child lifecycle callers](html-hosted-navigable-lifecycle--877d6146487f--3a9658155866.md),
[Iframe hooks](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md)
and [Frame hooks](html-language-obsolete--877d6146487f--66870e9cdabe.md)
remain separate unchanged references. The new reference separately records
[#1124](https://github.com/bj-data-eng/surgeist/issues/1124)'s caller-type
qualification: invoke the Document helper on the element's actual current node
Document D, obtain D.nodeNavigable N and append N to the source ancestor list.
N is the containing parent, not the new child; missing or retired N is not an
empty list. The literal source's Document/Navigable mismatch is not rewritten.

For no srcdoc and absent/empty src under an actual initial-about:blank parent,
the ancestor URL match returns null before the later URL/history, load and lazy
branches. This does not waive earlier #1218 policy work: present-empty src can
still require actual URL parsing/origin evidence in that separate phase.
Conditional URL/history replacement/finalization and later navigation branches
are not selected by this retention. No new HTML source identity, changed edition
or implementation/conformance verdict is implied.

### Initial opener-policy values and record

The [opener-policy reference](html-opener-policy-defaults--877d6146487f--d124556cde21.md) selects complete
raw64771–64827 from the existing HTML877d edition, original SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
It retains the complete value list and informative notes, followed by the
four-field record: value unsafe-none, reporting endpoint null, report-only
value unsafe-none, report-only reporting endpoint null. Native creation without
an opener does not select the distinct value noopener-allow-popups.

The [HTML language producer](../crates/surgeist-html/src/opener_policy.rs)
owns the exact five `HtmlOpenerPolicyValue` spellings and the immutable
`HtmlOpenerPolicy` initial record. This qualifies language values and defaults;
it establishes no Document provenance, actual parent association or policy
effects. The retained [BC/Document creator](html-hosted-global-agent-settings--877d6146487f--03a314e0aa55.md)
supplies the separate source condition and actual top-level active Document
policy inheritance. Consumers must copy that actual source-selected record
when the condition holds, rather than manufacture a fresh default from a
no-opener label.

Matching64829 onward, header parsing, response enforcement/reporting and
group-switch algorithms are outside this selection. Retaining all value
definitions does not admit their broader navigation effects or certify native
child lifecycle implementation. Existing source identities and edition remain
unchanged.

### Legacy Frame creation sandbox contribution

The [WebKit Frame witness](webkit-frame-sandbox--73aa6c89e2cb-metadata.md) records static metadata for nine
complete source files at the existing preferred WebKit revision
`73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Its exact Git blobs and line
locators corroborate empty legacy Frame element-specific flags while preserving
the parent's actual active sandbox restrictions. The witness is not a copied
browser implementation or an executed conformance test.

The selected [HTML sandboxing source](html-sandboxing--877d6146487f.md)
defines its sandbox slot only for Iframe but creation union raw65785–65794
refers to it for an element embedder. [#1124](https://github.com/bj-data-eng/surgeist/issues/1124),
“Legacy Frame: creation sandbox contribution”, supplies the separate bounded
qualification: only actual Iframe contributes its iframe-owned slot; legacy
Frame contributes an empty element-specific set, unioned with its current node
Document's actual active flags. Empty Frame contribution does not erase inherited
restrictions, and a raw sandbox attribute on Frame acquires no Iframe semantics.
Literal HTML remains unchanged; no upstream fix or newer edition is claimed.

This selects neither WebKit loader/IPC/remote-frame behavior nor arbitrary
frame-owner families, navigation, CSP or referrer-policy implementation. The
COOP language record/defaults remain separately qualified above; neither source
selection certifies a complete native child lifecycle.

### Child destruction and queued history lifetime

The [child lifetime reference](html-child-history-lifetime--pinned-browser-metadata.md)
combines the existing pinned HTML clauses with dated upstream discussion and
six metadata-only browser witnesses at the existing WebKit/Chromium revisions.
It identifies the lack of a late nested-history prune, cleanup-task eligibility
and canceled nonchanging-job settlement in the selected HTML text. The browser
witnesses support distinguishing live membership from retained cleanup ownership
and task execution from cancellation; they do not implement that exact HTML
counter algorithm or select a replacement rule.

Keep the literal HTML unchanged. Native lifetime and completion qualifications
belong to [#1124](https://github.com/bj-data-eng/surgeist/issues/1124) and
[#136](https://github.com/bj-data-eng/surgeist/issues/136). These source
observations alone establish neither implemented behavior nor child-lifecycle
readiness. No newer standard, browser loader/unload schedule, conditional
history pruning or general navigation service is adopted by this reference.

### HTML autocomplete credential recovery

For [#1133](https://github.com/bj-data-eng/surgeist/issues/1133), the
[bounded WebAuthn edge determination](html-autocomplete-webauthn--877d6146487f.md)
selects literal processing under the existing HTML revision. Bare `webauthn`
is author-invalid but recovers successfully; `username webauthn` retains the
terminal credential token as its processed field. The source's authored detail
grammar and processing recovery remain separate. Exact field and IDL token
casing follows HTML; the corroborating WebKit witness does not select blanket
lowercasing. The recorded upstream defect remains unresolved by the selected
edition. No browser-source excerpt, credential mediation or alternate
compatibility mode is incorporated.

### Disabled script preparation and standalone checkpoint sources

For [#1136](https://github.com/bj-data-eng/surgeist/issues/1136), the
[bounded MIME Sniffing definitions](mimesniff-javascript-essence--39aa53511b13.md)
select revision `39aa53511b13953d84fef8d4131d6f61d0ccbde6`, displayed
17 July 2026, complete original HTML SHA-256
`11caac49505995bcef51153a827c6fe96464111c8914e2c399d9e8fe9a467cd7`.
Only the exact 16 JavaScript MIME essence strings and their ASCII case-insensitive
string-match hook are selected. This is neither MIME-record parsing nor sniffing:
parameter stripping is not part of an essence match. HTML877d retains authority
over default type/language branches, its separate explicit-type ASCII-whitespace
stripping step, module/importmap/speculationrules recognition, script metadata
and the Disabled return before fetch/execution.

The [complete microtask checkpoint](html-microtask-checkpoint--877d6146487f.md)
retains queue/reentrancy definitions and all cleanup/timing hooks from that same
HTML edition. Disabled scripting alone does not establish an empty checkpoint.
The selected private standalone specialization requires the actual fresh
construction owner to establish absent executable microtasks, settings/globals,
IndexedDB transactions, kept JavaScript objects and running-script/frame-timing
domains, with no effect-producing callbacks or host integration. Structural
construction effects and script metadata remain required; cancellation/work
admission remains in source order. Captured/hosted execution does not acquire
this proof from an immutable capture or Disabled state.

[Long Animation Frames §3.2.2](https://w3c.github.io/long-animation-frames/#record-timing-info-for-microtask-checkpoint)
is an explanatory live citation inspected on 2026-10-11. No original bytes or
content hash were retained, and no timing-source commit is selected or counted.
The retained HTML checkpoint establishes that the hook exists. The absent-domain
disposition is an explicit architecture capability specialization, not an
implementation of the external timing algorithm or a normative null-script
return. These source selections do not certify a complete parser or browser
event loop.

### Offline Disabled SVG script and clone sources

For [#1136](https://github.com/bj-data-eng/surgeist/issues/1136), HTML877d's
foreign-content script-end operation delegates to the
[SVG Tiny 1.2 processing steps](svg-tiny12-script-processing--REC-20081222--2f36eb76440c.md#ScriptContentProcessing),
W3C Recommendation 22 December 2008. The partial reference retains the complete
selected four-step algorithm, load-notification qualification and subsequent
modification rules, with the edition's original notice and status. This selects
the delegated operation, not the whole SVG Tiny profile.

The [recorded offline reconciliation](https://github.com/bj-data-eng/surgeist/issues/1136#offline-disabled-svg-script-explicit-processing-reconciliation)
suppresses all acquisition and execution through the fresh standalone owner's
explicit closed capabilities. Tiny's already-processed and connectivity guards
remain; an eligible element still acquires its actual processed bit. Suppression
is separate from a failed fetch and does not classify or decode URLs. Neither
SVG2 script-disabled nor external-reference-disabled alone establishes this
stronger boundary. The [SVG2 observations](svg-script-observations--capture-20261012-metadata.md)
retain dated URL/hash provenance without selecting another production edition
or a named SVG processing mode. No HTML preparation algorithm or HTML script-end
microtask checkpoint is imported into this SVG operation. Any removed load
notification requires actual absence of recipients and retained effects.

The [clone determination](https://github.com/bj-data-eng/surgeist/issues/1136#svg-script-cloning-preserve-the-processed-bit-by-bounded-correction)
selects a bounded correction for the
[DOM source's missing SVG clone steps](dom-foundation--editor-capture-20261010--08dff0a4827f.md#concept-node-clone-ext).
The [pinned WebKit witness](webkit-svg-script-clone--73aa6c89e2cb-metadata.md)
supports copying the actual at-most-once bit before clone insertion, with the
clone marked non-parser-inserted. It contributes no browser source excerpt or
combined HTML/SVG preparation algorithm. Same-registry qualified true/false facts
are required; unknown does not become false. Non-parser insertion retains actual
document-order processing and topology effects without the parser wrapper.
These source selections do not certify implemented SVG script handling or a
complete parser.

### HTML link destinations and Fetch vocabulary

For [#1133 A010](https://github.com/bj-data-eng/surgeist/issues/1133#a010-link-destinations-selected-fetch-vocabulary),
HTML revision `877d6146487f15b15bc8dd0e3c625063a37b456a` delegates its module-preload
destination vocabulary to the [selected Fetch definition](fetch-script-like-destinations--e9460d1b22ea.md#request-destination-script-like)
at immutable revision `e9460d1b22ead1c6a72eeac37fb2ec0dc01aeb1d`.
The finite script-like set is audioworklet, paintworklet, script, serviceworker,
sharedworker and worker. The adjacent warning does not include xslt in that set.
HTML adds json, style and text, giving nine module-preload destinations. Its
union with fetch/font/image/script/style/track gives thirteen `as` states.

HTML owns whole-value ASCII-insensitive enumeration without trimming and without
missing, invalid or empty defaults. Later modulepreload processing selects
script for no state, including invalid values; a recognized image state stays
image and fails membership rather than recovering to script. Preload and
modulepreload author constraints remain independent when both rel tokens occur.
This is one bounded external vocabulary selection, not a new HTML edition,
Fetch translation/loading, worker execution or JavaScript binding. Source
selection does not certify the corresponding HTML operation as implemented.

### DOM-delegated XML pseudo-attribute grammar

The retained [DOM initialization](dom-foundation--editor-capture-20261010--08dff0a4827f.md#processinginstruction-initialize)
and [PI attribute-map update](dom-foundation--editor-capture-20261010--08dff0a4827f.md#update-attributes-from-data)
delegate parsing an existing data string to the
[whole XML-stylesheet §3](xml-stylesheet-pseudo-attributes--REC-xml-stylesheet-20101028--a0e1a5198a21.md#pseudo-attributes),
W3C Recommendation 28 October 2010, Second Edition. Its complete original
SHA-256 is `a0e1a5198a21c2aa4b623fc35b3c890d6e51c6c14ee126aed71cf3b93157515a`.
The [XML Fifth Edition grammar dependency](xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md)
selects the 26 November 2008 Recommendation; complete original SHA-256
`f58a4b9e1e5b8bac13fb55c4e5c7c8c5e2c176aa45ee68e2f1ab2498511927ec`.
That retained original explicitly discloses its 7 February 2013 RFC-link
correction. Neither selection upgrades the pinned HTML edition.

The XML dependency retains complete Char and S productions, exactly the three
complete NameStartChar/NameChar/Name rows, CharRef with its Legal Character
constraint and numeric interpretation, and complete §6 Notation. Negated classes
range over Char; discouraged-but-permitted characters are not extra errors.
Parsing is whole-string with exact names, the five predefined entity references
and numeric references, three prescribed value modifications, and duplicate or
illegal-character-reference failure. Empty sets and quoted empty values remain
permitted. DOM owns map clearing before parse and leaving it empty on error.

This does not select XML-stylesheet §4 style loading, XML document EOL
preprocessing, DTD/external entities, generic attribute normalization, case
folding or Unicode normalization. XML name reservation prose remains context,
not an additional Name-production exclusion. The partial references preserve
complete original STATUS and copyright/editor material and include the separate
2015 software-support derivative notices with complete local document terms.
They are not replacement technical specifications. Source retention does not
certify PI construction, runtime-source-index accounting, cancellation or
ordinary-template parser implementation.

### Fresh standalone EOF lifecycle sources

For [#1136](https://github.com/bj-data-eng/surgeist/issues/1136), the existing
HTML877d snapshot remains the language authority. The
[complete spin-event-loop algorithm](html-spin-event-loop--877d6146487f.md#spin-the-event-loop)
retains the whole wrapper and all four notes at original raw 81620–81655,
including the checkpoint before the parallel wait and queued task continuation.
The complete original is 14,930,614 bytes, SHA-256
877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f.
This adds a rendering to that existing identity; neither the language revision
nor catalog counts change.

The existing [end-of-parsing clause](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#the-end),
[readiness operation](html-language-foundations--877d6146487f--b55e73c31978.md#update-the-current-document-readiness)
and [complete checkpoint](html-microtask-checkpoint--877d6146487f.md) remain
separate owning source operations. The selected fresh standalone completion
reduces structurally absent browser domains explicitly; those eliminations are
a capability selection, not inserted normative early returns. In particular,
the spin source's checkpoint occurs even if its wait goal is already met.
Retaining this reference does not implement task scheduling, JavaScript stacks,
global event delivery, loading, a live Document lifecycle or complete parser
success. Hosted/captured/shadow domains require their own qualified facts.

## Counter image output determination — 2026-10-12

[#707](https://github.com/bj-data-eng/surgeist/issues/707) owns the selected
text-only consumer policy. The [Counter Styles Editor’s Draft, 8 October 2026](css-counter-styles-3--editor-capture-20261009--ef29d0a06a13.md)
remains the complete selected reference: a fresh check found identical
whitespace-normalized text in sections 2, 3.7, 3.8 and 3.9. Image symbols remain
at risk, and the general text-only conversion is not defined. The defined
speech fallback is not a general string-conversion algorithm.

The [current Lists 3 section 4.7](counter-image-output-css-lists-3--editor-capture-20261012--3059ea0c1e88.md)
retains string-or-image output and counter-stack ordering. Its bounded Markdown
conversion passed ID, link, prose and literal checks: 64 IDs, six literal
blocks, no tables. It does not adopt unrelated authored grammar changes.

The [pinned WebKit and Blink witness](counter-image-output-browser-witness--webkit-73aa6c89e2cb-blink-7984f9d11800.md)
records gated image syntax but text-only representation paths; it establishes
no complete browser image-to-string precedent. Therefore #707 explicitly
selects per-value same-integer fallback traversal for an image-bearing result
when the consumer requires text, preserving rich image output and the separate
normative speech rules. This is Surgeist policy, not a CSSWG resolution or
browser-parity claim. #534/#693/#694 own implementation and tests; resolving the
source decision does not complete them.

## HTML best-number serialization

[Architecture selection in #1134](https://github.com/bj-data-eng/surgeist/issues/1134#html-best-number-representation-bounded-ecmascript-2026-delegation),
also [referenced by parser #1136](https://github.com/bj-data-eng/surgeist/issues/1136#best-number-serialization-source-owner),
selects the dated ECMAScript 2026 (ECMA-262 seventeenth edition) ToString Number
branch → Number::toString radix 10. The
[bounded conversion](ecmascript-number-serialization--2026--cd65f2eaa201.md)
retains the complete two operations, Notes 1–3, and the exact Number/binary64
rounding and required mathematical definitions from three hashed official pages.
The actual publisher's Draft ECMA-262 / March 31, 2026 header remains visible;
the selected dated edition does not mean the rolling tc39.es/ecma262/ source.

The owning determination requires Note 2's optional ECMA recommendation as
Surgeist policy: shortest roundtripping decimal significand, nearest exact-real
candidate, then even decimal significand for an equal-distance tie. Radix-10
n/k formatting remains normative source behavior: fixed notation iff
-5 ≤ n ≤ 21, otherwise lowercase e with explicit exponent sign and no leading
exponent zeroes; both signed zeroes emit 0. Finite HTML inputs exclude ECMA's
nonfinite text branches. Number's valid authored strings remain unchanged;
Range uses best representation only where its actual source algorithm requests it.

The [separate Range arithmetic selection](https://github.com/bj-data-eng/surgeist/issues/1134#range-exact-arithmetic-and-the-finite-number-join)
chooses nearest finite representable exact-dyadic lattice candidates within
actual bounds, ties toward the larger number. That is a distinct Surgeist
interpretation, not an ECMA Range algorithm or decimal/tolerance browser-parity
claim. Unconstrained exact-real → Number conversion uses binary nearest/even;
decimal serialization uses Note 2's decimal nearest/even choice. No intermediate
rounding may be presented as proof of Range lattice membership.

The [Ecma notice and full terms](../licenses/ecma262/NOTICE.md) accompany this
bounded conversion. No JS runtime, BigInt/object coercion, other-radix behavior,
dtoa code or dependency is selected. This reference supplies source evidence,
not implementation or whole HTML/ECMA conformance acceptance.

## Remaining intrinsic HTML source clauses

The [nonce attributes](html-nonce--877d6146487f.md#nonce-attributes) and
[microdata encoding](html-microdata--877d6146487f.md#encoding-microdata) captures
retain complete sections 2.5.6 and 5.2 from the existing HTML877d edition for
[#1168](https://github.com/bj-data-eng/surgeist/issues/1168) and
[#1166](https://github.com/bj-data-eng/surgeist/issues/1166). They were extracted
offline on 13 October 2026 from the same complete original retrieved on
11 October, SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
No new standard edition is selected.

Nonce retains the independent content attribute and cryptographic slot, IDL
getter/setter, exact attribute-change hook, conditional browsing-context-connected
hiding and slot restoration, and cloning. Browsing-context connection and mixin
eligibility remain governed by the retained foundations and the same snapshot.
HTML consumes actual policy facts; this source does not assign CSP parsing or
enforcement to the HTML crate.

Microdata retains the generic model, item/type/identifier/reference relationships,
all property-name alternatives, property-value selection, the property crawl,
document cycle/orphan obligations and namespace. Section 5.3 vocabulary examples
are excluded. URL/encoding and vocabulary facts keep their actual external
owners; source retention does not certify those facts or implementation completion.

The [conversion report](CONVERSION-REPORT.md#intrinsic-html-supplements--2026-10-13)
records bounded extraction and fidelity checks. Specification prose remains
CC BY 4.0 under the [WHATWG notice](../licenses/whatwg/NOTICE.md#intrinsic-html-source-supplements--2026-10-13),
independently of BSD terms for source-code incorporation.

## Document policy-container initial state

The [bounded initial-state reference](html-policy-container-initial-state--877d6146487f.md)
retains the selected HTML877d Document default and complete policy-container
item/default list. It supplies source evidence for native Meta policy history
under [#1124](https://github.com/bj-data-eng/surgeist/issues/1124), alongside the
existing [Referrer Policy vocabulary](referrer-policy-vocabulary--cc435b05ca4a.md).
It does not establish fetched/parsed history, CSP enforcement or other policy
runtimes. The complete source hash and conversion limits are recorded in the
[conversion report](CONVERSION-REPORT.md#document-policy-container-initial-state--2026-10-16).
No new edition is selected.

## Hosted same-site definitions

The [bounded same-site reference](html-hosted-same-site--877d6146487f--d13a3b5702d5.md)
retains the complete site and origin comparison algorithms from the existing
HTML877d edition. It completes the definitions reached by the retained
[Window Settings setup and initial constructor](html-hosted-global-agent-settings--877d6146487f--03a314e0aa55.md).
Obtain-a-site returns an opaque origin unchanged; same-site then compares its
actual identity. Serialized `null` is not an identity comparison.

Cross-site ancestry uses current parent navigable and active-Document Settings;
top-level origin is the distinct original constructor operand. Native consumers
retain those roles and their finite qualified captures. This source addition
selects no new origin profile, URL/public-suffix service or standard edition.
The [conversion report](CONVERSION-REPORT.md#hosted-same-site-definitions--2026-10-16)
and [WHATWG notice](../licenses/whatwg/NOTICE.md#hosted-same-site-definitions--2026-10-16)
record exact extraction and attribution.

## URL equality and serialization

The [bounded URL reference](url-equality-serialization--fde3f74f0633.md) selects
URL Standard snapshot `fde3f74f063341a28d437216f75735b8b40128f5` (7 October 2026).
Complete original `url.bs` SHA-256:
`7f45ea5b6167f08a098619167254a99a3c8b5fd6514fff4963ede4a7c8e05818`.
Its complete selected units supply the serializers and URL equality reached by
HTML877d's ancestor-URL guard, plus the representation and encoding closure.
No replacement HTML edition, URL parser or general URL engine is selected.

URL equality compares actual serialized values with the requested fragment
exclusion. Queries, empty query delimiters, percent spelling and path spaces
remain significant. Record identity and HTML's matches-about:blank classification
are distinct predicates. The record model also permits paths produced outside
the URL parser, so the first literal number sign in a full serialization does
not necessarily delimit a fragment.

[#1124](https://github.com/bj-data-eng/surgeist/issues/1124) therefore requires
HTML-Semantics to retain the original producer's actual fragment boundary and
compare borrowed prefixes with finite admission and cancellation. A known-null
fragment uses the full serialization; a present fragment supplies its actual
UTF-16 delimiter offset. The producer attests that same-record boundary, while
the constructor checks its range and designated number-sign unit. No comparison
may infer the boundary, substitute identity, normalize text or turn unavailable
work into inequality. The consumer still qualifies each source-selected record
and revision independently. This selected implementation boundary is separate
from the literal URL algorithms and does not certify implementation completion.

The [conversion report](CONVERSION-REPORT.md#url-equality-and-serialization--2026-10-16)
and [WHATWG attribution](../licenses/whatwg/NOTICE.md#url-equality-and-serialization--2026-10-16)
record the bounded extraction and unchanged full license.

## Close-watcher cancel result

The [bounded cancellation reference](html-close-watcher-cancel-result--877d6146487f.md)
retains the cancel-action contract, complete request-to-close and close
algorithms, and Dialog's supplied cancel callback from the existing HTML877d
edition. The literal source's contradictory final `false` in the descriptive
cancel-action item remains visible. [Issue #1225](https://github.com/bj-data-eng/surgeist/issues/1225)
owns the narrow correction to `true`, based on the same edition's actual
callbacks and the pinned DOM event-return rules; no new edition or browser
override is selected. This interpretation does not establish a completed
request-close implementation.

The [conversion record](CONVERSION-REPORT.md#close-watcher-cancel-result--2026-10-17)
and [WHATWG notice](../licenses/whatwg/NOTICE.md#close-watcher-cancel-result--2026-10-17)
identify the bounded selection and existing license.

## Window close-watcher manager

The [bounded manager reference](html-close-watcher-manager--877d6146487f.md)
retains the existing HTML877d Window manager defaults, watcher definition and
active-membership predicate, complete establishment and destruction algorithms.
It preserves the literal descriptive cancellation boolean; the separately
retained [cancel-result interpretation](#close-watcher-cancel-result) remains
unchanged. User activation and platform processing are outside this selection.
The [conversion record](CONVERSION-REPORT.md#close-watcher-manager--2026-10-17)
and [WHATWG notice](../licenses/whatwg/NOTICE.md#close-watcher-manager--2026-10-17)
record the exact units. No new edition, source identity or implementation
completion is asserted.

## Dialog modal assertion comparison

The [dated current-source comparison](html-dialog-modal-assertion-comparison--2026-10-17.md)
retains the inactive open-hook return and selected post-event ShowModal steps
observed on 17 October 2026. These clauses preserve the pinned edition's order:
the open hook can return on inactivity, while ShowModal later asserts a nonnull
watcher without a second post-event activity check.

No replacement HTML edition or source correction is selected.
[Issue #1225](https://github.com/bj-data-eng/surgeist/issues/1225) owns the explicit
native failed-assertion disposition; the source comparison does not supply one.
The [catalog entry](SOURCE-CATALOG.md#dialog-modal-assertion-comparison--2026-10-17)
and [conversion record](CONVERSION-REPORT.md#dialog-modal-assertion-comparison--2026-10-17)
identify the bounded transcription and its limits.

## Fresh native Document focus initialization

[Issue #1220](https://github.com/bj-data-eng/surgeist/issues/1220) selects a
known-null focused-area designation for fresh native closed and hosted-initial
Documents. The pinned HTML focus model permits a null designation in its
currently-focused-area resolver but does not state a fresh initialization.
The [pinned WebKit witness](webkit-document-focus-initialization--73aa6c89e2cb.md)
corroborates the explicit native choice through Document's member declaration,
complete constructor and RefPtr's default-null initialization. This is a bounded
native qualification, not a missing HTML normative default.

Null designation, viewport designation, unavailable imported history and absent
system focus remain distinct. Resolving null designation to the candidate
Document does not write a viewport designation. The current-standard comparison
selects no replacement HTML edition. The witness's original legal headers and
[WebKit attribution](../licenses/webkit/NOTICE.md#fresh-document-focus-initialization)
cover its documentation excerpts; no browser source is compiled or linked.
