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

The authored `font-palette` longhand retains keywords, case-sensitive palette
names and recursive ordered `palette-mix()` values. Checked construction and
bounded specified serialization reuse the color interpolation and percentage
providers; palette lookup and color mixing remain downstream. See the
[palette reference](docs/reference.md#authored-font-palette) and
[executable example](examples/font_palette.rs).

## Start

```rust
use surgeist_css::{CssRecoveryAction, parse_sheet};

let report = parse_sheet(
    ".before { color: red; } @unknown value; .after { color: blue; }",
);
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

## License and attribution

See the [MIT license](LICENSE) and [third-party notices](NOTICE.md).
