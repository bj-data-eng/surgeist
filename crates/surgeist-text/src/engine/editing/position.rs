//! Placement-specific observations of the existing interaction rows.

use super::super::{
    PlacedLine,
    layout::data::{FinalizedVisualItem, PlacedLayout},
};
use crate::protocol::{ProtocolError, protocol_error};
use crate::{
    Affinity, Cursor, CursorGeometry, Movement, Point, Range, Rect, Result, SelectionGeometry,
    SelectionRect,
};

/// References real finalized rows. No second geometry or source owner is stored.
#[derive(Clone, Copy)]
pub(super) enum Target {
    Cluster { cluster: usize, right: bool },
    Anchor { anchor: usize },
    Strut,
}
#[derive(Clone, Copy)]
pub(crate) struct Position {
    pub(super) line: usize,
    pub(crate) source: Cursor,
    target: Target,
}

pub(super) fn prepare(lines: &[PlacedLine], stops: &[usize]) -> Vec<Position> {
    let mut positions = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        let first = positions.len();
        let mut push = |source: Cursor, target| {
            positions.push(Position {
                line: line_index,
                source,
                target,
            });
        };
        for item in &line.finalized.data.visual_order {
            match *item {
                FinalizedVisualItem::Cluster(index) => {
                    let cluster = &line.finalized.data.clusters[index];
                    for right in [false, true] {
                        let start = right == (cluster.level & 1 != 0);
                        let (source_index, affinity) = if start {
                            (cluster.source.start, Affinity::After)
                        } else {
                            (cluster.source.end, Affinity::Before)
                        };
                        if stops.binary_search(&source_index).is_ok() {
                            push(
                                Cursor::new(source_index, affinity),
                                Target::Cluster {
                                    cluster: index,
                                    right,
                                },
                            );
                        }
                    }
                }
                FinalizedVisualItem::Anchor(anchor) => {
                    let row = &line.finalized.data.anchors[anchor];
                    let unit = &line.finalized.selected.presentation.units[row.unit];
                    push(
                        Cursor::new(unit.source.start, Affinity::After),
                        Target::Anchor { anchor },
                    );
                    if !unit.source.is_empty() {
                        push(
                            Cursor::new(unit.source.end, Affinity::Before),
                            Target::Anchor { anchor },
                        );
                    }
                }
                FinalizedVisualItem::InlineBox(_) => {}
            }
        }
        if positions.len() == first {
            positions.push(Position {
                line: line_index,
                source: Cursor::new(
                    line.finalized.selected.presentation.source.start,
                    Affinity::After,
                ),
                target: Target::Strut,
            });
        }
    }
    positions
}

fn row(layout: &PlacedLayout, index: usize) -> Result<&Position> {
    layout
        .interaction
        .positions
        .get(index)
        .ok_or_else(|| protocol_error(ProtocolError::InvalidPlacedPosition { position: index }))
}
fn local_x(line: &PlacedLine, target: Target) -> f64 {
    match target {
        Target::Cluster { cluster, right } => {
            let cluster = &line.finalized.data.clusters[cluster];
            cluster.x + if right { cluster.advance } else { 0.0 }
        }
        Target::Anchor { anchor } => line.finalized.data.anchors[anchor].slot.x,
        Target::Strut => line.finalized.data.metrics.offset,
    }
}
pub(crate) fn geometry(layout: &PlacedLayout, index: usize) -> Result<CursorGeometry> {
    let position = row(layout, index)?;
    let line = &layout.lines[position.line];
    let metrics = line.finalized.data.metrics;
    Ok(CursorGeometry::new(crate::layout::line_rect(
        line,
        Rect::new(
            local_x(line, position.target),
            metrics.block_min,
            1.0,
            metrics.block_max - metrics.block_min,
        ),
    )))
}

