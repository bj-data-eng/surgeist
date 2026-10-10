#![forbid(unsafe_code)]
//! Selected WebKit Supports frames compose the existing CSS-owned providers.
//! Work tariffs below derive from the retained component and rule contracts;
//! punctuation contributes bytes and no fabricated semantic nodes.

use std::error::Error;
use surgeist_css::{
    CssEditedGroupPreludeRef as Prelude, CssEditedRuleView as View, CssKeyframeRuleView,
    CssKeyframesName, CssKeyframesRuleView, CssKeyframesString, CssKnownProperty as K,
    CssMarginBox, CssMarginRuleView, CssNamespaceContext, CssPageRuleView, CssPropertyNameRef,
    CssRule, CssRuleCssomFormat, CssRuleCssomSerializationErrorKind as Kind,
    CssSpecifiedDeclarationBlock as Block, CssSpecifiedDeclarationEntry as Entry,
    CssSpecifiedPageDeclarationBlock as PageBlock, CssSpecifiedPageDeclarationEntry as PageEntry,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind as Resource,
    CssSpecifiedValueSerializationLimits as Limits, CssValueOrigin, parse_keyframe_selector_list,
    parse_page_selector_list, parse_rule, parse_sheet,
};

fn property(block: &Block, name: K) -> &Entry {
    block
        .entries()
        .iter()
        .find(|entry| entry.property_name() == CssPropertyNameRef::Known(name))
        .unwrap()
}

fn page_property(block: &PageBlock, name: K) -> &Entry {
    block
        .entries()
        .iter()
        .find_map(|entry| match entry {
            PageEntry::Property(entry)
                if entry.property_name() == CssPropertyNameRef::Known(name) =>
            {
                Some(entry)
            }
            _ => None,
        })
        .unwrap()
}

#[test]
fn parsed_supports_condition_and_child_share_exact_limits_and_retry_atomically() {
    let report = parse_rule(
        "@supports(display:grid){.x{}}",
        &CssNamespaceContext::default(),
    );
    assert!(report.is_clean());
    let rule = report.syntax().as_ref().unwrap();
    let before = rule.clone();
    let expected = "@supports (display:grid) {\n  .x { }\n}";
    // Supports rule 1 + condition aggregate 1 + block/ident/colon/ident 4 =6.
    // Empty Style rule 1 + selector list/class 2 + build/output aggregates 2 =5.
    let exact = Limits::new(11, 11, expected.len());
    assert_eq!(rule.serialize_cssom_with_limits(exact).unwrap(), expected);
    for (limits, kind, path) in [
        (
            Limits::new(10, 11, expected.len()),
            Resource::InputNodeLimit,
            &[0][..],
        ),
        (
            Limits::new(11, 10, expected.len()),
            Resource::ProjectionNodeLimit,
            &[0][..],
        ),
        (
            Limits::new(11, 11, expected.len() - 1),
            Resource::ByteLimit,
            &[][..],
        ),
        (
            Limits::new(11, 11, expected.len() - 3),
            Resource::ByteLimit,
            &[][..],
        ),
    ] {
        let error = rule.serialize_cssom_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), Kind::Resource(kind));
        assert_eq!(error.rule_path(), path);
        // Declaration admission can own the resource failure; preserve its
        // real wrapper while following the public chain to the typed cause.
        let mut source = error.source().unwrap();
        loop {
            if let Some(value) = source.downcast_ref::<CssSpecifiedValueSerializationError>() {
                assert_eq!(value.kind(), kind);
                break;
            }
            source = source.source().expect("typed resource provider");
        }
        assert_eq!(rule, &before);
        assert_eq!(rule.serialize_cssom_with_limits(exact).unwrap(), expected);
    }
}

