#![forbid(unsafe_code)]
//! CSSOM WD 2021-08-26 §6.7.2 selects shorter component lists before
//! component serialization; its number clause allows six fractional places.
//! Backgrounds 3 CRD 2024-03-11 §§5.2–5.4 defines quad expansion.
//! Frozen WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d corroborates typed
//! equality before text: CSSBorderImageSlice.h (values/backgrounds), lines
//! 35–41, and CSSValueTypes.h, lines 97–118. Goldens select exact edges first,
//! then round each emitted coefficient; rounded text is never an equality oracle.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

#[derive(Clone, Copy, Debug)]
enum Slot {
    SliceNumber,
    SlicePercentage,
    WidthNumber,
    WidthLength,
    OutsetNumber,
    OutsetLength,
}
const SLOTS: [Slot; 6] = [
    Slot::SliceNumber,
    Slot::SlicePercentage,
    Slot::WidthNumber,
    Slot::WidthLength,
    Slot::OutsetNumber,
    Slot::OutsetLength,
];

#[derive(Clone, Debug, PartialEq)]
enum Group {
    Slice(CssBorderImageSlice),
    Width(CssBorderImageWidth),
    Outset(CssBorderImageOutset),
}
impl Group {
    fn serialize(&self, limits: Limits) -> Result<String, CssSpecifiedValueSerializationError> {
        match self {
            Self::Slice(value) => value.serialize_specified_with_limits(limits),
            Self::Width(value) => value.serialize_specified_with_limits(limits),
            Self::Outset(value) => value.serialize_specified_with_limits(limits),
        }
    }
    fn literals(&self) -> Vec<&CssComponentValue> {
        match self {
            Self::Slice(value) => value
                .values()
                .iter()
                .map(|edge| match edge {
                    CssBorderImageSliceComponent::Number(value) => {
                        value.literal_component().unwrap()
                    }
                    CssBorderImageSliceComponent::Percentage(value) => {
                        value.literal_component().unwrap()
                    }
                    _ => panic!("numeric edge"),
                })
                .collect(),
            Self::Width(value) => value
                .values()
                .iter()
                .map(|edge| match edge {
                    CssBorderImageWidthComponent::Number(value) => {
                        value.literal_component().unwrap()
                    }
                    CssBorderImageWidthComponent::LengthPercentage(value) => {
                        value.literal_component().unwrap()
                    }
                    _ => panic!("numeric edge"),
                })
                .collect(),
            Self::Outset(value) => value
                .values()
                .iter()
                .map(|edge| match edge {
                    CssBorderImageOutsetComponent::Number(value) => {
                        value.literal_component().unwrap()
                    }
                    CssBorderImageOutsetComponent::Length(value) => {
                        value.literal_component().unwrap()
                    }
                    _ => panic!("numeric edge"),
                })
                .collect(),
        }
    }
}
fn group(slot: Slot, coefficients: [&str; 4]) -> Group {
    let components: Vec<_> = coefficients
        .iter()
        .map(|coefficient| match slot {
            Slot::SlicePercentage => {
                CssComponentValue::try_token(&format!("{coefficient}%")).unwrap()
            }
            Slot::WidthLength | Slot::OutsetLength => {
                CssComponentValue::try_token(&format!("{coefficient}px")).unwrap()
            }
            _ => CssComponentValue::try_number(coefficient).unwrap(),
        })
        .collect();
    match slot {
        Slot::SliceNumber | Slot::SlicePercentage => Group::Slice(
            CssBorderImageSlice::try_new(
                components
                    .into_iter()
                    .map(|component| match slot {
                        Slot::SlicePercentage => CssBorderImageSliceComponent::Percentage(
                            CssSpecifiedNonNegativePercentage::try_from_component(component)
                                .unwrap(),
                        ),
                        _ => CssBorderImageSliceComponent::Number(
                            CssSpecifiedNonNegativeNumber::try_from_component(component).unwrap(),
                        ),
                    })
                    .collect(),
                false,
            )
            .unwrap(),
        ),
        Slot::WidthNumber | Slot::WidthLength => Group::Width(
            CssBorderImageWidth::try_new(
                components
                    .into_iter()
                    .map(|component| match slot {
                        Slot::WidthLength => CssBorderImageWidthComponent::LengthPercentage(
                            CssSpecifiedNonNegativeLengthPercentage::try_from_component(component)
                                .unwrap(),
                        ),
                        _ => CssBorderImageWidthComponent::Number(
                            CssSpecifiedNonNegativeNumber::try_from_component(component).unwrap(),
                        ),
                    })
                    .collect(),
            )
            .unwrap(),
        ),
        Slot::OutsetNumber | Slot::OutsetLength => Group::Outset(
            CssBorderImageOutset::try_new(
                components
                    .into_iter()
                    .map(|component| match slot {
                        Slot::OutsetLength => CssBorderImageOutsetComponent::Length(
                            CssSpecifiedNonNegativeLength::try_from_component(component).unwrap(),
                        ),
                        _ => CssBorderImageOutsetComponent::Number(
                            CssSpecifiedNonNegativeNumber::try_from_component(component).unwrap(),
                        ),
                    })
                    .collect(),
            )
            .unwrap(),
        ),
    }
}
fn raw(component: &CssComponentValue) -> String {
    CssComponentValues::try_new(vec![component.clone()])
        .unwrap()
        .serialize()
        .unwrap()
        .as_css()
        .to_owned()
}
fn image(text: &str) -> CssBorderImage {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::BorderImage),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::BorderImage(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed border-image")
    };
    value.border_image().clone()
}
const UNEQUAL: [&str; 4] = [".12345641", ".12345642", ".12345643", ".12345644"];
fn unequal_edges(slot: Slot, expected: &str) {
    let value = group(slot, UNEQUAL);
    let before = value.clone();
    let originals: Vec<_> = value.literals().iter().map(|value| raw(value)).collect();
    let origins: Vec<_> = value
        .literals()
        .iter()
        .map(|value| value.origin().clone())
        .collect();
    let result = value.serialize(Limits::default());
    assert_eq!(value, before);
    assert_eq!(
        value
            .literals()
            .iter()
            .map(|value| raw(value))
            .collect::<Vec<_>>(),
        originals
    );
    assert_eq!(
        value
            .literals()
            .iter()
            .map(|value| value.origin().clone())
            .collect::<Vec<_>>(),
        origins
    );
    assert!(
        origins
            .iter()
            .all(|origin| *origin == CssValueOrigin::Programmatic)
    );
    assert_eq!(result.unwrap(), expected);
}
macro_rules! unequal_test {
    ($name:ident, $slot:ident, $expected:literal) => {
        #[test]
        fn $name() {
            unequal_edges(Slot::$slot, $expected);
        }
    };
}
unequal_test!(
    slice_numbers_keep_four_exactly_unequal_edges,
    SliceNumber,
    "0.123456 0.123456 0.123456 0.123456"
);
unequal_test!(
    slice_percentages_keep_four_exactly_unequal_edges,
    SlicePercentage,
    "0.123456% 0.123456% 0.123456% 0.123456%"
);
unequal_test!(
    width_numbers_keep_four_exactly_unequal_edges,
    WidthNumber,
    "0.123456 0.123456 0.123456 0.123456"
);
unequal_test!(
    width_lengths_keep_four_exactly_unequal_edges,
    WidthLength,
    "0.123456px 0.123456px 0.123456px 0.123456px"
);
unequal_test!(
    outset_numbers_keep_four_exactly_unequal_edges,
    OutsetNumber,
    "0.123456 0.123456 0.123456 0.123456"
);
unequal_test!(
    outset_lengths_keep_four_exactly_unequal_edges,
    OutsetLength,
    "0.123456px 0.123456px 0.123456px 0.123456px"
);

