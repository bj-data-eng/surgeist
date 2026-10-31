//! Canonical body bidi facts, indexed once at exact logical event cuts.

use icu_properties::props::BidiClass;

use super::{
    BidiOwner, capacity,
    wrapping::{Event, LogicalEvent},
};
use crate::Result;
use crate::engine::bidi::EligibleStrong;
use crate::protocol::paragraph::SemanticAnchorItem;

struct BodyContext {
    start: usize,
    end: usize,
    strong: Option<u8>,
    section: usize,
}

/// Canonical body analysis, with genuine structural section ownership.
pub(crate) struct DirectionBody {
    owners: Vec<BidiOwner>,
    eligible: Vec<bool>,
    contexts: Vec<BodyContext>,
    sections: Vec<std::ops::Range<usize>>,
}
impl DirectionBody {
    pub(super) fn new(
        classes: impl ExactSizeIterator<Item = BidiClass>,
        owners: Vec<BidiOwner>,
        sections: &[std::ops::Range<usize>],
    ) -> Result<Self> {
        if classes.len() != owners.len() || sections.is_empty() {
            return Err(invalid());
        }
        let mut eligible = Vec::new();
        eligible
            .try_reserve_exact(owners.len())
            .map_err(|_| capacity())?;
        let mut contexts = Vec::new();
        let mut classes = classes;
        let mut previous = 0usize;
        for (section, range) in sections.iter().enumerate() {
            if range.start != previous || range.end < range.start || range.end > owners.len() {
                return Err(invalid());
            }
            let mut scanner = EligibleStrong::new();
            let mut strong = None;
            let mut start = range.start;
            for index in range.clone() {
                let class = classes.next().ok_or_else(invalid)?;
                let direction = scanner.advance(class);
                if strong.is_none() {
                    strong = direction;
                }
                eligible.push(direction.is_some());
                if class == BidiClass::ParagraphSeparator {
                    contexts.try_reserve(1).map_err(|_| capacity())?;
                    contexts.push(BodyContext {
                        start,
                        end: index + 1,
                        strong,
                        section,
                    });
                    scanner = EligibleStrong::new();
                    strong = None;
                    start = index + 1;
                }
            }
            contexts.try_reserve(1).map_err(|_| capacity())?;
            contexts.push(BodyContext {
                start,
                end: range.end,
                strong,
                section,
            });
            previous = range.end;
        }
        if previous != owners.len() || classes.next().is_some() {
            return Err(invalid());
        }
        let sections = super::storage::collect(sections.iter().cloned())?;
        Ok(Self {
            owners,
            eligible,
            contexts,
            sections,
        })
    }
    pub(super) fn owners(&self) -> &[BidiOwner] {
        &self.owners
    }

