use surgeist_html::values::{DocumentString, TextBlock};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureError, HtmlCaptureFact, HtmlCaptureLimits,
    HtmlCaptureRegistryId, HtmlCapturedDefinitions, HtmlCapturedRegistry,
    HtmlCustomElementDefinition, HtmlDefinitionLookupStep, HtmlHostFactsRevision,
    HtmlRegistryCapture, HtmlRegistryFormationError, HtmlRegistryScope,
};

fn global() -> HtmlRegistryCapture {
    HtmlCapture::complete_empty_global_registry_with(
        HtmlHostFactsRevision::fresh(),
        usize::MAX,
        |_| Ok::<_, ()>(()),
    )
    .unwrap()
}

fn definition(name: &str, local: &str) -> HtmlCustomElementDefinition {
    HtmlCustomElementDefinition::new(
        DocumentString::from(name),
        DocumentString::from(local),
        false,
        false,
    )
}

fn append(
    original: &HtmlRegistryCapture,
    definition: &HtmlCustomElementDefinition,
) -> HtmlRegistryCapture {
    original
        .capture()
        .append_registry_definition_with(
            original.registry(),
            definition,
            HtmlHostFactsRevision::fresh(),
            usize::MAX,
            |_| Ok::<_, ()>(()),
        )
        .unwrap()
}

#[test]
fn append_preserves_previous_evidence_and_lookup_order() {
    let empty = global();
    let autonomous = definition("x-entry", "x-entry");
    let first = append(&empty, &autonomous);
    let custom = HtmlCustomElementDefinition::new(
        DocumentString::from("x-button"),
        DocumentString::from("button"),
        true,
        true,
    );
    let next = append(&first, &custom);
    for (formed, expected) in [
        (&empty, &[][..]),
        (&first, std::slice::from_ref(&autonomous)),
        (&next, &[autonomous.clone(), custom.clone()][..]),
    ] {
        assert!(matches!(
            formed.capture().definitions(formed.registry()),
            Ok(HtmlCapturedDefinitions::Complete(records)) if records == expected
        ));
        assert_eq!(formed.capture().usage().definitions(), expected.len());
        assert_eq!(
            formed.capture().registry_scope(formed.registry()),
            Ok(HtmlRegistryScope::Global)
        );
    }

    let registry = HtmlCapturedRegistry::Registry(next.registry().clone());
    for (local, is, expected) in [
        ("x-entry", None, Some((0, &autonomous))),
        ("button", Some("x-button"), Some((1, &custom))),
        ("button", None, None),
        ("input", Some("x-button"), None),
    ] {
        let local = DocumentString::from(local);
        let is = is.map(DocumentString::from);
        let mut lookup = next
            .capture()
            .definition_lookup_cursor(&registry, &local, is.as_ref())
            .unwrap();
        let found = loop {
            if let HtmlDefinitionLookupStep::Ready(found) = lookup.step() {
                break found;
            }
        };
        assert_eq!(
            found.as_ref().map(|id| {
                assert_eq!(id.registry(), next.registry());
                assert_eq!(
                    first.capture().definition(id),
                    Err(HtmlCaptureError::ForeignCapture)
                );
                (id.ordinal(), next.capture().definition(id).unwrap())
            }),
            expected
        );
    }
}

#[test]
fn borrowed_ranges_keep_exact_units_and_obey_the_formation_frontier() {
    let original = global();
    let mut source = vec![0; 4096];
    source[19..23].copy_from_slice(&[120, 45, 97, 0xd800]);
    let block = TextBlock::from_utf16(source);
    let name = block.string(19..23).unwrap();
    let definition = HtmlCustomElementDefinition::new(name.clone(), name, false, false);
    let attempt = |bytes, step: &mut dyn FnMut(usize) -> Result<(), usize>| {
        original.capture().append_registry_definition_with(
            original.registry(),
            &definition,
            HtmlHostFactsRevision::fresh(),
            bytes,
            step,
        )
    };
    let mut charges = Vec::new();
    let formed = attempt(usize::MAX, &mut |units| {
        charges.push(units);
        Ok(())
    })
    .unwrap();
    let HtmlCapturedDefinitions::Complete(records) =
        formed.capture().definitions(formed.registry()).unwrap()
    else {
        panic!("complete registry");
    };
    assert_eq!(records[0].name().code_units(), &[120, 45, 97, 0xd800]);
    assert!(records[0].name().is_whole_block());
    assert!(records[0].local_name().is_whole_block());
    assert_eq!(formed.retained_storage_usage().utf16_extent(), 8);
    assert_eq!(
        formed.retained_storage_usage(),
        formed
            .capture()
            .retained_storage_usage_with(|| Ok::<_, ()>(()))
            .unwrap()
    );
    let peak = formed.formation_peak_bytes();
    assert_eq!(
        attempt(peak, &mut |_| Ok(()))
            .unwrap()
            .formation_peak_bytes(),
        peak
    );
    assert!(matches!(
        attempt(peak - 1, &mut |_| Ok(())),
        Err(HtmlRegistryFormationError::FormationBytes { limit, required })
            if limit == peak - 1 && required == peak
    ));
    for (refusal, _) in charges.iter().enumerate() {
        let mut observed = Vec::new();
        let error = attempt(usize::MAX, &mut |units| {
            observed.push(units);
            if observed.len() - 1 == refusal {
                return Err(refusal);
            }
            Ok(())
        })
        .unwrap_err();
        assert_eq!(error, HtmlRegistryFormationError::Interrupted(refusal));
        assert_eq!(observed, charges[..=refusal]);
    }
    assert_eq!(original.capture().usage().definitions(), 0);
}

