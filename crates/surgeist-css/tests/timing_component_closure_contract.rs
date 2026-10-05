#![forbid(unsafe_code)]
//! Checked construction retains browser-recovered components for inspection but
//! must reject their original implicit closure instead of reparsing repaired CSS.
//! Both checked front doors promise original-component error provenance.

use surgeist_css::*;

fn rejection_failures(property: CssKnownProperty, source: &str) -> Vec<String> {
    let mut failures = Vec::new();
    let components = parse_component_values(source).unwrap();
    let before = components.clone();
    let serialized = components.serialize().unwrap();
    // The component serializer exposes the original implicit delimiter origin;
    // it may emit a closing delimiter, but that does not make the input complete.
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
        }
    }
}

#[test]
fn recovered_transition_property_comments_are_rejected_at_original_eof() {
    assert_recovered_inputs_rejected(
        CssKnownProperty::TransitionProperty,
        &[
            "opacity/*",
            "none/*unfinished",
            "initial/*",
            "var(--names)/*",
        ],
    );
}

#[test]
fn recovered_transition_easing_delimiters_are_rejected_at_original_eof() {
    assert_recovered_inputs_rejected(
        CssKnownProperty::TransitionTimingFunction,
        &["steps(2,end", "cubic-bezier(0,0,1,1", "ease/*", "initial/*"],
    );
}

#[test]
fn recovered_animation_easing_delimiters_are_rejected_at_original_eof() {
    assert_recovered_inputs_rejected(
        CssKnownProperty::AnimationTimingFunction,
        &["steps(2,end", "cubic-bezier(0,0,1,1", "ease/*", "initial/*"],
    );
}

#[test]
fn complete_transition_names_comments_and_symbolic_values_remain_admitted() {
    assert_complete(
        CssKnownProperty::TransitionProperty,
        &["opacity/**/", "none", "initial", "var(--names)"],
    );
}

#[test]
fn complete_transition_easing_lists_and_symbolic_values_remain_admitted() {
    assert_complete(
        CssKnownProperty::TransitionTimingFunction,
        &[
            "steps(2,end), cubic-bezier(0,0,1,1)",
            "ease/**/",
            "initial",
            "var(--easing)",
        ],
    );
}

#[test]
fn complete_animation_easing_lists_and_symbolic_values_remain_admitted() {
    assert_complete(
        CssKnownProperty::AnimationTimingFunction,
        &[
            "steps(2,end), cubic-bezier(0,0,1,1)",
            "ease/**/",
            "initial",
            "var(--easing)",
        ],
    );
}
