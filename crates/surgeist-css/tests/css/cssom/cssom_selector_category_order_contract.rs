#![forbid(unsafe_code)]
//! Characterization of the accepted canonical-group selector contract. These
//! independent goldens retain repetition within each group and do not claim
//! exact upstream CSSOM conformity for mixed authored category order.

use surgeist_css::{
    CssAttributeCaseSensitivity, CssAttributeMatcher, CssAttributeName, CssAttributeSelector,
    CssCompoundSelector, CssIdent, CssNamespaceContext, CssPseudoClass, CssPseudoElement,
    CssPseudoElementSegment as Segment, CssQualifiedAttributeName, CssQualifiedNamePrefix, CssRule,
    CssRuleCssomSerializationErrorKind as RuleError, CssSelector, CssSimpleSelector as Simple,
    CssSpecifiedValueSerializationErrorKind as Resource,
    CssSpecifiedValueSerializationLimits as Limits, CssStyleSelector, CssValueOrigin, parse_rule,
    parse_selector, parse_sheet,
};

const AUTHORED: &str = ".One#X:hover[A='é'].Two#Y:focus[B='two'].One#X:hover[A='é']";
const CANONICAL: &str = "#X#Y#X.One.Two.One[A=\"é\"][B=\"two\"][A=\"é\"]:hover:focus:hover";

fn parsed(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source}: {report:?}");
    report.into_validation_result().unwrap().unwrap()
}

fn ident(value: &str) -> CssIdent {
    CssIdent::try_new(value).unwrap()
}

