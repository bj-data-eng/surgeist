//! Current immutable payload composition. Expected wrappers follow the selected
//! CSSOM 2026 keyframes branch and adopted Blink Page/margin witness; four-side
//! terminal values follow CSS2's independent margin assignment rule.
use crate::css::support::failure_classes::SerializationFailure as Resource;
use crate::css::support::serialization_limits::LimitFailure;
use crate::css::support::{graph_cssom, graph_specified};
use surgeist_css::CssParser;
use surgeist_css::declaration::CssImportance;
use surgeist_css::declaration::CssSpecifiedDeclarationBlock as Block;
use surgeist_css::declaration::CssSpecifiedDeclarationEntry as Entry;
use surgeist_css::property::CssKnownProperty as K;
use surgeist_css::property::CssPropertyNameRef;
use surgeist_css::rule::CssRuleAdmissionContext as Context;
use surgeist_css::rule::keyframes::{
    CssKeyframeRuleView, CssKeyframesName, CssKeyframesRuleView, CssKeyframesString,
    CssParsedKeyframeRule, CssParsedKeyframeSelectors,
};
use surgeist_css::rule::page::CssSpecifiedPageDeclarationBlock as PageBlock;
use surgeist_css::rule::page::CssSpecifiedPageDeclarationEntry as PageEntry;
use surgeist_css::rule::page::{
    CssMarginBox, CssMarginRuleView, CssPageBody, CssPageRuleView, CssPageSelectorList,
};
use surgeist_css::rule::{CssBlockContext, CssBlockFragment, CssRule, CssRuleSyntax};
use surgeist_css::selector::CssNamespaceContext;
use surgeist_css::serialization::CssGroupPreludeRef as Prelude;
use surgeist_css::serialization::CssRuleGraphInput as View;
use surgeist_css::serialization::CssSerializationLimits as Limits;

fn terminal(block: &Block, property: K) -> &Entry {
    block
        .entries()
        .iter()
        .find(|entry| entry.property_name() == CssPropertyNameRef::Known(property))
        .unwrap()
}
fn page_property(block: &PageBlock, property: K) -> &Entry {
    block
        .entries()
        .iter()
        .find_map(|entry| match entry {
            PageEntry::Property(entry)
                if entry.property_name() == CssPropertyNameRef::Known(property) =>
            {
                Some(entry)
            }
            _ => None,
        })
        .unwrap()
}
fn group_query() -> surgeist_css::query::media::CssMediaQueryList {
    let report = CssParser::default().parse_sheet("@media print {}");
    let CssRule::Media(media) = &report.syntax().rules()[0] else {
        panic!("media")
    };
    media.query().clone()
}

#[test]
fn contextual_admission_establishes_page_and_keyframes_group_permission_not_style_permission() {
    // Conditional3 allows ordinary rule lists; Nesting1 restricts rules with style
    // ancestry. Formatting views deliberately do not grant insertion permission.
    for source in ["@page invoice{}", "@keyframes k{from{}}"] {
        let classified = CssRuleSyntax::classify(&CssParser::default(), source);
        assert!(classified.is_clean());
        let candidate = classified.syntax().as_ref().unwrap();
        let admitted = candidate.admit(&CssNamespaceContext::default(), Context::Group);
        assert!(admitted.is_clean(), "{source}: {admitted:?}");
        assert!(matches!(
            admitted.syntax(),
            Some(CssRule::Page(_) | CssRule::Keyframes(_))
        ));
        let rejected = candidate.admit(&CssNamespaceContext::default(), Context::Style);
        crate::css::support::value_assertions::assert_first_code(
            rejected.diagnostics(),
            surgeist_css::diagnostic::CssErrorCode::InvalidAtRulePlacement,
        );
        assert!(rejected.syntax().is_none());
    }
}

