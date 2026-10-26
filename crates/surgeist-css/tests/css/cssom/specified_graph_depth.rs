//! `CssRuleGraphInput::serialize_specified` bounds structural graph depth with
//! `CssRuleSerializationLimits::max_depth`, exactly as `serialize_cssom` does. Root
//! depth is zero, so a chain of three nested rules needs `max_depth` 2.
use CssGroupPreludeRef as Prelude;
use CssRuleGraphInput as Input;
use surgeist_css::*;

fn limits(max_depth: usize) -> CssRuleSerializationLimits {
    CssRuleSerializationLimits {
        max_depth,
        ..CssRuleSerializationLimits::default()
    }
}

fn specified(input: Input<'_>, max_depth: usize) -> Result<String, CssRuleSerializationError> {
    input.serialize_specified(limits(max_depth))
}

fn cssom(input: Input<'_>, max_depth: usize) -> Result<String, CssRuleSerializationError> {
    input.serialize_cssom(
        &CssNamespaceContext::default(),
        CssStyleAncestor::Absent,
        limits(max_depth),
    )
}

fn is_depth_limit(error: &CssRuleSerializationError) -> bool {
    error.is_resource_limit()
        && error
            .serialization_error()
            .is_some_and(|error| error.is_depth_limit())
}

#[test]
fn nested_groups_are_bounded_by_the_same_depth_in_both_outputs() {
    let queries: &'static CssMediaQueryList = Box::leak(Box::new(
        CssMediaQueryList::parse(&CssParser::default(), "screen")
            .into_parts()
            .0,
    ));
    let leaf: &'static [Input<'static>] =
        Box::leak(Box::new([Input::group(Prelude::Media(queries), &[])]));
    let middle: &'static [Input<'static>] =
        Box::leak(Box::new([Input::group(Prelude::Media(queries), leaf)]));
    let root = Input::group(Prelude::Media(queries), middle);
    for max_depth in 0..=4 {
        let compact = specified(root, max_depth);
        let literal = cssom(root, max_depth);
        assert_eq!(compact.is_ok(), max_depth >= 2, "depth {max_depth}");
        assert_eq!(literal.is_ok(), max_depth >= 2, "depth {max_depth}");
        if let (Err(compact), Err(literal)) = (&compact, &literal) {
            assert!(is_depth_limit(compact), "{compact:?}");
            assert_eq!(compact.rule_path(), literal.rule_path());
        }
    }
    assert_eq!(
        specified(root, 2).unwrap(),
        "@media screen { @media screen { @media screen { } } }"
    );
}

#[test]
fn nested_style_rules_are_bounded_by_depth() {
    let report = CssParser::default().parse_sheet("a {}");
    let CssRule::Style(style) = &report.syntax().rules()[0] else {
        panic!("style rule");
    };
    let none = CssParser::default().parse_block_contents("");
    let block =
        CssSpecifiedDeclarationBlock::try_from_declarations(none.syntax(), Default::default())
            .unwrap();
    let leaf = [Input::style(style.selectors(), &block, &[])];
    let middle = [Input::style(style.selectors(), &block, &leaf)];
    let root = Input::style(style.selectors(), &block, &middle);
    for max_depth in 0..=3 {
        let result = specified(root, max_depth);
        assert_eq!(result.is_ok(), max_depth >= 2, "depth {max_depth}");
        if let Err(error) = result {
            assert!(is_depth_limit(&error), "{error:?}");
        }
    }
}

#[test]
fn a_borrowed_parsed_rule_is_bounded_by_depth() {
    let report = CssParser::default().parse_sheet("@media a { @media b { @media c {} } }");
    let root = Input::from_rule(&report.syntax().rules()[0]);
    for max_depth in 0..=3 {
        let result = specified(root, max_depth);
        assert_eq!(result.is_ok(), max_depth >= 2, "depth {max_depth}");
        if let Err(error) = result {
            assert!(is_depth_limit(&error), "{error:?}");
        }
    }
}