#[test]
fn exact_equivalent_spellings_still_compress_to_one_edge() {
    for (slot, expected) in [
        (Slot::SliceNumber, "1"),
        (Slot::SlicePercentage, "1%"),
        (Slot::WidthNumber, "1"),
        (Slot::WidthLength, "1px"),
        (Slot::OutsetNumber, "1"),
        (Slot::OutsetLength, "1px"),
    ] {
        assert_eq!(
            group(slot, ["+1.0", "1e0", "01", "1"])
                .serialize(Limits::new(9, 9, expected.len()))
                .unwrap(),
            expected,
            "{slot:?}"
        );
    }
}

#[test]
fn exact_quad_selection_keeps_two_and_three_even_when_text_is_equal() {
    for (coefficients, expected) in [
        (
            [".12345641", ".12345642", ".123456410", ".123456420"],
            "0.123456 0.123456",
        ),
        (
            [".12345641", ".12345642", ".12345643", ".123456420"],
            "0.123456 0.123456 0.123456",
        ),
    ] {
        assert_eq!(
            group(Slot::SliceNumber, coefficients)
                .serialize(Limits::default())
                .unwrap(),
            expected
        );
    }
}

#[test]
fn unequal_edges_visit_all_four_before_final_byte_failure() {
    // Group + four (component + scalar) = nine inputs and projections.
    // Four eight-byte numbers + three spaces = 35 final bytes. Suffixes
    // add four percentage bytes or eight length-unit bytes, respectively.
    for (slot, expected, bytes) in [
        (Slot::SliceNumber, "0.123456 0.123456 0.123456 0.123456", 35),
        (
            Slot::SlicePercentage,
            "0.123456% 0.123456% 0.123456% 0.123456%",
            39,
        ),
        (Slot::WidthNumber, "0.123456 0.123456 0.123456 0.123456", 35),
        (
            Slot::WidthLength,
            "0.123456px 0.123456px 0.123456px 0.123456px",
            43,
        ),
        (
            Slot::OutsetNumber,
            "0.123456 0.123456 0.123456 0.123456",
            35,
        ),
        (
            Slot::OutsetLength,
            "0.123456px 0.123456px 0.123456px 0.123456px",
            43,
        ),
    ] {
        let value = group(slot, UNEQUAL);
        let before = value.clone();
        for (limits, kind) in [
            (Limits::new(8, 9, bytes), Kind::InputNodeLimit),
            (Limits::new(9, 8, bytes), Kind::ProjectionNodeLimit),
            (Limits::new(9, 9, bytes - 1), Kind::ByteLimit),
        ] {
            let result = value.serialize(limits);
            assert_eq!(value, before);
            assert_eq!(result.unwrap_err().kind(), kind, "{slot:?}");
        }
        assert_eq!(
            value.serialize(Limits::new(9, 9, bytes)).unwrap(),
            expected,
            "{slot:?}"
        );
    }
}