#[test]
fn omitted_empty_children_consume_work_and_preserve_the_next_child_failure_index() {
    let report = parse_sheet("@supports(display:grid){} .x{}");
    assert!(report.is_clean());
    let [CssRule::Supports(supports), child] = report.syntax().rules() else {
        panic!("rules");
    };
    let empty = Block::try_from_entries(&[]).unwrap();
    let empty_child = View::try_nested_declarations(&empty).unwrap();
    let children = [empty_child, View::from_rule(child), empty_child];
    let group = View::try_group(Prelude::Supports(supports.condition()), &children).unwrap();
    let expected = "@supports (display:grid) {\n  .x { }\n}";
    // The parsed payload costs I11/P11. Each empty live view adds its real rule
    // and declaration aggregate: I2/P2, with no prefix or child output bytes.
    let exact = Limits::new(15, 15, expected.len());
    assert_eq!(group.serialize_cssom_with_limits(exact).unwrap(), expected);
    for (limits, kind, path) in [
        (
            Limits::new(8, 15, expected.len()),
            Resource::InputNodeLimit,
            &[1][..],
        ),
        (
            Limits::new(15, 8, expected.len()),
            Resource::ProjectionNodeLimit,
            &[1][..],
        ),
        (
            Limits::new(14, 15, expected.len()),
            Resource::InputNodeLimit,
            &[2][..],
        ),
        (
            Limits::new(15, 14, expected.len()),
            Resource::ProjectionNodeLimit,
            &[2][..],
        ),
        (
            Limits::new(15, 15, expected.len() - 1),
            Resource::ByteLimit,
            &[][..],
        ),
    ] {
        let error = group.serialize_cssom_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), Kind::Resource(kind));
        assert_eq!(error.rule_path(), path);
        assert_eq!(group.serialize_cssom_with_limits(exact).unwrap(), expected);
        assert_eq!(children[0].serialize_cssom().unwrap(), "");
        assert_eq!(children[2].serialize_cssom().unwrap(), "");
    }
}

