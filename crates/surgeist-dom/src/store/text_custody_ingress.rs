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
        let text = match incoming.value() {
            NewNode::Text(text) | NewNode::Comment(text) => text,
            NewNode::Attr(attribute) => attribute.value(),
            _ => return Ok(incoming),
        };
        self.list_block(text, cancel)?;
        // Comment and Attr occurrences are charged by their typed payloads.
        if matches!(incoming.value(), NewNode::Text(_)) && !retained_occurrence {
            let mut original = source::storage::Payload::default();
            original.listed_string(text)?;
            let original = custody::Usage::new(original.bytes, original.units);
            incoming
                .debit_mut()
                .grow(original, |next| self.admit_custody_usage(next, cancel))?;
        }
        Ok(incoming)
    }
}
