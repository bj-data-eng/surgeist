//! Canonical body bidi facts, indexed once at exact logical event cuts.

use icu_properties::props::BidiClass;

use super::{
    BidiOwner, capacity,
    wrapping::{Event, LogicalEvent},
};
use crate::Result;
use crate::engine::bidi::EligibleStrong;
use crate::protocol::paragraph::ParagraphAnchorItem;

struct BodyContext {
    end: usize,
    strong: Option<u8>,
}

/// Temporary canonical body association, retained only until logical binding.
pub(crate) struct DirectionBody {
    owners: Vec<BidiOwner>,
    eligible: Vec<bool>,
    contexts: Vec<BodyContext>,
}

impl DirectionBody {
    pub(super) fn new(
        classes: impl ExactSizeIterator<Item = BidiClass>,
        owners: Vec<BidiOwner>,
    ) -> Result<Self> {
        let mut eligible = Vec::new();
        eligible
            .try_reserve_exact(classes.len())
            .map_err(|_| capacity())?;
        let mut contexts = Vec::new();
        contexts.try_reserve(1).map_err(|_| capacity())?;
        let mut scanner = EligibleStrong::new();
        let mut strong = None;
        for (index, class) in classes.enumerate() {
            let direction = scanner.advance(class);
            if strong.is_none() {
                strong = direction;
            }
            eligible.push(direction.is_some());
            if class == BidiClass::ParagraphSeparator {
                contexts.try_reserve(1).map_err(|_| capacity())?;
                contexts.push(BodyContext {
                    end: index + 1,
                    strong,
                });
                scanner = EligibleStrong::new();
                strong = None;
            }
        }
        if eligible.len() != owners.len() {
            return Err(invalid());
        }
        contexts.try_reserve(1).map_err(|_| capacity())?;
        // A trailing B has a real empty following context; an empty body also
        // has one real no-strong context rather than unavailable information.
        contexts.push(BodyContext {
            end: owners.len(),
            strong,
        });
        Ok(Self {
            owners,
            eligible,
            contexts,
        })
    }

    pub(super) fn owners(&self) -> &[BidiOwner] {
        &self.owners
    }

    pub(crate) fn bind(self, events: &[LogicalEvent]) -> Result<DirectionIndex> {
        let mut prefix = Vec::new();
        prefix
            .try_reserve_exact(events.len().checked_add(1).ok_or_else(capacity)?)
            .map_err(|_| capacity())?;
        prefix.push(0);
        let mut contexts = Vec::new();
        contexts
            .try_reserve_exact(self.contexts.len())
            .map_err(|_| capacity())?;
        let mut consumed = 0;
        let mut context = 0;
        let mut start = 0;
        let mut count = 0;
        for (index, event) in events.iter().enumerate() {
            let owner = match event.kind {
                Event::Text(atom) => Some(BidiOwner::Atom(atom)),
                Event::Anchor(anchor) => match anchor.item {
                    ParagraphAnchorItem::Atomic(occurrence) => Some(BidiOwner::Atomic(occurrence)),
                    ParagraphAnchorItem::Control(occurrence) => {
                        Some(BidiOwner::Control(occurrence))
                    }
                    ParagraphAnchorItem::EmptyOrigin(_) => None,
                },
                Event::Deleted(..) => None,
            };
            if owner.is_some() && self.owners.get(consumed).copied() == owner {
                count += usize::from(self.eligible[consumed]);
                consumed += 1;
                if context + 1 < self.contexts.len() && consumed == self.contexts[context].end {
                    contexts.push(DirectionContext {
                        start,
                        end: index + 1,
                        strong: self.contexts[context].strong,
                    });
                    context += 1;
                    start = index + 1;
                }
            } else if matches!(event.kind, Event::Text(_)) {
                // Metadata need not contribute a bidi character, but a real
                // semantic atom must match its actual canonical body owner.
                return Err(invalid());
            }
            prefix.push(count);
        }
        if consumed != self.owners.len() || context + 1 != self.contexts.len() {
            return Err(invalid());
        }
        contexts.push(DirectionContext {
            start,
            end: events.len(),
            strong: self.contexts[context].strong,
        });
        Ok(DirectionIndex { contexts, prefix })
    }
}

struct DirectionContext {
    start: usize,
    end: usize,
    strong: Option<u8>,
}

pub(crate) struct DirectionIndex {
    contexts: Vec<DirectionContext>,
    prefix: Vec<usize>,
}

impl DirectionIndex {
    pub(crate) fn query(&self, start: usize, end: usize) -> Option<(Option<u8>, bool)> {
        if start > end || end >= self.prefix.len() {
            return None;
        }
        // Empty seams belong to the right/new context. The terminal empty
        // range remains valid even when its last context itself is empty.
        let index = self
            .contexts
            .partition_point(|context| context.end <= start)
            .min(self.contexts.len() - 1);
        let context = &self.contexts[index];
        if start < context.start || end > context.end {
            return None;
        }
        Some((context.strong, self.prefix[end] != self.prefix[start]))
    }
}

fn invalid() -> crate::Error {
    crate::protocol::protocol_error(crate::ProtocolError::InvalidLineDirectionQuery)
}

#[cfg(test)]
mod tests {
    use super::super::wrapping::LogicalEvent;
    use super::*;
    use crate::protocol::paragraph::{ParagraphAnchor, ParagraphAnchorItem};

