#![forbid(unsafe_code)]

//! Independent existing-front draft: Env1 WD20250923 §3; Syntax3 CRD20211224
//! §§5.4, 9.2; selected Fonts4 WD20260907 §6.9.1 and Counter Styles3 §3.
//! This file deliberately uses existing names so admission RED can precede a
//! functional phase extension. New phase views/reentry are separate tests.

use surgeist_css::{
    CssComponentValueRef, CssComponentValues, CssCounterStyleDescriptorKind as Counter,
    CssCounterStyleDescriptorRef as CounterOccurrence, CssDescriptorOccurrence, CssErrorCode,
    CssFontFeatureValueKind as Feature, CssFontFeatureValuesItem as Item, CssParseReport,
    CssParsedOrigin, CssRecoveryAction as Action, CssRule, CssValueOrigin,
    parse_counter_style_block, parse_counter_style_descriptor_value,
    parse_font_feature_display_value, parse_font_feature_value_block,
    parse_font_feature_value_indexes, parse_font_feature_values_block, parse_sheet,
};

const COUNTERS: [Counter; 10] = [
    Counter::System,
    Counter::Negative,
    Counter::Symbols,
    Counter::Prefix,
    Counter::Suffix,
    Counter::Range,
    Counter::Pad,
    Counter::Fallback,
    Counter::AdditiveSymbols,
    Counter::SpeakAs,
];
const FEATURES: [Feature; 7] = [
    Feature::Stylistic,
    Feature::HistoricalForms,
    Feature::Styleset,
    Feature::CharacterVariant,
    Feature::Swash,
    Feature::Ornaments,
    Feature::Annotation,
];

fn admitted<T: Clone + std::fmt::Debug>(report: &CssParseReport<Option<T>>) {
    assert!(report.is_clean(), "{report:?}");
    assert!(report.syntax().is_some(), "{report:?}");
    assert!(report.clone().into_validation_result().unwrap().is_some());
}

fn rejected<T: Clone + std::fmt::Debug>(report: CssParseReport<Option<T>>) {
    assert!(report.syntax().is_none(), "{report:?}");
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == Action::RejectInput)
    );
    assert!(report.into_validation_result().is_err());
}

fn whole(origin: &CssParsedOrigin, source: &str) {
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), source.len());
}

fn parsed_span(
    origin: &CssParsedOrigin,
    source: &str,
    start: (usize, u32, u32),
    end: (usize, u32, u32),
) {
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start.0);
    assert_eq!(origin.span().start().line().value(), start.1);
    assert_eq!(origin.span().start().column().value(), start.2);
    assert_eq!(origin.span().end().byte_offset().value(), end.0);
    assert_eq!(origin.span().end().line().value(), end.1);
    assert_eq!(origin.span().end().column().value(), end.2);
}

fn simple_env(components: &CssComponentValues, source: &str) {
    let [component] = components.items() else {
        panic!("one retained function")
    };
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("actual env function")
    };
    assert_eq!(function.name(), "env");
    let CssValueOrigin::Parsed(origin) = component.origin() else {
        panic!("actual source")
    };
    // The component origin owns "env("; the value carrier owns the whole input.
    parsed_span(origin, source, (0, 0, 0), (4, 0, 4));
    let CssValueOrigin::Parsed(closing) = function.closing_origin() else {
        panic!("authored closer")
    };
    parsed_span(closing, source, (13, 0, 13), (14, 0, 14));
    assert!(origin.source().same_snapshot(closing.source()));
}

