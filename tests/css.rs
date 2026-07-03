use surgeist::{adapters, css, style as s};

fn lower_sheet(input: &str) -> s::Sheet {
    let css = css::parse_sheet(input).expect("CSS should parse");
    adapters::lower_css_sheet_to_style(&css).expect("CSS should lower to style")
}

#[test]
fn parses_inline_block_display() {
    let sheet = lower_sheet(".panel { display: inline-block; }");
    let declarations = sheet.rules()[0].declarations();

    assert_eq!(
        declarations.get(s::Property::Display),
        Some(&s::Value::Display(s::Display::InlineBlock))
    );
}

#[test]
fn parses_class_tag_and_key_rules_into_ordered_style_sheet() {
    let sheet = lower_sheet(
        r#"
        .panel {
            display: grid;
            gap: 8px;
            width: 50%;
            color: #336699;
        }

        button.primary {
            display: flex;
            flex-direction: column;
        }

        #run {
            opacity: 0.75;
        }
        "#,
    );

    assert_eq!(sheet.rule_count(), 3);

    let panel = &sheet.rules()[0];
    assert_eq!(panel.selector(), &s::Selector::class("panel").unwrap());
    assert_eq!(
        panel.declarations().get(s::Property::Display),
        Some(&s::Value::Display(s::Display::Grid))
    );
    assert_eq!(
        panel.declarations().get(s::Property::RowGap),
        Some(&s::Value::Length(s::Length::px(8.0)))
    );
    assert_eq!(
        panel.declarations().get(s::Property::ColumnGap),
        Some(&s::Value::Length(s::Length::px(8.0)))
    );
    assert_eq!(
        panel.declarations().get(s::Property::Width),
        Some(&s::Value::Length(s::Length::percent(50.0)))
    );
    assert_eq!(
        panel.declarations().get(s::Property::Color),
        Some(&s::Value::Color(s::Color::rgba(
            0x33 as f32 / 255.0,
            0x66 as f32 / 255.0,
            0x99 as f32 / 255.0,
            1.0
        )))
    );

    assert_eq!(
        sheet.rules()[1].selector(),
        &s::Selector::compound()
            .tag("button")
            .unwrap()
            .class("primary")
            .unwrap()
            .selector()
    );
    assert_eq!(
        sheet.rules()[2].selector(),
        &s::Selector::key("run").unwrap()
    );
}

#[test]
fn parses_grid_flow_tolerance_values() {
    let sheet = lower_sheet(
        r#"
        .normal { grid-flow-tolerance: normal; }
        .length { grid-flow-tolerance: 12px; }
        .percent { grid-flow-tolerance: 25%; }
        .infinite { grid-flow-tolerance: infinite; }
        "#,
    );

    assert_eq!(
        sheet.rules()[0]
            .declarations()
            .get(s::Property::GridFlowTolerance),
        Some(&s::Value::GridFlowTolerance(s::GridFlowTolerance::Normal))
    );
    assert_eq!(
        sheet.rules()[1]
            .declarations()
            .get(s::Property::GridFlowTolerance),
        Some(&s::Value::GridFlowTolerance(s::GridFlowTolerance::Length(
            s::Length::px(12.0)
        )))
    );
    assert_eq!(
        sheet.rules()[2]
            .declarations()
            .get(s::Property::GridFlowTolerance),
        Some(&s::Value::GridFlowTolerance(s::GridFlowTolerance::Percent(
            25.0
        )))
    );
    assert_eq!(
        sheet.rules()[3]
            .declarations()
            .get(s::Property::GridFlowTolerance),
        Some(&s::Value::GridFlowTolerance(s::GridFlowTolerance::Infinite))
    );
}

#[test]
fn parses_gap_normal_distinct_from_explicit_zero() {
    let sheet = lower_sheet(
        r#"
        .normal { gap: normal; }
        .zero { gap: 0; }
        "#,
    );

    assert_eq!(
        sheet.rules()[0].declarations().get(s::Property::RowGap),
        Some(&s::Value::Length(s::Length::NORMAL))
    );
    assert_eq!(
        sheet.rules()[0].declarations().get(s::Property::ColumnGap),
        Some(&s::Value::Length(s::Length::NORMAL))
    );
    assert_eq!(
        sheet.rules()[1].declarations().get(s::Property::RowGap),
        Some(&s::Value::Length(s::Length::ZERO))
    );
    assert_eq!(
        sheet.rules()[1].declarations().get(s::Property::ColumnGap),
        Some(&s::Value::Length(s::Length::ZERO))
    );
}

