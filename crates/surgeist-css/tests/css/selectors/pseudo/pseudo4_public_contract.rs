#![forbid(unsafe_code)]
//! Additive API functional draft; run only after the selected identities exist.
//! Pseudo4 WD20250627 §§2.2/3.1/4/5/7.1, referenced Values4 custom-ident and
//! Selectors4 suffix restrictions own these symbolic authored expectations.
use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type LimitKind = CssSpecifiedValueSerializationErrorKind;

fn parsed(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().clone().unwrap()
}

fn compound(value: &CssSelector) -> &CssCompoundSelector {
    let CssSelector::Compound(value) = value else {
        panic!("originating compound")
    };
    value
}

fn segments(value: &CssSelector) -> &[CssPseudoElementSegment] {
    compound(value).pseudo_elements().unwrap().segments()
}

fn exact(source: &str, expected: &str) -> CssSelector {
    let value = parsed(source);
    let before = value.clone();
    assert_eq!(value.to_specified_css().unwrap(), expected);
    assert_eq!(value, before);
    assert_eq!(parsed(expected), value);
    let reused = CssSelectorList::try_new(vec![value.clone()]).unwrap();
    assert_eq!(reused.selectors(), std::slice::from_ref(&value));
    assert_eq!(reused.to_specified_css().unwrap(), expected);
    value
}

fn rejected(source: &str) {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.syntax().is_none(), "{source}: {report:?}");
    assert!(!report.is_clean());
}

fn suffix_list(value: &CssSelector) -> &CssPseudoSelectorList {
    let [
        _,
        CssPseudoElementSegment::PseudoClass(CssPseudoClass::Is(list)),
    ] = segments(value)
    else {
        panic!("one Is suffix")
    };
    list
}

fn retained(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(!report.is_clean(), "{source}");
    assert!(report.diagnostics().iter().any(
        |diagnostic| diagnostic.action() == CssRecoveryAction::PreserveInvalidSelectorListItem
    ));
    report.syntax().clone().unwrap()
}

fn raw(list: &CssPseudoSelectorList) -> &CssInvalidNestingSelectorItem {
    let [CssPseudoSelectorListItem::InvalidNesting(value)] = list.items() else {
        panic!("parser-owned retained item")
    };
    value
}

fn attach(
    element: CssPseudoElement,
    list: CssPseudoSelectorList,
) -> Option<CssPseudoElementSequence> {
    CssPseudoElementSequence::try_from_segments(vec![
        CssPseudoElementSegment::PseudoElement(element),
        CssPseudoElementSegment::PseudoClass(CssPseudoClass::Is(list)),
    ])
}

#[test]
fn every_new_identity_has_its_selected_typed_model_and_canonical_name() {
    for (source, expected, element) in [
        ("::PREFIX", "::prefix", CssPseudoElement::Prefix),
        ("::suffix", "::suffix", CssPseudoElement::Suffix),
        (
            "::search-text",
            "::search-text",
            CssPseudoElement::SearchText,
        ),
        (
            "::target-text",
            "::target-text",
            CssPseudoElement::TargetText,
        ),
        (
            "::spelling-error",
            "::spelling-error",
            CssPseudoElement::SpellingError,
        ),
        (
            "::grammar-error",
            "::grammar-error",
            CssPseudoElement::GrammarError,
        ),
        (
            "::highlight(Name)",
            "::highlight(Name)",
            CssPseudoElement::Highlight(CssCustomIdent::try_new("Name").unwrap()),
        ),
        (
            r"::\70 laceholder",
            "::placeholder",
            CssPseudoElement::Placeholder,
        ),
        (
            "::file-selector-button",
            "::file-selector-button",
            CssPseudoElement::FileSelectorButton,
        ),
        (
            "::details-content",
            "::details-content",
            CssPseudoElement::DetailsContent,
        ),
    ] {
        let value = exact(source, expected);
        assert_eq!(
            segments(&value),
            &[CssPseudoElementSegment::PseudoElement(element.clone())]
        );
        assert_eq!(
            CssPseudoElementSequence::try_new(vec![element])
                .unwrap()
                .segments(),
            segments(&value)
        );
        assert!(compound(&value).type_selector().is_none());
    }
    let ordered = exact(
        "p:FIRST-LETTER::prefix:hover",
        "p::first-letter::prefix:hover",
    );
    assert_eq!(
        compound(&ordered).type_selector().unwrap().local_name(),
        Some("p")
    );
    assert_eq!(
        segments(&ordered),
        &[
            CssPseudoElementSegment::PseudoElement(CssPseudoElement::FirstLetter),
            CssPseudoElementSegment::PseudoElement(CssPseudoElement::Prefix),
            CssPseudoElementSegment::PseudoClass(CssPseudoClass::Hover),
        ]
    );
}

