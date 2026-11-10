//! Custody of text at completed DOM-owned immutable value boundaries.
use super::*;

impl Draft {
    /// The actual Text occurrence is under affine custody. Other node kinds pass through.
    pub(in crate::store) fn admit_held_text(
        &self,
        mut incoming: custody::Held<NewNode>,
        retained_occurrence: bool,
        cancel: &AtomicBool,
    ) -> Result<custody::Held<NewNode>, DomError> {
        let NewNode::Text(text) = incoming.value() else {
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
        Ok(incoming)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DomNameLimits;

    use surgeist_html::values::{Namespace, QualifiedName};

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
        let original = second_value.block().downgrade();
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
        assert_eq!(second.value[0].value().retained_units(), 17);
        let local = second.value[0].name().local_name().retained_units();
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
}
