#![forbid(unsafe_code)]

use surgeist_css::{
    CssComponentValue, CssComponentValues, CssRule, CssSpecifiedRuleSerializationErrorKind,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits,
    CssSupportsTestBody, CssSupportsTestItem, CssValueTokenRef, parse_sheet,
};

fn definition(source: &str) -> String {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
        panic!("expected one named supports definition: {source}");
    };
    rule.to_specified_css().unwrap()
}

#[test]
fn canonical_body_keeps_earlier_bangs_and_distinguishes_at_rule_forms() {
    let source = "@supports-condition --feature { future:a !b; future:a !important !important; @probe x; @probe x {} & { future: 1/**/e3; } }";
    let expected = "@supports-condition --feature { future: a !b; future: a !important !important; @probe x; @probe x { } & { future: 1/**/e3; } }";
    assert_eq!(definition(source), expected);
    assert_eq!(definition(expected), expected);

    let report = parse_sheet(expected);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
        panic!("expected canonical definition");
    };
    let items = rule.body().items();
    let CssSupportsTestItem::Declarations(run) = &items[0] else {
        panic!("expected leading declaration run");
    };
    assert_eq!(run.declarations().len(), 2);
    assert!(
        run.declarations()[0]
            .value_components()
            .iter()
            .any(|value| {
                matches!(
                    value.view(),
                    surgeist_css::CssComponentValueRef::Token(CssValueTokenRef::Delim('!'))
                )
            })
    );
    assert!(run.declarations()[1].importance_components().is_some());
    assert!(matches!(items[1], CssSupportsTestItem::AtRule(ref rule) if rule.body().is_none()));
    assert!(matches!(items[2], CssSupportsTestItem::AtRule(ref rule) if rule.body().is_some()));
    let CssSupportsTestItem::QualifiedRule(qualified) = &items[3] else {
        panic!("expected nested qualified test");
    };
    let [CssSupportsTestItem::Declarations(nested)] = qualified.body().items() else {
        panic!("expected nested declaration run");
    };
    let tokens: Vec<_> = nested.declarations()[0]
        .value_components()
        .iter()
        .filter(|component| {
            !matches!(
                component.view(),
                surgeist_css::CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
            )
        })
        .collect();
    assert!(matches!(
        tokens[0].view(),
        surgeist_css::CssComponentValueRef::Token(CssValueTokenRef::Number(_))
    ));
    assert!(matches!(
        tokens[1].view(),
        surgeist_css::CssComponentValueRef::Comment(_)
    ));
    assert!(matches!(
        tokens[2].view(),
        surgeist_css::CssComponentValueRef::Token(CssValueTokenRef::Ident("e3"))
    ));
}

#[test]
fn independently_supplied_adjacent_tokens_gain_a_safe_separator() {
    let body = CssSupportsTestBody::try_from_components(
        CssComponentValues::try_new(vec![
            CssComponentValue::try_ident("future").unwrap(),
            CssComponentValue::try_token(":").unwrap(),
            CssComponentValue::try_token("1").unwrap(),
            CssComponentValue::try_ident("e3").unwrap(),
            CssComponentValue::try_token(";").unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(body.to_specified_css().unwrap(), "future: 1/**/e3;");
    let [CssSupportsTestItem::Declarations(run)] = body.items() else {
        panic!("expected a checked declaration run");
    };
    let value = run.declarations()[0].value_components();
    assert!(matches!(
        value[0].view(),
        surgeist_css::CssComponentValueRef::Token(CssValueTokenRef::Number(_))
    ));
    assert!(matches!(
        value[1].view(),
        surgeist_css::CssComponentValueRef::Token(CssValueTokenRef::Ident("e3"))
    ));
}

#[test]
fn sheet_budget_is_monotonic_across_palette_and_definition() {
    let source = "@font-palette-values --colors { font-family: Demo; }\n@supports-condition --feature { future:a; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [palette, named] = report.syntax().rules() else {
        panic!("expected two rules");
    };
    let first = palette.to_specified_css().unwrap();
    let second = named.to_specified_css().unwrap();
    let expected = format!("{first}\n{second}");
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);

    let error = report
        .syntax()
        .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
            usize::MAX,
            usize::MAX,
            expected.len() - 1,
        ))
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CssSpecifiedRuleSerializationErrorKind::Resource(
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        )
    );
    assert_eq!(error.rule_index(), Some(1));

    for (input, projection, expected_kind) in [
        (
            true,
            false,
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            false,
            true,
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        let limit = (1..100).find(|limit| {
            let limits = CssSpecifiedValueSerializationLimits::new(
                if input { *limit } else { usize::MAX },
                if projection { *limit } else { usize::MAX },
                usize::MAX,
            );
            palette.to_specified_css_with_limits(limits).is_ok()
                && named.to_specified_css_with_limits(limits).is_ok()
                && matches!(report.syntax().to_specified_css_with_limits(limits), Err(ref error) if error.kind() == CssSpecifiedRuleSerializationErrorKind::Resource(expected_kind) && error.rule_index() == Some(1))
        });
        assert!(
            limit.is_some(),
            "individual rules must fit while their aggregate fails"
        );
    }
}

#[test]
fn supported_definitions_compose_with_a_following_style_rule() {
    let report = parse_sheet("@supports-condition --feature {}\n.after { color: red; }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        "@supports-condition --feature { }\n.after { color: red; }"
    );
    assert_eq!(report, before);
}