#[test]
fn recursive_page_preserves_mixed_selected_order_shorthand_survivors_pending_and_margin_domain() {
    let source = "/*😀*/ @media print { @page old { margin:1px 2px; size:A4; @top-left { margin:3px 4px; content:'X'; } } }";
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let CssRule::Media(media) = &report.syntax().rules()[0] else {
        panic!("media")
    };
    let CssRule::Page(page) = &media.rules()[0] else {
        panic!("Page")
    };
    let selected = page
        .declarations()
        .try_specified(Default::default())
        .unwrap();
    let right = page_property(&selected, K::MarginRight);
    let descriptor = selected
        .entries()
        .iter()
        .find(|v| matches!(v, PageEntry::Descriptor(_)))
        .unwrap()
        .clone();
    let raw = CssBlockFragment::<CssPageBody>::parse(
        &CssParser::default(),
        "{ margin-top:var(--edited); --Case:RAW; }",
        &CssBlockContext::default(),
    );
    assert!(raw.is_clean());
    let raw_selected = raw
        .syntax()
        .as_ref()
        .unwrap()
        .body()
        .declarations()
        .try_specified(Default::default())
        .unwrap();
    let pending = page_property(&raw_selected, K::MarginTop);
    let custom = raw_selected.entries().iter().find(|v| matches!(v, PageEntry::Property(v) if matches!(v.property_name(), CssPropertyNameRef::Custom(_)))).unwrap().clone();
    let current = PageBlock::try_from_entries(
        &[
            PageEntry::Property(right.with_importance(CssImportance::Important)),
            descriptor,
            custom,
            PageEntry::Property(pending.clone()),
        ],
        Default::default(),
    )
    .unwrap();
    let old_margin = page.margin_rules()[0]
        .declarations()
        .try_specified_properties(Default::default())
        .unwrap();
    let left = terminal(&old_margin, K::MarginLeft);
    let current_margin = Block::try_from_margin_entries(
        &[left.clone(), terminal(&old_margin, K::Content).clone()],
        Default::default(),
    )
    .unwrap();
    let empty_margin = Block::try_from_margin_entries(&[], Default::default()).unwrap();
    let margins = [
        CssMarginRuleView::try_new(CssMarginBox::BottomRight, &empty_margin).unwrap(),
        CssMarginRuleView::try_new(CssMarginBox::TopCenter, &current_margin).unwrap(),
    ];
    let raw_selectors =
        CssPageSelectorList::parse(&CssParser::default(), " /*edit*/ invoice:first:left ");
    assert!(raw_selectors.is_clean());
    let selectors = raw_selectors.syntax().as_ref().unwrap();
    let leaf = View::page(CssPageRuleView::new(selectors, &current, &margins));
    let children = [leaf];
    let group = View::group(Prelude::Media(media.query()), &children);
    let expected = "@media print {\n  @page invoice:first:left { margin-right: 2px !important; size: a4; --Case: RAW; margin-top: var(--edited); @bottom-right { } @top-center { margin-left: 4px; content: \"X\"; } }\n}";
    assert_eq!(graph_cssom(group, Default::default()).unwrap(), expected);
    assert!(
        page_property(&current, K::MarginRight)
            .source()
            .same_occurrence(right.source())
    );
    assert!(
        page_property(&current, K::MarginTop)
            .source()
            .same_occurrence(pending.source())
    );
    assert!(
        terminal(&current_margin, K::MarginLeft)
            .source()
            .same_occurrence(left.source())
    );
    assert_eq!(
        selectors.origin().unwrap().source().as_str(),
        " /*edit*/ invoice:first:left "
    );
    assert_eq!(
        page_property(&current, K::MarginRight)
            .source()
            .parsed_value()
            .unwrap()
            .source()
            .as_str(),
        source
    );
    assert_eq!(
        page_property(&current, K::MarginTop)
            .source()
            .parsed_value()
            .unwrap()
            .source()
            .as_str(),
        "{ margin-top:var(--edited); --Case:RAW; }"
    );
    let wrong_domain =
        Block::try_from_entries(current_margin.entries(), Default::default()).unwrap();
    assert!(CssMarginRuleView::try_new(CssMarginBox::TopLeft, &wrong_domain).is_none());
    assert_eq!(report, before);
    let failure =
        graph_cssom(group, Limits::new(100_000, 100_000, expected.len() - 5)).unwrap_err();
    assert!(
        failure.is_resource_limit()
            && failure
                .serialization_error()
                .is_some_and(|error| error.is_byte_limit()),
        "{failure:?}"
    );
    assert_eq!(failure.rule_path(), &[0, 1]);
    assert_eq!(graph_cssom(group, Default::default()).unwrap(), expected);
    assert_eq!(report, before);
}

