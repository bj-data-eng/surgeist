#![forbid(unsafe_code)]
//! Independent contract cases for the selected public metadata and grammar slice.
//! CSS Box 3, Backgrounds 3, Cascade 5, Color 4, Fonts 4, Writing Modes 4,
//! Variables 1, Conditional Rules 5, Display 3, and the pinned Grid 3 define values and shorthand semantics.
//! Grammar-handle identity, explicit unavailable metadata, and source occurrence
//! retention are Surgeist public contracts. No contextual style is resolved.
use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const LONGHANDS: &[P] = &[
    P::ContainerName,
    P::ContainerType,
    P::MarginTop,
    P::MarginRight,
    P::MarginBottom,
    P::MarginLeft,
    P::PaddingTop,
    P::PaddingRight,
    P::PaddingBottom,
    P::PaddingLeft,
    P::BorderTopWidth,
    P::BorderRightWidth,
    P::BorderBottomWidth,
    P::BorderLeftWidth,
    P::BorderTopStyle,
    P::BorderRightStyle,
    P::BorderBottomStyle,
    P::BorderLeftStyle,
    P::BorderTopColor,
    P::BorderRightColor,
    P::BorderBottomColor,
    P::BorderLeftColor,
    P::BorderImageSource,
    P::BorderImageSlice,
    P::BorderImageWidth,
    P::BorderImageOutset,
    P::BorderImageRepeat,
    P::FlowTolerance,
    P::Color,
    P::FontFamily,
    P::TextOrientation,
    P::Opacity,
    P::Display,
    P::Order,
];
const SHORTHANDS: &[(P, &[P], &[P])] = &[
    (P::Container, &[P::ContainerName, P::ContainerType], &[]),
    (
        P::Margin,
        &[P::MarginTop, P::MarginRight, P::MarginBottom, P::MarginLeft],
        &[],
    ),
    (
        P::Padding,
        &[
            P::PaddingTop,
            P::PaddingRight,
            P::PaddingBottom,
            P::PaddingLeft,
        ],
        &[],
    ),
    (
        P::BorderWidth,
        &[
            P::BorderTopWidth,
            P::BorderRightWidth,
            P::BorderBottomWidth,
            P::BorderLeftWidth,
        ],
        &[],
    ),
    (
        P::BorderStyle,
        &[
            P::BorderTopStyle,
            P::BorderRightStyle,
            P::BorderBottomStyle,
            P::BorderLeftStyle,
        ],
        &[],
    ),
    (
        P::BorderColor,
        &[
            P::BorderTopColor,
            P::BorderRightColor,
            P::BorderBottomColor,
            P::BorderLeftColor,
        ],
        &[],
    ),
    (
        P::BorderTop,
        &[P::BorderTopWidth, P::BorderTopStyle, P::BorderTopColor],
        &[],
    ),
    (
        P::BorderRight,
        &[
            P::BorderRightWidth,
            P::BorderRightStyle,
            P::BorderRightColor,
        ],
        &[],
    ),
    (
        P::BorderBottom,
        &[
            P::BorderBottomWidth,
            P::BorderBottomStyle,
            P::BorderBottomColor,
        ],
        &[],
    ),
    (
        P::BorderLeft,
        &[P::BorderLeftWidth, P::BorderLeftStyle, P::BorderLeftColor],
        &[],
    ),
    (
        P::Border,
        &[
            P::BorderTopWidth,
            P::BorderRightWidth,
            P::BorderBottomWidth,
            P::BorderLeftWidth,
            P::BorderTopStyle,
            P::BorderRightStyle,
            P::BorderBottomStyle,
            P::BorderLeftStyle,
            P::BorderTopColor,
            P::BorderRightColor,
            P::BorderBottomColor,
            P::BorderLeftColor,
        ],
        &[
            P::BorderImageSource,
            P::BorderImageSlice,
            P::BorderImageWidth,
            P::BorderImageOutset,
            P::BorderImageRepeat,
        ],
    ),
];
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
fn assert_ordinary_initial(property: P, initial: &CssLonghandInitialValue) {
    assert_eq!(initial.property().known_property(), property);
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("fixed intrinsic initial")
    };
    assert_eq!(value.property().known_property(), property);
    match value.view() {
        CssLonghandValueRef::ContainerName(v) => assert_eq!(*v, CssContainerNames::None),
        CssLonghandValueRef::ContainerType(v) => assert_eq!(*v, CssContainerType::Normal),
        CssLonghandValueRef::MarginTop(v)
        | CssLonghandValueRef::MarginRight(v)
        | CssLonghandValueRef::MarginBottom(v)
        | CssLonghandValueRef::MarginLeft(v)
        | CssLonghandValueRef::PaddingTop(v)
        | CssLonghandValueRef::PaddingRight(v)
        | CssLonghandValueRef::PaddingBottom(v)
        | CssLonghandValueRef::PaddingLeft(v) => assert_eq!(*v, CssLength::Zero),
        CssLonghandValueRef::BorderTopWidth(v)
        | CssLonghandValueRef::BorderRightWidth(v)
        | CssLonghandValueRef::BorderBottomWidth(v)
        | CssLonghandValueRef::BorderLeftWidth(v) => assert_eq!(*v, CssLength::Medium),
        CssLonghandValueRef::BorderTopStyle(v)
        | CssLonghandValueRef::BorderRightStyle(v)
        | CssLonghandValueRef::BorderBottomStyle(v)
        | CssLonghandValueRef::BorderLeftStyle(v) => assert_eq!(*v, CssBorderStyle::None),
        CssLonghandValueRef::BorderTopColor(v)
        | CssLonghandValueRef::BorderRightColor(v)
        | CssLonghandValueRef::BorderBottomColor(v)
        | CssLonghandValueRef::BorderLeftColor(v) => assert!(v.is_current_color()),
        CssLonghandValueRef::BorderImageSource(v) => assert!(matches!(v, CssImageValue::None)),
        CssLonghandValueRef::BorderImageSlice(v) => {
            assert!(!v.fill());
            assert!(!v.values().is_empty());
            assert!(v.values().iter().all(
                |v| matches!(v, CssBorderImageSliceComponent::Percentage(n) if n.value() == 100.0)
            ));
        }
        CssLonghandValueRef::BorderImageWidth(v) => {
            assert!(!v.values().is_empty());
            assert!(
                v.values().iter().all(
                    |v| matches!(v, CssBorderImageWidthComponent::Number(n) if n.value() == 1.0)
                )
            );
        }
        CssLonghandValueRef::BorderImageOutset(v) => {
            assert!(!v.values().is_empty());
            assert!(v.values().iter().all(
                |v| matches!(v, CssBorderImageOutsetComponent::Number(n) if n.value() == 0.0)
            ));
        }
        CssLonghandValueRef::BorderImageRepeat(v) => {
            assert_eq!(v.horizontal(), CssBorderImageRepeatKeyword::Stretch);
            assert_eq!(v.vertical(), CssBorderImageRepeatKeyword::Stretch);
        }
        CssLonghandValueRef::Color(v) => {
            assert_eq!(v.system(), Some(CssAuthoredSystemColor::CanvasText))
        }
        CssLonghandValueRef::TextOrientation(v) => assert_eq!(*v, CssTextOrientation::Mixed),
        CssLonghandValueRef::FlowTolerance(v) => assert_eq!(v, &CssFlowTolerance::normal()),
        CssLonghandValueRef::Order(v) => assert_eq!(v, &CssIntegerValue::Literal(0)),
        CssLonghandValueRef::Display(v) => assert_eq!(
            *v,
            CssDisplayValue::OutsideInside {
                outside: CssDisplayOutside::Inline,
                inside: CssDisplayInside::Flow,
            },
        ),
        CssLonghandValueRef::Opacity(v) => {
            assert!(matches!(v, CssOpacityValue::Literal(value) if value.value() == 1.0));
        }
        other => panic!("unexpected ordinary initial: {other:?}"),
    }
}
fn metadata_and_initials() {
    let expected: Vec<_> = LONGHANDS
        .iter()
        .copied()
        .chain(SHORTHANDS.iter().map(|(p, _, _)| *p))
        .chain([P::All])
        .collect();
    assert_eq!(expected.len(), 46);
    let mut observed = Vec::new();
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
        match property.metadata() {
            Ok(metadata) => {
                assert!(expected.contains(&property));
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
    assert_eq!(observed.len(), expected.len());
    for &property in LONGHANDS {
        let metadata = longhand(property);
        assert_eq!(metadata.property().known_property(), property);
        assert_eq!(
            metadata.inherited_by_default(),
            matches!(property, P::Color | P::FontFamily | P::TextOrientation)
        );
        let initial = metadata.initial_value();
        if property == P::FontFamily {
            assert_eq!(initial.property().known_property(), P::FontFamily);
            let CssInitialValueRef::UserAgent(requirement) = initial.view() else {
                panic!("font family is UA-dependent")
            };
            assert_eq!(requirement, CssUserAgentInitial::FontFamily);
            assert_eq!(requirement.property().known_property(), P::FontFamily);
        } else {
            assert_ordinary_initial(property, &initial);
        }
    }
    let width_support: CssPropertySupportMetadata = property_support_metadata("width")
        .expect("recognized support catalog remains available without intrinsic metadata");
    assert_eq!(width_support.property(), P::Width);
    assert_eq!(width_support.canonical_name(), "width");
    assert_eq!(
        width_support.feature().id().as_str(),
        "baseline.property.width"
    );
    assert_eq!(
        width_support.feature().id(),
        P::Width.grammar().feature_id()
    );
    for property in [P::Width, P::Font, P::TextAlign] {
        assert!(
            matches!(property.metadata(), Err(CssPropertyMetadataError::Unavailable(g)) if g == property.grammar())
        );
    }
    println!("metadata recognition, inheritance and intrinsic initial values: ok");
}
fn memberships_and_resets() {
    for &(property, settable, reset) in SHORTHANDS {
        let CssPropertyKindRef::Shorthand(meta) = property.metadata().unwrap().kind() else {
            panic!("shorthand")
        };
        assert!(!meta.is_legacy());
        assert_eq!(names(meta.settable_members()), settable);
        assert_eq!(names(meta.reset_only_members()), reset);
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
            assert_eq!(color.system(), Some(CssAuthoredSystemColor::CanvasText));
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
