#![forbid(unsafe_code)]
//! Independent contract cases for the selected public metadata and grammar slice.
//! CSS Box 3, Backgrounds 3, Cascade 5, Color 4, Fonts 4, Writing Modes 4,
//! Variables 1, Conditional Rules 5, Display 3, Containment 2, Sizing 3/4,
//! the pinned Grid 3 and Speech 1 define values and shorthand semantics.
//! Speech 1 CRD 2023-02-14 §§7–12 and Property Index supply nineteen names,
//! intrinsic initials, inheritance and before/after shorthand membership:
//! https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#property-index
//! Grid 2 (2025-03-26) §7.6 defines implicit track sizes as noninherited,
//! initially one `auto` breadth: https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#auto-tracks
//! Grammar-handle identity, explicit unavailable metadata, and source occurrence
//! retention are Surgeist public contracts. No contextual style is resolved.
//! Selected Will Change 1, Scroll Anchoring 1, Fragmentation 3 and Scrollbars 1
//! define auto/slice initials; only scrollbar-color inherits among these controls.
//! Transitions 1 WD 2026-01-08 §§2.1–2.5 defines four noninherited one-entry
//! initials and the selected shorthand's four settable members without extra resets.
//! Animations 1 WD 2023-03-02 §§3.2–3.10 defines eight noninherited
//! one-entry initials and eight shorthand members without extra resets.
//! Compositing 1 CRD 2024-03-21 §§3.1, 3.4.2, 3.4.3 defines noninherited
//! blend/isolation initials and background's reset-only background-blend-mode.
//! Text 4 WD 2026-08-14 §§3–6 defines wrapping/whitespace constituents,
//! their exact inherited initials and two/three-member shorthand projections.
//! Text 4 §§2–9 defines transformation, separator, tab, breaking and indent
//! longhands; CSS2 §10.8.1 defines the noninherited baseline vertical alignment.
#[path = "../tests/common/property_expectations.rs"]
mod property_expectations;
use property_expectations::{CASES, InitialExpectation, MetadataExpectation};

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).expect("recognized decoded property grammar")
}
fn longhand(property: P) -> &'static CssLonghandMetadata {
    match property.metadata().unwrap().kind() {
        CssPropertyKindRef::Longhand(value) => value,
        other => panic!("longhand metadata: {other:?}"),
    }
}
fn names(ids: &[CssLonghandProperty]) -> Vec<P> {
    ids.iter().map(|id| id.known_property()).collect()
}
fn construct(grammar: CssPropertyGrammar, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar,
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap()
}
fn expanded(source: &CssDeclaration) -> CssLonghandContributions {
    match expand_declaration(source).unwrap() {
        CssExpansion::Contributions(CssContributions::Longhands(values)) => values,
        other => panic!("completed typed values: {other:?}"),
    }
}
fn pending(source: &CssDeclaration) -> CssPendingSubstitution {
    match expand_declaration(source).unwrap() {
        CssExpansion::Pending(value) => value,
        other => panic!("pending grammar: {other:?}"),
    }
}

