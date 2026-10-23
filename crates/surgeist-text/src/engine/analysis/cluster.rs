// Copyright 2025 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use icu_normalizer::properties::Decomposed;
use std::vec::Vec;

use super::AnalysisDataSources;

// Coverage scratch grows with the actual cluster, not a fixed prefix.

#[derive(Clone, Copy, Debug)]
struct Coverage {
    mapped: usize,
    total: usize,
}
impl Default for Coverage {
    fn default() -> Self {
        Self {
            mapped: 0,
            total: 1,
        }
    }
}
impl Coverage {
    fn complete(self) -> bool {
        self.mapped >= self.total
    }
}
impl PartialEq for Coverage {
    fn eq(&self, other: &Self) -> bool {
        (self.mapped as u128) * (other.total as u128)
            == (other.mapped as u128) * (self.total as u128)
    }
}
impl PartialOrd for Coverage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            ((self.mapped as u128) * (other.total as u128))
                .cmp(&((other.mapped as u128) * (self.total as u128))),
        )
    }
}

#[derive(Debug, Default)]
pub(crate) struct CharCluster {
    pub chars: Vec<Char>,
    pub is_emoji: bool,
    pub map_len: usize,
    pub force_normalize: bool,
    comp: Form,
    decomp: Form,
    best_ratio: Coverage,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Char {
    /// The character.
    pub ch: char,
    /// Whether the character
    pub is_control_character: bool,
    /// True if the character should be considered when mapping glyphs.
    pub contributes_to_shaping: bool,
}

/// Iterative status of testing actual nominal coverage of a character cluster.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Status {
    /// Mapping should be skipped.
    Discard,
    /// The best mapping so far.
    Keep,
    /// Complete mapping.
    Complete,
}

impl CharCluster {
    pub(crate) fn clear(&mut self) {
        self.chars.clear();
        self.is_emoji = false;
        self.map_len = 0;
        self.force_normalize = false;
        self.comp.clear();
        self.decomp.clear();
        self.best_ratio = Coverage::default();
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.chars.len()
    }

    #[inline(always)]
    fn contributes_to_shaping(ch: char, analysis_data_sources: &AnalysisDataSources) -> bool {
        let props = analysis_data_sources.properties(ch);
        super::contributes_to_shaping(props.general_category(), props.script())
    }

    fn decomposed(&mut self, analysis_data_sources: &AnalysisDataSources) -> Option<&[Char]> {
        match self.decomp.state {
            FormState::Invalid => None,
            FormState::None => {
                self.decomp.state = FormState::Invalid;

                // Only attempt pairwise normalization (1 <-> 2 characters)
                if self.chars.len() != 1 {
                    return None;
                }

                let decomposer = analysis_data_sources.decomposing_normalizer();
                let decomp = decomposer.decompose(self.chars[0].ch);
                match decomp {
                    Decomposed::Default | Decomposed::Singleton(_) => {
                        return None;
                    }
                    Decomposed::Expansion(a, b) => {
                        let mut copy = self.chars[0];
                        copy.ch = a;
                        copy.contributes_to_shaping =
                            Self::contributes_to_shaping(a, analysis_data_sources);
                        self.decomp.chars[0] = copy;

                        copy.ch = b;
                        copy.contributes_to_shaping =
                            Self::contributes_to_shaping(b, analysis_data_sources);
                        self.decomp.chars[1] = copy;

                        self.decomp.len = 2;
                    }
                }

                self.decomp.state = FormState::Valid;
                self.decomp.setup();
                Some(self.decomp.chars())
            }
            FormState::Valid => Some(self.decomp.chars()),
        }
    }

    fn composed(&mut self, analysis_data_sources: &AnalysisDataSources) -> Option<&[Char]> {
        match self.comp.state {
            FormState::Invalid => None,
            FormState::None => {
                self.comp.state = FormState::Invalid;

                // Only attempt pairwise normalization (1 <-> 2 characters)
                if self.chars.len() != 2 {
                    return None;
                }

                let composer = analysis_data_sources.composing_normalizer();
                let comp = composer.compose(self.chars[0].ch, self.chars[1].ch);
                match comp {
                    None => {}
                    Some(ch) => {
                        let mut copy = self.chars[0];
                        copy.ch = ch;
                        copy.contributes_to_shaping =
                            Self::contributes_to_shaping(ch, analysis_data_sources);
                        self.comp.chars[0] = copy;
                        self.comp.len = 1;
                    }
                }

                self.comp.state = FormState::Valid;
                self.comp.setup();
                Some(self.comp.chars())
            }
            FormState::Valid => Some(self.comp.chars()),
        }
    }

