//! Tests of crate-private construction and assembly.
//!
//! Each test here builds its subject through a crate-private constructor or
//! retained graph that external callers cannot reach; behavior reachable
//! through the public API is tested in `tests/css`.

use super::*;

fn source_position(line: u32, column: u32) -> CssSourcePosition {
    let source = format!(
        "{}{}",
        "\n".repeat(line as usize),
        " ".repeat(column as usize)
    );
    let mut input = cssparser::ParserInput::new(&source);
    let mut parser = cssparser::Parser::new(&mut input);
    while parser.next().is_ok() {}
    CssSourcePosition::from_cssparser(parser.position(), parser.current_source_location())
}

fn style_rule(rule: &CssRule) -> &CssStyleRule {
    match rule {
        CssRule::Style(rule) => rule,
        unexpected => panic!("expected style rule, got {unexpected:?}"),
    }
}

fn keyframes_rule(rule: &CssRule) -> &CssKeyframesRule {
    match rule {
        CssRule::Keyframes(rule) => rule,
        unexpected => panic!("expected keyframes rule, got {unexpected:?}"),
    }
}

/// The sole declaration of a clean `.test { name: value; }` sheet.
fn parse_single_declaration(property_name: &str, authored_value: &str) -> CssDeclaration {
    let input = format!(".test {{ {property_name}: {authored_value}; }}");
    let sheet = parse_sheet(&input).unwrap();
    let [rule] = sheet.rules() else {
        panic!("{input} should parse exactly one rule");
    };
    let [declaration] = style_rule(rule).declarations().as_slice() else {
        panic!("{input} should parse exactly one declaration");
    };
    declaration.clone()
}

#[test]
fn container_rule_accessors_expose_authored_structure() {
    let name = CssContainerName::try_new("sidebar").unwrap();
    let prelude = CssContainerPrelude::try_from_components(
        crate::parse_component_values("sidebar (inline-size > 30rem)").unwrap(),
    )
    .unwrap();
    let condition = prelude.entries()[0].query().unwrap().clone();
    let location = source_position(4, 9);
    let nested = CssRule::Style(CssStyleRule::new(
        CssStyleSelectorList::absolute(vec![CssSelector::Class("card".to_owned())]),
        CssDeclarationList::new(Vec::new()),
        Vec::new(),
        location,
    ));
    let rule = CssContainerRule::new(prelude.clone(), vec![nested.clone()], location);

    assert_eq!(style_rule(&nested).position(), location);
    assert_eq!(rule.prelude().entries()[0].name(), Some(&name));
    assert_eq!(rule.prelude().entries()[0].query().unwrap(), &condition);
    assert_eq!(rule.rules(), &[nested]);
    assert_eq!(rule.position(), Some(location));
    assert_eq!(CssRule::Container(rule.clone()), CssRule::Container(rule));
}

#[test]
fn keyframes_constructors_preserve_authored_empty_and_duplicate_states() {
    let location = source_position(1, 1);
    let name = CssKeyframesName::Ident(
        CssKeyframesIdent::try_new(CssCustomIdent::try_new("fade").unwrap()).unwrap(),
    );
    let from = CssKeyframeSelectorList::try_new(vec![CssKeyframeSelector::From]).unwrap();

    assert_eq!(CssKeyframesString::new("").as_str(), "");
    assert_eq!(CssKeyframesString::new("   ").as_str(), "   ");
    assert_eq!(CssKeyframePercent::try_new(-0.1), None);
    assert_eq!(CssKeyframePercent::try_new(100.1), None);
    assert_eq!(CssKeyframePercent::try_new(f64::NAN), None);
    assert_eq!(CssKeyframeSelectorList::try_new(Vec::new()), None);
    let duplicate_selectors = CssKeyframeSelectorList::try_new(vec![
        CssKeyframeSelector::From,
        CssKeyframeSelector::Percent(CssKeyframePercent::try_new(0.0).unwrap()),
    ])
    .unwrap();
    assert_eq!(duplicate_selectors.selectors().len(), 2);

    let duplicate_a = CssKeyframeBlock::new(
        from.clone(),
        CssKeyframeDeclarationList::new(Vec::new()),
        location,
    );
    let duplicate_b =
        CssKeyframeBlock::new(from, CssKeyframeDeclarationList::new(Vec::new()), location);
    let rule = CssKeyframesRule::new(name, vec![duplicate_a, duplicate_b], location);
    assert_eq!(rule.blocks().len(), 2);
    assert!(
        rule.blocks()
            .iter()
            .all(|block| block.declarations().is_empty())
    );

    let empty = CssKeyframesRule::new(
        CssKeyframesName::Ident(
            CssKeyframesIdent::try_new(CssCustomIdent::try_new("empty").unwrap()).unwrap(),
        ),
        Vec::new(),
        location,
    );
    assert!(empty.blocks().is_empty());
}