    pub(crate) fn bind(
        self,
        events: &[LogicalEvent],
        sections: &[std::ops::Range<usize>],
        combination: Option<&crate::transform::combined::CombinedOutput>,
    ) -> Result<DirectionIndex> {
        if sections.len() != self.sections.len() {
            return Err(invalid());
        }
        let mut prefix = Vec::new();
        prefix
            .try_reserve_exact(events.len().checked_add(1).ok_or_else(capacity)?)
            .map_err(|_| capacity())?;
        prefix.push(0);
        let mut owner_ends = super::storage::filled(
            self.owners.len().checked_add(1).ok_or_else(capacity)?,
            0usize,
        )?;
        let mut consumed = 0;
        let mut count = 0;
        let mut index = 0usize;
        let mut zero_cells: Vec<ZeroRawCell> = Vec::new();
        while index <= events.len() {
            if let Some(&BidiOwner::CombinedCell(cell)) = self.owners.get(consumed) {
                let binding = combination
                    .and_then(|output| output.bindings.get(cell))
                    .ok_or_else(invalid)?;
                let range = &binding.participants;
                if range.start < index || range.end < range.start || range.end > events.len() {
                    return Err(invalid());
                }
                if range.start == index {
                    // A genuine source cell owns one bidi contribution, even
                    // when all of its payload was suppressed. Its owner/body
                    // summary survives an empty logical participant interval.
                    if range.is_empty() {
                        let section = self.sections.partition_point(|range| range.end <= consumed);
                        if !self
                            .sections
                            .get(section)
                            .is_some_and(|range| range.contains(&consumed))
                            || binding.events.is_empty()
                            || zero_cells.last().is_some_and(|previous| {
                                previous.section > section
                                    || (previous.section == section
                                        && (previous.events.end > binding.events.start
                                            || previous.cut > range.start))
                            })
                        {
                            return Err(invalid());
                        }
                        super::storage::push(
                            &mut zero_cells,
                            ZeroRawCell {
                                section,
                                cut: range.start,
                                events: binding.events.clone(),
                                eligible: self.eligible[consumed],
                            },
                        )?;
                    } else {
                        count += usize::from(self.eligible[consumed]);
                    }
                    consumed += 1;
                    owner_ends[consumed] = range.end;
                    while index < range.end {
                        index += 1;
                        prefix.push(count);
                    }
                    continue;
                }
            }
            let Some(event) = events.get(index) else {
                break;
            };
            let owner = match event.kind {
                Event::Text(atom) => Some(BidiOwner::Atom(atom)),
                Event::Anchor(anchor) => match anchor.item {
                    SemanticAnchorItem::Atomic(i) => Some(BidiOwner::Atomic(i)),
                    SemanticAnchorItem::Control(i) => Some(BidiOwner::Control(i)),
                    SemanticAnchorItem::ComputedLineBoundary(i) => {
                        Some(BidiOwner::ComputedLineBoundary(i))
                    }
                    SemanticAnchorItem::EmptyOrigin(_) => None,
                },
                Event::Deleted(..) => None,
            };
            if owner.is_some() && self.owners.get(consumed).copied() == owner {
                count += usize::from(self.eligible[consumed]);
                consumed += 1;
                owner_ends[consumed] = index + 1;
            } else if matches!(event.kind, Event::Text(_)) {
                return Err(invalid());
            }
            prefix.push(count);
            index += 1;
        }
        if consumed != self.owners.len() {
            return Err(invalid());
        }
        let mut contexts = Vec::new();
        contexts
            .try_reserve_exact(self.contexts.len())
            .map_err(|_| capacity())?;
        let mut by_section = super::storage::filled(sections.len(), 0..0)?;
        for body in self.contexts {
            let owner_range = &self.sections[body.section];
            let event_range = &sections[body.section];
            let start = if body.start == owner_range.start {
                event_range.start
            } else {
                owner_ends[body.start]
            };
            let end = if body.end == owner_range.end {
                event_range.end
            } else {
                owner_ends[body.end]
            };
            if start > end || end > events.len() {
                return Err(invalid());
            }
            if by_section[body.section].is_empty() {
                by_section[body.section].start = contexts.len();
            }
            contexts.push(DirectionContext {
                start,
                end,
                strong: body.strong,
            });
            by_section[body.section].end = contexts.len();
        }
        let sections = super::storage::collect(sections.iter().cloned())?;
        let zero_prefix = if zero_cells.is_empty() {
            Vec::new()
        } else {
            let mut prefix = super::storage::filled(
                zero_cells.len().checked_add(1).ok_or_else(capacity)?,
                0usize,
            )?;
            for (index, cell) in zero_cells.iter().enumerate() {
                prefix[index + 1] = prefix[index] + usize::from(cell.eligible);
            }
            prefix
        };
        Ok(DirectionIndex {
            contexts,
            prefix,
            sections,
            by_section,
            zero_cells,
            zero_prefix,
        })
    }
}

struct DirectionContext {
    start: usize,
    end: usize,
    strong: Option<u8>,
}

/// A real opaque cell whose relation events have no raw logical participant.
/// Eligibility was resolved in the complete canonical body/isolate context.
struct ZeroRawCell {
    section: usize,
    cut: usize,
    events: std::ops::Range<usize>,
    eligible: bool,
}

