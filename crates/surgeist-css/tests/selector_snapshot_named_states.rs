#![forbid(unsafe_code)]
//! Literal authored expectations from Selectors 4, 2026-01-22, sections 3.5,
//! 5.4, 8.1, 10, 11.1, 11.5, 12.2.1 and 12.3.4. Matching is downstream.
//! Existing public boundaries keep these tests executable before new enum arms.
use surgeist_css::*;

const NAMES: [&str; 14] = [
    "defined",
    "any-link",
    "playing",
    "paused",
    "seeking",
    "buffering",
    "stalled",
    "muted",
    "volume-locked",
    "open",
    "picture-in-picture",
    "unchecked",
    "user-valid",
    "user-invalid",
];

fn parsed(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source:?}: {report:?}");
    report
        .into_validation_result()
        .unwrap()
        .expect("one selector")
}

fn exact(source: &str, expected: &str) -> CssSelector {
    let selector = parsed(source);
    let before = selector.clone();
    assert_eq!(selector.to_specified_css().unwrap(), expected, "{source:?}");
    assert_eq!(
        selector, before,
        "specified output preserves authored identity"
    );
    assert_eq!(
        parsed(expected),
        selector,
        "canonical spelling keeps identity"
    );
    selector
}

fn named(source: &str) -> CssPseudoClass {
    let CssSelector::PseudoClass(pseudo) = parsed(source) else {
        panic!("one symbolic named pseudo-class, without lowering: {source}");
    };
    assert!(!pseudo.has_pseudo_elements());
    pseudo
}

fn reject(source: &str) {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(
        report.syntax().is_none(),
        "no rejected graph: {source}: {report:?}"
    );
    assert!(!report.is_clean());
    let diagnostic = report.diagnostics().last().unwrap();
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    assert!(
        !report
            .diagnostics()
            .iter()
            .any(|d| { d.action() == CssRecoveryAction::RetainWithImplicitClosure })
    );
    let diagnostics = report.diagnostics().to_vec();
    assert_eq!(
        report.into_validation_result().unwrap_err().diagnostics(),
        diagnostics
    );
}

fn decoded_named_contract(name: &str, escaped: &str) {
    let canonical = format!(":{name}");
    let identity = named(&canonical);
    assert_eq!(
        exact(&canonical, &canonical),
        CssSelector::PseudoClass(identity.clone())
    );
    assert_eq!(
        exact(&format!(":{}", name.to_ascii_uppercase()), &canonical),
        CssSelector::PseudoClass(identity.clone())
    );
    assert_eq!(
        exact(escaped, &canonical),
        CssSelector::PseudoClass(identity.clone())
    );
    for control in [
        ":link",
        ":visited",
        ":checked",
        ":indeterminate",
        ":valid",
        ":invalid",
        ":modal",
        ":fullscreen",
        ":popover-open",
    ] {
        assert_ne!(
            identity,
            named(control),
            "{canonical} remains distinct from {control}"
        );
    }
    let checked = CssCompoundSelector::try_new(vec![
        CssSimpleSelector::Class(CssIdent::try_new("MiXeD").unwrap()),
        CssSimpleSelector::PseudoClass(identity.clone()),
        CssSimpleSelector::PseudoClass(identity.clone()),
    ])
    .unwrap();
    assert_eq!(checked.classes(), ["MiXeD"]);
    assert_eq!(checked.pseudo_classes(), [identity.clone(), identity]);
    assert_eq!(
        CssSelector::Compound(checked).to_specified_css().unwrap(),
        format!(".MiXeD{canonical}{canonical}")
    );
}

macro_rules! named_case {
    ($test:ident, $name:literal, $escaped:literal) => {
        #[test]
        fn $test() {
            decoded_named_contract($name, $escaped);
        }
    };
}

