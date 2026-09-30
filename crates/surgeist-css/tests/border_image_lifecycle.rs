#![forbid(unsafe_code)]

//! Backgrounds 3 CRD 2024-03-11 §§5.1–5.5, 5.7: typed authored
//! components, five-member shorthand, and omitted values reset to initials.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#border-image
//! Occurrence identity, strict immutable reentry and normalization budgets are
//! Surgeist public contracts exercised through existing public APIs.

use surgeist_css::*;

const MEMBERS: [CssKnownProperty; 5] = [
    CssKnownProperty::BorderImageSource,
    CssKnownProperty::BorderImageSlice,
    CssKnownProperty::BorderImageWidth,
    CssKnownProperty::BorderImageOutset,
    CssKnownProperty::BorderImageRepeat,
];

fn authored(value: &str) -> CssDeclaration {
    let css = format!("border-image:{value}!important");
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    report.syntax()[0].clone()
}

fn checked(value: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::BorderImage),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap()
}

fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let result = expand_declaration(source).expect("border-image intrinsic expansion");
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = result else {
        panic!("ordinary/global border-image produces terminal longhands");
    };
    assert_members(&values, source, None);
    values
}

fn assert_members(
    values: &CssLonghandContributions,
    source: &CssDeclaration,
    replacement: Option<&CssComponentValues>,
) {
    assert_eq!(
        values
            .items()
            .iter()
            .map(|item| item.property())
            .collect::<Vec<_>>(),
        MEMBERS
    );
    for item in values.items() {
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), source.importance());
        match (item.replacement_components(), replacement) {
            (None, None) => {}
            (Some(actual), Some(expected)) => {
                assert_eq!(
                    actual.serialize().unwrap().as_css(),
                    expected.serialize().unwrap().as_css()
                );
                assert_eq!(actual.items().len(), expected.items().len());
                for (actual, expected) in actual.items().iter().zip(expected.items()) {
                    assert_eq!(actual.origin(), expected.origin());
                }
            }
            _ => panic!("replacement provenance does not match supplied components"),
        }
    }
}

fn value(values: &CssLonghandContributions, index: usize) -> CssLonghandValueRef<'_> {
    let CssContributionValueRef::Ordinary(value) = values.items()[index].value() else {
        panic!("ordinary typed member");
    };
    value
}

fn assert_slice(slice: &CssBorderImageSlice, texts: [&str; 4], fill: bool) {
    assert_eq!(slice.fill(), fill);
    for (edge, text) in slice.values().iter().zip(texts) {
        if let Some(percent) = text.strip_suffix('%') {
            let CssBorderImageSliceComponent::Percentage(number) = edge else {
                panic!("percentage slice");
            };
            assert_eq!(number.serialize_specified().unwrap(), format!("{percent}%"));
        } else {
            let CssBorderImageSliceComponent::Number(number) = edge else {
                panic!("number slice");
            };
            assert_eq!(number.serialize_specified().unwrap(), text);
        }
    }
}

fn assert_width(width: &CssBorderImageWidth, texts: [&str; 4]) {
    for (edge, text) in width.values().iter().zip(texts) {
        match (edge, text) {
            (CssBorderImageWidthComponent::Auto, "auto") => {}
            (CssBorderImageWidthComponent::LengthPercentage(number), text)
                if text.ends_with('%') || text.ends_with("px") =>
            {
                assert_eq!(number.serialize_specified().unwrap(), text);
            }
            (CssBorderImageWidthComponent::Number(number), text)
                if !text.ends_with('%') && !text.ends_with("px") && text != "auto" =>
            {
                assert_eq!(number.serialize_specified().unwrap(), text);
            }
            _ => panic!("width branch for {text}: {edge:?}"),
        }
    }
}

fn assert_outset(outset: &CssBorderImageOutset, texts: [&str; 4]) {
    for (edge, text) in outset.values().iter().zip(texts) {
        match edge {
            CssBorderImageOutsetComponent::Length(number) if text.ends_with("px") => {
                assert_eq!(number.serialize_specified().unwrap(), text);
            }
            CssBorderImageOutsetComponent::Number(number) if !text.ends_with("px") => {
                assert_eq!(number.serialize_specified().unwrap(), text);
            }
            _ => panic!("outset branch for {text}: {edge:?}"),
        }
    }
}

