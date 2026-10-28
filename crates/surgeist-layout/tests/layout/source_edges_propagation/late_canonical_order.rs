//! Ordinary late admission must reconcile native pairs with a real empty scope.
//!
//! Reuses only that fixture's authentic host helpers; all product calls are public.
use super::*;

fn prepare_direct(processed: &Arc<t::ProcessedSource>) -> t::PreparedText {
    let a = t::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        t::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = a
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| t::WrappingPolicy {
            relation: r.handle(),
            ancestor_wrap: t::Wrap::None,
        })
        .collect();
    let a = a.with_wrapping_policies(policies).unwrap();
    let mut fonts = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    fonts.register_fonts(AHEM.to_vec()).unwrap();
    fonts.prepare(&a, t::PreparationOptions::default()).unwrap()
}

fn assert_late_geometry<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    controls: &[t::PreparedControl],
) {
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().id(), Some(t::Id::from_u64(658000)));
    assert_eq!(placed.source().revision(), 21);
    assert_eq!(placed.source().text(), &t::Text::from("PAQ"));
    assert_eq!(
        placed.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
        [t::Range::new(0, 1), t::Range::new(1, 3)]
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.baseline())
            .collect::<Vec<_>>(),
        [8.0, 18.0]
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.bounds().size.height)
            .collect::<Vec<_>>(),
        [10.0, 10.0]
    );
    let root = batch
        .final_entries()
        .iter()
        .find(|e| e.node() == 0)
        .unwrap()
        .output();
    assert_eq!(
        root.size,
        l::Size::new(S::from_f64(100.0), S::from_f64(20.0))
    );
    let mut lines = vec![None, None];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        lines[fragment.line_index()] = Some(fragment.finalized_fragment().finalized_line());
    }
    let mut sourced = vec![];
    for (i, line) in lines.into_iter().enumerate() {
        let line = line.unwrap();
        assert_eq!(line.measurements().fit().advance(), [15.0, 29.0][i]);
        let selected = line.selected_line();
        let request = selected.request();
        let mut selected_controls = request
            .controls
            .iter()
            .map(|allocation| {
                let ordinal = controls
                    .iter()
                    .position(|control| *control == allocation.occurrence)
                    .unwrap();
                assert_eq!(
                    allocation.strut,
                    Some(t::ControlStrut {
                        baseline: 8.0,
                        line_extent: 10.0,
                        alignment: t::LineBoxAlignment::Baseline
                    })
                );
                assert_eq!(allocation.boundary.unwrap().advance(), ADVANCES[ordinal]);
                ordinal
            })
            .collect::<Vec<_>>();
        selected_controls.sort_unstable();
        assert_eq!(
            selected_controls,
            if i == 0 { vec![0] } else { vec![1, 2, 3] }
        );
        assert_eq!(
            request.line_box_struts,
            if i == 0 {
                vec![t::LineBoxStrut {
                    metrics: t::LineStrut {
                        baseline: 8.0,
                        line_extent: 10.0,
                    },
                    alignment: t::LineBoxAlignment::Baseline,
                }]
            } else {
                vec![]
            }
        );
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            for glyph in run.glyphs() {
                sourced.push((glyph.range(), glyph.id(), glyph.advance()));
            }
        }
    }
    assert_eq!(
        sourced,
        [
            (t::Range::new(0, 1), 50, 10.0),
            (t::Range::new(1, 2), 35, 10.0),
            (t::Range::new(2, 3), 51, 10.0)
        ]
    );
    let mut posed = vec![];
    for run in placed.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.font_size(), 10.0);
        for glyph in run.glyphs() {
            posed.push((
                glyph.id(),
                glyph.advance(),
                run.origin().x + glyph.pen() + f64::from(glyph.x()),
                run.origin().y,
            ));
        }
    }
    assert_eq!(
        posed,
        [
            (50, 10.0, 0.0, 8.0),
            (35, 10.0, 3.0, 18.0),
            (51, 10.0, 19.0, 18.0)
        ]
    );
}