    fn index(classes: &[BidiClass]) -> DirectionIndex {
        let owners = (0..classes.len()).map(BidiOwner::Atom).collect();
        let events: Vec<_> = (0..classes.len())
            .map(|atom| LogicalEvent {
                atom,
                source: atom,
                kind: Event::Text(atom),
            })
            .collect();
        DirectionBody::new(classes.iter().copied(), owners)
            .unwrap()
            .bind(&events)
            .unwrap()
    }

    #[test]
    fn eligible_l_r_and_al_are_optional_and_full_context_owned() {
        use BidiClass as B;
        for (class, direction) in [
            (B::LeftToRight, 0),
            (B::RightToLeft, 1),
            (B::ArabicLetter, 1),
        ] {
            let body = index(&[B::EuropeanNumber, class, B::WhiteSpace]);
            assert_eq!(body.query(0, 1), Some((Some(direction), false)));
            assert_eq!(body.query(1, 2), Some((Some(direction), true)));
        }
        assert_eq!(
            index(&[B::EuropeanNumber, B::WhiteSpace]).query(0, 2),
            Some((None, false))
        );
        assert_eq!(
            index(&[B::RightToLeft, B::LeftToRight]).query(1, 2),
            Some((Some(1), true))
        );
    }

    #[test]
    fn suffix_queries_retain_matched_unmatched_and_nested_isolate_prefixes() {
        use BidiClass as B;
        let body = index(&[
            B::RightToLeftIsolate,
            B::RightToLeft,
            B::PopDirectionalIsolate,
            B::LeftToRight,
        ]);
        assert_eq!(body.query(1, 2), Some((Some(0), false)));
        assert_eq!(body.query(3, 4), Some((Some(0), true)));
        assert_eq!(
            index(&[B::PopDirectionalIsolate, B::ArabicLetter]).query(1, 2),
            Some((Some(1), true))
        );
        assert_eq!(
            index(&[B::FirstStrongIsolate, B::ArabicLetter, B::LeftToRight]).query(1, 3),
            Some((None, false))
        );
        let nested = index(&[
            B::FirstStrongIsolate,
            B::ArabicLetter,
            B::LeftToRightIsolate,
            B::LeftToRight,
            B::PopDirectionalIsolate,
            B::PopDirectionalIsolate,
            B::RightToLeft,
        ]);
        assert_eq!(nested.query(1, 5), Some((Some(1), false)));
        assert_eq!(nested.query(6, 7), Some((Some(1), true)));
    }

    #[test]
    fn paragraph_terminators_have_right_affine_empty_and_trailing_contexts() {
        use BidiClass as B;
        let body = index(&[
            B::ParagraphSeparator,
            B::ParagraphSeparator,
            B::LeftToRight,
            B::ParagraphSeparator,
        ]);
        assert_eq!(body.query(0, 1), Some((None, false)));
        assert_eq!(body.query(1, 2), Some((None, false)));
        assert_eq!(body.query(2, 4), Some((Some(0), true)));
        assert_eq!(body.query(2, 2), Some((Some(0), false)));
        assert_eq!(body.query(4, 4), Some((None, false)));
        assert_eq!(body.query(0, 3), None);
        assert_eq!(body.query(3, 2), None);
        assert_eq!(index(&[]).query(0, 0), Some((None, false)));
        let isolated = index(&[
            B::RightToLeftIsolate,
            B::RightToLeft,
            B::ParagraphSeparator,
            B::LeftToRight,
        ]);
        assert_eq!(isolated.query(0, 3), Some((None, false)));
        assert_eq!(isolated.query(3, 4), Some((Some(0), true)));
    }

    #[test]
    fn real_owner_binding_preserves_control_empty_deleted_and_source_zero_custody() {
        use BidiClass as B;
        let anchor = |item| {
            Event::Anchor(ParagraphAnchor {
                item,
                affinity: crate::Affinity::After,
            })
        };
        let events = [
            anchor(ParagraphAnchorItem::EmptyOrigin(0)),
            Event::Text(0),
            Event::Deleted(crate::Range::new(0, 1), 0),
            anchor(ParagraphAnchorItem::Control(1)),
            anchor(ParagraphAnchorItem::EmptyOrigin(1)),
            Event::Text(1),
            anchor(ParagraphAnchorItem::Atomic(7)),
        ]
        .into_iter()
        .map(|kind| LogicalEvent {
            atom: 0,
            source: 0,
            kind,
        })
        .collect::<Vec<_>>();
        let body = DirectionBody::new(
            [B::LeftToRight, B::ParagraphSeparator, B::RightToLeft].into_iter(),
            vec![
                BidiOwner::Atom(0),
                BidiOwner::Control(1),
                BidiOwner::Atom(1),
            ],
        )
        .unwrap()
        .bind(&events)
        .unwrap();
        assert_eq!(body.query(0, 4), Some((Some(0), true)));
        assert_eq!(body.query(2, 4), Some((Some(0), false)));
        assert_eq!(body.query(4, 5), Some((Some(1), false)));
        assert_eq!(body.query(4, 6), Some((Some(1), true)));
        assert_eq!(body.query(3, 6), None);
        assert_eq!(body.query(7, 7), Some((Some(1), false)));
    }
}
