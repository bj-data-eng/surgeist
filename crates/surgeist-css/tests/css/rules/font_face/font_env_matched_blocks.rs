//! CSS Env 1 WD 2025-09-23 §3 assumes the entire descriptor grammar valid
//! when its env functions are syntactically valid. Syntax 3 CRD 2026-10-01 §7.2
//! permits matched blocks in declaration-value, while excluding unmatched
//! closers and root semicolons/bangs. These tests use the public string and component fronts.

use crate::css::support::rules::{ExpectedRejection, descriptor_rejected_at, invalid_descriptor};
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValues, CssValueOrigin};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::rule::font_face::CssAuthoredFontFaceDescriptorValue as FaceValue;
use surgeist_css::rule::font_face::CssFontFaceDescriptorKind as FaceKind;
use surgeist_css::rule::font_face::{CssAuthoredFontFaceDescriptorValue, CssFontFaceDescriptors};
use surgeist_css::rule::font_palette_values::CssFontPaletteDescriptorKind as PaletteKind;
use surgeist_css::rule::font_palette_values::CssFontPaletteDescriptorValue as PaletteValue;
use surgeist_css::rule::font_palette_values::{
    CssFontPaletteDescriptor, CssFontPaletteDescriptorValue, CssFontPaletteDescriptorValueRef,
};
use surgeist_css::rule::{CssBlockContext, CssBlockFragment, CssRule};

const FACE_VALUES: [&str; 4] = [
    "env(width){}",
    "env(choice) {}",
    "{hello} env(choice)",
    "env(choice) {inner {deep}}",
];
const PALETTE_VALUES: [&str; 4] = [
    "env(x) {}",
    "env(choice) {}",
    "{hello} env(choice)",
    "env(choice) {inner {deep}}",
];
const PALETTE_KINDS: [PaletteKind; 3] = [
    PaletteKind::FontFamily,
    PaletteKind::BasePalette,
    PaletteKind::OverrideColors,
];

fn retained(components: &CssComponentValues, value: &str, original_source: &str) {
    assert_eq!(
        components
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        value
    );
    let mut first_source = None;
    for component in components.items() {
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("original parsed components must retain parsed provenance");
        };
        assert_eq!(origin.source().as_str(), original_source);
        if let Some(first) = first_source {
            assert!(origin.source().same_snapshot(first));
        } else {
            first_source = Some(origin.source());
        }
    }
}

fn pending_face(value: &FaceValue, text: &str, original_source: &str) {
    assert_eq!(value.kind(), FaceKind::FontWidth);
    let FaceValue::Pending(pending) = value else {
        panic!("valid env defers the complete font-width grammar");
    };
    retained(pending.components(), text, original_source);
}

fn pending_palette(value: &PaletteValue, kind: PaletteKind, text: &str, source: &str) {
    assert_eq!(value.kind(), kind);
    assert!(matches!(
        value.view(),
        CssFontPaletteDescriptorValueRef::Pending(_)
    ));
    retained(value.components(), text, source);
}

#[test]
fn raw_font_face_width_defers_matched_root_blocks_and_retains_original_components() {
    for value in FACE_VALUES {
        let report = CssAuthoredFontFaceDescriptorValue::parse(
            &CssParser::default(),
            value,
            FaceKind::FontWidth,
        );
        assert!(report.is_clean(), "{value}: {report:?}");
        pending_face(report.syntax().as_ref().unwrap(), value, value);
    }
}

#[test]
fn checked_font_face_width_defers_matched_root_blocks_without_losing_provenance() {
    for value in FACE_VALUES {
        let components = CssComponentValues::parse(&CssParser::default(), value).unwrap();
        let checked = FaceValue::try_from_components(
            FaceKind::FontWidth,
            components.clone(),
            Default::default(),
        )
        .unwrap();
        pending_face(&checked, value, value);
        let FaceValue::Pending(pending) = checked else {
            unreachable!()
        };
        assert_eq!(pending.components(), &components);
    }
}

// Syntax 3 CRD 2026-10-01 §5.5.6 step 8: as a declaration, a value that holds
// a block beside another value returns nothing, whatever env() could defer.
// The blocks stay data in the value fronts above.
//
// §5.5.5 then reads the same text as a qualified rule that ends at its block: a
// value that ends with the block is one dropped unit, and a value with
// something after the block leaves that something as a unit of its own, which
// a `;` stops.
fn dropped_for_its_block<T: std::fmt::Debug>(
    report: &surgeist_css::CssParseReport<T>,
    source: &str,
    value: &str,
) {
    let units = if value.ends_with('}') { 1 } else { 2 };
    assert_eq!(report.diagnostics().len(), units, "{source}: {report:?}");
    assert_eq!(
        report.diagnostics()[0].error().code(),
        CssErrorCode::UnexpectedToken,
        "{source}"
    );
}

