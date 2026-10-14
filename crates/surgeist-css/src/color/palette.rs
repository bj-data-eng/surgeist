//! Reached concrete palette facts, observed one actual fixed record per action.

use super::facts::{
    CssColorFactIdentity, CssColorFacts, CssColorPaletteEntry, CssColorPropertyFacts,
};
use super::keywords::canonical_system;
use super::numerical::value::CssUsedColor;
use crate::CssSystemColor;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaletteError {
    ForeignFacts,
    MissingCurrent,
    MissingPalette(CssSystemColor),
    Duplicate(CssSystemColor),
    WrongAction,
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PaletteResult {
    pub(crate) facts: CssColorFactIdentity,
    pub(crate) resource: Option<CssColorFactIdentity>,
    pub(crate) value: CssUsedColor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaletteAction {
    ReadProperties,
    ReadLength,
    ReadEntry { index: usize },
    CompareEarlier { index: usize },
    Finish,
    Done,
    Stopped,
}

enum State {
    Properties,
    Length,
    Entry,
    Duplicate,
    Finish,
    Done,
    Stopped,
}

pub(crate) struct PaletteLookup<'a> {
    facts: &'a CssColorFacts<'a>,
    expected: CssColorFactIdentity,
    wanted: CssSystemColor,
    length: usize,
    next: usize,
    earlier: usize,
    candidate: Option<CssColorPaletteEntry>,
    found: Option<PaletteResult>,
    state: State,
}
impl<'a> PaletteLookup<'a> {
    pub(crate) fn current(facts: &'a CssColorFacts<'a>, expected: CssColorFactIdentity) -> Self {
        Self {
            facts,
            expected,
            wanted: CssSystemColor::CanvasText,
            length: 0,
            next: 0,
            earlier: 0,
            candidate: None,
            found: None,
            state: State::Properties,
        }
    }
    pub(crate) fn system(
        facts: &'a CssColorFacts<'a>,
        expected: CssColorFactIdentity,
        keyword: CssSystemColor,
    ) -> Self {
        Self {
            wanted: canonical_system(keyword),
            state: State::Length,
            ..Self::current(facts, expected)
        }
    }
    pub(crate) fn action(&self) -> PaletteAction {
        match self.state {
            State::Properties => PaletteAction::ReadProperties,
            State::Length => PaletteAction::ReadLength,
            State::Entry => PaletteAction::ReadEntry { index: self.next },
            State::Duplicate => PaletteAction::CompareEarlier {
                index: self.earlier,
            },
            State::Finish => PaletteAction::Finish,
            State::Done => PaletteAction::Done,
            State::Stopped => PaletteAction::Stopped,
        }
    }
    /// The original consumer admits this actual fixed observation/comparison
    /// before calling. No array scan, allocation or provider is hidden here.
    pub(crate) fn step(&mut self, action: PaletteAction) -> Result<(), PaletteError> {
        if self.expected != self.facts.identity {
            return Err(PaletteError::ForeignFacts);
        }
        if action != self.action() {
            return Err(PaletteError::WrongAction);
        }
        match action {
            PaletteAction::ReadProperties => match self.facts.properties {
                CssColorPropertyFacts::Current(value) => {
                    self.found = Some(PaletteResult {
                        facts: self.expected,
                        resource: None,
                        value,
                    });
                    self.state = State::Done;
                }
                CssColorPropertyFacts::InitialProperties => self.state = State::Length,
                CssColorPropertyFacts::Unavailable => {
                    self.state = State::Stopped;
                    return Err(PaletteError::MissingCurrent);
                }
            },
            PaletteAction::ReadLength => {
                self.length = self.facts.palette.len();
                self.state = if self.length == 0 {
                    State::Finish
                } else {
                    State::Entry
                };
            }
            PaletteAction::ReadEntry { index } => {
                let entry = *self
                    .facts
                    .palette
                    .get(index)
                    .ok_or(PaletteError::WrongAction)?;
                self.candidate = Some(entry);
                self.earlier = 0;
                self.state = if index == 0 {
                    State::Finish
                } else {
                    State::Duplicate
                };
            }
            PaletteAction::CompareEarlier { index } => {
                let earlier = self
                    .facts
                    .palette
                    .get(index)
                    .ok_or(PaletteError::WrongAction)?;
                let candidate = self.candidate.ok_or(PaletteError::WrongAction)?;
                let key = canonical_system(candidate.keyword);
                if canonical_system(earlier.keyword) == key {
                    self.state = State::Stopped;
                    return Err(PaletteError::Duplicate(key));
                }
                self.earlier += 1;
                if self.earlier == self.next {
                    self.state = State::Finish;
                }
            }
            PaletteAction::Finish => {
                if let Some(entry) = self.candidate.take() {
                    if canonical_system(entry.keyword) == self.wanted {
                        self.found = Some(PaletteResult {
                            facts: self.expected,
                            resource: Some(entry.resource),
                            value: entry.value,
                        });
                    }
                    self.next += 1;
                }
                if self.next < self.length {
                    self.state = State::Entry;
                } else if self.found.is_some() {
                    self.state = State::Done;
                } else {
                    self.state = State::Stopped;
                    return Err(PaletteError::MissingPalette(self.wanted));
                }
            }
            PaletteAction::Done | PaletteAction::Stopped => return Err(PaletteError::Stopped),
        }
        Ok(())
    }
    pub(crate) fn result(&self) -> Result<PaletteResult, PaletteError> {
        if !matches!(self.state, State::Done) {
            return Err(PaletteError::Stopped);
        }
        self.found.ok_or(PaletteError::Stopped)
    }
    pub(crate) fn abandon(&mut self) {
        self.state = State::Stopped;
    }
}

#[cfg(test)]
mod tests {
    use super::super::facts::{CssColorSchemeFact, CssDeviceColorFact};
    use super::*;
    fn finish(operation: &mut PaletteLookup<'_>) -> Result<PaletteResult, PaletteError> {
        loop {
            if operation.action() == PaletteAction::Done {
                return operation.result();
            }
            operation.step(operation.action())?;
        }
    }
    #[test]
    fn initial_current_reaches_canvas_text_and_never_guesses_black() {
        let identity = CssColorFactIdentity::new(7, 3);
        let resource = CssColorFactIdentity::new(8, 4);
        let expected = CssUsedColor::from_srgba8([11, 22, 33, 255]);
        let palette = [CssColorPaletteEntry::new(
            CssSystemColor::CanvasText,
            resource,
            expected,
        )];
        let facts = CssColorFacts::new(
            identity,
            CssColorPropertyFacts::InitialProperties,
            CssColorSchemeFact::Unavailable,
            &palette,
            &[],
            &[],
            CssDeviceColorFact::Unknown,
        );
        assert_eq!(
            finish(&mut PaletteLookup::current(&facts, identity)).unwrap(),
            PaletteResult {
                facts: identity,
                resource: Some(resource),
                value: expected
            }
        );
        let absent = CssColorFacts::new(
            identity,
            CssColorPropertyFacts::InitialProperties,
            CssColorSchemeFact::Unavailable,
            &[],
            &[],
            &[],
            CssDeviceColorFact::Unknown,
        );
        assert_eq!(
            finish(&mut PaletteLookup::current(&absent, identity)),
            Err(PaletteError::MissingPalette(CssSystemColor::CanvasText))
        );
    }
    #[test]
    fn supplied_property_does_not_read_irrelevant_palette_or_scheme() {
        let id = CssColorFactIdentity::new(1, 1);
        let expected = CssUsedColor::from_srgba8([4, 5, 6, 255]);
        let duplicate = [CssColorPaletteEntry::new(CssSystemColor::Canvas, id, expected); 2];
        let facts = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Current(expected),
            CssColorSchemeFact::Unavailable,
            &duplicate,
            &[],
            &[],
            CssDeviceColorFact::Unknown,
        );
        let mut operation = PaletteLookup::current(&facts, id);
        operation.step(PaletteAction::ReadProperties).unwrap();
        assert_eq!(operation.action(), PaletteAction::Done);
        assert_eq!(operation.result().unwrap().resource, None);
    }
    #[test]
    fn deprecated_lookup_keeps_real_resource_and_duplicate_keys_are_not_silently_selected() {
        let id = CssColorFactIdentity::new(1, 2);
        let expected = CssUsedColor::opaque_black();
        let palette = [CssColorPaletteEntry::new(
            CssSystemColor::ButtonBorder,
            id,
            expected,
        )];
        let facts = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &palette,
            &[],
            &[],
            CssDeviceColorFact::Unknown,
        );
        assert_eq!(
            finish(&mut PaletteLookup::system(
                &facts,
                id,
                CssSystemColor::ActiveBorder
            ))
            .unwrap()
            .value,
            expected
        );
        let duplicated = [
            palette[0],
            CssColorPaletteEntry::new(CssSystemColor::ActiveBorder, id, expected),
        ];
        let invalid = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &duplicated,
            &[],
            &[],
            CssDeviceColorFact::Unknown,
        );
        assert_eq!(
            finish(&mut PaletteLookup::system(
                &invalid,
                id,
                CssSystemColor::ButtonBorder
            )),
            Err(PaletteError::Duplicate(CssSystemColor::ButtonBorder))
        );
        let mut foreign = PaletteLookup::system(
            &facts,
            CssColorFactIdentity::new(1, 3),
            CssSystemColor::ButtonBorder,
        );
        assert_eq!(
            foreign.step(foreign.action()),
            Err(PaletteError::ForeignFacts)
        );
        assert_eq!(foreign.action(), PaletteAction::ReadLength);
        foreign.abandon();
        assert_eq!(foreign.result(), Err(PaletteError::Stopped));
    }
}
