//! Specified serialization leaves out the component that the grammar supplies
//! when it is omitted: CSSOM 1 (WD 2021-08-26) §6.7.2 omits or replaces
//! component values with a shorter representation when the meaning of the value
//! does not change. The omitted value of each case comes from its property's
//! grammar, never from the serializer:
//!
//! - `place-items`, `place-self`: Box Alignment 3 (WD 2026-01-30) §7.3 and §6.3
//!   copy an omitted second value from the first.
//! - `place-content`: §5.2 copies it from the first, except that a baseline
//!   first value gives `start`.
//! - `counter-increment`, `counter-reset`, `counter-set`: Lists 3 (WD
//!   2020-11-17) §4.2 and §4.1 give an omitted `<integer>` the value `1` for
//!   `counter-increment` and `0` for the other two.
//! - `font-feature-settings`: Fonts 4 (WD 2026-09-07) §6.12 gives an omitted
//!   value, `on` and `1` the value `1`, and `off` the value `0`.

use crate::css::support::sole_declaration as declaration;
use surgeist_css::declaration::CssDeclaration;
use surgeist_css::property::CssKnownPropertyValueRef;
use surgeist_css::values::content::CssCounterChangesValue;
use surgeist_css::values::font::{CssAuthoredFontFeatureSettings, CssAuthoredFontFeatureValue};

fn specified(property: &str, value: &str) -> String {
    declaration(&format!("{property}: {value}"))
        .serialize_specified(Default::default())
        .unwrap()
}

/// The two serialized axis components of one place shorthand. Each component
/// serializes `first baseline` as `baseline` already, so equal text is equal
/// meaning.
fn axes(declaration: &CssDeclaration) -> (String, String) {
    let text = |result: Result<String, _>| result.unwrap();
    let limits = Default::default;
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::PlaceContent(value) => (
            text(value.value().align().serialize_specified(limits())),
            text(value.value().justify().serialize_specified(limits())),
        ),
        CssKnownPropertyValueRef::PlaceItems(value) => (
            text(value.value().align().serialize_specified(limits())),
            text(value.value().justify().serialize_specified(limits())),
        ),
        CssKnownPropertyValueRef::PlaceSelf(value) => (
            text(value.value().align().serialize_specified(limits())),
            text(value.value().justify().serialize_specified(limits())),
        ),
        other => panic!("not a place shorthand: {other:?}"),
    }
}

/// Serializes each authored value, checks the shortest form, and checks that
/// the serialized form reparses to the same two axis components.
#[track_caller]
fn assert_place_cases(property: &str, cases: &[(&str, &str)]) {
    for (authored, expected) in cases {
        let source = declaration(&format!("{property}: {authored}"));
        let serialized = source.serialize_specified(Default::default()).unwrap();
        assert_eq!(
            serialized,
            format!("{property}: {expected};"),
            "{property}: {authored}"
        );
        let reparsed = declaration(&serialized);
        assert_eq!(
            axes(&reparsed),
            axes(&source),
            "{property}: {authored} must keep its meaning as `{serialized}`"
        );
        assert_eq!(
            reparsed.serialize_specified(Default::default()).unwrap(),
            serialized,
            "{property}: {authored} must reach a fixed point"
        );
    }
}

#[test]
fn place_items_omits_a_second_value_that_the_first_would_supply() {
    // The first four rows are the browser witness of the issue.
    assert_place_cases(
        "place-items",
        &[
            ("stretch", "stretch"),
            ("stretch stretch", "stretch"),
            ("center start", "center start"),
            ("normal legacy", "normal legacy"),
            ("center center", "center"),
            ("normal normal", "normal"),
            ("self-start self-start", "self-start"),
            ("flex-end flex-end", "flex-end"),
            ("safe center safe center", "safe center"),
            ("unsafe end unsafe end", "unsafe end"),
        ],
    );
}

