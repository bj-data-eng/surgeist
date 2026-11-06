//! Adapt typed Rust function pointers to JavaScript native callbacks.

use super::private::IntoJsFunctionSealed;
use super::{IntoJsFunction, IntoJsFunctionWithCaptures};
use crate::interop::{JsRest, TryFromJsArgument};
use crate::{Context, NativeFunction, TryIntoJsResult};
use surgeist_js_gc::{Finalize, Trace};

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
            #[derive(Trace, Finalize)]
            struct Function<$($t,)* R> {
                // This concrete pointer carries no captures.
                #[unsafe_ignore_trace]
                callback: fn($($t,)* $($tail),*) -> R,
            }

            #[derive(Trace, Finalize)]
            struct StatefulFunction<$($t,)* R, Captures: Trace> {
                // All traceable state resides in captures.
                #[unsafe_ignore_trace]
                callback: fn(&Captures, $($t,)* $($tail),*) -> R,
                captures: Captures,
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
                        |this, args, state, $context| {
                            let $rest = args;
                            $(
                                let ($id, $rest) = $t::try_from_js_argument(this, $rest, $context)?;
                            )*
                            let result = (state.callback)($($id,)* $($argument),*);
                            result.try_into_js_result($context)
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
                Captures: Trace + 'static,
            {
                #[allow(unused_variables)]
                fn into_js_function_with_captures(self, captures: Captures) -> NativeFunction {
                    NativeFunction::from_fn_with_captures(
                        |this, args, state, $context| {
                            let $rest = args;
                            $(
                                let ($id, $rest) = $t::try_from_js_argument(this, $rest, $context)?;
                            )*
                            let result = (state.callback)(&state.captures, $($id,)* $($argument),*);
                            result.try_into_js_result($context)
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
        impl_signature!(($($id:$t),*); (ContextArgToken); (&mut Context); (context); rest, context);
        impl_signature!(($($id:$t),*); (JsRest<'_>); (JsRest<'_>); (rest.into()); rest, context);
        impl_signature!(($($id:$t),*); (JsRest<'_>, ContextArgToken);
            (JsRest<'_>, &mut Context); (rest.into(), context); rest, context);
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
