#![forbid(unsafe_code)]

//! Fonts 4 (2026-09-07) §§4, 9.2; Counter Styles 3 (2021-07-27) §3;
//! Color 5 (2026-09-08) §5.3; Syntax 3 (2021) component/declaration recovery.

#[path = "support/descriptor_phases.rs"]
mod descriptor_phases;
use descriptor_phases::*;

use surgeist_css::*;

fn body<T>(report: &CssParseReport<Option<CssBlockFragment<T>>>) -> &CssBlockFragment<T> {
    report.syntax().as_ref().expect("an admitted genuine body")
}

fn clean<T: Clone + std::fmt::Debug + PartialEq>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
) {
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report.clone().into_validation_result().unwrap(),
        report.syntax().clone(),
    );
}

fn recovered<T: Clone + std::fmt::Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    action: CssRecoveryAction,
) -> &CssRecoveryDiagnostic {
    assert!(!report.is_clean());
    assert!(report.clone().into_validation_result().is_err());
    report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.action() == action)
        .expect("the owning recovery action")
}

fn point(actual: CssSourcePosition, byte: usize, line: u32, column: u32) {
    assert_eq!(actual.byte_offset().value(), byte);
    assert_eq!(actual.line().value(), line);
    assert_eq!(actual.column().value(), column);
}

fn origin(actual: &CssParsedOrigin, source: &str, start: usize, end: usize) {
    assert_eq!(actual.source().as_str(), source);
    assert_eq!(actual.span().start().byte_offset().value(), start);
    assert_eq!(actual.span().end().byte_offset().value(), end);
}

fn rejected<T: Clone + std::fmt::Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    source: &str,
) {
    assert!(
        report.syntax().is_none(),
        "whole fragment must be rejected: {source}"
    );
    let diagnostic = recovered(report, CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
}

fn symbol(actual: &CssCounterSymbol, expected: &str, quoted: bool) {
    match (actual, quoted) {
        (CssCounterSymbol::String(actual), true) => assert_eq!(actual.as_str(), expected),
        (CssCounterSymbol::Ident(actual), false) => assert_eq!(actual.as_str(), expected),
        _ => panic!("unexpected counter symbol {actual:?}"),
    }
}

fn face_kinds(value: &CssFontFaceDescriptors) -> Vec<CssFontFaceDescriptorKind> {
    value
        .occurrences()
        .map(|record| record.value().kind())
        .collect()
}

fn palette_kinds(value: &[CssFontPaletteDescriptor]) -> Vec<CssFontPaletteDescriptorKind> {
    value.iter().map(|record| record.value().kind()).collect()
}

fn profile_kinds(value: &[CssColorProfileDescriptor]) -> Vec<CssColorProfileDescriptorKind> {
    value.iter().map(|record| record.value().kind()).collect()
}

#[test]
fn font_face_body_retains_order_and_effective_duplicate_display() {
    let source = "{font-family:A;src:url(a);font-display:swap;font-display:optional}";
    let report = parse_font_face_block(source);
    clean(&report);
    let fragment = body(&report);
    origin(fragment.origin(), source, 0, source.len());
    assert_eq!(
        face_kinds(fragment.body()),
        [
            CssFontFaceDescriptorKind::FontFamily,
            CssFontFaceDescriptorKind::Src,
            CssFontFaceDescriptorKind::FontDisplay,
            CssFontFaceDescriptorKind::FontDisplay,
        ]
    );
    let family = fragment
        .body()
        .effective(CssFontFaceDescriptorKind::FontFamily)
        .unwrap();
    let CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::FontFamily(
        family,
    )) = family.value()
    else {
        panic!("ordinary font family")
    };
    assert_eq!(family.as_str(), "A");
    let display = fragment
        .body()
        .effective(CssFontFaceDescriptorKind::FontDisplay)
        .unwrap();
    assert!(matches!(
        display.value(),
        CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::FontDisplay(
            CssFontDisplay::Optional
        ))
    ));
    // Positions belong to authored descriptor names, not a fabricated @font-face.
    point(
        fragment
            .body()
            .occurrences()
            .next()
            .unwrap()
            .position()
            .unwrap(),
        1,
        0,
        1,
    );
}

