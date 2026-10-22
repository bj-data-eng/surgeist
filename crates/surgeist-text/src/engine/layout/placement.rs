//! Checked outer placement of already finalized immutable lines.

use super::super::{FinalizedLine, PlacedLine, editing};
use super::{add, data::PlacedLayout, finite};
use crate::{Point, Result, TextNumericPhase};
use std::sync::Arc;

pub(super) fn place(finalized: Vec<FinalizedLine>) -> Result<PlacedLayout> {
    let prepared = Arc::clone(
        &finalized
            .first()
            .ok_or_else(|| {
                crate::Error::new(
                    crate::ErrorCode::LayoutFailed,
                    "standalone layout has no finalized line",
                )
            })?
            .prepared,
    );
    if finalized
        .iter()
        .any(|line| !Arc::ptr_eq(&prepared, &line.prepared))
    {
        return Err(crate::Error::new(
            crate::ErrorCode::LayoutFailed,
            "lines belong to different preparations",
        ));
    }
    let mut lines = Vec::with_capacity(finalized.len());
    let mut y: f64 = 0.0;
    let mut width: f64 = 0.0;
    let mut full_width: f64 = 0.0;
    for line in finalized {
        let metrics = line.data.metrics;
        let indent = line.selection.indent.max(0.0);
        width = width.max((metrics.advance + indent - metrics.trailing_whitespace).max(0.0));
        full_width = full_width.max((metrics.advance + indent).max(0.0));
        let origin = Point::new(0.0, if prepared.quantize { y.round() } else { y });
        add(
            "placed baseline",
            TextNumericPhase::Placement,
            origin.y,
            metrics.baseline,
        )?;
        add(
            "placed block start",
            TextNumericPhase::Placement,
            origin.y,
            metrics.block_min,
        )?;
        add(
            "placed block end",
            TextNumericPhase::Placement,
            origin.y,
            metrics.block_max,
        )?;
        y = add(
            "paragraph height",
            TextNumericPhase::Placement,
            y,
            metrics.line_height,
        )?;
        lines.push(PlacedLine {
            finalized: line,
            origin,
        });
    }
    finite("paragraph width", TextNumericPhase::Placement, width)?;
    finite(
        "paragraph full width",
        TextNumericPhase::Placement,
        full_width,
    )?;
    let interaction = editing::prepare(&prepared, &lines)?;
    Ok(PlacedLayout {
        prepared,
        lines,
        width,
        full_width,
        height: y,
        interaction,
    })
}