#[test]
fn highlight_payload_uses_the_existing_case_sensitive_custom_identifier_contract() {
    for (source, decoded, expected) in [
        ("::highlight(MyName)", "MyName", "::highlight(MyName)"),
        (r"::HIGHLIGHT(\4d yName)", "MyName", "::highlight(MyName)"),
        ("::highlight(none)", "none", "::highlight(none)"),
        (r"::highlight(A\ B)", "A B", r"::highlight(A\ B)"),
        ("::highlight(Δa)", "Δa", "::highlight(Δa)"),
    ] {
        let value = exact(source, expected);
        let [CssPseudoElementSegment::PseudoElement(CssPseudoElement::Highlight(name))] =
            segments(&value)
        else {
            panic!("Highlight payload")
        };
        assert_eq!(name.as_str(), decoded);
        let checked = CssCustomIdent::try_new(decoded).unwrap();
        assert_eq!(&checked, name);
        assert_eq!(
            CssPseudoElementSequence::try_new(vec![CssPseudoElement::Highlight(checked)])
                .unwrap()
                .segments(),
            segments(&value)
        );
    }
    assert_ne!(parsed("::highlight(MyName)"), parsed("::highlight(myname)"));
    for reserved in [
        "initial",
        "INHERIT",
        "unset",
        "revert",
        "revert-layer",
        "DeFaUlT",
    ] {
        assert!(CssCustomIdent::try_new(reserved).is_none());
        rejected(&format!("::highlight({reserved})"));
    }
    for invalid in [
        "::highlight()",
        "::highlight(\"Name\")",
        "::highlight(a b)",
        "::highlight(a,b)",
        "::highlight(1)",
        "::highlight (Name)",
    ] {
        rejected(invalid);
    }
}

#[test]
fn current_is_a_search_text_or_element_backed_suffix_and_resets_with_the_receiver() {
    use CssPseudoElementSegment::{PseudoClass as P, PseudoElement as E};
    let value = exact("::SEARCH-TEXT:CURRENT", "::search-text:current");
    assert_eq!(
        segments(&value),
        &[E(CssPseudoElement::SearchText), P(CssPseudoClass::Current)]
    );
    for receiver in [
        CssPseudoElement::SearchText,
        CssPseudoElement::FileSelectorButton,
        CssPseudoElement::DetailsContent,
    ] {
        assert!(
            CssPseudoElementSequence::try_from_segments(vec![
                E(receiver),
                P(CssPseudoClass::Current)
            ])
            .is_some()
        );
    }
    for receiver in [
        CssPseudoElement::Placeholder,
        CssPseudoElement::TargetText,
        CssPseudoElement::Prefix,
    ] {
        assert!(
            CssPseudoElementSequence::try_from_segments(vec![
                E(receiver),
                P(CssPseudoClass::Current)
            ])
            .is_none()
        );
    }
    for source in [
        "::search-text:not(:current)",
        "::search-text:is(:current,:hover)",
        "::search-text:where(:current)",
        "::details-content:current",
        "::details-content::search-text:not(:current)",
    ] {
        parsed(source);
    }
    for source in [
        ":current",
        ":current()",
        "::search-text:current()",
        "::search-text:past",
        "::search-text:future",
        "::target-text:current",
        "::search-text:not(:first-child)",
        "::details-content::placeholder:current",
        "::details-content::target-text:not(:current)",
    ] {
        rejected(source);
    }
    let detached = CssSelector::PseudoClass(CssPseudoClass::Current);
    assert_eq!(
        detached.to_specified_css().unwrap_err().kind(),
        LimitKind::UnrepresentableValue
    );
    assert!(CssSelectorList::try_new(vec![detached.clone()]).is_err());
    assert!(CssPseudoSelectorList::try_new(vec![detached]).is_err());

    let report = parse_selector(
        "::search-text:is(:first-child,:current,:hover)",
        &CssNamespaceContext::default(),
    );
    assert!(!report.is_clean());
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .to_specified_css()
            .unwrap(),
        "::search-text:is(:current, :hover)"
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropSelectorListItem)
    );
}

