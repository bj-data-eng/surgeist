//! Publication-only failures and the authoritative selected-realm check.

use super::*;
use crate::Runtime;
use surgeist_js_gc::heap::ObjectId;

/// Publication refused a realm belonging to another runtime.
///
/// Identities are diagnostic facts, without realm access authority. The source
/// is the exact TypeError produced by the shared receiving-ownership check.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("selected realm belongs to a different runtime")]
pub struct ForeignSelectedRealm {
    facts: Rc<RealmOwnershipFacts>,
    #[source]
    pub(super) original: PanicDiagnostic,
}

impl ForeignSelectedRealm {
    /// Identity of the refused selected realm.
    #[must_use]
    pub fn selected_realm(&self) -> ObjectId {
        self.facts.selected
    }

    /// Identity of the receiving context's realm at the check.
    #[must_use]
    pub fn receiving_realm(&self) -> ObjectId {
        self.facts.receiving
    }

    /// Complete immutable diagnostic of the original direct-call TypeError.
    #[must_use]
    pub fn diagnostic(&self) -> Diagnostic<'_> {
        self.original.view()
    }

    pub(super) fn from_stored(
        selected: ObjectId,
        receiving: ObjectId,
        original: &StoredDiagnostic,
    ) -> Self {
        Self {
            facts: Rc::new(RealmOwnershipFacts {
                selected,
                receiving,
            }),
            original: original.to_external(),
        }
    }
}

/// Shared immutable identity facts; these carry no runtime or root authority.
#[derive(Debug, PartialEq, Eq)]
struct RealmOwnershipFacts {
    selected: ObjectId,
    receiving: ObjectId,
}

/// Borrowed publication refusal, including the visible original TypeError.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticForeignSelectedRealm<'a> {
    pub(super) selected: ObjectId,
    pub(super) receiving: ObjectId,
    pub(super) diagnostic: Diagnostic<'a>,
}

impl<'a> DiagnosticForeignSelectedRealm<'a> {
    /// Identity of the refused selected realm.
    #[must_use]
    pub const fn selected_realm(self) -> ObjectId {
        self.selected
    }
    /// Identity of the receiving context's realm at the check.
    #[must_use]
    pub const fn receiving_realm(self) -> ObjectId {
        self.receiving
    }
    /// Complete visible original diagnostic.
    #[must_use]
    pub const fn diagnostic(self) -> Diagnostic<'a> {
        self.diagnostic
    }
}

/// Thread-safe export of a selected-realm publication refusal.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("selected realm belongs to a different runtime")]
pub struct ErasedForeignSelectedRealm {
    pub(super) selected: ObjectId,
    pub(super) receiving: ObjectId,
    #[source]
    pub(super) diagnostic: Box<JsErasedError>,
}

impl ErasedForeignSelectedRealm {
    /// Identity of the refused selected realm.
    #[must_use]
    pub const fn selected_realm(&self) -> ObjectId {
        self.selected
    }
    /// Identity of the receiving context's realm at the check.
    #[must_use]
    pub const fn receiving_realm(&self) -> ObjectId {
        self.receiving
    }
    /// Erased export of the original TypeError.
    #[must_use]
    pub fn diagnostic(&self) -> &JsErasedError {
        &self.diagnostic
    }
}

/// A mismatch can only be created by the actual receiving-runtime comparison.
pub(crate) struct RealmOwnershipMismatch {
    facts: Rc<RealmOwnershipFacts>,
    original: JsError,
}

impl RealmOwnershipMismatch {
    pub(crate) fn into_direct(self) -> JsError {
        self.original
    }

    pub(crate) fn into_publication(self) -> JsError {
        let backtrace = self.original.backtrace.clone();
        JsError {
            inner: Repr::Engine(EngineError::ForeignSelectedRealm(ForeignSelectedRealm {
                facts: self.facts,
                original: PanicDiagnostic::new(self.original),
            })),
            backtrace,
        }
    }
}

pub(crate) fn check_realm_ownership(
    runtime: &Runtime,
    receiving: ObjectId,
    selected: &Realm,
) -> Result<(), RealmOwnershipMismatch> {
    if runtime.same_heap(&selected.runtime()) {
        return Ok(());
    }
    Err(RealmOwnershipMismatch {
        facts: Rc::new(RealmOwnershipFacts {
            selected: selected.identity(),
            receiving,
        }),
        original: JsNativeError::typ()
            .with_message("realm belongs to a different runtime")
            .into(),
    })
}

#[cfg(test)]
#[path = "publication_tests.rs"]
mod tests;
