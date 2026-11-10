//! Interop module containing traits and types to ease integration between Boa
//! and Rust.

/// Internal module only.
pub(crate) mod private {
    /// Defines the supported argument signatures for Rust function pointers.
    pub trait IntoJsFunctionSealed<A, R> {}
}

/// Converts a typed Rust callback with explicitly traced captures to a native function.
///
/// The callback takes a shared reference to its captures before the converted
/// JavaScript arguments. Rust-owned mutable data can use shared interior mutability.
/// The result type must be `'static`.
///
/// ```
/// use surgeist_js::{Context, IntoJsFunctionWithCaptures, JsValue};
/// use std::{cell::Cell, rc::Rc};
/// use surgeist_js::{gc::heap::Trace, native_function::NativeCapture};
///
/// #[derive(Clone, Trace)]
/// struct Counter(#[heap_trace(skip)] Rc<Cell<i32>>);
///
/// impl NativeCapture for Counter {
///     type Stored = Self;
///     fn store(&self) -> Self { self.clone() }
///     fn restore(stored: &Self) -> Result<Self, surgeist_js::error::RootingError> {
///         Ok(stored.clone())
///     }
/// }
///
/// let mut context = Context::builder().build().unwrap();
/// let total = Counter(Rc::new(Cell::new(0)));
/// let add: fn(&Counter, i32) = |total, value| {
///     total.0.set(total.0.get() + value);
/// };
/// let add = add.into_js_function_with_captures(total.clone());
/// add.call(&JsValue::undefined(), &[JsValue::from(4)], context.access())
///     .unwrap();
/// assert_eq!(total.0.get(), 4);
/// ```
pub trait IntoJsFunctionWithCaptures<Args, Ret, Captures> {
    /// Converts the callback and its captures to a native function.
    fn into_js_function_with_captures(self, captures: Captures) -> NativeFunction;
}

/// Converts a typed function pointer to a native function.
///
/// The result type must be `'static`.
/// ```
/// # use surgeist_js::{IntoJsFunction, Context, JsValue, NativeFunction};
/// # let mut context = Context::builder().build().unwrap();
/// let f: fn(i32, i32) -> i32 = |a, b| a + b;
/// let f = f.into_js_function();
/// let result = f
///     .call(
///         &JsValue::undefined(),
///         &[JsValue::from(1), JsValue::from(2)],
///         context.access(),
///     )
///     .unwrap();
/// assert_eq!(result, JsValue::new(3));
/// ```
pub trait IntoJsFunction<Args, Ret>: private::IntoJsFunctionSealed<Args, Ret> + Copy {
    /// Converts the type into a JS function.
    fn into_js_function(self) -> NativeFunction;
}

mod into_js_arguments;
use crate::NativeFunction;
pub use into_js_arguments::*;

// Implement `IntoJsFunction` for functions with a various list of
// arguments.
mod into_js_function_impls;