#[test]
fn counter_body_retains_all_symbols_occurrences_and_last_valid_symbols() {
    let source = "{system:numeric;symbols:\"0\" \"1\";symbols:a b}";
    let report = parse_counter_style_block(source);
    clean(&report);
    let descriptors = body(&report).body();
    assert!(matches!(
        descriptors.system().unwrap().ordinary_system(),
        CssCounterStyleSystem::Numeric
    ));
    let occurrences = descriptors.occurrences().collect::<Vec<_>>();
    let [
        CssCounterStyleDescriptorRef::System(_),
        CssCounterStyleDescriptorRef::Symbols(first),
        CssCounterStyleDescriptorRef::Symbols(last),
    ] = occurrences.as_slice()
    else {
        panic!("system and both symbols occurrences")
    };
    assert_eq!(first.ordinary_symbols().symbols().len(), 2);
    symbol(&first.ordinary_symbols().symbols()[0], "0", true);
    symbol(&first.ordinary_symbols().symbols()[1], "1", true);
    symbol(&last.ordinary_symbols().symbols()[0], "a", false);
    symbol(&last.ordinary_symbols().symbols()[1], "b", false);
    assert_eq!(descriptors.symbols().unwrap(), *last);
    origin(body(&report).origin(), source, 0, source.len());
}

#[test]
fn palette_body_retains_required_family_and_duplicate_base_occurrences() {
    let source = "{font-family:A;base-palette:light;base-palette:dark}";
    let report = parse_font_palette_values_block(source);
    clean(&report);
    let descriptors = body(&report).body();
    assert_eq!(
        palette_kinds(descriptors),
        [
            CssFontPaletteDescriptorKind::FontFamily,
            CssFontPaletteDescriptorKind::BasePalette,
            CssFontPaletteDescriptorKind::BasePalette
        ]
    );
    let CssFontPaletteDescriptorValueRef::FontFamily(families) = descriptors[0].value().view()
    else {
        panic!("family list")
    };
    assert_eq!(
        families
            .iter()
            .map(CssFontFaceFamily::as_str)
            .collect::<Vec<_>>(),
        ["A"]
    );
    assert!(matches!(
        descriptors[1].value().view(),
        CssFontPaletteDescriptorValueRef::BasePalette(CssFontPaletteBase::Light)
    ));
    assert!(matches!(
        descriptors[2].value().view(),
        CssFontPaletteDescriptorValueRef::BasePalette(CssFontPaletteBase::Dark)
    ));
    origin(body(&report).origin(), source, 0, source.len());
}

#[test]
fn profile_body_retains_url_intent_and_comma_separated_component_names() {
    let source = "{src:url(p.icc);rendering-intent:perceptual;components:A, a, A}";
    let report = parse_color_profile_block(source);
    clean(&report);
    let descriptors = body(&report).body();
    assert_eq!(
        profile_kinds(descriptors),
        [
            CssColorProfileDescriptorKind::Src,
            CssColorProfileDescriptorKind::RenderingIntent,
            CssColorProfileDescriptorKind::Components
        ]
    );
    let CssColorProfileDescriptorValueRef::Src(url) = descriptors[0].value().view() else {
        panic!("authored profile URL")
    };
    assert_eq!(url.as_str(), "p.icc");
    assert!(matches!(
        descriptors[1].value().view(),
        CssColorProfileDescriptorValueRef::RenderingIntent(
            CssColorProfileRenderingIntent::Perceptual
        )
    ));
    let CssColorProfileDescriptorValueRef::Components(names) = descriptors[2].value().view() else {
        panic!("ordered profile component names")
    };
    assert_eq!(
        names.iter().map(|name| name.as_str()).collect::<Vec<_>>(),
        ["A", "a", "A"]
    );
    origin(body(&report).origin(), source, 0, source.len());
}

