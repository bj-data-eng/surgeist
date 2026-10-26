//! One boolean condition model over grammar-specific leaves (#1388). Media
//! Queries 4 §3 `<media-condition>`, Conditional 3 `<supports-condition>`,
//! Contain 3 `<container-condition>`, `<style-query>` and `<scroll-state-query>`,
//! and Conditional 5 `<boolean-condition>` share the not/and/or/group shape, and
//! every explicit grouping pair stays a structural layer in each of them.
use surgeist_css::*;

/// The structure of `condition`, with each leaf described by `leaf`.
fn shape<L: CssConditionLeaf + std::fmt::Debug>(
    condition: &CssCondition<L>,
    leaf: &dyn Fn(&L) -> String,
) -> String {
    let list = |list: &CssConditionList<L>| {
        list.conditions()
            .iter()
            .map(|condition| shape(condition, leaf))
            .collect::<Vec<_>>()
            .join(",")
    };
    match condition.kind() {
        CssConditionKind::Not(inner) => format!("not({})", shape(inner, leaf)),
        CssConditionKind::And(operands) => format!("and[{}]", list(operands)),
        CssConditionKind::Or(operands) => format!("or[{}]", list(operands)),
        CssConditionKind::Parenthesized(inner) => format!("group({})", shape(inner, leaf)),
        CssConditionKind::Leaf(value) => leaf(value),
        _ => panic!("unexpected condition kind: {:?}", condition.kind()),
    }
}

fn media_leaf(leaf: &CssMediaLeaf) -> String {
    match leaf {
        CssMediaLeaf::Feature(_) => "feature",
        CssMediaLeaf::CustomMediaReference(_) => "custom",
        CssMediaLeaf::UnknownFeature(_) => "unknown",
        CssMediaLeaf::GeneralEnclosed(_) => "enclosed",
        _ => panic!("unexpected media leaf"),
    }
    .to_owned()
}

fn supports_leaf(leaf: &CssSupportsLeaf) -> String {
    match leaf {
        CssSupportsLeaf::Declaration(_) => "declaration",
        CssSupportsLeaf::Selector(_) => "selector",
        CssSupportsLeaf::Named(_) => "named",
        CssSupportsLeaf::FontTech(_) => "font-tech",
        CssSupportsLeaf::FontFormat(_) => "font-format",
        CssSupportsLeaf::AtRule(_) => "at-rule",
        CssSupportsLeaf::GeneralEnclosed(_) => "enclosed",
        _ => panic!("unexpected supports leaf"),
    }
    .to_owned()
}

fn style_leaf(leaf: &CssStyleQueryLeaf) -> String {
    match leaf {
        CssStyleQueryLeaf::Feature(_) => "feature",
        CssStyleQueryLeaf::GeneralEnclosed(_) => "enclosed",
        _ => panic!("unexpected style-query leaf"),
    }
    .to_owned()
}

fn scroll_leaf(leaf: &CssScrollStateLeaf) -> String {
    match leaf {
        CssScrollStateLeaf::Feature(_) => "feature",
        CssScrollStateLeaf::GeneralEnclosed(_) => "enclosed",
        _ => panic!("unexpected scroll-state leaf"),
    }
    .to_owned()
}

fn container_leaf(leaf: &CssContainerLeaf) -> String {
    match leaf {
        CssContainerLeaf::Feature(_) => "size".to_owned(),
        CssContainerLeaf::GeneralEnclosed(_) => "enclosed".to_owned(),
        CssContainerLeaf::Style(query) => format!("style<{}>", shape(query, &style_leaf)),
        CssContainerLeaf::ScrollState(query) => format!("scroll<{}>", shape(query, &scroll_leaf)),
        _ => panic!("unexpected container leaf"),
    }
}

fn when_leaf(leaf: &CssWhenLeaf) -> String {
    match leaf {
        CssWhenLeaf::MediaFeature(_) => "media",
        CssWhenLeaf::SupportsDeclaration(_) => "supports",
        CssWhenLeaf::GeneralEnclosed(_) => "enclosed",
        _ => panic!("unexpected when leaf"),
    }
    .to_owned()
}

fn media(source: &str) -> String {
    let report = CssParser::default().parse_sheet(format!("@media {source} {{}}"));
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::Media(rule)] = report.syntax().rules() else {
        panic!("one media rule");
    };
    let [CssMediaQuery::Condition(condition)] = rule.query().queries() else {
        panic!("one media condition: {source}");
    };
    shape(condition, &media_leaf)
}

fn supports(source: &str) -> String {
    let report = CssParser::default().parse_sheet(format!("@supports {source} {{}}"));
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("one supports rule");
    };
    shape(rule.condition(), &supports_leaf)
}

fn container(source: &str) -> String {
    let report = CssParser::default().parse_sheet(format!("@container {source} {{}}"));
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("one container rule");
    };
    let [entry] = rule.prelude().entries() else {
        panic!("one container entry");
    };
    shape(entry.query().expect("a query"), &container_leaf)
}

