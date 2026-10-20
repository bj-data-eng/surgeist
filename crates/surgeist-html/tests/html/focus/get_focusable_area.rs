//! Independent literal traces from HTML877d raw60083–60110. Selection actions
//! carry real adapter obligations; these pure tests do not supply geometry.
use surgeist_html::{
    HtmlCurrentlyFocusedAreaOutcome as CurrentOutcome, HtmlGetFocusableAreaAction as A,
    HtmlGetFocusableAreaCursor as C, HtmlGetFocusableAreaError as E,
    HtmlGetFocusableAreaOutcome as O, HtmlGetFocusableAreaReply as R,
};

const REPLIES: [R; 22] = [
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
    R::CurrentlyFocusedArea(CurrentOutcome::Null),
    R::CurrentlyFocusedArea(CurrentOutcome::CandidateDocument),
    R::CurrentlyFocusedArea(CurrentOutcome::CandidateFocusedArea),
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
            R::CurrentlyFocusedArea(CurrentOutcome::Null),
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
        R::CurrentlyFocusedArea(CurrentOutcome::CandidateFocusedArea),
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
            R::CurrentlyFocusedArea(CurrentOutcome::CandidateFocusedArea),
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
    let CA::Complete(current_result) = current.action() else {
        panic!("the current-area child must actually complete")
    };
    step(
        &mut parent,
        A::RunCurrentlyFocusedArea,
        R::CurrentlyFocusedArea(current_result),
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

#[test]
fn candidate_document_from_real_current_area_does_not_enter_area_anchor_projection() {
    use surgeist_html::{
        HtmlCurrentlyFocusedAreaAction as CA, HtmlCurrentlyFocusedAreaCursor as CC,
        HtmlCurrentlyFocusedAreaOutcome as CO, HtmlCurrentlyFocusedAreaReply as CR,
        HtmlFocusedAreaKind,
    };

    // HTML877d raw59751–59758 returns the candidate Document when its
    // designation is null. It does not create a viewport or a focusable area.
    // The selected #1220 shadow-branch interpretation sends that Document to
    // real delegation without the area-only DOM-anchor/ancestor fast path.
    let mut current = CC::new();
    assert_eq!(current.action(), CA::ReadSystemFocus);
    current.reply(CR::SystemFocus(true)).unwrap();
    assert_eq!(current.action(), CA::CaptureTraversableActiveDocument);
    current.reply(CR::Applied).unwrap();
    assert_eq!(current.action(), CA::ReadLoopFocusedArea);
    current
        .reply(CR::LoopFocusedArea(HtmlFocusedAreaKind::Null))
        .unwrap();
    assert_eq!(current.action(), CA::ReadResultFocusedArea);
    current.reply(CR::ResultFocusedAreaPresent(false)).unwrap();
    assert_eq!(current.action(), CA::Complete(CO::CandidateDocument));

    let mut parent = delegating_shadow();
    assert_eq!(parent.action(), A::RunCurrentlyFocusedArea);
    let result = match current.action() {
        CA::Complete(result @ CO::CandidateDocument) => result,
        other => panic!("expected the actual completed Document result: {other:?}"),
    };
    parent.reply(R::CurrentlyFocusedArea(result)).unwrap();
    assert_ne!(parent.action(), A::CaptureCurrentAreaAnchor);
}

fn actual_current_result(has_designated_area: bool) -> CurrentOutcome {
    use surgeist_html::{
        HtmlCurrentlyFocusedAreaAction as CA, HtmlCurrentlyFocusedAreaCursor as CC,
        HtmlCurrentlyFocusedAreaReply as CR, HtmlFocusedAreaKind,
    };
    let mut current = CC::new();
    current.reply(CR::SystemFocus(true)).unwrap();
    assert_eq!(current.action(), CA::CaptureTraversableActiveDocument);
    current.reply(CR::Applied).unwrap();
    assert_eq!(current.action(), CA::ReadLoopFocusedArea);
    current
        .reply(CR::LoopFocusedArea(if has_designated_area {
            HtmlFocusedAreaKind::OtherArea
        } else {
            HtmlFocusedAreaKind::Null
        }))
        .unwrap();
    assert_eq!(current.action(), CA::ReadResultFocusedArea);
    current
        .reply(CR::ResultFocusedAreaPresent(has_designated_area))
        .unwrap();
    let CA::Complete(result) = current.action() else {
        panic!("actual current-area child must return")
    };
    result
}

fn actual_delegate_result(autofocus_descendant: bool) -> bool {
    use surgeist_html::{
        HtmlAutofocusDelegateAction as AA, HtmlAutofocusDelegateCursor as AC,
        HtmlAutofocusDelegateOutcome as AO, HtmlAutofocusDelegateReply as AR,
        HtmlFocusDelegateAction as DA, HtmlFocusDelegateCandidate as Candidate,
        HtmlFocusDelegateCursor as DC, HtmlFocusDelegateOutcome as DO,
        HtmlFocusDelegateReply as DR, HtmlFocusDelegateRoot as Root,
        HtmlFocusDelegateShadowRoot as Shadow, HtmlFocusDelegateTargetKind as Kind,
        HtmlFocusTriggerKind as Trigger,
    };
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
    assert_eq!(auto.action(), AA::BeginDescendants);
    auto.reply(AR::Applied).unwrap();
    assert_eq!(auto.action(), AA::NextDescendant);
    auto.reply(AR::DescendantPresent(autofocus_descendant))
        .unwrap();
    if autofocus_descendant {
        assert_eq!(auto.action(), AA::ReadAutofocus);
        auto.reply(AR::Autofocus(true)).unwrap();
        assert_eq!(auto.action(), AA::ReadFocusableArea);
        auto.reply(AR::FocusableArea(true)).unwrap();
        assert_eq!(
            auto.action(),
            AA::Complete(AO::Candidate(Candidate::Descendant))
        );
    } else {
        assert_eq!(auto.action(), AA::Complete(AO::Null));
    }
    let auto_present = match auto.action() {
        AA::Complete(AO::Null) => false,
        AA::Complete(AO::Candidate(_)) => true,
        other => panic!("actual autofocus child must complete: {other:?}"),
    };
    delegate
        .reply(DR::AutofocusDelegatePresent(auto_present))
        .unwrap();
    if !auto_present {
        assert_eq!(
            delegate.action(),
            DA::BeginDescendants {
                root: Root::ShadowRoot
            }
        );
        delegate.reply(DR::Applied).unwrap();
        assert_eq!(delegate.action(), DA::NextDescendant);
        delegate.reply(DR::DescendantPresent(false)).unwrap();
    }
    match delegate.action() {
        DA::Complete(DO::Null) => false,
        DA::Complete(DO::AutofocusDelegate | DO::Candidate(_)) => true,
        other => panic!("actual focus-delegate child must complete: {other:?}"),
    }
}

#[test]
fn current_document_result_runs_the_real_delegate_for_both_null_and_candidate_returns() {
    for autofocus_descendant in [false, true] {
        let result = actual_current_result(false);
        assert_eq!(result, CurrentOutcome::CandidateDocument);
        let mut parent = delegating_shadow();
        step(
            &mut parent,
            A::RunCurrentlyFocusedArea,
            R::CurrentlyFocusedArea(result),
        );
        assert_eq!(parent.action(), A::RunFocusDelegate);
        let waiting = parent;
        let present = actual_delegate_result(autofocus_descendant);
        assert_eq!(present, autofocus_descendant);
        assert_eq!(parent, waiting);
        step(
            &mut parent,
            A::RunFocusDelegate,
            R::FocusDelegatePresent(present),
        );
        complete(&mut parent, if present { O::Delegate } else { O::Null });
    }
}

#[test]
fn actual_area_and_viewport_with_document_anchor_still_project_their_real_anchors() {
    // These are distinct retained area fixtures. The second is a viewport,
    // whose Document anchor must not be confused with a bare Document result.
    for anchor_is_element in [true, false] {
        let result = actual_current_result(true);
        assert_eq!(result, CurrentOutcome::CandidateFocusedArea);
        let mut parent = delegating_shadow();
        step(
            &mut parent,
            A::RunCurrentlyFocusedArea,
            R::CurrentlyFocusedArea(result),
        );
        step(&mut parent, A::CaptureCurrentAreaAnchor, R::Applied);
        // The original target can contain the Element anchor. An Element is
        // not an inclusive ancestor of the viewport's Document anchor.
        step(
            &mut parent,
            A::ReadShadowIncludingInclusiveAncestor,
            R::ShadowIncludingInclusiveAncestor(anchor_is_element),
        );
        if anchor_is_element {
            complete(&mut parent, O::CurrentAnchor);
        } else {
            let present = actual_delegate_result(false);
            step(
                &mut parent,
                A::RunFocusDelegate,
                R::FocusDelegatePresent(present),
            );
            complete(&mut parent, O::Null);
        }
    }
}

#[test]
fn unavailable_current_child_is_not_a_null_or_document_reply_and_wrong_stage_preserves_wait() {
    use surgeist_html::{
        HtmlCurrentlyFocusedAreaAction as CA, HtmlCurrentlyFocusedAreaCursor as CC,
        HtmlCurrentlyFocusedAreaReply as CR,
    };
    let mut parent = delegating_shadow();
    let waiting = parent;
    let mut current = CC::new();
    current.reply(CR::SystemFocus(true)).unwrap();
    assert_eq!(current.action(), CA::CaptureTraversableActiveDocument);
    // No actual active-Document evidence means the child supplies no reply.
    assert_eq!(parent.action(), A::RunCurrentlyFocusedArea);
    assert_eq!(parent, waiting);
    assert_eq!(parent.reply(R::Applied), Err(E::WrongReply));
    assert_eq!(
        parent.reply(R::FocusDelegatePresent(false)),
        Err(E::WrongReply)
    );
    assert_eq!(parent, waiting);

    let mut other = before_shadow();
    let before = other;
    for result in [
        CurrentOutcome::Null,
        CurrentOutcome::CandidateDocument,
        CurrentOutcome::CandidateFocusedArea,
    ] {
        assert_eq!(
            other.reply(R::CurrentlyFocusedArea(result)),
            Err(E::WrongReply)
        );
        assert_eq!(other, before);
    }
}