fn exercise<S: l::LayoutScalar>(false_order: bool) {
    let mut input = source(Case::EmptyBefore);
    // A genuine lower Text cause, not a claim that Layout Line declarations or
    // the early adapter ran. There is no ResolvedWrapEdges or early-source call.
    input
        .computed_line_boundaries
        .push(t::ComputedLineBoundaryInput::Scope {
            scope: 2,
            side: t::SourceBoxSide::Before,
        });
    let processed = Arc::new(t::process_contextual_source(input).unwrap());
    assert_eq!(processed.source().text(), &t::Text::from("PAQ"));
    let outer = processed.captured_scope(1).unwrap();
    let inner = processed.captured_scope(2).unwrap();
    let empty = processed.captured_scope(3).unwrap();
    assert_eq!(empty.parent(), Some(outer.clone()));
    assert_eq!(inner.parent(), Some(outer.clone()));
    assert!(empty.exit_position() < inner.enter_position());
    assert!(empty.entry_control().is_none());
    assert!(empty.exit_control().is_none());
    for (scope, start, end) in [(outer, 0, 3), (inner.clone(), 1, 2)] {
        assert_eq!(scope.entry_control(), processed.captured_control(start));
        assert_eq!(scope.exit_control(), processed.captured_control(end));
    }
    assert!(processed.captured_control(4).is_none());
    let cause = processed.captured_computed_line_boundary(0).unwrap();
    assert_eq!(
        cause.target(),
        t::CapturedComputedLineBoundaryTarget::Scope {
            scope: inner,
            side: t::SourceBoxSide::Before
        }
    );
    assert!(processed.captured_computed_line_boundary(1).is_none());
    let prepared = prepare_direct(&processed);
    assert_eq!(prepared.control_views().count(), 4);
    let offers = prepared
        .opportunity_views()
        .map(|o| (o.kind(), o.boundary().view().source_before().get()))
        .collect::<Vec<_>>();
    assert_eq!(
        offers,
        [
            (t::PreparedBreakKind::Forced, 1),
            (t::PreparedBreakKind::ParagraphEnd, 3)
        ]
    );
    let forced = prepared
        .opportunity_views()
        .find(|o| o.kind() == t::PreparedBreakKind::Forced)
        .unwrap();
    assert_eq!(
        forced
            .computed_line_boundary()
            .unwrap()
            .view()
            .causes()
            .collect::<Vec<_>>(),
        [cause]
    );
    let mut host = Host::<S>::new(Case::EmptyBefore);
    // All actual normalized Layout declarations are neutral. Only Text's
    // independent computed mandatory capability is being consumed here.
    for declaration in &mut host.declarations {
        declaration.edges = l::WrapEdges::AUTO;
    }
    let binding = host.install(&processed, &prepared);
    let controls = binding
        .controls
        .iter()
        .map(|c| c.occurrence.clone())
        .collect::<Vec<_>>();
    let mut wrapping = host.final_wrapping(&processed);
    for scope in &mut wrapping.scopes {
        match scope {
            l::WrappingScopeInput::Captured { edges, .. }
            | l::WrappingScopeInput::NativePair { edges, .. } => *edges = l::WrapEdges::AUTO,
        }
    }
    let siblings = wrapping
        .child_order
        .iter_mut()
        .find(|row| {
            row.parent
                == l::GeneratedBoxRef::Inline(l::InlineBoxRef::NativePair { start: 2, end: 6 })
        })
        .unwrap();
    assert_eq!(siblings.children.len(), 2);
    assert_eq!(
        siblings.children[0].box_,
        l::GeneratedBoxRef::Inline(l::InlineBoxRef::Scope(empty))
    );
    assert_eq!(
        siblings.children[1].box_,
        l::GeneratedBoxRef::Inline(l::InlineBoxRef::NativePair { start: 3, end: 5 })
    );
    if false_order {
        siblings.children.swap(0, 1);
    }
    let tree = Final(&host);
    let result = l::LayoutFlowInput::try_new(&tree, 0, vec![binding], wrapping);
    if false_order {
        let error = result
            .err()
            .expect("complete [T,E] contradicts genuine captured [E,T]");
        assert!(
            matches!(
                error.kind(),
                l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::WrappingScope(
                    l::WrappingScopeError::ChildOrder
                ))
            ),
            "{error:?}"
        );
    } else {
        let flow = result
            .expect("genuine complete [E,T] must admit canonical empty scope with native sibling");
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(100.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        let batch = l::compute_layout_with_flow(&flow, 0, request).unwrap();
        assert_late_geometry(&batch, &controls);
    }
}

#[test]
fn ordinary_late_native_pair_accepts_complete_captured_empty_then_native_child_order() {
    exercise::<f32>(false);
    exercise::<f64>(false);
}

#[test]
fn ordinary_late_native_pair_rejects_complete_order_reversing_captured_empty_sibling() {
    exercise::<f32>(true);
    exercise::<f64>(true);
}
