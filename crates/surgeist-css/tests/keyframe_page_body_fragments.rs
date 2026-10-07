#![forbid(unsafe_code)]

//! Independent public draft: Animations 1 (2023-03-02) §3, CSS2 page/box
//! contexts, Values4 (2024-03-12) Appendix C, original-source Syntax recovery.

use surgeist_css::*;

const STANDARDS: CssParserContext = CssParserContext::new(CssParserMode::Standards);
const QUIRKS: CssParserContext = CssParserContext::new(CssParserMode::Quirks);

fn body<T>(report: &CssParseReport<Option<CssBlockFragment<T>>>) -> &CssBlockFragment<T> {
    report.syntax().as_ref().expect("retained genuine body")
}

fn clean<T: Clone + std::fmt::Debug + PartialEq>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
) {
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report.clone().into_validation_result().unwrap(),
        report.syntax().clone()
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
        .expect("the provider's owned recovery action")
}

fn rejected<T: Clone + std::fmt::Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    source: &str,
) {
    assert!(
        report.syntax().is_none(),
        "complete fragment rejected: {source}"
    );
    let diagnostic = recovered(report, CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
}

fn origin(actual: &CssParsedOrigin, source: &str, start: usize, end: usize) {
    assert_eq!(actual.source().as_str(), source);
    assert_eq!(actual.span().start().byte_offset().value(), start);
    assert_eq!(actual.span().end().byte_offset().value(), end);
}

fn point(actual: CssSourcePosition, byte: usize, line: u32, column: u32) {
    assert_eq!(actual.byte_offset().value(), byte);
    assert_eq!(actual.line().value(), line);
    assert_eq!(actual.column().value(), column);
}

fn keys(declarations: &CssKeyframeDeclarationList) -> Vec<CssPropertyNameRef<'_>> {
    declarations
        .iter()
        .map(CssKeyframeDeclaration::property_name)
        .collect()
}

fn page_keys(declarations: &CssDeclarationList) -> Vec<CssKnownProperty> {
    declarations
        .iter()
        .map(|declaration| declaration.known().unwrap().property())
        .collect()
}

fn opacity(declaration: &CssKeyframeDeclaration, expected: &str) {
    let CssKnownPropertyValueRef::Opacity(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed opacity")
    };
    let CssOpacityValue::Scalar(value) = value.value() else {
        panic!("ordinary exact opacity scalar")
    };
    assert_eq!(value.kind(), CssOpacityScalarKind::Number);
    assert_eq!(value.numeric().representation(), expected);
}

#[test]
fn keyframes_body_retains_ordered_duplicate_selectors_and_empty_child_blocks() {
    let source = "{from,0%,from{}50%{opacity:.5}to{opacity:1}100%{}}";
    let report = parse_keyframes_block(source);
    clean(&report);
    origin(body(&report).origin(), source, 0, source.len());
    let blocks = body(&report).body();
    assert_eq!(blocks.len(), 4);
    let [
        CssKeyframeSelector::From,
        CssKeyframeSelector::Percent(zero),
        CssKeyframeSelector::From,
    ] = blocks[0].selectors().selectors()
    else {
        panic!("all three authored selectors")
    };
    assert_eq!(zero.literal_value(), Some(0.0));
    assert!(blocks[0].declarations().is_empty());
    let [CssKeyframeSelector::Percent(half)] = blocks[1].selectors().selectors() else {
        panic!("50% selector")
    };
    assert_eq!(half.literal_value(), Some(50.0));
    opacity(&blocks[1].declarations()[0], ".5");
    assert!(matches!(
        blocks[2].selectors().selectors(),
        [CssKeyframeSelector::To]
    ));
    opacity(&blocks[2].declarations()[0], "1");
    let [CssKeyframeSelector::Percent(end)] = blocks[3].selectors().selectors() else {
        panic!("100% selector")
    };
    assert_eq!(end.literal_value(), Some(100.0));
    assert!(blocks[3].declarations().is_empty());
    point(blocks[0].position(), 1, 0, 1);
}

