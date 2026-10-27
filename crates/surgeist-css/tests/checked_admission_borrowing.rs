#![forbid(unsafe_code)]

//! Public checked admission must pay for retained output, not discarded graphs.
//! This standalone target has no test-harness threads sharing the allocator.
//! Fixtures, input clones, assertions, reporting and destruction are unmeasured.

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use std::alloc::System;
use std::hint::black_box;
use surgeist_css::component::{
    CssBlockKind, CssComponentValue, CssComponentValueRef, CssComponentValues, CssSerializedOrigin,
    CssValueOrigin,
};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssTokenKind};
use surgeist_css::property::{
    CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef, CssPropertyTarget,
    CssSvgGlyphOrientationVerticalValue,
};
use surgeist_css::query::media::{
    CssMediaLeaf, CssMediaRangeRef, CssUnknownMediaFeatureReason, CssUnknownMediaFeatureRef,
};
use surgeist_css::query::{CssCondition, CssConditionKind};
use surgeist_css::values::transform::CssRotate;
use surgeist_css::{CssParser, CssParserContext, CssParserMode};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const SMALL: usize = 128;
const LARGE: usize = 65_536;
const PAYLOAD_GROWTH: usize = LARGE - SMALL;
// Only fixed-size metadata may differ in these equal-component-count fixtures.
// This is much smaller than one enlarged comment, and is not a zero-allocation claim.
const METADATA_ALLOWANCE: usize = 1_024;

fn requested_bytes(stats: Stats) -> usize {
    stats
        .bytes_allocated
        .checked_add(usize::try_from(stats.bytes_reallocated).unwrap_or(0))
        .expect("bounded fixture allocation arithmetic")
}

fn measured<T>(operation: impl FnOnce() -> T) -> (T, Stats) {
    let region = Region::new(GLOBAL);
    let result = black_box(operation());
    let stats = region.change();
    (result, stats)
}

fn growth(name: &str, mut sample: impl FnMut(usize) -> Stats) -> usize {
    let _ = sample(SMALL);
    let _ = sample(LARGE);
    let small = sample(SMALL);
    let large = sample(LARGE);
    let growth = requested_bytes(large).saturating_sub(requested_bytes(small));
    println!(
        "{name}: payload {SMALL} -> {LARGE}; small={small:?}; large={large:?}; requested-byte growth={growth}; semantic/provenance assertions passed"
    );
    growth
}

fn budget(failures: &mut Vec<String>, name: &str, growth: usize, allowance: usize) {
    println!("{name}: observed growth={growth}; independently allowed growth={allowance}");
    if growth > allowance {
        failures.push(format!(
            "{name}: copied discarded payload: {growth} > {allowance}"
        ));
    }
}

fn assert_same_snapshot(actual: &CssValueOrigin, expected: &CssValueOrigin) {
    assert_eq!(actual, expected);
    match (actual, expected) {
        (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) => {
            assert!(actual.source().same_snapshot(expected.source()));
        }
        (CssValueOrigin::Programmatic, CssValueOrigin::Programmatic) => {}
        _ => panic!("expected actual parsed or programmatic origin"),
    }
}

fn children(value: &CssComponentValue) -> &CssComponentValues {
    match value.view() {
        CssComponentValueRef::Function(value) => value.values(),
        CssComponentValueRef::Block(value) => value.values(),
        _ => panic!("fixture enclosure"),
    }
}

