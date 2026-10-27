# surgeist-color

`surgeist-color` supplies checked numerical colors, conversion, interpolation,
transformations and resolved serialization without a CSS parser or contextual
facts. It has no Cargo dependencies and uses the standard library. CSS owns
authored syntax and resolution; consumers select transformation order and
publication policy.

## Native use

```rust
use surgeist_color::{Color, SerializationLimits, SerializationRequest, serialize};

let red = Color::from_srgba8([255, 0, 0, 255]);
let text = serialize(red, SerializationRequest::HtmlCompatible,
    SerializationLimits { actions: 100_000, digits: 2048, output_bytes: 4096 })?;
assert_eq!(text, "#ff0000");
# Ok::<(), surgeist_color::SerializationError>(())
```

The [native example](examples/native.rs) also constructs extended coordinates,
converts, interpolates and transforms them. It imports only this crate:

```sh
cargo run --offline --locked -j 1 -p surgeist-color --example native
```

`Color::new` takes proof-typed operands: `Finite` channels and `UnitInterval`
alpha, which reject NaN and both infinities when they are built. It preserves
missing components and extended finite channels and does not infer byte
provenance from integral floating-point values. `from_srgba8` establishes
actual channel and alpha byte provenance; an explicit `QuantizeSrgb`
transformation establishes channel bytes without inventing alpha bytes.
`identical` compares representation, including missing values, signed zero,
byte provenance and serialization family. Ordinary equality compares the
stored value using floating-point equality.

`ColorSpace` covers predefined RGB/XYZ spaces, HSL/HWB, Lab/LCH and
Oklab/Oklch. `RectangularCoordinates`, `HslCoordinates`, `HwbCoordinates` and
`PolarCoordinates` hold `Finite` coordinates for the corresponding numerical
conversion APIs. Every operation is total: conversion preserves finite extended
coordinates, and a result beyond binary64 is the signed closest finite value
(the CSS Values 4 closest supported value). This is a numeric-range rule only;
no gamut map is applied. The implementation uses
binary64 significand arithmetic, not exact-real or correctly rounded arithmetic.
Source-specific numerical policies remain documented in the conversion modules.

`SerializationFamily` selects the resolved spelling independently of authored
text. `SerializationRequest::HtmlCompatible` uses hexadecimal output only when
the value has actual sRGB byte channels and opaque alpha. Other values retain
their selected resolved color form. The shared `format` module owns finite
formatting and rounding, including non-color CSS consumers;
`byte_alpha_for_serialization` owns the byte-alpha spelling policy.

## Controlled operations

`Computation`, `Serialization` and `format::FiniteFormatter` are typestates.
Each transition consumes its continuation and returns the next state, so a
used, completed or abandoned operation cannot be driven again. A computation
exposes its kernel and layout; `compute` returns the total result and
`into_request` declines it. `step` on a serialization or formatter yields
exactly one pending action, or the completed `SerializationReceipt` or
`FormatSummary`; only that action's completion method exists on it. A compute
action computes, a digit read reads one record, a digit write mints a record
and a receipt that acknowledges its actual storage, and an emission
acknowledges the byte it issued. A caller admits each action's register and
`ActionLayout` before completing it. Declining an action, or refusing storage
through the receipt, returns the unchanged continuation. Dropping a
continuation or an action abandons it without further numerical work.

Keep `OperationOwner` as a separate stack local. Operations and `DigitSlot`
records borrow its real identity, phase and lifetime; a retained record
prevents owner reuse at compile time. A record of another operation or index
is a defect in the driving code and panics when it is read or acknowledged,
as slice indexing does. The record a receipt acknowledges must be the one the
caller actually stored at that index: the check covers only the record's
operation, index and digit, so a substituted record with the same identity is
accepted, and reading back a different stored record yields wrong output that
the fixed registers cannot detect. Bank disposal remains the caller's
responsibility. The consumer owns its output buffer and atomic publication.
Ordinary `serialize` services this same continuation with explicit action,
digit and output limits and fallible allocation.

## Scope and sources

This crate contains no authored CSS grammar, expression evaluator, named-color
lookup, system palette, profile binding, source coordinates or HTML controller.
CSS maps its authored spaces and resolved operands into the numerical API. HTML
and other consumers retain their own admission, fallback and provenance rules.
Rendering representation, clipping, painting and style orchestration belong to
their consumers.

The [source-edition reference](../../references/SOURCE-EDITIONS.md#html-color-delegation-and-intrinsic-contrast-policy)
records the selected standards and product determinations. The
[integration contract](../../docs/reference/color-crate-contracts.md) explains
the CSS/Color boundary. See [NOTICE.md](NOTICE.md) for source attribution and
[AGENTS.md](AGENTS.md) for focused verification. Public source is authoritative;
the root-owned [API audit](../../api/crates/surgeist-color.txt) is generated.