/// An executed deletion owns its original interval even though it paints no
/// cluster. Lend the existing occurrence row instead of reconstructing an edge
/// from a numerically adjacent source offset.
pub(super) fn deleted_position(layout: &PlacedLayout, value: Cursor) -> Option<usize> {
    layout
        .interaction
        .positions
        .iter()
        .enumerate()
        .find_map(|(index, position)| {
            let Target::Anchor { anchor } = position.target else {
                return None;
            };
            let line = &layout.lines[position.line];
            let unit = &line.finalized.selected.presentation.units
                [line.finalized.data.anchors[anchor].unit];
            let eligible = super::owns_source_side(unit.source, value);
            (matches!(
                unit.kind,
                crate::protocol::prepared::PresentationUnitKind::Deleted
            ) && eligible)
                .then_some(index)
        })
}
pub(crate) fn hit(layout: &PlacedLayout, point: Point) -> Option<usize> {
    let source = super::cursor::from_point(layout, point);
    let matching = layout
        .interaction
        .positions
        .iter()
        .any(|row| row.source == source);
    layout
        .interaction
        .positions
        .iter()
        .enumerate()
        .filter(|(_, row)| !matching || row.source == source)
        .min_by(|(a, _), (b, _)| {
            distance(
                geometry(layout, *a).expect("owned interaction row").rect(),
                point,
            )
            .total_cmp(&distance(
                geometry(layout, *b).expect("owned interaction row").rect(),
                point,
            ))
        })
        .map(|(index, _)| index)
}
fn distance(rect: Rect, point: Point) -> f64 {
    let dy = if point.y < rect.origin.y {
        rect.origin.y - point.y
    } else if point.y > rect.origin.y + rect.size.height {
        point.y - (rect.origin.y + rect.size.height)
    } else {
        0.0
    };
    // Scale the comparison without squared distances which can overflow for
    // otherwise admitted finite placement coordinates.
    (point.x - rect.origin.x).hypot(dy)
}
pub(crate) fn movement(layout: &PlacedLayout, index: usize, movement: Movement) -> Result<usize> {
    let current = row(layout, index)?;
    let rows = &layout.interaction.positions;
    let last = rows.len().saturating_sub(1);
    Ok(match movement {
        Movement::DocumentStart => 0,
        Movement::DocumentEnd => last,
        Movement::PreviousCluster => index.saturating_sub(1),
        Movement::NextCluster => (index + 1).min(last),
        Movement::LineStart => rows
            .iter()
            .position(|row| row.line == current.line)
            .unwrap_or(index),
        Movement::LineEnd => rows
            .iter()
            .rposition(|row| row.line == current.line)
            .unwrap_or(index),
        Movement::PreviousLine | Movement::NextLine => {
            let target = if movement == Movement::PreviousLine {
                current.line.checked_sub(1)
            } else {
                current
                    .line
                    .checked_add(1)
                    .filter(|&line| line < layout.lines.len())
            };
            match target {
                None => index,
                Some(target) => {
                    let x = geometry(layout, index)?.rect().origin.x;
                    rows.iter()
                        .enumerate()
                        .filter(|(_, row)| row.line == target)
                        .min_by(|(a, _), (b, _)| {
                            (geometry(layout, *a).expect("owned row").rect().origin.x - x)
                                .abs()
                                .total_cmp(
                                    &(geometry(layout, *b).expect("owned row").rect().origin.x - x)
                                        .abs(),
                                )
                        })
                        .map_or(index, |(index, _)| index)
                }
            }
        }
        Movement::PreviousWord | Movement::NextWord => {
            let forward = movement == Movement::NextWord;
            let target = match current.target {
                Target::Cluster { cluster, right } => {
                    let level = layout.lines[current.line].finalized.data.clusters[cluster].level;
                    super::cursor::logical_word_from_cluster(
                        layout,
                        current.source,
                        forward,
                        (current.line, cluster),
                        right != (level & 1 != 0),
                    )
                }
                Target::Anchor { anchor } => super::cursor::logical_word_from_anchor(
                    layout,
                    current.source,
                    forward,
                    current.line,
                    anchor,
                ),
                Target::Strut => return Ok(index),
            };
            match target.row {
                None => index,
                Some(((line, cluster), end)) => {
                    let level = layout.lines[line].finalized.data.clusters[cluster].level;
                    let right = end != (level & 1 != 0);
                    rows.iter()
                        .position(|position| {
                            position.line == line
                                && matches!(position.target, Target::Cluster {cluster: actual, right: actual_right}
                                    if actual == cluster && actual_right == right)
                        })
                        .ok_or_else(|| protocol_error(ProtocolError::InvalidPlacedPosition { position: index }))?
                }
            }
        }
    })
}
pub(crate) fn selection(
    layout: &PlacedLayout,
    anchor: usize,
    focus: usize,
) -> Result<SelectionGeometry> {
    let a = row(layout, anchor)?;
    let b = row(layout, focus)?;
    if anchor == focus {
        return Ok(SelectionGeometry::default());
    }
    if a.source.index() != b.source.index() {
        let (a, b) = if a.source.index() < b.source.index() {
            (a, b)
        } else {
            (b, a)
        };
        return super::selection::between(
            layout,
            Range::new(a.source.index(), b.source.index()),
            a.line,
            b.line,
        );
    }
    // Distinct source-less occurrences are selected through their actual line
    // slots even though the explicit source projection is a collapsed range.
    let (a, b) = if anchor < focus { (a, b) } else { (b, a) };
    let mut output = Vec::new();
    for index in a.line.min(b.line)..=a.line.max(b.line) {
        let line = &layout.lines[index];
        let metrics = line.finalized.data.metrics;
        let start = if index == a.line {
            local_x(line, a.target)
        } else {
            metrics.inline_min
        };
        let end = if index == b.line {
            local_x(line, b.target)
        } else {
            metrics.inline_max
        };
        output.push(SelectionRect::new(
            crate::layout::line_rect(
                line,
                Rect::new(
                    start.min(end),
                    metrics.block_min,
                    (end - start).abs(),
                    metrics.block_max - metrics.block_min,
                ),
            ),
            index,
        ));
    }
    Ok(SelectionGeometry::new(output))
}

