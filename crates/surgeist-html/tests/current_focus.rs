//! Independent literal HTML877d raw59747–59758 query traces. The pure owner
//! returns roles; exact Document/area identities remain with the actual adapter.
use surgeist_html::{
    HtmlCurrentlyFocusedAreaAction as A, HtmlCurrentlyFocusedAreaCursor as C,
    HtmlCurrentlyFocusedAreaError as E, HtmlCurrentlyFocusedAreaOutcome as O,
    HtmlCurrentlyFocusedAreaReply as R, HtmlFocusedAreaKind as Area,
};

const REPLIES: [R; 10] = [
    R::SystemFocus(false),
    R::SystemFocus(true),
    R::LoopFocusedArea(Area::Null),
    R::LoopFocusedArea(Area::NavigableContainer),
    R::LoopFocusedArea(Area::OtherArea),
    R::ContentNavigablePresent(false),
    R::ContentNavigablePresent(true),
    R::ResultFocusedAreaPresent(false),
    R::ResultFocusedAreaPresent(true),
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
fn complete(c: &mut C, outcome: O) {
    assert_eq!(c.action(), A::Complete(outcome));
    for reply in REPLIES {
        let before = *c;
        assert_eq!(c.reply(reply), Err(E::Terminal));
        assert_eq!(*c, before);
    }
}
fn system_focused() -> C {
    let mut c = C::new();
    step(&mut c, A::ReadSystemFocus, R::SystemFocus(true));
    step(&mut c, A::CaptureTraversableActiveDocument, R::Applied);
    c
}
#[test]
fn absent_system_focus_returns_null_before_any_document_read() {
    let mut c = C::new();
    step(&mut c, A::ReadSystemFocus, R::SystemFocus(false));
    complete(&mut c, O::Null);
}
#[test]
fn candidate_null_designation_returns_candidate_document_after_separate_final_read() {
    let mut c = system_focused();
    step(
        &mut c,
        A::ReadLoopFocusedArea,
        R::LoopFocusedArea(Area::Null),
    );
    step(
        &mut c,
        A::ReadResultFocusedArea,
        R::ResultFocusedAreaPresent(false),
    );
    complete(&mut c, O::CandidateDocument);
}
#[test]
fn ordinary_area_returns_actual_candidate_focused_area_without_content_query() {
    let mut c = system_focused();
    step(
        &mut c,
        A::ReadLoopFocusedArea,
        R::LoopFocusedArea(Area::OtherArea),
    );
    step(
        &mut c,
        A::ReadResultFocusedArea,
        R::ResultFocusedAreaPresent(true),
    );
    complete(&mut c, O::CandidateFocusedArea);
}
#[test]
fn navigable_container_without_content_returns_its_area_from_the_final_read() {
    let mut c = system_focused();
    step(
        &mut c,
        A::ReadLoopFocusedArea,
        R::LoopFocusedArea(Area::NavigableContainer),
    );
    step(
        &mut c,
        A::ReadContentNavigable,
        R::ContentNavigablePresent(false),
    );
    step(
        &mut c,
        A::ReadResultFocusedArea,
        R::ResultFocusedAreaPresent(true),
    );
    complete(&mut c, O::CandidateFocusedArea);
}
#[test]
fn navigable_container_with_content_captures_child_document_before_repeating_loop() {
    let mut c = system_focused();
    step(
        &mut c,
        A::ReadLoopFocusedArea,
        R::LoopFocusedArea(Area::NavigableContainer),
    );
    step(
        &mut c,
        A::ReadContentNavigable,
        R::ContentNavigablePresent(true),
    );
    step(&mut c, A::CaptureContentActiveDocument, R::Applied);
    step(
        &mut c,
        A::ReadLoopFocusedArea,
        R::LoopFocusedArea(Area::Null),
    );
    step(
        &mut c,
        A::ReadResultFocusedArea,
        R::ResultFocusedAreaPresent(false),
    );
    complete(&mut c, O::CandidateDocument);
}
#[test]
fn multiple_content_documents_are_visited_in_order_before_returning_deepest_area() {
    let mut c = system_focused();
    step(
        &mut c,
        A::ReadLoopFocusedArea,
        R::LoopFocusedArea(Area::NavigableContainer),
    );
    step(
        &mut c,
        A::ReadContentNavigable,
        R::ContentNavigablePresent(true),
    );
    step(&mut c, A::CaptureContentActiveDocument, R::Applied);
    step(
        &mut c,
        A::ReadLoopFocusedArea,
        R::LoopFocusedArea(Area::NavigableContainer),
    );
    step(
        &mut c,
        A::ReadContentNavigable,
        R::ContentNavigablePresent(true),
    );
    step(&mut c, A::CaptureContentActiveDocument, R::Applied);
    step(
        &mut c,
        A::ReadLoopFocusedArea,
        R::LoopFocusedArea(Area::OtherArea),
    );
    step(
        &mut c,
        A::ReadResultFocusedArea,
        R::ResultFocusedAreaPresent(true),
    );
    complete(&mut c, O::CandidateFocusedArea);
}
#[test]
fn final_result_uses_actual_final_designation_instead_of_cached_loop_observation() {
    for (loop_area, final_present, outcome) in [
        (Area::Null, true, O::CandidateFocusedArea),
        (Area::OtherArea, false, O::CandidateDocument),
    ] {
        let mut c = system_focused();
        step(
            &mut c,
            A::ReadLoopFocusedArea,
            R::LoopFocusedArea(loop_area),
        );
        step(
            &mut c,
            A::ReadResultFocusedArea,
            R::ResultFocusedAreaPresent(final_present),
        );
        complete(&mut c, outcome);
    }
    let mut c = system_focused();
    step(
        &mut c,
        A::ReadLoopFocusedArea,
        R::LoopFocusedArea(Area::NavigableContainer),
    );
    step(
        &mut c,
        A::ReadContentNavigable,
        R::ContentNavigablePresent(false),
    );
    step(
        &mut c,
        A::ReadResultFocusedArea,
        R::ResultFocusedAreaPresent(false),
    );
    complete(&mut c, O::CandidateDocument);
}
#[test]
fn malformed_stage_replies_preserve_pending_document_and_content_captures() {
    let mut c = C::new();
    let initial = c;
    assert_eq!(
        c.reply(R::ResultFocusedAreaPresent(false)),
        Err(E::WrongReply)
    );
    assert_eq!(c, initial);
    step(&mut c, A::ReadSystemFocus, R::SystemFocus(true));
    let waiting = c;
    assert_eq!(c.reply(R::LoopFocusedArea(Area::Null)), Err(E::WrongReply));
    assert_eq!(c, waiting);
    step(&mut c, A::CaptureTraversableActiveDocument, R::Applied);
    step(
        &mut c,
        A::ReadLoopFocusedArea,
        R::LoopFocusedArea(Area::NavigableContainer),
    );
    step(
        &mut c,
        A::ReadContentNavigable,
        R::ContentNavigablePresent(true),
    );
    let waiting = c;
    assert_eq!(
        c.reply(R::ContentNavigablePresent(false)),
        Err(E::WrongReply)
    );
    assert_eq!(c, waiting);
    step(&mut c, A::CaptureContentActiveDocument, R::Applied);
    step(
        &mut c,
        A::ReadLoopFocusedArea,
        R::LoopFocusedArea(Area::Null),
    );
    step(
        &mut c,
        A::ReadResultFocusedArea,
        R::ResultFocusedAreaPresent(false),
    );
    complete(&mut c, O::CandidateDocument);
}