fn record<T>(record: &CssDescriptorOccurrence<T>, source: &str, name: &str) {
    let origin = record.parsed_name().expect("actual descriptor name");
    // This literal ASCII body gives a unique name followed by colon, with no
    // value trivia. Locate that expected text in the stimulus, not parser output.
    let name_start = source.find(&format!("{name}:")).unwrap();
    let name_end = name_start + name.len();
    parsed_span(
        origin,
        source,
        (name_start, 0, name_start as u32),
        (name_end, 0, name_end as u32),
    );
    let value = record.parsed_value().expect("actual value region");
    let value_start = name_end + 1;
    let value_end = value_start + 14;
    parsed_span(
        value,
        source,
        (value_start, 0, value_start as u32),
        (value_end, 0, value_end as u32),
    );
    assert_eq!(&source[value_start..value_end], "env(choice, 7)");
    assert!(origin.source().same_snapshot(value.source()));
    let [component] = record.value_components().unwrap().items() else {
        panic!("one actual env component")
    };
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("actual env function")
    };
    let CssValueOrigin::Parsed(opening) = component.origin() else {
        panic!("actual opener")
    };
    parsed_span(
        opening,
        source,
        (value_start, 0, value_start as u32),
        (value_start + 4, 0, (value_start + 4) as u32),
    );
    let CssValueOrigin::Parsed(closing) = function.closing_origin() else {
        panic!("actual closer")
    };
    parsed_span(
        closing,
        source,
        (value_end - 1, 0, (value_end - 1) as u32),
        (value_end, 0, value_end as u32),
    );
    assert!(value.source().same_snapshot(opening.source()));
    assert!(value.source().same_snapshot(closing.source()));
}

#[test]
fn every_counter_descriptor_admits_env_and_keeps_its_value_window() {
    for kind in COUNTERS {
        let report = parse_counter_style_descriptor_value("env(choice, 7)", kind);
        admitted(&report);
        let value = report.syntax().as_ref().unwrap();
        assert_eq!(value.kind(), kind);
        whole(value.origin(), "env(choice, 7)");
        simple_env(value.components(), "env(choice, 7)");
    }
}

#[test]
fn outer_font_display_admits_env_without_an_ordinary_keyword() {
    let report = parse_font_feature_display_value("env(choice, 7)");
    admitted(&report);
    let value = report.syntax().as_ref().unwrap();
    whole(value.origin(), "env(choice, 7)");
    simple_env(value.components(), "env(choice, 7)");
}

#[test]
fn every_subsidiary_value_admits_env_without_completed_indexes() {
    for kind in FEATURES {
        let report = parse_font_feature_value_indexes("env(choice, 7)", kind);
        admitted(&report);
        let value = report.syntax().as_ref().unwrap();
        assert_eq!(value.kind(), kind);
        whole(value.origin(), "env(choice, 7)");
        simple_env(value.components(), "env(choice, 7)");
    }
}

#[test]
fn counter_body_retains_every_pending_descriptor_with_real_names_and_values() {
    let source = "{system:env(choice, 7);negative:env(choice, 7);symbols:env(choice, 7);prefix:env(choice, 7);suffix:env(choice, 7);range:env(choice, 7);pad:env(choice, 7);fallback:env(choice, 7);additive-symbols:env(choice, 7);speak-as:env(choice, 7)}";
    let report = parse_counter_style_block(source);
    admitted(&report);
    let fragment = report.syntax().as_ref().unwrap();
    whole(fragment.origin(), source);
    let occurrences = fragment.body().occurrences().collect::<Vec<_>>();
    let [
        CounterOccurrence::System(a),
        CounterOccurrence::Negative(b),
        CounterOccurrence::Symbols(c),
        CounterOccurrence::Prefix(d),
        CounterOccurrence::Suffix(e),
        CounterOccurrence::Range(f),
        CounterOccurrence::Pad(g),
        CounterOccurrence::Fallback(h),
        CounterOccurrence::AdditiveSymbols(i),
        CounterOccurrence::SpeakAs(j),
    ] = occurrences.as_slice()
    else {
        panic!("all ten actual ordered occurrences")
    };
    record(a, source, "system");
    record(b, source, "negative");
    record(c, source, "symbols");
    record(d, source, "prefix");
    record(e, source, "suffix");
    record(f, source, "range");
    record(g, source, "pad");
    record(h, source, "fallback");
    record(i, source, "additive-symbols");
    record(j, source, "speak-as");
    let sheet_source = format!("@counter-style demo{source}tail{{}}");
    let sheet = parse_sheet(&sheet_source);
    assert!(sheet.is_clean(), "{sheet:?}");
    let [CssRule::CounterStyle(rule), CssRule::Style(_)] = sheet.syntax().rules() else {
        panic!("counter and later sibling")
    };
    assert_eq!(rule.descriptors().occurrences().count(), 10);
}

