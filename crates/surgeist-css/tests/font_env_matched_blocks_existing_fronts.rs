#![forbid(unsafe_code)]

//! CSS Env 1 WD 2025-09-23 §3 assumes the entire descriptor grammar valid
//! when its env functions are syntactically valid. Syntax 3 CRD 2021-12-24 §8.2
//! permits matched blocks in declaration-value, while excluding unmatched
//! closers and root semicolons/bangs. These tests use existing public fronts.

use surgeist_css::{
    CssAuthoredFontFaceDescriptorValue as FaceValue, CssComponentValues,
    CssFontFaceDescriptorKind as FaceKind, CssFontPaletteDescriptorKind as PaletteKind,
    CssFontPaletteDescriptorValue as PaletteValue, CssFontPaletteDescriptorValueRef,
    CssRecoveryAction, CssRule, CssValueOrigin, parse_component_values, parse_font_face_block,
    parse_font_face_descriptor_value, parse_font_palette_descriptor_value,
    parse_font_palette_values_block, parse_sheet,
};

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
    assert_eq!(components.serialize().unwrap().as_css(), value);
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
        let report = parse_font_face_descriptor_value(value, FaceKind::FontWidth);
        assert!(report.is_clean(), "{value}: {report:?}");
        pending_face(report.syntax().as_ref().unwrap(), value, value);
    }
}

#[test]
fn checked_font_face_width_defers_matched_root_blocks_without_losing_provenance() {
    for value in FACE_VALUES {
        let components = parse_component_values(value).unwrap();
        let checked =
            FaceValue::try_from_components(FaceKind::FontWidth, components.clone()).unwrap();
        pending_face(&checked, value, value);
        let FaceValue::Pending(pending) = checked else {
            unreachable!()
        };
        assert_eq!(pending.components(), &components);
    }
}