#[test]
fn real_empty_bodies_do_not_fabricate_defaults_or_required_descriptors() {
    let face = parse_font_face_block("{}");
    clean(&face);
    assert_eq!(body(&face).body().occurrences().len(), 0);
    let counter = parse_counter_style_block("{}");
    clean(&counter);
    assert_eq!(body(&counter).body().occurrences().len(), 0);
    assert!(body(&counter).body().system().is_none());
    let profile = parse_color_profile_block("{}");
    clean(&profile);
    assert!(body(&profile).body().is_empty());
    for source in [
        "{}",
        "{base-palette:0}",
        "{font-family:serif;base-palette:dark}",
    ] {
        rejected(&parse_font_palette_values_block(source), source);
    }
    // Missing symbols is downstream counter usability, not an authored body error.
    let counter = parse_counter_style_block("{symbols:x}");
    clean(&counter);
    assert!(body(&counter).body().system().is_none());
    symbol(
        &body(&counter)
            .body()
            .symbols()
            .unwrap()
            .ordinary_symbols()
            .symbols()[0],
        "x",
        false,
    );
}

#[test]
fn counter_extends_symbol_combination_retains_authored_body_with_undefined_definition() {
    for source in [
        "{system:extends decimal;symbols:x}",
        "{system:extends decimal;additive-symbols:1 x}",
    ] {
        let report = parse_counter_style_block(source);
        clean(&report);
        assert_eq!(
            body(&report)
                .body()
                .prospective()
                .unwrap()
                .definition_status(),
            surgeist_css::CssCounterStyleDefinitionStatus::Undefined(
                surgeist_css::CssCounterStyleDefinitionIssue::ExtendsWithSymbols
            )
        );
    }
}

#[test]
fn no_real_curly_opener_is_distinct_from_an_empty_body() {
    for source in [
        "",
        " \t\r\n",
        "/* only trivia */",
        "[]",
        "()",
        "font-family:A",
        "}",
    ] {
        rejected(&parse_font_face_block(source), source);
        rejected(&parse_counter_style_block(source), source);
        rejected(&parse_font_palette_values_block(source), source);
        rejected(&parse_color_profile_block(source), source);
    }
}

#[test]
fn trailing_nontrivia_never_publishes_a_valid_first_body() {
    for tail in ["x", ";", "{}", "}", ")", "]"] {
        let source = format!("{{font-family:A}}{tail}");
        rejected(&parse_font_face_block(&source), &source);
        rejected(&parse_font_palette_values_block(&source), &source);
        let source = format!("{{prefix:\"a\"}}{tail}");
        rejected(&parse_counter_style_block(&source), &source);
        let source = format!("{{src:url(a)}}{tail}");
        rejected(&parse_color_profile_block(&source), &source);
    }
}

#[test]
fn unknown_descriptor_units_preserve_valid_neighbors_and_fail_clean_validation() {
    let face = parse_font_face_block("{font-family:A;bogus:1;src:url(a)}");
    assert_eq!(
        face_kinds(body(&face).body()),
        [
            CssFontFaceDescriptorKind::FontFamily,
            CssFontFaceDescriptorKind::Src
        ]
    );
    let counter = parse_counter_style_block("{prefix:\"a\";bogus:1;suffix:\"b\"}");
    assert_eq!(body(&counter).body().occurrences().len(), 2);
    symbol(
        body(&counter).body().prefix().unwrap().ordinary_prefix(),
        "a",
        true,
    );
    symbol(
        body(&counter).body().suffix().unwrap().ordinary_suffix(),
        "b",
        true,
    );
    let palette = parse_font_palette_values_block("{font-family:A;bogus:1;base-palette:dark}");
    assert_eq!(
        palette_kinds(body(&palette).body()),
        [
            CssFontPaletteDescriptorKind::FontFamily,
            CssFontPaletteDescriptorKind::BasePalette
        ]
    );
    let profile = parse_color_profile_block("{src:url(a);bogus:1;rendering-intent:perceptual}");
    assert_eq!(
        profile_kinds(body(&profile).body()),
        [
            CssColorProfileDescriptorKind::Src,
            CssColorProfileDescriptorKind::RenderingIntent
        ]
    );
    for (diagnostic, start) in [
        (recovered(&face, CssRecoveryAction::DropDescriptor), 15),
        (recovered(&counter, CssRecoveryAction::DropDescriptor), 12),
        (recovered(&palette, CssRecoveryAction::DropDescriptor), 15),
        (recovered(&profile, CssRecoveryAction::DropDescriptor), 12),
    ] {
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownDescriptor);
        point(diagnostic.error().position(), start, 0, start as u32);
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        // "bogus:1" is seven bytes; the next name follows its semicolon.
        assert!((start + 7..=start + 8).contains(&diagnostic.span().end().byte_offset().value()));
    }
}

