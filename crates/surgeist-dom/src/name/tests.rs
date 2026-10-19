use super::*;
use std::collections::hash_map::DefaultHasher;
use surgeist_html::{HtmlAttributeInput, HtmlAttributesInput, HtmlNameInput};

// Each fresh spelling below has one record within its owner. Registry lifecycle
// and collision tests belong to the actual interning owner, not this fixture.
fn atom(owner: DomOwnerId, text: DocumentString) -> DomNameAtom {
    DomNameAtom::from_record(Arc::new(NameRecord::new(
        owner,
        text,
        MaterializationId::fresh().unwrap(),
        0,
    )))
}
fn hash(value: &impl Hash) -> u64 {
    let mut hash = DefaultHasher::new();
    value.hash(&mut hash);
    hash.finish()
}
fn owner() -> DomOwnerId {
    DomOwnerId::allocate().unwrap()
}

#[test]
fn identity_clones_and_cross_owner_semantic_names_have_distinct_contracts() {
    let first = owner();
    let second = owner();
    let local = atom(first, "item".into());
    let alias = local.clone();
    assert_eq!(local, alias);
    assert_eq!(hash(&local), hash(&alias));
    assert_eq!(local.same_identity(&alias), Ok(true));
    assert_eq!(local.owner(), first);
    let other = atom(second, "item".into());
    assert_ne!(local, other);
    assert_eq!(local.same_identity(&other), Err(DomError::ForeignOwner));

    let left = DomQualifiedName::from_atoms(
        Some(atom(first, "urn:example".into())),
        Some("p".into()),
        local,
    )
    .unwrap();
    let right = DomQualifiedName::from_atoms(
        Some(atom(second, "urn:example".into())),
        Some("p".into()),
        other,
    )
    .unwrap();
    assert_eq!(left, right);
    assert_eq!(hash(&left), hash(&right));
    assert_eq!(
        left.same_expanded_identity(&right),
        Err(DomError::ForeignOwner)
    );
    assert_eq!(left.local_name().code_units(), &[0x69, 0x74, 0x65, 0x6d]);
}

#[test]
fn namespace_normalization_and_prefix_observations_remain_separate() {
    let owner = owner();
    let local = atom(owner, "name".into());
    let absent = DomQualifiedName::from_atoms(None, None, local.clone()).unwrap();
    let normalized =
        DomQualifiedName::from_atoms(Some(atom(owner, "".into())), None, local.clone()).unwrap();
    assert_eq!(absent, normalized);
    assert_eq!(hash(&absent), hash(&normalized));
    assert_eq!(normalized.namespace(), None);
    assert_eq!(normalized.namespace_uri(), None);
    let empty_prefix = DomQualifiedName::from_atoms(None, Some("".into()), local.clone()).unwrap();
    assert_ne!(absent, empty_prefix);
    assert_eq!(
        empty_prefix.prefix().map(DocumentString::code_units),
        Some([].as_slice())
    );
    assert_eq!(absent.same_expanded_identity(&empty_prefix), Ok(true));
    let prefixed = DomQualifiedName::from_atoms(None, Some("p".into()), local).unwrap();
    assert_eq!(prefixed.prefix().unwrap().code_units(), &[0x70]);
    assert_eq!(absent.same_expanded_identity(&prefixed), Ok(true));
    assert_ne!(prefixed, absent);
}

#[test]
fn mixed_owner_components_are_refused_before_a_canonical_name_exists() {
    let local = atom(owner(), "item".into());
    let namespace = atom(owner(), "urn:other".into());
    assert_eq!(
        DomQualifiedName::from_atoms(Some(namespace), None, local),
        Err(DomError::ForeignOwner)
    );
}

#[test]
fn borrowed_html_inputs_preserve_order_raw_values_and_shared_component_text() {
    let owner = owner();
    let text = atom(owner, "shared".into());
    // Namespace and local roles share the actual same canonical record.
    let name =
        DomQualifiedName::from_atoms(Some(text.clone()), Some("p".into()), text.clone()).unwrap();
    assert_eq!(name.namespace().unwrap(), name.local_name_atom());
    let value = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
    let attributes = [
        DomAttribute::from_parts(name.clone(), value.clone()),
        DomAttribute::from_parts(
            DomQualifiedName::from_atoms(None, None, atom(owner, "second".into())).unwrap(),
            "tail".into(),
        ),
    ];
    fn observe<N: HtmlNameInput + ?Sized, A: HtmlAttributesInput + ?Sized>(name: &N, attrs: &A) {
        assert_eq!(
            name.namespace_uri().unwrap().code_units(),
            &[0x73, 0x68, 0x61, 0x72, 0x65, 0x64]
        );
        assert_eq!(name.prefix().unwrap().code_units(), &[0x70]);
        assert_eq!(attrs.len(), 2);
        assert_eq!(
            attrs.get(0).unwrap().value().code_units(),
            &[0xd800, 0, 0xdc00]
        );
        assert_eq!(
            attrs.get(1).unwrap().name().local_name().code_units(),
            &[0x73, 0x65, 0x63, 0x6f, 0x6e, 0x64]
        );
        assert!(attrs.get(2).is_none());
    }
    observe(&name, attributes.as_slice());
    assert!(std::ptr::eq(
        attributes[0].name(),
        HtmlAttributeInput::name(&attributes[0])
    ));
    assert!(std::ptr::eq(
        &value.code_units()[0],
        &attributes[0].value().code_units()[0]
    ));
}

