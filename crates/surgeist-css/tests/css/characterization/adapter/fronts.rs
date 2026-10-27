//! Parsing fronts: one arm per stable front id.

use std::sync::Arc;

use surgeist_css::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueLimits, CssComponentValues,
};
use surgeist_css::conformance::{
    CssExclusionMetadata, CssExclusionReason, CssFeatureKind, CssFeatureMetadata,
    CssSpecificationSource, CssSpecificationTier, CssSupportStatus, conformance_exclusion,
    conformance_exclusions, feature_catalog, feature_metadata, property_support_metadata,
    specification_source, specification_sources,
};
use surgeist_css::declaration::{
    CssCustomPropertyName, CssDeclaration, CssDeclarationList, CssGlobalKeyword, CssImportance,
    CssSpecifiedDeclarationBlock,
};
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizedItem, CssRuleContext, CssRuleContextKindRef,
};
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssKnownProperty, CssPropertyGrammar,
    CssPropertyNameRef, CssPropertyTarget, CssPropertyTextTarget, CssPropertyValueError,
    CssSvgGlyphOrientationVerticalDeclaration,
};
use surgeist_css::query::media::{CssMediaQuery, CssMediaQueryList};
use surgeist_css::query::supports::{
    CssSupportsDeclaration, CssSupportsLeaf, CssSupportsTestBody, CssSupportsTestChild,
};
use surgeist_css::query::{CssCondition, CssWhenLeaf};
use surgeist_css::rule::color_profile::{
    CssColorProfileDescriptor, CssColorProfileDescriptorKind, CssColorProfileDescriptorValue,
};
use surgeist_css::rule::counter_style::{
    CssCounterStyleDescriptorKind, CssCounterStyleDescriptorValue, CssCounterStyleDescriptors,
};
use surgeist_css::rule::font_face::{
    CssAuthoredFontFaceDescriptorValue, CssFontFaceDescriptorKind, CssFontFaceDescriptors,
    CssFontFaceFamily, serialize_font_face_family_list,
};
use surgeist_css::rule::font_feature_values::{
    CssFontFeatureDisplayValue, CssFontFeatureValue, CssFontFeatureValueBlock,
    CssFontFeatureValueKind, CssFontFeatureValuesFamilyList, CssFontFeatureValuesItem,
};
use surgeist_css::rule::font_palette_values::{
    CssFontPaletteDescriptor, CssFontPaletteDescriptorKind, CssFontPaletteDescriptorValue,
};
use surgeist_css::rule::keyframes::{
    CssKeyframeBlock, CssKeyframeDeclarationList, CssParsedKeyframeRule, CssParsedKeyframeSelectors,
};
use surgeist_css::rule::page::{
    CssMarginDeclarationBlock, CssMarginRule, CssPageBody, CssPageDeclarationBlock,
    CssPageDescriptorKind, CssPageDescriptorValue, CssPageSelectorList,
};
use surgeist_css::rule::{
    CssBlockContext, CssBlockFragment, CssNamespaceName, CssNamespacePrefix, CssRule, CssRuleList,
    CssRuleSyntax, CssSheet, CssStyleAncestor, CssStyleBlock,
};
use surgeist_css::selector::{
    CssNamespaceContext, CssParsedStyleSelectors, CssRelativeSelectorList, CssSelector,
    CssStyleSelectorContext,
};
use surgeist_css::serialization::{
    CssRuleGraphInput, CssRuleSerializationLimits, CssSerializationError, CssSerializationLimits,
    serialize_css_comma_separated_list, serialize_css_identifier, serialize_css_string,
    serialize_css_whitespace_separated_list,
};
use surgeist_css::source::{
    CssDecodedStylesheetInput, CssStylesheetDecodeHints, CssStylesheetLocation, CssUtf16InputError,
    CssUtf16InputLimits, CssUtf16PropertyValueLimits,
};
use surgeist_css::values::color_adjust::CssColorScheme;
use surgeist_css::{
    CssParseReport, CssParser, CssParserContext, CssParserMode, CssValidationFailure,
};

use super::report::{Retained, property_id};
use super::{Limited, Observation, Stop, nonempty, parse_charge, serialization, utf16_charge};

/// A parser mode prefix: `quirks` or `standards`, alone or as `mode:rest`.
/// The context methods run only for a named mode.
fn mode(argument: &str) -> (Option<CssParserContext>, &str) {
    for (name, mode) in [
        ("quirks", CssParserMode::Quirks),
        ("standards", CssParserMode::Standards),
    ] {
        // `new` and `with_mode` must agree on the selected mode.
        let context = CssParserContext::new(mode);
        assert!(
            context.mode() == CssParserContext::default().with_mode(mode).mode(),
            "parser context constructors disagree on {name}"
        );
        if argument == name {
            return (Some(context), "-");
        }
        if let Some(rest) = argument
            .strip_prefix(name)
            .and_then(|rest| rest.strip_prefix(':'))
        {
            return (Some(context), rest);
        }
    }
    (None, argument)
}