#[test]
fn descriptor_failure_can_leave_a_retained_empty_body_while_palette_remains_required() {
    let source = "{bogus:1}";
    let face = parse_font_face_block(source);
    assert_eq!(body(&face).body().occurrences().len(), 0);
    recovered(&face, CssRecoveryAction::DropDescriptor);
    let counter = parse_counter_style_block(source);
    assert_eq!(body(&counter).body().occurrences().len(), 0);
    recovered(&counter, CssRecoveryAction::DropDescriptor);
    let profile = parse_color_profile_block(source);
    assert!(body(&profile).body().is_empty());
    recovered(&profile, CssRecoveryAction::DropDescriptor);
    rejected(&parse_font_palette_values_block(source), source);
}

#[test]
fn complete_value_errors_drop_only_the_bad_descriptor() {
    let face = parse_font_face_block("{font-family:A;font-display:swap extra;src:url(a)}");
    assert_eq!(
        face_kinds(body(&face).body()),
        [
            CssFontFaceDescriptorKind::FontFamily,
            CssFontFaceDescriptorKind::Src
        ]
    );
    recovered(&face, CssRecoveryAction::DropDescriptor);
    let counter = parse_counter_style_block("{prefix:\"a\";system:numeric extra;suffix:\"b\"}");
    assert_eq!(body(&counter).body().occurrences().len(), 2);
    assert!(body(&counter).body().system().is_none());
    recovered(&counter, CssRecoveryAction::DropDescriptor);
    let palette = parse_font_palette_values_block(
        "{font-family:A;base-palette:light extra;override-colors:0 red}",
    );
    assert_eq!(
        palette_kinds(body(&palette).body()),
        [
            CssFontPaletteDescriptorKind::FontFamily,
            CssFontPaletteDescriptorKind::OverrideColors
        ]
    );
    recovered(&palette, CssRecoveryAction::DropDescriptor);
    // The selected Color 5 components production uses commas, not spaces.
    let profile =
        parse_color_profile_block("{src:url(a);components:a b;rendering-intent:perceptual}");
    assert_eq!(
        profile_kinds(body(&profile).body()),
        [
            CssColorProfileDescriptorKind::Src,
            CssColorProfileDescriptorKind::RenderingIntent
        ]
    );
    recovered(&profile, CssRecoveryAction::DropDescriptor);
}

