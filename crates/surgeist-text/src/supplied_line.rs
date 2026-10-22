//! Supplied resolved-level ordering and text-edge decisions for mixed inline lines.

/// A supplied resolved Unicode bidi level, after implicit resolution.
///
/// Resolved levels include 126; the maximum explicit embedding depth of 125
/// is a different phase. This value does not perform Unicode analysis or L1.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BidiLevel(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BidiLevelError {
    OutOfRange { level: u8 },
}

impl BidiLevel {
    pub const fn try_new(level: u8) -> Result<Self, BidiLevelError> {
        if level <= 126 {
            Ok(Self(level))
        } else {
            Err(BidiLevelError::OutOfRange { level })
        }
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn is_rtl(self) -> bool {
        self.0 % 2 == 1
    }
}

/// Supplied whitespace behavior at a selected line edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineWhitespaceEdge {
    Preserve,
    DiscardAtLineStart,
    DiscardAtLineEnd,
    DiscardAtBoth,
}

/// One complete logical ordering slot, including atomic and boundary slots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuppliedInlineParticipant {
    Text {
        level: BidiLevel,
        edge: InlineWhitespaceEdge,
    },
    Atomic {
        level: BidiLevel,
    },
    Boundary {
        level: BidiLevel,
    },
}

impl SuppliedInlineParticipant {
    fn level(self) -> u8 {
        #[cfg(test)]
        ORDER_VISITS.with(|visits| visits.set(visits.get() + 1));
        match self {
            Self::Text { level, .. } | Self::Atomic { level } | Self::Boundary { level } => {
                level.get()
            }
        }
    }

    fn discards_at_start(self) -> bool {
        matches!(
            self,
            Self::Text {
                edge: InlineWhitespaceEdge::DiscardAtLineStart
                    | InlineWhitespaceEdge::DiscardAtBoth,
                ..
            }
        )
    }

    fn discards_at_end(self) -> bool {
        matches!(
            self,
            Self::Text {
                edge: InlineWhitespaceEdge::DiscardAtLineEnd | InlineWhitespaceEdge::DiscardAtBoth,
                ..
            }
        )
    }
}

/// A borrowed complete logical line with levels already supplied by its producer.
///
/// Forced breaks delimit these slices; they are not ordering slots. Discarded
/// text remains in the sequence. Ordinals are local to this input, not source
/// positions or preparation identities. This front chooses no breaks, computes
/// no geometry, and performs neither Unicode analysis nor L1 level resetting.
#[derive(Clone, Copy, Debug)]
pub struct SuppliedLine<'a> {
    participants: &'a [SuppliedInlineParticipant],
}

/// Edge decisions borrowing the exact supplied sequence.
#[derive(Clone, Copy, Debug)]
pub struct SuppliedLineEdges<'a> {
    line: SuppliedLine<'a>,
    leading_end: usize,
    trailing_start: usize,
}

/// Visual order borrowing both its exact logical sequence and caller scratch.
///
/// Neither backing can expire while this result is read:
/// ```compile_fail
/// use surgeist_text::{SuppliedLine, SuppliedInlineParticipant};
/// let order = {
///     let participants: [SuppliedInlineParticipant; 0] = [];
///     let scratch: &mut [usize] = &mut [];
///     SuppliedLine::new(&participants).reorder_into(scratch).unwrap()
/// };
/// let _ = order.iter().count();
/// ```
/// ```compile_fail
/// use surgeist_text::{SuppliedLine, SuppliedInlineParticipant};
/// let participants: [SuppliedInlineParticipant; 0] = [];
/// let order = {
///     let mut scratch = [];
///     SuppliedLine::new(&participants).reorder_into(&mut scratch).unwrap()
/// };
/// let _ = order.iter().count();
/// ```
#[derive(Debug)]
pub struct SuppliedVisualOrder<'a, 'o> {
    line: SuppliedLine<'a>,
    order: &'o [usize],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuppliedLineError {
    OrderBufferLength { expected: usize, actual: usize },
}

impl<'a> SuppliedLine<'a> {
    #[must_use]
    pub const fn new(participants: &'a [SuppliedInlineParticipant]) -> Self {
        Self { participants }
    }

    /// The consecutive logical prefix suppressed at this line's start.
    #[must_use]
    pub fn leading_discard_len(self) -> usize {
        self.participants
            .iter()
            .take_while(|participant| {
                #[cfg(test)]
                observe_edge_visit();
                participant.discards_at_start()
            })
            .count()
    }

