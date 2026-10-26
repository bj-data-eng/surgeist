//! CSSOM "serialize a selector" serializes each simple selector of a compound
//! in order (CSSOM ED 2026-08-31 §5.2). Mixed authored category order and
//! repetition survive checked construction, parsing and rule output, while the
//! per-category accessors keep their grouped matching-relevant view.

use crate::css::support::failure_classes::SerializationFailure as Resource;
use crate::css::support::serialization_limits::LimitFailure;
use crate::css::support::validated_selector as parsed;
use surgeist_css::{
    CssAttributeCaseSensitivity, CssAttributeMatcher, CssAttributeName, CssAttributeSelector,
    CssCompoundSelector, CssIdent, CssNamespaceContext, CssPseudoClass, CssPseudoElement,
    CssPseudoElementSegment as Segment, CssQualifiedAttributeName, CssQualifiedNamePrefix, CssRule,
    CssSelector, CssSerializationLimits as Limits, CssSimpleSelector as Simple, CssStyleSelector,
    CssStyleSelectorContext, CssValueOrigin, parse_rule, parse_sheet,
};

const AUTHORED: &str = ".One#X:hover[A='é'].Two#Y:focus[B='two'].One#X:hover[A='é']";
const SERIALIZED: &str = ".One#X:hover[A=\"é\"].Two#Y:focus[B=\"two\"].One#X:hover[A=\"é\"]";
const GROUPED: &str = "#X#Y#X.One.Two.One[A=\"é\"][B=\"two\"][A=\"é\"]:hover:focus:hover";

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
            Default::default(),
        )
        .unwrap(),
    )
}

#[test]
fn interleaved_repetitions_keep_authored_order_through_checked_parse_and_rule_output() {
    // Members serialize in authored order; the grouped accessors still expose
    // each category's relative order and every repeated member.
    let checked = CssSelector::Compound(
        CssCompoundSelector::try_new(
            vec![
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
            ],
            Default::default(),
        )
        .unwrap(),
    );
    assert_eq!(parsed(AUTHORED), checked);
    assert_eq!(parsed(SERIALIZED), checked);
    assert_ne!(parsed(GROUPED), checked);
    assert_eq!(
        parsed(GROUPED)
            .serialize_specified(Default::default())
            .unwrap(),
        GROUPED
    );
    assert_eq!(
        checked.serialize_specified(Default::default()).unwrap(),
        SERIALIZED
    );
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
    let expected = format!("{SERIALIZED} {{ opacity: 0; }}");
    assert_eq!(rule.serialize_cssom(Default::default()).unwrap(), expected);
    assert_eq!(
        rule.serialize_specified(Default::default()).unwrap(),
        expected
    );
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
fn authored_category_order_keeps_pseudo_element_attachments_order_sensitive() {
    let source = "Leaf:hover.c::part(first second):hover::before::marker";
    let expected = source;
    let selector = parsed(source);
    let before = selector.clone();
    assert_eq!(
        selector.serialize_specified(Default::default()).unwrap(),
        expected
    );
    // Part names include parsed source provenance in equality. Round-trip output
    // and semantic attachment are checked without erasing that provenance.
    assert_eq!(
        parsed(expected)
            .serialize_specified(Default::default())
            .unwrap(),
        expected
    );
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
    // ordinary pseudo-class list and its pseudo-element sequence.
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
    assert_eq!(
        rule.serialize_cssom(Default::default()).unwrap(),
        format!("{expected} {{ }}")
    );
    assert_eq!(
        rule.serialize_specified(Default::default()).unwrap(),
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
        crate::css::support::value_assertions::assert_first_code(
            rejected.diagnostics(),
            surgeist_css::CssErrorCode::InvalidSelector,
        );
    }
}

#[test]
fn interleaved_rule_siblings_share_exact_limits_and_failure_preserves_retry() {
    let report = parse_sheet(&format!("{AUTHORED}{{}}{AUTHORED}{{}}"));
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let expected = format!("{SERIALIZED} {{ }}\n{SERIALIZED} {{ }}");
    // Per selector: compound1 + ID3 + class3 + attribute3 + pseudo3 =13.
    // Per empty style: rule1 + selector-list1 + build1 + output1 =4 more.
    // Sheet1 + two complete Style17 =35 input and projection visits.
    let adequate = Limits::new(35, 35, expected.len());
    assert_eq!(report.syntax().serialize_cssom(adequate).unwrap(), expected);
    assert_eq!(
        report.syntax().serialize_specified(adequate).unwrap(),
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
            Limits::new(35, 35, SERIALIZED.len() + 5),
            Resource::ByteLimit,
        ),
    ] {
        let failure = report.syntax().serialize_cssom(limits).unwrap_err();
        assert_eq!(failure.limit_kind(), Some(kind));
        assert_eq!(failure.rule_path(), &[1]);
        assert_eq!(report, before);
        assert_eq!(report.syntax().serialize_cssom(adequate).unwrap(), expected);
    }
}

