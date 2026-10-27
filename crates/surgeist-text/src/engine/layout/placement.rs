//! Standalone policy produces ordinary requests for the one placement validator.

use std::sync::Arc;

use super::super::FinalizedLine;
use super::{add, data::PlacedLayout};
use crate::protocol::{
    AtomicPlacement, FinalizedLine as PublicFinalizedLine, LineAnchor, LineOrientation,
    LinePlacement, PlacementCoverage, PlacementEntry, PlacementRequest, PreparedText,
    placement::{PlacementMetricPolicy, place_request},
    selection::LineMetricPolicy,
};
use crate::{Point, Result, TextNumericPhase};

pub(super) fn place(finalized: Vec<Arc<FinalizedLine>>) -> Result<PlacedLayout> {
    let prepared = Arc::clone(
        &finalized
            .first()
            .ok_or_else(|| {
                crate::Error::new(
                    crate::ErrorCode::LayoutFailed,
                    "standalone layout has no finalized line",
                )
            })?
            .selected
            .prepared,
    );
    let mut lines = Vec::with_capacity(finalized.len());
    let mut y = 0.0_f64;
    for line in finalized {
        let metrics = line.data.metrics;
        let top = if line.selected.metric_policy == LineMetricPolicy::StandaloneQuantized {
            y.round()
        } else {
            y
        };
        let anchor = Point::new(
            0.0,
            add(
                "placed baseline",
                TextNumericPhase::Placement,
                top,
                metrics.baseline,
            )?,
        );
        let placed = crate::engine::PlacedLine {
            finalized: Arc::clone(&line),
            anchor,
            orientation: LineOrientation::Horizontal,
            atomics: Vec::new(),
        };
        let atomics = line
            .data
            .boxes
            .iter()
            .map(|item| {
                let (border, content, baseline) = match item.geometry {
                    super::data::FinalizedBoxGeometry::InFlow {
                        box_geometry, rect, ..
                    } => (
                        rect,
                        crate::Rect::new(
                            item.slot.x + box_geometry.content.origin.x,
                            item.slot.y + box_geometry.content.origin.y,
                            box_geometry.content.size.width,
                            box_geometry.content.size.height,
                        ),
                        item.slot.y + box_geometry.baseline,
                    ),
                    super::data::FinalizedBoxGeometry::OutOfFlow => {
                        // Only standalone placement owns these authored physical dimensions.
                        let size =
                            prepared.analyzed.processed.source().boxes()[item.occurrence].size();
                        let rect = crate::Rect::new(
                            item.slot.x,
                            item.slot.y - size.height,
                            size.width,
                            size.height,
                        );
                        (rect, rect, item.slot.y)
                    }
                };
                AtomicPlacement {
                    occurrence: item.atomic.clone(),
                    border: crate::layout::line_rect(&placed, border),
                    content: crate::layout::line_rect(&placed, content),
                    baseline: crate::layout::line_point(&placed, Point::new(item.slot.x, baseline))
                        .y,
                }
            })
            .collect();
        lines.push(PlacementEntry::Formatted(LinePlacement {
            line: PublicFinalizedLine { inner: line },
            anchor: LineAnchor {
                inline_origin: anchor.x,
                baseline: anchor.y,
            },
            orientation: LineOrientation::Horizontal,
            atomics,
        }));
        y = add(
            "paragraph height",
            TextNumericPhase::Placement,
            y,
            metrics.line_height,
        )?;
    }
    place_request(
        PlacementRequest {
            prepared: PreparedText { inner: prepared },
            coverage: PlacementCoverage::WholeParagraph,
            entries: lines,
        },
        PlacementMetricPolicy::StandaloneStacked,
    )
}