fn metadata_and_initials() {
    let mut identities = std::collections::HashSet::new();
    for case in CASES {
        assert!(
            identities.insert(case.property),
            "duplicate property expectation: {}",
            case.name
        );
    }
    let expected: Vec<_> = CASES
        .iter()
        .filter(|case| !matches!(case.metadata, MetadataExpectation::Unavailable))
        .map(|case| case.property)
        .collect();
    let mut observed = Vec::new();
    let mut unexpected = Vec::new();
    for &property in P::all() {
        let handle = property.grammar();
        assert_eq!(handle.target_property(), property);
        assert_eq!(handle.name(), property.canonical_name());
        assert_eq!(
            grammar(&property.canonical_name().to_ascii_uppercase()),
            handle
        );
        assert_eq!(handle.feature_id().as_str(), property.stable_id());
        for alias in property.aliases() {
            assert_eq!(grammar(alias), handle);
        }
        if let Some(case) = property_expectations::find(property) {
            assert_eq!(handle.name(), case.name);
            if let Some(feature_id) = case.feature_id() {
                assert_eq!(handle.feature_id().as_str(), feature_id);
                let support = property_support_metadata(case.name)
                    .expect("independently expected support catalog");
                assert_eq!(support.property(), case.property);
                assert_eq!(support.canonical_name(), case.name);
                assert_eq!(support.feature().id().as_str(), feature_id);
            }
        }
        match property.metadata() {
            Ok(metadata) => {
                if !expected.contains(&property) {
                    unexpected.push(property);
                }
                assert_eq!(metadata.grammar(), handle);
                observed.push(property);
            }
            Err(CssPropertyMetadataError::Unavailable(g)) => {
                assert!(!expected.contains(&property));
                assert_eq!(g, handle);
            }
            other => panic!("unexpected capability result: {other:?}"),
        }
    }
    assert!(unexpected.is_empty(), "unexpected metadata: {unexpected:?}");
    assert_eq!(observed.len(), expected.len());
    for case in CASES {
        match &case.metadata {
            MetadataExpectation::Longhand { inherited, initial } => {
                let metadata = longhand(case.property);
                assert_eq!(metadata.property().known_property(), case.property);
                assert_eq!(
                    metadata.inherited_by_default(),
                    *inherited,
                    "{} inheritance",
                    case.name
                );
                let actual = metadata.initial_value();
                assert_eq!(actual.property().known_property(), case.property);
                match initial {
                    InitialExpectation::Fixed(assertion) => {
                        let CssInitialValueRef::Value(value) = actual.view() else {
                            panic!("{} fixed intrinsic initial", case.name);
                        };
                        assert_eq!(value.property().known_property(), case.property);
                        assertion(value.view());
                    }
                    InitialExpectation::UserAgent(expected) => {
                        let CssInitialValueRef::UserAgent(actual) = actual.view() else {
                            panic!("{} UA-dependent initial", case.name);
                        };
                        assert_eq!(actual, *expected);
                        assert_eq!(actual.property().known_property(), case.property);
                    }
                }
            }
            MetadataExpectation::FourSide => assert!(matches!(
                case.property.metadata().unwrap().kind(),
                CssPropertyKindRef::FourSideShorthand(_)
            )),
            MetadataExpectation::UniversalReset => assert!(matches!(
                case.property.metadata().unwrap().kind(),
                CssPropertyKindRef::UniversalReset(_)
            )),
            MetadataExpectation::Shorthand { .. } | MetadataExpectation::Unavailable => {}
        }
    }
    println!("metadata recognition, inheritance and intrinsic initial values: ok");
}
fn memberships_and_resets() {
    for case in CASES {
        let MetadataExpectation::Shorthand { settable, reset } = &case.metadata else {
            continue;
        };
        let (property, settable, reset) = (case.property, *settable, *reset);
        let CssPropertyKindRef::Shorthand(meta) = property.metadata().unwrap().kind() else {
            panic!("shorthand")
        };
        assert!(!meta.is_legacy());
        assert_eq!(
            names(meta.settable_members()),
            settable,
            "{} settable members",
            property.canonical_name()
        );
        assert_eq!(
            names(meta.reset_only_members()),
            reset,
            "{} reset-only members",
            property.canonical_name()
        );
        let expected: Vec<_> = settable.iter().chain(reset).copied().collect();
        assert_eq!(names(meta.members()), expected);
        for (index, member) in expected.iter().enumerate() {
            assert!(!expected[..index].contains(member));
        }
        let source = construct(property.grammar(), "initial");
        let values = expanded(&source);
        assert_eq!(
            values
                .items()
                .iter()
                .map(|v| v.property())
                .collect::<Vec<_>>(),
            expected
        );
        for value in values.items() {
            assert!(matches!(
                value.value(),
                CssContributionValueRef::Global(CssGlobalKeyword::Initial)
            ));
            assert!(value.ordinary_value().is_none());
            assert!(value.source().same_occurrence(&source));
            assert_eq!(value.source().importance(), CssImportance::Important);
        }
    }
    let source = construct(P::ContainIntrinsicSize.grammar(), "auto 5px none");
    let values = expanded(&source);
    assert_eq!(
        values
            .items()
            .iter()
            .map(|value| value.property())
            .collect::<Vec<_>>(),
        [P::ContainIntrinsicWidth, P::ContainIntrinsicHeight]
    );
    let [width, height] = values.items() else {
        panic!("two contained intrinsic-size members")
    };
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::ContainIntrinsicWidth(width)) =
        width.value()
    else {
        panic!("typed contained intrinsic width")
    };
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::ContainIntrinsicHeight(height)) =
        height.value()
    else {
        panic!("typed contained intrinsic height")
    };
    assert_eq!(width.serialize_specified().unwrap(), "auto 5px");
    assert_eq!(height.serialize_specified().unwrap(), "none");
    let CssPropertyKindRef::UniversalReset(meta) = P::All.metadata().unwrap().kind() else {
        panic!("universal reset")
    };
    let custom = CssCustomPropertyName::try_new("--theme").unwrap();
    for (name, excluded) in [
        (CssPropertyNameRef::Known(P::Direction), true),
        (CssPropertyNameRef::Known(P::UnicodeBidi), true),
        (CssPropertyNameRef::Custom(&custom), true),
        (CssPropertyNameRef::Known(P::Color), false),
        (CssPropertyNameRef::Known(P::Opacity), false),
        (CssPropertyNameRef::Known(P::Width), false),
    ] {
        assert_eq!(meta.excludes(name), excluded);
        let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
            expand_declaration(&construct(P::All.grammar(), "unset")).unwrap()
        else {
            panic!("all reset")
        };
        assert_eq!(reset.excludes(name), excluded);
    }
    let border = expanded(&construct(P::Border.grammar(), "solid"));
    for value in border.items() {
        let ordinary = value
            .ordinary_value()
            .expect("ordinary plus intrinsic omitted/reset values");
        assert_eq!(ordinary.property().known_property(), value.property());
        assert!(
            matches!(value.value(), CssContributionValueRef::Ordinary(v) if v == ordinary.view())
        );
        if value.property() != P::BorderTopStyle
            && value.property() != P::BorderRightStyle
            && value.property() != P::BorderBottomStyle
            && value.property() != P::BorderLeftStyle
        {
            let initial = longhand(value.property()).initial_value();
            let CssInitialValueRef::Value(expected) = initial.view() else {
                panic!("fixed border initial")
            };
            assert_eq!(ordinary.view(), expected.view());
        }
    }
    for property in [P::GridTemplate, P::Grid] {
        let values = expanded(&construct(property.grammar(), "none"));
        for value in values.items() {
            let initial = longhand(value.property()).initial_value();
            let CssInitialValueRef::Value(expected) = initial.view() else {
                panic!("fixed Grid initial");
            };
            assert_eq!(value.ordinary_value().unwrap().view(), expected.view());
        }
        let source = construct(property.grammar(), "[top] \"a\" [end] / 1px");
        let values = expanded(&source);
        let CssContributionValueRef::Ordinary(CssLonghandValueRef::GridTemplateRows(rows)) =
            values.items()[0].value()
        else {
            panic!("effective rows");
        };
        assert_eq!(rows.serialize_specified().unwrap(), "[top] auto [end]");
        let CssContributionValueRef::Ordinary(CssLonghandValueRef::GridTemplateColumns(columns)) =
            values.items()[1].value()
        else {
            panic!("effective columns");
        };
        assert_eq!(columns.serialize_specified().unwrap(), "1px");
    }
    println!("terminal membership, omissions, reset-only values and all exclusions: ok");
}
fn grammar_identity_and_reentry() {
    let legacy = grammar("GLYPH-ORIENTATION-VERTICAL");
    let canonical = P::TextOrientation.grammar();
    assert_ne!(legacy, canonical);
    assert_eq!(legacy.name(), "glyph-orientation-vertical");
    assert_eq!(legacy.target_property(), P::TextOrientation);
    assert_eq!(
        legacy.feature_id().as_str(),
        "official.property-alias.glyph-orientation-vertical"
    );
    assert_eq!(P::TextOrientation.legacy_shorthands(), &[legacy]);
    assert_eq!(P::from_name("glyph-orientation-vertical"), None);
    for invalid in [" color", "color ", "co\\6cor", "--color", "not-a-property"] {
        assert!(CssPropertyGrammar::from_name(invalid).is_none());
    }
    let CssPropertyKindRef::Shorthand(meta) = legacy.metadata().unwrap().kind() else {
        panic!("legacy shorthand metadata")
    };
    assert!(meta.is_legacy());
    assert_eq!(names(meta.members()), [P::TextOrientation]);
    assert_eq!(names(meta.settable_members()), [P::TextOrientation]);
    assert!(meta.reset_only_members().is_empty());
    for css in ["initial", "var(--angle)"] {
        let old = construct(legacy, css);
        let new = construct(canonical, css);
        assert_eq!(old.known().unwrap().grammar(), legacy);
        assert_eq!(new.known().unwrap().grammar(), canonical);
        assert_ne!(old, new, "grammar identity affects structural equality");
        assert!(old.same_occurrence(&old.clone()));
        assert!(!old.same_occurrence(&construct(legacy, css)));
        assert_eq!(old, construct(grammar("glyph-orientation-vertical"), css));
    }
    let direct = construct(legacy, "90deg");
    assert_eq!(direct.known().unwrap().grammar(), legacy);
    let ordinary_canonical = construct(canonical, "sideways");
    assert_eq!(ordinary_canonical.known().unwrap().grammar(), canonical);
    assert_ne!(
        direct, ordinary_canonical,
        "ordinary equality retains grammar identity"
    );
    assert!(direct.same_occurrence(&direct.clone()));
    assert_eq!(direct, construct(legacy, "90deg"));
    assert!(matches!(
        expanded(&direct).items()[0].value(),
        CssContributionValueRef::Ordinary(CssLonghandValueRef::TextOrientation(
            CssTextOrientation::Sideways
        ))
    ));
    assert!(
        parse_property_value_for_grammar(
            canonical,
            parse_component_values("90deg").unwrap(),
            CssImportance::Normal
        )
        .is_err()
    );
    let source = construct(legacy, "var(--angle)");
    let handle = pending(&source);
    let replacement = parse_component_values("90deg").unwrap();
    assert!(
        handle
            .reenter(parse_component_values("bogus").unwrap())
            .is_err()
    );
    assert!(matches!(
        handle
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        CssExpansionErrorKind::ResidualSubstitution
    ));
    for _ in 0..2 {
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("validated longhand result")
        };
        assert_eq!(values.items().len(), 1);
        let value = &values.items()[0];
        assert_eq!(value.property(), P::TextOrientation);
        assert!(matches!(
            value.value(),
            CssContributionValueRef::Ordinary(CssLonghandValueRef::TextOrientation(
                CssTextOrientation::Sideways
            ))
        ));
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().known().unwrap().grammar(), legacy);
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert_eq!(
            value
                .replacement_components()
                .unwrap()
                .serialize()
                .unwrap()
                .as_css(),
            "90deg"
        );
        let actual_origin = value.replacement_components().unwrap().items()[0].origin();
        assert_eq!(actual_origin, replacement.items()[0].origin());
        let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) =
            (actual_origin, replacement.items()[0].origin())
        else {
            panic!("replacement retains parsed token identity")
        };
        assert!(actual.source().same_snapshot(expected.source()));
    }
    assert!(
        pending(&construct(canonical, "var(--angle)"))
            .reenter(replacement)
            .is_err()
    );
    let separate_tokens = CssComponentValues::try_new(vec![
        CssComponentValue::try_token("90").unwrap(),
        CssComponentValue::try_token("deg").unwrap(),
    ])
    .unwrap();
    assert!(
        parse_property_value_for_grammar(legacy, separate_tokens.clone(), CssImportance::Normal)
            .is_err()
    );
    assert!(
        handle.reenter(separate_tokens).is_err(),
        "number plus ident cannot accidentally become an angle dimension"
    );
    println!("legacy grammar identity and atomic reusable reentry: ok");
}
fn construction_provenance_and_normalization() {
    let css = "/*x*/g\\6cyph-orientation-vertical:90deg!important;color:CanvasText";
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    let old = &report.syntax()[0];
    assert_eq!(
        old.known().unwrap().grammar(),
        grammar("glyph-orientation-vertical")
    );
    assert_eq!(old.position().unwrap().byte_offset().value(), 5);
    assert!(old.parsed_name().is_some());
    assert!(old.parsed_value().is_some());
    let constructed = construct(grammar("glyph-orientation-vertical"), "90deg");
    assert!(constructed.position().is_none());
    assert!(constructed.parsed_name().is_none());
    assert!(constructed.parsed_value().is_none());
    assert_eq!(old.known(), constructed.known());
    for value in expanded(old).items() {
        assert!(value.source().same_occurrence(old));
        assert_eq!(value.source().parsed_name(), old.parsed_name());
    }
    let components =
        CssComponentValues::try_new(vec![CssComponentValue::try_dimension("90", "deg").unwrap()])
            .unwrap();
    let built = parse_property_value_for_grammar(
        grammar("glyph-orientation-vertical"),
        components,
        CssImportance::Normal,
    )
    .unwrap();
    assert!(built.position().is_none());
    assert!(
        built
            .value_components()
            .items()
            .iter()
            .all(|token| matches!(token.origin(), CssValueOrigin::Programmatic))
    );
    assert!(matches!(
        expanded(&built).items()[0].value(),
        CssContributionValueRef::Ordinary(CssLonghandValueRef::TextOrientation(
            CssTextOrientation::Sideways
        ))
    ));
    let font = expanded(&construct(P::FontFamily.grammar(), "serif"));
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::FontFamily(family)) =
        font.items()[0].value()
    else {
        panic!("ordinary authored family")
    };
    assert_eq!(family.families().len(), 1);
    assert_eq!(
        family.families()[0].generic_family(),
        Some(CssGenericFontFamily::Serif)
    );
    assert_eq!(
        font.items()[0]
            .ordinary_value()
            .unwrap()
            .property()
            .known_property(),
        P::FontFamily
    );
    let sheet = parse_sheet(".a {color:CanvasText!important; @media print {color:red} color:blue}");
    assert!(sheet.is_clean(), "{:?}", sheet.diagnostics());
    let normalized = normalize_sheet(sheet.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 3);
    for (index, declaration) in declarations.iter().enumerate() {
        assert_eq!(declaration.order(), index);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declaration.expansion()
        else {
            panic!("color longhand")
        };
        assert_eq!(values.items().len(), 1);
        let value = &values.items()[0];
        assert!(value.source().same_occurrence(declaration.source()));
        assert_eq!(value.property(), P::Color);
        let CssLonghandValueRef::Color(color) = value.ordinary_value().unwrap().view() else {
            panic!("authored color")
        };
        if index == 0 {
            assert_eq!(color.system(), Some(CssSystemColor::CanvasText));
            assert_eq!(value.source().importance(), CssImportance::Important);
        } else {
            assert_eq!(
                color.named().unwrap().name(),
                if index == 1 { "red" } else { "blue" }
            );
        }
    }
    assert!(
        declarations[0]
            .selector_context()
            .same_context(declarations[2].selector_context())
    );
    println!("parsed and constructed provenance with ordered color normalization: ok");
}
fn main() {
    metadata_and_initials();
    memberships_and_resets();
    grammar_identity_and_reentry();
    construction_provenance_and_normalization();
}
