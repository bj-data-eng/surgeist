//! The scope of a host's ledger operation that the HTML crates charge below.

use super::{HtmlResource, HtmlWork};
use std::fmt;
use surgeist_ledger::{Scope, SharedOperation};

/// A scope of an operation the host owns, which an HTML operation charges below instead of
/// charging an operation of its own.
///
/// A host that attributes HTML work to its own ledger operation supplies this value to the
/// parser, to a capture or semantic session, to a fact constructor, to a native construction
/// builder or to the controlled autocomplete materialization. The operation is an
/// `Operation<HtmlWork, HtmlResource>` that the host shared with
/// [`surgeist_ledger::Operation::share`], and the scope is one of it. Each HTML operation opens
/// the child scopes the schema names for it below the scope; the module documentation of
/// [`crate::accounting`] lists them, the custody each keeps in the host operation and the scope
/// bound they need.
///
/// A refusal by a cap of the supplied scope or of one of its ancestors stops the operation with
/// the typed stop that carries that scope's limit and total. The tariffs between the supplied
/// scope and the scopes above it must map each dimension to itself, because a refusal names the
/// dimension of the charged change.
///
/// Cloning the value shares the operation.
#[derive(Clone)]
pub struct HtmlHostScope {
    ledger: SharedOperation<HtmlWork, HtmlResource>,
    scope: Scope,
}

impl HtmlHostScope {
    /// The scope `scope` of `ledger`, an operation of the HTML schema pair that the host owns.
    pub fn new(ledger: SharedOperation<HtmlWork, HtmlResource>, scope: Scope) -> Self {
        Self { ledger, scope }
    }

    /// The host's operation.
    pub fn ledger(&self) -> &SharedOperation<HtmlWork, HtmlResource> {
        &self.ledger
    }

    /// The scope the HTML operations charge below.
    pub fn scope(&self) -> Scope {
        self.scope
    }
}

impl fmt::Debug for HtmlHostScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HtmlHostScope")
            .field("scope", &self.scope)
            .finish_non_exhaustive()
    }
}
