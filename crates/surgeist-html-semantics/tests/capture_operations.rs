use surgeist_html::*;
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureDefinitionId, HtmlCaptureError,
    HtmlCaptureFact, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCaptureTemplateCacheInput,
    HtmlCapturedIsValue, HtmlCapturedRegistry, HtmlCapturedShadowProperties,
    HtmlCapturedTemplateCache, HtmlCustomElementDefinition, HtmlDefinitionLookupCursor,
    HtmlDefinitionLookupStep, HtmlDocumentClass, HtmlScriptingStatus, HtmlTemplateDocumentRole,
};

fn builder() -> HtmlCaptureBuilder {
    HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(16, 32, 16, 512, 16, 4, 8),
        HtmlHostFactsRevision::fresh(),
    )
}

fn document(builder: &mut HtmlCaptureBuilder) -> HtmlCaptureBuilderSubject {
    builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap()
}

fn lookup(cursor: &mut HtmlDefinitionLookupCursor<'_>) -> (usize, Option<HtmlCaptureDefinitionId>) {
    let mut work = 0;
    for _ in 0..64 {
        work += cursor.next_cost();
        let transfers_registry = cursor.next_clones_registry();
        if let HtmlDefinitionLookupStep::Ready(result) = cursor.step() {
            assert_eq!(transfers_registry, result.is_some());
            return (work, result);
        }
        assert!(!transfers_registry);
    }
    panic!("the fixed six-unit lookup did not finish");
}

#[test]
fn ordered_lookup_preserves_exact_comparisons_and_original_registry_identity() {
    for (definition_name, local, is_value, expected_work) in [
        ("x-host", "x-host", "x-other", 27),
        ("x-good", "button", "x-good", 31),
    ] {
        let mut builder = builder();
        let handle = builder
            .add_registry(Some(vec![HtmlCustomElementDefinition::new(
                definition_name.into(),
                local.into(),
                false,
                false,
            )]))
            .unwrap();
        let capture = builder.finish().unwrap();
        let registry_id = capture.registry_id(&handle).unwrap();
        let registry = HtmlCapturedRegistry::Registry(registry_id.clone());
        let local = DocumentString::from(local);
        let is_value = DocumentString::from(is_value);
        let mut cursor = capture
            .definition_lookup_cursor(&registry, &local, Some(&is_value))
            .unwrap();
        let (work, result) = lookup(&mut cursor);
        // Two six-unit comparisons plus Ready: 27. The customized case
        // first spends three on a mismatch and one on the second search phase.
        assert_eq!(work, expected_work);
        let result = result.unwrap();
        assert_eq!(result.registry(), &registry_id);
        assert_eq!(
            capture.definition(&result).unwrap().name(),
            &DocumentString::from(definition_name)
        );
    }
}

#[test]
fn empty_null_unavailable_and_foreign_registry_operands_remain_distinct() {
    let mut builder = builder();
    let empty = builder.add_registry(Some(Vec::new())).unwrap();
    let unavailable = builder.add_registry(None).unwrap();
    let capture = builder.finish().unwrap();
    let empty = HtmlCapturedRegistry::Registry(capture.registry_id(&empty).unwrap());
    let unavailable_id = capture.registry_id(&unavailable).unwrap();
    let unavailable = HtmlCapturedRegistry::Registry(unavailable_id.clone());
    let local = DocumentString::from("button");
    let is_value = DocumentString::from("x-good");
    assert_eq!(
        lookup(
            &mut capture
                .definition_lookup_cursor(&empty, &local, Some(&is_value))
                .unwrap()
        ),
        (3, None)
    );
    assert_eq!(
        lookup(
            &mut capture
                .definition_lookup_cursor(&HtmlCapturedRegistry::Null, &local, None)
                .unwrap()
        ),
        (1, None)
    );
    assert!(matches!(
        capture.definition_lookup_cursor(&HtmlCapturedRegistry::Unavailable, &local, None),
        Err(HtmlCaptureError::MissingFact {
            fact: HtmlCaptureFact::Registry,
            ..
        })
    ));
    assert!(matches!(
        capture.definition_lookup_cursor(&unavailable, &local, None),
        Err(HtmlCaptureError::UnavailableDefinitions(id)) if id == unavailable_id
    ));
    let foreign = self::builder().finish().unwrap();
    assert!(matches!(
        foreign.definition_lookup_cursor(&empty, &local, None),
        Err(HtmlCaptureError::ForeignCapture)
    ));
}

#[test]
fn typed_document_cache_preserves_all_three_states_and_rejects_foreign_identity() {
    let mut builder = builder();
    let unknown = document(&mut builder);
    let absent = document(&mut builder);
    let cached = document(&mut builder);
    let inert = document(&mut builder);
    for (subject, role, cache) in [
        (
            &absent,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        ),
        (
            &inert,
            HtmlTemplateDocumentRole::InertTemplate,
            HtmlCaptureTemplateCacheInput::Absent,
        ),
        (
            &cached,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Document(inert.clone()),
        ),
    ] {
        builder
            .establish_document_template_facts(subject, HtmlDocumentClass::Html, role, cache)
            .unwrap();
    }
    let capture = builder.finish().unwrap();
    let id = |handle| {
        capture
            .document_id(&capture.subject_id(handle).unwrap())
            .unwrap()
    };
    assert_eq!(
        capture.template_document_cache(&id(&unknown)),
        Ok(HtmlCapturedTemplateCache::Unavailable)
    );
    assert_eq!(
        capture.template_document_cache(&id(&absent)),
        Ok(HtmlCapturedTemplateCache::Absent)
    );
    assert_eq!(
        capture.template_document_cache(&id(&cached)),
        Ok(HtmlCapturedTemplateCache::Document(id(&inert)))
    );
    let foreign = self::builder().finish().unwrap();
    assert_eq!(
        foreign.template_document_cache(&id(&unknown)),
        Err(HtmlCaptureError::ForeignCapture)
    );
}

#[test]
fn shadow_presence_preserves_unknown_absence_and_closed_root_presence() {
    let mut builder = builder();
    let document = document(&mut builder);
    let mut hosts = Vec::new();
    for local in ["div", "section", "span"] {
        hosts.push(
            builder
                .add_element(
                    &document,
                    QualifiedName::new(Namespace::html(), None, DocumentString::from(local)),
                    OrderedAttributes::default(),
                    HtmlCapturedIsValue::Absent,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap(),
        );
    }
    builder.establish_no_shadow_root(&hosts[1]).unwrap();
    builder
        .add_shadow_root(
            &hosts[2],
            HtmlCapturedShadowProperties::new(
                HtmlShadowRootProperties::new(
                    HtmlShadowRootMode::Closed,
                    HtmlSlotAssignment::Manual,
                    true,
                    false,
                    true,
                ),
                true,
                false,
                false,
            ),
            HtmlCaptureRegistryInput::Unavailable,
        )
        .unwrap();
    let capture = builder.finish().unwrap();
    for (host, expected) in hosts.iter().zip([None, Some(false), Some(true)]) {
        let id = capture
            .element_id(&capture.subject_id(host).unwrap())
            .unwrap();
        assert_eq!(capture.shadow_presence(&id), Ok(expected));
        let foreign = self::builder().finish().unwrap();
        assert_eq!(
            foreign.shadow_presence(&id),
            Err(HtmlCaptureError::ForeignCapture)
        );
    }
}
