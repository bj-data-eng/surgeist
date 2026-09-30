#![forbid(unsafe_code)]
//! Backgrounds 3 (2024-03-11), §§2.4, 2.5, 2.7, 2.8 and 2.9.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/
//! These noninherited list-valued longhands retain authored layer order.
//! Intrinsic expansion produces one terminal contribution per declaration,
//! without matching list lengths to background images or resolving used values.
//! Exact typed initials/new contribution variants/serializers are supplementary
//! functional tests once those APIs exist, rather than compile-failure RED.

use surgeist_css::*;

struct Case {
    property: CssKnownProperty,
    ordinary: &'static str,
    replacement: &'static str,
    programmatic: &'static str,
    invalid: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        property: CssKnownProperty::BackgroundSize,
        ordinary: "cover, 10px auto, 50%",
        replacement: "contain, calc(10px + 5%) auto",
        programmatic: "auto",
        invalid: &["-1px", "cover auto", "auto,"],
    },
    Case {
        property: CssKnownProperty::BackgroundRepeat,
        ordinary: "repeat-x, space round",
        replacement: "repeat-y, no-repeat",
        programmatic: "repeat",
        invalid: &["repeat-x round", "round,, repeat"],
    },
    Case {
        property: CssKnownProperty::BackgroundOrigin,
        ordinary: "border-box, content-box",
        replacement: "content-box, padding-box",
        programmatic: "padding-box",
        invalid: &["border-box padding-box", "content-box,"],
    },
    Case {
        property: CssKnownProperty::BackgroundClip,
        ordinary: "content-box, padding-box",
        replacement: "border-box, content-box",
        programmatic: "border-box",
        invalid: &["padding-box border-box", "border-box,"],
    },
    Case {
        property: CssKnownProperty::BackgroundAttachment,
        ordinary: "fixed, local",
        replacement: "scroll, fixed",
        programmatic: "scroll",
        invalid: &["fixed scroll", "local,"],
    },
];

fn declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    source.clone()
}

fn authored(case: &Case) -> CssDeclaration {
    declaration(&format!(
        "{}:{}!important",
        case.property.canonical_name(),
        case.ordinary
    ))
}

fn completed(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).expect("selected background list longhand must expand")
    else {
        panic!("completed longhand contribution")
    };
    let [item] = items.items() else {
        panic!("one property contribution regardless of layer count")
    };
    item.clone()
}

fn assert_ordinary(
    item: &CssLonghandContribution,
    source: &CssDeclaration,
    property: CssKnownProperty,
) {
    assert_eq!(item.property(), property);
    assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
    assert_eq!(
        item.ordinary_value().unwrap().property().known_property(),
        property
    );
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
}

macro_rules! metadata_case {
    ($name:ident, $property:ident) => {
        #[test]
        fn $name() {
            let property = CssKnownProperty::$property;
            let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
                panic!("selected property is a longhand")
            };
            assert_eq!(metadata.property().known_property(), property);
            assert!(!metadata.inherited_by_default());
        }
    };
}

metadata_case!(size_metadata_is_a_noninherited_longhand, BackgroundSize);
metadata_case!(repeat_metadata_is_a_noninherited_longhand, BackgroundRepeat);
metadata_case!(origin_metadata_is_a_noninherited_longhand, BackgroundOrigin);
metadata_case!(clip_metadata_is_a_noninherited_longhand, BackgroundClip);
metadata_case!(
    attachment_metadata_is_a_noninherited_longhand,
    BackgroundAttachment
);

#[test]
fn ordinary_authored_lists_expand_once_with_original_occurrence_and_importance() {
    for case in CASES {
        let source = authored(case);
        let item = completed(&source);
        assert_ordinary(&item, &source, case.property);
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
    }
}