#[test]
fn structural_child_is_fatal_for_font_face_and_recoverable_for_other_families() {
    let source = "{font-family:A;@unknown{}src:url(a)}";
    rejected(&parse_font_face_block(source), source);
    let counter = parse_counter_style_block("{prefix:\"a\";@unknown{}suffix:\"b\"}");
    assert_eq!(body(&counter).body().occurrences().len(), 2);
    symbol(
        body(&counter).body().suffix().unwrap().ordinary_suffix(),
        "b",
        true,
    );
    recovered(&counter, CssRecoveryAction::DropAtRule);
    let palette = parse_font_palette_values_block("{font-family:A;@unknown{}base-palette:dark}");
    assert_eq!(
        palette_kinds(body(&palette).body()),
        [
            CssFontPaletteDescriptorKind::FontFamily,
            CssFontPaletteDescriptorKind::BasePalette
        ]
    );
    recovered(&palette, CssRecoveryAction::DropAtRule);
    let profile = parse_color_profile_block("{src:url(a);@unknown{}rendering-intent:perceptual}");
    assert_eq!(
        profile_kinds(body(&profile).body()),
        [
            CssColorProfileDescriptorKind::Src,
            CssColorProfileDescriptorKind::RenderingIntent
        ]
    );
    recovered(&profile, CssRecoveryAction::DropAtRule);
}

#[test]
fn original_utf8_and_utf16_coordinates_exclude_outer_trivia_but_share_its_snapshot() {
    let source = " /*é*/\r\n{font-family:A} /*💡*/ ";
    assert_eq!(source.len(), 34);
    let face = parse_font_face_block(source);
    clean(&face);
    origin(body(&face).origin(), source, 9, 24);
    point(body(&face).origin().span().start(), 9, 1, 0);
    point(body(&face).origin().span().end(), 24, 1, 15);
    point(
        body(&face)
            .body()
            .occurrences()
            .next()
            .unwrap()
            .position()
            .unwrap(),
        10,
        1,
        1,
    );
    let palette = parse_font_palette_values_block(source);
    clean(&palette);
    origin(body(&palette).origin(), source, 9, 24);
    let descriptor = &body(&palette).body()[0];
    point(descriptor.position().unwrap(), 10, 1, 1);
    let [component] = descriptor.value().components().items() else {
        panic!("one family token")
    };
    let CssValueOrigin::Parsed(parsed) = component.origin() else {
        panic!("original family token")
    };
    origin(parsed, source, 22, 23);
    point(parsed.span().start(), 22, 1, 13);
    point(parsed.span().end(), 23, 1, 14);
    assert!(
        parsed
            .source()
            .same_snapshot(body(&palette).origin().source())
    );

    let source = " /*é*/\r\n{prefix:\"💡\"} /*x*/";
    let counter = parse_counter_style_block(source);
    clean(&counter);
    origin(body(&counter).origin(), source, 9, 24);
    point(body(&counter).origin().span().end(), 24, 1, 13);
    let prefix = body(&counter).body().prefix().unwrap();
    origin(prefix.parsed_name().unwrap(), source, 10, 16);
    origin(prefix.parsed_value().unwrap(), source, 17, 23);
    point(prefix.parsed_value().unwrap().span().start(), 17, 1, 8);
    point(prefix.parsed_value().unwrap().span().end(), 23, 1, 12);
    symbol(prefix.ordinary_prefix(), "💡", true);
    assert!(
        prefix
            .parsed_value()
            .unwrap()
            .source()
            .same_snapshot(body(&counter).origin().source())
    );

    let source = " /*é*/\r\n{src:url(a)} /*💡*/ ";
    let profile = parse_color_profile_block(source);
    clean(&profile);
    origin(body(&profile).origin(), source, 9, 21);
    point(body(&profile).origin().span().end(), 21, 1, 12);
    let descriptor = &body(&profile).body()[0];
    point(descriptor.position().unwrap(), 10, 1, 1);
    let [component] = descriptor.value().components().items() else {
        panic!("one URL token")
    };
    let CssValueOrigin::Parsed(parsed) = component.origin() else {
        panic!("original URL token")
    };
    origin(parsed, source, 14, 20);
    point(parsed.span().start(), 14, 1, 5);
    point(parsed.span().end(), 20, 1, 11);
    assert!(
        parsed
            .source()
            .same_snapshot(body(&profile).origin().source())
    );
}

fn implicit<T: Clone + std::fmt::Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    source: &str,
) {
    origin(body(report).origin(), source, 0, source.len());
    let diagnostic = recovered(report, CssRecoveryAction::RetainWithImplicitClosure);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.len()
    );
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
}

