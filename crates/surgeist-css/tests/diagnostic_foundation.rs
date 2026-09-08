use surgeist_css::{
    CssParseReport, CssRecoveryAction, CssRecoveryDiagnostic, CssValidationFailure,
};

fn inspect_report(report: &CssParseReport<usize>) {
    let _: &usize = report.syntax();
    let _: &[CssRecoveryDiagnostic] = report.diagnostics();
    let _: bool = report.is_clean();
}

fn decompose_report(report: CssParseReport<usize>) {
    let _: (usize, Vec<CssRecoveryDiagnostic>) = report.into_parts();
}

fn inspect_diagnostic(diagnostic: &CssRecoveryDiagnostic) {
    let _ = diagnostic.error();
    let _ = diagnostic.span();
    let _ = diagnostic.action();
}

fn inspect_validation_failure(failure: CssValidationFailure) {
    let _ = failure.diagnostics();
    let _ = failure.first();
    let _: Vec<CssRecoveryDiagnostic> = failure.into_diagnostics();
}

fn classify_action(action: CssRecoveryAction) -> &'static str {
    match action {
        CssRecoveryAction::DropDeclaration => "drop declaration",
        CssRecoveryAction::DropDescriptor => "drop descriptor",
        CssRecoveryAction::DropQualifiedRule => "drop qualified rule",
        CssRecoveryAction::DropAtRule => "drop at-rule",
        CssRecoveryAction::DropKeyframeBlock => "drop keyframe block",
        CssRecoveryAction::DropSelectorListItem => "drop selector-list item",
        CssRecoveryAction::ReplaceMediaQueryWithNever => "replace media query",
        CssRecoveryAction::RetainWithImplicitClosure => "retain implicit closure",
        CssRecoveryAction::IgnoreLegacyToken => "ignore legacy token",
        CssRecoveryAction::StopAtNestingLimit => "stop at nesting limit",
        _ => "future action",
    }
}

#[test]
fn report_public_consumer_surface_is_available_from_crate_root() {
    let _inspect_report: fn(&CssParseReport<usize>) = inspect_report;
    let _decompose_report: fn(CssParseReport<usize>) = decompose_report;
    let _inspect_diagnostic: fn(&CssRecoveryDiagnostic) = inspect_diagnostic;
    let _inspect_validation_failure: fn(CssValidationFailure) = inspect_validation_failure;
}

#[test]
fn report_public_recovery_actions_support_wildcard_compatible_matching() {
    let actions = [
        CssRecoveryAction::DropDeclaration,
        CssRecoveryAction::DropDescriptor,
        CssRecoveryAction::DropQualifiedRule,
        CssRecoveryAction::DropAtRule,
        CssRecoveryAction::DropKeyframeBlock,
        CssRecoveryAction::DropSelectorListItem,
        CssRecoveryAction::ReplaceMediaQueryWithNever,
        CssRecoveryAction::RetainWithImplicitClosure,
        CssRecoveryAction::IgnoreLegacyToken,
        CssRecoveryAction::StopAtNestingLimit,
    ];

    assert_eq!(actions.map(classify_action)[0], "drop declaration");
    assert_eq!(actions.map(classify_action)[9], "stop at nesting limit");
}
