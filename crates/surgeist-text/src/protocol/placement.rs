//! Admission of one explicitly anchored complete finalized-line set.

use std::{collections::BTreeSet, sync::Arc};

use super::{
    LineOrientation, LinePlacement, PlacementCoverage, PlacementEntry, PlacementRequest,
    PreparedBoundary, ProtocolContextField, ProtocolError, protocol_error,
};
use crate::engine::{PlacedLine, PreparedData, editing, layout::data::PlacedLayout};
use crate::{Layout, Point, Range, Rect, Result, TextNumericError, TextNumericPhase};

/// Original coverage plus its actual mixed-occurrence authority. A numeric
/// source interval alone cannot identify repeated source-less controls.
pub(crate) struct PlacedCoverage {
    pub(crate) start: PreparedBoundary,
    pub(crate) end: PreparedBoundary,
    pub(crate) source: Range,
}

impl PlacedCoverage {
    pub(crate) fn whole(prepared: &Arc<PreparedData>) -> Result<Self> {
        if prepared
            .tables
            .boundaries
            .first()
            .is_none_or(|row| row.participant != 0)
        {
            return Err(protocol_error(ProtocolError::EmptyPlacement));
        }
        let end = prepared
            .tables
            .boundaries
            .len()
            .checked_sub(1)
            .filter(|&index| {
                prepared.tables.boundaries[index].participant == prepared.tables.participants.len()
            })
            .ok_or_else(|| protocol_error(ProtocolError::EmptyPlacement))?;
        Ok(Self {
            start: PreparedBoundary {
                prepared: Arc::clone(prepared),
                index: 0,
            },
            end: PreparedBoundary {
                prepared: Arc::clone(prepared),
                index: end,
            },
            source: prepared
                .analyzed
                .paragraph
                .extent(&prepared.analyzed.processed),
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) enum PlacementMetricPolicy {
    Explicit,
    StandaloneStacked,
}

fn qualify_spacing_displacement(
    placed: &PlacedLine,
    before: crate::Point,
    after: crate::Point,
    half: f64,
) -> Result<()> {
    let before = crate::layout::inline_coordinate(placed.orientation, before);
    let after = crate::layout::inline_coordinate(placed.orientation, after);
    finite("placed spacing displacement", after)?;
    if half != 0.0 && before == after {
        return Err(crate::error::text_numeric_error(
            "placed spacing displacement",
            crate::TextNumericPhase::Placement,
            crate::TextNumericError::NonZeroLost,
            half,
        ));
    }
    Ok(())
}

/// A locally representable allocated shift can disappear at a physical anchor.
fn qualify_allocated_shift(
    placed: &PlacedLine,
    shift: crate::BaselineShift,
    inline: f64,
    local_baseline: f64,
) -> Result<()> {
    if shift.get() == 0.0 {
        return Ok(());
    }
    let baseline = placed.finalized.data.metrics.baseline;
    let before = crate::layout::line_point(placed, Point::new(inline, baseline + local_baseline));
    let after = crate::layout::line_point(
        placed,
        Point::new(inline, (baseline - shift.get()) + local_baseline),
    );
    let before = crate::layout::cross_coordinate(placed.orientation, before);
    let after = crate::layout::cross_coordinate(placed.orientation, after);
    finite("placed allocated baseline displacement", before)?;
    finite("placed allocated baseline displacement", after)?;
    if before == after {
        return Err(crate::error::text_numeric_error(
            "placed allocated baseline displacement",
            TextNumericPhase::Placement,
            TextNumericError::NonZeroLost,
            shift.get(),
        ));
    }
    Ok(())
}

impl Layout {
    /// Places these exact finalized lines once; no wrapping, font query, pixel
    /// snapping, or independent paragraph layout occurs here.
    pub fn from_finalized_lines(request: PlacementRequest) -> Result<Self> {
        let key = crate::Key::from_placement(&request);
        let placed = place_request(request, PlacementMetricPolicy::Explicit)?;
        let layout = Layout::from_placed(placed, key, crate::layout::LayoutBasis::Explicit);
        layout.qualify_decorations()?;
        Ok(layout)
    }
    /// Retained semantic completions with zero used line extent. These are not
    /// visible lines or caret targets; their original nominal metrics remain
    /// available from the genuine finalized receipt.
    pub fn nonformatted_lines(&self) -> impl Iterator<Item = &super::NonFormattedLinePlacement> {
        self.inner
            .nonformatted
            .iter()
            .map(|(_, placement)| placement)
    }
}

pub(crate) fn place_request(
    request: PlacementRequest,
    metric_policy: PlacementMetricPolicy,
) -> Result<PlacedLayout> {
    if request.entries.is_empty() {
        return Err(protocol_error(ProtocolError::EmptyPlacement));
    }
    let prepared = Arc::clone(&request.prepared.inner);
    let (start, end) = match &request.coverage {
        PlacementCoverage::WholeParagraph => {
            let coverage = PlacedCoverage::whole(&prepared)?;
            (coverage.start, coverage.end)
        }
        PlacementCoverage::Range { start, end } => (start.clone(), end.clone()),
    };
    for boundary in [&start, &end] {
        if !Arc::ptr_eq(&prepared, &boundary.prepared) {
            return Err(protocol_error(ProtocolError::ForeignPreparation {
                kind: super::ProtocolHandleKind::Boundary,
                occurrence: boundary.index,
            }));
        }
    }
    let start_index = start.index;
    let end_index = end.index;
    let row = |boundary: &PreparedBoundary| {
        prepared
            .tables
            .boundaries
            .get(boundary.index)
            .ok_or_else(|| {
                protocol_error(ProtocolError::InvalidBoundaryRange {
                    start: start_index,
                    end: end_index,
                    participants: prepared.tables.participants.len(),
                })
            })
    };
    let start_participant = row(&start)?.participant;
    let end_participant = row(&end)?.participant;
    if start_participant > end_participant {
        return Err(protocol_error(ProtocolError::InvalidBoundaryRange {
            start: start_index,
            end: end_index,
            participants: prepared.tables.participants.len(),
        }));
    }
    // Semantic line order need not be monotone in original source: a delayed
    // casing emission can follow a line which already owns a later input row.
    let source = request
        .entries
        .iter()
        .map(|entry| entry.finalized_line().inner.selected.presentation.source)
        .reduce(|a, b| Range::new(a.start.min(b.start), a.end.max(b.end)))
        .ok_or_else(|| protocol_error(ProtocolError::EmptyPlacement))?;
    let coverage = PlacedCoverage {
        start: start.clone(),
        end: end.clone(),
        source,
    };
    let mut next = start;
    let mut incoming = None;
    let mut seen = BTreeSet::new();
    let count = request.entries.len();
    let mut lines = Vec::with_capacity(count);
    let mut nonformatted = Vec::new();
    let mut receipts = Vec::with_capacity(count);
    let mut line_cuts = Vec::new();
    let mut relation_cut = 0usize;
    let mut bounds = None;
    let mut flow_bounds = None;
    let mut stacked_width = 0.0_f64;
    let mut stacked_full_width = 0.0_f64;
    let mut stacked_height = 0.0_f64;
    for (ordinal, entry) in request.entries.into_iter().enumerate() {
        let (placement, nonformatted_placement) = match entry {
            PlacementEntry::Formatted(placement) => (placement, None),
            PlacementEntry::NonFormatted(placement) => {
                let line = placement.line.finalized_line().clone();
                // The capability is opaque, but validate its semantic invariant
                // again at the single full-sequence placement front door.
                line.nonformatted()?;
                let numerical = LinePlacement {
                    line,
                    anchor: super::LineAnchor {
                        inline_origin: crate::layout::inline_coordinate(
                            placement.orientation,
                            placement.origin,
                        ),
                        baseline: crate::layout::cross_coordinate(
                            placement.orientation,
                            placement.origin,
                        ),
                    },
                    orientation: placement.orientation,
                    atomics: placement.atomics.clone(),
                };
                (numerical, Some(placement))
            }
        };
        let selected = &placement.line.inner.selected;
        if matches!(
            selected.request.termination,
            super::PreparedTermination::Measurement(_)
                | super::PreparedTermination::PhraseMeasurement(_)
        ) {
            return Err(protocol_error(ProtocolError::MeasurementNotPlaceable));
        }
        if !Arc::ptr_eq(&prepared, &selected.prepared) {
            return Err(protocol_error(ProtocolError::ForeignFinalizedLine {
                line: ordinal,
            }));
        }
        if !seen.insert(Arc::as_ptr(&placement.line.inner) as usize) {
            return Err(protocol_error(ProtocolError::DuplicateLine {
                line: ordinal,
            }));
        }
        if selected.request.start != next {
            return Err(protocol_error(ProtocolError::IncompleteLineCoverage {
                line: ordinal,
            }));
        }
        if (ordinal > 0 || matches!(request.coverage, PlacementCoverage::WholeParagraph))
            && selected.request.incoming != incoming
        {
            return Err(crate::hyphenation::error(
                crate::HyphenationError::InvalidContinuation,
            ));
        }
        incoming = placement.line.hyphenation_continuation();
        if ordinal + 1 == count
            && matches!(request.coverage, PlacementCoverage::WholeParagraph)
            && incoming.is_some()
        {
            return Err(crate::hyphenation::error(
                crate::HyphenationError::InvalidContinuation,
            ));
        }
        let from = row(&selected.request.start)?.participant;
        let to = row(&selected.request.end)?.participant;
        if to < from || to > end_participant || (to == from && ordinal + 1 != count) {
            return Err(protocol_error(ProtocolError::IncompleteLineCoverage {
                line: ordinal,
            }));
        }
        if selected.request.context.end == super::LineEnd::Paragraph && ordinal + 1 != count {
            return Err(protocol_error(ProtocolError::IncompleteLineCoverage {
                line: ordinal,
            }));
        }
        if ordinal + 1 == count
            && matches!(request.coverage, PlacementCoverage::WholeParagraph)
            && selected.request.context.end != super::LineEnd::Paragraph
        {
            return Err(protocol_error(ProtocolError::IncompleteLineCoverage {
                line: ordinal,
            }));
        }
        next = selected.request.end.clone();
        if placement.orientation == LineOrientation::Vertical
            && prepared.options.orientation() != LineOrientation::Vertical
        {
            return Err(protocol_error(ProtocolError::UnsupportedContext {
                field: ProtocolContextField::Orientation,
            }));
        }
        if placement.orientation != prepared.options.orientation()
            || placement.orientation != selected.request.context.orientation
        {
            return Err(protocol_error(ProtocolError::MismatchedContext {
                field: ProtocolContextField::Orientation,
            }));
        }
        finite("placed inline anchor", placement.anchor.inline_origin)?;
        finite("placed baseline anchor", placement.anchor.baseline)?;
        let placed = PlacedLine {
            anchor: crate::layout::physical_origin(
                placement.orientation,
                placement.anchor.inline_origin,
                placement.anchor.baseline,
            ),
            orientation: placement.orientation,
            finalized: Arc::clone(&placement.line.inner),
            atomics: placement.atomics.clone(),
        };
        validate_atomics(&prepared, ordinal, &placement, &placed)?;
        let cut = relation_cut;
        relation_cut += selected.presentation.selected_relation.events.len();
        receipts.push(Arc::clone(&placement.line.inner));
        if let Some(completion) = nonformatted_placement {
            nonformatted.push((ordinal, completion));
            continue;
        }
        line_cuts.push(cut);
        let metrics = placed.finalized.data.metrics;
        for item in &placed.finalized.data.boxes {
            let (own, local) = match item.geometry {
                crate::engine::layout::data::FinalizedBoxGeometry::InFlow {
                    alignment: super::LineBoxAlignment::Shift(shift),
                    box_geometry,
                    ..
                } => (shift.get(), box_geometry.baseline),
                _ => (0.0, 0.0),
            };
            let displacement = checked_add(
                "placed composed atomic frame",
                item.scope_displacement.get(),
                own,
            )?;
            qualify_allocated_shift(
                &placed,
                crate::BaselineShift::try_new(displacement)?,
                item.slot.x,
                local,
            )?;
        }
        for allocation in &selected.request.controls {
            if let Some(strut) = allocation.strut {
                let occurrence =
                    selected.prepared.tables.controls[allocation.occurrence.index].occurrence;
                let anchor=placed.finalized.data.anchors.iter().find(|anchor|matches!(
                    selected.presentation.units[anchor.unit].kind,super::prepared::PresentationUnitKind::Control{occurrence:actual} if actual==occurrence));
                let scope = anchor.map_or(0.0, |anchor| anchor.scope_displacement.get());
                let own = match strut.alignment {
                    super::LineBoxAlignment::Shift(shift) => shift.get(),
                    _ => 0.0,
                };
                let displacement = checked_add("placed composed control frame", scope, own)?;
                qualify_allocated_shift(
                    &placed,
                    crate::BaselineShift::try_new(displacement)?,
                    anchor.map_or(0.0, |anchor| anchor.slot.x),
                    0.0,
                )?;
            }
        }
        for (index, strut) in selected.request.line_box_struts.iter().enumerate() {
            if !matches!(
                strut.alignment,
                super::LineBoxAlignment::Top | super::LineBoxAlignment::Bottom
            ) {
                let owner = selected.scopes.strut_owners.get(index).copied().flatten();
                let scope = owner.map_or(0.0, |owner| {
                    placed.finalized.data.scope_allocations[owner]
                        .cumulative_displacement
                        .get()
                });
                let own = match strut.alignment {
                    super::LineBoxAlignment::Shift(shift) => shift.get(),
                    _ => 0.0,
                };
                let displacement = checked_add("placed composed box-strut frame", scope, own)?;
                qualify_allocated_shift(
                    &placed,
                    crate::BaselineShift::try_new(displacement)?,
                    0.0,
                    0.0,
                )?;
            }
        }
        // Glyph-outline axes are independent from the line's cell frame.
        // Qualify every actual emitted origin before publishing placement.
        for run in &placed.finalized.data.runs {
            if let Some(reference) = run.reference {
                qualify_allocated_shift(
                    &placed,
                    crate::BaselineShift::try_new(metrics.baseline - reference)?,
                    run.x,
                    0.0,
                )?;
            }
            let shaped = &placed.finalized.data.font_runs[run.run];
            let vector = crate::layout::line_vector(
                &placed,
                crate::Point::new(run.x, run.y - metrics.baseline),
            );
            // Finalization qualifies the local font displacement. Translation
            // can lose it again at a finite physical anchor, so check the
            // required cross-axis contribution in this owning phase as well.
            let origin = match placed.orientation {
                LineOrientation::Horizontal => crate::Point::new(
                    placed.anchor.x + vector.x,
                    checked_add("placed run origin y", placed.anchor.y, vector.y)?,
                ),
                LineOrientation::SidewaysClockwise
                | LineOrientation::SidewaysCounterclockwise
                | LineOrientation::Vertical => crate::Point::new(
                    checked_add("placed run origin x", placed.anchor.x, vector.x)?,
                    placed.anchor.y + vector.y,
                ),
            };
            finite("placed run origin x", origin.x)?;
            finite("placed run origin y", origin.y)?;
            for glyph in &placed.finalized.data.glyphs[run.glyphs.clone()] {
                let point = crate::layout::glyph_origin(
                    placed.orientation,
                    shaped.glyph_basis,
                    origin,
                    glyph.pen,
                    f64::from(glyph.glyph.x),
                    f64::from(glyph.glyph.y),
                );
                finite("placed glyph origin x", point.x)?;
                finite("placed glyph origin y", point.y)?;
                if glyph.word_half != 0.0 {
                    let before = crate::layout::glyph_origin(
                        placed.orientation,
                        shaped.glyph_basis,
                        origin,
                        glyph.pen_base,
                        f64::from(glyph.glyph.x),
                        f64::from(glyph.glyph.y),
                    );
                    qualify_spacing_displacement(&placed, before, point, glyph.word_half)?;
                }
            }
        }
        for unit in placed
            .finalized
            .data
            .word_anchors
            .iter()
            .chain(&placed.finalized.data.character_anchors)
        {
            let before = crate::layout::line_point(&placed, unit.base);
            let after = crate::layout::line_point(
                &placed,
                crate::Point::new(unit.base.x + unit.half, unit.base.y),
            );
            qualify_spacing_displacement(&placed, before, after, unit.half)?;
        }
        let metric_rect = crate::layout::line_rect(
            &placed,
            Rect::new(
                metrics.inline_min,
                metrics.block_min,
                metrics.inline_max - metrics.inline_min,
                metrics.block_max - metrics.block_min,
            ),
        );
        include(&mut bounds, metric_rect)?;
        for anchor in &placed.finalized.data.anchors {
            if !matches!(
                selected.presentation.units[anchor.unit].kind,
                super::prepared::PresentationUnitKind::Control { .. }
            ) {
                qualify_allocated_shift(&placed, anchor.scope_displacement, anchor.slot.x, 0.0)?;
            }
            if anchor.reference.is_some() {
                let rect = placed.finalized.data.cell_rect(
                    anchor.slot.x,
                    anchor.inline_end - anchor.slot.x,
                    anchor.reference,
                );
                // Source-only anchors still participate in the placed geometry
                // qualification without creating another metric strut.
                let rect = crate::layout::line_rect(&placed, rect);
                finite("placed scope anchor x", rect.origin.x)?;
                finite("placed scope anchor y", rect.origin.y)?;
                finite("placed scope anchor right", rect.origin.x + rect.size.width)?;
                finite(
                    "placed scope anchor bottom",
                    rect.origin.y + rect.size.height,
                )?;
            }
        }
        for cluster in &placed.finalized.data.clusters {
            let x0 = cluster.x.min(cluster.x + cluster.advance);
            let width = cluster.advance.abs();
            include(
                &mut bounds,
                crate::layout::line_rect(
                    &placed,
                    placed
                        .finalized
                        .data
                        .cell_rect(x0, width, cluster.reference),
                ),
            )?;
        }
        for atomic in &placed.atomics {
            include(&mut bounds, atomic.border)?;
        }
        let indent = selected.request.context.indent.max(0.0);
        let visible = (placed
            .finalized
            .data
            .measurements
            .as_ref()
            .expect("finalization installed actual measurements")
            .alignment()
            .advance()
            + indent)
            .max(0.0);
        let full = (metrics.advance + indent).max(0.0);
        finite("placed visible advance", visible)?;
        finite("placed full advance", full)?;
        include(
            &mut flow_bounds,
            crate::layout::line_rect(&placed, Rect::new(0.0, 0.0, visible, metrics.line_height)),
        )?;
        stacked_width = stacked_width.max(visible);
        stacked_full_width = stacked_full_width.max(full);
        stacked_height = checked_add(
            "stacked paragraph height",
            stacked_height,
            metrics.line_height,
        )?;
        lines.push(placed);
    }
    if next != end {
        return Err(protocol_error(ProtocolError::IncompleteLineCoverage {
            line: count,
        }));
    }
    if matches!(request.coverage, PlacementCoverage::WholeParagraph)
        && (source
            != prepared
                .analyzed
                .paragraph
                .extent(&prepared.analyzed.processed))
    {
        return Err(protocol_error(ProtocolError::IncompleteLineCoverage {
            line: count,
        }));
    }
    // A complete all-nonformatted sequence has semantic coverage, but no
    // visible allocation and therefore no fabricated strut rectangle/caret.
    let bounds = bounds.unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));
    let flow = flow_bounds.unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));
    let (width, full_width, height) = match metric_policy {
        PlacementMetricPolicy::StandaloneStacked => {
            (stacked_width, stacked_full_width, stacked_height)
        }
        PlacementMetricPolicy::Explicit => (flow.size.width, flow.size.width, flow.size.height),
    };
    let interaction = editing::prepare(&prepared, &lines, &line_cuts)?;
    let placed = PlacedLayout {
        prepared,
        lines,
        receipts,
        line_cuts,
        nonformatted,
        bounds,
        coverage,
        width,
        full_width,
        height,
        interaction,
    };
    for index in 0..placed.interaction.positions.len() {
        qualify_rect(editing::position::geometry(&placed, index)?.rect())?;
    }
    Ok(placed)
}

