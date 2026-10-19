#![forbid(unsafe_code)]
//! Functional additive payload and composed attachment contracts. Oracles are
//! Selectors4 WD20260122 §§3.6/3.10/Appendix B, Selectors3 legacy colon syntax,
//! imported Pseudo4 WD20250627 marker/element-backed definitions and the selected
//! Shadow1 slotted definition. Matching, specificity and live identity are absent.
use surgeist_css::*;

fn selector(source: &str) -> CssSelector {
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

fn unknown(value: &CssSelector) -> &CssUnknownWebkitPseudoElement {
    let [CssPseudoElementSegment::PseudoElement(CssPseudoElement::UnknownWebkit(value))] =
        compound(value).pseudo_elements().unwrap().segments()
    else {
        panic!("one symbolic unknown pseudo-element")
    };
    value
}

fn reject(source: &str) {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.syntax().is_none(), "{source}: {report:?}");
    assert!(!report.is_clean());
}

fn exact(source: &str, expected: &str) -> CssSelector {
    let value = selector(source);
    let before = value.clone();
    assert_eq!(value.to_specified_css().unwrap(), expected);
    assert_eq!(value, before);
    assert_eq!(selector(expected), value);
    let list = CssSelectorList::try_new(vec![value.clone()]).unwrap();
    assert_eq!(list.selectors(), std::slice::from_ref(&value));
    assert_eq!(list.to_specified_css().unwrap(), expected);
    value
}

#[test]
fn autofill_aliases_have_the_standard_identity_and_canonical_output() {
    for source in [
        ":autofill",
        ":AUTOFILL",
        ":-webkit-autofill",
        r":-\77 ebkit-auto\66 ill",
        ":/**/-WEBKIT-AUTOFILL",
    ] {
        let value = exact(source, ":autofill");
        assert_eq!(value, CssSelector::PseudoClass(CssPseudoClass::Autofill));
        assert!(!value.has_pseudo_elements());
    }
    let standard = selector("input:autofill");
    assert_eq!(selector("input:-WEBKIT-AUTOFILL"), standard);
    assert_eq!(standard.to_specified_css().unwrap(), "input:autofill");
}

#[test]
fn unknown_webkit_names_are_ascii_canonical_decoded_identifiers_not_function_tokens() {
    for (source, name, expected) in [
        ("::-WebKit-AsDf", "-webkit-asdf", "::-webkit-asdf"),
        (r"::-\77 ebkit-AsDf", "-webkit-asdf", "::-webkit-asdf"),
        ("::-webkit-", "-webkit-", "::-webkit-"),
        ("::-WebKit-ΔA", "-webkit-Δa", "::-webkit-Δa"),
        (r"::-WebKit-a\ b", "-webkit-a b", r"::-webkit-a\ b"),
        (r"::-WebKit-fn\(\)", "-webkit-fn()", r"::-webkit-fn\(\)"),
    ] {
        let value = exact(source, expected);
        assert!(value.has_pseudo_elements());
        assert_eq!(unknown(&value).as_str(), name);
        let checked = CssUnknownWebkitPseudoElement::try_new(name).unwrap();
        assert_eq!(&checked, unknown(&value));
        assert_eq!(checked.clone(), checked);
    }
    for decoded in [
        "-WebKit-X",
        "-WEBKIT-",
        "-WebKit-a b",
        "-WebKit-fn()",
        "-WebKit-ΔA",
    ] {
        let checked = CssUnknownWebkitPseudoElement::try_new(decoded).unwrap();
        assert_eq!(checked.as_str(), decoded.to_ascii_lowercase());
        assert!(
            CssPseudoElementSequence::try_new(vec![CssPseudoElement::UnknownWebkit(checked)])
                .is_some()
        );
    }
    for invalid in ["", "webkit-x", "-moz-x", ":-webkit-x", "-webkit-\0x"] {
        let error = CssUnknownWebkitPseudoElement::try_new(invalid).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssSelectorConstructionErrorKind::InvalidUnknownWebkitPseudoElementName
        );
    }
    for functional in ["::-webkit-x()", "::-WebKit-X(arg)", r"::-\77 ebkit-x()"] {
        reject(functional);
    }
}

