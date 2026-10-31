//! One font-backed logical-share allocator for line and Cell semantic units.
use super::*;
use crate::engine::layout::data::FinalizedCluster;
use crate::protocol::prepared::SelectedPresentation;

pub(super) struct MetricUnit {
    pub(super) atoms: std::ops::Range<usize>,
    pub(super) presentation: Range,
}
pub(super) struct MetricUnitIndex {
    pub(super) unit: usize,
    pub(super) atoms: std::ops::Range<usize>,
    pub(super) presentation: Range,
}
#[derive(Clone, Copy)]
pub(super) enum MetricUnits<'a> {
    Line(&'a SelectedPresentation),
    Cell(&'a [MetricUnit], &'a [MetricUnitIndex]),
}
impl MetricUnits<'_> {
    pub(super) fn atoms(self, index: usize) -> std::ops::Range<usize> {
        match self {
            Self::Line(value) => value.analysis.graphemes[index].clone(),
            Self::Cell(value, _) => value[index].atoms.clone(),
        }
    }
    pub(super) fn visit_provider_units(
        self,
        provider: &std::ops::Range<usize>,
        ledger: &mut Ledger,
        mut visit: impl FnMut(usize, &mut Ledger) -> Result<()>,
    ) -> Result<()> {
        match self {
            Self::Line(value) => {
                let first =
                    operations::partition_point(&value.analysis.graphemes, ledger.as_mut(), |g| {
                        g.end <= provider.start
                    })?;
                let end =
                    operations::partition_point(&value.analysis.graphemes, ledger.as_mut(), |g| {
                        g.start < provider.end
                    })?;
                for unit in first..end {
                    account::work(ledger, 1)?;
                    visit(unit, ledger)?;
                }
            }
            Self::Cell(_, index) => {
                let first = operations::partition_point(index, ledger.as_mut(), |g| {
                    g.atoms.end <= provider.start
                })?;
                let end = operations::partition_point(index, ledger.as_mut(), |g| {
                    g.atoms.start < provider.end
                })?;
                for row in &index[first..end] {
                    account::work(ledger, 1)?;
                    visit(row.unit, ledger)?;
                }
            }
        }
        Ok(())
    }
    fn at_presentation(self, position: usize, ledger: &mut Ledger) -> Result<Option<usize>> {
        Ok(match self {
            Self::Line(value) => {
                let index =
                    operations::partition_point(&value.analysis.graphemes, ledger.as_mut(), |g| {
                        value.projection.atoms[g.end - 1].presentation.end <= position
                    })?;
                (index < value.analysis.graphemes.len()).then_some(index)
            }
            Self::Cell(_, index) => {
                let at = operations::partition_point(index, ledger.as_mut(), |g| {
                    g.presentation.end <= position
                })?;
                index.get(at).map(|row| row.unit)
            }
        })
    }
}
struct ProviderShares {
    first: usize,
    units: Vec<usize>,
    edges: Vec<(f64, f64)>,
}
#[derive(Default)]
pub(super) struct GeometryCache {
    rows: Vec<ProviderShares>,
}
impl GeometryCache {
    pub(super) fn drop_scratch(mut self, ledger: &mut Ledger) -> Result<()> {
        for row in &mut self.rows {
            account::work(ledger, 1)?;
            account::drop_vec(ledger, std::mem::take(&mut row.units))?;
            account::drop_vec(ledger, std::mem::take(&mut row.edges))?;
        }
        account::drop_vec(ledger, self.rows)
    }
}
pub(super) fn range_geometry(
    clusters: &[FinalizedCluster],
    units: MetricUnits<'_>,
    group: usize,
    providers: &[std::ops::Range<usize>],
    cached: &mut GeometryCache,
    ledger: &mut Ledger,
    unsupported: impl Fn() -> Error,
) -> Result<Vec<MetricPiece>> {
    let mut result = Vec::new();
    for indices in providers {
        account::work(ledger, 1)?;
        let first = indices.start;
        let index = indices.end;
        let provider = &clusters[first].provider_atoms;
        let location = operations::binary_search_by(&cached.rows, ledger.as_mut(), |row| {
            row.first.cmp(&first)
        })?;
        if let Ok(at) = location {
            let row = &cached.rows[at];
            let ordinal =
                operations::binary_search_by(&row.units, ledger.as_mut(), |unit| unit.cmp(&group))?
                    .map_err(|_| unsupported())?;
            let (start, end) = row.edges[ordinal];
            account::push(
                ledger,
                &mut result,
                MetricPiece {
                    clusters: indices.clone(),
                    start,
                    end,
                },
            )?;
            continue;
        }
        let mut recipients = Vec::new();
        units.visit_provider_units(provider, ledger, |unit, ledger| {
            let atoms = units.atoms(unit);
            if !atoms.is_empty() && atoms.end > provider.start && atoms.start < provider.end {
                account::push(ledger, &mut recipients, unit)?;
            }
            Ok(())
        })?;
        operations::order_by(&mut recipients, ledger.as_mut(), Ord::cmp)?;
        dedup_sorted(&mut recipients, ledger)?;
        let Some(&begin) = recipients.first() else {
            return Err(unsupported());
        };
        let end = *recipients.last().expect("nonempty recipients");
        if recipients.len() == 1 {
            for (offset, cluster) in clusters[first..index].iter().enumerate() {
                account::work(ledger, 1)?;
                if cluster.disposition != SpacingSlotDisposition::DiscardedAtLineEdge {
                    account::push(
                        ledger,
                        &mut result,
                        MetricPiece {
                            clusters: first + offset..first + offset + 1,
                            start: cluster.x,
                            end: add("source range local advance end", cluster.x, cluster.advance)?,
                        },
                    )?;
                }
            }
            account::drop_vec(ledger, recipients)?;
            continue;
        }
        // Only real contributing units receive provider shares. A retained zero
        // semantic SPACE has no provider projection and cannot borrow a share.
        if units.atoms(begin).start != provider.start || units.atoms(end).end != provider.end {
            return Err(unsupported());
        }
        let rtl = !clusters[first].level.is_multiple_of(2);
        let mut previous = None;
        let mut raw = 0.0;
        for cluster in &clusters[first..index] {
            account::work(ledger, 1)?;
            if cluster.level != clusters[first].level
                || cluster.advance < 0.0
                || cluster.disposition == SpacingSlotDisposition::DiscardedAtLineEdge
            {
                return Err(unsupported());
            }
            let edge = add("source range provider edge", cluster.x, cluster.advance)?;
            if previous.is_some_and(|(x, right)| if rtl { edge > x } else { cluster.x < right }) {
                return Err(unsupported());
            }
            previous = Some((cluster.x, edge));
            raw = add("source range shaped advance", raw, cluster.shaped_advance)?;
        }
        let count = recipients.len();
        let share = raw / count as f64;
        let mut adjustments = account::filled(ledger, count, 0.0)?;
        for cluster in &clusters[first..index] {
            account::work(ledger, 1)?;
            let egc = units
                .at_presentation(cluster.presentation.start, ledger)?
                .ok_or_else(&unsupported)?;
            let at =
                operations::binary_search_by(&recipients, ledger.as_mut(), |unit| unit.cmp(&egc))?
                    .map_err(|_| unsupported())?;
            for value in cluster.logical_adjustments {
                account::work(ledger, 1)?;
                adjustments[at] = add("source range owned adjustment", adjustments[at], value)?;
            }
        }
        let right = add(
            "source range provider right",
            clusters[first].x,
            clusters[first].advance,
        )?;
        let mut offset = 0.0;
        let mut edges = Vec::new();
        for (relative, &adjustment) in adjustments.iter().enumerate() {
            account::work(ledger, 1)?;
            let raw_part = if relative + 1 == count {
                raw - share * relative as f64
            } else {
                share
            };
            let width = add("source range adjusted share", raw_part, adjustment)?;
            if width < 0.0 {
                return Err(unsupported());
            }
            let pair = if rtl {
                let end = add("source range logical share end", right, -offset)?;
                (add("source range logical share start", end, -width)?, end)
            } else {
                let start = add(
                    "source range logical share start",
                    clusters[first].x,
                    offset,
                )?;
                (start, add("source range logical share end", start, width)?)
            };
            account::push(ledger, &mut edges, pair)?;
            offset = add("source range preceding shares", offset, width)?;
        }
        account::drop_vec(ledger, adjustments)?;
        let ordinal =
            operations::binary_search_by(&recipients, ledger.as_mut(), |unit| unit.cmp(&group))?
                .map_err(|_| unsupported())?;
        let (start, end) = edges[ordinal];
        let at = location.expect_err("absent cache row");
        account::work(ledger, cached.rows.len() - at)?;
        account::push(
            ledger,
            &mut cached.rows,
            ProviderShares {
                first,
                units: recipients,
                edges,
            },
        )?;
        cached.rows[at..].rotate_right(1);
        account::push(
            ledger,
            &mut result,
            MetricPiece {
                clusters: first..index,
                start,
                end,
            },
        )?;
    }
    Ok(result)
}
