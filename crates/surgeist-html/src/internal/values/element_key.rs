//! The key of an element name: its namespace, the known vocabulary entry it spells, and the
//! name classes the tree construction algorithm tests.

use crate::{
    internal::values::{
        HTML_NAMESPACE_URI, MATHML_NAMESPACE_URI, SVG_NAMESPACE_URI,
        name_hash::{at, units_spell},
        semantic_input::HtmlNameInput,
    },
    vocabulary::{
        KnownHtmlElement, KnownMathmlElement, KnownSvgElement, MATHML_SPECIAL_ELEMENTS,
        MATHML_TEXT_INTEGRATION_POINTS, SVG_HTML_INTEGRATION_POINTS, SVG_SPECIAL_ELEMENTS,
    },
};
use std::fmt;

/// The namespace of an [`ElementKey`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ElementNamespace {
    /// The HTML namespace.
    Html,
    /// The MathML namespace.
    MathMl,
    /// The SVG namespace.
    Svg,
    /// No namespace, or any namespace other than the three above.
    Other,
}

const HTML: u8 = 0;
const MATHML: u8 = 1;
const SVG: u8 = 2;
const OTHER: u8 = 3;

const NAMESPACE_SHIFT: u32 = 24;
const TAG_SHIFT: u32 = 8;

const _: () = {
    assert!(KnownHtmlElement::ALL.len() < u16::MAX as usize);
    assert!(KnownMathmlElement::ALL.len() < u16::MAX as usize);
    assert!(KnownSvgElement::ALL.len() < u16::MAX as usize);
};

/// A MathML text integration point.
const TEXT_INTEGRATION_POINT: u8 = 1;
/// An SVG element that is an HTML integration point by name.
const HTML_INTEGRATION_POINT: u8 = 2;
/// A MathML or SVG element in the special category.
const FOREIGN_SPECIAL: u8 = 4;

/// An element name reduced to one integer: the namespace in the top byte, the known tag in the
/// next two bytes and the name classes in the low byte.
///
/// A key is computed from an element's namespace and local name, once, where the name is
/// created. Comparing and hashing keys compares and hashes one 32-bit integer.
///
/// A name of the HTML, MathML or SVG namespace that spells an entry of the vocabulary of its
/// namespace has a known tag: the position of the entry plus one. Any other name has tag zero,
/// the unknown key of its namespace, which carries no information about the name. Equality of two
/// keys therefore says that two names are the same name only when both keys are known
/// ([`Self::is_known`]); two unknown keys of one namespace are equal whatever the names are.
/// A consumer that needs the identity of unknown names keeps the names and compares them.
///
/// The known vocabularies are the HTML elements ([`KnownHtmlElement`]) and the MathML and SVG
/// names that the tree construction algorithm adjusts or tests ([`KnownMathmlElement`],
/// [`KnownSvgElement`]). Case is not folded: a key is computed from the spelling of an element
/// name, after any tag name adjustment.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ElementKey(u32);

impl ElementKey {
    const fn pack(namespace: u8, tag: u16, flags: u8) -> Self {
        Self(((namespace as u32) << NAMESPACE_SHIFT) | ((tag as u32) << TAG_SHIFT) | flags as u32)
    }

    /// The tag of the entry at `position`: one more than the position, so that zero is unknown.
    /// The assertions above prove that no position of a vocabulary reaches `u16::MAX`.
    const fn tag_of(position: usize) -> u16 {
        (position as u16).wrapping_add(1)
    }

    const fn of_html(known: KnownHtmlElement) -> Self {
        Self::pack(HTML, Self::tag_of(known as usize), 0)
    }

    const fn of_mathml(known: KnownMathmlElement) -> Self {
        let mut flags = 0;
        if mathml_in(&MATHML_TEXT_INTEGRATION_POINTS, known) {
            flags |= TEXT_INTEGRATION_POINT;
        }
        if mathml_in(&MATHML_SPECIAL_ELEMENTS, known) {
            flags |= FOREIGN_SPECIAL;
        }
        Self::pack(MATHML, Self::tag_of(known as usize), flags)
    }

    const fn of_svg(known: KnownSvgElement) -> Self {
        let mut flags = 0;
        if svg_in(&SVG_HTML_INTEGRATION_POINTS, known) {
            flags |= HTML_INTEGRATION_POINT;
        }
        if svg_in(&SVG_SPECIAL_ELEMENTS, known) {
            flags |= FOREIGN_SPECIAL;
        }
        Self::pack(SVG, Self::tag_of(known as usize), flags)
    }