#[test]
fn font_face_body_retains_pending_width_with_matched_blocks_and_ordinary_neighbor() {
    for value in FACE_VALUES {
        let source = format!("{{font-width:{value};font-display:swap;}}");
        let report = parse_font_face_block(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let descriptors = report.syntax().as_ref().unwrap().body();
        assert_eq!(descriptors.occurrences().len(), 2);
        pending_face(
            descriptors.effective(FaceKind::FontWidth).unwrap().value(),
            value,
            &source,
        );
        assert!(descriptors.effective(FaceKind::FontDisplay).is_some());
    }
}

#[test]
fn font_face_rule_retains_pending_width_with_matched_blocks_and_following_rule() {
    for value in FACE_VALUES {
        let source =
            format!("@font-face{{font-width:{value};font-display:swap;}} .after{{color:red;}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let [CssRule::FontFace(face), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("font-face and following rule must survive");
        };
        assert_eq!(face.descriptors().occurrences().len(), 2);
        pending_face(
            face.descriptors()
                .effective(FaceKind::FontWidth)
                .unwrap()
                .value(),
            value,
            &source,
        );
    }
}

#[test]
fn raw_palette_descriptors_defer_matched_root_blocks_and_retain_original_components() {
    for kind in PALETTE_KINDS {
        for value in PALETTE_VALUES {
            let report = parse_font_palette_descriptor_value(value, kind);
            assert!(report.is_clean(), "{}:{value}: {report:?}", kind.css_name());
            pending_palette(report.syntax().as_ref().unwrap(), kind, value, value);
        }
    }
}

#[test]
fn checked_palette_descriptors_defer_matched_root_blocks_without_losing_provenance() {
    for kind in PALETTE_KINDS {
        for value in PALETTE_VALUES {
            let components = parse_component_values(value).unwrap();
            let checked = PaletteValue::try_new(kind, components.clone()).unwrap();
            pending_palette(&checked, kind, value, value);
            assert_eq!(checked.components(), &components);
        }
    }
}

#[test]
fn palette_body_retains_pending_matched_blocks_and_real_family_neighbor() {
    for kind in PALETTE_KINDS {
        for value in PALETTE_VALUES {
            let source = format!("{{font-family:Demo;{}:{value};}}", kind.css_name());
            let report = parse_font_palette_values_block(&source);
            assert!(report.is_clean(), "{source}: {report:?}");
            let descriptors = report.syntax().as_ref().unwrap().body();
            assert_eq!(descriptors.len(), 2);
            pending_palette(descriptors[1].value(), kind, value, &source);
        }
    }
}

#[test]
fn palette_rule_retains_pending_matched_blocks_and_following_rule() {
    for kind in PALETTE_KINDS {
        for value in PALETTE_VALUES {
            let source = format!(
                "@font-palette-values --x{{font-family:Demo;{}:{value};}} .after{{color:red;}}",
                kind.css_name()
            );
            let report = parse_sheet(&source);
            assert!(report.is_clean(), "{source}: {report:?}");
            let [CssRule::FontPaletteValues(palette), CssRule::Style(_)] = report.syntax().rules()
            else {
                panic!("palette and following rule must survive");
            };
            assert_eq!(palette.descriptors().len(), 2);
            pending_palette(palette.descriptors()[1].value(), kind, value, &source);
        }
    }
}

#[test]
fn malformed_env_and_ordinary_root_blocks_remain_invalid_in_raw_and_checked_fronts() {
    for value in [
        "{}",
        "{hello}",
        "env() {}",
        "env(123) {}",
        "env(choice) env() {}",
    ] {
        let face = parse_font_face_descriptor_value(value, FaceKind::FontWidth);
        assert!(face.syntax().is_none(), "{value}: {face:?}");
        assert!(!face.is_clean());
        let components = parse_component_values(value).unwrap();
        assert!(FaceValue::try_from_components(FaceKind::FontWidth, components.clone()).is_err());
        for kind in PALETTE_KINDS {
            let palette = parse_font_palette_descriptor_value(value, kind);
            assert!(
                palette.syntax().is_none(),
                "{}:{value}: {palette:?}",
                kind.css_name()
            );
            assert!(!palette.is_clean());
            assert!(PaletteValue::try_new(kind, components.clone()).is_err());
        }
    }
}

#[test]
fn root_bangs_semicolons_and_unmatched_closers_are_not_rescued_by_valid_env() {
    for value in [
        "env(choice)!important",
        "env(choice);",
        "env(choice)}",
        "env(choice)]",
        "env(choice))",
    ] {
        let face = parse_font_face_descriptor_value(value, FaceKind::FontWidth);
        assert!(face.syntax().is_none(), "{value}: {face:?}");
        assert!(!face.is_clean());
        for kind in PALETTE_KINDS {
            let palette = parse_font_palette_descriptor_value(value, kind);
            assert!(
                palette.syntax().is_none(),
                "{}:{value}: {palette:?}",
                kind.css_name()
            );
            assert!(!palette.is_clean());
        }
        // Component admission may itself reject unmatched closers. If it admits
        // the token stream, the existing descriptor constructor must reject it.
        if let Ok(components) = parse_component_values(value) {
            assert!(
                FaceValue::try_from_components(FaceKind::FontWidth, components.clone()).is_err()
            );
            for kind in PALETTE_KINDS {
                assert!(PaletteValue::try_new(kind, components.clone()).is_err());
            }
        }
    }
}

#[test]
fn font_face_malformed_env_and_ordinary_blocks_drop_only_the_bad_descriptor() {
    for value in ["{}", "{hello}", "env() {}", "env(choice) env() {}"] {
        let body = format!("{{font-width:{value};font-display:swap;}}");
        let report = parse_font_face_block(&body);
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
        let sheet = parse_sheet(&source);
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
            let body = format!("{{font-family:Demo;{}:{value};}}", kind.css_name());
            let report = parse_font_palette_values_block(&body);
            assert!(!report.is_clean(), "{body}: {report:?}");
            assert_eq!(report.syntax().as_ref().unwrap().body().len(), 1);
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|d| d.action() == CssRecoveryAction::DropDescriptor)
            );
            let source = format!("@font-palette-values --x{body} .after{{color:red;}}");
            let sheet = parse_sheet(&source);
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