pub fn observe(front: &str, argument: &str, input: &str) -> Observation {
    let (context, argument) = mode(argument);
    let utf16_context = context.unwrap_or_default();
    match front {
        "sheet" => {
            let report =
                CssParser::new(context.unwrap_or_default(), Default::default()).parse_sheet(input);
            let mut observation = sheet(&report);
            retained_rules(&mut observation, report.syntax().rules());
            observation
        }
        "sheet.with-options" => {
            let location =
                (argument != "-").then(|| CssStylesheetLocation::new(argument.to_owned()));
            let report = CssParser::default()
                .parse_sheet(input)
                .with_location(location);
            let mut observation = sheet(&report);
            retained_rules(&mut observation, report.syntax().rules());
            observation
        }
        "sheet.bytes" => sheet_bytes(argument, input),
        "sheet.utf16" => {
            let mut observation = sheet_utf16(utf16_context, input, CssUtf16InputLimits::default());
            utf16_charge(&mut observation, |limits| {
                sheet_utf16(utf16_context, input, limits)
            });
            observation
        }
        "sheet.validate" => validation(
            CssParser::default()
                .parse_sheet(input)
                .into_validation_result(),
            sheet_serializations,
        ),
        "sheet.normalize" => normalized_sheet(&CssParser::default().parse_sheet(input), argument),
        "style" => {
            let report = CssParser::new(context.unwrap_or_default(), Default::default())
                .parse_block_contents(input);
            let mut observation = declaration_list(&report);
            let retained = Retained::declarations(report.syntax().as_slice());
            observation.retained = nonempty(retained.identities.join("~"));
            observation.authored = nonempty(retained.authored.join("~"));
            observation
        }
        "style.validate" => validation(
            CssParser::default()
                .parse_block_contents(input)
                .into_validation_result(),
            list_serializations,
        ),
        "declaration.expand" => {
            let report = CssParser::default().parse_block_contents(input);
            let mut observation = Observation::shaped("report");
            observation.report(&report);
            observation.add_fact(
                report
                    .syntax()
                    .iter()
                    .map(|declaration| match declaration.expand() {
                        Ok(expansion) => expansion_facts(&expansion),
                        Err(error) => format!("err:{}", super::error_text(&error)),
                    })
                    .collect::<Vec<_>>()
                    .join(" ; "),
            );
            observation
        }
        "block-contents" => with_parse_charge(
            declaration_list(
                &CssParser::new(context.unwrap_or_default(), Default::default())
                    .parse_block_contents(input),
            ),
            |limits| {
                declaration_list(
                    &CssParser::new(context.unwrap_or_default(), limits)
                        .parse_block_contents(input),
                )
            },
        ),
        "block-contents.utf16" => {
            let mut observation = block_contents_utf16(
                utf16_context,
                input,
                CssUtf16InputLimits::default(),
                CssComponentValueLimits::default(),
            );
            utf16_charge(&mut observation, |limits| {
                block_contents_utf16(
                    utf16_context,
                    input,
                    limits,
                    CssComponentValueLimits::default(),
                )
            });
            parse_charge(&mut observation, |limits| {
                block_contents_utf16(utf16_context, input, CssUtf16InputLimits::default(), limits)
            });
            observation
        }
        "block-contents.page" => with_parse_charge(
            optional(
                &CssParser::new(context.unwrap_or_default(), Default::default())
                    .parse_page_block_contents(input),
                page_declarations,
            ),
            |limits| {
                optional(
                    &CssParser::new(context.unwrap_or_default(), limits)
                        .parse_page_block_contents(input),
                    page_declarations,
                )
            },
        ),
        "block-contents.font-face" => with_parse_charge(
            optional(
                &CssParser::default().parse_font_face_block_contents(input),
                font_face_descriptors,
            ),
            |limits| {
                optional(
                    &CssParser::new(CssParserContext::default(), limits)
                        .parse_font_face_block_contents(input),
                    font_face_descriptors,
                )
            },
        ),
        "declaration-list" => declaration_list(
            &CssParser::new(context.unwrap_or_default(), Default::default())
                .parse_block_contents(input),
        ),
        "declaration-list.validate" => validation(
            CssParser::default()
                .parse_block_contents(input)
                .into_validation_result(),
            list_serializations,
        ),
        "property-value.text" => property_value_text(context, argument, input),
        "property-value.components" => {
            property_value_components(context, argument, input, "ordinary")
        }
        "property-value.page" => property_value_components(context, argument, input, "page"),
        "property-value.margin" => property_value_components(context, argument, input, "margin"),
        "property-value.grammar.text" => {
            let grammar = grammar(argument);
            optional(
                &CssParser::new(context.unwrap_or_default(), Default::default())
                    .parse_property_value(
                        input,
                        CssPropertyTextTarget::Grammar(grammar),
                        CssImportance::Normal,
                    ),
                declaration,
            )
        }
        "property-value.grammar.components" => {
            let grammar = grammar(argument);
            components_then(input, |values| {
                checked_declaration(CssDeclaration::try_from_components(
                    &CssParser::new(context.unwrap_or_default(), Default::default()),
                    CssPropertyTarget::Grammar(grammar),
                    values,
                    CssImportance::Normal,
                ))
            })
        }
        "svg-glyph-orientation-vertical.text" => optional(
            &CssSvgGlyphOrientationVerticalDeclaration::parse_attribute(
                &CssParser::new(svg_context(context), Default::default()),
                input,
            ),
            declaration,
        ),
        "svg-glyph-orientation-vertical.components" => components_then(input, |values| {
            checked_declaration(
                CssSvgGlyphOrientationVerticalDeclaration::try_attribute_from_components(
                    &CssParser::new(svg_context(context), Default::default()),
                    values,
                ),
            )
        }),
        "declaration" => optional(
            &CssDeclaration::parse(
                &CssParser::new(context.unwrap_or_default(), Default::default()),
                input,
            ),
            declaration,
        ),
        "rule" => {
            let namespaces = namespaces(argument);
            optional(
                &CssRule::parse(
                    &CssParser::new(context.unwrap_or_default(), Default::default()),
                    input,
                    &namespaces,
                ),
                |observation, rule| {
                    rule_serializations(observation, std::slice::from_ref(rule));
                    retained_rules(observation, std::slice::from_ref(rule));
                },
            )
        }
        "rule-graph" => rule_graph(input),
        "edited-rule" => edited_rule(input),
        "rule-syntax" => with_parse_charge(
            optional(
                &CssRuleSyntax::classify(&CssParser::default(), input),
                rule_syntax,
            ),
            |limits| {
                optional(
                    &CssRuleSyntax::classify(
                        &CssParser::new(CssParserContext::default(), limits),
                        input,
                    ),
                    rule_syntax,
                )
            },
        ),
        "selector" => optional(
            &CssSelector::parse(&CssParser::default(), input, &namespaces(argument)),
            |observation, selector| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    selector.serialize_specified(limits)
                });
            },
        ),
        "selector-list" => {
            let namespaces = namespaces(argument);
            optional(
                &CssParsedStyleSelectors::parse(
                    &CssParser::default(),
                    input,
                    &namespaces,
                    CssStyleSelectorContext::Ordinary,
                ),
                |observation, selectors| {
                    observation.add_fact(format!(
                        "selectors={}",
                        selectors.selectors().selectors().len()
                    ));
                    serialization(observation, "cssom", Limited::Values, |limits| {
                        selectors.selectors().serialize_cssom(
                            &namespaces,
                            CssStyleSelectorContext::Ordinary,
                            limits,
                        )
                    });
                },
            )
        }
        "style-selector-list" => {
            let namespaces = CssNamespaceContext::default();
            let context = style_selector_context(argument);
            let render = |report: &CssParseReport<Option<CssParsedStyleSelectors>>| {
                optional(report, |observation, selectors| {
                    serialization(observation, "cssom", Limited::Values, |limits| {
                        selectors.serialize_cssom(&namespaces, limits)
                    });
                })
            };
            with_parse_charge(
                render(&CssParsedStyleSelectors::parse(
                    &CssParser::default(),
                    input,
                    &namespaces,
                    context,
                )),
                |limits| {
                    render(&CssParsedStyleSelectors::parse(
                        &CssParser::new(CssParserContext::default(), limits),
                        input,
                        &namespaces,
                        context,
                    ))
                },
            )
        }
        "relative-selector-list" => {
            let namespaces = CssNamespaceContext::default();
            optional(
                &CssRelativeSelectorList::parse(&CssParser::default(), input, &namespaces),
                |observation, selectors| {
                    serialization(observation, "specified", Limited::Values, |limits| {
                        selectors.serialize_specified(limits)
                    });
                    serialization(observation, "cssom", Limited::Values, |limits| {
                        selectors.serialize_cssom(&namespaces, limits)
                    });
                },
            )
        }
        "media-query" => {
            let report = CssMediaQuery::parse(&CssParser::default(), input);
            let mut observation = Observation::shaped("report");
            observation.report(&report);
            media_query(&mut observation, report.syntax());
            observation
        }
        "media-query.cssom" => optional(
            &CssMediaQuery::parse_medium(&CssParser::default(), input),
            media_query,
        ),
        "media-query-list" => {
            let report = CssMediaQueryList::parse(
                &CssParser::new(context.unwrap_or_default(), Default::default()),
                input,
            );
            let mut observation = Observation::shaped("report");
            observation.report(&report);
            media_query_list(&mut observation, report.syntax());
            observation
        }
        "media-query-list.utf16" => {
            let mut observation =
                media_query_list_utf16(utf16_context, input, CssUtf16InputLimits::default());
            utf16_charge(&mut observation, |limits| {
                media_query_list_utf16(utf16_context, input, limits)
            });
            observation
        }
        "supports-condition" => {
            let namespaces = namespaces(argument);
            with_parse_charge(
                checked(
                    CssCondition::<CssSupportsLeaf>::parse(
                        &CssParser::new(utf16_context, Default::default()),
                        input,
                        &namespaces,
                    ),
                    supports_condition,
                ),
                |limits| {
                    checked(
                        CssCondition::<CssSupportsLeaf>::parse(
                            &CssParser::new(utf16_context, limits),
                            input,
                            &namespaces,
                        ),
                        supports_condition,
                    )
                },
            )
        }
        "supports.css-text" => checked(
            CssCondition::<CssSupportsLeaf>::parse_condition_text(
                &CssParser::new(context.unwrap_or_default(), Default::default()),
                input,
            ),
            supports_condition,
        ),
        "supports.declaration" => checked(
            CssSupportsDeclaration::parse(
                &CssParser::new(context.unwrap_or_default(), Default::default()),
                argument,
                input,
            ),
            |observation, declaration| {
                serialization(observation, "specified", Limited::Bytes, |limits| {
                    declaration
                        .serialize_tokens(crate::css::support::byte_limits(limits.max_css_bytes()))
                        .map(|value| value.as_css().to_owned())
                });
            },
        ),
        "supports.test-child" => with_parse_charge(
            optional(
                &CssSupportsTestChild::parse(&CssParser::default(), input),
                supports_test_child,
            ),
            |limits| {
                optional(
                    &CssSupportsTestChild::parse(
                        &CssParser::new(CssParserContext::default(), limits),
                        input,
                    ),
                    supports_test_child,
                )
            },
        ),
        "supports.test-body" => with_parse_charge(
            optional(
                &CssSupportsTestBody::parse(&CssParser::default(), input),
                supports_test_body,
            ),
            |limits| {
                optional(
                    &CssSupportsTestBody::parse(
                        &CssParser::new(CssParserContext::default(), limits),
                        input,
                    ),
                    supports_test_body,
                )
            },
        ),
        "when-condition" => with_parse_charge(
            checked(
                CssCondition::<CssWhenLeaf>::parse(
                    &CssParser::new(context.unwrap_or_default(), Default::default()),
                    input,
                ),
                when_condition,
            ),
            |limits| {
                checked(
                    CssCondition::<CssWhenLeaf>::parse(
                        &CssParser::new(context.unwrap_or_default(), limits),
                        input,
                    ),
                    when_condition,
                )
            },
        ),
        "keyframe-rule" => optional(
            &CssParsedKeyframeRule::parse(
                &CssParser::new(context.unwrap_or_default(), Default::default()),
                input,
            ),
            |observation, rule| keyframe_block(observation, rule.block()),
        ),
        "keyframe-selectors" => optional(
            &CssParsedKeyframeSelectors::parse(&CssParser::default(), input),
            |observation, selectors| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    selectors.selectors().serialize_key_text(limits)
                });
            },
        ),
        "page-selectors" => optional(
            &CssPageSelectorList::parse(&CssParser::default(), input),
            |observation, selectors| {
                serialization(observation, "cssom", Limited::Values, |limits| {
                    selectors.serialize_cssom(limits)
                });
            },
        ),
        "page-margin-rule" => with_parse_charge(
            optional(
                &CssMarginRule::parse(
                    &CssParser::new(context.unwrap_or_default(), Default::default()),
                    input,
                ),
                margin_rule,
            ),
            |limits| {
                optional(
                    &CssMarginRule::parse(
                        &CssParser::new(context.unwrap_or_default(), limits),
                        input,
                    ),
                    margin_rule,
                )
            },
        ),
        "font-feature-family-list" => with_parse_charge(
            optional(
                &CssFontFeatureValuesFamilyList::parse(
                    &CssParser::new(context.unwrap_or_default(), Default::default()),
                    input,
                ),
                family_list,
            ),
            |limits| {
                optional(
                    &CssFontFeatureValuesFamilyList::parse(
                        &CssParser::new(context.unwrap_or_default(), limits),
                        input,
                    ),
                    family_list,
                )
            },
        ),
        "font-feature-value" => optional(
            &CssFontFeatureValue::parse(&CssParser::default(), input, font_feature_kind(argument)),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.serialize_specified(limits)
                });
            },
        ),
        "font-feature-display-value" => optional(
            &CssFontFeatureDisplayValue::parse(&CssParser::default(), input),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.serialize_specified(limits)
                });
            },
        ),
        "descriptor.color-profile" => optional(
            &CssColorProfileDescriptorValue::parse(
                &CssParser::default(),
                input,
                color_profile_descriptor(argument),
            ),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.serialize_specified(limits)
                });
            },
        ),
        "descriptor.counter-style" => optional(
            &CssCounterStyleDescriptorValue::parse(
                &CssParser::default(),
                input,
                counter_style_descriptor(argument),
            ),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.serialize_specified(limits)
                });
            },
        ),
        "descriptor.font-face" => optional(
            &CssAuthoredFontFaceDescriptorValue::parse(
                &CssParser::default(),
                input,
                font_face_descriptor(argument),
            ),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.serialize_specified(limits)
                });
            },
        ),
        "descriptor.font-palette" => optional(
            &CssFontPaletteDescriptorValue::parse(
                &CssParser::default(),
                input,
                font_palette_descriptor(argument),
            ),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.serialize_specified(limits)
                });
            },
        ),
        "descriptor.page" => optional(
            &CssPageDescriptorValue::parse(&CssParser::default(), input, page_descriptor(argument)),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.serialize_specified(limits)
                });
            },
        ),
        "block.color-profile" => optional(
            &CssBlockFragment::<Vec<CssColorProfileDescriptor>>::parse(
                &CssParser::default(),
                input,
                &CssBlockContext::default(),
            ),
            |observation, block| {
                let descriptors = block.body();
                observation.add_fact(format!("descriptors={}", descriptors.len()));
                serialization(observation, "specified", Limited::Values, |limits| {
                    joined(descriptors, |descriptor| {
                        descriptor.serialize_specified(limits)
                    })
                });
            },
        ),
        "block.counter-style" => optional(
            &CssBlockFragment::<CssCounterStyleDescriptors>::parse(
                &CssParser::default(),
                input,
                &CssBlockContext::default(),
            ),
            |observation, block| {
                observation.add_fact(format!("entries={}", block.body().entries().len()));
            },
        ),
        "block.font-face" => optional(
            &CssBlockFragment::<CssFontFaceDescriptors>::parse(
                &CssParser::default(),
                input,
                &CssBlockContext::default(),
            ),
            |observation, block| font_face_descriptors(observation, block.body()),
        ),
        "block.font-feature-value" => optional(
            &CssBlockFragment::<CssFontFeatureValueBlock>::parse_definition(
                &CssParser::default(),
                input,
                font_feature_kind(argument),
            ),
            |observation, block| {
                observation.add_fact(format!("definitions={}", block.body().definitions().len()));
            },
        ),
        "block.font-feature-values" => optional(
            &CssBlockFragment::<Vec<CssFontFeatureValuesItem>>::parse(
                &CssParser::default(),
                input,
                &CssBlockContext::default(),
            ),
            |observation, block| {
                observation.add_fact(format!("items={}", block.body().len()));
            },
        ),
        "block.font-palette-values" => optional(
            &CssBlockFragment::<Vec<CssFontPaletteDescriptor>>::parse(
                &CssParser::default(),
                input,
                &CssBlockContext::default(),
            ),
            |observation, block| {
                let descriptors = block.body();
                observation.add_fact(format!("descriptors={}", descriptors.len()));
                serialization(observation, "specified", Limited::Values, |limits| {
                    joined(descriptors, |descriptor| {
                        descriptor.value().serialize_specified(limits)
                    })
                });
            },
        ),
        "block.group" => {
            let namespaces = namespaces(argument);
            optional(
                &CssBlockFragment::<CssRuleList>::parse(
                    &CssParser::new(context.unwrap_or_default(), Default::default()),
                    input,
                    &CssBlockContext::group(&namespaces),
                ),
                |observation, block| {
                    rule_serializations(observation, block.body().rules());
                    retained_rules(observation, block.body().rules());
                },
            )
        }
        "block.keyframe-declarations" => optional(
            &CssBlockFragment::<CssKeyframeDeclarationList>::parse(
                &CssParser::new(context.unwrap_or_default(), Default::default()),
                input,
                &CssBlockContext::default(),
            ),
            |observation, block| {
                serialization(observation, "cssom", Limited::Values, |limits| {
                    block.body().serialize_cssom(limits)
                });
            },
        ),
        "block.keyframes" => optional(
            &CssBlockFragment::<Vec<CssKeyframeBlock>>::parse(
                &CssParser::new(context.unwrap_or_default(), Default::default()),
                input,
                &CssBlockContext::default(),
            ),
            |observation, block| {
                observation.add_fact(format!("blocks={}", block.body().len()));
                serialization(observation, "specified", Limited::Values, |limits| {
                    joined(block.body(), |keyframe| {
                        keyframe.selectors().serialize_key_text(limits)
                    })
                });
                serialization(observation, "cssom", Limited::Values, |limits| {
                    joined(block.body(), |keyframe| {
                        keyframe.declarations().serialize_cssom(limits)
                    })
                });
            },
        ),
        "block.margin" => optional(
            &CssBlockFragment::<CssMarginDeclarationBlock>::parse(
                &CssParser::new(context.unwrap_or_default(), Default::default()),
                input,
                &CssBlockContext::default(),
            ),
            |observation, block| {
                serialization(observation, "cssom", Limited::Values, |limits| {
                    block.body().serialize_cssom(limits)
                });
            },
        ),
        "block.page" => optional(
            &CssBlockFragment::<CssPageBody>::parse(
                &CssParser::new(context.unwrap_or_default(), Default::default()),
                input,
                &CssBlockContext::default(),
            ),
            |observation, block| {
                let body = block.body();
                page_declarations(observation, body.declarations());
                observation.add_fact(format!("margin-rules={}", body.margin_rules().len()));
            },
        ),
        "block.scope" => {
            let namespaces = CssNamespaceContext::default();
            optional(
                &CssBlockFragment::<CssRuleList>::parse(
                    &CssParser::new(context.unwrap_or_default(), Default::default()),
                    input,
                    &CssBlockContext::scope(&namespaces, ancestry(argument)),
                ),
                |observation, block| retained_scoped(observation, block.body().rules()),
            )
        }
        "block.scoped-group" => {
            let namespaces = CssNamespaceContext::default();
            optional(
                &CssBlockFragment::<CssRuleList>::parse(
                    &CssParser::new(context.unwrap_or_default(), Default::default()),
                    input,
                    &CssBlockContext::scoped_group(&namespaces, ancestry(argument)),
                ),
                |observation, block| retained_scoped(observation, block.body().rules()),
            )
        }
        "block.supports-test" => optional(
            &CssBlockFragment::<CssSupportsTestBody>::parse(
                &CssParser::default(),
                input,
                &CssBlockContext::default(),
            ),
            |observation, block| supports_test_body(observation, block.body()),
        ),
        "block.style" => {
            let namespaces = CssNamespaceContext::default();
            optional(
                &CssStyleBlock::parse(
                    &CssParser::new(context.unwrap_or_default(), Default::default()),
                    input,
                    &namespaces,
                ),
                |observation, block| {
                    list_serializations(observation, block.declarations());
                    rule_serializations_into(observation, "rules", block.rules());
                    let mut retained = Retained::declarations(block.declarations().as_slice());
                    retained
                        .identities
                        .extend(Retained::rules(block.rules()).identities);
                    observation.retained = nonempty(retained.identities.join("~"));
                },
            )
        }
        "component-value" => with_parse_charge(
            optional(
                &CssComponentValue::parse(&CssParser::default(), input),
                component_value,
            ),
            |limits| {
                optional(
                    &CssComponentValue::parse(
                        &CssParser::new(CssParserContext::default(), limits),
                        input,
                    ),
                    component_value,
                )
            },
        ),
        "component-values" => with_parse_charge(
            checked(
                CssComponentValues::parse(&CssParser::default(), input),
                component_values,
            ),
            |limits| {
                checked(
                    CssComponentValues::parse(
                        &CssParser::new(CssParserContext::default(), limits),
                        input,
                    ),
                    component_values,
                )
            },
        ),
        "component-values.comma" => with_parse_charge(
            comma_values(&CssComponentValues::parse_comma_separated(
                &CssParser::default(),
                input,
            )),
            |limits| {
                comma_values(&CssComponentValues::parse_comma_separated(
                    &CssParser::new(CssParserContext::default(), limits),
                    input,
                ))
            },
        ),
        "color-scheme.utf16" => color_scheme_utf16(utf16_context, input),
        "serialize.identifier" => lexical(|limits| serialize_css_identifier(input, limits)),
        "serialize.string" => lexical(|limits| serialize_css_string(input, limits)),
        "serialize.comma-list" => {
            let items = input.split('|').collect::<Vec<_>>();
            lexical(|limits| serialize_css_comma_separated_list(&items, limits))
        }
        "serialize.whitespace-list" => {
            let items = input.split('|').collect::<Vec<_>>();
            lexical(|limits| serialize_css_whitespace_separated_list(&items, limits))
        }
        "serialize.font-face-family-list" => {
            let families = input
                .split('|')
                .map(CssFontFaceFamily::try_new)
                .collect::<Option<Vec<_>>>();
            match families {
                None => Observation::shaped("family:none"),
                Some(families) => {
                    lexical(|limits| serialize_font_face_family_list(&families, limits))
                }
            }
        }
        "conformance.feature-catalog" => {
            let mut observation = Observation::shaped("catalog");
            let catalog = feature_catalog();
            observation.add_fact(format!("features={}", catalog.len()));
            for feature in catalog {
                observation.add_fact(feature_facts(feature));
            }
            observation
        }
        "conformance.feature" => match feature_metadata(argument) {
            Some(feature) => {
                let mut observation = Observation::shaped("some");
                observation.add_fact(feature_facts(feature));
                observation
            }
            None => Observation::shaped("none"),
        },
        "conformance.specification-sources" => {
            let mut observation = Observation::shaped("catalog");
            let sources = specification_sources();
            observation.add_fact(format!("sources={}", sources.len()));
            for source in sources {
                observation.add_fact(source_facts(*source));
            }
            observation
        }
        "conformance.specification-source" => match specification_source(argument) {
            Some(source) => {
                let mut observation = Observation::shaped("some");
                observation.add_fact(source_facts(*source));
                observation
            }
            None => Observation::shaped("none"),
        },
        "conformance.exclusions" => {
            let mut observation = Observation::shaped("catalog");
            let exclusions = conformance_exclusions();
            observation.add_fact(format!("exclusions={}", exclusions.len()));
            for exclusion in exclusions {
                observation.add_fact(exclusion_facts(exclusion));
            }
            observation
        }
        "conformance.exclusion" => match conformance_exclusion(argument) {
            Some(exclusion) => {
                let mut observation = Observation::shaped("some");
                observation.add_fact(exclusion_facts(exclusion));
                observation
            }
            None => Observation::shaped("none"),
        },
        "conformance.property-support" => match property_support_metadata(argument) {
            Some(support) => {
                let mut observation = Observation::shaped("some");
                observation.add_fact(format!(
                    "property={} canonical={} aliases={} feature={}",
                    support.property().stable_id(),
                    support.canonical_name(),
                    nonempty(support.aliases().join(",")),
                    feature_facts(support.feature())
                ));
                observation
            }
            None => Observation::shaped("none"),
        },
        other => panic!("unknown characterization front `{other}`"),
    }
}

