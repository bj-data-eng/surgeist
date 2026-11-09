//! Implementation of [`TryFromJs`] for [`Either`].
//!
//! This will try to deserialize for the [`Either::Left`] type
//! first, and if it fails will try the [`Either::Right`] type.
//! Engine failures are returned immediately without trying another conversion.
//!
//! Upon failure of both, the second failure will be returned.
#![cfg(feature = "either")]

#[cfg(test)]
use crate::Context;
use crate::ContextAccess;

use crate::value::TryFromJs;
use either::Either;
use surgeist_js::{JsResult, JsValue};

impl<L, R> TryFromJs for Either<L, R>
where
    L: TryFromJs,
    R: TryFromJs,
{
    #[inline]
    fn try_from_js(value: &JsValue, mut context: ContextAccess<'_>) -> JsResult<Self> {
        L::try_from_js(value, context.reborrow())
            .map(Self::Left)
            .or_else(|error| {
                if error.as_engine().is_some() {
                    Err(error)
                } else {
                    R::try_from_js(value, context.reborrow()).map(Self::Right)
                }
            })
    }
}

#[test]
fn either() {
    let v = JsValue::new(123);
    let mut context = Context::default();

    assert_eq!(
        Either::<i32, i32>::try_from_js(&v, context.access()),
        Ok(Either::Left(123))
    );
    assert_eq!(
        Either::<i32, String>::try_from_js(&v, context.access()),
        Ok(Either::Left(123))
    );
    assert_eq!(
        Either::<String, i32>::try_from_js(&v, context.access()),
        Ok(Either::Right(123))
    );
    assert!(Either::<String, String>::try_from_js(&v, context.access()).is_err());
}