#[test]
fn place_items_keeps_a_second_value_that_the_first_would_not_supply() {
    assert_place_cases(
        "place-items",
        &[
            ("center left", "center left"),
            ("start right", "start right"),
            ("center legacy", "center legacy"),
            ("center legacy left", "center legacy left"),
            ("safe center center", "safe center center"),
            ("center safe center", "center safe center"),
            ("safe center unsafe center", "safe center unsafe center"),
            ("start end", "start end"),
        ],
    );
}

#[test]
fn place_items_baseline_forms_collapse_only_when_both_axes_mean_the_same() {
    assert_place_cases(
        "place-items",
        &[
            ("baseline", "baseline"),
            ("first baseline", "baseline"),
            ("baseline baseline", "baseline"),
            ("first baseline first baseline", "baseline"),
            ("first baseline baseline", "baseline"),
            ("baseline first baseline", "baseline"),
            ("last baseline", "last baseline"),
            ("last baseline last baseline", "last baseline"),
            ("baseline last baseline", "baseline last baseline"),
            ("last baseline baseline", "last baseline baseline"),
            ("last baseline first baseline", "last baseline baseline"),
            ("baseline center", "baseline center"),
            ("center baseline", "center baseline"),
        ],
    );
}

#[test]
fn place_self_omits_a_second_value_that_the_first_would_supply() {
    assert_place_cases(
        "place-self",
        &[
            ("auto", "auto"),
            ("auto auto", "auto"),
            ("auto right", "auto right"),
            ("end center", "end center"),
            ("center center", "center"),
            ("stretch stretch", "stretch"),
            ("normal normal", "normal"),
            ("self-end self-end", "self-end"),
            ("safe normal safe normal", "safe normal"),
            ("safe normal normal", "safe normal normal"),
            ("center right", "center right"),
            ("baseline baseline", "baseline"),
            ("first baseline baseline", "baseline"),
            ("last baseline last baseline", "last baseline"),
            ("baseline last baseline", "baseline last baseline"),
            ("last baseline baseline", "last baseline baseline"),
        ],
    );
}

#[test]
fn place_content_omits_a_second_value_that_the_first_would_supply() {
    assert_place_cases(
        "place-content",
        &[
            ("center", "center"),
            ("center center", "center"),
            ("center end", "center end"),
            ("space-between space-between", "space-between"),
            ("space-evenly space-evenly", "space-evenly"),
            ("stretch stretch", "stretch"),
            ("normal normal", "normal"),
            ("safe start safe start", "safe start"),
            ("safe start start", "safe start start"),
            ("start left", "start left"),
            ("start right", "start right"),
            ("space-between center", "space-between center"),
        ],
    );
}

#[test]
fn place_content_baseline_second_value_is_start_when_omitted() {
    // Box Alignment 3 §5.2: a baseline first value gives `start`, so the
    // second value is left out exactly when it is `start`.
    assert_place_cases(
        "place-content",
        &[
            ("baseline", "baseline"),
            ("baseline start", "baseline"),
            ("first baseline start", "baseline"),
            ("last baseline", "last baseline"),
            ("last baseline start", "last baseline"),
            ("baseline end", "baseline end"),
            ("baseline center", "baseline center"),
            ("baseline safe start", "baseline safe start"),
            ("baseline right", "baseline right"),
            ("last baseline end", "last baseline end"),
        ],
    );
}

#[test]
fn place_shorthand_values_serialize_the_same_through_typed_construction() {
    use surgeist_css::values::layout::{
        CssAlignItemsValue, CssAlignmentValue, CssJustifyItemsValue, CssPlaceItemsValue,
    };
    let stretch = CssAlignItemsValue::try_new(CssAlignmentValue::Stretch).unwrap();
    let explicit = CssPlaceItemsValue::new(
        stretch,
        CssJustifyItemsValue::try_new(CssAlignmentValue::Stretch).unwrap(),
    );
    assert_eq!(
        explicit.serialize_specified(Default::default()).unwrap(),
        "stretch"
    );
    assert_eq!(
        CssPlaceItemsValue::from_align(stretch)
            .serialize_specified(Default::default())
            .unwrap(),
        "stretch"
    );
}