#[test]
fn originating_compound_and_implied_universal_do_not_absorb_pseudo_suffix_conditions() {
    let value = exact(
        "button#Id.c:autofill::-WebKit-X:hover:focus",
        "button#Id.c:autofill::-webkit-x:hover:focus",
    );
    let origin = compound(&value);
    assert_eq!(origin.type_selector().unwrap().local_name(), Some("button"));
    assert_eq!(origin.ids(), &["Id"]);
    assert_eq!(origin.classes(), &["c"]);
    assert_eq!(origin.pseudo_classes(), &[CssPseudoClass::Autofill]);
    let [
        CssPseudoElementSegment::PseudoElement(CssPseudoElement::UnknownWebkit(name)),
        CssPseudoElementSegment::PseudoClass(CssPseudoClass::Hover),
        CssPseudoElementSegment::PseudoClass(CssPseudoClass::Focus),
    ] = origin.pseudo_elements().unwrap().segments()
    else {
        panic!("ordered suffix ownership")
    };
    assert_eq!(name.as_str(), "-webkit-x");
    let implicit = selector("::-webkit-x");
    assert!(compound(&implicit).type_selector().is_none());
    assert!(compound(&implicit).pseudo_classes().is_empty());
    let explicit = exact("*::-webkit-x", "*::-webkit-x");
    assert!(compound(&explicit).type_selector().unwrap().is_universal());
    let context = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("Svg").unwrap()),
        CssNamespaceName::new("urn:svg"),
    )]);
    let report = parse_selector("Svg|*::-WebKit-X", &context);
    assert!(report.is_clean(), "{report:?}");
    let value = report.syntax().as_ref().unwrap();
    let name = compound(value).type_selector().unwrap();
    assert!(name.is_universal());
    assert_eq!(
        name.namespace(),
        &CssNamespaceConstraint::Named(CssNamespacePrefix::try_new("Svg").unwrap())
    );
    assert_eq!(value.to_specified_css().unwrap(), "Svg|*::-webkit-x");
}

#[test]
fn defined_child_permissions_and_suffix_reset_share_checked_sequence_admission() {
    for parent in ["::before", "::after"] {
        selector(&format!("{parent}::marker"));
    }
    for child in [
        "::before",
        "::after",
        "::marker",
        "::part(label)",
        "::backdrop",
    ] {
        selector(&format!("::slotted(.item){child}"));
    }
    // Pseudo4 permits all otherwise admitted pseudo syntax after Part, even
    // combinations its matching rules define as never matching.
    for child in [
        "::before",
        "::after",
        "::first-line",
        "::first-letter",
        "::marker",
        "::selection",
        "::backdrop",
        "::slotted(.item)",
        "::part(next)",
        "::-webkit-x",
    ] {
        selector(&format!("::part(label){child}"));
    }
    for parent in [
        "::first-line",
        "::first-letter",
        "::marker",
        "::selection",
        "::backdrop",
        "::-webkit-x",
    ] {
        reject(&format!("{parent}::marker"));
    }
    for child in [
        "::first-line",
        "::first-letter",
        "::selection",
        "::-webkit-x",
    ] {
        reject(&format!("::slotted(.item){child}"));
    }
    for invalid in [
        "::before::before",
        "::after::after",
        "::before::after",
        "::marker::marker",
        "::part(label)::-webkit-x::marker",
    ] {
        reject(invalid);
    }

    use CssPseudoElementSegment::{PseudoClass as P, PseudoElement as E};
    let part = CssPseudoElement::Part(
        CssPartNameList::try_new(vec![CssPartName::try_new("label").unwrap()]).unwrap(),
    );
    let vendor = CssPseudoElement::UnknownWebkit(
        CssUnknownWebkitPseudoElement::try_new("-WebKit-X").unwrap(),
    );
    let segments = vec![
        E(part.clone()),
        P(CssPseudoClass::Autofill),
        E(vendor.clone()),
        P(CssPseudoClass::Hover),
    ];
    assert_eq!(
        CssPseudoElementSequence::try_from_segments(segments.clone())
            .unwrap()
            .segments(),
        segments
    );
    for invalid in [
        vec![E(vendor.clone()), E(CssPseudoElement::Marker)],
        vec![E(part), E(vendor.clone()), P(CssPseudoClass::Autofill)],
        vec![E(vendor), P(CssPseudoClass::FirstChild)],
    ] {
        assert!(CssPseudoElementSequence::try_from_segments(invalid).is_none());
    }
}

