//! Implementations of the `IntoJsFunction` trait for various function signatures.

use super::private::IntoJsFunctionSealed;
use super::{IntoJsFunctionCopied, IntoJsFunctionWithCaptures};
use crate::interop::{JsRest, TryFromJsArgument};
use crate::{Context, NativeFunction, TryIntoJsResult};
use surgeist_js_gc::Trace;

/// A token to represent the context argument in the function signature.
/// This should not be used directly and has no external meaning.
#[derive(Debug, Copy, Clone)]
pub struct ContextArgToken;

macro_rules! impl_into_js_function {
    ($($id: ident: $t: ident),*) => {
        impl<$($t,)* R, T> IntoJsFunctionSealed<($($t,)*), R> for T
        where
            $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
            R: TryIntoJsResult,
            T: FnMut($($t,)*) -> R + 'static
        {}

        impl<$($t,)* R, T> IntoJsFunctionSealed<($($t,)* ContextArgToken,), R> for T
        where
            $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
            R: TryIntoJsResult,
            T: FnMut($($t,)* &mut Context) -> R + 'static
        {}

        impl<$($t,)* R, T> IntoJsFunctionSealed<($($t,)* JsRest<'_>,), R> for T
        where
            $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
            R: TryIntoJsResult,
            T: FnMut($($t,)* JsRest<'_>) -> R + 'static
        {}

        impl<$($t,)* R, T> IntoJsFunctionSealed<($($t,)* JsRest<'_>, ContextArgToken), R> for T
        where
            $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
            R: TryIntoJsResult,
            T: FnMut($($t,)* JsRest<'_>, &mut Context) -> R + 'static
        {}

        impl<$($t,)* R, Callback, Captures> IntoJsFunctionWithCaptures<($($t,)*), R, Captures> for Callback
        where
            $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
            R: TryIntoJsResult,
            Callback: Fn(&Captures, $($t,)*) -> R + Copy + 'static,
            Captures: Trace + 'static,
        {
            #[allow(unused_variables)]
            fn into_js_function_with_captures(self, captures: Captures) -> NativeFunction {
                NativeFunction::from_copy_closure_with_captures(
                    move |this, args, captures, ctx| {
                        let rest = args;
                        $(
                            let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                        )*
                        self(captures, $($id,)*).try_into_js_result(ctx)
                    },
                    captures,
                )
            }
        }

        impl<$($t,)* R, Callback, Captures> IntoJsFunctionWithCaptures<($($t,)* JsRest<'_>,), R, Captures> for Callback
        where
            $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
            R: TryIntoJsResult,
            Callback: Fn(&Captures, $($t,)* JsRest<'_>) -> R + Copy + 'static,
            Captures: Trace + 'static,
        {
            #[allow(unused_variables)]
            fn into_js_function_with_captures(self, captures: Captures) -> NativeFunction {
                NativeFunction::from_copy_closure_with_captures(
                    move |this, args, captures, ctx| {
                        let rest = args;
                        $(
                            let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                        )*
                        self(captures, $($id,)* rest.into()).try_into_js_result(ctx)
                    },
                    captures,
                )
            }
        }

        impl<$($t,)* R, Callback, Captures> IntoJsFunctionWithCaptures<($($t,)* ContextArgToken,), R, Captures> for Callback
        where
            $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
            R: TryIntoJsResult,
            Callback: Fn(&Captures, $($t,)* &mut Context) -> R + Copy + 'static,
            Captures: Trace + 'static,
        {
            #[allow(unused_variables)]
            fn into_js_function_with_captures(self, captures: Captures) -> NativeFunction {
                NativeFunction::from_copy_closure_with_captures(
                    move |this, args, captures, ctx| {
                        let rest = args;
                        $(
                            let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                        )*
                        self(captures, $($id,)* ctx).try_into_js_result(ctx)
                    },
                    captures,
                )
            }
        }

        impl<$($t,)* R, Callback, Captures> IntoJsFunctionWithCaptures<($($t,)* JsRest<'_>, ContextArgToken), R, Captures> for Callback
        where
            $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
            R: TryIntoJsResult,
            Callback: Fn(&Captures, $($t,)* JsRest<'_>, &mut Context) -> R + Copy + 'static,
            Captures: Trace + 'static,
        {
            #[allow(unused_variables)]
            fn into_js_function_with_captures(self, captures: Captures) -> NativeFunction {
                NativeFunction::from_copy_closure_with_captures(
                    move |this, args, captures, ctx| {
                        let rest = args;
                        $(
                            let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                        )*
                        self(captures, $($id,)* rest.into(), ctx).try_into_js_result(ctx)
                    },
                    captures,
                )
            }
        }

        impl<$($t,)* R, T> IntoJsFunctionCopied<($($t,)*), R> for T
        where
            $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
            R: TryIntoJsResult,
            T: Fn($($t,)*) -> R + 'static + Copy,
        {
            #[allow(unused_variables)]
            fn into_js_function_copied(self, _context: &mut Context) -> NativeFunction {
                let s = self;
                NativeFunction::from_copy_closure(move |this, args, ctx| {
                    let rest = args;
                    $(
                        let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                    )*
                    let r = s( $($id,)* );
                    r.try_into_js_result(ctx)
                })
            }
        }

        impl<$($t,)* R, T> IntoJsFunctionCopied<($($t,)* JsRest<'_>,), R> for T
        where
            $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
            R: TryIntoJsResult,
            T: Fn($($t,)* JsRest<'_>) -> R + 'static + Copy,
        {
            #[allow(unused_variables)]
            fn into_js_function_copied(self, _context: &mut Context) -> NativeFunction {
                let s = self;
                NativeFunction::from_copy_closure(move |this, args, ctx| {
                    let rest = args;
                    $(
                        let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                    )*
                    let r = s( $($id,)* rest.into() );
                    r.try_into_js_result(ctx)
                })
            }
        }

        impl<$($t,)* R, T> IntoJsFunctionCopied<($($t,)* ContextArgToken,), R> for T
        where
            $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
            R: TryIntoJsResult,
            T: Fn($($t,)* &mut Context) -> R + 'static + Copy,
        {
            #[allow(unused_variables)]
            fn into_js_function_copied(self, _context: &mut Context) -> NativeFunction {
                let s = self;
                NativeFunction::from_copy_closure(move |this, args, ctx| {
                    let rest = args;
                    $(
                        let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                    )*
                    let r = s( $($id,)* ctx);
                    r.try_into_js_result(ctx)
                })
            }
        }

        impl<$($t,)* R, T> IntoJsFunctionCopied<($($t,)* JsRest<'_>, ContextArgToken), R> for T
        where
            $($t: for<'a> TryFromJsArgument<'a> + 'static,)*
            R: TryIntoJsResult,
            T: Fn($($t,)* JsRest<'_>, &mut Context) -> R + 'static + Copy,
        {
            #[allow(unused_variables)]
            fn into_js_function_copied(self, _context: &mut Context) -> NativeFunction {
                let s = self;
                NativeFunction::from_copy_closure(move |this, args, ctx| {
                    let rest = args;
                    $(
                        let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                    )*
                    let r = s( $($id,)* rest.into(), ctx);
                    r.try_into_js_result(ctx)
                })
            }
        }
    };
}

// Currently implemented up to 12 arguments. The empty argument list
// is implemented separately above.
// Consider that JsRest and JsThis are part of this list, but Context
// is not, as it is a special specialization of the template.
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
