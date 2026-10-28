//! The parser crate's token tree, fixed once to surgeist-css's payload.
//!
//! `surgeist_css_parser` is generic over the data a host attaches to a
//! document. CSS attaches [`Attached`]: the checked component a token belongs
//! to, and, to the document as a whole, the [`Host`] that built it. The
//! aliases below close the parameter, so a grammar names `Stream<'i>` and
//! `SyntaxError<'i, E>` and never sees the payload.

use std::rc::Rc;

use crate::component::CssComponentValue;
use crate::source::CssSourceSnapshot;
use crate::values::color_adjust::utf16_value::Control;

/// The document of a syntax tree: parsed source text, or checked components.
pub(crate) type Document<'d> = surgeist_css_parser::Document<'d, Attached<'d>>;
/// A token stream over a [`Document`] (Syntax 3 section 5.3).
pub(crate) type Stream<'i> = surgeist_css_parser::Stream<'i, Attached<'i>>;
/// A boundary in a [`Document`].
pub(crate) type Site<'i> = surgeist_css_parser::Site<'i, Attached<'i>>;
/// A restore point of a [`Stream`].
pub(crate) type Mark<'i> = surgeist_css_parser::stream::Mark<'i, Attached<'i>>;
/// A grammar failure: a stream failure or the grammar's own error `E`.
pub(crate) type SyntaxError<'i, E> = surgeist_css_parser::SyntaxError<'i, E, Attached<'i>>;
/// The token stream could not supply what the grammar asked for.
pub(crate) type StreamError<'i> = surgeist_css_parser::stream::StreamError<'i, Attached<'i>>;
/// Builds a [`Document`] from components that are already tokens.
pub(crate) type TreeBuilder<'d> = surgeist_css_parser::document::TreeBuilder<'d, Attached<'d>>;

/// What CSS attaches to the nodes of a document, and to the document.
#[derive(Clone)]
pub(crate) enum Attached<'d> {
    /// The payload of a node of a checked document: the component whose token
    /// the node holds, whether the node is a leaf, a group's opening or (on
    /// the group's end) its closing token.
    Component(&'d CssComponentValue),
    /// The payload of the document as a whole.
    Host(Rc<Host<'d>>),
}

impl std::fmt::Debug for Attached<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Component(component) => {
                formatter.debug_tuple("Component").field(component).finish()
            }
            Self::Host(host) => formatter
                .debug_struct("Host")
                .field("has_snapshot", &host.snapshot.is_some())
                .field("controlled", &host.control.is_some())
                .finish(),
        }
    }
}

/// What a document's readers need about the document that its nodes cannot
/// say: the snapshot its positions and origins belong to, and the control that
/// meters it.
pub(crate) struct Host<'d> {
    snapshot: Option<CssSourceSnapshot>,
    control: Option<&'d Control<'d>>,
}

impl<'d> Host<'d> {
    /// The host of a document parsed from the text of `snapshot`.
    pub(crate) fn source(snapshot: &CssSourceSnapshot) -> Self {
        Self {
            snapshot: Some(snapshot.clone()),
            control: None,
        }
    }

    /// The host of a document of checked components, metered by `control` when
    /// there is one, whose positions belong to `snapshot` when there is one.
    pub(crate) fn checked(
        snapshot: Option<&CssSourceSnapshot>,
        control: Option<&'d Control<'d>>,
    ) -> Self {
        Self {
            snapshot: snapshot.cloned(),
            control,
        }
    }

    pub(crate) fn into_attached(self) -> Attached<'d> {
        Attached::Host(Rc::new(self))
    }
}

/// The host a reader reaches from a document or a site.
pub(crate) fn host<'d>(document: &'d Document<'d>) -> Option<&'d Host<'d>> {
    match document.attached()? {
        Attached::Host(host) => Some(host),
        Attached::Component(_) => None,
    }
}

impl<'d> Host<'d> {
    pub(crate) fn snapshot(&self) -> Option<&CssSourceSnapshot> {
        self.snapshot.as_ref()
    }

    pub(crate) fn control(&self) -> Option<&'d Control<'d>> {
        self.control
    }
}
