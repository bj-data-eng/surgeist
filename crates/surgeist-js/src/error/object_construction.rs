//! Host-only failures in prepared engine object construction.

use surgeist_js_gc::heap::AccessError;
use thiserror::Error;

/// Trusted object preparation could not establish its required representation.
///
/// This rootless failure is distinct from invalid caller-authored input, root
/// acquisition refusal, native access refusal, and resource exhaustion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ObjectConstructionError {
    /// A trusted producer supplied a foreign, stale or incorrectly typed edge.
    #[error("trusted object construction rejected a stored edge: {0}")]
    InvalidStoredEdge(AccessError),
    /// Bulk publication requires a unique-shape representation.
    #[error("object construction requires a unique shape")]
    ExpectedUniqueShape,
    /// A named template contains an indexed property key.
    #[error("named object template contains an indexed key")]
    IndexedTemplateKey,
    /// A named template defines the same property more than once.
    #[error("named object template contains a duplicate property")]
    DuplicateTemplateProperty,
    /// Accessor flags do not describe a supported named-slot layout.
    #[error("object construction has an invalid accessor layout")]
    InvalidAccessorLayout,
    /// Prepared values do not match the layout's exact storage requirement.
    #[error("object construction requires {expected} named slots, received {actual}")]
    NamedSlotCount {
        /// Exact storage requirement established by the prepared layout.
        expected: usize,
        /// Number of supplied named storage values.
        actual: usize,
    },
    /// Fresh layout installation requires empty named storage.
    #[error("fresh object named slots are already initialized")]
    NamedSlotsAlreadyInitialized,
    /// An update cannot replace an existing property with a different width.
    #[error("object construction has an incompatible named-slot layout")]
    IncompatibleSlotLayout,
    /// The trusted target refused the requested internal prototype change.
    #[error("object construction prototype change was refused")]
    PrototypeRefused,
    /// A trusted fresh-object property installation was refused.
    #[error("object construction property installation was refused")]
    PropertyRefused,
}

impl From<ObjectConstructionError> for super::JsError {
    fn from(error: ObjectConstructionError) -> Self {
        super::EngineError::ObjectConstruction(error).into()
    }
}

#[cfg(test)]
mod tests {
    use super::ObjectConstructionError;
    use crate::{
        Context, JsError, NativeFunction, Source,
        error::{DiagnosticEngine, DiagnosticKind, EngineError, ErasedEngineError, PanicError},
        js_string,
    };
    use surgeist_js_gc::heap::AccessError;

    fn context() -> Context {
        let builder = Context::builder();
        #[cfg(feature = "fuzz")]
        let builder = builder.instructions_remaining(100_000);
        builder.build().unwrap()
    }

    #[test]
    fn construction_failures_preserve_the_exact_cause_in_every_error_view() {
        let mut context = context();
        for cause in [
            ObjectConstructionError::InvalidStoredEdge(AccessError::ForeignHeap),
            ObjectConstructionError::InvalidStoredEdge(AccessError::StaleHandle),
            ObjectConstructionError::InvalidStoredEdge(AccessError::WrongType),
            ObjectConstructionError::ExpectedUniqueShape,
            ObjectConstructionError::IndexedTemplateKey,
            ObjectConstructionError::DuplicateTemplateProperty,
            ObjectConstructionError::InvalidAccessorLayout,
            ObjectConstructionError::NamedSlotCount {
                expected: 2,
                actual: 1,
            },
            ObjectConstructionError::NamedSlotsAlreadyInitialized,
            ObjectConstructionError::IncompatibleSlotLayout,
            ObjectConstructionError::PrototypeRefused,
            ObjectConstructionError::PropertyRefused,
        ] {
            let error = JsError::from(cause);
            assert_eq!(
                error.as_engine(),
                Some(&EngineError::ObjectConstruction(cause))
            );
            assert!(!error.is_catchable());
            let restored = error.clone().into_stored().to_error().unwrap();
            assert_eq!(restored, error);
            assert_eq!(
                restored.into_erased(context.access()).as_engine(),
                Some(&ErasedEngineError::ObjectConstruction(cause))
            );
            let diagnostic_error: JsError = PanicError::new("construction source")
                .with_source(error)
                .into();
            let Some(EngineError::Panic(panic)) = diagnostic_error.as_engine() else {
                panic!("the diagnostic source must retain its engine wrapper")
            };
            let diagnostic = panic.diagnostic().unwrap();
            assert_eq!(
                diagnostic.kind(),
                DiagnosticKind::Engine(DiagnosticEngine::ObjectConstruction(cause))
            );
            assert_eq!(
                diagnostic.to_string(),
                EngineError::ObjectConstruction(cause).to_string()
            );
            assert_eq!(
                diagnostic.erase(&mut context).as_engine(),
                Some(&ErasedEngineError::ObjectConstruction(cause))
            );
        }
    }

    #[test]
    fn construction_host_failure_bypasses_guest_catch_without_publishing_a_catch_effect() {
        let mut context = context();
        context
            .register_global_callable(
                js_string!("refuseConstruction"),
                0,
                NativeFunction::from_fn_ptr(|_, _, _| {
                    Err(ObjectConstructionError::PrototypeRefused.into())
                }),
            )
            .unwrap();
        let error = context
            .eval(Source::from_bytes(
                "let caught = false; try { refuseConstruction(); } catch (_) { caught = true; }",
            ))
            .unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::ObjectConstruction(
                ObjectConstructionError::PrototypeRefused
            ))
        );
        assert_eq!(
            context.eval(Source::from_bytes("caught")).unwrap(),
            false.into()
        );
    }
}
