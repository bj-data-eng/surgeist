#![forbid(unsafe_code)]

//! Standalone public admission allocation contract, without test-harness threads.
//! Supplied comments are already owned input, and neither a CSS-wide declaration
//! nor a root-delimiter error needs an authored value string in its result.
//! Keeping component count fixed while enlarging only that comment must therefore
//! not require a payload-sized grammar transport or diagnostic mapping copy.
//! The allocator's safe instrumentation follows animation's allocation_check.

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use std::alloc::System;
use std::hint::black_box;
use surgeist_css::{
    CssComponentValues, CssDeclaration, CssErrorCode, CssGlobalKeyword, CssImportance,
    CssKnownProperty, CssParser, CssPropertyNameRef, CssPropertyTarget, CssSerializedOrigin,
    CssTokenKind, CssValueOrigin,
};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const SMALL_COMMENT_BYTES: usize = 128;
const LARGE_COMMENT_BYTES: usize = 65_536;
// Fixed-size cursor, declaration and error metadata may allocate. This allowance
// concerns growth with comment bytes; it is not a zero-allocation requirement.
const MAX_ADMISSION_BYTE_GROWTH: usize = 1_024;

#[derive(Clone, Copy)]
enum Outcome {
    GlobalInitial,
    RootSemicolon,
}

impl Outcome {
    fn tail(self) -> &'static str {
        match self {
            Self::GlobalInitial => "initial",
            Self::RootSemicolon => ";",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::GlobalInitial => "global-initial",
            Self::RootSemicolon => "root-semicolon",
        }
    }
}

fn fixture(comment_bytes: usize, outcome: Outcome) -> CssComponentValues {
    let source = format!("/*{}*/{}", "x".repeat(comment_bytes), outcome.tail());
    let values = CssComponentValues::parse(&CssParser::default(), &source)
        .expect("complete comment and root token");
    assert_eq!(values.items().len(), 2);
    values
}

fn measure(values: &CssComponentValues, outcome: Outcome) -> Stats {
    // Input cloning and expected-origin capture deliberately precede the region.
    let input = values.clone();
    let expected_token_origin = input.items()[1].origin().clone();
    let region = Region::new(GLOBAL);
    let result = black_box(CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Width)),
        black_box(input),
        CssImportance::Normal,
    ));
    let stats = region.change();

    // Public semantic expectations, diagnostics and destruction are unmeasured.
    match outcome {
        Outcome::GlobalInitial => {
            let declaration = result.expect("the whole CSS-wide initial value is valid");
            let known = declaration.known().expect("known width declaration");
            assert_eq!(known.property(), CssKnownProperty::Width);
            assert_eq!(known.global(), Some(CssGlobalKeyword::Initial));
            assert!(known.property_value().is_none());
            assert_eq!(declaration.importance(), CssImportance::Normal);
            assert_eq!(declaration.value_components(), values);
            assert!(declaration.position().is_none());
            assert!(declaration.parsed_name().is_none());
            assert!(declaration.parsed_value().is_none());
            assert_same_snapshot(
                declaration.value_components().items()[1].origin(),
                &expected_token_origin,
            );
            drop(declaration);
        }
        Outcome::RootSemicolon => {
            let error = result.expect_err("a root semicolon is not declaration-value data");
            assert_eq!(
                error.code(),
                Some(CssErrorCode::UnexpectedToken),
                "expected a root-token grammar rejection: {error:?}"
            );
            assert_eq!(error.encountered().unwrap().kind(), CssTokenKind::Semicolon);
            assert_eq!(error.encountered().unwrap().authored(), ";");
            let origin = error.origin();
            let CssSerializedOrigin::Token(actual_origin) = &origin else {
                panic!("expected the original semicolon token: {error:?}");
            };
            assert_eq!(actual_origin, &expected_token_origin);
            assert_same_snapshot(actual_origin, &expected_token_origin);
            let CssValueOrigin::Parsed(original) = actual_origin else {
                panic!("fixture token has its actual parsed source");
            };
            let start = original.span().start().byte_offset().value();
            let end = original.span().end().byte_offset().value();
            assert_eq!(&original.source().as_str()[start..end], ";");
            assert_eq!(end, original.source().as_str().len());
            drop(error);
        }
    }
    stats
}

fn assert_same_snapshot(actual: &CssValueOrigin, expected: &CssValueOrigin) {
    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) = (actual, expected)
    else {
        panic!("fixture token origins retain parsed snapshots");
    };
    assert!(actual.source().same_snapshot(expected.source()));
}

