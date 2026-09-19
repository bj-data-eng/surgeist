#![forbid(unsafe_code)]
//! Independent specified-value tables from Display 3 §2 and Grid 3 §2.2.
//! Legacy spellings remain distinct from modern outside/inside pairs (§2.6).
use surgeist_css::*;

fn declaration(input: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("display:{input}!important"));
    assert!(report.is_clean(), "{input}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    report.syntax()[0].clone()
}

fn value(declaration: &CssDeclaration) -> &CssDisplayPropertyValue {
    let CssKnownPropertyValueRef::Display(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary display")
    };
    value
}

fn cases() -> Vec<(CssDisplayValue, &'static str)> {
    use CssDisplayInside as I;
    use CssDisplayOutside as O;
    let mut cases = Vec::new();
    for (inside, outputs) in [
        (I::Flow, ["block", "inline", "run-in"]),
        (
            I::FlowRoot,
            ["flow-root", "inline flow-root", "run-in flow-root"],
        ),
        (I::Table, ["table", "inline table", "run-in table"]),
        (I::Flex, ["flex", "inline flex", "run-in flex"]),
        (I::Grid, ["grid", "inline grid", "run-in grid"]),
        (I::Ruby, ["block ruby", "ruby", "run-in ruby"]),
    ] {
        for (outside, output) in [O::Block, O::Inline, O::RunIn].into_iter().zip(outputs) {
            cases.push((CssDisplayValue::OutsideInside { outside, inside }, output));
        }
    }
    for (inside, outputs) in [
        (
            CssDisplayListItemInside::Flow,
            ["list-item", "inline list-item", "run-in list-item"],
        ),
        (
            CssDisplayListItemInside::FlowRoot,
            [
                "flow-root list-item",
                "inline flow-root list-item",
                "run-in flow-root list-item",
            ],
        ),
    ] {
        for (outside, output) in [O::Block, O::Inline, O::RunIn].into_iter().zip(outputs) {
            cases.push((CssDisplayValue::ListItem { outside, inside }, output));
        }
    }
    use CssDisplayInternal as N;
    for (internal, output) in [
        (N::TableRowGroup, "table-row-group"),
        (N::TableHeaderGroup, "table-header-group"),
        (N::TableFooterGroup, "table-footer-group"),
        (N::TableRow, "table-row"),
        (N::TableCell, "table-cell"),
        (N::TableColumnGroup, "table-column-group"),
        (N::TableColumn, "table-column"),
        (N::TableCaption, "table-caption"),
        (N::RubyBase, "ruby-base"),
        (N::RubyText, "ruby-text"),
        (N::RubyBaseContainer, "ruby-base-container"),
        (N::RubyTextContainer, "ruby-text-container"),
    ] {
        cases.push((CssDisplayValue::Internal(internal), output));
    }
    cases.extend([
        (CssDisplayValue::Box(CssDisplayBox::Contents), "contents"),
        (CssDisplayValue::Box(CssDisplayBox::None), "none"),
        (
            CssDisplayValue::Legacy(CssDisplayLegacy::InlineBlock),
            "inline-block",
        ),
        (
            CssDisplayValue::Legacy(CssDisplayLegacy::InlineTable),
            "inline-table",
        ),
        (
            CssDisplayValue::Legacy(CssDisplayLegacy::InlineFlex),
            "inline-flex",
        ),
        (
            CssDisplayValue::Legacy(CssDisplayLegacy::InlineGrid),
            "inline-grid",
        ),
        (CssDisplayValue::GridLanes, "grid-lanes"),
        (CssDisplayValue::InlineGridLanes, "inline-grid-lanes"),
    ]);
    cases
}

#[test]
fn every_constructed_specified_value_has_independent_canonical_text_and_payload() {
    let cases = cases();
    assert_eq!(cases.len(), 44);
    for (expected, text) in cases {
        // The constructed enum and literal output are independent expectations.
        assert_eq!(expected.serialize_specified().unwrap(), text);
        let parsed = declaration(text);
        assert_eq!(*value(&parsed).value(), expected);
        let before = parsed.clone();
        assert_eq!(value(&parsed).value().serialize_specified().unwrap(), text);
        assert_eq!(parsed, before);
        assert_eq!(value(&parsed).as_css(), text);

        let mut tokens = Vec::new();
        for word in text.split_whitespace() {
            if !tokens.is_empty() {
                tokens.push(CssComponentValue::try_token(" ").unwrap());
            }
            tokens.push(CssComponentValue::try_token(word).unwrap());
        }
        let components = CssComponentValues::try_new(tokens).unwrap();
        let constructed = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Display),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(*value(&constructed).value(), expected);
        assert!(constructed.parsed_value().is_none());
        assert_eq!(constructed.value_components(), &components);
        assert!(
            components
                .items()
                .iter()
                .all(|item| matches!(item.origin(), CssValueOrigin::Programmatic))
        );

        for (source, importance) in [
            (&parsed, CssImportance::Important),
            (&constructed, CssImportance::Normal),
        ] {
            let CssExpansion::Contributions(CssContributions::Longhands(contributions)) =
                expand_declaration(source).unwrap()
            else {
                panic!("one ordinary longhand")
            };
            let [contribution] = contributions.items() else {
                panic!("one contribution")
            };
            assert_eq!(contribution.property(), CssKnownProperty::Display);
            assert!(contribution.source().same_occurrence(source));
            assert_eq!(contribution.source().importance(), importance);
            let CssLonghandValueRef::Display(actual) =
                contribution.ordinary_value().unwrap().view()
            else {
                panic!("typed display payload")
            };
            assert_eq!(*actual, expected);
        }
    }
}

