//! Independent setter traces from HTML877d raw61117–61130 and the selected
//! #1243 KnownNull correction. These are pure language tests, not native Range,
//! Source publication, Selection binding or positive focus integration evidence.
use surgeist_html::{
    HtmlDesignModeAction as A, HtmlDesignModeCursor as C, HtmlDesignModeError as E,
    HtmlDesignModeInput as Input, HtmlDesignModeReply as R, HtmlFocusingAction as FA,
    HtmlFocusingCursor, HtmlFocusingFallback, HtmlFocusingReply as FR,
};

const REPLIES: [R; 8] = [
    R::Enabled(false),
    R::Enabled(true),
    R::NoActiveRange,
    R::ActiveRange,
    R::DocumentElementPresent(false),
    R::DocumentElementPresent(true),
    R::FocusingStepsCompleted,
    R::Applied,
];
fn check_rejected_replies(c: &mut C) {
    for reply in REPLIES {
        let allowed = matches!(
            (c.action(), reply),
            (A::ReadEnabled, R::Enabled(_))
                | (A::SetEnabled { .. } | A::ResetActiveRange, R::Applied)
                | (A::ReadActiveRange, R::NoActiveRange | R::ActiveRange)
                | (A::ReadDocumentElement, R::DocumentElementPresent(_))
                | (A::RunFocusingSteps, R::FocusingStepsCompleted)
        );
        if !allowed {
            let before = *c;
            assert_eq!(
                c.reply(reply),
                Err(if c.action() == A::Complete {
                    E::Terminal
                } else {
                    E::WrongReply
                })
            );
            assert_eq!(*c, before);
        }
    }
}
fn step(c: &mut C, expected: A, reply: R) {
    assert_eq!(c.action(), expected);
    check_rejected_replies(c);
    c.reply(reply).unwrap();
}
fn complete(c: &mut C) {
    assert_eq!(c.action(), A::Complete);
    check_rejected_replies(c);
}
fn enabled_on() -> C {
    let mut c = C::new(Input::On);
    step(&mut c, A::ReadEnabled, R::Enabled(false));
    step(&mut c, A::SetEnabled { enabled: true }, R::Applied);
    c
}

#[test]
fn exact_ascii_keywords_accept_every_case_combination_without_trimming() {
    for first in [0x6f, 0x4f] {
        for second in [0x6e, 0x4e] {
            let units = [first, second];
            assert_eq!(Input::from_utf16(&units), Input::On);
        }
        for second in [0x66, 0x46] {
            for third in [0x66, 0x46] {
                let units = [first, second, third];
                assert_eq!(Input::from_utf16(&units), Input::Off);
            }
        }
    }
    for text in [
        "", "o", "of", "onn", "offf", " on", "on ", " off", "off ", "on\n", "\ton", "true",
        "false", "ON\0", "oſ", "ｏｎ",
    ] {
        assert_eq!(
            Input::from_utf16(&text.encode_utf16().collect::<Vec<_>>()),
            Input::Invalid,
            "{text:?}"
        );
    }
    for units in [[0xd800, 0x6e], [0x6f, 0xdfff], [0x00d6, 0x6e]] {
        assert_eq!(Input::from_utf16(&units), Input::Invalid);
    }
    assert_eq!(Input::from_utf16(&[0x6f, 0x66, 0xd800]), Input::Invalid);
    assert_eq!(Input::from_utf16(&vec![0xd800; 100_000]), Input::Invalid);
}

#[test]
fn invalid_and_already_enabled_inputs_do_not_reset_or_focus() {
    let mut invalid = C::new(Input::Invalid);
    complete(&mut invalid);
    let mut already_on = C::new(Input::On);
    step(&mut already_on, A::ReadEnabled, R::Enabled(true));
    complete(&mut already_on);
}

#[test]
fn off_unconditionally_writes_false_without_reading_selection_or_focusing() {
    let mut c = C::new(Input::Off);
    step(&mut c, A::SetEnabled { enabled: false }, R::Applied);
    complete(&mut c);
}

#[test]
fn null_association_skips_reset_but_still_reads_the_current_document_element() {
    let mut c = enabled_on();
    step(&mut c, A::ReadActiveRange, R::NoActiveRange);
    step(
        &mut c,
        A::ReadDocumentElement,
        R::DocumentElementPresent(false),
    );
    complete(&mut c);
}

