use super::*;

fn version(revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: "style".into(),
        revision,
    }
}
fn batch(owner: &CssomStore) -> CssomBatch {
    owner
        .batch(CssomEditGuard::from_snapshot(&owner.snapshot()))
        .unwrap()
}
fn intrinsic(diagnostics: &[CssRecoveryDiagnostic]) -> usize {
    diagnostics
        .iter()
        .map(|value| {
            value
                .retained_backing_bytes_with(|| Ok::<_, ()>(()))
                .unwrap()
        })
        .sum()
}
#[test]
fn construction_receipt_observes_real_final_vec_spare_capacity_and_excludes_self() {
    let mut owner = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = batch(&owner);
    let constructed = edit
        .construct_sheet(
            CssomConstructorDocument {
                version: CssomInputVersion {
                    role: CssomInputRole::Document,
                    ..version(1)
                },
                identity: "document".into(),
                base_location: String::new(),
            },
            CssomSheetInit {
                media: CssomInitialMedia::Text("screen and"),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!constructed.media_diagnostics().is_empty());
    let (ticket, original) = constructed.into_parts();
    let mut diagnostics = Vec::with_capacity(original.len() + 31);
    diagnostics.extend(original);
    let expected = diagnostics.capacity() * std::mem::size_of::<CssRecoveryDiagnostic>()
        + intrinsic(&diagnostics);
    let before = diagnostics.clone();
    let result = CssomSheetConstruction::from_parts(ticket, diagnostics);
    assert_eq!(
        result.retained_backing_bytes_with(|| Ok::<_, ()>(())),
        Ok(expected)
    );
    assert_eq!(result.media_diagnostics(), before);
    let original_source = before[0].source().unwrap();
    assert!(original_source.same_snapshot(result.media_diagnostics()[0].source().unwrap()));
    let commit = owner.commit(edit).unwrap();
    assert!(commit.sheet(result.ticket()).is_ok());
    assert_eq!(
        result.retained_backing_bytes_with(|| Ok::<_, ()>(())),
        Ok(expected)
    );
}
#[test]
fn owner_attribute_result_and_pending_publication_keep_diagnostics_and_exact_source() {
    let mut owner = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = batch(&owner);
    let mut inputs = CssomSheetInputs::external(version(1), true);
    inputs.owner = Some("style".into());
    let ticket = edit.create_typed_sheet(CssSheet::new(), inputs).unwrap();
    let id = owner.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let mut edit = batch(&owner);
    let result = edit
        .update_sheet_owner_attributes(
            &id,
            CssomSheetOwnerAttributes {
                owner: "style",
                version: version(2),
                title: Some(&[0xd800]),
                media: CssomOwnerMediaInput::Utf8(Some("screen and")),
            },
        )
        .unwrap();
    assert!(result.changed());
    assert!(!result.diagnostics().is_empty());
    let source = result.diagnostics()[0].source().unwrap().clone();
    let before = result.diagnostics().to_vec();
    let expected = result.diagnostics.capacity() * std::mem::size_of::<CssRecoveryDiagnostic>()
        + intrinsic(result.diagnostics());
    assert_eq!(
        result.retained_backing_bytes_with(|| Ok::<_, ()>(())),
        Ok(expected)
    );
    let mut steps = 0;
    result
        .retained_backing_bytes_with(|| {
            steps += 1;
            Ok::<_, usize>(())
        })
        .unwrap();
    for allowance in 0..steps {
        let mut reached = 0;
        assert_eq!(
            result.retained_backing_bytes_with(|| {
                if reached == allowance {
                    return Err(reached);
                }
                reached += 1;
                Ok(())
            }),
            Err(CssRetainedPayloadError::Admission(allowance))
        );
        assert_eq!(reached, allowance);
        assert_eq!(result.diagnostics(), before);
    }
    owner.commit(edit).unwrap();
    assert_eq!(
        owner
            .snapshot()
            .sheet(&id)
            .unwrap()
            .inputs()
            .title
            .as_slice(),
        &[0xd800]
    );
    assert!(source.same_snapshot(result.diagnostics()[0].source().unwrap()));
    assert_eq!(
        result.retained_backing_bytes_with(|| Ok::<_, ()>(())),
        Ok(expected)
    );
    let empty = CssomSheetAttributeEdit {
        changed: false,
        diagnostics: Vec::with_capacity(17),
    };
    assert_eq!(
        empty.retained_backing_bytes_with(|| Ok::<_, ()>(())),
        Ok(empty.diagnostics.capacity() * std::mem::size_of::<CssRecoveryDiagnostic>())
    );
}
