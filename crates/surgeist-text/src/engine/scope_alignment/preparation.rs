//! Glyph-free captured font demands and actual prepared retention.

use super::*;

pub(crate) struct ScopeMetricsData {
    pub(crate) scope: CapturedScope,
    pub(crate) reference: ScopeFontReference,
    pub(crate) font: FontData,
    pub(crate) font_size: f32,
    pub(crate) synthesis: FontSynthesis,
    pub(crate) coords: Vec<i16>,
    pub(crate) line_height: f64,
    pub(crate) over: f64,
    pub(crate) under: f64,
    pub(crate) native_coordinate: f64,
    pub(crate) matching: [Option<f64>; 2],
    pub(crate) baselines: [Option<FontBaselineOutcome>; 2],
    pub(crate) upright_reference: Option<UprightMetricReference>,
    pub(crate) upright_provenance: Option<UprightMetricProvenance>,
}

/// Cells demand the genuine text container before any body trial, including
/// when source processing proved that there is no provider input. An Inline
/// owns its own metrics; a domain's derived RootInline uses that same actual
/// Block/AtomicBlock/OutOfFlowBlock scope, never its lexical parent.
pub(crate) fn prepare_cell_resources(
    data: &mut PreparedScopes,
    catalogue: &mut crate::catalogue::Catalogue,
    analyzed: &AnalyzedData,
    options: crate::PreparationOptions,
    budget: &mut Option<ScopeAlignmentBudget>,
) -> Result<()> {
    let Some(output) = analyzed
        .combination
        .as_ref()
        .filter(|output| !output.cells.is_empty())
    else {
        return Ok(());
    };
    if analyzed.paragraph.scope_topology.is_none() {
        return Err(refusal(ScopeAlignmentError::MissingCapture));
    }
    let budget = budget
        .as_mut()
        .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
    let mut pinned = retained_pins(data, budget)?;
    for binding in 0..output.bindings.len() {
        let scope = cell_container(analyzed, binding, budget)?;
        ensure_metric(
            data,
            catalogue,
            analyzed,
            options,
            (scope, [false; 2]),
            budget,
            &mut pinned,
        )?;
    }
    drop_scratch(pinned, budget)?;
    data.usage = Some(budget.usage());
    Ok(())
}

/// Read only the metric row already demanded by this actual cell container.
/// The caller retains its index, not a donated body font or duplicate receipt.
pub(crate) fn cell_metric_index(
    data: &PreparedScopes,
    scope: usize,
    budget: &mut accounting::Ledger,
) -> Result<usize> {
    let budget = budget
        .as_mut()
        .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
    metric_row(data, scope, budget)?
        .ok_or_else(|| refusal(ScopeAlignmentError::MissingMetrics { scope }))
}