#[test]
fn keyframes_rule_accessors_expose_authored_structure() {
    let name = CssKeyframesName::Ident(
        CssKeyframesIdent::try_new(CssCustomIdent::try_new("fade").unwrap()).unwrap(),
    );
    let selector = CssKeyframeSelectorList::try_new(vec![CssKeyframeSelector::From]).unwrap();
    let keyframes = parse_sheet("@keyframes fade { from { opacity: 0; } }").unwrap();
    let declaration = keyframes_rule(&keyframes.rules()[0]).blocks()[0].declarations()[0].clone();
    let block = CssKeyframeBlock::new(
        selector,
        CssKeyframeDeclarationList::new(vec![declaration.clone()]),
        source_position(2, 3),
    );
    let rule = CssKeyframesRule::new(name, vec![block], source_position(1, 1));

    assert_eq!(
        rule.name(),
        &CssKeyframesName::Ident(
            CssKeyframesIdent::try_new(CssCustomIdent::try_new("fade").unwrap()).unwrap()
        )
    );
    assert_eq!(rule.position(), source_position(1, 1));
    let [block] = rule.blocks() else {
        panic!("expected one keyframe block");
    };
    assert_eq!(block.position(), source_position(2, 3));
    assert_eq!(block.selectors().selectors(), &[CssKeyframeSelector::From]);
    assert_eq!(block.declarations().as_slice(), &[declaration]);
    assert_eq!(
        CssKeyframeSelector::From.offset().literal_value(),
        Some(0.0)
    );
    assert_eq!(
        CssKeyframeSelector::To.offset().literal_value(),
        Some(100.0)
    );
}

#[test]
fn layer_rule_models_preserve_authored_statement_and_block_shapes() {
    let reset = CssLayerName::try_new(["reset"]).unwrap();
    let components = CssLayerName::try_new(["theme", "components"]).unwrap();
    let names = CssLayerNameList::try_new(vec![reset.clone(), components.clone()]).unwrap();
    let statement_location = source_position(2, 3);
    let statement = CssLayerStatementRule::new(names.clone(), statement_location);

    assert_eq!(CssLayerNameList::try_new(Vec::new()), None);
    assert_eq!(statement.names().names(), names.names());
    assert_eq!(statement.position(), statement_location);
    assert_eq!(
        CssRule::LayerStatement(statement.clone()),
        CssRule::LayerStatement(statement)
    );

    let nested_location = source_position(4, 1);
    let nested = CssRule::Style(CssStyleRule::new(
        CssStyleSelectorList::absolute(vec![CssSelector::Class("button".to_owned())]),
        CssDeclarationList::new(Vec::new()),
        Vec::new(),
        nested_location,
    ));
    let block_location = source_position(4, 5);
    let named_block = CssLayerBlockRule::new(
        Some(components.clone()),
        vec![nested.clone()],
        block_location,
    );
    assert_eq!(style_rule(&nested).position(), nested_location);
    assert_eq!(named_block.name(), Some(&components));
    assert_eq!(named_block.rules(), &[nested]);
    assert_eq!(named_block.position(), Some(block_location));

    let anonymous_block = CssLayerBlockRule::new(None, Vec::new(), block_location);
    assert_eq!(anonymous_block.name(), None);
    assert!(anonymous_block.rules().is_empty());
    assert_eq!(
        CssRule::LayerBlock(anonymous_block.clone()),
        CssRule::LayerBlock(anonymous_block)
    );
}

