//! HTML877d find-markers decisions; topology and identities stay with adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Target {
    Marker,
    Start,
    End,
    Other,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum First {
    Continue,
    Marker,
    Start,
}
pub fn first(target: Target, name_matches: bool) -> First {
    match (target, name_matches) {
        (Target::Marker, true) => First::Marker,
        (Target::Start, true) => First::Start,
        _ => First::Continue,
    }
}
#[derive(Clone, Copy, Default)]
pub struct Siblings {
    nesting: usize,
}
impl Siblings {
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