#[test]
fn new_receiver_classes_share_checked_child_and_logical_attachment_permissions() {
    use CssPseudoElementSegment::{PseudoClass as P, PseudoElement as E};
    // Selectors4 §4.2/§16.1 keeps an empty Is after dropping the forbidden
    // structural member; Highlight still resets the element-backed permission.
    let source = "::details-content::highlight(Name):is(:first-child)";
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(!report.is_clean());
    let value = report.syntax().as_ref().unwrap();
    assert_eq!(
        segments(value),
        &[
            E(CssPseudoElement::DetailsContent),
            E(CssPseudoElement::Highlight(
                CssCustomIdent::try_new("Name").unwrap()
            )),
            P(CssPseudoClass::Is(
                CssPseudoSelectorList::try_new_forgiving(Vec::new()).unwrap()
            )),
        ]
    );
    assert_eq!(
        value.to_specified_css().unwrap(),
        "::details-content::highlight(Name):is()"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one original forbidden member")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropSelectorListItem);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    let start = source.rfind(":first-child").unwrap();
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.len() - 1
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        start + 1
    );
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("original selector diagnostic")
    };
    let token = detail.encountered().unwrap();
    assert_eq!(token.kind(), CssTokenKind::Ident);
    assert_eq!(token.authored(), "first-child");
    assert!(report.into_validation_result().is_err());
    for parent in [
        CssPseudoElement::FileSelectorButton,
        CssPseudoElement::DetailsContent,
    ] {
        let elements = vec![
            parent.clone(),
            CssPseudoElement::Before,
            CssPseudoElement::Marker,
        ];
        assert!(CssPseudoElementSequence::try_new(elements).is_some());
        assert!(
            CssPseudoElementSequence::try_from_segments(vec![
                E(parent),
                P(CssPseudoClass::FirstChild),
                E(CssPseudoElement::Placeholder),
                P(CssPseudoClass::Hover)
            ])
            .is_some()
        );
    }
    assert!(
        CssPseudoElementSequence::try_new(vec![
            CssPseudoElement::FirstLetter,
            CssPseudoElement::Suffix
        ])
        .is_some()
    );
    for invalid in [
        vec![CssPseudoElement::Before, CssPseudoElement::Prefix],
        vec![CssPseudoElement::FirstLine, CssPseudoElement::Suffix],
        vec![
            CssPseudoElement::FirstLetter,
            CssPseudoElement::Prefix,
            CssPseudoElement::Suffix,
        ],
        vec![CssPseudoElement::Placeholder, CssPseudoElement::Marker],
    ] {
        assert!(CssPseudoElementSequence::try_new(invalid).is_none());
    }
    for (source, expected) in [
        (
            "::file-selector-button:has(>.x)",
            "::file-selector-button:has(> .x)",
        ),
        (
            "::details-content:scope::part(label)",
            "::details-content:scope::part(label)",
        ),
        (
            "::slotted(.item)::placeholder:hover",
            "::slotted(.item)::placeholder:hover",
        ),
        (
            "::part(label)::details-content:first-child",
            "::part(label)::details-content:first-child",
        ),
        (
            "::part(label)::highlight(Name):is(:hover)",
            "::part(label)::highlight(Name):is(:hover)",
        ),
    ] {
        exact(source, expected);
    }
    for source in [
        "::details-content::placeholder:first-child",
        "::details-content::highlight(Name):not(:first-child)",
        "::slotted(.item)::highlight(Name)",
        "::file-selector-button > .x",
    ] {
        rejected(source);
    }
}