#[test]
fn invalid_keyframe_selector_units_drop_whole_children_and_keep_the_later_to_block() {
    for selector in ["-1%", "101%", "0", "50%,", ".bad", "fn(a)"] {
        let source = format!("{{{selector}{{opacity:.5}}to{{opacity:1}}}}");
        let report = parse_keyframes_block(&source);
        let [last] = body(&report).body().as_slice() else {
            panic!("later child survives: {selector}")
        };
        assert!(matches!(
            last.selectors().selectors(),
            [CssKeyframeSelector::To]
        ));
        opacity(&last.declarations()[0], "1");
        let diagnostic = recovered(&report, CssRecoveryAction::DropKeyframeBlock);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 1);
        assert!(
            diagnostic.span().end().byte_offset().value()
                <= 1 + selector.len() + "{opacity:.5}".len()
        );
    }
}

#[test]
fn at_rule_is_not_a_keyframe_child_and_does_not_admit_its_nested_to_rule() {
    let report = parse_keyframes_block("{from{opacity:0}@media all{to{opacity:.5}}to{opacity:1}}");
    let blocks = body(&report).body();
    assert_eq!(blocks.len(), 2);
    assert!(matches!(
        blocks[0].selectors().selectors(),
        [CssKeyframeSelector::From]
    ));
    assert!(matches!(
        blocks[1].selectors().selectors(),
        [CssKeyframeSelector::To]
    ));
    opacity(&blocks[0].declarations()[0], "0");
    opacity(&blocks[1].declarations()[0], "1");
    recovered(&report, CssRecoveryAction::DropKeyframeBlock);
}

#[test]
fn keyframe_declarations_retain_duplicates_timing_function_and_custom_occurrences() {
    let report = parse_keyframe_declaration_block(
        "{opacity:.5;opacity:1;animation-timing-function:linear;--x:1;--x:2}",
    );
    clean(&report);
    let declarations = body(&report).body();
    assert_eq!(declarations.len(), 5);
    opacity(&declarations[0], ".5");
    opacity(&declarations[1], "1");
    assert_eq!(
        declarations[2].known().unwrap().property(),
        CssKnownProperty::AnimationTimingFunction
    );
    for declaration in declarations.iter() {
        assert_eq!(declaration.source().importance(), CssImportance::Normal);
        assert_eq!(declaration.source().parser_context(), STANDARDS);
    }
    for declaration in &declarations.as_slice()[3..] {
        assert!(
            matches!(declaration.property_name(), CssPropertyNameRef::Custom(name) if name.as_str() == "--x")
        );
    }
    for (declaration, expected) in declarations.as_slice()[3..].iter().zip(["1", "2"]) {
        let [component] = declaration.source().value_components().items() else {
            panic!("one custom value number")
        };
        assert!(
            matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(value)) if value.representation() == expected)
        );
    }
}

#[test]
fn keyframe_animation_properties_except_timing_function_are_dropped_locally() {
    // Animations1 §3's defining-property exclusions, not a second property catalog.
    for declaration in [
        "animation-name:x",
        "animation-duration:1s",
        "animation-delay:1s",
        "animation-iteration-count:1",
        "animation-direction:normal",
        "animation-fill-mode:none",
        "animation-play-state:running",
        "animation:1s x",
    ] {
        let source = format!("{{opacity:.5;{declaration};color:red}}");
        let report = parse_keyframe_declaration_block(&source);
        assert_eq!(
            keys(body(&report).body()),
            [
                CssPropertyNameRef::Known(CssKnownProperty::Opacity),
                CssPropertyNameRef::Known(CssKnownProperty::Color)
            ]
        );
        opacity(&body(&report).body()[0], ".5");
        recovered(&report, CssRecoveryAction::DropDeclaration);
    }
}