pub(crate) struct DirectionIndex {
    contexts: Vec<DirectionContext>,
    prefix: Vec<usize>,
    sections: Vec<std::ops::Range<usize>>,
    by_section: Vec<std::ops::Range<usize>>,
    zero_cells: Vec<ZeroRawCell>,
    zero_prefix: Vec<usize>,
}
impl DirectionIndex {
    /// Source incidence supplements raw cuts only for an enclosed actual
    /// zero-raw cell. It does not turn an empty point into a strong range.
    pub(crate) fn query_source_in(
        &self,
        section: usize,
        participants: std::ops::Range<usize>,
        relation: std::ops::Range<usize>,
    ) -> Option<(Option<u8>, bool)> {
        let (strong, raw_strong) = self.query_in(section, participants.start, participants.end)?;
        if self.zero_cells.is_empty() {
            return Some((strong, raw_strong));
        }
        if relation.start > relation.end {
            return None;
        }
        let section_start = self
            .zero_cells
            .partition_point(|cell| cell.section < section);
        let section_end = self
            .zero_cells
            .partition_point(|cell| cell.section <= section);
        let cells = &self.zero_cells[section_start..section_end];
        // Each partition is monotone by the checked canonical owner/event
        // order. Both event enclosure and raw incidence are required.
        let first = cells
            .partition_point(|cell| cell.events.start < relation.start)
            .max(cells.partition_point(|cell| cell.cut < participants.start));
        let last = cells
            .partition_point(|cell| cell.events.end <= relation.end)
            .min(cells.partition_point(|cell| cell.cut <= participants.end));
        let cell_strong = first < last
            && self.zero_prefix[section_start + last] != self.zero_prefix[section_start + first];
        Some((strong, raw_strong || cell_strong))
    }

