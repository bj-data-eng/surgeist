#![forbid(unsafe_code)]
//! Independent source-entry characterization for existing palette/profile values.
//! Fonts4 WD20260907 §9.2 and Color5 WD20260908 §5.3 require their local values;
//! #504/public source contracts retain original value windows and typed depth256.

use std::fmt::Debug;
use surgeist_css::*;

fn point(actual: CssSourcePosition, expected: (usize, u32, u32)) {
    assert_eq!(actual.byte_offset().value(), expected.0);
    assert_eq!(actual.line().value(), expected.1);
    assert_eq!(actual.column().value(), expected.2);
}

fn empty<T: Debug>(report: CssParseReport<Option<T>>, source: &str, eof: (usize, u32, u32)) {
    assert!(report.syntax().is_none(), "{source:?}: {report:?}");
    assert!(!report.is_clean());
    // Missing required value is owned by actual EOF, not an invented descriptor
    // occurrence or the start of leading trivia. Do not impose a family error code.
    let diagnostic = report
        .diagnostics()
        .first()
        .expect("required-value rejection");
    point(diagnostic.error().position(), eof);
    point(diagnostic.span().start(), (0, 0, 0));
    point(diagnostic.span().end(), eof);
    assert!(report.into_validation_result().is_err());
}

fn window(
    values: &CssComponentValues,
    source: &str,
    eof: (usize, u32, u32),
    first_value_end: usize,
) {
    let first = values.items().first().expect("original leading trivia");
    let CssValueOrigin::Parsed(first) = first.origin() else {
        panic!("source-produced component");
    };
    point(first.span().start(), (0, 0, 0));
    let mut previous_end = 0;
    let mut first_value = false;
    for component in values.items() {
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("original root component");
        };
        assert_eq!(origin.source().as_str(), source);
        assert!(origin.source().same_snapshot(first.source()));
        assert_eq!(origin.span().start().byte_offset().value(), previous_end);
        previous_end = origin.span().end().byte_offset().value();
        if origin.span().start().byte_offset().value() == 9 {
            point(origin.span().start(), (9, 1, 0));
            point(
                origin.span().end(),
                (first_value_end, 1, (first_value_end - 9) as u32),
            );
            first_value = true;
        }
    }
    assert!(
        first_value,
        "actual semantic token begins after original CRLF"
    );
    assert_eq!(
        previous_end, eof.0,
        "complete original value window includes trailing trivia"
    );
    let CssValueOrigin::Parsed(last) = values.items().last().unwrap().origin() else {
        panic!("original final component");
    };
    point(last.span().end(), eof);
}

fn tail<T: Debug>(report: CssParseReport<Option<T>>, boundary: usize, end: usize) {
    assert!(
        report.syntax().is_none(),
        "no prefix value may escape: {report:?}"
    );
    assert!(!report.is_clean());
    let diagnostic = report
        .diagnostics()
        .first()
        .expect("root boundary rejection");
    point(
        diagnostic.error().position(),
        (boundary, 0, boundary as u32),
    );
    point(diagnostic.span().start(), (0, 0, 0));
    point(diagnostic.span().end(), (end, 0, end as u32));
    assert!(report.into_validation_result().is_err());
}

fn depth<T: Debug>(report: CssParseReport<Option<T>>, exceeded: bool) {
    // f() is outside all six selected value grammars; 256 must reach grammar
    // rejection, while 257 must retain the existing source resource identity.
    assert!(report.syntax().is_none(), "{report:?}");
    assert!(!report.is_clean());
    if exceeded {
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|d| d.action() == CssRecoveryAction::StopAtNestingLimit)
            .expect("typed source depth stop");
        let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
            panic!("resource error must not become descriptor grammar failure");
        };
        assert_eq!(detail.limit(), 256);
        point(diagnostic.error().position(), (512, 0, 512));
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
    } else {
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| d.error().code() == CssErrorCode::NestingLimit)
        );
    }
    assert!(report.into_validation_result().is_err());
}

#[test]
fn required_palette_and_profile_values_reject_trivia_only_at_original_eof() {
    for (source, eof) in [
        ("", (0, 0, 0)),
        (" \t", (2, 0, 2)),
        ("/**/", (4, 0, 4)),
        (" /*é*/\r\n", (9, 1, 0)),
    ] {
        for kind in [
            CssFontPaletteDescriptorKind::FontFamily,
            CssFontPaletteDescriptorKind::BasePalette,
            CssFontPaletteDescriptorKind::OverrideColors,
        ] {
            empty(
                parse_font_palette_descriptor_value(source, kind),
                source,
                eof,
            );
        }
        for kind in [
            CssColorProfileDescriptorKind::Src,
            CssColorProfileDescriptorKind::RenderingIntent,
            CssColorProfileDescriptorKind::Components,
        ] {
            empty(
                parse_color_profile_descriptor_value(source, kind),
                source,
                eof,
            );
        }
    }
}

