//! HTML877d find-markers decisions; topology and identities stay with adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Target {
    Marker,
    Start,
    End,
    Other,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum First {
    Continue,
    Marker,
    Start,
}
pub(crate) fn first(target: Target, name_matches: bool) -> First {
    match (target, name_matches) {
        (Target::Marker, true) => First::Marker,
        (Target::Start, true) => First::Start,
        _ => First::Continue,
    }
}
#[derive(Clone, Copy, Default)]
pub(crate) struct Siblings {
    nesting: usize,
}
impl Siblings {
    pub(crate) fn inspect(&mut self, target: Target) -> Result<bool, ()> {
        match target {
            Target::Start => self.nesting = self.nesting.checked_add(1).ok_or(())?,
            Target::End if self.nesting == 0 => return Ok(true),
            Target::End => self.nesting -= 1,
            _ => {}
        }
        Ok(false)
    }
}