#[test]
fn font_face_body_drops_width_with_a_block_beside_a_value_and_keeps_the_ordinary_neighbor() {
    for value in FACE_VALUES {
        let source = format!("{{font-width:{value};font-display:swap;}}");
        let report = CssBlockFragment::<CssFontFaceDescriptors>::parse(
            &CssParser::default(),
            &source,
            &CssBlockContext::default(),
        );
        dropped_for_its_block(&report, &source, value);
        let descriptors = report.syntax().as_ref().unwrap().body();
        assert_eq!(descriptors.occurrences().len(), 1);
        assert!(descriptors.effective(FaceKind::FontWidth).is_none());
        assert!(descriptors.effective(FaceKind::FontDisplay).is_some());
    }
}

#[test]
fn font_face_rule_drops_width_with_a_block_beside_a_value_and_keeps_the_following_rule() {
    for value in FACE_VALUES {
        let source =
            format!("@font-face{{font-width:{value};font-display:swap;}} .after{{color:red;}}");
        let report = CssParser::default().parse_sheet(&source);
        dropped_for_its_block(&report, &source, value);
        let [CssRule::FontFace(face), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("font-face and following rule must survive");
        };
        assert_eq!(face.descriptors().occurrences().len(), 1);
        assert!(face.descriptors().effective(FaceKind::FontWidth).is_none());
    }
}

#[test]
fn raw_palette_descriptors_defer_matched_root_blocks_and_retain_original_components() {
    for kind in PALETTE_KINDS {
        for value in PALETTE_VALUES {
            let report = CssFontPaletteDescriptorValue::parse(&CssParser::default(), value, kind);
            assert!(report.is_clean(), "{}:{value}: {report:?}", kind.as_str());
            pending_palette(report.syntax().as_ref().unwrap(), kind, value, value);
        }
    }
}

#[test]
fn checked_palette_descriptors_defer_matched_root_blocks_without_losing_provenance() {
    for kind in PALETTE_KINDS {
        for value in PALETTE_VALUES {
            let components = CssComponentValues::parse(&CssParser::default(), value).unwrap();
            let checked =
                PaletteValue::try_new(kind, components.clone(), Default::default()).unwrap();
            pending_palette(&checked, kind, value, value);
            assert_eq!(checked.components(), &components);
        }
    }
}

#[test]
fn palette_body_drops_a_block_beside_a_value_and_keeps_the_real_family_neighbor() {
    for kind in PALETTE_KINDS {
        for value in PALETTE_VALUES {
            let source = format!("{{font-family:Demo;{}:{value};}}", kind.as_str());
            let report = CssBlockFragment::<Vec<CssFontPaletteDescriptor>>::parse(
                &CssParser::default(),
                &source,
                &CssBlockContext::default(),
            );
            dropped_for_its_block(&report, &source, value);
            let descriptors = report.syntax().as_ref().unwrap().body();
            assert_eq!(descriptors.len(), 1);
        }
    }
}

#[test]
fn palette_rule_drops_a_block_beside_a_value_and_keeps_the_following_rule() {
    for kind in PALETTE_KINDS {
        for value in PALETTE_VALUES {
            let source = format!(
                "@font-palette-values --x{{font-family:Demo;{}:{value};}} .after{{color:red;}}",
                kind.as_str()
            );
            let report = CssParser::default().parse_sheet(&source);
            dropped_for_its_block(&report, &source, value);
            let [CssRule::FontPaletteValues(palette), CssRule::Style(_)] = report.syntax().rules()
            else {
                panic!("palette and following rule must survive");
            };
            assert_eq!(palette.descriptors().len(), 1);
        }
    }
}

#[test]
fn malformed_env_and_ordinary_root_blocks_remain_invalid_in_raw_and_checked_fronts() {
    // A root block opens no production of either ordinary grammar. Env 1
    // §3 defers only a syntactically valid env(), so a malformed one leaves
    // the ordinary grammar, which rejects the env() function token itself.
    for (value, token) in [
        ("{}", "{"),
        ("{hello}", "{"),
        ("env() {}", "env("),
        ("env(123) {}", "env("),
        ("env(choice) env() {}", "env("),
    ] {
        let at = |expected: ExpectedRejection| expected.at(token);
        let face_rejection = at(invalid_descriptor("font-face", "font-width"));
        let face = CssAuthoredFontFaceDescriptorValue::parse(
            &CssParser::default(),
            value,
            FaceKind::FontWidth,
        );
        assert!(face.syntax().is_none(), "{value}: {face:?}");
        face_rejection.assert_first(&face);
        let components = CssComponentValues::parse(&CssParser::default(), value).unwrap();
        let error = FaceValue::try_from_components(
            FaceKind::FontWidth,
            components.clone(),
            Default::default(),
        )
        .unwrap_err();
        let Some(detail) = error.diagnostic() else {
            panic!("{value}: grammar failure: {error:?}")
        };
        face_rejection.assert_kind(detail);
        // The palette fronts name their at-rule and descriptor too.
        for kind in PALETTE_KINDS {
            let palette_rejection = at(invalid_descriptor("font-palette-values", kind.as_str()));
            let palette = CssFontPaletteDescriptorValue::parse(&CssParser::default(), value, kind);
            assert!(
                palette.syntax().is_none(),
                "{}:{value}: {palette:?}",
                kind.as_str()
            );
            palette_rejection.assert_first(&palette);
            let error =
                PaletteValue::try_new(kind, components.clone(), Default::default()).unwrap_err();
            let Some(detail) = error.diagnostic() else {
                panic!("{}:{value}: grammar failure: {error:?}", kind.as_str())
            };
            palette_rejection.assert_kind(detail);
        }
    }
}

