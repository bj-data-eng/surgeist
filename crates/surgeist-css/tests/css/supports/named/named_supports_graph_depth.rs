//! #644's new graph-depth contract counts every actual native child edge.
use surgeist_css::{
    CssGroupPreludeRef, CssKeyframeRuleView, CssMarginBox, CssMarginRuleView, CssNamespaceContext,
    CssPageRuleView, CssParser, CssRule, CssRuleGraphInput as Input, CssRuleSerializationLimits,
    CssSpecifiedDeclarationBlock as Block, CssStyleAncestor,
};

fn format(
    input: Input<'_>,
    depth: usize,
) -> Result<String, surgeist_css::CssRuleSerializationError> {
    input.serialize_cssom(
        &CssNamespaceContext::default(),
        CssStyleAncestor::Absent,
        CssRuleSerializationLimits {
            max_depth: depth,
            ..Default::default()
        },
    )
}
fn depth_failure(input: Input<'_>, depth: usize, path: &[usize], block: Option<usize>) {
    let error = format(input, depth).expect_err("actual child edge must exhaust the depth bound");
    assert!(
        error.is_resource_limit()
            && error
                .serialization_error()
                .is_some_and(|error| error.is_depth_limit()),
        "{error:?}"
    );
    assert_eq!(error.rule_path(), path);
    assert_eq!(error.keyframe_block_index(), block);
}
fn serializes_within_default_limits(input: Input<'_>, expected: &str) {
    assert_eq!(
        input
            .serialize_cssom(
                &CssNamespaceContext::default(),
                CssStyleAncestor::Absent,
                Default::default()
            )
            .unwrap(),
        expected
    );
}

#[test]
fn parsed_ordinary_graph_counts_child_edges_and_keeps_root_zero() {
    let report = CssParser::default().parse_sheet("@media all { a {} }");
    assert!(report.is_clean());
    let input = Input::from_rule(&report.syntax().rules()[0]);
    let expected = "@media all {\n  a { }\n}";
    assert_eq!(format(input, 1).unwrap(), expected);
    serializes_within_default_limits(input, expected);
    let CssRule::Media(media) = &report.syntax().rules()[0] else {
        panic!()
    };
    assert_eq!(
        format(Input::from_rule(&media.rules()[0]), 0).unwrap(),
        "a { }"
    );
    depth_failure(input, 0, &[0], None);
}

#[test]
fn mixed_edited_parent_and_parsed_child_share_the_actual_depth() {
    let report = CssParser::default().parse_sheet("@media all { a {} }");
    assert!(report.is_clean());
    let CssRule::Media(media) = &report.syntax().rules()[0] else {
        panic!()
    };
    let leaf = Input::from_rule(&media.rules()[0]);
    assert_eq!(format(leaf, 0).unwrap(), "a { }");
    let children = [leaf];
    let input = Input::group(CssGroupPreludeRef::Media(media.query()), &children);
    let expected = "@media all {\n  a { }\n}";
    assert_eq!(format(input, 1).unwrap(), expected);
    serializes_within_default_limits(input, expected);
    depth_failure(input, 0, &[0], None);
}

#[test]
fn parsed_scoped_graph_counts_the_same_child_edges() {
    let report = CssParser::default().parse_sheet("@scope { @media all { a {} } }");
    assert!(report.is_clean(), "{report:?}");
    let CssRule::Scope(scope) = &report.syntax().rules()[0] else {
        panic!()
    };
    let input = Input::from_rule(&scope.rules().rules()[0]);
    let expected = "@media all {\n  a { }\n}";
    assert_eq!(format(input, 1).unwrap(), expected);
    serializes_within_default_limits(input, expected);
    depth_failure(input, 0, &[0], None);
}