#[test]
fn whole_keyframes_preserves_renamed_edited_deleted_and_appended_duplicate_children() {
    let source =
        "/*😀*/ @media print { @keyframes old { from {margin:1px 2px;opacity:0} to {opacity:1} } }";
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean());
    let before = report.clone();
    let CssRule::Media(media) = &report.syntax().rules()[0] else {
        panic!("media")
    };
    let CssRule::Keyframes(original) = &media.rules()[0] else {
        panic!("keyframes")
    };
    let first = &original.blocks()[0];
    let old =
        Block::try_from_keyframe_declarations(first.declarations(), Default::default()).unwrap();
    let right = terminal(&old, K::MarginRight);
    let edited =
        Block::try_from_keyframe_entries(std::slice::from_ref(right), Default::default()).unwrap();
    let key_text =
        CssParsedKeyframeSelectors::parse(&CssParser::default(), " /*edit*/ 50%, from, 50% ");
    assert!(key_text.is_clean());
    let current_first =
        CssKeyframeRuleView::try_new(key_text.syntax().as_ref().unwrap().selectors(), &edited)
            .unwrap();
    let appended = CssParsedKeyframeRule::parse(
        &CssParser::default(),
        " /*append*/ 50%, from, 50% {opacity:.5;}",
    );
    assert!(appended.is_clean());
    let appended_block = appended.syntax().as_ref().unwrap().block();
    let appended_declarations =
        Block::try_from_keyframe_declarations(appended_block.declarations(), Default::default())
            .unwrap();
    let current_appended =
        CssKeyframeRuleView::try_new(appended_block.selectors(), &appended_declarations).unwrap();
    // Consumer removed the original 'to' block, reordered the survivors, and
    // appended a duplicate keyText. Neither provider searches nor deduplicates.
    let children = [current_appended, current_first, current_appended];
    let name = CssKeyframesName::String(CssKeyframesString::new("fade in"));
    let whole = CssKeyframesRuleView::try_new(&name, &children, Default::default()).unwrap();
    assert_eq!(whole.name(), &name);
    assert_eq!(whole.rules().len(), 3);
    assert!(std::ptr::eq(
        whole.rules()[1].selectors(),
        key_text.syntax().as_ref().unwrap().selectors()
    ));
    assert!(std::ptr::eq(
        whole.rules()[2].declarations(),
        &appended_declarations
    ));
    let expected = "@keyframes fade\\ in {   50%, 0%, 50% { opacity: 0.5; }\n  50%, 0%, 50% { margin-right: 2px; }\n  50%, 0%, 50% { opacity: 0.5; }\n}";
    assert_eq!(whole.serialize_cssom(Default::default()).unwrap(), expected);
    assert!(
        whole.rules()[1].declarations().entries()[0]
            .source()
            .same_occurrence(right.source())
    );
    assert_eq!(
        appended_block.declarations()[0]
            .source()
            .parsed_value()
            .unwrap()
            .source()
            .as_str(),
        " /*append*/ 50%, from, 50% {opacity:.5;}"
    );
    assert_eq!(
        key_text
            .syntax()
            .as_ref()
            .unwrap()
            .origin()
            .source()
            .as_str(),
        " /*edit*/ 50%, from, 50% "
    );
    let leaf = View::keyframes(&name, &children);
    let group_children = [leaf];
    let group = View::group(Prelude::Media(media.query()), &group_children);
    assert_eq!(
        graph_cssom(group, Default::default()).unwrap(),
        format!("@media print {{\n  {expected}\n}}")
    );
    let failure = whole
        .serialize_cssom(Limits::new(100_000, 100_000, expected.len() - 6))
        .unwrap_err();
    assert!(
        failure.is_resource_limit()
            && failure
                .serialization_error()
                .is_some_and(|error| error.is_byte_limit()),
        "{failure:?}"
    );
    assert_eq!(failure.keyframe_block_index(), Some(2));
    assert_eq!(failure.rule_path(), &[] as &[usize]);
    assert_eq!(whole.serialize_cssom(Default::default()).unwrap(), expected);
    assert_eq!(report, before);
}

