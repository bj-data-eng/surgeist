use super::*;

fn spare(text: &str, requested: usize) -> String {
    let mut value = String::with_capacity(requested);
    value.push_str(text);
    value
}
fn measured(kind: &ErrorKind) -> usize {
    let mut observation = DiagnosticBacking {
        admit: &mut || Ok::<_, ()>(()),
        bytes: 0,
    };
    observation.kind(kind).unwrap();
    observation.bytes
}
#[test]
fn descriptor_boxes_conflict_vec_and_actual_spare_spelling_are_counted() {
    let at_rule = spare("font-face", 81);
    let descriptor = spare("src", 93);
    let token = spare("!", 117);
    let expected = at_rule.capacity()
        + std::mem::size_of::<CssDescriptorName>()
        + descriptor.capacity()
        + token.capacity();
    let kind = ErrorKind::InvalidDescriptorValue(CssDescriptorValueError {
        origin: DiagnosticOrigin::Token,
        at_rule: CssAtRuleName(at_rule),
        descriptor: Box::new(CssDescriptorName(descriptor)),
        expectation: EXPECT_DESCRIPTOR_VALUE,
        encountered: Some(CssTokenSummary {
            kind: CssTokenKind::Delim,
            authored: token,
        }),
    });
    assert_eq!(measured(&kind), expected);
    let cloned = kind.clone();
    let ErrorKind::InvalidDescriptorValue(value) = &cloned else {
        unreachable!()
    };
    let clone_expected = value.at_rule.0.capacity()
        + std::mem::size_of::<CssDescriptorName>()
        + value.descriptor.0.capacity()
        + value.encountered.as_ref().unwrap().authored.capacity();
    assert_eq!(cloned, kind);
    assert_eq!(measured(&cloned), clone_expected);

    let at_rule = spare("counter-style", 65);
    let responsible = spare("system", 73);
    let first = spare("symbols", 89);
    let second = spare("additive-symbols", 113);
    let mut conflicting = Vec::with_capacity(19);
    let expected = at_rule.capacity()
        + responsible.capacity()
        + first.capacity()
        + second.capacity()
        + conflicting.capacity() * std::mem::size_of::<CssDescriptorName>();
    conflicting.push(CssDescriptorName(first));
    conflicting.push(CssDescriptorName(second));
    assert_eq!(
        measured(&ErrorKind::InvalidDescriptorCombination(
            CssDescriptorCombinationError {
                at_rule: CssAtRuleName(at_rule),
                responsible: CssDescriptorName(responsible),
                conflicting,
            }
        )),
        expected
    );
}
#[test]
fn static_variants_and_nested_component_error_exclude_sources_and_inline_headers() {
    assert_eq!(
        measured(&ErrorKind::EscapeParseError(CssEscapeFailure::EndOfInput)),
        0
    );
    assert_eq!(
        measured(&ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: EXPECT_CSS_SYNTAX
        })),
        0
    );
    assert_eq!(
        measured(&ErrorKind::NestingLimit(CssNestingLimitError {
            limit: 7,
            enclosing_production: QUALIFIED_RULE
        })),
        0
    );
    let component = crate::CssComponentValueError::new(
        crate::CssComponentValueErrorKind::CapacityOverflow,
        crate::CssValueOrigin::Programmatic,
    );
    assert_eq!(
        measured(&ErrorKind::InvalidComponentValue(Box::new(component))),
        std::mem::size_of::<crate::CssComponentValueError>()
    );
}
#[test]
fn cancellation_and_each_one_short_observation_refuse_without_false_total() {
    let kind = ErrorKind::InvalidDescriptorCombination(CssDescriptorCombinationError {
        at_rule: CssAtRuleName(spare("counter-style", 64)),
        responsible: CssDescriptorName(spare("system", 72)),
        conflicting: vec![
            CssDescriptorName(spare("symbols", 88)),
            CssDescriptorName(spare("range", 96)),
        ],
    });
    let error = CssError {
        kind,
        position: CssSourcePosition::from_byte_offset_in("", 0),
    };
    let mut steps = 0;
    let expected = error
        .retained_backing_bytes_with(&mut || {
            steps += 1;
            Ok::<_, usize>(())
        })
        .unwrap();
    assert!(expected > 0);
    for allowance in 0..steps {
        let mut reached = 0;
        assert_eq!(
            error.retained_backing_bytes_with(&mut || {
                if reached == allowance {
                    return Err(reached);
                }
                reached += 1;
                Ok(())
            }),
            Err(CssRetainedPayloadError::admission(allowance))
        );
        assert_eq!(reached, allowance);
    }
    let mut reached = 0;
    assert_eq!(
        error.retained_backing_bytes_with(&mut || {
            reached += 1;
            Ok::<_, usize>(())
        }),
        Ok(expected)
    );
    assert_eq!(reached, steps);
}
#[test]
fn all_spelling_families_include_authored_capacity() {
    let name = || CssAtRuleName(spare("unknown", 47));
    let token = || CssTokenSummary {
        kind: CssTokenKind::Ident,
        authored: spare("word", 59),
    };
    let cases = [
        (
            ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
                expectation: EXPECT_CSS_SYNTAX,
                encountered: token(),
            }),
            59,
        ),
        (
            ErrorKind::InvalidAtRulePlacement(CssAtRulePlacementError {
                name: name(),
                expected_context: EXPECT_CSS_SYNTAX,
            }),
            47,
        ),
        (
            ErrorKind::InvalidAtRulePrelude(CssAtRuleSyntaxError {
                name: name(),
                production: QUALIFIED_RULE,
                expectation: EXPECT_CSS_SYNTAX,
                encountered: Some(token()),
            }),
            106,
        ),
        (
            ErrorKind::InvalidAtRuleBody(CssAtRuleSyntaxError {
                name: name(),
                production: QUALIFIED_RULE,
                expectation: EXPECT_CSS_SYNTAX,
                encountered: Some(token()),
            }),
            106,
        ),
        (
            ErrorKind::LegacyNavigationTarget(CssLegacyNavigationTargetError {
                target: spare("_legacy", 47),
            }),
            47,
        ),
        (
            ErrorKind::UnknownAtRule(CssUnknownAtRuleError { name: name() }),
            47,
        ),
        (
            ErrorKind::UnsupportedAtRule(CssUnsupportedAtRuleError {
                name: name(),
                feature: CssFeatureId::new("static"),
            }),
            47,
        ),
        (
            ErrorKind::InvalidQualifiedRule(CssQualifiedRuleError {
                production: QUALIFIED_RULE,
                expectation: EXPECT_CSS_SYNTAX,
                encountered: Some(token()),
            }),
            59,
        ),
        (
            ErrorKind::InvalidSelector(CssSelectorError {
                production: None,
                expectation: EXPECT_SELECTOR,
                encountered: Some(token()),
            }),
            59,
        ),
        (
            ErrorKind::InvalidMediaQuery(CssMediaQueryError {
                committed_context: false,
                feature: Some(CssMediaFeatureName(spare("width", 47))),
                expectation: EXPECT_MEDIA_QUERY,
                encountered: Some(token()),
            }),
            106,
        ),
        (
            ErrorKind::UnknownProperty(CssUnknownPropertyError {
                name: CssPropertyName(spare("unknown", 47)),
            }),
            47,
        ),
        (
            ErrorKind::UnsupportedProperty(CssUnsupportedPropertyError {
                name: CssPropertyName(spare("unknown", 47)),
                feature: CssFeatureId::new("static"),
            }),
            47,
        ),
        (
            ErrorKind::InvalidPropertyValue(PropertyValueRejection {
                property: CssKnownProperty::Width,
                expectation: EXPECT_PROPERTY_VALUE,
                encountered: Some(token()),
            }),
            59,
        ),
        (
            ErrorKind::UnknownDescriptor(CssUnknownDescriptorError {
                at_rule: name(),
                descriptor: CssDescriptorName(spare("x", 59)),
            }),
            106,
        ),
        (
            ErrorKind::UnsupportedDescriptor(CssUnsupportedDescriptorError {
                at_rule: name(),
                descriptor: CssDescriptorName(spare("x", 59)),
                feature: CssFeatureId::new("static"),
            }),
            106,
        ),
        (
            ErrorKind::InvalidColorSyntax(CssColorSyntaxError {
                component: Some(CssColorComponentName(spare("red", 47))),
                expectation: EXPECT_COLOR,
                encountered: Some(token()),
            }),
            106,
        ),
    ];
    for (kind, expected) in cases {
        assert_eq!(measured(&kind), expected, "{kind:?}");
    }
}
#[test]
fn namespace_and_annotation_contexts_measure_owner_spelling_and_tokens() {
    let spelling = spare("prefix", 97);
    let bytes = spelling.capacity();
    let prefix = crate::CssNamespacePrefix::try_new(spelling).unwrap();
    assert_eq!(
        measured(&ErrorKind::NamespaceRedeclaration(
            CssNamespaceRedeclarationError {
                prefix: Some(prefix),
                previous_position: CssSourcePosition::from_byte_offset_in("", 0),
            }
        )),
        bytes
    );
    let token = || CssTokenSummary {
        kind: CssTokenKind::Delim,
        authored: spare("!", 59),
    };
    let at_rule = spare("font-face", 71);
    let descriptor = spare("src", 83);
    let expected = at_rule.capacity() + descriptor.capacity() + 59;
    assert_eq!(
        measured(&ErrorKind::InvalidDeclarationAnnotation(
            CssDeclarationAnnotationError {
                context: CssDeclarationContext::Descriptor {
                    at_rule: CssAtRuleName(at_rule),
                    descriptor: CssDescriptorName(descriptor)
                },
                encountered: token(),
            }
        )),
        expected
    );
    for context in [
        CssDeclarationContext::SvgGlyph,
        CssDeclarationContext::KeyframeSvgGlyph,
        CssDeclarationContext::OrdinaryKnown(CssKnownProperty::Width),
        CssDeclarationContext::Keyframe(CssKnownProperty::Width),
    ] {
        assert_eq!(
            measured(&ErrorKind::InvalidDeclarationAnnotation(
                CssDeclarationAnnotationError {
                    context,
                    encountered: token()
                }
            )),
            59
        );
    }
}
#[test]
fn nested_component_origin_remains_in_existing_source_closure_separately() {
    let outer = crate::CssSourceSnapshot::new("outer");
    let nested = crate::CssSourceSnapshot::new("inner");
    let origin = crate::CssParsedOrigin::from_range(&nested, 0..5).unwrap();
    let error = CssError {
        kind: ErrorKind::InvalidComponentValue(Box::new(crate::CssComponentValueError::new(
            crate::CssComponentValueErrorKind::CapacityOverflow,
            crate::CssValueOrigin::Parsed(origin),
        ))),
        position: CssSourcePosition::from_byte_offset_in("outer", 0),
    };
    let span = crate::CssSourceSpan::new(error.position(), error.position()).unwrap();
    let mut diagnostic =
        crate::CssRecoveryDiagnostic::new(error, span, crate::CssRecoveryAction::DropDeclaration)
            .unwrap();
    diagnostic.bind_source(&outer);
    assert_eq!(
        diagnostic.retained_backing_bytes_with(|| Ok::<_, ()>(())),
        Ok(std::mem::size_of::<crate::CssComponentValueError>())
    );
    let mut actual = crate::CssSourceRetention::new(usize::MAX, 100, 10);
    actual.visit(&diagnostic).unwrap();
    let mut expected = crate::CssSourceRetention::new(usize::MAX, 100, 10);
    expected.source(&outer).unwrap();
    expected.source(&nested).unwrap();
    assert_eq!(
        actual.complete_payload_bytes_with(|| Ok::<_, ()>(())),
        expected.complete_payload_bytes_with(|| Ok::<_, ()>(()))
    );
}
#[test]
fn custom_property_annotation_spelling_is_owned_in_both_contexts() {
    for context in [
        CssDeclarationContext::OrdinaryCustom(
            CssCustomPropertyName::try_from_decoded("--custom").unwrap(),
        ),
        CssDeclarationContext::KeyframeCustom(
            CssCustomPropertyName::try_from_decoded("--custom").unwrap(),
        ),
    ] {
        assert_eq!(
            measured(&ErrorKind::InvalidDeclarationAnnotation(
                CssDeclarationAnnotationError {
                    context,
                    encountered: CssTokenSummary {
                        kind: CssTokenKind::Delim,
                        authored: "!".to_owned()
                    },
                }
            )),
            9
        );
    }
}
