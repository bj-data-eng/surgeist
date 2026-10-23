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
input model. Text derives the influencing relation after its admitted processing
and transparency rules, validates the supplied endpoint correspondence, and
rejects foreign, reversed, non-neighbor or conflicting same-relation declarations.
Keep one semantic permission owner for text and atomic endpoints. Reuse captured
occurrence/source identity; do not add a DOM tree or generic style interface to
Text. If attachment requires Text's processed neighbors, expose only the narrow
analyzed relation/provenance view needed by the producer. Native fixtures may
explicitly author these facts without claiming live Style authentication.

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

Finalization is deterministic for a preparation and complete selection context. Layout's feedback algorithm visits its finite candidate/context states until it selects a valid state or returns its existing no-fit outcome; revisiting a complete already-rejected candidate/context state is a typed feedback-cycle error (or the formatter’s explicit no-fit failure), even when feedback is identical and even for a cycle through multiple states. A stable fitting state commits; rejected states cannot be retried without a changed input. Do not introduce a magic maximum pass count. Layout must ensure progress in its own float/box algorithm. A no-fit line allowed by applicable overflow semantics remains a deliberate layout outcome, not a text finalizer fallback.

Layout supplies each eligible control's actual allocated strut geometry, qualified by its prepared control identity: finite baseline, line extent and alignment from the captured line-break or inline-boundary input. These are Layout contributions, not fabricated font metrics or atomic glyphs. Text combines them with its actual font strut during finalization; changed canonical contributions reenter height/float feedback. Pure formatting controls may have no allocated strut. Neutral inline start/end boundaries preserve actual adjacency and do not create break opportunities merely by existing. Layout retains clear and box/float decisions.

Native preparation explicitly captures its paragraph strut policy. A default-font strut resolves and retains the actual base-style face during preparation and contributes on every selected line, even when body spans use other fonts; a missing requested face is an error. Each body font contributes its own line-height strut, including its own half-leading, before Text combines baseline-, top- and bottom-aligned allocations. The allocation-only policy omits the base strut; it may prepare without a font when no other input requires one. Finalization requires actual contributions for every otherwise metric-less line, including a terminal empty line after a forced break. An explicit zero allocation is distinct from a missing allocation. Control presence and catalogue availability cannot silently choose this policy. These native strut roles follow the pinned [CSS Inline layout-bounds rules](../references/css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md#inline-height). Raw standalone layout retains its characterized source-adjacent metric policy; genuinely empty standalone text retains its admitted-font strut.

Retain the normalized formatter's deterministic alignment-group policy: let `b` and `a` be the independent maxima of signed before/after contributions in the baseline group, and `T`/`B` the largest top/bottom-aligned extents when present. Set `u = max(b + a, T)` and extend above by `max(B - u, 0)`; the resulting baseline is `b` plus that extension and the physical line extent is `max(0, u, B)`. An absent alignment group supplies no constraint, and an absent baseline group uses zero only as its coordinate origin. Preserve signed baseline contributions, font half-leading and overflowing box geometry; only the overall physical extent has a zero floor. This preserves the existing box/control formatter's choice where the pinned CSS Inline draft leaves tall top/bottom content positioning underdetermined; it is a Surgeist placement policy, not a claim that the draft specifies a unique baseline.

A native empty origin retains a zero-width, zero-height source-fragment anchor at its actual finalized baseline and inline position. Its occurrence carries no independently captured font, so its fragment bounds cannot borrow a neighboring font or the whole line enlarged by unrelated boxes. Caret and accessibility geometry retain their separate actual line-strut semantics. Raw standalone fragment geometry retains its characterized behavior.

The standalone `System::layout` convenience path retains the currently supported wrapping/alignment behavior by choosing lines through the same preparation/finalization machinery. Its simple paragraph chooser is private to text; layout’s composed CSS formatter does not use it as a second formatter. Unsupported style combinations remain explicit errors until the owning standards issue implements them.

The explicit native `TabContext::FontAdvance` policy uses the selected font's actual U+0009 glyph advance, preserving the standalone policy through the same finalizer. It does not approximate a tab with a space or implement CSS tab stops. `NoTabs` validates actual absence; requested `Stops` remains a precise unsupported capability until implemented. A taken discretionary opportunity requires its actual issued alternative, including an explicitly computed empty replacement; a natural paragraph end does not select a hyphen merely because the preceding source contains SHY. A source-defined replacement validates its real defining source occurrence. A control-defined replacement retains its exact control/opportunity/alternative owner and may have no text origin; incidental source-anchor provenance must be supplied and validated, never guessed from the numeric offset.

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
universal CSSWG resolution or exact browser-test agreement. The conditional
operation remains required work; the existing float adapter alone does not
establish its implementation.

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