fn media_group_sample(payload: usize, grouped: bool) -> Stats {
    let enclosed = format!("Future(/*{}*/)", "x".repeat(payload));
    let source = if grouped {
        format!("({enclosed})")
    } else {
        enclosed.clone()
    };
    let input = CssComponentValues::parse(&CssParser::default(), &source).unwrap();
    let expected = if grouped {
        &children(&input.items()[0]).items()[0]
    } else {
        &input.items()[0]
    };
    let expected_origin = expected.origin().clone();
    let (result, stats) = measured(|| {
        CssCondition::<CssMediaLeaf>::try_from_components(black_box(input), Default::default())
    });
    let condition = result.expect("explicit group around a valid general-enclosed function");
    let inner = if grouped {
        let CssConditionKind::Parenthesized(inner) = condition.kind() else {
            panic!("the group is preserved");
        };
        inner.as_ref()
    } else {
        &condition
    };
    let CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(enclosed_value)) = inner.kind() else {
        panic!("the future function remains opaque");
    };
    assert_same_snapshot(enclosed_value.component().origin(), &expected_origin);
    assert_eq!(
        enclosed_value
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        enclosed
    );
    assert_eq!(
        condition
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        source
    );
    stats
}

fn media_value_sample(payload: usize, inside: bool) -> Stats {
    let comment = format!("/*{}*/", "x".repeat(payload));
    let source = if inside {
        format!("(future: calc(1{comment}))")
    } else {
        format!("Future({comment})")
    };
    let input = CssComponentValues::parse(&CssParser::default(), &source).unwrap();
    let expected_origin = input.items()[0].origin().clone();
    let expected_function = if inside {
        children(&input.items()[0]).items().last().unwrap()
    } else {
        &input.items()[0]
    }
    .clone();
    let (result, stats) = measured(|| {
        CssCondition::<CssMediaLeaf>::try_from_components(black_box(input), Default::default())
    });
    let condition = result.expect("complete feature or opaque function");
    if !inside {
        let CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(enclosed)) = condition.kind()
        else {
            panic!("the control retains its opaque future function");
        };
        assert_same_snapshot(enclosed.component().origin(), &expected_origin);
        assert_eq!(enclosed.component(), &expected_function);
        assert_eq!(
            condition
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            source
        );
        return stats;
    }
    let CssConditionKind::Leaf(CssMediaLeaf::UnknownFeature(feature)) = condition.kind() else {
        panic!("unknown feature has an explicit semantic result");
    };
    assert_eq!(feature.name(), "future");
    assert_eq!(feature.reason(), CssUnknownMediaFeatureReason::UnknownName);
    assert_same_snapshot(feature.component().origin(), &expected_origin);
    let CssUnknownMediaFeatureRef::Range(range) = feature.view() else {
        panic!("plain range");
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("plain range");
    };
    assert_same_snapshot(value.items()[0].origin(), expected_function.origin());
    assert_eq!(value.items(), &[expected_function]);
    let expected = format!("calc(1{comment})");
    assert_eq!(
        value.serialize_tokens(Default::default()).unwrap().as_css(),
        expected
    );
    assert_eq!(
        condition
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        source
    );
    stats
}