fn requested_bytes(stats: Stats) -> usize {
    // stats_alloc records net reallocation growth separately from new allocations.
    // Retain those raw fields in the report; this sum is the budgeted metric.
    stats
        .bytes_allocated
        .checked_add(usize::try_from(stats.bytes_reallocated).unwrap_or(0))
        .expect("bounded fixture allocation arithmetic")
}

// A comment-only var() has no custom-property name. Its missing-name rejection
// retains a small closing-boundary error, never an authored output string.
fn missing_var_fixture(comment_bytes: usize) -> CssComponentValues {
    let source = format!("var(/*{}*/)", "x".repeat(comment_bytes));
    let values = CssComponentValues::parse(&CssParser::default(), &source)
        .expect("complete var function and comment");
    assert_eq!(values.items().len(), 1);
    let surgeist_css::CssComponentValueRef::Function(function) = values.items()[0].view() else {
        panic!("original var function");
    };
    assert_eq!(function.values().items().len(), 1);
    assert!(matches!(
        function.values().items()[0].view(),
        surgeist_css::CssComponentValueRef::Comment(_)
    ));
    values
}

fn measure_missing_var(
    values: &CssComponentValues,
    name: &surgeist_css::CssCustomPropertyName,
) -> Stats {
    let input = values.clone();
    let surgeist_css::CssComponentValueRef::Function(function) = input.items()[0].view() else {
        panic!("original var function");
    };
    let expected_closing = function.closing_origin().clone();
    let region = Region::new(GLOBAL);
    let result = black_box(CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Custom(name)),
        black_box(input),
        CssImportance::Normal,
    ));
    let stats = region.change();

    let error = result.expect_err("a comment cannot supply var()'s required name");
    assert_eq!(error.code(), Some(CssErrorCode::UnexpectedEnd), "{error:?}");
    let origin = error.origin();
    let CssSerializedOrigin::Token(actual_closing) = &origin else {
        panic!("explicit closing delimiter retains its original token origin: {error:?}");
    };
    assert_eq!(actual_closing, &expected_closing);
    assert_same_snapshot(actual_closing, &expected_closing);
    let CssValueOrigin::Parsed(original) = actual_closing else {
        panic!("explicit closing delimiter is parsed from the actual fixture");
    };
    let start = original.span().start().byte_offset().value();
    let end = original.span().end().byte_offset().value();
    assert_eq!(&original.source().as_str()[start..end], ")");
    assert_eq!(end, original.source().as_str().len());
    drop(error);
    stats
}

fn check_missing_var(failures: &mut Vec<String>) {
    let name = surgeist_css::CssCustomPropertyName::try_new("--x").unwrap();
    let small = missing_var_fixture(SMALL_COMMENT_BYTES);
    let large = missing_var_fixture(LARGE_COMMENT_BYTES);
    let _ = measure_missing_var(&small, &name);
    let _ = measure_missing_var(&large, &name);
    let small_stats = measure_missing_var(&small, &name);
    let large_stats = measure_missing_var(&large, &name);
    let growth = requested_bytes(large_stats).saturating_sub(requested_bytes(small_stats));
    println!(
        "missing-var-name: comment bytes {} -> {}; small={small_stats:?}; large={large_stats:?}; admission byte growth={growth}; allowance={MAX_ADMISSION_BYTE_GROWTH}; UnexpectedEnd and original explicit closing token passed",
        SMALL_COMMENT_BYTES, LARGE_COMMENT_BYTES,
    );
    if growth > MAX_ADMISSION_BYTE_GROWTH {
        failures.push(format!(
            "missing-var-name admission copied payload-sized input: byte growth {growth} exceeds {MAX_ADMISSION_BYTE_GROWTH}"
        ));
    }
}