fn with_parse_charge(
    mut observation: Observation,
    observe: impl Fn(CssComponentValueLimits) -> Observation,
) -> Observation {
    parse_charge(&mut observation, observe);
    observation
}

/// A report whose syntax may be absent.
fn optional<T>(
    report: &CssParseReport<Option<T>>,
    render: impl FnOnce(&mut Observation, &T),
) -> Observation {
    let mut observation = Observation::default();
    observation.report(report);
    match report.syntax() {
        Some(syntax) => {
            observation.shape = "report:some".to_owned();
            render(&mut observation, syntax);
        }
        None => observation.shape = "report:none".to_owned(),
    }
    observation
}

/// A checked front's result: the value, or the error chain.
fn checked<T, E: std::error::Error + 'static>(
    result: Result<T, E>,
    render: impl FnOnce(&mut Observation, &T),
) -> Observation {
    match result {
        Ok(value) => {
            let mut observation = Observation::shaped("ok");
            render(&mut observation, &value);
            observation
        }
        Err(error) => {
            let mut observation = Observation::shaped("err");
            observation.add_fact(super::error_text(&error));
            observation
        }
    }
}

/// A validation front: the clean syntax, or the failure's diagnostics.
fn validation<T>(
    result: Result<T, CssValidationFailure>,
    render: impl FnOnce(&mut Observation, &T),
) -> Observation {
    match result {
        Ok(value) => {
            let mut observation = Observation::shaped("ok");
            render(&mut observation, &value);
            observation
        }
        Err(failure) => {
            let mut observation = Observation::shaped("err");
            observation.diagnostics =
                nonempty(super::report::diagnostics(failure.diagnostics()).join("~"));
            let first = super::report::diagnostics(std::slice::from_ref(failure.first()));
            observation.add_fact(format!("first={}", first.join("")));
            observation
        }
    }
}