#[test]
fn keyframe_importance_is_invalid_for_known_and_custom_declarations() {
    for declaration in [
        "opacity:1!important",
        "--x:1!important",
        "animation-timing-function:linear!important",
    ] {
        let source = format!("{{opacity:.5;{declaration};color:red}}");
        let report = parse_keyframe_declaration_block(&source);
        assert_eq!(
            keys(body(&report).body()),
            [
                CssPropertyNameRef::Known(CssKnownProperty::Opacity),
                CssPropertyNameRef::Known(CssKnownProperty::Color)
            ]
        );
        recovered(&report, CssRecoveryAction::DropDeclaration);
    }
}

#[test]
fn structural_failure_rejects_direct_keyframe_declarations_but_only_the_owning_outer_child() {
    for child in [".bad{color:red}", "@media all{color:red}"] {
        let source = format!("{{opacity:.5;{child}opacity:1}}");
        rejected(&parse_keyframe_declaration_block(&source), &source);
        let source = format!("{{from{{opacity:.5;{child}opacity:1}}to{{opacity:1}}}}");
        let report = parse_keyframes_block(&source);
        let [last] = body(&report).body().as_slice() else {
            panic!("only later complete child")
        };
        assert!(matches!(
            last.selectors().selectors(),
            [CssKeyframeSelector::To]
        ));
        opacity(&last.declarations()[0], "1");
        recovered(&report, CssRecoveryAction::DropKeyframeBlock);
    }
}

#[test]
fn page_body_retains_five_physical_margin_properties_priority_and_duplicates() {
    let report = parse_page_block(
        "{margin:-1px 2% auto 3cm;margin-top:-4mm!important;margin-right:5in;margin-bottom:6pc;margin-left:7pt;margin-top:0}",
    );
    clean(&report);
    let declarations = body(&report).body();
    assert_eq!(
        page_keys(declarations),
        [
            CssKnownProperty::Margin,
            CssKnownProperty::MarginTop,
            CssKnownProperty::MarginRight,
            CssKnownProperty::MarginBottom,
            CssKnownProperty::MarginLeft,
            CssKnownProperty::MarginTop
        ]
    );
    assert_eq!(declarations[1].importance(), CssImportance::Important);
    assert_eq!(declarations[5].importance(), CssImportance::Normal);
    let CssKnownPropertyValueRef::Margin(value) =
        declarations[0].known().unwrap().property_value().unwrap()
    else {
        panic!("physical margin shorthand")
    };
    let [top, right, bottom, left] = value.value().assigned_values();
    assert_eq!(top.serialize_specified().unwrap(), "-1px");
    assert_eq!(right.serialize_specified().unwrap(), "2%");
    assert!(matches!(bottom, CssMarginValue::Auto));
    assert_eq!(left.serialize_specified().unwrap(), "3cm");
}

#[test]
fn page_css2_whitelist_rejects_other_properties_and_non_css2_margin_values_locally() {
    for invalid in [
        "color:red",
        "width:1px",
        "--x:1",
        "margin-inline-start:1px",
        "margin-top:1em",
        "margin-top:1ex",
        "margin-top:1rem",
        "margin-top:calc(1px + 2%)",
        "margin:logical 1px",
        "margin-top:7",
    ] {
        let source = format!("{{margin-top:1px;{invalid};margin-bottom:auto}}");
        let report = parse_page_block(&source);
        assert_eq!(
            page_keys(body(&report).body()),
            [CssKnownProperty::MarginTop, CssKnownProperty::MarginBottom]
        );
        recovered(&report, CssRecoveryAction::DropDeclaration);
    }
}

#[test]
fn page_css2_excludes_margin_boxes_and_nested_rules_but_retains_later_margin() {
    for child in [
        "@top-left{content:\"title\"}",
        "@media all{margin-top:9px}",
        ".nested{color:blue}",
    ] {
        let source = format!("{{margin-top:1px;{child}margin-bottom:2px}}");
        let report = parse_page_block(&source);
        assert_eq!(
            page_keys(body(&report).body()),
            [CssKnownProperty::MarginTop, CssKnownProperty::MarginBottom]
        );
        recovered(&report, CssRecoveryAction::DropAtRule);
    }
}

