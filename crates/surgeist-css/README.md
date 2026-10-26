# surgeist-css

`surgeist-css` is a Rust library for Surgeist consumers that need typed authored
CSS with browser-style recovery and structured diagnostics. It parses
stylesheets and style attributes while preserving valid siblings around malformed
or unsupported input. The current surface includes partial and recognized
unsupported productions; consult the [support reference](docs/reference.md#conformance-sources-and-atomic-records)
for exact boundaries. Cascade, substitution, matching, resource loading, and
layout belong to downstream consumers.

`CssParserContext::with_svg_glyph_orientation_vertical()` explicitly selects
the independent frozen SVG authored definition while ordinary fronts retain
the finite Writing Modes shorthand. The SVG body preserves Auto, Angle-root
values or finite implied-degree Number literals, actual document mode and
presentation-attribute admission through checked construction, pending reentry,
contributions and bounded specified output. Fixed attribute-value methods parse
Normal-importance values; root owns markup binding. Explicit finite grammar
handles keep their finite meaning. Unitless canonical output becomes degrees
and uses the shared rounding policy; computation and quadrant rounding remain
downstream. See the [glyph reference](docs/reference.md#independent-svg-glyph-definition)
for precedence, sources and unresolved normative clauses.

Byte stylesheets enter through `CssDecodedStylesheetInput::decode(bytes, hints, limits)` and
`CssParser::parse_sheet_bytes(&input)`, with `with_location` attaching the optional location. Decoding uses standard
Encoding labels, CSS fallback precedence and BOM overrides, with optional atomic
byte limits. The immutable input retains original bytes and the exact unfiltered
UTF-8 snapshot used by parsing. The stylesheet location is host-supplied symbolic
text; see the [entry-point reference](docs/reference.md#entry-points-and-features).

`CssParser::parse_sheet_utf16` accepts shared original UTF16,
uses the general CSS Syntax input cursor for newline/NUL/surrogate preprocessing,
and invokes the ordinary stylesheet grammar in the supplied complete context.
Finite preparation limits cover original units, normalized bytes, map entries,
position checkpoints, fixed decoder/checkpoint registers, retained backing and
work, including actual buffer movement during growth. A fallible callback is
polled during preparation and immediately before
and after grammar execution. The grammar retains its existing resource limits;
it does not provide fine-grained cancellation or a whole-grammar work budget.
Failure returns no partial report.

`CssMediaQueryList::parse_utf16` uses the same private
prepared-source owner and ordinary media-query grammar. `CssUtf16InputLimits`
and `CssUtf16InputError` apply to both real UTF16 grammar frontends; the UTF8
media-query function remains its actual input format, with a context method for
an explicit complete parser context. Parsed media lists retain a complete
`CssParseSource`, including empty/comment-only inputs and recovered members;
constructed lists remain unqualified. Diagnostics share the actual immutable
source and normalized coordinates. Media semantic equality excludes the added
parse attestation; source identity and full-context admission remain separate.

Every actual stylesheet parse retains a root `CssParseSource`, including
empty and fully recovered inputs. Constructed sheets have no parse attestation.
Root sheets, parsed origins and stylesheet recovery diagnostics share the actual
immutable `CssSourceSnapshot`. UTF16 snapshots additionally retain the original
shared units and exact normalized-scalar-boundary to original-unit mapping.
Diagnostic coordinates continue to address normalized parser UTF8, with original
unit intervals queried separately. Old rules and diagnostics keep their source
after a consumer replaces an association. Source and syntax equality keep their
existing text/semantic meaning; `same_snapshot` separately compares identity.

`CssSourceStorageUsage` accounts for actual String/checkpoint/UTF16/map backing
capacities. `CssSourceRetention` traverses retained CSS payloads under finite
visits/depth and accumulates each actual source identity once. It inspects stored
provenance independently of formatting, resource execution or Arc strong counts.
Source storage admission is separate from input parsing admission and caller-owned
historical captures.

`CssParser::parse_block_contents_utf16` admits
original UTF16 through that same preprocessing boundary and the existing raw
declaration-block grammar. Its sealed `CssDeclarationBlockParseReport` retains
the complete root source even for empty or wholly recovered input. Values and
diagnostics share that source, with byte coordinates in normalized UTF8 and a
separate original-unit map. No surrounding sheet or CSSOM winner selection is
introduced. Preparation has finite input/output/backing/work limits and polls;
the declaration arena separately admits the entire normalized input under its
cumulative component, depth and byte limits, including text after a root `}`.
General grammar execution remains opaque between pre/post polls. These limits
do not promise per-action grammar work or live-allocation accounting. Consumers
admit retained sources independently; a source receipt is not a complete report
or checked-value storage receipt.

`CssColorScheme::parse_utf16` supplies the independent
bounded authored `color-scheme` value front. It preserves the shared original
UTF16 and its preprocessed source through the existing component and property
grammars, including arbitrary substitution fallbacks. Concrete values, CSS-wide
keywords and substitution-dependent declarations retain their authored meaning;
the front does not perform cascade or substitution.

`CssUtf16PropertyValueLimits` combines preparation and component syntax limits
with whole-operation work, simultaneous live payload and retained-result bounds.
The caller's fallible admission callback receives work and prospective or actual
live storage observations before processing continues. A separate infallible
release callback observes storage only after it is dropped or transferred. This
lets a consumer compose its existing resource owner without reserving the whole
allowance or treating an admission failure as invalid CSS. Success retains the
report's exact source-inclusive storage receipt; failure returns no report and
releases its temporary custody.

The immutable `CssColorSchemeParseReport` couples the original parse source with
checked syntax success or rejection. Resource and caller interruption failures
remain outside that syntax result. Its meaningful equivalence query compares
original input, parser context and checked contents under caller admission;
equal normalized text alone does not establish equal UTF16 provenance, and fresh
parse identities alone do not make the content different. These guarantees are
specific to this value front; the stylesheet and media preparation contracts
above retain their own documented grammar limits.

Processing-result consumers can separately observe diagnostic-owned out-of-line
backing with `CssRecoveryDiagnostic::retained_backing_bytes_with`. It includes
actual String/Vec capacities and boxed payloads, excludes inline diagnostic
layout and source handles, and recomputes after clones. Caller admission precedes
each reached observation; errors distinguish admission refusal and byte overflow.
`CssSourceRetention::complete_payload_bytes_with` observes the same retained source
identity bank, adding actual source payload headers and original UTF16 Vec headers
to backing. Exact original UTF16 Arc aliases count once across distinct snapshots;
equal contents in different owners count separately. Existing backing-only source
quotas and visit tariffs are unchanged. The complete observation excludes the
collector's bank, opaque Arc control blocks and allocator overhead; it allocates
nothing and admits each reached source observation and identity comparison.

Authored Shapes 1 retains `shape-outside` with images, the eight shared basic
shapes and four reference boxes; `shape-image-threshold` keeps unrestricted
number/percentage values; `shape-margin` keeps exact nonnegative literals and
deferred length-percentage math. Checked models preserve omitted versus explicit
reference boxes through expansion, strict reentry and bounded specified output.
See the [Shapes reference](docs/reference.md#authored-shapes-properties).

Authored Motion Path retains the five offset longhands and `offset` shorthand,
including ray bearings, URLs, all eight shared basic shapes and the six imported
coordinate boxes. Checked models preserve optional constituents; expansion uses
central initials for omissions. Positions, paths, anchors and rotation remain
symbolic through strict reentry, normalization and bounded specified output.
See the [Motion reference](docs/reference.md#authored-motion-path).

`@font-face` types named-instance (`auto` or a string), language override
(`normal` or a string), and ascent/descent/line-gap overrides (`normal` or a
nonnegative percentage with checked math). Parsed occurrence order, recovery,
provenance and pending `env()` reentry use the shared descriptor lifecycle.
Their payloads provide bounded specified serialization; font matching and
metric execution belong downstream. See the
[descriptor reference](docs/reference.md#named-instance-language-and-metric-descriptors)
and [consumer example](examples/font_face_metrics.rs).

CSS Speech `pause-before`, `pause-after`, `rest-before`, and `rest-after` retain
`none`, symbolic break strengths, or exact nonnegative ordinary times with
checked time calculations. `pause` and `rest` retain one or two ordered values
and intrinsically expand them to before/after longhands. Checked composition
and bounded specified serialization preserve authored omissions and provenance;
canonical pair output omits a demonstrably equivalent second value. Strength
durations, pause collapse, and additive rest execution belong downstream.

CSS Speech `voice-family` retains prioritized quoted or identifier names,
ordered generic voices with optional exact positive variant indices, and the
whole-value `preserve` alternative. `voice-stress` retains its five symbolic
keywords. Checked models and bounded specified serialization use the shared
identifier, string and integer owners. Contextual voice selection and acoustic
realization belong downstream; see the [voice reference](docs/reference.md#authored-speech-voices).

CSS Speech `cue-before` and `cue-after` retain `none` or a shared authored URL
with an optional exact signed ordinary dB offset. `cue` retains one or two ordered
cues. Checked construction, expansion and bounded specified serialization preserve
authored omissions and token origins. Resource loading and contextual volume
resolution belong downstream; see the [cue reference](docs/reference.md#authored-speech-cues).

CSS Speech `voice-balance` retains five keywords or an unrestricted exact Number
value with pure Number math. `voice-volume` retains silent, a calibrated level
with optional signed dB offset, or an offset alone. Canonical output preserves
the meaning of an omitted level; computed clamping, inheritance and acoustic
calibration belong downstream. See the [mixing reference](docs/reference.md#authored-speech-mixing).

CSS Speech `voice-duration`, `voice-pitch`, `voice-range` and `voice-rate`
retain authored prosody independently of voice execution. Duration uses the
shared nonnegative ordinary time model; pitch/range separate positive ordinary
absolute frequency from signed relative frequency, percentage and local `st`
offsets. Typed math retains its specified phase, including relative
frequency-percentage calculations against an unresolved inherited frequency.
Checked composition and cumulative serialization preserve authored omissions;
see the [prosody reference](docs/reference.md#authored-speech-prosody) for
canonical output and precision limits.

Authored UI4 values include Outline with reusable one-dimensional stripes,
directional navigation, caret, interactivity, interest delays, accent color and
appearance. Typed models share intrinsic expansion and pending reentry;
canonical providers preserve optional fields and symbolic legacy navigation
diagnostics under cumulative limits. Focus, caret, timing and widget execution
remain downstream. See the [UI reference](docs/reference.md#remaining-authored-ui4-values)
and [stripe import](docs/reference.md#imported-one-dimensional-stripes).

The eight four-side border, inset, spacing and scroll shorthands expand to four
members in their authored physical or logical mode without complementary resets.
CSS-wide values select physical sides; strict substitution reentry selects the
replacement mode. Mode-aware metadata and normalization preserve occurrence
identity and cumulative limits. See the
[membership policy](docs/reference.md#four-side-shorthand-membership) for the
selected draft's source limit and the bounded compatibility decision.

Authored `@when` and `@else` retain checked symbolic conditions and ordered ordinary,
scoped and style-body children. Original-token gaps govern parsed Else chains;
complete checked assembly validates supplied adjacency and style ancestry.
Normalization retains branch contexts and shared specified output uses cumulative
limits. See [authored when and else groups](docs/reference.md#authored-when-and-else-groups)
for the adopted source profile and literal CSSOM availability.

Authored `@supports` and import conditions classify `font-tech()`,
`font-format()` and `at-rule()` predicates through the same checked grammar.
They reuse the font vocabulary, preserve format strings and actual at-keyword
arguments, and remain symbolic through lexical and specified output. See
[supports predicates](docs/reference.md#font-and-at-rule-supports-predicates)
for exact argument admission and opaque fallback.

Selectors canonicalize `:-webkit-autofill` to the standard Autofill identity and
retain nonfunctional unknown `::-webkit-*` names as checked symbolic match-nothing
pseudos. The Pseudo 4 vocabulary includes checked Highlight names, FirstLetter
Prefix/Suffix children, SearchText's nonfunctional Current suffix and the
Placeholder, FileSelectorButton and DetailsContent identities. The shared grammar
preserves originating compounds, ordered suffix attachments, explicit child
permissions and cumulative specified output. See
[pseudo-element attachment](docs/reference.md#pseudo-element-attachment-and-aliases)
for the admitted matrix, required aliases and the draft's at-risk suffix status.

Checked `CssBackgroundLayer` and `CssBackground` construction retains authored
omissions and enforces size/position and final-color constraints. Intrinsic
`background` expansion supplies all eight longhands with per-layer schema
initials. Its bounded specified serializer composes canonical children under
one cumulative resource policy, preserving symbolic values and counting omitted
authored nodes without charging discarded output bytes.

Checked `CssBorderImage` construction retains its five optional components and
requires a slice before width or outset. Intrinsic expansion supplies the five
ordered longhands from authored values or schema initials. Bounded specified
serialization compresses all four stored edges, preserves numeric branches and
symbolic calculations, and counts every child even when its initial is omitted.

Authored CSS Masking retains all eight mask layer fields, independent comma-list
arity and the six mask-border resets. Checked `CssMaskLayer` and `CssMaskBorder`
construction composes the shared image, position, size, repeat and nonnegative
four-side owners. Masking supplies its own six-box domain, trailing slice fill,
optional slash width, initials and ordered expansion. Clip-rule and mask-type
retain their distinct SVG keyword contracts. Bounded specified serialization,
strict replacement reentry and normalization preserve source occurrences and
symbolic values; see [authored Masking](docs/reference.md#authored-masking).

Checked `CssShadow` construction requires blur before spread; `CssBoxShadowList`
is nonempty. Box shadows retain signed offsets and spread, optional nonnegative
blur, optional color, and inset. `CssDropShadow` instead retains an optional
nonnegative standard deviation and cannot contain inset or spread. Lengths form
one contiguous grammar group. Box-shadow supplies a noninherited terminal
contribution with `none` initial. Bounded specified serializers on all four
shadow models preserve authored omissions, explicit zero and currentcolor,
list order, exact numeric literals, and symbolic math and colors.

`CssFilter` retains `none` or an ordered nonempty function/URL list. Both `filter`
and the named `backdrop-filter` exception are noninherited terminals with `none`
initials. `CssFilterHueRotate` preserves an omitted angle separately from its
effective `0deg`, unitless zero, and explicit angles. Bounded specified serializers
on the value, list, and function compose shared numeric, color, URL and shadow
providers under one cumulative budget without clamping amounts or resolving
symbolic values. The backdrop source selects only the named authored property,
not the complete Filter Effects 2 module or its execution behavior.

The four authored SVG filter declarations have complete CSS grammar support.
`flood-color` and `lighting-color` reuse the complete symbolic `CssColor` owner;
`flood-opacity` reuses exact `CssOpacityValue`, retaining negative and above-one
authored values. `CssColorInterpolationFilters` keeps auto, sRGB and linearRGB
distinct and emits lowercase specified keywords. Their intrinsic initials are
black, 1, white and linearRGB; only color-interpolation-filters inherits.
SVG attributes, computed clamping, applicability and filter execution remain
downstream. See the [property reference](docs/reference.md#authored-svg-filter-color-properties).

The authored `font-synthesis` shorthand accepts nonrepeated `weight`, `style`,
`small-caps`, and `position` capabilities in any order, or `none`. It contributes
four inherited longhands, each initially `auto`; selected capabilities become
`auto` and omitted capabilities become `none`. The four longhands have distinct
checked keyword models, including `oblique-only` for style. Bounded specified
serialization emits canonical grammar order. See the
[synthesis reference](docs/reference.md#authored-font-synthesis) and
[executable example](examples/font_synthesis.rs).

The authored View Transitions 1 surface retains `view-transition-name` and the
root, Group, ImagePair, Old and New pseudo-elements. Checked property names
exclude `none` and `auto`; functional selector arguments retain a wildcard or
custom identifier. Named descendants admit `:only-child` and the defined compound
tree edges. See the [View Transitions reference](docs/reference.md#authored-view-transitions).

The authored `font-palette` longhand retains keywords, case-sensitive palette
names and recursive ordered `palette-mix()` values. Checked construction and
bounded specified serialization reuse the color interpolation and percentage
providers; palette lookup and color mixing remain downstream. See the
[palette reference](docs/reference.md#authored-font-palette) and
[executable example](examples/font_palette.rs).

## Standalone used colors

Use `parse_used_color` to resolve a standalone color over an original borrowed
UTF-8 or UTF-16 input and explicit immutable `CssColorFacts`. Select
`CssColorGrammarRequest::WholeColor` for a complete value or `PrefixColor` for a
color prefix; the result retains the actual input, fact envelope and consumed
original-unit range. `CssColorOrdinaryOutcome` contains a
[`surgeist_color::Color`](../surgeist-color/README.md) or a
`CssColorStandardFailure`. Parsing creates no output. Its explicit
`CssColorOrdinaryLimits` bound source units, logical parse-bank records and
actions; they do not bound allocator capacity or replace a consumer's resource
controller. For a native numerical value, use Color's `serialize` with
`SerializationRequest` and `SerializationLimits`. See the
[public operation examples](tests/css/color/used_color_operations.rs).

The shared grammar recognizes all 21 retained color branches: currentcolor,
transparent, hex, named and system colors; rgb, hsl, hwb, lab, lch, oklab and
oklch; predefined and custom `color()`; relative custom colors, `alpha()` and
other relative colors; `color-mix()`, `light-dark()`, `contrast-color()` and
`device-cmyk()`. Absolute and relative predefined-space evaluation, numeric
expressions, missing-channel handling and mixing resolve CSS semantics before
or around Color's numerical operations. The ordinary authored parser also uses
this grammar through its actual Parsed and Components inputs, preserving
symbolic `CssColor` values and origins.

For an existing checked `CssColor`, use `resolve_authored_color` with the same
explicit facts. Its `CssAuthoredColorResult` borrows the original value and facts;
the outcome contains a numerical color or the actual standard-failure cause.
`CssAuthoredColorOrdinaryLimits` bounds the three logical record banks, path
steps and actions. Internally a crate-private controlled operation services the
same resolver with local banks and a path buffer, preserving Parsed,
Programmatic and ImplicitClosure origins without serialization or reparsing.
Colour numerics are total, so resolution has no numerical error. See the
[public authored-resolution examples](tests/css/color/color_authored_resolution.rs).

Resolution observes only reached dependencies. The no-element
`CssColorPropertyFacts::InitialProperties` branch resolves currentcolor through
CanvasText and therefore still needs the actual palette entry. Supplied current
color, system palette, effective light/dark scheme, profile metadata and
applicable profile/device conversions are explicit facts, joined to the expected
`CssColorFactIdentity` and their resource identities/revisions. The consumer
owns the join to its real capture and resources. Profile `parameter_count` is
independent of optional ordered component names. Only the selected
`light-dark()` arm needs contextual facts, although both arms must have valid
syntax. Known absent/invalid profile or device conditions retain their selected
standard recovery; unavailable facts do not invent a fallback. Custom-profile
forward conversions require applicable checked conversion records. Reached
reverse-profile and custom-interpolation conversions have explicit unsupported
boundaries where the fact carrier cannot represent the operation. Contextual
numeric units likewise require a supported basis. CSS performs no resource
loading or contextual style calculation.

The selected `contrast-color()` policy resolves the CSS input and delegates the
numerical black/white comparison to Color. It converts to XYZ D65 Y, ignores
alpha, clamps finite Y to [0, 1], and compares the black and white luminance
ratios; white wins a tie. This is Surgeist's selected UA policy. Its source
selection and capability boundaries are recorded in the
[Color delegation reference](../../references/SOURCE-EDITIONS.md#html-color-delegation-and-intrinsic-contrast-policy).

Color owns finite signed channels, missing channels/alpha, serialization
families, conversion, interpolation, transformations and resolved serialization.
Consumers import its numerical types directly. Ordered `Transformation`
requests make conversion, opaque alpha and sRGB quantization explicit;
equivalent coordinates do not establish byte provenance. Absolute
sRGB/HSL/HWB values can serialize as legacy rgb/rgba while equivalent relative
or mixed values preserve modern `color(srgb ...)` serialization. Missing
components retain `none` where required. The HTML-compatible request emits
lowercase hex only for an opaque value with established sRGB bytes. Color also
owns the shared incremental finite formatter and byte-alpha rounding policy
(236 serializes as 0.925; 237 as 0.93), separately from floating alpha; the
pinned nonnormative byte-236 example disagrees with that algorithm. CSS retains
authored exact-decimal formatting and specified serialization, preserving their
existing precision contract.

For a consumer with an existing controller, `CssColorParse::start` begins one
typestate parse. Each `step` consumes the parse and yields either one typed
`CssColorParseAction` or the completed `CssColorParseOutcome`. The consumer
admits and polls before servicing each source read, fact read, bank read/write
or fixed compute using the action's actual `CssColorActionLayout` and compiled
register sizes; each action exposes only its own completion, which returns the
parse. It owns the Algorithm, Arithmetic and Evaluation vectors and their real
capacity, copy and release events. A write mints a `CssColorSlot` and a receipt
whose `stored` acknowledges the actual storage. Lending a record of another
parse, bank or index is a defect in the driving code and panics, as slice
indexing does. `into_parse` declines an action without effect; dropping a parse
or an action abandons it and does not dispose caller storage. Records borrow
the exclusive `CssColorOperationOwner`, so it starts another parse only after
they are dropped; the outcome borrows no owner. The anchor is not a second
resource controller.

Retire actual CSS scratch before or after transforming a used checkpoint; bank
disposal is the consumer's policy. Keep a separate
`surgeist_color::OperationOwner` stack local for the numerical continuation.
`CssUsedColorCheckpoint::transform` consumes the checkpoint and lends that
anchor to Color's computation. CSS does not wrap serialization: the consumer serializes
`checkpoint.value()` with `surgeist_color::Serialization` directly and keeps the
checkpoint's `standard_failure()` and `source()` for publication. That
serializer's own typed actions lend a fourth, consumer-owned bank of Color
`DigitSlot` records, distinct from the three CSS parse banks, and emit one
admitted unit at a time. Digit records borrow the numerical anchor, so it
cannot be reused while any of them lives.

A `CssColorTransform` is total: `compute` makes the transformed value the
checkpoint's value, and `into_checkpoint` declines it and returns the original
checkpoint, including source, facts and standard-failure context. Declining does
not complete pending work or restart a cancelled controller. A standard-failure
checkpoint can accept an explicitly checked caller fallback while retaining that
context. Unavailable, unsupported, malformed-fact and malformed-input
`CssColorError`s remain distinct from standard invalidity; the parse's
compile-time and panic misuse contracts are listed on `CssColorParse`. A channel
value beyond binary64, including a top-level infinite
calculation, is the closest supported finite value, so no numerical error
exists. Resource
refusal and cancellation belong to the original controlled consumer. See the
[controlled protocol](src/color/bounded.rs) and [fact carriers](src/color/facts.rs).

The current [HTML adapter](../surgeist-html-parser/src/construction/input_color.rs)
uses the original UTF-16 source and its original parser controller with both
owners and the separate digit bank. HTML owns dirty/default/current selection,
standard-failure-only black fallback, transformation order and atomic publication
after serialization and cleanup. Its `HtmlColorProcessingError` projects the
source-only CSS error into fixed first-stop metadata: a numerical range is
relative to the selected color field, while the original HTML source binding
remains separate. This does not establish a complete contextual CSSOM/style
consumer or close [#805](https://github.com/bj-data-eng/surgeist/issues/805)'s
contextual and quirky-color/CSS.supports scope.

## Start

```rust
use surgeist_css::{CssRecoveryAction, CssParser};

let report = CssParser::default().parse_sheet(".before { color: red; } @unknown value; .after { color: blue; }");
assert_eq!(report.syntax().rules().len(), 2);
assert_eq!(report.diagnostics().len(), 1);
assert_eq!(report.diagnostics()[0].action(), CssRecoveryAction::DropAtRule);
```

The two style rules survive, and the unknown at-rule produces one diagnostic.
See [getting started](docs/getting-started.md) for local setup and verification.

## Documentation

- [Getting started](docs/getting-started.md): parse your first stylesheet.
- [How-to](docs/how-to.md): inspect declarations, handle recovery, and require clean input.
- [Reference](docs/reference.md): public interfaces, authored grammar families, and support metadata.
- [Explanation](docs/explanation.md): recovery, symbolic values, compatibility, and ownership.

Checked Fonts rule assembly and effective specified serialization preserve ordered
authored occurrences and source provenance. See the
[composition reference](docs/reference.md#checked-fonts-rule-composition) for
placement, namespace, nesting and cumulative resource contracts.

The [authored CSSOM declaration block](docs/reference.md#authored-cssom-declaration-blocks)
projects retained occurrences into specified order and reconstructs exact
shorthands under cumulative limits. It preserves original provenance and exposes
the pending-footprint boundary before physical or logical reentry is established.

The [literal CSSOM rule and sheet operations](docs/reference.md#authored-literal-cssom-rule-and-sheet-output)
compose selected rule formats with normalized declaration runs, namespace-safe
selector shortening and typed limits for formats without an adopted wrapper.
The [borrowed current graph input](docs/reference.md#borrowed-current-rule-graph-inputs)
assembles edited typed fields without preformatting descendants, then admits and
formats the complete request under one native cumulative allowance.

## License and attribution

See the [MIT license](LICENSE) and [third-party notices](NOTICE.md).
