#![forbid(unsafe_code)]

//! Existing-API regression for Fonts 4 sections 4.1, 6.9 and 13.2.
//! Fixtures use only descriptors and feature-index forms admitted before #759.

#[path = "support/descriptor_phases.rs"]
mod descriptor_phases;
use descriptor_phases::*;

use surgeist_css::{
    CssAuthoredFontFaceDescriptorValue as Authored, CssFontDisplay,
    CssFontFaceDescriptorKind as Kind, CssFontFaceDescriptorValue as Value,
    CssFontFeatureValueKind as BlockKind, CssFontFeatureValuesItem as Item,
    CssFontFeatureValuesRule, CssNormalizedItem, CssRule, CssRuleContextKindRef, CssSheet,
    CssSpecifiedRuleSerializationErrorKind as RuleError,
    CssSpecifiedValueSerializationErrorKind as ValueError,
    CssSpecifiedValueSerializationLimits as Limits, normalize_sheet, parse_sheet,
};

fn sheet(source: &str) -> CssSheet {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().clone()
}

fn features(rule: &CssRule) -> &CssFontFeatureValuesRule {
    let CssRule::FontFeatureValues(rule) = rule else {
        panic!("expected font-feature-values")
    };
    rule
}

type BlockSnapshot = (BlockKind, Vec<(String, Vec<String>)>);