fn measure_source_stripe(source: &str, flex: bool) -> Stats {
    let region = Region::new(GLOBAL);
    let report =
        black_box(surgeist_css::CssParser::default().parse_block_contents(black_box(source)));
    let stats = region.change();

    assert!(
        report.is_clean(),
        "a complete stripe calculation is admitted"
    );
    assert_eq!(report.syntax().len(), 1);
    let known = report.syntax()[0].known().unwrap();
    assert_eq!(known.property(), CssKnownProperty::OutlineColor);
    let surgeist_css::CssKnownPropertyValueRef::OutlineColor(value) =
        known.property_value().unwrap()
    else {
        panic!("outline-color retains its typed image");
    };
    let image = value.value().image_1d().unwrap();
    assert_eq!(image.stripes().stripes().len(), 1);
    let thickness = image.stripes().stripes()[0].thickness().unwrap();
    let components = match thickness {
        surgeist_css::CssStripeThickness::Flex(value) if flex => {
            value.calculation().unwrap().components()
        }
        surgeist_css::CssStripeThickness::LengthPercentage(value) if !flex => {
            value.value().calculation().unwrap().components()
        }
        _ => panic!("the literal fr/px unit selects its independent typed domain"),
    };
    let expected = if flex {
        "stripes(red calc(1fr))"
    } else {
        "stripes(red calc(1px))"
    };
    assert_eq!(
        image.serialize_specified(Default::default()).unwrap(),
        expected
    );
    let start = source.find("calc(").unwrap();
    assert_eq!(
        components
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        &source[start..source.len() - 1]
    );
    let CssValueOrigin::Parsed(origin) = components.items()[0].origin() else {
        panic!("calculation retains its actual source occurrence");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    let surgeist_css::CssComponentValueRef::Function(function) = components.items()[0].view()
    else {
        panic!("retained calculation root");
    };
    let original = image.stripes().original_component().unwrap();
    assert_same_snapshot(original.origin(), components.items()[0].origin());
    assert_same_snapshot(function.closing_origin(), components.items()[0].origin());
    drop(report);
    stats
}

fn check_source_stripe_selection(failures: &mut Vec<String>) {
    let growth = [true, false].map(|flex| {
        let unit = if flex { "fr" } else { "px" };
        let sources = [SMALL_COMMENT_BYTES, LARGE_COMMENT_BYTES].map(|length| {
            format!(
                "outline-color:stripes(red calc(1{unit}/*{}*/))",
                "x".repeat(length)
            )
        });
        for source in &sources {
            let _ = measure_source_stripe(source, flex);
        }
        let stats = sources
            .each_ref()
            .map(|source| measure_source_stripe(source, flex));
        let growth = requested_bytes(stats[1]).saturating_sub(requested_bytes(stats[0]));
        println!(
            "source-stripe-{unit}: small={:?}; large={:?}; retained byte growth={growth}",
            stats[0], stats[1]
        );
        growth
    });
    // Equal-length fr/px source values retain the same authored payload and one
    // typed calculation graph each. Choosing the first root cannot justify an
    // additional payload-sized input copy compared with the fallback root.
    let difference = growth[0].abs_diff(growth[1]);
    println!(
        "source-stripe-selection: growth difference={difference}; allowance={MAX_ADMISSION_BYTE_GROWTH}; typed domains, exact retained spelling and original snapshots passed"
    );
    if difference > MAX_ADMISSION_BYTE_GROWTH {
        failures.push(format!("source stripe selection copied its owned graph: growth difference {difference} exceeds {MAX_ADMISSION_BYTE_GROWTH}"));
    }
}

fn assert_retained_url_modifier(
    declaration: &surgeist_css::CssDeclaration,
    source: &str,
    inside: bool,
    comment_bytes: usize,
) {
    let known = declaration.known().unwrap();
    assert_eq!(known.property(), CssKnownProperty::BackgroundImage);
    let surgeist_css::CssKnownPropertyValueRef::BackgroundImage(value) =
        known.property_value().unwrap()
    else {
        panic!("typed background image");
    };
    let [surgeist_css::CssImageValue::Url(url)] = value.value().images() else {
        panic!("one retained URL image");
    };
    assert_eq!(url.as_str(), "asset");
    let [surgeist_css::CssUrlModifier::Function(modifier)] = url.modifiers() else {
        panic!("one functional extension modifier");
    };
    assert_eq!(modifier.name(), "future");
    assert!(modifier.request_modifier().is_none());
    let original = declaration.value_components().items()[0].origin();
    let CssValueOrigin::Parsed(root) = original else {
        panic!("actual source occurrence");
    };
    assert_eq!(root.source().as_str(), source);
    let [token] = modifier.argument_components().items() else {
        panic!("one quoted argument");
    };
    if inside {
        let decoded = format!("/*{}*/", "x".repeat(comment_bytes));
        let expected = format!("\"{decoded}\"");
        assert_eq!(modifier.arguments().as_css(), expected);
        assert_eq!(
            modifier
                .argument_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            expected
        );
        assert!(
            matches!(token.view(), surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::String(content)) if content == decoded)
        );
        assert_same_snapshot(token.origin(), original);
        let CssValueOrigin::Parsed(child) = token.origin() else {
            panic!("original string origin");
        };
        let start = source.find("\"/*").unwrap();
        assert_eq!(child.span().start().byte_offset().value(), start);
        assert_eq!(
            child.span().end().byte_offset().value(),
            start + comment_bytes + 6
        );
    } else {
        assert_eq!(modifier.arguments().as_css(), "\"\"");
        assert!(matches!(
            token.view(),
            surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::String(""))
        ));
    }
    let value_source = source.strip_prefix("background-image:").unwrap_or(source);
    assert_eq!(
        declaration
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        value_source
    );
}

