#![forbid(unsafe_code)]
//! Public expectations from CSSOM current fields, Nesting and the existing
//! native projection tariffs. Node composition is a new callable API, not RED.
use CssEditedGroupPreludeRef as Prelude;
use CssRuleCssomSerializationErrorKind as Error;
use CssRuleGraphInput as Input;
use CssSpecifiedDeclarationBlock as Block;
use CssSpecifiedValueSerializationErrorKind as Resource;
use CssSpecifiedValueSerializationLimits as Limits;
use surgeist_css::*;

fn ordinary(source: &str) -> Block {
    let report = parse_declaration_list_text(source);
    assert!(report.is_clean());
    Block::try_from_declarations(report.syntax()).unwrap()
}
fn text(input: Input<'_>) -> String {
    input
        .serialize_cssom(&CssNamespaceContext::default(), CssStyleAncestor::Absent)
        .unwrap()
}
fn format(input: Input<'_>, limits: Limits) -> Result<String, CssRuleCssomSerializationError> {
    input.serialize_cssom_with_limits(
        &CssNamespaceContext::default(),
        CssStyleAncestor::Absent,
        limits,
    )
}

#[test]
fn three_level_composition_spends_exact_native_budget_once_and_later_sibling_retry_is_atomic() {
    let query = parse_media_query_list("print").into_parts().0;
    let raw = parse_sheet("@import 'old.css' screen;");
    let before = raw.clone();
    let CssRule::Import(original) = &raw.syntax().rules()[0] else {
        panic!()
    };
    let empty_media = CssMediaQueryList::new(Vec::new());
    let name = CssKeyframesName::String(CssKeyframesString::new("k"));
    let children = [
        Input::keyframes(&name, &[]),
        Input::keyframes(&name, &[]),
        Input::import(original, &empty_media),
    ];
    let inner = [Input::group(Prelude::Media(&query), &children)];
    let middle = [Input::group(Prelude::Media(&query), &inner)];
    let root = Input::group(Prelude::Media(&query), &middle);
    let expected = "@media print {\n  @media print {\n  @media print {\n  @keyframes k { \n}\n  @keyframes k { \n}\n  @import url(\"old.css\");\n}\n}\n}";
    // Each media group: rule/list/query/type = 4; each empty keyframes:
    // rule/name = 2; import with empty current list: rule/target/list = 3.
    let exact = Limits::new(19, 19, expected.len());
    assert_eq!(format(root, exact).unwrap(), expected);
    for (limits, cause) in [
        (
            Limits::new(18, 19, expected.len()),
            Resource::InputNodeLimit,
        ),
        (
            Limits::new(19, 18, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
    ] {
        let error = format(root, limits).unwrap_err();
        assert_eq!(error.kind(), Error::Resource(cause));
        assert_eq!(error.rule_path(), &[0, 0, 2]);
        assert_eq!(format(root, exact).unwrap(), expected);
        assert_eq!(raw, before);
    }
    assert_eq!(
        format(root, Limits::new(19, 19, expected.len() - 1))
            .unwrap_err()
            .kind(),
        Error::Resource(Resource::ByteLimit)
    );
    assert_eq!(format(root, exact).unwrap(), expected);
    // Existing eager constructors keep their independent validation contracts.
    assert!(
        CssImportRuleView::try_new_with_limits(original, &empty_media, Limits::new(0, 0, 0))
            .is_err()
    );
    assert!(CssKeyframesRuleView::try_new_with_limits(&name, &[], Limits::new(0, 0, 0)).is_err());
}

#[test]
fn domain_mismatch_is_admitted_at_the_actual_child_path_and_empty_children_keep_their_tariff() {
    let query = parse_media_query_list("all").into_parts().0;
    let selectors = parse_selector_list(".x", &CssNamespaceContext::default())
        .into_parts()
        .0
        .unwrap();
    let valid = ordinary("");
    let wrong = Block::try_from_keyframe_entries(&[]).unwrap();
    for child in [
        Input::style(&selectors, &wrong, &[]),
        Input::nested_declarations(&wrong),
    ] {
        let children = [Input::nested_declarations(&valid), child];
        let root = Input::group(Prelude::Media(&query), &children);
        let failure = format(root, Limits::default()).unwrap_err();
        assert_eq!(
            failure.kind(),
            Error::Value(Resource::UnserializableBoundary)
        );
        assert_eq!(failure.rule_path(), &[1]);
    }
    let children = [
        Input::nested_declarations(&valid),
        Input::nested_declarations(&valid),
    ];
    let root = Input::group(Prelude::Media(&query), &children);
    let expected = "@media all {\n\n}";
    // Four native group nodes plus rule/block for each omitted child.
    assert_eq!(
        format(root, Limits::new(8, 8, expected.len())).unwrap(),
        expected
    );
    let failure = format(root, Limits::new(7, 8, expected.len())).unwrap_err();
    assert_eq!(failure.kind(), Error::Resource(Resource::InputNodeLimit));
    assert_eq!(failure.rule_path(), &[1]);
    assert_eq!(
        format(root, Limits::new(8, 8, expected.len())).unwrap(),
        expected
    );
}

#[test]
fn current_page_margin_and_keyframe_survivors_keep_order_domains_and_real_origins() {
    let source =
        String::from("/*😀*/ @page old { margin:1px 2px; @top-left{margin:3px 4px;content:'X'} }");
    let report = parse_sheet(&source);
    let before = report.clone();
    let CssRule::Page(page) = &report.syntax().rules()[0] else {
        panic!()
    };
    let old_page = page.declarations().try_specified().unwrap();
    let right = old_page.entries().iter().find(|e| matches!(e,CssSpecifiedPageDeclarationEntry::Property(p) if p.property_name()==CssPropertyNameRef::Known(CssKnownProperty::MarginRight))).unwrap().clone();
    let current_page = CssSpecifiedPageDeclarationBlock::try_from_entries(&[right]).unwrap();
    let old_margin = page.margin_rules()[0]
        .declarations()
        .try_specified_properties()
        .unwrap();
    let selected = old_margin
        .entries()
        .iter()
        .filter(|e| e.property_name() == CssPropertyNameRef::Known(CssKnownProperty::MarginLeft))
        .cloned()
        .collect::<Vec<_>>();
    let margin = Block::try_from_margin_entries(&selected).unwrap();
    let empty_margin = Block::try_from_margin_entries(&[]).unwrap();
    let margins = [
        CssMarginRuleView::try_new(CssMarginBox::BottomRight, &empty_margin).unwrap(),
        CssMarginRuleView::try_new(CssMarginBox::TopCenter, &margin).unwrap(),
    ];
    let selectors = parse_page_selector_list("invoice:first")
        .into_parts()
        .0
        .unwrap();
    let page_input = Input::page(CssPageRuleView::new(&selectors, &current_page, &margins));
    assert_eq!(
        text(page_input),
        "@page invoice:first { margin-right: 2px; @bottom-right { } @top-center { margin-left: 4px; } }"
    );
    assert!(
        margin.entries()[0]
            .source()
            .same_occurrence(selected[0].source())
    );
    assert_eq!(
        margin.entries()[0]
            .source()
            .parsed_value()
            .unwrap()
            .source()
            .as_str(),
        source
    );
    let key = parse_keyframe_rule("from {margin:1px 2px; opacity:0}")
        .into_parts()
        .0
        .unwrap();
    let original = Block::try_from_keyframe_declarations(key.block().declarations()).unwrap();
    let survivor = original
        .entries()
        .iter()
        .find(|e| e.property_name() == CssPropertyNameRef::Known(CssKnownProperty::MarginRight))
        .unwrap();
    let current = Block::try_from_keyframe_entries(std::slice::from_ref(survivor)).unwrap();
    let empty = Block::try_from_keyframe_entries(&[]).unwrap();
    let offsets = parse_keyframe_selector_list("50%, from, 50%")
        .into_parts()
        .0
        .unwrap();
    let children = [
        CssKeyframeRuleView::try_new(offsets.selectors(), &empty).unwrap(),
        CssKeyframeRuleView::try_new(offsets.selectors(), &current).unwrap(),
    ];
    let name = CssKeyframesName::String(CssKeyframesString::new("edited"));
    let input = Input::keyframes(&name, &children);
    assert_eq!(
        text(input),
        "@keyframes edited {   50%, 0%, 50% { }\n  50%, 0%, 50% { margin-right: 2px; }\n}"
    );
    assert!(
        current.entries()[0]
            .source()
            .same_occurrence(survivor.source())
    );
    assert_eq!(report, before);
    drop(source);
    assert!(margin.entries()[0].source().parsed_value().is_some());
}

#[test]
fn current_selected_font_face_uses_one_shared_block_writer_with_order_pending_empty_and_origins() {
    let source = String::from(
        "/*😀*/ font-family:Old; src:local(Old); font-display:swap; font-weight:env(weight)",
    );
    let report = parse_font_face_declaration_block_contents(&source);
    let occurrences = report
        .syntax()
        .as_ref()
        .unwrap()
        .occurrences()
        .cloned()
        .collect::<Vec<_>>();
    let current = CssSpecifiedFontFaceDeclarationBlock::try_from_entries(&[
        occurrences[3].clone(),
        occurrences[2].clone(),
    ])
    .unwrap();
    let input = Input::font_face(&current);
    let expected = "@font-face { font-weight: env(weight); font-display: swap; }";
    // Rule plus block/two entries/pending list,function,ident/display = 8.
    let exact = Limits::new(8, 8, expected.len());
    assert_eq!(format(input, exact).unwrap(), expected);
    assert_eq!(
        current.serialize_cssom().unwrap(),
        "font-weight: env(weight); font-display: swap;"
    );
    for (limits, kind) in [
        (Limits::new(7, 8, expected.len()), Resource::InputNodeLimit),
        (
            Limits::new(8, 7, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(8, 8, expected.len() - 1), Resource::ByteLimit),
    ] {
        assert_eq!(
            format(input, limits).unwrap_err().kind(),
            Error::Resource(kind)
        );
        assert_eq!(format(input, exact).unwrap(), expected);
    }
    assert_eq!(current.entries()[0].position(), occurrences[3].position());
    let CssAuthoredFontFaceDescriptorValue::Pending(pending) = current.entries()[0].value() else {
        panic!()
    };
    let CssValueOrigin::Parsed(origin) = pending.components().items()[0].origin() else {
        panic!()
    };
    assert_eq!(origin.source().as_str(), source);
    let empty = CssSpecifiedFontFaceDeclarationBlock::try_from_entries(&[]).unwrap();
    assert_eq!(
        format(Input::font_face(&empty), Limits::new(2, 2, 14)).unwrap(),
        "@font-face { }"
    );
    let query = parse_media_query_list("print").into_parts().0;
    let children = [Input::font_face(&empty), input];
    let root = Input::group(Prelude::Media(&query), &children);
    let expected = "@media print {\n  @font-face { }\n  @font-face { font-weight: env(weight); font-display: swap; }\n}";
    assert_eq!(
        format(root, Limits::new(14, 14, expected.len())).unwrap(),
        expected
    );
    let failure = format(root, Limits::new(13, 14, expected.len())).unwrap_err();
    assert_eq!(failure.rule_path(), &[1]);
    assert_eq!(failure.kind(), Error::Resource(Resource::InputNodeLimit));
    drop(source);
    assert!(origin.source().as_str().contains("font-family:Old"));
}

#[test]
fn namespace_and_external_style_ancestry_propagate_through_groups_and_restore_for_siblings() {
    let namespaces = CssNamespaceContext::from_bindings([
        (None, CssNamespaceName::new("urn:x")),
        (
            Some(CssNamespacePrefix::try_new("n").unwrap()),
            CssNamespaceName::new("urn:x"),
        ),
    ]);
    let selectors =
        parse_style_selector_list("n|leaf", &namespaces, CssStyleSelectorContext::Nested)
            .into_parts()
            .0
            .unwrap();
    let CssAdmittedStyleSelectors::Ordinary(list) = selectors.selectors() else {
        panic!()
    };
    let empty = ordinary("");
    let children = [Input::style(list, &empty, &[])];
    let query = parse_media_query_list("print").into_parts().0;
    let grouped = Input::group(Prelude::Media(&query), &children);
    assert_eq!(
        grouped
            .serialize_cssom(&namespaces, CssStyleAncestor::Present)
            .unwrap(),
        "@media print {\n  & leaf { }\n}"
    );
    let failure = grouped
        .serialize_cssom(&CssNamespaceContext::default(), CssStyleAncestor::Present)
        .unwrap_err();
    assert_eq!(failure.rule_path(), &[0]);
    assert_eq!(
        failure.kind(),
        Error::Value(Resource::NamespaceBindingUnavailable)
    );
    let outer_list = parse_selector_list(".outer", &namespaces)
        .into_parts()
        .0
        .unwrap();
    let root_children = [
        Input::style(&outer_list, &empty, &children),
        Input::style(&outer_list, &empty, &[]),
    ];
    let root = Input::group(Prelude::Media(&query), &root_children);
    assert_eq!(
        root.serialize_cssom(&namespaces, CssStyleAncestor::Absent)
            .unwrap(),
        "@media print {\n  .outer {\n  & leaf { }\n}\n  .outer { }\n}"
    );
    assert_eq!(selectors.origin().source().as_str(), "n|leaf");
}

#[test]
fn scoped_selectors_and_parsed_descendants_remain_in_their_actual_domains() {
    let namespaces = CssNamespaceContext::default();
    let raw = " /*😀*/ .child, > .next ";
    let selectors = parse_style_selector_list(
        raw,
        &namespaces,
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Present),
    )
    .into_parts()
    .0
    .unwrap();
    let CssAdmittedStyleSelectors::Scoped(list) = selectors.selectors() else {
        panic!()
    };
    let empty = ordinary("");
    let input = Input::scoped_style(list, &empty, &[]);
    assert_eq!(
        input
            .serialize_cssom(&namespaces, CssStyleAncestor::Present)
            .unwrap(),
        "& .child, & > .next { }"
    );
    assert_eq!(selectors.origin().source().as_str(), raw);
    let report = parse_sheet("@scope (.root) { .kept {color:red} }");
    let CssRule::Scope(scope) = &report.syntax().rules()[0] else {
        panic!()
    };
    let borrowed = Input::from_scoped_rule(&scope.rules().rules()[0]);
    assert_eq!(
        borrowed
            .serialize_cssom(&namespaces, CssStyleAncestor::Absent)
            .unwrap(),
        ".kept { color: red; }"
    );
}

#[test]
fn unavailable_wrappers_are_lazy_typed_descendants_and_do_not_hide_independent_payloads() {
    let query = parse_media_query_list("print").into_parts().0;
    let report = parse_sheet(
        "@scope (.a) { .b {color:red} } @counter-style ticks {system:cyclic;symbols:'x'}",
    );
    assert!(report.is_clean());
    let before = report.clone();
    for (rule, unavailable) in [
        (&report.syntax().rules()[0], CssRuleCssomFormat::Scope),
        (
            &report.syntax().rules()[1],
            CssRuleCssomFormat::CounterStyle,
        ),
    ] {
        let inputs = [Input::from_rule(rule)];
        let root = Input::group(Prelude::Media(&query), &inputs);
        let failure = format(root, Limits::default()).unwrap_err();
        assert_eq!(failure.kind(), Error::FormatUnavailable(unavailable));
        assert_eq!(failure.rule_path(), &[0]);
    }
    let CssRule::Scope(scope) = &report.syntax().rules()[0] else {
        panic!()
    };
    assert_eq!(
        scope
            .serialize_cssom_start(
                &CssNamespaceContext::default(),
                CssScopeNestingContext::None
            )
            .unwrap(),
        Some(".a".into())
    );
    assert_eq!(report, before);
}

#[test]
fn named_import_layer_partial_output_uses_the_native_component_meter_and_escaping() {
    let report = parse_sheet("@import 'x.css' layer(th\\65 me.f\\6f o); ");
    assert!(report.is_clean());
    let CssRule::Import(import) = &report.syntax().rules()[0] else {
        panic!()
    };
    let Some(CssImportLayer::Named(name)) = import.layer() else {
        panic!("{import:?}")
    };
    assert_eq!(name.components(), &["theme", "foo"]);
    let expected = "theme.foo";
    let exact = Limits::new(3, 3, expected.len());
    assert_eq!(
        name.serialize_specified_with_limits(exact).unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(2, 3, expected.len()), Resource::InputNodeLimit),
        (
            Limits::new(3, 2, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(3, 3, expected.len() - 1), Resource::ByteLimit),
    ] {
        assert_eq!(
            name.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(
            name.serialize_specified_with_limits(exact).unwrap(),
            expected
        );
    }
    assert_eq!(name.serialize_specified().unwrap(), expected);
    assert!(CssLayerName::try_new(Vec::<String>::new()).is_none());
    assert!(CssLayerName::try_new(["initial"]).is_none());
}

#[test]
fn graph_inputs_under_published_supports_keep_empty_visits_and_exact_atomic_meter() {
    let report = parse_sheet("@supports(display:grid){} .edited {color:red}");
    assert!(report.is_clean());
    let before = report.clone();
    let [CssRule::Supports(supports), CssRule::Style(style)] = report.syntax().rules() else {
        panic!()
    };
    let empty = ordinary("");
    let children = [
        Input::nested_declarations(&empty),
        Input::style(style.selectors(), &empty, &[]),
        Input::nested_declarations(&empty),
    ];
    let root = Input::group(Prelude::Supports(supports.condition()), &children);
    let expected = "@supports (display:grid) {\n  .edited { }\n}";
    // Supports rule/condition/list/block tokens = 6; each omitted current
    // NestedDeclarations costs 2; current Style rule/list/class/block costs 4.
    let exact = Limits::new(14, 14, expected.len());
    assert_eq!(format(root, exact).unwrap(), expected);
    for (limits, kind, path) in [
        (
            Limits::new(13, 14, expected.len()),
            Resource::InputNodeLimit,
            &[2][..],
        ),
        (
            Limits::new(14, 13, expected.len()),
            Resource::ProjectionNodeLimit,
            &[2][..],
        ),
        (
            Limits::new(14, 14, expected.len() - 1),
            Resource::ByteLimit,
            &[][..],
        ),
    ] {
        let error = format(root, limits).unwrap_err();
        assert_eq!(error.kind(), Error::Resource(kind));
        assert_eq!(error.rule_path(), path);
        assert_eq!(format(root, exact).unwrap(), expected);
        assert_eq!(report, before);
    }
    let omitted = [Input::nested_declarations(&empty)];
    assert_eq!(
        text(Input::group(
            Prelude::Supports(supports.condition()),
            &omitted
        )),
        "@supports (display:grid) {\n}"
    );
}

#[test]
fn published_supports_input_continuations_keep_namespace_ancestry_and_lazy_child_paths() {
    let namespaces = CssNamespaceContext::from_bindings([
        (None, CssNamespaceName::new("urn:x")),
        (
            Some(CssNamespacePrefix::try_new("n").unwrap()),
            CssNamespaceName::new("urn:x"),
        ),
    ]);
    let selected =
        parse_style_selector_list("n|leaf", &namespaces, CssStyleSelectorContext::Nested)
            .into_parts()
            .0
            .unwrap();
    let CssAdmittedStyleSelectors::Ordinary(selectors) = selected.selectors() else {
        panic!()
    };
    let report = parse_sheet("@supports(display:grid){} @scope(.host){.unused{color:red}}");
    let [CssRule::Supports(supports), scope] = report.syntax().rules() else {
        panic!()
    };
    let empty = ordinary("");
    let children = [Input::style(selectors, &empty, &[])];
    let inner = [Input::group(
        Prelude::Supports(supports.condition()),
        &children,
    )];
    let query = parse_media_query_list("print").into_parts().0;
    let root = Input::group(Prelude::Media(&query), &inner);
    assert_eq!(
        root.serialize_cssom(&namespaces, CssStyleAncestor::Present)
            .unwrap(),
        "@media print {\n  @supports (display:grid) {\n  & leaf { }\n}\n}"
    );
    let error = root
        .serialize_cssom(&CssNamespaceContext::default(), CssStyleAncestor::Present)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        Error::Value(Resource::NamespaceBindingUnavailable)
    );
    assert_eq!(error.rule_path(), &[0, 0]);
    let children = [Input::nested_declarations(&empty), Input::from_rule(scope)];
    let root = Input::group(Prelude::Supports(supports.condition()), &children);
    let error = format(root, Limits::default()).unwrap_err();
    assert_eq!(
        error.kind(),
        Error::FormatUnavailable(CssRuleCssomFormat::Scope)
    );
    assert_eq!(error.rule_path(), &[1]);
    assert_eq!(selected.origin().source().as_str(), "n|leaf");
}
