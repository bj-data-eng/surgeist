//! Independent focusing-step traces from HTML877d raw60163–60185. Called
//! chains/update remain real dependencies, not services supplied by this test.
use surgeist_html::{
    HtmlFocusingAction as A, HtmlFocusingCursor as C, HtmlFocusingError as E,
    HtmlFocusingFallback as Fallback, HtmlFocusingReply as R,
};

const REPLIES: [R; 19] = [
    R::InitialTargetFocusableArea(false),
    R::InitialTargetFocusableArea(true),
    R::ResolvedTargetPresent(false),
    R::ResolvedTargetPresent(true),
    R::TargetIsNavigableContainer(false),
    R::TargetIsNavigableContainer(true),
    R::ContentNavigablePresent(false),
    R::ContentNavigablePresent(true),
    R::ReplacementTargetFocusableArea(false),
    R::ReplacementTargetFocusableArea(true),
    R::ActualAnchorInert(false),
    R::ActualAnchorInert(true),
    R::CurrentlyFocusedAreaReady,
    R::SameTarget(false),
    R::SameTarget(true),
    R::CurrentFocusChainReady,
    R::FocusChainReady,
    R::FocusUpdateCompleted,
    R::Applied,
];
fn step(c: &mut C, action: A, reply: R) {
    assert_eq!(c.action(), action);
    for wrong in REPLIES {
        if std::mem::discriminant(&wrong) != std::mem::discriminant(&reply) {
            let before = *c;
            assert_eq!(c.reply(wrong), Err(E::WrongReply));
            assert_eq!(*c, before);
        }
    }
    c.reply(reply).unwrap();
}
fn complete(c: &mut C) {
    assert_eq!(c.action(), A::Complete);
    for reply in REPLIES {
        let before = *c;
        assert_eq!(c.reply(reply), Err(E::Terminal));
        assert_eq!(*c, before);
    }
}
fn initial_area() -> C {
    let mut c = C::new(Fallback::Omitted);
    step(
        &mut c,
        A::ReadInitialTargetFocusableArea,
        R::InitialTargetFocusableArea(true),
    );
    c
}
fn before_current_area() -> C {
    let mut c = initial_area();
    step(
        &mut c,
        A::ReadTargetIsNavigableContainer,
        R::TargetIsNavigableContainer(false),
    );
    step(
        &mut c,
        A::ReadReplacementTargetFocusableArea,
        R::ReplacementTargetFocusableArea(true),
    );
    step(
        &mut c,
        A::ReadActualAnchorInert,
        R::ActualAnchorInert(false),
    );
    c
}
fn equal_target_tail(c: &mut C) {
    step(c, A::RunCurrentlyFocusedArea, R::CurrentlyFocusedAreaReady);
    step(c, A::ReadSameTarget, R::SameTarget(true));
    complete(c);
}

