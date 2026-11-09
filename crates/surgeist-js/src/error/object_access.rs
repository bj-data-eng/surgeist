//! Host-only failures while acquiring a live object's typed access guard.

use thiserror::Error;

/// A live object could not provide the requested state and native-payload access.
///
/// This rootless failure is distinct from a stored-root acquisition failure and
/// from an ordinary optional downcast to a different requested native type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ObjectAccessError {
    /// Another alias holds incompatible access to the same object.
    #[error("object access is already borrowed")]
    Borrowed,
    /// The object's payload cannot supply its required native brand.
    #[error("object payload has the wrong native brand")]
    WrongNativeBrand,
}

impl From<ObjectAccessError> for super::JsError {
    fn from(error: ObjectAccessError) -> Self {
        super::EngineError::ObjectAccess(error).into()
    }
}

#[cfg(test)]
mod tests {
    use super::ObjectAccessError;
    use crate::{
        Context, JsError, NativeFunction, Source,
        error::{DiagnosticEngine, DiagnosticKind, EngineError, ErasedEngineError, PanicError},
    };

    fn context() -> Context {
        let builder = Context::builder();
        #[cfg(feature = "fuzz")]
        let builder = builder.instructions_remaining(100_000);
        builder.build().unwrap()
    }

    #[test]
    fn object_access_failures_retain_their_exact_cause_in_all_error_views() {
        let mut context = context();
        for cause in [
            ObjectAccessError::Borrowed,
            ObjectAccessError::WrongNativeBrand,
        ] {
            let error = JsError::from(cause);
            assert_eq!(error.as_engine(), Some(&EngineError::ObjectAccess(cause)));
            assert!(!error.is_catchable());
            let restored = error.clone().into_stored().to_error().unwrap();
            assert_eq!(restored, error);
            assert_eq!(
                restored.into_erased(context.access()).as_engine(),
                Some(&ErasedEngineError::ObjectAccess(cause))
            );
            let with_source: JsError = PanicError::new("access source").with_source(error).into();
            let Some(EngineError::Panic(panic)) = with_source.as_engine() else {
                panic!("the diagnostic source must retain its engine wrapper")
            };
            let diagnostic = panic.diagnostic().unwrap();
            assert_eq!(
                diagnostic.kind(),
                DiagnosticKind::Engine(DiagnosticEngine::ObjectAccess(cause))
            );
            assert_eq!(
                diagnostic.to_string(),
                EngineError::ObjectAccess(cause).to_string()
            );
            assert_eq!(
                diagnostic.erase(&mut context).as_engine(),
                Some(&ErasedEngineError::ObjectAccess(cause))
            );
        }
    }

    #[test]
    fn object_access_host_failures_bypass_guest_catch_handlers() {
        for cause in [
            ObjectAccessError::Borrowed,
            ObjectAccessError::WrongNativeBrand,
        ] {
            let mut context = context();
            let function = match cause {
                ObjectAccessError::Borrowed => {
                    NativeFunction::from_fn_ptr(|_, _, _| Err(ObjectAccessError::Borrowed.into()))
                }
                ObjectAccessError::WrongNativeBrand => NativeFunction::from_fn_ptr(|_, _, _| {
                    Err(ObjectAccessError::WrongNativeBrand.into())
                }),
            };
            context
                .register_global_callable(crate::js_string!("refuseAccess"), 0, function)
                .unwrap();
            let error = context
                .eval(Source::from_bytes(
                    "globalThis.caught = false; try { refuseAccess(); } catch (e) { caught = true; }",
                ))
                .unwrap_err();
            assert_eq!(error.as_engine(), Some(&EngineError::ObjectAccess(cause)));
            assert_eq!(
                context
                    .eval(Source::from_bytes("caught"))
                    .unwrap()
                    .as_boolean(),
                Some(false)
            );
        }
    }

    #[test]
    fn opaque_error_conversion_refuses_an_alias_and_preserves_same_value_retry() {
        let mut context = context();
        let value = context
            .eval(Source::from_bytes("new Error('retained')"))
            .unwrap();
        let object = value.as_object().unwrap();
        let identity = object.identity();
        let held = object.borrow_mut().unwrap();
        assert_eq!(
            JsError::from_opaque(value.clone()).unwrap_err(),
            ObjectAccessError::Borrowed
        );
        drop(held);

        let converted = JsError::from_opaque(value.clone()).unwrap();
        assert_eq!(converted.as_opaque(), Some(&value));
        assert_eq!(
            converted
                .into_opaque(context.access())
                .unwrap()
                .as_object()
                .unwrap()
                .identity(),
            identity
        );
    }

    #[test]
    fn refused_thrown_error_conversion_stops_before_guest_catch_publication() {
        let mut context = context();
        let value = context
            .eval(Source::from_bytes(
                "globalThis.marker = new Error('retained'); marker",
            ))
            .unwrap();
        let object = value.as_object().unwrap();
        let held = object.borrow_mut().unwrap();
        let error = context
            .eval(Source::from_bytes(
                "globalThis.caught = false; try { throw marker; } catch (e) { caught = true; }",
            ))
            .unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed))
        );
        drop(held);
        assert_eq!(
            context
                .global_object()
                .get(crate::js_string!("caught"), context.access())
                .unwrap()
                .as_boolean(),
            Some(false)
        );
        assert_eq!(
            JsError::from_opaque(value.clone()).unwrap().as_opaque(),
            Some(&value)
        );
    }

    #[test]
    fn erased_aggregate_keeps_nested_access_failure_instead_of_opaque_text() {
        let mut context = context();
        let value = context
            .eval(Source::from_bytes("new AggregateError([1], 'message')"))
            .unwrap();
        let aggregate = JsError::from_opaque(value.clone()).unwrap();
        let errors = value
            .as_object()
            .unwrap()
            .get(crate::js_string!("errors"), context.access())
            .unwrap()
            .as_object()
            .unwrap();
        let held = errors.borrow_mut().unwrap();
        assert_eq!(
            aggregate.into_erased(context.access()).as_engine(),
            Some(&ErasedEngineError::ObjectAccess(
                ObjectAccessError::Borrowed
            ))
        );
        drop(held);
        assert!(
            JsError::from_opaque(value)
                .unwrap()
                .try_native(context.access())
                .is_ok()
        );
    }

    #[test]
    fn opaque_error_rendering_refuses_a_payload_alias_and_retries_same_value() {
        let object = crate::JsObject::new_unique(&crate::Runtime::default(), None, true).unwrap();
        let error = JsError::from_opaque(object.clone().upcast().into()).unwrap();
        let held = object.borrow_mut().unwrap();
        assert_eq!(
            error.render().unwrap_err().as_engine(),
            Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed))
        );
        drop(held);
        assert_eq!(error.render().unwrap(), "Boolean { true }");
    }
}
