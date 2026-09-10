//! Retention diagnostics belong to the accepted grammar unit that opened them.
//! CSS Syntax closes blocks at EOF; Surgeist only reports retained closures.

use surgeist_css::{CssErrorCode, CssRecoveryAction, CssRule, parse_sheet, validate_sheet};

fn assert_actions(source: &str, discarded: usize, retained_closures: usize) {
    let report = parse_sheet(source);
    assert!(
        !report.syntax().rules().is_empty(),
        "retained rules must survive: {source}"
    );
    let drops = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::DropFontSourceListItem)
        .count();
    assert_eq!(drops, discarded, "{source}: {:?}", report.diagnostics());
    let closures = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure)
        .collect::<Vec<_>>();
    assert_eq!(
        closures.len(),
        retained_closures,
        "only retained grammar units close at EOF: {source}: {:?}",
        report.diagnostics()
    );
    assert_eq!(report.diagnostics().len(), discarded + retained_closures);
    for diagnostic in closures {
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            source.len()
        );
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn completed_declarations_cannot_retain_later_discarded_components() {
    for source in [
        "@font-face{font-family:Demo;src:url(valid),local(",
        "@font-face{src:url(previous);src:url(valid),local(",
        ".before{color:red}@font-face{src:url(valid),local(",
        ".before{--value:fn(x)}@font-face{src:url(valid),dummy(nested(",
    ] {
        let report = parse_sheet(source);
        let Some(CssRule::FontFace(face)) = report.syntax().rules().last() else {
            panic!("expected the retained font face: {source}");
        };
        assert_eq!(face.descriptors().src().unwrap().sources().len(), 1);
        assert_actions(source, 1, 1);
    }
}

#[test]
fn completed_preludes_cannot_retain_later_discarded_components() {
    for (source, closures) in [
        (".before{}@font-face{src:url(valid),local(", 1),
        ("@media screen{}@font-face{src:url(valid),local(", 1),
        (
            "@supports (display:grid){}@font-face{src:url(valid),local(",
            1,
        ),
        ("@media screen{@font-face{src:url(valid),local(", 2),
        (
            "@supports (display:grid){@font-face{src:url(valid),local(",
            2,
        ),
    ] {
        assert_actions(source, 1, closures);
    }
}

#[test]
fn preceding_units_do_not_erase_retained_component_closures() {
    for (source, discarded, closures) in [
        (
            "@font-face{font-family:Demo;src:url(valid),local(Last",
            0,
            2,
        ),
        (
            ".before{color:red}@font-face{src:url(valid),local(Last",
            0,
            2,
        ),
        ("@media screen{@font-face{src:dummy(),local(Last", 1, 3),
        ("@supports (display:grid){.x{--value:f(g([x", 0, 5),
    ] {
        assert_actions(source, discarded, closures);
    }
}
