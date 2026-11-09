use std::fmt::{self, Display};

use crate::{JsResult, JsValue};

/// A value renderer could not write its output or found invalid represented data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ValueRenderError {
    /// The destination refused formatted output.
    #[error("value rendering destination refused output")]
    Sink,
    /// An array has no required length property.
    #[error("rendered array has no length property")]
    MissingArrayLength,
    /// An array's required length is not numeric.
    #[error("rendered array length is not numeric")]
    InvalidArrayLength,
    /// An enumerated property could not be read from its object.
    #[error("rendered object is missing an enumerated property")]
    MissingProperty,
    /// Stored property data cannot be rendered as its represented descriptor kind.
    #[error("rendered object has an invalid property descriptor")]
    InvalidDescriptor,
}

pub(super) struct RenderSink<'a>(&'a mut dyn fmt::Write);

impl RenderSink<'_> {
    pub(super) fn write_str(&mut self, text: &str) -> JsResult<()> {
        self.0
            .write_str(text)
            .map_err(|_| ValueRenderError::Sink.into())
    }

    pub(super) fn write_char(&mut self, value: char) -> JsResult<()> {
        self.0
            .write_char(value)
            .map_err(|_| ValueRenderError::Sink.into())
    }

    pub(super) fn write_fmt(&mut self, arguments: fmt::Arguments<'_>) -> JsResult<()> {
        self.0
            .write_fmt(arguments)
            .map_err(|_| ValueRenderError::Sink.into())
    }
}

mod arguments;
mod array;
mod map;
mod object;
mod primitives;
mod set;
mod typed_array;
mod value;

/// This object is used for displaying a `Value`.
///
/// Use [`Self::render`] to obtain a string while preserving typed host failures.
#[derive(Debug, Clone, Copy)]
pub struct ValueDisplay<'value> {
    pub(super) value: &'value JsValue,
    pub(super) internals: bool,
}

impl ValueDisplay<'_> {
    /// Renders this value once, preserving typed host failures.
    ///
    /// # Errors
    ///
    /// Returns the original object-access failure or a [`ValueRenderError`] if
    /// represented display data is invalid.
    pub fn render(&self) -> JsResult<String> {
        let mut output = String::new();
        self.render_to(&mut output)?;
        Ok(output)
    }

    /// Writes this value once, preserving access errors and destination refusal.
    pub(crate) fn render_to(&self, output: &mut dyn fmt::Write) -> JsResult<()> {
        value::log_value_to(&mut RenderSink(output), self.value, self.internals, true)
    }

    /// Display internal information about value.
    ///
    /// By default this is `false`.
    #[inline]
    #[must_use]
    pub const fn internals(mut self, yes: bool) -> Self {
        self.internals = yes;
        self
    }
}

impl Display for ValueDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.render_to(f).map_err(|_| fmt::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::ValueRenderError;
    use crate::{
        JsObject, JsValue, Runtime,
        error::{EngineError, ObjectAccessError},
    };
    use std::fmt;

    #[test]
    fn checked_rendering_preserves_object_access_refusal_and_allows_same_object_retry() {
        let object = JsObject::new_unique(&Runtime::default(), None, true).unwrap();
        let value = JsValue::from(object.clone().upcast());
        let held = object.borrow_mut().unwrap();

        for error in [
            value.display().render().unwrap_err(),
            value.display_obj(false).unwrap_err(),
        ] {
            assert_eq!(
                error.as_engine(),
                Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed))
            );
            assert!(!error.is_catchable());
        }

        drop(held);
        assert_eq!(value.display().render().unwrap(), "Boolean { true }");
    }

    #[test]
    fn checked_rendering_preserves_destination_refusal_as_a_host_error() {
        struct RefusingOutput;

        impl fmt::Write for RefusingOutput {
            fn write_str(&mut self, _: &str) -> fmt::Result {
                Err(fmt::Error)
            }
        }

        let error = JsValue::from(42)
            .display()
            .render_to(&mut RefusingOutput)
            .unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::ValueRender(ValueRenderError::Sink))
        );
        assert!(!error.is_catchable());
    }
}
