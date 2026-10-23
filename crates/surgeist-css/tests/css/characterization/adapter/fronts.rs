//! Parsing fronts: one arm per stable front id.

use std::sync::Arc;

use surgeist_css::*;

use super::report::{Retained, property_id};
use super::{Limited, Observation, Stop, nonempty, parse_charge, serialization, utf16_charge};

/// Calls the `CssParserContext` method when the case names a parser mode, and
/// the free function of the same name otherwise.
macro_rules! via {
    ($context:expr, $front:ident($($argument:expr),* $(,)?)) => {
        match $context {
            Some(context) => context.$front($($argument),*),
            None => $front($($argument),*),
        }
    };
}

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
            let report = via!(context, parse_sheet(input));
            let mut observation = sheet(&report);
            retained_rules(&mut observation, report.syntax().rules());
            observation
        }
        "sheet.with-options" => {
            let options = CssStylesheetParseOptions {
                location: (argument != "-")
                    .then(|| CssStylesheetLocation::new(argument.to_owned())),
            };
            let report = parse_sheet_with_options(input, options);
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
        "sheet.validate" => validation(validate_sheet(input), |observation, sheet| {
            sheet_serializations(observation, sheet)
        }),
        "sheet.normalize" => normalized_sheet(&parse_sheet(input), argument),
        "style" => {
            let report = via!(context, parse_style_attribute(input));
            let mut observation = declaration_list(&report);
            let retained = Retained::declarations(report.syntax().as_slice());
            observation.retained = nonempty(retained.identities.join("~"));
            observation.authored = nonempty(retained.authored.join("~"));
            observation
        }
        "style.validate" => validation(validate_style_attribute(input), list_serializations),
        "declaration.expand" => {
            let report = parse_style_attribute(input);
            let mut observation = Observation::shaped("report");
            observation.report(&report);
            observation.add_fact(
                report
                    .syntax()
                    .iter()
                    .map(|declaration| match expand_declaration(declaration) {
                        Ok(expansion) => expansion_facts(&expansion),
                        Err(error) => format!("err:{}", super::error_text(&error)),
                    })
                    .collect::<Vec<_>>()
                    .join(" ; "),
            );
            observation
        }
        "block-contents" => with_parse_charge(
            declaration_list(&via!(context, parse_declaration_block_contents(input))),
            |limits| {
                declaration_list(&via!(
                    context,
                    parse_declaration_block_contents_with_limits(input, limits)
                ))
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
                &via!(context, parse_page_declaration_block_contents(input)),
                page_declarations,
            ),
            |limits| {
                optional(
                    &via!(
                        context,
                        parse_page_declaration_block_contents_with_limits(input, limits)
                    ),
                    page_declarations,
                )
            },
        ),
        "block-contents.font-face" => with_parse_charge(
            optional(
                &parse_font_face_declaration_block_contents(input),
                font_face_descriptors,
            ),
            |limits| {
                optional(
                    &parse_font_face_declaration_block_contents_with_limits(input, limits),
                    font_face_descriptors,
                )
            },
        ),
        "declaration-list" => declaration_list(&via!(context, parse_declaration_list_text(input))),
        "declaration-list.validate" => {
            validation(validate_declaration_list_text(input), list_serializations)
        }
        "property-value.text" => property_value_text(context, argument, input),
        "property-value.components" => {
            property_value_components(context, argument, input, "ordinary")
        }
        "property-value.page" => property_value_components(context, argument, input, "page"),
        "property-value.margin" => property_value_components(context, argument, input, "margin"),
        "property-value.grammar.text" => {
            let grammar = grammar(argument);
            optional(
                &via!(
                    context,
                    parse_property_value_text_for_grammar(input, grammar, CssImportance::Normal)
                ),
                declaration,
            )
        }
        "property-value.grammar.components" => {
            let grammar = grammar(argument);
            components_then(input, |values| {
                checked_declaration(via!(
                    context,
                    parse_property_value_for_grammar(grammar, values, CssImportance::Normal)
                ))
            })
        }
        "svg-glyph-orientation-vertical.text" => optional(
            &svg_context(context).parse_svg_glyph_orientation_vertical_attribute_value(input),
            declaration,
        ),
        "svg-glyph-orientation-vertical.components" => components_then(input, |values| {
            checked_declaration(
                svg_context(context)
                    .parse_svg_glyph_orientation_vertical_attribute_components(values),
            )
        }),
        "declaration" => optional(&via!(context, parse_declaration(input)), declaration),
        "rule" => {
            let namespaces = namespaces(argument);
            optional(
                &via!(context, parse_rule(input, &namespaces)),
                |observation, rule| {
                    rule_serializations(observation, std::slice::from_ref(rule));
                    retained_rules(observation, std::slice::from_ref(rule));
                },
            )
        }
        "rule-graph" => rule_graph(input),
        "edited-rule" => edited_rule(input),
        "rule-syntax" => with_parse_charge(
            optional(&classify_rule_syntax(input), rule_syntax),
            |limits| {
                optional(
                    &classify_rule_syntax_with_limits(input, limits),
                    rule_syntax,
                )
            },
        ),
        "selector" => optional(
            &parse_selector(input, &namespaces(argument)),
            |observation, selector| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    selector.to_specified_css_with_limits(limits)
                });
            },
        ),
        "selector-list" => {
            let namespaces = namespaces(argument);
            optional(
                &parse_selector_list(input, &namespaces),
                |observation, selectors| {
                    observation.add_fact(format!("selectors={}", selectors.selectors().len()));
                    serialization(observation, "cssom", Limited::Values, |limits| {
                        selectors.serialize_cssom_with_limits(
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
                        selectors.serialize_cssom_with_limits(&namespaces, limits)
                    });
                })
            };
            with_parse_charge(
                render(&parse_style_selector_list(input, &namespaces, context)),
                |limits| {
                    render(&parse_style_selector_list_with_limits(
                        input,
                        &namespaces,
                        context,
                        limits,
                    ))
                },
            )
        }
        "relative-selector-list" => {
            let namespaces = CssNamespaceContext::default();
            optional(
                &parse_relative_selector_list(input, &namespaces),
                |observation, selectors| {
                    serialization(observation, "specified", Limited::Values, |limits| {
                        selectors.to_specified_css_with_limits(limits)
                    });
                    serialization(observation, "cssom", Limited::Values, |limits| {
                        selectors.serialize_cssom_with_limits(&namespaces, limits)
                    });
                },
            )
        }
        "media-query" => {
            let report = parse_media_query(input);
            let mut observation = Observation::shaped("report");
            observation.report(&report);
            media_query(&mut observation, report.syntax());
            observation
        }
        "media-query.cssom" => optional(&parse_cssom_media_query(input), media_query),
        "media-query-list" => {
            let report = via!(context, parse_media_query_list(input));
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
                    utf16_context.parse_supports_condition(input, &namespaces),
                    supports_condition,
                ),
                |limits| {
                    checked(
                        utf16_context.parse_supports_condition_with_limits(
                            input,
                            &namespaces,
                            limits,
                        ),
                        supports_condition,
                    )
                },
            )
        }
        "supports.css-text" => checked(
            via!(context, parse_css_supports_condition(input)),
            supports_condition,
        ),
        "supports.declaration" => checked(
            via!(context, parse_css_supports_declaration(argument, input)),
            |observation, declaration| {
                serialization(observation, "specified", Limited::Bytes, |limits| {
                    declaration
                        .serialize_with_limit(limits.max_css_bytes())
                        .map(|value| value.as_css().to_owned())
                });
            },
        ),
        "supports.test-child" => with_parse_charge(
            optional(&parse_supports_test_child(input), supports_test_child),
            |limits| {
                optional(
                    &parse_supports_test_child_with_limits(input, limits),
                    supports_test_child,
                )
            },
        ),
        "supports.test-body" => with_parse_charge(
            optional(&parse_supports_test_contents(input), supports_test_body),
            |limits| {
                optional(
                    &parse_supports_test_contents_with_limits(input, limits),
                    supports_test_body,
                )
            },
        ),
        "when-condition" => with_parse_charge(
            checked(via!(context, parse_when_condition(input)), when_condition),
            |limits| {
                checked(
                    via!(context, parse_when_condition_with_limits(input, limits)),
                    when_condition,
                )
            },
        ),
        "keyframe-rule" => optional(
            &via!(context, parse_keyframe_rule(input)),
            |observation, rule| keyframe_block(observation, rule.block()),
        ),
        "keyframe-selectors" => optional(
            &parse_keyframe_selector_list(input),
            |observation, selectors| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    selectors.selectors().serialize_key_text_with_limits(limits)
                });
            },
        ),
        "page-selectors" => optional(
            &parse_page_selector_list(input),
            |observation, selectors| {
                serialization(observation, "cssom", Limited::Values, |limits| {
                    selectors.serialize_cssom_with_limits(limits)
                });
            },
        ),
        "page-margin-rule" => with_parse_charge(
            optional(&via!(context, parse_page_margin_rule(input)), margin_rule),
            |limits| {
                optional(
                    &via!(context, parse_page_margin_rule_with_limits(input, limits)),
                    margin_rule,
                )
            },
        ),
        "font-feature-family-list" => with_parse_charge(
            optional(
                &via!(context, parse_font_feature_values_family_list(input)),
                family_list,
            ),
            |limits| {
                optional(
                    &via!(
                        context,
                        parse_font_feature_values_family_list_with_limits(input, limits)
                    ),
                    family_list,
                )
            },
        ),
        "font-feature-value" => optional(
            &parse_font_feature_value(input, font_feature_kind(argument)),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.to_specified_css_with_limits(limits)
                });
            },
        ),
        "font-feature-display-value" => optional(
            &parse_font_feature_display_value(input),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.to_specified_css_with_limits(limits)
                });
            },
        ),
        "descriptor.color-profile" => optional(
            &parse_color_profile_descriptor_value(input, color_profile_descriptor(argument)),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.to_specified_css_with_limits(limits)
                });
            },
        ),
        "descriptor.counter-style" => optional(
            &parse_counter_style_descriptor_value(input, counter_style_descriptor(argument)),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.to_specified_css_with_limits(limits)
                });
            },
        ),
        "descriptor.font-face" => optional(
            &parse_font_face_descriptor_value(input, font_face_descriptor(argument)),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.serialize_specified_with_limits(limits)
                });
            },
        ),
        "descriptor.font-palette" => optional(
            &parse_font_palette_descriptor_value(input, font_palette_descriptor(argument)),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.to_specified_css_with_limits(limits)
                });
            },
        ),
        "descriptor.page" => optional(
            &parse_page_descriptor_value(input, page_descriptor(argument)),
            |observation, value| {
                serialization(observation, "specified", Limited::Values, |limits| {
                    value.serialize_specified_with_limits(limits)
                });
            },
        ),
        "block.color-profile" => {
            optional(&parse_color_profile_block(input), |observation, block| {
                let descriptors = block.body();
                observation.add_fact(format!("descriptors={}", descriptors.len()));
                serialization(observation, "specified", Limited::Values, |limits| {
                    joined(descriptors, |descriptor| {
                        descriptor.to_specified_css_with_limits(limits)
                    })
                });
            })
        }
        "block.counter-style" => {
            optional(&parse_counter_style_block(input), |observation, block| {
                observation.add_fact(format!("entries={}", block.body().entries().len()));
            })
        }
        "block.font-face" => optional(&parse_font_face_block(input), |observation, block| {
            font_face_descriptors(observation, block.body())
        }),
        "block.font-feature-value" => optional(
            &parse_font_feature_value_block(input, font_feature_kind(argument)),
            |observation, block| {
                observation.add_fact(format!("definitions={}", block.body().definitions().len()));
            },
        ),
        "block.font-feature-values" => optional(
            &parse_font_feature_values_block(input),
            |observation, block| {
                observation.add_fact(format!("items={}", block.body().len()));
            },
        ),
        "block.font-palette-values" => optional(
            &parse_font_palette_values_block(input),
            |observation, block| {
                let descriptors = block.body();
                observation.add_fact(format!("descriptors={}", descriptors.len()));
                serialization(observation, "specified", Limited::Values, |limits| {
                    joined(descriptors, |descriptor| {
                        descriptor.value().to_specified_css_with_limits(limits)
                    })
                });
            },
        ),
        "block.group" => {
            let namespaces = namespaces(argument);
            optional(
                &via!(context, parse_group_block(input, &namespaces)),
                |observation, block| {
                    rule_serializations(observation, block.body().rules());
                    retained_rules(observation, block.body().rules());
                },
            )
        }
        "block.keyframe-declarations" => optional(
            &via!(context, parse_keyframe_declaration_block(input)),
            |observation, block| {
                serialization(observation, "cssom", Limited::Values, |limits| {
                    block.body().serialize_cssom_with_limits(limits)
                });
            },
        ),
        "block.keyframes" => optional(
            &via!(context, parse_keyframes_block(input)),
            |observation, block| {
                observation.add_fact(format!("blocks={}", block.body().len()));
                serialization(observation, "specified", Limited::Values, |limits| {
                    joined(block.body(), |keyframe| {
                        keyframe.selectors().serialize_key_text_with_limits(limits)
                    })
                });
                serialization(observation, "cssom", Limited::Values, |limits| {
                    joined(block.body(), |keyframe| {
                        keyframe.declarations().serialize_cssom_with_limits(limits)
                    })
                });
            },
        ),
        "block.margin" => optional(
            &via!(context, parse_margin_block(input)),
            |observation, block| {
                serialization(observation, "cssom", Limited::Values, |limits| {
                    block.body().serialize_cssom_with_limits(limits)
                });
            },
        ),
        "block.page" => optional(
            &via!(context, parse_page_block(input)),
            |observation, block| {
                let body = block.body();
                page_declarations(observation, body.declarations());
                observation.add_fact(format!("margin-rules={}", body.margin_rules().len()));
            },
        ),
        "block.scope" => {
            let namespaces = CssNamespaceContext::default();
            optional(
                &via!(
                    context,
                    parse_scope_block(input, &namespaces, ancestry(argument))
                ),
                |observation, block| retained_scoped(observation, block.body().rules()),
            )
        }
        "block.scoped-group" => {
            let namespaces = CssNamespaceContext::default();
            optional(
                &via!(
                    context,
                    parse_scoped_group_block(input, &namespaces, ancestry(argument))
                ),
                |observation, block| retained_scoped(observation, block.body().rules()),
            )
        }
        "block.supports-test" => {
            optional(&parse_supports_test_block(input), |observation, block| {
                supports_test_body(observation, block.body())
            })
        }
        "block.style" => {
            let namespaces = CssNamespaceContext::default();
            optional(
                &via!(context, parse_style_block(input, &namespaces)),
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
            optional(&parse_component_value(input), component_value),
            |limits| {
                optional(
                    &parse_component_value_with_limits(input, limits),
                    component_value,
                )
            },
        ),
        "component-values" => with_parse_charge(
            checked(parse_component_values(input), component_values),
            |limits| {
                checked(
                    parse_component_values_with_limits(input, limits),
                    component_values,
                )
            },
        ),
        "component-values.comma" => with_parse_charge(
            comma_values(&parse_comma_separated_component_values(input)),
            |limits| {
                comma_values(&parse_comma_separated_component_values_with_limits(
                    input, limits,
                ))
            },
        ),
        "color-scheme.utf16" => color_scheme_utf16(utf16_context, input),
        "serialize.identifier" => lexical(|limits| {
            if limits == CssSpecifiedValueSerializationLimits::default() {
                serialize_css_identifier(input)
            } else {
                serialize_css_identifier_with_limits(input, limits)
            }
        }),
        "serialize.string" => lexical(|limits| {
            if limits == CssSpecifiedValueSerializationLimits::default() {
                serialize_css_string(input)
            } else {
                serialize_css_string_with_limits(input, limits)
            }
        }),
        "serialize.comma-list" => {
            let items = input.split('|').collect::<Vec<_>>();
            lexical(|limits| {
                if limits == CssSpecifiedValueSerializationLimits::default() {
                    serialize_css_comma_separated_list(&items)
                } else {
                    serialize_css_comma_separated_list_with_limits(&items, limits)
                }
            })
        }
        "serialize.whitespace-list" => {
            let items = input.split('|').collect::<Vec<_>>();
            lexical(|limits| {
                if limits == CssSpecifiedValueSerializationLimits::default() {
                    serialize_css_whitespace_separated_list(&items)
                } else {
                    serialize_css_whitespace_separated_list_with_limits(&items, limits)
                }
            })
        }
        "serialize.font-face-family-list" => {
            let families = input
                .split('|')
                .map(CssFontFaceFamily::try_new)
                .collect::<Option<Vec<_>>>();
            match families {
                None => Observation::shaped("family:none"),
                Some(families) => lexical(|limits| {
                    if limits == CssSpecifiedValueSerializationLimits::default() {
                        serialize_font_face_family_list(&families)
                    } else {
                        serialize_font_face_family_list_with_limits(&families, limits)
                    }
                }),
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
        sheet.to_specified_css_with_limits(limits)
    });
    serialization(observation, "cssom", Limited::Values, |limits| {
        sheet.serialize_cssom_with_limits(limits)
    });
}

