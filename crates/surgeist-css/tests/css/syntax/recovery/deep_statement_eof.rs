#![forbid(unsafe_code)]
//! Syntax 3 §5.4.2 returns an at-rule with an EOF fault. These cases compose
//! that contract with existing ordinary/scoped structural recovery boundaries.
use surgeist_css::{
    CssErrorCode, CssLayerNameList, CssParseReport, CssRecoveryAction, CssRule, CssScopedRule,
    CssSheet, CssSourcePosition, ErrorKind, parse_sheet, validate_sheet,
};

const DEPTHS: [usize; 6] = [63, 64, 65, 127, 128, 129];

fn specimen(depth: usize, scoped: bool, terminated: bool) -> (String, usize, usize) {
    // Every repeated opening contributes exactly one enclosing rule block.
    // The root sibling is outside all of them; comment braces are not blocks.
    let opening = if scoped { "@scope{" } else { "@media all{" };
    let mut source = format!("/*😀*/\r\n{}", opening.repeat(depth));
    let statement_start = source.len();
    source.push_str("@layer café");
    if terminated {
        source.push(';');
    }
    source.push_str(" /*🦀;{}*/ \t");
    let local_eof = source.len();
    source.push_str(&"}".repeat(depth));
    source.push_str(".after{color:red}");
    (source, statement_start, local_eof)
}

fn assert_original_position(position: CssSourcePosition, source: &str, offset: usize) {
    assert_eq!(position.byte_offset().value(), offset);
    assert_eq!(position.line().value(), 1);
    let (_, last_line) = source[..offset].rsplit_once("\r\n").unwrap();
    assert_eq!(
        position.column().value() as usize,
        last_line.encode_utf16().count()
    );
}

fn assert_layer(names: &CssLayerNameList) {
    let [name] = names.names() else {
        panic!("one retained named layer")
    };
    assert_eq!(name.components(), &["café".to_owned()]);
}

fn assert_ordinary_chain(sheet: &CssSheet, depth: usize, source: &str, start: usize) {
    let [CssRule::Media(first), CssRule::Style(_)] = sheet.rules() else {
        panic!("ordinary enclosing chain and later root style")
    };
    let mut parent = first;
    for _ in 1..depth {
        let [CssRule::Media(nested)] = parent.rules() else {
            panic!("every enclosing ordinary group survives")
        };
        parent = nested;
    }
    let [CssRule::LayerStatement(statement)] = parent.rules() else {
        panic!("named layer survives at the exact innermost ordinary depth")
    };
    assert_layer(statement.names());
    assert_original_position(statement.position(), source, start);
}

fn assert_scoped_chain(sheet: &CssSheet, depth: usize, source: &str, start: usize) {
    let [CssRule::Scope(first), CssRule::Style(_)] = sheet.rules() else {
        panic!("scope enclosing chain and later root style")
    };
    let mut parent = first;
    for _ in 1..depth {
        let [CssScopedRule::Scope(nested)] = parent.rules().rules() else {
            panic!("every enclosing scoped group survives")
        };
        parent = nested;
    }
    let [CssScopedRule::LayerStatement(statement)] = parent.rules().rules() else {
        panic!("named layer survives at the exact innermost scoped depth")
    };
    assert_layer(statement.names());
    assert_original_position(statement.position(), source, start);
}

fn assert_retained_tree(sheet: &CssSheet, depth: usize, scoped: bool, source: &str, start: usize) {
    if scoped {
        assert_scoped_chain(sheet, depth, source, start);
    } else {
        assert_ordinary_chain(sheet, depth, source, start);
    }
    let [_, CssRule::Style(after)] = sheet.rules() else {
        panic!("independent later root sibling")
    };
    assert_original_position(after.position(), source, source.find(".after").unwrap());
    let [declaration] = after.declarations().as_slice() else {
        panic!("later sibling retains its declaration")
    };
    assert_eq!(
        declaration.parsed_value().unwrap().source().as_str(),
        source
    );
}

fn assert_local_eof(report: &CssParseReport<CssSheet>, source: &str, start: usize, end: usize) {
    assert!(
        end < source.len(),
        "parent EOF precedes enclosing closes and sibling"
    );
    assert_eq!(&source[end..end + 1], "}", "actual explicit parent bound");
    let [diagnostic] = report.diagnostics() else {
        panic!("only one retained statement termination fault")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainNonconformingRule
    );
    let ErrorKind::UnexpectedEnd(detail) = diagnostic.error().kind() else {
        panic!("statement EOF diagnostic")
    };
    assert_eq!(
        detail.expectation().as_str(),
        "a semicolon or block terminating an at-rule"
    );
    assert_original_position(diagnostic.error().position(), source, end);
    assert_original_position(diagnostic.span().start(), source, start);
    assert_original_position(diagnostic.span().end(), source, end);
    assert!(!report.is_clean());
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert_eq!(
        report
            .clone()
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

fn check_depths(scoped: bool) {
    let cases: Vec<_> = DEPTHS
        .into_iter()
        .map(|depth| {
            let (source, start, end) = specimen(depth, scoped, false);
            let (terminated, control_start, _) = specimen(depth, scoped, true);
            let report = parse_sheet(&source);
            let control = parse_sheet(&terminated);
            (
                depth,
                source,
                start,
                end,
                terminated,
                control_start,
                report,
                control,
            )
        })
        .collect();
    // Invoke all twelve public specimens before any missing-EOF assertion.
    for (depth, source, start, end, terminated, control_start, report, control) in cases {
        assert_retained_tree(report.syntax(), depth, scoped, &source, start);
        assert_retained_tree(control.syntax(), depth, scoped, &terminated, control_start);
        assert!(
            control.is_clean(),
            "terminated control at depth {depth}: {:?}",
            control.diagnostics()
        );
        assert_eq!(validate_sheet(&terminated).unwrap(), *control.syntax());
        assert_local_eof(&report, &source, start, end);
    }
}

#[test]
fn ordinary_group_depth_transitions_retain_layer_with_exact_parent_local_eof() {
    check_depths(false);
}

#[test]
fn scoped_group_depth_transitions_retain_layer_with_exact_parent_local_eof() {
    check_depths(true);
}
