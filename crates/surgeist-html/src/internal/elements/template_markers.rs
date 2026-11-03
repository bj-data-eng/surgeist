//! HTML877d find-markers decisions; topology and identities stay with adapters.
/// What a descendant or sibling is, for HTML's "find markers" algorithm (#find-markers). In the
/// descendant pass callers classify only processing instructions whose `name` pseudo-attribute
/// matches the sought name, and `template_marker_first` further requires that match. In the sibling scan after a
/// `start`, every processing instruction is classified by its target alone, whatever its `name`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTemplateMarkerTarget {
    /// A processing instruction whose target is `marker`.
    Marker,
    /// A processing instruction whose target is `start`.
    Start,
    /// A processing instruction whose target is `end`.
    End,
    /// Any other node or target, which the algorithm skips.
    Other,
}
/// The result of examining one descendant in tree order; see [`template_marker_first`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTemplateMarkerFirst {
    /// This descendant neither is a `marker` nor starts a range: keep scanning.
    Continue,
    /// A matching `marker` instruction: the start and end markers are this one node.
    Marker,
    /// A matching `start` instruction: scan its following siblings for the matching `end`.
    Start,
}
/// Classifies one descendant: a `marker` or `start` whose name matches ends the first-pass
/// scan; everything else continues. `name_matches` is whether the instruction's `name`
/// pseudo-attribute equals the sought name.
pub fn template_marker_first(
    target: HtmlTemplateMarkerTarget,
    name_matches: bool,
) -> HtmlTemplateMarkerFirst {
    match (target, name_matches) {
        (HtmlTemplateMarkerTarget::Marker, true) => HtmlTemplateMarkerFirst::Marker,
        (HtmlTemplateMarkerTarget::Start, true) => HtmlTemplateMarkerFirst::Start,
        _ => HtmlTemplateMarkerFirst::Continue,
    }
}
/// The nesting counter for the sibling scan after a `start` instruction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlTemplateMarkerSiblings {
    nesting: usize,
}
impl HtmlTemplateMarkerSiblings {
    /// Examines the next sibling after a `start`. Returns `Ok(true)` for the `end` that closes the
    /// start (nesting zero), otherwise `Ok(false)`: a `start` raises the nesting and a nested `end`
    /// lowers it. Fails, without changing the nesting, if the nesting cannot be represented.
    pub fn inspect(
        &mut self,
        target: HtmlTemplateMarkerTarget,
    ) -> Result<bool, HtmlTemplateMarkerNestingOverflow> {
        match target {
            HtmlTemplateMarkerTarget::Start => {
                self.nesting = self
                    .nesting
                    .checked_add(1)
                    .ok_or(HtmlTemplateMarkerNestingOverflow)?;
            }
            HtmlTemplateMarkerTarget::End if self.nesting == 0 => return Ok(true),
            HtmlTemplateMarkerTarget::End => self.nesting -= 1,
            _ => {}
        }
        Ok(false)
    }
}

/// Marker nesting cannot be represented; the current nesting is unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlTemplateMarkerNestingOverflow;
impl std::fmt::Display for HtmlTemplateMarkerNestingOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HTML template marker nesting cannot be represented")
    }
}
impl std::error::Error for HtmlTemplateMarkerNestingOverflow {}
