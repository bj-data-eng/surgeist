//! Adapt typed Rust function pointers to JavaScript native callbacks.

use super::private::IntoJsFunctionSealed;
use super::{IntoJsFunction, IntoJsFunctionWithCaptures};
use crate::ContextAccess;
use crate::interop::{JsRest, TryFromJsArgument};
use crate::native_function::NativeCapture;
use crate::{NativeFunction, TryIntoJsResult};
use surgeist_js_gc::heap::Trace;

/// Distinguishes signatures with an engine context from converted JS arguments.
#[derive(Debug, Copy, Clone)]
pub struct ContextArgToken;

macro_rules! impl_signature {
    (
        ($($id:ident : $t:ident),*);
        ($($marker:ty),*);
        ($($tail:ty),*);
        ($($argument:expr),*);
        $rest:ident, $context:ident
    ) => {
        const _: () = {
            #[derive(Trace)]
            struct Function<$($t,)* R> {
                // This concrete pointer carries no captures.
                #[heap_trace(skip)]
                callback: fn($($t,)* $($tail),*) -> R,
            }

            impl<$($t: 'static,)* R: 'static> NativeCapture for Function<$($t,)* R> {
                type Stored = Self;

                fn store(&self) -> Self {
                    Self { callback: self.callback }
                }

                fn restore(stored: &Self) -> Result<Self, crate::error::RootingError> {
                    Ok(Self { callback: stored.callback })
                }
            }

            struct StatefulFunction<$($t,)* R, Captures> {
                callback: fn(&Captures, $($t,)* $($tail),*) -> R,
                captures: Captures,
            }

            #[derive(Trace)]
            struct StoredStatefulFunction<$($t,)* R, Captures: NativeCapture> {
                #[heap_trace(skip)]
                callback: fn(&Captures, $($t,)* $($tail),*) -> R,
                captures: Captures::Stored,
            }

            impl<$($t: 'static,)* R: 'static, Captures: NativeCapture> NativeCapture
                for StatefulFunction<$($t,)* R, Captures>
            {
                type Stored = StoredStatefulFunction<$($t,)* R, Captures>;

                fn store(&self) -> Self::Stored {
                    StoredStatefulFunction {
                        callback: self.callback,
                        captures: self.captures.store(),
                    }
                }

                fn restore(stored: &Self::Stored) -> Result<Self, crate::error::RootingError> {
                    Ok(Self {
                        callback: stored.callback,
                        captures: Captures::restore(&stored.captures)?,
                    })
                }
            }

            impl<$($t,)* R> IntoJsFunctionSealed<($($t,)* $($marker,)*), R>
                for fn($($t,)* $($tail),*) -> R
            where
                $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
                R: TryIntoJsResult + 'static,
            {}

            impl<$($t,)* R> IntoJsFunction<($($t,)* $($marker,)*), R>
                for fn($($t,)* $($tail),*) -> R
            where
                $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
                R: TryIntoJsResult + 'static,
            {
                #[allow(unused_variables)]
                fn into_js_function(self) -> NativeFunction {
                    NativeFunction::from_fn_with_captures(
                        |this, args, state, mut $context| {
                            let $rest = args;
                            $(
                                let ($id, $rest) = $t::try_from_js_argument(this, $rest, $context.reborrow())?;
                            )*
                            let result = (state.callback)($($id,)* $($argument),*);
                            result.try_into_js_result($context.reborrow())
                        },
                        Function { callback: self },
                    )
                }
            }

            impl<$($t,)* R, Captures> IntoJsFunctionWithCaptures<($($t,)* $($marker,)*), R, Captures>
                for fn(&Captures, $($t,)* $($tail),*) -> R
            where
                $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
                R: TryIntoJsResult + 'static,
                Captures: NativeCapture,
            {
                #[allow(unused_variables)]
                fn into_js_function_with_captures(self, captures: Captures) -> NativeFunction {
                    NativeFunction::from_fn_with_captures(
                        |this, args, state, mut $context| {
                            let $rest = args;
                            $(
                                let ($id, $rest) = $t::try_from_js_argument(this, $rest, $context.reborrow())?;
                            )*
                            let result = (state.callback)(&state.captures, $($id,)* $($argument),*);
                            result.try_into_js_result($context.reborrow())
                        },
                        StatefulFunction { callback: self, captures },
                    )
                }
            }
        };
    };
}

macro_rules! impl_into_js_function {
    ($($id:ident: $t:ident),*) => {
        impl_signature!(($($id:$t),*); (); (); (); rest, context);
        impl_signature!(($($id:$t),*); (ContextArgToken); (ContextAccess<'_>); (context.reborrow()); rest, context);
        impl_signature!(($($id:$t),*); (JsRest<'_>); (JsRest<'_>); (rest.into()); rest, context);
        impl_signature!(($($id:$t),*); (JsRest<'_>, ContextArgToken);
            (JsRest<'_>, ContextAccess<'_>); (rest.into(), context.reborrow()); rest, context);
    };
}

// Support up to twelve converted arguments and optional rest and context.
impl_into_js_function!();
impl_into_js_function!(a: A);
impl_into_js_function!(a: A, b: B);
impl_into_js_function!(a: A, b: B, c: C);
impl_into_js_function!(a: A, b: B, c: C, d: D);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F, g: G);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K, l: L);