fn joined<T, E>(items: &[T], serialize: impl Fn(&T) -> Result<String, E>) -> Result<String, E> {
    Ok(items
        .iter()
        .map(serialize)
        .collect::<Result<Vec<_>, _>>()?
        .join(" | "))
}

fn sheet(report: &CssParseReport<CssSheet>) -> Observation {
    let mut observation = Observation::shaped("report");
    observation.report(report);
    sheet_serializations(&mut observation, report.syntax());
    observation
}

fn sheet_serializations(observation: &mut Observation, sheet: &CssSheet) {
    serialization(observation, "specified", Limited::Values, |limits| {
        sheet.serialize_specified(limits)
    });
    serialization(observation, "cssom", Limited::Values, |limits| {
        sheet.serialize_cssom(limits)
    });
}

fn retained_rules(observation: &mut Observation, rules: &[CssRule]) {
    let retained = Retained::rules(rules);
    observation.retained = nonempty(retained.identities.join("~"));
    observation.authored = nonempty(retained.authored.join("~"));
}

fn retained_scoped(observation: &mut Observation, rules: &[CssRule]) {
    observation.add_fact(format!("rules={}", rules.len()));
    observation.retained = nonempty(Retained::scoped_rules(rules).identities.join("~"));
}

fn rule_serializations(observation: &mut Observation, rules: &[CssRule]) {
    serialization(observation, "specified", Limited::Values, |limits| {
        joined(rules, |rule| rule.serialize_specified(limits))
    });
    serialization(observation, "cssom", Limited::Values, |limits| {
        joined(rules, |rule| rule.serialize_cssom(limits))
    });
}

