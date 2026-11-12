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
        // A Text or Comment node lists the block its string borrows, whether this holder charges
        // the string itself or the formation that made it already did.
        let (NewNode::Text(text) | NewNode::Comment(text)) = incoming.value() else {
            return Ok(incoming);
        };
        self.blocks.adopt(
            text,
            |next| self.admit_custody_usage(next, cancel),
            || self.source_step(cancel),
        )?;
        // Only a Text holder charges its string here: a Comment's is charged where its payload is
        // formed, after the block is listed.
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
