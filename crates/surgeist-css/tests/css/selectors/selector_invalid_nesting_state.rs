//! Functional contracts for parser-owned invalid nesting items. Selected Nesting
//! 1 §3.1, Selectors §16.1 and Syntax §5.4.10 own retention and token detection.
use crate::css::support::color_and_selector_fixtures::{
    assert_selector_input_rejected, assert_strict_validation_rejects,
    assert_unrepresentable_construction,
};
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::serialization_limits::LimitFailure;
use surgeist_css::*;

fn parsed(source: &str) -> CssSelector {
    parse_selector(source, &CssNamespaceContext::default())
        .syntax()
        .clone()
        .expect("retained selector")
}
fn logical(selector: &CssSelector) -> &CssPseudoSelectorList {
    let CssSelector::PseudoClass(CssPseudoClass::Is(list) | CssPseudoClass::Where(list)) = selector
    else {
        panic!("logical list")
    };
    list
}
fn invalid(item: &CssPseudoSelectorListItem) -> &CssInvalidNestingSelectorItem {
    let CssPseudoSelectorListItem::InvalidNesting(value) = item else {
        panic!("explicit invalid state")
    };
    value
}
fn has(selector: CssSelector) -> CssSelector {
    CssSelector::PseudoClass(CssPseudoClass::Has(
        CssRelativeSelectorList::try_new(
            vec![CssRelativeSelector::new(
                CssSelectorCombinator::Descendant,
                selector,
            )],
            Default::default(),
        )
        .unwrap(),
    ))
}
fn unrepresentable(selector: &CssSelector) {
    let before = selector.clone();
    assert!(
        selector
            .serialize_specified(Default::default())
            .unwrap_err()
            .is_unrepresentable_value()
    );
    assert_eq!(*selector, before);
}