    pub(crate) fn query_in(
        &self,
        section: usize,
        start: usize,
        end: usize,
    ) -> Option<(Option<u8>, bool)> {
        let bounds = self.sections.get(section)?;
        if start > end || start < bounds.start || end > bounds.end || end >= self.prefix.len() {
            return None;
        }
        let contexts = &self.contexts[self.by_section.get(section)?.clone()];
        let index = contexts
            .partition_point(|context| context.end <= start)
            .min(contexts.len().checked_sub(1)?);
        let context = &contexts[index];
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

    /// Actual child InlineAll SPACE followed by an ordinary EN. The cell's
    /// source SPACE is removed only by its own inner edges. Optional genuine
    /// captured isolate controls surround that cell, outside its owned parts.
    fn source_zero_cell(isolate: Option<bool>) -> crate::AnalyzedSource {
        use crate::*;
        let source = Source::identified(" 1", Id::from_u64(78491), 3);
        let style = Style {
            white_space: WhiteSpace::Collapse,
            text_transform: TextTransform::none(),
            ..Style::default()
        };
        let origins = [Range::new(0, 1), Range::new(1, 2)]
            .map(|range| ParagraphOrigin {
                source: source.clone(),
                local: range,
                aggregate: range,
                kind: ParagraphOriginKind::Authored,
                text_node: None,
            })
            .to_vec();
        let mut controls = Vec::new();
        let mut events = vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
        ];
        if isolate.is_some() {
            controls.push(ParagraphControl {
                at: 0,
                kind: ParagraphControlKind::RightToLeftIsolate,
            });
            events.push(SourceProcessingEvent::Control {
                control: 0,
                affinity: Affinity::After,
            });
        }
        events.extend([
            SourceProcessingEvent::EnterScope(1),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitScope(1),
            SourceProcessingEvent::Connection(0),
        ]);
        if isolate == Some(true) {
            controls.push(ParagraphControl {
                at: 1,
                kind: ParagraphControlKind::PopDirectionalIsolate,
            });
            events.push(SourceProcessingEvent::Control {
                control: 1,
                affinity: Affinity::After,
            });
        }
        events.extend([
            SourceProcessingEvent::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::ExitScope(0),
        ]);
        let processed = std::sync::Arc::new(
            process_contextual_source(ContextualSourceInput {
                source,
                scope_alignment_limits: None,
                computed_line_boundaries: Vec::new(),
                segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
                envelope: SourceProcessingEnvelope::Complete,
                origins,
                connections: vec![SourceProcessingConnection {
                    before_origin: 0,
                    after_origin: 1,
                    kind: ParagraphConnectionKind::Continuous,
                }],
                controls,
                boundary_policies: vec![ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(0),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                }],
                replacements: Vec::new(),
                scopes: vec![
                    SourceProcessingScope::Block {
                        topology: SourceProcessingTopology::Complete,
                        trim: WhiteSpaceTrim::default(),
                        base_style: style.clone(),
                        used_direction: ParagraphBaseDirection::LeftToRight,
                        writing_axis: Some(SourceWritingAxis::Vertical),
                        horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                        font_reference: None,
                    },
                    SourceProcessingScope::Inline {
                        trim: WhiteSpaceTrim::default(),
                        style: Some(Style {
                            text_combine_upright: TextCombineUpright::All,
                            ..style
                        }),
                        boundaries: None,
                        alignment_boundary: None,
                        font_reference: None,
                        horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    },
                ],
                paragraphs: vec![SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: ParagraphStrut::AllocatedContributions,
                }],
                joint_flows: Vec::new(),
                events,
            })
            .unwrap(),
        );
        let paragraph = processed.paragraphs().next().unwrap();
        AnalyzedSource::from_processed_paragraph(
            paragraph,
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap()
    }

    #[test]
    fn actual_zero_raw_cell_has_direction_only_when_its_occurrence_is_enclosed() {
        let analyzed = source_zero_cell(None);
        let core = &analyzed.inner;
        let output = core.combination.as_ref().unwrap();
        assert_eq!(output.bindings.len(), 1);
        let cell = &output.bindings[0];
        assert!(cell.atoms.is_empty());
        assert!(cell.participants.is_empty());
        assert!(!cell.events.is_empty());
        assert_eq!(analyzed.source().text(), &crate::Text::from(" 1"));
        assert_eq!(core.projection.text, "1");
        assert_eq!(
            core.direction
                .query_source_in(0, cell.participants.clone(), cell.events.clone()),
            Some((Some(0), true)),
        );
        for cut in [cell.events.start, cell.events.end] {
            assert_eq!(
                core.direction
                    .query_source_in(0, cell.participants.clone(), cut..cut),
                Some((Some(0), false)),
            );
        }
        assert_eq!(
            core.direction
                .query_in(0, cell.participants.start, cell.participants.end),
            Some((Some(0), false)),
        );
    }

    #[test]
    fn actual_zero_raw_cell_does_not_bleed_into_its_following_weak_range() {
        let analyzed = source_zero_cell(None);
        let core = &analyzed.inner;
        let event = core
            .transform
            .events
            .iter()
            .position(|event| {
                matches!(
                    event, crate::transform::relation::RelationEvent::Text { atom }
                        if core.projection.atoms[*atom].code_point == 0x31
                )
            })
            .unwrap();
        let start = core.transform.logical_prefix_at_cut(event).unwrap();
        let end = core.transform.logical_prefix_at_cut(event + 1).unwrap();
        assert!(start < end);
        assert_eq!(
            core.direction.query_in(0, start, end),
            Some((Some(0), false))
        );
        assert_eq!(
            core.direction
                .query_source_in(0, start..end, event..event + 1),
            Some((Some(0), false)),
        );
    }

    #[test]
    fn actual_zero_raw_cell_keeps_full_matched_and_unmatched_isolate_eligibility() {
        for matched in [true, false] {
            let analyzed = source_zero_cell(Some(matched));
            let core = &analyzed.inner;
            let output = core.combination.as_ref().unwrap();
            assert_eq!(output.bindings.len(), 1);
            let cell = &output.bindings[0];
            assert!(cell.participants.is_empty());
            assert_eq!(core.paragraph.controls.len(), if matched { 2 } else { 1 });
            assert_eq!(
                core.direction
                    .query_source_in(0, cell.participants.clone(), cell.events.clone()),
                Some((None, false)),
                "matched={matched}",
            );
        }
    }

    fn index(classes: &[BidiClass]) -> DirectionIndex {
        let owners = (0..classes.len()).map(BidiOwner::Atom).collect();
        let events: Vec<_> = (0..classes.len())
            .map(|atom| LogicalEvent {
                atom,
                source: atom,
                kind: Event::Text(atom),
            })
            .collect();
        DirectionBody::new(
            classes.iter().copied(),
            owners,
            std::slice::from_ref(&(0..classes.len())),
        )
        .unwrap()
        .bind(&events, std::slice::from_ref(&(0..events.len())), None)
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
            assert_eq!(body.query_in(0, 0, 1), Some((Some(direction), false)));
            assert_eq!(body.query_in(0, 1, 2), Some((Some(direction), true)));
        }
        assert_eq!(
            index(&[B::EuropeanNumber, B::WhiteSpace]).query_in(0, 0, 2),
            Some((None, false))
        );
        assert_eq!(
            index(&[B::RightToLeft, B::LeftToRight]).query_in(0, 1, 2),
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
        assert_eq!(body.query_in(0, 1, 2), Some((Some(0), false)));
        assert_eq!(body.query_in(0, 3, 4), Some((Some(0), true)));
        assert_eq!(
            index(&[B::PopDirectionalIsolate, B::ArabicLetter]).query_in(0, 1, 2),
            Some((Some(1), true))
        );
        assert_eq!(
            index(&[B::FirstStrongIsolate, B::ArabicLetter, B::LeftToRight]).query_in(0, 1, 3),
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
        assert_eq!(nested.query_in(0, 1, 5), Some((Some(1), false)));
        assert_eq!(nested.query_in(0, 6, 7), Some((Some(1), true)));
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
        assert_eq!(body.query_in(0, 0, 1), Some((None, false)));
        assert_eq!(body.query_in(0, 1, 2), Some((None, false)));
        assert_eq!(body.query_in(0, 2, 4), Some((Some(0), true)));
        assert_eq!(body.query_in(0, 2, 2), Some((Some(0), false)));
        assert_eq!(body.query_in(0, 4, 4), Some((None, false)));
        assert_eq!(body.query_in(0, 0, 3), None);
        assert_eq!(body.query_in(0, 3, 2), None);
        assert_eq!(index(&[]).query_in(0, 0, 0), Some((None, false)));
        let isolated = index(&[
            B::RightToLeftIsolate,
            B::RightToLeft,
            B::ParagraphSeparator,
            B::LeftToRight,
        ]);
        assert_eq!(isolated.query_in(0, 0, 3), Some((None, false)));
        assert_eq!(isolated.query_in(0, 3, 4), Some((Some(0), true)));
    }

    #[test]
    fn real_owner_binding_preserves_control_empty_deleted_and_source_zero_custody() {
        use BidiClass as B;
        let processed = crate::process_source_whitespace(
            crate::Source::new(" "),
            crate::Style {
                white_space: crate::WhiteSpace::Discard,
                ..crate::Style::default()
            },
            crate::DocumentSegmentBreaks::LineFeedDefault,
        )
        .unwrap();
        assert!(processed.processed_text().is_empty());
        let deleted = processed.lexical.deletion(
            crate::Range::new(0, 1),
            crate::processing::DeletionCause::Discard,
        );
        let anchor = |item| {
            Event::Anchor(
                ParagraphAnchor {
                    item,
                    affinity: crate::Affinity::After,
                }
                .into(),
            )
        };
        let events = [
            anchor(ParagraphAnchorItem::EmptyOrigin(0)),
            Event::Text(0),
            Event::Deleted(crate::Range::new(0, 1), 0, deleted),
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
            std::slice::from_ref(&(0..3)),
        )
        .unwrap()
        .bind(&events, std::slice::from_ref(&(0..events.len())), None)
        .unwrap();
        assert_eq!(body.query_in(0, 0, 4), Some((Some(0), true)));
        assert_eq!(body.query_in(0, 2, 4), Some((Some(0), false)));
        assert_eq!(body.query_in(0, 4, 5), Some((Some(1), false)));
        assert_eq!(body.query_in(0, 4, 6), Some((Some(1), true)));
        assert_eq!(body.query_in(0, 3, 6), None);
        assert_eq!(body.query_in(0, 7, 7), Some((Some(1), false)));
    }
}
