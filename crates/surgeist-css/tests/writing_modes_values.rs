#![forbid(unsafe_code)]
//! Selected Writing Modes keyword values and intrinsic, context-free transport.
use surgeist_css::*;

#[derive(Clone, Copy, Debug)]
enum Expected {
    Direction(CssDirection),
    Bidi(CssUnicodeBidi),
    Writing(CssWritingMode),
    Orientation(CssTextOrientation),
}
impl Expected {
    fn property(self) -> CssKnownProperty {
        match self {
            Self::Direction(_) => CssKnownProperty::Direction,
            Self::Bidi(_) => CssKnownProperty::UnicodeBidi,
            Self::Writing(_) => CssKnownProperty::WritingMode,
            Self::Orientation(_) => CssKnownProperty::TextOrientation,
        }
    }
    fn assert_payload(self, value: CssLonghandValueRef<'_>) {
        match (self, value) {
            (Self::Direction(expected), CssLonghandValueRef::Direction(actual)) => {
                assert_eq!(*actual, expected)
            }
            (Self::Bidi(expected), CssLonghandValueRef::UnicodeBidi(actual)) => {
                assert_eq!(*actual, expected)
            }
            (Self::Writing(expected), CssLonghandValueRef::WritingMode(actual)) => {
                assert_eq!(*actual, expected)
            }
            (Self::Orientation(expected), CssLonghandValueRef::TextOrientation(actual)) => {
                assert_eq!(*actual, expected)
            }
            _ => panic!("expected {self:?}, got {value:?}"),
        }
    }
    fn serialize(
        self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        match self {
            Self::Direction(value) => value.serialize_specified_with_limits(limits),
            Self::Bidi(value) => value.serialize_specified_with_limits(limits),
            Self::Writing(value) => value.serialize_specified_with_limits(limits),
            Self::Orientation(value) => value.serialize_specified_with_limits(limits),
        }
    }
    fn default_serialize(self) -> Result<String, CssSpecifiedValueSerializationError> {
        match self {
            Self::Direction(value) => value.serialize_specified(),
            Self::Bidi(value) => value.serialize_specified(),
            Self::Writing(value) => value.serialize_specified(),
            Self::Orientation(value) => value.serialize_specified(),
        }
    }
}
const VALUES: &[(Expected, &str)] = &[
    (Expected::Direction(CssDirection::Ltr), "ltr"),
    (Expected::Direction(CssDirection::Rtl), "rtl"),
    (Expected::Bidi(CssUnicodeBidi::Normal), "normal"),
    (Expected::Bidi(CssUnicodeBidi::Embed), "embed"),
    (Expected::Bidi(CssUnicodeBidi::Isolate), "isolate"),
    (
        Expected::Bidi(CssUnicodeBidi::BidiOverride),
        "bidi-override",
    ),
    (
        Expected::Bidi(CssUnicodeBidi::IsolateOverride),
        "isolate-override",
    ),
    (Expected::Bidi(CssUnicodeBidi::Plaintext), "plaintext"),
    (
        Expected::Writing(CssWritingMode::HorizontalTb),
        "horizontal-tb",
    ),
    (Expected::Writing(CssWritingMode::VerticalRl), "vertical-rl"),
    (Expected::Writing(CssWritingMode::VerticalLr), "vertical-lr"),
    (Expected::Writing(CssWritingMode::SidewaysRl), "sideways-rl"),
    (Expected::Writing(CssWritingMode::SidewaysLr), "sideways-lr"),
    (Expected::Orientation(CssTextOrientation::Mixed), "mixed"),
    (
        Expected::Orientation(CssTextOrientation::Upright),
        "upright",
    ),
    (
        Expected::Orientation(CssTextOrientation::Sideways),
        "sideways",
    ),
];

