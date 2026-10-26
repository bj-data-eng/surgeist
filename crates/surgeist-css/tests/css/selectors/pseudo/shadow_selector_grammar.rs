//! Authored grammar imported by specs/catalog.json, without shadow-tree matching.
//! Host signatures: https://www.w3.org/TR/2014/WD-css-scoping-1-20140403/#host-selector
//! Slotted: https://github.com/w3c/csswg-drafts/blob/d2ed7a98bb7e499f3b04e6688941bb1a9823d396/css-shadow-1/Overview.bs#L522
//! Part: https://www.w3.org/TR/2025/WD-css-shadow-parts-1-20251216/#part
//! Suffix grammar: https://www.w3.org/TR/2026/WD-selectors-4-20260122/#pseudo-element-syntax
//! Element-backed suffixes: https://www.w3.org/TR/2025/WD-css-pseudo-4-20250627/#element-backed
//! Backdrop: https://www.w3.org/TR/2025/WD-css-position-4-20251007/#backdrop
//! Position 4 calls backdrop fully styleable; Pseudo 4 section 4 makes that a
//! subset of tree-abiding pseudo-elements, so the slotted suffix permits it.
use crate::css::support::color_and_selector_fixtures::assert_strict_validation_rejects;
use crate::css::support::color_and_selector_fixtures::{
    assert_selector_input_rejected, assert_selector_rejected as assert_rejected,
};
use surgeist_css::{
    CssErrorCode, CssNamespaceContext, CssNormalizedItem, CssParsedStyleSelectors, CssParser,
    CssPseudoClass, CssRecoveryAction, CssSelector, CssSelectorBinding, CssStyleSelectorContext,
};

fn assert_clean(source: &str) {
    let report = CssSelector::parse(
        &CssParser::default(),
        source,
        &CssNamespaceContext::default(),
    );
    assert!(report.is_clean(), "{source}: {report:?}");
    assert!(report.syntax().is_some(), "{source}: {report:?}");
    assert!(report.into_validation_result().is_ok(), "{source}");
}

#[test]
fn original_host_cases_accept_only_the_defined_nonempty_forms() {
    // Original selector/functional-pseudo/host.json cases. The imported grammar,
    // rather than upstream permissive parsing, supplies each expected outcome.
    for source in [":host(test)", ":host", ":hOsT(.a)", ":host(  .a.b  )"] {
        assert_clean(source);
    }
    for source in [
        ":host()",
        ":host(.a{)",
        ":host(,.b)",
        ":host(.a,)",
        ":host(var(--test))",
        ":host(foo,bar)",
    ] {
        assert_rejected(source);
    }
}

#[test]
fn original_host_context_cases_require_a_nonempty_function_argument() {
    // Original selector/functional-pseudo/host-context.json cases.
    for source in [
        ":host-context(test)",
        ":hOsT-cOntexT(.a)",
        ":host-context(  .a.b  )",
    ] {
        assert_clean(source);
    }
    for source in [
        ":host-context()",
        ":host-context(.a{)",
        ":host-context(,.b)",
        ":host-context(.a,)",
        ":host-context(var(--test))",
        ":host-context(foo,bar)",
        ":host-context",
    ] {
        assert_rejected(source);
    }
}

#[test]
fn original_slotted_cases_require_a_nonempty_function_argument() {
    // Original selector/functional-pseudo/slotted.json cases. Together these
    // three original-case tests contain ten clean and twenty rejected inputs.
    for source in ["::slotted(test)", "::SloTted(.a)", "::slotted(  .a.b  )"] {
        assert_clean(source);
    }
    for source in [
        "::slotted()",
        "::slotted(.a{)",
        "::slotted(,.b)",
        "::slotted(.a,)",
        "::slotted(var(--test))",
        "::slotted(foo,bar)",
        "::slotted",
    ] {
        assert_rejected(source);
    }
}

#[test]
fn shadow_pseudo_names_and_colons_are_adjacent_tokens() {
    for source in [
        ": host",
        ": host(.a)",
        ": host-context(.a)",
        ":: slotted(.a)",
        ":: part(label)",
        ": :slotted(.a)",
        ": :part(label)",
        ":host (.a)",
        "::part (label)",
    ] {
        assert_rejected(source);
    }
    // CSS comments do not produce whitespace tokens. These controls differ
    // from the rejected whitespace above without changing the pseudo's name.
    for source in [":/**/host(.a)", ":/**/:slotted(.a)", "::/**/part(label)"] {
        assert_clean(source);
    }
}

