#![forbid(unsafe_code)]
//! Strict checked property/grammar admission must reject original recovered
//! delimiters, including trivia, whole-value globals and pending substitution.
//! This uses the existing timing_component_closure_contract.rs provenance oracle:
//! repaired serialization can expose an implicit origin, but cannot close input.
//! No new borrowed variants or family expansion capability are required.

use surgeist_css::*;

fn rejection_failures(property: CssKnownProperty, source: &str) -> Vec<String> {
    let mut failures = Vec::new();
    let components = parse_component_values(source).unwrap();
    let before = components.clone();
    let serialized = components.serialize().unwrap();
    // Scan actual emitted origin mappings; do not infer an EOF position from
    // the repaired CSS length or assume the implicit ending is a suffix.
    let origin = (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("fixture retains an original implicit closure");
    let CssValueOrigin::ImplicitClosure { at, .. } = &origin else {
        unreachable!()
    };
    assert_eq!(at.source().as_str(), source);
    assert_eq!(at.span().start().byte_offset().value(), source.len());
    assert_eq!(at.span().start(), at.span().end());
    for (front, result) in [
        (
            "property",
            parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                CssImportance::Important,
            ),
        ),
        (
            "grammar",
            parse_property_value_for_grammar(
                property.grammar(),
                components.clone(),
                CssImportance::Important,
            ),
        ),
    ] {
        match result {
            Err(error)
                if matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
                ) && error.origin() == &CssSerializedOrigin::End(Some(origin.clone())) => {}
            Err(error) => failures.push(format!(
                "{front}/{source}: expected original implicit EOF; actual {error:?}"
            )),
            Ok(_) => failures.push(format!("{front}/{source}: accepted repaired components")),
        }
        assert_eq!(components, before);
    }
    failures
}

fn assert_recovered_inputs_rejected(property: CssKnownProperty, sources: &[&str]) {
    // Collect every fixture and both fronts before failing so one repaired
    // admission cannot hide the remaining property's provenance failures.
    let failures: Vec<_> = sources
        .iter()
        .flat_map(|source| rejection_failures(property, source))
        .collect();
    assert!(
        failures.is_empty(),
        "{}:\n{}",
        property.canonical_name(),
        failures.join("\n")
    );
}

fn assert_complete(property: CssKnownProperty, sources: &[&str]) {
    for &source in sources {
        let components = parse_component_values(source).unwrap();
        let before = components.clone();
        for declaration in [
            parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                CssImportance::Important,
            )
            .unwrap(),
            parse_property_value_for_grammar(
                property.grammar(),
                components.clone(),
                CssImportance::Important,
            )
            .unwrap(),
        ] {
            assert_eq!(declaration.value_components(), &components);
            assert_eq!(declaration.importance(), CssImportance::Important);
            assert_eq!(declaration.known().unwrap().grammar(), property.grammar());
            assert!(declaration.position().is_none());
        }
        assert_eq!(components, before);
    }
}

#[test]
fn recovered_animation_names_comments_and_strings_are_rejected_at_original_eof() {
    assert_recovered_inputs_rejected(
        CssKnownProperty::AnimationName,
        &[
            "fade/*",
            "none/*unfinished",
            "initial/*",
            "var(--names)/*",
            "\"fade",
        ],
    );
}

#[test]
fn recovered_animation_count_comments_are_rejected_at_original_eof() {
    assert_recovered_inputs_rejected(
        CssKnownProperty::AnimationIterationCount,
        &[
            "1/*",
            "2.5/*unfinished",
            "infinite/*",
            "initial/*",
            "var(--count)/*",
        ],
    );
}

#[test]
fn recovered_animation_direction_comments_are_rejected_at_original_eof() {
    assert_recovered_inputs_rejected(
        CssKnownProperty::AnimationDirection,
        &[
            "normal/*",
            "alternate-reverse/*unfinished",
            "initial/*",
            "var(--direction)/*",
        ],
    );
}

#[test]
fn recovered_animation_fill_comments_are_rejected_at_original_eof() {
    assert_recovered_inputs_rejected(
        CssKnownProperty::AnimationFillMode,
        &["both/*", "none/*unfinished", "initial/*", "var(--fill)/*"],
    );
}

#[test]
fn recovered_animation_play_state_comments_are_rejected_at_original_eof() {
    assert_recovered_inputs_rejected(
        CssKnownProperty::AnimationPlayState,
        &[
            "paused/*",
            "running/*unfinished",
            "initial/*",
            "var(--state)/*",
        ],
    );
}

#[test]
fn complete_animation_names_comments_strings_and_symbolic_values_remain_admitted() {
    assert_complete(
        CssKnownProperty::AnimationName,
        &[
            "fade/**/",
            "none/*unfinished*/",
            "initial/**/",
            "var(--names)/**/",
            "\"fade\"",
        ],
    );
}

#[test]
fn complete_animation_count_comments_and_symbolic_values_remain_admitted() {
    assert_complete(
        CssKnownProperty::AnimationIterationCount,
        &[
            "1/**/",
            "2.5/*unfinished*/",
            "infinite/**/",
            "initial/**/",
            "var(--count)/**/",
        ],
    );
}

#[test]
fn complete_animation_direction_comments_and_symbolic_values_remain_admitted() {
    assert_complete(
        CssKnownProperty::AnimationDirection,
        &[
            "normal/**/",
            "alternate-reverse/*unfinished*/",
            "initial/**/",
            "var(--direction)/**/",
        ],
    );
}

#[test]
fn complete_animation_fill_comments_and_symbolic_values_remain_admitted() {
    assert_complete(
        CssKnownProperty::AnimationFillMode,
        &[
            "both/**/",
            "none/*unfinished*/",
            "initial/**/",
            "var(--fill)/**/",
        ],
    );
}

#[test]
fn complete_animation_play_state_comments_and_symbolic_values_remain_admitted() {
    assert_complete(
        CssKnownProperty::AnimationPlayState,
        &[
            "paused/**/",
            "running/*unfinished*/",
            "initial/**/",
            "var(--state)/**/",
        ],
    );
}