#[test]
fn parses_css_edge_shorthands_into_canonical_declarations() {
    let sheet = lower_sheet(
        r#"
        .card {
            margin: 1px 2px 3px 4px;
            padding: 6px 8px;
            overflow: hidden scroll;
        }
        "#,
    );

    let declarations = sheet.rules()[0].declarations();
    assert_eq!(
        declarations.margin_edges(),
        Some(s::Edges::new(
            s::Length::px(1.0),
            s::Length::px(2.0),
            s::Length::px(3.0),
            s::Length::px(4.0)
        ))
    );
    assert_eq!(
        declarations.padding_edges(),
        Some(s::Edges::new(
            s::Length::px(6.0),
            s::Length::px(8.0),
            s::Length::px(6.0),
            s::Length::px(8.0)
        ))
    );
    assert_eq!(
        declarations.get(s::Property::OverflowX),
        Some(&s::Value::Overflow(s::Overflow::Hidden))
    );
    assert_eq!(
        declarations.get(s::Property::OverflowY),
        Some(&s::Value::Overflow(s::Overflow::Scroll))
    );
}

#[test]
fn parses_first_and_last_baseline_item_alignment() {
    let sheet = lower_sheet(
        ".a { align-items: first baseline; align-self: last baseline; justify-items: baseline; }",
    );

    let rule = &sheet.rules()[0];
    assert_eq!(
        rule.declarations().get(s::Property::AlignItems),
        Some(&s::Value::AlignItems(s::AlignItems::Baseline))
    );
    assert_eq!(
        rule.declarations().get(s::Property::AlignSelf),
        Some(&s::Value::AlignItems(s::AlignItems::LastBaseline))
    );
    assert_eq!(
        rule.declarations().get(s::Property::JustifyItems),
        Some(&s::Value::AlignItems(s::AlignItems::Baseline))
    );
}

#[test]
fn rejects_overflow_position_on_baseline_alignment() {
    for css in [
        ".a { align-items: safe baseline; }",
        ".a { align-self: unsafe first baseline; }",
        ".a { justify-self: safe last baseline; }",
    ] {
        assert!(
            css::parse_sheet(css).is_err(),
            "prefixed baseline alignment should be rejected: {css}"
        );
    }
}

#[test]
fn extended_sheets_preserve_manifest_load_order() {
    let mut sheet = lower_sheet(".panel { color: #111111; }");
    let theme = lower_sheet(".panel { color: #222222; }");

    sheet.extend(theme.rules().iter().cloned());

    assert_eq!(sheet.rule_count(), 2);
    assert_eq!(sheet.rules()[0].order(), 0);
    assert_eq!(sheet.rules()[1].order(), 1);
    assert_eq!(
        sheet.rules()[1].declarations().get(s::Property::Color),
        Some(&s::Value::Color(s::Color::rgba(
            0x22 as f32 / 255.0,
            0x22 as f32 / 255.0,
            0x22 as f32 / 255.0,
            1.0
        )))
    );
}

#[test]
fn rejects_unsupported_css_instead_of_recovering_like_a_browser() {
    let property = css::parse_sheet(".panel { unknown-prop: 1px; }").unwrap_err();
    assert!(
        property.message().contains("unsupported CSS property"),
        "{property}"
    );

    let at_rule =
        css::parse_sheet("@media (min-width: 1px) { .panel { display: flex; } }").unwrap_err();
    assert!(
        at_rule.message().contains("unsupported CSS at-rule"),
        "{at_rule}"
    );

    let selector = css::parse_sheet(".panel > .label { display: flex; }").unwrap_err();
    assert!(
        selector.message().contains("unexpected CSS token"),
        "{selector}"
    );
}

#[test]
fn rejects_style_domain_errors_during_css_style_lowering() {
    let css = css::parse_sheet(".panel { opacity: 2; }").unwrap();
    let error = adapters::lower_css_sheet_to_style(&css).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("Opacity must be between 0 and 1"),
        "{error}"
    );
}