#[test]
fn scope_rule_model_keeps_scoped_selectors_and_rules_separate() {
    assert_eq!(CssScopeSelectorList::try_new(Vec::new()), None);
    assert_eq!(CssScopedStyleSelectorList::try_new(Vec::new()), None);

    let root_selector = CssSelector::Class("card".to_owned());
    let root =
        CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(root_selector.clone())])
            .unwrap();
    let limit_selector = CssSelector::Class("boundary".to_owned());
    let limit =
        CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(limit_selector.clone())])
            .unwrap();

    let implicit_selector =
        CssScopedStyleSelector::Selector(CssSelector::Class("title".to_owned()));
    let relative = CssRelativeSelector::new(
        CssSelectorCombinator::Child,
        CssSelector::Class("action".to_owned()),
    );
    let relative_selector = CssScopedStyleSelector::Relative(relative.clone());
    let selectors = CssScopedStyleSelectorList::try_new(vec![
        implicit_selector.clone(),
        relative_selector.clone(),
    ])
    .unwrap();
    let declaration = parse_single_declaration("color", "rgba(0, 0, 0, 1)");
    let style = CssScopedStyleRule::new(
        selectors.clone(),
        CssDeclarationList::new(vec![declaration.clone()]),
        Vec::new(),
        source_position(5, 3),
    );
    let scoped_rules = CssScopedRuleList::from_rules(vec![CssScopedRule::Style(style.clone())]);
    let location = source_position(5, 1);
    let scope = CssScopeRule::new(
        Some(root.clone()),
        Some(limit.clone()),
        scoped_rules.clone(),
        location,
    );

    assert_eq!(
        root.selectors(),
        &[CssScopeSelector::Selector(root_selector)]
    );
    assert_eq!(
        limit.selectors(),
        &[CssScopeSelector::Selector(limit_selector)]
    );
    assert_eq!(
        selectors.selectors(),
        &[implicit_selector, relative_selector]
    );
    assert_eq!(style.selectors(), &selectors);
    assert_eq!(style.declarations().as_slice(), &[declaration]);
    assert_eq!(style.position(), source_position(5, 3));
    assert_eq!(scope.root(), Some(&root));
    assert_eq!(scope.limit(), Some(&limit));
    assert_eq!(scope.rules(), &scoped_rules);
    assert_eq!(scope.position(), Some(location));
    assert_eq!(CssRule::Scope(scope.clone()), CssRule::Scope(scope));

    let empty_rules = CssScopedRuleList::new();
    assert!(empty_rules.rules().is_empty());
}

#[test]
fn scope_selector_list_constructor_rejects_pseudo_elements() {
    let ordinary = CssSelector::Class("card".to_owned());
    let ordinary_list =
        CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(ordinary.clone())]).unwrap();
    assert_eq!(
        ordinary_list.selectors(),
        &[CssScopeSelector::Selector(ordinary)]
    );

    let pseudo_element = CssPseudoElementSequence::try_new(vec![CssPseudoElement::Before]).unwrap();
    let selector = CssSelector::Compound(
        CssCompoundSelector::new_with_scope_anchor_and_pseudo_elements(
            false,
            None,
            None,
            vec!["card".to_owned()],
            Vec::new(),
            Vec::new(),
            Some(pseudo_element),
        ),
    );

    assert_eq!(
        CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(selector)]),
        None
    );

    let first =
        CssCompoundSelector::new(None, None, vec!["card".to_owned()], Vec::new(), Vec::new());
    let part = CssComplexSelectorPart::new(
        CssSelectorCombinator::Child,
        CssCompoundSelector::new_with_scope_anchor_and_pseudo_elements(
            false,
            None,
            None,
            vec!["badge".to_owned()],
            Vec::new(),
            Vec::new(),
            Some(CssPseudoElementSequence::try_new(vec![CssPseudoElement::Before]).unwrap()),
        ),
    );
    let selector = CssSelector::Complex(CssComplexSelector::try_new(first, vec![part]).unwrap());

    assert_eq!(
        CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(selector)]),
        None
    );
}

