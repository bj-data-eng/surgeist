//! Actual group/terminal closure and rejected-use custody through the fitter.
use super::*;
use crate::test_support::text::{PreparedParagraph, ahem_style};
use crate::text_layout::fit::balance::completion_tests::{
    fit_with_allocation, fit_with_terminal_context,
};
use surgeist_text as text;

fn reset() {
    CLOSURES.with_borrow_mut(Vec::clear);
    GROUPS.with_borrow_mut(Vec::clear);
}

fn after_paragraph(source: &str) -> PreparedParagraph {
    PreparedParagraph::plain(
        1,
        text::Source::new(source),
        text::Style {
            wrap: text::Wrap::Word,
            white_space: text::WhiteSpace::PreserveBreaks,
            hyphens: text::Hyphens::Manual,
            locale: Some("und-Latn".into()),
            hyphenate_character: text::HyphenateCharacter::Auto,
            native_hyphenation_convention: Some(
                text::NativeHyphenationConvention::language_script(
                    "und-Latn",
                    "Latn",
                    "(".into(),
                    text::HyphenationPlacement::LogicalAfter,
                )
                .unwrap(),
            ),
            // A real After '(' consumes First=false on a receiving line. It does
            // not hang, so all declared Ahem advances below remain unchanged.
            hanging_punctuation: text::HangingPunctuation::new(true, None, false),
            ..ahem_style(10.0, 10.0)
        },
    )
}

fn facts(lines: &[FittedLine]) -> Vec<(usize, f64, LineEnd)> {
    lines
        .iter()
        .map(|line| {
            let selected = line.finalized.selected_line();
            (
                selected.request().end.view().source_after().get(),
                line.natural_advance,
                selected.request().context.end,
            )
        })
        .collect()
}

fn assert_rejected_receiver_retained(paragraph: &PreparedParagraph, lines: &[FittedLine]) {
    assert!(
        lines
            .iter()
            .all(|line| line.finalized.selected_line().request().incoming.is_none())
    );
    let uses = &lines.last().unwrap().role_uses;
    let used = uses
        .iter()
        .find(|used| used.request.incoming.is_some() && used.usage.first())
        .expect("actual rejected After receiver's consumed First fact must survive backtracking");
    assert_eq!(used.request.start.prepared_text(), paragraph.prepared);
    let incoming = used.request.incoming.as_ref().unwrap();
    assert_eq!(incoming.boundary(), used.request.start);
    assert!(
        !lines
            .iter()
            .any(|line| line.finalized.selected_line().request() == &used.request)
    );
}

#[test]
fn successful_pretty_keeps_actual_after_uses_discarded_by_internal_always_backtracking() {
    reset();
    let paragraph = after_paragraph("XXXX as\u{ad}sociate");
    let lines = fit_with_terminal_context(
        &paragraph,
        crate::TextWrapStyle::Pretty,
        100.0,
        TailContext::CompleteBlock,
    );
    assert_eq!(
        facts(&lines),
        [(5, 40.0, LineEnd::Soft), (15, 90.0, LineEnd::Paragraph)]
    );
    assert_rejected_receiver_retained(&paragraph, &lines);
    CLOSURES.with_borrow(|closures| assert!(closures.is_empty()));
    let Some(SelectionProgram::Pretty(basis)) = lines[0].program.as_deref() else {
        panic!("real Pretty program")
    };
    assert!(matches!(basis.disposition, PrettyDisposition::Stream(_)));
}

#[test]
fn terminal_closure_discards_improved_groups_once_but_keeps_dependencies_and_charged_work() {
    reset();
    let paragraph = after_paragraph("AAAAA BBBB C CCCCCCCCC\nXXXX as\u{ad}sociate\n");
    let mut before = 0;
    let mut after = 0;
    let lines = fit_with_allocation(
        &paragraph,
        crate::TextWrapStyle::Pretty,
        100.0,
        TailContext::CompleteBlock,
        |_, _, _, struts| {
            if CLOSURES.with_borrow(|closures| closures.is_empty()) {
                before += 1;
            } else {
                after += 1;
            }
            Ok(LineAllocations {
                line_box_struts: struts.to_vec(),
                atomics: vec![],
                controls: vec![],
                line_strut: None,
            })
        },
    )
    .unwrap_or_else(|_| panic!("ordinary terminal closure must succeed"));
    // First Forced group really improved, then the later mandatory closure
    // invalidated the entire unpublished optional program.
    GROUPS.with_borrow(|groups| assert_eq!(groups[0], [(6, 50.0), (13, 60.0), (23, 90.0)]));
    assert_eq!(
        facts(&lines),
        [
            (11, 100.0, LineEnd::Soft),
            (13, 10.0, LineEnd::Soft),
            (23, 90.0, LineEnd::Forced),
            (28, 40.0, LineEnd::Soft),
            (39, 90.0, LineEnd::Forced),
            (39, 0.0, LineEnd::Paragraph),
        ]
    );
    CLOSURES
        .with_borrow(|closures| assert_eq!(&**closures, &[ScopeExhaustion::TerminalConstraint]));
    assert!(
        before > 0 && after > 0,
        "both provisional and ordinary callback work is consumed"
    );
    assert!(matches!(
        lines.last().unwrap().disposition,
        LineDisposition::NonFormatted(_)
    ));
    assert_rejected_receiver_retained(&paragraph, &lines);
    let Some(SelectionProgram::Pretty(basis)) = lines[0].program.as_deref() else {
        panic!("real Pretty fallback program")
    };
    assert!(matches!(
        basis.disposition,
        PrettyDisposition::OrdinaryClosure(ScopeExhaustion::TerminalConstraint)
    ));
}