named_case!(
    defined_decodes_to_one_canonical_symbol,
    "defined",
    r":\44 EFINED"
);
named_case!(
    any_link_decodes_without_lowering,
    "any-link",
    r":\41 NY\2d LINK"
);
named_case!(
    playing_decodes_to_one_canonical_symbol,
    "playing",
    r":\50 LAYING"
);
named_case!(
    paused_decodes_to_one_canonical_symbol,
    "paused",
    r":\50 AUSED"
);
named_case!(
    seeking_decodes_to_one_canonical_symbol,
    "seeking",
    r":\53 EEKING"
);
named_case!(
    buffering_decodes_to_one_canonical_symbol,
    "buffering",
    r":\42 UFFERING"
);
named_case!(
    stalled_decodes_to_one_canonical_symbol,
    "stalled",
    r":\53 TALLED"
);
named_case!(muted_decodes_to_one_canonical_symbol, "muted", r":\4d UTED");
named_case!(
    volume_locked_decodes_to_one_canonical_symbol,
    "volume-locked",
    r":\56 OLUME\2d LOCKED"
);
named_case!(open_decodes_to_one_canonical_symbol, "open", r":\4f PEN");
named_case!(
    picture_in_picture_decodes_to_one_canonical_symbol,
    "picture-in-picture",
    r":\50 ICTURE\2d IN\2d PICTURE"
);
named_case!(
    unchecked_decodes_to_one_canonical_symbol,
    "unchecked",
    r":\55 NCHECKED"
);
named_case!(
    user_valid_decodes_to_one_canonical_symbol,
    "user-valid",
    r":\55 SER\2d VALID"
);
named_case!(
    user_invalid_decodes_to_one_canonical_symbol,
    "user-invalid",
    r":\55 SER\2d INVALID"
);

#[test]
fn named_states_are_pairwise_distinct_even_when_matching_states_overlap() {
    let identities: Vec<_> = NAMES
        .iter()
        .map(|name| named(&format!(":{name}")))
        .collect();
    for (index, identity) in identities.iter().enumerate() {
        for (other_index, other) in identities.iter().enumerate().skip(index + 1) {
            assert_ne!(
                identity, other,
                "{} and {}",
                NAMES[index], NAMES[other_index]
            );
        }
    }
}

#[test]
fn functional_nearby_unknown_and_non_ascii_folded_names_are_rejected() {
    for name in NAMES {
        for source in [
            format!(":{name}()"),
            format!(":{name}(x)"),
            format!(":{name}("),
            format!(":{name}-extra"),
            format!(":x-{name}"),
            format!("::{name}"),
            format!(": {name}"),
        ] {
            reject(&source);
        }
    }
    for source in [
        ":deﬁned",
        ":playıng",
        ":uſer-valid",
        ":open-state",
        ":un-checked",
    ] {
        reject(source);
    }
}