#[test]
fn empty_whole_keyframes_and_empty_children_keep_selected_name_classification_and_domain_proof() {
    let empty = Block::try_from_keyframe_entries(&[], Default::default()).unwrap();
    let raw = CssParsedKeyframeSelectors::parse(&CssParser::default(), "from, from");
    let child =
        CssKeyframeRuleView::try_new(raw.syntax().as_ref().unwrap().selectors(), &empty).unwrap();
    let children = [child, child];
    for (name, output) in [
        ("INITIAL", "\"INITIAL\""),
        ("fade in", "fade\\ in"),
        ("", ""),
    ] {
        let name = CssKeyframesName::String(CssKeyframesString::new(name));
        let whole = CssKeyframesRuleView::try_new(&name, &[], Default::default()).unwrap();
        assert_eq!(
            whole.serialize_cssom(Default::default()).unwrap(),
            format!("@keyframes {output} {{ \n}}")
        );
    }
    let name = CssKeyframesName::String(CssKeyframesString::new("none"));
    let whole = CssKeyframesRuleView::try_new(&name, &children, Default::default()).unwrap();
    assert_eq!(
        whole.serialize_cssom(Default::default()).unwrap(),
        "@keyframes \"none\" {   0%, 0% { }\n  0%, 0% { }\n}"
    );
    let ordinary = Block::try_from_entries(&[], Default::default()).unwrap();
    assert!(CssKeyframeRuleView::try_new(child.selectors(), &ordinary).is_err());
}

#[test]
fn coupled_recursive_payloads_share_input_projection_and_output_limits_and_allow_unchanged_retry() {
    let selectors = CssPageSelectorList::parse(&CssParser::default(), "");
    let page_block = PageBlock::try_from_entries(&[], Default::default()).unwrap();
    let page = View::page(CssPageRuleView::new(
        selectors.syntax().as_ref().unwrap(),
        &page_block,
        &[],
    ));
    let name = CssKeyframesName::String(CssKeyframesString::new("k"));
    let keyframes = View::keyframes(&name, &[]);
    let query = group_query();
    let children = [page, keyframes, page, keyframes];
    let inner = View::group(Prelude::Media(&query), &children);
    let outer_children = [inner];
    let outer = View::group(Prelude::Media(&query), &outer_children);
    let expected = "@media print {\n  @media print {\n  @page { }\n  @keyframes k { \n}\n  @page { }\n  @keyframes k { \n}\n}\n}";
    assert_eq!(graph_cssom(outer, Default::default()).unwrap(), expected);
    // An isolated empty payload fits ten nodes; the composed traversal does not.
    let ten = Limits::new(10, 10, 1000);
    assert!(graph_cssom(page, ten).is_ok());
    assert!(graph_cssom(keyframes, ten).is_ok());
    for (limits, kind) in [
        (Limits::new(10, 100_000, 1000), Resource::InputNodeLimit),
        (
            Limits::new(100_000, 10, 1000),
            Resource::ProjectionNodeLimit,
        ),
        (
            Limits::new(100_000, 100_000, expected.len() - 1),
            Resource::ByteLimit,
        ),
    ] {
        assert_eq!(
            graph_cssom(outer, limits).unwrap_err().limit_kind(),
            Some(kind)
        );
        assert_eq!(graph_cssom(outer, Default::default()).unwrap(), expected);
        if kind != Resource::ByteLimit {
            crate::css::support::value_assertions::assert_rule_resource(
                graph_specified(outer, limits),
                kind,
            );
        }
    }
    // Compact specified output has a wrapper byte count that differs from literal
    // CSSOM. A separate byte boundary checks that phase.
    use crate::css::support::value_assertions::assert_rule_resource;
    assert_rule_resource(
        graph_specified(outer, Limits::new(100_000, 100_000, 1)),
        Resource::ByteLimit,
    );
    assert_rule_resource(
        graph_specified(
            View::page(CssPageRuleView::new(
                selectors.syntax().as_ref().unwrap(),
                &page_block,
                &[],
            )),
            Limits::new(0, 100, 100),
        ),
        Resource::InputNodeLimit,
    );
    assert_rule_resource(
        CssKeyframesRuleView::try_new(&name, &[], Limits::new(100, 0, 100)),
        Resource::ProjectionNodeLimit,
    );
    assert_rule_resource(
        graph_specified(View::keyframes(&name, &[]), Limits::new(100, 100, 1)),
        Resource::ByteLimit,
    );
}

