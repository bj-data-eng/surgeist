use super::*;
use crate::testing::{assert_detected_replayed_and_reduced, key};
use surgeist_css::diagnostic::{CssErrorCode, CssEscapeFailure, CssRecoveryAction};
use surgeist_fuzz::{OutcomeKind, engine};

fn parser() -> CssParser {
    CssParser::default()
}

/// Handwritten facts from the crate reference; none is read back from a replay.
#[test]
fn literals_match_the_documented_recovery_facts() {
    let target = SourceTarget::new();
    let text = |id: u8| target.literal(id).unwrap().text;

    // Stylesheet task 1: the unknown at-rule is dropped, both style rules stay.
    let report = parser().parse_sheet(text(1));
    assert_eq!(report.syntax().rules().len(), 2);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );

    // A style attribute: importance is part of the declaration.
    let report = parser().parse_block_contents(text(2));
    assert!(report.is_clean());
    assert_eq!(report.syntax().len(), 2);

    // A leading U+FEFF is identifier content; alone it is an incomplete rule.
    assert!(parser().parse_sheet(text(3)).is_clean());
    assert!(!parser().parse_sheet(text(4)).is_clean());

    // `a {}` is consumed as a nested rule and dropped; the declarations stay.
    let report = parser().parse_block_contents(text(5));
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropQualifiedRule
    );

    // A root `}` ends the contents.
    assert_eq!(parser().parse_block_contents(text(6)).syntax().len(), 1);

    // An unterminated comment: one diagnostic at EOF, spanning from its opening.
    let source = text(7);
    let report = parser().parse_sheet(&source);
    assert_eq!(report.diagnostics().len(), 1);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::IgnoreUnterminatedComment
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.len()
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);

    // A string line continuation is valid; an escape at EOF is recovered.
    assert!(parser().parse_sheet(text(9)).is_clean());
    let report = parser().parse_sheet(text(10));
    let escape = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.error().escape_failure().is_some())
        .expect("an escape diagnostic");
    assert_eq!(
        escape.error().escape_failure(),
        Some(CssEscapeFailure::EndOfInput)
    );
    assert_eq!(escape.action(), CssRecoveryAction::RecoverEscape);

    // The later valid sibling survives a bad URL, which ends at its own `)`.
    assert_eq!(parser().parse_sheet(text(11)).syntax().rules().len(), 2);
    // An unclosed function consumes to EOF (Syntax 3 consume a function), so
    // no sibling follows it; the closure is reported, not silent.
    let report = parser().parse_sheet(text(22));
    assert_eq!(report.syntax().rules().len(), 1);
    assert!(!report.is_clean());
}

#[test]
fn every_literal_captures_replays_and_checks_every_front() {
    let target = SourceTarget::new();
    for id in 0..SourceTarget::LITERALS {
        let case = target.literal(id).unwrap();
        let mut captured = Vec::new();
        let observed = engine::evaluate_case(&target, &case, &mut |bytes| {
            captured = bytes.to_vec();
            Ok(())
        })
        .unwrap();
        assert_eq!(
            observed.outcome.kind(),
            OutcomeKind::Evaluated,
            "literal {id}: {:?}",
            observed.outcome.detail()
        );
        assert!(observed.outcome.checked_properties().len() >= 4);
        // A raw case is its exact source bytes.
        assert_eq!(captured, case.text.as_bytes());
        assert_eq!(target.decode(&captured).unwrap(), case);
        assert_eq!(
            engine::replay(&target, &captured, &mut |_| Ok(())).unwrap(),
            observed
        );
    }
    assert!(target.literal(SourceTarget::LITERALS).is_err());
}

#[test]
fn utf16_parity_runs_only_where_preprocessing_is_the_identity() {
    let target = SourceTarget::new();
    let checked = |text: &str| {
        target
            .evaluate(&SourceCase::new(text).unwrap())
            .unwrap()
            .checked_properties()
            .iter()
            .any(|property| property.as_str() == PARITY)
    };
    assert!(checked("a { b: c }"));
    assert!(checked("\u{feff}a { \u{1f600}: x }"));
    assert!(!checked("a {\r\n b: c }"));
    assert!(!checked("a { b: c\u{c} }"));
    assert!(!checked("a { b: c\0 }"));
}