/// Nested rules of a fragment that also has its own declarations, recorded as
/// a fact so the declarations keep the serialization columns.
fn rule_serializations_into(observation: &mut Observation, label: &str, rules: &[CssRule]) {
    let mut nested = Observation::default();
    rule_serializations(&mut nested, rules);
    observation.add_fact(format!("{label}.specified={}", nested.specified));
    observation.add_fact(format!("{label}.cssom={}", nested.cssom));
    if nested.charges != "-" {
        observation.add_charge(format!("{label}[{}]", nested.charges));
    }
}

fn sheet_bytes(argument: &str, input: &str) -> Observation {
    let hints = CssStylesheetDecodeHints {
        protocol_label: (argument != "-").then_some(argument),
        environment_encoding: None,
    };
    match CssDecodedStylesheetInput::decode(input.as_bytes(), hints, Default::default()) {
        Ok(decoded) => {
            let report = CssParser::default()
                .parse_sheet_bytes(&decoded)
                .with_location(None);
            let mut observation = sheet(&report);
            observation.add_fact(format!("encoding={}", decoded.encoding().name()));
            observation.add_fact(format!("replacements={}", decoded.had_replacements()));
            observation
        }
        Err(error) => {
            let mut observation = Observation::shaped("err");
            observation.add_fact(super::error_text(&error));
            observation
        }
    }
}

fn utf16(input: &str) -> Arc<Vec<u16>> {
    Arc::new(input.encode_utf16().collect())
}

fn utf16_failure<E: std::fmt::Debug + std::fmt::Display + 'static>(
    error: CssUtf16InputError<E>,
) -> Observation {
    let mut observation = Observation::shaped("err");
    observation.add_fact(super::error_text(&error));
    observation
}

fn sheet_utf16(context: CssParserContext, input: &str, limits: CssUtf16InputLimits) -> Observation {
    match CssParser::new(context, Default::default()).parse_sheet_utf16(
        utf16(input),
        limits,
        || Ok::<(), Stop>(()),
    ) {
        Ok(report) => {
            let mut observation = sheet(&report);
            retained_rules(&mut observation, report.syntax().rules());
            observation
        }
        Err(error) => utf16_failure(error),
    }
}

fn block_contents_utf16(
    context: CssParserContext,
    input: &str,
    input_limits: CssUtf16InputLimits,
    component_limits: CssComponentValueLimits,
) -> Observation {
    match CssParser::new(context, component_limits).parse_block_contents_utf16(
        utf16(input),
        input_limits,
        || Ok::<(), Stop>(()),
    ) {
        Ok(report) => declaration_list(report.report()),
        Err(error) => utf16_failure(error),
    }
}

fn media_query_list_utf16(
    context: CssParserContext,
    input: &str,
    limits: CssUtf16InputLimits,
) -> Observation {
    match CssMediaQueryList::parse_utf16(
        &CssParser::new(context, Default::default()),
        utf16(input),
        limits,
        || Ok::<(), Stop>(()),
    ) {
        Ok(report) => {
            let mut observation = Observation::shaped("report");
            observation.report(&report);
            media_query_list(&mut observation, report.syntax());
            observation
        }
        Err(error) => utf16_failure(error),
    }
}

fn color_scheme_utf16(context: CssParserContext, input: &str) -> Observation {
    let limits = CssUtf16PropertyValueLimits {
        input: CssUtf16InputLimits::default(),
        max_work: 4_000_000,
        max_live_bytes: 16 * 1024 * 1024,
        max_retained_bytes: 16 * 1024 * 1024,
    };
    match CssColorScheme::parse_utf16(
        &CssParser::new(context, Default::default()),
        utf16(input),
        limits,
        |_| Ok::<(), Stop>(()),
        |_| {},
    ) {
        Ok(report) => {
            let mut observation = checked_declaration_ref(report.result());
            let storage = report.retained_storage();
            observation.add_fact(format!(
                "retained-bytes={},retained-units={}",
                storage.bytes(),
                storage.utf16_units()
            ));
            observation
        }
        Err(error) => utf16_failure(error),
    }
}

fn declaration_list(report: &CssParseReport<CssDeclarationList>) -> Observation {
    let mut observation = Observation::shaped("report");
    observation.report(report);
    list_serializations(&mut observation, report.syntax());
    observation
}

fn list_serializations(observation: &mut Observation, list: &CssDeclarationList) {
    serialization(observation, "specified", Limited::Values, |limits| {
        list.serialize_specified(limits)
    });
    serialization(observation, "cssom", Limited::Values, |limits| {
        list.serialize_cssom(limits)
    });
}

fn page_declarations(observation: &mut Observation, block: &CssPageDeclarationBlock) {
    serialization(observation, "cssom", Limited::Values, |limits| {
        block.serialize_cssom(limits)
    });
}

fn font_face_descriptors(observation: &mut Observation, descriptors: &CssFontFaceDescriptors) {
    observation.add_fact(format!("occurrences={}", descriptors.occurrences().len()));
}

/// Property name and importance from a `name[!important]` argument; the
/// parser mode prefix is already removed.
struct PropertyArgument {
    known: Option<CssKnownProperty>,
    custom: Option<CssCustomPropertyName>,
    importance: CssImportance,
}

impl PropertyArgument {
    fn parse(argument: &str) -> Self {
        let (name, importance) = match argument.strip_suffix("!important") {
            Some(name) => (name, CssImportance::Important),
            None => (argument, CssImportance::Normal),
        };
        let (known, custom) = if name.starts_with("--") {
            (
                None,
                Some(CssCustomPropertyName::try_new(name).expect("custom property name")),
            )
        } else {
            (CssKnownProperty::from_name(name), None)
        };
        assert!(
            known.is_some() || custom.is_some() || name == "glyph-orientation-vertical",
            "unknown characterization property `{name}`"
        );
        Self {
            known,
            custom,
            importance,
        }
    }

    fn name(&self) -> CssPropertyNameRef<'_> {
        match (&self.known, &self.custom) {
            (Some(known), _) => CssPropertyNameRef::Known(*known),
            (None, Some(custom)) => CssPropertyNameRef::Custom(custom),
            (None, None) => CssPropertyNameRef::SvgGlyphOrientationVertical,
        }
    }
}

fn property_value_text(
    context: Option<CssParserContext>,
    argument: &str,
    input: &str,
) -> Observation {
    let property = PropertyArgument::parse(argument);
    optional(
        &CssParser::new(context.unwrap_or_default(), Default::default()).parse_property_value(
            input,
            CssPropertyTextTarget::Property(property.name()),
            property.importance,
        ),
        declaration,
    )
}

fn property_value_components(
    context: Option<CssParserContext>,
    argument: &str,
    input: &str,
    target: &str,
) -> Observation {
    let property = PropertyArgument::parse(argument);
    components_then(input, |values| {
        checked_declaration(match target {
            "ordinary" => CssDeclaration::try_from_components(
                &CssParser::new(context.unwrap_or_default(), Default::default()),
                CssPropertyTarget::Property(property.name()),
                values,
                property.importance,
            ),
            "page" => CssDeclaration::try_from_components(
                &CssParser::new(context.unwrap_or_default(), Default::default()),
                CssPropertyTarget::PageProperty(property.name()),
                values,
                property.importance,
            ),
            "margin" => CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::MarginProperty(property.name()),
                values,
                property.importance,
            ),
            other => unreachable!("unknown property context {other}"),
        })
    })
}

fn grammar(argument: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(argument)
        .or_else(|| CssKnownProperty::from_name(argument).map(CssKnownProperty::grammar))
        .unwrap_or_else(|| panic!("unknown characterization grammar `{argument}`"))
}

fn svg_context(context: Option<CssParserContext>) -> CssParserContext {
    context
        .unwrap_or_default()
        .with_svg_glyph_orientation_vertical()
}

