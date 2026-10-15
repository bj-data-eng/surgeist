//! Literal outcomes from HTML877d's appropriate-template-contents-owner-document
//! and the selected DOM create-element algorithm's no-definition branch.

use surgeist_html::{
    HtmlCustomElementState, HtmlTemplateContentsOwnerDocument,
    custom_element_state_without_definition, template_contents_owner_document,
};

#[test]
fn template_algorithm_created_document_reuses_itself_without_cache_consultation() {
    for ignored_cache in [false, true] {
        assert_eq!(
            template_contents_owner_document(true, ignored_cache),
            HtmlTemplateContentsOwnerDocument::CurrentDocument
        );
    }
}

#[test]
fn ordinary_document_reuses_its_inert_cache_or_requests_one_creation() {
    assert_eq!(
        template_contents_owner_document(false, true),
        HtmlTemplateContentsOwnerDocument::AssociatedInertDocument
    );
    assert_eq!(
        template_contents_owner_document(false, false),
        HtmlTemplateContentsOwnerDocument::CreateAssociatedInertDocument
    );
}

#[test]
fn known_no_definition_creation_only_marks_html_candidates_undefined() {
    assert_eq!(
        custom_element_state_without_definition(true, true),
        HtmlCustomElementState::Undefined
    );
    assert_eq!(
        custom_element_state_without_definition(true, false),
        HtmlCustomElementState::Uncustomized
    );
    for predicate in [false, true] {
        assert_eq!(
            custom_element_state_without_definition(false, predicate),
            HtmlCustomElementState::Uncustomized
        );
    }
}