#[test]
fn css_wide_keywords_remain_symbolic_on_each_selected_longhand() {
    for case in CASES {
        for (css, keyword) in [
            ("inherit", CssGlobalKeyword::Inherit),
            ("initial", CssGlobalKeyword::Initial),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(&format!(
                "{}:{css}!important",
                case.property.canonical_name()
            ));
            let item = completed(&source);
            assert_eq!(item.property(), case.property);
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.ordinary_value().is_none());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
}

#[test]
fn pending_lists_reenter_strictly_repeatably_with_replacement_origins() {
    for case in CASES {
        let source = declaration(&format!(
            "{}:var(--layer)!important",
            case.property.canonical_name()
        ));
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("substitution-dependent longhand is pending")
        };
        assert!(pending.source().same_occurrence(&source));
        for invalid in case.invalid {
            let error = pending
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err();
            assert!(matches!(
                error.kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
        }
        for residual in ["var(--again)", "env(layer)", "attr(layer)"] {
            assert_eq!(
                pending
                    .reenter(parse_component_values(residual).unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
        }
        let parsed = parse_component_values(case.replacement).unwrap();
        let programmatic = CssComponentValues::try_new(vec![
            CssComponentValue::try_ident(case.programmatic).unwrap(),
        ])
        .unwrap();
        for replacement in [parsed, programmatic] {
            for _ in 0..2 {
                let CssContributions::Longhands(items) =
                    pending.reenter(replacement.clone()).unwrap()
                else {
                    panic!("completed replacement")
                };
                let [item] = items.items() else {
                    panic!("one terminal replacement")
                };
                assert_ordinary(item, &source, case.property);
                assert_eq!(item.replacement_components(), Some(&replacement));
                assert_eq!(
                    item.replacement_components().unwrap().items()[0].origin(),
                    replacement.items()[0].origin()
                );
            }
        }
        let global = parse_component_values("revert-layer").unwrap();
        let CssContributions::Longhands(items) = pending.reenter(global.clone()).unwrap() else {
            panic!("completed global replacement")
        };
        let [item] = items.items() else {
            panic!("one symbolic global terminal")
        };
        assert_eq!(item.property(), case.property);
        assert_eq!(
            item.value(),
            CssContributionValueRef::Global(CssGlobalKeyword::RevertLayer)
        );
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&global));
        assert!(pending.source().same_occurrence(&source));
    }
}

#[test]
fn normalization_preserves_ordered_runs_and_one_unit_per_longhand_or_pending_group() {
    let css = concat!(
        ".a{background-size:cover,10px auto!important; background-repeat:repeat-x,space round}",
        ".b{background-origin:border-box,content-box; background-clip:content-box,padding-box;",
        "background-attachment:fixed,local; background-size:var(--layer)}"
    );
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    let [size, repeat, origin, clip, attachment, pending] = declarations.as_slice() else {
        panic!("six ordered authored occurrences across two rules")
    };
    for (index, (value, case)) in [size, repeat, origin, clip, attachment]
        .into_iter()
        .zip(CASES)
        .enumerate()
    {
        assert_eq!(value.order(), index);
        assert_eq!(value.source().known().unwrap().property(), case.property);
        let CssExpansion::Contributions(CssContributions::Longhands(items)) = value.expansion()
        else {
            panic!("completed list longhand")
        };
        let [item] = items.items() else {
            panic!("one terminal, independently of layer count")
        };
        assert_ordinary(item, value.source(), case.property);
    }
    assert_eq!(size.source().importance(), CssImportance::Important);
    assert_eq!(
        size.source().position().unwrap().byte_offset().value(),
        css.find("background-size").unwrap()
    );
    assert_eq!(pending.order(), 5);
    assert_eq!(
        pending.source().known().unwrap().property(),
        CssKnownProperty::BackgroundSize
    );
    let CssExpansion::Pending(handle) = pending.expansion() else {
        panic!("symbolic final occurrence")
    };
    assert!(handle.source().same_occurrence(pending.source()));
    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 5).unwrap();
    let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 5,
        }
    );
    assert_eq!(error.declaration_order(), Some(5));
    assert_eq!(
        error.declaration().unwrap().known().unwrap().property(),
        CssKnownProperty::BackgroundSize
    );
}