fn metric_row(
    data: &PreparedScopes,
    scope: usize,
    budget: &mut ScopeAlignmentBudget,
) -> Result<Option<usize>> {
    for (index, row) in data.metrics.iter().enumerate() {
        budget.work(1)?;
        if row.scope.index == scope {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn remember_pin(
    pinned: &mut Vec<(usize, usize)>,
    resource: (usize, usize),
    budget: &mut ScopeAlignmentBudget,
) -> Result<()> {
    for existing in pinned.iter() {
        budget.work(1)?;
        if *existing == resource {
            return Ok(());
        }
    }
    push(budget, pinned, resource)
}

fn retained_pins(
    data: &PreparedScopes,
    budget: &mut ScopeAlignmentBudget,
) -> Result<Vec<(usize, usize)>> {
    let mut pinned = Vec::new();
    for row in &data.metrics {
        budget.work(1)?;
        remember_pin(
            &mut pinned,
            (row.font.bytes().as_ptr() as usize, row.font.bytes().len()),
            budget,
        )?;
    }
    for reference in &data.cell_tabs {
        budget.work(1)?;
        for metric in [&reference.space, &reference.ch].into_iter().flatten() {
            budget.work(1)?;
            if let Some(font) = &metric.font {
                remember_pin(
                    &mut pinned,
                    (font.font.bytes().as_ptr() as usize, font.font.bytes().len()),
                    budget,
                )?;
            }
        }
    }
    Ok(pinned)
}

/// One canonical resource/metric owner serves both early cell demands and
/// later scope demands. Reuse preserves the selected resource, normalized
/// coordinates, query count and ongoing live/peak accounting.
fn ensure_metric(
    data: &mut PreparedScopes,
    catalogue: &mut crate::catalogue::Catalogue,
    analyzed: &AnalyzedData,
    options: crate::PreparationOptions,
    (scope, mut names): (usize, [bool; 2]),
    budget: &mut ScopeAlignmentBudget,
    pinned: &mut Vec<(usize, usize)>,
) -> Result<usize> {
    let capture = &analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .expect("admitted metric capture")
        .capture;
    let reference = capture
        .scope_alignment_fact_with_budget(scope, budget)?
        .and_then(|row| row.font_reference)
        .ok_or_else(|| refusal(ScopeAlignmentError::MissingFontReference { scope }))?;
    let axis = if options.orientation() == crate::LineOrientation::Vertical {
        crate::FontBaselineAxis::VerticalX
    } else {
        crate::FontBaselineAxis::HorizontalY
    };
    if reference.axis != axis {
        return Err(refusal(ScopeAlignmentError::UnsupportedReference { scope }));
    }
    names[baseline_index(reference.baseline)] = true;
    if let Some(native) = options.resolved_baseline_reference() {
        names[baseline_index(native)] = true;
    }
    if let Some(index) = metric_row(data, scope, budget)? {
        measure_baselines(&mut data.metrics[index], names, budget)?;
        return Ok(index);
    }
    let captured = CapturedScope {
        capture: Arc::clone(capture),
        index: scope,
    };
    let style = captured
        .style()
        .ok_or_else(|| refusal(ScopeAlignmentError::MissingStyle { scope }))?;
    let resolved = crate::engine::resolve::resolve_scope(
        catalogue.collection_mut(),
        style,
        options.scale(),
        budget,
    )?;
    let script = icu_properties::PropertyParser::<icu_properties::props::Script>::new()
        .get_strict_utf8(&reference.script.iso15924())
        .ok_or_else(|| refusal(ScopeAlignmentError::UnsupportedReference { scope }))?;
    let catalogue_identity = catalogue.identity().clone();
    let instance = crate::engine::shape::scope_font(
        catalogue.query(),
        &resolved,
        script,
        &catalogue_identity,
        budget,
        pinned,
    )?;
    remember_pin(
        pinned,
        (
            instance.font.bytes().as_ptr() as usize,
            instance.font.bytes().len(),
        ),
        budget,
    )?;
    let metrics = measure_scope(
        captured,
        reference,
        instance,
        resolved.line_height,
        options,
        budget,
    )?;
    crate::engine::resolve::drop_scope_style(resolved, budget)?;
    let index = data.metrics.len();
    push(budget, &mut data.metrics, metrics)?;
    measure_baselines(&mut data.metrics[index], names, budget)?;
    Ok(index)
}

/// Resolve the actual target/parent inventory, including selectable indications,
/// before publishing preparation. Context-only scopes induce no query by themselves.
pub(crate) fn prepare_resources(
    data: &mut PreparedScopes,
    catalogue: &mut crate::catalogue::Catalogue,
    analyzed: &AnalyzedData,
    options: crate::PreparationOptions,
    shaped: &crate::engine::layout::data::ShapedData,
    tables: &crate::protocol::prepared::PreparedTables,
    budget: Option<ScopeAlignmentBudget>,
) -> Result<()> {
    let Some(mut budget) = budget else {
        return Ok(());
    };
    let topology = analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .expect("active scope capture");
    let capture = &topology.capture;
    let visits = checked_sum(
        crate::ScopeAlignmentLimitKind::StructuralWork,
        shaped.scope_glyph_custody.len(),
        shaped.scope_relations.len(),
    )?;
    let visits = checked_sum(
        crate::ScopeAlignmentLimitKind::StructuralWork,
        visits,
        analyzed.paragraph.anchor_order.len(),
    )?;
    budget.work(checked_sum(
        crate::ScopeAlignmentLimitKind::StructuralWork,
        visits,
        tables.alternatives.len(),
    )?)?;
    let mut active = Vec::new();
    let mut activate = |owner: usize, budget: &mut ScopeAlignmentBudget| -> Result<()> {
        budget.work(1)?;
        for &target in &data.targets {
            budget.work(checked_sum(
                crate::ScopeAlignmentLimitKind::StructuralWork,
                1,
                active.len(),
            )?)?;
            if is_descendant(capture, target, owner) && !active.contains(&target) {
                push(budget, &mut active, target)?;
            }
        }
        Ok(())
    };
    for custody in &shaped.scope_glyph_custody {
        // Actual extracted glyph custody, not a painted cluster source hull.
        // Context is a genuine generated association, never a source root.
        match *custody {
            GlyphCustody::Input(owner)
            | GlyphCustody::Context(owner)
            | GlyphCustody::UnqualifiedContext(Some(owner)) => activate(owner, &mut budget)?,
            _ => {}
        }
    }
    for relation in &shaped.scope_relations {
        // These potential target ids came from the actual native operation;
        // unqualified context still demands coherent frames, not membership.
        for owner in [relation.first, relation.second].into_iter().flatten() {
            activate(owner, &mut budget)?;
        }
    }
    for cell in &shaped.cells {
        budget.work(1)?;
        // Even source-proved empty input participates in the real enclosing
        // frame. Its glyph-free receipt, not surviving glyphs, names the owner.
        visit_cell_frame_owners(analyzed, cell.binding, &mut budget, &mut activate)?;
    }
    for anchor in &analyzed.paragraph.anchor_order {
        use crate::protocol::paragraph::SemanticAnchorItem;
        let owner = match anchor.item {
            SemanticAnchorItem::Atomic(index) => {
                capture.events[capture.atomic_orders[index]].before
            }
            SemanticAnchorItem::Control(index) => {
                control_frame_owner(capture, topology, index, &mut budget)?
            }
            SemanticAnchorItem::ComputedLineBoundary(index) => {
                visit_computed_frame_owners(analyzed, index, &mut budget, &mut activate)?;
                None
            }
            _ => None,
        };
        if let Some(owner) = owner {
            activate(owner, &mut budget)?;
        }
    }
    for alternative in &tables.alternatives {
        if !alternative.text.is_empty()
            && let Some(owner) = alternative_owner(analyzed, alternative.definition)
        {
            activate(owner, &mut budget)?;
        }
    }
    let mut demands: Vec<(usize, [bool; 2])> = Vec::new();
    for &target in &active {
        let mut current = Some(target);
        while let Some(scope) = current {
            budget.work(checked_sum(
                crate::ScopeAlignmentLimitKind::StructuralWork,
                demands.len(),
                1,
            )?)?;
            let fact = &capture.scopes[scope];
            if fact.domain != topology.domain {
                break;
            }
            let alignment = capture.scope_alignment_fact_with_budget(scope, &mut budget)?;
            if fact.definition.is_none()
                && alignment.and_then(|row| row.alignment_boundary).is_none()
            {
                return Err(refusal(ScopeAlignmentError::MissingBoundary { scope }));
            }
            let reference = alignment
                .and_then(|row| row.font_reference)
                .ok_or_else(|| refusal(ScopeAlignmentError::MissingFontReference { scope }))?;
            let axis = if options.orientation() == crate::LineOrientation::Vertical {
                crate::FontBaselineAxis::VerticalX
            } else {
                crate::FontBaselineAxis::HorizontalY
            };
            if reference.axis != axis {
                return Err(refusal(ScopeAlignmentError::UnsupportedReference { scope }));
            }
            let index = if let Some(index) = demands.iter().position(|(value, _)| *value == scope) {
                index
            } else {
                let index = demands.len();
                push(&mut budget, &mut demands, (scope, [false; 2]))?;
                index
            };
            demands[index].1[baseline_index(reference.baseline)] = true;
            if let Some(native) = options.resolved_baseline_reference() {
                demands[index].1[baseline_index(native)] = true;
            }
            if fact.definition.is_some() {
                break;
            }
            if let Some(parent) = fact.parent {
                let parent_reference = capture
                    .scope_alignment_fact_with_budget(parent, &mut budget)?
                    .and_then(|row| row.font_reference)
                    .ok_or_else(|| {
                        refusal(ScopeAlignmentError::MissingFontReference { scope: parent })
                    })?;
                demands[index].1[baseline_index(parent_reference.baseline)] = true;
            }
            current = fact.parent;
        }
    }
    let mut pinned = retained_pins(data, &mut budget)?;
    if shaped.cell_arenas == crate::engine::layout::data::CellArenaOwnership::Owned {
        // Width preparation already admitted these actual backing resources.
        // A later real scope-font demand can reuse the same pin without a
        // second byte debit, just as it reuses an earlier metric resource.
        for run in &shaped.runs {
            budget.work(1)?;
            remember_pin(
                &mut pinned,
                (run.font.bytes().as_ptr() as usize, run.font.bytes().len()),
                &mut budget,
            )?;
        }
    }
    for &(scope, names) in &demands {
        ensure_metric(
            data,
            catalogue,
            analyzed,
            options,
            (scope, names),
            &mut budget,
            &mut pinned,
        )?;
    }
    // Release only after the allocations actually cease to coexist. Record
    // charges remain monotonic materialization counts for this transaction.
    drop_scratch(demands, &mut budget)?;
    drop_scratch(active, &mut budget)?;
    drop_scratch(pinned, &mut budget)?;
    data.retention = Some(prepared_retention(
        data,
        analyzed,
        shaped,
        tables,
        &mut budget,
    )?);
    data.usage = Some(budget.usage());
    Ok(())
}

fn drop_scratch<T>(values: Vec<T>, budget: &mut ScopeAlignmentBudget) -> Result<()> {
    let bytes = capacity_bytes(&values)?;
    drop(values);
    budget.release_bytes(bytes)
}

fn metric_work(font: &FontData, tags: &[[u8; 4]], budget: &mut ScopeAlignmentBudget) -> Result<()> {
    let face = skrifa::FontRef::from_index(font.bytes(), font.index())
        .map_err(|_| crate::Error::new(crate::ErrorCode::FontLoadFailed, "invalid scope font"))?;
    budget.work(tags.len())?;
    for tag in tags {
        if let Some(data) = face.table_data(skrifa::Tag::new(tag)) {
            budget.work(data.as_bytes().len())?;
        }
    }
    Ok(())
}

fn measure_scope(
    scope: CapturedScope,
    reference: ScopeFontReference,
    instance: crate::engine::shape::CanonicalFontInstance,
    own_line_height: crate::LineHeight,
    options: crate::PreparationOptions,
    budget: &mut ScopeAlignmentBudget,
) -> Result<ScopeMetricsData> {
    let crate::engine::shape::CanonicalFontInstance {
        font,
        font_size,
        synthesis,
        coords,
    } = instance;
    let vertical = options.orientation() == crate::LineOrientation::Vertical;
    if reference.axis
        != if vertical {
            crate::FontBaselineAxis::VerticalX
        } else {
            crate::FontBaselineAxis::HorizontalY
        }
    {
        return Err(refusal(ScopeAlignmentError::UnsupportedReference {
            scope: scope.index,
        }));
    }
    metric_work(
        &font,
        &[*b"head", *b"hhea", *b"OS/2", *b"fvar", *b"avar", *b"MVAR"],
        budget,
    )?;
    let instance = crate::EffectiveFontInstance::qualify_synthesized(
        font.face(),
        font_size,
        &coords,
        synthesis,
    )?;
    let mut native_coordinate = 0.0;
    let mut upright_reference = None;
    let mut upright_provenance = None;
    let (over, under, normal) = if vertical {
        metric_work(&font, &[*b"BASE", *b"vhea"], budget)?;
        let metrics = instance.upright_metrics(font.catalogue_identity(), reference.script)?;
        native_coordinate = metrics.central_coordinate_or_distance();
        upright_reference = Some(metrics.reference());
        upright_provenance = Some(metrics.provenance());
        let gap = if matches!(own_line_height, crate::LineHeight::MetricsRelative(_)) {
            let gap = instance.upright_line_gap(font.catalogue_identity())?;
            let scale = f64::from(font_size) / f64::from(metrics.units_per_em());
            crate::engine::layout::finite(
                "scope upright gap",
                crate::TextNumericPhase::Preparation,
                gap * scale,
            )?
        } else {
            0.0
        };
        let normal = if matches!(own_line_height, crate::LineHeight::MetricsRelative(_)) {
            let extent = crate::engine::layout::add(
                "scope upright extent",
                crate::TextNumericPhase::Preparation,
                metrics.used_over(),
                -metrics.used_under(),
            )?;
            crate::engine::layout::add(
                "scope upright normal extent",
                crate::TextNumericPhase::Preparation,
                extent,
                gap,
            )?
        } else {
            0.0
        };
        (metrics.used_over(), -metrics.used_under(), normal)
    } else {
        let metrics = instance.metrics(font.catalogue_identity())?;
        if let Some(name) = options.resolved_baseline_reference() {
            metric_work(&font, &[*b"BASE"], budget)?;
            let baseline = instance.baseline(
                font.catalogue_identity(),
                reference.script,
                reference.axis,
                name,
            )?;
            let FontBaselineOutcome::Available(value) = *baseline.outcome() else {
                return Err(refusal(ScopeAlignmentError::UnsupportedReference {
                    scope: scope.index,
                }));
            };
            native_coordinate = value.coordinate();
        }
        let over = crate::engine::layout::add(
            "scope font over",
            crate::TextNumericPhase::Preparation,
            metrics.ascent(),
            -native_coordinate,
        )?;
        let under = crate::engine::layout::add(
            "scope font under",
            crate::TextNumericPhase::Preparation,
            -metrics.descent(),
            native_coordinate,
        )?;
        let normal = if matches!(own_line_height, crate::LineHeight::MetricsRelative(_)) {
            let extent = crate::engine::layout::add(
                "scope font extent",
                crate::TextNumericPhase::Preparation,
                metrics.ascent(),
                -metrics.descent(),
            )?;
            crate::engine::layout::add(
                "scope normal extent",
                crate::TextNumericPhase::Preparation,
                extent,
                metrics.leading(),
            )?
        } else {
            0.0
        };
        (over, under, normal)
    };
    let over = crate::engine::layout::finite(
        "scope font over",
        crate::TextNumericPhase::Preparation,
        over,
    )?;
    let under = crate::engine::layout::finite(
        "scope font under",
        crate::TextNumericPhase::Preparation,
        under,
    )?;
    let line_height = crate::engine::resolve::font_line_height(
        font_size,
        own_line_height,
        normal,
        crate::TextNumericPhase::Preparation,
    )?;
    Ok(ScopeMetricsData {
        scope,
        reference,
        font,
        font_size,
        synthesis,
        coords,
        line_height,
        over,
        under,
        native_coordinate,
        matching: [None; 2],
        baselines: [None; 2],
        upright_reference,
        upright_provenance,
    })
}

fn measure_baselines(
    data: &mut ScopeMetricsData,
    names: [bool; 2],
    budget: &mut ScopeAlignmentBudget,
) -> Result<()> {
    let mut needed = [false; 2];
    for index in 0..needed.len() {
        budget.work(1)?;
        needed[index] = names[index] && data.matching[index].is_none();
    }
    if !needed.into_iter().any(|name| name) {
        return Ok(());
    }
    let instance = crate::EffectiveFontInstance::qualify_synthesized(
        data.font.face(),
        data.font_size,
        &data.coords,
        data.synthesis,
    )?;
    for name in [FontBaseline::Alphabetic, FontBaseline::Central] {
        let index = baseline_index(name);
        if !needed[index] {
            continue;
        }
        metric_work(&data.font, &[*b"BASE"], budget)?;
        let measurement = instance.baseline(
            data.font.catalogue_identity(),
            data.reference.script,
            data.reference.axis,
            name,
        )?;
        let outcome = *measurement.outcome();
        data.baselines[index] = Some(outcome);
        let coordinate = match (data.upright_reference, name, outcome) {
            (Some(UprightMetricReference::NominalVerticalPenX), FontBaseline::Central, _) => 0.0,
            (Some(UprightMetricReference::NominalVerticalPenX), _, _) => {
                return Err(refusal(ScopeAlignmentError::UnsupportedReference {
                    scope: data.scope.index,
                }));
            }
            (_, _, FontBaselineOutcome::Available(value)) => crate::engine::layout::add(
                "scope matching baseline",
                crate::TextNumericPhase::Preparation,
                value.coordinate(),
                -data.native_coordinate,
            )?,
            _ => {
                return Err(refusal(ScopeAlignmentError::UnsupportedReference {
                    scope: data.scope.index,
                }));
            }
        };
        data.matching[index] = Some(coordinate);
    }
    Ok(())
}

/// The sole description of this owner's retained allocations. Both scalar
/// retention and a union of related immutable results use these same witnesses;
/// the driver, rather than this owner, deduplicates shared resources and meters
/// its traversal/set scratch. An ordinal or witness never authorizes a scope.
pub(crate) fn visit_prepared_allocations(
    data: &PreparedScopes,
    analyzed: &AnalyzedData,
    shaped: &crate::engine::layout::data::ShapedData,
    tables: &crate::protocol::prepared::PreparedTables,
    visit: &mut super::census::AllocationVisitor<'_>,
) -> Result<()> {
    use super::census::AllocationWitness;
    let capture = &analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .expect("active capture")
        .capture;
    capture.visit_scope_alignment_allocations(visit)?;
    crate::engine::spacing::character::visit_shaped_allocations(shaped, visit)?;
    visit(AllocationWitness::object(data, 0))?;
    visit(AllocationWitness::buffer(
        &data.metrics,
        data.metrics.len(),
    )?)?;
    visit(AllocationWitness::buffer(
        &data.targets,
        data.targets.len(),
    )?)?;
    visit(AllocationWitness::buffer(
        &data.atom_owners,
        data.atom_owners.len(),
    )?)?;
    visit(AllocationWitness::buffer(&data.breaks, data.breaks.len())?)?;
    for owners in &data.atom_owners {
        visit(AllocationWitness::buffer(owners, owners.len())?)?;
    }
    for row in &data.metrics {
        visit(AllocationWitness::buffer(&row.coords, row.coords.len())?)?;
        visit(AllocationWitness::resource(row.font.bytes()))?;
    }
    if data.cell_tabs.capacity() != 0 {
        visit(AllocationWitness::buffer(
            &data.cell_tabs,
            data.cell_tabs.len(),
        )?)?;
    }
    for reference in &data.cell_tabs {
        for metric in [&reference.space, &reference.ch].into_iter().flatten() {
            if let Some(font) = &metric.font {
                visit(AllocationWitness::buffer(&font.coords, font.coords.len())?)?;
                visit(AllocationWitness::resource(font.font.bytes()))?;
            }
        }
    }
    visit_cell_allocations(shaped, visit)?;
    if !shaped.cells.is_empty() {
        accounting::visit_cell_items(shaped, visit)?;
    }
    if shaped.cell_arenas == crate::engine::layout::data::CellArenaOwnership::Owned {
        accounting::visit_cell_payload_arenas(shaped, visit)?;
        accounting::visit_cell_exclusions(shaped, visit)?;
        for run in &shaped.runs {
            visit(AllocationWitness::resource(run.font.bytes()))?;
        }
    }
    crate::protocol::prepared::visit_cell_allocations(tables, visit)?;
    visit(AllocationWitness::object(&shaped.scope_glyph_custody, 0))?;
    visit(AllocationWitness::buffer(
        &shaped.scope_glyph_custody,
        shaped.scope_glyph_custody.len(),
    )?)?;
    visit(AllocationWitness::object(&shaped.scope_relations, 0))?;
    visit(AllocationWitness::buffer(
        &shaped.scope_relations,
        shaped.scope_relations.len(),
    )?)?;
    Ok(())
}

/// Shared by prepared and selected owners. The outer cell buffer includes each
/// payload's Vec header; only the nested position backing is a second allocation.
pub(crate) fn visit_cell_allocations(
    shaped: &crate::engine::layout::data::ShapedData,
    visit: &mut super::census::AllocationVisitor<'_>,
) -> Result<()> {
    use super::census::AllocationWitness;
    if shaped.cells.is_empty() && shaped.cells.capacity() == 0 {
        return Ok(());
    }
    visit(AllocationWitness::object(&shaped.cells, 0))?;
    // This row census also admits the following per-cell payload inspections.
    visit(AllocationWitness::buffer(
        &shaped.cells,
        shaped.cells.len(),
    )?)?;
    for cell in &shaped.cells {
        if let crate::engine::layout::data::CellPayload::LaidOut { layout, .. } = &cell.payload {
            visit(AllocationWitness::buffer(
                &layout.positions,
                layout.positions.len(),
            )?)?;
            if layout.tabs.capacity() != 0 {
                visit(AllocationWitness::buffer(&layout.tabs, layout.tabs.len())?)?;
            }
        }
    }
    Ok(())
}

fn prepared_retention(
    data: &PreparedScopes,
    analyzed: &AnalyzedData,
    shaped: &crate::engine::layout::data::ShapedData,
    tables: &crate::protocol::prepared::PreparedTables,
    budget: &mut ScopeAlignmentBudget,
) -> Result<crate::ScopeAlignmentRetention> {
    super::census::retained_prepared(data, analyzed, shaped, tables, budget)
}

/// Complete the genuine fixed container references before any Cell packet trial.
pub(crate) fn prepare_cell_tabs(
    data: &mut PreparedScopes,
    catalogue: &mut crate::catalogue::Catalogue,
    analyzed: &AnalyzedData,
    options: crate::PreparationOptions,
    scratch: &mut crate::engine::shape::ShapeContext,
    ledger: &mut accounting::Ledger,
) -> Result<crate::tabs::cell::CellTabPlan> {
    use crate::tabs::cell::{CellTabMetric, CellTabOperation, CellTabPlan, CellTabReference};
    let mut plan = CellTabPlan::default();
    if options.tabs() != crate::TabPreparation::CssStops {
        return Ok(plan);
    }
    let Some(output) = &analyzed.combination else {
        return Ok(plan);
    };
    if output.bindings.is_empty() {
        return Ok(plan);
    }
    for (binding, row) in output.bindings.iter().enumerate() {
        accounting::work(ledger, 1)?;
        let mut scope = None;
        for atom in row.atoms.clone() {
            accounting::work(ledger, 1)?;
            if analyzed.projection.atoms[atom].code_point != 9 {
                continue;
            }
            let budget = ledger
                .as_mut()
                .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
            let owner = if let Some(owner) = scope {
                owner
            } else {
                let owner = cell_container(analyzed, binding, budget)?;
                scope = Some(owner);
                owner
            };
            let mut reference = None;
            for (index, existing) in data.cell_tabs.iter().enumerate() {
                budget.work(1)?;
                if existing.scope.index == owner {
                    reference = Some(index);
                    break;
                }
            }
            let reference = if let Some(index) = reference {
                index
            } else {
                let topology = analyzed
                    .paragraph
                    .scope_topology
                    .as_ref()
                    .ok_or_else(|| refusal(ScopeAlignmentError::MissingCapture))?;
                let index = data.cell_tabs.len();
                push(
                    budget,
                    &mut data.cell_tabs,
                    CellTabReference {
                        scope: CapturedScope {
                            capture: Arc::clone(&topology.capture),
                            index: owner,
                        },
                        direction: output.plan.groups[row.group].inner_direction,
                        space: None,
                        ch: None,
                    },
                )?;
                index
            };
            let size = analyzed.styles[analyzed.analysis.infos[atom].1]
                .style
                .authored()
                .tab_size;
            accounting::push(
                ledger,
                &mut plan.operations,
                CellTabOperation {
                    atom,
                    reference,
                    size,
                    period: 0.0,
                    threshold: 0.0,
                },
            )?;
        }
    }
    if plan.operations.is_empty() {
        return Ok(plan);
    }
    operations::order_by(&mut plan.operations, ledger.as_mut(), |a, b| {
        a.atom.cmp(&b.atom)
    })?;
    let mut pinned = retained_pins(
        data,
        ledger
            .as_mut()
            .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?,
    )?;
    let catalogue_identity = catalogue.identity().clone();
    for reference in 0..data.cell_tabs.len() {
        let mut needed = false;
        let mut space = false;
        for operation in &plan.operations {
            accounting::work(ledger, 1)?;
            if operation.reference == reference && operation.size.value() != 0.0 {
                needed = true;
                space |= operation.size.is_spaces();
            }
        }
        if !needed {
            continue;
        }
        let scope = data.cell_tabs[reference].scope.clone();
        let style = scope
            .style()
            .ok_or_else(|| refusal(ScopeAlignmentError::MissingStyle { scope: scope.index }))?;
        let resolved = crate::engine::resolve::resolve_cell_tab_style(
            catalogue.collection_mut(),
            style,
            options.scale(),
            ledger
                .as_mut()
                .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?,
        )?;
        let styles = [resolved];
        if space {
            let direction = data.cell_tabs[reference].direction;
            let probe = crate::engine::shape::CharacterProbe {
                input: cell_tab_probe_input(analyzed, &styles, &catalogue_identity),
                style_index: 0,
                ch: ' ',
                direction,
                content: crate::UprightContent::TabMeasurement {
                    role: crate::UprightTabMeasurement::SpaceInterval,
                    owner: crate::TextStyleOwner::Scope { scope: scope.index },
                },
            };
            let (advance, font) = crate::engine::shape::measure_character(
                catalogue.query(),
                probe,
                scratch,
                ledger,
                &pinned,
            )?
            .ok_or_else(|| {
                crate::protocol::protocol_error(crate::ProtocolError::UnsupportedContext {
                    field: crate::ProtocolContextField::TabMetrics,
                })
            })?;
            remember_pin(
                &mut pinned,
                (font.font.bytes().as_ptr() as usize, font.font.bytes().len()),
                ledger
                    .as_mut()
                    .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?,
            )?;
            data.cell_tabs[reference].space = Some(CellTabMetric {
                advance,
                font: Some(font),
            });
        }
        let mut ch_needed = false;
        for operation in &mut plan.operations {
            accounting::work(ledger, 1)?;
            if operation.reference != reference || operation.size.value() == 0.0 {
                continue;
            }
            let factor = if operation.size.is_spaces() {
                data.cell_tabs[reference]
                    .space
                    .as_ref()
                    .ok_or_else(|| {
                        refusal(ScopeAlignmentError::MissingMetrics {
                            scope: data.cell_tabs[reference].scope.index,
                        })
                    })?
                    .advance
            } else {
                f64::from(options.scale())
            };
            operation.period =
                crate::tabs::multiply("cell tab interval", operation.size.value(), factor)?;
            ch_needed |= operation.period != 0.0;
        }
        if ch_needed {
            let owner = data.cell_tabs[reference].scope.index;
            let probe = crate::engine::shape::CharacterProbe {
                input: cell_tab_probe_input(analyzed, &styles, &catalogue_identity),
                style_index: 0,
                ch: '0',
                direction: crate::ParagraphBaseDirection::LeftToRight,
                content: crate::UprightContent::TabMeasurement {
                    role: crate::UprightTabMeasurement::ZeroThreshold,
                    owner: crate::TextStyleOwner::Scope { scope: owner },
                },
            };
            let measured = crate::engine::shape::measure_character(
                catalogue.query(),
                probe,
                scratch,
                ledger,
                &pinned,
            )?;
            let metric = if let Some((advance, font)) = measured {
                remember_pin(
                    &mut pinned,
                    (font.font.bytes().as_ptr() as usize, font.font.bytes().len()),
                    ledger
                        .as_mut()
                        .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?,
                )?;
                CellTabMetric {
                    advance,
                    font: Some(font),
                }
            } else {
                CellTabMetric {
                    advance: crate::tabs::multiply(
                        "tab ch fallback",
                        f64::from(styles[0].style.font_size),
                        0.5,
                    )?,
                    font: None,
                }
            };
            if metric.advance < 0.0 {
                return Err(crate::protocol::protocol_error(
                    crate::ProtocolError::UnsupportedContext {
                        field: crate::ProtocolContextField::TabMetrics,
                    },
                ));
            }
            let threshold = crate::tabs::multiply("cell tab half ch", metric.advance, 0.5)?;
            data.cell_tabs[reference].ch = Some(metric);
            for operation in &mut plan.operations {
                accounting::work(ledger, 1)?;
                if operation.reference == reference && operation.period != 0.0 {
                    operation.threshold = threshold;
                }
            }
        }
        for style in styles {
            crate::engine::resolve::drop_cell_tab_style(style, ledger)?;
        }
    }
    accounting::drop_vec(ledger, pinned)?;
    Ok(plan)
}
fn cell_tab_probe_input<'a>(
    analyzed: &'a AnalyzedData,
    styles: &'a [crate::engine::resolve::ResolvedStyleRun],
    catalogue: &'a crate::FontCatalogueIdentity,
) -> crate::engine::ShapingInput<'a> {
    crate::engine::ShapingInput {
        formatting_breaks: &[],
        scope_custody: None,
        characters: None,
        separator_geometry: false,
        profile: crate::engine::upright::ShapingProfile::Horizontal,
        baseline_reference: Some(crate::FontBaseline::Alphabetic),
        projection: &analyzed.projection,
        analysis: &analyzed.analysis,
        styles,
        boxes: &[],
        font_catalogue: catalogue,
        paragraph: &analyzed.paragraph,
        css_tabs: true,
    }
}