#[test]
fn whole_keyframes_exact_empty_child_work_is_cumulative_not_restarted() {
    let declarations = Block::try_from_keyframe_entries(&[], Default::default()).unwrap();
    let raw = CssParsedKeyframeSelectors::parse(&CssParser::default(), "from, from");
    let child =
        CssKeyframeRuleView::try_new(raw.syntax().as_ref().unwrap().selectors(), &declarations)
            .unwrap();
    let children = [child, child];
    let name = CssKeyframesName::String(CssKeyframesString::new("k"));
    let expected = "@keyframes k {   0%, 0% { }\n  0%, 0% { }\n}";
    // Container/name 2; each block/list/two selectors/declaration aggregate 5.
    let exact = Limits::new(12, 12, expected.len());
    let whole = CssKeyframesRuleView::try_new(&name, &children, exact).unwrap();
    assert_eq!(whole.serialize_cssom(exact).unwrap(), expected);
    let leaf = View::keyframes(&name, &children);
    assert_eq!(graph_cssom(leaf, exact).unwrap(), expected);
    for (limits, kind) in [
        (
            Limits::new(11, 12, expected.len()),
            Resource::InputNodeLimit,
        ),
        (
            Limits::new(12, 11, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(12, 12, expected.len() - 5), Resource::ByteLimit),
    ] {
        let failure = graph_cssom(leaf, limits).unwrap_err();
        assert_eq!(failure.limit_kind(), Some(kind));
        assert_eq!(failure.keyframe_block_index(), Some(1));
        crate::css::support::value_assertions::assert_rule_resource(
            CssKeyframesRuleView::try_new(&name, &children, limits),
            kind,
        );
        assert_eq!(whole.serialize_cssom(exact).unwrap(), expected);
    }
    let query = group_query();
    let rules = [leaf];
    let group = View::group(Prelude::Media(&query), &rules);
    let group_expected = format!("@media print {{\n  {expected}\n}}");
    let failure =
        graph_cssom(group, Limits::new(1000, 1000, group_expected.len() - 8)).unwrap_err();
    assert!(
        failure.is_resource_limit()
            && failure
                .serialization_error()
                .is_some_and(|error| error.is_byte_limit()),
        "{failure:?}"
    );
    assert_eq!(failure.rule_path(), &[0]);
    assert_eq!(failure.keyframe_block_index(), Some(1));
    assert_eq!(
        graph_cssom(group, Default::default()).unwrap(),
        group_expected
    );
}

#[test]
fn unresolved_pending_page_footprint_remains_a_typed_source_attributed_capability_failure() {
    let raw = CssBlockFragment::<CssPageBody>::parse(
        &CssParser::default(),
        "/*😀*/ {margin:var(--edges);size:A4;}",
        &CssBlockContext::default(),
    );
    assert!(raw.is_clean());
    let before = raw.clone();
    let declarations = raw.syntax().as_ref().unwrap().body().declarations();
    let failure = declarations.try_specified(Default::default()).unwrap_err();
    let Some(failure) = failure.properties() else {
        panic!("property capability failure")
    };
    assert!(failure.pending_footprint_property() == Some(K::Margin));
    assert!(
        failure
            .declaration()
            .unwrap()
            .same_occurrence(&declarations.properties()[0])
    );
    assert_eq!(
        failure
            .declaration()
            .unwrap()
            .parsed_value()
            .unwrap()
            .source()
            .as_str(),
        "/*😀*/ {margin:var(--edges);size:A4;}"
    );
    assert_eq!(raw, before);
    assert!(declarations.try_specified(Default::default()).is_err());
}

#[test]
fn recursive_page_and_margin_keep_independent_fixed_footprint_pending_shorthand_members() {
    // Variables1 §3.2: an observed pending longhand serializes as empty;
    // reconstructing the original shorthand requires all same-origin members.
    // references/css-variables-1--CR-css-variables-1-20220616--5ce7b2c106af.md#pending-substitution-value
    let source = "/*😀*/ @media print { @page invoice { border-top:var(--page-edge); size:A4; @top-left { border-top:var(--margin-edge); content:'X'; } } }";
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let CssRule::Media(media) = &report.syntax().rules()[0] else {
        panic!("media")
    };
    let CssRule::Page(page) = &media.rules()[0] else {
        panic!("Page")
    };
    assert!(matches!(
        K::BorderTop.metadata().unwrap().kind(),
        surgeist_css::property::CssPropertyKindRef::Shorthand(_)
    ));
    let selected = page
        .declarations()
        .try_specified(Default::default())
        .unwrap();
    let pending = page_property(&selected, K::BorderTopColor);
    assert!(
        matches!(pending.value(), surgeist_css::declaration::CssSpecifiedDeclarationValueRef::PendingShorthand(origin) if origin.same_occurrence(pending.source()))
    );
    let descriptor = selected
        .entries()
        .iter()
        .find(|entry| matches!(entry, PageEntry::Descriptor(_)))
        .unwrap()
        .clone();
    let survivor = PageBlock::try_from_entries(
        &[descriptor, PageEntry::Property(pending.clone())],
        Default::default(),
    )
    .unwrap();
    let selected_margin = page.margin_rules()[0]
        .declarations()
        .try_specified_properties(Default::default())
        .unwrap();
    let pending_margin = terminal(&selected_margin, K::BorderTopColor);
    assert!(
        matches!(pending_margin.value(), surgeist_css::declaration::CssSpecifiedDeclarationValueRef::PendingShorthand(origin) if origin.same_occurrence(pending_margin.source()))
    );
    let margin_survivors = Block::try_from_margin_entries(
        &[
            terminal(&selected_margin, K::Content).clone(),
            pending_margin.clone(),
        ],
        Default::default(),
    )
    .unwrap();
    let margins = [CssMarginRuleView::try_new(CssMarginBox::TopLeft, &margin_survivors).unwrap()];
    let leaf = View::page(CssPageRuleView::new(page.selectors(), &survivor, &margins));
    let children = [leaf];
    let group = View::group(Prelude::Media(media.query()), &children);
    let expected = "@media print {\n  @page invoice { size: a4; border-top-color: ; @top-left { content: \"X\"; border-top-color: ; } }\n}";
    assert_eq!(graph_cssom(group, Default::default()).unwrap(), expected);
    assert_eq!(survivor.entries().len(), 2);
    assert_eq!(margin_survivors.entries().len(), 2);
    assert!(
        page_property(&survivor, K::BorderTopColor)
            .source()
            .same_occurrence(pending.source())
    );
    assert!(
        terminal(&margin_survivors, K::BorderTopColor)
            .source()
            .same_occurrence(pending_margin.source())
    );
    assert_eq!(pending.source().known().unwrap().property(), K::BorderTop);
    assert_eq!(
        pending_margin.source().known().unwrap().property(),
        K::BorderTop
    );
    assert_eq!(
        pending.source().parsed_value().unwrap().source().as_str(),
        source
    );
    assert_eq!(
        pending_margin
            .source()
            .parsed_value()
            .unwrap()
            .source()
            .as_str(),
        source
    );
    assert_eq!(graph_cssom(group, Default::default()).unwrap(), expected);
    assert_eq!(report, before);
}