fn validate_atomics(
    prepared: &Arc<PreparedData>,
    line: usize,
    placement: &LinePlacement,
    placed: &PlacedLine,
) -> Result<()> {
    let expected = &placed.finalized.data.boxes;
    let mut seen = BTreeSet::new();
    for actual in &placement.atomics {
        if !Arc::ptr_eq(prepared, &actual.occurrence.prepared) {
            return Err(protocol_error(ProtocolError::ForeignPreparation {
                kind: super::ProtocolHandleKind::Atomic,
                occurrence: actual.occurrence.index,
            }));
        }
        if !seen.insert(actual.occurrence.index) {
            return Err(protocol_error(ProtocolError::DuplicateAtomic {
                occurrence: actual.occurrence.index,
            }));
        }
        let item = expected
            .iter()
            .find(|item| item.atomic == actual.occurrence)
            .ok_or_else(|| {
                protocol_error(ProtocolError::UnexpectedAtomic {
                    occurrence: actual.occurrence.index,
                })
            })?;
        qualify_rect(actual.border)?;
        qualify_rect(actual.content)?;
        finite("placed atomic baseline", actual.baseline)?;
        if let crate::engine::layout::data::FinalizedBoxGeometry::InFlow {
            box_geometry,
            rect,
            ..
        } = item.geometry
        {
            let border = crate::layout::line_rect(placed, rect);
            let content = crate::layout::line_rect(
                placed,
                Rect::new(
                    item.slot.x + box_geometry.content.origin.x,
                    item.slot.y + box_geometry.content.origin.y,
                    box_geometry.content.size.width,
                    box_geometry.content.size.height,
                ),
            );
            let baseline = crate::layout::cross_coordinate(
                placed.orientation,
                crate::layout::line_point(
                    placed,
                    Point::new(item.slot.x, item.slot.y + box_geometry.baseline),
                ),
            );
            qualify_rect(border)?;
            qualify_rect(content)?;
            finite("placed atomic baseline", baseline)?;
            if border != actual.border || content != actual.content || baseline != actual.baseline {
                return Err(protocol_error(ProtocolError::AtomicPlacementMismatch {
                    line,
                    occurrence: actual.occurrence.index,
                }));
            }
        }
    }
    for item in expected {
        if !seen.contains(&item.atomic.index) {
            return Err(protocol_error(ProtocolError::MissingAtomic {
                occurrence: item.atomic.index,
            }));
        }
    }
    Ok(())
}