#[test]
fn compressed_edges_still_charge_four_visits_and_exact_scratch_bytes() {
    let value = group(
        Slot::SliceNumber,
        [".123456410", "12345641e-8", "+.12345641", "0.12345641"],
    );
    for (limits, kind) in [
        (Limits::new(8, 9, 8), Kind::InputNodeLimit),
        (Limits::new(9, 8, 8), Kind::ProjectionNodeLimit),
        (Limits::new(9, 9, 7), Kind::ByteLimit),
        // The first component's input is charged before its projection.
        (Limits::new(1, 1, 8), Kind::InputNodeLimit),
    ] {
        assert_eq!(value.serialize(limits).unwrap_err().kind(), kind);
    }
    assert_eq!(value.serialize(Limits::new(9, 9, 8)).unwrap(), "0.123456");
}

#[test]
fn positive_sub_micro_values_are_distinct_from_actual_zero() {
    for slot in SLOTS {
        let expected = match slot {
            Slot::SlicePercentage => "0% 0% 0% 0%",
            Slot::WidthLength | Slot::OutsetLength => "0px 0px 0px 0px",
            _ => "0 0 0 0",
        };
        assert_eq!(
            group(slot, ["0", ".0000001", "0", "-0"])
                .serialize(Limits::default())
                .unwrap(),
            expected,
            "{slot:?}"
        );
    }
}

#[test]
fn arbitrary_negative_exponents_remain_exactly_distinct_after_rounding_to_zero() {
    // These exponent magnitudes exceed i128::MAX; no fixed-width or float
    // conversion can establish the required exact inequality.
    let coefficients = [
        "1e-170141183460469231731687303715884105728",
        "1e-170141183460469231731687303715884105729",
        "1e-170141183460469231731687303715884105730",
        "1e-170141183460469231731687303715884105731",
    ];
    assert_eq!(
        group(Slot::SliceNumber, coefficients)
            .serialize(Limits::default())
            .unwrap(),
        "0 0 0 0"
    );
}

#[test]
fn exponent_compensation_and_signed_actual_zero_still_compress() {
    for coefficients in [
        [
            "1e-170141183460469231731687303715884105728",
            "10e-170141183460469231731687303715884105729",
            "100e-170141183460469231731687303715884105730",
            "1000e-170141183460469231731687303715884105731",
        ],
        [
            "0",
            "-0",
            "+0.000",
            "-0e-170141183460469231731687303715884105728",
        ],
    ] {
        assert_eq!(
            group(Slot::SliceNumber, coefficients)
                .serialize(Limits::new(9, 9, 1))
                .unwrap(),
            "0"
        );
    }
}