fn counter_text(property: &str, value: &str) -> String {
    specified(property, value)
        .strip_prefix(&format!("{property}: "))
        .and_then(|text| text.strip_suffix(';'))
        .unwrap()
        .to_owned()
}

/// Each entry of a counter value as its name and its effective integer: the
/// literal's exact value, or the property's default when it is omitted.
fn effective_counters(
    declaration: &CssDeclaration,
    default: i32,
) -> Vec<(String, Result<i32, String>)> {
    let value: &CssCounterChangesValue =
        match declaration.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::CounterReset(value) => value.value(),
            CssKnownPropertyValueRef::CounterIncrement(value) => value.value(),
            CssKnownPropertyValueRef::CounterSet(value) => value.value(),
            other => panic!("not a counter property: {other:?}"),
        };
    value
        .changes()
        .unwrap_or_default()
        .iter()
        .map(|change| {
            let integer = match change.value() {
                None => Ok(default),
                Some(number) => number
                    .literal()
                    .and_then(|literal| literal.value())
                    .ok_or_else(|| number.serialize_specified(Default::default()).unwrap()),
            };
            (change.name().as_str().to_owned(), integer)
        })
        .collect()
}

#[track_caller]
fn assert_counter_cases(property: &str, default: i32, cases: &[(&str, &str)]) {
    for (authored, expected) in cases {
        assert_eq!(
            counter_text(property, authored),
            *expected,
            "{property}: {authored}"
        );
        let source = declaration(&format!("{property}: {authored}"));
        let reparsed = declaration(&format!("{property}: {expected}"));
        assert_eq!(
            effective_counters(&reparsed, default),
            effective_counters(&source, default),
            "{property}: {authored} must keep its meaning as `{expected}`"
        );
    }
}

#[test]
fn counter_increment_omits_the_integer_one() {
    assert_counter_cases(
        "counter-increment",
        1,
        &[
            // Lists 3 §4.2: an omitted integer is 1, so `section` and
            // `section 1` are one value.
            ("section", "section"),
            ("section 1", "section"),
            ("section +1", "section"),
            ("section 01", "section"),
            ("none", "none"),
            ("a b 1 c 1 d 2", "a b c d 2"),
            ("a 2 a 1 a", "a 2 a a"),
            ("Section 1 section 1", "Section section"),
        ],
    );
}

#[test]
fn counter_increment_keeps_every_other_integer_and_symbolic_math() {
    assert_counter_cases(
        "counter-increment",
        1,
        &[
            ("section 0", "section 0"),
            ("section -0", "section 0"),
            ("section 2", "section 2"),
            ("section -1", "section -1"),
            ("a 0 b 0", "a 0 b 0"),
            ("section 2147483648", "section 2147483648"),
            ("section calc(1)", "section calc(1)"),
            ("section calc(0 + 1)", "section calc(1)"),
        ],
    );
}

#[test]
fn counter_reset_and_set_omit_the_integer_zero() {
    for property in ["counter-reset", "counter-set"] {
        assert_counter_cases(
            property,
            0,
            &[
                ("section", "section"),
                ("section 0", "section"),
                ("section -0", "section"),
                ("section +000", "section"),
                ("none", "none"),
                ("a b 0 c 0 d 2", "a b c d 2"),
                ("section 1", "section 1"),
                ("section -1", "section -1"),
                ("section 2147483648", "section 2147483648"),
                ("section calc(0)", "section calc(0)"),
                ("section calc(1 - 1)", "section calc(0)"),
            ],
        );
    }
}