    #[must_use]
    pub fn edges(self) -> SuppliedLineEdges<'a> {
        let leading_end = self.leading_discard_len();
        let trailing_len = self
            .participants
            .iter()
            .rev()
            .take_while(|participant| {
                #[cfg(test)]
                observe_edge_visit();
                participant.discards_at_end()
            })
            .count();
        SuppliedLineEdges {
            line: self,
            leading_end,
            trailing_start: self.participants.len() - trailing_len,
        }
    }

    /// Fill caller-owned scratch with L2 order, preserving every logical slot.
    ///
    /// A length error leaves scratch unchanged. This operation allocates no
    /// storage; it initializes scratch rather than trusting an input permutation.
    pub fn reorder_into<'o>(
        self,
        order: &'o mut [usize],
    ) -> Result<SuppliedVisualOrder<'a, 'o>, SuppliedLineError> {
        if order.len() != self.participants.len() {
            return Err(SuppliedLineError::OrderBufferLength {
                expected: self.participants.len(),
                actual: order.len(),
            });
        }
        for (index, slot) in order.iter_mut().enumerate() {
            *slot = index;
        }
        let minimum_odd = self
            .participants
            .iter()
            .map(|participant| participant.level())
            .filter(|level| level % 2 == 1)
            .min();
        if let Some(minimum_odd) = minimum_odd {
            let maximum = self
                .participants
                .iter()
                .map(|participant| participant.level())
                .max()
                .unwrap_or(minimum_odd);
            for level in (minimum_odd..=maximum).rev() {
                let mut start = 0;
                while start < order.len() {
                    while start < order.len() && self.participants[order[start]].level() < level {
                        start += 1;
                    }
                    let mut end = start;
                    while end < order.len() && self.participants[order[end]].level() >= level {
                        end += 1;
                    }
                    order[start..end].reverse();
                    start = end;
                }
            }
        }
        Ok(SuppliedVisualOrder { line: self, order })
    }
}

impl SuppliedLineEdges<'_> {
    #[must_use]
    pub const fn leading_discard_len(self) -> usize {
        self.leading_end
    }
    #[must_use]
    pub const fn trailing_discard_start(self) -> usize {
        self.trailing_start
    }
    #[must_use]
    pub fn is_discarded(self, index: usize) -> Option<bool> {
        (index < self.line.participants.len())
            .then_some(index < self.leading_end || index >= self.trailing_start)
    }
}

impl<'a> SuppliedVisualOrder<'a, '_> {
    /// Visual-order pairs of local logical ordinal and its authentic participant.
    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = (usize, &'a SuppliedInlineParticipant)> + '_ {
        self.order
            .iter()
            .map(|&index| (index, &self.line.participants[index]))
    }
}