#[test]
fn mixed_items_keep_source_order_complete_trivia_and_truthful_recovery() {
    let source = ":is(.A,\t/*L*/:NoPe(inner(&)) /*T*/ ,.B)";
    let report = parse_selector(source, &CssNamespaceContext::default());
    let selector = report.syntax().as_ref().unwrap();
    let [
        CssPseudoSelectorListItem::Selector(a),
        raw,
        CssPseudoSelectorListItem::Selector(b),
    ] = logical(selector).items()
    else {
        panic!("three ordered states")
    };
    assert_eq!(a, &CssSelector::Class("A".into()));
    assert_eq!(b, &CssSelector::Class("B".into()));
    let raw = invalid(raw);
    assert_eq!(raw.authored(), "\t/*L*/:NoPe(inner(&)) /*T*/ ");
    let span = raw.origin().span();
    assert_eq!(span.start().byte_offset().value(), 7);
    assert_eq!(span.end().byte_offset().value(), 35);
    assert_eq!(raw.origin().source().as_str(), source);
    assert_eq!(raw.components().component_count(), 9);
    assert_eq!(raw.components().nesting_depth(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("one original error")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::PreserveInvalidSelectorListItem
    );
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(
        selector.serialize_specified(Default::default()).unwrap(),
        ":is(.A,\t/*L*/:NoPe(inner(&)) /*T*/ , .B)"
    );
    assert_strict_validation_rejects(report);
}

#[test]
fn invalid_only_and_empty_and_valid_factories_are_distinct_states() {
    let selector = parsed(":where(:future(&))");
    assert_eq!(logical(&selector).items().len(), 1);
    assert_eq!(
        invalid(&logical(&selector).items()[0]).authored(),
        ":future(&)"
    );
    assert!(!logical(&selector).has_pseudo_elements());
    let empty = CssPseudoSelectorList::try_new_forgiving(Vec::new(), Default::default()).unwrap();
    assert!(empty.items().is_empty());
    assert_ne!(logical(&selector), &empty);
    let checked =
        CssPseudoSelectorList::try_new(vec![CssSelector::Class("A".into())], Default::default())
            .unwrap();
    assert_eq!(
        checked.items(),
        [CssPseudoSelectorListItem::Selector(CssSelector::Class(
            "A".into()
        ))]
    );
    assert_unrepresentable_construction(CssPseudoSelectorList::try_new_forgiving(
        vec![CssSelector::Class(String::new())],
        Default::default(),
    ));
}

#[test]
fn strict_not_and_nth_reject_cloned_invalid_state_atomically() {
    let list = logical(&parsed(":is(:future(&),.A)")).clone();
    for selector in [
        CssSelector::PseudoClass(CssPseudoClass::Not(list.clone())),
        CssSelector::PseudoClass(CssPseudoClass::NthChild(CssNthChildPattern::new(
            CssNthPattern::Odd,
            Some(list.clone()),
        ))),
        CssSelector::PseudoClass(CssPseudoClass::NthLastChild(CssNthChildPattern::new(
            CssNthPattern::Even,
            Some(list.clone()),
        ))),
    ] {
        unrepresentable(&selector);
        assert_unrepresentable_construction(CssSelectorList::try_new(
            vec![selector],
            Default::default(),
        ));
    }
    assert_eq!(
        CssSelector::PseudoClass(CssPseudoClass::Where(list))
            .serialize_specified(Default::default())
            .unwrap(),
        ":where(:future(&), .A)"
    );
}

#[test]
fn unchanged_and_stronger_receiving_contexts_preserve_the_original_invalid_state() {
    let selector = parsed(":is(:future(&))");
    assert_eq!(
        selector.serialize_specified(Default::default()).unwrap(),
        ":is(:future(&))"
    );
    assert_eq!(
        has(selector)
            .serialize_specified(Default::default())
            .unwrap(),
        ":has(:is(:future(&)))"
    );
    let source = ":has(:is(:has(&)))";
    let outer = parsed(source);
    assert_eq!(
        outer.serialize_specified(Default::default()).unwrap(),
        source
    );
    let CssSelector::PseudoClass(CssPseudoClass::Has(relative)) = &outer else {
        panic!("Has")
    };
    let inner = relative.selectors()[0].selector();
    assert_eq!(invalid(&logical(inner).items()[0]).authored(), ":has(&)");
    unrepresentable(inner); // Removing outer Has would re-enable the raw Has.
    assert_unrepresentable_construction(CssPseudoSelectorList::try_new(
        vec![inner.clone()],
        Default::default(),
    ));
    assert_eq!(
        outer.serialize_specified(Default::default()).unwrap(),
        source
    );
}

#[test]
fn compound_context_cannot_be_weakened_and_can_be_restored_at_its_owner() {
    let source = ":host(:is(.a &))";
    let host = parsed(source);
    assert_eq!(
        host.serialize_specified(Default::default()).unwrap(),
        source
    );
    let CssSelector::PseudoClass(CssPseudoClass::HostFunction(argument)) = &host else {
        panic!("Host")
    };
    let [CssPseudoClass::Is(list)] = argument.compound().pseudo_classes() else {
        panic!("inner Is")
    };
    assert_eq!(invalid(&list.items()[0]).authored(), ".a &");
    let detached = CssSelector::PseudoClass(CssPseudoClass::Is(list.clone()));
    unrepresentable(&detached);
    let ordinary = has(parsed(".A"));
    let CssSelector::PseudoClass(CssPseudoClass::Has(relative)) = ordinary else {
        panic!("Has")
    };
    // No mutation of checked relative members is available; receiving a compound
    // raw state through the ordinary relative factory already rejects weakening.
    assert_unrepresentable_construction(CssRelativeSelectorList::try_new(
        vec![CssRelativeSelector::new(
            CssSelectorCombinator::Descendant,
            detached.clone(),
        )],
        Default::default(),
    ));
    assert_eq!(
        relative.serialize_specified(Default::default()).unwrap(),
        ".A"
    );
    assert!(CssCompoundSelectorArgument::try_new(detached).is_some()); // Compound receiving context restores the proof.
    assert_eq!(
        has(host).serialize_specified(Default::default()).unwrap(),
        ":has(:host(:is(.a &)))"
    );
}

#[test]
fn raw_context_invalid_pseudo_elements_and_nested_has_are_not_admitted_children() {
    for source in [
        ":is(&::before,.A)",
        ":has(:where(:has(&),.A))",
        ":host(:is(.a &))",
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        let selector = report.syntax().as_ref().unwrap();
        assert!(!selector.has_pseudo_elements());
        assert_eq!(
            selector.serialize_specified(Default::default()).unwrap(),
            source.replace(",.A", ", .A")
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action()
                    == CssRecoveryAction::PreserveInvalidSelectorListItem)
        );
        assert_strict_validation_rejects(report);
    }
    for source in [
        ":has(:has(&))",
        ":not(&::before)",
        ":nth-child(odd of &::before)",
    ] {
        assert_selector_input_rejected(
            source,
            parse_selector(source, &CssNamespaceContext::default()),
        );
    }
}

