//! Direct controlled consumption of HTML's optional immutable string producer.
use super::*;
use surgeist_html::bounded::{
    HtmlStringPoolOperation, HtmlStringPoolResult, HtmlValuePoolActionKind, HtmlValuePoolProgress,
};

struct Pending<'a> {
    operation: Option<HtmlStringPoolOperation<'a>>,
    // Provisional producer data is destroyed before either of its credits.
    extra: custody::Debit,
    candidate: custody::Debit,
}
fn grow_to(
    debit: &mut custody::Debit,
    target: custody::Usage,
    admission: &mut Admission<'_>,
) -> Result<(), DomError> {
    let old = debit.usage();
    let added = custody::Usage::new(
        target.bytes().saturating_sub(old.bytes()),
        target.units().saturating_sub(old.units()),
    );
    if added != custody::Usage::ZERO {
        debit.grow(added, &mut *admission.global)?;
    }
    Ok(())
}
fn empty(ledger: &custody::Ledger) -> custody::Debit {
    ledger
        .retain(custody::Usage::ZERO, |_| Ok(()))
        .expect("zero original-ledger subset")
}
impl Session {
    pub(in crate::store) fn seed_string(
        &self,
        value: &DocumentString,
        admission: &mut Admission<'_>,
    ) -> Result<(), DomError> {
        // Required atom identity/backing never changes. Only future optional
        // value/Text selections may reuse this existing immutable allocation.
        drop(self.share_string(value, admission)?);
        Ok(())
    }
    pub(in crate::store) fn share_string(
        &self,
        value: &DocumentString,
        admission: &mut Admission<'_>,
    ) -> Result<Option<SelectedString>, DomError> {
        #[cfg(test)]
        if !self.fixture_strings_enabled() {
            return Ok(None);
        }
        if !self.may_attempt() {
            return Ok(None);
        }
        // DOM's finite optional-attempt policy may reject a cold oversized
        // input by extent. An already active HTML producer remains governed
        // by its own advertised descriptors and eligibility checks.
        if self.is_dormant()
            && (value.len() > banks::MAX_UNITS
                || value.shared_code_units().capacity() > banks::MAX_UNITS)
        {
            return Ok(None);
        }
        let (Some(owner), Some(id)) = (self.owner.upgrade(), self.ensure_active(admission)?) else {
            return Ok(None);
        };
        let (operation, ledger) = {
            let mut state = lock(&owner);
            if !state.mutable(id) {
                return Ok(None);
            }
            (state.strings.probe(value), state.ledger.clone())
        };
        let mut pending = Pending {
            operation: Some(operation),
            extra: empty(&ledger),
            candidate: empty(&ledger),
        };
        let mut promoted = false;
        let result = (|| {
            loop {
                let action = {
                    let mut state = lock(&owner);
                    state
                        .strings
                        .next_action(pending.operation.as_mut().expect("owned operation"))
                        .map_err(|_| DomError::InvalidSourceOperation)?
                };
                let Some(action) = action else {
                    if !promoted {
                        promoted = true;
                        let mut state = lock(&owner);
                        if state.mutable(id)
                            && state
                                .strings
                                .offer_miss(pending.operation.as_mut().unwrap())
                                .map_err(|_| DomError::InvalidSourceOperation)?
                        {
                            continue;
                        }
                    }
                    break;
                };
                let layout = action.layout();
                admission.work(html_work(layout.work())?)?;
                admission.work(html_work(layout.closing_work())?)?;
                let extra = {
                    let state = lock(&owner);
                    let target = custody::Usage::new(size_of::<Mutex<State>>(), 0)
                        .checked_add(state.banks.usage())?
                        .checked_add(html_usage(layout.pool()))?;
                    custody::Usage::new(
                        target.bytes().saturating_sub(state.storage.usage().bytes()),
                        target.units().saturating_sub(state.storage.usage().units()),
                    )
                };
                grow_to(&mut pending.extra, extra, admission)?;
                grow_to(
                    &mut pending.candidate,
                    html_usage(layout.candidate()),
                    admission,
                )?;
                let progress = match action.kind() {
                    HtmlValuePoolActionKind::Read => {
                        pending.operation.as_mut().unwrap().read(action)
                    }
                    HtmlValuePoolActionKind::Pool => lock(&owner)
                        .strings
                        .perform(pending.operation.as_mut().unwrap(), action),
                }
                .map_err(|_| DomError::InvalidSourceOperation)?;
                {
                    let mut state = lock(&owner);
                    let actual = state.usage()?;
                    let old = state.storage.usage();
                    let added = custody::Usage::new(
                        actual.bytes().saturating_sub(old.bytes()),
                        actual.units().saturating_sub(old.units()),
                    );
                    if added != custody::Usage::ZERO {
                        state
                            .storage
                            .absorb(
                                pending
                                    .extra
                                    .split(added)
                                    .expect("actual producer storage preadmitted before action"),
                            )
                            .expect("original optional ledger");
                    }
                    state.reconcile();
                }
                if progress == HtmlValuePoolProgress::CandidateReleased {
                    let old = std::mem::replace(&mut pending.candidate, empty(&ledger));
                    drop(old);
                }
            }
            let result = HtmlStringPoolOperation::try_finish(&mut pending.operation)
                .map_err(|_| DomError::InvalidSourceOperation)?;
            Ok(match result {
                HtmlStringPoolResult::Hit(value) => {
                    let credit = std::mem::replace(&mut pending.candidate, empty(&ledger));
                    Some(SelectedString(custody::Held::new(
                        value.into_value(),
                        credit,
                    )))
                }
                HtmlStringPoolResult::Miss
                | HtmlStringPoolResult::Offered(_)
                | HtmlStringPoolResult::Bypassed => None,
            })
        })();
        drop(pending);
        lock(&owner).reconcile();
        result
    }
}
