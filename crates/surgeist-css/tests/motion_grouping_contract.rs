#![forbid(unsafe_code)]
//! Motion WD20241105 offset imports the whole offset-rotate property.
//! Values3 §2.2: reorderable combinators are not associative; a || [b || c]
//! excludes b a c. The modifier and angle cannot straddle the distance.
use surgeist_css::*;

fn property() -> CssKnownProperty {
    CssKnownProperty::from_name("offset").expect("the selected offset property")
}

fn checked(value: &str, grammar: bool) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let components = parse_component_values(value).unwrap();
    if grammar {
        parse_property_value_for_grammar(property().grammar(), components, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property()),
            components,
            CssImportance::Important,
        )
    }
}

#[test]
fn whole_rotation_and_distance_can_swap_without_splitting_rotation() {
    for modifier in ["auto", "reverse"] {
        let expected = format!("offset: none 1px {modifier} 90deg !important;");
        for value in [
            format!("none 1px {modifier} 90deg"),
            format!("none 1px 90deg {modifier}"),
            format!("none {modifier} 90deg 1px"),
            format!("none 90deg {modifier} 1px"),
        ] {
            let report = parse_style_attribute(&format!("offset:{value}!important"));
            assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
            assert_eq!(report.syntax()[0].known().unwrap().property(), property());
            assert_eq!(report.syntax()[0].to_specified_css().unwrap(), expected);
            for grammar in [false, true] {
                assert_eq!(
                    checked(&value, grammar)
                        .unwrap()
                        .to_specified_css()
                        .unwrap(),
                    expected
                );
            }
        }
    }
}

#[test]
fn distance_cannot_split_the_quoted_rotation_constituent() {
    for modifier in ["auto", "reverse"] {
        for angle in ["90deg", "calc(90deg)"] {
            for value in [
                format!("none {modifier} 1px {angle}"),
                format!("none {angle} 1px {modifier}"),
            ] {
                let css = format!("/*😀*/color:red;offset:{value}!important;color:blue");
                let report = parse_style_attribute(&css);
                assert_eq!(report.syntax().len(), 2, "interleaved {value}");
                let [diagnostic] = report.diagnostics() else {
                    panic!("one grammar diagnostic")
                };
                assert!(matches!(
                    diagnostic.error().kind(),
                    ErrorKind::InvalidPropertyValue(_)
                ));
                assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
                assert_eq!(
                    diagnostic.span().start().byte_offset().value(),
                    css.find("offset:").unwrap()
                );
                assert_eq!(
                    validate_style_attribute(&css).unwrap_err().diagnostics(),
                    report.diagnostics()
                );
                for grammar in [false, true] {
                    assert!(
                        checked(&value, grammar).is_err(),
                        "checked interleaved {value}"
                    );
                    let text = if grammar {
                        parse_property_value_text_for_grammar(
                            &value,
                            property().grammar(),
                            CssImportance::Important,
                        )
                    } else {
                        parse_property_value_text(
                            &value,
                            CssPropertyNameRef::Known(property()),
                            CssImportance::Important,
                        )
                    };
                    assert!(!text.is_clean(), "text interleaved {value}");
                }
            }
        }
    }
}

#[test]
fn pending_grouping_failure_preserves_source_and_allows_whole_rotation_retry() {
    let report = parse_style_attribute("/*😀*/offset:var(--motion)!important");
    assert!(report.is_clean());
    let source = report.syntax()[0].clone();
    assert_eq!(source.known().unwrap().property(), property());
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending")
    };
    for value in ["none reverse 1px 90deg", "none 90deg 1px reverse"] {
        let replacement = parse_component_values(value).unwrap();
        let snapshot = replacement.clone();
        let error = handle.reenter(replacement.clone()).unwrap_err();
        assert!(matches!(
            error.kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        assert_eq!(replacement, snapshot);
        assert!(handle.source().same_occurrence(&source));
        assert_eq!(handle.source(), &source);
    }
    let replacement = parse_component_values("none reverse 90deg 1px").unwrap();
    let CssContributions::Longhands(items) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("terminals")
    };
    assert_eq!(items.items().len(), 5);
    for item in items.items() {
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.replacement_components(), Some(&replacement));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
    assert_eq!(source, report.syntax()[0]);
}
