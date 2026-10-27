//! Existing-API retention expectations from selected Nesting 1 section 3.1:
//! invalid forgiving items containing an actual ampersand delimiter survive
//! exactly as authored. Their preservation does not make invalid source clean.

use crate::css::support::color_and_selector_fixtures::{
    assert_selector_input_rejected, assert_strict_validation_rejects,
};
use surgeist_css::CssParser;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction, CssTokenKind};
use surgeist_css::rule::CssRule;
use surgeist_css::selector::{
    CssNamespaceContext, CssParsedStyleSelectors, CssPseudoClass, CssPseudoSelectorListItem,
    CssRelativeSelectorList, CssSelector, CssSelectorCombinator, CssStyleSelector,
    CssStyleSelectorContext,
};

fn retained(source: &str, literal: &str) {
    let report = CssSelector::parse(
        &CssParser::default(),
        source,
        &CssNamespaceContext::default(),
    );
    let selector = report
        .syntax()
        .as_ref()
        .expect("retained forgiving outer selector");
    // This is the intended callable RED observation, before any future-model
    // assertion. It must keep raw invalid item spelling and ordered boundaries.
    assert_eq!(
        selector.serialize_specified(Default::default()).unwrap(),
        literal,
        "{source:?}"
    );
    assert!(!report.is_clean());
    assert!(!report.diagnostics().is_empty());
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidSelector)
    );
    let diagnostics = report.diagnostics().to_vec();
    assert_eq!(
        report.into_validation_result().unwrap_err().diagnostics(),
        diagnostics
    );

    let reparsed = CssSelector::parse(
        &CssParser::default(),
        literal,
        &CssNamespaceContext::default(),
    );
    assert_eq!(
        reparsed
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        literal
    );
    assert!(!reparsed.is_clean());
    assert_strict_validation_rejects(reparsed);
}

fn members(selector: &CssSelector) -> &[CssPseudoSelectorListItem] {
    match selector {
        CssSelector::PseudoClass(CssPseudoClass::Is(list) | CssPseudoClass::Where(list)) => {
            list.items()
        }
        _ => panic!("logical forgiving selector"),
    }
}

#[test]
fn is_and_where_preserve_unknown_ampersand_items_with_exact_spelling_and_order() {
    for (source, literal) in [
        (":is(:NoPe(&),.Keep)", ":is(:NoPe(&), .Keep)"),
        (":where(:NoPe(&))", ":where(:NoPe(&))"),
        (
            r":is(:No\50 e(/*keep*/&),.Keep)",
            r":is(:No\50 e(/*keep*/&), .Keep)",
        ),
        (
            ":is(.Before,\t /*L*/:NoPe(&) /*T*/\t,.After)",
            ":is(.Before,\t /*L*/:NoPe(&) /*T*/\t, .After)",
        ),
    ] {
        retained(source, literal);
    }
    let report = CssSelector::parse(
        &CssParser::default(),
        ":where(.Before,:NoPe(/*one*/&/*two*/),.Middle,:Other(&),.After)",
        &CssNamespaceContext::default(),
    );
    let specified = report
        .syntax()
        .as_ref()
        .unwrap()
        .serialize_specified(Default::default())
        .unwrap();
    // Raw members carry all authored boundary trivia. Their preceding comma
    // adds no padding, while valid members keep canonical comma-space padding.
    let literal = ":where(.Before,:NoPe(/*one*/&/*two*/), .Middle,:Other(&), .After)";
    assert_eq!(specified, literal);
    let reparsed = CssSelector::parse(
        &CssParser::default(),
        literal,
        &CssNamespaceContext::default(),
    );
    assert_eq!(
        reparsed
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        literal
    );
    let mut previous = None;
    for authored in [
        ".Before",
        ":NoPe(/*one*/&/*two*/)",
        ".Middle",
        ":Other(&)",
        ".After",
    ] {
        let offset = specified
            .find(authored)
            .expect("complete exact authored member survives");
        assert!(previous.is_none_or(|previous| previous < offset));
        previous = Some(offset);
    }
    assert_strict_validation_rejects(reparsed);
    assert_strict_validation_rejects(report);
}

#[test]
fn invalid_constructs_find_ampersands_after_failure_and_in_function_descendants() {
    for (source, literal) in [
        (":is(???/*keep*/&,.Keep)", ":is(???/*keep*/&, .Keep)"),
        (
            ":where(:NoPe(inner(deeper(&))),.Keep)",
            ":where(:NoPe(inner(deeper(&))), .Keep)",
        ),
        (
            ":is(:nth-child(nope(&)),.Keep)",
            ":is(:nth-child(nope(&)), .Keep)",
        ),
        (":where(&div,.Keep)", ":where(&div, .Keep)"),
    ] {
        retained(source, literal);
    }
}

