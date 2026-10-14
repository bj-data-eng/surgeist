# Numerical color extraction contract

This reference defines the CSS → Color boundary for
[#1184](https://github.com/bj-data-eng/surgeist/issues/1184), under
[#1183](https://github.com/bj-data-eng/surgeist/issues/1183). It is an implementation
contract, not a claim that extraction has shipped. Its source basis is
`11fea87cca1674b93dc2bef57bb138ff55570cfe`; the implementation and its focused
regressions remain authoritative for existing numerical results, standards pins,
rounding and unsupported capabilities. The intended readers implement and review
the preparation, extraction and integration tasks #1185, #1188 and #1189.

`surgeist-color` owns checked numerical values and operations. `surgeist-css`
owns authored CSS and the single resolver that converts authored meaning and
explicit context into those values. A native numerical caller has no CSS source,
fact issuer or parser controller to invent. A controlled caller services finite
actions using its actual admission and storage. No new parsing, gamut mapping,
profile capability, render representation or external dependency is introduced.

## Owners and source crossings

All source paths in this section are relative to
[`crates/surgeist-css/src/`](../../crates/surgeist-css/src/).

| Current owner/path and actual caller | Destination and contract |
| --- | --- |
| `syntax/color.rs`: `CssPredefinedColorSpace`, `CssColorInterpolationSpace`, `CssHueInterpolationMethod`; used by grammar, channels, relative bindings, conversion, facts and serialization | Color owns numerical `PredefinedColorSpace`, `ColorSpace`, `HueInterpolation`. CSS reads authored names and retains authored omission; there is one numerical identity per space, not a CSS mirror enum. |
| `rectangular_color_conversion.rs` and its `colorimetry`, `transfer`, `wide` modules; called by `color/used.rs` | Color owns `RectangularSpace`, `RectangularCoordinates`, `RectangularConversionError` and the existing finite conversion implementation, including wide intermediate handling and missing-coordinate rules. |
| `hsl_color_conversion.rs`, `lab_color_conversion.rs`, `naive_color_conversion.rs`; called by `used.rs`, `device.rs` and specified color serialization | Color owns the checked coordinate carriers and pure conversions. CSS keeps the decision to invoke a conversion and its authored range/serialization phase. |
| `color/used.rs`; called by channels, relative, interpolation, resolution, device/profile conversion, serialization and HTML | Color owns `Color`, `ColorError`, numerical transformations, representation identity, alpha replacement and the existing black/white contrast calculation. |
| `color/channels.rs`: `Component`, `Domain`, `legacy_rgb_domains`, `scalar`, `hue`, `alpha`, `absolute_coordinates`; reached from `component_value.rs` and `resolution.rs` | CSS owns Number/Percentage/Degrees/Missing grammar, unit scaling, calculation-root NaN/infinity handling, range clamping and absolute/relative policy. Only checked finite coordinates cross to Color. `Component` does not become a Color type. |
| `color/interpolate.rs`: `prepare`, `interpolate`; used by `relative::Binding::new`, `resolution::MixTask` and folds | Color owns numerical preparation and two-color interpolation. CSS owns mix traversal, exact weight evaluation, omitted-weight normalization and source-ordered fold scheduling. |
| `color/interpolate.rs`: `WeightSummary`, `Weights`, `MixFold` | Keep the CSS mix recipe state in CSS. Its fold calls Color's two-color operation once per reached operand; do not export a variadic CSS mix interpreter from Color. |
| `color/relative.rs`: `Binding`, `channel`, `coordinates`, `alpha_only`; `expression.rs` variable evaluation and `resolution.rs` frames | CSS owns environment/channel-name bindings, RGB ×255 expression units, missing-to-zero inside calculations, expression execution, inherited/overridden alpha, source-selected family and reached branching. It calls Color's preparation/conversion/alpha/family methods. |
| `color/device.rs`: `ComputedCmyk::new`, `DeviceResolution`; `profile_conversion.rs`: input collection, reverse checks and conversion-record lookup | CSS owns syntax, resource identity, channel names/order, fact matching, capability/fallback selection and the unsupported reverse/custom branches. Color owns only the already-present context-free naive CMYK numerical conversion and replacement of resolved alpha on a supplied numerical result. No ICC engine is added. |
| `color/facts.rs`, `palette.rs`, `profile.rs`; `resolution.rs` callers | Stay CSS: current/system color, scheme, profile/device facts, identities, duplicate/foreign checks and reached-only lookup. Fact payloads use `Color` directly. |
| `color/authored.rs`: `Events`, `Builder`; called by `parser/color.rs` | Stay CSS: canonical checked-authored construction, exact component tokens, original origins and closure recovery. Add a borrowed authored-to-recipe feeder for resolution; never serialize the authored value and parse it again. |
| `color/grammar.rs`, `tokens.rs`, `literal.rs`, `decimal.rs`, `syntax_pass.rs`, `component.rs`, `component_value.rs`, `expression.rs`, `operation.rs`, `resolution.rs` | Stay CSS: original source decoding, tokens, grammar, exact numeric admission/evaluation, resolver state and errors. Numerical requests contain fixed resolved operands, not any of these records. |
| `color/protocol.rs`: mixed `Data`, owner/phase/bank/index qualification; `bounded.rs` public actions and checkpoints | Stay CSS. Remove numerical algorithms and the format-digit variant from these records. Color digit slots occupy a distinct caller-owned numerical bank and borrow a separately lent Color owner. CSS retains its three parse-bank identities; no CSS-private union becomes Color's public protocol. |
| `color/serialize.rs`: `Plan`, `UsedSerializer`; called by `bounded.rs` and `ordinary.rs` | Color owns the resolved serialization plan, finite digit work and byte emission. CSS wrappers retain source/fact checkpoints and delegate numerical actions. Native serialization uses Color directly. |
| `numeric_formatting/bounded.rs` and finite portions of `numeric_formatting.rs`; used by resolved/ordinary color and non-color projection | Color owns the single finite-bit formatter, digit continuation, rounded-millionths operation and finite zero classification. Exact decimal parsing, representation and specified-value policy stay CSS. |
| `exact_decimal.rs::DecimalRounding`; used by exact rationals and finite formatting | Color owns a small `RoundingMode` with the shared tie decision. CSS exact-decimal arithmetic imports that policy; its arbitrary-precision storage and digit scans stay CSS. |
| `syntax/color/serialization.rs::legacy_byte_alpha` and `color/serialize.rs::legacy_byte_alpha` | One Color `byte_alpha_for_serialization(u8) -> f64` supplies both paths; #1186 consolidates it before or during extraction. Floating alpha never gains byte provenance by numerical equality. |

This table includes the private crossings requiring a public seam. Internal
matrix arithmetic, transfer functions, exact decimal registers, lexical tables,
tokenizer states and resolver frames remain private to their selected owner.
CSS keeps `CssColor`, every authored function/profile/expression carrier,
`CssColorSyntax`, relative-channel names and authored interpolation wrappers.
`CssColorInterpolationMethod` retains authored hue omission and holds Color's
space/hue identities; it lowers to the checked numerical `InterpolationMethod`.
Its parsing error stays CSS, while invalid native numerical method construction
belongs to Color.

## Native numerical front door

The following is the selected public API shape. Signatures describe the future
API; current source has the corresponding `Css*` implementations where the table
identifies them. Fields of invariant-bearing structs are private.

```rust
// surgeist_color
pub enum PredefinedColorSpace {
    Srgb, SrgbLinear, DisplayP3, DisplayP3Linear, A98Rgb,
    ProphotoRgb, Rec2020, XyzD50, XyzD65,
}
pub enum ColorSpace {
    Predefined(PredefinedColorSpace), Hsl, Hwb, Lab, Lch, Oklab, Oklch,
}
pub enum HueInterpolation { Shorter, Longer, Increasing, Decreasing }
pub struct InterpolationMethod { /* checked numerical method */ }
impl InterpolationMethod {
    pub fn try_new(space: ColorSpace, hue: Option<HueInterpolation>)
        -> Result<Self, InterpolationError>;
    pub fn space(self) -> ColorSpace;
    pub fn hue(self) -> Option<HueInterpolation>;
}
pub enum SerializationFamily { Natural, ColorFunction, LegacySrgb }
pub enum Transformation {
    ForceOpaqueAlpha, Convert(ColorSpace), QuantizeSrgb, ColorFunction,
}
pub struct Color { /* space, coordinates, alpha, byte backing, family */ }
impl Color {
    pub fn try_new(space: ColorSpace, channels: [Option<f64>; 3], alpha: Option<f64>)
        -> Result<Self, ColorError>;
    pub const fn from_srgba8(value: [u8; 4]) -> Self;
    pub const fn opaque_black() -> Self;
    pub fn space(&self) -> ColorSpace;
    pub fn channels(&self) -> [Option<f64>; 3];
    pub fn alpha(&self) -> Option<f64>;
    pub fn srgb_bytes(&self) -> Option<[u8; 3]>;
    pub fn alpha_byte(&self) -> Option<u8>;
    pub fn serialization_family(&self) -> SerializationFamily;
    pub fn identical(self, other: Self) -> bool;
    pub fn converted(self, destination: ColorSpace) -> Result<Self, ColorError>;
    pub fn prepared_for_interpolation(self, destination: ColorSpace)
        -> Result<Self, ColorError>;
    pub fn interpolated(self, second: Self, method: InterpolationMethod, progress: f64)
        -> Result<Self, InterpolationError>;
    pub fn transformed(self, transformation: Transformation) -> Result<Self, ColorError>;
    pub fn with_alpha(self, alpha: Option<f64>) -> Result<Self, ColorError>;
    pub fn with_serialization_family(self, family: SerializationFamily)
        -> Result<Self, ColorError>;
    pub fn black_white_contrast(self) -> Result<Self, ColorError>;
}
pub enum SerializationRequest { Css, HtmlCompatible }
pub struct SerializationLimits {
    pub actions: usize, pub digits: usize, pub output_bytes: usize,
}
pub fn serialize(color: Color, request: SerializationRequest, limits: SerializationLimits)
    -> Result<String, SerializationError>;
pub fn byte_alpha_for_serialization(alpha: u8) -> f64;
```

Numerical space/hue/error enums are `#[non_exhaustive]`; request/family choices
are closed. These are pre-v1 breaking moves, with no old-path aliases or public
CSS reexports of numerical names. Color and coordinate carriers remain
`Clone + Copy + Debug + PartialEq`; they do not derive `Eq`, `Ord` or `Hash` over
floating values. `identical` compares coordinate and alpha bits, missingness,
actual byte backing, space and effective serialization family. It is the
identity predicate used by protocol record checks; ordinary `PartialEq` retains
its current numerical behavior, including equal signed zeros.

`try_new` visits channels 0, 1, 2, then alpha and returns the first
`ColorError::NonFinite { component }`; present alpha must then be in `[0,1]` or
returns `AlphaOutOfRange`. It preserves signed zero and all finite extended
channels, including signed chroma/saturation. `None` is an explicit missing
channel/alpha, not omitted syntax. Omitted authored alpha is resolved by CSS to
`Some(1.0)` before construction. No method accepts NaN/infinity as a missing
sentinel. Native construction neither clamps color channels nor infers bytes.
`ColorError` has `NonFinite { component: usize }`, `AlphaOutOfRange`,
`InvalidCoordinates`, `Unrepresentable`, `RequiresSrgb` and
`InvalidSerializationFamily` kinds. The existing specialized coordinate errors
retain their component indices and validation order rather than being flattened
into ColorError at their own front doors.

| Space | Ordered numerical units |
| --- | --- |
| Predefined RGB / XYZ | R,G,B / X,Y,Z normalized reference numbers; 100% is 1 |
| HSL | hue degrees, saturation percentage reference number, lightness percentage reference number; 100% is 100 |
| HWB | hue degrees, whiteness and blackness percentage reference numbers |
| Lab / LCH | native CIE lightness (usual 0–100), a/b or nonnegative converter chroma and degree hue |
| Oklab / Oklch | normalized lightness (usual 0–1), native a/b or converter chroma and degree hue |

The numerical `Color` carrier admits finite signed polar coordinates required
by resolved relative colors. Dedicated `HslCoordinates`, `HwbCoordinates` and
`PolarCoordinates` retain their current stricter constructors; they do not
silently accept states their current mathematical contracts reject. Their
renamed APIs, accessors and errors move directly from the existing HSL/HWB/Lab
modules by removing `Css` and the redundant `Color` qualifier. Thus
`CssHslColorCoordinates` becomes `HslCoordinates`, `CssHwbColorCoordinates`
becomes `HwbCoordinates`, `CssPolarColorCoordinates` becomes `PolarCoordinates`,
and `CssHslHwbConversionError`/`CssPolarColorConversionError` become
`HslHwbConversionError`/`PolarConversionError`. `RectangularCoordinates` retains
`try_new`, `space`, `channels`, and `convert_to`. The public functions
`convert_hsl_to_srgb`, `convert_srgb_to_hsl`, `convert_hwb_to_srgb`,
`convert_srgb_to_hwb`, `convert_lab_to_lch`, `convert_lch_to_lab`,
`convert_oklab_to_oklch`, `convert_oklch_to_oklab`,
`naively_convert_cmyk_to_srgba` and `naively_convert_srgba_to_cmyk` keep their
existing checked parameter/result semantics under Color. `NaiveConversionError`
replaces `CssNaiveColorConversionError`.

Same-space conversion returns the original value bit-for-bit. Cross-space
conversion preserves extended finite coordinates and alpha byte backing,
reusing the current missing-to-zero conversion rules and polar reconstruction;
it does not promise preservation of missing masks across all spaces. The
separate preparation operation preserves analogous missing components and
applies the existing powerless-hue thresholds only when converting spaces.
Signed polar chroma and HSL saturation use the existing opposite-hue Cartesian
adapter. Final unrepresentable results return `Unrepresentable`; arithmetic
does not fail merely because a wider intermediate exceeds finite f64.

`InterpolationMethod::try_new` rejects hue on a rectangular space and defaults
polar omission to `Shorter`; its `hue()` returns the effective strategy. The
two-color operation first checks finite progress in `[0,1]`, then prepares the
first and second operands in that order, carries analogous missing components,
uses the existing premultiplied-alpha calculation with hue excluded, preserves
both-missing components, and applies the selected hue arc. No extrapolation is
added. `InterpolationError` distinguishes `InvalidProgress`,
`HueRequiresPolarSpace`, and `Numerical(ColorError)`. CSS's ordered mix fold
retains the existing zero-total two-operand progress of 0.5, omitted-weight
normalization and final alpha multiplier; it requests `ColorFunction` after the
fold. These steps do not become an alternative native CSS resolver.

`ForceOpaqueAlpha` preserves channel/alpha backing and replaces alpha by unity;
`with_alpha` validates first, preserves channel backing, and intentionally makes
alpha floating. `QuantizeSrgb` requires encoded sRGB (`RequiresSrgb` otherwise),
clamps present coordinates to `[0,1]`, rounds positive half ties to bytes and
preserves missingness. It neither changes alpha nor infers alpha bytes.
`ColorFunction` selects modern serialization without a conversion. Family
selection accepts `LegacySrgb` only for encoded sRGB; `Natural` chooses the same
space-based natural family as `try_new`, while `ColorFunction` retains the
modern marker across conversions, including cylindrical spaces. Invalid family
selection returns `InvalidSerializationFamily` before mutation. `black_white_contrast`
retains the current XYZ-D65 luminance clamp and white/black comparison, ignores
alpha, chooses white on ties and returns actual opaque sRGB bytes. It selects no
new contrast or gamut algorithm.

## Controlled numerical execution

Color must support the original parser controller without depending on that
controller. It publishes concrete operations, actions and opaque digit records;
it does not publish a trait that claims admission happened elsewhere.

```rust
pub struct OperationOwner { /* exclusive monotonic phase */ }
pub struct DigitSlot<'owner> { /* owner, phase, index, checked digit */ }
pub enum NumericalRequest {
    Construct { space: ColorSpace, channels: [Option<f64>; 3], alpha: Option<f64> },
    Convert { value: Color, destination: ColorSpace },
    Prepare { value: Color, destination: ColorSpace },
    Interpolate { first: Color, second: Color, method: InterpolationMethod, progress: f64 },
    Transform { value: Color, transformation: Transformation },
    WithAlpha { value: Color, alpha: Option<f64> },
    WithFamily { value: Color, family: SerializationFamily },
    BlackWhiteContrast { value: Color },
}
pub struct Computation<'owner> { /* fixed registers only */ }
pub struct Serialization<'owner> { /* fixed plan and finite cursor only */ }
pub enum ActionDescription {
    Compute(Kernel), ReadDigit { index: usize }, AppendDigit { index: usize },
    ReplaceDigit { index: usize }, EmitByte { unit: u8 }, Ready, Failed, Abandoned,
}
pub struct ActionLayout {
    pub read_bytes: usize, pub write_bytes: usize, pub output_bytes: usize,
}
```

`OperationOwner::layout_bytes()`, `Computation::register_bytes()`,
`Computation::action_bytes()`, `Serialization::register_bytes()`,
`Serialization::action_bytes()` and the corresponding FiniteFormatter methods
are associated static functions callable before any instance is constructed.
`OperationOwner::new()` is allocation-free. `Computation::new(&mut owner,
request)` and `Serialization::new(&mut owner, value, request)` return
`Result<_, ProtocolError>`, advancing a checked phase only after initialization
is admitted. They copy fixed operands but do not execute them. Both expose
`register_bytes()`, `action_bytes()`, `action()`, `next_action()`, `abandon()`;
`Computation::take(slot: &mut Option<Self>) -> Result<Color, ComputationError>`
preflights through a shared borrow of the slot and removes its operation only
at Ready. EmptySlot, NotReady, Failed and Abandoned leave the original slot
unchanged. Failed returns its stored computation error; Abandoned returns
Terminated. A refused take can therefore be followed by the same pending action.
`ComputationError` distinguishes `Numerical(ColorError)`,
`Interpolation(InterpolationError)` and `Protocol(ProtocolError)`.
`Computation::original(&self) -> NumericalRequest` permits recovery of the exact
copyable request on failure/abandonment; it creates no result or resource credit.
One admitted compute executes exactly its fixed request, reusing the ordinary
kernel and its validation order. The public low-level coordinate operations
are also fixed kernels: a controlled caller must admit their actual fixed
operands/registers before invoking them, with no variable scans hidden inside.

`Kernel` identifies the actual fixed numerical request or the existing finite
formatting transitions (`PlanFinite`, `TrimMillionths`, `ExtractDigit`,
`MultiplyDigit`, `NextMultiply`, `PlanOutput`, `PlanEmission`) and serialization
planning/piece advancement. Scalar library arithmetic remains an atomic bounded
operation; the contract does not invent internal instruction counts.

An action guard exclusively borrows its operation and exposes `description()`
and `layout()` without inspecting variable payloads or doing numerical work.
Its consuming `compute()` executes the advertised fixed kernel; `read_digit(index,
&slot)` observes one qualified slot; `record(&mut self)` generates the advertised
append/replacement record after admission; `written(self, index, &slot)` checks
the actual stored record before acknowledging it; `emitted(self, unit)`
acknowledges an actual ASCII write. All return typed `ProtocolError` or the
operation's numerical error. Wrong action/index/digit/owner/phase or terminal
state is rejected without changing the pending action. `record()` is idempotent
for that pending write. Dropping a guard performs no compute, acknowledgement,
release, phase change or output. A rejected or refused action can be serviced
again against the same state.

An admitted computation that returns a numerical error enters Failed, stores
that first typed error and produces no replacement result. `failure()` borrows
that error; `take(&mut slot)` returns it while retaining the Failed operation.
Numerical failure is not a resumable pending
compute. Abandonment preserves the first failure for inspection. Protocol
errors are distinct: the pending action remains available for a corrected reply.
`ProtocolError` distinguishes `Overflow`, `ForeignSlot`, `WrongAction`,
`WrongIndex`, `WrongDigit`, `WrongReply`, `NotReady`, `StorageNotEmpty` and
`Terminated` and `EmptySlot`. Only Ready permits successful take; an early take
is NotReady. None of these refusals removes or replaces the operation.

`Computation::recover(slot: &mut Option<Self>) -> Result<NumericalRequest,
ProtocolError>` preflights Some, removes that operation in any state and returns
its original copyable request. `Serialization::recover(slot: &mut Option<Self>)
-> Result<SerializationRecovery, ProtocolError>` similarly returns the original
Color, SerializationRequest and acknowledged output length. FiniteFormatter's
recover returns `FormatRecovery` containing its original f64, rounding mode
and acknowledged length. Both recovery structs have read-only accessors.
Recovery performs no compute, output, capacity release or source publication.
Only EmptySlot refuses recovery, leaving None unchanged. These transitions
are associated functions, so Rust's guard borrow also prevents taking/recovering
an operation while one of its actions remains borrowed.

The caller admits fixed owner/register/guard layout before construction and
every guard, then admits actual payload reads, work, allocation/capacity, writes
and output before invoking that action. `ActionLayout` is computed with actual
compiled `size_of` values, including record wrappers and alignment, never
copied constants from the mixed CSS record. `register_bytes()` includes retained
plans/cursors. Caller wrappers add their own storage and checkpoint fields once.
`DigitSlot` contains only numerical ownership, phase, index and digit; it has no
CSS source/fact identity. Pointer/phase qualification is safe Rust and does not
use global IDs or unsafe code. OperationOwner privately contains a nonzero-sized
marker and checked usize phase. New receives `&'owner mut OperationOwner`,
increments its phase only on success and stores `&'owner OperationOwner` in the
continuation; DigitSlot stores that same shared reference, phase and index.
There is no interior-mutable numerical owner reachable through a CSS checkpoint.
The exclusive initialization borrow prevents concurrent new operations and
owner reuse while numerical records remain live. Checked phases additionally
reject retired or foreign records supplied through a public action.

Serialization appends scratch digits at the actual bank length, updates only an
existing qualified index, and never owns/reserves caller buffers. At most 309
coefficient digits are needed for one finite binary64 value; a complete color
may retain multiple numbers' records, so the total digit admission is the actual
reported append sequence, not a blanket 309-record promise. The caller may reuse
scratch only after the operation explicitly retires its phase; no action releases
capacity or caller ledger charges. `Serialization::take(slot: &mut Option<Self>)`
returns `Result<SerializationReceipt, SerializationError>` and removes the
operation only after all bytes are acknowledged. Empty/not-ready/failed/abandoned
refusals retain Some exactly as Computation does. The receipt carries `value()`
and `length()`, with no source/fact
fields and no borrowed digits. `abandon()` moves to Abandoned and performs no
useful work or resource cleanup. The caller frees its exact owned storage.

Admission refusal and cancellation are caller outcomes, not numerical invalidity.
The caller's first stop remains sticky; cleanup attempts every owned release
without polling or replacing that stop. Before stop, an unacknowledged action
can resume. After stop, abandon and release; restarting uses a new phase and
the retained original request/checkpoint. A local output prefix can exist after
acknowledged bytes, but is never a successful public result. Failed serialization
discards it. Only the complete receipt and fully built output may be installed.
Ordinary `serialize` services the same actions with local vectors, checked
logical action/digit/output ceilings and fallible allocation. Its
`SerializationError` distinguishes numerical, protocol, limit (resource,
required, allowed), allocation and count overflow. Those ordinary ceilings do
not claim physical allocation accounting or manufacture an HTML controller.

## One finite formatter and one rounding policy

Color exposes `format::RoundingMode::{AwayFromZero, TowardPositiveInfinity}`,
`format::format_finite(value: f64, rounding: RoundingMode, byte_limit: usize)
-> Result<String, FormatError>`, and `format::rounds_to_zero(value: f64,
rounding: RoundingMode) -> Result<bool, FormatError>`. Both validate finiteness;
`FormatError` distinguishes NonFinite, ByteLimit, CapacityOverflow and Protocol.
`RoundingMode::rounds_up(half_order: std::cmp::Ordering, negative: bool) -> bool`
owns the shared tie rule and is reused by CSS's exact arithmetic. It increments
above half; at exact half it increments always for AwayFromZero and only for
nonnegative values for TowardPositiveInfinity.

`format::FiniteFormatter<'owner>` uses `OperationOwner`, `DigitSlot` and the
same action guard contract as Serialization. Its constructor takes value and
rounding; it exposes `planned_length() -> Option<usize>` before any emission and
`take(slot: &mut Option<Self>) -> Result<FormatSummary, FormatError>` preflights
Ready before removal; all error paths retain the operation. Its summary has
`is_zero` and `emitted` accessors. This wraps the
existing `FiniteFormatCursor`; Color keeps that arithmetic private and shared
with complete color serialization. One kernel formats the actual finite bits
to at most six fractional places, strips trailing zeros, emits positive zero,
expands finite integers exactly, and preflights final output rather than a
temporary shortest-decimal spelling. No shortest-string conversion or float
rounding replacement is allowed. CSS's `format_projected_number` and
`format_ordinary_color_number` become error-mapping calls, not retained math.

CSS retains `LexicalDecimal`, arbitrary-length coefficient/exponent handling,
exact rationals, specified shifts/sign inversion, projection work and the exact
decimal digit emitter. That emitter accepts already-rounded exact decimal
digits; it is not a second finite-bit formatter. It never feeds an authored
coefficient through f64 merely to reuse Color. The existing authored endpoints,
coefficients beyond finite f64 and discarded-exponent behavior remain exact.

| Current finite consumer | Required owner/policy and regression consequence |
| --- | --- |
| `color/serialize.rs` controlled used output | Color serializer, ties toward positive infinity, caller-owned digit/write actions; preserve one-short and pending-byte behavior. |
| `syntax/color/serialization.rs::format_binary64` | CSS specified color policy calls Color finite formatting with positive-infinity ties on the existing post-scale binary result; exact color calculations retain their own rational phase. |
| `numeric/projection.rs` finite coefficient serialization | Color finite formatting with away-from-zero ties; CSS retains unit/suffix limits, expression visits and projection charges. |
| `page_keyframe_serialization.rs` computed selector percentages | Same projected formatter and away-from-zero ties; preserve rounded selector collisions, occurrence order, suppressed output and shared writer budgets. Page rules themselves retain their existing nonnumeric writer behavior. |
| `rotate.rs` projected axis zero classification, via `projected_number_rounds_to_zero` | Color `rounds_to_zero(_, AwayFromZero)`; CSS retains axis/identity decisions, exact authored sign inversion, traversal and suppression. No rendering-to-string probe is introduced. Actual projected output goes through the existing numeric projection path. |
| `numeric_formatting.rs` ordinary projected/ordinary-color wrappers and tests | Move finite tests with kernel; retain CSS error translation, exact decimal tests and policy selection. |

The exact authored decimal kernel remains CSS: `format_css_number` and
`format_css_number_with_sign` serve `specified_serialization.rs`,
`specified_numeric.rs`, `aspect_ratio.rs`, `font_style.rs`,
`transform_value_serialization.rs` and exact-number branches of `rotate.rs`.
`exact_decimal.rs` and `specified_serialization.rs` also use the already-rounded
exact decimal emitter. These paths do not move their storage or specified-value
policy to Color. `image_serialization.rs` consumes authored/numerical space
identities through CSS image policy; update its import without moving image
serialization into Color.

`byte_alpha_for_serialization` uses the existing integer two/three-decimal
Color 4 algorithm for every byte: try nearest hundredth and retain it if its
round trip recovers the byte, otherwise nearest thousandth. Both specified and
used serializers feed this result into the shared positive-tie finite formatter.
Bytes 236 and 237 produce 0.925 and 0.93 respectively. Actual floating alpha is
formatted from its own bits; it is not reclassified as byte alpha. Pinned source
edition/prose discrepancies stay documented with their existing selected policy.

## CSS resolution and provenance

The existing `parse_used_color(input, grammar_request, facts, expected_identity,
limits)` remains CSS and returns a source-qualified result whose Used payload
is Color. UTF-8 bytes and UTF-16 units remain borrowed original input; consumed
ranges are original units. Prefix parsing retains its actual end boundary.
Syntax validation visits complete unselected branches, while facts and numeric
evaluation occur only on reached branches. Source decoding, source-derived
failures and semantic numerical failures are separate.

There is currently no public checked-authored-to-used operation: `authored.rs`
constructs authored values but does not supply that front door. Add
`resolve_authored_color(&CssColor, &CssColorFacts, expected_identity,
CssColorOrdinaryLimits) -> Result<CssAuthoredColorResult, CssColorOrdinaryError>`.
The result borrows the actual authored root and facts, exposes `authored()`,
`facts()` and `outcome() -> &CssAuthoredColorOutcome`, where that enum contains
`Used(Color)` or `StandardFailure(CssAuthoredColorStandardFailure)`.
`CssAuthoredColorStandardFailure` contains the existing standard-failure kind
plus a checked component path and `Option<CssValueOrigin>`. A scalar/component
with an origin retains it; a programmatic hex/name/root that has no origin
returns None rather than inventing one. No input bytes, source offsets or source/fact identity are
invented. Programmatic `Components` origins remain Components; parsed origins
remain Parsed.

`CssColorComponentPath` owns a checked sequence of `CssColorPathStep` values,
with `steps() -> &[CssColorPathStep]` for inspection. The closed steps are
`Source`, `Channel(usize)`, `Alpha`, `First`, `Second`, `MixColor(usize)`,
`MixWeight(usize)` and `ExpressionOperand(usize)`: source denotes a relative,
alpha-wrapper or contrast origin; first/second denote light/dark respectively;
indices use the current branch's original child/channel/operand order. Only
the CSS feeder constructs paths and checks each step against its actual root.
Path backing is accumulated under the actual controller before the numerical
action/error publication; failure moves the already admitted path into its
error without allocating after a stop. Ordinary resolution uses the same
fallible storage boundary and maps refusal to its existing resource/allocation
error. No derived Copy implementation or implicit clone may conceal that
storage. The authored outcome borrows its failure payload for this reason.

Its controlled equivalent is
`CssAuthoredColorOperation::resolve_authored(&mut CssColorOperationOwner,
&CssColor, &CssColorFacts, expected_identity)`. It has the same bank/action,
layout, abandon and checkpoint lifecycle as the source operation, plus
`ReadAuthored { path, observation }` for one actual authored node, scalar/name
unit, exact digit or expression item. `path` is a checked opaque CSS-owned path;
`observation` is a closed CSS descriptor identifying precisely one requested
field or unit. The guard's `read_authored()` method performs that one borrowed
observation after caller admission, so a caller cannot supply an unrelated
node. It never recursively scans or clones the tree under one action. CSS
retains explicit traversal frames in its admitted algorithm bank; each next
child/path extension is separately admitted. The feeder lowers checked native
branches into the same canonical recipes/expressions and resolver as source
parsing, using original typed numeric tokens and origins. No public CSS syntax
service is exported merely for Color. Authored results use a distinct provenance
carrier from source results; the shared internal resolution value is Color.

The revised `CssColorOperationError::Numerical { error: ColorError,
site: CssColorResolutionSite }` attaches Color failures to the actual reached
CSS recipe. `CssColorResolutionSite` is `Source(CssColorInputRange)` or
`Authored { path: CssColorComponentPath, origin: Option<CssValueOrigin> }`;
it distinguishes original input units from authored component paths. Controlled
and ordinary errors retain this site even when no success checkpoint exists.
CSS owns
grammar, invalid/foreign fact, unavailable/unsupported, exact-expression,
source and protocol errors. Color owns numerical construction, interpolation,
conversion, formatting and numerical protocol errors. CSS wraps nested Color
protocol failures as `CssColorProtocolError::NumericalProtocol(ProtocolError)`;
it does not turn them into standard failure. Public errors implement Display
and Error, preserve their typed source, and do not reduce kinds to strings.
Standard grammar failure alone can mint the existing fallback checkpoint.
Missing facts, unsupported conversion, resource refusal and cancellation cannot.

CSS's `CssUsedColorCheckpoint`, `CssColorTransformation`,
`CssColorSerialization`, source descriptor and serialization receipt remain
meaningful source-qualified wrappers. They delegate actual numerical work and
digit handling to Color, retain source/fact ownership, and map numerical actions
into `CssColorActionDescription` before work. They are not no-op adapters:
they preserve consumed original units, chosen fallback, facts, owner/phase,
bank emptiness and recovery. Native callers bypass these wrappers. Remove
ordinary `serialize_used_color` and the native-only
`CssColorSerialization::from_used` constructor from CSS when Color's `serialize`
lands; a
source-qualified controlled wrapper remains because it carries real provenance.

### Separate anchors and checkpoint transitions

The caller owns two separate stack anchors. The existing
`CssColorOperationOwner` retains its nonzero marker and private `Cell<usize>`
CSS phase. The separate Color `OperationOwner` has a marker and plain usize
numerical phase. CSS never embeds the numerical owner beneath its shared
checkpoint reference. Neither a continuation nor its caller banks owns an
anchor it also references. The selected private borrow shape is:

```rust
struct CssContext<'css, 'facts> {
    owner: &'css CssColorOperationOwner,
    phase: usize,
    facts: &'facts CssColorFacts<'facts>,
    source: CssColorSourceDescriptor,
    standard_failure: Option<CssColorStandardFailure>,
}
pub struct CssUsedColorCheckpoint<'css, 'facts> {
    context: CssContext<'css, 'facts>,
    value: Color,
}
pub struct CssColorTransformation<'css, 'num, 'facts> {
    context: CssContext<'css, 'facts>,
    original: Color,
    numerical: Option<surgeist_color::Computation<'num>>,
}
pub struct CssColorSerialization<'css, 'num, 'facts> {
    context: CssContext<'css, 'facts>,
    original: Color,
    numerical: Option<surgeist_color::Serialization<'num>>,
}
```

The source operation and its three banks retain only `'css`:
`CssColorOperation<'css, 'facts>` and `[Vec<CssColorSlot<'css>>; 3]` in the ordinary
owner, or the existing three admitted OwnedVec banks in the parser owner.
Color's separate `[DigitSlot<'num>]` storage is not placed inside those CSS
slots. `Data::FormatDigit` and its CSS slot kind are removed. The actual parser
uses a fourth, separately admitted `OwnedVec<DigitSlot<'num>>` header during
serialization, with the existing AuxiliaryEntries resource category and actual
numerical-record capacity bytes. The source banks are released before that
header and its digits are created. This changes record layout and action shape,
not the controlling ledger or resource policy.

The public transition signatures are:

```rust
impl<'css, 'facts> CssUsedColorCheckpoint<'css, 'facts> {
    pub fn transform<'num>(
        slot: &mut Option<Self>,
        numerical_owner: &'num mut surgeist_color::OperationOwner,
        effect: surgeist_color::Transformation,
        banks: CssColorBankLengths,
    ) -> Result<CssColorTransformation<'css, 'num, 'facts>, CssColorOperationError>;
    pub fn serialize<'num>(
        slot: &mut Option<Self>,
        numerical_owner: &'num mut surgeist_color::OperationOwner,
        request: surgeist_color::SerializationRequest,
        banks: CssColorBankLengths,
        numerical_digits: usize,
    ) -> Result<CssColorSerialization<'css, 'num, 'facts>, CssColorOperationError>;
}
impl<'css, 'num, 'facts> CssColorTransformation<'css, 'num, 'facts> {
    pub fn take(slot: &mut Option<Self>)
        -> Result<CssUsedColorCheckpoint<'css, 'facts>, CssColorOperationError>;
    pub fn recover(slot: &mut Option<Self>)
        -> Result<CssUsedColorCheckpoint<'css, 'facts>, CssColorProtocolError>;
}
impl<'css, 'num, 'facts> CssColorSerialization<'css, 'num, 'facts> {
    pub fn take(slot: &mut Option<Self>)
        -> Result<CssColorSerializationReceipt<'css, 'facts>, CssColorOperationError>;
    pub fn recover(slot: &mut Option<Self>)
        -> Result<CssUsedColorCheckpoint<'css, 'facts>, CssColorProtocolError>;
}
```

`banks` and `numerical_digits` report the caller's actual admitted bank headers,
not a proposed allowance; serialize requires both CSS banks and the numerical
bank to be empty. Transformation requires the CSS banks empty and borrows no
digit bank. Both transition functions perform this fixed sequence:

1. The caller admits static CSS wrapper/numerical continuation layouts, actual
   checkpoint/header reads, phase transition work and writes before the call.
2. A shared slot borrow checks Some, actual bank emptiness, CSS anchor/phase and
   computes the next CSS phase with checked addition. No mutation occurs.
3. Color's `Computation::new(numerical_owner, request)` or
   `Serialization::new(numerical_owner, value, request)` checks numerical phase
   overflow before changing it. Refusal leaves both the CSS checkpoint slot and
   both phases unchanged. It neither reads a bank nor allocates.
4. After Color succeeds, CSS's private infallible commit sets its already checked
   next phase through Cell, removes the checkpoint with `slot.take()`, moves its
   context/value into the wrapper and stores the external-owner-borrowing
   numerical continuation. There is no fallible work after Color's phase change.
   Only CSS's exclusive non-Clone checkpoint owns this transition capability;
   slots cannot advance a phase, and Cell makes the anchor non-Sync. A shared
   CSS borrow therefore suffices for this controlled private phase commit.

All public layout getters remain associated static functions. A wrapper's
register size includes its inline context, original Color and nested numerical
continuation once; callers do not add that nested size again. Action layout
adds only actual externally read/written DigitSlot and output units. The
separate numerical owner and caller's vector headers/capacities are charged
separately. No old mixed-record size is used as numerical capacity accounting.

CSS parse-time ColorConversion, Interpolation and Contrast actions invoke fixed
Color numerical methods after their existing action admission; those kernels
allocate no digit storage and need no numerical anchor. After parsing,
transformation/serialization wrappers use the externally lent anchor above.
Serialization maps Color actions into
`CssColorActionDescription::{ReadNumericalDigit, AppendNumericalDigit,
UpdateNumericalDigit}` with an index, `Compute(CssColorKernel::Numerical(Kernel))`,
or the existing EmitUnit/Serialized/Stopped states. Its guard methods are
`read_numerical_digit(self, index, &DigitSlot<'num>)`,
`numerical_record(&mut self) -> Result<DigitSlot<'num>, CssColorOperationError>`
and `numerical_written(self, index, &DigitSlot<'num>)`. They qualify CSS context
and invoke the current nested Color action, which qualifies the numerical
anchor/phase/index/digit. The Color record borrows the external numerical anchor,
never the short guard or wrapper. Ordinary CSS slots retain their original
read/write guard methods; callers cannot interchange those two record types.

`take(&mut slot)` for each CSS source/authored operation, transformation and
serialization follows the same borrowed-slot preflight rule as Color: only a
completed success removes Some; not-ready, numerical failure, protocol error
and abandonment preserve the entire original wrapper. A completed standard
failure from a source/authored operation is also a valid completed outcome and
can be removed. `CssColorOperation::take` therefore changes from the current
consuming signature to `take(&mut Option<Self>)`; the authored equivalent uses
the same form. After successful transformation take, the returned checkpoint
has only `'css, 'facts`, so it keeps no numerical-owner borrow. Successful
serialization take returns a CSS receipt with only those same lifetimes.
Wrapper `recover(&mut slot)` removes any Some wrapper, explicitly recovers and
drops its nested numerical operation, and returns its original committed Color
with unchanged source/fact context and the wrapper's already committed CSS
phase. Only EmptySlot refuses recovery. A computed-but-not-taken value is not
committed; recovery returns the previous checkpoint. This permits cleanup and
retains provenance without restarting a stopped controller.

The real caller sequence is therefore:

1. Create CSS and numerical owners as separate admitted stack locals. Start
   `CssColorOperation::parse_used(&mut css_owner, ...)` and service its actual
   three banks through the original controller. Drop a refused action and
   retry it unchanged, or stop and abandon without useful work on Drop.
2. Use `CssColorOperation::take(&mut operation)` to obtain a checkpoint. Release
   all three actual source banks through the controller; a premature transition
   with a nonempty bank refuses without consuming that checkpoint.
3. In a short lexical scope, pass `&mut numerical_owner` to checkpoint transform,
   service its compute action, and use take to recover a new CSS checkpoint.
   Drop the now-empty wrapper Option before ending this scope. The CSS
   checkpoint remains alive while the numerical owner's borrow ends.
4. In another scope, create an admitted empty numerical-bank header and lend
   that numerical owner again to checkpoint serialize, passing the header's
   actual zero length. An empty header contains no borrowed record. Service reads,
   writes and output using actual headers and records, and take the receipt.
   Then release and drop the digit bank and the empty wrapper Option. A receipt
   alone does not end live digit-record borrows. Numerical owner reuse is legal
   only after those actual records/banks have been dropped; Rust enforces it.
5. On a stop, preserve the first stop, abandon, recover the original checkpoint
   if its provenance is needed, then release every owned digit/output bank.
   Refused take never forces recovery. After all numerical borrows end, the
   same owner can start a new phase while a CSS checkpoint still exists. Reuse
   of the CSS owner for a new parse additionally requires all old CSS slots,
   checkpoints and source-qualified receipts to be dropped.

No operation stores a mutable reference to an owner beneath a shared CSS
reference, and no struct stores both an owned anchor and a borrow of itself.
This arrangement deliberately uses an extra external anchor and typed numerical
bank instead of coupling two mutable phase protocols through shared records.

The same external-anchor rule applies to the controlled authored feeder and
its authored-qualified checkpoint wrappers, with authored path/origin context
instead of CssContext's source descriptor. It does not acquire a fake source
descriptor to reuse the source wrapper. Ordinary source/authored helpers own
their two anchors and banks locally; the HTML parser does not invoke those
helpers. Its existing source adapter starts `CssColorOperation::parse_used`
and services that operation with the original ParseControl. The controlled
authored operation is available for real authored callers; current input-color
construction does not need it.

Design validation compiled and ran a dependency-free safe Rust witness of this
exact two-anchor/context/continuation/digit-slot borrow shape with installed
Rust 1.97. The witness covered a live source bank refusing transition, numerical
phase overflow preserving the CSS checkpoint, dropped guards, refused early and
failed takes, explicit recovery, successful take, numerical-owner reuse while
a CSS receipt survives, and CSS-owner reuse after CSS borrows end. An additional
negative compilation kept a digit record live after removing its serializer;
the attempted new mutable numerical-owner borrow was rejected with E0499.
These checks establish the selected lifetime/transfer shape, not the future
public implementation's numerical correctness or actual parser accounting.

## Public use and HTML adapter

This native example is the intended Color-only public consumer. It is a future
API example, not an executed example at this source basis.

```rust
use surgeist_color::{Color, ColorSpace, PredefinedColorSpace, Transformation,
    SerializationLimits, SerializationRequest, serialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
let p3 = Color::try_new(
    ColorSpace::Predefined(PredefinedColorSpace::DisplayP3),
    [Some(1.2), Some(0.1), Some(-0.1)], Some(0.75),
)?;
let srgb = p3.converted(ColorSpace::Predefined(PredefinedColorSpace::Srgb))?;
let quantized = srgb.transformed(Transformation::QuantizeSrgb)?;
let text = serialize(quantized, SerializationRequest::Css,
    SerializationLimits { actions: 100_000, digits: 4_096, output_bytes: 1_024 })?;
assert!(!text.is_empty());
Ok(())
}
```

These numerical limits are caller choices, not product defaults. A render
consumer can use `srgb.channels()` and `alpha()` without serialization. Render
still owns missing-channel handling, clipping, f64→f32 lowering and its checked
`surgeist_render::Color::try_rgba`; its existing representation is unchanged.
Animation remains unchanged unless a concrete numerical consumer is separately
authorized. Neither crate becomes a mandatory path through the facade.

This future CSS example uses both public source and checked-authored paths:

```rust
use surgeist_css::{CssColor, CssHexColor, CssColorFactIdentity, CssColorFacts,
    CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact,
    CssColorInput, CssColorGrammarRequest, CssColorOrdinaryLimits,
    CssColorOrdinaryOutcome, CssAuthoredColorOutcome,
    parse_used_color, resolve_authored_color};

let facts = CssColorFacts::new(
    CssColorFactIdentity::new(7, 3), CssColorPropertyFacts::Unavailable,
    CssColorSchemeFact::Unavailable, &[], &[], &[], CssDeviceColorFact::Unknown);
let limits = CssColorOrdinaryLimits {
    input_units: 1_024, algorithm_records: 16_384,
    arithmetic_records: 4_096, evaluation_records: 4_096,
    actions: 4_000_000,
};
let original: Vec<u16> = "red".encode_utf16().collect();
let parsed = parse_used_color(CssColorInput::Utf16(&original),
    CssColorGrammarRequest::WholeColor, &facts, facts.identity(), limits).unwrap();
let authored = CssColor::from_hex(CssHexColor::try_new("ff0000").unwrap());
let resolved = resolve_authored_color(&authored, &facts, facts.identity(), limits).unwrap();
let CssColorOrdinaryOutcome::Used(from_source) = parsed.outcome() else { panic!("red") };
let CssAuthoredColorOutcome::Used(from_authored) = resolved.outcome() else { panic!("hex") };
assert!(from_source.identical(*from_authored));
assert!(std::ptr::eq(resolved.authored(), &authored));
```

These explicit fixture ceilings match the existing standalone color test's
logical allowances. The fact identity qualifies the supplied CSS envelope;
the native example requires none. Exact authored component construction uses
`CssComponentValue::try_number` and the existing checked scalar/function
constructors before this resolution call, retaining original tokens. The
authored path builds no source string.

The actual controlled HTML consumer is currently
[`construction/parser_session/input_color.rs`](../../crates/surgeist-html/src/construction/parser_session/input_color.rs).
After #1173/#1174/#1179 it belongs to `surgeist-html-parser`, which directly
depends on CSS and Color. The dependency-free `surgeist-html` shared-value
crate does not own a second adapter. Original-source, resource, lexical and
computation evidence stays in the parser owner, including the records currently
in
[`construction/parser_session/provenance.rs`](../../crates/surgeist-html/src/construction/parser_session/provenance.rs).
Stored numerical payloads use Color directly; CSS environment identities and
standard failures remain CSS types.

The public controlled path is `CssColorOperation::parse_used` with the actual
original UTF-16 buffer, actual CSS facts and existing ParseControl. Each action
is described, admitted through that ParseControl and then executed/read/written.
For a numerical transformation, the source-qualified CSS checkpoint delegates
to `Computation`; for serialization it delegates to `Serialization` and wraps
the numerical receipt with actual CSS provenance. The parser imports
`ColorSpace`, `PredefinedColorSpace`, `Transformation`, `SerializationRequest`
and Color from Color, and parsing/checkpoint types from CSS. This makes both
dependencies real while preserving the original control and source identity.

Preserve HTML's current dirty/default/current selection and standard-failure-only
opaque-black fallback. Preserve transformation order: force opaque when alpha
is disabled, convert to selected sRGB or Display-P3, quantize when the sRGB
branch requires bytes, then select the required modern family. The CSS numerical
action maps to actual kernel/layout admission before the operation. Serialized
ASCII units append to the parser's admitted UTF-16 output buffer. The parser
installs value/output/provenance only after complete receipt and all required
storage cleanup. ParseStop retains CSS operation errors and, where a direct
numerical operation is used, the typed Color error; cancellation/limits retain
their original stop. The fixed unavailable-environment identity in the existing
adapter qualifies that explicit environment policy, not an invented native
numerical source issuer.

This future public construction example uses that controlled implementation;
the parser owns ParseControl internally rather than asking its caller to create
one. The imports reflect the selected HTML split. The explicit finite limits
are the existing color-input fixture's allowances.

```rust
use surgeist_html::DocumentString;
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlSource, HtmlParseLimits,
    HtmlParseMetadataStep};
use surgeist_color::Color;

let limits = HtmlParseLimits::builder()
    .source_units(100_000).buffered_utf16_units(100_000)
    .token_attributes(10_000).auxiliary_entries(1_000_000)
    .managed_nodes(100_000).managed_edges(100_000).managed_attributes(100_000)
    .managed_utf16_units(100_000).source_map_entries(100_000)
    .source_map_payload_units(100_000).diagnostics(100_000)
    .diagnostic_payload_units(100_000).context_effects(100_000)
    .open_elements(100).tree_depth(100_000)
    .managed_capacity_bytes(100_000_000).peak_transient_bytes(100_000_000)
    .atomic_layout_bytes(100_000_000).work_credits(100_000_000).finish().unwrap();
let original = DocumentString::from_utf16(
    "<!doctype html><input type=color value=red>".encode_utf16().collect());
let request = HtmlDocumentParseRequest::offline(HtmlSource::utf16(original), limits);
let cancellation = request.cancellation().clone();
let report = request.parse().unwrap();
let expected = Color::from_srgba8([255, 0, 0, 255]);
assert_eq!(expected.srgb_bytes(), Some([255, 0, 0]));
let mut found = false;
let mut metadata = report.metadata();
loop {
    match metadata.advance() {
        HtmlParseMetadataStep::Record(record) if record.input_flags().is_some() => {
            assert_eq!(record.input_raw_value(), Some(&DocumentString::from("#ff0000")));
            found = true;
        }
        HtmlParseMetadataStep::Complete => break,
        _ => {}
    }
}
assert!(found);
drop(cancellation);
```

The cancellation handle can be used by the actual caller during parsing; merely
cloning it neither cancels nor replaces the request's controller. The parser's
public `HtmlParseColorComputation::value()` returns Color after extraction,
so callers can inspect the exact numerical payload alongside original-group,
operand-group, standard-failure and environment evidence. The native `expected`
value above does not manufacture that evidence.

## Material choices and delivery evidence

| Question | Selected result and decisive rejection |
| --- | --- |
| Which dependency direction preserves both native numerical use and CSS ownership? | CSS → Color. Reject Color → CSS, optional CSS integration features, formatter callbacks into CSS and a support crate: each defeats the selected zero-production-dependency numerical owner. |
| Is the boundary a CSS token component or a checked number? | Checked finite coordinates. Reject moving Number/Percentage/Degrees and exact expressions wholesale; that would create a second resolver or make native callers choose CSS grammar. |
| How does controlled work reuse the actual controller? | Concrete finite actions over caller storage, with no-op-on-drop guards. Reject an ordinary helper invoked after one blanket admission for a variable parse/format operation, or a privately invented controller. |
| Can the authored bridge reuse specified serialization? | No; feed original checked values into canonical recipe/resolver state. Stringification loses exact provenance and creates a second grammar/formatting pass. |
| Where do numerical formatting and byte-alpha decisions live? | One Color finite kernel/rounding policy and one byte-alpha operation; exact authored decimal machinery stays CSS. Reject duplication and f64 narrowing of authored exact coefficients. |
| Should numerical extraction replace backend/animation types? | No. Their valid states and precision policies belong to their consumers; no replacement or conversion shim is required by this extraction. |

The root manifest adds `crates/surgeist-color` and the intentional `color` facade
module; root `src/lib.rs` reexports the crate as a module, avoiding collision with
render's Color. CSS and the actual HTML parser adapter owner declare direct
path/version dependencies. Color has **zero production and build Cargo
dependencies**, including target-specific and optional declarations, across
every supported feature/target. Std is allowed. Do not add a build script or
feature to smuggle another owner into the graph. The root lockfile owns product
resolution; the API generator remains a separate workspace and discovers the
new crate through its existing target scan.

Move numerical unit tests, rustdoc examples and applicable source/license notices
with their implementation. Retain parser/resolver and exact numeric tests in
CSS; retain HTML input-color integration tests with their actual adapter owner.
Update root/crate AGENTS, README, source-edition ownership, legal notices and
documentation links to describe implemented boundaries after extraction. Refresh
`api/public-api.txt` and affected `api/crates/*.txt` through the root generator.
Do not edit generated audits by hand. Current layout/import origins do not
authorize acquiring new code or changing source pins.

#1185 prepares the seam while keeping the producer usable. #1188 extracts that
seam and integrates real consumers. #1186's alpha owner and #1187's resolver-state
cleanup remain required, independently schedulable. #1189 accepts their composed
result only after an independent review of the actual implementation basis.
HTML split coordination follows the adapter's actual availability; neither
whole parent is a blanket blocking dependency. Deferred fuzz campaigns #1172
are unchanged and no new campaigns are authorized here.

The focused acceptance evidence must cover:

- Color-only native compilation and construction/conversion/preparation/
  interpolation/transformation/serialization, plus dependency metadata and every
  manifest feature/target section proving the empty production/build graph.
- Current rectangular/HSL/HWB/Lab/naive-CMYK numerical oracles, extended finite
  values, signed zero, missing analogues, powerless hue, same-space identity,
  alpha/channel byte provenance, invalid native methods/progress and error order.
- `numeric_formatting` finite-bit tests: dyadic half neighbors, both signs,
  subnormals, exact 309-digit maximum integer, carries, zero classification and
  exact/one-short final bounds. Retain `bounded` wrong-reply/index/digit,
  refused append/replace, pending-byte and nonfinite cases against public APIs.
- All 256 byte alphas agree between specified and used serialization, while
  floating-alpha cases retain their separate results and identity.
- CSS source and checked-authored resolution, all native authored branches,
  exact coefficients beyond finite f64, Parsed/Components origins, complete
  unselected syntax validation, reached-only facts, prefix consumption and
  original UTF-16 coordinates. Check foreign/retired/incorrect slots, invalid
  replies, dropped actions and empty-bank checkpoint preconditions.
- General numeric projection and page/keyframe collision/suppression/shared
  budget tests, plus rotate shared budgets, no second traversal, zero-axis,
  negative-axis and restored-suppression regressions. Exact authored formatter
  tests stay in CSS and prove no precision narrowing.
- Real HTML input construction tests for standard-failure-only fallback,
  dirty/default/current selection, all transformation branches, original source
  attribution, one-short actual ledger resources, cancellation at parse,
  transform, formatting and output checkpoints, cleanup and atomic publication.
- Focused strict Clippy, formatting and facade checks, affected API audits and
  source scans proving no copied numerical/formatting/resolution algorithm,
  compatibility alias, forbidden dependency or unsafe implementation.

The prior 266-test review is baseline evidence only. This design document does
not claim those checks have run against the future public API or extracted crate.