fn assert_initial_members(values: &CssLonghandContributions) {
    assert!(matches!(
        value(values, 0),
        CssLonghandValueRef::BorderImageSource(CssImageValue::None)
    ));
    let CssLonghandValueRef::BorderImageSlice(slice) = value(values, 1) else {
        panic!("slice");
    };
    assert_slice(slice, ["100%"; 4], false);
    for edge in slice.values() {
        let CssBorderImageSliceComponent::Percentage(number) = edge else {
            panic!("percentage initial");
        };
        assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
    }
    let CssLonghandValueRef::BorderImageWidth(width) = value(values, 2) else {
        panic!("width");
    };
    assert_width(width, ["1"; 4]);
    for edge in width.values() {
        let CssBorderImageWidthComponent::Number(number) = edge else {
            panic!("number initial");
        };
        assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
    }
    let CssLonghandValueRef::BorderImageOutset(outset) = value(values, 3) else {
        panic!("outset");
    };
    assert_outset(outset, ["0"; 4]);
    for edge in outset.values() {
        let CssBorderImageOutsetComponent::Number(number) = edge else {
            panic!("number initial");
        };
        assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
    }
    let CssLonghandValueRef::BorderImageRepeat(repeat) = value(values, 4) else {
        panic!("repeat");
    };
    assert_eq!(repeat.horizontal(), CssBorderImageRepeatKeyword::Stretch);
    assert_eq!(repeat.vertical(), CssBorderImageRepeatKeyword::Stretch);
}

#[test]
fn border_image_grammar_controls_preserve_typed_arity_and_reject_invalid_construction() {
    for (css, expected) in [
        ("10", ["10", "10", "10", "10"]),
        ("10 20%", ["10", "20%", "10", "20%"]),
        ("10 20% 30", ["10", "20%", "30", "20%"]),
        ("10 20% 30 40%", ["10", "20%", "30", "40%"]),
    ] {
        for declaration in [authored(css), checked(css)] {
            let Some(CssKnownPropertyValueRef::BorderImage(wrapper)) =
                declaration.known().unwrap().property_value()
            else {
                panic!("typed shorthand");
            };
            let image = wrapper.border_image();
            assert_slice(image.slice().unwrap(), expected, false);
            assert!(image.source().is_none());
            assert!(image.width().is_none());
            assert!(image.outset().is_none());
            assert!(image.repeat().is_none());
        }
    }
    let number = CssSpecifiedNonNegativeNumber::try_from_component(
        CssComponentValue::try_number("1").unwrap(),
    )
    .unwrap();
    for count in [0, 5] {
        assert!(
            CssBorderImageSlice::try_new(
                vec![CssBorderImageSliceComponent::Number(number.clone()); count],
                false
            )
            .is_none()
        );
        assert!(
            CssBorderImageWidth::try_new(vec![
                CssBorderImageWidthComponent::Number(number.clone());
                count
            ])
            .is_none()
        );
        assert!(
            CssBorderImageOutset::try_new(vec![
                CssBorderImageOutsetComponent::Number(
                    number.clone()
                );
                count
            ])
            .is_none()
        );
    }
    let width =
        CssBorderImageWidth::try_new(vec![CssBorderImageWidthComponent::Number(number.clone())])
            .unwrap();
    let outset =
        CssBorderImageOutset::try_new(vec![CssBorderImageOutsetComponent::Number(number)]).unwrap();
    assert!(CssBorderImage::try_new(None, None, None, None, None).is_none());
    assert!(CssBorderImage::try_new(None, None, Some(width), None, None).is_none());
    assert!(CssBorderImage::try_new(None, None, None, Some(outset), None).is_none());
}