#[test]
fn compound_arguments_admit_nested_selector_grammars_without_becoming_lists() {
    for function in [":host", ":host-context", "::slotted"] {
        for argument in [
            "*",
            "|button",
            "*|button",
            "button#first#second.ready[title='hello']:hover",
            ":is(.a, .b)",
            ":not(.a.b)",
            ":has(> .child, + .next)",
            "&",
        ] {
            assert_clean(&format!("{function}({argument})"));
        }
        for argument in [
            ".a, .b",
            ".a .b",
            ".a > .b",
            ".a + .b",
            ".a ~ .b",
            "> .a",
            "::before",
            ".a::before",
            ":not(::before)",
            ":not(.a > .b)",
            ":not(:not(.a > .b))",
        ] {
            assert_rejected(&format!("{function}({argument})"));
        }
    }
}

#[test]
fn logical_arguments_inherit_the_shadow_functions_compound_restriction() {
    // Selectors 4 section 4 explicitly passes compound-only restrictions to
    // :is(), :where(), and :not(); forgiving lists discard the invalid member.
    // This does not replace :has()'s separately defined relative-list grammar.
    for function in [":host", ":host-context", "::slotted"] {
        for logical in ["is", "where"] {
            let source = format!("{function}(:{logical}(.a > .b, .valid))");
            let report = CssSelector::parse(
                &CssParser::default(),
                &source,
                &CssNamespaceContext::default(),
            );
            assert!(report.syntax().is_some(), "{source}: {report:?}");
            let [diagnostic] = report.diagnostics() else {
                panic!("one invalid compound-only member: {source}: {report:?}");
            };
            assert_eq!(
                diagnostic.error().code(),
                CssErrorCode::InvalidSelector,
                "{source}"
            );
            assert_eq!(
                diagnostic.action(),
                CssRecoveryAction::DropSelectorListItem,
                "{source}"
            );
            assert_strict_validation_rejects(report);
        }
        assert_clean(&format!("{function}(:has(> .a .b))"));
    }
    // The same logical argument is valid where complex selectors are allowed.
    assert_clean(":not(.a > .b)");
    assert_clean(":is(.a > .b, .valid)");
}

#[test]
fn part_arguments_are_identifiers_without_custom_identifier_exclusions() {
    for source in [
        "::part(label)",
        "::PaRt(Label active)",
        "::part( label /**/ active label )",
        "::part(initial inherit unset revert default)",
        r"::part(\31 st label\:icon)",
        r"::p\61 rt(label)",
    ] {
        assert_clean(source);
    }
    for source in [
        "::part",
        "::part()",
        "::part(/**/)",
        "::part(label,active)",
        "::part('label')",
        "::part(1)",
        "::part(*)",
        "::part(.label)",
        "::part(var(--part))",
        ":part(label)",
        ":slotted(.a)",
    ] {
        assert_rejected(source);
    }
}

#[test]
fn shadow_functions_preserve_the_enclosing_has_restriction() {
    for source in [
        ":has(:host(.a))",
        ":has(:host-context(.a))",
        ":host(:has(.a))",
        "::slotted(:has(> .a))",
        "::part(label):has(.a)",
    ] {
        assert_clean(source);
    }
    for source in [
        ":has(:host(:has(.a)))",
        ":has(:host-context(:has(.a)))",
        ":host(:has(:host(:has(.a))))",
        "::slotted(:has(:host-context(:has(.a))))",
        "::part(label):has(:host(:has(.a)))",
        ":has(::slotted(.a))",
        ":has(::part(label))",
    ] {
        assert_rejected(source);
    }
}

#[test]
fn slotted_suffixes_follow_tree_abiding_and_general_pseudo_class_rules() {
    for source in [
        "::slotted(.a)::before",
        "::slotted(.a)::after::marker",
        "::slotted(.a)::marker",
        "::slotted(.a)::part(label)",
        "::slotted(.a)::backdrop",
        "::slotted(.a):hover",
        "::slotted(.a):not(:hover)",
        "::slotted(.a):not(:hover:focus)",
        "::slotted(.a):is(:hover, :focus)",
        "::slotted(.a)::before:hover::marker",
        "::before:hover",
        "::before:not(:hover:focus)",
    ] {
        assert_clean(source);
    }
    for source in [
        "::slotted(.a)::selection",
        "::slotted(.a)::first-line",
        "::slotted(.a)::first-letter",
        "::slotted(.a)::slotted(.b)",
        "::slotted(.a):first-child",
        "::slotted(.a):not(:first-child)",
        "::slotted(.a):not(.a)",
        "::before:not(.a)",
        "::before:not(button)",
        "::before:not([title])",
        "::before:not(:hover > :hover)",
        "::slotted(.a)::marker::before",
        "::slotted(.a) > .child",
        "::slotted(.a).extra",
    ] {
        assert_rejected(source);
    }
}

