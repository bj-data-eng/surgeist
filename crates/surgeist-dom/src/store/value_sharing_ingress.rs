//! Optional sharing at completed DOM-owned immutable value boundaries.
use super::*;

impl Draft {
    /// The actual Text occurrence is under affine custody before the optional
    /// producer may retain an upgraded candidate. Other node kinds pass through.
    pub(in crate::store) fn share_held_text(
        &self,
        mut incoming: custody::Held<DomNodeKind>,
        retained_occurrence: bool,
        cancel: &AtomicBool,
    ) -> Result<custody::Held<DomNodeKind>, DomError> {
        let DomNodeKind::Text(text) = incoming.value() else {
            return Ok(incoming);
        };
        let Some(session) = self.sharing_session() else {
            return Ok(incoming);
        };
        let mut original = source::storage::Payload::default();
        original.string(text)?;
        let original = custody::Usage::new(original.bytes, original.units);
        // The producer classifies this exact field, rather than inferring
        // its presence from unrelated credit in the same candidate holder.
        if !retained_occurrence {
            incoming
                .debit_mut()
                .grow(original, |next| self.admit_custody_usage(next, cancel))?;
        }
        let DomNodeKind::Text(text) = incoming.value() else {
            unreachable!("the held candidate remains Text")
        };
        let selected =
            self.with_sharing_admission(cancel, |admission| session.share_string(text, admission))?;
        let Some(selected) = selected else {
            return Ok(incoming);
        };
        let mut selected = selected.into_held();
        let (old, mut original_credit) = incoming.into_parts();
        // The original occurrence ends only after its actual field is gone.
        drop(old);
        drop(original_credit.split(original)?);
        selected
            .debit_mut()
            .absorb(original_credit)
            .expect("sharing uses the original DOM custody domain");
        let (value, credit) = selected.into_parts();
        Ok(custody::Held::new(DomNodeKind::Text(value), credit))
    }

    /// Element prefixes are ordinary value text, never mandatory name atoms.
    /// This storage change does not replay native initialization or alter any
    /// namespace/local-name identity or prefix-presence distinction.
    pub(in crate::store) fn share_held_element_prefix(
        &self,
        mut incoming: custody::Held<DomNodeKind>,
        retained_occurrence: bool,
        cancel: &AtomicBool,
    ) -> Result<custody::Held<DomNodeKind>, DomError> {
        let DomNodeKind::Element(element) = incoming.value() else {
            return Ok(incoming);
        };
        let Some(session) = self.sharing_session() else {
            return Ok(incoming);
        };
        self.seed_name_atoms(element.name(), cancel)?;
        let Some(prefix) = element.name().prefix() else {
            return Ok(incoming);
        };
        let mut original = source::storage::Payload::default();
        original.string(prefix)?;
        let original = custody::Usage::new(original.bytes, original.units);
        // The producer classifies this exact field, rather than inferring
        // its presence from unrelated credit in the same candidate holder.
        if !retained_occurrence {
            incoming
                .debit_mut()
                .grow(original, |next| self.admit_custody_usage(next, cancel))?;
        }
        let DomNodeKind::Element(element) = incoming.value() else {
            unreachable!("the held candidate remains an Element")
        };
        let prefix = element.name().prefix().expect("original prefix is present");
        let selected = self
            .with_sharing_admission(cancel, |admission| session.share_string(prefix, admission))?;
        let Some(selected) = selected else {
            return Ok(incoming);
        };
        let (value, selected_credit) = selected.into_held().into_parts();
        let DomNodeKind::Element(element) = incoming.value_mut() else {
            unreachable!("the held candidate remains an Element")
        };
        drop(element.replace_name_prefix(Some(value)));
        incoming
            .debit_mut()
            .absorb(selected_credit)
            .expect("sharing uses the original DOM custody domain");
        drop(incoming.debit_mut().split(original)?);
        Ok(incoming)
    }

