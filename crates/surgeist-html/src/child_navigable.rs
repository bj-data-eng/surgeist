//! Local child-navigable guards over the live owner's actual relationship.

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