fn retained_rules(observation: &mut Observation, rules: &[CssRule]) {
    let retained = Retained::rules(rules);
    observation.retained = nonempty(retained.identities.join("~"));
    observation.authored = nonempty(retained.authored.join("~"));
}

fn retained_scoped(observation: &mut Observation, rules: &[CssScopedRule]) {
    observation.add_fact(format!("rules={}", rules.len()));
    observation.retained = nonempty(Retained::scoped_rules(rules).identities.join("~"));
}

fn rule_serializations(observation: &mut Observation, rules: &[CssRule]) {
    serialization(observation, "specified", Limited::Values, |limits| {
        joined(rules, |rule| rule.to_specified_css_with_limits(limits))
    });
    serialization(observation, "cssom", Limited::Values, |limits| {
        joined(rules, |rule| rule.serialize_cssom_with_limits(limits))
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
    match decode_stylesheet_bytes(input.as_bytes(), hints) {
        Ok(decoded) => {
            let report = parse_decoded_stylesheet(&decoded, None);
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
    match context.parse_sheet_utf16_with_limits(utf16(input), limits, || Ok::<(), Stop>(())) {
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
    match context.parse_declaration_block_contents_utf16_with_limits(
        utf16(input),
        input_limits,
        component_limits,
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
    match context
        .parse_media_query_list_utf16_with_limits(utf16(input), limits, || Ok::<(), Stop>(()))
    {
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
        syntax: CssComponentValueLimits::default(),
        max_work: 4_000_000,
        max_live_bytes: 16 * 1024 * 1024,
        max_retained_bytes: 16 * 1024 * 1024,
    };
    match context.parse_color_scheme_utf16_with_limits(
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
        list.to_specified_css_with_limits(limits)
    });
    serialization(observation, "cssom", Limited::Values, |limits| {
        list.serialize_cssom_with_limits(limits)
    });
}

fn page_declarations(observation: &mut Observation, block: &CssPageDeclarationBlock) {
    serialization(observation, "cssom", Limited::Values, |limits| {
        block.serialize_cssom_with_limits(limits)
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
        &via!(
            context,
            parse_property_value_text(input, property.name(), property.importance)
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
            "ordinary" => via!(
                context,
                parse_property_value(property.name(), values, property.importance)
            ),
            "page" => via!(
                context,
                parse_page_property_value(property.name(), values, property.importance)
            ),
            "margin" => parse_margin_property_value(property.name(), values, property.importance),
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
    match parse_component_values(input) {
        Ok(values) => front(values),
        Err(error) => {
            let mut observation = Observation::shaped("components:err");
            observation.add_fact(super::error_text(&error));
            observation
        }
    }
}

fn checked_declaration(result: Result<CssDeclaration, CssPropertyValueParseError>) -> Observation {
    checked(result, declaration)
}

fn checked_declaration_ref(
    result: &Result<CssDeclaration, CssPropertyValueSyntaxError>,
) -> Observation {
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
        declaration.to_specified_css_with_limits(limits)
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
    observation.specified =
        super::serialized(query.serialize().map(|value| value.as_css().to_owned()));
    serialization(observation, "cssom", Limited::Values, |limits| {
        query
            .serialize_cssom_with_limits(limits)
            .map(|value| value.as_css().to_owned())
    });
    observation.add_fact(format!("guaranteed-false={}", query.is_guaranteed_false()));
}

fn media_query_list(observation: &mut Observation, list: &CssMediaQueryList) {
    observation.specified =
        super::serialized(list.serialize().map(|value| value.as_css().to_owned()));
    serialization(observation, "cssom", Limited::Values, |limits| {
        list.serialize_cssom_with_limits(limits)
            .map(|value| value.as_css().to_owned())
    });
    observation.add_fact(format!("queries={}", list.queries().len()));
}

fn supports_condition(observation: &mut Observation, condition: &CssSupportsCondition) {
    serialization(observation, "specified", Limited::Bytes, |limits| {
        condition
            .serialize_with_limit(limits.max_css_bytes())
            .map(|value| value.as_css().to_owned())
    });
}

fn when_condition(observation: &mut Observation, condition: &CssWhenCondition) {
    serialization(observation, "specified", Limited::Bytes, |limits| {
        condition
            .serialize_with_limit(limits.max_css_bytes())
            .map(|value| value.as_css().to_owned())
    });
}

fn supports_test_child(observation: &mut Observation, child: &CssSupportsTestChild) {
    serialization(observation, "specified", Limited::Values, |limits| {
        child
            .as_ref()
            .to_specified_css_with_graph_limits(CssRuleGraphSerializationLimits {
                values: limits,
                ..CssRuleGraphSerializationLimits::default()
            })
    });
}

fn supports_test_body(observation: &mut Observation, body: &CssSupportsTestBody) {
    serialization(observation, "specified", Limited::Values, |limits| {
        body.to_specified_css_with_limits(limits)
    });
    serialization(observation, "cssom", Limited::Bytes, |limits| {
        body.serialize_with_limit(limits.max_css_bytes())
            .map(|value| value.as_css().to_owned())
    });
}

fn keyframe_block(observation: &mut Observation, block: &CssKeyframeBlock) {
    serialization(observation, "specified", Limited::Values, |limits| {
        block.selectors().serialize_key_text_with_limits(limits)
    });
    serialization(observation, "cssom", Limited::Values, |limits| {
        block.declarations().serialize_cssom_with_limits(limits)
    });
}

fn margin_rule(observation: &mut Observation, rule: &CssMarginRule) {
    serialization(observation, "cssom", Limited::Values, |limits| {
        rule.serialize_cssom_with_limits(limits)
    });
}

fn family_list(observation: &mut Observation, families: &CssFontFeatureValuesFamilyList) {
    serialization(observation, "specified", Limited::Values, |limits| {
        families.serialize_specified_with_limits(limits)
    });
}

fn component_value(observation: &mut Observation, value: &CssComponentValue) {
    match CssComponentValues::try_new(vec![value.clone()]) {
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
            .serialize_with_limit(limits.max_css_bytes())
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
                        .serialize_with_limit(limits.max_css_bytes())
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
    serialize: impl Fn(
        CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError>,
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
            ("report", None) => normalize_report(report).map(|normalized| {
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
            ("report", Some(limits)) => {
                normalize_report_with_limits(report, limits).map(|normalized| {
                    normalized
                        .syntax()
                        .items()
                        .iter()
                        .map(item_facts)
                        .collect::<Vec<_>>()
                })
            }
            ("sheet", None) => normalize_sheet(report.syntax())
                .map(|sheet| sheet.items().iter().map(item_facts).collect::<Vec<_>>()),
            ("sheet", Some(limits)) => normalize_sheet_with_limits(report.syntax(), limits)
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
    let block =
        match CssSpecifiedDeclarationBlock::try_from_expansions(std::slice::from_ref(expansion)) {
            Ok(block) => super::serialized(block.serialize_cssom()),
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

/// The rule graph of each parsed rule, and of the first style rule rebuilt
/// from its selectors and specified declarations.
fn rule_graph(input: &str) -> Observation {
    let report = parse_sheet(input);
    let mut observation = Observation::shaped("report");
    observation.report(&report);
    let namespaces = CssNamespaceContext::from_sheet(report.syntax());
    let rules = report.syntax().rules();
    serialization(&mut observation, "cssom", Limited::Values, |limits| {
        joined(rules, |rule| {
            CssRuleGraphInput::from_rule(rule).serialize_cssom_with_limits(
                &namespaces,
                CssStyleAncestor::Absent,
                limits,
            )
        })
    });
    observation.add_fact(format!(
        "graph-limits={}",
        super::serialized(joined(rules, |rule| {
            CssRuleGraphInput::from_rule(rule).serialize_cssom_with_graph_limits(
                &namespaces,
                CssStyleAncestor::Present,
                CssRuleGraphSerializationLimits::default(),
            )
        }))
    ));
    if let Some(CssRule::Style(style)) = rules.first() {
        match CssSpecifiedDeclarationBlock::try_from_declarations(style.declarations()) {
            Ok(block) => {
                observation.add_fact(format!(
                    "style-input={}",
                    super::serialized(
                        CssRuleGraphInput::style(style.selectors(), &block, &[])
                            .serialize_cssom(&namespaces, CssStyleAncestor::Absent)
                    )
                ));
                observation.add_fact(format!(
                    "nested-declarations-input={}",
                    super::serialized(
                        CssRuleGraphInput::nested_declarations(&block)
                            .serialize_cssom(&namespaces, CssStyleAncestor::Present)
                    )
                ));
            }
            Err(error) => observation.add_fact(format!("block-err={}", super::error_text(&error))),
        }
    }
    observation
}

/// Edited-rule views of each parsed rule, and of the first style rule rebuilt
/// from its selectors and specified declarations.
fn edited_rule(input: &str) -> Observation {
    let report = parse_sheet(input);
    let mut observation = Observation::shaped("report");
    observation.report(&report);
    let rules = report.syntax().rules();
    serialization(&mut observation, "specified", Limited::Values, |limits| {
        joined(rules, |rule| {
            CssEditedRuleView::from_rule(rule).to_specified_css_with_limits(limits)
        })
    });
    serialization(&mut observation, "cssom", Limited::Values, |limits| {
        joined(rules, |rule| {
            CssEditedRuleView::from_rule(rule).serialize_cssom_with_limits(limits)
        })
    });
    if let Some(CssRule::Style(style)) = rules.first() {
        match CssSpecifiedDeclarationBlock::try_from_declarations(style.declarations()) {
            Ok(block) => {
                observation.add_fact(format!(
                    "edited-style={}",
                    match CssEditedRuleView::try_style(style.selectors(), &block, &[]) {
                        Ok(view) => super::serialized(view.to_specified_css()),
                        Err(error) => format!("err:{}", super::error_text(&error)),
                    }
                ));
                observation.add_fact(format!(
                    "edited-nested-declarations={}",
                    match CssEditedRuleView::try_nested_declarations(&block) {
                        Ok(view) => super::serialized(view.serialize_cssom()),
                        Err(error) => format!("err:{}", super::error_text(&error)),
                    }
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