fn attribute(name: &str, value: &str) -> Simple {
    let name = CssQualifiedAttributeName::try_new(
        CssQualifiedNamePrefix::Unqualified,
        CssAttributeName::try_new(name).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    Simple::Attribute(
        CssAttributeSelector::try_new(
            name,
            CssAttributeMatcher::Equals(value.into()),
            CssAttributeCaseSensitivity::DocumentDefault,
        )
        .unwrap(),
    )
}

#[test]
fn interleaved_repetitions_keep_group_order_through_checked_parse_and_rule_output() {
    // The accepted groups are ID, class, attribute, then ordinary pseudo-class;
    // each group's relative order and every repeated member remain observable.
    let checked = CssSelector::Compound(
        CssCompoundSelector::try_new(vec![
            Simple::Class(ident("One")),
            Simple::Id(ident("X")),
            Simple::PseudoClass(CssPseudoClass::Hover),
            attribute("A", "é"),
            Simple::Class(ident("Two")),
            Simple::Id(ident("Y")),
            Simple::PseudoClass(CssPseudoClass::Focus),
            attribute("B", "two"),
            Simple::Class(ident("One")),
            Simple::Id(ident("X")),
            Simple::PseudoClass(CssPseudoClass::Hover),
            attribute("A", "é"),
        ])
        .unwrap(),
    );
    assert_eq!(parsed(AUTHORED), checked);
    assert_eq!(parsed(CANONICAL), checked);
    assert_eq!(checked.to_specified_css().unwrap(), CANONICAL);
    let CssSelector::Compound(compound) = &checked else {
        panic!("checked compound")
    };
    assert_eq!(compound.ids(), ["X", "Y", "X"]);
    assert_eq!(compound.classes(), ["One", "Two", "One"]);
    assert_eq!(
        compound.pseudo_classes(),
        [
            CssPseudoClass::Hover,
            CssPseudoClass::Focus,
            CssPseudoClass::Hover
        ]
    );

    let source = format!("/*😀*/\n{AUTHORED}{{opacity:0}}");
    let report = parse_rule(&source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let rule = report.syntax().as_ref().unwrap();
    let CssRule::Style(style) = rule else {
        panic!("style")
    };
    let [CssStyleSelector::Selector(selector)] = style.selectors().selectors() else {
        panic!("single selector")
    };
    assert_eq!(selector, &checked);
    assert_eq!(style.position().byte_offset().value(), "/*😀*/\n".len());
    let declaration = &style.declarations()[0];
    let saved = declaration.clone();
    let original_name = declaration.parsed_name().unwrap();
    assert_eq!(original_name.source().as_str(), source);
    assert_eq!(
        original_name.span().start().byte_offset().value(),
        source.find("opacity").unwrap()
    );
    assert_eq!(original_name.span().start().line().value(), 1);
    assert_eq!(
        original_name.span().start().column().value() as usize,
        AUTHORED.encode_utf16().count() + 1
    );
    let expected = format!("{CANONICAL} {{ opacity: 0; }}");
    assert_eq!(rule.serialize_cssom().unwrap(), expected);
    assert_eq!(rule.to_specified_css().unwrap(), expected);
    let roundtrip = parse_rule(&expected, &CssNamespaceContext::default());
    assert!(roundtrip.is_clean());
    let CssRule::Style(reparsed) = roundtrip.syntax().as_ref().unwrap() else {
        panic!("reparsed style")
    };
    assert_eq!(reparsed.selectors(), style.selectors());
    assert!(declaration.same_occurrence(&saved));
    assert_eq!(report, before);
}

#[test]
fn category_normalization_keeps_pseudo_element_attachments_order_sensitive() {
    let source = "Leaf:hover.c::part(first second):hover::before::marker";
    let expected = "Leaf.c:hover::part(first second):hover::before::marker";
    let selector = parsed(source);
    let before = selector.clone();
    assert_eq!(selector.to_specified_css().unwrap(), expected);
    // Part names include parsed source provenance in equality. Round-trip output
    // and semantic attachment are checked without erasing that provenance.
    assert_eq!(parsed(expected).to_specified_css().unwrap(), expected);
    let CssSelector::Compound(compound) = &selector else {
        panic!("originating compound")
    };
    assert_eq!(compound.pseudo_classes(), [CssPseudoClass::Hover]);
    let [
        Segment::PseudoElement(CssPseudoElement::Part(names)),
        Segment::PseudoClass(CssPseudoClass::Hover),
        Segment::PseudoElement(CssPseudoElement::Before),
        Segment::PseudoElement(CssPseudoElement::Marker),
    ] = compound.pseudo_elements().unwrap().segments()
    else {
        panic!("ordered part, attached hover, before and marker")
    };
    assert_eq!(names.names()[0].as_str(), "first");
    assert_eq!(names.names()[1].as_str(), "second");
    let CssValueOrigin::Parsed(origin) = names.names()[0].origin() else {
        panic!("original part name provenance")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find("first").unwrap()
    );
    // Moving the attached :hover to the originating element changes both its
    // ordinary pseudo group and its pseudo-element sequence.
    let moved = parsed("Leaf.c:hover:hover::part(first second)::before::marker");
    let CssSelector::Compound(moved) = moved else {
        panic!("moved originating compound")
    };
    assert_eq!(
        moved.pseudo_classes(),
        [CssPseudoClass::Hover, CssPseudoClass::Hover]
    );
    assert!(matches!(
        moved.pseudo_elements().unwrap().segments(),
        [
            Segment::PseudoElement(CssPseudoElement::Part(_)),
            Segment::PseudoElement(CssPseudoElement::Before),
            Segment::PseudoElement(CssPseudoElement::Marker),
        ]
    ));
    let report = parse_rule(&format!("{source}{{}}"), &CssNamespaceContext::default());
    assert!(report.is_clean(), "{report:?}");
    let rule = report.syntax().as_ref().unwrap();
    assert_eq!(rule.serialize_cssom().unwrap(), format!("{expected} {{ }}"));
    assert_eq!(
        rule.to_specified_css().unwrap(),
        format!("{expected} {{ }}")
    );
    assert_eq!(selector, before);
    for invalid in [
        "Leaf.c::before#X",
        "Leaf.c::before[A]",
        "Leaf.c::before > .tail",
    ] {
        let rejected = parse_rule(&format!("{invalid}{{}}"), &CssNamespaceContext::default());
        assert!(rejected.syntax().is_none(), "{invalid}: {rejected:?}");
        assert!(!rejected.is_clean());
    }
}

#[test]
fn interleaved_rule_siblings_share_exact_limits_and_failure_preserves_retry() {
    let report = parse_sheet(&format!("{AUTHORED}{{}}{AUTHORED}{{}}"));
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let expected = format!("{CANONICAL} {{ }}\n{CANONICAL} {{ }}");
    // Per selector: compound1 + ID3 + class3 + attribute3 + pseudo3 =13.
    // Per empty style: rule1 + selector-list1 + build1 + output1 =4 more.
    // Sheet1 + two complete Style17 =35 input and projection visits.
    let adequate = Limits::new(35, 35, expected.len());
    assert_eq!(
        report
            .syntax()
            .serialize_cssom_with_limits(adequate)
            .unwrap(),
        expected
    );
    assert_eq!(
        report
            .syntax()
            .to_specified_css_with_limits(adequate)
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            Limits::new(34, 35, expected.len()),
            Resource::InputNodeLimit,
        ),
        (
            Limits::new(35, 34, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(35, 35, expected.len() - 1), Resource::ByteLimit),
        (
            Limits::new(18, 35, expected.len()),
            Resource::InputNodeLimit,
        ),
        (
            Limits::new(35, 18, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (
            Limits::new(35, 35, CANONICAL.len() + 5),
            Resource::ByteLimit,
        ),
    ] {
        let failure = report
            .syntax()
            .serialize_cssom_with_limits(limits)
            .unwrap_err();
        assert_eq!(failure.kind(), RuleError::Resource(kind));
        assert_eq!(failure.rule_path(), &[1]);
        assert_eq!(report, before);
        assert_eq!(
            report
                .syntax()
                .serialize_cssom_with_limits(adequate)
                .unwrap(),
            expected
        );
    }
}
