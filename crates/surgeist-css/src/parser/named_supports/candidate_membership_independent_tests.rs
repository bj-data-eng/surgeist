#![forbid(unsafe_code)]
use super::*;
use crate::syntax_consumption::{
    CheckedLexeme, SyntaxInput, SyntaxInputLimits, SyntaxToken, SyntaxTokenFault, TokenPayload,
    normalize, source_document,
};
use crate::{CssComponentValueErrorKind, CssComponentValueRef, CssValueTokenRef};
use std::borrow::Cow;

fn checked(component: &CssComponentValue) -> SyntaxToken<'_> {
    let spelling = component.structural_lexeme(false).unwrap().0;
    SyntaxToken {
        payload: TokenPayload::Checked {
            component,
            lexeme: CheckedLexeme::Leaf,
        },
        spelling: Some(Cow::Borrowed(spelling)),
        origin: Cow::Borrowed(component.origin()),
    }
}

fn identity(actual: &CssValueOrigin, expected: &CssValueOrigin) {
    assert_eq!(actual, expected);
    if let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) = (actual, expected) {
        assert!(a.source().same_snapshot(b.source()));
    }
}

#[test]
fn actual_named_partition_keeps_error_membership_when_origins_are_nonmonotonic_or_programmatic() {
    let head = crate::parse_component_values("head:yes;bad_a:").unwrap();
    let good = crate::parse_component_values(";good:yes;bad_b:").unwrap();
    let bad_c = crate::parse_component_values(";bad_c:").unwrap();
    let last = crate::parse_component_values(";last:yes;").unwrap();
    let late_text = format!("{}\"broken\n", " ".repeat(32));
    let late_snapshot = CssSourceSnapshot::new(&late_text);
    let late_document = source_document(&late_text, &late_snapshot, 0).unwrap();
    let late = late_document
        .nodes
        .iter()
        .find_map(|node| match node {
            SyntaxNode::Error {
                token,
                cause: SyntaxTokenFault::BadString,
            } => Some(token.clone()),
            _ => None,
        })
        .unwrap();
    let early_snapshot = CssSourceSnapshot::new("url(a b)");
    let early_document = source_document(early_snapshot.as_str(), &early_snapshot, 0).unwrap();
    let early = early_document
        .nodes
        .iter()
        .find_map(|node| match node {
            SyntaxNode::Error {
                token,
                cause: SyntaxTokenFault::BadUrl,
            } => Some(token.clone()),
            _ => None,
        })
        .unwrap();
    let late_origin = late.origin.clone().into_owned();
    let early_origin = early.origin.clone().into_owned();
    let CssValueOrigin::Parsed(late_site) = &late_origin else {
        panic!("actual later source token")
    };
    let CssValueOrigin::Parsed(early_site) = &early_origin else {
        panic!("actual earlier source token")
    };
    assert_eq!(late_site.span().start().byte_offset().value(), 32);
    assert_eq!(early_site.span().start().byte_offset().value(), 0);
    assert!(!late_site.source().same_snapshot(early_site.source()));
    let mut tokens: Vec<_> = head.items().iter().map(checked).collect();
    tokens.push(late);
    tokens.extend(good.items().iter().map(checked));
    tokens.push(early);
    tokens.extend(bad_c.items().iter().map(checked));
    tokens.push(SyntaxToken {
        payload: TokenPayload::Native(Token::BadString("programmatic payload".into())),
        spelling: None,
        origin: Cow::Owned(CssValueOrigin::Programmatic),
    });
    tokens.extend(last.items().iter().map(checked));
    let document = normalize(
        SyntaxInput::Tokens(&tokens),
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    let mut issues = Vec::new();
    let body = partition(&document, document.root, None, true, &mut issues).unwrap();
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
        (CssComponentValueErrorKind::BadString, &late_origin),
        (CssComponentValueErrorKind::BadUrl, &early_origin),
        (
            CssComponentValueErrorKind::BadString,
            &CssValueOrigin::Programmatic,
        ),
    ]) {
        let PartitionIssue::Lexical(error, action) = issue else {
            panic!("actual lexical member fault")
        };
        assert_eq!(error.kind(), kind);
        assert_eq!(*action, CssRecoveryAction::DropDeclaration);
        identity(error.origin(), origin);
    }
    identity(body.recovery_origin().unwrap(), &late_origin);
    assert!(
        document
            .boundary(document.root, document.lists[document.root].len())
            .source
            .is_none(),
        "mixed passed items have conceptual EOF without invented aggregate source"
    );
}