#[test]
fn every_named_state_composes_with_ordinary_function_and_shadow_arguments() {
    for name in NAMES {
        for (source, expected) in [
            (
                format!("Leaf#ID.Class[Data=MiX I]:{name}"),
                format!("Leaf#ID.Class[Data=\"MiX\" i]:{name}"),
            ),
            (format!(".Parent > :{name}"), format!(".Parent > :{name}")),
            (
                format!(":is(:{name},.Other)"),
                format!(":is(:{name}, .Other)"),
            ),
            (
                format!(":where(:{name},.Other)"),
                format!(":where(:{name}, .Other)"),
            ),
            (
                format!(":not(:{name},.Other)"),
                format!(":not(:{name}, .Other)"),
            ),
            (
                format!(":has(> :{name},+ .Other)"),
                format!(":has(> :{name}, + .Other)"),
            ),
            (
                format!(":nth-child(2n of :{name},.Other)"),
                format!(":nth-child(2n of :{name}, .Other)"),
            ),
            (
                format!(":nth-last-child(2n of :{name})"),
                format!(":nth-last-child(2n of :{name})"),
            ),
            (format!(":host(:{name})"), format!(":host(:{name})")),
            (
                format!(":host-context(:{name})"),
                format!(":host-context(:{name})"),
            ),
            (
                format!("slot::slotted(:{name})"),
                format!("slot::slotted(:{name})"),
            ),
        ] {
            exact(&source, &expected);
        }
        let source = format!(":{name},.Other");
        let report = parse_selector_list(&source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {report:?}");
        let list = report.into_validation_result().unwrap().unwrap();
        assert_eq!(list.selectors()[0].selector(), &parsed(&format!(":{name}")));
        assert_eq!(
            list.selectors()[1].selector(),
            &CssSelector::Class("Other".into())
        );
        let source = format!("> :{name},+ .Other");
        let report = parse_relative_selector_list(&source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {report:?}");
        let list = report.into_validation_result().unwrap().unwrap();
        assert_eq!(
            list.to_specified_css().unwrap(),
            format!("> :{name}, + .Other")
        );
        assert_eq!(
            CssRelativeSelectorList::try_new(list.selectors().to_vec()).unwrap(),
            list
        );
    }
}

#[test]
fn every_named_state_preserves_strict_atomic_and_forgiving_member_recovery() {
    for name in NAMES {
        for source in [
            format!(".Before,:{name},:future,.After"),
            format!(".Before,:not(:{name},:future),.After"),
            format!(".Before,:has(:{name},:future),.After"),
            format!(".Before,:nth-child(2n of :{name},:future),.After"),
        ] {
            let report = parse_selector_list(&source, &CssNamespaceContext::default());
            assert!(
                report.syntax().is_none(),
                "atomic rejection: {source}: {report:?}"
            );
            assert_eq!(
                report.diagnostics().last().unwrap().action(),
                CssRecoveryAction::RejectInput
            );
            assert!(report.into_validation_result().is_err());
        }
        for function in ["is", "where"] {
            let source = format!("/*😀*/\r\n:{function}(:{name},:future,.After)");
            let report = parse_selector(&source, &CssNamespaceContext::default());
            assert!(!report.is_clean());
            let selector = report
                .syntax()
                .as_ref()
                .expect("retained forgiving selector");
            assert_eq!(
                selector.to_specified_css().unwrap(),
                format!(":{function}(:{name}, .After)")
            );
            let [diagnostic] = report.diagnostics() else {
                panic!("only the unknown member drops: {report:?}");
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropSelectorListItem);
            assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
            let start = source.find(":future").unwrap();
            assert_eq!(diagnostic.span().start().byte_offset().value(), start);
            assert_eq!(
                diagnostic.span().end().byte_offset().value(),
                start + ":future".len()
            );
            assert_eq!(
                diagnostic.error().position().byte_offset().value(),
                start + 1
            );
            assert_eq!(diagnostic.error().position().line().value(), 1);
            assert_eq!(
                diagnostic.error().position().column().value(),
                u32::try_from(
                    source[source.find('\n').unwrap() + 1..start + 1]
                        .encode_utf16()
                        .count()
                )
                .unwrap()
            );
            let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
                panic!("typed selector error");
            };
            assert_eq!(detail.encountered().unwrap().authored(), "future");
            let diagnostics = report.diagnostics().to_vec();
            assert_eq!(
                report.into_validation_result().unwrap_err().diagnostics(),
                diagnostics
            );
        }
    }
}

#[test]
fn every_named_state_obeys_element_backed_and_restricted_suffix_admission() {
    for name in NAMES {
        let pseudo = named(&format!(":{name}"));
        for receiver in [
            "::part(Label)",
            "::file-selector-button",
            "::details-content",
        ] {
            exact(&format!("{receiver}:{name}"), &format!("{receiver}:{name}"));
            exact(
                &format!("{receiver}:is(:{name})"),
                &format!("{receiver}:is(:{name})"),
            );
        }
        for receiver in [
            "::before",
            "::after",
            "::marker",
            "::selection",
            "::search-text",
            "::view-transition-old(*)",
            "slot::slotted(.Cell)",
        ] {
            reject(&format!("{receiver}:{name}"));
            reject(&format!("{receiver}:not(:{name})"));
            let source = format!("{receiver}:is(:{name},:hover)");
            let report = parse_selector(&source, &CssNamespaceContext::default());
            assert_eq!(
                report
                    .syntax()
                    .as_ref()
                    .unwrap()
                    .to_specified_css()
                    .unwrap(),
                format!("{receiver}:is(:hover)")
            );
            assert_eq!(report.diagnostics().len(), 1);
            assert_eq!(
                report.diagnostics()[0].action(),
                CssRecoveryAction::DropSelectorListItem
            );
        }
        for element in [CssPseudoElement::Before, CssPseudoElement::SearchText] {
            assert!(
                CssPseudoElementSequence::try_from_segments(vec![
                    CssPseudoElementSegment::PseudoElement(element),
                    CssPseudoElementSegment::PseudoClass(pseudo.clone()),
                ])
                .is_none()
            );
        }
        for element in [
            CssPseudoElement::FileSelectorButton,
            CssPseudoElement::DetailsContent,
        ] {
            let sequence = CssPseudoElementSequence::try_from_segments(vec![
                CssPseudoElementSegment::PseudoElement(element),
                CssPseudoElementSegment::PseudoClass(pseudo.clone()),
            ])
            .unwrap();
            assert_eq!(sequence.segments().len(), 2);
        }
        exact(&format!(":{name}::before"), &format!(":{name}::before"));
    }
}

#[test]
fn every_named_state_reuses_rules_nesting_scope_supports_and_original_positions() {
    for name in NAMES {
        let source = format!("/*😀*/\r\n:{name}{{color:red; > :{name}{{color:blue}}}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert_eq!(validate_sheet(&source).unwrap(), *report.syntax());
        let [CssRule::Style(rule)] = report.syntax().rules() else {
            panic!("named style rule");
        };
        assert_eq!(
            rule.selectors().selectors()[0].selector(),
            &parsed(&format!(":{name}"))
        );
        assert_eq!(
            rule.position().byte_offset().value(),
            source.find(':').unwrap()
        );
        assert_eq!(rule.position().line().value(), 1);
        assert_eq!(rule.position().column().value(), 0);
        let [CssRule::Style(child)] = rule.rules() else {
            panic!("nested rule");
        };
        let [CssStyleSelector::Relative(relative)] = child.selectors().selectors() else {
            panic!("symbolic relative child");
        };
        assert_eq!(relative.combinator(), CssSelectorCombinator::Child);
        assert_eq!(relative.selector(), &parsed(&format!(":{name}")));
        let source = format!("selector(:{})", name.to_ascii_uppercase());
        let components = parse_component_values(&source).unwrap();
        let original = components.clone();
        let condition =
            CssSupportsCondition::try_from_components(components, &CssNamespaceContext::default())
                .unwrap();
        let CssSupportsConditionKind::Selector(selector) = condition.kind() else {
            panic!("typed Supports selector: {condition:?}");
        };
        assert_eq!(selector.to_specified_css().unwrap(), format!(":{name}"));
        assert_eq!(condition.components(), original.items());
        assert_eq!(condition.serialize().unwrap().as_css(), source);
        let source = format!(
            "@scope (:{name}) to (:{name}) {{ :{name}{{}} }} @supports selector(:{name}) {{ :{name}{{}} }}"
        );
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert!(matches!(
            report.syntax().rules(),
            [CssRule::Scope(_), CssRule::Supports(_)]
        ));
        let CssRule::Supports(supports) = &report.syntax().rules()[1] else {
            unreachable!()
        };
        assert!(matches!(
            supports.condition().kind(),
            CssSupportsConditionKind::Selector(_)
        ));
    }
}

#[test]
fn named_rule_rejection_drops_only_its_complete_recovery_unit() {
    for name in NAMES {
        let rejected = format!(":{name}(){{color:red; .Child{{}}}}");
        let source = format!(".Before{{}}{rejected}.After{{}}");
        let report = parse_sheet(&source);
        let [CssRule::Style(before), CssRule::Style(after)] = report.syntax().rules() else {
            panic!("only neighboring rules retained: {report:?}");
        };
        assert_eq!(
            before.selectors().selectors()[0].selector(),
            &CssSelector::Class("Before".into())
        );
        assert_eq!(
            after.selectors().selectors()[0].selector(),
            &CssSelector::Class("After".into())
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one complete rule rejection");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            ".Before{}".len()
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            ".Before{}".len() + rejected.len()
        );
        assert!(validate_sheet(&source).is_err());
    }
}

#[test]
fn named_identity_and_lists_charge_cumulative_nodes_bytes_and_atomic_retry() {
    for name in NAMES {
        let literal = format!(":{name}");
        let selector = parsed(&literal);
        let before = selector.clone();
        assert_eq!(
            selector
                .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    literal.len()
                ))
                .unwrap(),
            literal
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, literal.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, literal.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, literal.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                selector
                    .to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(selector, before);
        }
        let members = vec![selector.clone(), selector];
        let output = format!("{literal}, {literal}");
        let list = CssSelectorList::try_new_with_limits(
            members.clone(),
            CssSpecifiedValueSerializationLimits::new(3, 3, output.len()),
        )
        .unwrap();
        assert_eq!(list.to_specified_css().unwrap(), output);
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(2, 3, output.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(3, 2, output.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(3, 3, output.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            let error = CssSelectorList::try_new_with_limits(members.clone(), limits).unwrap_err();
            let CssSelectorConstructionErrorKind::Specified(cause) = error.kind() else {
                panic!("typed resource error");
            };
            assert_eq!(cause.kind(), kind);
            assert_eq!(
                list.to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        assert_eq!(CssSelectorList::try_new(members).unwrap(), list);
        assert_eq!(list.to_specified_css().unwrap(), output);
    }
}

#[test]
fn checked_named_graphs_preserve_each_function_attachment_and_complex_relation() {
    for name in NAMES {
        let selector = parsed(&format!(":{name}"));
        let list = CssPseudoSelectorList::try_new(vec![selector.clone()]).unwrap();
        let argument = CssCompoundSelectorArgument::try_new(selector.clone()).unwrap();
        for (pseudo, function) in [
            (CssPseudoClass::Not(list.clone()), "not"),
            (CssPseudoClass::Is(list.clone()), "is"),
            (CssPseudoClass::Where(list.clone()), "where"),
            (CssPseudoClass::HostFunction(argument.clone()), "host"),
            (CssPseudoClass::HostContext(argument), "host-context"),
        ] {
            let graph = CssSelector::PseudoClass(pseudo);
            let expected = format!(":{function}(:{name})");
            assert_eq!(graph.to_specified_css().unwrap(), expected);
            assert_eq!(parsed(&expected), graph);
            assert_eq!(
                CssSelectorList::try_new(vec![graph])
                    .unwrap()
                    .to_specified_css()
                    .unwrap(),
                expected
            );
        }
        let relative = CssRelativeSelectorList::try_new(vec![CssRelativeSelector::new(
            CssSelectorCombinator::Child,
            selector.clone(),
        )])
        .unwrap();
        let graph = CssSelector::PseudoClass(CssPseudoClass::Has(relative));
        assert_eq!(
            graph.to_specified_css().unwrap(),
            format!(":has(> :{name})")
        );
        let graph = CssSelector::PseudoClass(CssPseudoClass::NthChild(CssNthChildPattern::new(
            CssNthPattern::Even,
            Some(list),
        )));
        assert_eq!(
            graph.to_specified_css().unwrap(),
            format!(":nth-child(2n of :{name})")
        );
        let CssSelector::PseudoClass(pseudo) = selector else {
            unreachable!()
        };
        let first = CssCompoundSelector::try_new(vec![CssSimpleSelector::Class(
            CssIdent::try_new("Parent").unwrap(),
        )])
        .unwrap();
        let child =
            CssCompoundSelector::try_new(vec![CssSimpleSelector::PseudoClass(pseudo)]).unwrap();
        let part = CssComplexSelectorPart::try_new(CssSelectorCombinator::Child, child).unwrap();
        let graph = CssSelector::Complex(CssComplexSelector::try_new(first, vec![part]).unwrap());
        assert_eq!(
            graph.to_specified_css().unwrap(),
            format!(".Parent > :{name}")
        );
        assert_eq!(parsed(&format!(".Parent > :{name}")), graph);
    }
}

#[test]
fn named_leaves_preserve_eof_recovery_and_the_existing_function_depth_boundary() {
    for name in NAMES {
        let source = format!(":is(:{name}");
        let report = parse_selector(&source, &CssNamespaceContext::default());
        assert_eq!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .to_specified_css()
                .unwrap(),
            format!(":is(:{name})")
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one retained EOF closure");
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            source.len()
        );
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
        assert!(report.into_validation_result().is_err());
        let source = format!("{}:{name}{}", ":is(".repeat(256), ")".repeat(256));
        exact(&source, &source);
        let source = format!("{}:{name}{}", ":is(".repeat(257), ")".repeat(257));
        let report = parse_selector(&source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none());
        let [diagnostic] = report.diagnostics() else {
            panic!("one nesting stop");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
        assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn existing_named_state_controls_keep_checked_identity_and_canonical_output() {
    for (source, expected, pseudo) in [
        (r":\48 OVER", ":hover", CssPseudoClass::Hover),
        (":CHECKED", ":checked", CssPseudoClass::Checked),
        (":VALID", ":valid", CssPseudoClass::Valid),
        (":LINK", ":link", CssPseudoClass::Link),
        (":VISITED", ":visited", CssPseudoClass::Visited),
    ] {
        assert_eq!(exact(source, expected), CssSelector::PseudoClass(pseudo));
    }
}