#[test]
fn implicit_outer_braces_retain_valid_payloads_at_original_eof() {
    let source = "{font-family:A";
    let face = parse_font_face_block(source);
    implicit(&face, source);
    assert_eq!(
        face_kinds(body(&face).body()),
        [CssFontFaceDescriptorKind::FontFamily]
    );
    let palette = parse_font_palette_values_block(source);
    implicit(&palette, source);
    assert_eq!(
        palette_kinds(body(&palette).body()),
        [CssFontPaletteDescriptorKind::FontFamily]
    );
    let source = "{prefix:\"a";
    let counter = parse_counter_style_block(source);
    implicit(&counter, source);
    symbol(
        body(&counter).body().prefix().unwrap().ordinary_prefix(),
        "a",
        true,
    );
    let source = "{src:url(a)";
    let profile = parse_color_profile_block(source);
    implicit(&profile, source);
    assert_eq!(
        profile_kinds(body(&profile).body()),
        [CssColorProfileDescriptorKind::Src]
    );
    implicit(&parse_font_face_block("{"), "{");
    implicit(&parse_counter_style_block("{"), "{");
    implicit(&parse_color_profile_block("{"), "{");
    rejected(&parse_font_palette_values_block("{"), "{");
}

fn implicit_function(
    values: &CssComponentValues,
    block: &CssParsedOrigin,
    source: &str,
    start: usize,
) {
    let [component] = values.items() else {
        panic!("one retained substitution function")
    };
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("retained function")
    };
    assert_eq!(function.name(), "env");
    let CssValueOrigin::Parsed(opening) = component.origin() else {
        panic!("real env opener")
    };
    origin(opening, source, start, start + 4);
    let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
        panic!("EOF closure must stay implicit")
    };
    origin(opening, source, start, start + 4);
    origin(at, source, source.len(), source.len());
    assert!(at.source().same_snapshot(block.source()));
}

#[test]
fn retained_pending_function_closures_keep_implicit_metadata_and_original_eof() {
    let source = "{font-family:env(family";
    let face = parse_font_face_block(source);
    implicit(&face, source);
    let CssAuthoredFontFaceDescriptorValue::Pending(value) =
        body(&face).body().occurrences().next().unwrap().value()
    else {
        panic!("pending font-family")
    };
    implicit_function(value.components(), body(&face).origin(), source, 13);
    let palette = parse_font_palette_values_block(source);
    implicit(&palette, source);
    let value = body(&palette).body()[0].value();
    assert!(matches!(
        value.view(),
        CssFontPaletteDescriptorValueRef::Pending(_)
    ));
    implicit_function(value.components(), body(&palette).origin(), source, 13);
    let source = "{src:env(profile";
    let profile = parse_color_profile_block(source);
    implicit(&profile, source);
    let value = body(&profile).body()[0].value();
    assert!(matches!(
        value.view(),
        CssColorProfileDescriptorValueRef::Pending(_)
    ));
    implicit_function(value.components(), body(&profile).origin(), source, 5);
}

#[test]
fn unterminated_comments_and_outer_braces_report_original_eof_without_losing_descriptors() {
    let source = "{font-family:A;/* unfinished";
    let face = parse_font_face_block(source);
    implicit(&face, source);
    assert_eq!(
        face_kinds(body(&face).body()),
        [CssFontFaceDescriptorKind::FontFamily]
    );
    assert_eq!(
        recovered(&face, CssRecoveryAction::IgnoreUnterminatedComment)
            .error()
            .position()
            .byte_offset()
            .value(),
        source.len()
    );
    let palette = parse_font_palette_values_block(source);
    implicit(&palette, source);
    assert_eq!(
        palette_kinds(body(&palette).body()),
        [CssFontPaletteDescriptorKind::FontFamily]
    );
    recovered(&palette, CssRecoveryAction::IgnoreUnterminatedComment);
    let source = "{prefix:\"a\";/* unfinished";
    let counter = parse_counter_style_block(source);
    implicit(&counter, source);
    symbol(
        body(&counter).body().prefix().unwrap().ordinary_prefix(),
        "a",
        true,
    );
    recovered(&counter, CssRecoveryAction::IgnoreUnterminatedComment);
    let source = "{src:url(a);/* unfinished";
    let profile = parse_color_profile_block(source);
    implicit(&profile, source);
    assert_eq!(
        profile_kinds(body(&profile).body()),
        [CssColorProfileDescriptorKind::Src]
    );
    recovered(&profile, CssRecoveryAction::IgnoreUnterminatedComment);
}