#[test]
fn mixed_font_body_retains_outer_display_and_all_seven_named_definition_owners() {
    let source = "{font-display:env(choice);@stylistic{Pick:env(choice)}@historical-forms{Pick:env(choice)}@styleset{Pick:env(choice)}@character-variant{Pick:env(choice)}@swash{Pick:env(choice)}@ornaments{Pick:env(choice)}@annotation{Pick:env(choice)}}";
    let report = parse_font_feature_values_block(source);
    admitted(&report);
    let fragment = report.syntax().as_ref().unwrap();
    whole(fragment.origin(), source);
    assert_eq!(fragment.body().len(), 8);
    assert!(matches!(fragment.body()[0], Item::FontDisplay(_)));
    for (item, kind) in fragment.body()[1..].iter().zip(FEATURES) {
        let Item::Block(block) = item else {
            panic!("actual subsidiary")
        };
        assert_eq!(block.kind(), kind);
        let [definition] = block.definitions() else {
            panic!("one pending named value")
        };
        assert_eq!(definition.name().as_str(), "Pick");
        assert!(definition.position().is_some());
    }
    let sheet_source = format!("@font-feature-values Demo{source}tail{{}}");
    let sheet = parse_sheet(&sheet_source);
    assert!(sheet.is_clean(), "{sheet:?}");
    let [CssRule::FontFeatureValues(rule), CssRule::Style(_)] = sheet.syntax().rules() else {
        panic!("features and later sibling")
    };
    assert_eq!(rule.items().len(), 8);
}

#[test]
fn standalone_subsidiary_bodies_retain_pending_names_in_every_kind() {
    for kind in FEATURES {
        let report = parse_font_feature_value_block("{Pick:env(choice)}", kind);
        admitted(&report);
        let block = report.syntax().as_ref().unwrap().body();
        assert_eq!(block.kind(), kind);
        assert!(
            block.position().is_none(),
            "no fabricated subsidiary at-keyword"
        );
        let [definition] = block.definitions() else {
            panic!("one pending named value")
        };
        assert_eq!(definition.name().as_str(), "Pick");
        assert_eq!(definition.position().unwrap().byte_offset().value(), 1);
    }
}

#[test]
fn valid_env_defers_the_whole_value_including_nested_and_residual_var_tokens() {
    // Invalid ordinary grammar and unresolved var tokens remain authored until
    // reentry; a malformed different family does not invalidate valid env.
    for source in [
        "bogus env(choice)",
        "f(env(choice, 7))",
        "env(choice) var(--later)",
        "env(choice) var()",
        "env(choice){}", // Matched blocks remain declaration-value data.
    ] {
        admitted(&parse_counter_style_descriptor_value(
            source,
            Counter::Range,
        ));
        admitted(&parse_font_feature_display_value(source));
        admitted(&parse_font_feature_value_indexes(
            source,
            Feature::Stylistic,
        ));
    }
}

#[test]
fn malformed_env_family_and_var_only_do_not_gain_descriptor_permission() {
    for source in ["env()", "env(123)", "env(choice) env()", "var(--later)"] {
        rejected(parse_counter_style_descriptor_value(source, Counter::Range));
        rejected(parse_font_feature_display_value(source));
        rejected(parse_font_feature_value_indexes(source, Feature::Stylistic));
    }
}

#[test]
fn root_annotations_and_outer_exhaustion_are_checked_before_env_deferral() {
    for source in ["env(choice)!important", "env(choice);", "env(choice)}"] {
        rejected(parse_counter_style_descriptor_value(
            source,
            Counter::Prefix,
        ));
        rejected(parse_font_feature_display_value(source));
        rejected(parse_font_feature_value_indexes(source, Feature::Styleset));
    }
    // Fallback itself is declaration-value: its root bang is forbidden, whereas
    // a bang inside a nested function is neither root importance nor that ban.
    rejected(parse_counter_style_descriptor_value(
        "env(choice, !important)",
        Counter::Prefix,
    ));
    rejected(parse_font_feature_display_value("env(choice, !important)"));
    rejected(parse_font_feature_value_indexes(
        "env(choice, !important)",
        Feature::Styleset,
    ));
    admitted(&parse_counter_style_descriptor_value(
        "env(choice, f(!important))",
        Counter::Prefix,
    ));
    admitted(&parse_font_feature_display_value(
        "env(choice, f(!important))",
    ));
    admitted(&parse_font_feature_value_indexes(
        "env(choice, f(!important))",
        Feature::Styleset,
    ));
}