fn declaration(property: CssKnownProperty, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{}:{text}!important", property.canonical_name()));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

#[test]
fn writing_modes_initials_are_direct_typed_values_with_distinct_inheritance() {
    for (expected, inherited) in [
        (Expected::Direction(CssDirection::Ltr), true),
        (Expected::Bidi(CssUnicodeBidi::Normal), false),
        (Expected::Writing(CssWritingMode::HorizontalTb), true),
        (Expected::Orientation(CssTextOrientation::Mixed), true),
    ] {
        let metadata = expected.property().metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("longhand")
        };
        assert_eq!(longhand.inherited_by_default(), inherited);
        let initial = longhand.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("ordinary initial")
        };
        expected.assert_payload(initial.view());
    }
}

#[test]
fn all_sixteen_typed_keywords_serialize_canonically_with_atomic_limits() {
    for &(value, text) in VALUES {
        assert_eq!(value.default_serialize().unwrap(), text);
        assert_eq!(
            value
                .serialize(CssSpecifiedValueSerializationLimits::new(1, 1, text.len()))
                .unwrap(),
            text
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 0, 0),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, 0),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, text.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, 0),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(value.serialize(limits).unwrap_err().kind(), kind);
        }
        let source = declaration(value.property(), text);
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("ordinary")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        value.assert_payload(item.ordinary_value().unwrap().view());
        assert!(item.source().same_occurrence(&source));
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            text
        );
    }
}

#[test]
fn checked_values_and_pending_reentry_preserve_typed_payload_and_both_origins() {
    for &(expected, text) in VALUES {
        let property = expected.property();
        let pending_source = declaration(property, "var(--mode)");
        let CssExpansion::Pending(pending) = expand_declaration(&pending_source).unwrap() else {
            panic!("pending")
        };
        for components in [
            parse_component_values(&text.to_ascii_uppercase()).unwrap(),
            CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()]).unwrap(),
        ] {
            let source = parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                CssImportance::Important,
            )
            .unwrap();
            let CssExpansion::Contributions(CssContributions::Longhands(items)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("ordinary")
            };
            let [item] = items.items() else {
                panic!("one terminal")
            };
            expected.assert_payload(item.ordinary_value().unwrap().view());
            assert_eq!(source.value_components(), &components);
            assert!(item.source().same_occurrence(&source));
            for _ in 0..2 {
                let CssContributions::Longhands(items) =
                    pending.reenter(components.clone()).unwrap()
                else {
                    panic!("replacement")
                };
                let [item] = items.items() else {
                    panic!("one terminal")
                };
                expected.assert_payload(item.ordinary_value().unwrap().view());
                assert!(item.source().same_occurrence(&pending_source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&components));
            }
        }
    }
}

#[test]
fn normalized_writing_modes_preserve_authored_order_and_do_not_apply_bidi_or_orientation() {
    for depth in [0, 63, 64, 65] {
        let text = format!(
            "@scope(.host){{{}.a{{direction:rtl;writing-mode:sideways-lr;text-orientation:upright;.child{{unicode-bidi:plaintext}}direction:ltr!important}}{}}}",
            "@media all{".repeat(depth),
            "}".repeat(depth)
        );
        let report = parse_sheet(&text);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        for _ in 0..2 {
            let sheet = normalize_sheet(report.syntax()).unwrap();
            let items: Vec<_> = sheet
                .items()
                .iter()
                .filter_map(|item| match item {
                    CssNormalizedItem::Declaration(value) => Some(value),
                    _ => None,
                })
                .collect();
            assert_eq!(items.len(), 5);
            let scope = items[0].selector_context().scope_context().unwrap();
            for (index, (item, expected)) in items
                .iter()
                .zip([
                    Expected::Direction(CssDirection::Rtl),
                    Expected::Writing(CssWritingMode::SidewaysLr),
                    Expected::Orientation(CssTextOrientation::Upright),
                    Expected::Bidi(CssUnicodeBidi::Plaintext),
                    Expected::Direction(CssDirection::Ltr),
                ])
                .enumerate()
            {
                assert_eq!(item.order(), index);
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    item.expansion()
                else {
                    panic!("ordinary")
                };
                let [value] = values.items() else {
                    panic!("one terminal")
                };
                expected.assert_payload(value.ordinary_value().unwrap().view());
                assert!(value.source().same_occurrence(item.source()));
                assert!(
                    item.selector_context()
                        .scope_context()
                        .unwrap()
                        .same_context(scope)
                );
            }
            assert!(
                items[3]
                    .selector_context()
                    .parent()
                    .unwrap()
                    .same_context(items[0].selector_context())
            );
            assert!(
                items[4]
                    .selector_context()
                    .same_context(items[0].selector_context())
            );
            assert_eq!(items[4].source().importance(), CssImportance::Important);
        }
    }
}

