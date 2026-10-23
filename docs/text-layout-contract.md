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

### Font-independent analysis and accessibility

`AnalyzedSource::new(Source, Style)` performs the font-independent analysis once and returns an immutable value with an opaque allocation identity. Clones retain that identity; independently analyzed equal text does not. It preserves exact source units, grapheme and host-projection mappings, source affinity and applicable language facts. Isolated UTF-16 units remain valid analysis inputs; a host requiring scalar UTF-8 may reject their accessibility projection at that boundary. Font preparation retains the same analyzed allocation. `System::layout_analyzed` consumes this analysis through the same preparation, selection, finalization and placement machinery as the `System::layout` convenience path.

This deliberately replaces the former successful fontless `Layout` phase while preserving its useful source-only accessibility capability. The existing `absent_selected_font_does_not_erase_original_selectable_graphemes` regression establishes that original graphemes remain selectable without glyphs. Preserve that behavior through the real analyzed-source front door in the same implementation change. Do not retain a fontless-layout branch or manufacture a font, glyph, prepared value or empty geometry to satisfy it.

A renderable shaping request without a usable admitted face returns a typed no-usable-face/font-resolution error. An empty `RegisteredOnly` catalogue is valid and is distinct from font-service unavailability. Empty, collapsed, control-only or box-only content does not automatically require a glyph face; any requested font-dependent line metrics still require their actual font. A failed layout publishes neither partial layout nor a source-only accessibility update. Callers may explicitly retain analysis and request source-only projection independently.

Source-only accessibility takes no placement point. It reports source values, units, language and selection mappings, omitting font/glyph facts, visual lines, character positions/widths and parent bounds until actual placed geometry exists. Missing geometry is neither a zero rectangle nor proven empty ink. Geometric accessibility enriches the same semantic analysis with the actual finalized rows, without a second segmenter or shaping pass.

Accessibility state binds to the exact analyzed allocation and an explicit source-only or placed phase; placed state additionally binds to the exact placement identity. Source-only reads require the source-only binding, and layout reads require the matching placed binding. Updates may coherently replace or enrich the binding, rebuilding optional fields so previous geometry cannot survive a source-only replacement. Preserve checked node-ID issuance, source affinity, focus rules and atomic failure. Verify selectable fontless graphemes, explicit layout refusal, ordinary registered-font enrichment, mismatched analysis/placement rejection, absent fabricated geometry and preservation of the previous accessibility state after failure.

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

An ordinary opportunity and a conditional opportunity are distinct terminating
capabilities. Conditional enumeration binds the exact preparation, unchanged line
start, first genuinely formatted ordinary ceiling and purpose. Finite overflow
fitting uses `Overflow`; unconstrained minimum-content pieces use `MinContent`.
Maximum-content does not request conditional cuts. Enumeration supplies legal
cuts, without proving ordinary-search exhaustion or any candidate's actual fit.
Raw analyses retain their separately characterized ordinary opportunity inventory;
this operation adds no raw conditional cuts. Common owner and interval validation
still applies. A consumer does not catch an unsupported-profile error or inspect
private preparation state to choose a different algorithm.

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

Layout supplies each eligible control's actual allocated strut geometry, qualified by its prepared control identity: finite baseline, line extent and alignment from the captured line-break or inline-boundary input. These are Layout contributions, not fabricated font metrics or atomic glyphs. Text combines them with its actual font strut during finalization; changed canonical contributions reenter height/float feedback. Pure formatting controls may have no allocated strut. Neutral inline start/end boundaries preserve actual adjacency and do not create break opportunities merely by existing. Layout retains clear and box/float decisions.

