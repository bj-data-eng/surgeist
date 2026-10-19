#![forbid(unsafe_code)]

//! Intrinsic longhand contracts for four CSS2.1 table keyword properties.
//! Source: https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html
//! Property tables: border-collapse, caption-side, empty-cells, table-layout.
//! Logical 1's inline-start/end caption extension is conditional on support for
//! unselected left/right caption values. Their rejection below is specific to
//! this selected profile, not a judgment that they are invalid in every CSS UA.

use surgeist_css::*;

type Property = CssKnownProperty;

#[derive(Clone, Copy, Debug)]
enum TableKeyword {
    BorderCollapse(CssBorderCollapse),
    CaptionSide(CssCaptionSide),
    EmptyCells(CssEmptyCells),
    TableLayout(CssTableLayout),
}

impl TableKeyword {
    fn assert_payload(self, actual: CssLonghandValueRef<'_>) {
        match (self, actual) {
            (Self::BorderCollapse(expected), CssLonghandValueRef::BorderCollapse(actual)) => {
                assert_eq!(*actual, expected)
            }
            (Self::CaptionSide(expected), CssLonghandValueRef::CaptionSide(actual)) => {
                assert_eq!(*actual, expected)
            }
            (Self::EmptyCells(expected), CssLonghandValueRef::EmptyCells(actual)) => {
                assert_eq!(*actual, expected)
            }
            (Self::TableLayout(expected), CssLonghandValueRef::TableLayout(actual)) => {
                assert_eq!(*actual, expected)
            }
            other => panic!("table longhand payload changed: {other:?}"),
        }
    }

    fn serialize(
        self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        match self {
            Self::BorderCollapse(value) => value.serialize_specified_with_limits(limits),
            Self::CaptionSide(value) => value.serialize_specified_with_limits(limits),
            Self::EmptyCells(value) => value.serialize_specified_with_limits(limits),
            Self::TableLayout(value) => value.serialize_specified_with_limits(limits),
        }
    }
}

const ORDINARY: &[(Property, &str, TableKeyword)] = &[
    (
        Property::BorderCollapse,
        "collapse",
        TableKeyword::BorderCollapse(CssBorderCollapse::Collapse),
    ),
    (
        Property::BorderCollapse,
        "SePaRaTe",
        TableKeyword::BorderCollapse(CssBorderCollapse::Separate),
    ),
    (
        Property::CaptionSide,
        "top",
        TableKeyword::CaptionSide(CssCaptionSide::Top),
    ),
    (
        Property::CaptionSide,
        "BoTtOm",
        TableKeyword::CaptionSide(CssCaptionSide::Bottom),
    ),
    (
        Property::EmptyCells,
        "show",
        TableKeyword::EmptyCells(CssEmptyCells::Show),
    ),
    (
        Property::EmptyCells,
        "HiDe",
        TableKeyword::EmptyCells(CssEmptyCells::Hide),
    ),
    (
        Property::TableLayout,
        "auto",
        TableKeyword::TableLayout(CssTableLayout::Auto),
    ),
    (
        Property::TableLayout,
        "FiXeD",
        TableKeyword::TableLayout(CssTableLayout::Fixed),
    ),
];

#[test]
fn table_keywords_serialize_all_eight_values_with_atomic_resource_limits() {
    for (value, text) in [
        (
            TableKeyword::BorderCollapse(CssBorderCollapse::Collapse),
            "collapse",
        ),
        (
            TableKeyword::BorderCollapse(CssBorderCollapse::Separate),
            "separate",
        ),
        (TableKeyword::CaptionSide(CssCaptionSide::Top), "top"),
        (TableKeyword::CaptionSide(CssCaptionSide::Bottom), "bottom"),
        (TableKeyword::EmptyCells(CssEmptyCells::Show), "show"),
        (TableKeyword::EmptyCells(CssEmptyCells::Hide), "hide"),
        (TableKeyword::TableLayout(CssTableLayout::Auto), "auto"),
        (TableKeyword::TableLayout(CssTableLayout::Fixed), "fixed"),
    ] {
        assert_eq!(
            value
                .serialize(CssSpecifiedValueSerializationLimits::default())
                .unwrap(),
            text
        );
        assert_eq!(
            value
                .serialize(CssSpecifiedValueSerializationLimits::new(1, 1, text.len()))
                .unwrap(),
            text
        );
        for (limits, expected) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, text.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, text.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, text.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(value.serialize(limits).unwrap_err().kind(), expected);
        }
    }
}