#[test]
fn border_image_metadata_sets_five_members_and_border_keeps_five_reset_only_members() {
    let metadata = CssKnownProperty::BorderImage
        .metadata()
        .expect("intrinsic border-image metadata");
    let CssPropertyKindRef::Shorthand(shorthand) = metadata.kind() else {
        panic!("shorthand metadata");
    };
    for actual in [shorthand.members(), shorthand.settable_members()] {
        assert_eq!(
            actual
                .iter()
                .map(|member| member.known_property())
                .collect::<Vec<_>>(),
            MEMBERS
        );
    }
    assert!(shorthand.reset_only_members().is_empty());
    assert!(!shorthand.is_legacy());
    for member in MEMBERS {
        let metadata = member.metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("terminal metadata");
        };
        assert!(!longhand.inherited_by_default());
        assert_eq!(longhand.initial_value().property().known_property(), member);
    }
    let metadata = CssKnownProperty::Border.metadata().unwrap();
    let CssPropertyKindRef::Shorthand(border) = metadata.kind() else {
        panic!("border shorthand");
    };
    assert_eq!(
        border
            .reset_only_members()
            .iter()
            .map(|member| member.known_property())
            .collect::<Vec<_>>(),
        MEMBERS
    );
}

#[test]
fn border_image_none_supplies_all_five_intrinsic_initials() {
    for source in [authored("none"), checked("none")] {
        assert_initial_members(&completed(&source));
    }
}

#[test]
fn border_image_explicit_components_expand_in_schema_order_without_resolving_domains() {
    for css in [
        "url(frame.png) 10% 20 30% 40 fill / auto 2 25% 4px / 0 2px 3 4px round space",
        "round space 10% 20 30% 40 fill / auto 2 25% 4px / 0 2px 3 4px url(frame.png)",
    ] {
        for source in [authored(css), checked(css)] {
            let values = completed(&source);
            assert!(
                matches!(value(&values, 0), CssLonghandValueRef::BorderImageSource(CssImageValue::Url(url)) if url.as_str() == "frame.png")
            );
            let CssLonghandValueRef::BorderImageSlice(slice) = value(&values, 1) else {
                panic!("slice");
            };
            assert_slice(slice, ["10%", "20", "30%", "40"], true);
            let CssLonghandValueRef::BorderImageWidth(width) = value(&values, 2) else {
                panic!("width");
            };
            assert_width(width, ["auto", "2", "25%", "4px"]);
            let CssLonghandValueRef::BorderImageOutset(outset) = value(&values, 3) else {
                panic!("outset");
            };
            assert_outset(outset, ["0", "2px", "3", "4px"]);
            let CssLonghandValueRef::BorderImageRepeat(repeat) = value(&values, 4) else {
                panic!("repeat");
            };
            assert_eq!(repeat.horizontal(), CssBorderImageRepeatKeyword::Round);
            assert_eq!(repeat.vertical(), CssBorderImageRepeatKeyword::Space);
        }
    }
}

#[test]
fn border_image_double_slash_omits_width_and_preserves_slice_fill_and_outset() {
    let source = authored("fill 10 20% // 2px repeat");
    let values = completed(&source);
    assert!(matches!(
        value(&values, 0),
        CssLonghandValueRef::BorderImageSource(CssImageValue::None)
    ));
    let CssLonghandValueRef::BorderImageSlice(slice) = value(&values, 1) else {
        panic!("slice");
    };
    assert_slice(slice, ["10", "20%", "10", "20%"], true);
    let CssLonghandValueRef::BorderImageWidth(width) = value(&values, 2) else {
        panic!("width");
    };
    assert_width(width, ["1"; 4]);
    let CssLonghandValueRef::BorderImageOutset(outset) = value(&values, 3) else {
        panic!("outset");
    };
    assert_outset(outset, ["2px"; 4]);
    let CssLonghandValueRef::BorderImageRepeat(repeat) = value(&values, 4) else {
        panic!("repeat");
    };
    assert_eq!(repeat.horizontal(), CssBorderImageRepeatKeyword::Repeat);
    assert_eq!(repeat.vertical(), CssBorderImageRepeatKeyword::Repeat);
}