/// Pseudo-Elements 4 (WD 2025-06-27) §5 "Element-backed Pseudo-Elements"
/// syntactically allows all pseudo-classes and pseudo-elements after an
/// element-backed pseudo-element, while structural pseudo-classes, `:has()`,
/// `:scope`, `:host*` and a nested `::part()` never match. Selectors 4
/// (WD 2026-01-22) §3.6.3 "Pseudo-classing Pseudo-elements" normatively allows
/// any pseudo-element to be followed by the logical combination and user
/// action pseudo-classes, lets other specifications allow more, and states
/// "Combinations that are not explicitly allowed are invalid selectors". Its
/// accompanying note says the logical combination pseudo-classes "pass any
/// restrictions on validity of selectors at their position to their
/// arguments", so `:not()` after `::part()` admits only pseudo-class
/// arguments: `:not(:first-child)` is valid (and never matches), while a
/// class, type, attribute or combinator argument is invalid.
///
/// Recorded witness divergence (#1326), not followed: WPT
/// `css/css-shadow/part/pseudo-classes-after-part.html`, byte-identical to the
/// pinned WebKit import and passed by pinned WebKit and Servo, expects
/// structural pseudo-classes and `:not(:first-child)` after `::part()` to be
/// invalid; it predates the Pseudo-Elements 4 text.
#[test]
fn element_backed_suffixes_retain_even_never_matching_authored_selectors() {
    for source in [
        "x-button::part(label):hover",
        "::part(label):first-child",
        "::part(label):scope",
        "::part(label):host",
        "::part(label):host(.a)",
        "::part(label):host-context(.a)",
        "::part(label):not(:first-child)",
        "::part(label):not(:first-child:hover)",
        "::part(label)::part(inner)",
        "::part(label)::slotted(.a)",
        "::part(label)::selection",
        "::part(label):hover::before",
    ] {
        assert_clean(source);
    }
    for source in [
        "::part(label).extra",
        "::part(label)#id",
        "::part(label)[title]",
        "::part(label) .child",
        "::part(label) > .child",
        "::part(label)::before:first-child",
        "::part(label)::before:not(:first-child)",
        "::part(label)::before::after",
        "::part(label):not(.a)",
        "::part(label):not(button)",
        "::part(label):not([title])",
        "::part(label):not(:hover > :hover)",
    ] {
        assert_rejected(source);
    }
}

#[test]
fn pseudo_class_after_part_does_not_move_to_the_originating_element() {
    // Exact retained selector meaning is a public parser contract. Admission
    // alone would miss moving a suffix condition onto the originating element.
    for (source, expected_leading_pseudos) in [
        (".button:hover::part(label)", &[CssPseudoClass::Hover][..]),
        (".button::part(label):hover", &[][..]),
        (".button::part(label):hover::before", &[][..]),
        (".button::part(label)::before:hover", &[][..]),
    ] {
        let report = CssSelector::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
        );
        assert!(report.is_clean(), "{source}: {report:?}");
        let Some(CssSelector::Compound(compound)) = report.syntax() else {
            panic!("retained originating compound: {source}: {report:?}");
        };
        assert_eq!(compound.classes(), &["button".to_owned()], "{source}");
        assert_eq!(
            compound.pseudo_classes(),
            expected_leading_pseudos,
            "{source}"
        );
    }
}

#[test]
fn forgiving_suffix_lists_inherit_the_current_pseudo_element_restrictions() {
    for source in [
        "::slotted(.a):is(:first-child, :hover)",
        "::slotted(.a):where(:first-child, :hover)",
        "::part(label)::before:is(:first-child, :hover)",
        "::part(label):is(.a, :hover)",
        "::slotted(.a):is(.a, :hover)",
        "::before:is(.a, :hover)",
    ] {
        let report = CssSelector::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
        );
        assert!(report.syntax().is_some(), "{source}: {report:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid forgiving member: {source}: {report:?}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidSelector,
            "{source}"
        );
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropSelectorListItem,
            "{source}"
        );
        assert_strict_validation_rejects(report);
    }
    // The same structural condition is syntactically valid immediately after
    // an element-backed pseudo-element, although it cannot match there.
    assert_clean("::part(label):is(:first-child, :hover)");
}

