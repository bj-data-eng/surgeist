//! Dependency-free finite numerical color values and controlled operations.
//!
//! Every numerical operation is total over proof-typed operands. [`Finite`]
//! and [`UnitInterval`] reject NaN and both infinities when they are built;
//! a result beyond binary64 is the signed closest finite value.
//!
//! ```
//! use surgeist_color::{Color, ColorSpace, Finite, PredefinedColorSpace, UnitInterval};
//! let srgb = ColorSpace::Predefined(PredefinedColorSpace::Srgb);
//! let red = Color::new(srgb, [Finite::new(f64::MAX), Finite::new(0.0), None], UnitInterval::new(1.0));
//! let xyz = red.converted(ColorSpace::Predefined(PredefinedColorSpace::XyzD65));
//! assert_eq!(xyz.channels()[0], Some(f64::MAX));
//! ```
//!
//! A proof has no public constructor other than its checked `new`:
//!
//! ```compile_fail,E0423
//! let finite = surgeist_color::Finite(1.0);
//! ```
//!
//! ```compile_fail,E0423
//! let unit = surgeist_color::UnitInterval(2.0);
//! ```
//!
//! A color takes proven channels, never raw `f64` values:
//!
//! ```compile_fail,E0308
//! use surgeist_color::{Color, ColorSpace, UnitInterval};
//! let color = Color::new(ColorSpace::Lab, [Some(50.0), None, None], UnitInterval::new(1.0));
//! ```
//!
//! The fallible raw constructor no longer exists:
//!
//! ```compile_fail,E0599
//! use surgeist_color::{Color, ColorSpace};
//! let color = Color::try_new(ColorSpace::Lab, [Some(50.0), None, None], Some(1.0));
//! ```
//!
//! # Controlled operations
//!
//! [`Computation`], [`Serialization`] and [`format::FiniteFormatter`] are
//! typestates over an exclusive [`OperationOwner`]. Every transition consumes
//! its continuation. `step` yields exactly one pending action, which exposes
//! only its own completion method, or the completed receipt. The caller admits
//! each action's [`ActionLayout`] and owns every [`DigitSlot`] and output byte:
//!
//! ```
//! use surgeist_color::{
//!     Color, DigitSlot, OperationOwner, Serialization, SerializationAction as Action,
//!     SerializationRequest, SerializationStep,
//! };
//! let mut owner = OperationOwner::new();
//! let red = Color::from_srgba8([255, 0, 0, 255]);
//! let mut digits: Vec<DigitSlot<'_>> = Vec::new();
//! let mut output = String::new();
//! let mut serialization = Serialization::new(&mut owner, red, SerializationRequest::Css);
//! let receipt = loop {
//!     let action = match serialization.step() {
//!         SerializationStep::Complete(receipt) => break receipt,
//!         SerializationStep::Pending(action) => action,
//!     };
//!     serialization = match action {
//!         Action::Compute(compute) => compute.compute(),
//!         Action::ReadDigit(read) => {
//!             let slot = &digits[read.index()];
//!             read.read(slot)
//!         }
//!         Action::AppendDigit(write) => {
//!             let (record, receipt) = write.record();
//!             digits.push(record);
//!             receipt.stored(&digits[digits.len() - 1])
//!         }
//!         Action::ReplaceDigit(write) => {
//!             let index = write.index();
//!             let (record, receipt) = write.record();
//!             digits[index] = record;
//!             receipt.stored(&digits[index])
//!         }
//!         Action::Emit(emit) => {
//!             output.push(char::from(emit.unit()));
//!             emit.emitted()
//!         }
//!     };
//! };
//! assert_eq!(output, "rgb(255, 0, 0)");
//! assert_eq!(receipt.length(), output.len());
//! drop(digits);
//! // The receipt and the dropped records no longer borrow the owner.
//! let _next = Serialization::new(&mut owner, receipt.value(), SerializationRequest::HtmlCompatible);
//! ```
//!
//! A record from another operation or index panics when it is read or
//! acknowledged, as slice indexing does. The remaining misuses do not compile.
//! A computation is consumed by computing it:
//!
//! ```compile_fail,E0382
//! use surgeist_color::{Color, Computation, NumericalRequest, OperationOwner};
//! let mut owner = OperationOwner::new();
//! let request = NumericalRequest::WithAlpha { value: Color::opaque_black(), alpha: None };
//! let computation = Computation::new(&mut owner, request);
//! let _value = computation.compute();
//! let _kernel = computation.kernel();
//! ```
//!
//! No slot-based `take` exists; the result comes only from `compute`:
//!
//! ```compile_fail,E0599
//! use surgeist_color::{Color, Computation, NumericalRequest, OperationOwner};
//! let mut owner = OperationOwner::new();
//! let request = NumericalRequest::WithAlpha { value: Color::opaque_black(), alpha: None };
//! let _value = Computation::take(&mut Some(Computation::new(&mut owner, request)));
//! ```
//!
//! A pending digit read has no emission acknowledgement:
//!
//! ```compile_fail,E0599
//! fn misuse(read: surgeist_color::ReadDigit<'_>) {
//!     let _serialization = read.emitted();
//! }
//! ```
//!
//! An emission acknowledges the byte its action issued; the caller supplies none:
//!
//! ```compile_fail,E0061
//! fn misuse(emit: surgeist_color::Emit<'_>) {
//!     let _serialization = emit.emitted(b'#');
//! }
//! ```
//!
//! A live digit record keeps its owner borrowed, so no new operation starts:
//!
//! ```compile_fail,E0499
//! use surgeist_color::{
//!     Color, OperationOwner, Serialization, SerializationAction as Action, SerializationRequest,
//!     SerializationStep,
//! };
//! let mut owner = OperationOwner::new();
//! let red = Color::from_srgba8([255, 0, 0, 255]);
//! let mut serialization = Serialization::new(&mut owner, red, SerializationRequest::Css);
//! let digit = loop {
//!     serialization = match serialization.step() {
//!         SerializationStep::Pending(Action::Compute(compute)) => compute.compute(),
//!         SerializationStep::Pending(Action::Emit(emit)) => emit.emitted(),
//!         SerializationStep::Pending(Action::AppendDigit(write)) => break write.record().0,
//!         _ => panic!("a coefficient digit precedes every read"),
//!     };
//! };
//! let second = Serialization::new(&mut owner, red, SerializationRequest::Css);
//! drop(digit);
//! drop(second);
//! ```
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]
mod finite;
pub use finite::{Finite, UnitInterval};
mod spaces;
pub use spaces::*;
mod value;
pub use value::{Color, SerializationFamily, Transformation};
mod interpolation;
mod serialization;
pub use serialization::SerializationRequest;
mod protocol;
pub use protocol::{ActionLayout, DigitSlot, Kernel, OperationOwner};
mod controlled;
pub use controlled::{
    Computation, DigitReceipt, Emit, NumericalRequest, ReadDigit, Serialization,
    SerializationAction, SerializationCompute, SerializationError, SerializationLimits,
    SerializationReceipt, SerializationResource, SerializationStep, WriteDigit, serialize,
};
mod alpha;
pub mod format;
pub mod named;
pub use alpha::byte_alpha_for_serialization;
mod rectangular_color_conversion;
pub use rectangular_color_conversion::{RectangularCoordinates, RectangularSpace};
mod hsl_color_conversion;
pub use hsl_color_conversion::{
    HslCoordinates, HwbCoordinates, convert_hsl_to_srgb, convert_hwb_to_srgb, convert_srgb_to_hsl,
    convert_srgb_to_hwb,
};
mod lab_color_conversion;
pub use lab_color_conversion::{
    PolarCoordinates, convert_lab_to_lch, convert_lch_to_lab, convert_oklab_to_oklch,
    convert_oklch_to_oklab,
};
mod naive_color_conversion;
pub use naive_color_conversion::{naively_convert_cmyk_to_srgba, naively_convert_srgba_to_cmyk};