#[test]
fn retained_invalid_items_require_true_permission_set_inclusion_for_checked_reuse() {
    let generic = retained("::placeholder:is(:NoPe(&))");
    let search = retained("::search-text:is(:NoPe(&))");
    let backed = retained("::details-content:is(:NoPe(&))");
    for (source, value) in [
        ("::placeholder:is(:NoPe(&))", &generic),
        ("::search-text:is(:NoPe(&))", &search),
        ("::details-content:is(:NoPe(&))", &backed),
    ] {
        let item = raw(suffix_list(value));
        assert_eq!(item.authored(), ":NoPe(&)");
        assert_eq!(item.origin().source().as_str(), source);
        assert_eq!(
            item.origin().span().start().byte_offset().value(),
            source.find(":NoPe").unwrap()
        );
        assert_eq!(
            item.origin().span().end().byte_offset().value(),
            source.len() - 1
        );
        assert_eq!(value.to_specified_css().unwrap(), source);
        let cloned = item.clone();
        assert!(
            cloned
                .origin()
                .source()
                .same_snapshot(item.origin().source())
        );
        let detached = CssSelector::PseudoClass(CssPseudoClass::Is(suffix_list(value).clone()));
        assert_eq!(
            detached.to_specified_css().unwrap_err().kind(),
            LimitKind::UnrepresentableValue
        );
        assert!(CssSelectorList::try_new(vec![detached]).is_err());
    }
    let before = search.clone();
    assert!(attach(CssPseudoElement::SearchText, suffix_list(&search).clone()).is_some());
    assert!(attach(CssPseudoElement::Placeholder, suffix_list(&search).clone()).is_some());
    assert!(
        attach(
            CssPseudoElement::DetailsContent,
            suffix_list(&search).clone()
        )
        .is_none()
    );
    assert!(attach(CssPseudoElement::SearchText, suffix_list(&generic).clone()).is_none());
    assert!(
        attach(
            CssPseudoElement::DetailsContent,
            suffix_list(&generic).clone()
        )
        .is_none()
    );
    assert!(attach(CssPseudoElement::SearchText, suffix_list(&backed).clone()).is_some());
    assert!(attach(CssPseudoElement::Placeholder, suffix_list(&backed).clone()).is_some());
    assert_ne!(raw(suffix_list(&generic)), raw(suffix_list(&search)));
    assert_eq!(search, before);

    // Has prohibition and suffix/compound restrictions are independent axes.
    // SearchText narrows suffix classes but would re-enable Has: incomparable.
    let nested_source = "::details-content:has(:is(:NoPe(&)))";
    let nested = retained(nested_source);
    let [
        _,
        CssPseudoElementSegment::PseudoClass(CssPseudoClass::Has(relative)),
    ] = segments(&nested)
    else {
        panic!("Has suffix")
    };
    let CssSelector::PseudoClass(CssPseudoClass::Is(list)) = relative.selectors()[0].selector()
    else {
        panic!("nested Is")
    };
    let item = raw(list);
    assert_eq!(item.authored(), ":NoPe(&)");
    assert_eq!(item.origin().source().as_str(), nested_source);
    assert!(attach(CssPseudoElement::SearchText, list.clone()).is_none());
    assert!(attach(CssPseudoElement::Placeholder, list.clone()).is_none());
    assert_eq!(nested.to_specified_css().unwrap(), nested_source);
}

#[test]
fn strict_forgiving_scope_and_supports_consumers_apply_their_actual_pseudo_boundaries() {
    let highlight = parsed("::highlight(Name)");
    assert!(CssPseudoSelectorList::try_new(vec![highlight.clone()]).is_err());
    assert!(CssPseudoSelectorList::try_new_forgiving(vec![highlight.clone()]).is_err());
    assert!(CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(highlight)]).is_none());
    for source in [
        ":not(.keep,::highlight(Name))",
        ":has(> ::placeholder)",
        ":nth-child(1 of ::target-text)",
        "::slotted(::placeholder)",
    ] {
        rejected(source);
    }
    for function in ["is", "where"] {
        let source = format!(":{function}(::highlight(Name),.keep)");
        let report = parse_selector(&source, &CssNamespaceContext::default());
        assert!(!report.is_clean());
        assert_eq!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .to_specified_css()
                .unwrap(),
            format!(":{function}(.keep)")
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropSelectorListItem)
        );
    }
    let scope = parse_sheet("@scope(::highlight(Name)){.lost{color:red}}.after{color:blue}");
    assert!(matches!(scope.syntax().rules(), [CssRule::Style(_)]));
    assert!(
        scope
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropAtRule)
    );
    for source in [
        "selector(::highlight(Name))",
        "selector(::search-text:current)",
        "selector(::first-letter::suffix)",
    ] {
        let value = CssSupportsCondition::try_from_components(
            parse_component_values(source).unwrap(),
            &CssNamespaceContext::default(),
        )
        .unwrap();
        assert!(matches!(
            value.kind(),
            CssSupportsConditionKind::Selector(_)
        ));
        assert_eq!(value.serialize().unwrap().as_css(), source);
    }
    let invalid = CssSupportsCondition::try_from_components(
        parse_component_values("selector(::highlight())").unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    assert!(matches!(
        invalid.kind(),
        CssSupportsConditionKind::GeneralEnclosed(_)
    ));
}