#[test]
fn root_bangs_semicolons_and_unmatched_closers_are_not_rescued_by_valid_env() {
    for (value, token) in [
        ("env(choice)!important", "!"),
        ("env(choice);", ";"),
        ("env(choice)}", "}"),
        ("env(choice)]", "]"),
        ("env(choice))", ")"),
    ] {
        let face_rejection = descriptor_rejected_at("font-face", "font-width", token);
        let face = CssAuthoredFontFaceDescriptorValue::parse(
            &CssParser::default(),
            value,
            FaceKind::FontWidth,
        );
        assert!(face.syntax().is_none(), "{value}: {face:?}");
        face_rejection.assert_first(&face);
        for kind in PALETTE_KINDS {
            let palette = CssFontPaletteDescriptorValue::parse(&CssParser::default(), value, kind);
            assert!(
                palette.syntax().is_none(),
                "{}:{value}: {palette:?}",
                kind.as_str()
            );
            descriptor_rejected_at("font-palette-values", kind.as_str(), token)
                .assert_first(&palette);
        }
        // Component admission may itself reject unmatched closers. If it admits
        // the token stream, the descriptor constructor must reject it.
        if let Ok(components) = CssComponentValues::parse(&CssParser::default(), value) {
            let error = FaceValue::try_from_components(
                FaceKind::FontWidth,
                components.clone(),
                Default::default(),
            )
            .unwrap_err();
            let Some(detail) = error.diagnostic() else {
                panic!("{value}: grammar failure: {error:?}")
            };
            face_rejection.assert_kind(detail);
            for kind in PALETTE_KINDS {
                let error = PaletteValue::try_new(kind, components.clone(), Default::default())
                    .unwrap_err();
                let Some(detail) = error.diagnostic() else {
                    panic!("{}:{value}: grammar failure: {error:?}", kind.as_str())
                };
                descriptor_rejected_at("font-palette-values", kind.as_str(), token)
                    .assert_kind(detail);
            }
        }
    }
}

#[test]
fn font_face_malformed_env_and_ordinary_blocks_drop_only_the_bad_descriptor() {
    for value in ["{}", "{hello}", "env() {}", "env(choice) env() {}"] {
        let body = format!("{{font-width:{value};font-display:swap;}}");
        let report = CssBlockFragment::<CssFontFaceDescriptors>::parse(
            &CssParser::default(),
            &body,
            &CssBlockContext::default(),
        );
        assert!(!report.is_clean(), "{body}: {report:?}");
        let descriptors = report.syntax().as_ref().unwrap().body();
        assert!(descriptors.effective(FaceKind::FontWidth).is_none());
        assert!(descriptors.effective(FaceKind::FontDisplay).is_some());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::DropDescriptor)
        );
        let source = format!("@font-face{body} .after{{color:red;}}");
        let sheet = CssParser::default().parse_sheet(&source);
        assert!(!sheet.is_clean(), "{source}: {sheet:?}");
        let [CssRule::FontFace(face), CssRule::Style(_)] = sheet.syntax().rules() else {
            panic!("valid siblings survive")
        };
        assert!(face.descriptors().effective(FaceKind::FontWidth).is_none());
        assert!(
            face.descriptors()
                .effective(FaceKind::FontDisplay)
                .is_some()
        );
    }
}

#[test]
fn palette_malformed_env_and_ordinary_blocks_drop_only_the_bad_descriptor() {
    for kind in PALETTE_KINDS {
        for value in ["{}", "{hello}", "env() {}", "env(choice) env() {}"] {
            let body = format!("{{font-family:Demo;{}:{value};}}", kind.as_str());
            let report = CssBlockFragment::<Vec<CssFontPaletteDescriptor>>::parse(
                &CssParser::default(),
                &body,
                &CssBlockContext::default(),
            );
            assert!(!report.is_clean(), "{body}: {report:?}");
            assert_eq!(report.syntax().as_ref().unwrap().body().len(), 1);
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|d| d.action() == CssRecoveryAction::DropDescriptor)
            );
            let source = format!("@font-palette-values --x{body} .after{{color:red;}}");
            let sheet = CssParser::default().parse_sheet(&source);
            assert!(!sheet.is_clean(), "{source}: {sheet:?}");
            let [CssRule::FontPaletteValues(palette), CssRule::Style(_)] = sheet.syntax().rules()
            else {
                panic!("valid siblings survive")
            };
            assert_eq!(palette.descriptors().len(), 1);
            assert_eq!(
                palette.descriptors()[0].value().kind(),
                PaletteKind::FontFamily
            );
        }
    }
}