#[test]
fn table_keywords_report_complete_authored_grammar_from_the_pinned_css2_tables() {
    for (property, name, feature_id) in [
        (
            Property::BorderCollapse,
            "border-collapse",
            "official.property.border-collapse",
        ),
        (
            Property::CaptionSide,
            "caption-side",
            "official.property.caption-side",
        ),
        (
            Property::EmptyCells,
            "empty-cells",
            "official.property.empty-cells",
        ),
        (
            Property::TableLayout,
            "table-layout",
            "official.property.table-layout",
        ),
    ] {
        let support = property_support_metadata(name).expect("recognized table property");
        assert_eq!(support.property(), property);
        let feature = support.feature();
        assert_eq!(feature.id().as_str(), feature_id);
        assert_eq!(feature.source().id().as_str(), "O-CSS2");
        assert_eq!(
            feature.source().url(),
            Some("https://www.w3.org/TR/2011/REC-CSS2-20110607/")
        );
        assert_eq!(feature.production(), format!("tables.html#propdef-{name}"));
        assert_eq!(feature.status(), CssSupportStatus::Complete);
        assert_eq!(feature.supported_subset(), None);
        assert_eq!(feature.unsupported_remainder(), None);
        assert_eq!(feature.recognized_unsupported_code(), None);
    }
}

fn declaration(property: Property, value: &str, importance: CssImportance) -> CssDeclaration {
    let suffix = if importance == CssImportance::Important {
        "!important"
    } else {
        ""
    };
    let source = format!("{}:{value}{suffix}", property.canonical_name());
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one parsed declaration: {source}")
    };
    declaration.clone()
}

fn assert_authored_keyword(source: &CssDeclaration, expected: TableKeyword, authored: &str) {
    let view = source
        .known()
        .unwrap()
        .property_value()
        .expect("ordinary property value");
    match (expected, view) {
        (
            TableKeyword::BorderCollapse(expected),
            CssKnownPropertyValueRef::BorderCollapse(value),
        ) => {
            assert_eq!(value.collapse(), &expected);
            assert_eq!(value.as_css(), authored);
        }
        (TableKeyword::CaptionSide(expected), CssKnownPropertyValueRef::CaptionSide(value)) => {
            assert_eq!(value.side(), &expected);
            assert_eq!(value.as_css(), authored);
        }
        (TableKeyword::EmptyCells(expected), CssKnownPropertyValueRef::EmptyCells(value)) => {
            assert_eq!(value.cells(), &expected);
            assert_eq!(value.as_css(), authored);
        }
        (TableKeyword::TableLayout(expected), CssKnownPropertyValueRef::TableLayout(value)) => {
            assert_eq!(value.layout(), &expected);
            assert_eq!(value.as_css(), authored);
        }
        other => panic!("property/value coupling changed: {other:?}"),
    }
}

fn one_longhand(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("selected table longhand expands")
    else {
        panic!("one completed longhand contribution")
    };
    let [item] = values.items() else {
        panic!("exactly one table longhand contribution")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert_eq!(
        item.ordinary_value().unwrap().property().known_property(),
        item.property()
    );
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    values
}

#[test]
fn table_keywords_have_source_derived_longhand_inheritance_and_fixed_initials() {
    // CSS2.1 table property definitions give separate/top/show/auto as initials.
    for (property, inherited, expected) in [
        (
            Property::BorderCollapse,
            true,
            TableKeyword::BorderCollapse(CssBorderCollapse::Separate),
        ),
        (
            Property::CaptionSide,
            true,
            TableKeyword::CaptionSide(CssCaptionSide::Top),
        ),
        (
            Property::EmptyCells,
            true,
            TableKeyword::EmptyCells(CssEmptyCells::Show),
        ),
        (
            Property::TableLayout,
            false,
            TableKeyword::TableLayout(CssTableLayout::Auto),
        ),
    ] {
        let metadata = property.metadata().expect("intrinsic table metadata");
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("{property:?} must be a terminal longhand")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert_eq!(longhand.inherited_by_default(), inherited);
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), property);
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("{property:?} has a fixed intrinsic initial")
        };
        assert_eq!(value.property().known_property(), property);
        expected.assert_payload(value.view());
    }
}