#[test]
fn namespaces_and_lexical_probe_origins_survive_checked_reuse_without_payload_origins() {
    let source = "/*😀*/@namespace p 'urn:p';@supports selector(p|Input::HIGHLIGHT(\\4d iXeD)){p|Input::highlight(MiXeD){color:red}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Namespace(_), CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("namespace and Supports")
    };
    let condition = rule.condition();
    let CssSupportsConditionKind::Selector(value) = condition.kind() else {
        panic!("typed selector probe")
    };
    let origin = compound(value);
    assert_eq!(
        origin.type_selector().unwrap().namespace(),
        &CssNamespaceConstraint::Named(CssNamespacePrefix::try_new("p").unwrap())
    );
    assert_eq!(
        value.to_specified_css().unwrap(),
        "p|Input::highlight(MiXeD)"
    );
    let function = condition
        .components()
        .iter()
        .find_map(|value| match value.view() {
            CssComponentValueRef::Function(value) => Some(value),
            _ => None,
        })
        .unwrap();
    let highlight_function = function
        .values()
        .items()
        .iter()
        .find_map(|value| match value.view() {
            CssComponentValueRef::Function(value) => Some(value),
            _ => None,
        })
        .unwrap();
    let name = highlight_function
        .values()
        .items()
        .iter()
        .find(|value| {
            matches!(
                value.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Ident("MiXeD"))
            )
        })
        .unwrap();
    let CssValueOrigin::Parsed(parsed_origin) = name.origin() else {
        panic!("original lexical argument")
    };
    assert_eq!(parsed_origin.source().as_str(), source);
    let offset = source.find(r"\4d iXeD").unwrap();
    assert_eq!(parsed_origin.span().start().byte_offset().value(), offset);
    assert_eq!(
        parsed_origin.span().end().byte_offset().value(),
        offset + r"\4d iXeD".len()
    );
    assert_eq!(
        condition.serialize().unwrap().as_css(),
        r" selector(p|Input::HIGHLIGHT(\4d iXeD))"
    );
    let context = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("p").unwrap()),
        CssNamespaceName::new("urn:p"),
    )]);
    let reused = CssSupportsCondition::try_from_components(
        CssComponentValues::try_new(condition.components().to_vec()).unwrap(),
        &context,
    )
    .unwrap();
    assert_eq!(&reused, condition);
    let [CssRule::Style(style)] = rule.rules() else {
        panic!("retained style")
    };
    assert_eq!(
        style.position().byte_offset().value(),
        source.find("p|Input::highlight").unwrap()
    );
    assert_eq!(style.selectors().selectors()[0].selector(), value);
    let sheet = CssSheet::try_from_rules(report.syntax().rules().to_vec()).unwrap();
    assert_eq!(sheet.rules(), report.syntax().rules());
    assert_eq!(
        sheet.to_specified_css().unwrap(),
        report.syntax().to_specified_css().unwrap()
    );
}

