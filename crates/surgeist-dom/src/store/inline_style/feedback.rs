//! Resident standard-operation outcome and exact Pending-effect feedback.
use super::*;
use surgeist_cssom::{
    CssomAdmissionRequest, CssomOperationOutcome, CssomOwnerEffectStatus, CssomOwnerNotification,
};

#[derive(Clone)]
pub(in crate::store) struct Echo {
    pub(in crate::store) element: NodeId,
    pub(in crate::store) value: DocumentString,
}
struct Original {
    value: DomInlineStyleReturn,
    publication: surgeist_cssom::CssomPublication,
    diagnostics: Vec<surgeist_css::CssRecoveryDiagnostic>,
    effects: Vec<surgeist_cssom::CssomOwnerEffect>,
}
impl Original {
    fn unit(outcome: CssomOperationOutcome<()>) -> Self {
        let ((), publication, diagnostics, effects) = outcome.into_parts();
        Self {
            value: DomInlineStyleReturn::Unit,
            publication,
            diagnostics,
            effects,
        }
    }
    fn removed(outcome: CssomOperationOutcome<String>) -> Self {
        let (value, publication, diagnostics, effects) = outcome.into_parts();
        Self {
            value: DomInlineStyleReturn::Removed(value),
            publication,
            diagnostics,
            effects,
        }
    }
}
pub(super) struct Resident {
    element: NodeId,
    original: Option<Original>,
    limits: DomInlineStyleFeedbackLimits,
    work: source::formation::SourceWorkCustody,
    policy: Arc<custody::Held<DomInlineStyleLimits>>,
    observation: Option<crate::DomSourceEventId>,
}
impl Owner {
    pub(super) fn feedback_target(&self) -> Option<NodeId> {
        self.feedback
            .as_ref()
            .filter(|held| held.value().original.is_some())
            .map(|held| held.value().element)
    }
    pub(super) fn require_no_feedback(&self) -> Result<(), DomInlineStyleError> {
        if let Some(element) = self.feedback_target() {
            Err(DomInlineStyleError::OperationPending { element })
        } else {
            Ok(())
        }
    }
}
/// Exclusive access to one committed standard operation. Dropping this handle
/// parks the same outcome and effect with its original DOM owner.
pub struct DomInlineStyleOperation<'a> {
    dom: &'a mut Dom,
}
impl<'a> DomInlineStyleOperation<'a> {
    /// Exact bound target, including generation and owner.
    pub fn element(&self) -> NodeId {
        self.resident().element
    }
    /// Original standard return value, even while feedback remains incomplete.
    pub fn value(&self) -> &DomInlineStyleReturn {
        &self.original().value
    }
    /// Original committed producer publication.
    pub fn publication(&self) -> &surgeist_cssom::CssomPublication {
        &self.original().publication
    }
    /// Original diagnostics, retained without replaying their producer.
    pub fn diagnostics(&self) -> &[surgeist_css::CssRecoveryDiagnostic] {
        &self.original().diagnostics
    }
    /// Actual single-use effect status. Terminal states are not revived.
    pub fn owner_effects(&self) -> &[surgeist_cssom::CssomOwnerEffect] {
        &self.original().effects
    }
    /// Cumulative DOM work already spent by this original operation.
    pub fn spent_work(&self) -> u64 {
        self.resident().work.work().load(Ordering::Relaxed)
    }
    /// Retry only canonical feedback. A refusal preserves the original effect,
    /// outcome and spent work; this never reruns a standard mutation/resolver.
    pub fn complete(self, cancel: &AtomicBool) -> DomInlineStyleMutation<'a> {
        match self.dom.complete_inline_feedback(cancel) {
            Ok(completion) => DomInlineStyleMutation::Complete(completion),
            Err(reason) => DomInlineStyleMutation::Incomplete {
                operation: self,
                reason,
            },
        }
    }
    fn resident(&self) -> &Resident {
        self.dom
            .inline_styles
            .as_ref()
            .expect("retained owner")
            .feedback
            .as_ref()
            .expect("retained feedback")
            .value()
    }
    fn original(&self) -> &Original {
        self.resident()
            .original
            .as_ref()
            .expect("committed outcome")
    }
}
// A resolver unwind must remove only the prepublication empty holder. Once an
// outcome exists, this same slot owns it through every later refusal/unwind.
struct Ingress<'a>(&'a mut Option<custody::Held<Box<Resident>>>);
impl Drop for Ingress<'_> {
    fn drop(&mut self) {
        if self
            .0
            .as_ref()
            .is_some_and(|held| held.value().original.is_none())
        {
            self.0.take();
        }
    }
}
impl Dom {
    pub(in crate::store) fn blocked_inline_recording(&self) -> Option<recording::Recording> {
        let owner = self.inline_styles.as_ref()?;
        let resident = owner.feedback.as_ref()?.value();
        resident.original.as_ref()?;
        Some(recording::Recording::blocked(
            &owner.index,
            resident.element,
            resident.policy.clone(),
        ))
    }
    pub(in crate::store) fn inline_feedback_observation(
        &self,
    ) -> Option<(NodeId, crate::DomSourceEventId)> {
        let resident = self.inline_styles.as_ref()?.feedback.as_ref()?.value();
        resident.original.as_ref()?;
        Some((resident.element, resident.observation?))
    }
    /// Standard setCssText on the actual same-object bound Properties. CSSOM
    /// commits first; canonical refusal returns an owned incomplete operation.
    pub fn set_inline_style_css_text<'a>(
        &'a mut self,
        element: NodeId,
        inputs: &DomInlineStyleInputs,
        text: &str,
        limits: DomInlineStyleMutationLimits,
        resolver: impl FnOnce(
            &CssomAdmissionRequest<'_>,
        ) -> Result<CssomAdmission, surgeist_cssom::CssomError>,
        cancel: &AtomicBool,
    ) -> Result<DomInlineStyleMutation<'a>, DomInlineStyleError> {
        self.mutate_inline(element, inputs, limits, cancel, |object, store| {
            object
                .set_css_text(store, text, limits.declaration, resolver)
                .map(Original::unit)
        })
    }
    /// Standard setProperty, including its original name/priority/no-op rules.
    pub fn set_inline_style_property<'a>(
        &'a mut self,
        element: NodeId,
        inputs: &DomInlineStyleInputs,
        property: DomInlineStyleProperty<'_>,
        limits: DomInlineStyleMutationLimits,
        resolver: impl FnOnce(
            &CssomAdmissionRequest<'_>,
        ) -> Result<CssomAdmission, surgeist_cssom::CssomError>,
        cancel: &AtomicBool,
    ) -> Result<DomInlineStyleMutation<'a>, DomInlineStyleError> {
        self.mutate_inline(element, inputs, limits, cancel, |object, store| {
            object
                .set_property(
                    store,
                    property.name,
                    property.value,
                    property.priority,
                    limits.declaration,
                    resolver,
                )
                .map(Original::unit)
        })
    }
    /// Standard removeProperty and its original removed serialized value.
    pub fn remove_inline_style_property<'a>(
        &'a mut self,
        element: NodeId,
        inputs: &DomInlineStyleInputs,
        name: &str,
        limits: DomInlineStyleMutationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomInlineStyleMutation<'a>, DomInlineStyleError> {
        self.mutate_inline(element, inputs, limits, cancel, |object, store| {
            object
                .remove_property(store, name, limits.declaration)
                .map(Original::removed)
        })
    }
    /// Borrow the original parked feedback operation, without changing its
    /// policy, producer receipt, target, effect or cumulative work.
    pub fn resume_inline_style_feedback(
        &mut self,
    ) -> Result<DomInlineStyleOperation<'_>, DomInlineStyleError> {
        if self
            .inline_styles
            .as_ref()
            .and_then(|o| o.feedback_target())
            .is_none()
        {
            return Err(DomInlineStyleError::Unbound);
        }
        Ok(DomInlineStyleOperation { dom: self })
    }
    fn mutate_inline<'a>(
        &'a mut self,
        element: NodeId,
        inputs: &DomInlineStyleInputs,
        limits: DomInlineStyleMutationLimits,
        cancel: &AtomicBool,
        mutate: impl FnOnce(
            &CssStyleProperties,
            &mut CssomStore,
        ) -> Result<Original, surgeist_cssom::CssomError>,
    ) -> Result<DomInlineStyleMutation<'a>, DomInlineStyleError> {
        self.require_source_entry()?;
        check_cancel(cancel)?;
        self.state
            .node(self.owner, element)?
            .element()
            .ok_or(DomError::WrongNodeKind)?;
        let owner = self
            .inline_styles
            .as_ref()
            .ok_or(DomInlineStyleError::Unconfigured)?;
        owner.require_no_feedback()?;
        if !owner.index.membership.contains(element) {
            return Err(DomInlineStyleError::Unbound);
        }
        // Reuse the actual current binding's external-profile qualification;
        // this route cannot create a block or silently replace its packet.
        let object = self.inline_style(
            element,
            inputs,
            DomInlineStyleRequestLimits {
                input: Default::default(),
                declaration: limits.declaration,
            },
            |_, _| unreachable!("existing binding does not parse"),
            cancel,
        )?;
        let owner = self.inline_styles.as_ref().expect("qualified owner");
        let formation = self.source_observation_formation(cancel)?;
        let p = processing(&owner.limits, formation.as_ref(), cancel);
        if limits.feedback.outputs == 0 {
            return Err(DomError::SourceOutputLimit.into());
        }
        let mut retain = |usage: custody::Usage| {
            p.admit(usage)?;
            admit(
                usage.bytes(),
                limits.feedback.retained_bytes,
                DomResource::SourceRetainedBytes,
            )?;
            admit(
                usage.units(),
                limits.feedback.retained_units,
                DomResource::SourceRetainedUnits,
            )
        };
        let (_, work) =
            source::formation::SourceWorkCustody::allocate(0, &self.custody_registry, &mut retain)?;
        let debit = self.custody_registry.ledger().retain(
            custody::Usage::new(
                size_of::<Resident>()
                    .checked_add(size_of::<DomInlineStyleCompletion>())
                    .and_then(|n| n.checked_add(size_of::<DomInlineStyleMutation<'_>>()))
                    .ok_or(DomError::RepresentationExhausted)?,
                0,
            ),
            &mut retain,
        )?;
        let policy_credit = self.custody_registry.ledger().retain(
            custody::Usage::new(size_of::<custody::Held<DomInlineStyleLimits>>(), 0),
            &mut retain,
        )?;
        let policy = Arc::new(custody::Held::new(owner.limits.clone(), policy_credit));
        let observation = self
            .source_operation
            .as_ref()
            .and_then(|operation| operation.inline_feedback_observation());
        let holder = custody::Held::new(
            Box::new(Resident {
                element,
                original: None,
                limits: limits.feedback,
                work,
                policy,
                observation,
            }),
            debit,
        );
        let owner = self.inline_styles.as_mut().expect("qualified owner");
        owner.feedback = Some(holder);
        let ingress = Ingress(&mut owner.feedback);
        let original = mutate(&object, owner.store.as_mut().expect("bound store"))?;
        ingress
            .0
            .as_mut()
            .expect("preadmitted holder")
            .value_mut()
            .original = Some(original);
        drop(ingress);
        match self.complete_inline_feedback(cancel) {
            Ok(completion) => Ok(DomInlineStyleMutation::Complete(completion)),
            Err(reason) => Ok(DomInlineStyleMutation::Incomplete {
                operation: DomInlineStyleOperation { dom: self },
                reason,
            }),
        }
    }
    fn prepare_inline_feedback(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<Option<PreparedPublication>, DomInlineStyleError> {
        let owner = self.inline_styles.as_ref().expect("resident owner");
        let resident = owner.feedback.as_ref().expect("resident feedback").value();
        let original = resident.original.as_ref().expect("committed outcome");
        if original.effects.is_empty() {
            return Ok(None);
        }
        let same_observation = resident.observation.is_some()
            && self.state.incomplete_hosted.is_none()
            && self
                .source_operation
                .as_ref()
                .and_then(|operation| operation.inline_feedback_observation())
                == resident.observation;
        if !same_observation {
            self.require_source_entry()?;
        }
        check_cancel(cancel)?;
        assert_eq!(
            original.effects.len(),
            1,
            "one actual bound standard mutation"
        );
        let effect = &original.effects[0];
        if effect.status() != CssomOwnerEffectStatus::Pending {
            return Err(DomInlineStyleError::TerminalEffect {
                status: effect.status(),
            });
        }
        let store = owner.store.as_ref().expect("bound store");
        store.validate_pending_owner_effect(effect)?;
        let limits = resident.limits;
        let element = resident.element;
        self.state.node(self.owner, element)?;
        let work = resident.work.clone();
        let (mut draft, _) = Draft::new_source_root_with_work(
            self,
            Arc::new(BatchIdentity),
            source::initial::CommandStorage {
                commands: &[],
                capacity: 0,
            },
            crate::DomSourceLimits::new(
                limits.retained_bytes,
                limits.retained_units,
                limits.work,
                1,
                limits.outputs,
                0,
                1,
            ),
            source::initial::Replay {
                work: work.work().clone(),
                identity: work.identity(),
                names: None,
            },
            cancel,
        )?;
        let formation = source::formation::Formation::new(&draft, cancel)?;
        formation
            .as_ref()
            .expect("feedback envelope")
            .retain_bytes(
                size_of::<CssomStyleAttributeChange<'_>>()
                    .checked_add(effect.owner().identity.len())
                    .and_then(|n| n.checked_add(5))
                    .ok_or(DomError::RepresentationExhausted)?,
                cancel,
            )?;
        for _ in 0..effect
            .value()
            .len()
            .checked_add(effect.owner().identity.len())
            .and_then(|n| n.checked_add(5))
            .ok_or(DomError::RepresentationExhausted)?
        {
            source::formation::step(formation.as_ref(), cancel)?;
        }
        let change = CssomStyleAttributeChange {
            owner: effect.owner().clone(),
            local_name: "style".into(),
            namespace: None,
            value: Some(CssomStyleAttributeInput::Utf8(effect.value())),
            echo: Some(effect.id().clone()),
        };
        assert_eq!(
            store.classify_pending_owner_notification(effect, &change)?,
            CssomOwnerNotification::MatchingEcho
        );
        let value = source::formation::string(effect.value(), formation.as_ref(), cancel)?;
        let name = source::formation::string("style", formation.as_ref(), cancel)?;
        draft.inline_echo = Some(Echo {
            element,
            value: value.clone(),
        });
        draft.set_attribute(
            Key::Existing(element),
            None,
            &name,
            &value,
            self.limits,
            cancel,
        )?;
        let mut prepared = draft.finish(self, cancel)?;
        prepared.state.incomplete_source = self.state.incomplete_source;
        let mut publication = self.prepare_publication(prepared, cancel)?;
        publication.close_ordinary_sharing(cancel)?;
        if publication
            .changes
            .len()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?
            > limits.outputs
        {
            return Err(DomError::SourceOutputLimit.into());
        }
        check_cancel(cancel)?;
        Ok(Some(publication))
    }
    fn complete_inline_feedback(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<DomInlineStyleCompletion, DomInlineStyleError> {
        let publication = self.prepare_inline_feedback(cancel)?;
        let dom = if let Some(publication) = publication {
            // The producer, effect and publication fields are disjoint actual
            // owners. No fallible DOM operation or callback follows application.
            let Dom {
                state,
                state_custody,
                style_cssom_roots,
                revision,
                history,
                active_task,
                inline_styles,
                ..
            } = self;
            let Owner {
                store,
                index,
                feedback,
                ..
            } = inline_styles.as_mut().expect("resident owner").as_mut();
            let resident = feedback.as_mut().expect("resident feedback").value_mut();
            let effect = &mut resident
                .original
                .as_mut()
                .expect("committed outcome")
                .effects[0];
            let application = store
                .as_mut()
                .expect("bound store")
                .begin_owner_application(effect)?;
            let commit = PublicationFields {
                state,
                state_custody,
                style_cssom_roots,
                revision,
                history,
                active_task,
                inline_index: Some(index),
            }
            .publish(publication)
            .commit;
            application.finish_success();
            Some(commit)
        } else {
            None
        };
        let held = self
            .inline_styles
            .as_mut()
            .expect("resident owner")
            .feedback
            .take()
            .expect("resident feedback");
        let (resident, debit) = held.into_parts();
        let Original {
            value,
            publication,
            diagnostics,
            effects,
        } = resident.original.expect("committed outcome");
        Ok(DomInlineStyleCompletion {
            value,
            publication,
            diagnostics,
            effects,
            dom,
            _custody: debit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_canceled_effect_remains_terminal_and_blocks_coherent_completion() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                100_000,
                64,
                32,
                32,
                8,
                1_000_000,
                crate::DomNameLimits::new(16_777_216, 4096, 1_000_000, 65536).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let element = batch.create_element_ns(&document, crate::Namespace::html(), "div");
        let commit = dom.apply(batch, &cancel).unwrap();
        let document = commit.document(&document).unwrap();
        let element = commit.created(&element).unwrap().unwrap();
        dom.configure_inline_styles(
            DomInlineStyleLimits::new(Default::default(), 8, 64_000_000, 8_000_000, 4_000_000)
                .unwrap(),
            &cancel,
        )
        .unwrap();
        let inputs = DomInlineStyleInputs::new(
            dom.snapshot().document(document).unwrap().stamp(),
            Default::default(),
            CssomInputVersion {
                role: CssomInputRole::Support,
                identity: "terminal-test-support".into(),
                revision: 1,
            },
            Default::default(),
        )
        .unwrap();
        dom.inline_style(
            element,
            &inputs,
            DomInlineStyleRequestLimits {
                input: Default::default(),
                declaration: Default::default(),
            },
            |_, _| panic!("empty input"),
            &cancel,
        )
        .unwrap();
        let before = dom.capture_inline_style(element).unwrap().unwrap();
        let mutation = dom
            .set_inline_style_css_text(
                element,
                &inputs,
                "--x: 1",
                DomInlineStyleMutationLimits {
                    declaration: Default::default(),
                    feedback: DomInlineStyleFeedbackLimits::new(64_000_000, 1_000_000, 0, 64),
                },
                |_| panic!("custom property needs no decision"),
                &cancel,
            )
            .unwrap();
        match mutation {
            DomInlineStyleMutation::Incomplete { operation, reason } => {
                assert!(matches!(
                    reason,
                    DomInlineStyleError::Dom(DomError::SourceWorkLimit { limit: 0 })
                ));
                assert_eq!(
                    operation.owner_effects()[0].status(),
                    CssomOwnerEffectStatus::Pending
                );
            }
            _ => panic!("actual feedback work must refuse"),
        }
        let owner = dom.inline_styles.as_mut().unwrap();
        let effect = &mut owner
            .feedback
            .as_mut()
            .unwrap()
            .value_mut()
            .original
            .as_mut()
            .unwrap()
            .effects[0];
        let id = effect.id().clone();
        effect.cancel().unwrap(); // Actual CSSOM transition, no status fabrication.
        let revision = owner.store.as_ref().unwrap().snapshot().revision().clone();
        for _ in 0..2 {
            match dom
                .resume_inline_style_feedback()
                .unwrap()
                .complete(&cancel)
            {
                DomInlineStyleMutation::Incomplete { operation, reason } => {
                    assert!(matches!(
                        reason,
                        DomInlineStyleError::TerminalEffect {
                            status: CssomOwnerEffectStatus::Canceled
                        }
                    ));
                    assert_eq!(operation.owner_effects()[0].id(), &id);
                    assert_eq!(
                        operation.owner_effects()[0].status(),
                        CssomOwnerEffectStatus::Canceled
                    );
                }
                _ => panic!("terminal effect cannot revive"),
            }
        }
        assert_eq!(
            dom.inline_styles
                .as_ref()
                .unwrap()
                .store
                .as_ref()
                .unwrap()
                .snapshot()
                .revision(),
            &revision
        );
        assert_eq!(dom.snapshot().stamp(), before.dom().stamp());
        assert!(
            matches!(dom.capture_inline_style(element), Err(DomInlineStyleError::OperationPending { element: target }) if target == element)
        );
        assert_eq!(
            before.properties().css_text(Default::default()).unwrap(),
            ""
        );
    }
    #[test]
    fn pending_feedback_refuses_reached_fullscreen_scope_change_before_exposing_source() {
        let cancel = AtomicBool::new(false);
        let limits = |units| {
            DomLimits::new(
                64,
                units,
                256,
                128,
                128,
                16,
                4_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_fullscreen_exits(4)
            .unwrap()
            .with_fullscreen_events(4)
            .unwrap()
        };
        for explicit in [false, true] {
            let (mut dom, _hosted, document, subject) =
                source::operation::fullscreen::native_fixture(limits(100_000));
            let parser = surgeist_css::CssParserContext::new(
                if dom.snapshot().document(document).unwrap().metadata().mode()
                    == crate::HtmlDocumentMode::Quirks
                {
                    CssParserMode::Quirks
                } else {
                    CssParserMode::Standards
                },
            );
            let mut batch = DomBatch::new();
            let a = batch.create_element_ns(document, crate::Namespace::html(), "div");
            let b = batch.create_element_ns(document, crate::Namespace::html(), "span");
            batch.insert_before(&b, subject, None);
            let created = dom.apply(batch, &cancel).unwrap();
            let a = created.created(&a).unwrap().unwrap();
            let b = created.created(&b).unwrap().unwrap();
            dom.configure_inline_styles(
                DomInlineStyleLimits::new(Default::default(), 8, 64_000_000, 8_000_000, 4_000_000)
                    .unwrap(),
                &cancel,
            )
            .unwrap();
            let support = CssomInputVersion {
                role: CssomInputRole::Support,
                identity: "blocked-native-support".into(),
                revision: 1,
            };
            for element in [a, b] {
                let inputs = DomInlineStyleInputs::new(
                    dom.snapshot().document(document).unwrap().stamp(),
                    parser,
                    support.clone(),
                    Default::default(),
                )
                .unwrap();
                dom.inline_style(
                    element,
                    &inputs,
                    DomInlineStyleRequestLimits {
                        input: Default::default(),
                        declaration: Default::default(),
                    },
                    |_, _| panic!("empty"),
                    &cancel,
                )
                .unwrap();
            }
            let old = dom.capture_inline_style(a).unwrap().unwrap();
            dom.set_limits(limits(dom.usage().string_code_units()))
                .unwrap();
            let inputs = DomInlineStyleInputs::new(
                dom.snapshot().document(document).unwrap().stamp(),
                parser,
                support.clone(),
                Default::default(),
            )
            .unwrap();
            match dom
                .set_inline_style_css_text(
                    a,
                    &inputs,
                    "--x: 1",
                    DomInlineStyleMutationLimits {
                        declaration: Default::default(),
                        feedback: DomInlineStyleFeedbackLimits::new(
                            64_000_000, 1_000_000, 4_000_000, 64,
                        ),
                    },
                    |_| panic!("custom property"),
                    &cancel,
                )
                .unwrap()
            {
                DomInlineStyleMutation::Incomplete { operation, reason } => {
                    assert!(matches!(
                        reason,
                        DomInlineStyleError::Dom(DomError::ResourceLimit {
                            resource: DomResource::StringCodeUnits,
                            ..
                        })
                    ));
                    assert_eq!(
                        operation.owner_effects()[0].status(),
                        CssomOwnerEffectStatus::Pending
                    );
                }
                _ => panic!("actual canonical limit refuses"),
            }
            let before = dom.snapshot().stamp();
            let source_limits =
                crate::DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 256, 8192, 4);
            let mut source = if explicit {
                let context = crate::DomInlineStyleSourceContext::new(
                    before,
                    vec![
                        crate::DomInlineStyleProfile::new(
                            b,
                            document,
                            parser,
                            support,
                            Default::default(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap();
                dom.begin_source_edit_with_inline_styles(
                    crate::DomSourceEdit::detach(subject),
                    context,
                    source_limits,
                    DomInlineStyleRequestLimits {
                        input: Default::default(),
                        declaration: Default::default(),
                    },
                    &cancel,
                )
                .unwrap()
            } else {
                dom.begin_source_edit(
                    crate::DomSourceEdit::detach(subject),
                    source_limits,
                    &cancel,
                )
                .unwrap()
            };
            let result = if explicit {
                source
                    .advance_with_inline_styles(|_, _| panic!("no producer work"), &cancel)
                    .map(|p| p.into_parts().0)
                    .map_err(|e| match e {
                        DomInlineStyleError::Dom(e) => e,
                        other => panic!("unexpected {other:?}"),
                    })
            } else {
                source.advance(&cancel)
            };
            assert!(
                matches!(result, Err(DomError::InlineStyleOperationPending { element }) if element == a)
            );
            assert_eq!(source.snapshot().stamp(), before);
            assert!(source.snapshot().node(subject).unwrap().parent().is_some());
            drop(source);
            assert!(
                dom.pending_source_edit().is_none(),
                "no request escaped the refused segment"
            );
            dom.set_limits(limits(100_000)).unwrap();
            let completion = dom
                .resume_inline_style_feedback()
                .unwrap()
                .complete(&cancel);
            match completion {
                DomInlineStyleMutation::Complete(c) => {
                    assert_eq!(
                        c.owner_effects()[0].status(),
                        CssomOwnerEffectStatus::Applied
                    );
                }
                DomInlineStyleMutation::Incomplete { reason, .. } => {
                    panic!("unblocked original feedback {reason:?}")
                }
            }
            assert_eq!(old.properties().css_text(Default::default()).unwrap(), "");
            assert_eq!(
                dom.capture_inline_style(a)
                    .unwrap()
                    .unwrap()
                    .properties()
                    .css_text(Default::default())
                    .unwrap(),
                "--x: 1;"
            );
        }
    }
}
