//! CSS Nesting 1 sections 3.1 and 4 permit repeated nesting selectors in
//! compounds and functional selectors without multiplying the parent list.
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/

use surgeist_css::{
    CssCompoundSelector, CssPseudoClass, CssRule, CssSelector, CssSelectorCombinator,
    CssStyleSelector, parse_sheet,
};

fn compound_anchors(compound: &CssCompoundSelector) -> usize {
    assert!(
        !compound.has_scope_anchor(),
        "nesting is not a scope-root anchor"
    );
    compound.nesting_selectors()
        + compound
            .pseudo_classes()
            .iter()
            .map(pseudo_anchors)
            .sum::<usize>()
}

fn pseudo_anchors(pseudo: &CssPseudoClass) -> usize {
    match pseudo {
        CssPseudoClass::Is(list) | CssPseudoClass::Where(list) | CssPseudoClass::Not(list) => {
            list.selectors().iter().map(selector_anchors).sum()
        }
        CssPseudoClass::Has(list) => list
            .selectors()
            .iter()
            .map(|relative| selector_anchors(relative.selector()))
            .sum(),
        CssPseudoClass::NthChild(pattern) | CssPseudoClass::NthLastChild(pattern) => {
            pattern.selector_list().map_or(0, |list| {
                list.selectors().iter().map(selector_anchors).sum()
            })
        }
        _ => 0,
    }
}

fn selector_anchors(selector: &CssSelector) -> usize {
    match selector {
        CssSelector::Compound(compound) => compound_anchors(compound),
        CssSelector::Complex(complex) => {
            compound_anchors(complex.first())
                + complex
                    .rest()
                    .iter()
                    .map(|part| compound_anchors(part.selector()))
                    .sum::<usize>()
        }
        CssSelector::PseudoClass(pseudo) => pseudo_anchors(pseudo),
        _ => 0,
    }
}

#[test]
fn explicit_nesting_anchors_survive_every_valid_selector_position() {
    for (authored, anchors) in [
        ("&&", 2),
        ("div&", 1),
        (".class&", 1),
        ("#id&", 1),
        ("[attr]&", 1),
        (":hover&", 1),
        ("& > .bar", 1),
        ("& .bar & .baz & .qux", 3),
        (".test > & .bar", 1),
        (":is(.bar, &.baz)", 1),
        ("&:is(.bar, &.baz)", 2),
        (":where(&, .child)", 1),
        (":not(&)", 1),
        (":has(> &)", 1),
        (":nth-child(2n of &)", 1),
    ] {
        let source = format!(".parent, #other {{ {authored} {{ color: green }} }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{authored}: {:?}", report.diagnostics());
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("one parent")
        };
        assert_eq!(parent.selectors().selectors().len(), 2);
        let [CssRule::Style(child)] = parent.rules() else {
            panic!("one authored child: {authored}")
        };
        assert_eq!(child.selectors().selectors().len(), 1);
        assert_eq!(
            selector_anchors(child.selectors().selectors()[0].selector()),
            anchors,
            "{authored}"
        );
        assert_eq!(
            child.position().byte_offset().value(),
            source.find(authored).unwrap()
        );
        assert_eq!(child.declarations().len(), 1);
    }
}

#[test]
fn leading_relative_combinators_preserve_their_explicit_inner_anchors() {
    for (authored, expected) in [
        ("> & .bar", CssSelectorCombinator::Child),
        ("+ .bar &", CssSelectorCombinator::NextSibling),
    ] {
        let source = format!(".parent {{ {authored} {{ color: green }} }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{authored}: {:?}", report.diagnostics());
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("one parent")
        };
        let [CssRule::Style(child)] = parent.rules() else {
            panic!("one child")
        };
        let [CssStyleSelector::Relative(relative)] = child.selectors().selectors() else {
            panic!("authored leading combinator retained")
        };
        assert_eq!(relative.combinator(), expected);
        assert_eq!(selector_anchors(relative.selector()), 1);
    }
}

#[test]
fn nesting_type_order_and_nested_has_restrictions_recover_the_inner_rule() {
    for authored in ["&div", "&&div", "&:has(:has(.child))"] {
        let source = format!(".parent {{ {authored} {{ color: green }} color: red }}");
        let report = parse_sheet(&source);
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("one retained parent")
        };
        assert!(!report.is_clean(), "{authored}");
        assert!(
            parent
                .rules()
                .iter()
                .all(|rule| !matches!(rule, CssRule::Style(_))),
            "{authored}"
        );
        let declaration_count = parent.declarations().len()
            + parent
                .rules()
                .iter()
                .map(|rule| {
                    if let CssRule::NestedDeclarations(run) = rule {
                        run.declarations().len()
                    } else {
                        0
                    }
                })
                .sum::<usize>();
        assert_eq!(
            declaration_count, 1,
            "following declaration survives {authored}"
        );
    }
}