fn measure_checked_url_modifier(
    values: &CssComponentValues,
    source: &str,
    inside: bool,
    comment_bytes: usize,
) -> Stats {
    let input = values.clone();
    let region = Region::new(GLOBAL);
    let result = black_box(CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage)),
        black_box(input),
        CssImportance::Normal,
    ));
    let stats = region.change();
    let declaration = result.expect("an unknown modifier keeps generic component syntax");
    assert_retained_url_modifier(&declaration, source, inside, comment_bytes);
    assert_eq!(declaration.value_components(), values);
    drop(declaration);
    stats
}

fn measure_source_url_modifier(source: &str, inside: bool, comment_bytes: usize) -> Stats {
    let region = Region::new(GLOBAL);
    let report =
        black_box(surgeist_css::CssParser::default().parse_block_contents(black_box(source)));
    let stats = region.change();
    assert!(report.is_clean(), "complete functional URL modifier source");
    assert_eq!(report.syntax().len(), 1);
    // The source cursor and promoted temporary component have already dropped.
    assert_retained_url_modifier(&report.syntax()[0], source, inside, comment_bytes);
    drop(report);
    stats
}

fn check_url_modifier_ownership(failures: &mut Vec<String>) {
    for checked in [true, false] {
        let growth = [true, false].map(|inside| {
            let samples = [SMALL_COMMENT_BYTES, LARGE_COMMENT_BYTES].map(|length| {
                let comment = format!("/*{}*/", "x".repeat(length));
                let value = if inside {
                    format!("url(\"asset\" future(\"{comment}\"))")
                } else {
                    format!("url(\"asset\" {comment}future(\"\"))")
                };
                let source = if checked { value } else { format!("background-image:{value}") };
                let values = if checked { Some(CssComponentValues::parse(&CssParser::default(), &source).unwrap()) } else { None };
                (source, values, length)
            });
            let measure = |(source, values, length): &(String, Option<CssComponentValues>, usize)| {
                if let Some(values) = values {
                    measure_checked_url_modifier(values, source, inside, *length)
                } else {
                    measure_source_url_modifier(source, inside, *length)
                }
            };
            for sample in &samples { let _ = measure(sample); }
            let stats = samples.each_ref().map(measure);
            let growth = requested_bytes(stats[1]).saturating_sub(requested_bytes(stats[0]));
            println!("url-modifier: checked={checked}, inside={inside}; small={:?}; large={:?}; growth={growth}", stats[0], stats[1]);
            growth
        });
        // The equal-size inputs retain the same complete declaration spelling.
        // Only the inside case additionally retains its argument String and the
        // string token's decoded text plus exact spelling in one owned argument graph.
        // Permit those three required payload spellings and fixed metadata.
        let allowance = 3 * (LARGE_COMMENT_BYTES - SMALL_COMMENT_BYTES) + MAX_ADMISSION_BYTE_GROWTH;
        let difference = growth[0].saturating_sub(growth[1]);
        println!(
            "url-modifier-ownership: checked={checked}; growth difference={difference}; allowance={allowance}; typed arguments, exact spelling and original snapshots passed"
        );
        if difference > allowance {
            failures.push(format!("URL modifier copied a temporary function: checked={checked}, growth difference {difference} exceeds {allowance}"));
        }
    }
}

fn main() {
    let mut failures = Vec::new();
    for outcome in [Outcome::GlobalInitial, Outcome::RootSemicolon] {
        let small = fixture(SMALL_COMMENT_BYTES, outcome);
        let large = fixture(LARGE_COMMENT_BYTES, outcome);
        // Warm both public paths before comparing equally shaped retained inputs.
        let _ = measure(&small, outcome);
        let _ = measure(&large, outcome);
        let small_stats = measure(&small, outcome);
        let large_stats = measure(&large, outcome);
        let growth = requested_bytes(large_stats).saturating_sub(requested_bytes(small_stats));
        println!(
            "{}: comment bytes {} -> {}; small={small_stats:?}; large={large_stats:?}; admission byte growth={growth}; allowance={MAX_ADMISSION_BYTE_GROWTH}",
            outcome.name(),
            SMALL_COMMENT_BYTES,
            LARGE_COMMENT_BYTES,
        );
        if growth > MAX_ADMISSION_BYTE_GROWTH {
            failures.push(format!(
                "{} admission copied payload-sized input: byte growth {growth} exceeds {MAX_ADMISSION_BYTE_GROWTH}",
                outcome.name(),
            ));
        }
        drop(small);
        drop(large);
    }
    check_missing_var(&mut failures);
    check_source_stripe_selection(&mut failures);
    check_url_modifier_ownership(&mut failures);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