#[test]
fn identity_compares_spelling_and_intrinsic_proof_without_source_coordinates() {
    let a = parsed(":is(:future(&))");
    let b = parsed("/*😀*/\r\n:where(:future(&))");
    let left = invalid(&logical(&a).items()[0]);
    let right = invalid(&logical(&b).items()[0]);
    assert_eq!(left, right);
    assert_ne!(left.origin().span(), right.origin().span());
    assert!(
        !left
            .origin()
            .source()
            .same_snapshot(right.origin().source())
    );
    let cloned = right.clone();
    assert!(
        cloned
            .origin()
            .source()
            .same_snapshot(right.origin().source())
    );
    let case = parsed(":is(:Future(&))");
    assert_ne!(left, invalid(&logical(&case).items()[0]));
    let restrictive = parsed(":has(:is(:future(&)))");
    let CssSelector::PseudoClass(CssPseudoClass::Has(list)) = &restrictive else {
        panic!("Has")
    };
    assert_ne!(
        left,
        invalid(&logical(list.selectors()[0].selector()).items()[0])
    );
}

#[test]
fn namespace_invalidity_remains_bound_to_original_parse_and_reparse_is_explicit() {
    let source = ":is(p|Leaf&,.A)";
    let original = parsed(source);
    let raw = invalid(&logical(&original).items()[0]);
    assert_eq!(raw.authored(), "p|Leaf&");
    let literal = ":is(p|Leaf&, .A)";
    assert_eq!(
        original.serialize_specified(Default::default()).unwrap(),
        literal
    );
    let context = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("p").unwrap()),
        CssNamespaceName::new("urn:p"),
    )]);
    let reparsed = parse_selector(literal, &context)
        .into_validation_result()
        .unwrap()
        .unwrap();
    assert!(matches!(
        logical(&reparsed).items()[0],
        CssPseudoSelectorListItem::Selector(CssSelector::Compound(_))
    ));
    assert_ne!(original, reparsed);
    assert_eq!(raw.authored(), "p|Leaf&");
    assert_eq!(
        original.serialize_specified(Default::default()).unwrap(),
        literal
    );
}

#[test]
fn implicit_eof_terminations_preserve_authored_chunks_and_emit_stable_complete_syntax() {
    for (source, authored, literal) in [
        (":is(:future(&", ":future(&", ":is(:future(&))"),
        (
            ":where(:future(&/*tail",
            ":future(&/*tail",
            ":where(:future(&/*tail*/))",
        ),
        (
            ":is(:future(&'tail",
            ":future(&'tail",
            ":is(:future(&'tail'))",
        ),
        (
            ":is(:future(&url(tail",
            ":future(&url(tail",
            ":is(:future(&url(tail)))",
        ),
        (
            ":is(:future(&tail\\",
            ":future(&tail\\",
            ":is(:future(&tail\\fffd ))",
        ),
        (
            ":is(:future(&'tail\\",
            ":future(&'tail\\",
            ":is(:future(&'tail\\\n'))",
        ),
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        let selector = report.syntax().as_ref().unwrap();
        assert_eq!(invalid(&logical(selector).items()[0]).authored(), authored);
        assert_eq!(
            selector.serialize_specified(Default::default()).unwrap(),
            literal
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action()
                    == CssRecoveryAction::RetainWithImplicitClosure)
        );
        let reparsed = parsed(literal);
        assert_eq!(
            reparsed.serialize_specified(Default::default()).unwrap(),
            literal
        );
        assert_strict_validation_rejects(report);
    }
}

