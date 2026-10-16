//! Local child-navigable guards over the live owner's actual relationship.

use crate::DocumentString;

/// Source candidate selection in shared Frame/Iframe attribute processing.
#[derive(Clone, Copy, Debug)]
pub enum HtmlFrameIframeSrcDecision<'a> {
    AboutBlank,
    /// Encoding-parse these exact nonempty units against the current Document.
    /// Actual parse failure keeps the initial about:blank candidate; unavailable
    /// evidence is not failure. This does not create hyperlink-cache history.
    ParseSrc(&'a DocumentString),
}

/// Shared attribute processing starts with about:blank and parses only a
/// present, nonempty src. Admit the optional input/length header once before
/// inspecting it. No code units are scanned, copied, normalized or retained.
///
/// This is a later algorithm than Permissions Policy's declared-origin rule:
/// that distinct rule can require parsing a present-empty src.
pub fn frame_iframe_src_with<E>(
    raw: Option<&DocumentString>,
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<HtmlFrameIframeSrcDecision<'_>, E> {
    step()?;
    Ok(match raw {
        Some(raw) if !raw.is_empty() => HtmlFrameIframeSrcDecision::ParseSrc(raw),
        _ => HtmlFrameIframeSrcDecision::AboutBlank,
    })
}

/// The next local step after the actual inclusive-ancestor URL comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFrameIframeAfterAncestors {
    ReturnNull,
    /// Execute the real child's URL/history update, then return its candidate
    /// URL to the remaining family steps. This value acknowledges no update.
    UpdateUrlAndHistory,
    ReturnUrl,
}

/// HTML877d shared Frame/Iframe attribute processing, source steps 3–5.
/// The caller supplies actual URL-value equality excluding only fragments;
/// record identity or matches-about:blank alone is insufficient. A match
/// returns null before even reading the later about:blank predicate.
/// Otherwise read that qualified predicate once, then select the actual
/// initial-insertion URL/history update or return the candidate unchanged.
/// No ancestor traversal, URL comparison or history mutation occurs here.
pub fn frame_iframe_after_ancestors_with<E>(
    matches_ancestor: bool,
    initial_insertion: bool,
    read_matches_blank: impl FnOnce() -> Result<bool, E>,
) -> Result<HtmlFrameIframeAfterAncestors, E> {
    if matches_ancestor {
        return Ok(HtmlFrameIframeAfterAncestors::ReturnNull);
    }
    let matches_blank = read_matches_blank()?;
    Ok(if matches_blank && initial_insertion {
        HtmlFrameIframeAfterAncestors::UpdateUrlAndHistory
    } else {
        HtmlFrameIframeAfterAncestors::ReturnUrl
    })
}

/// Actual current presence supplied by the canonical relationship owner.
/// This value carries no identity, origin, resource, history or creation proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlChildNavigablePresence {
    Null,
    Present,
}

/// The first content-Document/Window guard, before their distinct access rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlChildNavigableAccessDecision {
    ReturnNull,
    /// Read the actual child and execute the applicable origin/Window rules.
    Continue,
}

/// The content guard for an element that potentially delays the load event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlChildNavigableLoadDelayDecision {
    DoesNotDelay,
    /// Consult actual child load state; presence does not prove a load delay.
    Continue,
}

impl HtmlChildNavigablePresence {
    pub const fn content_access(self) -> HtmlChildNavigableAccessDecision {
        match self {
            Self::Null => HtmlChildNavigableAccessDecision::ReturnNull,
            Self::Present => HtmlChildNavigableAccessDecision::Continue,
        }
    }

    /// Applicable only after the element's potential-delay classification.
    pub const fn load_delay(self) -> HtmlChildNavigableLoadDelayDecision {
        match self {
            Self::Null => HtmlChildNavigableLoadDelayDecision::DoesNotDelay,
            Self::Present => HtmlChildNavigableLoadDelayDecision::Continue,
        }
    }

    /// First guard for Frame src and Iframe src/srcdoc/sandbox mutations.
    /// True requires the caller's remaining family-specific steps, including
    /// srcdoc precedence; it does not parse attributes or update live policy.
    pub const fn attribute_change_requires_processing(self) -> bool {
        matches!(self, Self::Present)
    }

    /// Destroy-child returns on null before clearing or destroying anything.
    /// True requires the real destruction owner; it does not clear the relation.
    pub const fn removal_requires_child_destruction(self) -> bool {
        matches!(self, Self::Present)
    }
}
