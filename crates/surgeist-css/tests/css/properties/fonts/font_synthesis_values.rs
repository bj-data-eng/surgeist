#![forbid(unsafe_code)]

//! Functional coverage for new checked payloads and emission methods. The oracle
//! is Fonts 4 WD 2026-09-07 §§2.8.1–2.8.5, including canonical grammar order and
//! the four-column expansion table. These APIs had no preimplementation RED.
use surgeist_css::*;

const CAPABILITIES: [&str; 4] = ["weight", "style", "small-caps", "position"];

fn declaration(name: &str, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{text}!important"));
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    report.syntax()[0].clone()
}

fn specified(
    declaration: &CssDeclaration,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String, CssSpecifiedValueSerializationError> {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::FontSynthesis(value) => {
            value.synthesis().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::FontSynthesisWeight(value) => {
            value.weight().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::FontSynthesisStyle(value) => {
            value.style().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::FontSynthesisSmallCaps(value) => {
            value.small_caps().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::FontSynthesisPosition(value) => {
            value.position().serialize_specified_with_limits(limits)
        }
        _ => panic!("synthesis family"),
    }
}

#[test]
fn checked_four_member_sets_reject_empty_and_emit_every_semantic_combination() {
    assert!(CssFontSynthesisValues::try_new(false, false, false, false).is_none());
    assert_eq!(
        CssFontSynthesis::None.serialize_specified().unwrap(),
        "none"
    );
    for mask in 1u8..16 {
        let selected = std::array::from_fn::<_, 4, _>(|index| mask & (1 << index) != 0);
        let [weight, style, small_caps, position] = selected;
        let values = CssFontSynthesisValues::try_new(weight, style, small_caps, position).unwrap();
        assert_eq!(
            [
                values.weight(),
                values.style(),
                values.small_caps(),
                values.position()
            ],
            selected
        );
        let expected = CAPABILITIES
            .iter()
            .zip(selected)
            .filter_map(|(name, selected)| selected.then_some(*name))
            .collect::<Vec<_>>()
            .join(" ");
        let value = CssFontSynthesis::Values(values);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        // A reversed authored order must normalize to the independent grammar order.
        let reversed = expected
            .split_whitespace()
            .rev()
            .collect::<Vec<_>>()
            .join(" ");
        let source = declaration("font-synthesis", &reversed);
        let CssKnownPropertyValueRef::FontSynthesis(parsed) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("synthesis")
        };
        assert_eq!(parsed.synthesis(), &value);
        assert_eq!(parsed.synthesis().serialize_specified().unwrap(), expected);
        let reparsed = declaration("font-synthesis", &expected);
        let CssKnownPropertyValueRef::FontSynthesis(roundtrip) =
            reparsed.known().unwrap().property_value().unwrap()
        else {
            panic!("synthesis")
        };
        assert_eq!(roundtrip.synthesis(), &value);
    }
}

#[test]
fn longhand_payloads_preserve_distinct_choices_and_emit_canonical_keywords() {
    assert_eq!(
        CssFontSynthesisWeight::Auto.serialize_specified().unwrap(),
        "auto"
    );
    assert_eq!(
        CssFontSynthesisWeight::None.serialize_specified().unwrap(),
        "none"
    );
    assert_eq!(
        CssFontSynthesisStyle::Auto.serialize_specified().unwrap(),
        "auto"
    );
    assert_eq!(
        CssFontSynthesisStyle::None.serialize_specified().unwrap(),
        "none"
    );
    assert_eq!(
        CssFontSynthesisStyle::ObliqueOnly
            .serialize_specified()
            .unwrap(),
        "oblique-only"
    );
    assert_eq!(
        CssFontSynthesisSmallCaps::Auto
            .serialize_specified()
            .unwrap(),
        "auto"
    );
    assert_eq!(
        CssFontSynthesisSmallCaps::None
            .serialize_specified()
            .unwrap(),
        "none"
    );
    assert_eq!(
        CssFontSynthesisPosition::Auto
            .serialize_specified()
            .unwrap(),
        "auto"
    );
    assert_eq!(
        CssFontSynthesisPosition::None
            .serialize_specified()
            .unwrap(),
        "none"
    );

    for (name, text, expected) in [
        ("font-synthesis-weight", "AuTo", "auto"),
        ("font-synthesis-weight", "NoNe", "none"),
        ("font-synthesis-style", "AuTo", "auto"),
        ("font-synthesis-style", "NoNe", "none"),
        ("font-synthesis-style", "OBLIQUE-ONLY", "oblique-only"),
        ("font-synthesis-small-caps", "AuTo", "auto"),
        ("font-synthesis-small-caps", "NoNe", "none"),
        ("font-synthesis-position", "AuTo", "auto"),
        ("font-synthesis-position", "NoNe", "none"),
    ] {
        let source = declaration(name, text);
        let output = specified(&source, CssSpecifiedValueSerializationLimits::default()).unwrap();
        assert_eq!(output, expected);
        let reparsed = declaration(name, &output);
        let CssExpansion::Contributions(CssContributions::Longhands(before)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("longhand")
        };
        let CssExpansion::Contributions(CssContributions::Longhands(after)) =
            expand_declaration(&reparsed).unwrap()
        else {
            panic!("longhand")
        };
        assert_eq!(
            before.items()[0].ordinary_value(),
            after.items()[0].ordinary_value()
        );
    }
}

#[test]
fn escaped_keywords_decode_without_replacing_authored_origins() {
    for (name, text, expected) in [
        (
            "font-synthesis",
            "p\\6f sition small\\2d caps w\\65 ight st\\79 le",
            "weight style small-caps position",
        ),
        ("font-synthesis-style", "oblique\\2d only", "oblique-only"),
        ("font-synthesis-weight", "a\\75 to", "auto"),
        ("font-synthesis-small-caps", "n\\6f ne", "none"),
        ("font-synthesis-position", "a\\75 to", "auto"),
    ] {
        let source = declaration(name, text);
        let origin = source.parsed_value().unwrap();
        assert_eq!(origin.span().start().byte_offset().value(), name.len() + 1);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            name.len() + 1 + text.len()
        );
        assert_eq!(
            specified(&source, CssSpecifiedValueSerializationLimits::default()).unwrap(),
            expected
        );
    }
}

#[test]
fn exact_input_projection_and_byte_budgets_are_cumulative_and_failure_is_atomic() {
    for (name, text, expected, nodes) in [
        (
            "font-synthesis",
            "POSITION SMALL-CAPS STYLE WEIGHT",
            "weight style small-caps position",
            4,
        ),
        (
            "font-synthesis",
            "position small-caps",
            "small-caps position",
            2,
        ),
        ("font-synthesis", "none", "none", 1),
        ("font-synthesis-weight", "AUTO", "auto", 1),
        ("font-synthesis-style", "OBLIQUE-ONLY", "oblique-only", 1),
        ("font-synthesis-small-caps", "NONE", "none", 1),
        ("font-synthesis-position", "AUTO", "auto", 1),
    ] {
        let source = declaration(name, text);
        let before = source.clone();
        let exact = CssSpecifiedValueSerializationLimits::new(nodes, nodes, expected.len());
        assert_eq!(specified(&source, exact).unwrap(), expected);
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(nodes - 1, nodes, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, nodes - 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, nodes, expected.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(0, nodes, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, 0, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, nodes, 0),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                specified(&source, limits).unwrap_err().kind(),
                kind,
                "{name}"
            );
            assert!(source.same_occurrence(&before));
            assert_eq!(source.value_components(), before.value_components());
            assert_eq!(specified(&source, exact).unwrap(), expected);
        }
    }
}

#[test]
fn normalization_counts_all_four_members_and_returns_the_exact_late_failure() {
    let text = concat!(
        ".a{font-synthesis:weight!important;font-synthesis:none;",
        "font-synthesis-style:oblique-only;font-synthesis-weight:var(--synthesis);color:red}"
    );
    let report = parse_sheet(text);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let exact = CssNormalizationLimits::try_new(0, 1, 5, 11).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 5);
    for (index, (value, members)) in declarations.iter().zip([4, 4, 1, 1, 1]).enumerate() {
        assert_eq!(value.order(), index);
        let count = match value.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert!(
                    values
                        .items()
                        .iter()
                        .all(|item| item.source().same_occurrence(value.source()))
                );
                values.items().len()
            }
            CssExpansion::Pending(pending) => {
                assert!(pending.source().same_occurrence(value.source()));
                1
            }
            _ => panic!("synthesis or color contribution"),
        };
        assert_eq!(count, members);
    }
    let color = declarations[4].source().clone();
    for (limits, resource, limit) in [
        (
            CssNormalizationLimits::try_new(0, 1, 5, 10).unwrap(),
            CssNormalizationResource::Contributions,
            10,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 4, 11).unwrap(),
            CssNormalizationResource::Declarations,
            4,
        ),
    ] {
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit }
        );
        assert_eq!(error.declaration_order(), Some(4));
        assert!(error.declaration().unwrap().same_occurrence(&color));
        assert_eq!(error.position(), color.position());
        let after = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
        let after_color = after
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .next_back()
            .unwrap();
        assert!(after_color.source().same_occurrence(&color));
    }
}

#[test]
fn synthesis_support_rows_use_the_selected_fonts4_source_and_complete_productions() {
    for name in [
        "font-synthesis",
        "font-synthesis-weight",
        "font-synthesis-style",
        "font-synthesis-small-caps",
        "font-synthesis-position",
    ] {
        let metadata = property_support_metadata(name).unwrap();
        let feature = metadata.feature();
        assert_eq!(feature.source().id().as_str(), "I-FONTS4-20260907");
        assert_eq!(feature.production(), format!("#propdef-{name}"));
        assert_eq!(feature.status(), CssSupportStatus::Complete);
        assert_eq!(feature.supported_subset(), None);
        assert_eq!(feature.unsupported_remainder(), None);
        assert_eq!(feature.id().as_str(), format!("official.property.{name}"));
        assert_eq!(metadata.canonical_name(), name);
    }
}