#[test]
fn table_keywords_from_parsed_and_checked_construction_contribute_once() {
    for &(property, keyword, expected) in ORDINARY {
        let parsed = declaration(property, keyword, CssImportance::Important);
        assert_authored_keyword(&parsed, expected, keyword);
        let parsed_items = one_longhand(&parsed);
        assert!(parsed_items.items()[0].replacement_components().is_none());
        expected.assert_payload(parsed_items.items()[0].ordinary_value().unwrap().view());

        let components = CssComponentValues::try_new(vec![
            CssComponentValue::try_token(keyword).expect("one identifier token"),
        ])
        .unwrap();
        assert!(matches!(
            components.items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
        let constructed = parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Important,
        )
        .expect("checked programmatic table keyword");
        assert!(constructed.parsed_value().is_none());
        assert_eq!(constructed.value_components(), &components);
        assert_authored_keyword(&constructed, expected, keyword);
        let constructed_items = one_longhand(&constructed);
        assert!(
            constructed_items.items()[0]
                .replacement_components()
                .is_none()
        );
        expected.assert_payload(
            constructed_items.items()[0]
                .ordinary_value()
                .unwrap()
                .view(),
        );
    }
}

#[test]
fn table_keywords_keep_globals_symbolic_and_reenter_pending_values_strictly() {
    for (property, valid, expected) in [
        (
            Property::BorderCollapse,
            "separate",
            TableKeyword::BorderCollapse(CssBorderCollapse::Separate),
        ),
        (
            Property::CaptionSide,
            "bottom",
            TableKeyword::CaptionSide(CssCaptionSide::Bottom),
        ),
        (
            Property::EmptyCells,
            "hide",
            TableKeyword::EmptyCells(CssEmptyCells::Hide),
        ),
        (
            Property::TableLayout,
            "fixed",
            TableKeyword::TableLayout(CssTableLayout::Fixed),
        ),
    ] {
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
        ] {
            let source = declaration(property, text, CssImportance::Important);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).expect("CSS-wide table value")
            else {
                panic!("global table longhand")
            };
            let [item] = values.items() else {
                panic!("one global table contribution")
            };
            assert_eq!(item.property(), property);
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
        }

        let source = declaration(property, "var(--table-keyword)", CssImportance::Important);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("table property substitution remains pending")
        };
        assert!(handle.source().same_occurrence(&source));
        let invalid = parse_component_values(&format!("{valid} {valid}")).unwrap();
        assert!(matches!(
            handle.reenter(invalid).unwrap_err().kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        assert_eq!(
            handle
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        let replacement = parse_component_values(valid).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = handle
                .reenter(replacement.clone())
                .expect("reusable strict reentry")
            else {
                panic!("one completed replacement longhand")
            };
            let [item] = values.items() else {
                panic!("one replacement contribution")
            };
            assert_eq!(item.property(), property);
            assert_eq!(
                item.ordinary_value().unwrap().property().known_property(),
                property
            );
            expected.assert_payload(item.ordinary_value().unwrap().view());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn table_keyword_normalization_keeps_source_order_after_declaration_recovery() {
    let report = parse_sheet(concat!(
        ".a{border-collapse:collapse;caption-side:top;",
        "caption-side:top bottom;empty-cells:hide;table-layout:fixed!important}"
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid compound caption-side declaration")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let normalized = normalize_sheet(report.syntax()).expect("four supported table terminals");
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 4);
    for (index, property) in [
        Property::BorderCollapse,
        Property::CaptionSide,
        Property::EmptyCells,
        Property::TableLayout,
    ]
    .into_iter()
    .enumerate()
    {
        let item = declarations[index];
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().property(), property);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("normalized table longhand")
        };
        let [value] = values.items() else {
            panic!("one normalized terminal")
        };
        assert_eq!(value.property(), property);
        assert!(value.source().same_occurrence(item.source()));
    }
    assert_eq!(
        declarations[3].source().importance(),
        CssImportance::Important
    );
}

#[test]
fn table_keyword_invalid_values_and_unselected_caption_extension_recover_atomically() {
    for (property, invalid) in [
        (Property::BorderCollapse, "merge"),
        (Property::BorderCollapse, "collapse separate"),
        (Property::CaptionSide, "top bottom"),
        (Property::CaptionSide, "left"),
        (Property::CaptionSide, "right"),
        (Property::CaptionSide, "inline-start"),
        (Property::CaptionSide, "inline-end"),
        (Property::EmptyCells, "auto"),
        (Property::EmptyCells, "show hide"),
        (Property::TableLayout, "collapse"),
        (Property::TableLayout, "auto fixed"),
    ] {
        let source = format!(
            "color:red;{}:{invalid};color:blue",
            property.canonical_name()
        );
        let report = parse_style_attribute(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid table declaration: {source}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            Property::Color
        );
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            Property::Color
        );
        assert!(validate_style_attribute(&source).is_err(), "{source}");
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(property),
                parse_component_values(invalid).unwrap(),
                CssImportance::Normal,
            )
            .is_err(),
            "checked construction must reject {source}"
        );
    }
}
