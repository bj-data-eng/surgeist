#![forbid(unsafe_code)]

//! Existing public-boundary controls for CSSOM declaration-block work (#831).
//! Independent authority: the pinned CSSOM 1 WD 2021-08-26, §§6.5, 6.6
//! (`parse-a-css-declaration-block`) and 6.7.1–6.7.2; Variables 1 §2 supplies
//! case-sensitive custom names. Speech 1 §§8.1–8.2 supplies the selected pair
//! shorthand grammar used by the canonical-value control.
//!
//! These controls do not claim the absent declaration/block serializers,
//! preferred shorthand ordering, reverse reconstruction, mutable priority or
//! explicit case-flag API. New APIs need functional tests with implementation.
//! Authored component serialization is lexical, not CSSOM canonical value text.

use surgeist_css::*;

fn authored_value(declaration: &CssDeclaration) -> &str {
    let origin = declaration.parsed_value().unwrap();
    &origin.source().as_str()
        [origin.span().start().byte_offset().value()..origin.span().end().byte_offset().value()]
}

#[test]
fn declaration_block_drops_invalid_units_and_retains_valid_source_order() {
    // CSSOM §6.6: parse the declaration list, apply each owning grammar and
    // discard invalid declarations without discarding following valid ones.
    let source =
        "width:1px; unknown:ignored; width:red; broken; --Case:{a:b;c:d}; height:2px!important";
    let report = parse_style_attribute(source);
    let declarations = report.syntax().as_slice();
    assert_eq!(declarations.len(), 3, "{report:?}");
    assert_eq!(
        declarations[0].property_name(),
        CssPropertyNameRef::Known(CssKnownProperty::Width)
    );
    assert_eq!(declarations[1].custom().unwrap().name().as_str(), "--Case");
    assert_eq!(
        declarations[2].property_name(),
        CssPropertyNameRef::Known(CssKnownProperty::Height)
    );
    assert_eq!(declarations[2].importance(), CssImportance::Important);
    assert!(!report.is_clean());
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );
    let starts: Vec<_> = declarations
        .iter()
        .map(|declaration| declaration.position().unwrap().byte_offset().value())
        .collect();
    assert_eq!(
        starts,
        vec![
            0,
            source.find("--Case").unwrap(),
            source.find("height").unwrap()
        ]
    );
}

#[test]
fn parsing_preserves_duplicate_occurrences_before_cssom_serialization() {
    // Parsing (§6.6) retains source order. The separate serialization algorithm
    // accounts for already-serialized names; parsing must not erase history.
    let report = parse_style_attribute("width:1px!important; width:2px; width:3px!important");
    assert!(report.is_clean(), "{report:?}");
    let declarations = report.syntax().as_slice();
    assert_eq!(declarations.len(), 3);
    for declaration in declarations {
        assert_eq!(
            declaration.property_name(),
            CssPropertyNameRef::Known(CssKnownProperty::Width)
        );
    }
    assert_eq!(
        declarations
            .iter()
            .map(CssDeclaration::importance)
            .collect::<Vec<_>>(),
        vec![
            CssImportance::Important,
            CssImportance::Normal,
            CssImportance::Important
        ]
    );
    assert!(!declarations[0].same_occurrence(&declarations[1]));
    assert!(declarations[0].same_occurrence(&declarations[0].clone()));
    assert_eq!(
        declarations.iter().map(authored_value).collect::<Vec<_>>(),
        vec!["1px", "2px", "3px"]
    );
}

#[test]
fn known_names_are_insensitive_and_custom_names_retain_case_and_components() {
    // CSSOM §6.5 derives name sensitivity from the owning specification;
    // Variables 1 §2 sets the custom-property case-sensitive flag.
    let report = parse_style_attribute("WiDtH:1px; --Case:Mixed; --case:other; --Empty:!IMPORTANT");
    assert!(report.is_clean(), "{report:?}");
    let declarations = report.syntax().as_slice();
    assert_eq!(declarations.len(), 4);
    assert_eq!(
        declarations[0].property_name(),
        CssPropertyNameRef::Known(CssKnownProperty::Width)
    );
    assert_eq!(declarations[1].custom().unwrap().name().as_str(), "--Case");
    assert_eq!(declarations[2].custom().unwrap().name().as_str(), "--case");
    assert_ne!(
        declarations[1].property_name(),
        declarations[2].property_name()
    );
    assert_eq!(
        declarations[1]
            .value_components()
            .serialize()
            .unwrap()
            .as_css(),
        "Mixed"
    );
    assert_eq!(declarations[3].importance(), CssImportance::Important);
    assert_eq!(declarations[3].value_components().items().len(), 0);
}