#[test]
fn actual_provider_and_feedback_failures_never_become_ordinary_success() {
    for feedback in [false, true] {
        reset();
        let paragraph = after_paragraph("AAA BB C");
        let mut calls = 0;
        let result = fit_with_allocation(
            &paragraph,
            crate::TextWrapStyle::Pretty,
            60.0,
            TailContext::Unrestricted,
            |_, _, _, _| {
                calls += 1;
                if feedback {
                    Err(FitError::FeedbackCycle)
                } else {
                    Err(FitError::Dependency)
                }
            },
        );
        if feedback {
            assert!(matches!(result, Err(FitError::FeedbackCycle)));
        } else {
            assert!(matches!(result, Err(FitError::Dependency)));
        }
        assert!(calls > 0);
        CLOSURES.with_borrow(|closures| assert!(closures.is_empty()));
    }
}

#[test]
fn the_actual_ordinary_fallback_failure_preserves_its_owner_error() {
    reset();
    let paragraph = after_paragraph("XXXX as\u{ad}sociate\n");
    let mut ordinary_calls = 0;
    let result = fit_with_allocation(
        &paragraph,
        crate::TextWrapStyle::Pretty,
        100.0,
        TailContext::CompleteBlock,
        |_, _, _, struts| {
            if CLOSURES.with_borrow(|closures| !closures.is_empty()) {
                ordinary_calls += 1;
                return Err(FitError::Dependency);
            }
            Ok(LineAllocations {
                line_box_struts: struts.to_vec(),
                atomics: vec![],
                controls: vec![],
                line_strut: None,
            })
        },
    );
    assert!(matches!(result, Err(FitError::Dependency)));
    assert_eq!(ordinary_calls, 1);
    CLOSURES.with_borrow(|closures| assert_eq!(closures.len(), 1));
}

#[test]
fn pretty_ordinary_closure_replays_the_real_float_journal_and_agrees_with_a_fresh_run() {
    use crate::test_support::layout_tree::PublicLayoutTreeOf;

    fn lane<S: crate::LayoutScalar>() {
        let raw = "AAAAA BBBB C CCCCCCCCC\nXXXX as\u{ad}sociate\n";
        let paragraph = after_paragraph(raw);
        // Native floats are genuine transparent run nodes, not Text atomic
        // occurrences. Preserve the actual origin map and native encounter order.
        let mut binding = paragraph.binding(0);
        binding.nodes = vec![2, 1];
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [2, 1])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                crate::NodeInputOf {
                    display: crate::Display::Block,
                    text_wrap_style: crate::TextWrapStyle::Pretty,
                    hyphenate_limit_last: crate::HyphenateLimitLast::Always,
                    size: crate::Size::new(
                        crate::PreferredSizeOf::px(S::from_f64(120.0)),
                        crate::PreferredSizeOf::AUTO,
                    ),
                    ..crate::NodeInputOf::default()
                },
            )
            .input(1, paragraph.text_input(1))
            .style(
                2,
                crate::NodeInputOf {
                    display: crate::Display::Block,
                    float: crate::Float::Left,
                    size: crate::Size::new(
                        crate::PreferredSizeOf::px(S::from_f64(20.0)),
                        crate::PreferredSizeOf::px(S::from_f64(100.0)),
                    ),
                    ..crate::NodeInputOf::default()
                },
            );
        let flow = crate::LayoutFlowInput::try_new(
            &tree,
            0,
            vec![paragraph.binding(0)],
            crate::WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let compute = || {
            crate::compute_layout_with_flow(
                &flow,
                0,
                crate::LayoutRootRequestOf::viewport(crate::Size::new(
                    crate::AvailableOf::Definite(S::from_f64(120.0)),
                    crate::AvailableOf::MAX_CONTENT,
                ))
                .unwrap(),
            )
            .unwrap()
        };
        let first = compute();
        let fresh = compute();
        assert_eq!(first.final_entries(), fresh.final_entries());
        for batch in [&first, &fresh] {
            let floats = batch
                .final_entries()
                .iter()
                .filter(|entry| entry.node() == 2)
                .collect::<Vec<_>>();
            assert_eq!(floats.len(), 1);
            assert_eq!(floats[0].output().location, crate::Point::ZERO);
            assert_eq!(
                floats[0].output().size,
                crate::Size::new(S::from_f64(20.0), S::from_f64(100.0))
            );
            let placed = batch.placed_text()[0].layout();
            assert_eq!(
                placed.source().text().code_units(),
                raw.encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(placed.lines().len(), 5);
            let mut facts = vec![None; 5];
            for entry in batch.final_inline_fragments() {
                let fragment = entry.fragment();
                let line = fragment.finalized_fragment().finalized_line();
                let selected = line.selected_line();
                assert_eq!(selected.request().context.available_inline, Some(100.0));
                facts[fragment.line_index()] = Some((
                    selected.request().end.view().source_after().get(),
                    line.measurements().alignment().advance(),
                ));
            }
            assert_eq!(
                facts,
                [
                    Some((11, 100.0)),
                    Some((13, 10.0)),
                    Some((23, 90.0)),
                    Some((28, 40.0)),
                    Some((39, 90.0))
                ]
            );
            let nonformatted = placed.nonformatted_lines().collect::<Vec<_>>();
            assert_eq!(nonformatted.len(), 1);
            let terminal = nonformatted[0].line.finalized_line();
            assert_eq!(
                terminal.selected_line().request().context.end,
                LineEnd::Paragraph
            );
            assert_eq!(
                terminal.selected_line().request().context.available_inline,
                Some(100.0)
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}