/// Component values parsed from `input`, then handed to a checked front.
fn components_then(
    input: &str,
    front: impl FnOnce(CssComponentValues) -> Observation,
) -> Observation {
    match CssComponentValues::parse(&CssParser::default(), input) {
        Ok(values) => front(values),
        Err(error) => {
            let mut observation = Observation::shaped("components:err");
            observation.add_fact(super::error_text(&error));
            observation
        }
    }
}

fn checked_declaration(result: Result<CssDeclaration, CssPropertyValueError>) -> Observation {
    checked(result, declaration)
}

fn checked_declaration_ref(result: &Result<CssDeclaration, CssPropertyValueError>) -> Observation {
    match result {
        Ok(value) => {
            let mut observation = Observation::shaped("ok");
            declaration(&mut observation, value);
            observation
        }
        Err(error) => {
            let mut observation = Observation::shaped("err");
            observation.add_fact(super::error_text(error));
            observation
        }
    }
}

fn declaration(observation: &mut Observation, declaration: &CssDeclaration) {
    let importance = match declaration.importance() {
        CssImportance::Normal => "normal",
        CssImportance::Important => "important",
    };
    observation.add_fact(format!(
        "property={} importance={importance}",
        property_id(declaration.property_name())
    ));
    serialization(observation, "specified", Limited::Values, |limits| {
        declaration.serialize_specified(limits)
    });
}

fn rule_syntax(observation: &mut Observation, syntax: &CssRuleSyntax) {
    observation.add_fact(format!(
        "at-rule={} components={}",
        syntax.at_rule_name().unwrap_or("-"),
        syntax.component_count()
    ));
}

fn media_query(observation: &mut Observation, query: &CssMediaQuery) {
    observation.specified = super::serialized(
        query
            .serialize_tokens(Default::default())
            .map(|value| value.as_css().to_owned()),
    );
    serialization(observation, "cssom", Limited::Values, |limits| {
        query
            .serialize_cssom(limits)
            .map(|value| value.as_css().to_owned())
    });
    observation.add_fact(format!("guaranteed-false={}", query.is_guaranteed_false()));
}

fn media_query_list(observation: &mut Observation, list: &CssMediaQueryList) {
    observation.specified = super::serialized(
        list.serialize_tokens(Default::default())
            .map(|value| value.as_css().to_owned()),
    );
    serialization(observation, "cssom", Limited::Values, |limits| {
        list.serialize_cssom(limits)
            .map(|value| value.as_css().to_owned())
    });
    observation.add_fact(format!("queries={}", list.queries().len()));
}

fn supports_condition(observation: &mut Observation, condition: &CssCondition<CssSupportsLeaf>) {
    serialization(observation, "specified", Limited::Bytes, |limits| {
        condition
            .serialize_tokens(crate::css::support::byte_limits(limits.max_css_bytes()))
            .map(|value| value.as_css().to_owned())
    });
}

fn when_condition(observation: &mut Observation, condition: &CssCondition<CssWhenLeaf>) {
    serialization(observation, "specified", Limited::Bytes, |limits| {
        condition
            .serialize_tokens(crate::css::support::byte_limits(limits.max_css_bytes()))
            .map(|value| value.as_css().to_owned())
    });
}

fn supports_test_child(observation: &mut Observation, child: &CssSupportsTestChild) {
    serialization(observation, "specified", Limited::Values, |limits| {
        child.as_ref().serialize_specified(limits)
    });
}

fn supports_test_body(observation: &mut Observation, body: &CssSupportsTestBody) {
    serialization(observation, "specified", Limited::Values, |limits| {
        body.serialize_specified(limits)
    });
    serialization(observation, "cssom", Limited::Bytes, |limits| {
        body.serialize_tokens(crate::css::support::byte_limits(limits.max_css_bytes()))
            .map(|value| value.as_css().to_owned())
    });
}

fn keyframe_block(observation: &mut Observation, block: &CssKeyframeBlock) {
    serialization(observation, "specified", Limited::Values, |limits| {
        block.selectors().serialize_key_text(limits)
    });
    serialization(observation, "cssom", Limited::Values, |limits| {
        block.declarations().serialize_cssom(limits)
    });
}

fn margin_rule(observation: &mut Observation, rule: &CssMarginRule) {
    serialization(observation, "cssom", Limited::Values, |limits| {
        rule.serialize_cssom(limits)
    });
}

fn family_list(observation: &mut Observation, families: &CssFontFeatureValuesFamilyList) {
    serialization(observation, "specified", Limited::Values, |limits| {
        families.serialize_specified(limits)
    });
}

fn component_value(observation: &mut Observation, value: &CssComponentValue) {
    match CssComponentValues::try_new(vec![value.clone()], Default::default()) {
        Ok(values) => component_values(observation, &values),
        Err(error) => observation.add_fact(format!("wrap:{}", super::error_text(&error))),
    }
}

fn component_values(observation: &mut Observation, values: &CssComponentValues) {
    observation.add_fact(format!(
        "items={} components={}",
        values.items().len(),
        values.component_count()
    ));
    serialization(observation, "specified", Limited::Bytes, |limits| {
        values
            .serialize_tokens(crate::css::support::byte_limits(limits.max_css_bytes()))
            .map(|value| value.as_css().to_owned())
    });
}

fn comma_values(report: &CssParseReport<Vec<Option<CssComponentValues>>>) -> Observation {
    let mut observation = Observation::shaped("report");
    observation.report(report);
    let items = report.syntax();
    observation.add_fact(format!("items={}", items.len()));
    serialization(&mut observation, "specified", Limited::Bytes, |limits| {
        Ok::<_, CssComponentValueError>(
            items
                .iter()
                .map(|item| match item {
                    Some(values) => values
                        .serialize_tokens(crate::css::support::byte_limits(limits.max_css_bytes()))
                        .map(|value| format!("[{}]", value.as_css())),
                    None => Ok("[-]".to_owned()),
                })
                .collect::<Result<Vec<_>, _>>()?
                .join(" "),
        )
    });
    observation
}

fn namespaces(argument: &str) -> CssNamespaceContext {
    match argument {
        "svg" => CssNamespaceContext::from_bindings([(
            CssNamespacePrefix::try_new("svg"),
            CssNamespaceName::new("http://www.w3.org/2000/svg"),
        )]),
        _ => CssNamespaceContext::default(),
    }
}

fn ancestry(argument: &str) -> CssStyleAncestor {
    match argument {
        "present" => CssStyleAncestor::Present,
        "absent" => CssStyleAncestor::Absent,
        other => panic!("unknown style ancestry `{other}`"),
    }
}

fn style_selector_context(argument: &str) -> CssStyleSelectorContext {
    match argument {
        "ordinary" => CssStyleSelectorContext::Ordinary,
        "nested" => CssStyleSelectorContext::Nested,
        "scoped-present" => CssStyleSelectorContext::Scoped(CssStyleAncestor::Present),
        "scoped-absent" => CssStyleSelectorContext::Scoped(CssStyleAncestor::Absent),
        other => panic!("unknown style selector context `{other}`"),
    }
}

fn font_feature_kind(argument: &str) -> CssFontFeatureValueKind {
    match argument {
        "styleset" => CssFontFeatureValueKind::Styleset,
        "stylistic" => CssFontFeatureValueKind::Stylistic,
        "swash" => CssFontFeatureValueKind::Swash,
        "ornaments" => CssFontFeatureValueKind::Ornaments,
        "annotation" => CssFontFeatureValueKind::Annotation,
        "character-variant" => CssFontFeatureValueKind::CharacterVariant,
        "historical-forms" => CssFontFeatureValueKind::HistoricalForms,
        other => panic!("unknown font feature value kind `{other}`"),
    }
}

fn color_profile_descriptor(argument: &str) -> CssColorProfileDescriptorKind {
    match argument {
        "src" => CssColorProfileDescriptorKind::Src,
        "rendering-intent" => CssColorProfileDescriptorKind::RenderingIntent,
        "components" => CssColorProfileDescriptorKind::Components,
        other => panic!("unknown color-profile descriptor `{other}`"),
    }
}

fn counter_style_descriptor(argument: &str) -> CssCounterStyleDescriptorKind {
    match argument {
        "system" => CssCounterStyleDescriptorKind::System,
        "symbols" => CssCounterStyleDescriptorKind::Symbols,
        "additive-symbols" => CssCounterStyleDescriptorKind::AdditiveSymbols,
        "negative" => CssCounterStyleDescriptorKind::Negative,
        "prefix" => CssCounterStyleDescriptorKind::Prefix,
        "suffix" => CssCounterStyleDescriptorKind::Suffix,
        "range" => CssCounterStyleDescriptorKind::Range,
        "pad" => CssCounterStyleDescriptorKind::Pad,
        "speak-as" => CssCounterStyleDescriptorKind::SpeakAs,
        "fallback" => CssCounterStyleDescriptorKind::Fallback,
        other => panic!("unknown counter-style descriptor `{other}`"),
    }
}