fn include(bounds: &mut Option<Rect>, rect: Rect) -> Result<()> {
    qualify_rect(rect)?;
    *bounds = Some(if let Some(old) = *bounds {
        let x = old.origin.x.min(rect.origin.x);
        let y = old.origin.y.min(rect.origin.y);
        let right = (old.origin.x + old.size.width).max(rect.origin.x + rect.size.width);
        let bottom = (old.origin.y + old.size.height).max(rect.origin.y + rect.size.height);
        let union = Rect::new(x, y, right - x, bottom - y);
        qualify_rect(union)?;
        union
    } else {
        rect
    });
    Ok(())
}
fn qualify_rect(rect: Rect) -> Result<()> {
    let right = rect.origin.x + rect.size.width;
    let bottom = rect.origin.y + rect.size.height;
    for (field, value) in [
        ("placed x", rect.origin.x),
        ("placed y", rect.origin.y),
        ("placed width", rect.size.width),
        ("placed height", rect.size.height),
        ("placed right", right),
        ("placed bottom", bottom),
    ] {
        finite(field, value)?;
    }
    if rect.size.width < 0.0 || rect.size.height < 0.0 {
        return Err(numeric(
            "placed rectangle extent",
            TextNumericError::OutOfRange,
            rect.size.width.min(rect.size.height),
        ));
    }
    if rect.size.width > 0.0 && right <= rect.origin.x {
        return Err(numeric(
            "placed width",
            TextNumericError::PositiveLost,
            rect.size.width,
        ));
    }
    if rect.size.height > 0.0 && bottom <= rect.origin.y {
        return Err(numeric(
            "placed height",
            TextNumericError::PositiveLost,
            rect.size.height,
        ));
    }
    Ok(())
}
fn checked_add(field: &'static str, base: f64, delta: f64) -> Result<f64> {
    let value = base + delta;
    finite(field, value)?;
    if delta != 0.0 && value == base {
        return Err(numeric(field, TextNumericError::NonZeroLost, delta));
    }
    Ok(value)
}
fn finite(field: &'static str, value: f64) -> Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(numeric(field, TextNumericError::NonFinite, value))
    }
}
fn numeric(field: &'static str, category: TextNumericError, value: f64) -> crate::Error {
    crate::error::text_numeric_error(field, TextNumericPhase::Placement, category, value)
}
