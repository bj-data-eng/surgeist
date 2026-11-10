//! Element keys and the foreign name tables.
//!
//! The expected tables are the pinned HTML Standard's: the SVG tag name adjustment table of
//! 13.2.6.5, the integration points of 13.2.6 and the special category and default scope list of
//! 13.2.4.2.

use std::collections::HashSet;
use surgeist_html::{
    values::{DocumentString, Namespace, QualifiedName},
    vocabulary::{
        ElementKey, ElementNamespace, KnownHtmlElement, KnownMathmlElement, KnownSvgElement,
        MATHML_SCOPE_BOUNDARIES, MATHML_SPECIAL_ELEMENTS, MATHML_TEXT_INTEGRATION_POINTS,
        SVG_HTML_INTEGRATION_POINTS, SVG_SCOPE_BOUNDARIES, SVG_SPECIAL_ELEMENTS,
        SVG_TAG_NAME_ADJUSTMENTS,
    },
};

const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
const MATHML_NAMESPACE: &str = "http://www.w3.org/1998/Math/MathML";
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

const ADJUSTMENTS: [(&str, &str); 37] = [
    ("altglyph", "altGlyph"),
    ("altglyphdef", "altGlyphDef"),
    ("altglyphitem", "altGlyphItem"),
    ("animatecolor", "animateColor"),
    ("animatemotion", "animateMotion"),
    ("animatetransform", "animateTransform"),
    ("clippath", "clipPath"),
    ("feblend", "feBlend"),
    ("fecolormatrix", "feColorMatrix"),
    ("fecomponenttransfer", "feComponentTransfer"),
    ("fecomposite", "feComposite"),
    ("feconvolvematrix", "feConvolveMatrix"),
    ("fediffuselighting", "feDiffuseLighting"),
    ("fedisplacementmap", "feDisplacementMap"),
    ("fedistantlight", "feDistantLight"),
    ("fedropshadow", "feDropShadow"),
    ("feflood", "feFlood"),
    ("fefunca", "feFuncA"),
    ("fefuncb", "feFuncB"),
    ("fefuncg", "feFuncG"),
    ("fefuncr", "feFuncR"),
    ("fegaussianblur", "feGaussianBlur"),
    ("feimage", "feImage"),
    ("femerge", "feMerge"),
    ("femergenode", "feMergeNode"),
    ("femorphology", "feMorphology"),
    ("feoffset", "feOffset"),
    ("fepointlight", "fePointLight"),
    ("fespecularlighting", "feSpecularLighting"),
    ("fespotlight", "feSpotLight"),
    ("fetile", "feTile"),
    ("feturbulence", "feTurbulence"),
    ("foreignobject", "foreignObject"),
    ("glyphref", "glyphRef"),
    ("lineargradient", "linearGradient"),
    ("radialgradient", "radialGradient"),
    ("textpath", "textPath"),
];

const MATHML_TEXT_POINTS: [&str; 5] = ["mi", "mo", "mn", "ms", "mtext"];
const MATHML_SPECIAL: [&str; 6] = ["mi", "mo", "mn", "ms", "mtext", "annotation-xml"];
const SVG_SPECIAL: [&str; 3] = ["foreignObject", "desc", "title"];