#[test]
fn real_empty_bodies_and_recovered_empty_bodies_remain_distinct() {
    let outer = parse_keyframes_block("{}");
    clean(&outer);
    assert!(body(&outer).body().is_empty());
    let child = parse_keyframe_declaration_block("{}");
    clean(&child);
    assert!(body(&child).body().is_empty());
    let page = parse_page_block("{}");
    clean(&page);
    assert!(body(&page).body().is_empty());
    let outer = parse_keyframes_block("{.bad{opacity:.5}}");
    assert!(body(&outer).body().is_empty());
    recovered(&outer, CssRecoveryAction::DropKeyframeBlock);
    let child = parse_keyframe_declaration_block("{bogus:1}");
    assert!(body(&child).body().is_empty());
    recovered(&child, CssRecoveryAction::DropDeclaration);
    let page = parse_page_block("{color:red}");
    assert!(body(&page).body().is_empty());
    recovered(&page, CssRecoveryAction::DropDeclaration);
}

#[test]
fn no_curly_opener_and_trailing_nontrivia_reject_the_entire_fragment() {
    for source in [
        "",
        " \t\r\n",
        "/* empty */",
        "[]",
        "()",
        "opacity:.5",
        "}",
        "{}x",
        "{};",
        "{}{}",
        "{})",
        "{}]",
        "{}}",
    ] {
        rejected(&parse_keyframes_block(source), source);
        rejected(&parse_keyframe_declaration_block(source), source);
        rejected(&parse_page_block(source), source);
    }
}

#[test]
fn nonempty_first_body_is_not_published_after_failed_outer_exhaustion() {
    for tail in ["x", ";", "{}", "}", ")", "]"] {
        let source = format!("{{from{{opacity:.5}}}}{tail}");
        rejected(&parse_keyframes_block(&source), &source);
        let source = format!("{{opacity:.5}}{tail}");
        rejected(&parse_keyframe_declaration_block(&source), &source);
        let source = format!("{{margin-top:1px}}{tail}");
        rejected(&parse_page_block(&source), &source);
    }
}

#[test]
fn trailing_value_token_drops_the_whole_declaration_and_keeps_its_neighbor() {
    let child = parse_keyframe_declaration_block("{opacity:.5 extra;color:red}");
    assert_eq!(
        keys(body(&child).body()),
        [CssPropertyNameRef::Known(CssKnownProperty::Color)]
    );
    recovered(&child, CssRecoveryAction::DropDeclaration);
    let outer = parse_keyframes_block("{from{opacity:.5 extra;color:red}to{opacity:1}}");
    assert_eq!(body(&outer).body().len(), 2);
    assert_eq!(
        keys(body(&outer).body()[0].declarations()),
        [CssPropertyNameRef::Known(CssKnownProperty::Color)]
    );
    opacity(&body(&outer).body()[1].declarations()[0], "1");
    recovered(&outer, CssRecoveryAction::DropDeclaration);
    let page = parse_page_block("{margin-top:1px extra;margin-bottom:2px}");
    assert_eq!(
        page_keys(body(&page).body()),
        [CssKnownProperty::MarginBottom]
    );
    recovered(&page, CssRecoveryAction::DropDeclaration);
}

