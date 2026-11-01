//! Explicit producer facts for owning fixtures with a known containing context.

/// Returns the genuine segment only for a fixture with one structural section.
pub(crate) fn only_source_segment(prepared: &crate::PreparedText) -> crate::PreparedSourceSegment {
    let mut segments = prepared.source_segments();
    let segment = segments.next().expect("fixture has a source segment");
    assert!(
        segments.next().is_none(),
        "fixture has exactly one source segment"
    );
    segment
}

pub(crate) fn bind_missing(
    analyzed: &crate::AnalyzedSource,
    ancestor_wrap: crate::Wrap,
) -> crate::AnalyzedSource {
    assert!(analyzed.inner.paragraph.base_direction.is_some());
    let policies = analyzed
        .wrapping_relations()
        .filter(|view| view.ancestor_wrap().is_none())
        .map(|view| crate::WrappingPolicy {
            relation: view.handle(),
            ancestor_wrap,
        })
        .collect();
    analyzed.with_wrapping_policies(policies).unwrap()
}

/// Owning legacy fixtures explicitly model neutral inline endpoints, including
/// endpoints outside their selected line. Supply only facts actually demanded.
pub(crate) fn select_neutral(
    prepared: &crate::PreparedText,
    mut request: crate::LineSelectionRequest,
) -> crate::Result<crate::SelectedLine> {
    while let Some(occurrence) = prepared.boundary_context_demand(&request)? {
        request
            .boundary_context
            .push(crate::InlineBoundaryContextAllocation {
                occurrence,
                boundary: crate::InlineBoundaryAllocation::CONTRIBUTION_ZERO,
            });
    }
    prepared.select(request)
}