#[test]
fn nested_logical_and_public_list_fronts_retain_invalid_nesting_items() {
    retained(
        ":is(:where(:NoPe(&),.Keep),.After)",
        ":is(:where(:NoPe(&), .Keep), .After)",
    );
    let report = CssParsedStyleSelectors::parse(
        &CssParser::default(),
        ".Before,:where(:NoPe(&)),.After",
        &CssNamespaceContext::default(),
        CssStyleSelectorContext::Ordinary,
    );
    let list = report.syntax().as_ref().unwrap();
    assert_eq!(list.selectors().selectors().len(), 3);
    for (member, literal) in
        list.selectors()
            .selectors()
            .iter()
            .zip([".Before", ":where(:NoPe(&))", ".After"])
    {
        assert_eq!(
            member
                .selector()
                .serialize_specified(Default::default())
                .unwrap(),
            literal
        );
    }
    assert_eq!(
        list.selectors().selectors()[0].selector(),
        &CssSelector::Class("Before".into())
    );
    assert_eq!(
        list.selectors().selectors()[2].selector(),
        &CssSelector::Class("After".into())
    );
    assert_strict_validation_rejects(report);

    let report = CssRelativeSelectorList::parse(
        &CssParser::default(),
        "> .Before,+ :is(:NoPe(&),.Keep)",
        &CssNamespaceContext::default(),
    );
    let list = report.syntax().as_ref().unwrap();
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        "> .Before, + :is(:NoPe(&), .Keep)"
    );
    assert_eq!(list.selectors().len(), 2);
    assert_eq!(
        list.selectors()[0].combinator(),
        CssSelectorCombinator::Child
    );
    assert_eq!(
        list.selectors()[1].combinator(),
        CssSelectorCombinator::NextSibling
    );
    assert_strict_validation_rejects(report);
}

#[test]
fn preserved_item_keeps_original_unicode_source_error_identity_and_member_span() {
    let source = "/*😀*/\r\n:is(:NoPe(/*Ω*/&),.Keep)";
    let report = CssSelector::parse(
        &CssParser::default(),
        source,
        &CssNamespaceContext::default(),
    );
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        ":is(:NoPe(/*Ω*/&), .Keep)"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one responsible invalid-item diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 15);
    assert_eq!(diagnostic.error().position().line().value(), 1);
    assert_eq!(diagnostic.error().position().column().value(), 5);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 14);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 28);
    assert_eq!(&source[14..28], ":NoPe(/*Ω*/&)");
    let detail = diagnostic.error();
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidSelector,
        "typed invalid selector"
    );
    assert_eq!(
        detail.production().unwrap().as_str(),
        "baseline.selector.complex"
    );
    let token = detail.encountered().unwrap();
    assert_eq!(token.kind(), CssTokenKind::Function);
    assert_eq!(token.authored(), "NoPe(");
    let diagnostics = report.diagnostics().to_vec();
    assert_eq!(
        report.into_validation_result().unwrap_err().diagnostics(),
        diagnostics
    );
}

#[test]
fn nested_rule_preserves_the_invalid_item_without_losing_parent_or_siblings() {
    let source = ".Parent { :is(:NoPe(&),.Keep) { color:red; } .After {} } .Outside {}";
    let report = CssParser::default().parse_sheet(source);
    let [CssRule::Style(parent), CssRule::Style(outside)] = report.syntax().rules() else {
        panic!("parent and surrounding rule");
    };
    let [CssRule::Style(child), CssRule::Style(after)] = parent.rules() else {
        panic!("both authored children retained");
    };
    assert_eq!(
        child.selectors().selectors()[0]
            .selector()
            .serialize_specified(Default::default())
            .unwrap(),
        ":is(:NoPe(&), .Keep)"
    );
    assert!(matches!(
        child.selectors().selectors(),
        [CssStyleSelector::Selector(_)]
    ));
    assert_eq!(
        parent.selectors().selectors()[0].selector(),
        &CssSelector::Class("Parent".into())
    );
    assert_eq!(
        after.selectors().selectors()[0].selector(),
        &CssSelector::Class("After".into())
    );
    assert_eq!(
        outside.selectors().selectors()[0].selector(),
        &CssSelector::Class("Outside".into())
    );
    assert_eq!(child.declarations().len(), 1);
    assert!(parent.declarations().is_empty());
    assert_strict_validation_rejects(report);
}