#[test]
fn parsed_page_margin_is_a_native_child_edge() {
    let empty = CssParser::default().parse_sheet("@page {}");
    assert!(empty.is_clean());
    assert_eq!(
        format(Input::from_rule(&empty.syntax().rules()[0]), 0).unwrap(),
        "@page { }"
    );
    let report = CssParser::default().parse_sheet("@page { @top-left {} }");
    assert!(report.is_clean());
    let input = Input::from_rule(&report.syntax().rules()[0]);
    let expected = "@page { @top-left { } }";
    assert_eq!(format(input, 1).unwrap(), expected);
    serializes_within_default_limits(input, expected);
    depth_failure(input, 0, &[0], None);
}

#[test]
fn selected_page_margin_is_a_native_child_edge() {
    let report = CssParser::default().parse_sheet("@page {}");
    assert!(report.is_clean());
    let CssRule::Page(page) = &report.syntax().rules()[0] else {
        panic!()
    };
    let selected = page
        .declarations()
        .try_specified(Default::default())
        .unwrap();
    let block = Block::try_from_margin_entries(&[], Default::default()).unwrap();
    let margins = [CssMarginRuleView::try_new(CssMarginBox::TopLeft, &block).unwrap()];
    let input = Input::page(CssPageRuleView::new(page.selectors(), &selected, &margins));
    let expected = "@page { @top-left { } }";
    assert_eq!(format(input, 1).unwrap(), expected);
    serializes_within_default_limits(input, expected);
    depth_failure(input, 0, &[0], None);
}

#[test]
fn parsed_keyframe_block_is_a_native_child_edge() {
    let empty = CssParser::default().parse_sheet("@keyframes k {}");
    assert!(empty.is_clean());
    assert_eq!(
        format(Input::from_rule(&empty.syntax().rules()[0]), 0).unwrap(),
        "@keyframes k { \n}"
    );
    let report = CssParser::default().parse_sheet("@keyframes k { from {} }");
    assert!(report.is_clean());
    let input = Input::from_rule(&report.syntax().rules()[0]);
    let expected = "@keyframes k {   0% { }\n}";
    assert_eq!(format(input, 1).unwrap(), expected);
    serializes_within_default_limits(input, expected);
    depth_failure(input, 0, &[], Some(0));
}

#[test]
fn selected_keyframe_block_is_a_native_child_edge() {
    let report = CssParser::default().parse_sheet("@keyframes k { from {} }");
    assert!(report.is_clean());
    let CssRule::Keyframes(frames) = &report.syntax().rules()[0] else {
        panic!()
    };
    let block = Block::try_from_keyframe_entries(&[], Default::default()).unwrap();
    let blocks = [CssKeyframeRuleView::try_new(frames.blocks()[0].selectors(), &block).unwrap()];
    let input = Input::keyframes(frames.name(), &blocks);
    let expected = "@keyframes k {   0% { }\n}";
    assert_eq!(format(input, 1).unwrap(), expected);
    serializes_within_default_limits(input, expected);
    depth_failure(input, 0, &[], Some(0));
}

#[test]
fn rendering_ancestors_reduce_the_native_child_allowance_without_resets() {
    let report = CssParser::default()
        .parse_sheet("@media all { @page { @top-left {} } @keyframes k { from {} } }");
    assert!(report.is_clean(), "{report:?}");
    let CssRule::Media(media) = &report.syntax().rules()[0] else {
        panic!()
    };
    let children = [
        Input::from_rule(&media.rules()[0]),
        Input::from_rule(&media.rules()[1]),
    ];
    let input = Input::group(CssGroupPreludeRef::Media(media.query()), &children);
    let expected = "@media all {\n  @page { @top-left { } }\n  @keyframes k {   0% { }\n}\n}";
    assert_eq!(format(input, 2).unwrap(), expected);
    serializes_within_default_limits(input, expected);
    depth_failure(input, 1, &[0, 0], None);
    let keyframe_children = [children[1]];
    let keyframes = Input::group(CssGroupPreludeRef::Media(media.query()), &keyframe_children);
    assert_eq!(
        format(keyframes, 2).unwrap(),
        format(keyframes, usize::MAX).unwrap()
    );
    depth_failure(keyframes, 1, &[0], Some(0));
}
