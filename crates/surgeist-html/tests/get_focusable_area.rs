//! Independent literal traces from HTML877d raw60083–60110. Selection actions
//! carry real adapter obligations; these pure tests do not supply geometry.
use surgeist_html::{
    HtmlGetFocusableAreaAction as A, HtmlGetFocusableAreaCursor as C,
    HtmlGetFocusableAreaError as E, HtmlGetFocusableAreaOutcome as O,
    HtmlGetFocusableAreaReply as R,
};

const REPLIES: [R; 21] = [
    R::AreaWithFocusableShapes(false),
    R::AreaWithFocusableShapes(true),
    R::ElementWithFocusableScrollRegions(false),
    R::ElementWithFocusableScrollRegions(true),
    R::DocumentElement(false),
    R::DocumentElement(true),
    R::Navigable(false),
    R::Navigable(true),
    R::NavigableContainer(false),
    R::NavigableContainer(true),
    R::ContentNavigablePresent(false),
    R::ContentNavigablePresent(true),
    R::DelegatingShadowHost(false),
    R::DelegatingShadowHost(true),
    R::CurrentlyFocusedAreaPresent(false),
    R::CurrentlyFocusedAreaPresent(true),
    R::ShadowIncludingInclusiveAncestor(false),
    R::ShadowIncludingInclusiveAncestor(true),
    R::FocusDelegatePresent(false),
    R::FocusDelegatePresent(true),
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
fn before_document_element() -> C {
    let mut c = C::new();
    step(
        &mut c,
        A::ReadAreaWithFocusableShapes,
        R::AreaWithFocusableShapes(false),
    );
    step(
        &mut c,
        A::ReadElementWithFocusableScrollRegions,
        R::ElementWithFocusableScrollRegions(false),
    );
    c
}
fn before_navigable() -> C {
    let mut c = before_document_element();
    step(&mut c, A::ReadDocumentElement, R::DocumentElement(false));
    c
}
fn before_container() -> C {
    let mut c = before_navigable();
    step(&mut c, A::ReadNavigable, R::Navigable(false));
    c
}
fn before_shadow() -> C {
    let mut c = before_container();
    step(
        &mut c,
        A::ReadNavigableContainer,
        R::NavigableContainer(false),
    );
    c
}
fn delegating_shadow() -> C {
    let mut c = before_shadow();
    step(
        &mut c,
        A::ReadDelegatingShadowHost,
        R::DelegatingShadowHost(true),
    );
    c
}

#[test]
fn area_shape_clause_requires_first_image_map_selection_before_any_later_clause() {
    let mut c = C::new();
    step(
        &mut c,
        A::ReadAreaWithFocusableShapes,
        R::AreaWithFocusableShapes(true),
    );
    // The selected action requires the first img in ordinary tree order using
    // the target's map, not the first already-known/focusable retained shape.
    let before = c;
    assert_eq!(
        c.reply(R::ElementWithFocusableScrollRegions(true)),
        Err(E::WrongReply)
    );
    assert_eq!(c, before);
    step(&mut c, A::SelectFirstImageMapShape, R::Applied);
    complete(&mut c, O::MapShape);
}
#[test]
fn scroll_clause_requires_first_flat_preorder_region_before_document_element_clause() {
    let mut c = C::new();
    step(
        &mut c,
        A::ReadAreaWithFocusableShapes,
        R::AreaWithFocusableShapes(false),
    );
    step(
        &mut c,
        A::ReadElementWithFocusableScrollRegions,
        R::ElementWithFocusableScrollRegions(true),
    );
    let before = c;
    assert_eq!(c.reply(R::DocumentElement(true)), Err(E::WrongReply));
    assert_eq!(c, before);
    step(&mut c, A::SelectFirstScrollableRegion, R::Applied);
    complete(&mut c, O::ScrollRegion);
}
#[test]
fn document_element_captures_actual_viewport_before_navigable_clause() {
    let mut c = before_document_element();
    step(&mut c, A::ReadDocumentElement, R::DocumentElement(true));
    step(&mut c, A::CaptureViewport, R::Applied);
    complete(&mut c, O::Viewport);
}
#[test]
fn navigable_returns_actual_active_document_with_distinct_document_result_role() {
    let mut c = before_navigable();
    step(&mut c, A::ReadNavigable, R::Navigable(true));
    step(&mut c, A::CaptureNavigableActiveDocument, R::Applied);
    complete(&mut c, O::NavigableActiveDocument);
}
#[test]
fn container_with_actual_content_returns_its_active_document_before_shadow_clause() {
    let mut c = before_container();
    step(
        &mut c,
        A::ReadNavigableContainer,
        R::NavigableContainer(true),
    );
    step(
        &mut c,
        A::ReadContentNavigable,
        R::ContentNavigablePresent(true),
    );
    step(&mut c, A::CaptureContentActiveDocument, R::Applied);
    complete(&mut c, O::ContentActiveDocument);
}
#[test]
fn container_without_content_continues_to_shadow_clause_and_nonmatching_target_returns_null() {
    let mut c = before_container();
    step(
        &mut c,
        A::ReadNavigableContainer,
        R::NavigableContainer(true),
    );
    step(
        &mut c,
        A::ReadContentNavigable,
        R::ContentNavigablePresent(false),
    );
    step(
        &mut c,
        A::ReadDelegatingShadowHost,
        R::DelegatingShadowHost(false),
    );
    complete(&mut c, O::Null);
    let mut c = before_shadow();
    step(
        &mut c,
        A::ReadDelegatingShadowHost,
        R::DelegatingShadowHost(false),
    );
    complete(&mut c, O::Null);
}
#[test]
fn known_null_current_area_skips_anchor_queries_and_runs_real_delegate_for_either_return() {
    for present in [false, true] {
        let mut c = delegating_shadow();
        step(
            &mut c,
            A::RunCurrentlyFocusedArea,
            R::CurrentlyFocusedAreaPresent(false),
        );
        step(
            &mut c,
            A::RunFocusDelegate,
            R::FocusDelegatePresent(present),
        );
        complete(&mut c, if present { O::Delegate } else { O::Null });
    }
}
#[test]
fn actual_current_anchor_in_shadow_including_inclusive_ancestry_bypasses_delegate() {
    let mut c = delegating_shadow();
    step(
        &mut c,
        A::RunCurrentlyFocusedArea,
        R::CurrentlyFocusedAreaPresent(true),
    );
    step(&mut c, A::CaptureCurrentAreaAnchor, R::Applied);
    step(
        &mut c,
        A::ReadShadowIncludingInclusiveAncestor,
        R::ShadowIncludingInclusiveAncestor(true),
    );
    complete(&mut c, O::CurrentAnchor);
}
#[test]
fn current_anchor_outside_ancestry_runs_delegate_and_keeps_delegate_result_role() {
    for present in [false, true] {
        let mut c = delegating_shadow();
        step(
            &mut c,
            A::RunCurrentlyFocusedArea,
            R::CurrentlyFocusedAreaPresent(true),
        );
        step(&mut c, A::CaptureCurrentAreaAnchor, R::Applied);
        step(
            &mut c,
            A::ReadShadowIncludingInclusiveAncestor,
            R::ShadowIncludingInclusiveAncestor(false),
        );
        step(
            &mut c,
            A::RunFocusDelegate,
            R::FocusDelegatePresent(present),
        );
        complete(&mut c, if present { O::Delegate } else { O::Null });
    }
}
#[test]
fn genuine_existing_children_complete_before_parent_accepts_their_nullable_returns() {
    use surgeist_html::{
        HtmlAutofocusDelegateAction as AA, HtmlAutofocusDelegateCursor as AC,
        HtmlAutofocusDelegateOutcome as AO, HtmlAutofocusDelegateReply as AR,
        HtmlCurrentlyFocusedAreaAction as CA, HtmlCurrentlyFocusedAreaCursor as CC,
        HtmlCurrentlyFocusedAreaOutcome as CO, HtmlCurrentlyFocusedAreaReply as CR,
        HtmlFocusDelegateAction as DA, HtmlFocusDelegateCursor as DC,
        HtmlFocusDelegateOutcome as DO, HtmlFocusDelegateReply as DR,
        HtmlFocusDelegateRoot as Root, HtmlFocusDelegateShadowRoot as Shadow,
        HtmlFocusDelegateTargetKind as Kind, HtmlFocusTriggerKind as Trigger,
    };
    let mut parent = delegating_shadow();
    let mut current = CC::new();
    current.reply(CR::SystemFocus(false)).unwrap();
    assert_eq!(current.action(), CA::Complete(CO::Null));
    step(
        &mut parent,
        A::RunCurrentlyFocusedArea,
        R::CurrentlyFocusedAreaPresent(false),
    );
    let waiting = parent;
    let mut delegate = DC::new(Kind::Other);
    delegate
        .reply(DR::ShadowRoot(Shadow::DelegatesFocus))
        .unwrap();
    assert_eq!(
        delegate.action(),
        DA::RunAutofocusDelegate {
            root: Root::ShadowRoot
        }
    );
    let mut auto = AC::new(Trigger::Other);
    auto.reply(AR::Applied).unwrap();
    assert_eq!(auto.action(), AA::NextDescendant);
    auto.reply(AR::DescendantPresent(false)).unwrap();
    assert_eq!(auto.action(), AA::Complete(AO::Null));
    delegate.reply(DR::AutofocusDelegatePresent(false)).unwrap();
    assert_eq!(
        delegate.action(),
        DA::BeginDescendants {
            root: Root::ShadowRoot
        }
    );
    delegate.reply(DR::Applied).unwrap();
    delegate.reply(DR::DescendantPresent(false)).unwrap();
    assert_eq!(delegate.action(), DA::Complete(DO::Null));
    assert_eq!(parent, waiting);
    assert_eq!(parent.reply(R::Applied), Err(E::WrongReply));
    assert_eq!(parent, waiting);
    step(
        &mut parent,
        A::RunFocusDelegate,
        R::FocusDelegatePresent(false),
    );
    complete(&mut parent, O::Null);
}
