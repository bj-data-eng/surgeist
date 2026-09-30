#![forbid(unsafe_code)]

//! Functional specified-value expectations from CSS Lists 3 (2020-11-17) §3.7.
//! The keywords are symbolic; element or parent directionality is selected later.

use surgeist_css::*;

fn parsed(value: &str) -> CssDeclaration {
    let source = format!("marker-side:{value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one marker-side declaration")
    };
    declaration.clone()
}

fn current(declaration: &CssDeclaration) -> CssMarkerSide {
    let CssKnownPropertyValueRef::MarkerSide(wrapper) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("marker-side current wrapper")
    };
    *wrapper.value()
}

#[test]
fn both_checked_values_and_parsed_wrappers_have_distinct_symbolic_meanings() {
    for (value, expected, canonical) in [
        ("match-self", CssMarkerSide::MatchSelf, "match-self"),
        ("match-parent", CssMarkerSide::MatchParent, "match-parent"),
        ("MATCH-PARENT", CssMarkerSide::MatchParent, "match-parent"),
        (r"m\61 tch-self", CssMarkerSide::MatchSelf, "match-self"),
    ] {
        let declaration = parsed(value);
        assert_eq!(current(&declaration), expected);
        assert_eq!(expected.serialize_specified().unwrap(), canonical);
        assert_eq!(
            declaration.value_components().serialize().unwrap().as_css(),
            value
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&declaration).unwrap()
        else {
            panic!("one ordinary marker-side contribution")
        };
        let [item] = values.items() else {
            panic!("one terminal")
        };
        let CssLonghandValueRef::MarkerSide(terminal) = item.ordinary_value().unwrap().view()
        else {
            panic!("typed marker-side contribution")
        };
        assert_eq!(*terminal, expected);
    }
    assert_ne!(CssMarkerSide::MatchSelf, CssMarkerSide::MatchParent);
}

#[test]
fn programmatic_declaration_retains_origin_and_the_constructed_keyword() {
    let components =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("match-parent").unwrap()])
            .unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::MarkerSide),
        components,
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(current(&declaration), CssMarkerSide::MatchParent);
    assert_eq!(
        CssMarkerSide::MatchParent.serialize_specified().unwrap(),
        "match-parent"
    );
    assert!(matches!(
        declaration.value_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&declaration).unwrap()
    else {
        panic!("programmatic declaration expands")
    };
    let [item] = values.items() else {
        panic!("one terminal")
    };
    assert!(item.source().same_occurrence(&declaration));
    assert_eq!(item.source().importance(), CssImportance::Important);
}

#[test]
fn keyword_serialization_has_exact_independent_node_and_byte_limits() {
    for (value, canonical) in [
        (CssMarkerSide::MatchSelf, "match-self"),
        (CssMarkerSide::MatchParent, "match-parent"),
    ] {
        let exact = CssSpecifiedValueSerializationLimits::new(1, 1, canonical.len());
        assert_eq!(
            value.serialize_specified_with_limits(exact).unwrap(),
            canonical
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, canonical.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, canonical.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, canonical.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(value.serialize_specified().unwrap(), canonical);
        }
    }
}