#[test]
fn raw_cases_reject_malformed_utf8_and_oversize_input_without_conversion() {
    let target = SourceTarget::new();
    assert!(target.decode(b"a{}\xc0\x80").is_err());
    assert!(target.decode(b"\xed\xa0\x80").is_err());
    assert!(target.decode(&vec![b'a'; MAX_BYTES + 1]).is_err());
    assert!(SourceCase::new("x".repeat(MAX_BYTES + 1)).is_err());
    let exact = "\u{1f600}\0\r\n a{}";
    assert_eq!(target.decode(exact.as_bytes()).unwrap().text(), exact);
}

fn crlf_is_two_breaks(text: &str, offset: usize) -> (u32, u32) {
    // Planted: counts the CR and the LF of a CRLF as two line ends.
    let (line, column) = check::expected_coordinates(text, offset);
    (line + text[..offset].matches("\r\n").count() as u32, column)
}

const CRLF: Checker = Checker {
    coordinates: crlf_is_two_breaks,
};

#[test]
fn a_broken_coordinate_model_is_detected_replayed_and_reduced_through_diagnostics() {
    let broken = SourceTarget::with_checker(CRLF);
    let sound = SourceTarget::new();
    // The invalid declaration follows a CRLF, so its diagnostic sits on line 2.
    let case =
        SourceCase::new("a {\r\n  color: red;\r\n  width: ;\r\n}\r\n.b { color: blue }").unwrap();
    let reduced = assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &case,
        &key(check::COORDINATES, "line-column-disagree"),
    );
    // The raw reduction keeps valid UTF-8 and the CRLF the failure needs.
    let reduced = sound.decode(&reduced).unwrap();
    assert!(reduced.text().contains("\r\n"));
    assert!(reduced.text().len() < case.text().len());
}

#[test]
fn a_broken_coordinate_model_is_detected_through_retained_node_positions() {
    let broken = SourceTarget::with_checker(CRLF);
    let sound = SourceTarget::new();
    // A clean sheet has no diagnostics, so only retained provenance can disagree.
    let case = SourceCase::new("a {\r\n  color: red;\r\n}\r\n.b { color: blue }").unwrap();
    assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &case,
        &key(nodes::RETAINED, "line-column-disagree"),
    );
}

#[test]
fn generation_is_deterministic_bounded_and_reaches_recovery_and_retention() {
    let target = SourceTarget::new();
    let (mut retained, mut recovered, mut declarations) = (0, 0, 0);
    for seed in 0_u32..500 {
        let entropy: Vec<u8> = (0..160_u32)
            .map(|slot| (seed.wrapping_mul(2_654_435_761).wrapping_add(slot * 40_503) >> 9) as u8)
            .collect();
        let Generation::Generated(case) = target.generate(&entropy).unwrap() else {
            panic!("source generation never discards");
        };
        assert_eq!(
            target.generate(&entropy).unwrap(),
            Generation::Generated(case.clone())
        );
        assert!(case.text.len() <= MAX_BYTES);
        let report = parser().parse_sheet(&case.text);
        retained += usize::from(!report.syntax().rules().is_empty());
        recovered += usize::from(!report.is_clean());
        declarations += usize::from(
            !parser()
                .parse_block_contents(&case.text)
                .syntax()
                .is_empty(),
        );
        let observed = engine::evaluate_case(&target, &case, &mut |_| Ok(())).unwrap();
        assert_ne!(
            observed.outcome.kind(),
            OutcomeKind::Failure,
            "seed {seed}: {:?}",
            observed.outcome.detail()
        );
    }
    assert!(retained > 100 && recovered > 100 && declarations > 50);
}