#[test]
fn border_image_expansion_preserves_checked_symbolic_numeric_roots() {
    let source = checked("calc(-1) fill / calc(1 + 1) / calc(2px + 3px)");
    let values = completed(&source);
    let CssLonghandValueRef::BorderImageSlice(slice) = value(&values, 1) else {
        panic!("slice");
    };
    assert!(slice.fill());
    for edge in slice.values() {
        let CssBorderImageSliceComponent::Number(number) = edge else {
            panic!("number math");
        };
        assert!(number.literal_component().is_none());
        assert!(number.calculation().is_some(), "deferred range evaluation");
    }
    let CssLonghandValueRef::BorderImageWidth(width) = value(&values, 2) else {
        panic!("width");
    };
    for edge in width.values() {
        let CssBorderImageWidthComponent::Number(number) = edge else {
            panic!("number multiplier math");
        };
        assert!(number.literal_component().is_none());
        assert!(number.calculation().is_some());
    }
    let CssLonghandValueRef::BorderImageOutset(outset) = value(&values, 3) else {
        panic!("outset");
    };
    for edge in outset.values() {
        let CssBorderImageOutsetComponent::Length(length) = edge else {
            panic!("length math");
        };
        assert!(length.literal_component().is_none());
        assert!(length.calculation().is_some());
    }
}

#[test]
fn border_image_css_wide_keywords_propagate_to_exactly_five_members() {
    for (css, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        for source in [authored(css), checked(css)] {
            for item in completed(&source).items() {
                assert!(
                    matches!(item.value(), CssContributionValueRef::Global(actual) if actual == keyword)
                );
            }
        }
    }
}

#[test]
fn border_image_pending_reentry_rejects_atomically_and_retries_with_mixed_origins() {
    for source in [authored("var(--frame)"), checked("var(--frame)")] {
        let CssExpansion::Pending(pending) =
            expand_declaration(&source).expect("supported pending shorthand")
        else {
            panic!("pending shorthand");
        };
        let before = source
            .value_components()
            .serialize()
            .unwrap()
            .as_css()
            .to_owned();
        for _ in 0..2 {
            for invalid in ["10 /", "10 //", "10 / 2 /", "none none", "10 / -1"] {
                let components = parse_component_values(invalid).unwrap();
                let direct = parse_property_value(
                    CssPropertyNameRef::Known(CssKnownProperty::BorderImage),
                    components.clone(),
                    CssImportance::Normal,
                )
                .unwrap_err();
                let error = pending.reenter(components).unwrap_err();
                let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
                    panic!("strict grammar failure: {error:?}");
                };
                assert_eq!(actual.kind(), direct.kind());
                assert_eq!(actual.origin(), direct.origin());
            }
            for residual in [
                "var(--x)",
                "f(env(x))",
                "[f(attr(data-x))]",
                "f(v\\61r(--x))",
            ] {
                let error = pending
                    .reenter(parse_component_values(residual).unwrap())
                    .unwrap_err();
                assert!(matches!(
                    error.kind(),
                    CssExpansionErrorKind::ResidualSubstitution
                ));
            }
            assert!(pending.source().same_occurrence(&source));
            assert_eq!(
                pending
                    .source()
                    .value_components()
                    .serialize()
                    .unwrap()
                    .as_css(),
                before
            );
        }
        let parsed = parse_component_values("10 fill / 2 / 1 round").unwrap();
        let mut items = parsed.items().to_vec();
        let width_index = items.iter().position(|item| matches!(item.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.representation() == "2")).unwrap();
        items[width_index] = CssComponentValue::try_number("2").unwrap();
        let replacement = CssComponentValues::try_new(items).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed reentry");
            };
            assert_members(&values, &source, Some(&replacement));
            let CssLonghandValueRef::BorderImageSlice(slice) = value(&values, 1) else {
                panic!("slice");
            };
            assert_slice(slice, ["10"; 4], true);
            for edge in slice.values() {
                let CssBorderImageSliceComponent::Number(number) = edge else {
                    panic!("number slice");
                };
                assert!(matches!(number.origin(), CssValueOrigin::Parsed(_)));
            }
            let CssLonghandValueRef::BorderImageWidth(width) = value(&values, 2) else {
                panic!("width");
            };
            assert_width(width, ["2"; 4]);
            for edge in width.values() {
                let CssBorderImageWidthComponent::Number(number) = edge else {
                    panic!("number width");
                };
                assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
            }
        }
        let replacement = parse_component_values("revert-layer").unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("global reentry");
        };
        assert_members(&values, &source, Some(&replacement));
        assert!(values.items().iter().all(|item| matches!(
            item.value(),
            CssContributionValueRef::Global(CssGlobalKeyword::RevertLayer)
        )));
        assert_eq!(
            pending
                .source()
                .value_components()
                .serialize()
                .unwrap()
                .as_css(),
            before
        );
    }
}