#[test]
fn invalid_descriptor_annotations_drop_only_the_child_and_keep_later_env() {
    let counter = parse_counter_style_block("{prefix:env(choice)!important;suffix:env(later)}");
    assert!(!counter.is_clean());
    assert!(
        counter
            .diagnostics()
            .iter()
            .any(|d| d.action() == Action::DropDescriptor)
    );
    let records = counter
        .syntax()
        .as_ref()
        .unwrap()
        .body()
        .occurrences()
        .collect::<Vec<_>>();
    assert!(matches!(records.as_slice(), [CounterOccurrence::Suffix(_)]));
    assert!(counter.into_validation_result().is_err());
    let outer = parse_font_feature_values_block(
        "{font-display:env(choice)!important;font-display:env(later)}",
    );
    assert!(!outer.is_clean());
    assert!(
        outer
            .diagnostics()
            .iter()
            .any(|d| d.action() == Action::DropDescriptor)
    );
    assert!(matches!(
        outer.syntax().as_ref().unwrap().body().as_slice(),
        [Item::FontDisplay(_)]
    ));
    assert!(outer.into_validation_result().is_err());
    let child = parse_font_feature_value_block(
        "{bad:env(choice)!important;Keep:env(later)}",
        Feature::Styleset,
    );
    assert!(!child.is_clean());
    assert!(
        child
            .diagnostics()
            .iter()
            .any(|d| d.action() == Action::DropDescriptor)
    );
    let [keep] = child.syntax().as_ref().unwrap().body().definitions() else {
        panic!("later child survives")
    };
    assert_eq!(keep.name().as_str(), "Keep");
    assert!(child.into_validation_result().is_err());
}

fn implicit<T: Clone + std::fmt::Debug>(report: CssParseReport<Option<T>>) {
    assert!(report.syntax().is_some(), "{report:?}");
    assert!(!report.is_clean());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == Action::RetainWithImplicitClosure)
    );
    assert!(report.into_validation_result().is_err());
}

#[test]
fn implicit_source_env_closure_retains_the_candidate_but_fails_clean_validation() {
    fn check(components: &CssComponentValues, value_origin: &CssParsedOrigin) {
        whole(value_origin, "env(choice");
        let [component] = components.items() else {
            panic!("one implicit env")
        };
        let CssComponentValueRef::Function(function) = component.view() else {
            panic!("env function")
        };
        let CssValueOrigin::Parsed(opener) = component.origin() else {
            panic!("authored opener")
        };
        parsed_span(opener, "env(choice", (0, 0, 0), (4, 0, 4));
        let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
            panic!("EOF implied closer")
        };
        parsed_span(opening, "env(choice", (0, 0, 0), (4, 0, 4));
        parsed_span(at, "env(choice", (10, 0, 10), (10, 0, 10));
        assert!(value_origin.source().same_snapshot(opening.source()));
        assert!(value_origin.source().same_snapshot(at.source()));
    }
    let counter = parse_counter_style_descriptor_value("env(choice", Counter::Prefix);
    assert!(counter.syntax().is_some(), "{counter:?}");
    let value = counter.syntax().as_ref().unwrap();
    check(value.components(), value.origin());
    implicit(counter);
    let display = parse_font_feature_display_value("env(choice");
    assert!(display.syntax().is_some(), "{display:?}");
    let value = display.syntax().as_ref().unwrap();
    check(value.components(), value.origin());
    implicit(display);
    let indexes = parse_font_feature_value_indexes("env(choice", Feature::Styleset);
    assert!(indexes.syntax().is_some(), "{indexes:?}");
    let value = indexes.syntax().as_ref().unwrap();
    check(value.components(), value.origin());
    implicit(indexes);
}