#[test]
fn per_occurrence_receipts_include_actual_text_capacity_without_record_bodies() {
    let owner = owner();
    let mut backing = Vec::with_capacity(19);
    backing.extend([0x61, 0xd800]);
    let text = DocumentString::from_utf16(backing);
    let capacity = text.shared_code_units().capacity();
    let local = atom(owner, text.clone());
    let name =
        DomQualifiedName::from_atoms(Some(local.clone()), Some(text.clone()), local).unwrap();
    let attribute = DomAttribute::from_parts(name, text.clone());
    assert_eq!(attribute.logical_units(), Some(8));
    let payload = attribute.retained_payload().unwrap();
    assert_eq!(
        payload.bytes,
        4 * (size_of::<Vec<u16>>() + capacity * size_of::<u16>())
    );
    assert_eq!(payload.units, 4 * capacity);
    let record = NameRecord::usage(&text).unwrap();
    assert_eq!(
        record.bytes(),
        size_of::<NameRecord>() + size_of::<Vec<u16>>() + capacity * size_of::<u16>()
    );
    assert_eq!(record.units(), capacity);
}

#[test]
fn attributes_compare_exact_values_across_owners() {
    let make = |owner| {
        DomAttribute::from_parts(
            DomQualifiedName::from_atoms(None, None, atom(owner, "data-x".into())).unwrap(),
            DocumentString::from_utf16(vec![0x61, 0xd800]),
        )
    };
    let left = make(owner());
    let right = make(owner());
    assert_eq!(left, right);
    assert_eq!(hash(&left), hash(&right));
    let different = DomAttribute::from_parts(
        right.name().clone(),
        DocumentString::from_utf16(vec![0x61, 0xdc00]),
    );
    assert_ne!(left, different);
}

#[test]
fn canonical_attribute_array_aliases_keep_one_target_and_exact_cached_occurrences() {
    let local = atom(owner(), "data-x".into());
    let name = DomQualifiedName::from_atoms(None, None, local).unwrap();
    let mut text = Vec::with_capacity(13);
    text.extend([0x61, 0xd800]);
    let text = DocumentString::from_utf16(text);
    let mut entries = Vec::with_capacity(7);
    entries.push(DomAttribute::from_parts(name, text));
    let capacity = entries.capacity();
    let expected_text = entries
        .iter()
        .flat_map(|attribute| [attribute.name().local_name(), attribute.value()])
        .fold((0usize, 0usize), |(bytes, units), text| {
            let capacity = text.shared_code_units().capacity();
            (
                bytes + size_of::<Vec<u16>>() + capacity * size_of::<u16>(),
                units + capacity,
            )
        });
    let mut visits = 0;
    let value = AttributeValues::new(entries, || {
        visits += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(visits, 1);
    let expected_shallow = size_of::<crate::store::payload::Payload<AttributeValues>>()
        + capacity * size_of::<DomAttribute>();
    assert_eq!(
        value.shallow_usage().unwrap(),
        MaterializationUsage::new(expected_shallow, 0)
    );
    let bank = AttributeStorage::from_payload(Arc::new(crate::store::payload::Payload {
        identity: MaterializationId::fresh().unwrap(),
        own_usage: value.shallow_usage().unwrap(),
        value,
    }));
    let clone = bank.clone();
    let target = Arc::downgrade(bank.payload().unwrap());
    assert!(bank.same_storage(&clone));
    assert_eq!(bank.capacity(), capacity);
    assert_eq!(bank.logical_units(), 8);
    assert_eq!(
        (bank.text_payload().bytes, bank.text_payload().units),
        expected_text
    );
    assert_eq!(bank.own_usage().bytes(), expected_shallow);
    drop(bank);
    assert_eq!(clone.entries()[0].value().code_units(), &[0x61, 0xd800]);
    assert_eq!(clone.logical_units(), 8);
    drop(clone);
    assert!(target.upgrade().is_none());
}

#[test]
fn absent_and_reserved_empty_attribute_arrays_have_equal_values_and_distinct_storage() {
    let absent = AttributeStorage::default();
    assert!(absent.payload().is_none());
    assert_eq!(absent.own_usage(), MaterializationUsage::ZERO);
    let entries = Vec::with_capacity(5);
    let capacity = entries.capacity();
    let reserved = AttributeStorage::from_entries(entries);
    assert_eq!(reserved, absent);
    assert!(!reserved.same_storage(&absent));
    assert_eq!(reserved.capacity(), capacity);
    assert_eq!(reserved.logical_units(), 0);
    assert_eq!(
        (reserved.text_payload().bytes, reserved.text_payload().units),
        (0, 0)
    );
    assert_eq!(
        reserved.own_usage().bytes(),
        size_of::<crate::store::payload::Payload<AttributeValues>>()
            + capacity * size_of::<DomAttribute>()
    );
    let clone = reserved.clone();
    drop(reserved);
    assert_eq!(clone.capacity(), capacity);
    assert_eq!(clone.entries(), &[]);
}
