//! Explicit domain font context and limits do not classify an empty inline box.
//! Source custody remains genuine even when no scope frame is demanded.
use super::*;

fn context_only<S: LayoutScalar>() {
    let limits = text::ScopeAlignmentLimits::new(4096, 4096, 1_000_000, 16_777_216);
    let reference = text::ScopeFontReference {
        script: text::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: text::FontBaselineAxis::HorizontalY,
        baseline: text::FontBaseline::Alphabetic,
    };
    for direction in [Direction::Ltr, Direction::Rtl] {
        let fixture = EmptyBox::new_with_context(direction, Some(limits), Some(reference));
        assert_eq!(fixture.scope.alignment_boundary(), None);
        assert_eq!(fixture.scope.font_reference(), None);
        assert_eq!(fixture.scope.scope_alignment_limits(), Some(limits));
        assert_eq!(fixture.scope.domain().font_reference(), Some(reference));
        assert_eq!(fixture.scope.parent(), Some(fixture.scope.domain()));
        assert!(
            fixture
                .binding
                .prepared
                .scope_alignment_retention()
                .is_some()
        );
        let prepared_usage = fixture.binding.prepared.scope_alignment_usage();
        assert_eq!(prepared_usage.unwrap().font_queries(), 0);
        assert_eq!(
            fixture
                .scope
                .scope_alignment_usage()
                .unwrap()
                .font_queries(),
            0
        );

        let supplied = text::LineBoxStrut {
            scope: None,
            metrics: text::LineStrut {
                baseline: 16.0,
                line_extent: 20.0,
            },
            alignment: text::LineBoxAlignment::Baseline,
        };
        // The authentic WrappingScopeInput::Captured box supplies this owner.
        // A canonical request keeps that source fact without treating it as an
        // authored alignment classification or demanding its absent font facts.
        let canonical = text::LineBoxStrut {
            scope: Some(fixture.scope.clone()),
            ..supplied.clone()
        };
        // Ahem10 has 8/2 metrics. The supplied empty box has 16/4 metrics, so
        // its assigned line is 20 high with baseline16; the other is 10 high
        // with baseline8. Stacking yields these independent old native oracles.
        for (edges, baselines, heights, owning_line) in [
            (
                WrapEdges {
                    before: WrapEdge::Avoid,
                    after: WrapEdge::Auto,
                },
                [16.0, 28.0],
                [20.0, 10.0],
                0,
            ),
            (
                WrapEdges {
                    before: WrapEdge::Auto,
                    after: WrapEdge::Avoid,
                },
                [8.0, 26.0],
                [10.0, 20.0],
                1,
            ),
        ] {
            // EmptyBox::run passes normalized alignment rows []. The child has
            // real EnterScope/ExitScope events but no source glyph or endpoint.
            let batch = fixture.run::<S>(
                edges,
                Some(InlineBoxMetricContribution::Strut(supplied.clone())),
            );
            assert_result(
                &batch,
                &fixture,
                baselines,
                heights,
                canonical.clone(),
                owning_line,
            );
            let mut seen = [false; 2];
            for entry in batch.final_inline_fragments() {
                let fragment = entry.fragment();
                let index = fragment.line_index();
                seen[index] = true;
                let line = fragment.finalized_fragment().finalized_line();
                let selected = line.selected_line();
                assert_eq!(selected.scopes().len(), 0);
                assert!(line.scope_allocations().is_empty());
                assert_eq!(selected.scope_alignment_usage().unwrap().font_queries(), 0);
                assert_eq!(line.scope_alignment_usage().unwrap().font_queries(), 0);
                if index == owning_line {
                    assert_eq!(
                        selected.request().line_box_struts.as_slice(),
                        std::slice::from_ref(&canonical)
                    );
                    assert_eq!(
                        line.line_box_strut_scope_displacement(0).unwrap().get(),
                        0.0
                    );
                } else {
                    assert!(selected.request().line_box_struts.is_empty());
                }
            }
            assert_eq!(seen, [true, true]);
            assert_eq!(
                fixture.binding.prepared.scope_alignment_usage(),
                prepared_usage
            );
            assert_eq!(fixture.scope.alignment_boundary(), None);
            assert_eq!(fixture.scope.font_reference(), None);
        }
    }
}

#[test]
fn domain_font_context_with_limits_preserves_unclassified_empty_box_geometry_and_source_custody() {
    context_only::<f32>();
    context_only::<f64>();
}
