//! Post-event current-Document cleanup under the original admitted controller.
use super::*;
struct Output {
    commit: Option<DomCommit>,
    _state: Arc<State>,
    _state_custody: transaction::StateCustody,
    _history: history::Retirement,
    _formation: Option<source::formation::Formation>,
    _delivery: Option<DomParsedHtmlStyleDeliveryPublication>,
    _admission: control::Admission,
    _meter: Meter,
}
/// Actual parsed cleanup publication and its retained original output/retirement
/// custody. This is independent of the sheet that originally queued the event.
pub struct DomParsedHtmlStyleNotificationCompletion {
    output: Option<Box<Output>>,
}
impl DomParsedHtmlStyleNotificationCompletion {
    pub fn dom(&self) -> &DomCommit {
        self.output
            .as_ref()
            .and_then(|output| output.commit.as_ref())
            .expect("published cleanup")
    }
}
impl std::fmt::Debug for DomParsedHtmlStyleNotificationCompletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomParsedHtmlStyleNotificationCompletion")
            .field("dom", self.dom())
            .finish()
    }
}
impl Drop for DomParsedHtmlStyleNotificationCompletion {
    fn drop(&mut self) {
        if let Some(boxed) = self.output.take() {
            // Finish the partially moved Box scope BEFORE its graphs and the
            // controlling Meter. Plain `let output = *boxed` reverses that order.
            let output = {
                let inner = boxed;
                *inner
            };
            drop(output);
        }
    }
}
impl Dom {
    pub(in crate::store) fn finish_parsed_style_notification(
        &mut self,
        token: &mut DomStyleNotificationToken,
        input: &mut Option<DomParsedHtmlStyleDeliveryPublication>,
        cancel: &AtomicBool,
    ) -> Result<crate::DomStyleNotificationCompletion, DomError> {
        if self
            .parsed_style_notification_requirement(token, cancel)?
            .is_some()
        {
            let crate::DomStyleNotificationOrigin::ParsedOccurrence { contribution, .. } =
                token.origin()
            else {
                return Err(DomError::StyleProtocol);
            };
            return Err(DomError::ParsedAssociationSynchronousPending {
                pending: *contribution,
            });
        }
        let custody = token.parsed.as_deref().ok_or(DomError::StyleProtocol)?;
        let actual = input.as_ref().ok_or(DomError::StyleProtocol)?;
        if !custody.attested
            || custody.right.is_some()
            || actual.qualification != custody.qualification
            || actual.association != token.association()
            || !std::ptr::eq(
                actual._meter.entry(),
                custody
                    .meter
                    .try_lock()
                    .map_err(|e| guard_error(e, custody.qualification.contribution()))?
                    .entry(),
            )
        {
            return Err(DomError::StyleProtocol);
        }
        let id = custody.qualification.contribution();
        let next_notifications = actual._meter.entry().control.discharge_notification()?;
        let guard = custody
            .meter
            .try_lock()
            .map_err(|error| guard_error(error, id))?;
        let meter = guard.fork();
        meter.retain(
            size_of::<Output>() + size_of::<DomParsedHtmlStyleNotificationCompletion>(),
            0,
            0,
            cancel,
        )?;
        self.custody_registry.reconcile_releases();
        let admission = meter.admission(self.custody_registry.total(), cancel)?;
        let mut draft = Draft::new_parsed(self, Arc::new(BatchIdentity), &admission, cancel)?;
        let key = draft.resolve(token.target().into())?;
        draft
            .element(key)?
            .language()
            .style_state()
            .ok_or(DomError::WrongNodeKind)?;
        // The profile excludes executable script bookkeeping only. The final
        // source-required unblock uses the actual target's CURRENT Document.
        let document = draft.node(key)?.document();
        draft.relations_retain(
            document,
            transaction::RelationField::RenderBlocking,
            |candidate| candidate != key,
            cancel,
        )?;
        let prepared = draft.finish(self, cancel)?;
        let mut publication = self.prepare_publication(prepared, cancel)?;
        publication.close_ordinary_sharing(cancel)?;
        let history = self.history.retain_evictions(
            &publication.history_plan,
            self.custody_registry.ledger(),
            |total| admission.controlled(total, cancel),
            || admission.step(cancel),
        )?;
        drop(draft);
        meter.retain(
            0,
            0,
            publication
                .changes
                .len()
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
            cancel,
        )?;
        let mut result = DomParsedHtmlStyleNotificationCompletion {
            output: Some(Box::new(Output {
                commit: None,
                _state: self.state.clone(),
                _state_custody: self.state_custody.clone(),
                _history: history,
                _formation: publication._formation.take(),
                _delivery: None,
                _admission: admission,
                _meter: meter,
            })),
        };
        result
            .output
            .as_ref()
            .expect("preformed cleanup")
            ._admission
            .step(cancel)?;
        result.output.as_mut().expect("preformed cleanup").commit =
            Some(self.publish_prepared(publication));
        let output = result.output.as_mut().expect("preformed cleanup");
        output
            ._meter
            .entry()
            .control
            .publish_discharge(next_notifications);
        output._delivery = input.take();
        token.complete = true;
        Ok(crate::DomStyleNotificationCompletion::ParsedComplete(
            result,
        ))
    }
}