#[test]
fn strings_comments_and_escaped_identifiers_do_not_turn_invalid_items_into_nesting_items() {
    for name in ["is", "where"] {
        for item in [
            r#":NoPe("&")"#,
            ":NoPe(/*&*/.X)",
            r":NoPe(\&)",
            r":NoPe(\26)",
        ] {
            let report = CssSelector::parse(
                &CssParser::default(),
                format!(":{name}({item},.Keep)"),
                &CssNamespaceContext::default(),
            );
            let selector = report.syntax().as_ref().unwrap();
            assert_eq!(
                members(selector),
                [CssPseudoSelectorListItem::Selector(CssSelector::Class(
                    "Keep".into()
                ))]
            );
            assert_eq!(
                selector.serialize_specified(Default::default()).unwrap(),
                format!(":{name}(.Keep)")
            );
            let [diagnostic] = report.diagnostics() else {
                panic!("one genuinely discarded invalid item");
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropSelectorListItem);
            assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
            assert_strict_validation_rejects(report);
        }
    }
}

#[test]
fn valid_nesting_members_remain_symbolic_and_distinct_from_literal_scope() {
    let report = CssSelector::parse(
        &CssParser::default(),
        ":is(&&.Keep,:scope)",
        &CssNamespaceContext::default(),
    );
    assert!(report.is_clean());
    let selector = report.into_validation_result().unwrap().unwrap();
    let [
        CssPseudoSelectorListItem::Selector(CssSelector::Compound(nesting)),
        CssPseudoSelectorListItem::Selector(CssSelector::PseudoClass(CssPseudoClass::Scope)),
    ] = members(&selector)
    else {
        panic!("nesting and literal Scope identities");
    };
    assert_eq!(nesting.nesting_selectors(), 2);
    assert_eq!(nesting.scope_anchors(), 0);
    assert_eq!(nesting.classes(), ["Keep"]);
    assert_eq!(
        selector.serialize_specified(Default::default()).unwrap(),
        ":is(&&.Keep, :scope)"
    );
    assert_eq!(
        CssSelector::parse(
            &CssParser::default(),
            ":is(&&.Keep, :scope)",
            &CssNamespaceContext::default()
        )
        .into_validation_result()
        .unwrap()
        .unwrap(),
        selector
    );
}

#[test]
fn empty_forgiving_results_strict_lists_and_enclosing_any_value_gate_keep_their_contracts() {
    for name in ["is", "where"] {
        let source = format!(":{name}()");
        let empty = CssSelector::parse(
            &CssParser::default(),
            &source,
            &CssNamespaceContext::default(),
        );
        let selector = empty
            .syntax()
            .as_ref()
            .expect("retained recovered empty pseudo");
        assert!(members(selector).is_empty());
        assert_eq!(
            selector.serialize_specified(Default::default()).unwrap(),
            source
        );
        let [diagnostic] = empty.diagnostics() else {
            panic!("one zero-width dropped attempted member");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropSelectorListItem);
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        let (offset, column) = if name == "is" { (4, 4) } else { (7, 7) };
        for position in [
            diagnostic.error().position(),
            diagnostic.span().start(),
            diagnostic.span().end(),
        ] {
            assert_eq!(position.byte_offset().value(), offset);
            assert_eq!(position.line().value(), 0);
            assert_eq!(position.column().value(), column);
        }
        assert_strict_validation_rejects(empty);
        let discarded = CssSelector::parse(
            &CssParser::default(),
            format!(":{name}(:NoPe(.X))"),
            &CssNamespaceContext::default(),
        );
        assert!(members(discarded.syntax().as_ref().unwrap()).is_empty());
        assert_eq!(
            discarded
                .syntax()
                .as_ref()
                .unwrap()
                .serialize_specified(Default::default())
                .unwrap(),
            format!(":{name}()")
        );
        assert_eq!(
            discarded.diagnostics()[0].action(),
            CssRecoveryAction::DropSelectorListItem
        );
        assert_strict_validation_rejects(discarded);
    }
    for source in [
        ":not(.Keep,:NoPe(&))",
        ":not()",
        ":is(:NoPe(&),])",
        ":where(:NoPe(&),url(a b))",
        ":is(:NoPe(&),\"bad\n)",
    ] {
        assert_selector_input_rejected(
            source,
            CssSelector::parse(
                &CssParser::default(),
                source,
                &CssNamespaceContext::default(),
            ),
        );
    }
    let report = CssParser::default().parse_sheet(
        ".Parent { color:green; .Keep,:not(:NoPe(&)) { color:red; .Never {} } .After {} }",
    );
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("retained parent");
    };
    let [CssRule::Style(after)] = parent.rules() else {
        panic!("only valid sibling survives");
    };
    assert_eq!(
        after.selectors().selectors()[0].selector(),
        &CssSelector::Class("After".into())
    );
    assert_eq!(parent.declarations().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("one entire invalid nested rule");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_strict_validation_rejects(report);
}
