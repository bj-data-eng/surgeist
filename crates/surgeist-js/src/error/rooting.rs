//! Host-only failures while restoring roots from stored engine values.

use surgeist_js_gc::heap::AccessError;
use thiserror::Error;

/// A stored value could not regain the roots required for external use.
///
/// This compact failure carries no guest values or heap-root authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RootingError {
    /// The heap refused access; preserves its exact failure category.
    #[error("root access refused: {0}")]
    Access(#[from] AccessError),
    /// The stored allocation or its heap is no longer available.
    #[error("stored root has expired")]
    Expired,
    /// A restored object does not have the required native brand.
    #[error("stored object has the wrong native brand")]
    WrongNativeBrand,
    /// A restored object is not callable.
    #[error("stored object is not callable")]
    NotCallable,
    /// Fallible temporary storage reservation failed.
    #[error("could not allocate root restoration storage")]
    Allocation,
    /// Root restoration storage exceeds the representable size.
    #[error("root restoration storage exceeds the representable size")]
    SizeOverflow,
}

impl From<RootingError> for super::JsError {
    fn from(error: RootingError) -> Self {
        super::EngineError::Rooting(error).into()
    }
}

#[cfg(test)]
mod tests {
    use super::RootingError;
    use crate::{
        Context, JsError, JsNativeError, JsObject, Runtime,
        error::{DiagnosticEngine, DiagnosticKind, EngineError, ErasedEngineError, PanicError},
    };
    use std::{cell::Cell, rc::Rc};
    use surgeist_js_gc::heap::{AccessError, AllocationCollection};

    #[test]
    fn rooting_failures_preserve_exact_payload_through_all_error_views() {
        let mut context = Context::default();
        for failure in [
            RootingError::Access(AccessError::ForeignHeap),
            RootingError::Access(AccessError::StaleHandle),
            RootingError::Access(AccessError::WrongType),
            RootingError::Access(AccessError::PayloadBorrowed),
            RootingError::Access(AccessError::HeapBorrowed),
            RootingError::Expired,
            RootingError::WrongNativeBrand,
            RootingError::NotCallable,
            RootingError::Allocation,
            RootingError::SizeOverflow,
        ] {
            let error = JsError::from(failure);
            assert_eq!(error.as_engine(), Some(&EngineError::Rooting(failure)));
            assert!(!error.is_catchable());
            let restored = error.clone().into_stored().to_error().unwrap();
            assert_eq!(restored, error);
            assert_eq!(
                restored.into_erased(context.access()).as_engine(),
                Some(&ErasedEngineError::Rooting(failure))
            );
            let diagnostic_error: JsError =
                PanicError::new("rooting source").with_source(error).into();
            let Some(EngineError::Panic(panic)) = diagnostic_error.as_engine() else {
                unreachable!()
            };
            assert_eq!(
                panic.diagnostic().unwrap().kind(),
                DiagnosticKind::Engine(DiagnosticEngine::Rooting(failure))
            );
            assert_eq!(
                panic.diagnostic().unwrap().to_string(),
                EngineError::Rooting(failure).to_string()
            );
        }
    }

    #[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
    struct DropProbe(#[heap_trace(skip)] Rc<Cell<usize>>);

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn failed_error_restoration_releases_the_live_prefix_without_changing_storage() {
        for diagnostic in [false, true] {
            let runtime = Runtime::default();
            runtime
                .heap
                .set_allocation_collection(AllocationCollection::Manual)
                .unwrap();
            let expired_runtime = Runtime::default();
            let drops = Rc::new(Cell::new(0));
            let first = JsObject::new_unique(&runtime, None, DropProbe(drops.clone()))
                .unwrap()
                .upcast();
            let last =
                JsObject::new_unique(&expired_runtime, None, DropProbe(Rc::new(Cell::new(0))))
                    .unwrap()
                    .upcast();
            let source: JsError = JsNativeError::aggregate(vec![
                JsError::from_opaque(first.into()),
                JsError::from_opaque(last.into()),
            ])
            .into();
            let stored = if diagnostic {
                JsError::from(PanicError::new("partial roots").with_source(source)).into_stored()
            } else {
                source.into_stored()
            };
            drop(expired_runtime);
            for _ in 0..2 {
                assert_eq!(stored.to_error(), Err(RootingError::Expired));
                assert_eq!(drops.get(), 0);
            }
            runtime.collect().unwrap();
            assert_eq!(
                drops.get(),
                1,
                "failed restoration must release its completed prefix"
            );
        }
    }

    #[test]
    fn heap_access_refusal_preserves_error_storage_and_allows_retry() {
        for diagnostic in [false, true] {
            let runtime = Runtime::default();
            let drops = Rc::new(Cell::new(0));
            let object = JsObject::new_unique(&runtime, None, DropProbe(drops.clone()))
                .unwrap()
                .upcast();
            let identity = object.identity();
            let source = JsError::from_opaque(object.into());
            let original = if diagnostic {
                JsError::from(PanicError::new("retry").with_source(source))
            } else {
                source
            };
            let stored = original.clone().into_stored();
            runtime
                .heap
                .read(|_| {
                    assert_eq!(
                        stored.to_error(),
                        Err(RootingError::Access(AccessError::HeapBorrowed))
                    );
                })
                .unwrap();
            let restored = stored.to_error().unwrap();
            assert_eq!(restored, original);
            if diagnostic {
                let Some(EngineError::Panic(panic)) = restored.as_engine() else {
                    unreachable!()
                };
                let DiagnosticKind::Opaque(leaf) = panic.diagnostic().unwrap().kind() else {
                    unreachable!()
                };
                assert_eq!(leaf.object_identity(), Some(identity));
            } else {
                assert_eq!(
                    restored
                        .as_opaque()
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .identity(),
                    identity
                );
            }
            drop(original);
            runtime.collect().unwrap();
            assert_eq!(
                drops.get(),
                0,
                "successful restoration owns independent roots"
            );
            drop(restored);
            runtime.collect().unwrap();
            assert_eq!(drops.get(), 1);
        }
    }
}
