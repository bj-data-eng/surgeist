#![forbid(unsafe_code)]
//! Existing complex assembly must apply one default specified-output budget to
//! the complete graph. Parts come from the real parser; compound conversion uses
//! the existing checked Shadow argument boundary, without private-field access.

use surgeist_css::{
    CssComplexSelector, CssComplexSelectorPart, CssCompoundSelector, CssCompoundSelectorArgument,
    CssNamespaceContext, CssSelector, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, parse_selector,
};

fn parsed(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source:?}: {report:?}");
    report.into_validation_result().unwrap().unwrap()
}

fn complex(source: &str) -> CssComplexSelector {
    let CssSelector::Complex(value) = parsed(source) else {
        panic!("expected complex selector for {source:?}");
    };
    value
}

fn class_compound(value: String) -> CssCompoundSelector {
    CssCompoundSelectorArgument::try_new(CssSelector::Class(value))
        .expect("nonempty decoded class is a valid compound argument")
        .compound()
        .clone()
}

fn child_part() -> CssComplexSelectorPart {
    let value = complex(".One > .Two");
    let [part] = value.rest() else {
        panic!("one parsed child part");
    };
    part.clone()
}

#[test]
fn complex_assembly_preserves_order_and_legal_terminal_pseudo_element() {
    let value = complex(".Seed > .Child + .Next ~ .Later || .Cell::before");
    let actual = CssComplexSelector::try_new(value.first().clone(), value.rest().to_vec()).unwrap();
    assert_eq!(actual, value);
    let selector = CssSelector::Complex(actual);
    let literal = ".Seed > .Child + .Next ~ .Later || .Cell::before";
    assert_eq!(selector.to_specified_css().unwrap(), literal);
    assert_eq!(parsed(literal), selector);
}

#[test]
fn complex_assembly_rejects_empty_rest_and_nonterminal_pseudo_element() {
    let ordinary = complex(".One > .Two");
    assert!(CssComplexSelector::try_new(ordinary.first().clone(), Vec::new()).is_err());

    let terminal = complex(".One > .Two::before");
    let first = terminal.rest()[0].selector().clone();
    assert!(CssComplexSelector::try_new(first, ordinary.rest().to_vec()).is_err());

    let mut rest = terminal.rest().to_vec();
    rest.extend_from_slice(ordinary.rest());
    assert!(CssComplexSelector::try_new(ordinary.first().clone(), rest).is_err());
}

#[test]
fn selected_shadow_host_has_remains_valid_at_complex_assembly() {
    // Selected Shadow admission keeps the outer host argument compound-only,
    // while Has retains its independent strict relative argument grammar.
    let first = CssCompoundSelectorArgument::try_new(parsed(":host(:has(> .One))"))
        .unwrap()
        .compound()
        .clone();
    let terminal = complex(".Parent > .Cell::before");
    let actual = CssComplexSelector::try_new(first, terminal.rest().to_vec()).unwrap();
    let selector = CssSelector::Complex(actual);
    let literal = ":host(:has(> .One)) > .Cell::before";
    assert_eq!(selector.to_specified_css().unwrap(), literal);
    assert_eq!(parsed(literal), selector);
}

#[test]
fn complex_assembly_admits_exact_default_input_node_boundary() {
    assert_eq!(
        CssSpecifiedValueSerializationLimits::default().max_input_nodes(),
        65_536
    );
    // One complex + a compound with two classes + 32,766 compounds with one
    // class each: 1 + 3 + 32,766 * 2 = 65,536 semantic nodes.
    let value = complex(".One.Two > .Two");
    let rest = vec![child_part(); 32_766];
    let actual = CssComplexSelector::try_new(value.first().clone(), rest.clone()).unwrap();
    assert_eq!(actual.first(), value.first());
    assert!(
        actual.rest() == rest,
        "complete ordered parts remain unchanged"
    );
    let literal = format!(".One.Two{}", " > .Two".repeat(32_766));
    assert!(
        CssSelector::Complex(actual).to_specified_css().unwrap() == literal,
        "exact input-boundary graph preserves its independently authored output"
    );
}

#[test]
fn complex_assembly_rejects_one_node_above_default_cumulative_budget() {
    // The one additional class makes 65,537 nodes. Every supplied compound is
    // individually valid and tiny; rejecting this graph requires aggregation.
    let value = complex(".One.Two.Three > .Two");
    assert_eq!(
        CssSelector::Compound(value.first().clone())
            .to_specified_css()
            .unwrap(),
        ".One.Two.Three"
    );
    let part = child_part();
    assert_eq!(
        CssSelector::Compound(part.selector().clone())
            .to_specified_css()
            .unwrap(),
        ".Two"
    );
    let actual = CssComplexSelector::try_new(value.first().clone(), vec![part; 32_766]);
    if let Ok(value) = &actual {
        assert_eq!(
            CssSelector::Complex(value.clone())
                .to_specified_css()
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit
        );
    }
    assert!(
        actual.is_err(),
        "complex assembly admitted 65,537 cumulative input nodes"
    );
}

#[test]
fn complex_assembly_admits_exact_default_utf8_byte_boundary() {
    assert_eq!(
        CssSpecifiedValueSerializationLimits::default().max_css_bytes(),
        1_048_576
    );
    // A dot + 524,284 two-byte é code points + the seven-byte child suffix.
    let name = "é".repeat(524_284);
    let literal = format!(".{name} > .Two");
    assert_eq!(literal.len(), 1_048_576);
    let first = class_compound(name);
    let rest = vec![child_part()];
    let actual = CssComplexSelector::try_new(first.clone(), rest.clone()).unwrap();
    assert!(actual.first() == &first, "decoded class remains unchanged");
    assert_eq!(actual.rest(), rest);
    assert!(
        CssSelector::Complex(actual).to_specified_css().unwrap() == literal,
        "exact byte-boundary graph preserves its independently authored output"
    );
}

#[test]
fn complex_assembly_rejects_utf8_bytes_above_default_cumulative_budget() {
    // One extra ASCII a adds one byte. The first compound alone remains inside
    // the byte limit; adding the child relationship makes 1,048,577 UTF-8 bytes.
    let name = format!("{}a", "é".repeat(524_284));
    let first = class_compound(name.clone());
    assert!(
        CssSelector::Compound(first.clone())
            .to_specified_css()
            .unwrap()
            == format!(".{name}"),
        "first compound is independently representable within the default byte budget"
    );
    let literal = format!(".{name} > .Two");
    assert_eq!(literal.len(), 1_048_577);
    let actual = CssComplexSelector::try_new(first, vec![child_part()]);
    if let Ok(value) = &actual {
        assert_eq!(
            CssSelector::Complex(value.clone())
                .to_specified_css()
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
    }
    assert!(
        actual.is_err(),
        "complex assembly admitted 1,048,577 cumulative UTF-8 bytes"
    );
}