fn rejected_sample(payload: usize, property: CssKnownProperty, inside: bool) -> Stats {
    // Incompatible dimensions reject the property. Rate attributes its failed
    // percentage root to calc(; the other domains identify 1s. The comment is
    // trivia and cannot become successful output or retained error content.
    let first = match property {
        CssKnownProperty::GridTemplateColumns | CssKnownProperty::GridAutoColumns => "1fr",
        CssKnownProperty::GridRowStart => "1",
        CssKnownProperty::VoiceRate => "1%",
        CssKnownProperty::VoicePitch | CssKnownProperty::VoiceRange => "1Hz",
        _ => unreachable!("bounded case inventory"),
    };
    let comment = format!("/*{}*/", "x".repeat(payload));
    // The same complete declaration text includes the payload in both cases.
    // A grammar-valid companion keeps outside trivia within the authored range
    // without selecting an unrelated duplicate-offset or trailing-token error.
    let companion = match property {
        CssKnownProperty::VoicePitch | CssKnownProperty::VoiceRange => "high",
        CssKnownProperty::VoiceRate => "fast",
        CssKnownProperty::GridRowStart => "name",
        _ => "0",
    };
    let source = if inside {
        format!("calc({first} + {comment}1s) {companion}")
    } else {
        format!("calc({first} + 1s){comment} {companion}")
    };
    let input = CssComponentValues::parse(&CssParser::default(), &source).unwrap();
    let rate = property == CssKnownProperty::VoiceRate;
    let expected = if rate {
        input.items()[0].origin()
    } else {
        children(&input.items()[0]).items().last().unwrap().origin()
    }
    .clone();
    let (result, stats) = measured(|| {
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
            black_box(input),
            CssImportance::Important,
        )
    });
    let error = result.expect_err("a time cannot be added to this property's numeric domain");
    assert_eq!(
        error.code(),
        Some(CssErrorCode::InvalidPropertyValue),
        "{property:?}: contextual numeric rejection: {error:?}"
    );
    assert_eq!(error.known_property(), Some(property));
    let token = error
        .encountered()
        .expect("the incompatible dimension is present");
    assert_eq!(
        token.kind(),
        if rate {
            CssTokenKind::Function
        } else {
            CssTokenKind::Dimension
        },
        "{property:?}: {error:?}"
    );
    assert_eq!(token.authored(), if rate { "calc(" } else { "1s" });
    let origin = error.origin();
    let CssSerializedOrigin::Token(actual) = &origin else {
        panic!("responsible literal");
    };
    assert_same_snapshot(actual, &expected);
    let CssValueOrigin::Parsed(actual) = actual else {
        panic!("parsed fixture origin");
    };
    let start = actual.span().start().byte_offset().value();
    let end = actual.span().end().byte_offset().value();
    assert_eq!(
        &actual.source().as_str()[start..end],
        if rate { "calc(" } else { "1s" }
    );
    assert_eq!(start, if rate { 0 } else { source.find("1s").unwrap() });
    stats
}

fn conflicting_speech_sample(payload: usize, property: CssKnownProperty, inside: bool) -> Stats {
    let comment = format!("/*{}*/", "x".repeat(payload));
    let source = if inside {
        format!("high calc(1Hz{comment}) absolute")
    } else {
        format!("high {comment}calc(1Hz) absolute")
    };
    let input = CssComponentValues::parse(&CssParser::default(), &source).unwrap();
    let expected = input.items().last().unwrap().origin().clone();
    let (result, stats) = measured(|| {
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
            black_box(input),
            CssImportance::Normal,
        )
    });
    let error = result.expect_err("a relative level cannot be combined with absolute");
    assert_eq!(
        error.code(),
        Some(CssErrorCode::InvalidPropertyValue),
        "the conflicting absolute keyword is the present-token rejection: {error:?}"
    );
    assert_eq!(error.known_property(), Some(property));
    let token = error
        .encountered()
        .expect("the conflicting absolute keyword is present");
    assert_eq!(token.kind(), CssTokenKind::Ident);
    assert_eq!(token.authored(), "absolute");
    let origin = error.origin();
    let CssSerializedOrigin::Token(actual) = &origin else {
        panic!("original conflicting keyword");
    };
    assert_same_snapshot(actual, &expected);
    stats
}

const SVG: CssParserContext =
    CssParserContext::new(CssParserMode::Standards).with_svg_glyph_orientation_vertical();

