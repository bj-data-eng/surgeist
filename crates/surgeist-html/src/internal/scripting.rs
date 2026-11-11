use crate::attributes::sandbox::{HtmlSandboxFlag, HtmlSandboxFlags};
use crate::security::nonce::HtmlBrowsingContextPresence;

/// Known script engine support for the relevant settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptingEngine {
    /// The native owner has no script engine.
    NoEngine,
    /// The native owner has authenticated a scripting engine.
    Supported,
}

/// Embedder decisions for the current relevant settings. The native host
/// supplies browsing-context and active sandbox facts independently.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlScriptingPolicy {
    /// The user permits scripting for these settings.
    pub user_allows: bool,
    /// The automation policy permits scripting for these settings.
    pub automation_allows: bool,
}

impl HtmlScriptingPolicy {
    /// HTML's scripting condition for Window settings and their active sandbox.
    pub fn window_settings_enabled(
        self,
        engine: HtmlScriptingEngine,
        active_sandbox: &HtmlSandboxFlags,
    ) -> bool {
        engine == HtmlScriptingEngine::Supported
            && self.user_allows
            && !active_sandbox.contains(HtmlSandboxFlag::Scripts)
            && self.automation_allows
    }

    /// HTML's scripting condition for a Document associated with Window
    /// settings, including the independent node-document browsing-context check.
    pub fn document_enabled(
        self,
        engine: HtmlScriptingEngine,
        active_sandbox: &HtmlSandboxFlags,
        browsing_context: HtmlBrowsingContextPresence,
    ) -> bool {
        self.window_settings_enabled(engine, active_sandbox)
            && browsing_context == HtmlBrowsingContextPresence::Present
    }
}