/// One feature as its tag and its effective value; a value beyond `i32` or a
/// symbolic one is its own text.
fn effective_features(declaration: &CssDeclaration) -> Vec<(String, Result<i32, String>)> {
    let CssKnownPropertyValueRef::FontFeatureSettings(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("font-feature-settings wrapper")
    };
    let CssAuthoredFontFeatureSettings::Features(list) = value.value() else {
        panic!("feature list")
    };
    list.features()
        .iter()
        .map(|feature| {
            let effective = match feature.value() {
                CssAuthoredFontFeatureValue::Omitted | CssAuthoredFontFeatureValue::On => Ok(1),
                CssAuthoredFontFeatureValue::Off => Ok(0),
                CssAuthoredFontFeatureValue::Index(number) => number
                    .literal()
                    .and_then(|literal| literal.value())
                    .ok_or_else(|| number.serialize_specified(Default::default()).unwrap()),
                _ => panic!("unexpected feature value"),
            };
            (feature.tag().as_str().to_owned(), effective)
        })
        .collect()
}

#[track_caller]
fn assert_feature_cases(cases: &[(&str, &str)]) {
    for (authored, expected) in cases {
        assert_eq!(
            counter_text("font-feature-settings", authored),
            *expected,
            "font-feature-settings: {authored}"
        );
        let source = declaration(&format!("font-feature-settings: {authored}"));
        let reparsed = declaration(&format!("font-feature-settings: {expected}"));
        assert_eq!(
            effective_features(&reparsed),
            effective_features(&source),
            "{authored} must keep its meaning as `{expected}`"
        );
    }
}

#[test]
fn font_feature_settings_omits_the_value_one_and_writes_off_as_zero() {
    assert_feature_cases(&[
        // The issue's spec form: `on` is 1 and an omitted value is 1.
        ("\"kern\" on, \"liga\" 0", "\"kern\", \"liga\" 0"),
        ("\"kern\"", "\"kern\""),
        ("\"kern\" on", "\"kern\""),
        ("\"kern\" ON", "\"kern\""),
        ("\"kern\" 1", "\"kern\""),
        ("\"kern\" 01", "\"kern\""),
        ("\"kern\" +1", "\"kern\""),
        // Fonts 4 §6.12: `off` is 0, and `0` is the shorter spelling.
        ("\"liga\" off", "\"liga\" 0"),
        ("\"liga\" OFF", "\"liga\" 0"),
        ("\"liga\" 0", "\"liga\" 0"),
        ("\"liga\" 00", "\"liga\" 0"),
        ("\"liga\" 2", "\"liga\" 2"),
        ("\"cv01\" 12", "\"cv01\" 12"),
        (
            "\"kern\" on, \"kern\" off, \"kern\", \"kern\" 2",
            "\"kern\", \"kern\" 0, \"kern\", \"kern\" 2",
        ),
        ("\"b\\61 rs\" on, \"zero\" off", "\"bars\", \"zero\" 0"),
    ]);
}

#[test]
fn font_feature_settings_keeps_large_integers_and_symbolic_math() {
    for (authored, expected) in [
        ("\"cv01\" 2147483648", "\"cv01\" 2147483648"),
        ("\"cv01\" calc(1)", "\"cv01\" calc(1)"),
        ("\"cv01\" calc(0)", "\"cv01\" calc(0)"),
        ("\"cv01\" calc(2 - 1)", "\"cv01\" calc(1)"),
    ] {
        assert_eq!(
            counter_text("font-feature-settings", authored),
            expected,
            "{authored}"
        );
    }
    assert_eq!(
        counter_text("font-feature-settings", "normal"),
        "normal",
        "normal is the whole value"
    );
}

#[test]
fn font_variation_settings_keeps_its_required_number() {
    // Fonts 4 §6.12: the number of `<variation-tag-value>` is required, so
    // nothing in it is a default.
    for (authored, expected) in [
        ("\"wght\" 1", "\"wght\" 1"),
        ("\"wght\" 0", "\"wght\" 0"),
        ("\"wght\" 1, \"wdth\" 0", "\"wght\" 1, \"wdth\" 0"),
    ] {
        assert_eq!(
            counter_text("font-variation-settings", authored),
            expected,
            "{authored}"
        );
    }
}