#[test]
fn known_initial_area_bypasses_resolution_and_ignores_unused_supplied_fallback() {
    for fallback in [Fallback::Omitted, Fallback::Supplied] {
        let mut c = C::new(fallback);
        step(
            &mut c,
            A::ReadInitialTargetFocusableArea,
            R::InitialTargetFocusableArea(true),
        );
        step(
            &mut c,
            A::ReadTargetIsNavigableContainer,
            R::TargetIsNavigableContainer(false),
        );
        step(
            &mut c,
            A::ReadReplacementTargetFocusableArea,
            R::ReplacementTargetFocusableArea(true),
        );
        step(&mut c, A::ReadActualAnchorInert, R::ActualAnchorInert(true));
        complete(&mut c);
    }
}
#[test]
fn null_resolution_without_fallback_returns_before_container_or_focus_queries() {
    let mut c = C::new(Fallback::Omitted);
    step(
        &mut c,
        A::ReadInitialTargetFocusableArea,
        R::InitialTargetFocusableArea(false),
    );
    step(
        &mut c,
        A::RunGetFocusableArea,
        R::ResolvedTargetPresent(false),
    );
    complete(&mut c);
}
#[test]
fn null_resolution_installs_supplied_fallback_once_without_second_resolution() {
    let mut c = C::new(Fallback::Supplied);
    step(
        &mut c,
        A::ReadInitialTargetFocusableArea,
        R::InitialTargetFocusableArea(false),
    );
    step(
        &mut c,
        A::RunGetFocusableArea,
        R::ResolvedTargetPresent(false),
    );
    step(&mut c, A::UseFallbackTarget, R::Applied);
    step(
        &mut c,
        A::ReadTargetIsNavigableContainer,
        R::TargetIsNavigableContainer(false),
    );
    step(
        &mut c,
        A::ReadReplacementTargetFocusableArea,
        R::ReplacementTargetFocusableArea(true),
    );
    step(&mut c, A::ReadActualAnchorInert, R::ActualAnchorInert(true));
    complete(&mut c);
}
#[test]
fn actual_nonnull_resolution_does_not_replace_result_with_supplied_fallback() {
    let mut c = C::new(Fallback::Supplied);
    step(
        &mut c,
        A::ReadInitialTargetFocusableArea,
        R::InitialTargetFocusableArea(false),
    );
    step(
        &mut c,
        A::RunGetFocusableArea,
        R::ResolvedTargetPresent(true),
    );
    step(
        &mut c,
        A::ReadTargetIsNavigableContainer,
        R::TargetIsNavigableContainer(false),
    );
    step(
        &mut c,
        A::ReadReplacementTargetFocusableArea,
        R::ReplacementTargetFocusableArea(false),
    );
    equal_target_tail(&mut c);
}
#[test]
fn content_active_document_replacement_precedes_fresh_area_test_and_skips_old_anchor() {
    let mut c = initial_area();
    step(
        &mut c,
        A::ReadTargetIsNavigableContainer,
        R::TargetIsNavigableContainer(true),
    );
    step(
        &mut c,
        A::ReadContentNavigable,
        R::ContentNavigablePresent(true),
    );
    step(&mut c, A::CaptureContentActiveDocument, R::Applied);
    // The original area was true. The replacement Document has its own fresh
    // classification and must not reuse the old target/anchor's classification.
    step(
        &mut c,
        A::ReadReplacementTargetFocusableArea,
        R::ReplacementTargetFocusableArea(false),
    );
    equal_target_tail(&mut c);
}
#[test]
fn container_without_content_keeps_target_and_inert_anchor_returns_before_current_area() {
    let mut c = initial_area();
    step(
        &mut c,
        A::ReadTargetIsNavigableContainer,
        R::TargetIsNavigableContainer(true),
    );
    step(
        &mut c,
        A::ReadContentNavigable,
        R::ContentNavigablePresent(false),
    );
    step(
        &mut c,
        A::ReadReplacementTargetFocusableArea,
        R::ReplacementTargetFocusableArea(true),
    );
    step(&mut c, A::ReadActualAnchorInert, R::ActualAnchorInert(true));
    complete(&mut c);
}
#[test]
fn actual_current_area_identity_equality_returns_before_either_chain() {
    let mut c = before_current_area();
    equal_target_tail(&mut c);
}
#[test]
fn different_target_requires_old_chain_then_new_chain_then_actual_focus_update_completion() {
    let mut c = before_current_area();
    step(
        &mut c,
        A::RunCurrentlyFocusedArea,
        R::CurrentlyFocusedAreaReady,
    );
    step(&mut c, A::ReadSameTarget, R::SameTarget(false));
    step(&mut c, A::RunCurrentFocusChain, R::CurrentFocusChainReady);
    step(&mut c, A::RunFocusChain, R::FocusChainReady);
    let waiting = c;
    // Producing the update action, or an effect acknowledgment, is not its
    // genuine called completion. Earlier chain receipts cannot settle it.
    assert_eq!(c.reply(R::Applied), Err(E::WrongReply));
    assert_eq!(c, waiting);
    assert_eq!(c.reply(R::FocusChainReady), Err(E::WrongReply));
    assert_eq!(c, waiting);
    step(&mut c, A::RunFocusUpdate, R::FocusUpdateCompleted);
    complete(&mut c);
}
#[test]
fn real_current_area_child_null_result_is_not_whole_focusing_completion() {
    use surgeist_html::{
        HtmlCurrentlyFocusedAreaAction as CA, HtmlCurrentlyFocusedAreaCursor as CC,
        HtmlCurrentlyFocusedAreaOutcome as CO, HtmlCurrentlyFocusedAreaReply as CR,
    };
    let mut parent = before_current_area();
    let mut child = CC::new();
    child.reply(CR::SystemFocus(false)).unwrap();
    assert_eq!(child.action(), CA::Complete(CO::Null));
    step(
        &mut parent,
        A::RunCurrentlyFocusedArea,
        R::CurrentlyFocusedAreaReady,
    );
    step(&mut parent, A::ReadSameTarget, R::SameTarget(false));
    // Known absent traversable / known null current area can qualify an actual
    // empty current chain. This parent still requires its distinct real receipt
    // and the subsequent new-chain/update dependencies; it invents no chain.
    step(
        &mut parent,
        A::RunCurrentFocusChain,
        R::CurrentFocusChainReady,
    );
    step(&mut parent, A::RunFocusChain, R::FocusChainReady);
    step(&mut parent, A::RunFocusUpdate, R::FocusUpdateCompleted);
    complete(&mut parent);
}