Native preparation explicitly captures its paragraph strut policy. A default-font strut resolves and retains the actual base-style face during preparation and contributes on every selected line, even when body spans use other fonts; a missing requested face is an error. Each body font contributes its own line-height strut, including its own half-leading, before Text combines baseline-, top- and bottom-aligned allocations. The allocation-only policy omits the base strut; it may prepare without a font when no other input requires one. Finalization requires actual contributions for every otherwise metric-less line, including a terminal empty line after a forced break. An explicit zero allocation is distinct from a missing allocation. Control presence and catalogue availability cannot silently choose this policy. These native strut roles follow the pinned [CSS Inline layout-bounds rules](../references/css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md#inline-height). Raw standalone layout retains its characterized source-adjacent metric policy; genuinely empty standalone text retains its admitted-font strut.

Retain the normalized formatter's deterministic alignment-group policy: let `b` and `a` be the independent maxima of signed before/after contributions in the baseline group, and `T`/`B` the largest top/bottom-aligned extents when present. Set `u = max(b + a, T)` and extend above by `max(B - u, 0)`; the resulting baseline is `b` plus that extension and the physical line extent is `max(0, u, B)`. An absent alignment group supplies no constraint, and an absent baseline group uses zero only as its coordinate origin. Preserve signed baseline contributions, font half-leading and overflowing box geometry; only the overall physical extent has a zero floor. This preserves the existing box/control formatter's choice where the pinned CSS Inline draft leaves tall top/bottom content positioning underdetermined; it is a Surgeist placement policy, not a claim that the draft specifies a unique baseline.

A native empty origin retains a zero-width, zero-height source-fragment anchor at its actual finalized baseline and inline position. Its occurrence carries no independently captured font, so its fragment bounds cannot borrow a neighboring font or the whole line enlarged by unrelated boxes. Caret and accessibility geometry retain their separate actual line-strut semantics. Raw standalone fragment geometry retains its characterized behavior.

The standalone `System::layout` convenience path retains the currently supported wrapping/alignment behavior by choosing lines through the same preparation/finalization machinery. Its simple paragraph chooser is private to text; layout’s composed CSS formatter does not use it as a second formatter. Unsupported style combinations remain explicit errors until the owning standards issue implements them.

The explicit native `TabContext::FontAdvance` policy uses the selected font's actual U+0009 glyph advance, preserving the standalone policy through the same finalizer. It does not approximate a tab with a space or implement CSS tab stops. `NoTabs` validates actual absence. The [CSS tab contract](#preserved-css-tab-stops) below defines the independently prepared CSS-stop operation and its required block capture. A taken discretionary opportunity requires its actual issued alternative, including an explicitly computed empty replacement; a natural paragraph end does not select a hyphen merely because the preceding source contains SHY. A source-defined replacement validates its real defining source occurrence. A control-defined replacement retains its exact control/opportunity/alternative owner and may have no text origin; incidental source-anchor provenance must be supplied and validated, never guessed from the numeric offset.

### Native whitespace edges and hanging

Text owns selected-line Phase II classification under #64/#72; Layout consumes
its measurements for fit, intrinsic sizing, justification and placement. This
native profile follows the pinned [Phase II](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#white-space-phase-2)
and [hanging definitions](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#hanging),
with #74's [preserve-spaces determination](../references/SOURCE-EDITIONS.md#operative-source-decisions-by-issue).
It selects the required operation, without claiming unqualified feature support.

Classify actual processed code points, governing styles and canonical line order.
Other space separators are Unicode Zs excluding U+0020 and U+00A0. NBSP cannot
inherit their removal or hanging rules. Collapse removes its applicable leading
and trailing U+0020 and trailing U+1680, retaining source custody. Under Preserve
with wrapping enabled, eligible trailing sequences hang unconditionally at a real
Soft ending and conditionally before an actual forced break or block end.
PreserveSpaces applies that same Phase II rule after its own Phase I conversion;
a converted LF does not supply a forced break. No preservation-rule hanging
applies under nowrap. BreakSpaces requires its real processing and after-each-space
opportunities, and never hangs its preserved spaces/tabs/separators; it cannot be
implemented as a finalizer flag. Missing modes retain their positive requirements.

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
hanging5; in width40 it hangs0 and centers as a 20px sequence. Preserve signed flow
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

Actual nonzero inline border/padding hanging barriers require their own captured
boundary facts. Neutral native controls do not prove arbitrary styled boundaries
have zero edges. Add that concrete producer/consumer operation without waiting
for a complete Style engine; no-barrier cases remain independently implementable.
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
that measurement. A positive interval uses half of the hypothetical ZERO advance
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
even at zero extent. A negative resolved interval currently requires a precise
unsupported capability: it is not invalid CSS and must not be clamped, made
positive or recomputed without spacing. Nonfinite arithmetic, overflow and
nonzero numeric collapse are distinct numeric failures, never semantic zero.
Negative-interval execution remains required work for full #64/#72 acceptance.

Use physical +x coordinates temporarily, with unaligned paragraph progression
start defined as zero. The initial pen is +indent for LTR and -indent for RTL.
Layout supplies the actual block content-start minus actual paragraph-start
displacement in this frame, recomputed for each float band and retry. Direction
alone does not prove that displacement. With block origin O and positive period T,
the grid is O+k*T for signed integer k. Paragraph progression selects the closest
strictly ahead stop, followed by exactly one additional stop if the distance is
below half-ch. Extending this grid beyond the block edges is an explicit Surgeist
policy, not an established browser or CSSWG resolution. For example, block RTL
origin 100, paragraph LTR pen zero and period 40 select stop 20 when half-ch is at
most 20; a larger threshold selects 60.

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
and cycle detection. Unknown block extent remains unknown. A positive preserved
tab whose intrinsic phase depends on an unknown opposite edge requires a precise
unresolved-intrinsic-tab-phase capability; zero or removed tabs do not demand that
edge. A known placement frame does not redefine an unconstrained intrinsic query.
No zero-origin substitution or self-consistency solver is selected. This case
also remains required work for full acceptance. Qualification must cover actual
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
style for font selection and hyphenation eligibility, and `none` suppresses both.
Conflicting modes or duplicate definitions are invalid. A bare source SHY uses
style resolution rather than silently becoming an ordinary break without an
indication. A terminal paragraph end does not select that SHY alternative.

Retain the actual resolved convention, governing style and font/resource context
in preparation and all affected cache identities. Qualification covers none versus
visible-hyphen breaks, manual with unknown language, exact specified strings and
empty endings, explicit-script precedence and supported script fallback, known
unsupported conventions, primary-font versus adjacent-fallback coverage, genuine
source/control custody, min-content and unchanged authored/selection/search text.
This determination supplies requirements, not an implementation-completion claim.

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
`Auto` admits automatic production; its `Manual` or `None` does not. Empty
wrappers and out-of-flow occurrences cannot own the cut. Nearest-common-ancestor
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
For `as` + SHY + `sociate` with 10px letters and a 10px indication, width 80 takes
the explicit cut after `as`; at width 60 its 70px residual portion permits
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
For example, `as` + SHY + `sociate` with 10px letters and a 10px indication has
MinContent 50px through pieces 30/30/50, refining its manual 30/70 partition.
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

For the 10px-letter, 10px-indication `as` + SHY + `sociate` example, MinContent is
50px at limit 1 or `no-limit`, and 70px at zero when no independent breaking
opportunity applies. MaxContent remains 90px. Query order cannot change these
measurements or the next actual line's eligibility. These intrinsic and emergency
rules are explicit Surgeist policy, not a claim of uniquely specified CSS behavior
or implementation completion.

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
