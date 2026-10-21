//! Original rendering facts and one canonical publication, never native replay.
use super::*;
use crate::{DomCommit, NodeId};
use DomParsedHtmlStyleRenderingRequirement as Required;
use surgeist_html::{
    HtmlMediaCursorAction, HtmlMediaValueClass, HtmlMediaValueCursor, KnownHtmlAttribute,
};
use surgeist_html_semantics::HtmlDocumentClass;

// All graphs precede their original controlling receipts in drop order. The
// real output slot is filled only by the infallible publication tail.
struct Output {
    dom: Option<DomCommit>,
    qualification: DomParsedHtmlStyleQualification,
    element: NodeId,
    member: bool,
    _state: Arc<State>,
    _state_custody: transaction::StateCustody,
    _history: history::Retirement,
    _formation: Option<source::formation::Formation>,
    _admission: control::Admission,
    _prefix: Option<frontier::ExecutedPrefix>,
    _meter: Meter,
}
/// Historical evidence from one completed original rendering invocation. This
/// owns actual output/retirement custody, not general Style readiness or live
/// membership authority. The whole original contribution remains Pending.
pub struct DomParsedHtmlStyleRenderingCompletion {
    output: Option<Box<Output>>,
}
impl Drop for DomParsedHtmlStyleRenderingCompletion {
    fn drop(&mut self) {
        if let Some(boxed) = self.output.take() {
            // The inner binding's scope ends before Output is destroyed. This
            // deallocates the box first, then drops its moved graphs before
            // their original Meter. A plain `let output = *boxed` would leave
            // the partially moved box alive until after the output's drop.
            let output = {
                let inner = boxed;
                *inner
            };
            drop(output);
        }
    }
}
impl DomParsedHtmlStyleRenderingCompletion {
    fn value(&self) -> &Output {
        self.output
            .as_deref()
            .expect("only live outputs are observed")
    }

    pub fn qualification(&self) -> &DomParsedHtmlStyleQualification {
        &self.value().qualification
    }
    pub fn element(&self) -> NodeId {
        self.value().element
    }
    pub fn dom(&self) -> &DomCommit {
        self.value()
            .dom
            .as_ref()
            .expect("only published outputs escape")
    }
    pub fn phase(&self) -> DomParsedHtmlStylePhase {
        DomParsedHtmlStylePhase::RenderingComplete
    }
    /// Exact canonical membership at this operation's publication coordinate;
    /// later edits and the general Pending readiness guard remain independent.
    pub fn render_blocking_member(&self) -> bool {
        self.value().member
    }
}
impl std::fmt::Debug for DomParsedHtmlStyleRenderingCompletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomParsedHtmlStyleRenderingCompletion")
            .field("qualification", self.qualification())
            .field("element", &self.element())
            .field("member", &self.render_blocking_member())
            .field("stamp", &self.dom().changes().to())
            .finish()
    }
}