#[test]
fn bad_url_descriptor_preserves_the_previous_url_and_following_intent() {
    let report =
        parse_color_profile_block("{src:url(good);src:url(a b);rendering-intent:perceptual}");
    assert_eq!(
        profile_kinds(body(&report).body()),
        [
            CssColorProfileDescriptorKind::Src,
            CssColorProfileDescriptorKind::RenderingIntent
        ]
    );
    let CssColorProfileDescriptorValueRef::Src(url) = body(&report).body()[0].value().view() else {
        panic!("original good URL")
    };
    assert_eq!(url.as_str(), "good");
    recovered(&report, CssRecoveryAction::DropDescriptor);
}

fn nesting<T: Clone + std::fmt::Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    source: &str,
    unit_start: usize,
    value_start: usize,
    later_start: usize,
    first_excessive_function: usize,
) {
    let diagnostic = recovered(report, CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("native typed nesting error")
    };
    assert_eq!(detail.limit(), 256);
    point(
        diagnostic.error().position(),
        value_start + first_excessive_function * 2,
        0,
        (value_start + first_excessive_function * 2) as u32,
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), unit_start);
    assert!(diagnostic.span().end().byte_offset().value() <= later_start);
    assert!(
        diagnostic.span().end().byte_offset().value()
            > diagnostic.error().position().byte_offset().value()
    );
    origin(body(report).origin(), source, 0, source.len());
}

#[test]
fn real_outer_brace_counts_once_and_excessive_descriptor_units_retain_both_neighbors() {
    for depth in [255, 256] {
        // Only the stimulus is generated. One brace + 255 functions = 256;
        // the zero-based function 255 is the first over-limit group at 257.
        let value = format!("{}x{}", "f(".repeat(depth), ")".repeat(depth));
        let face_source = format!("{{font-family:A;bad:{value};font-display:swap}}");
        let face = parse_font_face_block(&face_source);
        assert_eq!(
            face_kinds(body(&face).body()),
            [
                CssFontFaceDescriptorKind::FontFamily,
                CssFontFaceDescriptorKind::FontDisplay
            ]
        );
        let counter_source = format!("{{prefix:\"a\";bad:{value};suffix:\"b\"}}");
        let counter = parse_counter_style_block(&counter_source);
        assert_eq!(body(&counter).body().occurrences().len(), 2);
        symbol(
            body(&counter).body().prefix().unwrap().ordinary_prefix(),
            "a",
            true,
        );
        symbol(
            body(&counter).body().suffix().unwrap().ordinary_suffix(),
            "b",
            true,
        );
        let palette_source = format!("{{font-family:A;bad:{value};base-palette:dark}}");
        let palette = parse_font_palette_values_block(&palette_source);
        assert_eq!(
            palette_kinds(body(&palette).body()),
            [
                CssFontPaletteDescriptorKind::FontFamily,
                CssFontPaletteDescriptorKind::BasePalette
            ]
        );
        let profile_source = format!("{{src:url(a);bad:{value};rendering-intent:perceptual}}");
        let profile = parse_color_profile_block(&profile_source);
        assert_eq!(
            profile_kinds(body(&profile).body()),
            [
                CssColorProfileDescriptorKind::Src,
                CssColorProfileDescriptorKind::RenderingIntent
            ]
        );
        if depth == 255 {
            for diagnostic in [
                recovered(&face, CssRecoveryAction::DropDescriptor),
                recovered(&counter, CssRecoveryAction::DropDescriptor),
                recovered(&palette, CssRecoveryAction::DropDescriptor),
                recovered(&profile, CssRecoveryAction::DropDescriptor),
            ] {
                assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownDescriptor);
            }
            assert!(
                face.diagnostics()
                    .iter()
                    .chain(counter.diagnostics())
                    .chain(palette.diagnostics())
                    .chain(profile.diagnostics())
                    .all(|diagnostic| diagnostic.error().code() != CssErrorCode::NestingLimit)
            );
        } else {
            nesting(&face, &face_source, 15, 19, 19 + value.len() + 1, 255);
            nesting(&counter, &counter_source, 12, 16, 16 + value.len() + 1, 255);
            nesting(&palette, &palette_source, 15, 19, 19 + value.len() + 1, 255);
            nesting(&profile, &profile_source, 12, 16, 16 + value.len() + 1, 255);
        }
    }
}