#[test]
fn actual_association_requires_reset_before_reading_document_element() {
    let mut c = enabled_on();
    step(&mut c, A::ReadActiveRange, R::ActiveRange);
    step(&mut c, A::ResetActiveRange, R::Applied);
    step(
        &mut c,
        A::ReadDocumentElement,
        R::DocumentElementPresent(false),
    );
    complete(&mut c);
}

fn real_focusing_child(outer: &C) {
    // A genuine focusing language invocation, omitted fallback and trigger. Its
    // actual target resolves to a viewport already holding focus. No native
    // focusability or current-area service is asserted by these pure replies.
    let mut child = HtmlFocusingCursor::new(HtmlFocusingFallback::Omitted);
    for (action, reply) in [
        (
            FA::ReadInitialTargetFocusableArea,
            FR::InitialTargetFocusableArea(false),
        ),
        (FA::RunGetFocusableArea, FR::ResolvedTargetPresent(true)),
        (
            FA::ReadTargetIsNavigableContainer,
            FR::TargetIsNavigableContainer(false),
        ),
        (
            FA::ReadReplacementTargetFocusableArea,
            FR::ReplacementTargetFocusableArea(true),
        ),
        (FA::ReadActualAnchorInert, FR::ActualAnchorInert(false)),
        (FA::RunCurrentlyFocusedArea, FR::CurrentlyFocusedAreaReady),
        (FA::ReadSameTarget, FR::SameTarget(true)),
    ] {
        assert_eq!(outer.action(), A::RunFocusingSteps);
        assert_eq!(child.action(), action);
        child.reply(reply).unwrap();
    }
    assert_eq!(child.action(), FA::Complete);
}

#[test]
fn both_association_branches_with_a_root_require_a_real_focusing_child() {
    for association in [R::NoActiveRange, R::ActiveRange] {
        let mut c = enabled_on();
        step(&mut c, A::ReadActiveRange, association);
        if association == R::ActiveRange {
            step(&mut c, A::ResetActiveRange, R::Applied);
        }
        step(
            &mut c,
            A::ReadDocumentElement,
            R::DocumentElementPresent(true),
        );
        check_rejected_replies(&mut c);
        real_focusing_child(&c);
        step(&mut c, A::RunFocusingSteps, R::FocusingStepsCompleted);
        complete(&mut c);
    }
}

#[test]
fn missing_facts_leave_the_exact_reached_action_retained() {
    let mut c = enabled_on();
    let before = c;
    // An unavailable association has no successful reply. Inspection and wrong
    // reset/child receipts cannot advance this operation to the no-range branch.
    for _ in 0..3 {
        assert_eq!(c.action(), A::ReadActiveRange);
        assert_eq!(c.reply(R::Applied), Err(E::WrongReply));
        assert_eq!(c, before);
    }
    step(&mut c, A::ReadActiveRange, R::ActiveRange);
    let retained_reset = c;
    check_rejected_replies(&mut c);
    assert_eq!(c, retained_reset);
    step(&mut c, A::ResetActiveRange, R::Applied);
    step(
        &mut c,
        A::ReadDocumentElement,
        R::DocumentElementPresent(true),
    );
    let retained_focus = c;
    check_rejected_replies(&mut c);
    assert_eq!(c, retained_focus);
    real_focusing_child(&c);
    step(&mut c, A::RunFocusingSteps, R::FocusingStepsCompleted);
    complete(&mut c);
}

#[test]
fn a_nested_off_invocation_does_not_make_outer_completion_write_enabled_again() {
    let mut outer = enabled_on();
    step(&mut outer, A::ReadActiveRange, R::NoActiveRange);
    step(
        &mut outer,
        A::ReadDocumentElement,
        R::DocumentElementPresent(true),
    );
    let mut nested = C::new(Input::Off);
    step(&mut nested, A::SetEnabled { enabled: false }, R::Applied);
    complete(&mut nested);
    real_focusing_child(&outer);
    step(&mut outer, A::RunFocusingSteps, R::FocusingStepsCompleted);
    complete(&mut outer);
}
