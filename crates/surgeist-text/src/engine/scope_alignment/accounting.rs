//! Operation-local allocation accounting for scope finalization and replay.
//! Native input resources stay with their retained owner. Every new buffer is
//! reserved before mutation; dropped scratch releases bytes, never record work.
use crate::engine::layout::data::{CellPayload, OrdinalMap, PreparedCell, PreparedRun, ShapedData};
use crate::processing::contextual::alignment::ScopeAlignmentBudget;
use crate::{Result, ScopeAlignmentError, ScopeAlignmentLimitKind};

pub(crate) type Ledger = Option<ScopeAlignmentBudget>;

pub(crate) fn bytes<T>(values: &Vec<T>) -> Result<usize> {
    values
        .capacity()
        .checked_mul(std::mem::size_of::<T>())
        .ok_or_else(|| {
            super::refusal(ScopeAlignmentError::Overflow {
                kind: ScopeAlignmentLimitKind::PayloadBytes,
            })
        })
}
pub(crate) fn work(ledger: &mut Ledger, count: usize) -> Result<()> {
    if let Some(budget) = ledger {
        budget.work(count)?;
    }
    Ok(())
}
pub(crate) fn header<T>(ledger: &mut Ledger) -> Result<()> {
    if let Some(budget) = ledger {
        budget.retain_bytes(std::mem::size_of::<T>())?;
    }
    Ok(())
}
/// The Cell-created outer item arena has one persistent header and one actual
/// shared backing. Prepared body payloads otherwise retain native ownership.
pub(crate) fn visit_cell_items(
    shaped: &ShapedData,
    visit: &mut super::census::AllocationVisitor<'_>,
) -> Result<()> {
    use super::census::AllocationWitness as W;
    visit(W::object(&shaped.items, 0))?;
    visit(W::buffer(&shaped.items, shaped.items.len())?)
}

/// Canonical selected Cell ownership inventory. Once Cell copies enter these
/// arenas, each whole shared allocation is owned, including ordinary rows that
/// already occupy its capacity. Nested coordinates have embedded headers.
pub(crate) fn visit_cell_arenas(
    shaped: &ShapedData,
    visit: &mut super::census::AllocationVisitor<'_>,
) -> Result<()> {
    use super::census::AllocationWitness as W;
    visit(W::object(&shaped.runs, 0))?;
    visit(W::buffer(&shaped.runs, shaped.runs.len())?)?;
    for run in &shaped.runs {
        visit(W::buffer(&run.coords, run.coords.len())?)?;
    }
    visit(W::object(&shaped.clusters, 0))?;
    visit(W::buffer(&shaped.clusters, shaped.clusters.len())?)?;
    visit(W::object(&shaped.glyphs, 0))?;
    visit(W::buffer(&shaped.glyphs, shaped.glyphs.len())?)?;
    visit(W::object(&shaped.provider_groups, 0))?;
    visit(W::buffer(
        &shaped.provider_groups,
        shaped.provider_groups.len(),
    )?)?;
    visit_cell_items(shaped, visit)
}

/// Transfer the actual existing selected shared arenas to Cell ownership once,
/// before their first Cell copy or growth. Subsequent mutation uses reserve/push
/// on this same ledger; no capacity that preceded adoption is debited unfunded.
pub(crate) fn adopt_cell_arenas(ledger: &mut Ledger, shaped: &ShapedData) -> Result<()> {
    let Some(budget) = ledger else { return Ok(()) };
    visit_cell_arenas(shaped, &mut |allocation| {
        let inspections = allocation.records().checked_add(1).ok_or_else(|| {
            super::refusal(ScopeAlignmentError::Overflow {
                kind: ScopeAlignmentLimitKind::StructuralWork,
            })
        })?;
        budget.work(inspections)?;
        budget.add_records(allocation.records())?;
        budget.retain_bytes(allocation.payload_bytes())
    })
}

