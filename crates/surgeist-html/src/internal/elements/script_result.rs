//! Script result values for the selected native/Disabled preparation profile.

/// The actual script result supported by the closed native/Disabled profile.
///
/// HTML877d's [script processing model](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#script-processing-model)
/// initializes the result to "uninitialized". Preparation through the Disabled
/// return does not replace it. This value is neither a failed-load null nor a
/// loaded, compiled or executed script/import-map/speculation-rules result.
/// Missing retained result evidence cannot be replaced by this initial value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptResult {
    /// No result has been installed by the supported operation.
    Uninitialized,
}

/// No result-ready callback can be installed by the native/Disabled profile.
///
/// HTML initializes the steps to null. The supported preparation prefix never
/// installs callbacks, so their actual slot is `Option<HtmlScriptReadySteps>`
/// with value `None`. This uninhabited type adds no callback/execution capability;
/// missing retained steps must remain explicitly unavailable outside this type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptReadySteps {}