#[test]
fn parsed_wrapper_accessors_serialize_case_escapes_and_trivia_without_mutation() {
    for (property, input, expected_raw, expected) in [
        (CssKnownProperty::Direction, r"\72 tl/**/", r"\72 tl", "rtl"),
        (
            CssKnownProperty::UnicodeBidi,
            "ISOLATE-OVERRIDE/**/",
            "ISOLATE-OVERRIDE",
            "isolate-override",
        ),
        (
            CssKnownProperty::WritingMode,
            r"SIDEWAYS-\6c r/**/",
            r"SIDEWAYS-\6c r",
            "sideways-lr",
        ),
        (
            CssKnownProperty::TextOrientation,
            "UPRIGHT/**/",
            "UPRIGHT",
            "upright",
        ),
    ] {
        let source = declaration(property, input);
        let components = source.value_components().clone();
        macro_rules! project {
            ($wrapper:expr, $value:expr, $expected:expr) => {{
                let value = $value;
                assert_eq!(value, &$expected);
                let output = value.serialize_specified().unwrap();
                let error = value
                    .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                        1, 1, 0,
                    ))
                    .unwrap_err();
                ($wrapper.as_css(), output, error.kind())
            }};
        }
        let (raw, canonical, error) = match source.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::Direction(value) => {
                project!(value, value.current(), CssDirection::Rtl)
            }
            CssKnownPropertyValueRef::UnicodeBidi(value) => {
                project!(value, value.bidi(), CssUnicodeBidi::IsolateOverride)
            }
            CssKnownPropertyValueRef::WritingMode(value) => {
                project!(value, value.current(), CssWritingMode::SidewaysLr)
            }
            CssKnownPropertyValueRef::TextOrientation(value) => {
                project!(value, value.orientation(), CssTextOrientation::Upright)
            }
            _ => panic!("selected keyword wrapper"),
        };
        assert_eq!(raw, expected_raw);
        assert_eq!(canonical, expected);
        assert_eq!(error, CssSpecifiedValueSerializationErrorKind::ByteLimit);
        assert_eq!(source.value_components(), &components);
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            input
        );
    }
}

#[test]
fn recovered_keyword_neighbor_preserves_four_occurrences_with_exact_budgets() {
    let source = ".a{direction:rtl;unicode-bidi:rtl;writing-mode:sideways-lr;text-orientation:upright;unicode-bidi:isolate!important}";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert!(validate_sheet(source).is_err());
    for (limits, resource, limit) in [
        (
            CssNormalizationLimits::try_new(0, 0, 4, 4).unwrap(),
            CssNormalizationResource::Rules,
            0,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 3, 4).unwrap(),
            CssNormalizationResource::Declarations,
            3,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 4, 3).unwrap(),
            CssNormalizationResource::Contributions,
            3,
        ),
    ] {
        assert_eq!(
            normalize_sheet_with_limits(report.syntax(), limits)
                .unwrap_err()
                .kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit }
        );
    }
    let sheet = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 4, 4).unwrap(),
    )
    .unwrap();
    let items: Vec<_> = sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 4);
    for (index, (item, expected)) in items
        .iter()
        .zip([
            Expected::Direction(CssDirection::Rtl),
            Expected::Writing(CssWritingMode::SidewaysLr),
            Expected::Orientation(CssTextOrientation::Upright),
            Expected::Bidi(CssUnicodeBidi::Isolate),
        ])
        .enumerate()
    {
        assert_eq!(item.order(), index);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("ordinary")
        };
        let [value] = values.items() else {
            panic!("one contribution")
        };
        expected.assert_payload(value.ordinary_value().unwrap().view());
        assert!(value.source().same_occurrence(item.source()));
        assert_eq!(
            item.source().importance(),
            if index == 3 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
    }
}
