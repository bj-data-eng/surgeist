# Text preparation, selected lines and renderer admission

This is the normative implementation contract selected for [#1080](https://github.com/bj-data-eng/surgeist/issues/1080), [#1082](https://github.com/bj-data-eng/surgeist/issues/1082), [#72](https://github.com/bj-data-eng/surgeist/issues/72) and [#73](https://github.com/bj-data-eng/surgeist/issues/73). It assigns behavior to the existing text, layout, render, window and runtime owners; it introduces no crate or interchangeable provider protocol. It does not settle the separate CSS feature issues or claim that Parley implements those standards. Preserve supported text behavior while replacing the Parley intermediary, with the explicit source representation and offset change below. This is a pre-v1 replacement without compatibility machinery.

## Evidence and authority

The repository owns this cross-crate contract; its consumer implementations follow the ownership table below. It applies when implementing the linked text/layout/render contributions and their adapters. Current source remains authoritative for implemented behavior; this document specifies the selected replacement. A change to an observable rule requires an explicit replacement determination in this document and the affected issue, not an implementation-local exception. Completion requires the consumer-specific evidence below; until then, these decisions do not assert feature support.

The inspected repository basis is `3c412118b5e9f4ad8a34a1331f8c500618599498`. Current code builds and independently wraps a Parley layout in [system.rs](https://github.com/bj-data-eng/surgeist/blob/3c412118b5e9f4ad8a34a1331f8c500618599498/crates/surgeist-text/src/system.rs#L130), projects already placed lines in [layout.rs](https://github.com/bj-data-eng/surgeist/blob/3c412118b5e9f4ad8a34a1331f8c500618599498/crates/surgeist-text/src/layout.rs#L86), and represents cluster interaction rectangles using advance and line height at [line 239](https://github.com/bj-data-eng/surgeist/blob/3c412118b5e9f4ad8a34a1331f8c500618599498/crates/surgeist-text/src/layout.rs#L239). These facts establish the replacement surface; they are not the future contract.

The retained Parley source witness is [Parley 0.9 text contract witness](../references/parley-0.9-text-contract-witness.md). It records the registry checksum, exact file hashes, excerpts and license. Parley 0.9 stores normalized coordinates and synthesis separately from its font blob, and shapes with Harfrust. Its builder also performs analysis, resolution and shaping before the line breaker. Those mechanisms justify splitting preparation from selected-line finalization. The current public wrapper omits those effective instance facts.

Layout has [supplied-level L2 reordering](https://github.com/bj-data-eng/surgeist/blob/3c412118b5e9f4ad8a34a1331f8c500618599498/crates/surgeist-layout/src/inline/mod.rs#L693) beside its box fitting and float selection. Render [preflights actual glyph representations](https://github.com/bj-data-eng/surgeist/blob/3c412118b5e9f4ad8a34a1331f8c500618599498/crates/surgeist-render/src/vello_engine/glyph.rs#L168), but currently uses [empty normalized coordinates](https://github.com/bj-data-eng/surgeist/blob/3c412118b5e9f4ad8a34a1331f8c500618599498/crates/surgeist-render/src/vello_engine/glyph.rs#L33). The changes below are design decisions required to join those owners correctly, not claims that the current code already supports the join.

## Owners

| Owner | Required responsibility |
| --- | --- |
| Text | Authored/processed mappings; Unicode analysis; font matching/fallback; shaping; opportunities and replacement alternatives; line-edge text processing; selected-line shaping and spacing; line bidi ordering including atomic participants; font/glyph outline metrics; cursor, selection, hit and geometric accessibility projections from finalized geometry; source-only accessibility from analyzed source. |
| Layout | Box and float constraints, intrinsic box aggregation, available spans, actual break choice, spacing allocation, line/fragment placement, baselines and atomic box geometry. |
| Render | Safe font-resource admission; executing the effective font instance; selecting and validating the glyph representation used by the backend; its local ink bounds; rasterization. |
| Consuming subsystems | Layout owns its style/text adapters and composed line selection; render owns conversion from placed text/layout results to admitted draw data; window owns host accessibility conversion. |
| Runtime | Captured inputs, scheduling and coherent publication of the consumers' off-side results. It does not own their semantic conversions. |
| Root | Facade, curated reexports and bootstrap. |

This ownership follows the current [producer matrix in #483](https://github.com/bj-data-eng/surgeist/issues/483). Dependency edges point toward consumed producers: layout may consume style/text, render may consume placed layout/text, and window may consume the producer results needed for host projection. Text does not import layout or render; style does not import text or layout. [#1157](https://github.com/bj-data-eng/surgeist/issues/1157) owns generated-content plans, occurrence provenance and staged page/fragment feedback. That semantic source mapping accompanies this contract's evaluated text-source ranges; generated text does not acquire fabricated DOM nodes or authored offsets.

Move the supplied-level bidi reorder and applicable line-edge text operations out of layout when their callers use text. Keep line fitting, floats, intrinsic box aggregation and physical placement in layout. Include atomic inline participants and directional boundary controls in the ordering input: reordering text-only runs cannot establish the order of a mixed line. Fixture font-width heuristics do not become production algorithms.

## Concrete text values and lifetime

Expose concrete owned values in `surgeist-text`, with private validated fields and ordinary constructors/accessors. The names here define roles; implementation may align names with the existing API without adding equivalent parallel models.

`AnalyzedSource` owns immutable authored text, validated style/inline source facts, processed text, Unicode analysis and logical units. `PreparedText` retains that same analyzed value and adds resolved fonts, shaping, break alternatives and a captured font catalogue. Authored storage and source-facing positions/ranges use exact UTF-16 code units, matching DOM CharacterData and Range offsets. This replaces the earlier authored UTF-8 requirement. Units retain their exact source ranges and an explicit mapping to processed/provider buffers; private UTF-8 byte offsets must not masquerade as source indices. Removed/collapsed content retains zero-extent source anchors with affinity; expansions retain their many-to-many source association. A presentation-only replacement, including a selected hyphen, retains an insertion anchor and does not invent authored characters. Plaintext projection reads the authored/source mapping and its own required trim/collapse rules, not emitted glyph strings.

Text accepts immutable code-unit backing without depending on DOM or HTML for their string type. Rust string construction may encode into that same source model; it does not establish a second offset convention. Preserve isolated surrogates and every in-bounds code-unit endpoint, including the boundary within a valid surrogate pair. Raw edits and source extraction operate on those exact units and may join or split pairs. Retain source identity/revision and association to each original text occurrence through lowering; a generated occurrence does not acquire fabricated DOM offsets. See the implemented [DOM range contract](../crates/surgeist-dom/docs/ranges-and-editing.md) and [immutable document strings](../crates/surgeist-html/src/values.rs).

Provider buffers contain Unicode scalars in the provider's required encoding. A valid surrogate pair projects to one scalar mapped to both original units; an isolated surrogate projects to one U+FFFD for glyph production, with distinct provenance from a literal U+FFFD. This is the selected Surgeist rendering policy, not a change to DOMString or source data. Derive source properties from the original code points before this glyph substitution: the retained [bidi data determination](../references/SOURCE-EDITIONS.md#html-language-and-directionality-sources) gives isolated surrogate code points class L. Substituting U+FFFD before property analysis can change directionality and is not an acceptable implementation. Keep transformations and their resulting semantic properties associated with their own phase. The repaired glyph buffer cannot replace the source/property input. Preserve actual text adjacency and boundary controls rather than joining units across an absent text connection.

Exact source endpoints and projected carets are distinct. An endpoint within a scalar or grapheme retains its original code-unit position. Upstream/before affinity projects to the preceding legal logical caret stop; downstream/after affinity projects to the following stop, before bidi maps logical order to visual geometry. Hit testing and caret movement need not produce interior-scalar stops. A collapsed logical range remains collapsed; independently rounding its ends must not turn it into selection of a character. Source extraction and editing retain the original endpoints rather than the projected caret. This affinity rule is a Surgeist policy. Selection rectangles follow their separate [CSSOM View partial-unit rule](../references/cssom-view-1--WD-cssom-view-1-20250916--693b1c1fe7f7.md#dom-range-getclientrects): a range selecting half a surrogate pair or part of a typographic character unit includes that full unit for rectangle computation. Do not substitute caret snapping for that geometry rule or expand the retained source range.

A preparation has an opaque owner-issued identity. Clones retain the same identity; independently prepared equal strings do not gain interchangeable unit handles. An internally shared allocation token with identity comparison is sufficient: do not hash content into a supposedly unique public identity. Source ID/revision and font generation are observable provenance, not substitutes for this identity. Use `usize` for source/unit indices and checked arithmetic; do not inherit Parley's compressed `u8`/`u16` text length/offset fields as public limits.

`PreparedText` retains strong ownership of every font resource it uses. A face consists of immutable font bytes, collection index and text-owned resource identity. An `EffectiveFontInstance` additionally carries the resolved normalized coordinates for every variation axis, the actual shaping size and concrete synthesis parameters. Coordinates are the actual canonical values passed to shaping/metrics, not merely authored variation settings. Synthetic skew and emboldening must have concrete executable values, not an unconsumed suggestion. The synthesis resolver records its executable skew transform and emboldening displacement once for both text geometry and render admission. Fontique’s boolean faux-bold suggestion alone is insufficient; an unresolved suggestion is a typed unsupported-instance error, never silently ignored. The applicable font-synthesis feature issue owns which synthesis is requested/permitted; #1080/#1082 own converting the selected result to these concrete executable parameters.

Font catalogue changes publish a new immutable catalogue identity and invalidate preparation cache lookup for new requests. Existing preparations retain their old fonts and remain valid for reading/finalization. Each consuming adapter rejects joining them into a newly captured composed result whose source/style/font identities differ. A numeric generation counter must fail on exhaustion rather than saturate and reuse an identity. Refresh must actually publish the refreshed catalogue and replay registrations, rather than merely advance a counter. Provide concrete safe font registration and a `RegisteredOnly` catalogue mode for reproducible tests; preserve the default system catalogue behavior. This is not a font-provider trait.

### Text transformations and correspondence

Text owns one immutable transformation result after executed Phase I and before
Unicode analysis, font selection, shaping and wrapping. Follow the pinned
[transform order](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#transforming):
word-space transformation, casing, full-width, then full-size-kana. Original
UTF-16 and Phase-I text retain their existing coordinates and
[plaintext semantics](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#plaintext).
Transformation output has its actual UTF-16 extent; provider bytes remain a
separate domain. For preserved `A𛄲C`, original and Phase-I endpoints remain 4,
while full-size-kana presentation ends at 3. Prepared processed readbacks must
continue to name the Phase-I domain.

Record correspondence at the actual emission or removal, composed through
Phase I to ordered original occurrence-qualified pieces. Each emitted scalar
retains its contributing pieces and actual style owner. Expansions may repeat
one original interval; contractions may retain multiple original units or pieces.
Zero-output input retains its association and affinity. Preserve holes and
distinct occurrences rather than replace them with a source hull, output diff,
or equal-text deduplication. Isolated surrogates remain original units through
this phase; glyph replacement is separate. Context crosses genuine connected
origins, neutral inline/empty occurrences and out-of-flow boxes, while actual
disconnections and atomic/forced boundaries retain their roles.

All downstream analysis, opportunities, preparation, finalization and consumers
use this result. Retain the prescribed full conditional and known-language
casing requirements; per-scalar contextless casing is insufficient. Full-width
uses the raw Unicode wide/narrow relation, not compatibility normalization, and
maps original U+0020 only when its Phase-I whitespace is preserved. Eligible
separator-generated U+0020 qualifies for full-width in every whitespace mode,
including Collapse and Discard, under the
[native generation determination](../references/SOURCE-EDITIONS.md#separator-generated-space-and-full-width).
Its generation is separate from original-space preservation and later Phase-II
collapsibility. Never replay Phase I or coalesce adjacent generated spaces;
later removal/hanging uses the actual character and governing style.

Genuine native wbr capture retains its complete computed Style and occurrence
identity independently of generic optional-break controls. Expand at its actual
post-Phase-I scalar seam before rebuilding graphemes. Preserve exact mixed-event
order, including distinct equal-anchor occurrences. An expanded wbr contributes
text with its own control authority and no original pieces; it does not add a
second control participant or allocation. Generated-only Text units retain their
control bindings in Layout's occurrence-order validation.

A genuine wbr separates original UTF-16 decoding before Phase I regardless of
replacement or suppression. Surrogate halves across it remain isolated original
units through later projections; provider-only glyph repair cannot change their
semantics. Keep this decoding boundary separate from whitespace, word/phrase,
casing, bidi and shaping context. Ordinary origin, inline and out-of-flow seams
do not gain a decoding boundary.

If positive expansion requires an interior cut in an indivisible document-delimiter
replacement, return the precise native seam refusal before publication. None and
demonstrably forced-suppressed occurrences retain their unexpanded behavior.
Do not move the anchor using grapheme affinity or split Phase-I output. Keep
contextual delimiter validation intact; it does not admit such interior controls.

Retain contextual connection-event order through lowering. Use actual in-flow
atomic/forced occurrences and connection receipts for semantic barriers; numeric
source endpoints alone cannot place barriers around newly generated text.
Coincident direct captures without the connection side needed by positive wbr
expansion return precise missing-context refusal. Typography affinity and nearby
forced characters do not establish that side or prove same-component suppression.

Original U+200B replacement uses the same genuine connection-side boundaries for
forced adjacency. If a flat capture's unknown coincident control side can change
that replacement, refuse with the original separator's range and connection;
never invent an emitting control. Preserve known no-op, source-hard-break
suppression and intervening-character outcomes without demanding unused context.

Original and control contributions may share one actual grapheme Range unit.
Retain both contributors and their real fragments without lending the generated
separator a defining source interval. Source-range closure selects the actual
unit containing an eligible authored contribution; exact placed positions can
select generated-only geometry while extracting no invented source or plaintext.
Accessibility retains authored-text descriptors independently of that geometry.

For automatic separator insertion, Text detects phrases on the connected
post-Phase-I semantic input before any separator replacement. Its native Japanese provider
uses the pinned BudouX model; CSS eligibility, actual declared language and scope
custody remain Text-owned. Unknown language produces no proposal. Operational
failure in supported analysis cannot masquerade as unsupported language or a
successful empty result. Preserve actual input identity, coverage and model
edition with the immutable analysis. The native detection limits and capture
API are described in [Text's reference](../crates/surgeist-text/docs/reference.md#native-automatic-phrase-separators).

Incident text requests detection; the entire incident inline-boundary strip
determines the outermost expansion owner, including empty scopes. Its complete
Style governs generated output, without requiring that owner itself to request
detection. Preserve separate virtual and genuine wbr occurrences at a shared
seam. Missing demanded topology or Style refuses precisely; true no-output paths
do not consume unused context. Balanced capture remains the sole supplied tree.

A virtual occurrence has no authored source interval or control identity. Retain
its real issued scope, generated geometry and empty insertion anchor through
selection and placement. Text's shared capture owns canonical scope, control,
paragraph and event identities. Prepared handles retain their own phase identity
and project to those facts. A scope's global pair remains complete across actual
paragraph cuts; local endpoint absence is separate from missing capture facts.

Layout derives one global inline association registry from canonical endpoints
and validated actual control-node bindings. A domain keeps its actual paragraph
container; an atomic content domain must match its exact nominated atomic node.
Authenticate intersections and tree order through captured events, including
endpoint-only and nonemitting paragraphs. A paragraph vector or numeric source
ordering cannot establish scope traversal authority.

Order is domain-local. Text projects authenticated source events; Layout derives
actual observations independently from validated child/run order. A parent's
atomic occurrence is ordered in that parent, while the atomic's independently
captured content is validated inside its exact nominated box. Plain child blocks
instead retain genuine interruption boundaries in parent flow. Deferred child
capture cannot reorder parent siblings or transfer content to another atomic.

Generated scope contributors determine the required proof regions: an inline
owner requires its complete own-domain extent; a domain owner requires its full
own-domain flow. Preserve disjoint regions. Resolving known ancestor endpoints
or a nominee's parent binding proves association and containment without requiring
unrelated interiors. Every genuine Block interruption inside a demanded region
still requires its actual child-domain association. Ordinary local binding checks
remain mandatory even when there is no generated contributor.

A genuine zero-participation captured paragraph can supply an empty domain's
actual container association without requesting a Text run. Both actual prepared
and supplied occurrence inventories must be empty, with no line strut request.
It retains capture/cache/invalidation identity but produces no additional line,
glyph, fragment or placed Text result; the box keeps its ordinary layout behavior.
Missing source bindings or removed-only source cannot masquerade as emptiness.

A distinct computed-only captured paragraph can be scheduled without host children
when its genuine mandatory participants and unique native child gap are proved.
Retain exact paragraph, preparation and container identity through lookup, pending
results, box custody, replay and cache ownership; an empty node vector is not a
run identity or a prefix match. Shared gaps use canonical same-capture event order,
including real child-Block fences. Empty-container and trailing gaps execute once
through Layout's existing formatter and flow transitions. Unproven or ambiguous
native placement refuses; no child, origin or source offset may be invented to
make a slot. Strict association metadata remains separate. Numerical finalization
and CSS phantom disposition follow the same complete placement contract below.

Output preserves the whole scope association and each fragment's own local
prepared/placed occurrence. For a virtual-separator Scope fragment, boundary
nodes prove association rather than substitute geometric anchors: never
interpolate from them, duplicate its geometry onto both, or donate it to
neighboring text. Actual endpoint nodes retain their separate
[control footprints](#native-inline-endpoint-geometry). Scope/capture/preparation,
actual endpoints, participating containers and relevant tree dependencies enter cache/invalidation ownership.
Complete spanning captures execute; truly absent or foreign facts remain precise
errors without invented continuation controls or geometry.

Full-size-kana uses
the complete pinned [Appendix G](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#small-kana).
Expose only executed options and report their specific capabilities. A complete
kana contribution does not establish casing, width, math or separator support.

For capitalize, classify the immutable pre-casing input using the native UAX29
word policy and effective typographic-unit properties. The first Letter/Number
unit occupies the initial slot even when it is a number or caseless letter;
do not skip it to find a later lowercase scalar. Connected origin, font, style
and transparent out-of-flow seams do not restart words. The unchanged word tail
is distinct from the full contextual title operation within an eligible initial
unit. Preserve uniform-language canonical-equivalence behavior, including Greek
subscript sequences, and applicable conditional effects on later marks.

The [native component-casing determination](https://github.com/bj-data-eng/surgeist/issues/63#issuecomment-6376400472)
retains each component's actual requested operation and applicable language.
Evaluate its initial-unit membership and conditions against the same immutable
input; another component cannot donate its transform or locale. For initial
`ij`, Capitalize/nl plus None/nl produces `Ij`, None/nl plus Capitalize/nl
produces `iJ`, Capitalize/root plus Capitalize/nl produces `IJ`, and
Capitalize/nl plus Capitalize/root produces `Ij`. These mixed-unit results are
Surgeist policy where CSS leaves split-unit rendering undefined. For capitalize,
Lithuanian dot removal requires the dot component itself to request Capitalize
with applicable Lithuanian, belong to the eligible unit, and satisfy the real
original-input After_Soft_Dotted condition. Keep its zero-output ownership.

Explicit script subtags take precedence. For a known language without an
explicit script, the native casing policy uses the paired extended likely-subtag
data from `icu_locale_data` 2.2.0 (CLDR 48.2.0), retaining an authored region.
Do not turn an unknown language into a guessed one, infer language from script,
or use a fallback font's locale. The full implied-script table is outside CSS's
specified correspondence; this is a declared data choice. Apply appropriate
language rules in their actual writing system rather than copy script-blind
provider dispatch. For example, explicit `tr-Arab` does not request Latin
Turkic casing; the paired data's bare `crh` and explicit `crh-Latn` differ.

Math-auto is an exclusive computed alternative to the combinable case, width
and kana operations. Use the pinned
[MathML Core mapping](https://www.w3.org/TR/2025/CR-mathml-core-20250624/#math-auto-transform)
and its referenced Appendix C.1. The
[native cardinality determination](https://github.com/bj-data-eng/surgeist/issues/63#issuecomment-6378219209)
tests complete original Text-node content for one listed character, then maps
its actual surviving contribution after Phase I. Original `SPACE c SPACE`
remains ineligible even if genuine Discard leaves only `c`. Keep distinct nodes
distinct, and do not manufacture eligibility by splitting an origin, selecting
a substring or trimming aggregate element text. This original-cardinality
choice resolves a source precision gap as native policy.

Execution requires truthful complete-node content and its association with the
captured original contributions. Missing grouping is a typed missing-context
refusal, not successful identity transformation. Unmapped complete content
remains unchanged; no normalization or isolated-unit repair can invent a mapping.
An explicit computed math-auto request has no inferred namespace or font-style
gate. The actual `mi` default and `mathvariant=normal` hint belong to their
producer adapter, separately from Text executing a normalized native capture.

The [native expansion-interior determination](https://github.com/bj-data-eng/surgeist/issues/63#issuecomment-6375717099)
preserves the actual presentation boundary in `ß → S|S`. Its immutable placed
position retains correspondence and output subposition. Interior upstream/trailing
affinity projects to the contributing original end with upstream affinity;
downstream/leading projects to the original start with downstream affinity.
These are logical sides, independent of physical RTL order. With multiple
contributing pieces, use the last endpoint or first start in logical source order
and retain occurrence identity. This projection is lossy; source cursors cannot
reconstruct an interior placed occurrence. Ordinary outer boundaries retain
their existing projection.

Exact placed selection derives geometry and authored coverage from selected
presentation units, never sorted projected source endpoints. Selecting only the
receiving `S` has receiving-only geometry and full original `ß` coverage;
selecting both pieces extracts that authored occurrence once. A complete source
Range selecting `ß` includes both attributable pieces, while a placement-restricted
query reports only that placement's geometry without splitting original custody.
A collapsed exact selection has no selected authored coverage. Exact source
extraction preserves original UTF-16; plaintext applies its separate trim/collapse
policy. Neither operation creates fractional source editing rights.

A legal interior cut resumes at the next transformed participant/subposition
through the existing issued boundaries. Original coverage may overlap lines;
presentation progress must advance. Ordinary AL–AL supplies no `S|S` break;
break-all and conditional anywhere retain their own gates. Do not synthesize
authored consumption or reuse indication-specific `HyphenationContinuation`.
The interior projection and coverage choices are Surgeist policy, not a browser
interoperability claim. Introduce their exact-selection API with an actual
expansion-capable transform, through one selection owner and direct caller
migration.

The [native zero-output determination](https://github.com/bj-data-eng/surgeist/issues/63#issuecomment-6376502179)
preserves removed input as ordered relation occurrences. Before a removed row
projects to its source start with downstream affinity; after it projects to its
source end with upstream affinity. The cuts remain distinct even when output
offset and geometry coincide. Adjacent emission/row handles may alias one cut
with different affinities. Consecutive removals require actual occurrence/cut
identity, not a binary affinity flag.

Exact selection includes the relation occurrences traversed. For original
`I\u{0307}` lowered to `i`, selecting only the emission through its trailing
cut extracts original `I`; extending across the removed-dot occurrence includes
the dot. Likewise, selecting just `n` in Irish `n-uisce` uppercased to `nUISCE`
extracts `n`, while traversing the retained removed hyphen includes `n-`.
A removal-only selection is noncollapsed, has original source coverage, and has
no independent emitted rectangle. Identical relation cuts are collapsed. The
full source-bearing line extent includes its zero-output custody. These are
owner-issued relation positions; they imply neither fabricated glyph units,
new wrapping opportunities, false output progress nor mandatory extra visible
keyboard stops.

The [native wrap-seam determination](https://github.com/bj-data-eng/surgeist/issues/63#issuecomment-6376584001)
binds a legal prepared boundary to the cut after consecutive removals within
its authentic local mixed-participant/relation seam. The terminating line owns
them; the receiving line starts at the same cut. Equal numeric output offsets
do not authorize crossing genuine atomic, control or scope boundaries. Leading
and trailing removals remain with their genuine scope's first and last
source-bearing lines. A genuine output-empty extent retains custody through the
real empty-line/finalization path. Neither prepared start=end nor finite removal
traversal can invent a wrap line or establish presentation progress.

Qualification includes all kana mappings and unmapped controls, supplementary
contraction with distinct Phase-I/presentation readbacks, actual registered-font
glyphs and metrics, source/plaintext custody and shared composed geometry.
Expansion acceptance additionally requires unwrapped and legally wrapped
`ß → SS`, both logical affinities and RTL mirrors, receiving-only selection and
extraction, both-piece extraction once, source Range over both lines, collapsed
selection, distinct equal occurrences, mixed source pieces and real conditional
and known-language length-changing mappings. These requirements do not claim
implementation before their owning contribution is qualified.
Also qualify legal wrapping beside removals, consecutive/leading/trailing
removals and genuine mixed zero-width participants. Combine actual reordered
Greek output, separated contributions of one input row and a real removal;
check semantic event order, original-piece order, emission-only/removal-only
and reversed selection, and whole-line custody independently.

### Font-independent analysis and accessibility

Ordinary line-break analysis retains the complete connected logical text
component across source and style seams. Each candidate uses its following
logical text unit's `WordBreak` while retaining earlier Unicode context, such as
regional-indicator pairing and opening punctuation across spaces. A computed-style
change does not end that context. Existing semantic barriers, typography and
captured ancestor wrapping still govern analysis and admission.

`AnalyzedSource::new(Source, Style)` performs the font-independent analysis once and returns an immutable value with an opaque allocation identity. Clones retain that identity; independently analyzed equal text does not. It preserves exact source units, grapheme and host-projection mappings, source affinity and applicable language facts. Isolated UTF-16 units remain valid analysis inputs; a host requiring scalar UTF-8 may reject their accessibility projection at that boundary. Font preparation retains the same analyzed allocation. `System::layout_analyzed` consumes this analysis through the same preparation, selection, finalization and placement machinery as the `System::layout` convenience path.

This deliberately replaces the former successful fontless `Layout` phase while preserving its useful source-only accessibility capability. The existing `absent_selected_font_does_not_erase_original_selectable_graphemes` regression establishes that original graphemes remain selectable without glyphs. Preserve that behavior through the real analyzed-source front door in the same implementation change. Do not retain a fontless-layout branch or manufacture a font, glyph, prepared value or empty geometry to satisfy it.

A renderable shaping request without a usable admitted face returns a typed no-usable-face/font-resolution error. An empty `RegisteredOnly` catalogue is valid and is distinct from font-service unavailability. Empty, collapsed, control-only or box-only content does not automatically require a glyph face; any requested font-dependent line metrics still require their actual font. A failed layout publishes neither partial layout nor a source-only accessibility update. Callers may explicitly retain analysis and request source-only projection independently.

Source-only accessibility takes no placement point. It reports source values, units, language and selection mappings, omitting font/glyph facts, visual lines, character positions/widths and parent bounds until actual placed geometry exists. Missing geometry is neither a zero rectangle nor proven empty ink. Geometric accessibility enriches the same semantic analysis with the actual finalized rows, without a second segmenter or shaping pass.

Accessibility state binds to the exact analyzed allocation and an explicit source-only or placed phase; placed state additionally binds to the exact placement identity. Source-only reads require the source-only binding, and layout reads require the matching placed binding. Updates may coherently replace or enrich the binding, rebuilding optional fields so previous geometry cannot survive a source-only replacement. Preserve checked node-ID issuance, source affinity, focus rules and atomic failure. Verify selectable fontless graphemes, explicit layout refusal, ordinary registered-font enrichment, mismatched analysis/placement rejection, absent fabricated geometry and preservation of the previous accessibility state after failure.

### Native word-break policies

`WordBreak::BreakWord` retains its declared value and the separately declared
`OverflowWrap` in validated/captured styles and cache identity. Text gives them
one effective policy: ordinary Normal word breaking and conditional
Anywhere overflow wrapping, regardless of the declared OverflowWrap value.
The [pinned CSS definition](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#valdef-word-break-break-word)
defines this property value.

This adds no ordinary word-interior cuts by itself. Native Overflow and
MinContent conditional queries consume the effective Anywhere policy; Layout
still establishes ordinary-fit exhaustion, chooses a real conditional trial
and finalizes it through Text. Max-content remains unwrapped. In particular,
WordBreak BreakWord includes these cuts in min-content, while OverflowWrap
BreakWord alone excludes them. Actual hyphenation remains Normal word-break
behavior, subject to its independent owner, LineBreak and other admission gates.

Ordinary following-box ownership and native emergency logical-before ownership
remain separate. Standalone applies the same effective pair through its existing
raw Normal/Anywhere selection path; its characterized raw candidate ownership
does not become the native captured-paragraph contract.

`WordBreak::Manual` retains Normal's ordinary rules with lexical line breaking
deliberately disabled. SA units use their AL/CM interpretation for line rules;
missing lexical capability does not activate fallback under Manual. Declared
overflow wrapping remains effective. Orthographic opportunities, actual Wbr and
ZWSP, forced breaks and independently admitted LineBreak Anywhere retain their
own rules. Hyphenation also retains its actual owner and policy: Manual is not
Hyphens None or KeepAll. The declared values remain visible in captured styles
and cache identity; editing, accessibility and other word metadata do not change.

`WordBreak::AutoPhrase` retains its declaration and overflow policy. It requests
Normal ordinary rules with phrase preference and suppresses actual-owner
hyphenation during ordinary selection, including empty indications, even when
its language has no phrase model. A neighboring Normal owner retains its own
eligible alternatives. Observed words, resource cuts, editing and source text
remain independent facts.

For supported Japanese, phrase detection runs on the complete connected **final
transformed** semantic input. It shares the qualified raw provider and validated
ja/absent-or-Jpan admission with separator insertion, while each phase retains
its own input identity and CSS/custody decisions. Font, source and ordinary
style fragmentation do not truncate that input. Phrase preference adds no
separator or source character. Unknown/unsupported language retains Normal's
ordinary opportunities; a supported no-cut result remains a whole phrase.

Prepared Text retains independent causes at each actual mixed boundary. It
suppresses only phrase-interior implicit Normal causes controlled by the
requesting owner. Explicit Wbr/ZWSP, required LineBreak and atomic causes keep
their independent authority, with actual TCU and wrapping/NCA constraints.
Suppressed Normal causes and otherwise eligible owner hyphens remain in the
same immutable preparation, available only through authenticated restoration.
They are excluded from ordinary enumeration and unqualified selection.

Finite selection first exhausts ordinary choices. Only a real measurement of
the affected phrase piece overflowing an otherwise-empty eligible line permits
its Normal restoration; if insufficient, try eligible owner hyphenation before
arbitrary overflow. Measurements use the shared finalizer and cannot be placed.
After a restored internal cut, a new query issues the whole remaining piece at
its authenticated start and incoming continuation, retaining full phrase
provenance. Measure only unconsumed participation in the new frame, and first
try its ordinary residual ending. Prior-line or intrinsic measurements grant no
finite authority. Unsupported language needs no invented phrase measurement to
reach its eligible hyphen restoration stage after actual ordinary exhaustion.

The [intrinsic determination](../references/text-auto-phrase-intrinsic--determination-20261025.md)
retains genuine Anywhere intrinsic opportunities independently of finite stages;
Normal/hyphen restoration cuts are excluded from min-content. Max-content opens
no restoration stage. Text validates its issued handles and visible request
context; each candidate may converge to its own numerical width, height, tabs
and allocations. Layout owns numerical fit, the full private empty-line/float/
replay basis and restarting on material external changes. Equivalent queries
retain semantic identity for cache and pass convergence. Standalone consumes the same prepared
AutoPhrase stages through its own fit context and shared finalizer.

### Native line-break policies

Text owns `Style::line_break` and its `LineBreak` values Auto, Loose, Normal,
Strict and Anywhere. The captured value remains visible in validated styles
and participates in cache identity. Auto uses Strict's ordinary restrictions
as a deterministic native policy; it does not change with the available width.
Ordinary restrictions use the following logical text contribution's value and
content writing system, retaining the complete connected analysis context.

Loose, Normal and Strict apply the required CJK distinctions in the
[pinned Text 4 line-break clause](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#line-break-property).
BreakAll's CJ-letter-to-ID conversion precedes the class-based CJ restriction;
named iteration marks retain their specific restrictions. The
[small-kana determination](../references/text-small-kana-precedence--determination-20261025.md)
defines this precedence and the full-size-kana contrast.
Explicit script subtags take precedence over the language: ja-Latn and zh-Latn
do not select CJK conventions, while ko-Hani and en-Hira do. An omitted language
does not become an inferred language from the text. Independent whitespace,
Wbr, ZWSP, forced-break and atomic permissions retain their own authority;
KeepAll does not erase a required line-break permission.

Anywhere creates equally ranked ordinary opportunities at admitted text/text
typographic-unit edges when either actual adjacent text contribution requests
it. It overrides the specified text glue and word-break prohibitions without
using overflow-wrap's emergency proof or priority. Typography, actual ancestor
wrapping, semantic barriers and hard-break precedence still apply. Empty and
out-of-flow metadata supply no style authority, and a source-free control does
not become a text unit or an Anywhere donor. All consumers, including intrinsic
sizing, see the same ordinary opportunity inventory.

The [Anywhere owner determination](../references/text-anywhere-owner-precedence--determination-20261025.md)
keeps hyphenation eligibility with its actual source-SHY, discretionary-control
or logical-preceding Auto owner. An Anywhere owner excludes empty and nonempty
hyphenation alternatives while retaining truthful cut/resource observations.
A Normal-owned alternative can coexist with an independently eligible ordinary
None variant supplied by following Anywhere text. An ordinary neighbor does
not take over the indication's style or identity.

At actual text/atomic relations, Anywhere preserves the specific prohibition
against an atomic-created cut adjacent to non-NBSP GL, WJ or ZWJ. NBSP retains
its atomic compatibility exception, subject to actual ancestor wrapping. This
does not change the atomic's geometry or margin-edge custody. A genuine Wbr or
forced occurrence retains its independent authority at its exact mixed cut.

The current typography boundary is the retained extended-grapheme analysis.
These operations do not establish additional syllabic tailoring or complete
lexical/orthographic support for every writing system. Those capabilities remain
separate from accepting a native LineBreak value.

### Missing lexical capability

The selected non-complex line segmenter has no Thai/Lao lexical models. Ordinary
SA context uses AL, or CM for SA nonspacing/spacing marks, through a provider-only
input with exact original endpoints. A provider slice end is not an analyzed
linguistic boundary. For adjacent Thai or Lao SA letter units in the same
writing system, Text supplies
ordinary fallback opportunities between whole retained typographic units.
Normal and KeepAll retain this fallback; effective BreakWord uses Normal.
BreakAll retains its independent ID-based permission. The
[lexical-fallback determination](../references/text-lexical-fallback--determination-20261025.md)
defines the bounded KeepAll policy and capability distinction.

Eligibility follows actual processed script and character properties under
known, absent, und or mismatched language metadata; the declared locale stays
unchanged. Actual following-text policy, ancestor wrapping, glue, hard breaks,
semantic barriers and exact mixed-cut custody still govern admission. Inline
and out-of-flow metadata cannot supply authority. Layout consumes these offers
before emergency overflow, including for min-content; max-content stays unwrapped.

These cuts record missing lexical capability, not detected linguistic words.
They do not add editing, accessibility, phrase or hyphenation word boundaries.
A qualified analyzer finding no internal boundary must not activate fallback.
This implementation qualifies Thai/Lao grapheme-based units only; dictionary
admission and other writing systems' syllabic tailoring remain separate work.

### Empty text and font struts

For genuinely empty source with no inline participants, intrinsic text advance and `Metrics::full_width` are zero. Retain the actual admitted-font empty-line strut, baseline and height, one `0..0` line and the original endpoint zero. No source character, glyph or phantom space advance is introduced. In the registered Ahem size-20, unconstrained, unquantized profile, the result is size `(0, 20)`, full width zero, baseline 16 and no clusters or glyphs. A real authored space retains its actual advance.

This intentionally replaces the former intermediary's empty-source `full_width` of 20 in that profile; its synthesized space was an implementation placeholder, not authored text. The zero-width owned representation was already drafted when this difference was characterized, so qualification must not claim a historical preimplementation RED. Preserve the baseline observation and independently test the selected functional result.

This rule does not set an empty box's allocated width, erase inline participants, classify all controls as zero-advance, or equate empty ink with zero line height. Caller constraints, alignment, placement and Layout minimum sizes remain their owners' decisions. Font-derived strut metrics require the actual admitted font; source-only analysis remains available without it. Empty accessibility and caret geometry may use the real strut without acquiring a synthetic source unit or glyph.

### Concrete outline synthesis

For permitted faux bold on admitted horizontal outlines, select the [pinned WebKit extracted-path operation](../references/webkit-synthetic-bold-outline--73aa6c89e2cb.md). Text resolves `b = actual_size / 36.0_f32` once from the actual canonical shaping size. The positive displacement must remain finite and nonzero; otherwise return a typed numeric error. Retain the concrete operation, displacement and executable skew in the effective instance and its cache identity. The font-synthesis feature owner still decides whether synthesis is permitted; natural variation-axis selection remains distinct.

Render appends each original outline and a copy translated by positive local x displacement `b` to one canonical path, filled once with the nonzero winding rule. Preserve the existing checked evaluation `x*s + k*y*s + gx`, then add the copy displacement before canonical `f32` conversion; y remains `-y*s + gy`. Here `s` is actual size divided by units per em, `k` is the executable x-shear and `(gx, gy)` is the actual local glyph position. Apply the outer placement transform afterward. Do not scale the displacement again or reverse it for RTL. A positive displacement absorbed by canonical point conversion is a typed numeric refusal. Large physical origins remain in the outer `f64` transform.

Advances, glyph positions, carets and font line metrics do not gain a bold-spacing adjustment. Apply the outline operation to actual positioned glyphs, including zero-advance marks; an outline-empty glyph remains empty. Text outline geometry and Render ink include the emitted copies, with bounds derived from their canonical geometry. Additional path elements and live storage count against the existing limits, and failure publishes no prefix. Existing representation restrictions remain explicit capability errors.

This is the extracted-path policy. Appending contours is not a geometric boolean union and does not promise native CoreText two-pass alpha, antialiasing or CTM-adjusted displacement. The selected profile covers the existing horizontal outline and x-shear convention; vertical/oriented synthesis needs its own qualified operation before admission. This determination selects replacement behavior and does not assert that the current Text resolver or Render entry already implements it.

## Selected-line protocol

Native atomic-neighbor opportunities use the captured [nearest-common-ancestor wrapping policy](../references/css-text-3--CRD-css-text-3-20260814--17c26f1b4145.md#line-breaking-scope) and the pinned [GL/WJ/ZWJ restrictions](../references/css-text-3--CRD-css-text-3-20260814--17c26f1b4145.md#atomic-compat-wrap), including the NBSP exception. An independently qualified opportunity created by a disappearing SPACE retains its containing box's wrapping authority; denying atomic-created permission cannot erase that separate owner. Out-of-flow occurrences, neutral controls and empty origins do not create permission. When they separate two influencing neighbors, each existing mixed cut may lend the same permitted semantic opportunity; equivalent cuts must agree on that relation's captured wrapping policy. Selecting the exact cut determines the retained occurrences' line ownership. No canonical cut is invented from their shared source offset. An absent influencing neighbor at the beginning or end does not create an atomic soft opportunity; the actual paragraph terminator still includes all trailing occurrences. Explicit forced, optional and discretionary controls retain their own break authority.

The wrapping fact belongs to the true ordered influencing-neighbor relation,
including text/text and in-flow atomic relations. A captured relation identifies
its original endpoint occurrences and, for text, the relevant source boundaries
or spans within them, together with their actual nearest-common-ancestor wrapping
permission. An aggregate source offset or adjacent-origin connection index alone
does not identify that pair. Distinct immediate connection policies through empty
or out-of-flow occurrences may legitimately differ; do not select one by cut
position, combine their booleans or require them to agree. All eligible mixed cuts
in the same relation consult its permission while retaining their distinct
occurrence custody. Permission alone creates no EGC opportunity.

The consuming capture producer computes this ancestor permission from its actual
input model. It reads Text's postprocessing neighbor/provenance views through
`AnalyzedSource::wrapping_relations`, associates the issued relations with its
original native occurrences, and supplies their actual `WrappingPolicy` values to
`with_wrapping_policies`. Text validates exact context ownership and the actual
ordered relation; foreign handles, invalid applicability and conflicting
same-relation declarations fail. All native boundary-defined permission uses
one normalized relation owner. Existing incident facts contribute only when they
provably identify that exact pair; equal style values do not prove ancestry.

Binding returns a new immutable analyzed context and identity while sharing its
already executed source, projection and Unicode analysis. It does not rerun
processing or mutate the old context. Clones retain their context identity; a
changed binding cannot reuse old relation handles, preparation or cache identity.
Previously issued analyses, preparations and finalized results remain valid for
their own captured contexts. The producer binds before preparing its composed
text result. Native fixtures may explicitly author these facts without claiming
live Style authentication. Text acquires no DOM tree or generic style interface.

If an operation requires a relation permission that the capture cannot establish,
return a typed missing-relation-wrap capability error for that operation. Do not
silently omit otherwise-required conditional opportunities or reject independent
ordinary/forced operations that do not consume the missing fact. This bounded
refusal leaves the producer/model extension required under #66/#75/#72. The true
ancestor owner follows the retained [Text4 line-breaking scope](../references/style-specialized-determinations-css-text-4--editor-capture-20261011--2bb0ce6fbd6f.md#line-breaking-scope);
the logical-before `overflow-wrap` owner remains the separately adopted #75 policy.
Disappearing-character and explicit-control authority remain distinct.

Hard-break precedence belongs to Text’s opportunity inventory under the [pinned UAX #14 dependency](../references/uax14--17.0.0--rev55-metadata.md). A soft or discretionary opportunity cannot precede an actual processed hard separator or semantic `ForcedBreak`, including across transparent occurrences. Preserve the forced-after opportunities, including consecutive hard separators. An original newline processed into collapsible SPACE is not a hard separator. Layout consumes those legal offers; available width does not determine their legality.

1. `System::prepare` obtains or retains the immutable analyzed source, resolves fonts and returns `PreparedText`. It does not choose the composed paragraph's line breaks. Its preliminary measurements/opportunities are explicitly preliminary wherever selected boundaries can change shaping or line-edge processing.
2. Layout’s consuming adapter lowers text units, opportunities and metrics to layout's validated inline inputs. Layout chooses a candidate logical range, break alternative and float-dependent span. It supplies the complete logical participant sequence, including atomic inline identities and directional controls, plus the relevant line context.
3. `PreparedText` validates a selection and creates `SelectedLine`. Private handles bind the selection to that preparation; validate range order, unit boundaries, break legality, selected replacement, atomic participant uniqueness and context. Selection context includes writing mode/orientation, start/end and forced/soft break facts, tab origin, available text span, indentation/hanging facts and exact spacing allocations. Do not pass unrelated paint policy into shaping.
4. Text finalizes that selection. `FinalizedLine` owns its selected preparation, the applied replacement and processed mapping, final effective font runs/glyphs, actual advances, line-edge/hanging facts, text metrics and outline ink, visual order including atomic placeholders, and caret/selection anchors. It returns line-local geometry. It never chooses a different break or places a box. Finalization applies the selected line's shaping boundaries, bidi line rules and text spacing to one result shared by all consumers.
5. Layout evaluates the returned metrics against its candidate span, including changes to float intersection caused by actual line height. It can choose another candidate/context and finalize again. After choosing a fitting line it supplies its space allocation and obtains the final geometry; it rechecks any resulting width/height feedback before committing. Preliminary metrics are never silently treated as final.
6. Layout supplies validated placement of those exact finalized lines, including line/fragment IDs, physical origin, writing-mode basis, baseline and atomic box rectangles, to text's placed-layout constructor. The constructor validates identities, completeness and geometry before returning a placed `Layout`. Painting, hit testing, movement, selection and accessibility consume this placed result. They cannot call a separate paragraph wrapper to recreate it.

Ordinary, conditional and non-placeable measurement terminations have distinct
issued authority. Conditional search binds the exact preparation, unchanged line
start/incoming, first genuinely formatted ordinary ceiling and purpose. Finite
overflow fitting uses `Overflow`; unconstrained minimum-content pieces use
`MinContent`. Maximum-content opens no search. Stages supply legal endpoint and
variant capabilities; the consumer owns actual search exhaustion and fit.
AutoPhrase's ordered restoration uses this same protocol and finalizer in native
and Standalone selection. Existing raw modes retain their characterized ordinary
inventory. Common owner, interval and phase validation applies without private
preparation inspection or alternate preparation.

If earlier ordinary Soft endpoints finalize without retained flow participation,
Layout may supply their actual `FinalizedLine` receipts to conditional enumeration.
Text validates complete ordered endpoint and legal-variant coverage from the same
unconsumed start. A mandatory or paragraph ending cannot be skipped. Zero advance,
empty ink and font-only metrics do not prove a phantom prefix; retained zero or
signed content still participates. A meaningful alternative prevents skipping its
endpoint even if it does not fit. The eventual selection retains every original
occurrence and revalidates contribution roles and allocation presence against the
receipts. A control with no strut cannot acquire an allocated strut, including a
zero strut, under that proof. Numerical geometry may legitimately converge.
Receipt allocations are proof provenance, not conditional opportunity identity.
Layout also filters phantom conditional trials before committing a line.

Finalization is deterministic for a preparation and complete selection context. Layout's feedback algorithm visits its finite candidate/context states until it selects a valid state or returns its existing no-fit outcome; revisiting a complete already-rejected candidate/context state is a typed feedback-cycle error (or the formatter’s explicit no-fit failure), even when feedback is identical and even for a cycle through multiple states. A stable fitting state commits; rejected states cannot be retried without a changed input. Do not introduce a magic maximum pass count. Layout must ensure progress in its own float/box algorithm. A no-fit line allowed by applicable overflow semantics remains a deliberate layout outcome, not a text finalizer fallback.

Layout supplies each eligible control's actual allocated strut geometry, qualified by its prepared control identity: finite baseline, line extent and alignment from the captured line-break or inline-boundary input. These are Layout contributions, not fabricated font metrics or atomic glyphs. Text combines them with its actual font strut during finalization; changed canonical contributions reenter height/float feedback. Pure formatting controls may have no allocated strut. Inline start/end boundaries do not create break opportunities merely by existing; zero edges preserve joining, while positive inter-unit edges bound selected shaping as specified below. Layout retains clear and box/float decisions.

Additional used box allocations enter `LineSelectionRequest::line_box_struts`
and `FlowObservationRequest::line_box_struts`. Each `LineBoxStrut` pairs actual
`LineStrut` metrics with alignment and joins the same metric grouping operation.
Numerical validation and request identity retain the complete list. An empty
list contributes nothing; it cannot supply missing metrics to an allocation-only
line. Text accepts numerical allocations; the consuming Layout adapter owns
their genuine box attribution, assignment and feedback, including avoiding a
second allocation for a contribution already owned by a selected control.

Native preparation explicitly captures its paragraph strut policy. A default-font strut resolves and retains the actual base-style face during preparation and contributes on every selected line, even when body spans use other fonts; a missing requested face is an error. Each body font contributes its own line-height strut, including its own half-leading, before Text combines baseline-, top- and bottom-aligned allocations. The allocation-only policy omits the base strut; it may prepare without a font when no other input requires one. Finalization requires actual contributions for every otherwise metric-less line, including a terminal empty line after a forced break. An explicit zero allocation is distinct from a missing allocation. Control presence and catalogue availability cannot silently choose this policy. These native strut roles follow the pinned [CSS Inline layout-bounds rules](../references/css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md#inline-height). Raw standalone layout retains its characterized source-adjacent metric policy; genuinely empty standalone text retains its admitted-font strut.

Metric completeness does not establish that composed CSS Layout contains a
formatted line. Layout applies the [phantom-line disposition](../references/text-wrap-edges--native-determination.md#paragraph-completion-and-retained-pins)
to each actual selection, independently of earlier source-node presence. A
phantom terminal completion retains its genuine ParagraphEnd, actual numerical
finalization, box/source custody and whole-paragraph coverage, but consumes no
formatted-line role or positive layout/rendered/caret rectangle. Its used block
extent is zero; real out-of-flow descendant positioning follows the CSS
zero-height rule. Numerical struts alone cannot make it a formatted line. Raw
standalone Text placement retains its separate native metric policy.

Retain the normalized formatter's deterministic alignment-group policy: let `b` and `a` be the independent maxima of signed before/after contributions in the baseline group, and `T`/`B` the largest top/bottom-aligned extents when present. Set `u = max(b + a, T)` and extend above by `max(B - u, 0)`; the resulting baseline is `b` plus that extension and the physical line extent is `max(0, u, B)`. An absent alignment group supplies no constraint, and an absent baseline group uses zero only as its coordinate origin. Preserve signed baseline contributions, font half-leading and overflowing box geometry; only the overall physical extent has a zero floor. This preserves the existing box/control formatter's choice where the pinned CSS Inline draft leaves tall top/bottom content positioning underdetermined; it is a Surgeist placement policy, not a claim that the draft specifies a unique baseline.

A native empty origin retains a zero-width, zero-height nominal source-fragment anchor at its actual finalized baseline and inline position. Its occurrence carries no independently captured font, so its fragment bounds cannot borrow a neighboring font or the whole line enlarged by unrelated boxes. Nonformatted placement retains this occurrence on its genuine finalized receipt without publishing a source or caret rectangle. Caret and accessibility geometry come from actual formatted lines. Raw standalone fragment geometry retains its characterized behavior.

The standalone `System::layout` convenience path retains the currently supported wrapping/alignment behavior by choosing lines through the same preparation/finalization machinery. Its simple paragraph chooser is private to text; layout’s composed CSS formatter does not use it as a second formatter. Unsupported style combinations remain explicit errors until the owning standards issue implements them.

The explicit native `TabContext::FontAdvance` policy uses the selected font's actual U+0009 glyph advance, preserving the standalone policy through the same finalizer. It does not approximate a tab with a space or implement CSS tab stops. `NoTabs` validates actual absence. The [CSS tab contract](#preserved-css-tab-stops) below defines the independently prepared CSS-stop operation and its required block capture. A taken discretionary opportunity requires its actual issued alternative, including an explicitly computed empty replacement; a natural paragraph end does not select a hyphen merely because the preceding source contains SHY. A source-defined replacement validates its real defining source occurrence. A control-defined replacement retains its exact control/opportunity/alternative owner and may have no text origin; incidental source-anchor provenance must be supplied and validated, never guessed from the numeric offset.

### Native box-edge wrapping

The [box-edge determination](../references/text-wrap-edges--native-determination.md)
defines actual-box propagation, independent edge/inside avoidance and early
mandatory-boundary custody. Layout owns applicability and the effective plan;
Text admits distinct computed mandatory causes before processing executes.
The post-analysis relation-permission binding above cannot introduce such a
hard boundary. Final admission revalidates influencing tree semantics and binds
the actual source, computed causes and prepared identities.

### Hanging punctuation and formatted-line roles

Text owns the normalized `HangingPunctuation` declaration, complete selected
typographic-unit classification and natural/allocated advance accounting. First
and Last use the candidate's actual formatting-container roles; ForceEnd is
unconditional and AllowEnd is conditional on natural overflow. Preserve the
[native punctuation determination](../references/text-hanging-punctuation--native-determination.md)
for effective properties, attributed advances and same-unit union. Hanging
changes fit, alignment and intrinsic contributions without removing glyphs,
font struts, justification participation, source pieces or interaction geometry.
Inline border/padding barriers use actual intervening selected-line endpoint
contributions. An off-selected joining-context allocation supplies no line-local
decoration and cannot independently veto hanging; captured enclosure alone is
also insufficient. The native determination defines this profile and the
current-fragment incidence required by an additional decoration capability.

`LineContext::formatted_line` supplies First and Last independently through
`FormattedLineContext`. Unknown differs from affirmative false. Text demands a
missing role only when eligible, unbarred punctuation needs it; this includes a
dependency visible only in an intrinsic measure or a zero-advance eligible unit.
`LineMeasurements::formatted_line_usage()` reports roles actually consumed,
including supplied false values. A role does not establish CSS block-end
whitespace authority. Candidate-specific roles belong to exact request/replay
identity, rather than a conditional search's stable branch context.

Layout owns actual formatted-line chronology and container/child propagation.
Only actual formatted lines count, including empty Forced lines and real
numeric-only atomic content. Retained nonformatted completions and rejected
phantom trials consume no formatted ordinal or role. Last dependencies use
authentic terminal mixed occurrences and actual
in-flow child summaries through the existing unpublished pass/replay gate.
Validate every consumed assumption before publishing geometry. A demanded cached
subtree without a matching summary is a semantic miss; no-demand formatting
retains its ordinary cache path. Speculative float summaries merge only when
the actual child packet is accepted.

Detached word/phrase measurements use authentic prospective roles. Text issues
their measurement-flow recipes and nonplaceable post-collapse participation
observations; consumers interpret those observations using their own emission
rules. Retained participation alone never proves a generated line. Preserve the
full manual-partition target, distinguish a genuine predecessor from a phantom
phrase prefix, and account for future Forced/empty/receiving obligations before
combining local closure with the external owner tail. Trials do not commit line
history, and intrinsic invocations do not borrow finite history. Standalone uses
the same punctuation kernel with its actual raw metric and emission profile.

### Native inline endpoint geometry

Layout owns used border/padding on each endpoint's own inline side. Its
[InlineBoundaryEdgeOf<S>](../crates/surgeist-layout/src/node_input/inline.rs)
constructor checks each component and their sum in scalar `S`, with explicit
`ZERO` for a genuine used zero. `InlineBoundaryInputOf::new(kind, metrics, edge)`
requires that value. Lower the one checked `edge.advance()` result once to Text's
`f64` total; do not widen the components separately and recompute the sum. Layout
retains the components, Start/End role and own writing-mode/direction for output.
Text's [InlineBoundaryAllocation](../crates/surgeist-text/src/protocol/mod.rs)
checks the finite nonnegative total and normalizes negative zero. Unknown Style
geometry is not zero; no compatibility constructor, Boolean barrier, padded
atomic substitute or new font/style supplies a missing endpoint fact.

#### Selected and context-only admission

Every selected InlineBoundaryStart/End has a `ControlAllocation` with
`boundary: Some(...)`, including affirmative zero; non-boundary controls have
`None`. The existing optional strut remains an independent vertical allocation.
Omitted controls yield `MissingControl`, missing endpoint geometry yields
`MissingControlBoundaryGeometry`, and wrong-role geometry yields
`InvalidControlGeometry`. Foreign, duplicate and unexpected controls retain their
existing errors.

An endpoint outside selected flow can still affect original joining context.
`LineSelectionRequest::boundary_context` supplies these genuine local
`InlineBoundaryContextAllocation` rows separately: they allocate no flow, strut,
placement or fragment, and cannot duplicate selected controls or clone a remote
paragraph's global endpoint. [PreparedText::boundary_context_demand](../crates/surgeist-text/src/protocol/selection.rs)
validates the complete candidate and returns the next missing `PreparedControl`,
or `None` when its consumed context is covered. It performs no shaping, fitting,
font-catalogue lookup or preparation mutation. Layout resolves each demand from
the authenticated control-node binding and explicit normalized used edge.

Supply rows in deterministic first-demand encounter order. A partial query admits
only an initial prefix; a direct `select` request may supply the whole list in
that same order without querying first. A proved positive barrier stops the
outward context walk; zero continues to the next influential endpoint. Unrelated
or unreachable endpoints are not demanded. `select` returns
`MissingBoundaryContext` for an unanswered demand, `DuplicateBoundaryContext` for
duplicates/selected overlap, and `UnexpectedBoundaryContext` for wrong-role,
out-of-order or surplus rows. Validate handles, roles, numbers and canonical
coverage before publishing a selection; never invent zero for a missing fact.

#### Joining, cuts and ordered contributions

Text derives endpoint incidence from actual captured events and typography-safe
cuts. At an authentic Soft opportunity, closing events attach to the outgoing
line and opening events to the incoming line, using genuine event/scope incidence.
Retain opportunity ownership, positive-source deleted occurrences and distinct
disappearing-SPACE cuts. Do not invent a cut, strand a closing endpoint at an
artificial earlier Soft end, or rewrite Forced/domain/atomic chronology.

Positive inter-unit endpoints bound the selected original and presentation
joining domains, including consumed off-selection original context and computed
Before/After attachments. Incoming indications retain their
[receiver-side mixed-event custody](#hyphenation-indication-bidi) on either zero
or positive edges. Zero preserves permitted whole-word context across an
unforced Soft cut. Rebuild only selection-local boundaries/assignments using the
retained font instances; do not requery a catalogue or mutate preparation.
An endpoint inside an indivisible typographic unit remains a numeric contribution
with its own interval, while the unit retains shared glyph associations and
provider positions. Count the edge once before positioning the next unit;
never move an attached mark independently or introduce an interior break.

One canonical visual-order contribution supplies raw/occupied flow, fit,
alignment, MinContent/MaxContent, tabs, prefixes, placement and overflow. Positive
selected edges count as retained flow; context-only rows contribute nothing.
`FinalizedPrefixView` retains the same provider-closed grouping and exact source
chronology. CSS tab resolution includes preceding endpoint advances in the real
pen and recomputes after spacing. Layout's numeric box/control-only path consumes
the same used advance through Text-owned ordering without inventing text rows.
Every ordinary/conditional/measurement trial and float/justification retry uses
the complete resolved request and actual finalization; preliminary glyph widths
cannot replace edge-aware receipts. Layout never adds the width again afterward.

Collapse/PreserveBreaks edge removal remains transparent to inline controls.
For remaining-separator hanging, zero endpoints are transparent and a positive
endpoint stops the scan locally; a later eligible sequence on its edge side may
still hang. The endpoint's own advance is ordinary flow, never whitespace
exclusion. In the Preserve/PreserveSpaces case with wrapping enabled, Start0 and
a real Forced ending, X10 plus SPACE10 with End0 at width15 has raw20, fit10,
alignment15, MinContent10, MaxContent20 and actual hanging5. End(border2,padding3) gives raw/fit/alignment/MinContent/MaxContent25,
with no hanging through that edge. Natural eligibility remains frozen through
allocated-spacing replay; this is no global scope-wide disable-hanging flag.

#### Output, identity and failure

| Observation | Endpoint custody |
| --- | --- |
| `FinalizedControlView` | `inline_bounds()` is the checked physical left/right interval; `slot().x` is left, with the actual control baseline. Alignment translates both ends. |
| `FinalizedFragmentOwner::Control` | The same source-free occurrence gets the interval at actual line-metric height. Original/local/aggregate/processed source ranges remain absent; no glyph ink is invented. |
| Layout endpoint node | Publish a width×0 footprint and its own-side border/padding, using actual endpoint writing-mode/direction and Start/End role. Preserve the Control fragment without replacing this footprint with a text-fragment envelope. Numeric-only flow publishes the same footprint without Text fragments. |
| Virtual-separator Scope fragment | Retain only that separator's actual local geometry, empty authored association and canonical global scope identity. Several separators keep separate occurrences; no separator means no Scope fragment. No endpoint strip is donated to them. |
| Original source and exact placed positions | Endpoints add no UTF-16, source Range membership, glyph or plaintext. Exact relation-event Before/After positions distinguish the two interval sides: even level maps to left/right, odd to right/left. Exact selected-event ranges include the endpoint's actual line-height interval even with glyphs selected; source-offset queries do not invent that membership. |

The node footprint and Control fragment observe one contribution, not two widths.
Source/caret/Range, Render and accessibility consume the same displaced genuine
geometry and retain source-free endpoint custody. Global scope pairing remains
capture-owned; paragraph cuts manufacture no continuation endpoint or envelope.

Selected allocations and the exact reached context list enter request equality,
candidate reuse, spacing replay and cycle checks. Layout retains dependencies on
each demanded endpoint, including off-selection nodes. A changed edge or
candidate/alternative/incoming/purpose request reruns demand: zero↔positive can
reveal or hide farther context. Geometry-only changes reuse the same preparation
but rebuild affected joining, glyph, prefix, fit, tab, fragment and float results;
unreached endpoints enter no Text request identity. Rigid translation can reuse
an unchanged complete receipt. Missing facts, numeric/shaping failures and late
feedback refusal publish nothing and leave earlier completed results immutable.

This operation consumes native used geometry. It does not resolve live Style or
authored percentages, add vertical Text, or construct full inline-element
decoration envelopes/background/border painting. It does not change raw
standalone whitespace, the six modes, #74 or control/default-ignorable rules.

### Native control-character presentation

This is the selected #64 contract; implementation qualification remains with
that open issue. Host-defined segment-break interpretation precedes this
operation. Eligible characters are Cc excluding TAB, LF, CR and actual
document-defined segment-break occurrences. A supplied NUL is eligible;
undeclared VT, FF and NEL are visible symbols rather than breaks or whitespace.
The [source determination](../references/text-control-presentation--native-determination.md)
distinguishes normative visibility from the native property choices below.

Text supplies an always-selected deterministic code-point box using a private
real font resource. Its FontData, EffectiveFontInstance, glyph geometry, scale,
metrics and Render paint must describe the actual resource. It is a bundled UA
presentation resource usable under RegisteredOnly, outside caller registration
and system-font discovery. It cannot serve unrelated missing characters or
default-font struts. Retained snapshots keep their real resource lifetime.
Font-provided control visibility is not the selection criterion. Zero font
size, clipping and transparent styling do not constitute synthesis failures.

The native ordinary-symbol presentation profile is:

| Property | Selected value |
| --- | --- |
| General category and script | So and Common |
| Bidi class | ON |
| Grapheme, word and sentence break | Other |
| Line break | AL |
| East Asian width | N |
| Joining type and canonical combining class | Non_Joining and 0 |
| Whitespace, default-ignorable, case, letter, number and emoji properties | Absent |

Apply that same per-occurrence profile throughout presentation, including
shaping. This is neither a Unicode database mutation nor U+FFFD substitution.
Retain exact original UTF-16, scalar identity, source ranges, copy and plaintext.
Controls are bidi neutrals and interrupt cursive joining. Following combining
marks retain ordinary symbol-plus-mark grapheme behavior; font fallback cannot
create a segmentation boundary or replace usable marks with missing glyphs.
AL participates in normal line-breaking and word-breaking rules without
introducing unconditional opportunities or an indivisible whole-word rule.

### Native default-ignorable fallback

This selected #64 contract distinguishes genuine native shaping from fallback
display. The pinned [Text 4 whitespace rules](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#white-space-processing)
require unsupported Default_Ignorable characters to be ignored for rendering.
The [Unicode rendering guidance](https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-5/)
and [unsupported-character FAQ](https://www.unicode.org/faq/unsup_char.html)
distinguish normal rendering, including effects on neighbors, from invisible,
nonadvancing fallback. They do not prescribe a provider receipt or a universal
test of font intent. The following support boundary is native policy;
implementation qualification remains with the open issue.

Retain original characters and meaningful joining, variation, bidi, break,
SHY and ZWSP effects through native shaping. A completed UVS, normalization
composition or actual GSUB/AAT action is a native participation witness,
including an identity substitution; nominal cmap coverage or GDEF glyph class
alone is not. Preserve actual surviving joint glyphs and their metrics.
An earlier participation witness cannot authorize an output that a later
native action deletes.

Use the selected Unicode 17 full Default_Ignorable property independently of
the provider's lookup-skipping predicate. Unsupported residuals have no paint,
shaped advance or independent letter/word spacing, including U+180F, U+1BCA0–3,
reserved members and nominally visible glyphs. Zero advance alone does not prove
invisibility, and replacing a residual with the selected font's SPACE is not a
paint guarantee. Failed variation sequences retain their ordinary base without
independent selector display.

Exactly U+115F, U+1160, U+3164 and U+FFA0 retain the provider's recognized
spacing-filler role when the actual selected face supplies a nonzero nominal
glyph. This narrow resource exception does not validate arbitrary font ink or
extend to other mapped default-ignorables. Missing, unsubstituted fillers fall
back invisibly; genuine native filler consumption retains its actual joint
result. Keep lookup participation independent from fallback suppression.

The maintained provider tracks only original default-ignorable input ordinals
and their actual descendants through normalization, substitution, insertion,
ligature transfer and deletion. Ordinals count invocation-body scalars,
excluding pre/post context; they are neither UTF-16 offsets nor caller cluster
integers. Cluster merging does not transfer ancestry. Synthetic repair output
does not inherit an anchor's ancestry merely through a low-level row copy.
An actual AAT insertion inherits its selected anchor as normal output while an
unchanged unresolved anchor remains independently residual.

Classify each original candidate from final surviving descendants: Normal when
any normal descendant survives, Residual when only unresolved descendants
survive, and Deleted when none survive. Retain ancestry after substitution so
later partial or total deletion is observable. Separately identify every final
unresolved output slot; an input can have both a normal descendant and a
residual slot. Suppress the latter without erasing the former.

The provider publishes one immutable, status-gated borrowed receipt containing
sorted, duplicate-free residual-input ordinals, deleted-input ordinals and
residual final-glyph indices. Finalization precedes Complete and charges all
discovery, carrier, union, deletion, relocation and receipt work/storage to the
existing aggregate invocation limits, alongside joining and control metadata.
No partial receipt is admitted after refusal. Empty and all-deleted output
still cross this fence. Clear/reuse resets input identity and receipt contents;
readback performs no allocation or shaping work.

Text maps admitted input ordinals through the exact emitted presentation atoms.
It omits only residual output slots from paint and excludes Residual/Deleted
inputs from advance division and independent spacing. The last eligible
component receives the exact advance remainder. A group with no eligible input
cannot receive surviving normal advance; inconsistent provider output fails
admission. Preserve authentic provider groups, semantic graphemes, source-only
pieces, affinities, exact original UTF-16, copy and plaintext. A computed visible
hyphen is classified by its actual emitted atom, not its authored SHY source.

### Native whitespace processing modes

Text's `WhiteSpace` values are Collapse, Discard, Preserve, PreserveSpaces,
PreserveBreaks and BreakSpaces. Genuine captured trimming runs before Phase I;
source-local processing retains its narrower explicit source/delimiter inputs.
The [Text reference](../crates/surgeist-text/docs/reference.md#source-local-whitespace-processing)
records each mode's lexical operation and original-source correspondence.

PreserveBreaks converts TAB and ordinary CR to collapsible U+0020, collapses
adjacent eligible spaces across genuine Collapse/PreserveBreaks spans, and removes
collapsible SPACE/TAB around actual delimiters within their real domain. Each
surviving delimiter becomes one LF; duplicate-break collapsing and contextual
break transformation do not apply to these LFs. Consecutive delimiters therefore
produce distinct Forced opportunities, including under `Wrap::None`. Explicit
exterior trimming may remove original delimiters before this preservation step.
Each disappearing original SPACE retains its actual containing style's Wrap gate.

BreakSpaces preserves SPACE/TAB and actual delimiter LFs; ordinary CR becomes
preserved SPACE. Preparation issues an opportunity immediately after each eligible
preserved SPACE/TAB or other space separator at that character's actual end, only
at a typography-safe cut with genuine neighboring/NCA wrapping permission.
Forced, explicit and atomic owners retain precedence. The mode supplies no
before-first-space opportunity and cannot relocate a suppressed interior SPACE
cut to the far end of a combining typographic unit. Its preserved TAB uses the
same genuine font-advance/CSS-stop demands as Preserve.

Follow the [glue source reconciliation](../references/css-text-break-spaces-glue--source-reconciliation-20261024.md)
and the retained [Unicode 17/revision 55](../references/uax14--17.0.0--rev55-metadata.md)
boundary rules: U+2007/U+202F retain their own GL protection, and WJ/ZWJ and
indivisible typographic units remain protected. A following GL is not a blanket
veto: the adopted SPACE/TAB-before-GL exceptions remain legal when their actual
boundary permits. Preserve genuine higher-priority ZW opportunities. This
contract does not adopt Unicode 18/revision 57's changed BA exception.

These modes execute through native captured, analyzed and prepared paragraph
paths. Raw standalone formatting retains its Preserve-only profile: new modes
refuse with `ErrorDetail::UnsupportedTextStyle`, the matching
`TextStyleFeature::{WhiteSpacePreserveBreaks, WhiteSpaceBreakSpaces}` and
`UnsupportedTextStyleReason::RequiresProcessedSourceLayout`. Original UTF-16,
deleted-source affinity and partial plaintext retain their existing custody.
Copied plaintext preserves actual LF/SPACE/TAB outcomes without serializing Soft
wraps, transformations or tab placement. PreserveSpaces retains #74's separate
TAB/delimiter-to-SPACE conversion and Preserve-equivalent hanging; its converted
LF is not Forced. Collapse, Discard and Preserve retain their own rules.
[Native endpoint geometry](#native-inline-endpoint-geometry) supplies the separate
used border/padding contribution without changing these mode meanings.

### Line-box direction and accepted-line context

The [Text 4 line-box direction rule](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#bidi-linebox)
separates the containing block, the bidi paragraph and the line box. Layout's
normalized `LineBoxDirectionPolicy::Normal` uses the actual containing direction.
`Plaintext` uses the full bidi paragraph's first-strong direction for a candidate
with eligible strong content; a weak or empty candidate inherits the preceding
accepted line-box direction, or the containing direction on the first line.
These normalized inputs do not implement a computed Style producer or all
`unicode-bidi` semantics.

Text owns immutable strong facts from the canonical body after Phase I and
semantic transformations. `PreparedText::line_direction_facts` authenticates its
issued start/end boundaries and returns `LineDirectionFacts`: the optional full
bidi-context strong direction and whether that actual candidate contains an
eligible strong character. Absence is affirmative no-strong, independent of the
captured resolved body base. The common P2/P3 owner excludes isolate contents
with their true prefix context; a suffix query cannot reset isolate depth.
Actual evaluated/generated body content counts, while a selected hyphen or
continuation indication cannot supply paragraph authority.

Facts are indexed once over actual body contexts, including B-class separators
and genuine ForcedBreak LF. A nonempty query may include its context's terminating
separator but cannot cross into another context. An empty query at an internal
seam belongs to the right context; terminal empty queries retain that empty
context. Foreign, reversed and cross-context queries fail rather than returning
a fabricated LTR value. Each native or captured paragraph declares its actual
body extent; numeric adjacency cannot join separate preparations into a larger
bidi paragraph.

Every selected `LineContext` supplies a resolved `line_box_direction`, separate
from its validated `base_direction`. Text consumes the former for Start/End
alignment, indent side and overflow placement. It preserves the latter's body
levels, ordering, mirroring, shaping, source/caret custody and physical whitespace
exclusions. CSS tabs keep their separately captured nearest-block/grid basis.
The raw standalone profile explicitly selects its own resolved line-box basis;
it does not claim to be a CSS Plaintext producer.

Layout resolves each ordinary, conditional and measurement candidate from its
actual range. A failed wider strong candidate cannot freeze the direction of a
successful weak prefix. Only actual accepted lines, including genuine empty or
Forced lines, advance the containing formatter's previous-direction state.
Speculative trials, float/height retries and detached measurements do not.
Carry accepted state across runs in one container and reset it in a new
container. MinContent and MaxContent have independent invocation-local chronology;
query order and finite cache history cannot change their result. Actual policy,
containing basis, incoming accepted state and resolved candidate context remain
part of captured pass/request/replay identity.

### Native text-group alignment

The [native group-alignment determination](../references/text-group-alignment--native-determination.md)
owns same-BFC membership, natural pre-justification measurement, nested residual
padding and per-line side resolution under #68. Text's alignment-only content
frame is distinct from the outer line frame and earlier wrapping/tab/hanging
basis. Layout resolves groups and spacing before publishing coherent geometry.
The determination defines required behavior; #1478 tracks its implementation.

### Native whitespace edges and hanging

Text owns selected-line Phase II classification under #64/#72; Layout consumes
its measurements for fit, intrinsic sizing, justification and placement. This
native profile follows the pinned [Phase II](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#white-space-phase-2)
and [hanging definitions](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#hanging),
with #74's [preserve-spaces determination](../references/SOURCE-EDITIONS.md#operative-source-decisions-by-issue).
It selects the required operation, without claiming unqualified feature support.

Classify actual processed code points, governing styles and canonical line order.
Other space separators are Unicode Zs excluding U+0020 and U+00A0. NBSP cannot
inherit their removal or hanging rules. Collapse and PreserveBreaks remove their
applicable leading/trailing U+0020 and trailing U+1680, retaining leading U+1680
and exact source custody. Their remaining eligible separator sequences hang
unconditionally, including under nowrap; PreserveBreaks still retains actual
LF/Forced chronology. Surviving
Discard characters retain ordinary advance, including original U+3000 and
separator-generated U+0020/U+3000. Discard supplies no conditional or
unconditional whitespace-rule hanging at soft, forced or block ends, including
nowrap, and no hanging-only terminal/available-edge requirement. Independent
applicable rules remain separate. This is the
[native Discard determination](../references/text-word-space-generation--native-determination.md#surviving-discard-characters-in-phase-ii).
Under Preserve
with wrapping enabled, eligible trailing sequences hang unconditionally at a real
Soft ending and conditionally before an actual forced break or block end.
PreserveSpaces applies that same Phase II rule after its own Phase I conversion;
a converted LF does not supply a forced break. No preservation-rule hanging
applies under nowrap. BreakSpaces whitespace processing retains every preserved
SPACE/TAB/separator contribution without line-edge removal, whitespace hanging or intrinsic omission at Soft,
Forced or BlockEnd, under either wrap policy. These rows retain ordinary flow
and stop preceding hanging groups, even at zero advance. Fit, alignment,
MinContent and MaxContent consume the same retained contribution. MinContent may
choose genuine after-each opportunities; MaxContent still honors actual Forced
lines. This mode therefore depends on its prepared opportunity inventory as well
as finalization, rather than a finalizer flag.

An independently eligible punctuation cause still applies in Discard or
BreakSpaces. For example, First can hang a surviving U+3000 on an actual first
formatted line; its whitespace mode alone cannot erase that permission. Union
the causes by actual contribution identity instead of omitting a unit twice.

An issued paragraph ending proves preparation completion, not CSS block end.
Native input retains a distinct `ParagraphTerminalEdge` capture: `Unknown`,
`BlockEnd` or `NonBlockBoundary`. An explicit native producer supplies its genuine
normalized fact. Contextual processing derives it from actual paragraph/domain
custody and the owning block scope's exit, including ordered empty and out-of-flow
occurrences; neither numeric extent nor paragraph-table order establishes it.
Failure to prove BlockEnd means Unknown unless positive evidence establishes a
non-block boundary. Retain this fact through analysis, preparation and reuse.
Require missing terminal authority only when an eligible sequence's measures
consume it; a known non-block boundary cannot manufacture a Soft or Forced break.
Real Forced/Soft, NBSP and other independent cases do not acquire that requirement.

A zero natural contribution can leave terminal authority unconsumed. Retain that
dependence for the actual row and any earlier terminal group it governs. After
supplied spacing and tab recomputation, refuse atomically with the missing or
unsupported cause if an affected measure, occupied interval or group result now
depends on that authority. Unrelated ordinary content does not trigger refusal;
net allocation or total line advance cannot prove independence. Successful
immaterial rows report `UnconsumedTerminal`, without asserting an ordinary
classification or supplying block-end authority. This guard does not reclassify
frozen natural eligibility or terminal groups.

One Text-owned receipt distinguishes raw signed flow and physical geometry from
fit advance/occupied bounds, alignment/justification measure, actual hanging and
MinContent/MaxContent contributions. Consumers cannot subtract generic trailing
whitespace. Unconditional hanging excludes its contribution from those measures;
conditional hanging excludes the full eligible contribution from fit/MinContent,
retains it for MaxContent, and excludes only actual overflow for alignment and
justification. Thus X10 plus SPACE10 in width15 retains raw20, fit10 and actual
hanging5, alignment15, MinContent10 and MaxContent20; in width40 it hangs0
and centers as a 20px sequence. Preserve signed flow
separately from physical outward overlap, without taking absolute values or
clamping signed contributions. A hanging glyph retains its advance, source,
caret, Range and paint geometry.

An unconstrained native selection retains one purpose-neutral Text metric frame,
derived from its alignment measurement. Layout selects the corresponding intrinsic
contribution for its sizing report and allocation; that choice does not replace
the actual placed Text metric bounds or retained fragment geometry. For the
conditional X10 plus SPACE10 case, MinContent can contribute 10px while the actual
metric and fragment observations reach 20px. An intrinsic query does not impose a
finite used width of 10px. Layout anchors the Text frame and intrinsic allocation
with their respective physical widths, including in RTL. Overflow consumes actual
geometry independently.

Determine conditional overflow before justification from the actual natural
candidate, including authored spacing and tab resolution. Earlier unconditional
groups become terminal only when every subsequent conditional glyph actually
hangs. Apply the corresponding inclusion rules separately to each intrinsic
purpose; an absent width is not a purpose tag. Allocated-spacing finalization
uses the same owners and recomputes tabs and feedback. Alignment cannot decide
the natural hanging condition again. Preserve complete allocation/replay identity
and the existing cycle/failure rules.

For allocated conditional hanging, [adopted native policy](../references/SOURCE-EDITIONS.md#conditional-hanging-after-spacing-allocation)
freezes the natural candidate's eligible units, relevant edge/frame and terminal
groups. After applying the complete allocation, recompute those eligible units'
excluded contributions in that same prealignment frame. Neither the original
pixel exclusion nor a fraction of the unit's advance is fixed. In coordinates
increasing toward the relevant line end, a qualifying contribution traversing
from u to v beyond edge B has signed exclusion:

```text
max(v - B, 0) - max(u - B, 0)
```

Subtract that signed contribution from raw flow for alignment. Physical outward
overlap is separately the intersection of the geometric segment with the outward
half-line; reverse advances must not become positive exclusions by absolute value.
Use actual contribution ownership, including upstream displacement, without
double-counting spacing. For X10 plus SPACE10 at width15, adding 4 to the space
gives raw24, hanging9 and alignment15; adding -8 gives raw12 and hanging0.
A naturally fitting conditional unit remains ineligible after later expansion.
An eligible unit can contract to zero exclusion without reclassifying other units
or terminal groups. Alignment translates the result without reevaluating this
receipt. Changed breaks or ownership rebuild natural classification through replay.
The separate fit/MinContent/MaxContent inclusion rules remain unchanged.

[Native endpoint allocations](#native-inline-endpoint-geometry) supply actual
border/padding advances and local hanging barriers. Zero controls do not prove
uncaptured styled boundaries have zero edges.
This profile does not silently replace raw standalone wrapping/alignment policy.
Keep that existing operation explicit while sharing mechanics and truthful
receipts; no compatibility alias or second whitespace formatter is selected.

### Preserved CSS tab stops

Text owns preserved-tab advances and source custody; Layout owns the actual block
edge, line span and placement supplied to finalization. This contract selects the
CSS operation under #64/#72, independently of native font-advance tabs. It follows
the pinned [Phase II](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#white-space-phase-2),
[tab-size](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#tab-size-property)
and [operation order](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#order).
The [source determination](../references/SOURCE-EDITIONS.md#preserved-css-tab-stops)
separates normative requirements from the native policies below.

Capture the actual nearest block's style and explicit used direction separately
from the paragraph's base style and bidi direction. Contextual inputs reuse their
real block-domain style; flat inputs supply that reference explicitly. Preparation
chooses either font-advance or CSS-stop demand independently of the capture.
Resolve every selectable original and replacement TAB under the chosen demand
before publishing the preparation. Missing required facts or resources fail
atomically. A later request must match that prepared capability; `NoTabs` instead
checks actual selected absence. Retain every consumed font instance and include
the capture, style and demand in their owning cache identities.

A checked tab size is either a finite nonnegative number or absolute length,
initially eight spaces. A number multiplies the nearest block's hypothetical
SPACE advance including that block's letter- and word-spacing. A length bypasses
that measurement. A nonzero resolved interval uses half of the hypothetical ZERO advance
under the tab's own style for its threshold. Resolve these metrics through their
actual font-selection authorities, without fabricated authored characters or
neighboring-font substitution. The pinned [ch definition](../references/css-values-4--WD-css-values-4-20240312--7b6b68c34d00.md#ch)
supplies the horizontal 0.5em fallback when measurement is impossible; half-ch
then means 0.25em. Retain measured-versus-fallback provenance. Scale style lengths
and spacing once under the existing preparation coordinate contract.

The horizontal hypothetical metric operation shapes each scalar in isolation
under its actual owning style. Both scalars use Text's existing Common-script
Latin fallback. SPACE follows the nearest block's captured used direction;
ZERO uses LTR, matching its isolated EN analysis even in an RTL paragraph.
Retain direction-dependent provider features as well as explicit font settings.
This is the selected isolated metric profile, not a requirement to borrow script
or shaping context from neighboring text, nor a claim of arbitrary contextual or
vertical metric support. The source clauses require influencing glyph-selection
settings; they do not prescribe a nominal-only measurement algorithm.

Authored zero suppresses advance and paint without SPACE, ZERO or TAB probes.
A spacing-derived zero may consume SPACE to establish zero, then needs no
threshold metric. Preserve original source, caret endpoints and selection custody
even at zero extent. Nonfinite arithmetic, overflow and nonzero numeric collapse
are distinct numeric failures, never semantic zero. A negative specified tab size
remains invalid; a negative interval derived from valid sizing and actual spacing
is a separate state.

For used layout, use physical +x coordinates temporarily, with unaligned paragraph progression
start defined as zero. The initial pen is +indent for LTR and -indent for RTL.
Layout supplies the actual block content-start minus actual paragraph-start
displacement in this frame, recomputed for each float band and retry. Direction
alone does not prove that displacement. Retain the signed finite resolved
interval T and its metric provenance. For T != 0, use grid period D=abs(T):
O+k*T and O+k*D describe the same stop set for signed integer k. With physical pen
p and paragraph sign s=+1 for LTR or -1 for RTL, let q=s*(p-O). Select
k=floor(q/D)+1, then increment k exactly once if k*D-q is below the authentic
half-ch threshold. Equality does not bump. The destination is O+s*k*D; aligned
pens advance to the next stop. Use mathematical floor for negative coordinates.
Do not loop until the threshold is reached, advance backward or clamp to zero.
An unrepresentable consumed phase, gap or destination is a numeric failure.

Applying this forward grid to negative spacing-derived intervals is the
[adopted native policy](https://github.com/bj-data-eng/surgeist/issues/64#issuecomment-6374973383).
The magnitude is a grid period, not a rewrite of the signed SPACE metric or
specified spacing. Pinned CSS and inspected WebKit do not establish this
negative-interval algorithm. Extending the grid beyond block edges is also an
explicit Surgeist policy, not an established browser or CSSWG resolution.
For example, block RTL origin 100, paragraph LTR pen zero and period 40 select
stop 20 when half-ch is at most 20; a larger threshold selects 60. With Ahem's
20px SPACE, zero letter-spacing, `word-spacing: -21px` and `tab-size: 4`, T=-4px.
For a 20px ZERO, half-ch=10px, O=0 and pen=0, LTR advances to 8px and RTL to -8px:
the one bump may still leave a gap below half-ch.

Resolve after canonical line bidi ordering, using actual ordered text, atomics
and supplied spacing. A CSS tab has source-owned, glyph-free cluster content;
it does not acquire a dummy font run, provider TAB width or extra tab spacing.
RTL traverses reversed canonical visual progression toward negative x. Retain
only resolved cluster advances, not that temporary signed pen, in logical prefix
receipts. Existing positive-x visual emission and later alignment/placement use
those same rows. Indentation influences tab distance and ordinary placement once;
alignment does not reevaluate the grid. Actual struts or supplied allocations
provide line metrics; font-dependent observations of genuinely fontless content
require their own precise missing-metric result.

Changed spacing allocations require tab recomputation: expansion before a tab
may be absorbed by a shorter tab advance. Layout uses actual finalized feedback
for fit, floats, intrinsic sizing and justification, retaining request identity
and cycle detection.

For unconstrained min-/max-content measurement, select a logical measurement
grid from the unindented paragraph line-start, advancing in paragraph direction.
Initialize its pen with the applicable resolved indent and accumulate actual
canonical advances. The nearest block still owns the period metrics and the
tab's style owns half-ch. Reset this grid for each measured line under the
existing first-line, each-line and hanging-indent rules; intrinsic percentage
indent retains its existing zero treatment. Inline edges contribute advances
without resetting the grid. The signed logical position q uses the same
forward-stop formula and exactly one half-ch bump defined above.

This is the [adopted intrinsic convention](https://github.com/bj-data-eng/surgeist/issues/64#issuecomment-6389474658),
supported by the [complete pinned Blink path](../references/blink-intrinsic-tab-phase--daf70f9ddee4.md).
It does not assign a physical value to an unknown block edge or infer it from an
incidental placement frame. Request/context and cache identity distinguish this
operation from used layout. Finite placement recomputes tabs from the real
block content-start edge with actual float/indent displacement; intrinsic
geometry cannot be reused as placed geometry. No width self-consistency solver
is selected. These origin choices do not change opportunities, bidi, hanging,
spacing or source custody. The convention is native policy completing a source
gap, without a CSSWG-resolution or executed browser-parity claim.

`TabContext::IntrinsicCssStops` explicitly requests this unbounded measurement.
It requires CSS-stop preparation and no finite available width. Used layout
instead supplies `CssStops { origin: Some(actual_origin) }`; an unavailable used
origin remains `None` and a selected nonzero tab reports missing `TabOrigin`.
Absent or zero tabs do not consume that origin. Width absence alone never
selects the intrinsic operation. Retained requests and cache fingerprints keep
the modes distinct even when their numeric advances coincide.

For Ahem10, period40 and half-ch5, Preserve plus Word wrapping, zero-indent
`A TAB B` has max-content advance50. Min-content has raw line advances40 and10,
issued contributions10 and10, and aggregate inline extent10: the first line's
preserved trailing TAB hangs at its Soft ending. Its30px remains in raw/source
geometry. These intrinsic results hold across all four paragraph/block
direction pairs, independently of incidental placement widths. Used LTR layout
with real RTL block edge100 has raw advance30 instead. Assert the raw,
purpose-specific and placed observations separately. Qualification must cover actual
Text and Layout operations, source/interaction geometry, original and replacement
tabs, metric resources, numeric failures and retry purity before support is claimed.

For selected discretionary replacements, preserve original characters' joining
decisions from the authentic unbroken processed source and its actual context
barriers, as required by the pinned [word-break shaping rule](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#word-break-shaping).
Original and computed ownership alone does not introduce a shaping boundary.
Otherwise-compatible presentation retains joint substitution and positioning,
including genuine mixed graphemes, ligatures and cross-cluster adjustments.
Cluster membership and dependencies observed before correcting joining do not
certify that independently shaping those contributions would be equivalent.

The maintained shaping provider owns the operation separating original joining
analysis from emitted presentation. Text's private adapter supplies validated
original-to-presentation correspondence; the provider preserves it through its
own normalization before applying joining decisions, substitution and positioning.
Decomposition, reordering and composition must retain sufficient correspondence;
ambiguous association returns a precise refusal. Text does not implement another
joining state machine, inject arbitrary feature masks, reuse captured glyphs or
repair positioned advances afterward. Completion and refusal accounting cover
the complete operation before any finalized result is published.

For computed characters, derive joining behavior from the actual selected
presentation; original contributions retain their independently determined source
joining decisions. The computed channel's Unicode context is the complete emitted
selected-line presentation within its genuine context barriers. It includes
selected characters in other font or metric runs and excludes unselected source
outside that line. Font and instance boundaries constrain joint substitution and
positioning without introducing Unicode joining-context barriers.
This is the adopted Surgeist policy for admitted computed
replacements, rather than a CSS-prescribed algorithm for arbitrary strings.
Preserved joining decisions do not require identical glyph IDs when legitimate
contextual substitutions or ligatures change the emitted glyphs. Qualification
distinguishes those decisions from unrelated glyph identity, exercises actual
cross-boundary positioning and normalization, and preserves forced/disconnected
barriers and failure atomicity. A precise capability refusal leaves the affected
implementation requirement incomplete; it does not establish a shaping boundary
or permit weaker mixed-interaction expectations.

### Hyphenation policy and indication ownership

Text owns hyphenation opportunity production, indication choice and indication
placement as distinct decisions. Under `none`, suppress explicit and automatic
hyphenation opportunities, including an otherwise unadorned SHY; preserve ordinary
breaks at visible hyphens. Under `manual`, authentic authored conditional markers
need no dictionary. Automatic opportunities require both known content language
and an appropriate actual resource; an indication fallback supplies neither.
These rules follow the pinned [hyphens property](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#hyphens-property).
Layout retains candidate priority, fit, intrinsic sizing and committed-line limits.

Resolve an indication convention with its actual basis: supported language and
writing system, supported declared-writing-system fallback, or the explicit
unknown-convention default below. Follow [Appendix F](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#script-tagging):
an explicit script overrides the language's usual script; absent language-specific
knowledge, use supported conventions of the declared writing system. Unknown
language does not prevent manual hyphenation. A known applicable convention that
requires unsupported behavior returns a precise capability error; the unknown
default cannot certify that convention or complete that implementation scope.
Native captured convention facts remain distinguishable from Text-owned rules
and do not prove web-language coverage.

Preserve a specified indication string exactly, including empty. Its placement
follows the applicable convention, defaulting to logical immediately before the
break when no other placement convention is established, as the pinned
[hyphenate-character property](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#hyphenate-character)
requires. A known required after-break or two-sided placement needs its actual
capability rather than a substituted before-break insertion. An empty string
still selects a real hyphenation ending and retains joining and source semantics;
it needs no visible indication font or placement operation. No truncation is
required; any adopted truncation preserves whole typographic character units.

In a captured native convention, `BothSides` emits the same resolved string once
on each side of the selected cut. Its Before and After manifestations use their
respective attachments and fresh joint shaping. The ending selects one
alternative and counts once, while its receiving prefix creates no additional
ending. This native form does not represent conventions requiring different
strings on the two sides or establish a language table.

For an automatic indication with no established language or writing-system
convention, Surgeist selects the bounded [WebKit font fallback](../references/webkit-hyphen-indication--73aa6c89e2cb-metadata.md):
choose U+2010 when the indication-owning style's resolved primary font has a
nonzero glyph for it, otherwise choose U+002D; use logical-before placement.
This is an explicit unknown-convention policy, not all-language support. The
source SHY's own style governs its indication; a source-less typed control needs
its actual captured style authority. Resolve the primary font against the same
captured catalogue and instance selection, independently of adjacent shaped-run
fallback. After choosing the string, use normal font selection and shaping; the
character choice does not promise glyph coverage or permit fabricated metrics.

Primary resolution is a distinct operation following the owning style's ordered
families, resolved generics and actual face attributes. The selected
[first-available-font rule](../references/css-fonts-4-width-matching--editor-capture-20261010--72f4c49ea2ec.md#first-available-font)
uses U+0020 eligibility under admitted `unicode-range` restrictions, not the
presence of a space glyph or U+2010 coverage. If listed faces are unavailable,
only a genuinely configured and captured default-primary authority can supply
the default; script fallback candidates do not establish that authority. With
neither an eligible family face nor such a default, return a precise missing-primary
failure for this auto-indication probe. This bounded failure does not make family
list misses universally invalid under CSS Fonts.

The current native raw-font catalogue has no CSS `@font-face` range restriction
or separate default-primary capture. Its indication probe therefore qualifies
ordered unrestricted native families and resolved generics, without the metric
strut query's Latin/Han fallback. This does not complete CSS font-face range or
default-font integration. Preserve existing strut selection independently.
Specified strings, empty strings and suppressed hyphenation do not require this
auto-indication probe; nonempty specified strings still need normal shaping
resources.

Each authentic defining occurrence has one indication input mode: resolve from
its governing style and convention, or consume explicitly resolved native text
alternatives. The latter retains a nonempty ordered alternative list, including
empty strings, and its native provenance; it does not assert language-convention
coverage. It is not a second style-string override. Both modes use the governing
style for font selection and hyphenation eligibility, and `none` or that owner's
BreakAll suppresses both. Conflicting modes or duplicate definitions are invalid.
A bare source SHY uses style resolution rather than silently becoming an
ordinary break without an indication. Independent ordinary after-SHY permission follows the
[actual-owner rules](#breakall-and-independent-ordinary-cuts); suppression alone
cannot create it. A terminal paragraph end does not select that SHY alternative.

Retain the actual resolved convention, governing style and font/resource context
in preparation and all affected cache identities. Qualification covers none versus
visible-hyphen breaks, manual with unknown language, exact specified strings and
empty endings, explicit-script precedence and supported script fallback, known
unsupported conventions, primary-font versus adjacent-fallback coverage, genuine
source/control custody, min-content and unchanged authored/selection/search text.
This determination supplies requirements, not an implementation-completion claim.

A nonempty LogicalAfter or BothSides ending retains a Text-issued continuation
for its exact preparation, alternative and mixed cut. The receiving selection
supplies that incoming choice separately from its own outgoing ending.
Whole-paragraph placement requires exact adjacent pairing and no discarded final
obligation.
A range may retain an open outgoing obligation or authentic incoming choice
without claiming an omitted predecessor was committed or widening original
source-query coverage. Receiving an indication does not create another
hyphenated ending for line-count policy.

#### BreakAll and independent ordinary cuts

Follow the [BreakAll hyphenation determination](../references/text-break-all-hyphenation--determination-20261024.md):
the actual Source SHY owner, discretionary Control owner, or logical-preceding
text owner of an Automatic cut cannot admit a hyphenation alternative when that
owner has `WordBreak::BreakAll`. Apply this alongside its existing hyphens gate,
without transferring ownership to the following text box. Suppression includes
empty indications: there is no alternative trial, continuation or hyphenated-line
effect, rather than merely hidden ink. Retain truthful cut/resource observations;
a suppressed alternative does not require indication resolution or font matching.
Language/resource qualification and wrapping retain their own authorities.

Independently evaluate ordinary permission at the actual after-source-SHY mixed
cut. In otherwise eligible Latin `a SHY b` with uniform BreakAll, the following
letter/class rule admits an ordinary cut after SHY, with no indication; it does
not create a break before SHY. The following ordinary box's BreakAll, actual
Unicode neighbors, typographic-unit boundary and native nearest-common-ancestor
Wrap permission govern that choice. The SHY owner's local Wrap gate governs its
manual choice separately and cannot stand in for ordinary NCA permission.

For an otherwise legal after-source-SHY relation with hyphens:manual and all
other manual-admission requirements satisfied:

| SHY owner | Following ordinary owner | Actual choices at that cut |
| --- | --- | --- |
| BreakAll | BreakAll | Allowed, nonhyphenating `None` only. |
| BreakAll | Normal | No SHY-derived offer; independent permissions survive. |
| Normal | BreakAll | Allowed with both `None` and the admitted manual alternative. |
| Normal | Normal | Discretionary manual alternative without ordinary promotion. |

Disabling hyphens removes the manual choice without removing independently legal
`None`. Equal source offsets cannot promote another endpoint: retain the exact
source-after-SHY participant, original source custody and authentic Soft endpoint
attachment. A source-free discretionary Control has no implicit BA character and
cannot borrow the source-SHY permission. Consecutive SHYs, punctuation, marks and
glue require their actual rules/context. Genuine Wbr, ZWSP, forced, atomic and
always-visible-hyphen permissions remain independent; none donates proof to a
suppressed SHY or an equal-offset control.

#### Admitted partitions and complete variant evidence

A manual partition requires an actual matching admitted Source/Control
alternative at its cut, not merely a retained cut observation, opportunity
presence or Discretionary kind. `PreparedWordView::portions` skips
unadmitted manual cuts without advancing the portion start or replacing its
incoming choice. Allowed plus a manual alternative remains a genuine partition;
Allowed/None-only does not become one because a Source cut was observed.

An explicit Wbr can independently share an endpoint with Auto. For genuine
`as` + Wbr + `sociate`, Normal or KeepAll, en-US, hyphens:auto, wrapping enabled
and no word-space transform, the complete lexical word remains `associate`.
The actual boundary after Wbr retains both nonhyphenating `None` and the resource's
independent Auto choice; a later genuinely issued offer supplies the
conditional-enumeration ceiling. With 10px letters and a 10px indication, the
full-prefix None/Auto trials retain their distinct 20/30px advances. Authenticate Wbr, source, resource and
mixed-cut identity; a different pre-Wbr offer cannot supply that evidence.
Transformed full-width spelling is no resource proxy: the query does not perform
width normalization.

Complete conditional evidence still covers each actual variant separately.
`AutoFitExhausted` and `AutoIntrinsicRefinement` cannot discharge the endpoint's
`None` or manual siblings; `ConsumerHyphenationPolicy` cannot discharge `None`.
The admitted-manual predicate also governs `AutoManualPriority`; no observation
or unrelated earlier offer weakens ordinary barriers or partition requirements.

### Hyphenation indication bidi

Text resolves the original paragraph's bidi state without generated hyphenation
indications and applies its actual selected-line L1/L2 processing. An indication
cannot change original embedding, strong-type or bracket resolution, or leave
state for another line or indication. This applies to Source, Control and
Automatic definitions and to both style-resolved and explicitly resolved native
indication alternatives.

Each nonempty indication has a local bidi domain attached at its authenticated
logical before/after boundary. The outer attachment retains the originating or
receiving run's full level; the indication's internal base direction uses that
level's parity. Keep the outer and local level spaces distinct rather than
adding their levels. The attachment authority is:

| Placement and definition | Actual level owner |
| --- | --- |
| Before, Source SHY | Its analyzed original-source occurrence |
| Before, typed Control | Its authenticated analyzed control occurrence |
| Before, Automatic | The original contribution immediately preceding the cut |
| After | The original contribution immediately following the generating cut on the receiving continuation line, with its actual selected-line directional authority |

At a mixed-direction After boundary, use the following side. Retain the original
occurrence, cut side and selected-line association, not an unqualified integer
level. Transparent empty and out-of-flow events supply no level; do not skip a
genuine formatting character merely to find visible content. Receiving direction
and geometric attachment do not transfer the indication's established style,
font, language convention or DOM ownership. A nonempty After indication without
authenticated receiving continuation authority returns
`HyphenationError::MissingReceivingBidiAuthority`; an atomic receiver retains
`UnsupportedReceivingRole`. Paragraph direction, adjacent fallback fonts and
equal-offset anchors cannot substitute. Empty indications retain their
continuation/selected-ending obligation identity without visible insertion.

For incoming LogicalAfter, including the After side of BothSides, insert the
whole indication immediately before its authenticated receiver in the selected
logical mixed-event sequence. Preserve every genuine prefix control in captured
order: Start, indication, receiver in that case, at both zero and positive Start
widths. Nested starts/ends and empty metadata retain that order; edge magnitude
changes the barrier, not attachment identity. Authenticate the actual participant
and scalar, including a genuine formatting control or deleted original with
directional authority, rather than searching for the first visible glyph.

Insert the local indication domain at the receiver's full authenticated outer
level; its internal base still uses that level's parity, and each control retains
its own level. Reorder the actual mixed sequence with original
L1 processing independent of the generated string. Endpoint parity alone cannot
determine general mixed-level placement, and post-shaping coordinate moves cannot
repair the sequence. This is the explicit [incoming-edge determination](../references/text-incoming-indication-edge--determination-20261024.md),
not a browser-interoperability claim.

Use this same attachment for original-context seeds, transported presentation
cuts, shaping and placement. An eligible positive Start precedes the whole
indication-plus-receiver side: it bounds genuine original context across itself
without dividing their computed joining. Whole-indication custody creates no
atomic box or mandatory shaping barrier; retain local bidi, shared graphemes and
compatible joint GSUB/GPOS. Outgoing Before custody is unchanged.

Resolve the exact indication string internally with the existing UBA owner and
the supplied local base direction. Strong characters, brackets, embeddings,
isolates and overrides affect only that domain. Unmatched terminators cannot
close original-source state, and unclosed local embeddings cannot escape it.
Place the internally ordered result at its logical attachment side. Do not
flatten all its characters to the attachment level, strip controls, or insert
synthetic isolate characters into authored text.

This bidi scope does not itself create a shaping barrier. Preserve the adopted
original and computed joining contexts, genuine shared graphemes, and joint
substitution and positioning across otherwise compatible original/computed
runs. Different resolved directional runs retain their actual shaping
constraints. If local bidi order, shared-unit correspondence and joint shaping
cannot be composed faithfully, return a specific capability refusal before
publication. Paragraph or segment separators inside an indication require an
explicit local handling rule or a precise refusal, never an invented original
paragraph break. Refused strings remain outstanding support.

U+0009 TAB follows the chosen prepared
[preserved-tab operation](#preserved-css-tab-stops), with local L1 and actual
ordered positioning. Its CSS-stop and native font-advance demands stay distinct;
it does not introduce an original forced break. Other unsupported separators
retain their precise capability refusal.

This is explicit Surgeist policy. The pinned
[WebKit display mechanism](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/display/InlineDisplayContentBuilder.cpp#L210)
supports attaching a Before indication to an already resolved run. It does not
establish After placement or arbitrary-string handling; those extensions are
not uniquely prescribed by the pinned
[hyphenate-character clause](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#hyphenate-character).
The prior full-paragraph variant-analysis effect is not an exception for Before.

Qualification includes a real existing-callable Before case where an indication
strong character, bracket or explicit control otherwise changes original bidi;
opposite-direction After attachment; simultaneous incoming and outgoing
indications; local control containment; genuine shared-unit shaping; unchanged
subsequent-line source bidi; and precise missing-authority/composition refusals.
Exact font-query witness reuse must follow these authenticated directional
inputs. These are requirements, not an implementation-completion claim.

### Automatic resources, word custody and initial limits

The selected first automatic resource is Hypher 0.1.8 with allocation and only
English enabled, using its pinned American-English 2005-05-30 patterns. Apply all
14 source exceptions, including explicit zero-cut entries; absence of an
exception differs from an exception forbidding cuts. Keep the provider and source
data exact. Text separately selects the source's stated `democrat` correction at
character boundaries 3 and 4, for that exact word only. Match exceptions and this
correction without ASCII case distinctions; do not imply Unicode case folding,
derivative-word corrections or complete English coverage.

A lexical word spans connected text origins, styles, fonts and transparent
out-of-flow occurrences. Actual hard separators, in-flow atomics and disconnected
text remain genuine domain boundaries. Resource qualification examines every
actual language contribution to the complete word: this resource admits only
`en-US`, ignoring ASCII case. Other regions, generic `en`, variants and extensions
do not acquire implicit coverage. An unqualified mixed-language word retains
manual opportunities and a truthful resource observation; it is not split into
artificial independently hyphenated words.

The logical-preceding actual text contribution owns an automatic cut's style,
origin, `hyphens`, indication, convention and font-selection authority. Its
`Auto` permits automatic production subject to its other gates, including that
owner's non-BreakAll word-break; its `Manual` or `None` does not. Empty wrappers
and out-of-flow occurrences cannot own the cut. Nearest-common-ancestor
wrapping permission remains separate. Neither a first-run style nor the adjacent
effective fallback font can replace the actual owner. These mixed-style and
resource choices are explicit Surgeist policy where the pinned specification
does not fully determine the result.

Text retains complete-word and cut observations with their exact ordered source
pieces, processed correspondence, mixed boundary, governing contribution, counts
and pattern/source-exception/Text-correction provenance. An automatic indication
is a genuine computed insertion consuming no authored interval, rather than an
injected SHY, fabricated control or caller-authored replacement. Provider UTF-8 is
a temporary mapped view: preserve original SHY occurrences and exact UTF-16
custody, validate scalar/grapheme boundaries, and never fuse isolated surrogate
halves across removed pieces. Long words use checked lengths and allocation
without an arbitrary word quota.

For the temporary resource query, omit authentic U+00AD SHY and retain every
other formatting control and combining mark unchanged. This is a bounded native
query policy, separate from counting, shaping and Range segmentation; it adds no
normalization, Unicode folding or isolated-surrogate replacement. Preserve exact
correspondence through omitted SHY gaps and map resource cuts only to validated,
unambiguous original mixed boundaries. A coincident authored SHY keeps its own
identity and preference; do not relabel it automatic. Retained controls may limit
resource recognition and must remain visible in the resource observation.

For the native CSS operation, resolve initial character limits to total 5,
before 2 and after 2, for manual and automatic hyphenation, including an empty
indication. Text owns these counts and admission. Exclude nonspacing marks and
intra-word punctuation as specified by
[hyphenate-limit-chars](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#hyphenate-char-limits);
SHY and the inserted indication are not authored lexical letters. The numeric
resolution is a Surgeist choice. Applying these limits to SHY follows the pinned
inclusive prose; the CSSWG issue 5090 dispute remains unresolved, so this is not
a claim of standards consensus or universal browser agreement. The initial zone
is zero, successive-line limit is unbounded and last-line exclusion is none.
Nondefault limits and broader resource coverage remain separate requirements.

For native character limits, Text consumes a computed total/before/after triple
on the governing `Style`, retaining `auto` or a nonnegative count in each slot.
The consuming style adapter owns expansion of the specified CSS values and
checked integer conversion: one value becomes `(total, auto, auto)`, two become
`(total, before, before)`, and three remain distinct. Text does not parse that
grammar or wrap or saturate unrepresentable external integers. Native preparation
resolves each `auto` independently to the selected 5/2/2 fallback. Preserve the
symbolic triple in captured style and cache identity even when explicit numbers
have the same used values.

Apply the actual cut owner's limits against the single complete-word count map;
a preparation-wide override cannot represent different governing styles within
one word. Compare all three minima independently, without adding them, clamping
them to word length or allocating by their values. Zero removes only that numeric
restriction; it supplies no lexical word, count observation or eligible cut.
For example, `ab` + SHY + `cd` fails the initial total minimum of 5, while an
explicit 4/2/2 triple permits count admission, subject to the remaining checks.
Keep admission before indication-resource and placement resolution. The raw
unconstrained operation remains an actual operation distinction, rather than a
synthetic zero triple or an interpretation of `auto` as disabling limits.

Compute total, before and after from one complete-word extended-grapheme map,
using the selected Unicode rules across transparent origin/style boundaries and
retaining actual source correspondence. Do not delete formatting controls and
resegment, or independently resegment the two sides of a cut. Each counting unit
contributes zero or one, independently of scalar/UTF-16 length, glyph count or
advance. An optional ligature does not merge independent counting units, and a
count map cannot authorize a cut inside an otherwise indivisible unit.

Use [CSS Text §1.4](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#characters)
and [Appendix E](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#character-properties)
for the unit basis and effective category, including the Common-script enclosing
mark exceptions. Apply the specified Mn and intra-word P* exclusions. A base
with nonspacing or spacing combining marks normally counts one unit; a standalone
Mn unit counts zero, while an otherwise eligible standalone Mc unit can count
one. A Common-script enclosing mark can change the effective category; do not
reject its cluster solely because the underlying base is punctuation. Numbers,
symbols and other eligible units are not excluded merely for being nonletters;
lexical-word membership remains a separate determination.

Additionally, Surgeist selects a zero independent contribution for formatting
controls (General_Category=Cf), including ALM, bidi controls, ZWJ and ZWNJ. A
control within an otherwise countable unit neither adds a count nor disqualifies
that unit; a control-only unit counts zero. This is operation-specific native
tailoring, not an exclusion expressly supplied by the character-limit clause.
Original SHY and generated indication characters also count zero. Retain their
source, bidi, shaping and opportunity provenance. Qualification includes ALM on
both sides and repeated ALM, base/standalone marks, intra-word punctuation,
enclosing-mark cases, supplementary characters, SHY and empty indications.

Preserve the existing raw `System::layout`/`layout_analyzed` count policy at its
actual operation boundary. Native preparation remains the CSS operation even
without an explicit paragraph direction. A direct Text consumer uses the same
selected-line protocol; no second CSS formatter or compatibility lane is needed.
Candidate provenance remains distinct from admitted selectable opportunities.

For finite fitting, prefer actual eligible explicit conditional hyphens
before automatic fallback. An automatic cut is available only within a portion
that still cannot fit an otherwise available line after that manual partition;
failing the whole word or the space left after unrelated content is insufficient.
Include the actual indication and finalized occupied geometry in the fit test.
For `as` + SHY + `sociate` with 10px letters and a 10px `LogicalBefore`
indication, width 80 takes the explicit cut after `as`; at width 60 its 70px
residual portion permits
automatic fallback. Raw standalone layout also preserves this priority. Layout
owns actual fit, float replay and committed-line state, while Text supplies the
same word/cut authority. Rejected trials remain pure. Producer and consumers must
deliver these semantics together before automatic offers become selectable.

For MinContent, retain admitted manual boundaries and let eligible automatic cuts
subdivide their residual portions through the existing intrinsic search. Resource,
count, cut-owner, wrapping and actual finalized indication geometry still apply.
This is Surgeist's explicit intrinsic interpretation of the hyphenation rule and
[intrinsic sizing](../references/css-sizing-3--WD-css-sizing-3-20260904--ca0f1db3d5ad.md#intrinsic-sizes),
not a finite overflow receipt or an unrestricted replacement of the manual
partition. Purpose and partition/candidate identity govern admission and reuse;
absent available width proves neither intrinsic purpose nor failure to fit.
For example, `as` + SHY + `sociate` with 10px letters and a 10px
`LogicalBefore` indication has MinContent 50px through pieces 30/30/50, refining
its manual 30/70 partition.
MaxContent measures the unbroken word at 90px without selecting hyphenation
alternatives. Independent `anywhere` versus `break-word` intrinsic rules remain
unchanged. Finite fit exclusions cannot be replayed as intrinsic determinations.

These are integration requirements, not a claim that automatic resource delivery
or the complete hyphenation contribution is implemented.

### Hyphenation zone and the ordinary fallback line

Layout consumes the block container's computed `hyphenate-limit-zone` as a signed
finite length-percentage, initially zero. The consuming style adapter owns
inheritance and conversion. The pinned [zone clause](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#hyphenate-size-limits)
defines pre-justification unfilled space and percentages relative to the line-box
length. Its un-ranged grammar permits signed values under the
[numeric range rules](../references/css-values-4--WD-css-values-4-20240312--7b6b68c34d00.md#numeric-ranges);
do not reject or clamp a negative computed zone.

Surgeist applies the zone gate only to automatic hyphenation, including an empty
automatic indication. Otherwise admitted manual SHY and explicit empty endings
remain eligible. Passing this gate permits consideration without overriding
manual priority, character or sequence limits, wrapping permission or actual fit.
This applicability choice and the reference-frame and intrinsic rules below are
explicit native policies where the source does not fully determine behavior.

Use the genuine legal ordinary, unhyphenated fallback that would leave the word
for the next line. Finalize it independently from the unchanged incoming line
start with no spacing allocations, actual line-edge processing and contextual
shaping, in its own converged float and height frame. Retain its prealignment
typographic alignment measure, including selected hanging exclusions. A prefix
fold on a longer hyphenated line does not finalize that shorter line ending;
glyph ink bounds and an occupied bounding rectangle are also different measures.

In coordinates increasing toward logical line end, define unfilled space as
`U = max(0, fallback_line_end - (actual_content_start + alignment_measure))`.
The authentic content start includes indent and displacement. Preserve signed
advances; the zero floor defines unfilled space rather than clamping authored
spacing. Resolve the percentage component against the full fallback line-box
inline span after float constraints, before subtracting indent or consumed
content. Preserve typed numeric failures when consuming that basis.

Permit automatic consideration exactly when `U > Z`, where `Z` is the used zone.
Equality is within the permitted unfilled space: `U = 0, Z = 0` fails this gate,
while the same gap with a negative zone passes. The partial-word candidate must
then satisfy its own fit and convergence; it cannot impose its frame on the
fallback. For example, collapsed `X SPACE associate` in a 60px line with 10px
letters has a natural fallback advance of 10px and unfilled space of 50px. A
45px zone permits consideration; counting its discarded trailing space would
incorrectly reduce that gap to 40px.

At line start or with genuinely no legal ordinary fallback leaving the word for
the next line, the zone gate is inapplicable. Do not manufacture an empty line,
ordinary offer or measurement role. Zero advance alone does not prove absence
of a formatted prefix. Failure to represent or finalize an existing legal
fallback is a capability failure, not this exemption. MinContent does not apply
the zone gate and retains its existing manual-partition/automatic-refinement
policy and independent sequence-limit-zero rule. MaxContent remains unbroken.
Zone probes and suppression dispositions preserve actual source, opportunity,
reference-frame and purpose identity without committing history or claiming
phantom participation or fit exhaustion. These requirements do not establish
implementation completion.

### Successive hyphenated lines and intrinsic measurement

Layout consumes the block container's computed `hyphenate-limit-lines` value:
`no-limit` or a nonnegative integer. The consuming style adapter owns inheritance
and checked external integer conversion. This is separate from each text cut's
character minima. The pinned [line-limit clause](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#hyphenate-line-limits)
sets the successive-line limit but leaves emergency interaction incompletely
defined. Surgeist adopts the following native committed-line policy, supported
by the bounded [WebKit source witness](../references/webkit-text-source-gaps-hyphenation--73aa6c89e2cb.md).

At the start of an actual line, suppress automatic hyphenation when the block's
committed successive-hyphenated-line count reaches its finite limit. `no-limit`
does not suppress it. Zero suppresses automatic opportunities immediately while
retaining eligible manual conditional endings. Update state only when a line is
committed: a selected genuine hyphenation ending increments the count, including
manual endings and empty indications; any other ending resets it. A literal
visible hyphen's ordinary break and an arbitrary unhyphenated emergency break
are resets. Rejected candidates, float retries and measurements do not commit.
Keep the block's actual line history across captured text-run boundaries;
another block's independent line history cannot replace it.

An already permitted exceptional hyphenation ending retains its actual trigger
and increments the committed count, even at the limit. This grants no new
exception: an automatic opportunity cannot bypass zero or a reached cap merely
because it prevents overflow or is called an emergency. Preserve the original
opportunity inventory, authored text and previously committed lines.

MinContent is a measurement of pieces, not a committed line sequence. With
`no-limit` or a positive limit, retain the admitted manual partition and eligible
automatic refinement without reading or mutating the actual counter or creating
a hypothetical counter. At zero, retain the manual partition but exclude automatic
refinement. MaxContent remains unbroken. Carry explicit measurement purpose;
missing available width alone does not identify an intrinsic query. Positive
finite-fitting suppression cannot be replayed as an intrinsic exclusion.

For the 10px-letter, 10px-indication `as` + SHY + `sociate` example with
`LogicalBefore`, MinContent is 50px at limit 1 or `no-limit`, and 70px at zero
when no independent breaking
opportunity applies. MaxContent remains 90px. Query order cannot change these
measurements or the next actual line's eligibility. These intrinsic and emergency
rules are explicit Surgeist policy, not a claim of uniquely specified CSS behavior
or implementation completion.

### Unfragmented element-tail hyphenation restriction

Layout owns the block container's computed `hyphenate-limit-last` policy, with
`none` as the initial value. The consuming style adapter owns inheritance and
conversion. The pinned [last-line clause](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#hyphenate-line-limits)
includes element, column, page and spread boundaries. The requirements below
select the unfragmented element-terminal part of `always`; they do not establish
fragmentation support or complete the five-value feature. They are explicit
Surgeist policy where the specification leaves edge cases undetermined.

For finite line selection with `always`, use the authenticated owning block's
completed in-flow line sequence. Find its last line containing actual in-flow
content, ignoring trailing empty lines only for this determination and retaining
their source and layout geometry. Content is not defined by positive advance or
visible ink. The protected line is that final content line's immediate
soft-wrapped predecessor, in the same uninterrupted forced-break segment, which
actually continues into it. Do not scan backward across a forced break. A
one-line final segment has no protected predecessor; earlier segments are not
retrospectively restricted. If the final content line ends with a genuine forced
break and only empty terminal lines follow, its soft predecessor remains
protected.

The protected predecessor cannot end in an admitted Source, Control or Automatic
hyphenation alternative, including an empty indication. Ordinary literal visible
hyphen breaks remain distinct. This inclusive reading retains the unresolved
[CSSWG issue 5090](https://github.com/w3c/csswg-drafts/issues/5090) dispute rather
than claiming settled interoperability. An oversized final word grants no new
hyphenation exception: use independently permitted nonhyphenating emergency
breaking or ordinary overflow behavior. A genuine forced break between two
content lines prevents the protected relationship.

Reselect through the existing line-selection and finalization machinery, then
validate the resulting complete tail before publication. Reflow can change the
protected line. Rejected sequences cannot leave permanent bans or committed
source, indentation, float or successive-line state; removing an already-shaped
indication does not implement the restriction. For example, the pinned
`to show Antarc-` / `tica.` example becomes `to show` / `Antarctica.`, subject to
the actual line geometry, rather than merely banning an alternative on the
paragraph's final line.

`BlockEnd` alone cannot authenticate the completed in-flow tail or its actual
soft/forced continuation relationships. `Unknown` cannot imply exemption, and
`NonBlockBoundary` cannot prove a later formatted line. Missing necessary tail
ownership is a capability limitation. Column, page and spread restrictions need
genuine fragmentation-break production and line association; forced LF, grid
columns, clipping and paint fragments do not supply that authority.

MinContent retains its existing manual partition and automatic refinement rules
without constructing a hypothetical protected tail. MaxContent remains unbroken.
Neither intrinsic purpose consumes or produces finite last-line restriction
evidence. This intrinsic exemption is native policy. These requirements do not
assert implementation completion.

### Float encounters inside shared text units

A float remains a Layout-owned box at its original source-order occurrence. It
does not divide a prepared paragraph or create a Text atomic, control, caret or
break opportunity. Layout replays float encounters while selecting and fitting
the paragraph's actual lines, retaining the same preparation and source mapping.

The pinned CSS Text [character-unit discussion](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#characters),
[text-processing adjacency](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#text-encoding)
and [boundary-shaping rules](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#boundary-shaping)
do not specify a numeric prefix advance for a float seam strictly inside one
shared typographic or shaping unit. Surgeist uses the following deterministic
accounting policy for that gap; this is not a browser-equivalence claim.

Text identifies the smallest enclosing finalized grouping closed under both
grapheme indivisibility and actual shared provider-cluster associations. Neither
association alone nor extraction components define this grouping. At an interior
seam, the encounter's logical text prefix includes
that whole unit's real advance exactly once. At its start the unit is excluded;
at its end it is included. Logical end is independent of text direction and
float side. Two floats inside the same unit have the same text-prefix
contribution, while their distinct box placements and exclusions accumulate in
source order. Signed advances need not grow with source order; encounter progress
does not follow numeric prefix size. Prefix advance is distinct from occupied
extent, ink and physical bidi position. For a shared 12.5px ligature, the interior
seam counts 12.5px, not a 6.25px extraction component. This does not assert independently shaped
prefix geometry or change glyphs, extraction rectangles, source ranges or carets.

The observation belongs to the exact finalized line, preparation, queried
original occurrence and selected context. Text returns the original seam and
effective whole-unit accounting contribution distinctly, preserving signed
advance and actual spacing, trailing-collapse and hanging facts. Layout does not
reconstruct this from glyph ink, average character widths or extraction clusters.
Actual preceding flow participation remains separate from width charging: retained
zero/signed text or allocated participants can precede the seam, while a later
part of an included shared unit does not. Deleted, discarded and empty occurrences
add no participation. This does not decide CSS line-box existence or visibility.

Layout uses that observation with actual margin-box, clear, fit and source-order
constraints. Changed bands, allocations or legal line-break reshaping require
the corresponding candidate to be finalized and checked again. If the unit moves
to another line, replay its float encounter against that candidate; do not keep
an obsolete placement above it or fabricate a partial glyph/accounting split.
A genuine Text-issued legal break may reshape or dissolve an optional ligature;
derive the grouping anew from that finalized candidate. Rewinds restore provisional
float state without double insertion. Complete feedback/cycle identity includes
event progress and provisional placements. Only a coherent accepted result is
published; refusal preserves the previous batch.

Qualification includes real shared ligatures and combining graphemes, start,
interior and end seams, two floats in one unit, both directions and float sides,
and a fit threshold distinguishing a half-unit from the whole unit. Include
negative and zero advances, line-edge discards, replacements and equal-offset
occurrence ties. Preserve
glyph/source/caret mapping and exercise actual deferral, legal-break reshaping,
replay, repeated-state refusal and rollback. The policy does not itself qualify
the general float algorithm or resolve live Style/DOM producer authentication.

### Conditional wrapping around captured floats

For native horizontal finite fitting, establish the highest legal encountered-float
assignment first, including mandatory clearance or downward movement needed for
the float itself to fit. Exhaust ordinary endpoints and alternatives in that
assignment, with each candidate's actual shaping, allocations, encounter replay,
canonical placement and height-qualified bands converged. A later ordinary fit
wins over a conditional cut.

If no ordinary candidate fits, try eligible Text-issued conditional cuts before
optionally deferring an encountered float below the same ordinary line. Replay
each changed cut from the accepted checkpoint using its own finalized prefix
and legal placement; reshaping may change which float positions are legal. A
fitting conditional cut wins over the optional deferral. If
neither kind fits, try the permitted deferral and restart ordinary-first search
for the changed assignment. Moving the line through an existing float's bands
is a separate subsequent fallback. Disabled or unsuccessful conditional wrapping
does not remove the ordinary deferral behavior.

With Ahem advances of 10px, `A | float(8px × 4px) | X` in a 20px block with
`overflow-wrap:anywhere` selects `A` beside the higher float in the 12px band,
then `X`. The Soft endpoint includes the tied non-interior float encounter;
replay must retain that float rather than obtain a fit by dropping it. A Forced
endpoint excludes the equality event. A float seam itself creates no break.

This is Surgeist's explicit ordering determination, inferred from the pinned
[float reflow and higher-placement rules](../references/css2--visuren.html--3f334c530cf4.md#floats)
and [overflow-wrap conditions](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#overflow-wrap-property).
The corresponding [CSS2.2 rules](https://www.w3.org/TR/CSS22/visuren.html#floats)
and [current Text4 clause](https://drafts.csswg.org/css-text-4/#overflow-wrap-property)
do not explicitly order these competing assignments. This is not a claim of a
universal CSSWG resolution or exact browser-test agreement. Native conditional
fitting uses this ordering for its admitted horizontal capture. Broader
hyphenation, live producer joins and the full #66/#72 contribution retain their
separate requirements.

The finite-fit precondition does not gate `anywhere` min-content opportunities
on failure of an unconstrained ordinary trial. Min-content selects its eligible
pieces independently; `break-word` adds no such intrinsic cuts, and max-content
adds neither kind. Qualification includes the 8px example, a later ordinary fit,
absent conditional permission, mandatory deferral of a float that cannot itself
fit beside its prefix, no fitting conditional candidate, Soft/Forced seam ties,
and rollback into a changed assignment.

### Canonical boxes and coherent line placement

For composed horizontal lines, Layout settles its canonical boxes, available spans, float bands and atomic allocations before accepting Text's finalized geometry. This includes the selected Layout pixel-rounding policy wherever it affects those facts. Pixel snapping is distinct from scalar conversion and canonical provider precision. If final ancestor placement changes the rounding phase and thereby changes a dimension, relative baseline or float intersection, Layout reevaluates the affected candidate and its feedback before placement. A final rounding pass cannot silently change an accepted line's inputs. This replaces independent rounding of text-fragment origins, ends and baselines for the composed protocol.

Layout selects one physical anchor per finalized line: its inline origin and baseline, rounded once under the requested Layout policy using the actual containing placement and global reference. Every run, source fragment, cluster, caret and selection receives the same rigid translation of immutable line-local geometry. Line top is the placed baseline minus actual ascent; fractional local positions, advances and sizes remain fractional. A line spanning multiple source nodes has one common formatting-context anchor. Node-local views are coordinate conversions from that placed geometry using actual committed ancestor origins, with no independent snapping. Layout's line allocation rectangles and Text's metric or ink bounds retain their distinct roles.

Logical line allocations follow the pinned [line-box stacking rules](../references/css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md#line-boxes). Retain their unsnapped progression independently of each physical Text anchor: advance by the accepted allocation height and any genuine float or clearance displacement. A rigid placement translation does not become additional flow advance or change the next logical start. Natural container height follows those allocations; actual placed metric and ink bounds contribute to their respective overflow calculations. Both allocation and placed metric footprints must satisfy their applicable float constraints; a gap between those footprints is not newly occupied space. Revalidate changed canonical allocations and float intersections under the feedback rules below.

An in-flow atomic participant carries both its actual canonical box rectangle and baseline, and its logical flow allocation including signed used margins, spacing and baseline-relative ascent/descent. Layout derives that allocation from canonical box facts under its box rules; it is neither the painted border width nor a nonnegative clamp of the margin box. Explicit box-origin/baseline offsets relate the Text participant to the actual committed node rectangle. Any independent atomic snap displacement relative to the line is part of those facts before acceptance. Text consumes the supplied allocation and participant identity without interpreting CSS margins. The pinned [CSS Inline layout-bounds and baseline rules](../references/css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md) motivate this distinction; integer snapping and the single-anchor granularity are Surgeist policy, not CSS requirements.

An out-of-flow occurrence retains its issued identity, original anchor and affinity through selection and finalization without contributing flow advance, line metrics or a bidi placeholder. Its actual rectangle comes from Layout's existing absolute-child operation and is required when placing that occurrence. Selection does not require a fabricated provisional rectangle. A captured absolute node with no Text atomic occurrence remains an explicit Layout-owned node; the adapter does not invent a Text box for it. Both cases preserve real text adjacency and actual static-position anchors.

A change to atomic dimensions, relative baseline, margin-edge advance, occupied extent, spacing, tab origin or float band reenters the applicable selection, finalization and fit checks. A common line translation needs only placement when it leaves those constraints unchanged; it does not require shaping again. Changed constraints or an independent atomic translation require the corresponding revalidation. The complete-state cycle/no-fit rules above apply. Once stable, Layout places those exact finalized lines, derives node/source associations and atomic rectangles from the same records, and builds the interaction, accessibility and Render-facing views before atomically publishing `CompletedLayoutBatch`. No subsequent rounding modifies Text-derived geometry. Live Style authentication under #1159 is separate from this explicit-input native protocol.

Qualification must cover a ligature/mark line crossing source nodes with fractional widths and invariant relative geometry under placement and source repartition; fractional nested ancestor origins; mixed LTR/RTL atomics with fractional and negative margins and baseline offsets; and a canonical atomic change that genuinely changes height/float feedback or yields explicit cycle/no-fit. Verify common translation without reshaping, scalar conversion and association rejection, and late refusal preserving the previous batch. Vertical/oriented profiles require their own qualified placement mapping before admission.

### Source positions and placed caret occurrences

A source `Cursor` preserves an original code-unit endpoint and affinity. It cannot uniquely identify every placed caret occurrence: source-less forced controls and empty origins can produce several distinct lines at the same source offset. Preserve those actual line and anchor occurrences rather than merge them by numeric source range. `PlacedPosition` retains the exact immutable placement and a private validated occurrence; line/anchor readback and physical hits issue it. Clones retain that identity, while a separately placed equal result does not admit the handle.

The source projection is one-way: an exact position beside an atomic box can project to the same `Cursor` as a different edge selected by the raw source-affinity rules. Matching projected offsets and affinities cannot recover that exact occurrence or promise its geometry on a source-query round trip. Retain the issued position when exact row and edge identity matter.

Raw word movement classifies the actual original-source endpoint, including one inside a deleted whitespace run. Processing does not turn that endpoint into the following word's start; traversal still uses the retained source word facts and actual row correspondence.

Exact position geometry and movement use the same finalized interaction rows as source queries, without a parallel caret engine. A raw-source geometric or movement request must validate coverage and identify a unique eligible occurrence under the existing source-affinity rules; an uncovered or genuinely ambiguous request returns a typed error. It cannot invent a middle line, silently omit an occurrence, or substitute whole-paragraph coverage. Preserve existing unambiguous standalone behavior and source-only accessibility. Source extraction and editing retain their source-domain semantics; a source-less placed occurrence does not acquire authored characters or fabricated DOM offsets.

Verify more than two same-offset caret lines, physical hits and movement through their exact issued positions, source-query ambiguity and coverage refusal, foreign-placement rejection, and reuse of the same geometry for selection and geometric accessibility. These requirements establish the composed position boundary and do not claim support before its implementation is qualified.

### Nonempty Text range fragments

Text owns an occurrence-qualified nonempty original-source range query over the
actual placed result. Its ordered observations retain selected source
intersections, expanded typographic-unit coverage where applicable, finalized
line/fragment and placement identities, advance edges and associated font-metric
extent. Native `Layout::source_range_fragments` implements this observation
boundary for admitted horizontal placements; the complete web consumer remains
separate. Generic source slots and caret affinities are distinct.
The selected evidence is the [current CSSOM View §9 capture](../references/cssom-view-range--editor-capture-20261022.md),
checked with the [bounded CSSWG and browser-test audit](../references/range-geometry-csswg-tests--20261022.md).
That audit found no adopted resolution for the remaining endpoint choices in
the discussions inspected; participant proposals and collapsed-only tests do
not establish nonempty deleted-source behavior.

Range units use extended grapheme clusters as their basis, with operation-specific
tailoring under [CSS Text's character-unit definition](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#typographic-character-unit).
They are distinct from float-prefix units: an optional Latin `AA` ligature can
share one provider cluster while containing two Range units; a base and combining
mark remain one grapheme-based unit. Partial selection expands the actual Range
unit, not every provider cluster touching it.

Preserve authentic per-unit advance positions and allocated adjustments first.
For an admitted monotonic horizontal shaped cluster containing multiple complete
Range units without a finer retained advance partition, allocate its otherwise
unpartitioned shaped advance equally among those units in logical order, mapped
through the actual run direction. A 12.5px `AA` shaped advance gives each unit
6.25px before separately owned adjustments. Compose letter/word spacing,
justification and boundary adjustments at their actual owners exactly once;
do not equally redistribute the whole finalized allocation. This is an explicit
Surgeist metric-allocation policy supported by the
[bounded Blink mechanism](../references/blink-formatting-unit-range--daf70f9ddee4.md#range-unit-advance-allocation),
not a CSS-mandated fraction, independently shaped letter width or glyph-ink clip.
Keep shaping unchanged. Ambiguous or nonmonotonic correspondence, unsupported
unit/font boundaries and unrecoverable adjustment ownership return a precise
capability error. Routine optional Latin ligatures need no font caret metadata
for this qualified fallback. Refusal does not complete unsupported-input work.

For a nonempty range selecting an independently retained zero-advance formatting
unit, Surgeist selects a zero-width rectangle at that unit's actual finalized
post-L1/L2 position. Its block extent uses the unit's actual retained resolved
font ascent/descent around the same placed baseline. Missing required metric
association returns a precise capability error; neither a neighboring glyph,
requested family, line-height strut nor outline ink supplies a replacement.
This forbids query-time font guessing, not valid font selection or fallback
during preparation. Using the actual prepared unit's resolved font is Surgeist's
association policy; CSS does not identify a unique font for every control.
This is an explicit policy for a gap in the pinned
[CSSOM View Range rules](../references/cssom-view-range--editor-capture-20261022.md).
The [bounded Blink witness](../references/blink-formatting-unit-range--daf70f9ddee4.md)
does not establish a universal alternative endpoint convention.

Collapsed ranges retain their separate endpoint semantics. Selecting part of an
actual indivisible typographic unit includes that whole unit. Do not relocate a
selected formatting unit to a neighboring visible run, matching isolate end,
pre-L1 position or chosen caret.
Keep genuine zero-width rectangles; bounding-rectangle aggregation is separate.

For valid, fully covered nonempty selections with no eligible retained presentation
attributable to the selected source, executed trim/discard/segment-break deletions
and Phase-II removed edge spaces yield no geometry observations. Apply the same explicit no-observation
policy to suppressed collapsed duplicates, while preserving their distinct
source anchors and opportunity facts. A surviving space representative supplies
geometry only when its own original contribution is selected. Carrying deleted
or duplicate custody alongside it does not select its advance. Preserved spaces
with genuine zero allocations and retained formatting units remain distinct.
Mixed queries report eligible retained units in content order and account for
the complete requested source coverage; absent or ambiguous mapping is an error,
not a successful empty result. A wholly removed origin needs no synthesized font
or endpoint rectangle. These are Surgeist query dispositions for gaps in the
selected Range rules, using the executed
[CSS Text whitespace distinctions](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#white-space-processing).
These dispositions can differ from Blink's zero-boundary candidates for some
deleted subranges; whole-document fixture agreement does not prove their parity.

A selected replacement with a validated nonempty `Source` definition is eligible
when the requested original occurrence/range positively intersects that defining
interval. Endpoint contact, an insertion anchor or a `Control` origin alone is
insufficient. Admit that replacement's actual presentation Range units; an empty
replacement adds none. Preserve the selected original intersections, defining
source relation, alternative ownership and expanded units separately. Deleted
source custody adds no second rectangle. This is explicit Source-replacement
attribution policy, not a consequence mandated by source-preserving hyphenation.
An eligible original and a computed contribution sharing one actual Range unit
select that unit with truthful contributor provenance. Sharing only an optional
multi-unit ligature does not select the computed neighbor's share. Automatic
hyphens without a replaced authored interval require a genuine typed relation
to their generating selected source boundary/span before a consumer can claim
their geometry; control anchors do not supply that missing capability.

For an automatic insertion, preparation retains its immediately preceding genuine
source-backed grapheme in the same lexical word, with all exact ordered source
pieces, separately from the generating mixed boundary. A nonempty request selects
the indication when it positively intersects any of those pieces under the actual
original occurrence identity. Endpoint contact alone does not qualify. This is
Surgeist's automatic-insertion attribution policy; the
[Range rule](../references/cssom-view-range--editor-capture-20261022.md) supports
partial-unit expansion but does not specify this membership predicate. The relation
consumes no source and is not a replacement interval or a word/style/provider-cluster
envelope. A grapheme crossing origins retains each genuine piece; selected
intersections and attribution remain separate observations. Empty origins and
source-less controls supply no relation. Cross transparent events only under
authentic continuity; never search across a disconnection or skip an authored
zero-width formatting grapheme because its advance or count is zero. Missing or
ambiguous attribution is a precise capability failure. After eligibility, use
the existing presentation Range-unit closure once, preserving shared indivisible
units and separately attributable optional-ligature shares. An empty indication
adds no units. Existing Source and Control replacement rules remain unchanged.

A selected preserved authored newline contributes one zero-inline-advance
font-metric rectangle at its actual finalized position on the line it terminates.
Use that item's retained resolved font and placed baseline, including newline-only
and consecutive-newline lines; missing required metrics or placement refuses.
Keep this zero rectangle in a larger query. Where the admitted processing mode
maps CRLF to one forced-break unit, selecting either original half or both yields
that unit once, with selected intersections separate from expanded unit coverage.
Do not introduce CRLF normalization into other modes or DOM offsets: HTML input
normalization precedes the DOM, and a direct DOM CR is not automatically a CSS
segment break. A terminal newline adds
only its terminating-line rectangle; the following empty line has no additional
selected authored unit. A source-less `br` has no Text-source rectangle. This
explicit newline policy uses neither a chosen caret nor the editing-selection
marker, and does not assert exact browser equivalence. The inspected browser
newline test supports relative vertical ordering; it does not prove this policy's
width, font height, terminal-line exclusion, CRLF mapping or rectangle count.

Layout supplies committed placement and node/occurrence associations. Its
[#860 CSSOM View consumer](https://github.com/bj-data-eng/surgeist/issues/860),
under [#850](https://github.com/bj-data-eng/surgeist/issues/850), validates actual
DOM Range membership/endpoints, maps selected Text portions to those occurrences,
includes required selected-element border boxes in content order, and applies
ancestor/viewport transforms and scroll/zoom once. The complete consumer also
handles collapsed ranges, detached ranges' empty result, and unrendered Text
without a layout occurrence. Those valid DOM cases are distinct from missing
mapping within an admitted placed occurrence. Attached shadow-tree Text cannot
be rejected merely because its ordinary root is not the Document; the inspected
shadow test's cross-root setter case does not define a composed range. Suppressing
descendants' element boxes under the selected-element rule does not suppress
their independently selected Text contributions. Excluding source-less `br`/controls
from Text-source eligibility does not exclude selected-element border geometry.
Native observations may be finer than web rectangles; the specification does
not require one rectangle per grapheme, so consumer grouping and bounding-rectangle
aggregation remain separate. A fixture adapter can request the same Text
observations from explicit captured inputs without claiming the full live join.

The retained RTL PDI case therefore keeps native x=24, while the authenticated
Chrome Range reports x=72. These remain explicit strict oracle differences,
not passing or unsupported cases, and neither coordinate is claimed uniquely
required by CSS. Qualify font versus line metrics, independent zero units versus
partial shared units and collapsed ranges, exact source/placement admission,
LTR/RTL, zero-rectangle retention and failure atomicity. No bidi-ordering change
or browser-number substitution follows from this policy.

## Numbers, errors and publication

Shared logical geometry uses finite `f64` values. Provider font sizes and shaping glyph geometry remain canonical `f32` where the providers require that precision. Text reports actual canonical glyph positions and advances; widening those values to `f64` is exact. Layout’s adapter converts to its selected scalar at named boundaries: an `f32` layout rounds once, then returns its actual rounded allocation/context to finalization; an `f64` layout retains its values until the provider boundary. Finalizer converts provider-required spacing/positions once using IEEE round-to-nearest, ties-to-even and returns the actual result for fit/placement. No layer uses the unrounded requested value as if it were emitted geometry. Do not infer success from an unchecked `as` cast.

Logical letter/word spacing, justification and cumulative pens remain `f64` when no provider consumes an adjusted value. Preserve the raw provider glyph advance separately from these logical adjustments. Render's existing glyph input retains finite `f64` run-local positions until its actual path/backend conversion: compose the finalized pen with the canonical offset, and carry the run origin in the outer transform. Apply run translation before caller placement. Decorations already expressed in paragraph coordinates receive caller placement once. Preserve one combined NonZero fill per consecutive foreground outline span. Selected intrinsic color or bitmap representations separate those spans, including when they produce Empty output; their paint and clip operations retain their order. Do not merge foreground geometry across an intrinsic barrier or split a span in a way that changes winding or opacity.

Validate every conversion for finite output and value category: a positive size/extent cannot become zero, negative, infinity or NaN; a nonzero signed offset cannot silently become zero. Negative finite glyph advances and coordinates remain valid where the current contracts allow them. Preserve large physical origins in render's `f64` transform instead of adding them to local `f32` glyph coordinates. No infinity sentinel represents an unconstrained width; use the existing optional constraint. Bounds are computed from the admitted canonical geometry and transformed in their supported precision.

Use typed errors for invalid source/style, invalid selection/break/replacement, wrong preparation or fragment identity, mismatched captured source/style/font catalogue, numeric conversion with phase/value/category, font/provider admission, unsupported capability and feedback cycle. Check narrowing needed by a provider's actual index API and report its representability limit; impose no unrelated text length cap. Malformed external fonts/ranges cannot reach an `unwrap`, assertion or overflow panic. Error payloads must not expose Parley types or backend-named reasons. Extend the existing error model rather than adding compatibility wrappers.

Each operation builds and validates a complete result before publishing it. An error publishes no partial placed layout, source mapping, scene batch or accessibility update. Successful memoization of immutable intermediate values is allowed, but cannot change observable font/source state after a failed request. Render and window build their paint and geometric host accessibility outputs off-side from one captured placed result; source-only accessibility uses its exact analyzed allocation. Text supplies interaction projections. Runtime publishes the coherent view only after all required joins/admissions succeed. A render failure cannot leave glyphs from one preparation paired with interaction from another.

## Renderer admission and ink

#1082 includes the necessary render-owned extension: a concrete headless glyph-preparation operation accepts the face, effective instance and canonical local glyph geometry; validates font/glyph resources; selects the same outline/COLR/bitmap representation used by drawing; and returns an admitted immutable glyph run with local `Ink` or proven `Empty` bounds. Drawing consumes this result, including its normalized coordinates and synthesis, rather than reselecting a representation or using empty coordinates. No new adapter trait is required.

Render owns resource/instance identity within that admitted result and keeps bytes alive through scene execution. Render’s consuming adapter maps the captured text face/instance to admitted render resources without collision-prone content hashes or caller-supplied IDs masquerading as admission. A changed face/axis/size/synthesis value cannot reuse an incompatible prepared glyph run. Safe invalid-font, out-of-range-glyph and unsupported-representation errors occur before scene publication. Bounds must cover the admitted representation, including bearings, overhang, marks, variation and synthesis; a conservative representation-derived bound is valid, an advance rectangle is not. If the renderer cannot establish a required graph bound for a representation, it rejects that graph admission with a typed capability error. An unsupported device operation is not silently rendered as a different font instance.

Text owns font/glyph outline metrics needed for layout and decorations. It also exposes resolved font metrics without requiring a glyph run, so an empty decorating fragment can query its associated decorating-box font at that box’s computed size and effective variation coordinates. Strikeout position is a center offset positive above the alphabetic baseline. Use valid same-instance font strikeout metrics. Skrifa’s decoration offset is top-defined; normalize its offset to the owned center convention by subtracting half its reported thickness. If the provider decoration is absent or has a nonfinite offset/thickness or nonpositive thickness, use center offset half the resolved ascent and thickness computed size divided by 18 in scaled logical units. Reject invalid ascent/size rather than creating an invalid fallback. These are explicit Surgeist fallbacks, not empty-set averages or a CSS-mandated formula. Place a horizontal decoration with center `baseline - offset`, then subtract half the used thickness for its top edge. Writing-mode conversion occurs at the placement boundary. The Parley witness retains the upstream ascent/2 fallback and raw units-per-em/18 expression; the replacement deliberately scales the fallback thickness and does not copy that raw value. Render's admitted representation bounds establish graph paint ink and may differ for color/bitmap glyphs. Neither replaces advance/line boxes, clip, overflow or interaction geometry. Preserve `Unspecified` only for direct render paths that already support it; graph commands require admitted `Ink`/`Empty`. Do not require GPU access for font/instance admission or bound calculation. Backend-local preparation and rasterization remain in render; render owns the consuming adapter and paint association.

Accessibility remains an optional text-owned projection from analyzed source or its committed placed layout, following the [phase and binding rules above](#font-independent-analysis-and-accessibility). Host parent/focus identity remains supplied by the caller. Geometric cursor/selection/hit APIs use the same source affinity and finalized visual order, independent of paint ink.

## Parley mechanism closure and direct dependencies

This is a mechanism-level extraction inventory proven by the installed 0.9 source; adaptation must close its actual private calls and characterize behavior before deleting the intermediary. It is not permission to copy the entire upstream package.

| Installed source | Mechanism to retain/adapt | Excluded or split responsibility |
| --- | --- | --- |
| `analysis/mod.rs`, `analysis/cluster.rs`, `bidi.rs` | Unicode/cluster analysis and bidi facts. | Additional CSS feature semantics stay in their existing issues. |
| `resolve/mod.rs`, `resolve/range.rs`, relevant `style/` conversion | Font-family, feature/variation/style resolution for the current owned source model. | Tree builder and upstream style front door are not required; adapt `setting.rs`/`inline_box.rs` conversions to the existing owned setting/inline-box values instead of retaining parallel public models. |
| `shape/mod.rs`, `shape/cache.rs`, `convert.rs` | Harfrust buffers/plans/instances, fallback shaping, source clusters and safe provider conversions. | Replace external-input unwraps and compact-offset assumptions. |
| `context.rs`, `builder.rs`, `font.rs` | Necessary preparation orchestration and safe Fontique calls, folded into existing `System`. | Do not ingest parallel public builder/context/font APIs. |
| `layout/data.rs`, `layout/run.rs`, cluster/line access mechanisms | Shaped storage, normalized coordinates, effective font metrics, cluster/source mapping and finalized line access. | Separate shaped data from line placement; do not preserve provider-chosen composed wrapping. |
| `layout/line_break.rs`, `layout/alignment.rs` | Selected-line finalization/line-edge and glyph-adjustment mechanisms; required simple standalone chooser subset. | Composed break/float choice, box allocation and placement stay in layout. Consolidate duplicated L2 reorder. |
| `editing/cursor.rs`, `editing/selection.rs`, needed layout cursor helpers | Cursor, word/visual movement and selections over committed lines. | `PlainEditor` editing orchestration is not required by the current wrapper. |
| `layout/accessibility.rs` | Optional AccessKit projection over owned committed data. | No Parley carrier in the public optional API. |
| `lru_cache.rs`, required `util.rs` functions | Private caches/math needed by retained callers. | Cache capacities are performance choices, not semantic text caps; prevent epoch overflow. |

The baseline is Parley `0.9.0`, crates.io checksum `8fad031076f48f0d4d85ce1aea9b94b4e715a4d636a030a123038f8f5b5e4343`, whose complete resolved dependency identities remain in [Cargo.lock](https://github.com/bj-data-eng/surgeist/blob/3c412118b5e9f4ad8a34a1331f8c500618599498/Cargo.lock#L2540). Retain the following direct safe providers for that mechanism closure, using the already resolved versions. These are requested features, not a claim about the entire workspace's unified feature set.

| Provider | Resolved version | Text requested features (`default-features = false`) | Role |
| --- | --- | --- | --- |
| Fontique | 0.9.0 | `std`, `system` | Safe font catalogue/matching/fallback and byte resources; native implementation remains external. |
| Harfrust | 0.6.2 | `std` | Shaping. Parley 0.9 is not Swash-backed. |
| Skrifa | 0.42.1 | `std` | Same-instance font/outline metrics. Render independently owns its Skrifa feature needs. |
| ICU normalizer/properties/segmenter | 2.2.0 each | `compiled_data` | Current Unicode analysis data. Segmenter's `auto`/LSTM feature is not implied by current Parley behavior. |
| Parley Data | 0.9.0 | `baked` | Unicode analysis data; package use does not preserve the Parley intermediary. |
| Parlance | 0.1.0 | `std` | Font attributes/settings shared with Fontique. |
| Linebender resource handle | 0.1.1 | `std` | Safe immutable resource handles where the retained provider API requires them. |
| AccessKit (optional) | 0.24.1 | none | `text-accessibility` directly activates this provider and owned projection, without `parley/accesskit`. |

Keep the current empty default feature set. The owned caches and accessibility maps use the standard library and private LRU; they do not retain the upstream Hashbrown helper calls or require a direct Hashbrown dependency. Remove the text leaf's render feature/dependency through #1082 after moving consumers. COSMIC Text and fontdb are not Text dependencies; their former Swash stack is not the source for a new owned font stack. Do not ingest Fontique's native unsafe implementations. Source ingestion preserves original copyright and Apache-2.0 OR MIT attribution, records derived file identities/modifications, and obeys the repository's safe Rust policy. No new source acquisition is needed to define this contract.

## Required evidence and remaining implementation checks

Use deterministic registered fonts and independent expected facts, not only nonempty/finite assertions. #1080 checks ligature/mark/Arabic clusters, selected-boundary shaping, bidi with atomic placeholders, whitespace/expansion mappings, selected replacements, variation/synthesis metrics, source/font lifetime and actual refresh/cache invalidation. #1082 checks exact admitted face/instance/glyph/transform/bounds payloads, color/bitmap versus outline admission, empty ink, unsupported graph bounds, malformed fonts, precision edges and atomic failures headlessly; selected GPU checks confirm the admitted parameters execute unchanged. #72 checks float-dependent candidate feedback, spacing feedback, one committed line set for all consumers and source-preserving plaintext. #73 checks that integration evidence against its retained standards conformance obligations; unit provider tests do not establish conformance.

Browser fixture qualification uses actual captured source, context and the same pinned licensed font resources in the browser and Text. Measured widths, estimated baselines and observed bidi output are oracles, not preparation inputs. The former numeric-only inline-segment protocol cannot qualify this composed path. Layout owns its replacement capture/validation/XML/reader contract under [#1450](https://github.com/bj-data-eng/surgeist/issues/1450); shared browser/process mechanics remain Generator-owned. Range fragment rectangles are distinct from glyph ink. Preserve explicit disposition of every affected historical case, regenerate changed expectations through the producer, and leave real resource or unsupported-profile gaps visible rather than treating them as passed parity.

Before removing Parley, implementation must verify the private extraction call closure, characterize the standalone supported API against deterministic expectations, and establish the renderer's exact synthesis/color/bitmap bounds execution for the chosen backend. UTF-16 evidence includes exact source/edit round trips, both half-pair ranges, collapsed interior endpoints, RTL pair/grapheme geometry, isolated units and literal U+FFFD provenance, and edits joining/splitting pairs. Verify original property analysis independently from glyph replacement, and reuse exact finalized geometry for selection and accessibility. Those are finite implementation checks inside #1080/#1082, not blanket prerequisite gates. Separate source/standards ambiguities remain in their current issues; this contract does not resolve them by inheriting Parley behavior.