#[test]
fn shadow_selector_arguments_do_not_consume_outer_list_separators() {
    let context = CssNamespaceContext::default();
    let report = CssParsedStyleSelectors::parse(
        &CssParser::default(),
        ":host(.a), ::slotted(.b), ::part(label)",
        &context,
        CssStyleSelectorContext::Ordinary,
    );
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .selectors()
            .selectors()
            .len(),
        3
    );
    // Shadow 1: :host() takes one <compound-selector>, so its comma
    // invalidates the whole ordinary list.
    let source = ":host(.a, .b), .valid";
    assert_selector_input_rejected(
        source,
        CssParsedStyleSelectors::parse(
            &CssParser::default(),
            source,
            &context,
            CssStyleSelectorContext::Ordinary,
        ),
    );
}

#[test]
fn function_contained_nesting_anchors_keep_the_enclosing_parent_binding() {
    // Nesting 1 section 4 and CssSelectorBinding preserve function-contained
    // anchors instead of adding an implicit descendant relationship.
    for (selector, has_anchor) in [
        (":host(&)", true),
        (":host-context(&)", true),
        ("::slotted(&)", true),
        ("::part(label):host(&)", true),
        (":host(.leaf)", false),
        (":host-context(.leaf)", false),
        ("::slotted(.leaf)", false),
        ("::part(label):host(.leaf)", false),
    ] {
        for has_parent in [false, true] {
            let source = if has_parent {
                format!(".parent, #alternate {{ {selector} {{ color: red }} }}")
            } else {
                format!("{selector} {{ color: red }}")
            };
            let report = CssParser::default().parse_sheet(&source);
            assert!(report.is_clean(), "{source}: {report:?}");
            let normalized = report.syntax().normalize(Default::default()).unwrap();
            let declarations: Vec<_> = normalized
                .items()
                .iter()
                .filter_map(|item| {
                    if let CssNormalizedItem::Declaration(declaration) = item {
                        Some(declaration)
                    } else {
                        None
                    }
                })
                .collect();
            let [declaration] = declarations.as_slice() else {
                panic!("one retained declaration: {source}");
            };
            let context = declaration.selector_context();
            assert_eq!(context.selectors().len(), 1, "{source}");
            let binding = if has_anchor {
                CssSelectorBinding::ExplicitAnchors
            } else if has_parent {
                CssSelectorBinding::ImplicitDescendant
            } else {
                CssSelectorBinding::Absolute
            };
            assert_eq!(context.selectors()[0].binding(), binding, "{source}");
            assert_eq!(context.parent().is_some(), has_parent, "{source}");
            assert!(context.scope_context().is_none(), "{source}");
            if let Some(parent) = context.parent() {
                assert_eq!(parent.selectors().len(), 2, "{source}");
            }
        }
    }
}

#[test]
fn function_contained_scope_anchors_remain_distinct_from_nesting_anchors() {
    for (selector, binding) in [
        (":host(&)", CssSelectorBinding::ScopeAnchors),
        (":host-context(&)", CssSelectorBinding::ScopeAnchors),
        ("::slotted(&)", CssSelectorBinding::ScopeAnchors),
        ("::part(label):host(&)", CssSelectorBinding::ScopeAnchors),
        (":host(.leaf)", CssSelectorBinding::Absolute),
        (":host-context(.leaf)", CssSelectorBinding::Absolute),
        ("::slotted(.leaf)", CssSelectorBinding::Absolute),
        ("::part(label):host(.leaf)", CssSelectorBinding::Absolute),
    ] {
        let source = format!("@scope (.root) {{ {selector} {{ color: red }} }}");
        let report = CssParser::default().parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let normalized = report.syntax().normalize(Default::default()).unwrap();
        let declarations: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| {
                if let CssNormalizedItem::Declaration(declaration) = item {
                    Some(declaration)
                } else {
                    None
                }
            })
            .collect();
        let [declaration] = declarations.as_slice() else {
            panic!("one retained declaration: {source}");
        };
        let context = declaration.selector_context();
        assert_eq!(context.selectors().len(), 1, "{source}");
        assert_eq!(context.selectors()[0].binding(), binding, "{source}");
        assert!(context.parent().is_none(), "{source}");
        assert!(context.scope_context().is_some(), "{source}");
    }
}