fn nested_selector_texts(source: &str) -> Vec<(String, String, usize)> {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {report:?}");
    let [CssRule::Style(outer)] = report.syntax().rules() else {
        panic!("{source}: one outer style rule")
    };
    outer
        .rules()
        .iter()
        .map(|rule| {
            let CssRule::Style(inner) = rule else {
                panic!("{source}: nested style rule")
            };
            let [CssStyleSelector::Selector(selector)] = inner.selectors().selectors() else {
                panic!("{source}: one selector")
            };
            let CssSelector::Compound(compound) = selector else {
                panic!("{source}: one compound selector")
            };
            (
                inner
                    .selectors()
                    .serialize_cssom(
                        &CssNamespaceContext::default(),
                        CssStyleSelectorContext::Nested,
                        Default::default(),
                    )
                    .unwrap(),
                selector.serialize_specified(Default::default()).unwrap(),
                compound.nesting_selectors(),
            )
        })
        .collect()
}

#[test]
fn nesting_selector_serializes_at_its_authored_compound_position() {
    // CSS Nesting 1 §2: `&` is a simple selector that may appear anywhere in a
    // compound; CSSOM serializes the compound's simple selectors in order. The
    // nesting count, which fixes its specificity contribution, is unchanged.
    let cases = [
        (".class&", 1),
        ("div&.a", 1),
        (".a&.b", 1),
        ("&.a&", 2),
        (":hover&#x", 1),
        ("[x]&:is(div)", 1),
    ];
    let source = format!(
        ".foo{{{}}}",
        cases
            .iter()
            .map(|(selector, _)| format!("{selector}{{color:green}}"))
            .collect::<String>()
    );
    let expected: Vec<(String, String, usize)> = cases
        .iter()
        .map(|(selector, count)| ((*selector).to_owned(), (*selector).to_owned(), *count))
        .collect();
    assert_eq!(nested_selector_texts(&source), expected);
}

#[test]
fn scope_anchor_serializes_at_its_authored_compound_position() {
    // Inside `@scope`, `&` is a scope anchor (Cascade 6 §2.5); it keeps its
    // authored compound position the same way.
    let report = parse_sheet("@scope(.outer){.a&{color:green}:hover&.b{color:green}}");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("one scope rule")
    };
    let actual: Vec<(String, usize)> = scope
        .rules()
        .rules()
        .iter()
        .map(|rule| {
            let surgeist_css::CssRule::Style(style) = rule else {
                panic!("scoped style rule")
            };
            let [surgeist_css::CssStyleSelector::Selector(selector)] =
                style.selectors().selectors()
            else {
                panic!("one scoped selector")
            };
            let CssSelector::Compound(compound) = selector else {
                panic!("one compound")
            };
            (
                selector.serialize_specified(Default::default()).unwrap(),
                compound.scope_anchors(),
            )
        })
        .collect();
    assert_eq!(actual, [(".a&".to_owned(), 1), (":hover&.b".to_owned(), 1)]);
}