#[test]
fn successful_values_keep_the_complete_unicode_window_without_named_occurrences() {
    // Prefix bytes0..9, including é and CRLF. Suffix has ten UTF8 bytes and
    // eight UTF16 units. These literal offsets follow the original spelling.
    for (kind, source, eof, first_end) in [
        (
            CssFontPaletteDescriptorKind::FontFamily,
            " /*é*/\r\nDemo /*💡*/ ",
            (23, 1, 12),
            13,
        ),
        (
            CssFontPaletteDescriptorKind::BasePalette,
            " /*é*/\r\nlight /*💡*/ ",
            (24, 1, 13),
            14,
        ),
        (
            CssFontPaletteDescriptorKind::OverrideColors,
            " /*é*/\r\n0 red /*💡*/ ",
            (24, 1, 13),
            10,
        ),
    ] {
        let report = parse_font_palette_descriptor_value(source, kind);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        let value = report.syntax().as_ref().unwrap();
        assert_eq!(value.kind(), kind);
        window(value.components(), source, eof, first_end);
        assert!(report.into_validation_result().unwrap().is_some());
    }
    for (kind, source, eof, first_end) in [
        (
            CssColorProfileDescriptorKind::Src,
            " /*é*/\r\nurl(p) /*💡*/ ",
            (25, 1, 14),
            15,
        ),
        (
            CssColorProfileDescriptorKind::RenderingIntent,
            " /*é*/\r\nperceptual /*💡*/ ",
            (29, 1, 18),
            19,
        ),
        (
            CssColorProfileDescriptorKind::Components,
            " /*é*/\r\na, b /*💡*/ ",
            (23, 1, 12),
            10,
        ),
    ] {
        let report = parse_color_profile_descriptor_value(source, kind);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        let value = report.syntax().as_ref().unwrap();
        assert_eq!(value.kind(), kind);
        window(value.components(), source, eof, first_end);
        assert!(report.into_validation_result().unwrap().is_some());
    }
}

#[test]
fn a_valid_value_before_a_root_tail_is_rejected_with_original_boundary_provenance() {
    for (kind, value, boundary) in [
        (CssFontPaletteDescriptorKind::FontFamily, "Demo", 4),
        (CssFontPaletteDescriptorKind::BasePalette, "light", 5),
        (CssFontPaletteDescriptorKind::OverrideColors, "0 red", 5),
    ] {
        for suffix in [";", "}"] {
            tail(
                parse_font_palette_descriptor_value(&format!("{value}{suffix}"), kind),
                boundary,
                boundary + 1,
            );
        }
    }
    for (kind, value, boundary) in [
        (CssColorProfileDescriptorKind::Src, "url(p)", 6),
        (
            CssColorProfileDescriptorKind::RenderingIntent,
            "perceptual",
            10,
        ),
        (CssColorProfileDescriptorKind::Components, "a, b", 4),
    ] {
        for suffix in [";", "}"] {
            tail(
                parse_color_profile_descriptor_value(&format!("{value}{suffix}"), kind),
                boundary,
                boundary + 1,
            );
        }
    }
}

#[test]
fn source_depth_256_reaches_grammar_but_257_keeps_the_actual_typed_stop() {
    for (nested, exceeded) in [(256, false), (257, true)] {
        let source = format!("{}x{}", "f(".repeat(nested), ")".repeat(nested));
        for kind in [
            CssFontPaletteDescriptorKind::FontFamily,
            CssFontPaletteDescriptorKind::BasePalette,
            CssFontPaletteDescriptorKind::OverrideColors,
        ] {
            depth(parse_font_palette_descriptor_value(&source, kind), exceeded);
        }
        for kind in [
            CssColorProfileDescriptorKind::Src,
            CssColorProfileDescriptorKind::RenderingIntent,
            CssColorProfileDescriptorKind::Components,
        ] {
            depth(
                parse_color_profile_descriptor_value(&source, kind),
                exceeded,
            );
        }
    }
}

#[test]
fn each_family_retains_its_own_pending_eof_closure_and_fails_clean_validation() {
    // Palette accepts selected var substitution; profile admits env, not var.
    let palette = parse_font_palette_descriptor_value(
        "var(--choice",
        CssFontPaletteDescriptorKind::BasePalette,
    );
    assert!(matches!(
        palette.syntax().as_ref().unwrap().view(),
        CssFontPaletteDescriptorValueRef::Pending(_)
    ));
    let profile = parse_color_profile_descriptor_value(
        "env(profile",
        CssColorProfileDescriptorKind::Components,
    );
    assert!(matches!(
        profile.syntax().as_ref().unwrap().view(),
        CssColorProfileDescriptorValueRef::Pending(_)
    ));
    for (source, values, diagnostics, eof) in [
        (
            "var(--choice",
            palette.syntax().as_ref().unwrap().components(),
            palette.diagnostics(),
            12,
        ),
        (
            "env(profile",
            profile.syntax().as_ref().unwrap().components(),
            profile.diagnostics(),
            11,
        ),
    ] {
        let [component] = values.items() else {
            panic!("one actual pending function");
        };
        let CssComponentValueRef::Function(function) = component.view() else {
            panic!("function");
        };
        let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
            panic!("actual source EOF closure");
        };
        assert_eq!(opening.source().as_str(), source);
        point(opening.span().start(), (0, 0, 0));
        point(at.span().start(), (eof, 0, eof as u32));
        point(at.span().end(), (eof, 0, eof as u32));
        let diagnostic = diagnostics
            .iter()
            .find(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
            .expect("retained source closure diagnostic");
        point(diagnostic.error().position(), (eof, 0, eof as u32));
    }
    assert!(palette.into_validation_result().is_err());
    assert!(profile.into_validation_result().is_err());
    let invalid_profile = parse_color_profile_descriptor_value(
        "var(--choice)",
        CssColorProfileDescriptorKind::Components,
    );
    assert!(invalid_profile.syntax().is_none());
    assert!(invalid_profile.into_validation_result().is_err());
}