#[test]
fn genuine_source_envelopes_and_child_occurrences_retain_utf8_utf16_ranges() {
    let source = " /*é*/\r\n{from{--💡:1;opacity:.5}} /*x*/";
    let outer = parse_keyframes_block(source);
    clean(&outer);
    let fragment = body(&outer);
    origin(fragment.origin(), source, 9, 36);
    point(fragment.origin().span().start(), 9, 1, 0);
    point(fragment.origin().span().end(), 36, 1, 25);
    let [block] = fragment.body().as_slice() else {
        panic!("one from block")
    };
    point(block.position(), 10, 1, 1);
    point(block.declarations()[0].position(), 15, 1, 6);
    origin(
        block.declarations()[0].source().parsed_name().unwrap(),
        source,
        15,
        21,
    );
    let declaration = &block.declarations()[1];
    point(declaration.position(), 24, 1, 13);
    origin(declaration.source().parsed_value().unwrap(), source, 32, 34);
    point(
        declaration.source().parsed_value().unwrap().span().start(),
        32,
        1,
        21,
    );
    assert!(
        declaration
            .source()
            .parsed_value()
            .unwrap()
            .source()
            .same_snapshot(fragment.origin().source())
    );

    let source = " /*é*/\r\n{--💡:1;opacity:.5} /*x*/";
    let child = parse_keyframe_declaration_block(source);
    clean(&child);
    let fragment = body(&child);
    origin(fragment.origin(), source, 9, 30);
    point(fragment.origin().span().end(), 30, 1, 19);
    point(fragment.body()[1].position(), 19, 1, 8);
    origin(
        fragment.body()[1].source().parsed_value().unwrap(),
        source,
        27,
        29,
    );
    point(
        fragment.body()[1]
            .source()
            .parsed_value()
            .unwrap()
            .span()
            .start(),
        27,
        1,
        16,
    );
    assert!(
        fragment.body()[0]
            .source()
            .parsed_name()
            .unwrap()
            .source()
            .same_snapshot(fragment.origin().source())
    );

    let source = " /*é*/\r\n{margin-top:1px} /*💡*/ ";
    assert_eq!(source.len(), 35);
    let page = parse_page_block(source);
    clean(&page);
    let fragment = body(&page);
    origin(fragment.origin(), source, 9, 25);
    point(fragment.origin().span().start(), 9, 1, 0);
    point(fragment.origin().span().end(), 25, 1, 16);
    let declaration = &fragment.body()[0];
    point(declaration.position().unwrap(), 10, 1, 1);
    origin(declaration.parsed_name().unwrap(), source, 10, 20);
    origin(declaration.parsed_value().unwrap(), source, 21, 24);
    point(
        declaration.parsed_value().unwrap().span().start(),
        21,
        1,
        12,
    );
    assert!(
        declaration
            .parsed_value()
            .unwrap()
            .source()
            .same_snapshot(fragment.origin().source())
    );
}

#[test]
fn recovery_coordinates_after_a_supplementary_name_use_original_utf16_columns() {
    let source = " /*é*/\r\n{--💡:1;bogus:2;opacity:.5} /*x*/";
    let report = parse_keyframe_declaration_block(source);
    assert_eq!(body(&report).body().len(), 2);
    let diagnostic = recovered(&report, CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    point(diagnostic.error().position(), 19, 1, 8);
    point(diagnostic.span().start(), 19, 1, 8);
    point(diagnostic.span().end(), 27, 1, 16);
    point(body(&report).body()[1].position(), 27, 1, 16);
    opacity(&body(&report).body()[1], ".5");
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
fn implicit_braces_retain_real_empty_or_nonempty_bodies_at_original_eof() {
    implicit(&parse_keyframes_block("{"), "{");
    implicit(&parse_keyframe_declaration_block("{"), "{");
    implicit(&parse_page_block("{"), "{");
    let source = "{from{opacity:.5";
    let outer = parse_keyframes_block(source);
    implicit(&outer, source);
    let [block] = body(&outer).body().as_slice() else {
        panic!("one real opened child")
    };
    opacity(&block.declarations()[0], ".5");
    let source = "{opacity:.5";
    let child = parse_keyframe_declaration_block(source);
    implicit(&child, source);
    opacity(&body(&child).body()[0], ".5");
    let source = "{margin-top:1px";
    let page = parse_page_block(source);
    implicit(&page, source);
    assert_eq!(page_keys(body(&page).body()), [CssKnownProperty::MarginTop]);
}

#[test]
fn retained_function_eof_is_implicit_and_page_math_is_still_outside_its_domain() {
    let source = "{opacity:calc(25%";
    let report = parse_keyframe_declaration_block(source);
    implicit(&report, source);
    let declaration = &body(&report).body()[0];
    let CssKnownPropertyValueRef::Opacity(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed opacity")
    };
    assert!(matches!(
        value.value(),
        CssOpacityValue::PercentageCalculation(_)
    ));
    let [component] = declaration.source().value_components().items() else {
        panic!("one calc function")
    };
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("calc function")
    };
    let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
        panic!("original implicit function close")
    };
    origin(opening, source, 9, 14);
    origin(at, source, source.len(), source.len());
    assert!(at.source().same_snapshot(body(&report).origin().source()));
    let source = "{margin-top:calc(1px";
    let page = parse_page_block(source);
    implicit(&page, source);
    assert!(body(&page).body().is_empty());
    recovered(&page, CssRecoveryAction::DropDeclaration);
}

