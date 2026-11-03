//! HTML877d find-markers decisions; topology and identities stay with adapters.
/// What a descendant or sibling is, for HTML's "find markers" algorithm
/// (#find-markers). Callers classify only processing instructions whose `name` pseudo-attribute
/// matches the sought marker name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Target {
    /// A processing instruction whose target is `marker`.
    Marker,
    /// A processing instruction whose target is `start`.
    Start,
    /// A processing instruction whose target is `end`.
    End,
    /// Any other node or target, which the algorithm skips.
    Other,
}
/// The result of examining one descendant in tree order; see [`first`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum First {
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
pub fn first(target: Target, name_matches: bool) -> First {
    match (target, name_matches) {
        (Target::Marker, true) => First::Marker,
        (Target::Start, true) => First::Start,
        _ => First::Continue,
    }
}
/// The nesting counter for the sibling scan after a `start` instruction.
#[derive(Clone, Copy, Default)]
pub struct Siblings {
    nesting: usize,
}
impl Siblings {
    /// Examines the next sibling after a `start`. Returns `Ok(true)` for the `end` that closes the
    /// start (nesting zero), otherwise `Ok(false)`: a `start` raises the nesting and a nested `end`
    /// lowers it. Fails, without changing the nesting, if the nesting cannot be represented.
    pub fn inspect(&mut self, target: Target) -> Result<bool, HtmlTemplateMarkerNestingOverflow> {
        match target {
            Target::Start => {
                self.nesting = self
                    .nesting
                    .checked_add(1)
                    .ok_or(HtmlTemplateMarkerNestingOverflow)?;
            }
            Target::End if self.nesting == 0 => return Ok(true),
            Target::End => self.nesting -= 1,
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