#[test]
fn parsed_reverse_solidus_keeps_newline_or_escaped_token_termination() {
    let source = ":is(!&\\\n)";
    let selector = parsed(source);
    assert_eq!(invalid(&logical(&selector).items()[0]).authored(), "!&\\\n");
    // A newline after the delimiter makes this original token boundary safe.
    assert_eq!(
        selector.serialize_specified(Default::default()).unwrap(),
        source
    );
    let unsafe_source = ":is(!&\\)";
    let unsafe_selector = parsed(unsafe_source);
    // The escape consumes ')' into an identifier here: its EOF completion belongs
    // to the token owner, rather than a bare reverse-solidus output sentinel.
    assert_eq!(
        unsafe_selector
            .serialize_specified(Default::default())
            .unwrap(),
        ":is(!&\\))"
    );
}

#[test]
fn retained_component_graph_and_closings_share_exact_cumulative_limits_and_atomic_retry() {
    let source = ":is(:future(&),:other(&))";
    let selector = parsed(source);
    // Pseudo=1, two raw carriers=2, two graphs(colon,function,delim)=6.
    // Each actual function-closing traversal additionally costs one projection.
    let limits = CssSerializationLimits::new(9, 11, source.len());
    assert_eq!(selector.serialize_specified(limits).unwrap(), source);
    let before = selector.clone();
    for (limits, kind) in [
        (
            CssSerializationLimits::new(8, 11, source.len()),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(9, 10, source.len()),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(9, 11, source.len() - 1),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&selector.serialize_specified(limits).unwrap_err()),
            kind
        );
        assert_eq!(selector, before);
    }
    assert_eq!(
        selector
            .serialize_specified(CssSerializationLimits::new(9, 11, source.len()))
            .unwrap(),
        source
    );
}

#[test]
fn deep_unknown_components_are_charged_fully_and_parser_resource_errors_are_not_forgiven() {
    let depth = 64;
    let source = format!(":is(:future({}&{}))", "x(".repeat(depth), ")".repeat(depth));
    let selector = parsed(&source);
    // Raw graph: colon + future + depth x functions + delimiter. Closings depth+1.
    let input = depth + 5;
    let projection = input + depth + 1;
    let limits = CssSerializationLimits::new(input, projection, source.len());
    assert_eq!(selector.serialize_specified(limits).unwrap(), source);
    assert!(
        selector
            .serialize_specified(CssSerializationLimits::new(
                input - 1,
                projection,
                source.len()
            ))
            .unwrap_err()
            .is_input_node_limit()
    );
    let source = format!(":is(:future({}&{}))", "x(".repeat(255), ")".repeat(255));
    let report = parse_selector(&source, &CssNamespaceContext::default());
    assert!(report.syntax().is_none());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.error().code() == CssErrorCode::NestingLimit)
    );
    assert!(!report.diagnostics().iter().any(
        |diagnostic| diagnostic.action() == CssRecoveryAction::PreserveInvalidSelectorListItem
    ));
}