#[test]
fn comment_eof_recovery_is_independent_of_retained_body_grammar() {
    let source = "{from{opacity:.5}/* unfinished";
    let outer = parse_keyframes_block(source);
    implicit(&outer, source);
    assert_eq!(body(&outer).body().len(), 1);
    assert_eq!(
        recovered(&outer, CssRecoveryAction::IgnoreUnterminatedComment)
            .error()
            .position()
            .byte_offset()
            .value(),
        source.len()
    );
    let source = "{opacity:.5;/* unfinished";
    let child = parse_keyframe_declaration_block(source);
    implicit(&child, source);
    assert_eq!(body(&child).body().len(), 1);
    recovered(&child, CssRecoveryAction::IgnoreUnterminatedComment);
    let source = "{margin-top:1px;/* unfinished";
    let page = parse_page_block(source);
    implicit(&page, source);
    assert_eq!(body(&page).body().len(), 1);
    recovered(&page, CssRecoveryAction::IgnoreUnterminatedComment);
}

#[test]
fn mode_methods_admit_quirky_keyframe_lengths_and_preserve_the_original_number() {
    let source = "{margin-top:7;color:red}";
    let standards = STANDARDS.parse_keyframe_declaration_block(source);
    assert_eq!(standards, parse_keyframe_declaration_block(source));
    assert_eq!(
        keys(body(&standards).body()),
        [CssPropertyNameRef::Known(CssKnownProperty::Color)]
    );
    recovered(&standards, CssRecoveryAction::DropDeclaration);
    let quirks = QUIRKS.parse_keyframe_declaration_block(source);
    clean(&quirks);
    assert_eq!(
        keys(body(&quirks).body()),
        [
            CssPropertyNameRef::Known(CssKnownProperty::MarginTop),
            CssPropertyNameRef::Known(CssKnownProperty::Color)
        ]
    );
    let declaration = body(&quirks).body()[0].source();
    assert_eq!(declaration.parser_context(), QUIRKS);
    let CssKnownPropertyValueRef::MarginTop(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed quirky margin")
    };
    let value = value.value().length_percentage().unwrap();
    assert!(value.is_quirky_length());
    assert_eq!(value.serialize_specified().unwrap(), "7px");
    let [component] = declaration.value_components().items() else {
        panic!("one original number")
    };
    assert_eq!(value.literal_component(), Some(component));
    assert!(
        matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.representation() == "7")
    );
    let CssValueOrigin::Parsed(parsed) = component.origin() else {
        panic!("original source number")
    };
    origin(parsed, source, 12, 13);

    let source = "{from{margin-top:7;color:red}}";
    let standards = STANDARDS.parse_keyframes_block(source);
    assert_eq!(standards, parse_keyframes_block(source));
    assert_eq!(
        keys(body(&standards).body()[0].declarations()),
        [CssPropertyNameRef::Known(CssKnownProperty::Color)]
    );
    recovered(&standards, CssRecoveryAction::DropDeclaration);
    let quirks = QUIRKS.parse_keyframes_block(source);
    clean(&quirks);
    assert_eq!(body(&quirks).body()[0].declarations().len(), 2);
    assert!(
        body(&quirks).body()[0]
            .declarations()
            .iter()
            .all(|declaration| declaration.source().parser_context() == QUIRKS)
    );
}

