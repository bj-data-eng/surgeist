//! Captured and declared-native box edges follow their immediate parent's sides.
//! Unequal1/2/3/4px edges qualify physical glyph/strip positions and distinguish
//! full-execution TAB receipts from a newly selected partial recipe.

use super::*;

fn control(fixture: &Fixture, node: u32) -> text::PreparedControl {
    fixture
        .binding
        .controls
        .iter()
        .find(|row| row.node == node)
        .unwrap()
        .occurrence
        .clone()
}

fn direction(rtl: bool) -> Direction {
    if rtl { Direction::Rtl } else { Direction::Ltr }
}

fn axes(rtl: bool) -> FlowAxes {
    FlowAxes::new(WritingMode::HorizontalTb, direction(rtl))
}

fn wrapping(fixture: &Fixture, captured_child: bool) -> WrappingInput<u32> {
    let p = control(fixture, 20).captured_control().unwrap().scope();
    let mut scopes = vec![WrappingScopeInput::Captured {
        scope: p,
        wrap_inside: WrapInside::Auto,
        edges: WrapEdges::AUTO,
        metric_contribution: None,
    }];
    scopes.push(if captured_child {
        WrappingScopeInput::Captured {
            scope: control(fixture, 22).captured_control().unwrap().scope(),
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
            metric_contribution: None,
        }
    } else {
        // A REAL declaration over the actual two captured occurrences. Neither
        // shared cuts nor matching roles constructs this pair on the caller's behalf.
        WrappingScopeInput::NativePair {
            start: 22,
            end: 23,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
            metric_contribution: None,
        }
    });
    WrappingInput {
        scopes,
        child_order: Vec::new(),
    }
}

fn native_tree<S: LayoutScalar>(
    fixture: &Fixture,
    formatter_rtl: bool,
    width: f64,
    box_edges: bool,
) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, fixture.binding.nodes.iter().copied())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: direction(formatter_rtl),
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        );
    for row in &fixture.binding.origins {
        tree = tree.children(row.node, []).input(
            row.node,
            LayoutInputOf::inline_text(TextParticipation::new(row.origin.clone())),
        );
    }
    for row in &fixture.binding.controls {
        let kind = match row.occurrence.view().kind() {
            text::ParagraphControlKind::InlineBoundaryStart => InlineBoundaryKind::Start,
            text::ParagraphControlKind::InlineBoundaryEnd => InlineBoundaryKind::End,
            other => panic!("this genuine capture has only four endpoints: {other:?}"),
        };
        let amount = if !box_edges {
            0.0
        } else {
            match row.node {
                20 => 1.0,
                22 => 2.0,
                23 => 3.0,
                21 => 4.0,
                _ => unreachable!(),
            }
        };
        // All dimensions are genuine nonnegative used border components.
        // No margin, manufactured strut or change to the source bidi is involved.
        let edge = InlineBoundaryEdgeOf::try_new(S::ZERO, S::from_f64(amount), S::ZERO).unwrap();
        let metrics = InlineMetricsOf::try_new(
            InlineBaselineContext::Physical,
            S::from_f64(8.0),
            S::from_f64(10.0),
        )
        .unwrap();
        let input = if box_edges {
            // P's immediate parent is actual B. C's parent is actual P,
            // whose host-resolved used direction opposes B. P is source-owned
            // and represented by its real endpoints, not a fake native Box.
            InlineBoundaryInputOf::box_edge(
                kind,
                metrics,
                edge,
                axes(if matches!(row.node, 20 | 21) {
                    formatter_rtl
                } else {
                    !formatter_rtl
                }),
            )
        } else {
            InlineBoundaryInputOf::contribution(kind, metrics, edge).unwrap()
        }
        .with_writing_mode(WritingMode::HorizontalTb)
        .with_direction(direction(formatter_rtl));
        assert_eq!(input.writing_mode(), WritingMode::HorizontalTb);
        assert_eq!(
            input.direction(),
            direction(formatter_rtl),
            "all OLD fields are B's formatter"
        );
        assert_eq!(
            input.parent_edge_axes(),
            if box_edges {
                Some(axes(if matches!(row.node, 20 | 21) {
                    formatter_rtl
                } else {
                    !formatter_rtl
                }))
            } else {
                None
            }
        );
        tree = tree
            .children(row.node, [])
            .input(row.node, LayoutInputOf::inline_boundary(input));
    }
    tree
}

