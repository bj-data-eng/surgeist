//! What each open element is to the scope walks, decided once when it is pushed.
//!
//! HTML §13.2.4.2 (the stack of open elements) defines "has an element in
//! scope" by walking the stack from the current node and stopping at the first
//! element that is the target or a boundary of the scope. Which names bound
//! which scope never changes, so the walk need not compare names again for
//! every token: each entry records its class when it is pushed, and the stack
//! keeps ascending lists of the indices of the classes a query asks for.

use super::{
    Scope,
    tag_sets::{self, HTML, MATHML, SVG},
};
use crate::parser::control::{HtmlParseStop, HtmlParseWorkCategory, ParseControl};
use surgeist_html::{
    values::QualifiedName,
    vocabulary::{MATHML_SCOPE_BOUNDARIES, SVG_SCOPE_BOUNDARIES},
};

/// A kind of open element the stack lists by index, in ascending order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Listing {
    /// An HTML `template`.
    Template,
    /// An HTML `p`.
    Paragraph,
    /// A boundary of the default scope other than a `template`: the HTML
    /// `applet`, `caption`, `html`, `table`, `td`, `th`, `marquee`, `object`
    /// and `select`, and the MathML and SVG integration elements.
    ScopeBoundary,
    /// An HTML `button`, which also bounds button scope.
    ButtonBoundary,
}
impl Listing {
    pub(super) const ALL: [Self; 4] = [
        Self::Template,
        Self::Paragraph,
        Self::ScopeBoundary,
        Self::ButtonBoundary,
    ];
}

/// The classes of one open element, as a set of flags.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ElementClass(u8);

const TEMPLATE: u8 = 1;
const PARAGRAPH: u8 = 1 << 1;
const SCOPE: u8 = 1 << 2;
const LIST: u8 = 1 << 3;
const BUTTON: u8 = 1 << 4;
const TABLE: u8 = 1 << 5;

/// The HTML local names with a class, and their flags. A name absent here has
/// none. Table scope's boundaries are `html`, `table` and `template`; every
/// other scope's are the default list, `ol` and `ul` for list items, and
/// `button` for buttons.
const HTML_CLASSES: [(&str, u8); 14] = [
    ("p", PARAGRAPH),
    ("applet", SCOPE),
    ("caption", SCOPE),
    ("html", SCOPE | TABLE),
    ("table", SCOPE | TABLE),
    ("td", SCOPE),
    ("th", SCOPE),
    ("marquee", SCOPE),
    ("object", SCOPE),
    ("select", SCOPE),
    ("template", TEMPLATE | TABLE),
    ("ol", LIST),
    ("ul", LIST),
    ("button", BUTTON),
];

impl ElementClass {
    /// Classifies an element by its name. This is the only place the names are
    /// compared; the charges are those of comparing them.
    pub(super) fn classify(
        name: &QualifiedName,
        control: &mut ParseControl,
    ) -> Result<Self, HtmlParseStop> {
        if tag_sets::namespace(name, HTML, control)? {
            for (candidate, flags) in HTML_CLASSES {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if tag_sets::ascii_eq(name.local_name(), candidate, control)? {
                    return Ok(Self(flags));
                }
            }
            return Ok(Self(0));
        }
        // annotation-xml bounds scope whatever its encoding.
        let bounds = if tag_sets::namespace(name, MATHML, control)? {
            tag_sets::local_in_names(
                name,
                MATHML_SCOPE_BOUNDARIES
                    .iter()
                    .map(|known| known.local_name()),
                control,
            )?
        } else if tag_sets::namespace(name, SVG, control)? {
            tag_sets::local_in_names(
                name,
                SVG_SCOPE_BOUNDARIES.iter().map(|known| known.local_name()),
                control,
            )?
        } else {
            false
        };
        Ok(Self(if bounds { SCOPE } else { 0 }))
    }

    /// Whether the stack lists this element under `listing`.
    pub(super) fn listed(self, listing: Listing) -> bool {
        self.0
            & match listing {
                Listing::Template => TEMPLATE,
                Listing::Paragraph => PARAGRAPH,
                Listing::ScopeBoundary => SCOPE,
                Listing::ButtonBoundary => BUTTON,
            }
            != 0
    }

    /// Whether an element of this class bounds `scope`.
    pub(super) fn bounds(self, scope: Scope) -> bool {
        self.0
            & match scope {
                Scope::Default => TEMPLATE | SCOPE,
                Scope::ListItem => TEMPLATE | SCOPE | LIST,
                Scope::Button => TEMPLATE | SCOPE | BUTTON,
                Scope::Table => TABLE,
            }
            != 0
    }
}