#[test]
fn unicode_trivia_keeps_the_whole_window_and_real_env_utf16_coordinates() {
    let source = " /*😀*/\r\nenv(choice, 7) /*é*/ ";
    fn check(components: &CssComponentValues, origin: &CssParsedOrigin, source: &str) {
        whole(origin, source);
        assert_eq!(origin.span().end().byte_offset().value(), 33);
        assert_eq!(origin.span().end().line().value(), 1);
        assert_eq!(origin.span().end().column().value(), 21);
        let component = components
            .items()
            .iter()
            .find(|c| matches!(c.view(), CssComponentValueRef::Function(_)))
            .unwrap();
        let CssComponentValueRef::Function(function) = component.view() else {
            panic!("actual env function")
        };
        let CssValueOrigin::Parsed(opening) = component.origin() else {
            panic!("actual env opener")
        };
        parsed_span(opening, source, (11, 1, 0), (15, 1, 4));
        let CssValueOrigin::Parsed(closing) = function.closing_origin() else {
            panic!("actual env closer")
        };
        parsed_span(closing, source, (24, 1, 13), (25, 1, 14));
        assert!(origin.source().same_snapshot(opening.source()));
        assert!(origin.source().same_snapshot(closing.source()));
    }
    let counter = parse_counter_style_descriptor_value(source, Counter::Prefix);
    admitted(&counter);
    let value = counter.syntax().as_ref().unwrap();
    check(value.components(), value.origin(), source);
    let display = parse_font_feature_display_value(source);
    admitted(&display);
    let value = display.syntax().as_ref().unwrap();
    check(value.components(), value.origin(), source);
    let indexes = parse_font_feature_value_indexes(source, Feature::Styleset);
    admitted(&indexes);
    let value = indexes.syntax().as_ref().unwrap();
    check(value.components(), value.origin(), source);
}

#[test]
fn actual_256_openers_admit_env_and_257_preserves_resource_precedence() {
    fn check<T: Clone + std::fmt::Debug>(report: CssParseReport<Option<T>>, depth: usize) {
        if depth == 256 {
            admitted(&report);
        } else {
            assert!(report.syntax().is_none(), "{report:?}");
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|d| d.error().code() == CssErrorCode::NestingLimit)
            );
            assert!(report.into_validation_result().is_err());
        }
    }
    for depth in [256, 257] {
        // env contributes one opener; the fallback contributes depth - 1.
        let source = format!(
            "env(choice,{}x{})",
            "f(".repeat(depth - 1),
            ")".repeat(depth - 1)
        );
        check(
            parse_counter_style_descriptor_value(&source, Counter::Prefix),
            depth,
        );
        check(parse_font_feature_display_value(&source), depth);
        check(
            parse_font_feature_value_indexes(&source, Feature::Styleset),
            depth,
        );
    }
}

#[test]
fn real_rules_select_later_pending_occurrences_and_keep_specified_output_symbolic() {
    let report = parse_sheet("@counter-style demo{prefix:\"old\";prefix:env(choice)}tail{}");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("counter plus real sibling")
    };
    assert_eq!(rule.descriptors().occurrences().count(), 2);
    assert_eq!(
        rule.descriptor_specified_css(Counter::Prefix).unwrap(),
        "env(choice)"
    );
    let report = parse_sheet(
        "@font-feature-values Demo{font-display:swap;font-display:env(display);@styleset{Pick:1;Pick:env(index)}}tail{}",
    );
    assert!(report.is_clean(), "{report:?}");
    let [
        rule @ CssRule::FontFeatureValues(features),
        CssRule::Style(_),
    ] = report.syntax().rules()
    else {
        panic!("font features plus real sibling")
    };
    assert_eq!(features.items().len(), 3);
    let Item::Block(block) = &features.items()[2] else {
        panic!("actual styleset")
    };
    assert_eq!(block.definitions().len(), 2);
    assert_eq!(
        rule.to_specified_css().unwrap(),
        "@font-feature-values Demo { font-display: env(display); @styleset { Pick: env(index); } }"
    );
    assert!(report.into_validation_result().is_ok());
}