#[test]
fn page_mode_method_preserves_context_without_bypassing_the_css2_value_filter() {
    let source = "{margin-top:7;margin-bottom:0}";
    let standards = STANDARDS.parse_page_block(source);
    assert_eq!(standards, parse_page_block(source));
    assert_eq!(
        page_keys(body(&standards).body()),
        [CssKnownProperty::MarginBottom]
    );
    recovered(&standards, CssRecoveryAction::DropDeclaration);
    let quirks = QUIRKS.parse_page_block(source);
    assert_eq!(
        page_keys(body(&quirks).body()),
        [CssKnownProperty::MarginBottom]
    );
    recovered(&quirks, CssRecoveryAction::DropDeclaration);
    assert_eq!(body(&quirks).body()[0].parser_context(), QUIRKS);
    for context in [STANDARDS, QUIRKS] {
        let report = context.parse_page_block("{margin-top:7px}");
        clean(&report);
        assert_eq!(
            page_keys(body(&report).body()),
            [CssKnownProperty::MarginTop]
        );
        assert_eq!(body(&report).body()[0].parser_context(), context);
    }
}

fn nesting<T: Clone + std::fmt::Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    source: &str,
    unit_start: usize,
    value_start: usize,
    later_start: usize,
    function_index: usize,
) {
    let diagnostic = recovered(report, CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("native resource identity")
    };
    assert_eq!(detail.limit(), 256);
    let expected = value_start + 2 * function_index;
    point(diagnostic.error().position(), expected, 0, expected as u32);
    assert_eq!(diagnostic.span().start().byte_offset().value(), unit_start);
    assert!(diagnostic.span().end().byte_offset().value() > expected);
    assert!(diagnostic.span().end().byte_offset().value() <= later_start);
    origin(body(report).origin(), source, 0, source.len());
}

#[test]
fn direct_keyframe_declaration_depth_counts_its_real_brace_and_retains_later_declaration() {
    for functions in [255, 256] {
        let prefix = "{opacity:.5;--v:";
        let value = format!("{}x{}", "f(".repeat(functions), ")".repeat(functions));
        let source = format!("{prefix}{value};color:red}}");
        let report = parse_keyframe_declaration_block(&source);
        let declarations = body(&report).body();
        opacity(&declarations[0], ".5");
        if functions == 255 {
            clean(&report);
            assert_eq!(declarations.len(), 3);
            assert!(
                matches!(declarations[1].property_name(), CssPropertyNameRef::Custom(name) if name.as_str() == "--v")
            );
            assert_eq!(
                declarations[2].known().unwrap().property(),
                CssKnownProperty::Color
            );
        } else {
            assert_eq!(
                keys(declarations),
                [
                    CssPropertyNameRef::Known(CssKnownProperty::Opacity),
                    CssPropertyNameRef::Known(CssKnownProperty::Color)
                ]
            );
            nesting(
                &report,
                &source,
                "{opacity:.5;".len(),
                prefix.len(),
                prefix.len() + value.len() + 1,
                255,
            );
        }
    }
}