pub(crate) fn validate_source(layout: &PlacedLayout, value: Cursor) -> Result<()> {
    layout.prepared.analyzed.paragraph.admit(
        layout.prepared.analyzed.processed.source(),
        Range::new(value.index(), value.index()),
    )?;
    let coverage = layout.coverage.source;
    let paragraph = layout
        .prepared
        .analyzed
        .paragraph
        .extent(&layout.prepared.analyzed.processed);
    let outside_facing_start = coverage.start != paragraph.start
        && value.index() == coverage.start
        && value.affinity() == Affinity::Before;
    let outside_facing_end = coverage.end != paragraph.end
        && value.index() == coverage.end
        && value.affinity() == Affinity::After;
    if value.index() < coverage.start
        || value.index() > coverage.end
        || outside_facing_start
        || outside_facing_end
    {
        return Err(protocol_error(ProtocolError::OutsidePlacement {
            index: value.index(),
            coverage,
        }));
    }
    // Only distinct actual source-less occurrence rows create this ambiguity;
    // ordinary source edges continue to use the existing affinity policy.
    let mut first = None;
    for position in &layout.interaction.positions {
        if position.source.index() != value.index() {
            continue;
        }
        let identity = match position.target {
            Target::Anchor { anchor } => {
                let line = &layout.lines[position.line];
                let unit = &line.finalized.selected.presentation.units
                    [line.finalized.data.anchors[anchor].unit];
                if !unit.source.is_empty() {
                    continue;
                }
                use crate::protocol::prepared::PresentationUnitKind;
                match unit.kind {
                    PresentationUnitKind::Replacement { alternative } => {
                        (position.line, 1, alternative)
                    }
                    PresentationUnitKind::EmptyOrigin { origin } => (position.line, 2, origin),
                    _ => (position.line, 0, anchor),
                }
            }
            Target::Strut => (position.line, 3, 0),
            Target::Cluster { cluster, .. } => {
                let line = &layout.lines[position.line];
                let cluster = &line.finalized.data.clusters[cluster];
                let alternative =
                    line.finalized
                        .selected
                        .presentation
                        .units
                        .iter()
                        .find_map(|unit| {
                            if unit.presentation.start < cluster.presentation.end
                            && cluster.presentation.start < unit.presentation.end
                            && let crate::protocol::prepared::PresentationUnitKind::Replacement {
                                alternative,
                            } = unit.kind
                        {
                            return Some(alternative);
                        }
                            None
                        });
                let Some(alternative) = alternative else {
                    continue;
                };
                (position.line, 1, alternative)
            }
        };
        if first.is_some_and(|old| old != identity) {
            return Err(protocol_error(ProtocolError::AmbiguousSourcePosition {
                index: value.index(),
                affinity: value.affinity(),
            }));
        }
        first = Some(identity);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::paragraph::{
        ParagraphAnchor, ParagraphAnchorItem, ParagraphControl, ParagraphControlKind,
    };
    use crate::protocol::*;
    use crate::{
        Alignment, AnalyzedSource, DocumentSegmentBreaks, Font, FontCatalogueMode, Layout,
        LineHeight, Source, Style, System, SystemOptions,
    };

    fn four_source_less_lines() -> Layout {
        native_four_lines(false)
    }

    fn native_four_lines(with_text: bool) -> Layout {
        native_break_lines(
            if with_text { "A" } else { "" },
            usize::from(with_text),
            3,
            false,
        )
    }

    fn native_break_lines(text: &str, at: usize, breaks: usize, disconnected: bool) -> Layout {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let source = Source::new(text);
        let source_end = source.text().len();
        let analyzed = AnalyzedSource::from_paragraph(ParagraphInput {
            source: source.clone(),
            base_style: Style {
                font: Font::new().family("Ahem"),
                size: 20.0,
                line_height: LineHeight::FontSizeRelative(1.0),
                ..Style::default()
            },
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: crate::protocol::paragraph::ParagraphStrut::DefaultFont,
            origins: if disconnected {
                assert_eq!(text, "AB");
                vec![
                    crate::ParagraphOrigin {
                        source: Source::new("A"),
                        local: Range::new(0, 1),
                        aggregate: Range::new(0, 1),
                        kind: crate::ParagraphOriginKind::Authored,
                    },
                    crate::ParagraphOrigin {
                        source: Source::new("B"),
                        local: Range::new(0, 1),
                        aggregate: Range::new(1, 2),
                        kind: crate::ParagraphOriginKind::Authored,
                    },
                ]
            } else if source_end != 0 {
                vec![crate::ParagraphOrigin {
                    source,
                    local: Range::new(0, source_end),
                    aggregate: Range::new(0, source_end),
                    kind: crate::ParagraphOriginKind::Authored,
                }]
            } else {
                Vec::new()
            },
            connections: if disconnected {
                vec![crate::ParagraphConnection {
                    at: 1,
                    kind: crate::ParagraphConnectionKind::Disconnected,
                }]
            } else {
                Vec::new()
            },
            controls: vec![
                ParagraphControl {
                    at,
                    kind: ParagraphControlKind::ForcedBreak
                };
                breaks
            ],
            boundary_policies: Vec::new(),
            replacements: Vec::new(),
            anchor_order: (if breaks == 3 {
                vec![2, 0, 1]
            } else {
                (0..breaks).collect()
            })
            .into_iter()
            .map(|index| ParagraphAnchor {
                item: ParagraphAnchorItem::Control(index),
                affinity: Affinity::After,
            })
            .collect(),
        })
        .unwrap();
        let prepared = system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        let participants: Vec<_> = prepared.participants().collect();
        let boundaries: Vec<_> = prepared
            .boundary_views()
            .map(|view| view.handle())
            .collect();
        let opportunities: Vec<_> = prepared
            .opportunity_views()
            .filter(|view| {
                matches!(
                    view.kind(),
                    PreparedBreakKind::Forced | PreparedBreakKind::ParagraphEnd
                )
            })
            .map(|view| view.handle())
            .collect();
        assert_eq!(participants.len(), source_end + breaks);
        assert_eq!(opportunities.len(), breaks + 1);
        let mut start = boundaries[0].clone();
        let mut from = 0;
        let mut lines = Vec::new();
        for (index, opportunity) in opportunities.into_iter().enumerate() {
            let end = opportunity.view().boundary();
            let to = end.view().participant_offset();
            let selected = prepared
                .select(LineSelectionRequest {
                    start: start.clone(),
                    end: end.clone(),
                    participants: participants[from..to].to_vec(),
                    termination: opportunity,
                    alternative: None,
                    context: LineContext {
                        base_direction: ParagraphBaseDirection::LeftToRight,
                        orientation: LineOrientation::Horizontal,
                        start: if index == 0 {
                            LineStart::Paragraph
                        } else {
                            LineStart::Continuation
                        },
                        end: if index == breaks {
                            LineEnd::Paragraph
                        } else {
                            LineEnd::Forced
                        },
                        available_inline: None,
                        indent: 0.0,
                        alignment: Alignment::Start,
                        tabs: TabContext::NoTabs,
                        hanging: HangingContext::None,
                        spacing: SpacingPolicy::None,
                    },
                    atomics: Vec::new(),
                    line_strut: None,
                    controls: participants[from..to]
                        .iter()
                        .filter_map(|participant| {
                            if let PreparedParticipant::Control(control) = participant {
                                Some(ControlAllocation {
                                    occurrence: control.clone(),
                                    strut: None,
                                })
                            } else {
                                None
                            }
                        })
                        .collect(),
                })
                .unwrap();
            let expected_control = selected
                .request()
                .participants
                .iter()
                .find_map(|participant| {
                    if let PreparedParticipant::Control(control) = participant {
                        Some(control.clone())
                    } else {
                        None
                    }
                });
            let line = selected.finalize(SpacingAllocations::None).unwrap();
            if let Some(expected) = expected_control {
                assert_eq!(line.control_slots().next().unwrap().control(), expected);
                assert!(line.fragment_views().any(|fragment| matches!(fragment.owner(),
                    crate::protocol::readback::FinalizedFragmentOwner::Control(actual) if actual == expected)));
            }
            lines.push(LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 16.0 + index as f64 * 20.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: Vec::new(),
            });
            start = end;
            from = to;
        }
        Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::WholeParagraph,
            lines,
        })
        .unwrap()
    }

    #[test]
    fn genuine_same_source_controls_retain_middle_positions_and_foreign_placement() {
        let layout = four_source_less_lines();
        let positions: Vec<_> = layout.positions().collect();
        assert_eq!(positions.len(), 4);
        let metric_runs = layout.glyph_runs();
        assert_eq!(metric_runs.len(), 4);
        assert!(metric_runs.iter().all(|run| run.glyphs().is_empty()));
        assert_eq!(
            metric_runs
                .iter()
                .map(|run| run.style().size)
                .collect::<Vec<_>>(),
            [20.0; 4]
        );
        assert!(
            metric_runs
                .iter()
                .all(|run| run.style() == layout.default_style())
        );
        assert!(
            positions
                .iter()
                .all(|position| position.source_cursor() == Cursor::new(0, Affinity::After))
        );
        let a = &positions[1];
        let b = &positions[2];
        assert_ne!(a, b);
        assert_eq!(
            layout.position_geometry(a).unwrap().rect(),
            Rect::new(0.0, 20.0, 1.0, 20.0)
        );
        assert_eq!(
            layout.position_geometry(b).unwrap().rect(),
            Rect::new(0.0, 40.0, 1.0, 20.0)
        );
        assert_eq!(layout.move_position(a, Movement::NextLine).unwrap(), *b);
        assert_eq!(
            layout.move_position(b, Movement::PreviousCluster).unwrap(),
            *a
        );
        let selection = layout.position_selection(a, b).unwrap();
        assert_eq!(selection.rects().len(), 2);
        assert_eq!(selection.rects()[0].line(), 1);
        assert_eq!(selection.rects()[1].line(), 2);
        let clone = layout.clone();
        assert_eq!(
            clone.position_geometry(a).unwrap(),
            layout.position_geometry(a).unwrap()
        );
        let foreign = four_source_less_lines();
        assert!(foreign.position_geometry(a).is_err());
        assert!(
            layout
                .position_selection(a, &foreign.positions().nth(2).unwrap())
                .is_err()
        );
        assert!(layout.cursor(Cursor::new(0, Affinity::After)).is_err());
        assert_eq!(layout.source().text().code_units(), &[]);
    }

    fn partial_word_line() -> Layout {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let source = Source::new("abcdef");
        let analysis = AnalyzedSource::from_paragraph(ParagraphInput {
            source: source.clone(),
            base_style: Style {
                font: Font::new().family("Ahem"),
                size: 20.0,
                line_height: LineHeight::FontSizeRelative(1.0),
                ..Style::default()
            },
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: crate::ParagraphStrut::DefaultFont,
            origins: vec![crate::ParagraphOrigin {
                source,
                local: Range::new(0, 6),
                aggregate: Range::new(0, 6),
                kind: crate::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![ParagraphControl {
                at: 4,
                kind: ParagraphControlKind::OptionalBreak {
                    wrap: crate::Wrap::Word,
                },
            }],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![ParagraphAnchor {
                item: ParagraphAnchorItem::Control(0),
                affinity: Affinity::After,
            }],
        })
        .unwrap();
        let prepared = system
            .prepare(&analysis, PreparationOptions::default())
            .unwrap();
        let start = prepared
            .boundary_views()
            .find(|view| view.source_before().get() == 2)
            .unwrap()
            .handle();
        let termination = prepared
            .opportunity_views()
            .find(|view| {
                view.kind() == PreparedBreakKind::Allowed
                    && view.boundary().view().source_before().get() == 4
            })
            .unwrap()
            .handle();
        let end = termination.view().boundary();
        let participants: Vec<_> = prepared
            .participants()
            .skip(start.view().participant_offset())
            .take(end.view().participant_offset() - start.view().participant_offset())
            .collect();
        let controls = participants
            .iter()
            .filter_map(|participant| {
                if let PreparedParticipant::Control(control) = participant {
                    Some(ControlAllocation {
                        occurrence: control.clone(),
                        strut: None,
                    })
                } else {
                    None
                }
            })
            .collect();
        let selected = prepared
            .select(LineSelectionRequest {
                start: start.clone(),
                end: end.clone(),
                participants,
                termination,
                alternative: None,
                context: LineContext {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    orientation: LineOrientation::Horizontal,
                    start: LineStart::Continuation,
                    end: LineEnd::Soft,
                    available_inline: None,
                    indent: 0.0,
                    alignment: Alignment::Start,
                    tabs: TabContext::NoTabs,
                    hanging: HangingContext::None,
                    spacing: SpacingPolicy::None,
                },
                atomics: vec![],
                controls,
                line_strut: None,
            })
            .unwrap();
        let line = selected.finalize(SpacingAllocations::None).unwrap();
        Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::Range { start, end },
            lines: vec![LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 16.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            }],
        })
        .unwrap()
    }

    fn refuses_global_target(movement: Movement, index: usize) {
        let layout = partial_word_line();
        assert_eq!(layout.lines()[0].range(), Range::new(2, 4));
        let error=layout.move_cursor(Cursor::new(3,Affinity::After),movement).expect_err("a source-global target outside this actual placement cannot acquire its edge geometry");
        assert_eq!(
            error.detail(),
            Some(&crate::ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::OutsidePlacement {
                    index,
                    coverage: Range::new(2, 4)
                }
            })
        );
    }
    #[test]
    fn partial_source_document_start_refuses_the_uncovered_original_start() {
        refuses_global_target(Movement::DocumentStart, 0);
    }
    #[test]
    fn partial_source_document_end_refuses_the_uncovered_original_end() {
        refuses_global_target(Movement::DocumentEnd, 6);
    }
    #[test]
    fn partial_source_previous_word_refuses_the_uncovered_original_word_start() {
        refuses_global_target(Movement::PreviousWord, 0);
    }
    #[test]
    fn partial_source_next_word_refuses_the_uncovered_original_word_end() {
        refuses_global_target(Movement::NextWord, 6);
    }
    #[test]
    fn partial_source_exact_positions_keep_their_real_placement_relative_edges() {
        let layout = partial_word_line();
        let positions: Vec<_> = layout.positions().collect();
        let middle = positions
            .iter()
            .find(|position| position.source_cursor().index() == 3)
            .unwrap();
        assert_eq!(
            layout
                .move_position(middle, Movement::DocumentStart)
                .unwrap(),
            positions[0]
        );
        assert_eq!(
            layout.move_position(middle, Movement::DocumentEnd).unwrap(),
            *positions.last().unwrap()
        );
        assert_eq!(
            layout
                .position_geometry(&positions[0])
                .unwrap()
                .rect()
                .origin
                .x,
            0.0
        );
        assert_eq!(
            layout
                .position_geometry(positions.last().unwrap())
                .unwrap()
                .rect()
                .origin
                .x,
            40.0
        );
        let previous_word = layout
            .move_position(middle, Movement::PreviousWord)
            .unwrap();
        let next_word = layout.move_position(middle, Movement::NextWord).unwrap();
        assert_eq!(previous_word.source_cursor().index(), 2);
        assert_eq!(next_word.source_cursor().index(), 4);
        assert_eq!(
            layout
                .position_geometry(&previous_word)
                .unwrap()
                .rect()
                .origin
                .x,
            0.0
        );
        assert_eq!(
            layout
                .position_geometry(&next_word)
                .unwrap()
                .rect()
                .origin
                .x,
            40.0
        );
    }

    #[test]
    fn partial_source_raw_endpoint_affinity_requires_the_selected_side() {
        let layout = partial_word_line();
        for (cursor, x) in [
            (Cursor::new(2, Affinity::After), 0.0),
            (Cursor::new(4, Affinity::Before), 40.0),
        ] {
            assert_eq!(layout.cursor(cursor).unwrap().rect().origin.x, x);
        }
        for cursor in [
            Cursor::new(2, Affinity::Before),
            Cursor::new(4, Affinity::After),
        ] {
            assert_eq!(
                layout.cursor(cursor).unwrap_err().detail(),
                Some(&crate::ErrorDetail::InvalidTextProtocol {
                    reason: ProtocolError::OutsidePlacement {
                        index: cursor.index(),
                        coverage: Range::new(2, 4)
                    }
                })
            );
        }
        // This exact native-control position is covered even though its source
        // projection's downstream authored side lies outside the placement.
        let control = layout
            .positions()
            .find(|position| position.source_cursor() == Cursor::new(4, Affinity::After))
            .unwrap();
        assert_eq!(
            layout.position_geometry(&control).unwrap().rect().origin.x,
            40.0
        );
    }

    fn native_whitespace_line(text: &str, white_space: crate::WhiteSpace) -> Layout {
        native_whitespace_line_with_direction(
            text,
            white_space,
            ParagraphBaseDirection::LeftToRight,
        )
    }

    fn native_whitespace_prepared(
        text: &str,
        white_space: crate::WhiteSpace,
        base_direction: ParagraphBaseDirection,
    ) -> PreparedText {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let source = Source::new(text);
        let end = text.encode_utf16().count();
        let analysis = AnalyzedSource::from_paragraph(ParagraphInput {
            source: source.clone(),
            base_style: Style {
                font: Font::new().family("Ahem"),
                size: 20.0,
                white_space,
                line_height: LineHeight::FontSizeRelative(1.0),
                ..Style::default()
            },
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            base_direction,
            paragraph_strut: crate::ParagraphStrut::DefaultFont,
            origins: vec![crate::ParagraphOrigin {
                source,
                local: Range::new(0, end),
                aggregate: Range::new(0, end),
                kind: crate::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        })
        .unwrap();
        system
            .prepare(&analysis, PreparationOptions::default())
            .unwrap()
    }

    fn native_whitespace_line_with_direction(
        text: &str,
        white_space: crate::WhiteSpace,
        base_direction: ParagraphBaseDirection,
    ) -> Layout {
        let prepared = native_whitespace_prepared(text, white_space, base_direction);
        let termination = prepared
            .opportunity_views()
            .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        let selected = prepared
            .select(LineSelectionRequest {
                start: prepared.boundary_views().next().unwrap().handle(),
                end: termination.view().boundary(),
                participants: prepared.participants().collect(),
                termination,
                alternative: None,
                context: LineContext {
                    base_direction,
                    orientation: LineOrientation::Horizontal,
                    start: LineStart::Paragraph,
                    end: LineEnd::Paragraph,
                    available_inline: None,
                    indent: 0.0,
                    alignment: Alignment::Start,
                    tabs: TabContext::NoTabs,
                    hanging: HangingContext::None,
                    spacing: SpacingPolicy::None,
                },
                atomics: vec![],
                controls: vec![],
                line_strut: None,
            })
            .unwrap();
        let line = selected.finalize(SpacingAllocations::None).unwrap();
        Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::WholeParagraph,
            lines: vec![LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 16.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            }],
        })
        .unwrap()
    }

    #[test]
    fn native_collapsed_gap_next_word_uses_the_next_actual_source_cluster() {
        let preserved = native_whitespace_line("A B", crate::WhiteSpace::Preserve);
        assert_eq!(
            preserved
                .move_cursor(Cursor::new(1, Affinity::After), Movement::NextWord)
                .unwrap(),
            Cursor::new(2, Affinity::Before)
        );
        assert_eq!(preserved.source().text().code_units(), [0x41, 0x20, 0x42]);
        let collapsed = native_whitespace_line("A  B", crate::WhiteSpace::Collapse);
        assert_eq!(
            collapsed
                .analyzed_source()
                .processed_source()
                .processed_text()
                .code_units(),
            [0x41, 0x20, 0x42]
        );
        assert_eq!(
            collapsed.source().text().code_units(),
            [0x41, 0x20, 0x20, 0x42]
        );
        assert_eq!(
            collapsed
                .clusters()
                .iter()
                .map(|cluster| cluster.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 2), Range::new(3, 4)]
        );
        let moved = collapsed
            .move_cursor(Cursor::new(1, Affinity::After), Movement::NextWord)
            .unwrap();
        assert_eq!(moved, Cursor::new(3, Affinity::Before));
        let b_start = collapsed
            .clusters()
            .into_iter()
            .find(|cluster| cluster.range() == Range::new(3, 4))
            .unwrap()
            .range()
            .start;
        assert_eq!(moved.index(), b_start);
        assert_eq!(collapsed.cursor(moved).unwrap().rect().origin.x, 40.0);
        assert_eq!(
            collapsed.move_cursor(moved, Movement::NextWord).unwrap(),
            Cursor::new(4, Affinity::Before)
        );
    }

    #[test]
    fn native_collapsed_gap_previous_word_uses_the_previous_actual_source_cluster() {
        let preserved = native_whitespace_line("A B", crate::WhiteSpace::Preserve);
        assert_eq!(
            preserved
                .move_cursor(Cursor::new(2, Affinity::After), Movement::PreviousWord)
                .unwrap(),
            Cursor::new(0, Affinity::After)
        );
        let collapsed = native_whitespace_line("A  B", crate::WhiteSpace::Collapse);
        assert_eq!(
            collapsed.source().text().code_units(),
            [0x41, 0x20, 0x20, 0x42]
        );
        assert_eq!(
            collapsed
                .move_cursor(Cursor::new(3, Affinity::After), Movement::PreviousWord)
                .unwrap(),
            Cursor::new(0, Affinity::After)
        );
    }

    #[test]
    fn native_mixed_occurrence_next_cluster_refuses_an_ambiguous_source_projection() {
        let layout = native_four_lines(true);
        assert_eq!(layout.lines().len(), 4);
        let error = layout
            .move_cursor(Cursor::new(0, Affinity::After), Movement::NextCluster)
            .expect_err("the target projects several real control/line occurrences to source1");
        assert_eq!(
            error.detail(),
            Some(&crate::ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::AmbiguousSourcePosition {
                    index: 1,
                    affinity: Affinity::Before
                }
            })
        );
    }
    #[test]
    fn native_mixed_occurrence_next_line_refuses_an_ambiguous_source_projection() {
        let layout = native_four_lines(true);
        assert_eq!(layout.lines().len(), 4);
        let error = layout
            .move_cursor(Cursor::new(0, Affinity::After), Movement::NextLine)
            .expect_err("the target projects several real control/line occurrences to source1");
        assert_eq!(
            error.detail(),
            Some(&crate::ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::AmbiguousSourcePosition {
                    index: 1,
                    affinity: Affinity::After
                }
            })
        );
    }
    #[test]
    fn native_mixed_occurrence_exact_handles_traverse_the_real_cluster_and_line_targets() {
        let layout = native_four_lines(true);
        let start = layout
            .positions()
            .find(|position| position.source_cursor() == Cursor::new(0, Affinity::After))
            .unwrap();
        let cluster = layout.move_position(&start, Movement::NextCluster).unwrap();
        let line = layout.move_position(&start, Movement::NextLine).unwrap();
        assert_eq!(cluster.source_cursor(), Cursor::new(1, Affinity::Before));
        assert_eq!(line.source_cursor(), Cursor::new(1, Affinity::After));
        assert_eq!(
            layout.position_geometry(&cluster).unwrap().rect(),
            Rect::new(20.0, 0.0, 1.0, 20.0)
        );
        assert_eq!(
            layout.position_geometry(&line).unwrap().rect(),
            Rect::new(0.0, 20.0, 1.0, 20.0)
        );
        assert_ne!(cluster, line);
    }

    #[test]
    fn native_collapsed_gap_affinity_probe_keeps_the_actual_b_start_geometry() {
        let layout = native_whitespace_line("A  B", crate::WhiteSpace::Collapse);
        let line = &layout.inner.lines[0].finalized;
        let b = line
            .data
            .clusters
            .iter()
            .find(|cluster| cluster.source == Range::new(3, 4))
            .unwrap();
        assert_eq!(
            (b.source, b.presentation, b.x, b.advance),
            (Range::new(3, 4), Range::new(2, 3), 40.0, 20.0)
        );
        let anchor = line
            .data
            .anchors
            .iter()
            .find(|anchor| {
                matches!(
                    line.selected.presentation.units[anchor.unit].kind,
                    crate::protocol::prepared::PresentationUnitKind::Deleted
                )
            })
            .unwrap();
        let deleted = &line.selected.presentation.units[anchor.unit];
        assert_eq!(
            (
                deleted.source,
                deleted.processed,
                deleted.presentation,
                anchor.slot
            ),
            (
                Range::new(2, 3),
                Range::new(2, 2),
                Range::new(2, 2),
                Point::new(40.0, 16.0)
            )
        );
        // Only original-source affinity differs. The actual source/presentation
        // rows and finite physical anchor are the same in these two queries.
        assert_eq!(
            layout
                .cursor(Cursor::new(3, Affinity::After))
                .unwrap()
                .rect()
                .origin
                .x,
            40.0
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(3, Affinity::Before))
                .unwrap()
                .rect()
                .origin
                .x,
            40.0
        );
    }

    #[test]
    fn native_exact_word_target_keeps_the_b_row_across_a_forced_control() {
        let layout = native_break_lines("AB", 1, 1, true);
        assert_eq!(layout.lines().len(), 2);
        let b = layout
            .clusters()
            .into_iter()
            .find(|cluster| cluster.range() == Range::new(1, 2))
            .unwrap();
        assert_eq!(b.bounds(), Rect::new(0.0, 20.0, 20.0, 20.0));
        let prepared_b = layout
            .inner
            .prepared
            .shaped
            .clusters
            .iter()
            .find(|cluster| cluster.source == Range::new(1, 2))
            .unwrap();
        assert_eq!(prepared_b.boundary, crate::engine::analysis::Boundary::Line);
        let start = layout
            .positions()
            .find(|position| position.source_cursor() == Cursor::new(0, Affinity::After))
            .unwrap();
        let moved = layout.move_position(&start, Movement::NextWord).unwrap();
        assert_eq!(
            layout.position_geometry(&moved).unwrap().rect(),
            Rect::new(0.0, 20.0, 1.0, 20.0)
        );
        assert_eq!(moved.source_cursor(), Cursor::new(1, Affinity::After));
    }

    #[test]
    fn native_preserved_long_space_interior_word_uses_the_original_segment() {
        let layout = native_whitespace_line("A    B", crate::WhiteSpace::Preserve);
        assert_eq!(
            layout.source().text().code_units(),
            [0x41, 0x20, 0x20, 0x20, 0x20, 0x42]
        );
        let cursor = Cursor::new(3, Affinity::After);
        let next = layout.move_cursor(cursor, Movement::NextWord).unwrap();
        let previous = layout.move_cursor(cursor, Movement::PreviousWord).unwrap();
        assert_eq!(next, Cursor::new(5, Affinity::Before));
        assert_eq!(previous, Cursor::new(1, Affinity::Before));
    }

    #[test]
    fn native_collapsed_gap_rtl_affinities_keep_the_actual_deleted_slot() {
        let layout = native_whitespace_line_with_direction(
            "\u{05d0}  \u{05d1}",
            crate::WhiteSpace::Collapse,
            ParagraphBaseDirection::RightToLeft,
        );
        let line = &layout.inner.lines[0].finalized;
        let bet = line
            .data
            .clusters
            .iter()
            .find(|cluster| cluster.source == Range::new(3, 4))
            .unwrap();
        assert_eq!((bet.level, bet.x, bet.advance), (1, 0.0, 20.0));
        let anchor = line
            .data
            .anchors
            .iter()
            .find(|anchor| {
                matches!(
                    line.selected.presentation.units[anchor.unit].kind,
                    crate::protocol::prepared::PresentationUnitKind::Deleted
                )
            })
            .unwrap();
        assert_eq!(anchor.slot, Point::new(20.0, 16.0));
        for affinity in [Affinity::Before, Affinity::After] {
            assert_eq!(
                layout
                    .cursor(Cursor::new(3, affinity))
                    .unwrap()
                    .rect()
                    .origin
                    .x,
                anchor.slot.x
            );
        }
    }

    #[test]
    fn native_collapsed_gap_interior_word_queries_use_real_neighbor_rows() {
        let layout = native_whitespace_line("A    B", crate::WhiteSpace::Collapse);
        assert_eq!(
            layout.source().text().code_units(),
            [0x41, 0x20, 0x20, 0x20, 0x20, 0x42]
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(3, Affinity::After))
                .unwrap()
                .rect()
                .origin
                .x,
            40.0
        );
        let next = layout
            .move_cursor(Cursor::new(3, Affinity::After), Movement::NextWord)
            .unwrap();
        let previous = layout
            .move_cursor(Cursor::new(3, Affinity::After), Movement::PreviousWord)
            .unwrap();
        assert_eq!(
            [next, previous],
            [
                Cursor::new(5, Affinity::Before),
                Cursor::new(1, Affinity::Before)
            ]
        );
    }

    fn native_whitespace_soft_lines(
        text: &str,
        mode: crate::WhiteSpace,
        split_at: usize,
    ) -> Layout {
        let prepared = native_whitespace_prepared(text, mode, ParagraphBaseDirection::LeftToRight);
        let split = prepared
            .opportunity_views()
            .find(|view| {
                view.kind() == PreparedBreakKind::Allowed
                    && view.boundary().view().source_before().get() == split_at
            })
            .expect("the actual lexical boundary after the whitespace is Allowed")
            .handle();
        let end = prepared
            .opportunity_views()
            .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        let mut start = prepared.boundary_views().next().unwrap().handle();
        let mut lines = Vec::new();
        for (index, termination) in [split, end].into_iter().enumerate() {
            let end = termination.view().boundary();
            let participants = prepared
                .participants()
                .skip(start.view().participant_offset())
                .take(end.view().participant_offset() - start.view().participant_offset())
                .collect();
            let selected = prepared
                .select(LineSelectionRequest {
                    start: start.clone(),
                    end: end.clone(),
                    participants,
                    termination,
                    alternative: None,
                    context: LineContext {
                        base_direction: ParagraphBaseDirection::LeftToRight,
                        orientation: LineOrientation::Horizontal,
                        start: if index == 0 {
                            LineStart::Paragraph
                        } else {
                            LineStart::Continuation
                        },
                        end: if index == 0 {
                            LineEnd::Soft
                        } else {
                            LineEnd::Paragraph
                        },
                        available_inline: None,
                        indent: 0.0,
                        alignment: Alignment::Start,
                        tabs: TabContext::NoTabs,
                        hanging: HangingContext::None,
                        spacing: SpacingPolicy::None,
                    },
                    atomics: vec![],
                    controls: vec![],
                    line_strut: None,
                })
                .unwrap();
            lines.push(LinePlacement {
                line: selected.finalize(SpacingAllocations::None).unwrap(),
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 16.0 + index as f64 * 40.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            });
            start = end;
        }
        Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::WholeParagraph,
            lines,
        })
        .unwrap()
    }

    #[test]
    fn native_soft_line_deleted_end_keeps_original_affinity_side_eligibility() {
        for (text, mode, split, space_advance, upstream_x) in [
            ("A B", crate::WhiteSpace::Preserve, 2, 20.0, 40.0),
            ("A  B", crate::WhiteSpace::Collapse, 3, 0.0, 20.0),
        ] {
            let layout = native_whitespace_soft_lines(text, mode, split);
            assert_eq!(
                layout
                    .lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                [Range::new(0, split), Range::new(split, text.len())]
            );
            // The first Soft line applies the existing Collapse edge policy
            // before placing its actual Deleted anchor; Preserve keeps space.
            let first = &layout.inner.lines[0].finalized;
            let space = first
                .data
                .clusters
                .iter()
                .find(|cluster| cluster.source == Range::new(1, 2))
                .unwrap();
            assert_eq!((space.x, space.advance), (20.0, space_advance));
            if mode == crate::WhiteSpace::Collapse {
                let deleted = first
                    .data
                    .anchors
                    .iter()
                    .find(|anchor| {
                        matches!(
                            first.selected.presentation.units[anchor.unit].kind,
                            crate::protocol::prepared::PresentationUnitKind::Deleted
                        )
                    })
                    .unwrap();
                assert_eq!(deleted.slot, Point::new(20.0, 16.0));
            }
            assert_eq!(
                layout
                    .cursor(Cursor::new(split, Affinity::Before))
                    .unwrap()
                    .rect(),
                Rect::new(upstream_x, 0.0, 1.0, 20.0)
            );
            assert_eq!(
                layout
                    .cursor(Cursor::new(split, Affinity::After))
                    .unwrap()
                    .rect(),
                Rect::new(0.0, 40.0, 1.0, 20.0)
            );
        }
    }

    #[test]
    fn native_deleted_original_endpoints_keep_canonical_raw_affinities() {
        let layout = native_whitespace_line(" A ", crate::WhiteSpace::Discard);
        assert!(
            layout
                .inner
                .prepared
                .tables
                .units
                .iter()
                .any(|unit| unit.deleted && unit.source == Range::new(0, 1))
        );
        assert!(
            layout
                .inner
                .prepared
                .tables
                .units
                .iter()
                .any(|unit| unit.deleted && unit.source == Range::new(2, 3))
        );
        assert_eq!(
            layout
                .move_cursor(Cursor::new(0, Affinity::Before), Movement::DocumentStart)
                .unwrap(),
            Cursor::new(0, Affinity::After)
        );
        assert_eq!(
            layout
                .move_cursor(Cursor::new(3, Affinity::After), Movement::DocumentEnd)
                .unwrap(),
            Cursor::new(3, Affinity::Before)
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(0, Affinity::Before))
                .unwrap()
                .rect()
                .origin
                .x,
            0.0
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(3, Affinity::After))
                .unwrap()
                .rect()
                .origin
                .x,
            20.0
        );
    }

    #[test]
    fn native_deleted_start_next_word_keeps_the_original_whitespace_segment() {
        for mode in [crate::WhiteSpace::Preserve, crate::WhiteSpace::Collapse] {
            let layout = native_whitespace_line("A  B", mode);
            assert_eq!(
                layout.source().text().code_units(),
                [0x41, 0x20, 0x20, 0x42]
            );
            if mode == crate::WhiteSpace::Collapse {
                assert!(
                    layout
                        .inner
                        .prepared
                        .tables
                        .units
                        .iter()
                        .any(|unit| unit.deleted && unit.source == Range::new(2, 3))
                );
                let b_start = layout
                    .clusters()
                    .into_iter()
                    .find(|cluster| cluster.range() == Range::new(3, 4))
                    .unwrap()
                    .range()
                    .start;
                assert!(2 < b_start);
            }
            for affinity in [Affinity::After, Affinity::Before] {
                assert_eq!(
                    layout
                        .move_cursor(Cursor::new(2, affinity), Movement::NextWord)
                        .unwrap(),
                    Cursor::new(3, Affinity::Before)
                );
            }
        }
    }

    #[test]
    fn native_deleted_leading_previous_word_cannot_move_forward() {
        for mode in [crate::WhiteSpace::Preserve, crate::WhiteSpace::Discard] {
            let layout = native_whitespace_line("  B", mode);
            assert_eq!(layout.source().text().code_units(), [0x20, 0x20, 0x42]);
            if mode == crate::WhiteSpace::Discard {
                assert!(
                    layout
                        .inner
                        .prepared
                        .tables
                        .units
                        .iter()
                        .any(|unit| unit.deleted && unit.source == Range::new(0, 2))
                );
                assert_eq!(layout.clusters()[0].range(), Range::new(2, 3));
            }
            assert_eq!(
                layout
                    .move_cursor(Cursor::new(0, Affinity::After), Movement::PreviousWord)
                    .unwrap(),
                Cursor::new(0, Affinity::After)
            );
        }
    }
}