    pub(in crate::store) fn seed_name_atoms(
        &self,
        name: &crate::DomQualifiedName,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(session) = self.sharing_session() else {
            return Ok(());
        };
        for atom in name.namespace().into_iter().chain([name.local_name_atom()]) {
            self.with_sharing_admission(cancel, |admission| {
                session.seed_string(atom.text(), admission)
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomNameLimits, Namespace, QualifiedName};

    fn limits() -> DomLimits {
        DomLimits::new(
            64,
            65_536,
            128,
            0,
            0,
            64,
            1_000_000,
            DomNameLimits::new(16_777_216, 4096, 1_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
    }

    macro_rules! attributes {
        ($draft:expr, $value:expr, $cancel:expr) => {{
            let raw = QualifiedName::new(Namespace::none(), None, "data-x");
            let name = $draft.bind_name(&raw, None, $cancel).unwrap();
            let mut input = $draft
                .attribute_input(1, None, $cancel, |entries| reserve(entries, 1))
                .unwrap();
            input
                .entries
                .push(crate::DomAttribute::from_parts(name, $value));
            input
        }};
    }

    #[test]
    fn resolved_bank_hit_precedes_payload_identity_and_releases_unused_input_backing() {
        let cancel = AtomicBool::new(false);
        let dom = Dom::new(limits()).unwrap();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        draft.ensure_sharing(&dom.value_sharing, &cancel).unwrap();
        let mut units = Vec::with_capacity(17);
        units.extend_from_slice(&[0xd800, 0, 120]);
        let first = draft
            .form_attributes(
                attributes!(&draft, DocumentString::from_utf16(units), &cancel),
                None,
                &cancel,
            )
            .unwrap();
        let second_value = DocumentString::from_utf16(vec![0xd800, 0, 120]);
        let original = Arc::downgrade(second_value.shared_code_units());
        let input = attributes!(&draft, second_value, &cancel);
        let refusal = payload::identity_refusal::Refusal::begin();
        let second = draft.form_attributes(input, None, &cancel).unwrap();
        assert!(!refusal.reached(), "a hit reserves no new payload identity");
        assert!(Arc::ptr_eq(
            first.value.payload().unwrap(),
            second.value.payload().unwrap()
        ));
        assert!(
            original.upgrade().is_none(),
            "unused input fields are destroyed"
        );
        assert_eq!(second.value[0].value().code_units(), &[0xd800, 0, 120]);
        assert_eq!(second.value[0].value().shared_code_units().capacity(), 17);
        let local = second.value[0]
            .name()
            .local_name()
            .shared_code_units()
            .capacity();
        let expected = custody::Usage::new(
            2 * size_of::<Vec<u16>>() + (local + 17) * size_of::<u16>(),
            local + 17,
        );
        assert_eq!(
            second.selected.as_ref().unwrap().usage(),
            expected,
            "only the new field occurrences remain after target adoption"
        );
        assert_eq!(
            second.value[0].name().owner(),
            first.value[0].name().owner()
        );
    }

    #[test]
    fn text_producer_provenance_preserves_unrelated_credit_on_reuse() {
        let cancel = AtomicBool::new(false);
        let dom = Dom::new(limits()).unwrap();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        draft.ensure_sharing(&dom.value_sharing, &cancel).unwrap();
        let mut units = Vec::with_capacity(19);
        units.extend_from_slice(&[0xd800, 0, 120]);
        let first = draft
            .share_held_text(
                custody::Held::new(
                    DomNodeKind::Text(DocumentString::from_utf16(units)),
                    draft
                        .custody_ledger()
                        .retain(custody::Usage::ZERO, |_| Ok(()))
                        .unwrap(),
                ),
                false,
                &cancel,
            )
            .unwrap();
        let DomNodeKind::Text(first_text) = first.value() else {
            panic!("actual Text");
        };
        assert_eq!(first_text.shared_code_units().capacity(), 19);
        let selected = custody::Usage::new(size_of::<Vec<u16>>() + 19 * size_of::<u16>(), 19);
        assert_eq!(first.debit().usage(), selected);
        let extra = custody::Usage::new(777, 111);
        for retained_occurrence in [false, true] {
            let value = DocumentString::from_utf16(vec![0xd800, 0, 120]);
            let original = Arc::downgrade(value.shared_code_units());
            let capacity = value.shared_code_units().capacity();
            let original_extent = custody::Usage::new(
                size_of::<Vec<u16>>() + capacity * size_of::<u16>(),
                capacity,
            );
            let owned = if retained_occurrence {
                extra.checked_add(original_extent).unwrap()
            } else {
                extra
            };
            let input = custody::Held::new(
                DomNodeKind::Text(value),
                draft.custody_ledger().retain(owned, |_| Ok(())).unwrap(),
            );
            let result = draft
                .share_held_text(input, retained_occurrence, &cancel)
                .unwrap();
            let DomNodeKind::Text(value) = result.value() else {
                panic!("same Text kind");
            };
            assert!(Arc::ptr_eq(
                value.shared_code_units(),
                first_text.shared_code_units()
            ));
            assert_eq!(value.code_units(), &[0xd800, 0, 120]);
            assert!(original.upgrade().is_none());
            assert_eq!(
                result.debit().usage(),
                selected.checked_add(extra).unwrap(),
                "producer-qualified original field ends without subtracting unrelated credit"
            );
        }
    }

    #[test]
    fn comment_candidate_keeps_its_exact_backing_and_credit() {
        let cancel = AtomicBool::new(false);
        let dom = Dom::new(limits()).unwrap();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        draft.ensure_sharing(&dom.value_sharing, &cancel).unwrap();
        let text = DocumentString::from_utf16(vec![0xd800, 0]);
        let pointer = Arc::as_ptr(text.shared_code_units());
        let debit = draft
            .custody_ledger()
            .retain(custody::Usage::new(37, 5), |_| Ok(()))
            .unwrap();
        let baseline = draft.custody_ledger().total();
        let result = draft
            .share_held_text(
                custody::Held::new(DomNodeKind::Comment(text), debit),
                false,
                &cancel,
            )
            .unwrap();
        let DomNodeKind::Comment(text) = result.value() else {
            panic!("same node kind");
        };
        assert_eq!(Arc::as_ptr(text.shared_code_units()), pointer);
        assert_eq!(result.debit().usage(), custody::Usage::new(37, 5));
        assert_eq!(draft.custody_ledger().total(), baseline);
    }
}
