//! Input suggestions-source selection shared by captured and live tree readers.

use surgeist_html::{
    attributes::HtmlInputType, bounded::attributes::input_list_applies, values::DocumentString,
    vocabulary::KnownHtmlElement,
};

/// Resolve an Input's suggestions source from one qualified observation.
///
/// The reader provides the exact content attribute and finds the first
/// matching element in ordinary tree order, including the root. Only that
/// candidate is checked for HTML Datalist identity.
///
/// # Errors
///
/// Returns the first refusal of a reached reader operation.
pub fn input_suggestions_source<State, Node, E>(
    state: &mut State,
    list: Option<&DocumentString>,
    input_type: impl FnOnce(&mut State) -> Result<HtmlInputType, E>,
    first_by_id: impl FnOnce(&mut State, &DocumentString) -> Result<Option<Node>, E>,
    element_kind: impl FnOnce(&mut State, &Node) -> Result<Option<KnownHtmlElement>, E>,
) -> Result<Option<Node>, E> {
    let Some(list) = list.filter(|list| !list.is_empty()) else {
        return Ok(None);
    };
    if !input_list_applies(input_type(state)?) {
        return Ok(None);
    }
    let Some(candidate) = first_by_id(state, list)? else {
        return Ok(None);
    };
    Ok((element_kind(state, &candidate)? == Some(KnownHtmlElement::Datalist)).then_some(candidate))
}
