# Numerical color extraction contract

This reference defines the CSS → Color boundary for
[#1184](https://github.com/bj-data-eng/surgeist/issues/1184), under
[#1183](https://github.com/bj-data-eng/surgeist/issues/1183). It is an implementation
contract; `surgeist-color` now exists and `surgeist-css` consumes it. Its
source basis is
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

CSS source paths in this section are relative to
[`crates/surgeist-css/src/`](../../crates/surgeist-css/src); Color source paths
are relative to [`crates/surgeist-color/src/`](../../crates/surgeist-color/src).
Public CSS items are named with their module, as in the
[CSS audit](../../api/crates/surgeist-css.txt).

| Owner/path and actual caller | Owner and contract |
| --- | --- |
| Color `spaces.rs`: `PredefinedColorSpace`, `ColorSpace`, `HueInterpolation`; used by CSS grammar, channels, relative bindings, conversion, facts and serialization | Color owns the numerical `PredefinedColorSpace`, `ColorSpace`, `HueInterpolation`. CSS reads authored names and retains authored omission; there is one numerical identity per space and no CSS mirror enum. |
| Color `rectangular_color_conversion.rs` and its `colorimetry`, `transfer`, `wide` modules; reached from CSS through `Color::converted` in `color/resolution.rs` and `color/syntax/serialization.rs` | Color owns `RectangularSpace`, `RectangularCoordinates` and the finite conversion implementation, including wide intermediate handling and missing-coordinate rules. |
| Color `hsl_color_conversion.rs`, `lab_color_conversion.rs`, `naive_color_conversion.rs`; CSS calls the naive CMYK conversions from `color/device.rs` | Color owns the checked coordinate carriers and pure conversions. CSS keeps the decision to invoke a conversion and its authored range/serialization phase. |
| Color `value.rs`: `Color`; used by CSS `color/channels.rs`, `relative.rs`, `interpolate.rs`, `resolution.rs`, `device.rs`, `profile_conversion.rs`, the `color/controlled/` checkpoints and HTML | Color owns `Color`, numerical transformations, representation identity, alpha replacement and the black/white contrast calculation. |
| `color/channels.rs`: `Component`, `Domain`, `legacy_rgb_domains`, `scalar`, `hue`, `alpha`, `absolute_coordinates`; reached from `component_value.rs` and `resolution.rs` | CSS owns Number/Percentage/Degrees/Missing grammar, unit scaling, calculation-root NaN/infinity handling, range clamping and absolute/relative policy. Only checked finite coordinates cross to Color. `Component` does not become a Color type. |
| `color/interpolate.rs`: calls to `Color::prepared_for_interpolation` and `Color::interpolated`; used by `relative::Binding::new`, `resolution.rs` and folds | Color owns numerical preparation and two-color interpolation. CSS owns mix traversal, exact weight evaluation, omitted-weight normalization and source-ordered fold scheduling. |
| `color/interpolate.rs`: `WeightSummary`, `Weights`, `MixFold` | The CSS mix recipe state stays in CSS. Its fold calls Color's two-color operation once per reached operand; Color exports no variadic mix interpreter. |
| `color/relative.rs`: `Binding`, `channel`, `coordinates`, `alpha_only`; `expression.rs` variable evaluation and `resolution.rs` frames | CSS owns environment/channel-name bindings, RGB ×255 expression units, missing-to-zero inside calculations, expression execution, inherited/overridden alpha, source-selected family and reached branching. It calls Color's preparation/conversion/alpha/family methods. |
| `color/device.rs`: `ComputedCmyk::new`, `DeviceResolution`; `profile_conversion.rs`: input collection, reverse checks and conversion-record lookup | CSS owns syntax, resource identity, channel names/order, fact matching, capability/fallback selection and the unsupported reverse/custom branches. Color owns only the context-free naive CMYK numerical conversion and replacement of resolved alpha on a supplied numerical result. No ICC engine exists. |
| `color/facts.rs`, `palette.rs`, `profile.rs`; `resolution.rs` callers | CSS owns current/system color, scheme, profile/device facts, identities, duplicate/foreign checks and reached-only lookup. Fact payloads use `Color` directly. |
| `color/authored.rs`: `Events`, `Builder`; called by `parser/color.rs` | CSS owns canonical checked-authored construction, exact component tokens, original origins and closure recovery. `color/authored_resolution/` feeds the borrowed authored graph to resolution (`color::CssColor::resolve`); the authored value is never serialized and parsed again. |
| `color/grammar.rs`, `tokens.rs`, `literal.rs`, `decimal.rs`, `syntax_pass.rs`, `component.rs`, `component_value.rs`, `expression.rs`, `operation.rs`, `resolution.rs` | CSS owns original source decoding, tokens, grammar, exact numeric admission/evaluation, resolver state and errors. Numerical requests contain fixed resolved operands, not any of these records. |
| `color/controlled/protocol.rs`: the mixed `Data` record, owner/phase/bank/index qualification (`color::controlled::CssColorOperationOwner`, `CssColorBank`, `CssColorSlot`); `color/controlled/bounded.rs`: the typestate `color::controlled::CssColorParse`, its action guards and the `CssUsedColorCheckpoint` | CSS owns these records. They hold no numerical algorithm and no format-digit variant. Color digit slots occupy a distinct caller-owned numerical bank and borrow a separately lent Color owner. CSS has three parse-bank identities (`CssColorBank::{Algorithm, Arithmetic, Evaluation}`); no CSS-private union is Color's public protocol. |
| Serialization of a used color (CSS holds none); `color::controlled::CssUsedColorCheckpoint::value()` yields the `Color` | Color owns the resolved serialization plan, finite digit work and byte emission. A consumer drives `surgeist_color::Serialization` on that value with its own digit and output storage; CSS defines no serialization wrapper. `color::parse_used_color` and `color::CssColor::resolve` return a `Color` (inside `CssColorOrdinaryOutcome::Used` or `CssAuthoredColorOutcome::Used`) that `surgeist_color::serialize` accepts directly. |
| `serialization/numeric_formatting.rs` (finite portion); used by resolved/ordinary color and non-color projection | Color owns the single finite-bit formatter, digit continuation, rounded-millionths operation and finite zero classification (`format::format_finite`, `format::rounds_to_zero`). Exact decimal parsing, representation and specified-value policy stay CSS. |
| `serialization/exact_decimal.rs`; used by exact rationals and finite formatting | Color owns the small `RoundingMode` with the shared tie decision. CSS exact-decimal arithmetic calls `RoundingMode::rounds_up`; its arbitrary-precision storage and digit scans stay CSS. |
| `color/syntax/serialization.rs`, which calls Color's `byte_alpha_for_serialization` | One Color `byte_alpha_for_serialization(u8) -> f64` supplies the byte-alpha path. Floating alpha never gains byte provenance by numerical equality. |

This table includes the private crossings requiring a public seam. Internal
matrix arithmetic, transfer functions, exact decimal registers, lexical tables,
tokenizer states and resolver frames remain private to their selected owner.
CSS keeps `color::CssColor`, every authored function/profile/expression
carrier, `color::CssColorSyntax`, relative-channel names and authored
interpolation wrappers. `color::CssColorInterpolationMethod` retains authored
hue omission (`hue()` is the authored hue, `effective_hue()` the effective one)
and holds Color's space/hue identities; it lowers to the checked numerical
`InterpolationMethod`. Its construction error is `color::CssColorConstructionError`,
while invalid native numerical method construction belongs to Color.

## Native numerical front door

The following lists the public API of the numerical front door, abridged from
the `surgeist-color` audit,
[`api/crates/surgeist-color.txt`](../../api/crates/surgeist-color.txt), which
carries every signature. `surgeist-css` holds no mirror implementation of any
of it. Fields of invariant-bearing structs are private.

```rust
// surgeist_color
pub struct Finite(/* private */);          // finite binary64
impl Finite {
    pub const fn new(value: f64) -> Option<Self>;
    pub const fn get(self) -> f64;
}
pub struct UnitInterval(/* private */);    // finite value in [0, 1]
impl UnitInterval {
    pub const fn new(value: f64) -> Option<Self>;
    pub fn clamped(value: Finite) -> Self;
    pub const fn get(self) -> f64;
}
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
    pub const fn try_new(space: ColorSpace, hue: Option<HueInterpolation>)
        -> Result<Self, InterpolationError>;
    pub const fn space(self) -> ColorSpace;
    pub const fn hue(self) -> Option<HueInterpolation>;
}
pub enum SerializationFamily { Natural, ColorFunction, LegacySrgb }
pub enum Transformation {
    ForceOpaqueAlpha, Convert(ColorSpace), QuantizeSrgb, ColorFunction,
}
pub struct Color { /* space, coordinates, alpha, byte backing, family */ }
impl Color {
    pub fn new(space: ColorSpace, channels: [Option<Finite>; 3], alpha: Option<UnitInterval>)
        -> Self;
    pub const fn from_srgba8(value: [u8; 4]) -> Self;
    pub const fn opaque_black() -> Self;
    pub const fn space(&self) -> ColorSpace;
    pub fn channels(&self) -> [Option<f64>; 3];
    pub fn finite_channels(&self) -> [Option<Finite>; 3];
    pub const fn alpha(&self) -> Option<f64>;
    pub fn unit_alpha(&self) -> Option<UnitInterval>;
    pub const fn srgb_bytes(&self) -> Option<[u8; 3]>;
    pub const fn alpha_byte(&self) -> Option<u8>;
    pub fn serialization_family(&self) -> SerializationFamily;
    pub fn identical(self, other: Self) -> bool;
    pub fn converted(self, destination: ColorSpace) -> Self;
    pub fn prepared_for_interpolation(self, destination: ColorSpace) -> Self;
    pub fn interpolated(self, second: Self, method: InterpolationMethod, progress: UnitInterval)
        -> Self;
    pub fn transformed(self, transformation: Transformation) -> Self;
    pub fn with_alpha(self, alpha: Option<UnitInterval>) -> Self;
    pub const fn with_color_function(self) -> Self;
    pub fn to_legacy_srgb(self) -> Self;
    pub fn black_white_contrast(self) -> Self;
}
pub enum SerializationRequest { Css, HtmlCompatible }
pub struct SerializationLimits {
    pub actions: usize, pub digits: usize, pub output_bytes: usize,
}
pub fn serialize(color: Color, request: SerializationRequest, limits: SerializationLimits)
    -> Result<String, SerializationError>;
pub fn byte_alpha_for_serialization(alpha: u8) -> f64;
```

`PredefinedColorSpace`, `ColorSpace`, `HueInterpolation`, `RectangularSpace`
and the three error enums (`InterpolationError`, `SerializationError` and
`format::FormatError`) are `#[non_exhaustive]`; `SerializationRequest`,
`Transformation` and `SerializationFamily` are closed. These are pre-v1
breaking moves, with no old-path aliases or public CSS reexports of numerical
names. `Finite`, `UnitInterval`, `Color` and the coordinate carriers are
`Clone + Copy + Debug + PartialEq`; they do not derive `Eq`, `Ord` or `Hash`
over floating values. `identical` compares coordinate and alpha bits,
missingness, actual byte backing, space and effective serialization family;
ordinary `PartialEq` compares the stored values numerically, so it treats equal
signed zeros as equal.

Every operation above is total. Invalid numbers cannot reach one: `Finite::new`
returns `None` for NaN and both infinities, and `UnitInterval::new` returns
`None` outside `[0,1]` (so also for NaN and the infinities). A caller meets the
refusal where it builds the operand, and the operations that follow cannot
fail. `UnitInterval::new` admits negative zero and keeps its bits;
`UnitInterval::clamped` takes a `Finite` to the nearest value in `[0,1]`. There
is no raw-`f64` constructor of `Color` and no error type for color
construction, conversion or transformation. A value that would leave binary64
is returned as the signed closest finite value (see below).

`Color::new` takes `Finite` channels and an optional `UnitInterval` alpha. It
preserves signed zero and all finite extended channels, including signed
chroma/saturation, and neither clamps color channels nor infers byte backing
from integral floating-point values. `None` is an explicit missing
channel/alpha, not omitted syntax: CSS resolves omitted authored alpha to
`Some(1.0)` before construction. `Color::channels` and `alpha` give plain
numbers for arithmetic and display; `finite_channels` and `unit_alpha` give the
same values back as proofs for another operation. `Color::new` selects the
color-function family for a predefined space and the natural family for every
other space; `from_srgba8` establishes actual channel and alpha byte backing
and the legacy `rgb()` family. `serialization_family` reports `ColorFunction`
when the modern marker is set, otherwise `LegacySrgb` for sRGB and `Natural`
for every other space.

| Space | Ordered numerical units |
| --- | --- |
| Predefined RGB / XYZ | R,G,B / X,Y,Z normalized reference numbers; 100% is 1 |
| HSL | hue degrees, saturation percentage reference number, lightness percentage reference number; 100% is 100 |
| HWB | hue degrees, whiteness and blackness percentage reference numbers |
| Lab / LCH | native CIE lightness (usual 0–100), a/b or nonnegative converter chroma and degree hue |
| Oklab / Oklch | normalized lightness (usual 0–1), native a/b or converter chroma and degree hue |

The numerical `Color` carrier admits finite signed polar coordinates required
by resolved relative colors. Dedicated `HslCoordinates`, `HwbCoordinates` and
`PolarCoordinates` are Color's carriers for the HSL, HWB and Lab/Oklab
conversions. They hold `Finite` coordinates, so their constructors and the
conversion functions are total and define no conversion error types.
`RectangularCoordinates` has `new`, `space`, `channels`, and `convert_to`. The
public functions
`convert_hsl_to_srgb`, `convert_srgb_to_hsl`, `convert_hwb_to_srgb`,
`convert_srgb_to_hwb`, `convert_lab_to_lch`, `convert_lch_to_lab`,
`convert_oklab_to_oklch`, `convert_oklch_to_oklab`,
`naively_convert_cmyk_to_srgba` and `naively_convert_srgba_to_cmyk` take and
return `Finite` coordinates.

Same-space conversion returns the original value bit-for-bit. Cross-space
conversion preserves extended finite coordinates and alpha byte backing,
reusing the current missing-to-zero conversion rules and polar reconstruction;
it does not promise preservation of missing masks across all spaces. The
separate preparation operation preserves analogous missing components and
applies the existing powerless-hue thresholds only when converting spaces.
Signed polar chroma and HSL saturation use the existing opposite-hue Cartesian
adapter. Arithmetic does not fail merely because a wider intermediate exceeds
finite f64: only the final channel is limited, and a channel whose exact value
is beyond binary64 is the signed closest finite value (`f64::MAX` with the
sign), the CSS Values 4 closest supported value. This is a numeric-range rule
and applies no gamut mapping.

`InterpolationMethod::try_new` is the one fallible Color constructor: it
rejects a hue strategy on a rectangular space with
`InterpolationError::HueRequiresPolarSpace` and defaults polar omission to
`Shorter`; its `hue()` returns the effective strategy. Progress is a
`UnitInterval`, so its range is established by the type and no extrapolation is
possible. The two-color operation prepares the first and second operands in
that order, carries analogous missing components, uses the existing
premultiplied-alpha calculation with hue excluded, preserves both-missing
components, applies the selected hue arc and keeps the interpolated alpha in
`[0,1]`. CSS's ordered mix fold retains the existing zero-total two-operand
progress of 0.5, omitted-weight normalization and final alpha multiplier; it
requests the color-function family after the fold. These steps do not become an
alternative native CSS resolver.

`ForceOpaqueAlpha` preserves channel backing and replaces alpha by unity: byte
alpha becomes byte 255, and floating or missing alpha becomes 1.0. `with_alpha`
preserves channel backing and intentionally makes alpha floating.
`QuantizeSrgb` converts a value in another space to sRGB, clamps present
coordinates to `[0,1]`, rounds positive half ties to bytes and preserves
missingness: the channels become sRGB byte channels only when all three are
present, and a missing channel keeps the value floating. It neither changes
alpha nor infers alpha bytes. `ColorFunction` (the `Transformation` variant and
`with_color_function`) selects modern serialization without a conversion, and
the marker survives later conversions, including to cylindrical spaces.
`to_legacy_srgb` converts to sRGB and selects the legacy `rgb()` family,
clearing that marker; `Transformation` has no legacy variant.
`black_white_contrast` converts to XYZ-D65, reads Y (zero when missing) clamped
to `[0,1]`, compares the white and black contrast ratios, ignores alpha,
chooses white on ties and returns actual opaque sRGB bytes. It selects no new
contrast or gamut algorithm.

## Controlled numerical execution

Color must support the original parser controller without depending on that
controller. It publishes concrete operations, actions and opaque digit records;
it does not publish a trait that claims admission happened elsewhere. Three
continuations exist: `Computation` for one fixed numerical request,
`Serialization` for a resolved color's text and `format::FiniteFormatter` for
one finite number. Each is a typestate over an exclusive `OperationOwner`:
every transition consumes the continuation and returns the next state, so a
used or completed operation cannot be driven again, and dropping a
continuation or one of its actions abandons it without further numerical work.

```rust
// surgeist_color, abridged from the audit
pub struct OperationOwner { /* exclusive monotonic phase */ }
pub struct DigitSlot<'owner> { /* owner, phase, index, checked digit */ }
pub enum NumericalRequest {
    Construct { space: ColorSpace, channels: [Option<Finite>; 3], alpha: Option<UnitInterval> },
    Convert { value: Color, destination: ColorSpace },
    Prepare { value: Color, destination: ColorSpace },
    Interpolate { first: Color, second: Color, method: InterpolationMethod, progress: UnitInterval },
    WithAlpha { value: Color, alpha: Option<UnitInterval> },
    WithColorFunction { value: Color },
    ToLegacySrgb { value: Color },
    BlackWhiteContrast { value: Color },
    Transform { value: Color, transformation: Transformation },
}
impl NumericalRequest {
    pub const fn kernel(self) -> Kernel;
    pub fn execute(self) -> Color;
}
pub struct Computation<'owner> { /* fixed registers only */ }
impl<'owner> Computation<'owner> {
    pub fn new(owner: &'owner mut OperationOwner, request: NumericalRequest) -> Self;
    pub const fn kernel(&self) -> Kernel;
    pub const fn layout(&self) -> ActionLayout;
    pub fn compute(self) -> Color;
    pub const fn into_request(self) -> NumericalRequest;
}
pub struct Serialization<'owner> { /* fixed plan and finite cursor only */ }
impl<'owner> Serialization<'owner> {
    pub fn new(owner: &'owner mut OperationOwner, value: Color, request: SerializationRequest)
        -> Self;
    pub fn step(self) -> SerializationStep<'owner>;
}
pub enum SerializationStep<'owner> {
    Pending(SerializationAction<'owner>), Complete(SerializationReceipt),
}
pub enum SerializationAction<'owner> {
    Compute(SerializationCompute<'owner>), ReadDigit(ReadDigit<'owner>),
    AppendDigit(WriteDigit<'owner>), ReplaceDigit(WriteDigit<'owner>), Emit(Emit<'owner>),
}
pub struct ActionLayout {
    pub read_bytes: usize, pub write_bytes: usize, pub output_bytes: usize,
}
```

`OperationOwner::new()` is allocation-free (`OperationOwner` also implements
`Default`). `Computation::new(&mut owner, request)` and
`Serialization::new(&mut owner, value, request)` begin a checked phase of the
owner and copy fixed operands without executing them. A computation observes no
digit record and emits nothing, so `Computation::compute` executes the whole
request. It returns the `Color` directly, with the same kernels as the
`Color` methods and no error: every request is total, and
`NumericalRequest::execute` runs the same request without a continuation.
`Computation::into_request` declines the computation and returns the unexecuted
request. The public low-level coordinate operations are likewise total over
`Finite` operands, and a controlled caller admits their fixed operands and
registers before invoking them, with no variable scans hidden inside.

`Kernel` names the work an action performs. For a computation it is the actual
fixed request (`Construct`, `Convert(space)`, `Prepare(space)`, `Interpolate`,
`Transform(transformation)`, `WithAlpha`, `WithColorFunction`, `ToLegacySrgb` or
`BlackWhiteContrast`). For serialization it is `SerializationPlan`,
`SerializationPiece` or, while a number is formatted, a finite-formatting
transition: `PlanFinite`, `TrimMillionths`, `ExtractDigit`, `MultiplyDigit`,
`NextMultiply`, `PlanOutput` or `PlanEmission`. A `FiniteFormatter` issues those
same transitions. Scalar library arithmetic remains an atomic bounded
operation; the contract does not invent internal instruction counts.

`Serialization::step(self)` consumes the serialization and yields either
`SerializationStep::Pending` with exactly one `SerializationAction` or
`Complete` with a `SerializationReceipt`. An action exposes `layout()` without
inspecting variable payloads or doing numerical work, and each variant exposes
only its own completion, so a completion of the wrong kind is a compile error:

| Action | Inspection | Completion, returning the next `Serialization` |
| --- | --- | --- |
| `Compute(SerializationCompute)` | `kernel()` | `compute()` runs the advertised fixed kernel |
| `ReadDigit(ReadDigit)` | `index()` | `read(&DigitSlot)` observes one qualified record |
| `AppendDigit`, `ReplaceDigit` (`WriteDigit`) | `index()` | `record()` mints the `DigitSlot` and a `DigitReceipt`; the receipt's `stored(&DigitSlot)` acknowledges the caller's actual storage |
| `Emit(Emit)` | `unit()` | `emitted()` acknowledges the ASCII byte the action issued |

`SerializationAction::into_serialization` and `DigitReceipt::into_serialization`
decline an action, or report refused storage, without effect: the returned
continuation waits at the same action, and a refused write can be serviced again
against the same state. `FiniteFormatter` has the same shape in the `format`
module, as `FormatStep`, `FormatAction`, `FormatCompute`, `FormatReadDigit`,
`FormatWriteDigit`, `FormatDigitReceipt` and `FormatEmit`, with
`into_formatter` for declining, and completes with a `FormatSummary`.

The caller admits the fixed owner and register layout before construction and
every action's `ActionLayout` before completing it, then admits the actual
payload reads, work, allocation or capacity, writes and output of that action.
`OperationOwner::layout_bytes()`, `Computation::register_bytes()`,
`Serialization::register_bytes()` and `action_bytes()`, and the corresponding
`FiniteFormatter` functions are associated functions callable before any
instance exists. They and `ActionLayout` report actual compiled `size_of`
values, including record wrappers and alignment, never constants copied from
the mixed CSS record. A digit read observes a `DigitSlot` and its owner
qualification, a digit write mints one `DigitSlot`, an emission writes one
output byte, and a computation does none of these. `register_bytes()` includes
retained plans and cursors; caller wrappers add their own storage and
checkpoint fields once.

`DigitSlot` contains only numerical ownership, phase, index and digit; it has no
CSS source or fact identity. Pointer and phase qualification is safe Rust and
uses no global IDs or unsafe code. `OperationOwner` privately holds a
nonzero-sized marker and a checked `usize` phase. Constructing a continuation
takes `&'owner mut OperationOwner`, begins the next phase and stores
`&'owner OperationOwner`; each `DigitSlot` stores that same shared reference,
the phase and its index. There is no interior-mutable numerical owner reachable
through a CSS checkpoint. The exclusive borrow prevents concurrent new
operations while a continuation or any record is live, and the phase check
rejects retired or foreign records supplied through a public action. A computed
`Color` and a `SerializationReceipt` (`value()` and `length()`, no source or fact
fields and no borrowed digits) no longer borrow the owner, so the owner serves
the next operation once the caller drops the digit records.

A mismatch between a record and the pending action is a defect in the driving
code and panics, as slice indexing does, rather than returning an error:
reading a record that another operation or index minted panics, and so does
acknowledging storage with such a record or with one holding a different digit.
The check covers only the record's operation, index and digit. It cannot tell
whether the caller stored the minted record or another of the same identity,
and reading back a different stored record yields wrong output that the fixed
registers cannot detect. An owner that has begun `usize::MAX` operations also
panics; no input reaches that count. Declining an action leaves the same action
pending, so a caller whose admission refuses it can stop and drop the
continuation. A caller that must retry starts a new operation from the original
color and request, which begins a new phase.

Serialization appends scratch digits at the actual bank length, updates only an
existing qualified index, and never owns or reserves caller buffers. The bank
index continues across the numbers of one color, so the digits a color retains
are the total of its appends. At most 309 coefficient digits are needed for one
finite binary64 value; the total digit admission is the actual reported append
sequence, not a blanket 309-record promise. The caller reuses scratch only
after dropping the records, and no action releases capacity or caller ledger
charges. The consumer owns its output buffer and frees its exact owned storage;
`Serialization` performs no cleanup.

Admission refusal and cancellation are caller outcomes, not numerical
invalidity. The caller's first stop remains sticky; cleanup attempts every owned
release without polling or replacing that stop. Before the stop, an
unacknowledged action can still be completed. After it, the caller drops the
action and releases its storage; restarting uses a new phase and the retained
original request or checkpoint. A local output prefix can exist after
acknowledged bytes, but is never a successful public result. A refused
serialization discards it. Only the complete receipt and fully built output may
be installed. Ordinary `serialize` services the same actions with local
vectors, checked logical action, digit and output ceilings and fallible
allocation. Its `SerializationError` distinguishes `Limit` (a
`SerializationResource` of `Actions`, `Digits` or `OutputBytes`, with the
required and allowed counts), `Allocation` (the resource whose allocation
failed) and `CountOverflow`. Those ordinary ceilings do not claim physical
allocation accounting or manufacture an HTML controller.

## One finite formatter and one rounding policy

Color exposes `format::RoundingMode::{AwayFromZero, TowardPositiveInfinity}`,
`format::format_finite(value: Finite, rounding: RoundingMode, byte_limit: usize)
-> Result<String, FormatError>`, and `format::rounds_to_zero(value: Finite,
rounding: RoundingMode) -> bool`. The `Finite` operand carries the finiteness
proof, so neither function validates its input. `FormatError` is
`#[non_exhaustive]` and distinguishes `ByteLimit` and `CapacityOverflow`.
`RoundingMode::rounds_up(self, ordering: std::cmp::Ordering, negative: bool) ->
bool` owns the shared tie rule and is reused by CSS's exact arithmetic. It
increments above half; at exact half it increments always for AwayFromZero and
only for nonnegative values for TowardPositiveInfinity.

`format::FiniteFormatter<'owner>` uses `OperationOwner`, `DigitSlot` and the
same typestate contract as Serialization, described under
[controlled numerical execution](#controlled-numerical-execution). Its
constructor `new(owner, value: Finite, rounding)` takes the value and rounding;
`planned_length() -> Option<usize>` reports the exact output length once
planning has run, before any emission; `step()` yields a `FormatStep` that is
either one pending `FormatAction` or `Complete` with a `FormatSummary`, whose
`is_zero()` and `emitted()` accessors report the zero classification and the
count of acknowledged bytes. The formatter shares its arithmetic with complete
color serialization; Color keeps that arithmetic private. One kernel formats
the actual finite bits to at most six fractional places, strips trailing zeros,
emits positive zero, expands finite integers exactly, and preflights final
output rather than a temporary shortest-decimal spelling. No shortest-string
conversion or float rounding replacement is allowed.

CSS reaches the formatter through three functions in
`serialization/numeric_formatting.rs`. `format_projected_number` and
`format_ordinary_color_number` build a `Finite` from the projected scalar and
call `format::format_finite` with `AwayFromZero` and `TowardPositiveInfinity`
respectively; they map `FormatError::ByteLimit` and `CapacityOverflow` to the
CSS serialization errors of the same names and any other error to
`UnrepresentableValue`. `projected_number_rounds_to_zero` calls
`format::rounds_to_zero` with `AwayFromZero`. These wrappers retain no rounding
arithmetic. The specified-value serializer in `color/syntax/serialization.rs`
formats ordinary color numbers through `format_ordinary_color_number`, but its
byte-alpha spelling passes `byte_alpha_for_serialization` to CSS's own
`format_binary64` in `serialization/specified_serialization.rs`, which does not
call Color's formatter.

CSS retains `LexicalDecimal`, arbitrary-length coefficient/exponent handling,
exact rationals, specified shifts/sign inversion, projection work and the exact
decimal digit emitter. That emitter accepts already-rounded exact decimal
digits; it is not a second finite-bit formatter. It never feeds an authored
coefficient through f64 merely to reuse Color. The existing authored endpoints,
coefficients beyond finite f64 and discarded-exponent behavior remain exact.

| Finite consumer | Owner/policy and regression consequence |
| --- | --- |
| Used-color output of `color::controlled::CssUsedColorCheckpoint` and `color::CssColorOrdinaryOutcome::Used` | Color serializer (`surgeist_color::Serialization`), ties toward positive infinity, caller-owned digit/write actions; preserve one-short and pending-byte behavior. |
| `serialization/specified_serialization.rs::format_binary64`, called for byte alpha by `color/syntax/serialization.rs` and for scalar text by `numeric/projection.rs` | CSS specified color policy keeps its own nearest-decimal selection, taking the numerically greater candidate at an exact decimal midpoint, on the post-scale binary result; exact color calculations retain their own rational phase. It does not call Color's formatter. |
| `numeric/projection.rs` finite coefficient serialization | Color finite formatting with away-from-zero ties; CSS retains unit/suffix limits, expression visits and projection charges. |
| `rule/keyframes/page_keyframe_serialization.rs` computed selector percentages | Same projected formatter and away-from-zero ties; preserve rounded selector collisions, occurrence order, suppressed output and shared writer budgets. Page rules themselves retain their existing nonnumeric writer behavior. |
| `values/transform/rotate.rs` projected axis zero classification, via `projected_number_rounds_to_zero` in `serialization/numeric_formatting.rs` | Color `rounds_to_zero(_, AwayFromZero)`; CSS retains axis/identity decisions, exact authored sign inversion, traversal and suppression. No rendering-to-string probe is introduced. Actual projected output goes through the existing numeric projection path. |
| `serialization/numeric_formatting.rs` ordinary projected/ordinary-color wrappers and tests | Finite tests live with the kernel in Color; CSS retains error translation, exact decimal tests and policy selection. |

The exact authored decimal kernel remains CSS: `format_css_number` and
`format_css_number_with_sign` (both in `serialization/numeric_formatting.rs`)
serve `serialization/specified_serialization.rs`, `numeric/specified_numeric.rs`,
`numeric/value.rs`, `numeric/projection.rs`, `color/syntax/serialization.rs`,
`values/transform/transform_value_serialization.rs` and exact-number branches of
`values/transform/rotate.rs`. `serialization/exact_decimal.rs` and
`serialization/specified_serialization.rs` also use the already-rounded exact
decimal emitter. These paths keep their storage and specified-value policy in
CSS. `values/image/image_serialization.rs` consumes numerical space identities
through CSS image policy, importing `surgeist_color::ColorSpace`; image
serialization stays in CSS.

`byte_alpha_for_serialization` uses the existing integer two/three-decimal
Color 4 algorithm for every byte: try nearest hundredth and retain it if its
round trip recovers the byte, otherwise nearest thousandth. Color's serializer
feeds this result, for the legacy `rgba()` spelling, into the shared
positive-tie finite formatter; the specified
serializer feeds it to CSS's `format_binary64`, which takes the greater candidate
at an exact decimal midpoint.
Bytes 236 and 237 produce 0.925 and 0.93 respectively. Actual floating alpha is
formatted from its own bits; it is not reclassified as byte alpha. Pinned source
edition/prose discrepancies stay documented with their existing selected policy.

## CSS resolution and provenance

CSS resolves color through three public fronts in
[`surgeist_css::color`](../../crates/surgeist-css/src/color/): the ordinary
source front `parse_used_color`, the ordinary authored front `CssColor::resolve`
and the caller-serviced source protocol `controlled::CssColorParse`. Every
numerical step they take is a total `surgeist-color` operation over finite
proof types, so none of them reports a numerical failure. The controlled
authored-color protocol that `CssColor::resolve` drives is crate-private; no
consumer services it.

### Source resolution

`color::parse_used_color(input: CssColorInput, request: CssColorGrammarRequest,
facts: &CssColorFacts, expected: CssColorFactIdentity, limits:
CssColorOrdinaryLimits) -> Result<CssColorOrdinaryResult, CssColorResolveError>`
returns a source-qualified result whose Used payload is Color.
`CssColorInput::Utf8` bytes and `CssColorInput::Utf16` units remain borrowed
original input, and the result borrows that input and the facts through
`input()` and `facts()`. `source()` returns the `CssColorSourceDescriptor`
(input kind and length, grammar request, fact identity and consumed range) and
`outcome()` returns `CssColorOrdinaryOutcome`: `Used(Color)` or
`StandardFailure(CssColorStandardFailure)`, whose `kind` is a
`CssColorStandardFailureKind` and whose `source` is the original
`CssColorInputRange`. Consumed ranges are original units, and
`CssColorGrammarRequest::PrefixColor` retains its actual end boundary.
Syntax validation visits complete unselected branches, while facts and numeric
evaluation occur only on reached branches. Source decoding
(`CssColorError::malformed_input_offset`), source-derived standard failures and
reached fact or capability failures are separate.

`CssColorOrdinaryLimits` names `input_units`, `algorithm_records`,
`arithmetic_records`, `evaluation_records` and `actions`; every field value is
valid, zero refuses, and a refusal is a `CssColorResolveError` with
`exhausted_limit()`, never a standard color failure. `Default` admits 1,024
input units and limits the records and actions to four times the worst
requirement per unit of the characterization corpus's color cases, scaled to
that length (275,295 algorithm records, 3,476 arithmetic records, 3,476
evaluation records, 2,036,468,185 actions). The margin holds for the corpus,
not for every input of that length, and the actions ceiling is a coarse work
bound: many extreme literals such as `1e308` can need far more work than any
corpus input before the refusal. Callers with longer inputs, more extreme
literals or a tighter work budget pass explicit limits, raising the record and
action ceilings together with `input_units`. The ordinary driver admits the input
length, counts each serviced action once, admits each bank's required length
before an append and reports a refused allocation. It creates no hidden
controller and retires its banks before it returns.

### Authored resolution

`CssColor::resolve(&self, facts: &CssColorFacts, expected: CssColorFactIdentity,
limits: CssAuthoredColorOrdinaryLimits) -> Result<CssAuthoredColorResult,
CssColorResolveError>` resolves a checked authored color to a used color
without serialization, source reconstruction or a source string. The result
borrows the actual authored root and facts and exposes `authored()`, `facts()`
and `outcome() -> &CssAuthoredColorOutcome`, where that enum contains
`Used(Color)` or `StandardFailure(CssAuthoredColorStandardFailure)`.
`CssAuthoredColorStandardFailure` has public fields `kind`
(`CssColorStandardFailureKind`), `path` (`CssColorComponentPath`) and `origin`
(`Option<CssValueOrigin>`). A scalar or component that retains an origin
reports it; a programmatic hex, name or root without one reports `None` rather
than an invented origin. No input bytes, source offsets or source or fact
identity are invented: an authored failure carries a component path, not a
source range.

`CssColorComponentPath` owns a checked sequence of `CssColorPathStep` values,
with `steps() -> &[CssColorPathStep]` for inspection and `capacity()` and
`capacity_bytes()` for the allocation it owns. `CssColorPathStep` is
`#[non_exhaustive]`; its current steps are `Source`,
`Channel(usize)`, `Alpha`, `First`, `Second`, `MixColor(usize)`,
`MixWeight(usize)` and `ExpressionOperand(usize)`: `Source` denotes the source
color of a relative or `alpha()` color or the operand of `contrast-color()`;
`First` and `Second` denote the light and dark branches of `light-dark()`;
indices use the current branch's original child, channel or operand order. Only
the CSS resolver constructs paths, and it checks each step against its actual
root. The path allocation moves into a failure without a clone. Neither the
path nor a failure that contains it is `Clone`, so the outcome is borrowed from
the result.

`CssAuthoredColorOrdinaryLimits` contains `algorithm_records`,
`arithmetic_records`, `evaluation_records`, `path_steps` and `actions`.
`path_steps` limits the path allocation's required and actual capacity,
independently of active depth; the three bank ceilings likewise observe actual
retained capacity. `CssColorOrdinaryResource::PathSteps` names the path
resource, and authored paths never consume `InputUnits`. The driver counts each
action once and checks requested and observed capacity even after an
allocation failure or overshoot. It creates no second controller or hidden
tree-size budget, and it retires its banks and path buffer before it returns,
moving any published failure path into the result. `Default` gives the same
record and action ceilings as `CssColorOrdinaryLimits::default()` and 1,024
`path_steps`, one per source unit of the 1,024-unit default length; the fields
stay public, zero refuses, and callers with larger sources pass explicit
limits.

### Errors

`CssColorError` is the reached semantic failure of one color parse. Exactly
one accessor returns a value: `unavailable()` (`CssColorUnavailable`, for
example `Scheme` for an absent scheme fact under `light-dark()`),
`unsupported()` (`CssColorUnsupported`), `fact_defect()`
(`CssColorFactDefect`, including `Foreign` for a fact envelope that is not the
expected identity) and `malformed_input_offset()`. `CssColorResolveError` is
the ordinary fronts' error: it wraps a `CssColorError` (`color_error()`) or
reports an exhausted ordinary resource (`exhausted_limit()`) or a refused
allocation (`failed_allocation()`), both naming a `CssColorOrdinaryResource`.
Both implement `Display` and `Error` and keep their typed cause.

CSS owns grammar, invalid and foreign fact, unavailable, unsupported,
exact-expression and source errors. Color owns numerical construction,
interpolation, conversion and formatting, and those operations are total, so
no numerical failure exists to wrap. A mismatched slot, index or digit is a
defect in the driving code: it panics with a documented message and has no
error variant. A standard grammar failure is a successful outcome, not an
error; it alone can mint the fallback checkpoint described below. Missing
facts, unsupported conversion, resource refusal and cancellation cannot.

### Controlled source parse

`color::controlled::CssColorParse<'owner, 'facts>` is the pending source
parse. The caller owns the exclusive `CssColorOperationOwner` anchor, the three
banks (`CssColorBank::{Algorithm, Arithmetic, Evaluation}`) of
`CssColorSlot` records and their admission:

```rust,ignore
let mut parse = CssColorParse::start(
    &mut owner, kind, length, request, &facts, expected,
)?; // Err(CssColorError): fact_defect() == Some(CssColorFactDefect::Foreign)
let outcome = loop {
    match parse.step() {
        CssColorParseStep::Complete(outcome) => break outcome,
        CssColorParseStep::Pending(action) => {
            // admit action.layout(); a refusal drops the action and abandons the parse
            parse = service(action)?; // each action has one completion method
        }
    }
};
```

`step(self)` consumes the parse and yields either `CssColorParseStep::Pending`
with exactly one `CssColorParseAction` guard or `Complete` with a
`CssColorParseOutcome`. The guards are `ReadInput` (`offset()`, `read(unit)`),
`ReadSlot` (`bank()`, `index()`, `read(&CssColorSlot)`), `AppendSlot` and
`UpdateSlot` (`CssColorWriteSlot`: `record()` mints a `CssColorSlot` and a
`CssColorSlotReceipt`; `stored(&slot)` acknowledges storage and `into_parse()`
reports refused storage), `ReadFact` (`fact()`, `read()`) and `Compute`
(`kernel()`, `compute()`). Each guard exposes only its own completion, so a
wrong completion is a compile error. A semantic failure consumes the parse with
a `CssColorError`; dropping a parse or a guard abandons it, and
`CssColorParseAction::into_parse` declines an action without effect and leaves
the parse pending. Nothing executes abandoned work or refunds storage.

`CssColorSlot` is an opaque record only CSS can mint. It borrows the owner, so
the owner serves a new parse only after the caller drops the earlier parse's
slots; the outcome borrows only the facts. Reading or storing a record of
another parse, bank or index panics, as slice indexing does. The compile-fail
doctests on `CssColorParse` pin the rejected misuse (use after `step`, a second
`record`, a second parse while a slot lives, a slot outliving its owner).

Admission stays with the consumer. `CssColorParse::initialization_layout()`,
`register_bytes()` and `action_bytes()` report the static fixed layouts,
`bank_length(bank)` the current bank length, and `CssColorParseAction::layout()`
a `CssColorActionLayout` of the input, slot, fact-record and numerical payload
extents plus the `CssColorKernel` the action runs. The consumer admits these
and its own vector capacity, allocation and copy events; CSS never converts a
layout into a guessed weight. Numerical digit storage belongs to Color's
serialization, not to the CSS slots.

### Checkpoints, transformation and serialization

A completed parse is `CssColorParseOutcome::Used(CssUsedColorCheckpoint)` or
`StandardFailure(CssColorFailureCheckpoint)`, both borrowing only `'facts`. A
used checkpoint exposes `value()`, `consumed()`, `source()`, `fact_identity()`,
`facts()` and `standard_failure()`, which returns the failure an explicit
fallback replaced or `None` when the color parsed directly. A failure
checkpoint exposes `failure()` and `fallback(self, Color) ->
CssUsedColorCheckpoint`, the only way to turn a standard failure into a used
checkpoint; it takes a Color the consumer chose.

`CssUsedColorCheckpoint::transform(self, owner: &mut
surgeist_color::OperationOwner, transformation: Transformation) ->
CssColorTransform` starts one numerical transformation. `CssColorTransform`
wraps `surgeist_color::Computation`, so it has no error: `kernel()`,
`layout()` and the static `register_bytes()` support admission,
`compute(self)` returns a `CssUsedColorCheckpoint` whose value is the
transformed Color, and `into_checkpoint(self)` declines it. The checkpoint's
value can therefore come only from CSS resolution, an explicit fallback on a
standard failure or a CSS-issued transformation, which keeps the provenance
guarantee. Only a used checkpoint transforms. Transformation order and the
choice of fallback belong to the consumer.

CSS does not wrap serialization. The consumer serializes `checkpoint.value()`
with `surgeist_color::Serialization::new(&mut OperationOwner, value,
SerializationRequest)`, drives its `step()` and `SerializationAction` guards
with its own caller-owned `DigitSlot` bank, and keeps `standard_failure()`,
`source()` and `consumed()` from the checkpoint for publication. The ordinary
`surgeist_color::serialize(value, request, limits)` serves callers without a
controller. The two anchors are separate: `CssColorOperationOwner` serves the
source parse, and a Color `OperationOwner` serves transformation and
serialization. Digit records borrow the numerical owner, so a new numerical
operation cannot start while old digits live.

The real caller sequence is:

1. Create the CSS and numerical owners as separate stack locals. Start
   `CssColorParse::start(&mut css_owner, ...)` and service its three banks
   through the consumer's controller. Drop a refused action to abandon the
   parse, or call `into_parse` to decline it and leave the parse pending.
2. On `Complete(Used(checkpoint))`, drop the source banks. On
   `Complete(StandardFailure(failure))`, either report the failure or call
   `failure.fallback(color)` to continue with a used checkpoint.
3. For each transformation, pass `&mut numerical_owner` to
   `checkpoint.transform(..)`, admit `layout()`, call `compute()` and keep the
   returned checkpoint. The numerical borrow ends with the call to `compute()`.
4. Serialize `checkpoint.value()` with Color's `Serialization` over the
   numerical owner and a caller-owned digit bank. Drop the digit bank before
   the numerical owner serves another operation.
5. Reusing the CSS owner for a new parse requires dropping every old CSS slot.

The HTML parser does not invoke the ordinary helpers. Its source adapter
(`construction/css_color.rs`) starts `CssColorParse::start` and services the
parse with its own ParseControl; `construction/input_color.rs` then
transforms and serializes through the Color types above. Authored callers use
`CssColor::resolve`; current input-color construction does not need it.

## Public use and HTML adapter

This native example is the intended Color-only public consumer.

```rust
use surgeist_color::{Color, ColorSpace, Finite, PredefinedColorSpace, SerializationLimits,
    SerializationRequest, Transformation, UnitInterval, serialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p3 = Color::new(
        ColorSpace::Predefined(PredefinedColorSpace::DisplayP3),
        [Finite::new(1.2), Finite::new(0.1), Finite::new(-0.1)],
        UnitInterval::new(0.75),
    );
    let srgb = p3.converted(ColorSpace::Predefined(PredefinedColorSpace::Srgb));
    let quantized = srgb.transformed(Transformation::QuantizeSrgb);
    let text = serialize(quantized, SerializationRequest::Css,
        SerializationLimits { actions: 100_000, digits: 4_096, output_bytes: 1_024 })?;
    assert!(quantized.srgb_bytes().is_some());
    assert_eq!(quantized.alpha_byte(), None);
    assert!(text.starts_with("color(srgb ") && text.ends_with(" / 0.75)"));
    Ok(())
}
```

`Finite::new` and `UnitInterval::new` return the `Option` operands that
`Color::new` takes, and each is `Some` here because its argument is finite or
within `[0,1]`. A missing channel is spelled `None`, not passed as NaN. The
channel values lie outside the sRGB gamut, so `converted` returns extended sRGB
coordinates without clamping them. `QuantizeSrgb` clamps those coordinates to `[0,1]` and
rounds them to bytes, so the color gains sRGB byte channels while its alpha
stays floating at 0.75 and gains no alpha byte. Because the original value was
built in a predefined space, it carries the color-function family through both
steps, and the Css request spells it as `color(srgb ...)` rather than `rgb()`.
Only two steps can refuse: building an operand, where `Finite::new` or
`UnitInterval::new` returns `None`, and `serialize`, which can refuse a limit or
an allocation.

These numerical limits are caller choices, not product defaults. A render
consumer can use `srgb.channels()` and `alpha()` without serialization. Render
still owns missing-channel handling, clipping, f64→f32 lowering and its checked
`surgeist_render::Color::try_rgba`; its existing representation is unchanged.
Animation remains unchanged unless a concrete numerical consumer is separately
authorized. Neither crate becomes a mandatory path through the facade.

This CSS example uses both public source and checked-authored paths. All items
are in `surgeist_css::color`:

```rust
use surgeist_css::color::{CssColor, CssHexColor, CssColorFactIdentity, CssColorFacts,
    CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact,
    CssColorInput, CssColorGrammarRequest, CssColorOrdinaryLimits,
    CssColorOrdinaryOutcome, CssAuthoredColorOutcome, CssAuthoredColorOrdinaryLimits,
    parse_used_color};

let facts = CssColorFacts::new(
    CssColorFactIdentity::new(7, 3), CssColorPropertyFacts::Unavailable,
    CssColorSchemeFact::Unavailable, &[], &[], &[], CssDeviceColorFact::Unknown);
let limits = CssColorOrdinaryLimits::default();
let original: Vec<u16> = "red".encode_utf16().collect();
let parsed = parse_used_color(CssColorInput::Utf16(&original),
    CssColorGrammarRequest::WholeColor, &facts, facts.identity(), limits).unwrap();
let authored = CssColor::from_hex(CssHexColor::try_new("ff0000").unwrap());
let authored_limits = CssAuthoredColorOrdinaryLimits::default();
let resolved = authored.resolve(&facts, facts.identity(), authored_limits).unwrap();
let CssColorOrdinaryOutcome::Used(from_source) = parsed.outcome() else { panic!("red") };
let CssAuthoredColorOutcome::Used(from_authored) = resolved.outcome() else { panic!("hex") };
assert!(from_source.identical(*from_authored));
assert!(std::ptr::eq(resolved.authored(), &authored));
```

The default ceilings cover short color text like the fixture; a caller with
longer inputs passes explicit limits. The fact identity qualifies the supplied
CSS envelope; the native example requires none. Exact authored component construction uses
`component::CssComponentValue::try_number` and the existing checked scalar/function
constructors before this resolution call, retaining original tokens. The
authored path builds no source string.

The actual controlled HTML consumer is currently
[`construction/input_color.rs`](../../crates/surgeist-html-parser/src/construction/input_color.rs),
which drives the CSS parse through the shared action servicing in
[`construction/css_color.rs`](../../crates/surgeist-html-parser/src/construction/css_color.rs).
It belongs to `surgeist-html-parser`, which directly depends on CSS and Color.
The `surgeist-html` shared-value
crate does not own a second adapter. Original-source, resource, lexical and
computation evidence stays in the parser owner, including the records currently
in
[`construction/provenance.rs`](../../crates/surgeist-html-parser/src/construction/provenance.rs).
Stored numerical payloads use Color directly; CSS environment identities and
standard failures remain CSS types.

The public controlled path is `color::controlled::CssColorParse`. The parser
starts it with `CssColorParse::start(&mut owner, kind, length, request, facts,
expected)` over an exclusive `CssColorOperationOwner`, the length of the actual
original UTF-16 buffer and actual CSS facts, and drives it with its existing
ParseControl. `step()` consumes the parse and yields either one pending
`CssColorParseAction` guard or the `CssColorParseOutcome`. Each action is
described by its layout, admitted through that ParseControl and then executed,
read or written by the guard's single completion method, which returns the
parse; dropping a guard or the parse abandons it. The parser owns the three slot
banks. A used color is a `CssUsedColorCheckpoint`; a standard failure is a
`CssColorFailureCheckpoint` whose `fallback(value)` supplies the consumer's used
color. For a numerical transformation,
`CssUsedColorCheckpoint::transform(&mut OperationOwner, Transformation)` yields
a total `CssColorTransform` that wraps `Computation`, and `compute()` returns the
next checkpoint. CSS does not wrap serialization: the parser drives
`surgeist_color::Serialization` directly over the checkpoint's `value()` with
its own `DigitSlot` bank and keeps the checkpoint's source descriptor and
`standard_failure()` as provenance. The parser imports `ColorSpace`,
`PredefinedColorSpace`, `Transformation`, `SerializationRequest`, `Serialization`
and Color from Color, and the parse and checkpoint types from
`surgeist_css::color::controlled`. This makes both dependencies real while
preserving the original control and source identity.

Preserve HTML's current dirty/default/current selection and standard-failure-only
opaque-black fallback. Preserve transformation order: force opaque when alpha
is disabled, convert to selected sRGB or Display-P3, quantize when the sRGB
branch requires bytes, then select the required modern family. The CSS action
layout maps to actual kernel/layout admission before the action. Serialized
ASCII units append to the parser's admitted UTF-16 output buffer. The parser
installs value/output/provenance only after complete receipt and all required
storage cleanup. ParseStop retains `CssColorError` and, where serialization
is driven directly, the typed Color `SerializationError`; cancellation/limits retain
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
  alpha/channel byte provenance, rejected native methods, and the `Finite` and
  `UnitInterval` constructors' refusal of NaN, infinities and out-of-range alpha.
- `numeric_formatting` finite-bit tests: dyadic half neighbors, both signs,
  subnormals, exact 309-digit maximum integer, carries, zero classification and
  exact/one-short final bounds. Retain `bounded` wrong-reply/index/digit,
  refused append/replace and pending-byte cases against public APIs.
- All 256 byte alphas agree between specified and used serialization, while
  floating-alpha cases retain their separate results and identity.
- CSS source and checked-authored resolution, all native authored branches,
  exact coefficients beyond finite f64, Parsed/Programmatic origins, complete
  unselected syntax validation, reached-only facts, prefix consumption and
  original UTF-16 coordinates. The typestate rejects repeated steps,
  completions of the wrong action kind and use of an abandoned parse at compile
  time (the `compile_fail` rustdoc cases on `color::controlled::CssColorParse`);
  the same rustdoc shows the owner reused once the records are dropped.
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

The prior 266-test review is baseline evidence only. This document does not
claim those checks have run against the current Color API.
