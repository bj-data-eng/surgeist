//! Native Form scalar initialization, HTML 877d §4.10.3 and §4.10.22.3.

/// Actual fresh native Form scalar state, separate from its past names map.
///
/// The [Form reset method](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-form-reset)
/// uses the locked-for-reset mark. The
/// [submission algorithm](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-algorithm)
/// initializes constructing-entry-list and firing-submission-events to false.
///
/// This allocation-free value supplies only the native scalar tuple. The
/// consuming owner retains the real past names map, node identities, storage,
/// admission and publication. It establishes no complete Form/Element state,
/// event dispatch, reset, submission or entry-list construction capability.
/// Missing imported or retained flags cannot be replaced with fresh state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlFormNativeState {
    locked_for_reset: bool,
    constructing_entry_list: bool,
    firing_submission_events: bool,
}

impl HtmlFormNativeState {
    /// Native Form creation starts unmarked for reset and with both submission
    /// flags false. DOM cloning initializes a fresh Form; adopting or capturing
    /// an existing Form preserves its actual state instead of reinitializing it.
    pub const fn fresh() -> Self {
        Self {
            locked_for_reset: false,
            constructing_entry_list: false,
            firing_submission_events: false,
        }
    }

    /// Whether this actual native state is marked as locked for reset.
    pub const fn locked_for_reset(self) -> bool {
        self.locked_for_reset
    }

    /// Whether entry-list construction is in progress in this actual state.
    pub const fn constructing_entry_list(self) -> bool {
        self.constructing_entry_list
    }

    /// Whether submission events are being fired in this actual state.
    pub const fn firing_submission_events(self) -> bool {
        self.firing_submission_events
    }
}
