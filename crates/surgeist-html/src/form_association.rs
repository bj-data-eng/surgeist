//! Native association categories, HTML877d §4.10.2. These values grant no
//! creation profile, custom definition, form-owner identity or execution capability.

use crate::KnownHtmlElement;

// One semantic family mapping. Canonical tag spellings remain owned by
// KnownHtmlElement; there is no second name inventory here. The first six
// candidates preserve the ordinary parser creation comparison order.
macro_rules! native_form_kinds {
    ($($kind:ident => $element:ident),+ $(,)?) => {
        /// A native HTML element's intrinsic form-association category.
        ///
        /// This excludes form-associated custom elements: their actual definition
        /// supplies the qualified form-associated flag. Native classification is
        /// independent of parser metadata admission and input type state.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum HtmlNativeFormAssociationKind {
            $($kind),+
        }

        impl HtmlNativeFormAssociationKind {
            /// Classifies an already recognized exact HTML element identity.
            pub const fn from_element(element: KnownHtmlElement) -> Option<Self> {
                match element {
                    $(KnownHtmlElement::$element => Some(Self::$kind),)+
                    _ => None,
                }
            }

            /// Returns the canonical vocabulary identity for this native kind.
            pub const fn element(self) -> KnownHtmlElement {
                match self { $(Self::$kind => KnownHtmlElement::$element),+ }
            }

            /// Native listedness does not depend on input type or attributes.
            pub const fn listed(self) -> bool {
                !matches!(self, Self::Image)
            }
        }

        const CANDIDATES: &[HtmlNativeFormAssociationKind] = &[
            $(HtmlNativeFormAssociationKind::$kind),+
        ];
    };
}

native_form_kinds! {
    Image => Img,
    Button => Button,
    Fieldset => Fieldset,
    Object => Object,
    Textarea => Textarea,
    Output => Output,
    Input => Input,
    Select => Select,
}

/// Classifies through caller-admitted exact comparisons of borrowed operands.
///
/// Each callback compares the actual expanded name with the supplied canonical
/// HTML vocabulary identity, including namespace. A consumer whose owning
/// operation has already established HTML namespace may compare only its actual
/// local name. Prefix metadata is irrelevant; case folding is not performed.
///
/// The callback owns all operand access, admission and cancellation. It may
/// return false without reading a candidate already inapplicable to its actual
/// creation profile. In particular a parser must not admit Input/Select metadata
/// from this result without its separately validated creation capability.
/// No later comparison occurs after a match or error. No strings are scanned,
/// copied or allocated by this decision itself.
pub fn native_form_association_with<E>(
    mut compare: impl FnMut(KnownHtmlElement) -> Result<bool, E>,
) -> Result<Option<HtmlNativeFormAssociationKind>, E> {
    for &kind in CANDIDATES {
        if compare(kind.element())? {
            return Ok(Some(kind));
        }
    }
    Ok(None)
}