#[test]
fn border_image_normalization_counts_five_completed_members_and_one_pending_occurrence() {
    for (image, members, pending) in [("none", 6, false), ("var(--frame)", 2, true)] {
        let css = format!(".frame {{ border-image:{image}!important; color:red }}");
        let report = parse_sheet(&css);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let CssRule::Style(style) = &report.syntax().rules()[0] else {
            panic!("style rule");
        };
        let sheet = normalize_sheet_with_limits(
            report.syntax(),
            CssNormalizationLimits::try_new(0, 1, 2, members).unwrap(),
        )
        .expect("exact normalization limits");
        let [
            CssNormalizedItem::Rule(rule),
            CssNormalizedItem::Declaration(first),
            CssNormalizedItem::Declaration(second),
        ] = sheet.items()
        else {
            panic!("one rule, two declaration ordinals");
        };
        assert_eq!([first.order(), second.order()], [0, 1]);
        assert!(first.rule_context().same_context(rule));
        assert!(second.rule_context().same_context(rule));
        assert!(first.source().same_occurrence(&style.declarations()[0]));
        assert!(second.source().same_occurrence(&style.declarations()[1]));
        assert_eq!(first.source().importance(), CssImportance::Important);
        if pending {
            let CssExpansion::Pending(value) = first.expansion() else {
                panic!("pending group");
            };
            assert!(value.source().same_occurrence(&style.declarations()[0]));
        } else {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                first.expansion()
            else {
                panic!("completed group");
            };
            assert_members(values, &style.declarations()[0], None);
            assert_initial_members(values);
        }
        let CssExpansion::Contributions(CssContributions::Longhands(sibling)) = second.expansion()
        else {
            panic!("ordinary sibling");
        };
        assert_eq!(sibling.items()[0].property(), CssKnownProperty::Color);
        for (limit, ordinal) in [(members - 1, 1), (if pending { 0 } else { 4 }, 0)] {
            let error = normalize_sheet_with_limits(
                report.syntax(),
                CssNormalizationLimits::try_new(0, 1, 2, limit).unwrap(),
            )
            .unwrap_err();
            assert!(
                matches!(error.kind(), CssNormalizationErrorKind::LimitExceeded { resource: CssNormalizationResource::Contributions, limit: actual } if *actual == limit)
            );
            assert_eq!(error.declaration_order(), Some(ordinal));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(&style.declarations()[ordinal])
            );
            assert!(error.rule_context().is_some());
        }
    }
}

#[test]
fn border_image_invalid_slash_and_duplicate_controls_drop_only_the_responsible_declaration() {
    // These coordinates are independently retained in the existing
    // border_image_grammars.rs public regression at the stated source basis.
    for (property, invalid, responsible) in [
        ("border-image", "url(frame.png) / 2", "/"),
        ("border-image", "10 // 1 2 3 4 5", "5"),
        ("border-image-slice", "1 fill fill", "fill"),
    ] {
        let declaration = format!("{property}: {invalid};");
        let css = format!("--😀: kept; {declaration} color: red");
        let report = parse_style_attribute(&css);
        let [diagnostic] = report.diagnostics() else {
            panic!("one diagnostic");
        };
        assert_eq!(report.syntax().len(), 2);
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let start = css.find(&declaration).unwrap();
        let responsible_index = if responsible == "fill" {
            invalid.rfind(responsible).unwrap()
        } else {
            invalid.find(responsible).unwrap()
        };
        let offset = start + property.len() + 2 + responsible_index;
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + declaration.len()
        );
        assert_eq!(diagnostic.error().position().byte_offset().value(), offset);
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            css[..offset].encode_utf16().count()
        );
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
    for invalid in ["none none", "round round round", "10 fill fill"] {
        let css = format!("border-image:{invalid};color:red");
        let report = parse_style_attribute(&css);
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one duplicate diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::BorderImage),
                parse_component_values(invalid).unwrap(),
                CssImportance::Normal
            )
            .is_err()
        );
    }
}