    pub(crate) fn map(
        &mut self,
        f: impl Fn(char) -> bool,
        analysis_data_sources: &AnalysisDataSources,
    ) -> Status {
        let len = self.len();
        if len == 0 {
            return Status::Complete;
        }
        let prev_ratio = self.best_ratio;
        let mut ratio;
        if self.force_normalize && self.composed(analysis_data_sources).is_some() {
            ratio = self.comp.map(&f);
            if ratio > self.best_ratio {
                self.best_ratio = ratio;
                if ratio.complete() {
                    return Status::Complete;
                }
            }
        }
        ratio = Mapper {
            chars: &self.chars[..len],
            map_len: self.map_len.max(1),
        }
        .map(&f);
        if ratio > self.best_ratio {
            self.best_ratio = ratio;
            if ratio.complete() {
                return Status::Complete;
            }
        }
        if self.decomposed(analysis_data_sources).is_some() {
            ratio = self.decomp.map(&f);
            if ratio > self.best_ratio {
                self.best_ratio = ratio;
                if ratio.complete() {
                    return Status::Complete;
                }
            }
            if !self.force_normalize && self.composed(analysis_data_sources).is_some() {
                ratio = self.comp.map(&f);
                if ratio > self.best_ratio {
                    self.best_ratio = ratio;
                    if ratio.complete() {
                        return Status::Complete;
                    }
                }
            }
        }
        if self.best_ratio > prev_ratio {
            Status::Keep
        } else {
            Status::Discard
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum FormState {
    None,
    Valid,
    Invalid,
}

#[derive(Clone, Debug)]
pub(crate) struct Form {
    chars: [Char; 2],
    len: usize,
    map_len: usize,
    state: FormState,
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl Form {
    fn new() -> Self {
        Self {
            chars: [Char::default(), Char::default()],
            len: 0,
            map_len: 0,
            state: FormState::None,
        }
    }

    fn clear(&mut self) {
        self.chars = [Char::default(), Char::default()];
        self.len = 0;
        self.map_len = 0;
        self.state = FormState::None;
    }

    #[inline(always)]
    fn chars(&self) -> &[Char] {
        &self.chars[..self.len]
    }

    #[inline(always)]
    fn setup(&mut self) {
        self.map_len = (self
            .chars()
            .iter()
            .filter(|c| !c.is_control_character)
            .count())
        .max(1);
    }

    #[inline(always)]
    fn map(&self, f: &impl Fn(char) -> bool) -> Coverage {
        Mapper {
            chars: &self.chars[..self.len],
            map_len: self.map_len,
        }
        .map(f)
    }
}

struct Mapper<'a> {
    chars: &'a [Char],
    map_len: usize,
}

impl Mapper<'_> {
    fn map(&self, f: &impl Fn(char) -> bool) -> Coverage {
        if self.map_len == 0 {
            return Coverage {
                mapped: 1,
                total: 1,
            };
        }
        let mut mapped = 0;
        for character in self.chars {
            let covered = f(character.ch);
            if character.contributes_to_shaping {
                mapped += usize::from(covered);
            } else if self.map_len == 1 {
                mapped += 1;
            }
        }
        Coverage {
            mapped,
            total: self.map_len,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nominal_coverage_visits_every_character_beyond_thirty_two() {
        let mut cluster = CharCluster {
            map_len: 65,
            chars: vec![
                Char {
                    ch: 'A',
                    contributes_to_shaping: true,
                    ..Char::default()
                };
                65
            ],
            ..CharCluster::default()
        };
        assert_eq!(
            cluster.map(|_| true, &AnalysisDataSources),
            Status::Complete
        );
    }
}
