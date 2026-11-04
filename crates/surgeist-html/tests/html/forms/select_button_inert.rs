//! HTML 877d6146487f #the-button-element (forms capture line 7399): a button that is the first
//! child which is an element of a select is inert. The oracle is the sentence itself: each stage
//! admits exactly one kind and the final operand is the first-element-child position.
use surgeist_html::bounded::select::HtmlSelectButton;
use surgeist_html::vocabulary::KnownHtmlElement;

fn kinds() -> Vec<Option<KnownHtmlElement>> {
    KnownHtmlElement::ALL
        .iter()
        .copied()
        .map(Some)
        .chain([None])
        .collect()
}

#[test]
fn only_a_button_that_is_the_first_element_child_of_a_select_is_inert() {
    let button = HtmlSelectButton::new(Some(KnownHtmlElement::Button)).unwrap();
    let child = button.child_of(Some(KnownHtmlElement::Select)).unwrap();
    assert!(child.is_inert(true));
    assert!(!child.is_inert(false));
}

#[test]
fn only_a_button_kind_reaches_the_button_stage() {
    for kind in kinds() {
        assert_eq!(
            HtmlSelectButton::new(kind).is_some(),
            kind == Some(KnownHtmlElement::Button),
            "{kind:?}"
        );
    }
}

#[test]
fn only_a_select_parent_reaches_the_select_child_stage() {
    let button = HtmlSelectButton::new(Some(KnownHtmlElement::Button)).unwrap();
    for parent in kinds() {
        assert_eq!(
            button.child_of(parent).is_some(),
            parent == Some(KnownHtmlElement::Select),
            "{parent:?}"
        );
    }
}