#[test]
fn nested_and_scoped_bodies_preserve_parent_context_source_order_and_canonical_pseudos() {
    let source = "/*😀*/.parent::details-content{color:red;@supports selector(.probe::highlight(Name)){color:blue;& .child::search-text:current{color:green}}}@scope(.root){.scoped::placeholder:hover{color:black}}";
    let expected = ".parent::details-content { color: red; @supports selector(.probe::highlight(Name)) { color: blue; & .child::search-text:current { color: green; } } }\n@scope (.root) { .scoped::placeholder:hover { color: black; } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 4);
    for (order, (value, color)) in declarations
        .iter()
        .zip(["red", "blue", "green", "black"])
        .enumerate()
    {
        assert_eq!(value.order(), order);
        assert_eq!(
            value
                .source()
                .value_components()
                .serialize()
                .unwrap()
                .as_css()
                .trim(),
            color
        );
        assert_eq!(
            value.source().parsed_value().unwrap().source().as_str(),
            source
        );
        assert_eq!(
            value.source().position().unwrap().byte_offset().value(),
            source.find(&format!("color:{color}")).unwrap()
        );
    }
    let parent = declarations[0].selector_context();
    assert_eq!(
        parent.selectors()[0].selector(),
        &parsed(".parent::details-content")
    );
    assert!(declarations[1].selector_context().same_context(parent));
    let child = declarations[2].selector_context();
    assert!(child.parent().unwrap().same_context(parent));
    assert_eq!(
        child.selectors()[0].selector().to_specified_css().unwrap(),
        "& .child::search-text:current"
    );
    assert!(declarations[3].selector_context().scope_context().is_some());
    assert_eq!(
        declarations[3].selector_context().selectors()[0]
            .selector()
            .to_specified_css()
            .unwrap(),
        ".scoped::placeholder:hover"
    );
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    let reused = CssSheet::try_from_rules(report.syntax().rules().to_vec()).unwrap();
    assert_eq!(reused.to_specified_css().unwrap(), expected);
    assert_eq!(report, before);
    drop(report);
    assert_eq!(
        parent.selectors()[0].selector(),
        &parsed(".parent::details-content")
    );
    let replay = parse_sheet(expected);
    assert!(replay.is_clean(), "{replay:?}");
    assert_eq!(replay.syntax().to_specified_css().unwrap(), expected);
}

#[test]
fn cumulative_node_and_escaped_byte_limits_fail_atomically_and_allow_retry() {
    // Public graph accounting: list aggregate plus two compound/element pairs.
    let members = vec![parsed("::placeholder"), parsed("::target-text")];
    let literal = "::placeholder, ::target-text";
    let exact_limit = Limits::new(5, 5, literal.len());
    let list = CssSelectorList::try_new_with_limits(members.clone(), exact_limit).unwrap();
    let before = list.clone();
    assert_eq!(
        list.to_specified_css_with_limits(exact_limit).unwrap(),
        literal
    );
    for (limits, cause) in [
        (Limits::new(4, 5, literal.len()), LimitKind::InputNodeLimit),
        (
            Limits::new(5, 4, literal.len()),
            LimitKind::ProjectionNodeLimit,
        ),
        (Limits::new(5, 5, literal.len() - 1), LimitKind::ByteLimit),
    ] {
        for member in &members {
            assert!(member.to_specified_css_with_limits(limits).is_ok());
        }
        let error = CssSelectorList::try_new_with_limits(members.clone(), limits).unwrap_err();
        let CssSelectorConstructionErrorKind::Specified(cause_error) = error.kind() else {
            panic!("shared specified resource cause")
        };
        assert_eq!(cause_error.kind(), cause);
        assert_eq!(
            list.to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            cause
        );
        assert_eq!(list, before);
    }
    assert_eq!(
        list.to_specified_css_with_limits(exact_limit).unwrap(),
        literal
    );
    assert_eq!(CssSelectorList::try_new(members).unwrap(), list);

    let escaped = vec![
        parsed(r"::highlight(A\ B)"),
        parsed("::search-text:current"),
    ];
    let expected = r"::highlight(A\ B), ::search-text:current";
    let fit = Limits::new(usize::MAX, usize::MAX, expected.len());
    let short = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    let checked = CssSelectorList::try_new_with_limits(escaped.clone(), fit).unwrap();
    let unchanged = checked.clone();
    for member in &escaped {
        assert!(member.to_specified_css_with_limits(short).is_ok());
    }
    assert_eq!(checked.to_specified_css_with_limits(fit).unwrap(), expected);
    assert!(CssSelectorList::try_new_with_limits(escaped, short).is_err());
    assert_eq!(
        checked
            .to_specified_css_with_limits(short)
            .unwrap_err()
            .kind(),
        LimitKind::ByteLimit
    );
    assert_eq!(checked, unchanged);
    assert_eq!(checked.to_specified_css_with_limits(fit).unwrap(), expected);
}