fn names<T: Copy>(group: &[T], local_name: fn(T) -> &'static str) -> Vec<&'static str> {
    group.iter().map(|known| local_name(*known)).collect()
}

fn key(namespace: &str, local: &str) -> ElementKey {
    let uri: Vec<u16> = namespace.encode_utf16().collect();
    let local: Vec<u16> = local.encode_utf16().collect();
    ElementKey::from_units(Some(&uri), &local)
}

#[test]
fn the_svg_tag_name_adjustments_are_the_pinned_table() {
    let generated: Vec<(&str, &str)> = SVG_TAG_NAME_ADJUSTMENTS
        .iter()
        .map(|(original, element)| (*original, element.local_name()))
        .collect();
    assert_eq!(generated, ADJUSTMENTS);
}

#[test]
fn the_foreign_name_groups_are_the_pinned_lists() {
    assert_eq!(
        names(
            &MATHML_TEXT_INTEGRATION_POINTS,
            KnownMathmlElement::local_name
        ),
        MATHML_TEXT_POINTS
    );
    assert_eq!(
        names(&SVG_HTML_INTEGRATION_POINTS, KnownSvgElement::local_name),
        SVG_SPECIAL
    );
    assert_eq!(
        names(&MATHML_SPECIAL_ELEMENTS, KnownMathmlElement::local_name),
        MATHML_SPECIAL
    );
    assert_eq!(
        names(&SVG_SPECIAL_ELEMENTS, KnownSvgElement::local_name),
        SVG_SPECIAL
    );
    assert_eq!(
        names(&MATHML_SCOPE_BOUNDARIES, KnownMathmlElement::local_name),
        MATHML_SPECIAL
    );
    assert_eq!(
        names(&SVG_SCOPE_BOUNDARIES, KnownSvgElement::local_name),
        SVG_SPECIAL
    );
}

#[test]
fn the_foreign_vocabularies_hold_the_names_the_tree_builder_tests_and_adjusts() {
    assert_eq!(
        names(KnownMathmlElement::ALL, KnownMathmlElement::local_name),
        MATHML_SPECIAL
    );
    let mut svg: Vec<&str> = ADJUSTMENTS.iter().map(|(_, adjusted)| *adjusted).collect();
    svg.extend(["desc", "title", "script"]);
    assert_eq!(
        names(KnownSvgElement::ALL, KnownSvgElement::local_name),
        svg
    );
    assert_eq!(
        KnownMathmlElement::AnnotationXml.local_name(),
        "annotation-xml"
    );
    assert_eq!(
        KnownSvgElement::ForeignObject.qualified_name(),
        QualifiedName::new(Namespace::svg(), None, "foreignObject")
    );
    assert_eq!(
        KnownMathmlElement::Mtext.qualified_name(),
        QualifiedName::new(Namespace::mathml(), None, "mtext")
    );
}

#[test]
fn every_known_name_has_its_own_known_key_that_reads_back() {
    let mut seen = HashSet::new();
    for known in KnownHtmlElement::ALL {
        let computed = key(HTML_NAMESPACE, known.local_name());
        assert_eq!(computed, ElementKey::from(*known));
        assert_eq!(computed.namespace(), ElementNamespace::Html);
        assert_eq!(computed.html(), Some(*known));
        assert!(computed.is_known());
        assert!(seen.insert(computed), "{known:?}");
    }
    for known in KnownMathmlElement::ALL {
        let computed = key(MATHML_NAMESPACE, known.local_name());
        assert_eq!(computed, ElementKey::from(*known));
        assert_eq!(computed.namespace(), ElementNamespace::MathMl);
        assert_eq!(computed.mathml(), Some(*known));
        assert!(seen.insert(computed), "{known:?}");
    }
    for known in KnownSvgElement::ALL {
        let computed = key(SVG_NAMESPACE, known.local_name());
        assert_eq!(computed, ElementKey::from(*known));
        assert_eq!(computed.namespace(), ElementNamespace::Svg);
        assert_eq!(computed.svg(), Some(*known));
        assert!(seen.insert(computed), "{known:?}");
    }
    // The same spelling in two namespaces is two keys.
    assert_ne!(key(HTML_NAMESPACE, "title"), key(SVG_NAMESPACE, "title"));
    assert_eq!(
        seen.len(),
        KnownHtmlElement::ALL.len() + KnownMathmlElement::ALL.len() + KnownSvgElement::ALL.len()
    );
}

#[test]
fn an_unknown_name_has_the_unknown_key_of_its_namespace() {
    let unknown_html = key(HTML_NAMESPACE, "x-widget");
    assert!(!unknown_html.is_known());
    assert_eq!(unknown_html.namespace(), ElementNamespace::Html);
    assert_eq!(unknown_html.html(), None);
    // Every unknown name of a namespace shares one key: it is not an identity of the name.
    assert_eq!(unknown_html, key(HTML_NAMESPACE, "other-widget"));
    assert_eq!(unknown_html, key(HTML_NAMESPACE, ""));
    assert_ne!(unknown_html, key(SVG_NAMESPACE, "x-widget"));
    assert_ne!(unknown_html, key(HTML_NAMESPACE, "div"));
    for (namespace, expected) in [
        (MATHML_NAMESPACE, ElementNamespace::MathMl),
        (SVG_NAMESPACE, ElementNamespace::Svg),
    ] {
        let unknown = key(namespace, "unlisted");
        assert!(!unknown.is_known());
        assert_eq!(unknown.namespace(), expected);
        assert_eq!(unknown.mathml(), None);
        assert_eq!(unknown.svg(), None);
        assert!(!unknown.is_foreign_special());
    }
    // A name that is known in another namespace, or in another case, is unknown here.
    for (namespace, local) in [
        (HTML_NAMESPACE, "foreignObject"),
        (HTML_NAMESPACE, "DIV"),
        (SVG_NAMESPACE, "div"),
        (SVG_NAMESPACE, "foreignobject"),
        (SVG_NAMESPACE, "clippath"),
        (MATHML_NAMESPACE, "title"),
        (MATHML_NAMESPACE, "MI"),
    ] {
        assert!(!key(namespace, local).is_known(), "{namespace} {local}");
    }
}

#[test]
fn a_name_without_a_known_namespace_has_the_other_key() {
    let other = ElementKey::from_units(None, &[0x64, 0x69, 0x76]);
    assert_eq!(other.namespace(), ElementNamespace::Other);
    assert!(!other.is_known());
    for namespace in [
        "urn:other",
        "",
        "HTTP://www.w3.org/1999/xhtml",
        "http://www.w3.org/1999/xhtml ",
        "http://www.w3.org/1999/xhtm",
    ] {
        assert_eq!(key(namespace, "div"), other, "{namespace:?}");
    }
    let lone_surrogate = ElementKey::from_units(Some(&[0xD800]), &[0x64]);
    assert_eq!(lone_surrogate, other);
}

#[test]
fn a_key_from_a_name_ignores_the_prefix_and_equals_the_key_from_its_units() {
    let prefixed = QualifiedName::new(
        Namespace::html(),
        Some(DocumentString::from("h")),
        DocumentString::from("div"),
    );
    assert_eq!(
        ElementKey::from_name(&prefixed),
        ElementKey::from(KnownHtmlElement::Div)
    );
    for known in KnownSvgElement::ALL {
        assert_eq!(
            ElementKey::from_name(&known.qualified_name()),
            ElementKey::from(*known)
        );
    }
    assert_eq!(
        ElementKey::from_name(&QualifiedName::new(Namespace::none(), None, "div")),
        ElementKey::from_units(None, &[0x64, 0x69, 0x76])
    );
}

#[test]
fn the_class_flags_are_the_membership_of_the_pinned_lists() {
    for known in KnownMathmlElement::ALL {
        let computed = ElementKey::from(*known);
        let name = known.local_name();
        assert_eq!(
            computed.is_mathml_text_integration_point(),
            MATHML_TEXT_POINTS.contains(&name),
            "{name}"
        );
        assert_eq!(
            computed.is_foreign_special(),
            MATHML_SPECIAL.contains(&name),
            "{name}"
        );
        assert!(!computed.is_svg_html_integration_point(), "{name}");
    }
    for known in KnownSvgElement::ALL {
        let computed = ElementKey::from(*known);
        let name = known.local_name();
        assert_eq!(
            computed.is_svg_html_integration_point(),
            SVG_SPECIAL.contains(&name),
            "{name}"
        );
        assert_eq!(
            computed.is_foreign_special(),
            SVG_SPECIAL.contains(&name),
            "{name}"
        );
        assert!(!computed.is_mathml_text_integration_point(), "{name}");
    }
    for known in KnownHtmlElement::ALL {
        let computed = ElementKey::from(*known);
        assert!(!computed.is_mathml_text_integration_point());
        assert!(!computed.is_svg_html_integration_point());
        assert!(!computed.is_foreign_special());
    }
    // The classes belong to the name in its namespace: the HTML `title` is not an SVG one.
    assert!(!key(HTML_NAMESPACE, "title").is_svg_html_integration_point());
    assert!(key(SVG_NAMESPACE, "title").is_svg_html_integration_point());
    assert!(!key(HTML_NAMESPACE, "mi").is_mathml_text_integration_point());
}

#[test]
fn a_key_is_one_copyable_integer_that_names_its_parts_in_debug_output() {
    assert_eq!(std::mem::size_of::<ElementKey>(), 4);
    let copied = ElementKey::from(KnownSvgElement::Title);
    let same = copied;
    assert_eq!(copied, same);
    let shown = format!("{copied:?}");
    assert!(
        shown.contains("Svg") && shown.contains("\"title\""),
        "{shown}"
    );
}
