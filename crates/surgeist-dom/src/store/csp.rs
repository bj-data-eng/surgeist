//! One admitted issuer per canonical owner; immutable associations live in metadata.
use super::*;
use std::cell::RefCell;
use surgeist_html::{HtmlCspError, HtmlCspOwner};

pub(super) fn owner(
    ledger: &custody::Ledger,
) -> Result<custody::Held<RefCell<HtmlCspOwner>>, DomError> {
    let mut debit = None;
    let owner = HtmlCspOwner::new_with(&mut |layout| {
        let bytes = layout
            .retained()
            .storage()
            .bytes()
            .checked_add(
                size_of::<custody::Held<RefCell<HtmlCspOwner>>>() - size_of::<HtmlCspOwner>(),
            )
            .ok_or(DomError::RepresentationExhausted)?;
        debit = Some(ledger.retain(custody::Usage::new(bytes, 0), |_| Ok(()))?);
        Ok(())
    })
    .map_err(|error| match error {
        HtmlCspError::Interrupted(error) => error,
        // This fixed protocol constructor has no external artifact or inputs;
        // its only fallible protocol operation is checked representation.
        HtmlCspError::Protocol(_) => DomError::RepresentationExhausted,
    })?;
    Ok(custody::Held::new(
        RefCell::new(owner),
        debit.expect("issuer admitted before formation"),
    ))
}
