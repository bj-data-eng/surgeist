use super::*;
use crate::DomNameAtom;
use crate::name::NameRecord;
use crate::store::MaterializationId;
use std::collections::HashMap;
use std::{mem::size_of, sync::Arc};
use surgeist_html::values::{Namespace, QualifiedName};

// These receipt fixtures construct valid canonical values with one record per
// owner/spelling. Actual registry admission/lifecycle is tested by its owner.
struct Names {
    owner: crate::DomOwnerId,
    atoms: HashMap<DocumentString, DomNameAtom>,
    registry: crate::store::bank::CustodyRegistry,
    controls: Vec<crate::store::bank::ObjectCustody>,
    children: Vec<ElementChildren>,
}
impl Names {
    fn new() -> Self {
        let ledger = crate::store::custody::Ledger::default();
        let registry = crate::store::bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
        Self {
            registry,
            controls: Vec::new(),
            children: Vec::new(),
            owner: crate::DomOwnerId::allocate().unwrap(),
            atoms: HashMap::new(),
        }
    }
    fn atom(&mut self, text: &DocumentString) -> DomNameAtom {
        let row = self.atoms.len();
        self.atoms
            .entry(text.clone())
            .or_insert_with(|| {
                DomNameAtom::from_record(Arc::new(NameRecord::new(
                    self.owner,
                    text.clone(),
                    MaterializationId::fresh().unwrap(),
                    row,
                )))
            })
            .clone()
    }
    fn name(&mut self, raw: &QualifiedName) -> DomQualifiedName {
        DomQualifiedName::from_atoms(
            raw.namespace().uri().map(|text| self.atom(text)),
            raw.prefix().cloned(),
            self.atom(raw.local_name()),
        )
        .unwrap()
    }
    fn name_target(&mut self, name: DomQualifiedName) -> Arc<Payload<ElementName>> {
        let text = name.retained_payload().unwrap();
        let usage = crate::store::custody::Usage::new(
            size_of::<Payload<ElementName>>() + text.bytes,
            text.units,
        );
        let formed = crate::store::payload::form_group(
            &self.registry,
            None,
            crate::store::source::storage::Payload::default(),
            usage,
            &mut |_| Ok(()),
            || Ok(ElementName { value: name }),
        )
        .unwrap();
        self.controls.push(formed.custody);
        formed.value
    }
}
fn native(names: &mut Names, local: &str) -> DomElement {
    let raw = QualifiedName::new(Namespace::html(), None, local);
    crate::names::validate_element_components(&raw, &mut || Ok(())).unwrap();
    let name = names.name(&raw);
    let name = names.name_target(name);
    let cancel = AtomicBool::new(false);
    let formed = DomElement::native(
        name,
        None,
        crate::DomDocumentEnvironment::new(crate::DomNativeInputPlatform::headless()),
        ElementFormation::new(
            &names.registry,
            &crate::store::block_custody::BlockAdoptions::new(names.registry.clone()),
            None,
            &cancel,
            &mut |_| Ok(()),
        ),
    )
    .unwrap();
    names.children.push(formed.children);
    formed.value
}
#[test]
fn scalar_family_clones_and_refused_attribute_effects_preserve_the_original_state() {
    let mut names = Names::new();
    let original = native(&mut names, "marquee");
    let mut candidate = original.clone();
    candidate.set_marquee_running(true).unwrap();
    candidate.set_captured_in_view_transition(true);
    assert_eq!(candidate.units(), original.units());
    assert!(Arc::ptr_eq(
        candidate.name_payload(),
        original.name_payload()
    ));
    assert_ne!(candidate, original);

    let changed_name = names.name(&QualifiedName::new(Namespace::none(), None, "data-x"));
    let attributes = [crate::DomAttribute::from_parts(
        changed_name.clone(),
        "four".into(),
    )];
    let cancel = AtomicBool::new(true);
    let rejected = candidate.with_attribute_effect(
        &[] as &[crate::DomAttribute],
        attributes.as_slice(),
        &changed_name,
        NonceUpdate::Unchanged,
        ElementFormation::new(
            &names.registry,
            &crate::store::block_custody::BlockAdoptions::new(names.registry.clone()),
            None,
            &cancel,
            &mut |_| Ok(()),
        ),
    );
    assert!(matches!(rejected, Err(DomError::Cancelled)));
    assert_eq!(candidate.units(), original.units());
    let mut common = native(&mut names, "div");
    assert_eq!(
        common.set_marquee_running(true),
        Err(DomError::MarqueeInapplicable)
    );
}

#[test]
fn unrelated_attributes_alias_the_name_and_exact_native_group_without_admission() {
    let mut names = Names::new();
    let original = native(&mut names, "input");
    let changed = names.name(&QualifiedName::new(Namespace::none(), None, "data-x"));
    let attributes = [crate::DomAttribute::from_parts(
        changed.clone(),
        "literal".into(),
    )];
    let cancel = AtomicBool::new(false);
    let formed = original
        .with_attribute_effect(
            &[] as &[crate::DomAttribute],
            attributes.as_slice(),
            &changed,
            NonceUpdate::Unchanged,
            ElementFormation::new(
                &names.registry,
                &crate::store::block_custody::BlockAdoptions::new(names.registry.clone()),
                None,
                &cancel,
                &mut |_| panic!("unchanged semantic groups acquire no new payload or control"),
            ),
        )
        .unwrap();
    let edited = &formed.value;
    assert!(Arc::ptr_eq(original.name_payload(), edited.name_payload()));
    assert!(Arc::ptr_eq(
        original.input_payload().unwrap(),
        edited.input_payload().unwrap()
    ));
    assert_eq!(original.input_state(), edited.input_state());
    assert!(formed.children.native.is_none());
    assert!(formed.children.nonce.is_none());
}
