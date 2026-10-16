use surgeist_html::DocumentString;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureError, HtmlCapturedDefinitions, HtmlCapturedRegistry,
    HtmlDefinitionLookupStep, HtmlEmptyGlobalRegistryError, HtmlHostFactsRevision,
    HtmlRegistryScope,
};

#[test]
fn complete_empty_global_registry_is_a_real_qualified_known_miss() {
    let revision = HtmlHostFactsRevision::fresh();
    let formed =
        HtmlCapture::complete_empty_global_registry_with(revision.clone(), usize::MAX, |_| {
            Ok::<_, ()>(())
        })
        .unwrap();
    let capture = formed.capture();
    assert_eq!(capture.host_facts_revision(), &revision);
    assert_eq!(capture.usage().subjects(), 0);
    assert_eq!(capture.usage().registries(), 1);
    assert_eq!(capture.usage().definitions(), 0);
    assert_eq!(
        capture.registry_scope(formed.registry()),
        Ok(HtmlRegistryScope::Global)
    );
    assert!(matches!(
        capture.definitions(formed.registry()),
        Ok(HtmlCapturedDefinitions::Complete([]))
    ));
    assert_eq!(
        formed.retained_storage_usage(),
        capture
            .retained_storage_usage_with(|| Ok::<_, ()>(()))
            .unwrap()
    );
    assert_eq!(formed.retained_storage_usage().utf16_extent(), 0);
    assert!(formed.formation_peak_bytes() > formed.retained_storage_usage().retained_bytes());

    // The actual registry is non-null. Both autonomous and customized-name
    // searches run the shared ordered lookup and establish an actual miss.
    let registry = HtmlCapturedRegistry::Registry(formed.registry().clone());
    let local = DocumentString::from("button");
    let is_value = DocumentString::from("x-button");
    let mut cursor = capture
        .definition_lookup_cursor(&registry, &local, Some(&is_value))
        .unwrap();
    assert!(matches!(cursor.step(), HtmlDefinitionLookupStep::Searching));
    assert!(matches!(cursor.step(), HtmlDefinitionLookupStep::Searching));
    assert!(matches!(
        cursor.step(),
        HtmlDefinitionLookupStep::Ready(None)
    ));

    let other =
        HtmlCapture::complete_empty_global_registry_with(revision, usize::MAX, |_| Ok::<_, ()>(()))
            .unwrap();
    assert_ne!(capture.identity(), other.capture().identity());
    assert_eq!(
        other.capture().registry_scope(formed.registry()),
        Err(HtmlCaptureError::ForeignCapture)
    );
    let expected = formed.registry().clone();
    let (capture, registry) = formed.into_parts();
    assert_eq!(registry, expected);
    assert!(matches!(
        capture.definitions(&registry),
        Ok(HtmlCapturedDefinitions::Complete([]))
    ));
}

#[test]
fn formation_peak_is_an_admitted_boundary_not_a_post_creation_estimate() {
    let formed = HtmlCapture::complete_empty_global_registry_with(
        HtmlHostFactsRevision::fresh(),
        usize::MAX,
        |_| Ok::<_, ()>(()),
    )
    .unwrap();
    let peak = formed.formation_peak_bytes();
    let expected = formed.retained_storage_usage();
    let exact = HtmlCapture::complete_empty_global_registry_with(
        HtmlHostFactsRevision::fresh(),
        peak,
        |_| Ok::<_, ()>(()),
    )
    .unwrap();
    assert_eq!(exact.retained_storage_usage(), expected);
    assert_eq!(exact.formation_peak_bytes(), peak);
    assert!(matches!(
        HtmlCapture::complete_empty_global_registry_with(
            HtmlHostFactsRevision::fresh(), peak - 1, |_| Ok::<_, ()>(()),
        ),
        Err(HtmlEmptyGlobalRegistryError::FormationBytes { limit, required })
            if limit == peak - 1 && required == peak
    ));
    assert!(matches!(
        HtmlCapture::complete_empty_global_registry_with(
            HtmlHostFactsRevision::fresh(), 0, |_| Ok::<_, ()>(()),
        ),
        Err(HtmlEmptyGlobalRegistryError::FormationBytes { limit: 0, required }) if required > 0
    ));
}

#[test]
fn every_work_or_cancellation_refusal_preserves_the_original_first_error() {
    let mut total = 0;
    HtmlCapture::complete_empty_global_registry_with(
        HtmlHostFactsRevision::fresh(),
        usize::MAX,
        |units| {
            total += units;
            Ok::<_, usize>(())
        },
    )
    .unwrap();
    assert!(total > 0);
    for limit in 0..total {
        let mut used = 0;
        let result = HtmlCapture::complete_empty_global_registry_with(
            HtmlHostFactsRevision::fresh(),
            usize::MAX,
            |units| {
                if used + units > limit {
                    return Err((used, units));
                }
                used += units;
                Ok(())
            },
        );
        assert!(
            matches!(result, Err(HtmlEmptyGlobalRegistryError::Interrupted((before, cost)))
            if before == used && before + cost > limit)
        );
    }
    // A pending cancellation is first, even with an insufficient byte budget.
    assert!(matches!(
        HtmlCapture::complete_empty_global_registry_with(HtmlHostFactsRevision::fresh(), 0, |_| {
            Err("cancelled")
        },),
        Err(HtmlEmptyGlobalRegistryError::Interrupted("cancelled"))
    ));
}