#[cfg(test)]
thread_local! {
    static ORDER_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static EDGE_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
fn observe_edge_visit() {
    EDGE_VISITS.with(|visits| visits.set(visits.get() + 1));
}

#[cfg(test)]
mod tests {
    use super::*;
    fn text(level: u8, edge: InlineWhitespaceEdge) -> SuppliedInlineParticipant {
        SuppliedInlineParticipant::Text {
            level: BidiLevel::try_new(level).unwrap(),
            edge,
        }
    }
    fn order(participants: &[SuppliedInlineParticipant]) -> Vec<usize> {
        let mut scratch = vec![usize::MAX; participants.len()];
        let result = SuppliedLine::new(participants)
            .reorder_into(&mut scratch)
            .unwrap();
        for (index, participant) in result.iter() {
            assert!(std::ptr::eq(participant, &participants[index]));
        }
        result.iter().map(|(index, _)| index).collect()
    }
    #[test]
    fn resolved_levels_include_implicit_126_and_reject_larger_values() {
        for level in 0..=126 {
            assert_eq!(BidiLevel::try_new(level).unwrap().get(), level);
        }
        assert!(!BidiLevel::try_new(126).unwrap().is_rtl());
        assert!(BidiLevel::try_new(125).unwrap().is_rtl());
        for level in 127..=255 {
            assert_eq!(
                BidiLevel::try_new(level),
                Err(BidiLevelError::OutOfRange { level })
            );
        }
    }
    #[test]
    fn l2_preserves_all_mixed_slots_and_top_resolved_level() {
        use InlineWhitespaceEdge::Preserve;
        assert_eq!(
            order(&[0, 1, 2, 2, 1, 0].map(|level| text(level, Preserve))),
            [0, 4, 2, 3, 1, 5]
        );
        assert_eq!(
            order(&[
                text(1, InlineWhitespaceEdge::DiscardAtBoth),
                SuppliedInlineParticipant::Boundary {
                    level: BidiLevel::try_new(1).unwrap()
                },
                SuppliedInlineParticipant::Atomic {
                    level: BidiLevel::try_new(2).unwrap()
                },
                text(1, Preserve)
            ]),
            [3, 2, 1, 0]
        );
        assert_eq!(
            order(&[125, 126, 126, 125].map(|level| text(level, Preserve))),
            [3, 1, 2, 0]
        );
        assert_eq!(
            order(&[0, 126, 2].map(|level| text(level, Preserve))),
            [0, 1, 2]
        );
        assert_eq!(order(&[]), []);
    }
    #[test]
    fn scratch_length_errors_leave_both_short_and_long_buffers_unchanged() {
        let participants = [text(1, InlineWhitespaceEdge::Preserve)];
        for mut scratch in [vec![], vec![9, 8]] {
            let before = scratch.clone();
            assert_eq!(
                SuppliedLine::new(&participants)
                    .reorder_into(&mut scratch)
                    .unwrap_err(),
                SuppliedLineError::OrderBufferLength {
                    expected: 1,
                    actual: before.len()
                }
            );
            assert_eq!(scratch, before);
        }
    }
    #[test]
    fn edge_policies_stop_at_preserved_text_atomic_and_boundary_slots() {
        use InlineWhitespaceEdge::*;
        for barrier in [
            text(0, Preserve),
            SuppliedInlineParticipant::Atomic {
                level: BidiLevel::try_new(0).unwrap(),
            },
            SuppliedInlineParticipant::Boundary {
                level: BidiLevel::try_new(1).unwrap(),
            },
        ] {
            let participants = [
                text(0, DiscardAtLineStart),
                barrier,
                text(0, DiscardAtLineEnd),
            ];
            let edges = SuppliedLine::new(&participants).edges();
            assert_eq!(edges.leading_discard_len(), 1);
            assert_eq!(edges.trailing_discard_start(), 2);
            assert_eq!(
                (0..4)
                    .map(|index| edges.is_discarded(index))
                    .collect::<Vec<_>>(),
                [Some(true), Some(false), Some(true), None]
            );
        }
        let participants = [text(0, DiscardAtBoth); 3];
        let edges = SuppliedLine::new(&participants).edges();
        assert_eq!(edges.leading_discard_len(), 3);
        assert_eq!(edges.trailing_discard_start(), 0);
        assert!((0..3).all(|index| edges.is_discarded(index) == Some(true)));
        assert_eq!(SuppliedLine::new(&[]).edges().is_discarded(0), None);
        assert_eq!(
            SuppliedLine::new(&[text(0, DiscardAtLineEnd)]).leading_discard_len(),
            0
        );
        assert_eq!(
            SuppliedLine::new(&[text(0, DiscardAtLineStart)])
                .edges()
                .trailing_discard_start(),
            1
        );
    }
    #[test]
    fn actual_l2_level_reads_are_bounded_for_one_odd_level() {
        for count in [64, 128] {
            let participants = vec![text(1, InlineWhitespaceEdge::Preserve); count];
            ORDER_VISITS.with(|visits| visits.set(0));
            assert_eq!(order(&participants), (0..count).rev().collect::<Vec<_>>());
            assert_eq!(ORDER_VISITS.with(std::cell::Cell::get), count * 3 + 1);
        }
    }
    #[test]
    fn actual_edge_scans_visit_linear_prefix_and_suffix_work() {
        for count in [64, 128] {
            let participants = vec![text(0, InlineWhitespaceEdge::DiscardAtBoth); count];
            EDGE_VISITS.with(|visits| visits.set(0));
            let edges = SuppliedLine::new(&participants).edges();
            assert_eq!(edges.leading_discard_len(), count);
            assert_eq!(edges.trailing_discard_start(), 0);
            assert_eq!(EDGE_VISITS.with(std::cell::Cell::get), count * 2);
        }
    }
}
