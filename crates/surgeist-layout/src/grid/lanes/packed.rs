use super::*;

#[derive(Debug)]
pub(in crate::grid) struct PackedLaneItem<S: LayoutScalar> {
    pub(in crate::grid) source_index: usize,
    pub(in crate::grid) grid_axis_start: usize,
    pub(in crate::grid) grid_axis_span: usize,
    pub(in crate::grid) offset: S,
    pub(in crate::grid) lane_axis_margin_box: S,
    pub(in crate::grid) alignment_extent: S,
}

#[derive(Debug)]
pub(in crate::grid) struct PackedLaneTrack {
    pub(in crate::grid) first: usize,
    pub(in crate::grid) last: usize,
}

#[derive(Debug)]
pub(in crate::grid) struct PackedLanePlacement<S: LayoutScalar> {
    pub(in crate::grid) grid_axis: GridAxisKind,
    pub(in crate::grid) lane_axis: GridAxisKind,
    pub(in crate::grid) items: Vec<PackedLaneItem<S>>,
    pub(in crate::grid) tracks: Vec<Option<PackedLaneTrack>>,
    pub(in crate::grid) range_start: S,
    pub(in crate::grid) range_end: S,
}

impl<S: LayoutScalar> PackedLanePlacement<S> {
    pub(in crate::grid) fn new(
        grid_axis: GridAxisKind,
        lane_axis: GridAxisKind,
        mut items: Vec<PackedLaneItem<S>>,
        track_count: usize,
        gap: S,
    ) -> Self {
        let mut order = (0..items.len()).collect::<Vec<_>>();
        // Stable ties retain order-modified placement order, including zero extents.
        order.sort_by(|&left, &right| {
            items[left]
                .offset
                .partial_cmp(&items[right].offset)
                .expect("packed stacking positions are finite")
        });
        let mut tracks = (0..track_count).map(|_| None).collect::<Vec<_>>();
        let mut alignment_ends: Vec<Option<S>> = vec![None; items.len()];
        for index in order {
            let placement = &items[index];
            let start = placement.grid_axis_start - 1;
            let end = start + placement.grid_axis_span;
            for track in &mut tracks[start..end] {
                if let Some(PackedLaneTrack { last, .. }) = track {
                    let previous = &items[*last];
                    let end = if placement.grid_axis_span > 1 {
                        placement.offset - gap
                    } else {
                        previous.offset + previous.lane_axis_margin_box
                    };
                    alignment_ends[*last] =
                        Some(alignment_ends[*last].map_or(end, |old| old.min(end)));
                    *last = index;
                } else {
                    *track = Some(PackedLaneTrack {
                        first: index,
                        last: index,
                    });
                }
            }
        }
        let range_start = tracks
            .iter()
            .flatten()
            .map(|track| items[track.first].offset)
            .reduce(S::min)
            .unwrap_or(S::ZERO);
        let range_end = tracks
            .iter()
            .flatten()
            .map(|track| {
                let item = &items[track.last];
                item.offset + item.lane_axis_margin_box
            })
            .reduce(S::max)
            .unwrap_or(S::ZERO);
        for track in tracks.iter().flatten() {
            let end = &mut alignment_ends[track.last];
            *end = Some(end.map_or(range_end, |old| old.min(range_end)));
        }
        for (item, end) in items.iter_mut().zip(alignment_ends) {
            let outer_end = item.offset + item.lane_axis_margin_box;
            let slack = (end.expect("each packed item occupies a track") - outer_end).max(S::ZERO);
            // Signed margin-box extent is preserved; only adjacent slack is nonnegative.
            item.alignment_extent = item.lane_axis_margin_box + slack;
        }
        Self {
            grid_axis,
            lane_axis,
            items,
            tracks,
            range_start,
            range_end,
        }
    }

    pub(in crate::grid) fn range_extent(&self) -> S {
        self.range_end - self.range_start
    }

    pub(in crate::grid) fn item(&self, source_index: usize) -> Option<&PackedLaneItem<S>> {
        self.items
            .iter()
            .find(|item| item.source_index == source_index)
    }

    pub(in crate::grid) fn replace_stacking_baselines<Node>(
        &self,
        items: &[PendingGridItem<'_, Node, S>],
        flow: crate::geometry::FlowAxes,
        baselines: &mut BaselinesOf<S>,
    ) {
        let axis = grid_axis_physical_axis(flow, self.lane_axis);
        let decreasing = flow.physical_axis_progression(axis).is_decreasing();
        let boundary = |first: bool| {
            self.tracks
                .iter()
                .flatten()
                .filter_map(|track| {
                    let packed = &self.items[if first { track.first } else { track.last }];
                    let item = items
                        .iter()
                        .find(|item| item.source_index == packed.source_index)?;
                    // Missing ports are synthesized from this boundary item's
                    // border edges in an axis-compatible writing mode.
                    let synthesis_flow = if item.child_flow_axes.block_axis() == axis {
                        item.child_flow_axes
                    } else if flow.block_axis() == axis {
                        flow
                    } else {
                        let writing_mode = match (axis, item.style.direction) {
                            (PhysicalAxis::Vertical, _) => crate::WritingMode::HorizontalTb,
                            (PhysicalAxis::Horizontal, Direction::Ltr) => {
                                crate::WritingMode::VerticalLr
                            }
                            (PhysicalAxis::Horizontal, Direction::Rtl) => {
                                crate::WritingMode::VerticalRl
                            }
                        };
                        crate::geometry::FlowAxes::new(writing_mode, item.style.direction)
                    };
                    let output = item.output.baselines();
                    let port = if first {
                        output.first_or_synthesize_block_baseline(synthesis_flow, item.output.size)
                    } else {
                        output.last_or_synthesize_block_baseline(synthesis_flow, item.output.size)
                    };
                    let point = port.translated(item.location);
                    match axis {
                        PhysicalAxis::Horizontal => point.x,
                        PhysicalAxis::Vertical => point.y,
                    }
                })
                .reduce(|left, right| {
                    if first != decreasing {
                        left.min(right)
                    } else {
                        left.max(right)
                    }
                })
        };
        let point = |coordinate| match axis {
            PhysicalAxis::Horizontal => Point::new(coordinate, None),
            PhysicalAxis::Vertical => Point::new(None, coordinate),
        };
        baselines.replace_first_axis(point(boundary(true)));
        baselines.replace_last_axis(point(boundary(false)));
    }
}