#[test]
fn defaults_permutations_case_and_comments_preserve_specified_meaning_and_raw_text() {
    for (input, canonical) in [
        ("flow", "block"),
        ("block flow", "block"),
        ("FLOW BLOCK", "block"),
        ("flow inline", "inline"),
        ("run-in", "run-in"),
        ("ruby", "ruby"),
        ("ruby block", "block ruby"),
        ("flex inline", "inline flex"),
        ("inline/**/flex", "inline flex"),
        (r"fl\65 x INLINE", "inline flex"),
        ("flow list-item block", "list-item"),
        ("list-item flow-root inline", "inline flow-root list-item"),
        ("list-item run-in flow-root", "run-in flow-root list-item"),
    ] {
        let parsed = declaration(input);
        let before = parsed.clone();
        assert_eq!(
            value(&parsed).value().serialize_specified().unwrap(),
            canonical
        );
        assert_eq!(
            value(&parsed).value(),
            value(&declaration(canonical)).value()
        );
        assert_eq!(value(&parsed).as_css(), input);
        assert_eq!(parsed, before);
        assert_eq!(
            value(&parsed)
                .value()
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 0))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(parsed, before);
        assert_eq!(value(&parsed).as_css(), input);
    }
    assert_eq!(
        *value(&declaration("ruby")).value(),
        CssDisplayValue::OutsideInside {
            outside: CssDisplayOutside::Inline,
            inside: CssDisplayInside::Ruby,
        }
    );
    assert_eq!(
        *value(&declaration("list-item")).value(),
        CssDisplayValue::ListItem {
            outside: CssDisplayOutside::Block,
            inside: CssDisplayListItemInside::Flow,
        }
    );
}

#[test]
fn legacy_specified_identity_is_distinct_and_frozen_projection_is_exact() {
    for (legacy, modern) in [
        ("inline-block", "inline flow-root"),
        ("inline-table", "inline table"),
        ("inline-flex", "inline flex"),
        ("inline-grid", "inline grid"),
    ] {
        assert_ne!(
            value(&declaration(legacy)).value(),
            value(&declaration(modern)).value()
        );
        assert_eq!(value(&declaration(modern)).i01_subset(), None);
    }
    for (text, expected) in [
        ("block", CssDisplay::Block),
        ("flow", CssDisplay::Block),
        ("block flow", CssDisplay::Block),
        ("flex", CssDisplay::Flex),
        ("block flex", CssDisplay::Flex),
        ("grid", CssDisplay::Grid),
        ("grid block", CssDisplay::Grid),
        ("inline-block", CssDisplay::InlineBlock),
        ("inline-grid", CssDisplay::InlineGrid),
        ("grid-lanes", CssDisplay::GridLanes),
        ("inline-grid-lanes", CssDisplay::InlineGridLanes),
        ("none", CssDisplay::None),
    ] {
        assert_eq!(
            value(&declaration(text)).i01_subset(),
            Some(&expected),
            "{text}"
        );
    }
    for text in [
        "inline",
        "table-cell",
        "contents",
        "list-item",
        "inline-table",
        "inline-flex",
        "run-in",
    ] {
        assert_eq!(value(&declaration(text)).i01_subset(), None, "{text}");
    }
}

#[test]
fn all_display_values_obey_atomic_projection_and_byte_limits() {
    use CssSpecifiedValueSerializationErrorKind as K;
    for (value, text) in cases() {
        let before = value;
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, 0),
                K::ByteLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(0, 0, 0),
                K::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, 0),
                K::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, text.len() - 1),
                K::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(value, before);
        }
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    text.len()
                ))
                .unwrap(),
            text
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    usize::MAX,
                    usize::MAX,
                    usize::MAX
                ))
                .unwrap(),
            text
        );
    }
}

#[test]
fn display_initial_is_noninherited_inline_flow() {
    let metadata = CssKnownProperty::Display.metadata().unwrap();
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("longhand")
    };
    assert!(!longhand.inherited_by_default());
    let initial = longhand.initial_value();
    assert_eq!(
        initial.property().known_property(),
        CssKnownProperty::Display
    );
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("fixed initial")
    };
    let CssLonghandValueRef::Display(value) = initial.view() else {
        panic!("display initial")
    };
    assert_eq!(
        *value,
        CssDisplayValue::OutsideInside {
            outside: CssDisplayOutside::Inline,
            inside: CssDisplayInside::Flow,
        }
    );
}