pub(crate) fn reserve_vec<T>(
    ledger: &mut Ledger,
    values: &mut Vec<T>,
    additional: usize,
) -> Result<()> {
    if let Some(budget) = ledger {
        let required = values.len().checked_add(additional).ok_or_else(|| {
            super::refusal(ScopeAlignmentError::Overflow {
                kind: ScopeAlignmentLimitKind::PayloadBytes,
            })
        })?;
        if required > values.capacity() {
            // This is the real requested arena capacity, including its actual
            // relocation overlap, rather than a hypothetical allocation size.
            let target = values
                .capacity()
                .checked_mul(2)
                .unwrap_or(required)
                .max(required);
            budget.reserve(values, target - values.len())?;
        }
    } else {
        values.reserve(additional);
    }
    Ok(())
}
pub(crate) fn push<T>(ledger: &mut Ledger, values: &mut Vec<T>, value: T) -> Result<()> {
    if let Some(budget) = ledger {
        budget.add_records(1)?;
        budget.work(1)?;
    }
    reserve_vec(ledger, values, 1)?;
    values.push(value);
    Ok(())
}
pub(crate) fn collect<T>(
    ledger: &mut Ledger,
    values: impl IntoIterator<Item = T>,
) -> Result<Vec<T>> {
    let mut result = Vec::new();
    let values = values.into_iter();
    let (lower, upper) = values.size_hint();
    if upper == Some(lower) {
        reserve_vec(ledger, &mut result, lower)?;
    }
    for value in values {
        push(ledger, &mut result, value)?;
    }
    Ok(result)
}
pub(crate) fn filled<T: Clone>(ledger: &mut Ledger, count: usize, value: T) -> Result<Vec<T>> {
    let mut result = Vec::new();
    if let Some(budget) = ledger {
        budget.add_records(count)?;
        budget.work(count)?;
    }
    reserve_vec(ledger, &mut result, count)?;
    result.resize(count, value);
    Ok(result)
}
pub(crate) fn clone_vec<T: Clone>(ledger: &mut Ledger, values: &[T]) -> Result<Vec<T>> {
    collect(ledger, values.iter().cloned())
}
pub(crate) fn drop_vec<T>(ledger: &mut Ledger, values: Vec<T>) -> Result<()> {
    let bytes = bytes(&values)?;
    drop(values);
    if let Some(budget) = ledger {
        budget.release_bytes(bytes)?;
    }
    Ok(())
}
pub(crate) fn clone_run(ledger: &mut Ledger, row: &PreparedRun) -> Result<PreparedRun> {
    Ok(PreparedRun {
        font: row.font.clone(),
        font_size: row.font_size,
        font_attrs: row.font_attrs,
        synthesis: row.synthesis,
        coords: clone_vec(ledger, &row.coords)?,
        style: row.style,
        script: row.script,
        level: row.level,
        atoms: row.atoms.clone(),
        glyph_basis: row.glyph_basis,
        metrics: row.metrics,
    })
}
fn clone_cell(ledger: &mut Ledger, row: &PreparedCell) -> Result<PreparedCell> {
    work(ledger, 1)?;
    let payload = match &row.payload {
        CellPayload::NoProviderInput => CellPayload::NoProviderInput,
        CellPayload::Shaped {
            runs,
            clusters,
            glyphs,
            scope_relations,
            advance,
            positions,
            frame,
        } => CellPayload::Shaped {
            runs: runs.clone(),
            clusters: clusters.clone(),
            glyphs: glyphs.clone(),
            scope_relations: scope_relations.clone(),
            advance: advance.clone(),
            positions: clone_vec(ledger, positions)?,
            frame: *frame,
        },
    };
    Ok(PreparedCell {
        binding: row.binding,
        metrics: row.metrics,
        level: row.level,
        extent: row.extent,
        before: row.before,
        after: row.after,
        payload,
    })
}
pub(crate) fn clone_shaped(ledger: &mut Ledger, source: &ShapedData) -> Result<ShapedData> {
    header::<ShapedData>(ledger)?;
    let mut runs = Vec::new();
    for row in &source.runs {
        let row = clone_run(ledger, row)?;
        push(ledger, &mut runs, row)?;
    }
    let mut invocations = Vec::new();
    for row in &source.character_invocations {
        let mut components = Vec::new();
        for set in &row.components {
            let set = clone_vec(ledger, set)?;
            push(ledger, &mut components, set)?;
        }
        let mut generated = Vec::new();
        for set in &row.generated {
            let set = clone_vec(ledger, set)?;
            push(ledger, &mut generated, set)?;
        }
        let mut constraints = Vec::new();
        for constraint in &row.constraints {
            let constraint = crate::engine::spacing::character::Constraint {
                run: constraint.run,
                components: clone_vec(ledger, &constraint.components)?,
                required: constraint.required,
                kind: constraint.kind.clone(),
                provenance: clone_vec(ledger, &constraint.provenance)?,
            };
            push(ledger, &mut constraints, constraint)?;
        }
        let row = crate::engine::spacing::character::Invocation {
            atoms: row.atoms.clone(),
            run: row.run,
            complete: row.complete,
            path: row.path,
            components,
            generated,
            constraints,
        };
        push(ledger, &mut invocations, row)?;
    }
    let mut word_spacing = Vec::new();
    for row in &source.word_spacing {
        let row = crate::engine::spacing::WordSpacingUnit {
            atoms: row.atoms.clone(),
            base: row.base,
            components: clone_vec(ledger, &row.components)?,
            authored: row.authored,
            allocation: row.allocation,
            geometry: row.geometry,
            shared: row.shared.clone(),
        };
        push(ledger, &mut word_spacing, row)?;
    }
    let mut cells = Vec::new();
    for row in &source.cells {
        let row = clone_cell(ledger, row)?;
        push(ledger, &mut cells, row)?;
    }
    Ok(ShapedData {
        scope_glyph_custody: clone_vec(ledger, &source.scope_glyph_custody)?,
        scope_relations: clone_vec(ledger, &source.scope_relations)?,
        // Font-query witnesses remain in the immutable selected basis. Geometry
        // replay consumes no resolver and needs no redundant witness allocation.
        font_queries: Vec::new(),
        runs,
        clusters: clone_vec(ledger, &source.clusters)?,
        glyphs: clone_vec(ledger, &source.glyphs)?,
        provider_groups: clone_vec(ledger, &source.provider_groups)?,
        character_gaps: clone_vec(ledger, &source.character_gaps)?,
        character_invocations: invocations,
        character_requirements: source.character_requirements.clone(),
        character_exclusions: source.character_exclusions.clone(),
        atomic_gaps: OrdinalMap {
            rows: clone_vec(ledger, &source.atomic_gaps.rows)?,
        },
        word_spacing,
        items: clone_vec(ledger, &source.items)?,
        cells,
        empty_run: source
            .empty_run
            .as_ref()
            .map(|row| clone_run(ledger, row))
            .transpose()?,
    })
}
pub(crate) fn drop_shaped(ledger: &mut Ledger, source: ShapedData) -> Result<()> {
    let released = if ledger.is_some() {
        let mut total = std::mem::size_of::<ShapedData>();
        let mut include = |count: usize| -> Result<()> {
            total = total.checked_add(count).ok_or_else(|| {
                super::refusal(ScopeAlignmentError::Overflow {
                    kind: ScopeAlignmentLimitKind::PayloadBytes,
                })
            })?;
            Ok(())
        };
        for run in &source.runs {
            include(bytes(&run.coords)?)?;
        }
        if let Some(run) = &source.empty_run {
            include(bytes(&run.coords)?)?;
        }
        for row in &source.character_invocations {
            for set in row.components.iter().chain(&row.generated) {
                include(bytes(set)?)?;
            }
            for row in &row.constraints {
                include(bytes(&row.components)?)?;
                include(bytes(&row.provenance)?)?;
            }
            include(bytes(&row.components)?)?;
            include(bytes(&row.generated)?)?;
            include(bytes(&row.constraints)?)?;
        }
        for row in &source.word_spacing {
            include(bytes(&row.components)?)?;
        }
        for row in &source.cells {
            work(ledger, 1)?;
            if let CellPayload::Shaped { positions, .. } = &row.payload {
                include(bytes(positions)?)?;
            }
        }
        include(bytes(&source.scope_glyph_custody)?)?;
        include(bytes(&source.scope_relations)?)?;
        include(bytes(&source.runs)?)?;
        include(bytes(&source.clusters)?)?;
        include(bytes(&source.glyphs)?)?;
        include(bytes(&source.provider_groups)?)?;
        include(bytes(&source.character_gaps)?)?;
        include(bytes(&source.character_invocations)?)?;
        include(bytes(&source.atomic_gaps.rows)?)?;
        include(bytes(&source.word_spacing)?)?;
        include(bytes(&source.items)?)?;
        include(bytes(&source.cells)?)?;
        total
    } else {
        0
    };
    drop(source);
    if let Some(budget) = ledger {
        budget.release_bytes(released)?;
    }
    Ok(())
}

pub(crate) fn try_collect<T>(
    ledger: &mut Ledger,
    values: impl IntoIterator<Item = Result<T>>,
) -> Result<Vec<T>> {
    let mut result = Vec::new();
    let values = values.into_iter();
    let (lower, upper) = values.size_hint();
    if upper == Some(lower) {
        reserve_vec(ledger, &mut result, lower)?;
    }
    for value in values {
        push(ledger, &mut result, value?)?;
    }
    Ok(result)
}
pub(crate) fn insert<T>(
    ledger: &mut Ledger,
    map: &mut OrdinalMap<T>,
    key: usize,
    value: T,
) -> Result<()> {
    work(ledger, map.rows.len().saturating_add(1))?;
    if map.get(&key).is_none() {
        if let Some(budget) = ledger {
            budget.add_records(1)?;
        }
        reserve_vec(ledger, &mut map.rows, 1)?;
    }
    map.insert(key, value);
    Ok(())
}