#[test]
fn length_zero_normalization_preserves_kind_and_unit_boundaries() {
    let length_percentage = |text| {
        CssBorderImageWidthComponent::LengthPercentage(
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(
                CssComponentValue::try_token(text).unwrap(),
            )
            .unwrap(),
        )
    };
    let value = CssBorderImageWidth::try_new(vec![
        length_percentage("0"),
        length_percentage("0px"),
        length_percentage("-0"),
        length_percentage("+0px"),
    ])
    .unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "0px");
    let value = CssBorderImageWidth::try_new(vec![
        CssBorderImageWidthComponent::Number(
            CssSpecifiedNonNegativeNumber::try_from_component(
                CssComponentValue::try_number("0").unwrap(),
            )
            .unwrap(),
        ),
        length_percentage("0px"),
        length_percentage("0em"),
        length_percentage("0%"),
    ])
    .unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "0 0px 0em 0%");
    let length = |text| {
        CssBorderImageOutsetComponent::Length(
            CssSpecifiedNonNegativeLength::try_from_component(
                CssComponentValue::try_token(text).unwrap(),
            )
            .unwrap(),
        )
    };
    let value = CssBorderImageOutset::try_new(vec![
        length("0"),
        length("0px"),
        length("-0"),
        length("+0px"),
    ])
    .unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "0px");
    let value = CssBorderImageOutset::try_new(vec![
        length("0px"),
        length("0em"),
        length("0px"),
        length("0em"),
    ])
    .unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "0px 0em");
    let value = CssBorderImageSlice::try_new(
        vec![
            CssBorderImageSliceComponent::Number(
                CssSpecifiedNonNegativeNumber::try_from_component(
                    CssComponentValue::try_number("0").unwrap(),
                )
                .unwrap(),
            ),
            CssBorderImageSliceComponent::Percentage(
                CssSpecifiedNonNegativePercentage::try_from_component(
                    CssComponentValue::try_token("0%").unwrap(),
                )
                .unwrap(),
            ),
        ],
        false,
    )
    .unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "0 0%");
}

#[test]
fn parsed_quad_preserves_raw_components_and_origins_after_serialization() {
    let authored = ".12345641 .12345642 .12345643 .12345644";
    let value = image(authored);
    let before = value.clone();
    let slice = Group::Slice(value.slice().unwrap().clone());
    for (literal, expected) in slice.literals().iter().zip(UNEQUAL) {
        assert_eq!(raw(literal), expected);
        let CssValueOrigin::Parsed(origin) = literal.origin() else {
            panic!("parsed origin")
        };
        assert_eq!(origin.source().as_str(), authored);
    }
    let serialized = value.serialize_specified();
    assert_eq!(value, before);
    assert_eq!(serialized.unwrap(), "0.123456 0.123456 0.123456 0.123456");
}

#[test]
fn rounded_near_initial_values_are_retained_in_shorthand() {
    for (authored, expected) in [
        ("100.0000001%", "100%"),
        ("100% / 1.0000001", "100% / 1"),
        ("100% / 1 / .0000001", "100% / / 0"),
        ("100.0000001% / 1.0000001 / .0000001", "100% / 1 / 0"),
    ] {
        let value = image(authored);
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected, "{authored}");
        assert_eq!(value, before);
    }
}

#[test]
fn ordinary_length_compression_folds_unit_case_without_converting_units() {
    for (authored, expected, bytes) in [
        (
            [
                ".12345641PX",
                ".12345641pX",
                ".123456410px",
                "12345641e-8px",
            ],
            "0.123456px",
            10,
        ),
        (["1in", "96px", "1in", "96px"], "1in 96px", 8),
        (["0PX", "0em", "+0px", "-0em"], "0px 0em", 7),
    ] {
        let components: Vec<_> = authored
            .iter()
            .map(|text| CssComponentValue::try_token(text).unwrap())
            .collect();
        let width = CssBorderImageWidth::try_new(
            components
                .iter()
                .cloned()
                .map(|component| {
                    CssBorderImageWidthComponent::LengthPercentage(
                        CssSpecifiedNonNegativeLengthPercentage::try_from_component(component)
                            .unwrap(),
                    )
                })
                .collect(),
        )
        .unwrap();
        let outset = CssBorderImageOutset::try_new(
            components
                .into_iter()
                .map(|component| {
                    CssBorderImageOutsetComponent::Length(
                        CssSpecifiedNonNegativeLength::try_from_component(component).unwrap(),
                    )
                })
                .collect(),
        )
        .unwrap();
        for value in [Group::Width(width), Group::Outset(outset)] {
            let before = value.clone();
            assert_eq!(value.serialize(Limits::new(9, 9, bytes)).unwrap(), expected);
            for (limits, kind) in [
                (Limits::new(8, 9, bytes), Kind::InputNodeLimit),
                (Limits::new(9, 8, bytes), Kind::ProjectionNodeLimit),
                (Limits::new(9, 9, bytes - 1), Kind::ByteLimit),
            ] {
                assert_eq!(value.serialize(limits).unwrap_err().kind(), kind);
                assert_eq!(value, before);
            }
            for (component, authored) in value.literals().iter().zip(authored) {
                assert_eq!(raw(component), authored);
                assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
                let CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) =
                    component.view()
                else {
                    panic!("length token")
                };
                assert!(authored.ends_with(unit));
            }
        }
    }
}