#[test]
fn empty_and_wholly_invalid_blocks_have_no_retained_declarations() {
    // CSSOM §6.6 returns an empty list when no declaration survives.
    for source in ["", " /**/ ; ; "] {
        let report = parse_style_attribute(source);
        assert!(report.syntax().is_empty(), "{source:?}: {report:?}");
    }
    let report = parse_style_attribute("unknown:x; width:red; height:");
    assert!(report.syntax().is_empty(), "{report:?}");
    assert!(!report.is_clean());
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );
}

#[test]
fn property_value_reentry_matches_the_entire_grammar_and_excludes_priority() {
    // CSSOM §6.7.1 explicitly returns null for grammar failure, including
    // !important. Importance is supplied separately to the existing API.
    for source in [
        "red",
        "1px extra",
        "1px!important",
        "1px!/**/IMPORTANT",
        "1px;",
        "inherit extra",
        "var(foo)",
    ] {
        let report = parse_property_value_text(
            source,
            CssPropertyNameRef::Known(CssKnownProperty::Width),
            CssImportance::Normal,
        );
        assert!(report.syntax().is_none(), "{source:?}: {report:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
        );
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Width),
                parse_component_values(source).unwrap(),
                CssImportance::Normal
            )
            .is_err(),
            "{source:?}"
        );
    }
    for source in ["1px", " /**/ 1px ", "inherit", "var(--width)"] {
        let report = parse_property_value_text(
            source,
            CssPropertyNameRef::Known(CssKnownProperty::Width),
            CssImportance::Important,
        );
        assert!(report.is_clean(), "{source:?}: {report:?}");
        let declaration = report.syntax().as_ref().unwrap();
        assert_eq!(declaration.importance(), CssImportance::Important);
        assert!(declaration.parsed_name().is_none());
        assert_eq!(
            declaration.parsed_value().unwrap().source().as_str(),
            source
        );
    }
}

#[test]
fn owning_pair_serializer_shortens_redundancy_and_reenters_its_grammar() {
    // CSSOM §6.7.2 omits redundant optional components. The owning Speech
    // grammar gives one value both sides; its model retains explicit omission
    // identity, so compare expanded semantic sides rather than authored trees.
    let report = parse_style_attribute("pause:1s 1000ms!important");
    assert!(report.is_clean(), "{report:?}");
    let declaration = &report.syntax()[0];
    let CssKnownPropertyValueRef::Pause(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("pause owner")
    };
    let retained = value.value().clone();
    assert!(retained.authored_after().is_some());
    let canonical = retained.serialize_specified().unwrap();
    assert_eq!(canonical, "1s");
    assert_eq!(value.value(), &retained);
    let reparsed = parse_property_value_text(
        &canonical,
        CssPropertyNameRef::Known(CssKnownProperty::Pause),
        CssImportance::Important,
    );
    assert!(reparsed.is_clean(), "{reparsed:?}");
    for source in [declaration, reparsed.syntax().as_ref().unwrap()] {
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(source).unwrap()
        else {
            panic!("complete pause expansion")
        };
        assert_eq!(
            values
                .items()
                .iter()
                .map(CssLonghandContribution::property)
                .collect::<Vec<_>>(),
            vec![CssKnownProperty::PauseBefore, CssKnownProperty::PauseAfter]
        );
        for contribution in values.items() {
            assert_eq!(contribution.source().importance(), CssImportance::Important);
            match contribution.ordinary_value().unwrap().view() {
                CssLonghandValueRef::PauseBefore(value) => {
                    assert_eq!(value.serialize_specified().unwrap(), "1s")
                }
                CssLonghandValueRef::PauseAfter(value) => {
                    assert_eq!(value.serialize_specified().unwrap(), "1s")
                }
                _ => panic!("pause longhand owner"),
            }
        }
    }
}