#[test]
fn sharing_whole_strings_charges_their_actual_spare_capacity() {
    let original = global();
    let tight = DocumentString::from_utf16(vec![120, 45, 97]);
    let mut units = Vec::with_capacity(2048);
    units.extend_from_slice(tight.code_units());
    let wide = DocumentString::from_utf16(units);
    let extra_units = 2 * (wide.retained_units() - tight.retained_units());
    let extra_bytes = extra_units * size_of::<u16>();
    let tight = append(
        &original,
        &HtmlCustomElementDefinition::new(tight.clone(), tight, false, false),
    );
    let shared = append(
        &original,
        &HtmlCustomElementDefinition::new(wide.clone(), wide.clone(), false, false),
    );
    assert!(matches!(
        shared.capture().definitions(shared.registry()),
        Ok(HtmlCapturedDefinitions::Complete([record]))
            if record.name().shares_storage_with(&wide) && record.local_name().shares_storage_with(&wide)
    ));
    assert_eq!(
        shared.formation_peak_bytes(),
        tight.formation_peak_bytes() + extra_bytes
    );
    assert_eq!(
        shared.retained_storage_usage().retained_bytes(),
        tight.retained_storage_usage().retained_bytes() + extra_bytes
    );
    assert_eq!(
        shared.retained_storage_usage().utf16_extent(),
        tight.retained_storage_usage().utf16_extent() + extra_units
    );
    assert_eq!(shared.capture().usage().utf16_units(), 6);
}

#[test]
fn append_refuses_unqualified_definitions_and_preserves_registry_scope() {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(0, 0, 0, 0, 0, 3, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let unavailable = builder.add_registry(None).unwrap();
    let unknown_scope = builder.add_registry(Some(Vec::new())).unwrap();
    let scoped = builder.add_registry(Some(Vec::new())).unwrap();
    builder
        .establish_registry_scope(&scoped, HtmlRegistryScope::Scoped)
        .unwrap();
    let capture = builder.finish().unwrap();
    let unavailable = capture.registry_id(&unavailable).unwrap();
    let unknown_scope = capture.registry_id(&unknown_scope).unwrap();
    let scoped = capture.registry_id(&scoped).unwrap();
    let original = definition("x-item", "input");
    let attempt = |capture: &HtmlCapture,
                   registry: &HtmlCaptureRegistryId,
                   definition: &HtmlCustomElementDefinition| {
        capture.append_registry_definition_with(
            registry,
            definition,
            HtmlHostFactsRevision::fresh(),
            usize::MAX,
            |_| Ok::<_, ()>(()),
        )
    };
    let formed = attempt(&capture, &scoped, &original).unwrap();
    assert_eq!(
        formed.capture().registry_scope(formed.registry()),
        Ok(HtmlRegistryScope::Scoped)
    );
    for (capture, registry, definition, expected) in [
        (
            &capture,
            formed.registry(),
            original.clone(),
            HtmlCaptureError::ForeignCapture,
        ),
        (
            &capture,
            &unavailable,
            original.clone(),
            HtmlCaptureError::UnavailableDefinitions(unavailable.clone()),
        ),
        (
            &capture,
            &unknown_scope,
            original,
            HtmlCaptureError::MissingFact {
                subject: None,
                fact: HtmlCaptureFact::RegistryScope,
            },
        ),
        (
            formed.capture(),
            formed.registry(),
            definition("x-item", "button"),
            HtmlCaptureError::DuplicateDefinitionName,
        ),
        (
            formed.capture(),
            formed.registry(),
            definition("font-face", "button"),
            HtmlCaptureError::InvalidDefinitionName,
        ),
        (
            formed.capture(),
            formed.registry(),
            definition("x-valid", ""),
            HtmlCaptureError::InvalidDefinitionName,
        ),
    ] {
        assert_eq!(
            attempt(capture, registry, &definition).unwrap_err(),
            HtmlRegistryFormationError::Capture(expected)
        );
    }
}
