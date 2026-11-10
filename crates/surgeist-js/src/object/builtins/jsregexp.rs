//! A Rust API wrapper for Boa's `RegExp` Builtin ECMAScript Object
use crate::ContextAccess;
use crate::{
    JsExpect, JsNativeError, JsResult, JsValue,
    builtins::RegExp,
    error::PanicError,
    object::{JsArray, JsObject},
    value::TryFromJs,
};

use std::ops::Deref;

/// `JsRegExp` provides a wrapper for Boa's implementation of the ECMAScript `RegExp` builtin object
///
/// # Examples
///
/// Create a `JsRegExp` and run RegExp.prototype.test( String )
///
/// ```
/// # use surgeist_js::{
/// #  object::builtins::JsRegExp,
/// #  Context, JsValue, JsResult,js_string
/// # };
/// # fn main() -> JsResult<()> {
/// // Initialize the `Context`
/// let context = &mut Context::builder().build().unwrap();
///
/// // Create a new RegExp with pattern and flags
/// let regexp = JsRegExp::new(js_string!("foo"), js_string!("gi"), context.access())?;
///
/// let test_result = regexp.test(js_string!("football"), context.access())?;
/// assert!(test_result);
///
/// let to_string = regexp.to_string(context.access())?;
/// assert_eq!(to_string, String::from("/foo/gi"));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct JsRegExp {
    inner: JsObject,
}

impl JsRegExp {
    /// Create a new `JsRegExp` object
    /// ```
    /// # use surgeist_js::{
    /// #  object::builtins::JsRegExp,
    /// #  Context, JsValue, JsResult, js_string
    /// # };
    /// # fn main() -> JsResult<()> {
    /// // Initialize the `Context`
    /// let context = &mut Context::builder().build().unwrap();
    ///
    /// // Create a new RegExp with pattern and flags
    /// let regexp = JsRegExp::new(js_string!("foo"), js_string!("gi"), context.access())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<S>(pattern: S, flags: S, context: ContextAccess<'_>) -> JsResult<Self>
    where
        S: Into<JsValue>,
    {
        let context = context.into_owner();
        let regexp = RegExp::initialize(None, &pattern.into(), &flags.into(), context)?
            .as_object()
            .js_expect("RegExp::initialize must return a RegExp object")?
            .clone();

        Ok(Self { inner: regexp })
    }