#[test]
fn scoped_group_rule_models_keep_scoped_children() {
    let child_selector =
        CssScopedStyleSelectorList::try_new(vec![CssScopedStyleSelector::Selector(
            CssSelector::Class("label".to_owned()),
        )])
        .unwrap();
    let location = source_position(8, 9);
    let child = CssScopedRule::Style(CssScopedStyleRule::new(
        child_selector,
        CssDeclarationList::new(Vec::new()),
        Vec::new(),
        location,
    ));
    let scoped_children = CssScopedRuleList::from_rules(vec![child.clone()]);
    let query = CssMediaQueryList::new(vec![
        crate::parse_media_query("screen")
            .into_validation_result()
            .unwrap(),
    ]);
    let media = CssScopedMediaRule::new(query.clone(), scoped_children.clone(), location);
    assert_eq!(media.query(), &query);
    assert_eq!(media.rules(), &scoped_children);
    assert_eq!(media.position(), Some(location));

    let prelude = CssContainerPrelude::try_from_components(
        crate::parse_component_values("sidebar (inline-size > 30rem)").unwrap(),
    )
    .unwrap();
    let condition = prelude.entries()[0].query().unwrap().clone();
    let name = CssContainerName::try_new("sidebar").unwrap();
    let container = CssScopedContainerRule::new(prelude.clone(), scoped_children.clone(), location);
    assert_eq!(container.prelude().entries()[0].name(), Some(&name));
    assert_eq!(container.prelude().entries()[0].query(), Some(&condition));
    assert_eq!(container.rules(), &scoped_children);
    assert_eq!(container.position(), Some(location));

    let layer_name = CssLayerName::try_new(["theme"]).unwrap();
    let layer_names = CssLayerNameList::try_new(vec![layer_name.clone()]).unwrap();
    let statement = CssScopedLayerStatementRule::new(layer_names.clone(), location);
    assert_eq!(statement.names(), &layer_names);
    assert_eq!(statement.position(), location);

    let block =
        CssScopedLayerBlockRule::new(Some(layer_name.clone()), scoped_children.clone(), location);
    assert_eq!(block.name(), Some(&layer_name));
    assert_eq!(block.rules(), &scoped_children);
    assert_eq!(block.position(), Some(location));

    assert_eq!(
        CssScopedRule::Media(media.clone()),
        CssScopedRule::Media(media)
    );
    assert_eq!(
        CssScopedRule::Container(container.clone()),
        CssScopedRule::Container(container)
    );
    assert_eq!(
        CssScopedRule::LayerStatement(statement.clone()),
        CssScopedRule::LayerStatement(statement)
    );
    assert_eq!(
        CssScopedRule::LayerBlock(block.clone()),
        CssScopedRule::LayerBlock(block)
    );
}

#[test]
fn scoped_selector_model_preserves_authored_scope_anchor_marker() {
    let anchored = CssCompoundSelector::new_with_scope_anchor(
        true,
        None,
        None,
        vec!["active".to_owned()],
        Vec::new(),
        vec![CssPseudoClass::Scope],
    );

    assert!(anchored.has_scope_anchor());
    assert_eq!(anchored.classes(), &["active".to_owned()]);
    assert_eq!(anchored.pseudo_classes(), &[CssPseudoClass::Scope]);

    let selector = CssScopedStyleSelector::Selector(CssSelector::Compound(anchored.clone()));
    let list = CssScopedStyleSelectorList::try_new(vec![selector.clone()]).unwrap();
    assert_eq!(list.selectors(), &[selector]);
}

