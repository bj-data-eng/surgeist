//! HTML877d Details attribute and insertion requirements.
//! Actual namespace/name recognition, tree membership, mutation storage and
//! notification recipients remain qualified by the consuming adapter.

/// An actual Details attribute recognized by its exact local name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDetailsAttribute {
    /// The `name` attribute, which groups details elements for exclusivity.
    Name,
    /// The `open` boolean attribute, which sets the open state.
    Open,
    /// Any other attribute; it has no effect on details state.
    Other,
}
/// The two distinct source algorithms; neither is an end-of-parse cleanup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDetailsExclusivity {
    /// HTML's "ensure details exclusivity by closing the given element if needed"
    /// (#ensure-details-exclusivity-by-closing-the-given-element-if-needed), run when `name`
    /// changes and by the details element's insertion steps.
    CloseGiven,
    /// HTML's "ensure details exclusivity by closing other elements if needed"
    /// (#ensure-details-exclusivity-by-closing-other-elements-if-needed), run when `open` is
    /// added.
    CloseOther,
}
/// Whether a details element is open, as reported by its toggle events.
///
/// The states are the `closed` and `open` values of the ToggleEvent `oldState` and `newState`
/// strings; see [`HtmlDetailsState::as_str`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDetailsState {
    /// The details element is closed: no `open` attribute.
    Closed,
    /// The details element is open: the `open` attribute is present.
    Open,
}
impl HtmlDetailsState {
    /// Exact state string used by the selected Details ToggleEvent clause.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
        }
    }
}

/// The exact payload queued by HTML877d's Details notification algorithm.
/// Details supplies no source Element; task identities and recipients belong
/// to the consuming owner. Equal states remain an actual notification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDetailsToggleEvent {
    old: HtmlDetailsState,
    new: HtmlDetailsState,
}
impl HtmlDetailsToggleEvent {
    /// Queue with the pending tracker's old state, when one exists, and the
    /// latest transition's new state. The consumer removes/requeues the task.
    pub const fn queued(
        old: HtmlDetailsState,
        new: HtmlDetailsState,
        tracked_old: Option<HtmlDetailsState>,
    ) -> Self {
        Self {
            old: match tracked_old {
                Some(first) => first,
                None => old,
            },
            new,
        }
    }
    /// First old state retained by the source task tracker.
    pub const fn old_state(self) -> HtmlDetailsState {
        self.old
    }
    /// Latest queued new state, independent of attributes at delivery time.
    pub const fn new_state(self) -> HtmlDetailsState {
        self.new
    }
}
/// What a details attribute change obliges the consumer to do; see [`details_attribute_effect`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDetailsAttributeEffect {
    /// No details behavior: another namespace, another attribute or an unchanged `open` presence.
    None,
    /// Run exclusivity by closing the given element; the `name` attribute changed.
    CloseGiven,
    /// The `open` presence changed: queue the details notification task steps
    /// (#details-notification-task-steps) with these states.
    Notify {
        /// State before the change.
        old: HtmlDetailsState,
        /// State after the change.
        new: HtmlDetailsState,
        /// Whether the consumer must also close the other details elements in the same name group
        /// (true only when the element became open).
        close_other: bool,
    },
}
/// Fixed attribute-change decision. Values' presence is the actual DOM change
/// input; their UTF-16 code units are irrelevant to the open transition. This operation
/// does not dispatch an event, certify a closed recipient domain or mutate a tree.
pub fn details_attribute_effect(
    namespace: &crate::internal::values::Namespace,
    attribute: HtmlDetailsAttribute,
    old: Option<&[u16]>,
    new: Option<&[u16]>,
) -> HtmlDetailsAttributeEffect {
    if namespace.uri().is_some() {
        return HtmlDetailsAttributeEffect::None;
    }
    match attribute {
        HtmlDetailsAttribute::Name => HtmlDetailsAttributeEffect::CloseGiven,
        HtmlDetailsAttribute::Open if old.is_none() && new.is_some() => {
            HtmlDetailsAttributeEffect::Notify {
                old: HtmlDetailsState::Closed,
                new: HtmlDetailsState::Open,
                close_other: true,
            }
        }
        HtmlDetailsAttribute::Open if old.is_some() && new.is_none() => {
            HtmlDetailsAttributeEffect::Notify {
                old: HtmlDetailsState::Open,
                new: HtmlDetailsState::Closed,
                close_other: false,
            }
        }
        _ => HtmlDetailsAttributeEffect::None,
    }
}