#[test]
fn generic_suffixes_are_admitted_and_part_permission_resets_after_unknown_segment() {
    // Selectors4 §3.6.3 is a syntax permission, including the source's at-risk
    // user-action combinations. No assertion here supplies matching truth.
    for parent in [
        "::before",
        "::after",
        "::first-line",
        "::first-letter",
        "::marker",
        "::selection",
        "::backdrop",
        "::slotted(.item)",
        "::-webkit-x",
    ] {
        for suffix in [
            ":hover",
            ":active",
            ":focus",
            ":focus-visible",
            ":focus-within",
            ":not(:hover)",
            ":is(:hover)",
            ":where(:focus)",
        ] {
            selector(&format!("{parent}{suffix}"));
        }
        reject(&format!("{parent}:autofill"));
        reject(&format!("{parent}:not(:autofill)"));
    }
    for suffix in [
        ":autofill",
        ":first-child",
        ":has(>.item)",
        ":scope",
        ":host",
        ":not(:autofill)",
    ] {
        selector(&format!("::part(label){suffix}"));
    }
    reject("::part(label)::-webkit-x:autofill");
    selector("::part(label)::-webkit-x:focus");
    let report = parse_selector(
        "::part(label)::-webkit-x:is(:autofill,:hover)",
        &CssNamespaceContext::default(),
    );
    assert!(!report.is_clean());
    let value = report.syntax().as_ref().unwrap();
    assert_eq!(
        value.to_specified_css().unwrap(),
        "::part(label)::-webkit-x:is(:hover)"
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropSelectorListItem)
    );
}

#[test]
fn legacy_colons_stay_pseudo_elements_and_function_adjacency_is_preserved() {
    for name in ["before", "after", "first-line", "first-letter"] {
        let one = selector(&format!(":{name}"));
        let two = selector(&format!("::{name}"));
        assert_eq!(one, two);
        assert!(one.has_pseudo_elements());
        assert_eq!(one.to_specified_css().unwrap(), format!("::{name}"));
        reject(&format!(":not(:{name})"));
    }
    for invalid in [
        ":marker",
        ":selection",
        ":backdrop",
        ":-webkit-x",
        "::autofill",
        ":autofill()",
        "::before()",
        "::part (label)",
        "::slotted (.item)",
        ": -webkit-autofill",
        ":: -webkit-x",
        ": :before",
    ] {
        reject(invalid);
    }
    exact("::/**/-WebKit-X", "::-webkit-x");
    exact(":/**/:before", "::before");
}

#[test]
fn all_current_pseudo_rows_reject_internal_and_sibling_combinators() {
    // Selectors4 §3.6.5 requires an explicit internal-structure definition.
    // None is selected for this admitted set; ::shadow is merely illustrative.
    for pseudo in [
        "::before",
        "::after",
        "::first-line",
        "::first-letter",
        "::marker",
        "::selection",
        "::backdrop",
        "::slotted(.item)",
        "::part(label)",
        "::-webkit-x",
    ] {
        for combinator in [" ", " > ", " + ", " ~ ", " || "] {
            reject(&format!("{pseudo}{combinator}.child"));
        }
    }
    exact(".parent > .child::-WebKit-X", ".parent > .child::-webkit-x");
    let terminal = selector(".parent > .child::-webkit-x");
    let CssSelector::Complex(value) = terminal else {
        panic!("terminal pseudo complex")
    };
    assert_eq!(
        CssComplexSelector::try_new(value.first().clone(), value.rest().to_vec()).unwrap(),
        value
    );
    let wrong_first = value.rest()[0].selector().clone();
    let error = CssComplexSelector::try_new(wrong_first, value.rest().to_vec()).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssSelectorConstructionErrorKind::NonTerminalPseudoElement { compound_index: 0 }
    );
}