struct Facts {
    potentially_blocking: bool,
    unconditional_media: bool,
    context: Option<Required>,
    html: bool,
    body: Option<bool>,
}
fn facts(
    meter: &Meter,
    q: &DomParsedHtmlStyleQualification,
    cancel: &AtomicBool,
) -> Result<Facts, DomParsedHtmlStyleError> {
    let scratch = meter.fork();
    // Both variable buffers remain live together; count their headers as well
    // as the actual capacity admitted by append_group. The blocking Arc target
    // is admitted separately before converting its Vec.
    scratch.retain(size_of::<Vec<u16>>() * 2, 0, 0, cancel)?;
    let report = &meter.entry().report.value().value;
    let mut cursor = report.pending_associations();
    loop {
        scratch.step(cancel)?;
        match cursor.advance() {
            Step::Occurrence(row) if row.id() == *q.occurrence() => {
                if row.subject().id() != *q.subject() || row.context().epoch() != *q.context_epoch()
                {
                    return Err(DomParsedHtmlStyleError::WrongRequest);
                }
                let original = match row.document() {
                    HtmlParseMetadataDocument::Owned(node) => {
                        crate::DomParsedHtmlHistoricalDocument::Constructed(node.id())
                    }
                    HtmlParseMetadataDocument::Captured(id) => {
                        crate::DomParsedHtmlHistoricalDocument::Captured(id.id())
                    }
                };
                if &original != q.original_document() {
                    return Err(DomParsedHtmlStyleError::WrongRequest);
                }
                let inputs = row.style().ok_or(DomParsedHtmlStyleError::WrongRequest)?;
                let mut parser_matches = false;
                if let Some(parser) = inputs.creating_parser_document() {
                    let parser = match parser {
                        HtmlParseMetadataDocument::Owned(node) => {
                            crate::DomParsedHtmlHistoricalDocument::Constructed(node.id())
                        }
                        HtmlParseMetadataDocument::Captured(id) => {
                            crate::DomParsedHtmlHistoricalDocument::Captured(id.id())
                        }
                    };
                    let mut found = false;
                    for binding in &meter.entry().documents {
                        scratch.step(cancel)?;
                        if binding.source == parser {
                            found = true;
                            parser_matches = binding.document == q.document();
                            break;
                        }
                    }
                    if !found {
                        // The earlier metered lookup found no original parser mapping.
                        return Err(DomParsedHtmlStyleError::WrongRequest);
                    }
                }
                let blocking = if let Some(group) = inputs.attribute(KnownHtmlAttribute::Blocking) {
                    let mut units = Vec::new();
                    append_group(report, group, &mut units, &scratch, cancel)?;
                    scratch.retain(size_of::<Vec<u16>>(), 0, 0, cancel)?;
                    Some(DocumentString::from_utf16(units))
                } else {
                    None
                };
                let explicit = surgeist_html::blocking_tokens_with(blocking.as_ref(), || {
                    scratch.step(cancel)
                })?;
                let potential = surgeist_html::style_is_potentially_render_blocking(
                    explicit,
                    inputs.state(),
                    parser_matches,
                );
                // A proven false potential predicate needs no media environment.
                if !potential {
                    return Ok(Facts {
                        potentially_blocking: false,
                        unconditional_media: false,
                        context: None,
                        html: false,
                        body: None,
                    });
                }
                let media = if let Some(group) = inputs.attribute(KnownHtmlAttribute::Media) {
                    let mut units = Vec::new();
                    append_group(report, group, &mut units, &scratch, cancel)?;
                    Some(units)
                } else {
                    None
                };
                let mut media_cursor = HtmlMediaValueCursor::new(media.as_deref());
                while media_cursor.next_action() != HtmlMediaCursorAction::Complete {
                    scratch.step(cancel)?;
                    media_cursor.advance();
                }
                let media = media_cursor
                    .result()
                    .expect("admitted complete classification");
                let unconditional =
                    media.class() == HtmlMediaValueClass::Absent || media.matches_unconditionally();
                if !unconditional {
                    return Ok(Facts {
                        potentially_blocking: true,
                        unconditional_media: false,
                        context: None,
                        html: false,
                        body: None,
                    });
                }
                let context = row.context();
                scratch.step(cancel)?;
                let requirement = context.requirement(Role::DocumentEligibility);
                let mut missing =
                    (requirement != Requirement::Known).then_some(Required::OriginalContext {
                        role: Role::DocumentEligibility,
                        requirement,
                    });
                let html = match context.document_class() {
                    HtmlDocumentClass::Html => true,
                    HtmlDocumentClass::NonHtml => false,
                    HtmlDocumentClass::Unavailable => {
                        missing.get_or_insert(Required::OriginalContext {
                            role: Role::DocumentEligibility,
                            requirement: Requirement::Unavailable(surgeist_html_parser::HtmlParseAssociationMissingKind::OriginalContext),
                        });
                        false
                    }
                };
                // A known non-HTML Document cannot block rendering; no body
                // observation is needed to prove that separate eligibility gate.
                let body = if html && missing.is_none() {
                    scratch.step(cancel)?;
                    let requirement = context.requirement(Role::BodyPresence);
                    if requirement != Requirement::Known {
                        missing = Some(Required::OriginalContext {
                            role: Role::BodyPresence,
                            requirement,
                        });
                    }
                    let body = context.body_present();
                    if body.is_none() && missing.is_none() {
                        missing = Some(Required::OriginalContext {
                            role: Role::BodyPresence,
                            requirement: Requirement::Unavailable(surgeist_html_parser::HtmlParseAssociationMissingKind::OriginalContext),
                        });
                    }
                    body
                } else {
                    None
                };
                return Ok(Facts {
                    potentially_blocking: true,
                    unconditional_media: true,
                    context: missing,
                    html,
                    body,
                });
            }
            Step::Complete => return Err(DomParsedHtmlStyleError::WrongRequest),
            _ => (),
        }
    }
}
impl DomParsedHtmlStyleRuntimeHold<'_> {
    /// Resolve supported original media/render predicates and publish actual
    /// membership with the same cursor. Nontrivial media remains precise Pending;
    /// the final installed body's presence never replaces the original fact.
    pub fn process_rendering(
        &mut self,
        q: &DomParsedHtmlStyleQualification,
        handoff: DomParsedHtmlStyleNotificationHandoff<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleRenderingProgress, DomParsedHtmlStyleError> {
        self.meter
            .entry()
            .control
            .require_execution(self.meter.entry().id)?;
        self.meter.step(cancel)?;
        let mut graph = lock(self.meter.entry())?;
        let held = graph
            .as_mut()
            .ok_or(DomParsedHtmlStyleError::WrongRequest)?;
        if held.value().qualification() != q {
            return Err(DomParsedHtmlStyleError::WrongRequest);
        }
        let Graph::Inline(request) = held.value().as_ref() else {
            return Err(DomParsedHtmlStyleError::WrongPhase {
                phase: held.value().phase(),
            });
        };
        if request.cursor.action() != Action::ReadRenderBlocking {
            return Err(DomParsedHtmlStyleError::WrongPhase {
                phase: held.value().phase(),
            });
        }
        if request.cursor.profile() != HtmlStyleExecutionProfile::NoExecutableJavaScript {
            return Err(DomParsedHtmlStyleError::WrongExecutionProfile {
                profile: request.cursor.profile(),
            });
        }
        self.meter.step(cancel)?;
        if self.dom.pending_associations.document(q.document()) != Some(self.meter.entry().id) {
            return Err(DomParsedHtmlStyleError::WrongRequest);
        }
        let mut element = None;
        for binding in self.meter.entry().bindings.iter() {
            self.meter.step(cancel)?;
            if &binding.source == q.subject() {
                element = Some(binding.node);
                break;
            }
        }
        let element = element.ok_or(DomParsedHtmlStyleError::WrongRequest)?;
        let snapshot = self.dom.snapshot();
        snapshot.document(q.document())?;
        let node = snapshot.node(element)?;
        let style = node
            .as_style()
            .ok_or(DomParsedHtmlStyleError::WrongRequest)?;
        if node.node.document() != q.document()
            || style.state().association_availability()
                != crate::DomStyleSheetAvailability::PendingParsedProcessing
            || style.state().sheet.as_ref().is_none_or(|sheet| {
                sheet.root() != q.document().node() || sheet.association().element() != element
            })
        {
            return Err(DomParsedHtmlStyleError::WrongRequest);
        }
        drop(snapshot);
        if matches!(&handoff,DomParsedHtmlStyleNotificationHandoff::Transfer(receiver) if receiver.is_some())
        {
            return Err(DomParsedHtmlStyleError::ReceiverOccupied);
        }
        frontier::ensure_next(self.dom, &self.meter, q, cancel)?;
        let facts = facts(&self.meter, q, cancel)?;
        if let Some(required) = facts.context {
            self.meter.retain(
                size_of::<DomParsedHtmlStyleRenderingProgress>(),
                0,
                1,
                cancel,
            )?;
            return Ok(DomParsedHtmlStyleRenderingProgress::Pending(required));
        }
        if facts.potentially_blocking && !facts.unconditional_media {
            self.meter.retain(
                size_of::<DomParsedHtmlStyleRenderingProgress>(),
                0,
                1,
                cancel,
            )?;
            return Ok(DomParsedHtmlStyleRenderingProgress::Pending(
                Required::MediaEnvironment,
            ));
        }
        let mut successor = request.cursor;
        let potential = facts.potentially_blocking && facts.unconditional_media;
        successor.reply(Reply::Blocking(potential))?;
        let add = if potential {
            let eligible = facts.html
                && surgeist_html::document_allows_render_blocking(
                    true,
                    facts.body.expect("known original body"),
                );
            successor.reply(Reply::Applied)?;
            eligible
        } else {
            false
        };
        debug_assert_eq!(successor.action(), Action::Complete);
        let transfer = match handoff {
            DomParsedHtmlStyleNotificationHandoff::Retain => None,
            DomParsedHtmlStyleNotificationHandoff::Transfer(receiver) => Some((
                receiver,
                notification::form_continuation(&self.meter, request, cancel)?,
            )),
        };
        let next_prefix = frontier::ExecutedPrefix {
            last: q.occurrence().clone(),
        };
        let mut prefix = self
            .meter
            .entry()
            .executed_prefix
            .try_lock()
            .map_err(|e| notification::guard_error(e, self.meter.entry().id))?;

        let meter = self.meter.fork();
        meter.retain(
            size_of::<Output>() + size_of::<DomParsedHtmlStyleRenderingProgress>(),
            0,
            0,
            cancel,
        )?;
        self.dom.custody_registry.reconcile_releases();
        let admission = meter.admission(self.dom.custody_registry.total(), cancel)?;
        let mut draft = Draft::new_parsed(self.dom, Arc::new(BatchIdentity), &admission, cancel)?;
        let key = Key::Existing(element);
        if add {
            draft.style_add_render_blocking(key, cancel)?;
        }
        draft.source_step(cancel)?;
        let member = draft.role_facts.count(
            transaction::role_facts::Role::RenderBlocking,
            draft.live_key(Key::Existing(q.document().node())),
            draft.live_key(key),
        ) != 0;
        let prepared = draft.finish(self.dom, cancel)?;
        let mut publication = self.dom.prepare_publication(prepared, cancel)?;
        publication.close_ordinary_sharing(cancel)?;
        let history = self.dom.history.retain_evictions(
            &publication.history_plan,
            self.dom.custody_registry.ledger(),
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
        let mut completion = DomParsedHtmlStyleRenderingCompletion {
            output: Some(Box::new(Output {
                dom: None,
                qualification: q.clone(),
                element,
                member,
                _state: self.dom.state.clone(),
                _state_custody: self.dom.state_custody.clone(),
                _history: history,
                _formation: publication._formation.take(),
                _admission: admission,
                _prefix: None,
                _meter: meter,
            })),
        };
        completion.value()._admission.step(cancel)?;
        // The sole fence: genuine reserved publication, same Box-owned request
        // and successor moves. No fallibility, allocation or old graph disposal.
        completion.output.as_mut().expect("preformed output").dom =
            Some(self.dom.publish_prepared(publication));
        let old = std::mem::replace(held.value_mut().as_mut(), Graph::Transition);
        let Graph::Inline(mut request) = old else {
            unreachable!("qualified original rendering cursor")
        };
        request.cursor = successor;
        **held.value_mut() = Graph::Rendered {
            request,
            notification: if transfer.is_some() {
                DomParsedHtmlStyleNotificationHandoffStatus::Transferred
            } else {
                DomParsedHtmlStyleNotificationHandoffStatus::Retained
            },
        };
        completion
            .output
            .as_mut()
            .expect("preformed output")
            ._prefix = prefix.replace(next_prefix);
        if let Some((receiver, value)) = transfer {
            *receiver = Some(value);
        }
        Ok(DomParsedHtmlStyleRenderingProgress::Complete(completion))
    }
}

#[cfg(test)]
pub(super) mod tests;
