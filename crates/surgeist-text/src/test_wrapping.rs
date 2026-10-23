//! Explicit producer facts for owning fixtures with a known containing context.

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