#[test]
fn highlight_function_and_checked_rule_composition_respect_the_structural_ceiling() {
    let opening = "@media all{";
    let leaf = ".leaf::highlight(Name){color:red}";
    // The structural ceiling counts block-bearing rules: 255 groups plus this
    // style rule reach256. At256 groups, preflight rejects the style body's
    // opening brace before selector interpretation; functional selector depth
    // has its own check and is not the rejected boundary in this specimen.
    let accepted_source = format!("{}{leaf}{}", opening.repeat(255), "}".repeat(255));
    let rejected_source = format!("{}{leaf}{}", opening.repeat(256), "}".repeat(256));
    let accepted = parse_sheet(&accepted_source);
    let rejected = parse_sheet(&rejected_source);
    assert!(accepted.is_clean(), "{accepted:?}");
    let mut rules = accepted.syntax().rules();
    for _ in 0..255 {
        let [CssRule::Media(group)] = rules else {
            panic!("retained ambient group")
        };
        rules = group.rules();
    }
    let [CssRule::Style(style)] = rules else {
        panic!("retained Highlight rule")
    };
    assert_eq!(
        style.selectors().selectors()[0]
            .selector()
            .to_specified_css()
            .unwrap(),
        ".leaf::highlight(Name)"
    );
    assert_eq!(style.position().byte_offset().value(), opening.len() * 255);
    assert_eq!(
        style.declarations()[0]
            .parsed_value()
            .unwrap()
            .source()
            .as_str(),
        accepted_source
    );
    assert_eq!(
        style.declarations()[0]
            .value_components()
            .serialize()
            .unwrap()
            .as_css(),
        "red"
    );

    assert!(!rejected.is_clean());
    let [diagnostic] = rejected.diagnostics() else {
        panic!("one first over-limit style body")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        opening.len() * 256 + leaf.find('{').unwrap()
    );
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("typed nesting limit")
    };
    assert_eq!(detail.limit(), 256);
    assert_eq!(
        detail.enclosing_production().as_str(),
        "baseline.rule.style"
    );
    let mut rules = rejected.syntax().rules();
    for _ in 0..256 {
        let [CssRule::Media(group)] = rules else {
            panic!("retained group around rejected style body")
        };
        rules = group.rules();
    }
    assert!(rules.is_empty());
    assert!(rejected.into_validation_result().is_err());

    // Checked rule assembly documents a separate block-bearing rule ceiling.
    // Existing parsed descendants keep their genuine source position; new
    // wrappers have none. No forbidden logical pseudo-element graph is made.
    let original = "/*😀*/.leaf::highlight(Name){color:red}";
    let parsed = parse_sheet(original);
    assert!(parsed.is_clean(), "{parsed:?}");
    let mut child = parsed.syntax().rules()[0].clone();
    let query_report = parse_sheet("@media all{}");
    assert!(query_report.is_clean());
    let [CssRule::Media(query_rule)] = query_report.syntax().rules() else {
        panic!("query carrier")
    };
    let query = query_rule.query().clone();
    let context = CssNamespaceContext::default();
    for _ in 1..256 {
        let group = CssMediaRule::try_new(query.clone(), vec![child], &context).unwrap();
        assert_eq!(group.position(), None);
        child = CssRule::Media(group);
    }
    let before = child.clone();
    let checked = CssSheet::try_from_rules(vec![child.clone()]).unwrap();
    assert_eq!(checked.rules(), std::slice::from_ref(&child));
    let error = CssMediaRule::try_new(query, vec![child.clone()], &context).unwrap_err();
    assert_eq!(error.kind(), CssRuleConstructionErrorKind::NestingLimit);
    assert_eq!(error.path(), &[0; 256]);
    assert_eq!(
        error.position().unwrap().byte_offset().value(),
        original.find(".leaf").unwrap()
    );
    assert_eq!(child, before);
    assert!(CssSheet::try_from_rules(vec![child]).is_ok());
}