fn when(source: &str) -> String {
    let report = CssParser::default().parse_sheet(format!("@when {source} {{}}"));
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::When(rule)] = report.syntax().rules() else {
        panic!("one when rule");
    };
    shape(rule.condition(), &when_leaf)
}

#[test]
fn media_conditions_keep_negation_junction_and_group_layers() {
    assert_eq!(media("(color)"), "feature");
    assert_eq!(media("((color))"), "group(feature)");
    assert_eq!(media("(((color)))"), "group(group(feature))");
    assert_eq!(media("(color) and (hover)"), "and[feature,feature]");
    assert_eq!(
        media("(color) or (hover) or (pointer)"),
        "or[feature,feature,feature]"
    );
    assert_eq!(
        media("not ((color) and (hover))"),
        "not(group(and[feature,feature]))"
    );
    assert_eq!(
        media("(color) and ((hover) or (--wide))"),
        "and[feature,group(or[feature,custom])]"
    );
    assert_eq!(media("(unknown-thing: 1)"), "unknown");
    assert_eq!(media("future(thing)"), "enclosed");
}

#[test]
fn supports_conditions_keep_negation_junction_and_group_layers() {
    assert_eq!(supports("(display:flex)"), "declaration");
    assert_eq!(supports("((display:flex))"), "group(declaration)");
    assert_eq!(supports("(((display:flex)))"), "group(group(declaration))");
    assert_eq!(
        supports("(display:flex) and (color:red)"),
        "and[declaration,declaration]"
    );
    assert_eq!(
        supports("not ((display:flex) and (color:red))"),
        "not(group(and[declaration,declaration]))"
    );
    assert_eq!(supports("(not (display:flex))"), "group(not(declaration))");
    assert_eq!(
        supports("selector(a > b) or font-tech(variations)"),
        "or[selector,font-tech]"
    );
    assert_eq!(
        supports("font-format(woff2) and (selector(a) or at-rule(@media))"),
        "and[font-format,group(or[selector,at-rule])]"
    );
    assert_eq!(supports("future(thing)"), "enclosed");
}

#[test]
fn container_conditions_keep_negation_junction_group_and_nested_query_layers() {
    assert_eq!(container("(min-width:1px)"), "size");
    assert_eq!(container("((min-width:1px))"), "group(size)");
    assert_eq!(
        container("(min-width:1px) and ((width < 2px) or style(--x:1))"),
        "and[size,group(or[size,style<feature>])]"
    );
    assert_eq!(
        container("not style(((--x:1)) or (--y:2))"),
        "not(style<or[group(group(feature)),group(feature)]>)"
    );
    assert_eq!(
        container("scroll-state((stuck:top) or (snapped:x))"),
        "scroll<or[group(feature),group(feature)]>"
    );
    assert_eq!(container("future(thing)"), "enclosed");
}

#[test]
fn when_conditions_keep_negation_junction_and_group_layers() {
    assert_eq!(when("media(color)"), "media");
    assert_eq!(when("((media(color)))"), "group(group(media))");
    assert_eq!(
        when("media(color) and supports(display:flex)"),
        "and[media,supports]"
    );
    assert_eq!(
        when("not (media(color) or media(hover))"),
        "not(group(or[media,media]))"
    );
    assert_eq!(when("future(thing)"), "enclosed");
}

#[test]
fn every_grouping_pair_keeps_its_own_origin_and_serialization_is_unchanged() {
    let report = CssParser::default().parse_sheet("@supports  ( ( display:flex ) ) {}");
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("supports");
    };
    let condition = rule.condition();
    let CssConditionKind::Parenthesized(inner) = condition.kind() else {
        panic!("outer pair: {:?}", condition.kind());
    };
    // The outer condition starts at its pair; the inner one at the declaration test.
    assert_eq!(
        condition.position().map(|p| p.byte_offset().value()),
        Some(11)
    );
    assert_eq!(inner.position().map(|p| p.byte_offset().value()), Some(13));
    assert_eq!(
        condition
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "  ( ( display:flex ) ) "
    );
    assert_eq!(
        report
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        "@supports ( ( display:flex ) ) { }"
    );
}

#[test]
fn condition_lists_require_two_operands_of_the_same_grammar() {
    let report = CssParser::default().parse_sheet("@supports (a:b) and (c:d) {}");
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("supports");
    };
    let CssConditionKind::And(list) = rule.condition().kind() else {
        panic!("and");
    };
    let [first, second] = list.conditions() else {
        panic!("two operands");
    };
    assert!(CssConditionList::<CssSupportsLeaf>::try_new(Vec::new()).is_none());
    assert!(CssConditionList::try_new(vec![first.clone()]).is_none());
    let rebuilt = CssConditionList::try_new(vec![first.clone(), second.clone()]).unwrap();
    assert_eq!(&rebuilt, list);
}