#[test]
fn strict_and_forgiving_contexts_preserve_pseudo_exclusion_and_original_function_diagnostics() {
    let vendor = selector("::-webkit-x");
    assert!(CssPseudoSelectorList::try_new(vec![vendor.clone()]).is_err());
    assert!(
        CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(vendor.clone())]).is_none()
    );
    assert!(CssPseudoSelectorList::try_new_forgiving(vec![vendor]).is_err());
    for strict in [
        ":not(.Keep,::-webkit-x)",
        ":has(> ::-webkit-x)",
        ":nth-child(1 of ::-webkit-x)",
        "::slotted(::-webkit-x)",
    ] {
        reject(strict);
    }
    for function in ["is", "where"] {
        let report = parse_selector(
            &format!(":{function}(::-webkit-x,.Keep)"),
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
            format!(":{function}(.Keep)")
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropSelectorListItem)
        );
    }
    selector("::slotted(:has(>.child))::before"); // exact adopted Shadow/Has control
    selector("::slotted(:-webkit-autofill)::before");
    let list = parse_selector_list(
        ".Keep,::-webkit-jkl(),.Tail",
        &CssNamespaceContext::default(),
    );
    assert!(list.syntax().is_none());
    let scope = parse_sheet("@scope(::-webkit-x){.lost{color:red}}.after{color:blue}");
    assert!(matches!(scope.syntax().rules(), [CssRule::Style(_)]));
    assert!(
        scope
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropAtRule)
    );

    let raw = "/*😀*/\r\n/*é*/::-WeBkIt-JkL(x)";
    let fragment = parse_selector(raw, &CssNamespaceContext::default());
    let rejected = format!("{raw}{{}}");
    let sheet_source = format!("{rejected}.after{{color:red}}");
    let sheet = parse_sheet(&sheet_source);
    assert!(fragment.syntax().is_none());
    let [CssRule::Style(after)] = sheet.syntax().rules() else {
        panic!("later valid sibling survives")
    };
    assert_eq!(
        after.selectors().selectors()[0].selector(),
        &CssSelector::Class("after".into())
    );
    let name_offset = raw.find("-WeBkIt-JkL(").unwrap();
    let line_start = raw.find('\n').unwrap() + 1;
    for (diagnostics, action, span_start, span_end) in [
        (
            fragment.diagnostics(),
            CssRecoveryAction::RejectInput,
            0,
            raw.len(),
        ),
        (
            sheet.diagnostics(),
            CssRecoveryAction::DropQualifiedRule,
            raw.find(':').unwrap(),
            rejected.len(),
        ),
    ] {
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.action() == action)
            .unwrap();
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            name_offset
        );
        assert_eq!(diagnostic.error().position().line().value(), 1);
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            raw[line_start..name_offset].encode_utf16().count()
        );
        assert_eq!(diagnostic.span().start().byte_offset().value(), span_start);
        assert_eq!(diagnostic.span().end().byte_offset().value(), span_end);
        let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
            panic!("structured selector failure")
        };
        let token = detail.encountered().unwrap();
        assert_eq!(token.kind(), CssTokenKind::Function);
        assert_eq!(token.authored(), "-WeBkIt-JkL(");
    }
}