fn font_face_descriptor(argument: &str) -> CssFontFaceDescriptorKind {
    match argument {
        "src" => CssFontFaceDescriptorKind::Src,
        "font-family" => CssFontFaceDescriptorKind::FontFamily,
        "font-weight" => CssFontFaceDescriptorKind::FontWeight,
        "font-display" => CssFontFaceDescriptorKind::FontDisplay,
        "unicode-range" => CssFontFaceDescriptorKind::UnicodeRange,
        other => panic!("unknown font-face descriptor `{other}`"),
    }
}

fn font_palette_descriptor(argument: &str) -> CssFontPaletteDescriptorKind {
    match argument {
        "base-palette" => CssFontPaletteDescriptorKind::BasePalette,
        "font-family" => CssFontPaletteDescriptorKind::FontFamily,
        "override-colors" => CssFontPaletteDescriptorKind::OverrideColors,
        other => panic!("unknown font-palette descriptor `{other}`"),
    }
}

fn page_descriptor(argument: &str) -> CssPageDescriptorKind {
    match argument {
        "size" => CssPageDescriptorKind::Size,
        "marks" => CssPageDescriptorKind::Marks,
        "bleed" => CssPageDescriptorKind::Bleed,
        "page-orientation" => CssPageDescriptorKind::PageOrientation,
        other => panic!("unknown page descriptor `{other}`"),
    }
}

/// A lexical serializer: its text and charged work.
fn lexical(
    serialize: impl Fn(CssSerializationLimits) -> Result<String, CssSerializationError>,
) -> Observation {
    let mut observation = Observation::shaped("serialization");
    serialization(&mut observation, "specified", Limited::Values, serialize);
    observation
}

/// Normalized items of a parsed sheet (`sheet`) or report (`report`), and the
/// smallest normalization limits that keep the outcome.
fn normalized_sheet(report: &CssParseReport<CssSheet>, argument: &str) -> Observation {
    let render = |limits: Option<CssNormalizationLimits>| {
        let mut observation = Observation::shaped("normalized");
        let result = match (argument, limits) {
            ("report", None) => report.normalize(Default::default()).map(|normalized| {
                observation.clean = normalized.is_clean().to_string();
                observation.diagnostics =
                    nonempty(super::report::diagnostics(normalized.diagnostics()).join("~"));
                normalized
                    .syntax()
                    .items()
                    .iter()
                    .map(item_facts)
                    .collect::<Vec<_>>()
            }),
            ("report", Some(limits)) => report.normalize(limits).map(|normalized| {
                normalized
                    .syntax()
                    .items()
                    .iter()
                    .map(item_facts)
                    .collect::<Vec<_>>()
            }),
            ("sheet", None) => report
                .syntax()
                .normalize(Default::default())
                .map(|sheet| sheet.items().iter().map(item_facts).collect::<Vec<_>>()),
            ("sheet", Some(limits)) => report
                .syntax()
                .normalize(limits)
                .map(|sheet| sheet.items().iter().map(item_facts).collect::<Vec<_>>()),
            (other, _) => panic!("unknown normalization input `{other}`"),
        };
        match result {
            Ok(items) => {
                observation.add_fact(format!("items={}", items.len()));
                for item in items {
                    observation.add_fact(item);
                }
            }
            Err(error) => {
                observation.shape = "err".to_owned();
                observation.add_fact(super::error_text(&error));
            }
        }
        observation
    };
    let mut observation = render(None);
    let defaults = CssNormalizationLimits::default();
    let baseline = render(Some(defaults));
    let same = |limits: Option<CssNormalizationLimits>| {
        limits.is_some_and(|limits| {
            let probe = render(Some(limits));
            probe.shape == baseline.shape && probe.facts == baseline.facts
        })
    };
    let depth = super::smallest(defaults.max_rule_depth() as usize, |value| {
        same(CssNormalizationLimits::try_new(
            u32::try_from(value).unwrap(),
            defaults.max_rules(),
            defaults.max_declarations(),
            defaults.max_contributions(),
        ))
    });
    let rules = super::smallest(defaults.max_rules(), |value| {
        same(CssNormalizationLimits::try_new(
            defaults.max_rule_depth(),
            value,
            defaults.max_declarations(),
            defaults.max_contributions(),
        ))
    });
    let declarations = super::smallest(defaults.max_declarations(), |value| {
        same(CssNormalizationLimits::try_new(
            defaults.max_rule_depth(),
            defaults.max_rules(),
            value,
            defaults.max_contributions(),
        ))
    });
    let contributions = super::smallest(defaults.max_contributions(), |value| {
        same(CssNormalizationLimits::try_new(
            defaults.max_rule_depth(),
            defaults.max_rules(),
            defaults.max_declarations(),
            value,
        ))
    });
    observation.add_charge(format!(
        "normalize:depth={},rules={},declarations={},contributions={}",
        super::amount(depth),
        super::amount(rules),
        super::amount(declarations),
        super::amount(contributions)
    ));
    observation
}

fn item_facts(item: &CssNormalizedItem) -> String {
    match item {
        CssNormalizedItem::Declaration(declaration) => format!(
            "declaration#{}:{}:rule={}:{}",
            declaration.order(),
            property_id(declaration.source().property_name()),
            rule_context_name(declaration.rule_context()),
            expansion_facts(declaration.expansion())
        ),
        CssNormalizedItem::Rule(context) => format!("rule:{}", rule_context_name(context)),
        _ => "future-item".to_owned(),
    }
}

fn rule_context_name(context: &CssRuleContext) -> String {
    let name = match context.kind() {
        CssRuleContextKindRef::ColorProfile(_) => "color-profile",
        CssRuleContextKindRef::Container { .. } => "container",
        CssRuleContextKindRef::CounterStyle(_) => "counter-style",
        CssRuleContextKindRef::CustomMedia(_) => "custom-media",
        CssRuleContextKindRef::Else(_) => "else",
        CssRuleContextKindRef::FontFace(_) => "font-face",
        CssRuleContextKindRef::FontFeatureValues(_) => "font-feature-values",
        CssRuleContextKindRef::FontPaletteValues(_) => "font-palette-values",
        CssRuleContextKindRef::Import(_) => "import",
        CssRuleContextKindRef::Keyframes(_) => "keyframes",
        CssRuleContextKindRef::LayerBlock(_) => "layer-block",
        CssRuleContextKindRef::LayerStatement(_) => "layer-statement",
        CssRuleContextKindRef::Media(_) => "media",
        CssRuleContextKindRef::Namespace(_) => "namespace",
        CssRuleContextKindRef::NestedDeclarations(_) => "nested-declarations",
        CssRuleContextKindRef::Page(_) => "page",
        CssRuleContextKindRef::Scope { .. } => "scope",
        CssRuleContextKindRef::ScopedStyle(_) => "scoped-style",
        CssRuleContextKindRef::Style(_) => "style",
        CssRuleContextKindRef::Supports(_) => "supports",
        CssRuleContextKindRef::SupportsCondition(_) => "supports-condition",
        CssRuleContextKindRef::When(_) => "when",
        _ => "future",
    };
    match context.parent() {
        Some(parent) => format!("{}/{name}", rule_context_name(parent)),
        None => name.to_owned(),
    }
}

