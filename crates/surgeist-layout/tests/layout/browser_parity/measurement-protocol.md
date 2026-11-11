# Browser measurement protocol, version 4

This is the private interchange between layout's bundled JavaScript helper and
its Rust browser-corpus adapter. It describes measured fixture data, not authored
CSS, a public layout API, or the generator crate's report schema. Browser reports
continue to use schema 4 independently of this protocol version.

## Root requests and observation frames

Version 4 separates sizing operation, physical availability and observation
frame. Preferred CSS width/height, an observed root rectangle and a
viewport-filling test do not issue intrinsic operations. The closed
`viewport.rootContext` alternatives are:

| Kind | Required fields | Observation |
| --- | --- | --- |
| `absolute-root-local` | `containingWidth`, `containingHeight`, `containingDirection`, `hypotheticalDirection`, `hypotheticalDisplay`, `hypotheticalAnchor`, `hypotheticalInlineOffset`, `fallback` | `observation-frame="root-border-local"`; no document-space placement claim. |
| `block-flow-auto` | `parentInlineExtent`, `parentDirection`, `fallback` | `observation-frame="root-border"` with independently checked actual Native body placement. |

Absolute-local containing extents and direction belong to the actual ICB/html
frame. Hypothetical direction belongs to the body; root direction remains its
own captured style. The authenticated pre-blockification display comes from the
original inline declaration or the exact owned base sheet's DIV default, not
computed Grid. Its selected set is `block`, `flow-root`, `flex`, `inline-flex`,
`grid`, `inline-grid` and `grid-lanes`, subject to all other source checks.

The anchor is `inline-start`. Its finite signed offset `s` is a distance along
the hypothetical body's progression over the same physical containing width
`W`, not a physical-left coordinate. LTR and RTL both use `max(0, W - s)` once
as inline availability; the offset itself is never converted to `W - s` for RTL.
The request keeps full actual CB width/height as percentage bases, Normal
operations and independently indefinite block availability. Native shrink-to-fit
acceptance uses min/max contributions. Root-local output starts at zero.

The ordinary zero-offset profile has an empty, unindented, start-aligned
hypothetical first line, with only collapsible ASCII whitespace/comments and no
inline advance donor or preceding float. A separate literal-indent profile permits
one body declaration, `text-indent`, as a finite nonnegative literal pixel length
not greater than the containing width. HTML and body must both be horizontal LTR,
with normal whitespace, start alignment and the admitted ordinary/isolate bidi
state. The body has no element sibling of the root. The hypothetical display must
be `flex` or `inline-flex`, and every direct Flex item must be an empty DIV with
positive fixed-pixel width/height and `flex-shrink:0`, using the controlled base
edges and positioning. Hypothetical block Flex has `s = 0`; inline Flex receives
the literal first-line indent. Other inherited-indent consumers, nested/text
content, nonzero cross-direction indentation and other body declarations refuse.
The profile does not invent a font, strut or line rectangle.

A separately qualified static spacer prefix may contain empty DIVs with exactly
`display:block; position:static; width:0px; height:<positive literal px>`.
They must have zero actual edges, no float/clear or external displacement, and
precede the absolute root; element siblings after it refuse. They affect only the
omitted common block translation. The complete root-local dependency proof still
applies: no spacer/root rectangle, used body height or line metric supplies a
request operand. Preserved whitespace, leading floats and arbitrary siblings
remain unsupported. Indent and spacer profiles are not combined.

BlockFlow has Normal operations, definite parent inline extent, independently
indefinite Auto block extent and percentage parent `(Some(inline), None)`.
Root flow remains independent. The reader checks actual unrounded Native body
placement: LTR starts at zero; RTL at `parentInlineExtent - actual Native root
width`. It then normalizes root observations using the computed Native origin.
Only a source-qualified static/all-Auto root lowers to Native in-flow Relative
with zero displacement; original static positioning remains in XML/provenance.
Other static/offset profiles refuse.

The public Native `PositionedRootContext` remains separate for
containing-frame placement; see the [Native root contexts](../../../src/output.rs).
V4 has no `positioned-auto-insets` wire alias. Native root-local padding/border
admission requires a definite preferred width under its sizing resolver. Browser
absolute-local source admission permits padding/border only when the actual
preferred inline projection is finite nonnegative pixels, or a percentage that
resolves finitely against the full authenticated containing width. Margins stay
zero. Auto, intrinsic and general calc preferred widths do not acquire an
edge-sensitive shrink-to-fit formula through this profile.

The non-collapsing initial-origin check remains with BlockFlow's placement
certificate. An actual absolute root establishes its own formatting context,
including visible-overflow Block; absolute-local admission instead requires the
complete subtree translation-independence proof below. It does not certify a
hypothetical block origin or derive one from zero. Source-frame admission alone
does not establish Native font or geometry parity.

### Source and stylesheet custody

The adapter authenticates the exact captured base sheet, profile hash
`5d00a3f3c55322b7002b065eacc6b4f3f14ecad83f757c79679b6ec6dee4fec6`.
The helper authenticates the original direct-body DIV source and one base LINK.
It replaces only that exact live LINK in place with one owned STYLE containing
unchanged captured bytes, including the data-URI font. Every variant checks the
retained owner, text, CSSOM rules and complete sheet inventory. Repeated setup
retains that owner; a marker cannot authorize replacement or repair changed
source. Additional/adopted/opaque/changed sheets refuse.

The source script profile permits zero scripts, or one src-only helper in HEAD
with no non-whitespace text. The live script census must match the source.
The helper profile retains the original node, attributes, exact text and
resolved URL. Each variant checks
these facts before and after description. Failed reconfiguration retains those
owners and refuses mutation; restoration permits full revalidation.

The Subgrid body, atomic percentage and BR font-band profiles use
[profile-owned helper predicates](../../bin/surgeist-layout-generate/measurement/).
Preparation and captured-input validation share those predicates. Generic root
helper recognition does not grant specialized eligibility.

