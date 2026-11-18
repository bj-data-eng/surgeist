//! Explicit producer facts for owning fixtures with a known containing context.

/// Returns the segment only for a fixture with one structural section.
pub(crate) fn only_source_segment(
    prepared: &crate::protocol::PreparedText,
) -> crate::protocol::PreparedSourceSegment {
    let mut segments = prepared.source_segments();
    let segment = segments.next().expect("fixture has a source segment");
    assert!(
        segments.next().is_none(),
        "fixture has exactly one source segment"
    );
    segment
}

pub(crate) fn bind_missing(
    analyzed: &crate::analyzed::AnalyzedSource,
    ancestor_wrap: crate::style::Wrap,
) -> crate::analyzed::AnalyzedSource {
    assert!(analyzed.inner.paragraph.base_direction.is_some());
    let policies = analyzed
        .wrapping_relations()
        .filter(|view| view.ancestor_wrap().is_none())
        .map(|view| crate::wrapping::WrappingPolicy {
            relation: view.handle(),
            ancestor_wrap,
        })
        .collect();
    analyzed.with_wrapping_policies(policies).unwrap()
}

/// Owning legacy fixtures explicitly model neutral inline endpoints, including
/// endpoints outside their selected line. Supply only facts actually demanded.
pub(crate) fn select_neutral(
    prepared: &crate::protocol::PreparedText,
    mut request: crate::protocol::LineSelectionRequest,
) -> crate::Result<crate::protocol::SelectedLine> {
    while let Some(occurrence) = prepared.boundary_context_demand(&request)? {
        request
            .boundary_context
            .push(crate::protocol::InlineBoundaryContextAllocation {
                occurrence,
                boundary: crate::protocol::InlineBoundaryAllocation::CONTRIBUTION_ZERO,
            });
    }
    prepared.select(request)
}
