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
    let mut pinned = Vec::new();
    for &(scope, names) in &demands {
        let captured = CapturedScope {
            capture: Arc::clone(capture),
            index: scope,
        };
        let reference = capture
            .scope_alignment_fact_with_budget(scope, &mut budget)?
            .and_then(|row| row.font_reference)
            .expect("admitted reference demand");
        let style = captured
            .style()
            .ok_or_else(|| refusal(ScopeAlignmentError::MissingStyle { scope }))?;
        let resolved = crate::engine::resolve::resolve_scope(
            catalogue.collection_mut(),
            style,
            options.scale(),
            &mut budget,
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
            &mut budget,
            &pinned,
        )?;
        let resource = (
            instance.font.bytes().as_ptr() as usize,
            instance.font.bytes().len(),
        );
        // Conservative duplicate-check/insertion admission, not a comparison trace.
        budget.work(checked_sum(
            crate::ScopeAlignmentLimitKind::StructuralWork,
            pinned.len(),
            1,
        )?)?;
        // scope_font charged the actual first pinned backing before building
        // its coordinates; this list is only bounded duplicate-check scratch.
        if !pinned.contains(&resource) {
            push(&mut budget, &mut pinned, resource)?;
        }
        let metrics = measure_scope(
            captured,
            reference,
            instance,
            resolved.line_height,
            options,
            &mut budget,
        )?;
        crate::engine::resolve::drop_scope_style(resolved, &mut budget)?;
        push(&mut budget, &mut data.metrics, metrics)?;
        // Measure only the names actually demanded by this target/parent graph.
        let index = data.metrics.len() - 1;
        measure_baselines(&mut data.metrics[index], names, &mut budget)?;
    }
    // Release only after the allocations actually cease to coexist. Record
    // charges remain monotonic materialization counts for this transaction.
    drop_scratch(demands, &mut budget)?;
    drop_scratch(active, &mut budget)?;
    drop_scratch(pinned, &mut budget)?;
    data.retention = Some(prepared_retention(data, analyzed, shaped, &mut budget)?);
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
    let instance = crate::EffectiveFontInstance::qualify_synthesized(
        data.font.face(),
        data.font_size,
        &data.coords,
        data.synthesis,
    )?;
    for name in [FontBaseline::Alphabetic, FontBaseline::Central] {
        let index = baseline_index(name);
        if !names[index] {
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

fn prepared_retention(
    data: &PreparedScopes,
    analyzed: &AnalyzedData,
    shaped: &crate::engine::layout::data::ShapedData,
    budget: &mut ScopeAlignmentBudget,
) -> Result<crate::ScopeAlignmentRetention> {
    super::census::retained_prepared(data, analyzed, shaped, budget)
}