The closed profile has no extra scripts/frames or HTML inline declarations.
Body declarations and extra body content are limited to the literal-indent and
static-spacer profiles above, and only for absolute-root-local capture. Source
and live body serialization must match. Retained live child-node references
also authenticate body membership, source order and identity on every variant;
repeated setup cannot repair replaced or reordered children. Source and live
subtree/frame facts must remain stable.
Absolute-local capture additionally checks every box for local facts that do
not depend on a common translation. Fixed/sticky/transformed frames, external
sizing and anchors, remaining-block-space dependence, noninitial scrolling and unqualified
typed-inline/observation channels refuse. Boxless `display:none`/`contents`
observations refuse with `RootLocalBoxlessObservation`; their world-zero rectangle
is not normalized into a fabricated local zero. A zero-size box remains
a box. Hidden clipping within the admitted moving subtree remains local. The base
`.viewport` class refuses because its sizes are absent from this inline-declaration-only sizing channel.

A closed absolute-local profile also permits nonhorizontal plain-box descendants
beneath the horizontal root. Each such descendant must author finite nonnegative
literal physical pixel width and height. The proof uses those source declarations;
used rectangles and resolved dimensions do not establish fixed sizing. Ordinary
empty Flex leaves retain their existing eligibility.

If any descendant uses this profile, the whole moving subtree has a restricted
inline source vocabulary: display, writing mode, direction, physical width/height,
Grid templates/auto-flow and Grid placement. Other declarations refuse, including
logical sizes, bounds, aspect ratio, position/inset overrides, clipping and scroll
dependencies. Descendants retain the authenticated static/relative all-Auto-inset
base behavior, zero edges, visible overflow and normal whitespace. Individual
transforms and motion paths must also be absent in computed style. Only
CSS-collapsible ASCII whitespace is admitted as text; preserved whitespace and
Unicode spacing glyphs refuse. Existing tag, display, source/live identity and
producer-channel checks still apply. This profile does not admit nonhorizontal
roots, Auto/intrinsic orthogonal sizes or the ElementScrollSizes channel.

The fractional spacer controls independently check complete local geometry and
raw DOM-size invariance, with a changed world-rounded height phase at the root
and its first child. Source admission does not certify that a browser implements
the adopted Grid Lanes orientation semantics; those comparisons retain their
own source applicability boundary.

The descriptor used by both root contexts validates preferred-size, bound,
flex-basis, edge and gap projections. A present value whose actual projection is
absent refuses with `Root context: UnresolvedSizingDeclaration`;
for example, an unsupported own-width unit or unresolved own-width variable
cannot disappear into Native Auto. Existing min/max computed projections and
represented literals/calcs remain intact. Unconsumed shorthand reset fields are
not sizing operands and do not trigger a blanket CSS-wide refusal. Token-parser
grammars retain their absence contract; validation belongs to the descriptor
consumer. This does not implement a cascade or qualify every source declaration.
Preferred, minimum and maximum sizing share one actual node-writing-mode
projection. Each group refuses simultaneous physical and corresponding logical
size on one physical axis with `ConflictingSizingDeclarations`; declaration
order or priority is not guessed. Single logical sizes/bounds and mixed
declarations on distinct axes retain their role. Minimum/maximum getters keep
their existing computed projection before the shared mapping. Gap shorthand
repeats a value only when one component was authored. An unsupported second
component remains absent for validation and cannot be replaced by the first.
Padding and inset inline-start/inline-end declarations share one mapping through
that node's actual writing mode and direction. CSSOM supplies expanded logical
longhands for authored inline shorthands. Physical and logical declarations on
the same physical side refuse with `Root context: ConflictingEdgeDeclarations`;
distinct or opposite sides remain independent, with existing literal, calc and
percentage projection checks. The absolute-root-local source closure refuses
logical block padding/inset declarations with
`Root context: UnrepresentedLogicalBlockEdges`; this does not add block-edge,
border or margin normalization or a cascade interpreter.
After root-context source checks, parsed inline `border-block*` and
`border-inline*` declarations on the root or its descendants refuse with
`Root context: UnrepresentedLogicalBorder`. This includes shorthands and
mixed physical and logical declarations. Physical border widths use the
existing physical fields.
Broader source profiles or a changed base asset need independent qualification.

Extents come from the standards-mode unscaled CSSOM viewport after scrollbar
reservation, with matching visual viewport and actual body inline extent. Used
Auto body height and browser root rectangles never supply request operands.
The `.rtl *` selector does not set body direction. Unsupported contexts retain
precise `Root context:` reasons; no historical MaxContent field becomes Normal.

Fallback requires independent `initialContainingBlock` physical width/height,
`nearestScrollport: "none"` and `containingLimits` with both axes `unbounded`.
This profile proves absent intervening element scrollports and default local
limits. Other states need their own source proof. Native `OrthogonalFallback`
owners retain/replace those components: a nearer unfixed scrollport must not
inherit an older fixed bound.

### Continuous local rounding and separate diagnostics

Let `Q` be the existing coordinate-rounding operation. Absolute-local sizes use
`Q(worldEnd - rootStart) - Q(worldStart - rootStart)`: continuous corners are
normalized before rounding. Child locations retain `Q(worldStart - parentStart)`,
not a difference of separately rounded root-local starts. Root selected x/y are
zero. Unrounded sizes and child parent-local locations remain measured, with
root x/y zero. The measured root start is an observation operand only.

BlockFlow retains its actual world-corner size phase and parent-local locations,
plus its body-placement check. Neither path drops size, child, scroll, source
or control comparisons because root observations are local.

Every wire `layoutInput: "box"`, including a BR/control box, requires
`worldRoundedLayout`: the former smart-rounded values, with parent-local
`Q(delta)` x/y and world-corner width/height. They are not document positions.
Full x/y/width/height are required; scrollWidth/scrollHeight and
clientWidth/clientHeight are optional complete pairs. Values remain finite f64,
with signed locations and nonnegative extents. Non-box typed text, boundaries
and reason-only unsupported alternatives cannot carry this field.

