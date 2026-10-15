//! Intrinsic custom-element state and DOM3071e5f26944 create-element's
//! #concept-create-element no-definition branch.

/// Actual DOM custom-element state, independent of names and current attributes.
///
/// Captured availability and qualified definition/registry associations belong
/// to the consuming capture owner. This scalar supplies no execution capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCustomElementState {
    Undefined,
    Failed,
    Uncustomized,
    Precustomized,
    Custom,
}

/// Selects initial state only after the actual definition lookup is known-null.
///
/// This is DOM's selected create-element step 6, consumed by HTML construction.
/// The caller must establish the no-definition branch through its actual lookup
/// contract. An unavailable/incomplete registry, unsupported reaction domain,
/// or the spelling alone does not prove that branch.
///
/// `custom_name_or_saved_is` is the established predicate: the local name is a
/// valid custom-element name, or the saved creation `is` value is nonnull. It is
/// not inferred from a later content attribute. Shared custom-name rules own
/// validation; the consuming adapter owns its admission and saved value.
/// Non-HTML namespaces ignore this predicate and do not require its inspection.
pub const fn custom_element_state_without_definition(
    is_html_namespace: bool,
    custom_name_or_saved_is: bool,
) -> HtmlCustomElementState {
    if is_html_namespace && custom_name_or_saved_is {
        HtmlCustomElementState::Undefined
    } else {
        HtmlCustomElementState::Uncustomized
    }
}