#[test]
fn failed_structural_child_depth_retains_native_error_and_later_palette_profile_descriptors() {
    for depth in [254, 255] {
        // Outer brace + failed child's brace + 254 functions reaches 256.
        let value = format!("{}x{}", "f(".repeat(depth), ")".repeat(depth));
        let palette_source = format!("{{font-family:A;@unknown{{bad:{value}}}base-palette:dark}}");
        let palette = parse_font_palette_values_block(&palette_source);
        assert_eq!(
            palette_kinds(body(&palette).body()),
            [
                CssFontPaletteDescriptorKind::FontFamily,
                CssFontPaletteDescriptorKind::BasePalette
            ]
        );
        let profile_source =
            format!("{{src:url(a);@unknown{{bad:{value}}}rendering-intent:perceptual}}");
        let profile = parse_color_profile_block(&profile_source);
        assert_eq!(
            profile_kinds(body(&profile).body()),
            [
                CssColorProfileDescriptorKind::Src,
                CssColorProfileDescriptorKind::RenderingIntent
            ]
        );
        if depth == 254 {
            recovered(&palette, CssRecoveryAction::DropAtRule);
            recovered(&profile, CssRecoveryAction::DropAtRule);
            assert!(
                palette
                    .diagnostics()
                    .iter()
                    .chain(profile.diagnostics())
                    .all(|diagnostic| diagnostic.error().code() != CssErrorCode::NestingLimit)
            );
        } else {
            nesting(&palette, &palette_source, 15, 28, 28 + value.len() + 1, 254);
            nesting(&profile, &profile_source, 12, 25, 25 + value.len() + 1, 254);
        }
    }
}

#[test]
fn real_rule_resource_recovery_keeps_the_next_native_top_level_sibling() {
    let deep = format!("{}x{}", "f(".repeat(256), ")".repeat(256));
    // These are genuine whole-rule controls, secondary to the direct fronts.
    for (head, tail) in [
        (
            "@font-face{font-family:A;bad:",
            ";font-display:swap}.after{}",
        ),
        (
            "@counter-style x{prefix:\"a\";bad:",
            ";suffix:\"b\"}.after{}",
        ),
        (
            "@font-palette-values --x{font-family:A;bad:",
            ";base-palette:dark}.after{}",
        ),
        (
            "@color-profile --x{src:url(a);bad:",
            ";rendering-intent:perceptual}.after{}",
        ),
    ] {
        let source = format!("{head}{deep}{tail}");
        let report = parse_sheet(&source);
        assert!(matches!(report.syntax().rules(), [_, CssRule::Style(_)]));
        assert!(
            report
                .diagnostics()
                .iter()
                .any(
                    |diagnostic| diagnostic.error().code() == CssErrorCode::NestingLimit
                        && diagnostic.action() == CssRecoveryAction::StopAtNestingLimit
                )
        );
        assert!(report.into_validation_result().is_err());
    }
}
