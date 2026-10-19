#![forbid(unsafe_code)]

use surgeist_css::*;

// Ordinary pseudo-class permission includes OnlyChild and generic suffixes;
// VT1's named descendants require exactly that narrower set. SearchText's
// Current remains outside ordinary permission (Selectors4/Pseudo4).
#[test]
fn ordinary_invalid_logical_proof_can_be_reused_by_named_descendant_but_not_search_text() {
    let report = parse_selector(":is(:NoPe(&))", &CssNamespaceContext::default());
    assert!(report.diagnostics().iter().any(|diagnostic| {
        diagnostic.action() == CssRecoveryAction::PreserveInvalidSelectorListItem
    }));
    let CssSelector::PseudoClass(CssPseudoClass::Is(list)) = report.syntax().as_ref().unwrap()
    else {
        panic!("ordinary logical proof")
    };
    let argument = CssViewTransitionNameSelector::Name(CssCustomIdent::try_new("Card").unwrap());
    let sequence = CssPseudoElementSequence::try_from_segments(vec![
        CssPseudoElementSegment::PseudoElement(CssPseudoElement::ViewTransitionOld(argument)),
        CssPseudoElementSegment::PseudoClass(CssPseudoClass::Is(list.clone())),
    ]);
    assert!(sequence.is_some());
    assert!(
        CssPseudoElementSequence::try_from_segments(vec![
            CssPseudoElementSegment::PseudoElement(CssPseudoElement::SearchText),
            CssPseudoElementSegment::PseudoClass(CssPseudoClass::Is(list.clone())),
        ])
        .is_none()
    );
}
