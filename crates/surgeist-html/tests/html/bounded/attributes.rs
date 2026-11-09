use crate::html::support::{Recorder, Refused, Refuser};
use std::{convert::Infallible, sync::Arc};
use surgeist_html::{
    accounting::{HtmlChargedError, HtmlWork},
    bounded::values::{
        HtmlAttributeNameIndex, HtmlAttributeStorage, HtmlAttributeValidation,
        HtmlAttributeValidationAction, HtmlAttributeValidationError, HtmlNameIndexSlot,
        HtmlValidatedAttributes,
    },
    values::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName},
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

/// Charges and performs one action, returning the operation's own fault. The recorder admits every
/// charge, so no charge stops.
fn advance(
    action: HtmlAttributeValidationAction<'_>,
    meter: &mut Recorder,
) -> Result<(), HtmlAttributeValidationError> {
    match action.advance(meter) {
        Ok(()) => Ok(()),
        Err(HtmlChargedError::Faulted(fault)) => Err(fault),
        Err(HtmlChargedError::Stopped(never)) => {
            let never: Infallible = never;
            match never {}
        }
    }
}

/// Performs every comparison, leaving the cursor with its latched failure when one occurs.
fn run(validation: &mut HtmlAttributeValidation) -> Result<(), HtmlAttributeValidationError> {
    let mut meter = Recorder::default();
    while let Some(action) = validation.next_action()? {
        advance(action, &mut meter)?;
    }
    Ok(())
}