XML retains them separately as
`<diagnostics><world-rounded-layout node-path="0.0" .../></diagnostics>`.
Numbers are unitless CSS pixels; optional pairs use `scroll-width`/`scroll-height`
and `client-width`/`client-height`.
Root path is `0`; later dot-separated indices select the actual measured child
array. Structural XML metadata is not a child occurrence. The reader checks the
per-box census against the input tree. Diagnostics are separate from ordinary
expectations and Native inputs. The fixed wrapping edition explicitly selects
the root world-corner height for one additional exact comparison under the
[wrapping policy](../../../semantic/wrapping-world-corner-policy.md). The
edition retains the ordinary root-local height as a separate comparison.

Historical XML 1/2/3 remains exact raw, unqualified capture. Its viewport cannot
issue a current Native root operation: `MissingRootContextAuthority` remains
visible. Native tests may use retained Source/font IR with independently
authored requests. Old captures are not relabeled or supplied missing facts;
fresh V4 capture is required for current browser parity. Prepared font/source
capabilities are unchanged.

The opaque analytic provider remains separate. Known, definite, formal MIN/MAX
and parallel unbounded requests retain distinct operations. It does not claim
the complete orthogonal infinite-containing-block algorithm: capture and reader
refuse opaque analytic text through an orthogonal ancestor, including inherited
infinity. Actual core-owned leaf/containing axes are checked before accepting
Normal/Indefinite inline requests; no universal fallback cap is applied.
Prepared paragraphs retain their independent source/font/context path; the
absolute-local closure does not admit unqualified typed-inline channels.

### Explicit element scroll sizes

The qualifier's `--element-scroll-size-diagnostics --fixture <declared-source>`
mode requests four DOM integer observations in addition to existing geometry.
The canonical invocation input `observation-request.json` is registered in the
same input/report contract as source resources. Preparation and validation bind
the request to that input. Ordinary generation omits this input and sideband.

The closed profile is V4 `absolute-root-local`, horizontal ordinary source DIV
boxes, including DIV-backed opaque text leaves, with zero padding, borders and
installed/reserved gutters. Visible, Clip and Hidden overflow are supported.
Inline, boxless, replaced, prepared/control, vertical and Auto/Scroll profiles
refuse; unsupported occurrences cannot disappear from a partial table.

The optional `element-scroll-sizes` sibling follows `expectations` and precedes
`diagnostics`, with `profile="horizontal-zero-edge-divs"`. Its `element` rows
carry `node-path`, `client-width`, `client-height`, `scroll-width` and
`scroll-height`. Paths cover the complete original source tree in preorder,
starting at `0`; anonymous Native text helpers do not acquire source rows.
Duplicate, missing, reordered, unknown or misplaced rows/payloads refuse.

All four values come from the same selected smart-rounded or unrounded wire
object, before layout-number conversion. They are lossless integers in
`0..=i32::MAX`, with client extents no greater than scroll extents. The reader
compares them exactly, separately from ordinary layout tolerances. Historical
expectation trees and their operational scroll-range spans are unchanged.
World-rounded diagnostics never supply semantic values.

Native comparison uses the same computation's unrounded canonical scrollport
and complete scrollable overflow. For horizontal flow with port edges
`L,T,R,B` and overflow edges `Xmin,Ymin,Xmax,Ymax`, scrolling width is
`max(R,Xmax)-L` in LTR and `R-min(L,Xmin)` in RTL; height is
`max(B,Ymax)-T`. Client sizes are the port extents. Round each represented local
extent to the nearest integer, positive halves upward, after lossless widening;
adding `0.5` in f32 would corrupt some already integral values. Visible/Clip
operational ranges remain zero. A missing or unsupported projection invalidates
the complete case rather than becoming a numeric difference.

### Functional qualification before adoption

Use the real [qualification example](../../../examples/qualify-root-context.rs)
with `layout-golden-generate` and a verified cached browser. Its single source
records own scheduling, declared outcomes and historical observation hashes.
It uses the actual Layout adapter/public BrowserCorpus rather than a source or
cascade mock. Follow the [corpus procedures](README.md) and
[crate guide](../../../AGENTS.md) for execution/publication.

Require closed decoder/reader controls for complete payloads, explicit zero,
finite ranges, fallback facts, local-zero versus actual body placement,
world-diagnostic census and historical refusal. Real capture preserves original
source/resources and every historical observation field. Broader source profiles
need their own real-browser operand/refusal qualification; schema checks alone
establish no such provenance. Preserve original XML/raw mismatches until faithful
capture and full comparison exist. Filtered or finite-example generation does
not verify the full corpus. Do not alter counts, fonts, HTML, CSS or geometry to
hide missing source authority.

## Source-qualified Grid alignment reference

Every supported source-qualified V4 root carries required
`gridAlignmentProfile: { profile: 1, owners: [...] }`. Owners are complete
source-preorder input paths: root `/`, then actual child indices such as
`/0/0`. If every Grid owner is horizontal, the owner list is empty even when
its paragraphs use vertical named metrics. The wire and reader independently
derive required owner paths from the actual layout-ready-inline region, Box
Grid role and own vertical writing mode, then require both records and header
to match. This predicate demands the full typography proof; it does not infer
a name. Coherent record and list deletion therefore cannot erase source intent.
No second topology, fixture-name rule or child-metric registry is involved. XML retains the root
JSON in `grid-alignment-profile`. Missing headers, individual deletions,
duplicates, wrong scope, or orphan/out-of-order records refuse.

Each demanded actual HTML DIV/SPAN Grid owner separately carries
`gridAlignmentReference`, retained as `grid-alignment-reference` in XML:
profile 1; own Grid/InlineGrid display, writing mode and direction; own
Mixed/Upright text orientation; supported default dominant-baseline `auto`
and raw non-SVG alignment-baseline `auto`/`baseline`; actual source-bound
font descriptor/resource; and resolved reference `central`. Every own raw
role/frame/font operand must agree, with explicit font authority preserved
through default-compressed attributes. Authored reference overrides, foreign
fonts, sideways orientation and unsupported authority refuse rather than
selecting a fallback. Existing enclosing source/root profiles still govern
which HTML roles can reach this capability.