#[test]
fn font_face_rule_accessors_expose_authored_structure() {
    let descriptors = CssFontFaceDescriptors::new(vec![
        CssFontFaceDescriptor::new(
            CssFontFaceDescriptorValue::FontFamily(
                CssFontFaceFamily::try_new("Avenir Next").unwrap(),
            )
            .into(),
        ),
        CssFontFaceDescriptor::new(
            CssFontFaceDescriptorValue::Src(
                CssFontFaceSourceList::try_new(vec![CssFontFaceSource::Url(
                    CssFontFaceUrlSource::new(
                        crate::CssUrl::new("fonts/avenir.woff2"),
                        None,
                        Vec::new(),
                    ),
                )])
                .unwrap(),
            )
            .into(),
        ),
        CssFontFaceDescriptor::new(
            CssFontFaceDescriptorValue::FontStyle(CssFontFaceStyle::Keyword(
                CssFontStyleKeyword::Normal,
            ))
            .into(),
        ),
        CssFontFaceDescriptor::new(
            CssFontFaceDescriptorValue::FontDisplay(CssFontDisplay::Auto).into(),
        ),
    ]);
    let location = source_position(9, 5);
    let rule = CssFontFaceRule::new(descriptors.clone()).with_position(location);

    assert_eq!(rule.descriptors(), &descriptors);
    assert_eq!(rule.position(), Some(location));
    assert_eq!(CssRule::FontFace(rule.clone()), CssRule::FontFace(rule));
}

#[test]
fn checked_part_and_complex_reject_invalid_private_compound_graphs() {
    // The owning private harness can express corrupt provider internals that
    // public checked compound construction correctly prevents from escaping.
    let empty = CssPseudoSelectorList::try_new_forgiving(Vec::new()).unwrap();
    let invalids = [
        CssCompoundSelector::new(None, None, vec![String::new()], Vec::new(), Vec::new()),
        CssCompoundSelector::new(None, None, vec!["a\0b".into()], Vec::new(), Vec::new()),
        CssCompoundSelector::new(
            None,
            None,
            Vec::new(),
            Vec::new(),
            vec![CssPseudoClass::Not(empty)],
        ),
    ];
    let valid = CssCompoundSelector::new(None, None, vec!["One".into()], Vec::new(), Vec::new());
    for invalid in invalids {
        let part_error =
            CssComplexSelectorPart::try_new(CssSelectorCombinator::Child, invalid.clone())
                .unwrap_err();
        let first_error = CssComplexSelector::try_new(
            invalid.clone(),
            vec![CssComplexSelectorPart::new(
                CssSelectorCombinator::Child,
                valid.clone(),
            )],
        )
        .unwrap_err();
        let rest_error = CssComplexSelector::try_new(
            valid.clone(),
            vec![CssComplexSelectorPart::new(
                CssSelectorCombinator::Child,
                invalid,
            )],
        )
        .unwrap_err();
        for error in [part_error, first_error, rest_error] {
            let CssSelectorConstructionErrorKind::Specified(cause) = error.kind() else {
                panic!("complete graph admission error: {error:?}");
            };
            assert_eq!(
                cause.kind(),
                CssSpecifiedValueSerializationErrorKind::UnrepresentableValue
            );
        }
    }
    let part =
        CssComplexSelectorPart::try_new(CssSelectorCombinator::Child, valid.clone()).unwrap();
    let value = CssComplexSelector::try_new(valid, vec![part]).unwrap();
    assert_eq!(
        CssSelector::Complex(value).to_specified_css().unwrap(),
        ".One > .One"
    );
}