/// Performs every comparison and finishes, returning the cause of whichever step refused.
fn drive(
    mut validation: HtmlAttributeValidation,
) -> Result<HtmlValidatedAttributes, HtmlAttributeValidationError> {
    run(&mut validation)?;
    validation.finish().map_err(|failure| *failure.cause())
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
    let validation = HtmlAttributeStorage::from_shared_attributes(backing.clone()).validate();
    let attributes = drive(validation).unwrap().into_attributes();
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
    let mut validation = HtmlAttributeStorage::from_shared_attributes(backing.clone()).validate();
    let expected = HtmlAttributeValidationError::Duplicate {
        first: 1,
        duplicate: 2,
    };
    assert_eq!(run(&mut validation).unwrap_err(), expected);
    // The finish refuses with the latched failure and hands the cursor back unchanged.
    let refusal = validation.finish().unwrap_err();
    assert_eq!(*refusal.cause(), expected);
    let (mut validation, _) = refusal.into_parts();
    assert_eq!(validation.next_action().unwrap_err(), expected);
    let recovered = validation.abort().into_shared_attributes();
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
        HtmlAttributeStorage::from_shared_attributes(Arc::new(vec![left, right])).validate();
    // Pair, namespace lengths, two namespace units, local lengths, local unit.
    // These fixed layouts are the admission contract, not an inferred total.
    let expected = [(2, 0), (2, 0), (0, 4), (0, 4), (2, 0), (0, 4)];
    for (index, (visits, bytes)) in expected.into_iter().enumerate() {
        let action = validation.next_action().unwrap().unwrap();
        let mut meter = Recorder::default();
        let outcome = advance(action, &mut meter);
        assert_eq!(
            (
                meter.total(HtmlWork::RecordVisit),
                meter.total(HtmlWork::InspectByte)
            ),
            (visits, bytes)
        );
        // A pairwise comparison has no index to write or copy.
        assert_eq!(meter.total(HtmlWork::RecordWrite), 0);
        assert_eq!(meter.total(HtmlWork::CopyByte), 0);
        if index == expected.len() - 1 {
            assert_eq!(
                outcome,
                Err(HtmlAttributeValidationError::Duplicate {
                    first: 0,
                    duplicate: 1
                })
            );
        } else {
            outcome.unwrap();
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
            HtmlAttributeStorage::from_shared_attributes(backing.clone()).validate();
        let mut meter = Recorder::default();
        for _ in 0..completed_actions {
            advance(validation.next_action().unwrap().unwrap(), &mut meter).unwrap();
        }
        // An action whose charge is refused leaves the cursor at the same comparison.
        let mut refused = Refuser::default();
        let action = validation.next_action().unwrap().unwrap();
        assert_eq!(
            action.advance(&mut refused),
            Err(HtmlChargedError::Stopped(Refused))
        );
        // The finish is refused as not ready and hands the cursor back at the same comparison.
        let refusal = validation.finish().unwrap_err();
        assert_eq!(*refusal.cause(), HtmlAttributeValidationError::NotReady);
        let (mut validation, _) = refusal.into_parts();
        let mut again = Refuser::default();
        let action = validation.next_action().unwrap().unwrap();
        assert_eq!(
            action.advance(&mut again),
            Err(HtmlChargedError::Stopped(Refused))
        );
        assert_eq!(again.attempts, refused.attempts);
        let recovered = validation.abort();
        assert!(Arc::ptr_eq(recovered.shared_attributes(), &backing));
        assert_eq!(recovered.shared_attributes().as_slice(), backing.as_slice());
    }
    let validation = HtmlAttributeStorage::from_shared_attributes(backing.clone()).validate();
    assert_eq!(
        drive(validation).unwrap().into_attributes().as_slice(),
        backing.as_slice()
    );
}

#[test]
fn empty_and_singleton_storage_finish_without_comparisons() {
    for attributes in [vec![], vec![attribute(Namespace::none(), None, "")]] {
        let backing = Arc::new(attributes);
        let mut validation =
            HtmlAttributeStorage::from_shared_attributes(backing.clone()).validate();
        assert!(validation.next_action().unwrap().is_none());
        assert_eq!(
            validation
                .finish()
                .unwrap()
                .into_attributes()
                .as_slice()
                .as_ptr(),
            backing.as_ptr()
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
            let validation =
                HtmlAttributeStorage::from_shared_attributes(Arc::new(attributes)).validate();
            let controlled = drive(validation);
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

#[test]
fn indexed_validation_matches_pairwise_validation_and_returns_the_borrowed_slots() {
    // Names that differ only in namespace, case and prefix, with a repeat of the second.
    let cases: [Vec<Attribute>; 3] = [
        vec![
            attribute(Namespace::none(), None, "a"),
            attribute(Namespace::from_uri("urn:n"), Some("p"), "a"),
            attribute(Namespace::none(), Some("q"), "A"),
        ],
        vec![
            attribute(Namespace::none(), None, "a"),
            attribute(Namespace::from_uri("urn:n"), Some("p"), "a"),
            attribute(Namespace::from_uri("urn:n"), Some("q"), "a"),
        ],
        (0..200)
            .map(|i| attribute(Namespace::none(), None, format!("n{i}")))
            .collect(),
    ];
    for attributes in cases {
        let pairwise = {
            let owner = HtmlAttributeStorage::from_shared_attributes(Arc::new(attributes.clone()))
                .validate();
            drive(owner).map(|validated| validated.as_slice().len())
        };
        let slots = HtmlAttributeNameIndex::slots_required(attributes.len()).unwrap();
        let storage = Vec::<HtmlNameIndexSlot>::with_capacity(slots);
        let capacity = storage.capacity();
        let mut owner = HtmlAttributeStorage::from_shared_attributes(Arc::new(attributes))
            .validate_indexed(HtmlAttributeNameIndex::new(storage));
        let mut outcome = Ok(());
        loop {
            match owner.next_action() {
                Ok(Some(action)) => {
                    if let Err(error) = advance(action, &mut Recorder::default()) {
                        outcome = Err(error);
                        break;
                    }
                }
                Ok(None) => break,
                Err(error) => {
                    outcome = Err(error);
                    break;
                }
            }
        }
        let index = owner.take_index().expect("lent index");
        assert_eq!(index.capacity(), capacity, "the borrowed slots do not grow");
        assert!(owner.take_index().is_none());
        let indexed = outcome.and_then(|()| {
            owner
                .finish()
                .map(|validated| validated.as_slice().len())
                .map_err(|failure| *failure.cause())
        });
        assert_eq!(indexed, pairwise);
        drop(index.into_storage());
    }
}
