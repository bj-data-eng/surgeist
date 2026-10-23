//! A rejected `||`/`&&` group or `#` list names the responsible authored
//! component: `CssPropertyValueError::encountered()` is the first component
//! the grammar cannot admit while one is present, and the separator itself
//! when a list ends after it (the last authored token, reference "Source
//! coordinates and diagnostics").

use crate::css::support::property_admission::GrammarFacts;
use surgeist_css::*;

fn encountered(error: &dyn GrammarFacts) -> Option<String> {
    assert_eq!(
        error.grammar_code(),
        Some(CssErrorCode::InvalidPropertyValue),
        "property-mapped grammar diagnostic, got {error:?}"
    );
    error
        .responsible_token()
        .map(|token| token.authored().to_owned())
}

#[track_caller]
fn assert_encountered(name: &str, value: &str, expected: Option<&str>) {
    let property = CssKnownProperty::from_name(name).unwrap();
    let expected = expected.map(str::to_owned);
    let report = parse_style_attribute(&format!("{name}: {value}"));
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "{name}: {value}: one diagnostic, got {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(
        encountered(diagnostic.error()),
        expected,
        "{name}: {value} (style)"
    );
    let sheet = parse_sheet(&format!(".a{{{name}: {value}}}"));
    let [diagnostic] = sheet.diagnostics() else {
        panic!("{name}: {value}: one sheet diagnostic")
    };
    assert_eq!(
        encountered(diagnostic.error()),
        expected,
        "{name}: {value} (sheet)"
    );
    let text = parse_property_value_text(
        value,
        CssPropertyNameRef::Known(property),
        CssImportance::Normal,
    );
    let [diagnostic] = text.diagnostics() else {
        panic!("{name}: {value}: one text diagnostic")
    };
    assert_eq!(
        encountered(diagnostic.error()),
        expected,
        "{name}: {value} (text)"
    );
    for grammar in [false, true] {
        let components = parse_component_values(value).unwrap();
        let error = if grammar {
            parse_property_value_for_grammar(property.grammar(), components, CssImportance::Normal)
        } else {
            parse_property_value(
                CssPropertyNameRef::Known(property),
                components,
                CssImportance::Normal,
            )
        }
        .unwrap_err();
        assert!(
            error.code().is_some(),
            "{name}: {value}: grammar error, got {error:?}"
        );
        assert_eq!(
            encountered(&error),
            expected,
            "{name}: {value} (checked, grammar {grammar})"
        );
    }
}

#[test]
fn rejected_groups_name_the_unadmitted_or_repeated_component() {
    for (name, value, token) in [
        ("font-synthesis", "weight foo", "foo"),
        ("font-synthesis", "weight weight", "weight"),
        ("text-transform", "uppercase foo", "foo"),
        ("word-space-transform", "space foo", "foo"),
        ("flex-flow", "row 5px", "5px"),
        ("contain", "size 5px", "5px"),
        ("grid-row-start", "2 \"x\"", "\"x\""),
        ("grid-row-start", "2 span foo", "foo"),
        ("overflow-clip-margin", "content-box \"x\"", "\"x\""),
        ("voice-volume", "loud soft", "soft"),
        ("voice-rate", "fast slow", "slow"),
        ("text-emphasis-skip", "spaces spaces", "spaces"),
        ("hanging-punctuation", "first 5px", "5px"),
        ("text-wrap", "wrap nowrap", "nowrap"),
        ("white-space", "collapse 5px", "5px"),
        ("text-decoration", "underline red overline", "overline"),
        (
            "font-variant",
            "common-ligatures small-caps contextual",
            "contextual",
        ),
        ("border", "solid solid", "solid"),
        ("outline", "auto auto auto", "auto"),
        ("transition", "opacity 1s 2s 3s", "3s"),
        ("background", "red, url(a)", "red"),
    ] {
        assert_encountered(name, value, Some(token));
    }
}

#[test]
fn a_list_ending_after_its_comma_names_that_comma() {
    for (name, value) in [("transition-duration", "1s,"), ("font-family", "serif,")] {
        assert_encountered(name, value, Some(","));
    }
}