#[test]
fn complex_selector_constructor_rejects_parts_after_pseudo_elements() {
    let pseudo_element = CssPseudoElementSequence::try_new(vec![CssPseudoElement::Before]).unwrap();
    let first = CssCompoundSelector::new_with_scope_anchor_and_pseudo_elements(
        false,
        None,
        None,
        vec!["button".to_owned()],
        Vec::new(),
        Vec::new(),
        Some(pseudo_element),
    );
    let part = CssComplexSelectorPart::new(
        CssSelectorCombinator::Descendant,
        CssCompoundSelector::new(None, None, vec!["icon".to_owned()], Vec::new(), Vec::new()),
    );

    assert!(CssComplexSelector::try_new(first, vec![part]).is_err());
}

#[test]
fn pseudo_selector_list_constructor_accepts_complex_selectors() {
    assert!(CssPseudoSelectorList::try_new(Vec::new()).is_err());

    let first =
        CssCompoundSelector::new(None, None, vec!["field".to_owned()], Vec::new(), Vec::new());
    let part = CssComplexSelectorPart::new(
        CssSelectorCombinator::Descendant,
        CssCompoundSelector::new(None, None, vec!["icon".to_owned()], Vec::new(), Vec::new()),
    );
    let complex = CssSelector::Complex(CssComplexSelector::try_new(first, vec![part]).unwrap());

    let list = CssPseudoSelectorList::try_new(vec![complex.clone()]).unwrap();
    assert_eq!(
        list.items(),
        &[CssPseudoSelectorListItem::Selector(complex)]
    );
}

#[test]
fn custom_property_value_accessors_preserve_authored_css() {
    let value = CssCustomPropertyValue::new(
        CssAuthoredDeclarationValue::try_new("calc(var(--space) * 2)").unwrap(),
    );
    assert_eq!(value.as_css(), "calc(var(--space) * 2)");
    assert!(!value.is_empty());
}

#[test]
fn variable_dependent_value_preserves_authored_css() {
    let authored = CssAuthoredDeclarationValue::try_new("var(--space)").unwrap();
    let value = CssSubstitutionDependentValue::new(authored);
    assert_eq!(value.as_css(), "var(--space)");
}

#[test]
fn variable_reference_and_fallback_accessors_preserve_authored_css() {
    let fallback_reference =
        CssVariableReference::new(CssCustomPropertyName::try_new("--fallback").unwrap(), None);
    let fallback = CssVariableFallback::new(
        CssAuthoredDeclarationValue::try_new("calc(1px + var(--fallback))").unwrap(),
        vec![fallback_reference.clone()],
    );
    let reference = CssVariableReference::new(
        CssCustomPropertyName::try_new("--space").unwrap(),
        Some(fallback),
    );
    assert_eq!(reference.name().as_str(), "--space");
    let fallback = reference.fallback().unwrap();
    assert_eq!(fallback.as_css(), "calc(1px + var(--fallback))");
    assert_eq!(fallback.references(), &[fallback_reference]);
}

// Checked public list admission makes these old invalid retained graphs
// unconstructible externally. Keep the exact later owning-boundary defenses here
// using existing private assembly, without adding a production-visible hook.
mod selector_list_admission_defenses {
    use super::*;