fn svg_sample(payload: usize, svg: bool) -> Stats {
    let source = format!("calc(90deg/*{}*/)", "x".repeat(payload));
    let input = CssComponentValues::parse(&CssParser::default(), &source).unwrap();
    let expected_origin = input.items()[0].origin().clone();
    let (result, stats) = measured(|| {
        CssDeclaration::try_from_components(
            &CssParser::new(SVG, Default::default()),
            CssPropertyTarget::Property(if svg {
                CssPropertyNameRef::SvgGlyphOrientationVertical
            } else {
                CssPropertyNameRef::Known(CssKnownProperty::Rotate)
            }),
            black_box(input),
            CssImportance::Normal,
        )
    });
    let declaration = result.expect("angle calculation is valid in the SVG grammar");
    let angle = if svg {
        declaration
            .svg_glyph_orientation_vertical()
            .unwrap()
            .value()
            .unwrap()
            .angle()
            .unwrap()
    } else {
        let CssKnownPropertyValueRef::Rotate(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("angle rotation control");
        };
        let CssRotate::Value(value) = value.value() else {
            panic!("angle-only rotation");
        };
        assert!(value.axis().is_none());
        value.angle()
    };
    let calculation = angle.calculation().unwrap();
    assert_eq!(
        declaration
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        source
    );
    assert_eq!(
        calculation
            .components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        source
    );
    assert_same_snapshot(
        calculation.components().items()[0].origin(),
        &expected_origin,
    );
    assert_eq!(
        CssSvgGlyphOrientationVerticalValue::try_from_angle(angle.clone())
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "calc(90deg)"
    );
    stats
}