/// The contributions of an expansion, and their CSSOM serialization through a
/// specified declaration block.
fn expansion_facts(expansion: &CssExpansion) -> String {
    let shape = match expansion {
        CssExpansion::Contributions(contributions) => match contributions {
            CssContributions::Longhands(longhands) => format!(
                "longhands[{}]",
                longhands
                    .items()
                    .iter()
                    .map(|longhand| format!(
                        "{}:{}",
                        longhand.property().stable_id(),
                        match longhand.value() {
                            CssContributionValueRef::Global(_) => "global",
                            CssContributionValueRef::Ordinary(_) => "ordinary",
                            CssContributionValueRef::SystemFont(_) => "system-font",
                            CssContributionValueRef::UserAgentInitial(_) => "ua-initial",
                            _ => "future",
                        }
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            CssContributions::Custom(custom) => {
                format!("custom[{}]", custom.declaration().name().as_str())
            }
            CssContributions::UniversalReset(reset) => {
                format!("universal-reset[{}]", global_keyword(reset.keyword()))
            }
            CssContributions::SvgGlyphOrientationVertical(_) => "svg-glyph".to_owned(),
            _ => "future".to_owned(),
        },
        CssExpansion::Pending(pending) => {
            format!("pending[{}]", property_id(pending.source().property_name()))
        }
        _ => "future".to_owned(),
    };
    let block = match CssSpecifiedDeclarationBlock::try_from_expansions(
        std::slice::from_ref(expansion),
        Default::default(),
    ) {
        Ok(block) => super::serialized(block.serialize_cssom(Default::default())),
        Err(error) => format!("err:{}", super::error_text(&error)),
    };
    format!("{shape}=>{block}")
}

fn global_keyword(keyword: CssGlobalKeyword) -> &'static str {
    match keyword {
        CssGlobalKeyword::Inherit => "inherit",
        CssGlobalKeyword::Initial => "initial",
        CssGlobalKeyword::Unset => "unset",
        CssGlobalKeyword::Revert => "revert",
        CssGlobalKeyword::RevertLayer => "revert-layer",
        _ => "future",
    }
}

/// Graph limits with `values` and no structural depth bound, the contract of
/// the value-limited graph serializers this front observed before they took
/// graph limits.
fn unbounded_depth(values: CssSerializationLimits) -> CssRuleSerializationLimits {
    CssRuleSerializationLimits {
        values,
        max_depth: usize::MAX,
    }
}

/// The rule graph of each parsed rule, and of the first style rule rebuilt
/// from its selectors and specified declarations.
fn rule_graph(input: &str) -> Observation {
    let report = CssParser::default().parse_sheet(input);
    let mut observation = Observation::shaped("report");
    observation.report(&report);
    let namespaces = CssNamespaceContext::from_sheet(report.syntax());
    let rules = report.syntax().rules();
    serialization(&mut observation, "cssom", Limited::Values, |limits| {
        joined(rules, |rule| {
            CssRuleGraphInput::from_rule(rule).serialize_cssom(
                &namespaces,
                CssStyleAncestor::Absent,
                unbounded_depth(limits),
            )
        })
    });
    observation.add_fact(format!(
        "graph-limits={}",
        super::serialized(joined(rules, |rule| {
            CssRuleGraphInput::from_rule(rule).serialize_cssom(
                &namespaces,
                CssStyleAncestor::Present,
                CssRuleSerializationLimits::default(),
            )
        }))
    ));
    if let Some(CssRule::Style(style)) = rules.first() {
        match CssSpecifiedDeclarationBlock::try_from_declarations(
            style.declarations(),
            Default::default(),
        ) {
            Ok(block) => {
                observation.add_fact(format!(
                    "style-input={}",
                    super::serialized(
                        CssRuleGraphInput::style(style.selectors(), &block, &[]).serialize_cssom(
                            &namespaces,
                            CssStyleAncestor::Absent,
                            unbounded_depth(Default::default()),
                        )
                    )
                ));
                observation.add_fact(format!(
                    "nested-declarations-input={}",
                    super::serialized(
                        CssRuleGraphInput::nested_declarations(&block).serialize_cssom(
                            &namespaces,
                            CssStyleAncestor::Present,
                            unbounded_depth(Default::default()),
                        )
                    )
                ));
            }
            Err(error) => observation.add_fact(format!("block-err={}", super::error_text(&error))),
        }
    }
    observation
}

/// Graph inputs of each parsed rule, and of the first style rule rebuilt
/// from its selectors and specified declarations.
fn edited_rule(input: &str) -> Observation {
    let report = CssParser::default().parse_sheet(input);
    let mut observation = Observation::shaped("report");
    observation.report(&report);
    let namespaces = CssNamespaceContext::from_sheet(report.syntax());
    let rules = report.syntax().rules();
    serialization(&mut observation, "specified", Limited::Values, |limits| {
        joined(rules, |rule| {
            CssRuleGraphInput::from_rule(rule).serialize_specified(unbounded_depth(limits))
        })
    });
    serialization(&mut observation, "cssom", Limited::Values, |limits| {
        joined(rules, |rule| {
            CssRuleGraphInput::from_rule(rule).serialize_cssom(
                &namespaces,
                CssStyleAncestor::Absent,
                unbounded_depth(limits),
            )
        })
    });
    if let Some(CssRule::Style(style)) = rules.first() {
        match CssSpecifiedDeclarationBlock::try_from_declarations(
            style.declarations(),
            Default::default(),
        ) {
            Ok(block) => {
                observation.add_fact(format!(
                    "edited-style={}",
                    super::serialized(
                        CssRuleGraphInput::style(style.selectors(), &block, &[])
                            .serialize_specified(unbounded_depth(Default::default()))
                    )
                ));
                observation.add_fact(format!(
                    "edited-nested-declarations={}",
                    super::serialized(
                        CssRuleGraphInput::nested_declarations(&block).serialize_cssom(
                            &namespaces,
                            CssStyleAncestor::Absent,
                            unbounded_depth(Default::default()),
                        )
                    )
                ));
            }
            Err(error) => observation.add_fact(format!("block-err={}", super::error_text(&error))),
        }
    }
    observation
}

fn feature_facts(feature: &CssFeatureMetadata) -> String {
    format!(
        "{} kind={} spelling={} production={} status={} subset={} remainder={} unsupported-code={} source={}",
        feature.id().as_str(),
        match feature.kind() {
            CssFeatureKind::ContainerQuery => "container-query",
            CssFeatureKind::Declaration => "declaration",
            CssFeatureKind::Descriptor => "descriptor",
            CssFeatureKind::MediaQuery => "media-query",
            CssFeatureKind::Property => "property",
            CssFeatureKind::PropertyAlias => "property-alias",
            CssFeatureKind::Rule => "rule",
            CssFeatureKind::Selector => "selector",
            CssFeatureKind::Value => "value",
            _ => "future",
        },
        feature.spelling(),
        feature.production(),
        match feature.status() {
            CssSupportStatus::Complete => "complete",
            CssSupportStatus::Partial => "partial",
            CssSupportStatus::RecognizedUnsupported => "recognized-unsupported",
        },
        feature.supported_subset().unwrap_or("-"),
        feature.unsupported_remainder().unwrap_or("-"),
        feature
            .recognized_unsupported_code()
            .map_or("-", super::report::code_name),
        feature.source().id().as_str()
    )
}

fn source_facts(source: CssSpecificationSource) -> String {
    format!(
        "{} module={} level={} tier={} url={} provenance={}",
        source.id().as_str(),
        source.module(),
        source.level(),
        match source.tier() {
            CssSpecificationTier::LaterStandard => "later-standard",
            CssSpecificationTier::Snapshot2026Interop => "snapshot-2026-interop",
            CssSpecificationTier::Snapshot2026Official => "snapshot-2026-official",
            CssSpecificationTier::Snapshot2026PreCrException => "snapshot-2026-pre-cr-exception",
            CssSpecificationTier::Snapshot2026Reliable => "snapshot-2026-reliable",
            CssSpecificationTier::Snapshot2026Stable => "snapshot-2026-stable",
            CssSpecificationTier::SurgeistExtension => "surgeist-extension",
            _ => "future",
        },
        source.url().unwrap_or("-"),
        source.repository_provenance().unwrap_or("-")
    )
}

fn exclusion_facts(exclusion: &CssExclusionMetadata) -> String {
    format!(
        "{} production={} reason={} source={} superseded-by={}",
        exclusion.id().as_str(),
        exclusion.production(),
        match exclusion.reason() {
            CssExclusionReason::InformativeOnly => "informative-only",
            CssExclusionReason::OutsideAuthoredSyntaxBoundary => "outside-authored-syntax-boundary",
            CssExclusionReason::SupersededWithoutCurrentProduction => {
                "superseded-without-current-production"
            }
            _ => "future",
        },
        exclusion.source().id().as_str(),
        exclusion.superseding_ids().map_or_else(
            || "-".to_owned(),
            |ids| ids
                .iter()
                .map(|id| id.as_str())
                .collect::<Vec<_>>()
                .join(",")
        )
    )
}
