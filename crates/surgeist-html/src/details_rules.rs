//! HTML877d Details attribute and insertion requirements.
//! Actual namespace/name recognition, tree membership, mutation storage and
//! notification recipients remain qualified by the consuming adapter.

/// An actual Details attribute recognized by its exact local name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDetailsAttribute {
    Name,
    Open,
    Other,
}
/// The two distinct source algorithms; neither is an end-of-parse cleanup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDetailsExclusivity {
    CloseGiven,
    CloseOther,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDetailsState {
    Closed,
    Open,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDetailsAttributeEffect {
    None,
    CloseGiven,
    Notify {
        old: HtmlDetailsState,
        new: HtmlDetailsState,
        close_other: bool,
    },
}
/// Fixed attribute-change decision. Values' presence is the actual DOM change
/// input; their contents are irrelevant to the open transition. This operation
/// does not dispatch an event, certify a closed recipient domain or mutate a tree.
pub fn details_attribute_effect(
    namespace: &crate::Namespace,
    attribute: HtmlDetailsAttribute,
    old: Option<&crate::DocumentString>,
    new: Option<&crate::DocumentString>,
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