fn compute<S: LayoutScalar>(
    fixture: &Fixture,
    formatter_rtl: bool,
    width: f64,
    captured_child: bool,
    box_edges: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = native_tree(fixture, formatter_rtl, width, box_edges);
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![fixture.binding.clone()],
        wrapping(fixture, captured_child),
        Vec::new(),
    )
    .expect("real nested source/native declarations are admitted before geometry");
    let p = control(fixture, 20).captured_control().unwrap().scope();
    assert_eq!(
        flow.associations.captured_binding(&p),
        Some(TextScopeAssociation::Inline { start: 20, end: 21 })
    );
    assert_eq!(
        flow.associations.captured_binding(&p.parent().unwrap()),
        Some(TextScopeAssociation::ParagraphContainer(0))
    );
    if captured_child {
        let c = control(fixture, 22).captured_control().unwrap().scope();
        assert_eq!(
            flow.associations.captured_binding(&c),
            Some(TextScopeAssociation::Inline { start: 22, end: 23 })
        );
    }
    compute_layout_with_flow(&flow, 0, request(width)).unwrap()
}

fn source_premises(fixture: &Fixture, value: &str, rtl: bool, captured_child: bool) {
    assert_eq!(fixture.binding.nodes, [20, 22, 10, 23, 21]);
    assert_eq!(fixture.binding.origins.len(), 1);
    let origin = fixture.binding.origins[0].origin.clone();
    let end = value.encode_utf16().count();
    assert_eq!(
        origin.view().source().text().code_units(),
        value.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(origin.view().local_range(), text::Range::new(0, end));
    assert_eq!(origin.view().aggregate_range(), text::Range::new(0, end));
    let [ps, cs, ce, pe] =
        [20, 22, 23, 21].map(|node| control(fixture, node).captured_control().unwrap());
    assert!(ps.event_position() < cs.event_position());
    assert!(cs.event_position() < origin.view().capture_position().unwrap());
    assert!(origin.view().capture_position().unwrap() < ce.event_position());
    assert!(ce.event_position() < pe.event_position());
    let p = ps.scope();
    let b = p.parent().unwrap();
    assert_eq!(b.role(), text::ScopeRole::Domain);
    assert_eq!(p.role(), text::ScopeRole::Inline);
    assert_eq!(p.domain(), b);
    assert_eq!(pe.scope(), p);
    assert_eq!(p.entry_control(), Some(ps.clone()));
    assert_eq!(p.exit_control(), Some(pe.clone()));
    let issued = fixture
        .binding
        .prepared
        .native_inline_box(cs.clone(), ce.clone())
        .unwrap();
    if captured_child {
        let c = cs.scope();
        assert_eq!(c.parent(), Some(p.clone()));
        assert_eq!(c.entry_control(), Some(cs.clone()));
        assert_eq!(c.exit_control(), Some(ce.clone()));
        assert_eq!(issued, text::InlineBoxSource::captured(c));
    } else {
        assert_eq!(cs.scope(), p);
        assert_eq!(ce.scope(), p);
        assert_ne!(p.entry_control(), Some(cs));
        assert_ne!(p.exit_control(), Some(ce));
        assert_ne!(issued, text::InlineBoxSource::captured(p));
    }
    // Authored ASCII A/B is genuinely strong L. B LTR resolves it to0;
    // B RTL resolves it to2. No direction override, forged UBA level or extra
    // bidi control is inserted to make the endpoint geometry pass.
    for unit in fixture
        .binding
        .prepared
        .unit_views()
        .filter(|unit| !unit.is_whitespace())
    {
        assert_eq!(unit.level().get(), if rtl { 2 } else { 0 });
        assert_eq!(
            (unit.advance(), unit.ascent(), unit.descent()),
            (10.0, 8.0, 2.0)
        );
        for instance in unit.font_instances() {
            let instance = instance.unwrap();
            assert_eq!(instance.face().bytes(), AHEM);
            assert_eq!(instance.font_size(), 10.0);
        }
    }
}

fn glyphs(line: &text::FinalizedLine) -> Vec<(u32, f64, f64)> {
    qualify(line);
    line.run_views()
        .flat_map(|run| {
            let x = run.origin().x;
            run.glyphs()
                .map(move |glyph| (glyph.id(), x + glyph.pen(), f64::from(glyph.advance())))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn assert_strip<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    fixture: &Fixture,
    node: u32,
    start: f64,
    end: f64,
    right: bool,
) {
    let value = end - start;
    let native = output(batch, node);
    assert_eq!(
        (native.location.x, native.size.width),
        (S::from_f64(start), S::from_f64(value))
    );
    assert_eq!(
        native.border,
        if right {
            Edges::new(S::ZERO, S::from_f64(value), S::ZERO, S::ZERO)
        } else {
            Edges::new(S::ZERO, S::ZERO, S::ZERO, S::from_f64(value))
        }
    );
    assert_eq!(native.padding, Edges::ZERO);
    let line = finalized_lines(batch).remove(0);
    let occurrence = control(fixture, node);
    let slot = line
        .control_slots()
        .find(|slot| slot.control() == occurrence)
        .unwrap();
    assert_eq!(slot.strip_interval(), (start, end));
    let fragments: Vec<_> = batch
        .unrounded_inline_fragments()
        .iter()
        .filter(|entry| matches!(entry.owner(), InlineFragmentOwner::Node(n) if *n == node))
        .collect();
    assert_eq!(fragments.len(), 1);
    assert!(matches!(
        fragments[0].fragment().finalized_fragment().view().owner(),
        text::FinalizedFragmentOwner::Control(_)
    ));
    let rect = fragments[0].fragment().rect().unwrap();
    assert_eq!(
        (rect.origin().x, rect.size().width, rect.size().height),
        (S::from_f64(start), S::from_f64(value), S::from_f64(10.0))
    );
}

fn nested<S: LayoutScalar>(rtl: bool, captured_child: bool) {
    let fixture = Fixture::nested_parent("A", rtl, false, captured_child);
    source_premises(&fixture, "A", rtl, captured_child);
    let source_levels: Vec<_> = fixture
        .binding
        .prepared
        .control_views()
        .map(|row| (row.handle(), row.level().get()))
        .collect();
    // This existing Native operation deliberately claims no box geometry,
    // even though these controls have genuine captured ownership.
    let ordinary = compute::<S>(&fixture, rtl, 10.0, captured_child, false);
    let baseline = finalized_lines(&ordinary);
    assert_eq!(baseline.len(), 1);
    assert_eq!(baseline[0].feedback().advance, 10.0);
    assert_eq!(glyphs(&baseline[0]), [(35, 0.0, 10.0)]);

    let batch = compute::<S>(&fixture, rtl, 20.0, captured_child, true);
    let lines = finalized_lines(&batch);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    assert_eq!(line.prepared_text(), fixture.binding.prepared);
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(glyphs(line), [(35, if rtl { 6.0 } else { 4.0 }, 10.0)]);
    if rtl {
        // P's parent B is RTL; C's parent P is LTR. Mirror the REAL sides,
        // not the source glyph or the unsigned control labels.
        for (node, start, end, right) in [
            (21, 0.0, 4.0, false),
            (22, 4.0, 6.0, false),
            (23, 16.0, 19.0, true),
            (20, 19.0, 20.0, true),
        ] {
            assert_strip(&batch, &fixture, node, start, end, right);
        }
    } else {
        for (node, start, end, right) in [
            (20, 0.0, 1.0, false),
            (23, 1.0, 4.0, false),
            (22, 14.0, 16.0, true),
            (21, 16.0, 20.0, true),
        ] {
            assert_strip(&batch, &fixture, node, start, end, right);
        }
    }
    assert_eq!(
        fixture
            .binding
            .prepared
            .control_views()
            .map(|row| (row.handle(), row.level().get()))
            .collect::<Vec<_>>(),
        source_levels
    );
    let replay = line.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(
        replay.selected_line().request(),
        line.selected_line().request()
    );
    assert_eq!(glyphs(&replay), glyphs(line));
    for node in [20, 22, 23, 21] {
        let occurrence = control(&fixture, node);
        let bounds = |line: &text::FinalizedLine| {
            line.control_slots()
                .find(|slot| slot.control() == occurrence)
                .unwrap()
                .strip_interval()
        };
        assert_eq!(bounds(&replay), bounds(line));
    }
    // Prior products remain their own immutable operations.
    assert_eq!(glyphs(&baseline[0]), [(35, 0.0, 10.0)]);
}

#[test]
fn captured_parent_direction_places_nested_unequal_edges_in_both_scalar_lanes() {
    for rtl in [false, true] {
        nested::<f32>(rtl, true);
        nested::<f64>(rtl, true);
    }
}

#[test]
fn declared_captured_native_pair_uses_its_actual_parent_in_both_scalar_lanes() {
    for rtl in [false, true] {
        nested::<f32>(rtl, false);
        nested::<f64>(rtl, false);
    }
}

fn tab_prefix<S: LayoutScalar>(captured_child: bool) {
    let fixture = Fixture::nested_parent("A\tB", false, true, captured_child);
    source_premises(&fixture, "A\tB", false, captured_child);
    let ordinary = compute::<S>(&fixture, false, 30.0, captured_child, false);
    let baseline = finalized_lines(&ordinary).remove(0);
    assert_eq!(baseline.feedback().advance, 30.0);
    assert_eq!(glyphs(&baseline), [(35, 0.0, 10.0), (36, 20.0, 10.0)]);

    let batch = compute::<S>(&fixture, false, 36.0, captured_child, true);
    let full = finalized_lines(&batch).remove(0);
    // Actual grid20: Ps1 + Ce3 + A10 puts TAB at14; it executes6 to20.
    // B10, Cs2 and Pe4 then finish36. These are actual full receipts.
    assert_eq!(full.feedback().advance, 36.0);
    assert_eq!(glyphs(&full), [(35, 4.0, 10.0), (36, 20.0, 10.0)]);
    assert_strip(&batch, &fixture, 20, 0.0, 1.0, false);
    assert_strip(&batch, &fixture, 23, 1.0, 4.0, false);
    assert_strip(&batch, &fixture, 22, 30.0, 32.0, true);
    assert_strip(&batch, &fixture, 21, 32.0, 36.0, true);
    let cs = control(&fixture, 22);
    assert_eq!(
        full.prefix_at(&text::TextSeam::AfterParticipant(
            text::PreparedParticipant::Control(cs),
        ))
        .unwrap()
        .advance(),
        3.0,
        "logical Ps1+Cs2 despite Cs being physically at30"
    );
    let origin = fixture.binding.origins[0].origin.clone();
    let seam = text::TextSeam::Origin {
        origin: origin.clone(),
        local: text::SourcePosition::try_new(origin.view().source().text(), 2).unwrap(),
    };
    assert_eq!(
        full.prefix_at(&seam).unwrap().advance(),
        19.0,
        "logical Ps1+Cs2+A10+the ACTUAL full TAB6, not physical prefix20"
    );

    let prepared = &fixture.binding.prepared;
    let end = prepared
        .opportunity_views()
        .find(|offer| {
            offer.kind() == text::PreparedBreakKind::Allowed
                && offer.boundary().view().source_before().get() == 2
                && offer.boundary().view().source_after().get() == 2
        })
        .expect("genuine preserved-TAB soft opportunity, not an invented boundary")
        .handle();
    let mut recipe = full.selected_line().request().clone();
    recipe.end = end.view().boundary();
    recipe.termination = text::PreparedTermination::Ordinary(end);
    recipe.context.end = text::LineEnd::Soft;
    let count = recipe.end.view().participant_offset();
    recipe.participants = prepared.participants().take(count).collect();
    recipe.controls.retain(|row| {
        recipe.participants.iter().any(|participant|
        matches!(participant,text::PreparedParticipant::Control(c) if c == &row.occurrence))
    });
    recipe.boundary_context.clear();
    assert_eq!(recipe.controls.len(), 2);
    assert!(
        recipe
            .controls
            .iter()
            .all(|row| row.occurrence == control(&fixture, 20)
                || row.occurrence == control(&fixture, 22))
    );
    // Supply only context actually requested by the existing source owner.
    // Context keeps genuine BoxEdge identity/amount; it never schedules flow.
    while let Some(needed) = prepared.boundary_context_demand(&recipe).unwrap() {
        assert!(
            !recipe
                .boundary_context
                .iter()
                .any(|row| row.occurrence == needed)
        );
        let allocation = full
            .selected_line()
            .request()
            .controls
            .iter()
            .find(|row| row.occurrence == needed)
            .unwrap()
            .boundary
            .clone()
            .unwrap();
        recipe
            .boundary_context
            .push(text::InlineBoundaryContextAllocation {
                occurrence: needed,
                boundary: allocation,
            });
    }
    let partial = prepared
        .select(recipe)
        .unwrap()
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: text::ScopeAllocations::None,
        })
        .unwrap();
    // NEW execution: Ps1,A10 ->TAB at11 executes9 to20, then selected Cs2.
    // Neither genuinely unselected Ce3 nor Pe4 contributes. No old TAB receipt
    // is imported into this independently selected recipe.
    assert_eq!(partial.feedback().advance, 22.0);
    assert_eq!(glyphs(&partial), [(35, 1.0, 10.0)]);
    assert_eq!(partial.control_slots().count(), 2);
    assert_eq!(full.prefix_at(&seam).unwrap().advance(), 19.0);
    let replay = full.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(replay.prefix_at(&seam).unwrap().advance(), 19.0);
    assert_eq!(glyphs(&replay), glyphs(&full));
    assert_eq!(baseline.feedback().advance, 30.0);
}

#[test]
fn frozen_logical_tab_prefix_and_new_partial_recipe_keep_their_own_executions() {
    for captured_child in [true, false] {
        tab_prefix::<f32>(captured_child);
        tab_prefix::<f64>(captured_child);
    }
}

#[path = "parent_geometry/source_ranges.rs"]
mod source_ranges;

#[path = "parent_geometry_capture_admission.rs"]
mod capture_admission;

#[path = "parent_geometry/signed_tab.rs"]
mod signed_tab;

#[path = "parent_geometry/signed_physical_fold.rs"]
mod signed_physical_fold;