#[test]
fn outer_keyframes_counts_both_real_braces_and_preserves_child_and_outer_siblings() {
    for functions in [254, 255] {
        let prefix = "{from{opacity:.5;--v:";
        let value = format!("{}x{}", "f(".repeat(functions), ")".repeat(functions));
        let source = format!("{prefix}{value};color:red}}to{{opacity:1}}}}");
        let report = parse_keyframes_block(&source);
        let blocks = body(&report).body();
        assert_eq!(blocks.len(), 2);
        opacity(&blocks[0].declarations()[0], ".5");
        opacity(&blocks[1].declarations()[0], "1");
        if functions == 254 {
            clean(&report);
            assert_eq!(blocks[0].declarations().len(), 3);
        } else {
            assert_eq!(
                keys(blocks[0].declarations()),
                [
                    CssPropertyNameRef::Known(CssKnownProperty::Opacity),
                    CssPropertyNameRef::Known(CssKnownProperty::Color)
                ]
            );
            nesting(
                &report,
                &source,
                "{from{opacity:.5;".len(),
                prefix.len(),
                prefix.len() + value.len() + 1,
                254,
            );
        }
    }
}

#[test]
fn page_descriptor_unit_depth_preserves_native_limit_identity_and_later_margin() {
    for functions in [255, 256] {
        let prefix = "{margin-top:1px;bad:";
        let value = format!("{}x{}", "f(".repeat(functions), ")".repeat(functions));
        let source = format!("{prefix}{value};margin-bottom:2px}}");
        let report = parse_page_block(&source);
        assert_eq!(
            page_keys(body(&report).body()),
            [CssKnownProperty::MarginTop, CssKnownProperty::MarginBottom]
        );
        if functions == 255 {
            assert_eq!(
                recovered(&report, CssRecoveryAction::DropDeclaration)
                    .error()
                    .code(),
                CssErrorCode::UnknownProperty
            );
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .all(|diagnostic| diagnostic.error().code() != CssErrorCode::NestingLimit)
            );
        } else {
            nesting(
                &report,
                &source,
                "{margin-top:1px;".len(),
                prefix.len(),
                prefix.len() + value.len() + 1,
                255,
            );
        }
    }
}

#[test]
fn failed_keyframe_selector_block_keeps_native_resource_identity_and_later_child() {
    for functions in [254, 255] {
        let prefix = "{from{opacity:0}.bad{--v:";
        let value = format!("{}x{}", "f(".repeat(functions), ")".repeat(functions));
        let source = format!("{prefix}{value}}}to{{opacity:1}}}}");
        let report = parse_keyframes_block(&source);
        let blocks = body(&report).body();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(
            blocks[0].selectors().selectors(),
            [CssKeyframeSelector::From]
        ));
        assert!(matches!(
            blocks[1].selectors().selectors(),
            [CssKeyframeSelector::To]
        ));
        if functions == 254 {
            recovered(&report, CssRecoveryAction::DropKeyframeBlock);
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .all(|diagnostic| diagnostic.error().code() != CssErrorCode::NestingLimit)
            );
        } else {
            nesting(
                &report,
                &source,
                "{from{opacity:0}".len(),
                prefix.len(),
                prefix.len() + value.len() + 1,
                254,
            );
        }
    }
}

#[test]
fn genuine_whole_rules_keep_native_top_level_siblings_after_resource_recovery() {
    let keyframe_value = format!("{}x{}", "f(".repeat(255), ")".repeat(255));
    let report = parse_sheet(&format!(
        "@keyframes x{{from{{opacity:.5;--v:{keyframe_value};color:red}}to{{opacity:1}}}}.after{{}}"
    ));
    let [CssRule::Keyframes(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("real keyframes rule and following style")
    };
    assert_eq!(rule.blocks().len(), 2);
    assert_eq!(rule.blocks()[0].declarations().len(), 2);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(
                |diagnostic| diagnostic.error().code() == CssErrorCode::NestingLimit
                    && diagnostic.action() == CssRecoveryAction::StopAtNestingLimit
            )
    );
    let page_value = format!("{}x{}", "f(".repeat(256), ")".repeat(256));
    let report = parse_sheet(&format!(
        "@page{{margin-top:1px;bad:{page_value};margin-bottom:2px}}.after{{}}"
    ));
    let [CssRule::Page(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("real Page rule and following style")
    };
    assert_eq!(
        page_keys(rule.declarations()),
        [CssKnownProperty::MarginTop, CssKnownProperty::MarginBottom]
    );
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