fn blocks(rule: &CssFontFeatureValuesRule) -> Vec<BlockSnapshot> {
    rule.items()
        .iter()
        .filter_map(|item| match item {
            Item::Block(block) => Some((
                block.kind(),
                block
                    .definitions()
                    .iter()
                    .map(|definition| {
                        (
                            definition.name().as_str().to_owned(),
                            definition
                                .ordinary_indexes()
                                .iter()
                                .map(|index| index.as_decimal_str().to_owned())
                                .collect(),
                        )
                    })
                    .collect(),
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn font_face_emits_effective_descriptors_in_canonical_relative_order() {
    let input = sheet(
        r#"@font-face {
        font-style: italic; font-weight: 300; font-width: 75%;
        font-feature-settings: "liga" 0; unicode-range: U+20-7E;
        src: url("font.woff2"); font-family: Demo; font-weight: 600;
    }"#,
    );
    let before = input.clone();
    let output = input.rules()[0].to_specified_css().unwrap();
    let reparsed = sheet(&output);
    let [CssRule::FontFace(face)] = reparsed.rules() else {
        panic!("font-face")
    };
    assert_eq!(
        face.descriptors()
            .occurrences()
            .map(|record| record.value().kind())
            .collect::<Vec<_>>(),
        [
            Kind::FontFamily,
            Kind::Src,
            Kind::UnicodeRange,
            Kind::FontFeatureSettings,
            Kind::FontWidth,
            Kind::FontWeight,
            Kind::FontStyle
        ]
    );
    let Some(record) = face.descriptors().effective(Kind::FontWeight) else {
        panic!("weight")
    };
    let Authored::Ordinary(Value::FontWeight(weight)) = record.value() else {
        panic!("weight")
    };
    assert_eq!(weight.serialize_specified().unwrap(), "600");
    assert_eq!(input, before);
}

#[test]
fn empty_and_partial_font_faces_emit_only_present_descriptors() {
    for source in ["@font-face {}", "@font-face { font-weight: 500; }"] {
        let input = sheet(source);
        let before = input.clone();
        let output = input.rules()[0].to_specified_css().unwrap();
        let reparsed = sheet(&output);
        let [CssRule::FontFace(face)] = reparsed.rules() else {
            panic!("font-face")
        };
        let expected = if source == "@font-face {}" {
            vec![]
        } else {
            vec![Kind::FontWeight]
        };
        assert_eq!(
            face.descriptors()
                .occurrences()
                .map(|record| record.value().kind())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(input, before);
    }
}

#[test]
fn all_seven_feature_blocks_and_outer_display_round_trip() {
    let input = sheet(
        r#"@font-feature-values Demo, "Other Font" {
        font-display: swap;
        @stylistic { fancy: 1; } @historical-forms { old: 1 2; }
        @styleset { joined: 1 2; } @character-variant { open: 1 2; }
        @swash { flowing: 2; } @ornaments { fleur: 3; } @annotation { circled: 4; }
    }"#,
    );
    let before = input.clone();
    let output = input.rules()[0].to_specified_css().unwrap();
    let reparsed = sheet(&output);
    let original = features(&input.rules()[0]);
    let emitted = features(&reparsed.rules()[0]);
    assert_eq!(emitted.families(), original.families());
    assert_eq!(blocks(emitted), blocks(original));
    let display = emitted
        .items()
        .iter()
        .filter_map(|item| match item {
            Item::FontDisplay(display) => Some(display.value().ordinary_display()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(display, [CssFontDisplay::Swap]);
    assert_eq!(input, before);
}

#[test]
fn feature_blocks_merge_last_names_in_last_occurrence_order() {
    let input = sheet(
        "@font-feature-values Demo {
        font-display: swap;
        @swash { pretty: 0; cool: 2; Pretty: 3; }
        @stylistic { fancy: 1; }
        @swash { pretty: 1; }
        font-display: optional;
    }",
    );
    let before = input.clone();
    let output = input.rules()[0].to_specified_css().unwrap();
    let reparsed = sheet(&output);
    let emitted = features(&reparsed.rules()[0]);
    assert_eq!(
        blocks(emitted),
        vec![
            (
                BlockKind::Swash,
                vec![
                    ("cool".into(), vec!["2".into()]),
                    ("Pretty".into(), vec!["3".into()]),
                    ("pretty".into(), vec!["1".into()])
                ]
            ),
            (
                BlockKind::Stylistic,
                vec![("fancy".into(), vec!["1".into()])]
            ),
        ]
    );
    let display = emitted
        .items()
        .iter()
        .filter_map(|item| match item {
            Item::FontDisplay(display) => Some(display.value().ordinary_display()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(display, [CssFontDisplay::Optional]);
    assert_eq!(input, before);
}

#[test]
fn sheet_composes_fonts_in_source_order_with_atomic_output_byte_limits() {
    let input = sheet(
        "@font-face { font-family: Demo; font-weight: 400; }
        @font-feature-values Demo { @swash { pretty: 1; } }
        @font-face { font-family: Other; }",
    );
    let before = input.clone();
    let output = input.to_specified_css().unwrap();
    let reparsed = sheet(&output);
    assert!(matches!(
        reparsed.rules(),
        [
            CssRule::FontFace(_),
            CssRule::FontFeatureValues(_),
            CssRule::FontFace(_)
        ]
    ));
    let pieces = input
        .rules()
        .iter()
        .map(|rule| rule.to_specified_css().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(output, pieces.join("\n"));
    assert_eq!(
        input
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, output.len()))
            .unwrap(),
        output
    );
    let error = input
        .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, output.len() - 1))
        .unwrap_err();
    assert_eq!(error.kind(), RuleError::Resource(ValueError::ByteLimit));
    assert_eq!(error.rule_index(), Some(2));
    assert_eq!(input, before);
}

#[test]
fn authored_duplicates_provenance_and_normalized_payloads_are_retained() {
    let input = sheet(
        "@font-face { font-weight: 300; font-weight: 600; }
        @font-feature-values Demo { @swash { pretty: 0; pretty: 1; }
            @swash { pretty: 2; } }",
    );
    let [CssRule::FontFace(face), CssRule::FontFeatureValues(feature)] = input.rules() else {
        panic!("ordered font rules")
    };
    assert_eq!(face.descriptors().occurrences().len(), 2);
    assert!(
        face.descriptors()
            .occurrences()
            .all(|record| record.position().is_some())
    );
    assert_eq!(feature.items().len(), 2);
    assert_eq!(blocks(feature)[0].1.len(), 2);
    assert!(feature.items().iter().all(|item| matches!(item,
        Item::Block(block) if block.position().is_some()
            && block.definitions().iter().all(|definition| definition.position().is_some()))));
    let normalized = normalize_sheet(&input).unwrap();
    let contexts = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => Some(context),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(contexts.len(), 2);
    assert!(matches!(contexts[0].kind(), CssRuleContextKindRef::FontFace(value) if value == face));
    assert!(
        matches!(contexts[1].kind(), CssRuleContextKindRef::FontFeatureValues(value) if value == feature)
    );
}

#[test]
fn generic_group_composes_the_existing_font_rule_provider() {
    let input = sheet("@media all { @font-face { font-family: Demo; } }");
    let before = input.clone();
    assert_eq!(
        input.to_specified_css().unwrap(),
        "@media all { @font-face { font-family: Demo; } }"
    );
    assert_eq!(input, before);
}
