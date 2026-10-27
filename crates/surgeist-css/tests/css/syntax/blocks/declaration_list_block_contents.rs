//! Selected Syntax 3 (2026-10-01) §5.5.4 consumes every block with §5.5.5
//! "consume a block's contents". A failed declaration is reconsumed as a
//! qualified rule whose stop token is `;`, and an at-keyword starts a nested
//! at-rule. §7.1 makes qualified rules and at-rules automatically invalid in a
//! `<declaration-list>`, so the owning rule drops each such unit on its own
//! and keeps the declarations around it; the rule itself survives.
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block-contents
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#block-contents
use crate::css::support::authored_syntax::assert_strict_rejection;
use surgeist_css::CssParser;
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::rule::font_face::CssFontFaceDescriptorKind as Kind;
use surgeist_css::rule::font_feature_values::CssFontFeatureValuesItem;
use surgeist_css::rule::{CssRule, CssSheet};

fn sheet(source: &str, actions: &[CssRecoveryAction]) -> CssSheet {
    let report = CssParser::default().parse_sheet(source);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        actions,
        "{source}: {report:?}"
    );
    assert_strict_rejection(
        CssParser::default()
            .parse_sheet(source)
            .into_validation_result(),
        report.diagnostics(),
    );
    report.syntax().clone()
}

#[test]
fn font_face_drops_nested_rules_and_keeps_surrounding_descriptors() {
    for (source, action) in [
        (
            "@font-face{.x{} font-family:a; src:url(b)}",
            CssRecoveryAction::DropQualifiedRule,
        ),
        // An ident-started unit is first tried as a descriptor.
        (
            "@font-face{font-family:a; b{} src:url(b)}",
            CssRecoveryAction::DropDescriptor,
        ),
        (
            "@font-face{@foo{} font-family:a; src:url(b)}",
            CssRecoveryAction::DropAtRule,
        ),
    ] {
        let sheet = sheet(source, &[action]);
        let [CssRule::FontFace(face)] = sheet.rules() else {
            panic!("{source}: {sheet:?}")
        };
        assert!(face.descriptors().effective(Kind::FontFamily).is_some());
        assert!(face.descriptors().effective(Kind::Src).is_some());
    }
}

#[test]
fn keyframe_blocks_drop_nested_rules_and_keep_surrounding_declarations() {
    // `.x{}` starts with a delimiter, so §5.5.5 consumes it directly as a
    // nested qualified rule. `color:red {} opacity:0` is first tried as a
    // declaration; §5.5.6 returns nothing because a top-level {}-block sits
    // beside other value tokens, so the mark is restored and `color:red {}` is
    // reconsumed as a qualified rule that ends after its block. `opacity:0`
    // and the final `color:red` are then ordinary declarations. The dropped
    // ident-started unit keeps the `DropDeclaration` label of the declaration
    // attempt that failed first, as other ident-started body units do; its
    // span is exactly the reconsumed qualified rule.
    for (source, action, unit, expected) in [
        (
            "@keyframes k{from{.x{} color:red}}",
            CssRecoveryAction::DropQualifiedRule,
            ".x{}",
            &[("color", "red")][..],
        ),
        (
            "@keyframes k{from{color:red {} opacity:0; color:red}}",
            CssRecoveryAction::DropDeclaration,
            "color:red {}",
            &[("opacity", "0"), ("color", "red")][..],
        ),
    ] {
        let report = CssParser::default().parse_sheet(source);
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: {report:?}")
        };
        assert_eq!(diagnostic.action(), action, "{source}");
        let start = source.find(unit).unwrap();
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            start,
            "{source}"
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + unit.len(),
            "{source}"
        );
        assert_strict_rejection(
            CssParser::default()
                .parse_sheet(source)
                .into_validation_result(),
            report.diagnostics(),
        );
        let [CssRule::Keyframes(keyframes)] = report.syntax().rules() else {
            panic!("{source}: {report:?}")
        };
        let [block] = keyframes.blocks() else {
            panic!("{source}: one keyframe block: {keyframes:?}")
        };
        let actual: Vec<_> = block
            .declarations()
            .iter()
            .map(|declaration| {
                (
                    declaration.known().unwrap().property().canonical_name(),
                    declaration
                        .source()
                        .value_components()
                        .serialize_tokens(Default::default())
                        .unwrap()
                        .as_css()
                        .to_owned(),
                )
            })
            .collect();
        let expected: Vec<_> = expected
            .iter()
            .map(|(name, value)| (*name, (*value).to_owned()))
            .collect();
        assert_eq!(actual, expected, "{source}");
    }
}

#[test]
fn descriptor_rules_drop_nested_qualified_rules_and_keep_descriptors() {
    let sheet_value = sheet(
        "@font-palette-values --p{.y{} font-family:a}",
        &[CssRecoveryAction::DropQualifiedRule],
    );
    let [CssRule::FontPaletteValues(palette)] = sheet_value.rules() else {
        panic!("{sheet_value:?}")
    };
    assert_eq!(palette.descriptors().len(), 1);

    let sheet_value = sheet(
        "@color-profile --p{.y{} src:url(a)}",
        &[CssRecoveryAction::DropQualifiedRule],
    );
    let [CssRule::ColorProfile(profile)] = sheet_value.rules() else {
        panic!("{sheet_value:?}")
    };
    assert_eq!(profile.descriptors().len(), 1);

    let sheet_value = sheet(
        "@font-feature-values f{@swash{.x{} a:1}}",
        &[CssRecoveryAction::DropQualifiedRule],
    );
    let [CssRule::FontFeatureValues(features)] = sheet_value.rules() else {
        panic!("{sheet_value:?}")
    };
    let [CssFontFeatureValuesItem::Block(block)] = features.items() else {
        panic!("{features:?}")
    };
    assert_eq!(block.definitions().len(), 1);
}
