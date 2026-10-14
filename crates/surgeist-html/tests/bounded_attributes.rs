use std::sync::Arc;
use surgeist_html::{
    Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName,
    bounded::{
        HtmlAttributeStorage, HtmlAttributeValidation, HtmlAttributeValidationError,
        HtmlValidatedAttributes,
    },
};

fn attribute(
    namespace: Namespace,
    prefix: Option<&str>,
    local: impl Into<DocumentString>,
) -> Attribute {
    Attribute::new(
        QualifiedName::new(namespace, prefix.map(DocumentString::from), local),
        "value",
    )
}

fn drive(
    owner: &mut Option<HtmlAttributeValidation>,
) -> Result<HtmlValidatedAttributes, HtmlAttributeValidationError> {
    while let Some(action) = owner.as_mut().unwrap().next_action()? {
        action.advance()?;
    }
    HtmlAttributeValidation::try_finish(owner)
}

#[test]
fn shared_utf16_adoption_and_cloning_preserve_exact_units_and_actual_capacity() {
    let mut units = Vec::with_capacity(23);
    units.extend([0xd800, 0, 32, 0xdfff, 0xd83d, 0xde00]);
    let backing = Arc::new(units);
    let value = DocumentString::from_shared_code_units(backing.clone());
    let alias = value.clone();
    assert!(Arc::ptr_eq(value.shared_code_units(), &backing));
    assert!(Arc::ptr_eq(alias.shared_code_units(), &backing));
    assert_eq!(value.code_units(), [0xd800, 0, 32, 0xdfff, 0xd83d, 0xde00]);
    assert_eq!(value.shared_code_units().capacity(), backing.capacity());
    assert_eq!(value.to_utf8().unwrap_err().code_unit_offset(), 0);
    assert_eq!(
        alias.code_points().collect::<Vec<_>>(),
        [0xd800, 0, 32, 0xdfff, 0x1f600]
    );
}

#[test]
fn successful_validation_moves_the_original_backing_in_source_order() {
    let mut attributes = Vec::with_capacity(17);
    attributes.extend([
        attribute(
            Namespace::none(),
            None,
            DocumentString::from_utf16([0xd800, 0]),
        ),
        attribute(
            Namespace::none(),
            None,
            DocumentString::from_utf16([0xd801, 0]),
        ),
        attribute(
            Namespace::from_uri("urn:x"),
            Some("p"),
            DocumentString::from_utf16([0xd800, 0]),
        ),
    ]);
    let backing = Arc::new(attributes);
    let mut validation =
        Some(HtmlAttributeStorage::from_shared_attributes(backing.clone()).validate());
    let attributes = drive(&mut validation).unwrap().into_attributes();
    assert!(validation.is_none());
    assert_eq!(attributes.as_slice(), backing.as_slice());
    assert_eq!(attributes.as_slice().as_ptr(), backing.as_ptr());
    assert_eq!(Arc::strong_count(&backing), 2);
    assert_eq!(
        attributes
            .iter()
            .next()
            .unwrap()
            .name()
            .local_name()
            .code_units(),
        [0xd800, 0]
    );
}

#[test]
fn duplicate_refusal_is_sticky_and_retains_the_exact_allocation() {
    let backing = Arc::new(vec![
        attribute(Namespace::none(), None, "a"),
        attribute(Namespace::none(), Some("first"), "b"),
        attribute(Namespace::none(), Some("second"), "b"),
        attribute(Namespace::none(), None, "a"),
    ]);
    let mut validation =
        Some(HtmlAttributeStorage::from_shared_attributes(backing.clone()).validate());
    let expected = HtmlAttributeValidationError::Duplicate {
        first: 1,
        duplicate: 2,
    };
    assert_eq!(drive(&mut validation).unwrap_err(), expected);
    assert_eq!(
        HtmlAttributeValidation::try_finish(&mut validation).unwrap_err(),
        expected
    );
    assert_eq!(
        validation.as_mut().unwrap().next_action().unwrap_err(),
        expected
    );
    let recovered = validation.take().unwrap().abort().into_shared_attributes();
    assert!(Arc::ptr_eq(&recovered, &backing));
    assert_eq!(
        recovered[1].name().prefix().unwrap().to_utf8().unwrap(),
        "first"
    );
    assert_eq!(
        recovered[2].name().prefix().unwrap().to_utf8().unwrap(),
        "second"
    );
    assert_eq!(Arc::strong_count(&backing), 2);
}

