//! Independent literal algorithm traces from HTML877d raw60113–60159.
use surgeist_html::{
    HtmlAutofocusDelegateAction as AA, HtmlAutofocusDelegateCursor as AC,
    HtmlAutofocusDelegateError as AE, HtmlAutofocusDelegateOutcome as AO,
    HtmlAutofocusDelegateReply as AR, HtmlFocusDelegateAction as A,
    HtmlFocusDelegateCandidate as Candidate, HtmlFocusDelegateCursor as C,
    HtmlFocusDelegateError as E, HtmlFocusDelegateOutcome as O, HtmlFocusDelegateReply as R,
    HtmlFocusDelegateRoot as Root, HtmlFocusDelegateShadowRoot as Shadow,
    HtmlFocusDelegateTargetKind as Kind, HtmlFocusTriggerKind as Trigger,
};

const REPLIES: [R; 14] = [
    R::ShadowRoot(Shadow::Absent),
    R::ShadowRoot(Shadow::DelegatesFocus),
    R::ShadowRoot(Shadow::DoesNotDelegateFocus),
    R::AutofocusDelegatePresent(false),
    R::AutofocusDelegatePresent(true),
    R::DescendantPresent(false),
    R::DescendantPresent(true),
    R::SequentiallyFocusable(false),
    R::SequentiallyFocusable(true),
    R::FocusableArea(false),
    R::FocusableArea(true),
    R::ResolvedAreaPresent(false),
    R::ResolvedAreaPresent(true),
    R::Applied,
];
const AUTO_REPLIES: [AR; 11] = [
    AR::DescendantPresent(false),
    AR::DescendantPresent(true),
    AR::Autofocus(false),
    AR::Autofocus(true),
    AR::FocusableArea(false),
    AR::FocusableArea(true),
    AR::ResolvedAreaPresent(false),
    AR::ResolvedAreaPresent(true),
    AR::ClickFocusable(false),
    AR::ClickFocusable(true),
    AR::Applied,
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
fn auto_step(c: &mut AC, action: AA, reply: AR) {
    assert_eq!(c.action(), action);
    for wrong in AUTO_REPLIES {
        if std::mem::discriminant(&wrong) != std::mem::discriminant(&reply) {
            let before = *c;
            assert_eq!(c.reply(wrong), Err(AE::WrongReply));
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
fn auto_complete(c: &mut AC, outcome: AO) {
    assert_eq!(c.action(), AA::Complete(outcome));
    for reply in AUTO_REPLIES {
        let before = *c;
        assert_eq!(c.reply(reply), Err(AE::Terminal));
        assert_eq!(*c, before);
    }
}
fn start_fallback(kind: Kind, shadow: Shadow, root: Root) -> C {
    let mut c = C::new(kind);
    step(&mut c, A::ReadShadowRoot, R::ShadowRoot(shadow));
    step(
        &mut c,
        A::RunAutofocusDelegate { root },
        R::AutofocusDelegatePresent(false),
    );
    step(&mut c, A::BeginDescendants { root }, R::Applied);
    c
}
fn start_auto(trigger: Trigger) -> AC {
    let mut c = AC::new(trigger);
    auto_step(&mut c, AA::BeginDescendants, AR::Applied);
    c
}
fn autofocus_descendant(c: &mut AC) {
    auto_step(c, AA::NextDescendant, AR::DescendantPresent(true));
    auto_step(c, AA::ReadAutofocus, AR::Autofocus(true));
}
#[test]
fn nondelegating_shadow_host_returns_null_without_autofocus_or_descendant_passes() {
    for kind in [Kind::Dialog, Kind::Other] {
        let mut c = C::new(kind);
        step(
            &mut c,
            A::ReadShadowRoot,
            R::ShadowRoot(Shadow::DoesNotDelegateFocus),
        );
        complete(&mut c, O::Null);
    }
}
#[test]
fn actual_root_selection_and_autofocus_result_precede_any_fallback_scan() {
    for (shadow, root) in [
        (Shadow::Absent, Root::Target),
        (Shadow::DelegatesFocus, Root::ShadowRoot),
    ] {
        let mut c = C::new(Kind::Dialog);
        step(&mut c, A::ReadShadowRoot, R::ShadowRoot(shadow));
        step(
            &mut c,
            A::RunAutofocusDelegate { root },
            R::AutofocusDelegatePresent(true),
        );
        complete(&mut c, O::AutofocusDelegate);
    }
}
#[test]
fn actual_empty_tree_runs_autofocus_exhaustion_then_distinct_fallback_exhaustion() {
    for (shadow, root) in [
        (Shadow::Absent, Root::Target),
        (Shadow::DelegatesFocus, Root::ShadowRoot),
    ] {
        let mut c = C::new(Kind::Dialog);
        step(&mut c, A::ReadShadowRoot, R::ShadowRoot(shadow));
        assert_eq!(c.action(), A::RunAutofocusDelegate { root });
        let mut auto = start_auto(Trigger::Other);
        auto_step(&mut auto, AA::NextDescendant, AR::DescendantPresent(false));
        auto_complete(&mut auto, AO::Null);
        step(
            &mut c,
            A::RunAutofocusDelegate { root },
            R::AutofocusDelegatePresent(false),
        );
        step(&mut c, A::BeginDescendants { root }, R::Applied);
        step(&mut c, A::NextDescendant, R::DescendantPresent(false));
        complete(&mut c, O::Null);
    }
}
#[test]
fn dialog_uses_sequential_focusability_and_other_target_uses_area_status() {
    let mut dialog = start_fallback(Kind::Dialog, Shadow::Absent, Root::Target);
    step(&mut dialog, A::NextDescendant, R::DescendantPresent(true));
    step(
        &mut dialog,
        A::ReadSequentiallyFocusable,
        R::SequentiallyFocusable(true),
    );
    complete(&mut dialog, O::Candidate(Candidate::Descendant));
    let mut other = start_fallback(Kind::Other, Shadow::DelegatesFocus, Root::ShadowRoot);
    step(&mut other, A::NextDescendant, R::DescendantPresent(true));
    step(&mut other, A::ReadFocusableArea, R::FocusableArea(true));
    complete(&mut other, O::Candidate(Candidate::Descendant));
}
#[test]
fn failed_primary_criterion_calls_resolution_directly_and_returns_actual_resolved_role() {
    for kind in [Kind::Dialog, Kind::Other] {
        let mut c = start_fallback(kind, Shadow::Absent, Root::Target);
        step(&mut c, A::NextDescendant, R::DescendantPresent(true));
        if kind == Kind::Dialog {
            step(
                &mut c,
                A::ReadSequentiallyFocusable,
                R::SequentiallyFocusable(false),
            );
        } else {
            step(&mut c, A::ReadFocusableArea, R::FocusableArea(false));
        }
        step(&mut c, A::GetFocusableArea, R::ResolvedAreaPresent(true));
        complete(&mut c, O::Candidate(Candidate::ResolvedArea));
    }
}
#[test]
fn null_resolution_advances_once_then_first_later_match_returns() {
    let mut c = start_fallback(Kind::Dialog, Shadow::Absent, Root::Target);
    step(&mut c, A::NextDescendant, R::DescendantPresent(true));
    step(
        &mut c,
        A::ReadSequentiallyFocusable,
        R::SequentiallyFocusable(false),
    );
    step(&mut c, A::GetFocusableArea, R::ResolvedAreaPresent(false));
    step(&mut c, A::NextDescendant, R::DescendantPresent(true));
    step(
        &mut c,
        A::ReadSequentiallyFocusable,
        R::SequentiallyFocusable(true),
    );
    complete(&mut c, O::Candidate(Candidate::Descendant));
}
#[test]
fn absent_autofocus_skips_all_area_queries_and_nullable_resolution_continues() {
    let mut c = start_auto(Trigger::Click);
    auto_step(&mut c, AA::NextDescendant, AR::DescendantPresent(true));
    auto_step(&mut c, AA::ReadAutofocus, AR::Autofocus(false));
    autofocus_descendant(&mut c);
    auto_step(&mut c, AA::ReadFocusableArea, AR::FocusableArea(false));
    auto_step(&mut c, AA::GetFocusableArea, AR::ResolvedAreaPresent(false));
    auto_step(&mut c, AA::NextDescendant, AR::DescendantPresent(false));
    auto_complete(&mut c, AO::Null);
}
#[test]
fn nonclick_autofocus_returns_direct_or_resolved_area_without_click_query() {
    for direct in [true, false] {
        let mut c = start_auto(Trigger::Other);
        autofocus_descendant(&mut c);
        auto_step(&mut c, AA::ReadFocusableArea, AR::FocusableArea(direct));
        let candidate = if direct {
            Candidate::Descendant
        } else {
            auto_step(&mut c, AA::GetFocusableArea, AR::ResolvedAreaPresent(true));
            Candidate::ResolvedArea
        };
        auto_complete(&mut c, AO::Candidate(candidate));
    }
}
#[test]
fn click_autofocus_checks_selected_area_and_rejection_does_not_end_search() {
    for direct in [true, false] {
        let mut c = start_auto(Trigger::Click);
        autofocus_descendant(&mut c);
        auto_step(&mut c, AA::ReadFocusableArea, AR::FocusableArea(direct));
        let candidate = if direct {
            Candidate::Descendant
        } else {
            auto_step(&mut c, AA::GetFocusableArea, AR::ResolvedAreaPresent(true));
            Candidate::ResolvedArea
        };
        auto_step(
            &mut c,
            AA::ReadClickFocusable { candidate },
            AR::ClickFocusable(false),
        );
        autofocus_descendant(&mut c);
        auto_step(&mut c, AA::ReadFocusableArea, AR::FocusableArea(true));
        auto_step(
            &mut c,
            AA::ReadClickFocusable {
                candidate: Candidate::Descendant,
            },
            AR::ClickFocusable(true),
        );
        auto_complete(&mut c, AO::Candidate(Candidate::Descendant));
    }
}
#[test]
fn click_autofocus_returns_resolved_area_when_that_area_is_click_focusable() {
    let mut c = start_auto(Trigger::Click);
    autofocus_descendant(&mut c);
    auto_step(&mut c, AA::ReadFocusableArea, AR::FocusableArea(false));
    auto_step(&mut c, AA::GetFocusableArea, AR::ResolvedAreaPresent(true));
    auto_step(
        &mut c,
        AA::ReadClickFocusable {
            candidate: Candidate::ResolvedArea,
        },
        AR::ClickFocusable(true),
    );
    auto_complete(&mut c, AO::Candidate(Candidate::ResolvedArea));
}
#[test]
fn called_resolution_wait_retains_exact_caller_until_authentic_nested_result() {
    let mut caller = start_fallback(Kind::Dialog, Shadow::Absent, Root::Target);
    step(&mut caller, A::NextDescendant, R::DescendantPresent(true));
    step(
        &mut caller,
        A::ReadSequentiallyFocusable,
        R::SequentiallyFocusable(false),
    );
    let waiting = caller;
    // A nested focus-delegate may be reached by called get-the-focusable-area.
    let mut nested = C::new(Kind::Other);
    step(
        &mut nested,
        A::ReadShadowRoot,
        R::ShadowRoot(Shadow::DelegatesFocus),
    );
    step(
        &mut nested,
        A::RunAutofocusDelegate {
            root: Root::ShadowRoot,
        },
        R::AutofocusDelegatePresent(true),
    );
    complete(&mut nested, O::AutofocusDelegate);
    assert_eq!(caller, waiting);
    // Neither nested dispatch nor a general Applied receipt is the called
    // resolution's actual nullable return.
    assert_eq!(caller.reply(R::Applied), Err(E::WrongReply));
    assert_eq!(caller, waiting);
    step(
        &mut caller,
        A::GetFocusableArea,
        R::ResolvedAreaPresent(true),
    );
    complete(&mut caller, O::Candidate(Candidate::ResolvedArea));
}
