#![forbid(unsafe_code)]

//! Inspect ordered media operators, explicit grouping and original source positions.

use surgeist_css::{
    CssCondition, CssConditionKind, CssMediaFeatureKind, CssMediaFeatureQuery, CssMediaLeaf,
    CssMediaQuery, CssParser,
};

fn condition(source: &str) -> CssCondition<CssMediaLeaf> {
    let report = CssMediaQuery::parse(&CssParser::default(), source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssMediaQuery::Condition(value) = report.into_validation_result().unwrap() else {
        panic!("condition-only query expected");
    };
    value
}

fn boolean(value: &CssCondition<CssMediaLeaf>, name: CssMediaFeatureKind, offset: usize) {
    assert_eq!(
        value
            .position()
            .expect("parsed media position")
            .byte_offset()
            .value(),
        offset
    );
    assert!(
        matches!(value.kind(), CssConditionKind::Leaf(CssMediaLeaf::Feature(CssMediaFeatureQuery::Boolean(actual))) if *actual == name)
    );
}

fn main() {
    let source = "/*😀*/ (not (color)) or ((monochrome) and (width))";
    let parsed = condition(source);
    let CssConditionKind::Or(or) = parsed.kind() else {
        panic!("outer Or cannot become opaque");
    };
    let [left, right] = or.conditions() else {
        panic!("two Or operands");
    };
    assert_eq!(
        left.position()
            .expect("parsed media position")
            .byte_offset()
            .value(),
        source.find("(not").unwrap()
    );
    let CssConditionKind::Parenthesized(negation) = left.kind() else {
        panic!("explicit left grouping");
    };
    assert_eq!(
        negation
            .position()
            .expect("parsed media position")
            .byte_offset()
            .value(),
        source.find("not ").unwrap()
    );
    let CssConditionKind::Not(color) = negation.kind() else {
        panic!("Not inside group");
    };
    boolean(
        color.as_ref(),
        CssMediaFeatureKind::Color,
        source.find("(color)").unwrap(),
    );
    assert_eq!(
        right
            .position()
            .expect("parsed media position")
            .byte_offset()
            .value(),
        source.find("((monochrome)").unwrap()
    );
    let CssConditionKind::Parenthesized(conjunction) = right.kind() else {
        panic!("right group");
    };
    let CssConditionKind::And(and) = conjunction.kind() else {
        panic!("And inside group");
    };
    assert_eq!(
        conjunction
            .position()
            .expect("parsed media position")
            .byte_offset()
            .value(),
        source.find("(monochrome)").unwrap()
    );
    let [monochrome, width] = and.conditions() else {
        panic!("two ordered And operands");
    };
    boolean(
        monochrome,
        CssMediaFeatureKind::Monochrome,
        source.find("(monochrome)").unwrap(),
    );
    boolean(
        width,
        CssMediaFeatureKind::Width,
        source.find("(width)").unwrap(),
    );

    let source = "screen and ((color) or (monochrome))";
    let report = CssMediaQuery::parse(&CssParser::default(), source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssMediaQuery::Typed(typed) = report.syntax() else {
        panic!("typed query");
    };
    let group = typed.condition().expect("typed condition");
    assert_eq!(
        group
            .position()
            .expect("parsed media position")
            .byte_offset()
            .value(),
        source.find("((color)").unwrap()
    );
    let CssConditionKind::Parenthesized(disjunction) = group.kind() else {
        panic!("typed grouped Or");
    };
    assert_eq!(
        disjunction
            .position()
            .expect("parsed media position")
            .byte_offset()
            .value(),
        source.find("(color)").unwrap()
    );
    let CssConditionKind::Or(or) = disjunction.kind() else {
        panic!("Or inside typed group");
    };
    let [color, monochrome] = or.conditions() else {
        panic!("two ordered Or operands");
    };
    boolean(
        color,
        CssMediaFeatureKind::Color,
        source.find("(color)").unwrap(),
    );
    boolean(
        monochrome,
        CssMediaFeatureKind::Monochrome,
        source.find("(monochrome)").unwrap(),
    );

    println!("media grouping semantic contract: ok");
}