#[test]
fn exact_namespace_comparison_precedes_local_name_and_ignores_prefix_and_value() {
    let namespace = Namespace::from_uri(DocumentString::from_utf16([0xd800, 0]));
    let left = attribute(namespace.clone(), Some("left"), "a");
    let right = Attribute::new(
        QualifiedName::new(namespace, Some(DocumentString::from("right")), "a"),
        "different value",
    );
    let mut validation =
        Some(HtmlAttributeStorage::from_shared_attributes(Arc::new(vec![left, right])).validate());
    // Pair, namespace lengths, two namespace units, local lengths, local unit.
    // These fixed layouts are the admission contract, not an inferred total.
    let expected = [(2, 0), (2, 0), (0, 4), (0, 4), (2, 0), (0, 4)];
    for (index, (visits, bytes)) in expected.into_iter().enumerate() {
        let action = validation.as_mut().unwrap().next_action().unwrap().unwrap();
        assert_eq!(
            (
                action.layout().record_visits(),
                action.layout().inspected_bytes()
            ),
            (visits, bytes)
        );
        if index == expected.len() - 1 {
            assert_eq!(
                action.advance(),
                Err(HtmlAttributeValidationError::Duplicate {
                    first: 0,
                    duplicate: 1
                })
            );
        } else {
            action.advance().unwrap();
        }
    }
}

#[test]
fn every_interrupted_comparison_recovers_all_storage_without_publication() {
    let backing = Arc::new(vec![
        attribute(Namespace::from_uri("urn:a"), None, "ab"),
        attribute(Namespace::from_uri("urn:a"), None, "ac"),
    ]);
    // Pair, namespace lengths and five units, local lengths and two units.
    for completed_actions in 0..10 {
        let mut validation =
            Some(HtmlAttributeStorage::from_shared_attributes(backing.clone()).validate());
        for _ in 0..completed_actions {
            validation
                .as_mut()
                .unwrap()
                .next_action()
                .unwrap()
                .unwrap()
                .advance()
                .unwrap();
        }
        assert_eq!(
            HtmlAttributeValidation::try_finish(&mut validation).unwrap_err(),
            HtmlAttributeValidationError::NotReady
        );
        let refused_layout = {
            let action = validation.as_mut().unwrap().next_action().unwrap().unwrap();
            action.layout()
        };
        assert_eq!(
            validation
                .as_mut()
                .unwrap()
                .next_action()
                .unwrap()
                .unwrap()
                .layout(),
            refused_layout
        );
        let recovered = validation.take().unwrap().abort();
        assert!(Arc::ptr_eq(recovered.shared_attributes(), &backing));
        assert_eq!(recovered.shared_attributes().as_slice(), backing.as_slice());
    }
    let mut validation =
        Some(HtmlAttributeStorage::from_shared_attributes(backing.clone()).validate());
    assert_eq!(
        drive(&mut validation).unwrap().into_attributes().as_slice(),
        backing.as_slice()
    );
}

#[test]
fn empty_and_singleton_storage_finish_without_comparisons() {
    for attributes in [vec![], vec![attribute(Namespace::none(), None, "")]] {
        let backing = Arc::new(attributes);
        let mut validation =
            Some(HtmlAttributeStorage::from_shared_attributes(backing.clone()).validate());
        assert!(
            validation
                .as_mut()
                .unwrap()
                .next_action()
                .unwrap()
                .is_none()
        );
        assert_eq!(
            HtmlAttributeValidation::try_finish(&mut validation)
                .unwrap()
                .into_attributes()
                .as_slice()
                .as_ptr(),
            backing.as_ptr()
        );
        assert_eq!(
            HtmlAttributeValidation::try_finish(&mut validation).unwrap_err(),
            HtmlAttributeValidationError::Consumed
        );
    }
}

#[test]
fn exact_namespace_and_local_name_pairs_match_native_duplicate_identity() {
    let names = [
        (Namespace::none(), DocumentString::from("")),
        (Namespace::none(), DocumentString::from("A")),
        (Namespace::none(), DocumentString::from("a")),
        (Namespace::from_uri("urn:a"), DocumentString::from("a")),
        (Namespace::from_uri("urn:A"), DocumentString::from("a")),
        (
            Namespace::from_uri(DocumentString::from_utf16([0xd800])),
            DocumentString::from_utf16([0xdfff]),
        ),
    ];
    for (left_index, (left_ns, left_local)) in names.iter().enumerate() {
        for (right_index, (right_ns, right_local)) in names.iter().enumerate() {
            let attributes = vec![
                attribute(left_ns.clone(), Some("p"), left_local.clone()),
                attribute(right_ns.clone(), Some("q"), right_local.clone()),
            ];
            let native = OrderedAttributes::new(attributes.clone());
            let mut validation =
                Some(HtmlAttributeStorage::from_shared_attributes(Arc::new(attributes)).validate());
            let controlled = drive(&mut validation);
            // Entries above are independently distinct expanded names.
            if left_index == right_index {
                assert_eq!(
                    controlled.unwrap_err(),
                    HtmlAttributeValidationError::Duplicate {
                        first: 0,
                        duplicate: 1
                    }
                );
                let error = native.unwrap_err();
                assert_eq!((error.first_index(), error.duplicate_index()), (0, 1));
            } else {
                assert_eq!(controlled.unwrap().into_attributes(), native.unwrap());
            }
        }
    }
}
