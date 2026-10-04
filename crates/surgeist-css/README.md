# surgeist-css

`surgeist-css` is a Rust library for Surgeist consumers that need typed authored
CSS with browser-style recovery and structured diagnostics. It parses
stylesheets and style attributes while preserving valid siblings around malformed
or unsupported input. The current surface includes partial and recognized
unsupported productions; consult the [support reference](docs/reference.md#conformance-sources-and-atomic-records)
for exact boundaries. Cascade, substitution, matching, resource loading, and
layout belong to downstream consumers.

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

## License and attribution

See the [MIT license](LICENSE) and [third-party notices](NOTICE.md).