#[test]
fn parsed_lists_preserve_exact_ordered_size_repeat_box_and_attachment_payloads() {
    for case in CASES {
        let source = authored(case);
        assert!(matches!(
            source.value_components().items()[0].origin(),
            CssValueOrigin::Parsed(_)
        ));
        match source.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::BackgroundSize(value) => {
                let [
                    CssBackgroundSize::Cover,
                    CssBackgroundSize::Explicit {
                        width: CssBackgroundSizeComponent::Length(width),
                        height: Some(CssBackgroundSizeComponent::Auto),
                    },
                    CssBackgroundSize::Explicit {
                        width: CssBackgroundSizeComponent::Length(last),
                        height: None,
                    },
                ] = value.sizes().sizes()
                else {
                    panic!("cover, explicit width/auto, then width with omitted second component")
                };
                assert!(matches!(width.literal_component().unwrap().view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
                    if number.representation() == "10" && unit == "px"));
                assert!(matches!(last.literal_component().unwrap().view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
                    if number.representation() == "50"));
                assert!(matches!(width.origin(), CssValueOrigin::Parsed(_)));
            }
            CssKnownPropertyValueRef::BackgroundRepeat(value) => assert_eq!(
                value.repeats().repeats(),
                &[
                    CssBackgroundRepeat::RepeatX,
                    CssBackgroundRepeat::Axes {
                        x: CssBackgroundRepeatStyle::Space,
                        y: CssBackgroundRepeatStyle::Round
                    },
                ]
            ),
            CssKnownPropertyValueRef::BackgroundOrigin(value) => assert_eq!(
                value.boxes().boxes(),
                &[CssBackgroundBox::BorderBox, CssBackgroundBox::ContentBox]
            ),
            CssKnownPropertyValueRef::BackgroundClip(value) => assert_eq!(
                value.boxes().boxes(),
                &[CssBackgroundBox::ContentBox, CssBackgroundBox::PaddingBox]
            ),
            CssKnownPropertyValueRef::BackgroundAttachment(value) => assert_eq!(
                value.attachments().attachments(),
                &[
                    CssBackgroundAttachment::Fixed,
                    CssBackgroundAttachment::Local
                ]
            ),
            _ => panic!("selected background longhand payload"),
        }
    }
    let symbolic = declaration("background-size:contain, calc(10px + 5%) auto");
    let CssKnownPropertyValueRef::BackgroundSize(value) =
        symbolic.known().unwrap().property_value().unwrap()
    else {
        panic!("symbolic size")
    };
    let [
        CssBackgroundSize::Contain,
        CssBackgroundSize::Explicit {
            width: CssBackgroundSizeComponent::Length(width),
            height: Some(CssBackgroundSizeComponent::Auto),
        },
    ] = value.sizes().sizes()
    else {
        panic!("contain and one symbolic explicit size")
    };
    assert!(width.calculation().is_some());
    assert!(width.literal_component().is_none());
}

#[test]
fn invalid_lists_drop_only_the_declaration_with_structured_property_diagnostics() {
    for case in CASES {
        for invalid in case.invalid {
            let css = format!(
                "{}:{invalid}; background-image:none",
                case.property.canonical_name()
            );
            let report = parse_style_attribute(&css);
            let [diagnostic] = report.diagnostics() else {
                panic!("one invalid-list diagnostic")
            };
            assert_eq!(
                diagnostic.error().code(),
                CssErrorCode::InvalidPropertyValue
            );
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
                panic!("structured property diagnostic")
            };
            assert_eq!(detail.property(), case.property);
            let [sibling] = report.syntax().as_slice() else {
                panic!("only valid sibling retained")
            };
            assert_eq!(
                sibling.known().unwrap().property(),
                CssKnownProperty::BackgroundImage
            );
            assert_eq!(
                validate_style_attribute(&css).unwrap_err().diagnostics(),
                report.diagnostics()
            );
        }
    }
}