    /// Create a `JsRegExp` from a regular expression `JsObject`
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.is::<RegExp>() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not a RegExp")
                .into())
        }
    }

    /// Returns a boolean value for whether the `d` flag is present in `JsRegExp` flags
    #[inline]
    pub fn has_indices(&self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        RegExp::get_has_indices(&self.inner.clone().into(), &[], context).and_then(|v| {
            v.as_boolean()
                .js_expect("value must be a bool")
                .map_err(Into::into)
        })
    }

    /// Returns a boolean value for whether the `g` flag is present in `JsRegExp` flags
    #[inline]
    pub fn global(&self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        RegExp::get_global(&self.inner.clone().into(), &[], context).and_then(|v| {
            v.as_boolean()
                .js_expect("value must be a bool")
                .map_err(Into::into)
        })
    }

    /// Returns a boolean value for whether the `i` flag is present in `JsRegExp` flags
    #[inline]
    pub fn ignore_case(&self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        RegExp::get_ignore_case(&self.inner.clone().into(), &[], context).and_then(|v| {
            v.as_boolean()
                .js_expect("value must be a bool")
                .map_err(Into::into)
        })
    }

    /// Returns a boolean value for whether the `m` flag is present in `JsRegExp` flags
    #[inline]
    pub fn multiline(&self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        RegExp::get_multiline(&self.inner.clone().into(), &[], context).and_then(|v| {
            v.as_boolean()
                .js_expect("value must be a bool")
                .map_err(Into::into)
        })
    }

    /// Returns a boolean value for whether the `s` flag is present in `JsRegExp` flags
    #[inline]
    pub fn dot_all(&self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        RegExp::get_dot_all(&self.inner.clone().into(), &[], context).and_then(|v| {
            v.as_boolean()
                .js_expect("value must be a bool")
                .map_err(Into::into)
        })
    }

    /// Returns a boolean value for whether the `u` flag is present in `JsRegExp` flags
    #[inline]
    pub fn unicode(&self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        RegExp::get_unicode(&self.inner.clone().into(), &[], context).and_then(|v| {
            v.as_boolean()
                .js_expect("value must be a bool")
                .map_err(Into::into)
        })
    }

    /// Returns a boolean value for whether the `y` flag is present in `JsRegExp` flags
    #[inline]
    pub fn sticky(&self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        RegExp::get_sticky(&self.inner.clone().into(), &[], context).and_then(|v| {
            v.as_boolean()
                .js_expect("value must be a bool")
                .map_err(Into::into)
        })
    }

    /// Returns the flags of `JsRegExp` as a string
    /// ```
    /// # use surgeist_js::{
    /// #  object::builtins::JsRegExp,
    /// #  Context, JsValue, JsResult, js_string
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # let context = &mut Context::builder().build().unwrap();
    /// let regexp = JsRegExp::new(js_string!("foo"), js_string!("gi"), context.access())?;
    ///
    /// let flags = regexp.flags(context.access())?;
    /// assert_eq!(flags, String::from("gi"));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn flags(&self, context: ContextAccess<'_>) -> JsResult<String> {
        let context = context.into_owner();
        RegExp::get_flags(&self.inner.clone().into(), &[], context).and_then(|v| {
            v.as_string()
                .js_expect("value must be string")?
                .to_std_string()
                .map_err(|e| PanicError::new(e.to_string()).into())
        })
    }

    /// Returns the source pattern of `JsRegExp` as a string
    /// ```
    /// # use surgeist_js::{
    /// #  object::builtins::JsRegExp,
    /// #  Context, JsValue, JsResult, js_string
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # let context = &mut Context::builder().build().unwrap();
    /// let regexp = JsRegExp::new(js_string!("foo"), js_string!("gi"), context.access())?;
    ///
    /// let src = regexp.source(context.access())?;
    /// assert_eq!(src, String::from("foo"));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn source(&self, context: ContextAccess<'_>) -> JsResult<String> {
        let context = context.into_owner();
        RegExp::get_source(&self.inner.clone().into(), &[], context).and_then(|v| {
            v.as_string()
                .js_expect("value must be string")?
                .to_std_string()
                .map_err(|e| PanicError::new(e.to_string()).into())
        })
    }

    /// Executes a search for a match between `JsRegExp` and the provided string
    /// ```
    /// # use surgeist_js::{
    /// #  object::builtins::JsRegExp,
    /// #  Context, JsValue, JsResult, js_string
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # let context = &mut Context::builder().build().unwrap();
    /// let regexp = JsRegExp::new(js_string!("foo"), js_string!("gi"), context.access())?;
    ///
    /// let test_result = regexp.test(js_string!("football"), context.access())?;
    /// assert!(test_result);
    /// # Ok(())
    /// # }
    /// ```
    pub fn test<S>(&self, search_string: S, context: ContextAccess<'_>) -> JsResult<bool>
    where
        S: Into<JsValue>,
    {
        let context = context.into_owner();
        RegExp::test(&self.inner.clone().into(), &[search_string.into()], context).and_then(|v| {
            v.as_boolean()
                .js_expect("value must be a bool")
                .map_err(Into::into)
        })
    }

    /// Executes a search for a match in a specified string
    ///
    /// Returns a `JsArray` containing matched value and updates the `lastIndex` property, or `None`
    pub fn exec<S>(&self, search_string: S, context: ContextAccess<'_>) -> JsResult<Option<JsArray>>
    where
        S: Into<JsValue>,
    {
        let context = context.into_owner();
        RegExp::exec(&self.inner.clone().into(), &[search_string.into()], context).and_then(|v| {
            if v.is_null() {
                Ok(None)
            } else {
                let obj = v.to_object(context.access())?;
                let array = JsArray::from_object(obj)
                    .js_expect("from_object must not fail if value is an array object")?;
                Ok(Some(array))
            }
        })
    }

    /// Return a string representing the regular expression.
    /// ```
    /// # use surgeist_js::{
    /// #  object::builtins::JsRegExp,
    /// #  Context, JsValue, JsResult, js_string
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # let context = &mut Context::builder().build().unwrap();
    /// let regexp = JsRegExp::new(js_string!("foo"), js_string!("gi"), context.access())?;
    ///
    /// let to_string = regexp.to_string(context.access())?;
    /// assert_eq!(to_string, "/foo/gi");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn to_string(&self, context: ContextAccess<'_>) -> JsResult<String> {
        let context = context.into_owner();
        RegExp::to_string(&self.inner.clone().into(), &[], context).and_then(|v| {
            v.as_string()
                .js_expect("value must be a string")?
                .to_std_string()
                .map_err(|e| PanicError::new(e.to_string()).into())
        })
    }
}

impl From<JsRegExp> for JsObject {
    #[inline]
    fn from(o: JsRegExp) -> Self {
        o.inner.clone()
    }
}

impl From<JsRegExp> for JsValue {
    #[inline]
    fn from(o: JsRegExp) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsRegExp {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromJs for JsRegExp {
    fn try_from_js(value: &JsValue, _context: ContextAccess<'_>) -> JsResult<Self> {
        if let Some(o) = value.as_object() {
            Self::from_object(o.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("value is not a RegExp object")
                .into())
        }
    }
}
