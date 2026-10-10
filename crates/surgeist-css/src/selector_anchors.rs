//! Shared semantic ampersand containment, including retained invalid members.
use crate::*;

fn any<T, E>(
    values: impl IntoIterator<Item = T>,
    mut test: impl FnMut(T) -> Result<bool, E>,
) -> Result<bool, E> {
    for value in values {
        if test(value)? {
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Clone, Copy)]
pub(crate) enum AnchorKind {
    Nesting,
    Scope,
    Either,
    TypedNesting,
    TypedScope,
}

pub(crate) fn selector_has_anchor<E>(
    selector: &CssSelector,
    kind: AnchorKind,
    visit: &mut impl FnMut(u32) -> Result<(), E>,
) -> Result<bool, E> {
    selector_anchor_at_depth(selector, kind, visit, 0)
}

fn selector_anchor_at_depth<E>(
    selector: &CssSelector,
    kind: AnchorKind,
    visit: &mut impl FnMut(u32) -> Result<(), E>,
    depth: u32,
) -> Result<bool, E> {
    visit(depth)?;
    match selector {
        CssSelector::Compound(compound) => compound_has_anchor(compound, kind, visit, depth),
        CssSelector::Complex(complex) => {
            if compound_has_anchor(complex.first(), kind, visit, depth)? {
                Ok(true)
            } else {
                any(complex.rest().iter(), |part| {
                    compound_has_anchor(part.selector(), kind, visit, depth)
                })
            }
        }
        CssSelector::PseudoClass(pseudo) => pseudo_has_anchor(pseudo, kind, visit, depth),
        CssSelector::Tag(_) | CssSelector::Key(_) | CssSelector::Class(_) => Ok(false),
    }
}

fn compound_has_anchor<E>(
    compound: &CssCompoundSelector,
    kind: AnchorKind,
    visit: &mut impl FnMut(u32) -> Result<(), E>,
    depth: u32,
) -> Result<bool, E> {
    visit(depth)?;
    let direct = match kind {
        AnchorKind::Nesting | AnchorKind::TypedNesting => compound.nesting_selectors() > 0,
        AnchorKind::Scope | AnchorKind::TypedScope => compound.has_scope_anchor(),
        AnchorKind::Either => compound.nesting_selectors() > 0 || compound.has_scope_anchor(),
    };
    if direct
        || any(compound.pseudo_classes().iter(), |pseudo| {
            pseudo_has_anchor(pseudo, kind, visit, depth)
        })?
    {
        return Ok(true);
    }
    if let Some(sequence) = compound.pseudo_elements() {
        any(sequence.segments().iter(), |segment| match segment {
            CssPseudoElementSegment::PseudoClass(pseudo) => {
                pseudo_has_anchor(pseudo, kind, visit, depth)
            }
            CssPseudoElementSegment::PseudoElement(CssPseudoElement::Slotted(argument)) => {
                compound_has_anchor(argument.compound(), kind, visit, depth + 1)
            }
            CssPseudoElementSegment::PseudoElement(_) => Ok(false),
        })
    } else {
        Ok(false)
    }
}

fn pseudo_has_anchor<E>(
    pseudo: &CssPseudoClass,
    kind: AnchorKind,
    visit: &mut impl FnMut(u32) -> Result<(), E>,
    depth: u32,
) -> Result<bool, E> {
    visit(depth)?;
    match pseudo {
        CssPseudoClass::HostFunction(argument) | CssPseudoClass::HostContext(argument) => {
            compound_has_anchor(argument.compound(), kind, visit, depth + 1)
        }
        CssPseudoClass::Not(list) | CssPseudoClass::Is(list) | CssPseudoClass::Where(list) => {
            any(list.items().iter(), |item| match item {
                CssPseudoSelectorListItem::Selector(selector) => {
                    selector_anchor_at_depth(selector, kind, visit, depth + 1)
                }
                CssPseudoSelectorListItem::InvalidNesting(_) => {
                    Ok(matches!(kind, AnchorKind::Nesting | AnchorKind::Either))
                }
            })
        }
        CssPseudoClass::Has(list) => any(list.selectors().iter(), |relative| {
            selector_anchor_at_depth(relative.selector(), kind, visit, depth + 1)
        }),
        CssPseudoClass::NthChild(pattern) | CssPseudoClass::NthLastChild(pattern) => {
            if let Some(list) = pattern.selector_list() {
                any(list.items().iter(), |item| match item {
                    CssPseudoSelectorListItem::Selector(selector) => {
                        selector_anchor_at_depth(selector, kind, visit, depth + 1)
                    }
                    CssPseudoSelectorListItem::InvalidNesting(_) => {
                        Ok(matches!(kind, AnchorKind::Nesting | AnchorKind::Either))
                    }
                })
            } else {
                Ok(false)
            }
        }
        CssPseudoClass::Host
        | CssPseudoClass::Root
        | CssPseudoClass::Scope
        | CssPseudoClass::Defined
        | CssPseudoClass::AnyLink
        | CssPseudoClass::Link
        | CssPseudoClass::Visited
        | CssPseudoClass::Target
        | CssPseudoClass::Current
        | CssPseudoClass::Dir(_)
        | CssPseudoClass::Lang(_)
        | CssPseudoClass::Hover
        | CssPseudoClass::Active
        | CssPseudoClass::Focus
        | CssPseudoClass::FocusVisible
        | CssPseudoClass::FocusWithin
        | CssPseudoClass::Disabled
        | CssPseudoClass::Enabled
        | CssPseudoClass::Checked
        | CssPseudoClass::Unchecked
        | CssPseudoClass::Required
        | CssPseudoClass::Optional
        | CssPseudoClass::Valid
        | CssPseudoClass::Invalid
        | CssPseudoClass::UserValid
        | CssPseudoClass::UserInvalid
        | CssPseudoClass::PlaceholderShown
        | CssPseudoClass::Autofill
        | CssPseudoClass::FirstChild
        | CssPseudoClass::LastChild
        | CssPseudoClass::OnlyChild
        | CssPseudoClass::Empty
        | CssPseudoClass::FirstOfType
        | CssPseudoClass::LastOfType
        | CssPseudoClass::OnlyOfType
        | CssPseudoClass::NthOfType(_)
        | CssPseudoClass::NthLastOfType(_)
        | CssPseudoClass::Playing
        | CssPseudoClass::Paused
        | CssPseudoClass::Seeking
        | CssPseudoClass::Buffering
        | CssPseudoClass::Stalled
        | CssPseudoClass::Muted
        | CssPseudoClass::VolumeLocked
        | CssPseudoClass::Open
        | CssPseudoClass::Modal
        | CssPseudoClass::Fullscreen
        | CssPseudoClass::PictureInPicture
        | CssPseudoClass::PopoverOpen
        | CssPseudoClass::Default
        | CssPseudoClass::Indeterminate
        | CssPseudoClass::ReadOnly
        | CssPseudoClass::ReadWrite
        | CssPseudoClass::InRange
        | CssPseudoClass::OutOfRange => Ok(false),
    }
}