    /// The key of an element name, from the UTF-16 units of its namespace URI (`None` for no
    /// namespace) and of its local name. The prefix does not take part.
    ///
    /// The computation is `const`, allocation-free and bounded by the longest known name
    /// whatever the length of the units.
    pub const fn from_units(namespace: Option<&[u16]>, local_name: &[u16]) -> Self {
        let Some(uri) = namespace else {
            return Self::pack(OTHER, 0, 0);
        };
        if units_spell(uri, HTML_NAMESPACE_URI) {
            match KnownHtmlElement::recognize(local_name) {
                Some(known) => Self::of_html(known),
                None => Self::pack(HTML, 0, 0),
            }
        } else if units_spell(uri, MATHML_NAMESPACE_URI) {
            match KnownMathmlElement::recognize(local_name) {
                Some(known) => Self::of_mathml(known),
                None => Self::pack(MATHML, 0, 0),
            }
        } else if units_spell(uri, SVG_NAMESPACE_URI) {
            match KnownSvgElement::recognize(local_name) {
                Some(known) => Self::of_svg(known),
                None => Self::pack(SVG, 0, 0),
            }
        } else {
            Self::pack(OTHER, 0, 0)
        }
    }

    /// The key of an expanded name, as [`Self::from_units`] computes it.
    pub fn from_name(name: &(impl HtmlNameInput + ?Sized)) -> Self {
        Self::from_units(
            name.namespace_uri().map(|uri| uri.code_units()),
            name.local_name().code_units(),
        )
    }

    /// The namespace of the key.
    pub const fn namespace(self) -> ElementNamespace {
        match (self.0 >> NAMESPACE_SHIFT) as u8 {
            HTML => ElementNamespace::Html,
            MATHML => ElementNamespace::MathMl,
            SVG => ElementNamespace::Svg,
            _ => ElementNamespace::Other,
        }
    }

    const fn tag(self) -> u16 {
        (self.0 >> TAG_SHIFT) as u16
    }

    const fn flags(self) -> u8 {
        self.0 as u8
    }

    /// Whether the name is an entry of the vocabulary of its namespace.
    pub const fn is_known(self) -> bool {
        self.tag() != 0
    }

    /// The HTML element of a known HTML-namespace key.
    pub const fn html(self) -> Option<KnownHtmlElement> {
        match self.namespace() {
            ElementNamespace::Html => match self.tag().checked_sub(1) {
                Some(position) => at(KnownHtmlElement::ALL, position as usize),
                None => None,
            },
            _ => None,
        }
    }

    /// The MathML element of a known MathML-namespace key.
    pub const fn mathml(self) -> Option<KnownMathmlElement> {
        match self.namespace() {
            ElementNamespace::MathMl => match self.tag().checked_sub(1) {
                Some(position) => at(KnownMathmlElement::ALL, position as usize),
                None => None,
            },
            _ => None,
        }
    }

    /// The SVG element of a known SVG-namespace key.
    pub const fn svg(self) -> Option<KnownSvgElement> {
        match self.namespace() {
            ElementNamespace::Svg => match self.tag().checked_sub(1) {
                Some(position) => at(KnownSvgElement::ALL, position as usize),
                None => None,
            },
            _ => None,
        }
    }

    /// Whether this is a MathML text integration point (HTML 13.2.6): `mi`, `mo`, `mn`, `ms` or
    /// `mtext` in the MathML namespace.
    pub const fn is_mathml_text_integration_point(self) -> bool {
        self.flags() & TEXT_INTEGRATION_POINT != 0
    }

    /// Whether this is an SVG element that is an HTML integration point by name (HTML 13.2.6):
    /// `foreignObject`, `desc` or `title`. A MathML `annotation-xml` is one only with a matching
    /// `encoding` attribute, which a key does not know.
    pub const fn is_svg_html_integration_point(self) -> bool {
        self.flags() & HTML_INTEGRATION_POINT != 0
    }

    /// Whether this is a MathML or SVG element of the special category (HTML 13.2.4.2).
    pub const fn is_foreign_special(self) -> bool {
        self.flags() & FOREIGN_SPECIAL != 0
    }
}

const fn mathml_in(group: &[KnownMathmlElement], known: KnownMathmlElement) -> bool {
    let mut rest = group;
    while let Some((&member, tail)) = rest.split_first() {
        if member as usize == known as usize {
            return true;
        }
        rest = tail;
    }
    false
}

const fn svg_in(group: &[KnownSvgElement], known: KnownSvgElement) -> bool {
    let mut rest = group;
    while let Some((&member, tail)) = rest.split_first() {
        if member as usize == known as usize {
            return true;
        }
        rest = tail;
    }
    false
}

impl From<KnownHtmlElement> for ElementKey {
    fn from(known: KnownHtmlElement) -> Self {
        Self::of_html(known)
    }
}

impl From<KnownMathmlElement> for ElementKey {
    fn from(known: KnownMathmlElement) -> Self {
        Self::of_mathml(known)
    }
}

impl From<KnownSvgElement> for ElementKey {
    fn from(known: KnownSvgElement) -> Self {
        Self::of_svg(known)
    }
}

impl fmt::Debug for ElementKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.namespace() {
            ElementNamespace::Html => self.html().map(KnownHtmlElement::local_name),
            ElementNamespace::MathMl => self.mathml().map(KnownMathmlElement::local_name),
            ElementNamespace::Svg => self.svg().map(KnownSvgElement::local_name),
            ElementNamespace::Other => None,
        };
        formatter
            .debug_struct("ElementKey")
            .field("namespace", &self.namespace())
            .field("name", &name)
            .field("flags", &format_args!("{:#06b}", self.flags()))
            .finish()
    }
}