#[test]
fn all_empty_children_have_exact_bytes_but_retain_their_own_work_tariffs() {
    let report = parse_sheet("@supports(display:grid){}");
    let CssRule::Supports(supports) = &report.syntax().rules()[0] else {
        panic!("Supports")
    };
    let empty = Block::try_from_entries(&[]).unwrap();
    let child = View::try_nested_declarations(&empty).unwrap();
    let children = [child, child];
    let group = View::try_group(Prelude::Supports(supports.condition()), &children).unwrap();
    let expected = "@supports (display:grid) {\n}";
    assert_eq!(
        group
            .serialize_cssom_with_limits(Limits::new(10, 10, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(9, 10, expected.len()), Resource::InputNodeLimit),
        (
            Limits::new(10, 9, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(10, 10, expected.len() - 1), Resource::ByteLimit),
    ] {
        assert_eq!(
            group
                .serialize_cssom_with_limits(limits)
                .unwrap_err()
                .kind(),
            Kind::Resource(kind)
        );
        assert_eq!(group.serialize_cssom().unwrap(), expected);
    }
}

#[test]
fn unavailable_descendants_keep_their_typed_cause_and_index_after_an_empty_child() {
    let report = parse_sheet("@supports(display:grid){} @container(width>1px){}");
    assert!(report.is_clean());
    let [CssRule::Supports(supports), container] = report.syntax().rules() else {
        panic!("rules")
    };
    let empty = Block::try_from_entries(&[]).unwrap();
    let children = [
        View::try_nested_declarations(&empty).unwrap(),
        View::from_rule(container),
    ];
    let group = View::try_group(Prelude::Supports(supports.condition()), &children).unwrap();
    for _ in 0..2 {
        let error = group.serialize_cssom().unwrap_err();
        assert_eq!(
            error.kind(),
            Kind::FormatUnavailable(CssRuleCssomFormat::Container)
        );
        assert_eq!(error.rule_path(), &[1]);
        assert!(error.source().is_none());
    }
    assert_eq!(
        group.to_specified_css().unwrap(),
        "@supports (display:grid) { @container (width>1px) { } }"
    );
    assert_eq!(children[0].serialize_cssom().unwrap(), "");
}

#[test]
fn supports_preserves_actual_sheet_namespace_bindings_and_detached_symbolic_spelling() {
    let source =
        "/*😀*/ @namespace n 'urn:x'; @namespace 'urn:x'; @supports(display:grid){n|leaf[|a]{}}";
    let report = parse_sheet(source);
    assert!(report.is_clean());
    let before = report.clone();
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        "@namespace n url(\"urn:x\");\n@namespace url(\"urn:x\");\n@supports (display:grid) {\n  leaf[a] { }\n}"
    );
    let supports = &report.syntax().rules()[2];
    assert_eq!(
        supports.serialize_cssom().unwrap(),
        "@supports (display:grid) {\n  n|leaf[a] { }\n}"
    );
    let CssRule::Supports(supports) = supports else {
        panic!("Supports")
    };
    let CssValueOrigin::Parsed(origin) = supports.condition().components()[0].origin() else {
        panic!("parsed origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(report, before);
}

#[test]
fn nested_supports_preserves_logical_and_future_condition_tokens_without_simplification() {
    let report = parse_sheet(
        "@supports ((display:grid) or (display:flex)) and (future-token(foo)) { @supports not (color:red) { .x{} } }",
    );
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let expected = "@supports ((display:grid) or (display:flex)) and (future-token(foo)) {\n  @supports not (color:red) {\n  .x { }\n}\n}";
    assert_eq!(report.syntax().serialize_cssom().unwrap(), expected);
    assert_eq!(report.syntax().serialize_cssom().unwrap(), expected);
    assert_eq!(report, before);
}

#[test]
fn supports_composes_current_page_margin_and_keyframe_edits_without_source_reconstruction() {
    let source = "@supports(display:grid){@page old{margin:1px 2px 3px 4px;size:A4;@top-left{margin:5px 6px}} @keyframes old{from{margin:1px 2px}}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let CssRule::Supports(supports) = &report.syntax().rules()[0] else {
        panic!("Supports")
    };
    let [CssRule::Page(page), CssRule::Keyframes(keyframes)] = supports.rules() else {
        panic!("children")
    };
    let original_page = page.declarations().try_specified().unwrap();
    let right = page_property(&original_page, K::MarginRight);
    let size = original_page
        .entries()
        .iter()
        .find(|entry| matches!(entry, PageEntry::Descriptor(_)))
        .unwrap()
        .clone();
    let current_page =
        PageBlock::try_from_entries(&[PageEntry::Property(right.clone()), size]).unwrap();
    let original_margin = page.margin_rules()[0]
        .declarations()
        .try_specified_properties()
        .unwrap();
    let left = property(&original_margin, K::MarginLeft);
    let current_margin = Block::try_from_margin_entries(std::slice::from_ref(left)).unwrap();
    let margins = [CssMarginRuleView::try_new(CssMarginBox::TopCenter, &current_margin).unwrap()];
    let selectors = parse_page_selector_list(" /*edit*/ invoice:left ");
    assert!(selectors.is_clean());
    let page_view = View::try_page(CssPageRuleView::new(
        selectors.syntax().as_ref().unwrap(),
        &current_page,
        &margins,
    ))
    .unwrap();

    let original_keyframe =
        Block::try_from_keyframe_declarations(keyframes.blocks()[0].declarations()).unwrap();
    let keyframe_right = property(&original_keyframe, K::MarginRight);
    let current_keyframe =
        Block::try_from_keyframe_entries(std::slice::from_ref(keyframe_right)).unwrap();
    let keys = parse_keyframe_selector_list(" /*edit*/ 50% ");
    assert!(keys.is_clean());
    let blocks = [CssKeyframeRuleView::try_new(
        keys.syntax().as_ref().unwrap().selectors(),
        &current_keyframe,
    )
    .unwrap()];
    let name = CssKeyframesName::String(CssKeyframesString::new("current"));
    let keyframes_view =
        View::try_keyframes(CssKeyframesRuleView::try_new(&name, &blocks).unwrap()).unwrap();
    let children = [keyframes_view, page_view, keyframes_view];
    let group = View::try_group(Prelude::Supports(supports.condition()), &children).unwrap();
    let expected = "@supports (display:grid) {\n  @keyframes current {   50% { margin-right: 2px; }\n}\n  @page invoice:left { margin-right: 2px; size: a4; @top-center { margin-left: 6px; } }\n  @keyframes current {   50% { margin-right: 2px; }\n}\n}";
    assert_eq!(group.serialize_cssom().unwrap(), expected);
    assert_eq!(
        group
            .serialize_cssom_with_limits(Limits::new(10000, 100000, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        group
            .serialize_cssom_with_limits(Limits::new(10000, 100000, expected.len() - 1))
            .unwrap_err()
            .kind(),
        Kind::Resource(Resource::ByteLimit)
    );
    assert_eq!(group.serialize_cssom().unwrap(), expected);
    assert!(
        page_property(&current_page, K::MarginRight)
            .source()
            .same_occurrence(right.source())
    );
    assert!(
        property(&current_margin, K::MarginLeft)
            .source()
            .same_occurrence(left.source())
    );
    assert!(
        current_keyframe.entries()[0]
            .source()
            .same_occurrence(keyframe_right.source())
    );
    assert_eq!(
        selectors
            .syntax()
            .as_ref()
            .unwrap()
            .origin()
            .unwrap()
            .source()
            .as_str(),
        " /*edit*/ invoice:left "
    );
    assert_eq!(
        keys.syntax().as_ref().unwrap().origin().source().as_str(),
        " /*edit*/ 50% "
    );
    assert_eq!(report, before);
}
