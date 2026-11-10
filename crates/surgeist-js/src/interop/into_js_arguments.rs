use crate::ContextAccess;
use std::{
    cell::{Ref, RefMut},
    ops::Deref,
};
use surgeist_js::{JsNativeError, JsObject, JsResult, JsValue, NativeObject};
use surgeist_js::{class::Class, value::TryFromJs};

/// Create a Rust value from a JS argument. This trait is used to
/// convert arguments from JS to Rust types. It allows support
/// for optional arguments or rest arguments.
pub trait TryFromJsArgument<'a>: Sized {
    /// Try to convert a JS argument into a Rust value, returning the
    /// value and the rest of the arguments to be parsed.
    ///
    /// # Errors
    /// Any parsing errors that may occur during the conversion.
    fn try_from_js_argument(
        this: &'a JsValue,
        rest: &'a [JsValue],
        context: ContextAccess<'_>,
    ) -> JsResult<(Self, &'a [JsValue])>;
}

impl<'a, T: TryFromJs> TryFromJsArgument<'a> for T {
    fn try_from_js_argument(
        _: &'a JsValue,
        rest: &'a [JsValue],
        mut context: ContextAccess<'_>,
    ) -> JsResult<(Self, &'a [JsValue])> {
        match rest.split_first() {
            Some((first, rest)) => Ok((first.try_js_into(context.reborrow())?, rest)),
            None => T::try_from_js(&JsValue::undefined(), context.reborrow()).map(|v| (v, rest)),
        }
    }
}

/// An argument that would be ignored in a JS function. This is equivalent of typing
/// `()` in Rust functions argument, but more explicit.
#[derive(Debug, Clone, Copy)]
pub struct Ignore;

impl<'a> TryFromJsArgument<'a> for Ignore {
    fn try_from_js_argument(
        _this: &'a JsValue,
        rest: &'a [JsValue],
        _: ContextAccess<'_>,
    ) -> JsResult<(Self, &'a [JsValue])> {
        Ok((Ignore, &rest[1..]))
    }
}

/// An argument that when used in a JS function will empty the list
/// of JS arguments as `JsValue`s. This can be used for having the
/// rest of the arguments in a function. It should be the last
/// argument of your function, before the `Context` argument if any.
///
/// For example,
/// ```
/// # use surgeist_js::{Context, ContextAccess, JsValue, IntoJsFunction};
/// # use surgeist_js::interop::JsRest;
/// # let mut context = Context::builder().build().unwrap();
/// let sums: fn(JsRest<'_>, ContextAccess<'_>) -> i32 = |args, mut context| {
///     args.iter()
///         .map(|i| i.try_js_into::<i32>(context.reborrow()).unwrap())
///         .sum::<i32>()
/// };
/// let sums = sums.into_js_function();
///
/// let result = sums
///     .call(
///         &JsValue::undefined(),
///         &[JsValue::from(1), JsValue::from(2), JsValue::from(3)],
///         context.access(),
///     )
///     .unwrap();
/// assert_eq!(result, JsValue::new(6));
/// ```
#[derive(Debug, Clone)]
pub struct JsRest<'a>(pub &'a [JsValue]);

#[allow(unused)]
impl<'a> JsRest<'a> {
    /// Consumes the `JsRest` and returns the inner list of `JsValue`.
    #[must_use]
    pub fn into_inner(self) -> &'a [JsValue] {
        self.0
    }

    /// Transforms the `JsRest` into a `Vec<JsValue>`.
    #[must_use]
    pub fn to_vec(self) -> Vec<JsValue> {
        self.0.to_vec()
    }

    /// Returns an iterator over the inner list of `JsValue`.
    pub fn iter(&self) -> impl Iterator<Item = &JsValue> {
        self.0.iter()
    }

    /// Returns the length of the inner list of `JsValue`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the inner list of `JsValue` is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> From<&'a [JsValue]> for JsRest<'a> {
    fn from(values: &'a [JsValue]) -> Self {
        Self(values)
    }
}

impl<'a> IntoIterator for JsRest<'a> {
    type Item = &'a JsValue;
    type IntoIter = std::slice::Iter<'a, JsValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().iter()
    }
}

/// An argument that when used in a JS function will capture all
/// the arguments that can be converted to `T`. The first argument
/// that cannot be converted to `T` will stop the conversion.
///
/// For example,
/// ```
/// # use surgeist_js::{Context, ContextAccess, JsValue, IntoJsFunction};
/// # use surgeist_js::interop::JsAll;
/// # let mut context = Context::builder().build().unwrap();
/// let sums: fn(JsAll<i32>, ContextAccess<'_>) -> i32 = |args, _context| {
///     args.iter().sum()
/// };
/// let sums = sums.into_js_function();
///
/// let result = sums
///     .call(
///         &JsValue::undefined(),
///         &[
///             JsValue::from(1),
///             JsValue::from(2),
///             JsValue::from(3),
///             JsValue::from(true),
///             JsValue::from(4),
///         ],
///         context.access(),
///     )
///     .unwrap();
/// assert_eq!(result, JsValue::new(6));
/// ```
#[derive(Debug, Clone)]
pub struct JsAll<T: TryFromJs>(pub Vec<T>);

impl<T: TryFromJs> JsAll<T> {
    /// Consumes the `JsAll` and returns the inner list of `T`.
    #[must_use]
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    /// Returns an iterator over the inner list of `T`.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }

    /// Returns a mutable iterator over the inner list of `T`.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.0.iter_mut()
    }

    /// Returns the length of the inner list of `T`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the inner list of `T` is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a, T: TryFromJs> TryFromJsArgument<'a> for JsAll<T> {
    fn try_from_js_argument(
        _this: &'a JsValue,
        mut rest: &'a [JsValue],
        mut context: ContextAccess<'_>,
    ) -> JsResult<(Self, &'a [JsValue])> {
        let mut values = Vec::new();

        while !rest.is_empty() {
            match rest[0].try_js_into(context.reborrow()) {
                Ok(value) => {
                    values.push(value);
                    rest = &rest[1..];
                }
                Err(error) if !error.is_catchable() => return Err(error),
                Err(_) => break,
            }
        }
        Ok((JsAll(values), rest))
    }
}

/// Captures the `this` value in a JS function. Although this can be
/// specified multiple times as argument, it will always be filled
/// with clone of the same value.
#[derive(Debug, Clone)]
pub struct JsThis<T: TryFromJs>(pub T);

impl<'a, T: TryFromJs> TryFromJsArgument<'a> for JsThis<T> {
    fn try_from_js_argument(
        this: &'a JsValue,
        rest: &'a [JsValue],
        mut context: ContextAccess<'_>,
    ) -> JsResult<(Self, &'a [JsValue])> {
        Ok((JsThis(this.try_js_into(context.reborrow())?), rest))
    }
}

impl<T: TryFromJs> Deref for JsThis<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Captures a class instance from the `this` value in a JS function. The class
/// will be a non-mutable reference of Rust type `T`, if it is an instance of `T`.
///
/// To have more flexibility on the parsing of the `this` value, you can use the
/// [`JsThis`] capture instead.
#[derive(Debug, Clone)]
pub struct JsClass<T: NativeObject> {
    inner: JsObject<T>,
}

impl<T: NativeObject> JsClass<T> {
    /// Returns the inner object directly.
    #[must_use]
    pub fn inner(&self) -> JsObject<T> {
        self.inner.clone()
    }

    /// Borrow a reference to the class instance of type `T`.
    ///
    /// Returns the exact access refusal if an alias prevents the borrow.
    pub fn borrow(&self) -> Result<Ref<'_, T>, crate::error::ObjectAccessError> {
        self.inner.borrow_data()
    }

    /// Mutably borrows the payload of a host-defined class.
    ///
    /// The caller must preserve the heap edges described by [`Class`].
    ///
    /// Returns the exact access refusal if an alias prevents the borrow.
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, JsValue, builtins::array_buffer::ArrayBuffer};
    /// use surgeist_js::interop::{JsClass, TryFromJsArgument};
    /// use surgeist_js::object::builtins::JsArrayBuffer;
    /// let mut context = Context::builder().build().unwrap();
    /// let this = JsValue::from(JsArrayBuffer::new(8, context.access()).unwrap());
    /// let (class, _) = JsClass::<ArrayBuffer>::try_from_js_argument(&this, &[], context.access()).unwrap();
    /// let _ = class.borrow_mut();
    /// ```
    pub fn borrow_mut(&self) -> Result<RefMut<'_, T>, crate::error::ObjectAccessError>
    where
        T: Class,
    {
        self.inner.borrow_data_mut()
    }
}

impl<T: NativeObject + Clone> JsClass<T> {
    /// Clones the inner class instance.
    ///
    /// Returns the exact access refusal if an alias prevents the read.
    pub fn clone_inner(&self) -> Result<T, crate::error::ObjectAccessError> {
        Ok(self.inner.borrow()?.data().clone())
    }
}

impl<'a, T: NativeObject + 'static> TryFromJsArgument<'a> for JsClass<T> {
    fn try_from_js_argument(
        this: &'a JsValue,
        rest: &'a [JsValue],
        _context: ContextAccess<'_>,
    ) -> JsResult<(Self, &'a [JsValue])> {
        let inner = this
            .as_object()
            .and_then(|o| o.clone().downcast::<T>().ok())
            .ok_or_else(|| JsNativeError::typ().with_message("invalid this for class method"))?;

        Ok((JsClass { inner }, rest))
    }
}

/// Captures a [`ContextData`] data from the [`crate::Context`] as a JS function argument,
/// based on its type.
///
/// The host defined type must implement [`Clone`], otherwise the borrow
/// checker would not be able to ensure the safety of the context while
/// making the function call. Because of this, it is recommended to use
/// types that are cheap to clone.
///
/// For example,
/// ```
/// # use surgeist_js::{Context, IntoJsFunction, JsData, JsValue};
/// # use surgeist_js::interop::ContextData;
///
/// #[derive(Clone, Debug, JsData, surgeist_js::gc::heap::Trace)]
/// struct CustomHostDefinedStruct {
///     pub counter: usize,
/// }
/// let mut context = Context::builder().build().unwrap();
/// context.insert_data(CustomHostDefinedStruct { counter: 123 });
/// let f: fn(ContextData<CustomHostDefinedStruct>) -> usize =
///     |ContextData(host)| host.counter + 1;
/// let f = f.into_js_function();
///
/// assert_eq!(
///     f.call(&JsValue::undefined(), &[], context.access()),
///     Ok(JsValue::new(124))
/// );
/// ```
#[derive(Debug, Clone)]
pub struct ContextData<T: Clone>(pub T);

impl<'a, T: NativeObject + Clone> TryFromJsArgument<'a> for ContextData<T> {
    fn try_from_js_argument(
        _this: &'a JsValue,
        rest: &'a [JsValue],
        context: ContextAccess<'_>,
    ) -> JsResult<(Self, &'a [JsValue])> {
        match context.get_data::<T>() {
            Some(value) => Ok((ContextData(value.clone()), rest)),
            None => Err(JsNativeError::typ()
                .with_message("Context data not found")
                .into()),
        }
    }
}
