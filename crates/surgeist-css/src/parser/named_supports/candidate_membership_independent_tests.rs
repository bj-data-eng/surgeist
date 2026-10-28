#![forbid(unsafe_code)]
use crate::component::CssValueOrigin;
use crate::component::values::CssComponentValueErrorKind;
use crate::component::{CssComponentValueRef, CssValueTokenRef};
use crate::diagnostic::CssRecoveryAction;
use crate::parser::named_supports::{PartitionIssue, partition};
use crate::query::supports::CssSupportsTestItem;
use crate::source::{CssParsedOrigin, CssSourceSnapshot};
use crate::syntax_consumption::DocumentExt;
use crate::syntax_consumption::Supplied::{Checked, FaultAt};
use crate::syntax_consumption::supplied_document;
use surgeist_css_parser::Token;

fn identity(actual: &CssValueOrigin, expected: &CssValueOrigin) {
    assert_eq!(actual, expected);
    if let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) = (actual, expected) {
        assert!(a.source().same_snapshot(b.source()));
    }
}

#[test]
fn actual_named_partition_keeps_error_membership_when_origins_are_nonmonotonic_or_programmatic() {
    let head = crate::component::values::parse_component_values("head:yes;bad_a:").unwrap();
    let good = crate::component::values::parse_component_values(";good:yes;bad_b:").unwrap();
    let bad_c = crate::component::values::parse_component_values(";bad_c:").unwrap();
    let last = crate::component::values::parse_component_values(";last:yes;").unwrap();
    // The faults sit at intervals of one text, out of order, so each issue's
    // origin tells which fault it is: the list's first fault is the last in the
    // text, and the recovery origin must be that one and not the earliest.
    let text = CssSourceSnapshot::new(&"0123456789".repeat(6));
    let fault_ranges = [41..44, 3..5, 20..22];
    let mut tokens: Vec<_> = head.items().iter().map(Checked).collect();
    tokens.push(FaultAt(Token::BadString, fault_ranges[0].clone()));
    tokens.extend(good.items().iter().map(Checked));
    tokens.push(FaultAt(Token::BadUrl, fault_ranges[1].clone()));
    tokens.extend(bad_c.items().iter().map(Checked));
    tokens.push(FaultAt(Token::BadString, fault_ranges[2].clone()));
    tokens.extend(last.items().iter().map(Checked));
    let document = supplied_document(&tokens, Some(&text));
    let fault_origins: Vec<_> = fault_ranges
        .iter()
        .map(|range| {
            CssValueOrigin::Parsed(CssParsedOrigin::from_range(&text, range.clone()).unwrap())
        })
        .collect();
    let mut issues = Vec::new();
    let body = partition(&document, document.root_list(), None, true, &mut issues).unwrap();
    let declarations: Vec<_> = body
        .items()
        .iter()
        .flat_map(|item| match item {
            CssSupportsTestItem::Declarations(run) => run.declarations().iter().collect::<Vec<_>>(),
            _ => panic!("bad declarations cannot become generic rules"),
        })
        .collect();
    assert_eq!(
        declarations
            .iter()
            .map(|declaration| declaration.property())
            .collect::<Vec<_>>(),
        ["head", "good", "last"]
    );
    for (declaration, source) in declarations.iter().zip([&head, &good, &last]) {
        let [value] = declaration.value_components() else {
            panic!("original one-value declaration")
        };
        assert!(matches!(
            value.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Ident("yes"))
        ));
        let original = source
            .items()
            .iter()
            .find(|value| {
                matches!(
                    value.view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Ident("yes"))
                )
            })
            .unwrap();
        identity(value.origin(), original.origin());
    }
    assert_eq!(issues.len(), 3);
    for (issue, (kind, origin)) in issues.iter().zip([
        (CssComponentValueErrorKind::BadString, &fault_origins[0]),
        (CssComponentValueErrorKind::BadUrl, &fault_origins[1]),
        (CssComponentValueErrorKind::BadString, &fault_origins[2]),
    ]) {
        let PartitionIssue::Lexical(error, action) = issue else {
            panic!("actual lexical member fault")
        };
        assert_eq!(error.kind(), kind);
        assert_eq!(*action, CssRecoveryAction::DropDeclaration);
        identity(error.origin(), origin);
    }
    identity(body.recovery_origin().unwrap(), &fault_origins[0]);
    assert_eq!(
        document
            .boundary(
                document.root_list(),
                document.list(document.root_list()).len()
            )
            .offset,
        Some(text.as_str().len()),
        "the mixed list ends where its text ends, not at an item's interval"
    );
}