#[test]
fn canonical_pseudos_survive_nested_scoped_normalization_and_original_supports_components() {
    let source = "/*😀*/.Parent:-WEBKIT-AUTOFILL{color:red;@supports selector(.probe::-WebKit-X){color:blue;& .child::-WebKit-Y:focus{color:green}}}@scope(.root){.scoped::-WebKit-Z:hover{color:black}}";
    let expected = ".Parent:autofill { color: red; @supports selector(.probe::-WebKit-X) { color: blue; & .child::-webkit-y:focus { color: green; } } }\n@scope (.root) { .scoped::-webkit-z:hover { color: black; } }";
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
    for (ordinal, (value, css)) in declarations
        .iter()
        .zip(["red", "blue", "green", "black"])
        .enumerate()
    {
        assert_eq!(value.order(), ordinal);
        assert_eq!(
            value
                .source()
                .value_components()
                .serialize()
                .unwrap()
                .as_css()
                .trim(),
            css
        );
        assert_eq!(
            value.source().parsed_value().unwrap().source().as_str(),
            source
        );
        assert_eq!(
            value.source().position().unwrap().byte_offset().value(),
            source.find(&format!("color:{css}")).unwrap()
        );
    }
    let parent = declarations[0].selector_context();
    assert_eq!(
        parent.selectors()[0].selector(),
        &selector(".Parent:autofill")
    );
    assert!(declarations[1].selector_context().same_context(parent));
    let child = declarations[2].selector_context();
    assert!(child.parent().unwrap().same_context(parent));
    assert!(child.selectors()[0].selector().has_pseudo_elements());
    assert_eq!(
        child.selectors()[0].selector().to_specified_css().unwrap(),
        "& .child::-webkit-y:focus"
    );
    assert!(declarations[3].selector_context().scope_context().is_some());
    assert!(
        declarations[3].selector_context().selectors()[0]
            .selector()
            .has_pseudo_elements()
    );
    let supports = normalized
        .items()
        .iter()
        .find_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::Supports(value) => Some(value),
                _ => None,
            },
            _ => None,
        })
        .unwrap();
    assert!(
        matches!(supports.kind(), CssSupportsConditionKind::Selector(value) if value == &selector(".probe::-webkit-x"))
    );
    assert_eq!(
        supports.serialize().unwrap().as_css(),
        " selector(.probe::-WebKit-X)"
    );
    let function = supports
        .components()
        .iter()
        .find(|value| matches!(value.view(), CssComponentValueRef::Function(_)))
        .unwrap();
    let CssComponentValueRef::Function(function) = function.view() else {
        unreachable!()
    };
    let name = function.values().items().iter().find(|value| matches!(value.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) if name == "-WebKit-X")).unwrap();
    let CssValueOrigin::Parsed(origin) = name.origin() else {
        panic!("original selector probe token")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find("-WebKit-X").unwrap()
    );
    assert_eq!(
        origin.span().end().byte_offset().value(),
        source.find("-WebKit-X").unwrap() + "-WebKit-X".len()
    );
    let checked = CssSupportsCondition::try_from_components(
        CssComponentValues::try_new(supports.components().to_vec()).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    assert_eq!(&checked, supports);
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    let reused = CssSheet::try_from_rules(report.syntax().rules().to_vec()).unwrap();
    assert_eq!(reused.rules(), report.syntax().rules());
    assert_eq!(reused.to_specified_css().unwrap(), expected);
    assert_eq!(report, before);
    drop(report);
    assert_eq!(
        parent.selectors()[0].selector(),
        &selector(".Parent:autofill")
    );
    let replay = parse_sheet(expected);
    assert!(replay.is_clean(), "{replay:?}");
    assert_eq!(replay.syntax().to_specified_css().unwrap(), expected);
    for source in ["selector(:-webkit-autofill)", "selector(::-webkit-x)"] {
        let condition = CssSupportsCondition::try_from_components(
            parse_component_values(source).unwrap(),
            &CssNamespaceContext::default(),
        )
        .unwrap();
        assert!(matches!(
            condition.kind(),
            CssSupportsConditionKind::Selector(_)
        ));
    }
    let invalid = CssSupportsCondition::try_from_components(
        parse_component_values("selector(::-webkit-x())").unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    assert!(matches!(
        invalid.kind(),
        CssSupportsConditionKind::GeneralEnclosed(_)
    ));
}

#[test]
fn canonical_escaping_and_list_byte_budget_are_cumulative_and_atomic_for_checked_reuse() {
    let members = vec![
        selector(":-WEBKIT-AUTOFILL"),
        selector(r"::-WebKit-a\ b"),
        selector("::-WebKit-ΔA"),
    ];
    let expected = r":autofill, ::-webkit-a\ b, ::-webkit-Δa";
    let exact = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len());
    let list = CssSelectorList::try_new_with_limits(members.clone(), exact).unwrap();
    let before = list.clone();
    assert_eq!(list.to_specified_css_with_limits(exact).unwrap(), expected);
    let short =
        CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len() - 1);
    for member in &members {
        assert!(member.to_specified_css_with_limits(short).is_ok());
    }
    let error = CssSelectorList::try_new_with_limits(members.clone(), short).unwrap_err();
    let CssSelectorConstructionErrorKind::Specified(error) = error.kind() else {
        panic!("shared specified-byte construction failure")
    };
    assert_eq!(
        error.kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(
        list.to_specified_css_with_limits(short).unwrap_err().kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(list, before);
    assert_eq!(list.to_specified_css_with_limits(exact).unwrap(), expected);
    assert_eq!(CssSelectorList::try_new(members).unwrap(), list);
    let replay = parse_selector_list(expected, &CssNamespaceContext::default());
    assert!(replay.is_clean(), "{replay:?}");
    assert_eq!(
        replay
            .syntax()
            .as_ref()
            .unwrap()
            .selectors()
            .iter()
            .map(CssStyleSelector::selector)
            .collect::<Vec<_>>(),
        list.selectors().iter().collect::<Vec<_>>()
    );
}
