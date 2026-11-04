//! HTML 877d6146487f #the-button-element (forms capture line 7399): a button that is the first
//! child which is an element of a select is inert. The oracle is the sentence itself: the
//! predicate holds for exactly one kind pair and the first-element-child position.
use surgeist_html::bounded::select::button_is_inert_in_select;
use surgeist_html::vocabulary::KnownHtmlElement;

#[test]
fn only_a_button_that_is_the_first_element_child_of_a_select_is_inert() {
    assert!(button_is_inert_in_select(
        Some(KnownHtmlElement::Button),
        Some(KnownHtmlElement::Select),
        true
    ));
    assert!(!button_is_inert_in_select(
        Some(KnownHtmlElement::Button),
        Some(KnownHtmlElement::Select),
        false
    ));
}

#[test]
fn every_other_kind_pair_is_not_inert_at_the_first_element_child_position() {
    let mut inert_pairs = 0;
    let kinds = KnownHtmlElement::ALL
        .iter()
        .copied()
        .map(Some)
        .chain([None])
        .collect::<Vec<_>>();
    for button in &kinds {
        for parent in &kinds {
            if button_is_inert_in_select(*button, *parent, true) {
                inert_pairs += 1;
                assert_eq!(*button, Some(KnownHtmlElement::Button));
                assert_eq!(*parent, Some(KnownHtmlElement::Select));
            }
            assert!(!button_is_inert_in_select(*button, *parent, false));
        }
    }
    assert_eq!(inert_pairs, 1);
}