The source adapter derives the name from that full actual Grid-owner context.
Pinned Blink 79051446 `grid_item.cc` reads the immediate parent Grid's font
baseline; `ComputedStyle::GetFontBaseline` resolves the default through actual
orientation. The [pinned writing-mode text baseline rule](../../../../../references/css-writing-modes-4--CR-css-writing-modes-4-20190730--30a59de49343.md#text-baselines)
selects Central for the qualified vertical Mixed/Upright context. Writing mode
alone, descendant paragraph certificates, alignment First/Last roles or target
coordinates do not establish this choice.

The existing actual loaded CSS FontFace owner is shared. A Grid-only source
can demand that same preparation; it neither constructs another loader nor
borrows a paragraph font. Grid name selection issues no line-height, zero-strut,
BR, glyph or paragraph-metric claim. In particular the Grid's own normal
line-height remains unchanged, while an actual atomic paragraph separately
retains its H0 checks. Real loaded-font custody observes both declared named
consumer counts using the same retained face/load owner.

After independent validation, the reader assigns only the existing local
`NodeInput.grid_baseline_references` selectors to Named(Central). Inherited
subgrid axes retain their actual owner; First/Last selection, paragraph
`inline_baseline_context`, BR metrics and source correspondence do not change.
Unrequested authored Native input retains Physical under its explicit
Native test entry. That entry is distinct from the strict captured comparison,
which requires the enclosing profile even if every record was deleted. There
is no old-qualified-capture fallback or reinterpretation of historical XML.

## Source-qualified atomic-only named metrics

V4 optionally carries `atomicParagraphMetrics`, independently versioned by
`profile: 1`. It is an explicit new source capability, not a reinterpretation
of Physical measurements. XML retains the same JSON as the containing input
box's `atomic-paragraph-metrics` attribute. Absence keeps the existing Physical
operation. Prepared source/font capture remains separate and unchanged.

The closed source candidate is a layout-ready, non-Prepared vertical block at
line-height 0 with exactly two actual empty inline-block SPAN occurrences and one
intervening actual inline BR. Only discarded ASCII whitespace/comments may be
omitted. The profile authenticates the actual source indices and emitted child
indices, order, homogeneous frame, default inline alignment and break roles.
Its required `verticalAlign: "baseline"` is the applicable HTML alignment
authority on the containing paragraph and each actual occurrence. Raw wire
styles and XML may omit the separate non-inherited `vertical-align` field
when it has its semantic initial baseline value; every present value must
agree. The computed certificate field itself remains required. Inherited
font-family/font-size authority remains explicit and is not inferred from
absence.
Occurrences must be undisplaced: static positioning, or relative positioning
with Auto or zero used insets. All-Auto relative insets use zero under
[CSS2 relative positioning](../../../../../references/css2--visuren.html--3f334c530cf4.md#relative-positioning).
Nonzero resolved displacement refuses this closed profile.
Mixed typography supplies neutral atomic levels 0/1 from the actual LTR/RTL
paragraph direction. Upright is admitted only with actual LTR direction; the
separate used-direction projection needed for Upright+RTL is not inferred.
Authored bidi/break/strut overrides and other content require separate proof.

The first font must be the actual normal 400 Ahem face from the controlled base
stylesheet: the 1,496-byte WOFF2 SHA256
`e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c`.
This is Ahem 1.1, not the Prepared 1.50 TTF resource. The adapter already checks the
exact base resource; its one CSS FontFace is loaded in the asynchronous setup
stage without constructing a replacement face. Capture retains that loaded
object and checks the same unique FontFaceSet member on every variant.
Missing, competing, replaced, unloaded or unsupported face/style/metric authority
refuses. Repeated setup never reloads or substitutes a face. A transient competing
face must actually be removed before pristine capture can resume.

The analytic certificate is limited to actual 10px regular font metrics and the
supported default Mixed/Upright Central reference and line-fit context. The
certificate preserves actual computed `alignmentBaseline` as `auto` or
`baseline`; it does not turn the SVG property token into a general baseline
alias. Pinned Blink 79051446 keeps this raw CSSOM token, skips its SVG
alignment switch for HTML, and applies actual HTML `vertical-align: baseline`
while the inline owner selects Central for vertical Mixed/Upright. The
original WOFF2 has UPEM 1000, hhea/OS2 ascent 800/descent −200/gap 0, with no BASE,
vhea, vmtx, fvar or MVAR. Used Central has symmetric 5/5 sides. At line-height 0,
half-leading −5/−5 yields the certified before 0/after 0 tuple for the paragraph and
each actual BR. Zero line-height alone is never this proof. Unsupported font,
dominant/alignment-baseline, line-fit/trim or occurrence overrides refuse before
certificate issuance. This is a closed resource proof, not a generic font-table
parser, font provider identity, glyph paragraph or rectangle-derived metric.

Raw named-coordinate absence and used upright metrics remain distinct. Text's
`baseline(VerticalX, Central)` may be unavailable while `upright_metrics`
separately supplies used em sides at the nominal pen. Text's public SFNT/TTC
metric input does not admit these WOFF2 bytes unchanged; the closed proof does
not register the different TTF under a fictional shared identity. See the
[actual font metric owner](../../../../surgeist-text/src/font/metrics.rs),
[used upright adapter](../../../../surgeist-text/src/font/metrics/upright.rs)
and [font provenance](fonts/PROVENANCE.md).

The decoder and independent reader validate the complete certificate and every
actual atomic/BR binding. The reader sets only that paragraph's intrinsic
`InlineBaselineContext::Named(Central)` and uses the matching checked zero BR
metrics. No synthetic Text occurrence or boundary is inserted. An asymmetric
signed zero-height tuple cannot fit the existing checked InlineMetrics API and
refuses rather than being clamped. Parent Grid alignment-reference selection is
a separate owner; a named paragraph does not choose or relabel that reference.

The existing qualification inventory records the expected six paragraph/BR
certificates for each original column/vertical-row source. Its real-browser
[loaded-font custody mode](../../../examples/qualify-root-context/README.md#loaded-font-custody)
inserts a real competing FontFace from the same owned src, requires exact
refusal, removes only that introduced face, and requires fresh pristine capture
with the original loaded identity. Destructive stylesheet checks use a separate
non-font declaration because detaching a stylesheet can replace its FontFace
generation. Unit browser-host controls do not substitute for this real
FontFaceSet qualification. Strict
Native errors and original XML observations remain visible; named metrics do not
certify complete row geometry or whole corpus conformance.

## Authentic prepared-text input

The original-source profile introduced in version 2 is independently validated
by the Rust generator and the default XML reader. The reader does not import the
generator's wire types. `data-surgeist-prepared-text="ahem-ttf-v1"` opts a real
formatting container into capture. Its `<prepared-paragraph capture="…"/>`
contains JSON, escaped as an XML attribute; source strings are arrays of exact
UTF-16 code units. This profile precisely refuses isolated surrogates and source
without the pinned nominal coverage. It never normalizes them to replacement
characters or substitutes a different string.

Prepared fixtures explicitly author `unicode-bidi: normal` on the formatting
container, alongside the pinned font context. The opt-in does not override a
computed UA `isolate` value or authorize unrepresented isolation boundaries;
the helper validates the actual normal-bidi context before capture.

The vertical profile also requires computed `dominant-baseline: auto`. Its
authenticated Mixed/Vertical preparation resolves Text's central baseline, so
the reader requests `InlineBaselineContext::Named(Central)` on that paragraph
container. Layout owns matching child baselines and synthesis from
absence. Raw physical alignment and other typographic contexts retain their
separate operations; writing mode alone does not grant named-baseline authority.

The capture records:

- The exact font resource, face index, CSS descriptors, feature/variation and
  synthesis policy, and explicit successful FontFace loading.
- Resolved base direction, supported writing mode, canonical font size, logical
  line height and spacing, whitespace/wrap policy, inherited language and color.
- Original text occurrences, exact aggregate ranges, incident connections and
  captured containing wrap policies.
- Actual inline controls, atomic roles and original anchors, with one complete
  occurrence order. Empty origins are retained, including their identity.

The horizontal profile admits the covered original text described below. The
bounded vertical profile admits `vertical-lr` and `vertical-rl` with actual
computed `text-orientation: mixed`, fixed-pixel empty in-flow inline-blocks, forced
breaks and original collapsing formatting whitespace (LF, CR and SPACE). It preserves each
original whitespace occurrence and its exact position among the controls and
atomics. Visible vertical glyph text, other orientations and conflicting child
frames refuse. Capture direction and writing mode must match the actual
container; a generated variant name supplies neither fact. Vertical Range
observations use the physical top edge and height, independently of preparation.
Other vertical boundary controls remain outside this profile. Prepared whitespace
units can persist until Text selects and finalizes their line-edge disposition;
their preparation inventory is not a visibility test.

BR input participation does not select its observation contract. A fixture that
requests `data-surgeist-br-observation="font-band"` retains the browser's selected
BR rectangle as `<browser-br-font-band x="…" y="…" width="…" height="…"/>`
alongside its `<browser-control>` identity and line relations. These coordinates
use the same actual parent-local and rounding basis as ordinary measured
geometry. Atomic/BR and empty-line metric fixtures explicitly retain this request;
deliberately relation-only fixtures do not acquire one implicitly. Unknown
requests and non-BR/hidden/block/contradictory measurement roles refuse.

The [horizontal BR edition](captured/horizontal-br-font-band-qualification/)
qualifies the four original direct-child BR variants with the pinned Ahem face
at 20px and a 30px line height. Text returns the actual paragraph-default
instance and pre-leading ascent and descent for the exact unshifted forced-break
fragment on its placed line. Layout joins the visible source child to that
occurrence and uses the control endpoint and finalized baseline in the capture
root's coordinates.

The pinned Chromium [BR control fragment](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/logical_line_builder.cc#L419-L459)
uses the [primary font metrics before leading](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/inline_box_state.cc#L104-L151).
[CSSOM View](../../../../../references/cssom-view-1--WD-cssom-view-1-20250916--693b1c1fe7f7.md)
retains the degenerate element rectangle. Ahem's 1000 units per em and
800/200 ascent and descent yield 16px and 4px at 20px. The first line
baseline is 21px. The band spans y=5 to y=25 with zero width at x=10 in LTR
and x=110 in RTL. The rounded profile applies the browser's root-local
corner rounding. The integer font metrics and unshifted horizontal frame
bound this projection.

The [vertical BR edition](captured/vertical-br-font-band-qualification/)
qualifies twelve original variants across vertical-lr LTR, vertical-rl LTR and
vertical-rl RTL. Each original author stylesheet supplies child direction
inheritance. The authenticated first-block frame supplies a zero hypothetical
block offset from the ICB's physical top, independently of the absolute sizing
bases and the root-local origin. Layout fits the root's Auto inline size using
that offset and the actual ICB.

Text supplies the exact placed upright control and its retained Central font
basis. The browser projection uses that same instance's ordinary pre-leading
font height, split around Central. Ahem20 yields 10px to each side. Physical
line-over is right in both vertical writing modes. The projected bands have
width 20px and zero height, at (5, 30), (55, 30) and (55, 0) respectively.
[Chromium Central split](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/fonts/font_metrics.cc#L181-L188) ·
[Line-relative writing mode](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/text/writing_mode.h#L66-L78)

The [empty-line BR edition](captured/empty-line-br-font-band-qualification/)
captures the base stylesheet's embedded Ahem WOFF2. Browser capture checks the
only eligible loaded face, its descriptors and the encoded bytes. Text
prepares both ordered forced breaks using the same encoded resource through
the Font decoder. Each band binds to its own finalized control. The two
horizontal bands begin at block coordinates 5px and 35px. The two vertical-rl
bands begin at physical x coordinates 55px and 25px in the zero-inline-extent
root. The reader compares all four coordinates separately from parent geometry
and control relations. Unqualified structural observations retain their raw
coordinates and return an unsupported observation.

The injected prepared profiles use the byte-identical
[pinned TTF](fonts/PROVENANCE.md). The adapter hashes the captured
`fonts/Ahem.ttf` input before injecting those
same bytes as an explicit font-setup argument. Browser setup loads one explicit
FontFace from that exact byte buffer and checks nominal cmap coverage and loaded
status; the independent reader separately hashes
its retained bytes and prepares through Text `RegisteredOnly`. Glyph-free bidi
format controls have an explicit closed list. Hebrew, tabs and unsupported
shaping/style profiles are refusals, not fallback-font success. The browser's
family alias names this one resource; Text uses its real internal Ahem family.

Native control topology is checked against the exact immutable prepared control
roles and original occurrence mapping. Balanced metricless start/end controls
may enclose zero, one or multiple source occurrences, including nested
transparent containers. An empty container has no fabricated authored origin.
Mismatched, unbalanced or unbound controls refuse. Historical numeric-boundary
schema tests retain their separate original triplet/atomic constraints.

Marked transparent inline containers are admitted only with zero actual margin,
padding and border edges, baseline alignment, no transform/float/displacement,
and the same canonical font size and logical line height as their containing
element. The base CSS's unshifted relative positioning is admitted. The element's
own style is qualified even when empty. Independent struts, bidi/feature context
outside the source profile, and nontransparent boxes precisely refuse; they are
never replaced by invented zero metrics or discarded geometry.

Inputs never include measured glyph advances, resolved output bidi levels,
Range line groups, observed atomic dimensions, or estimated baselines. The first
atomic source profile requires actual authored pixel width/height and an empty
subtree. Percentage/auto/nested-content atomics need further capture of their
real Layout inputs. Layout computes the box allocation before handing that
allocation to Text. Current capture also refuses mixed containing wrap policy
around atomics rather than guessing common-ancestor facts.

Browser `Range.getClientRects()` output is a separate `rangeFragments` observation
channel; XML names it `<range-fragments><range-fragment …/>`. Its origin/line,
physical edge, start and advance are observations only. These rectangles are
neither font outline ink nor preparation inputs. Compositor/layout assertions
continue to compare actual independent production geometry.

Native Layout fragments carry actual Text-issued association handles and physical
geometry, not a visual ordinal derived from association-table order. Native
comparison uses those owners, source correspondence, line identity and actual
run/glyph/slot placement. The historical XML `visual_index` field remains a
required, validated metadata field in its old numeric schema; attempting to use
it as a native ordering assertion precisely refuses. It is never replaced with
another fragment enumeration.

Historical `inline-text` tuples are schema-only data. The
[prepared Text accounting](prepared-text-accounting.json) records the original
source and XML hashes for all 124 variants and identifies each current
successor. The generation report for each edition binds its browser artifacts
to their source and input digests.

Eight source-bound successors and twelve additional prepared representatives
have Native comparisons. Eighteen match the pinned browser. Two have the RTL
PDI difference described below. The separate BR metric fixtures use the font-band contract above.

The [prepared atomic percentage edition](captured/prepared-atomic-percentage-qualification/)
uses the original paragraph and its inline block's authored 20px width and
50% height. Layout resolves the 20px by 40px border box in the paragraph's
definite containing frame before Text prepares the source. The pinned Ahem
observations compare both boxes, both line identities and both projected
Range rectangles. The historical generic-font captures separately record the
same box sizes.

## Executed Range observation difference

The `prepared_text_continuity_bidi` border-box and content-box RTL variants
retain original PDI at aggregate UTF-16 `[4,5)`, the same pinned Ahem face, and
actual issued source/line/placement associations. Native selected-line L1 resets
the trailing PDI to the RTL paragraph level before canonical L2 ordering. Its
zero-width source slot is at physical x=24; Chrome's captured Range rectangle is
at x=72. Visible glyph positions and source coverage agree. The
[Native source-anchor policy](../../../../../docs/text-layout-contract.md#nonempty-text-range-fragments)
selects that actual post-L1/L2 position; the Native anchor decision is settled.
Neither coordinate is claimed uniquely required by CSS. Version 2 stores only
the observed inline edge/start/advance; it does not retain the browser rectangle's
height. Native source-fragment height must not be substituted for that observation.

The V4 prepared Range request binds the original HTML, its pinned TTF preload,
the effective loaded face, and the root's border frame. The four-variant
[receipt](captured/prepared-range-differences/raw-comparisons.jsonl) records
every box field and Range row. Each byte-pinned browser row ordinal binds to
ordered Text query units and piece occurrences. Qualification checks each
piece's finalized-line issuer and local and aggregate UTF-16 intersections
before grouping pieces or comparing coordinates. The two LTR controls match.
The RTL policy
accepts only the occurrence 1, line 0, zero-advance PDI start at browser x=72
and Native x=24. Qualification checks the actual Native node packets and
issued fragments in both phases, selected lines, glyph positions, effective
face, and placed PDI Range and caret. The retained V2 viewport cannot issue
a source-qualified root operation.

[CSSOM View §9](../../../../../references/cssom-view-1--WD-cssom-view-1-20250916--693b1c1fe7f7.md#dom-range-getclientrects)
defines Text-node Range rectangles using font metrics and advances, but does not
choose a unique invisible-format-control caret affinity. The native line rule
is [UAX#9 revision 51, L1/L2](https://www.unicode.org/reports/tr9/tr9-51.html#L1).
The executed Chrome tag's [trailing-item reset](https://github.com/chromium/chromium/blob/149.0.7827.115/third_party/blink/renderer/core/layout/inline/logical_line_builder.cc#L653)
and [trailing-space classification/split](https://github.com/chromium/chromium/blob/149.0.7827.115/third_party/blink/renderer/core/layout/inline/line_breaker.cc#L189)
retain a different item-level path for this PDI. Its
[Range entry](https://github.com/chromium/chromium/blob/149.0.7827.115/third_party/blink/renderer/core/dom/range.cc#L1532)
delegates to [actual text fragments](https://github.com/chromium/chromium/blob/149.0.7827.115/third_party/blink/renderer/core/layout/layout_text.cc#L516),
through the browser's item-level layout. Live DOM Range projection is tracked in
[#850](https://github.com/bj-data-eng/surgeist/issues/850) and
[#860](https://github.com/bj-data-eng/surgeist/issues/860).

The historical version 1 `rangeInks` JSON / `range-inks` XML spellings below remain
only as their declared historical parser contract. Current private models and
diagnostics call these **Range rectangles**, never glyph ink.

## Transport and evaluation

The adapter prepares one document and evaluates `getTestData()` once. The helper
returns a **JSON string**, whose object has exactly these five required fields:

| Field | Type | Measurement state / case suffix |
| --- | --- | --- |
| `schemaVersion` | Integer `4` | Version of this measurement contract |
| `borderBoxLtrData` | Node object | `border-box ltr` / `border_box_ltr` |
| `contentBoxLtrData` | Node object | `content-box ltr` / `content_box_ltr` |
| `borderBoxRtlData` | Node object | `border-box rtl` / `border_box_rtl` |
| `contentBoxRtlData` | Node object | `content-box rtl` / `content_box_rtl` |

Measurements occur in the table's order by changing `document.body.className`
and describing the same `#test-root` each time. For the Prepared profile,
the adapter explicitly loads its pinned face before measurement. It adds no
second document and derives no source inputs from measurements. A missing root
returns all four variants as `layoutInput: "unsupported"` nodes with
`unsupportedReason: "Unsupported missing #test-root fixture root"`; it performs
no measurements or body-class changes.

The helper is injected only if `getTestData` is missing. A fixture's preloaded
helper remains in effect and must implement version 4. Unversioned and unknown
versions are errors; there is no production legacy decoder or automatic helper
replacement. Preservation tests explicitly transform captured old measurements
into the historical version-1 wire shape offline. That test harness is not part
of the production decoder.

## Decoding and classification

The adapter first decodes the complete envelope into closed wire structures.
Object fields cannot be repeated or undeclared. Field types and discriminants
must match the declared alternatives; strings are not numbers, null is not an
empty object, and malformed tracks cannot be silently dropped.

Only after wire decoding succeeds does each variant select its first unsupported
reason: inspect the root, then visit children in their existing order, recursively
depth first. That reason classifies the entire variant. A measured box can carry
an unsupported reason; the explicit `unsupported` tag is the reason-only form.
Unsupported variants do not need complete supported-measurement geometry. A
malformed wire value still fails decoding even when another node
contains an unsupported reason.

Supported variants undergo semantic validation before XML writing. Validated
values distinguish boxes, inline text, inline boundaries, and line controls;
viewport context; observation forms; dimensions and tracks; and coupled inline
metrics. XML writing receives those values and formats them without discovering
malformed payloads or substituting fabricated zeroes.

Errors identify the affected case/variant when available, the node path, the
field path, and the error kind. JSON decoding errors retain their underlying
source. An invalid supported measurement is an adapter error, not an unsupported
outcome. The shared generator retains responsibility for fixture accounting and
failure-atomic publication.

## Field conventions and alternatives

JavaScript `undefined` properties are omitted by `JSON.stringify`. Optional
fields describe absence, rather than an instruction to infer another observed
value. Optional object fields reject explicit null; a null grid-area cell is a
separate semantic value. Explicit empty observations are preserved.

| Fields or structure | Meaning of omission, null, or an empty value |
| --- | --- |
| Envelope version and four variants | Required; omission and null are errors. |
| `layoutInput` | Required explicit tag: `box`, `prepared-text`, `inline-text` (historical schema only), `inline-boundary`, or `unsupported`. No node kind is inferred from a missing tag. |
| `unsupportedReason` | Optional classification on a measured node; required by the reason-only `unsupported` tag. Classification uses the first reason after wire validation. |
| `children` | Required ordered array on supported nodes; `[]` means no children. Unsupported reason-only nodes omit it. |
| `worldRoundedLayout` | Required on every box; absent on non-box alternatives. Complete finite x/y/width/height, with optional paired scroll/client extents; retained only as separate world-phase diagnostics. |
| Optional `style` fields | Omission means no explicit fixture declaration. XML default-value omission remains a serialization rule; the decoder does not fabricate an observed value. |
| `style.inlineMetrics` | Optional numeric object with required `baseline` and `lineHeight`. Omit the object when metrics are absent. Empty strings, individual metric fields, and null are invalid. |
| `size`, `minSize`, `maxSize`, `gap`, edge groups | Optional structured dimensions; missing axes or edges remain absent. The helper omits an entire group when none of its entries is present. |
| `gridTemplateAreas` | Optional nonempty rows of area-name identifiers or null cells. Rows have equal width, and every repeated name fills one rectangle. A null cell is the empty grid-area token `.`; it is not an omitted row. |
| `fragments`, `rangeInks` | Absence means no observation of that category. `[]` records an explicitly empty observation and produces `<fragments/>` or `<range-inks/>`. The categories cannot coexist on one node. |
| `shapeBands` | When present, a nonempty ordered table. Each query has required `bandMinimum` and `bandMaximum`, and optionally `interval: { minimum, maximum }`. Both interval endpoints are required when the interval exists. Flat endpoints and null are invalid. |
| `inlineBoundary` | A `start` or `end` descriptor. A start has either no metrics or both `baseline` and `lineHeight`; an end has no metrics. Boundaries have no layout observation or child payload. |
| `replacementInlineExtent` | Present only with `followingBreak: "allowed-with-replacement"`, where it is required. Other break kinds cannot carry replacement data. |
| Grid reference profile | Every supported source-qualified root requires `gridAlignmentProfile` profile 1 and its complete owner paths, including an explicit empty list. `gridAlignmentReference` is a separate own-Grid typography record, never a paragraph strut. |
| Layout-ready marker fields | When present, `layoutReadyInlineRoot` and `layoutReadyAnonymousGridTextWrapper` are exactly `true`. Absence means no marker. |

An ordinary box uses geometry observations and may include explicit fragments.
Inline text has at least one complete `inlineSegments` entry and may use either
geometry/fragment observations or Range rectangle observations. Range rectangle text has no
ordinary node geometry, scroll state, fragments, or child expectations. An empty
Range rectangle array remains a valid observation of significant collapsed whitespace.

Each text segment carries its source ID, inline extent, baseline, line height,
bidi level, whitespace-edge behavior, and following-break behavior. Whitespace
edges are `preserve`, `discard-at-start`, `discard-at-end`, or `discard-at-both`.
Breaks are `prohibited`, `allowed`, `allowed-with-replacement`, or `mandatory`.
Atomic participation carries the same applicable bidi/break information and
refers to an actual box child. It cannot use a text replacement break. Explicit
line-control participation is `forced-break` on an inline `br`; a block `br`
remains a box.

The supported root requires an explicit `useRounding` boolean and the closed
version-4 `viewport.rootContext` described above. Child viewport data cannot
replace root authority. Historical width/height and root/flex-item tags remain
opaque archival metadata; neither intrinsic keywords nor observed host extents
are converted into a new operation.

Dimensions use an explicit `unit`: numeric `px`, `percent`, or `fraction`;
symbolic `calc` or `sizing` strings; or keyword-only `auto`, `none`, `content`,
`max-content`, `min-content`, `stretch`, `fit-content`, or `contain`.
Percent values are fractions, so `0.5` writes `50%`. Symbolic sizing strings keep
their existing spelling; the adapter does not add a CSS parser.
Structured units are constrained by their property. Margins and insets allow
signed lengths or `auto`; padding, border, and gaps require nonnegative lengths.
Preferred/minimum sizes accept lengths and their sizing keywords; maximum sizes
use `none` instead of `auto`. Only flex basis admits `content`, and fractions are
track maxima, not box dimensions. Numeric font, line-height, flex-basis, size, and
track extents must be nonnegative for supported measurements. Symbolic strings
remain opaque and do not gain an authored-CSS validation path.

Captured style discriminants are closed, including alignment, float/clear,
text alignment, and vertical alignment. Unknown values fail wire validation
before unsupported classification. Alignment uses the fixture consumer's
explicit keywords and supported `safe`/`unsafe` forms.

Track alternatives have explicit `kind` tags: scalar dimensions, line-name
lists, subgrid line-name lists, and function forms. A scalar track must include
`kind: "scalar"`; a subgrid must include `lineNames`, using `[]` for no names.
`fit-content` takes one length limit; `minmax` takes a non-fractional minimum and
a maximum that may be fractional; `repeat` takes a repetition and at least one track. A
repetition is a positive integer, `auto-fill`, or `auto-fit`. Grid placements are
`auto`, numeric lines/spans, or named lines/spans. A named placement has a name
and optional `occurrence`. Omission selects the unindexed named form; an explicit
line occurrence is signed and nonzero, and an explicit span occurrence is
positive. A zero occurrence and a named-placement `value` field are invalid.

## Numeric and XML preservation

Browser numbers remain `f64` during decoding, validation, and lowering.
Coordinates and interval endpoints must be finite; extents must also be
nonnegative. Coupled baseline and line-height values satisfy
`0 <= baseline <= lineHeight`. Zero-height text/control metrics remain legal;
a metric-bearing start boundary has the stricter `lineHeight > 0` rule. Shape
queries and optional intervals have ordered endpoints. Integer identifiers and
indices are bounded to their consuming fixture types; supplied resolved bidi
levels are `0..=126` (explicit depth 125 is a different phase).
Supported text and style strings must contain only XML 1.0 characters. Tabs,
line feeds, carriage returns, and legal Unicode remain unchanged; forbidden
control characters fail at their original node and field location. This check
does not parse symbolic sizing expressions or other opaque style strings.

Only selected ordinary layout geometry crosses the existing `f32` formatting
boundary, after representability has been checked. Segment metrics, fragments,
Range rectangles,
and dimensions retain their existing `f64` formatting. The root's `useRounding`
chooses smart-rounded or unrounded geometry for the tree. Root `x` and `y` write
as zero; child geometry stays relative to its parent. Scroll extents use the
existing nonnegative difference from naive client measurements.

World-rounded diagnostics retain f64 precision independently of selected ordinary
geometry formatting. They serialize under the separate `diagnostics` tree
with checked node paths. The wrapping qualification reads its designated
browser operand from that tree after capture authentication.

Source segment IDs retain their helper-provided values. Atomic placeholder
indices count lowered input children, including inline boundaries. Expectation
children omit those boundaries; browser-control observation indices refer to
that filtered expectation order. Range line indices retain the helper's
containing-root line registry. No protocol step renumbers these identities.

Supported captured font families, font sizes and numeric line heights serialize
explicitly. The fixture reader inherits omitted typography from the parent.

XML attribute order, field-specific default omission, whitespace, escaping,
text trimming, and formatting are preservation contracts. The fixture reader in `support.rs`
remains an independent consumer: it parses generated XML and compares layout
without importing private measurement types. Versioned recapture retains original
XML bytes and exact raw comparisons in history. Fresh metadata/diagnostics come
from authenticated current inputs and are not patched into old artifacts.
Accounting and each fixture's comparison outcome remain independently visible.
