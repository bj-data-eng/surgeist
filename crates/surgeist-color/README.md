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

`Color::try_new` checks finite present components and alpha in `[0, 1]` while
preserving missing components and extended finite channels. It does not infer
byte provenance from integral floating-point values. `from_srgba8` establishes
actual channel and alpha byte provenance; an explicit `QuantizeSrgb`
transformation establishes channel bytes without inventing alpha bytes.
`identical` compares representation, including missing values, signed zero,
byte provenance and serialization family. Ordinary equality compares the
stored value using floating-point equality.

`ColorSpace` covers predefined RGB/XYZ spaces, HSL/HWB, Lab/LCH and
Oklab/Oklch. `RectangularCoordinates`, `HslCoordinates`, `HwbCoordinates` and
`PolarCoordinates` offer the corresponding checked numerical conversion APIs.
Conversion preserves finite extended coordinates and reports an unrepresentable
final result, except that forward HSL/HWB conversion returns the signed closest
finite channel (the CSS Values 4 closest supported value); it does not apply a
global gamut map. The implementation uses
binary64 significand arithmetic, not exact-real or correctly rounded arithmetic.
Source-specific numerical policies remain documented in the conversion modules.

`SerializationFamily` selects the resolved spelling independently of authored
text. `SerializationRequest::HtmlCompatible` uses hexadecimal output only when
the value has actual sRGB byte channels and opaque alpha. Other values retain
their selected resolved color form. The shared `format` module owns finite
formatting and rounding, including non-color CSS consumers;
`byte_alpha_for_serialization` owns the byte-alpha spelling policy.

## Controlled operations

`Computation`, `Serialization` and `format::FiniteFormatter` expose pending
actions and their actual register/read/write/output layouts. A caller admits an
action before executing its guard. Dropping an unexecuted guard leaves the
pending action unchanged. Numerical computations use fixed registers;
serialization and finite formatting lend `DigitSlot` records to caller-owned
banks and emit one byte at a time. They allocate no hidden digit bank.

Keep `OperationOwner` as a separate stack local. Operations and records borrow
its real identity, phase and lifetime; retained records prevent owner reuse.
Foreign records, wrong indices/replies and terminated operations produce typed
protocol errors. Bank disposal remains the caller's responsibility.

`take(&mut Option<Operation>)` removes only a completed success. A refusal
preserves the operation. `recover` returns its original request or value and
removes the operation without completing more numerical work. A computed result
is committed only by successful `take`. Serialization recovery also reports
already emitted length; the consumer owns its output buffer and atomic
publication. Ordinary `serialize` services this same protocol with explicit
action, digit and output limits and fallible allocation.

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
