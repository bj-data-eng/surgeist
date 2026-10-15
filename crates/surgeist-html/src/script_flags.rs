//! Intrinsic HTML script flags from HTML 877d6146487f #script-processing-model.
//!
//! These scalar values carry no document identity, result, runnable preparation,
//! or execution capability. The adapter establishes each hook's applicability
//! and performs its document-slot and other state changes in source order.

/// Established HTML script processing flags, distinct from SVG processed state.
///
/// Native creation uses [`Self::fresh`]. Retained construction uses
/// [`Self::from_retained`] without rerunning initialization. These values alone
/// do not certify a complete script state or its source/capture qualification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlScriptFlags {
    force_async: bool,
    already_started: bool,
    external_file: bool,
    ready_to_be_parser_executed: bool,
    delaying_load: bool,
}

impl HtmlScriptFlags {
    /// Source-defined native creation: force async is true; all other flags
    /// are false. Parser insertion and inert-fragment rules are separate hooks.
    pub const fn fresh() -> Self {
        Self::from_retained(true, false, false, false, false)
    }

    /// Preserves independently established scalar slots, without inferring
    /// document associations, readiness, execution, or construction history.
    pub const fn from_retained(
        force_async: bool,
        already_started: bool,
        external_file: bool,
        ready_to_be_parser_executed: bool,
        delaying_load: bool,
    ) -> Self {
        Self {
            force_async,
            already_started,
            external_file,
            ready_to_be_parser_executed,
            delaying_load,
        }
    }

    pub const fn force_async(self) -> bool {
        self.force_async
    }
    pub const fn already_started(self) -> bool {
        self.already_started
    }
    pub const fn external_file(self) -> bool {
        self.external_file
    }
    pub const fn ready_to_be_parser_executed(self) -> bool {
        self.ready_to_be_parser_executed
    }
    pub const fn delaying_load(self) -> bool {
        self.delaying_load
    }

    /// Parser-created script's local force-async step. The adapter independently
    /// establishes its parser document; this does not mark the script started.
    pub fn mark_parser_inserted(&mut self) {
        self.force_async = false;
    }

    /// The async content attribute's Added hook, including during cloning.
    /// Setting an already-present attribute is not an Added trigger.
    pub fn async_attribute_added(&mut self) {
        self.force_async = false;
    }

    /// Preparation step 4, after clearing a nonnull parser document, when the
    /// actual async attribute is absent. Other preparation work stays pending.
    pub fn parser_document_cleared_without_async(&mut self) {
        self.force_async = true;
    }

    /// Preparation step 14's flag effect after restoring the saved nonnull
    /// parser document. The adapter owns that independent reference transfer.
    pub fn parser_document_restored(&mut self) {
        self.force_async = false;
    }

    /// The already-started write required by the actual applicable source
    /// hook, including inert fragment creation, parser EOF, or preparation.
    /// This does not run any of those algorithms or execute the script.
    pub fn mark_already_started(&mut self) {
        self.already_started = true;
    }

    /// HTML script cloning copies only already-started. The target's native
    /// initialization and actual copied-attribute effects remain intact.
    pub fn clone_already_started_from(&mut self, source: Self) {
        self.already_started = source.already_started;
    }
}