#[test]
fn retained_unknown_ampersand_marks_real_nested_selector_binding_before_normalization() {
    let report = parse_sheet(".Parent { :is(:future(&)) { color:red; } }");
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 1);
    let context = declarations[0].selector_context();
    assert!(context.parent().is_some());
    assert_eq!(
        context.selectors()[0].binding(),
        CssSelectorBinding::ExplicitAnchors
    );
    assert_eq!(
        context.selectors()[0]
            .selector()
            .serialize_specified(Default::default())
            .unwrap(),
        ":is(:future(&))"
    );
}

#[test]
fn suffix_context_relocation_obeys_element_backing_permissions() {
    fn suffix(selector: &CssSelector) -> CssPseudoClass {
        let CssSelector::Compound(compound) = selector else {
            panic!("pseudo compound")
        };
        let [_, CssPseudoElementSegment::PseudoClass(pseudo)] =
            compound.pseudo_elements().unwrap().segments()
        else {
            panic!("logical suffix")
        };
        pseudo.clone()
    }
    let part = parsed("::part(x):is(:future(&))");
    let pseudo = suffix(&part);
    assert_eq!(
        part.serialize_specified(Default::default()).unwrap(),
        "::part(x):is(:future(&))"
    );
    assert!(
        CssPseudoElementSequence::try_from_segments(vec![
            CssPseudoElementSegment::PseudoElement(CssPseudoElement::Before),
            CssPseudoElementSegment::PseudoClass(pseudo),
        ])
        .is_some()
    ); // Element-backed to non-element-backed tightens permission.
    let before = parsed("::before:is(:future(&))");
    assert_eq!(
        before.serialize_specified(Default::default()).unwrap(),
        "::before:is(:future(&))"
    );
    let pseudo = suffix(&before);
    unrepresentable(&CssSelector::PseudoClass(pseudo.clone()));
    assert!(
        CssPseudoElementSequence::try_from_segments(vec![
            CssPseudoElementSegment::PseudoElement(CssPseudoElement::Part(
                CssPartNameList::try_new(vec![CssPartName::try_new("x").unwrap()]).unwrap()
            )),
            CssPseudoElementSegment::PseudoClass(pseudo),
        ])
        .is_none()
    ); // Re-enabling element-backed permission would weaken it.
}

#[test]
fn url_string_comment_and_escaped_identifier_ampersands_are_not_delimiters() {
    for item in [
        ":future(url(&))",
        ":future(url('&'))",
        ":future(/*&*/x)",
        r":future(\&)",
        r":future(\26)",
    ] {
        let report = parse_selector(&format!(":is({item},.A)"), &CssNamespaceContext::default());
        assert_eq!(
            logical(report.syntax().as_ref().unwrap()).items(),
            [CssPseudoSelectorListItem::Selector(CssSelector::Class(
                "A".into()
            ))]
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropSelectorListItem
        );
    }
}

#[test]
fn trivia_unicode_bytes_and_real_rule_retry_are_cumulative() {
    let literal = ":is(:future(/*Ω*/&), .A)";
    let selector = parsed(literal);
    // Pseudo + raw carrier + colon/function/comment/delim + admitted Class = 7.
    assert_eq!(
        selector
            .serialize_specified(CssSerializationLimits::new(7, 8, literal.len()))
            .unwrap(),
        literal
    );
    assert!(
        selector
            .serialize_specified(CssSerializationLimits::new(7, 8, literal.len() - 1))
            .unwrap_err()
            .is_byte_limit()
    );
    let report = parse_sheet(":is(:future(/*Ω*/&),.A) {color:red;} .B {color:blue;}");
    let sheet = report.syntax();
    let css = sheet.serialize_specified(Default::default()).unwrap();
    let before = sheet.clone();
    assert_eq!(
        sheet
            .serialize_specified(CssSerializationLimits::new(
                usize::MAX,
                usize::MAX,
                css.len() - 1
            ))
            .unwrap_err()
            .limit_kind(),
        Some(SerializationFailure::ByteLimit)
    );
    assert_eq!(*sheet, before);
    assert_eq!(sheet.serialize_specified(Default::default()).unwrap(), css);
    assert!(css.contains(literal));
}