fn retained_mixed_svg_outputs_survive_input_and_sibling_drop() {
    let (first, second, original) = {
        let source = String::from("/*😀*/90deg");
        let parsed = CssComponentValues::parse(&CssParser::default(), &source).unwrap();
        let leaf = parsed.items().last().unwrap().clone();
        let original = leaf.origin().clone();
        let input = CssComponentValues::try_new(
            vec![
                CssComponentValue::try_function(
                    "cAlC",
                    CssComponentValues::try_new(vec![leaf], Default::default()).unwrap(),
                )
                .unwrap(),
            ],
            Default::default(),
        )
        .unwrap();
        let first = CssDeclaration::try_from_components(
            &CssParser::new(SVG, Default::default()),
            CssPropertyTarget::Property(CssPropertyNameRef::SvgGlyphOrientationVertical),
            input.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        let second = CssDeclaration::try_from_components(
            &CssParser::new(SVG, Default::default()),
            CssPropertyTarget::Property(CssPropertyNameRef::SvgGlyphOrientationVertical),
            input,
            CssImportance::Normal,
        )
        .unwrap();
        (first, second, original)
    };
    drop(first);
    let value = second
        .svg_glyph_orientation_vertical()
        .unwrap()
        .value()
        .unwrap();
    let components = value.angle().unwrap().calculation().unwrap().components();
    assert!(matches!(
        components.items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    let leaf = &children(&components.items()[0]).items()[0];
    assert_same_snapshot(leaf.origin(), &original);
    let CssValueOrigin::Parsed(origin) = leaf.origin() else {
        panic!("original parsed leaf");
    };
    assert_eq!(origin.span().start().byte_offset().value(), 8);
    assert_eq!(
        components
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "cAlC(90deg)"
    );
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "calc(90deg)"
    );
}

fn retained_mixed_media_outputs_survive_input_and_sibling_drop() {
    let (first, second, original) = {
        let source = String::from("/*😀*/1e2");
        let parsed = CssComponentValues::parse(&CssParser::default(), &source).unwrap();
        let leaf = parsed.items().last().unwrap().clone();
        let original = leaf.origin().clone();
        let function = CssComponentValue::try_function(
            "Future",
            CssComponentValues::try_new(vec![leaf], Default::default()).unwrap(),
        )
        .unwrap();
        let group = CssComponentValue::try_block(
            CssBlockKind::Parenthesis,
            CssComponentValues::try_new(vec![function], Default::default()).unwrap(),
        )
        .unwrap();
        let input = CssComponentValues::try_new(vec![group], Default::default()).unwrap();
        (
            CssCondition::<CssMediaLeaf>::try_from_components(input.clone(), Default::default())
                .unwrap(),
            CssCondition::<CssMediaLeaf>::try_from_components(input, Default::default()).unwrap(),
            original,
        )
    };
    drop(first);
    let CssConditionKind::Parenthesized(inner) = second.kind() else {
        panic!("explicit group");
    };
    let CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(enclosed)) = inner.kind() else {
        panic!("future function");
    };
    assert!(matches!(second.origin(), CssValueOrigin::Programmatic));
    assert!(matches!(
        enclosed.component().origin(),
        CssValueOrigin::Programmatic
    ));
    let leaf = &children(enclosed.component()).items()[0];
    assert_same_snapshot(leaf.origin(), &original);
    let CssValueOrigin::Parsed(origin) = leaf.origin() else {
        panic!("original parsed leaf");
    };
    assert_eq!(origin.span().start().byte_offset().value(), 8);
    assert_eq!(
        second
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "(Future(1e2))"
    );
}

fn main() {
    retained_mixed_svg_outputs_survive_input_and_sibling_drop();
    retained_mixed_media_outputs_survive_input_and_sibling_drop();
    let mut failures = Vec::new();
    let direct = growth("opaque-media-direct", |payload| {
        media_group_sample(payload, false)
    });
    let grouped = growth("opaque-media-group", |payload| {
        media_group_sample(payload, true)
    });
    // Both results retain precisely the same opaque function. Grouping adds only
    // outer origin/closing metadata, so it cannot require another payload copy.
    budget(
        &mut failures,
        "media-group-observation",
        grouped.saturating_sub(direct),
        METADATA_ALLOWANCE,
    );
    let opaque = growth("media-number-opaque-control", |payload| {
        media_value_sample(payload, false)
    });
    let inside = growth("media-number-comment-inside", |payload| {
        media_value_sample(payload, true)
    });
    // Both canonical outputs emit the payload once, cancelling existing
    // serialization costs. The opaque control retains one function graph;
    // the feature additionally retains its public value-range graph, including
    // the comment's decoded content and exact spelling.
    budget(
        &mut failures,
        "media-number-classification",
        inside.saturating_sub(opaque),
        2 * PAYLOAD_GROWTH + METADATA_ALLOWANCE,
    );
    for property in [
        CssKnownProperty::GridTemplateColumns,
        CssKnownProperty::GridAutoColumns,
        CssKnownProperty::GridRowStart,
        CssKnownProperty::VoiceRate,
        CssKnownProperty::VoicePitch,
        CssKnownProperty::VoiceRange,
    ] {
        let name = format!("rejected-{property:?}");
        let outside = growth(&format!("{name}-outside"), |payload| {
            rejected_sample(payload, property, false)
        });
        let inside = growth(&format!("{name}-inside"), |payload| {
            rejected_sample(payload, property, true)
        });
        // The public error retains just a root or incompatible token origin.
        // Moving the same trivia inside rejected math cannot require a graph copy.
        budget(
            &mut failures,
            &name,
            inside.saturating_sub(outside),
            METADATA_ALLOWANCE,
        );
    }
    for property in [CssKnownProperty::VoicePitch, CssKnownProperty::VoiceRange] {
        let name = format!("conflicting-{property:?}");
        let outside = growth(&format!("{name}-outside"), |payload| {
            conflicting_speech_sample(payload, property, false)
        });
        let inside = growth(&format!("{name}-inside"), |payload| {
            conflicting_speech_sample(payload, property, true)
        });
        budget(
            &mut failures,
            &name,
            inside.saturating_sub(outside),
            METADATA_ALLOWANCE,
        );
    }
    let rotate = growth("retained-rotate-angle", |payload| {
        svg_sample(payload, false)
    });
    let svg = growth("retained-svg-angle", |payload| svg_sample(payload, true));
    // Both independently retain their full declaration components and typed
    // angle calculation, including the comment's decoded content and spelling.
    // The angle-only SVG grammar needs no extra graph compared with rotate.
    budget(
        &mut failures,
        "svg-retained-output",
        svg.saturating_sub(rotate),
        METADATA_ALLOWANCE,
    );
    assert!(
        failures.is_empty(),
        "discarded checked admission copies:\n{}",
        failures.join("\n")
    );
}