    fn parsed(source: &str) -> CssSelector {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source:?}: {report:?}");
        report.into_validation_result().unwrap().unwrap()
    }

    fn exact(source: &str, expected: &str) {
        let selector = parsed(source);
        let before = selector.clone();
        assert_eq!(selector.to_specified_css().unwrap(), expected);
        assert_eq!(selector, before);
        assert_eq!(parsed(expected).to_specified_css().unwrap(), expected);
    }

    fn pseudo_element_list() -> CssPseudoSelectorList {
        CssPseudoSelectorList::new(vec![parsed("::before")])
    }

    fn invalid_graph(selector: CssSelector) {
        let before = selector.clone();
        let result = selector.to_specified_css();
        assert!(
            result.is_err(),
            "invalid retained graph emitted: {result:?}"
        );
        assert_eq!(
            result.unwrap_err().kind(),
            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue
        );
        assert_eq!(
            selector, before,
            "rejected emission must not mutate its input"
        );
    }

    #[test]
    fn retained_not_graph_cannot_emit_a_pseudo_element_member() {
        exact(":not(.One)", ":not(.One)");
        invalid_graph(CssSelector::PseudoClass(CssPseudoClass::Not(
            pseudo_element_list(),
        )));
    }

    #[test]
    fn retained_is_graph_cannot_emit_a_pseudo_element_member() {
        exact(":is(.One)", ":is(.One)");
        invalid_graph(CssSelector::PseudoClass(CssPseudoClass::Is(
            pseudo_element_list(),
        )));
    }

    #[test]
    fn retained_where_graph_cannot_emit_a_pseudo_element_member() {
        exact(":where(.One)", ":where(.One)");
        invalid_graph(CssSelector::PseudoClass(CssPseudoClass::Where(
            pseudo_element_list(),
        )));
    }

    #[test]
    fn retained_nth_child_graph_cannot_emit_a_pseudo_element_of_member() {
        exact(":nth-child(2n+1 of .One)", ":nth-child(2n+1 of .One)");
        invalid_graph(CssSelector::PseudoClass(CssPseudoClass::NthChild(
            CssNthChildPattern::new(CssNthPattern::Odd, Some(pseudo_element_list())),
        )));
    }

    #[test]
    fn retained_nth_last_child_graph_cannot_emit_a_pseudo_element_of_member() {
        exact(":nth-last-child(2 of .One)", ":nth-last-child(2 of .One)");
        invalid_graph(CssSelector::PseudoClass(CssPseudoClass::NthLastChild(
            CssNthChildPattern::new(CssNthPattern::Integer(2), Some(pseudo_element_list())),
        )));
    }

    #[test]
    fn retained_compound_arguments_reject_logical_pseudo_element_members() {
        for invalid in [
            CssPseudoClass::Where(pseudo_element_list()),
            CssPseudoClass::NthChild(CssNthChildPattern::new(
                CssNthPattern::Odd,
                Some(pseudo_element_list()),
            )),
        ] {
            assert!(
                CssCompoundSelectorArgument::try_new(CssSelector::PseudoClass(invalid)).is_none()
            );
        }
    }

    #[test]
    fn retained_part_suffix_rejects_a_logical_pseudo_element_member() {
        let part = CssPseudoElement::Part(
            CssPartNameList::try_new(vec![CssPartName::try_new("label").unwrap()]).unwrap(),
        );
        let logical = CssPseudoClass::Is(pseudo_element_list());
        assert!(
            CssPseudoElementSequence::try_from_segments(vec![
                CssPseudoElementSegment::PseudoElement(part),
                CssPseudoElementSegment::PseudoClass(logical),
            ])
            .is_none()
        );
    }

    #[test]
    fn retained_scope_boundary_lists_reject_recursive_pseudo_elements() {
        let selector = parsed(".root::before");
        for invalid in [
            CssSelector::PseudoClass(CssPseudoClass::Is(CssPseudoSelectorList::new(vec![
                selector.clone(),
            ]))),
            CssSelector::PseudoClass(CssPseudoClass::Not(CssPseudoSelectorList::new(vec![
                selector,
            ]))),
        ] {
            assert!(
                CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(invalid)]).is_none()
            );
        }
    }

    #[test]
    fn retained_scope_rule_checks_a_nul_identifier_in_functional_arguments() {
        let selector =
            CssSelector::PseudoClass(CssPseudoClass::Is(CssPseudoSelectorList::new(vec![
                CssSelector::Class("\0".into()),
            ])));
        let root =
            CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(selector)]).unwrap();
        let error = CssScopeRule::try_new(
            Some(root),
            None,
            Vec::new(),
            &CssNamespaceContext::default(),
            CssScopeNestingContext::None,
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            CssRuleConstructionErrorKind::InvalidSelectorIdentifier
        );
        assert!(error.path().is_empty());
        assert_eq!(error.position(), None);
    }
}
