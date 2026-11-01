//! Real Session execution consumes original policy, never the installed final policy.
//! This closed trusted adapter recognizes two serialized policies. It is not a
//! CSP parser/evaluator implementation or a grammar-conformance claim.
use super::super::meta::cancel;
use super::super::*;
use super::consumption::{envelope, parsing};
use super::style_decision::{installed, observe};
use super::style_sheet::{bind, create, pending};
use std::{convert::Infallible, mem::size_of, sync::Arc};
use surgeist_dom::*;
use surgeist_html::*;
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParsePendingAssociationStep, HtmlSource};

const ALLOW: &str = "style-src 'unsafe-inline'";
const DENY: &str = "style-src 'none'";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Rule {
    Inline,
    None,
}
#[derive(Debug)]
struct NativePolicy {
    rules: [Rule; 4],
    len: usize,
}
impl NativePolicy {
    fn layout() -> HtmlCspArtifactLayout {
        HtmlCspArtifactLayout::try_new(
            HtmlCspStorage::try_new(size_of::<Self>(), 0, 1).unwrap(),
            HtmlCspDisposalWork::try_new(8, size_of::<Self>()).unwrap(),
        )
        .unwrap()
    }
    fn allows_inline(&self) -> bool {
        self.rules[..self.len]
            .iter()
            .all(|rule| *rule == Rule::Inline)
    }
}
impl HtmlCspPolicyArtifact for NativePolicy {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        Self::layout()
    }
}
fn allow(_: HtmlCspLayout) -> Result<(), Infallible> {
    Ok(())
}
#[derive(Debug)]
enum FixtureError {
    Unrecognized,
    Capacity,
}
struct Processor {
    authority: HtmlCspProcessorAuthority,
    profile: HtmlCspProcessorProfile,
    calls: usize,
}
impl Processor {
    fn new() -> Self {
        let authority = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = authority.profile_with(1, &mut allow).unwrap();
        Self {
            authority,
            profile,
            calls: 0,
        }
    }
}
impl HtmlMetaCspProcessor for Processor {
    type Error = FixtureError;
    fn profile(&self) -> &HtmlCspProcessorProfile {
        &self.profile
    }
    fn prepare<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<Self::Error>, HtmlCspError<E>> {
        self.profile
            .validate_selected_with(request.qualification().processor(), admit)?;
        let prior = request
            .prior_policy()
            .borrow_native_with::<NativePolicy, E>(&self.profile, admit)?;
        let q = request.qualification();
        admit(HtmlCspLayout::processing(
            u64::try_from(q.content().len()).unwrap() * 4 + 16,
            HtmlCspStorage::default(),
            HtmlCspDisposalWork::default(),
            size_of::<NativePolicy>(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        assert_eq!(q.source(), HtmlMetaCspSource::Meta);
        assert_eq!(q.disposition(), HtmlMetaCspDisposition::Enforce);
        assert_eq!(
            q.removed_directives(),
            [
                HtmlMetaCspRemovedDirective::ReportUri,
                HtmlMetaCspRemovedDirective::FrameAncestors,
                HtmlMetaCspRemovedDirective::Sandbox
            ]
        );
        let rule = if q
            .content()
            .code_units()
            .iter()
            .copied()
            .eq(ALLOW.encode_utf16())
        {
            Rule::Inline
        } else if q
            .content()
            .code_units()
            .iter()
            .copied()
            .eq(DENY.encode_utf16())
        {
            Rule::None
        } else {
            return Ok(HtmlMetaCspOutcome::Failed(FixtureError::Unrecognized));
        };
        let (mut rules, len) = match prior {
            HtmlCspNativePolicy::InitialEmpty => ([Rule::Inline; 4], 0),
            HtmlCspNativePolicy::External(prior) => (prior.rules, prior.len),
        };
        if len == rules.len() {
            return Ok(HtmlMetaCspOutcome::Refused(FixtureError::Capacity));
        }
        rules[len] = rule;
        let layout = NativePolicy::layout();
        admit(HtmlCspLayout::processing(
            8,
            layout.storage(),
            layout.disposal(),
            size_of::<NativePolicy>(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        let native = Arc::new(NativePolicy {
            rules,
            len: len + 1,
        });
        self.calls += 1;
        match self.authority.attest_changed_with(request, native, admit) {
            Ok(prepared) => Ok(HtmlMetaCspOutcome::Completed(prepared)),
            Err(rejected) => {
                drop(rejected.retained);
                release(layout.storage());
                Err(rejected.error)
            }
        }
    }
}
fn parsed(
    first: &str,
    last: &str,
) -> (
    Arc<surgeist_html_parser::HtmlParseReport>,
    Processor,
    HtmlCspAssociation,
    HtmlCspAssociation,
) {
    let mut processor = Processor::new();
    let report = Arc::new(HtmlDocumentParseRequest::offline(HtmlSource::utf8(format!(
        "<!doctype html><head><meta http-equiv=content-security-policy content=\"{first}\"><style title=chosen nonce=original>p{{color:red}}</style><meta http-equiv=content-security-policy content=\"{last}\"></head>")), parsing())
        .parse_with_meta_csp(&mut processor).unwrap());
    assert_eq!(processor.calls, 2);
    let mut cursor = report.pending_associations();
    let mut policies = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParsePendingAssociationStep::Fault(fault) => panic!("{fault}"),
            HtmlParsePendingAssociationStep::Occurrence(row) if row.requires_processing() => {
                policies.push(row.context().csp_association().unwrap().clone())
            }
            HtmlParsePendingAssociationStep::Complete => break,
            _ => (),
        }
    }
    assert_eq!(
        policies.len(),
        2,
        "actual first and ParserPopped Style operations"
    );
    let second = policies.pop().unwrap();
    let first = policies.pop().unwrap();
    (report, processor, first, second)
}
fn associated(session: &DomSession, document: DocumentId) -> HtmlCspAssociation {
    let snapshot = session.snapshot();
    match snapshot
        .document(document)
        .unwrap()
        .metadata()
        .policy_container()
        .csp()
    {
        DomDocumentCspState::Associated(policy) => policy.clone(),
        _ => panic!("actual installed final association"),
    }
}
fn prepare(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
) -> DomParsedHtmlStyleQualification {
    assert_eq!(
        session
            .style_execution()
            .prepare_parsed_style(&mut session.dom(), id, envelope(), &cancel())
            .unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    let view = session.parsed_html_style_prefix(id).unwrap();
    assert!(view.initial_policy_request().is_none());
    assert!(view.policy_ready());
    assert_eq!(
        session
            .dom
            .parsed_html_dispatch_cursor()
            .advance(&cancel())
            .unwrap()
            .unwrap()
            .state(),
        DomParsedHtmlExecutionState::Unstarted
    );
    view.qualification().unwrap().clone()
}
fn decision(
    session: &DomSession,
    id: DomParsedHtmlAssociationId,
    expected: &HtmlCspAssociation,
    profile: &HtmlCspProcessorProfile,
) -> DomParsedHtmlStyleInlinePolicyObservation {
    observe(session, id, &mut |request| {
        assert_eq!(request.policy().document(), expected.document());
        assert_eq!(request.policy().generation(), expected.generation());
        assert_eq!(request.policy().policy(), expected.policy());
        let HtmlCspNativePolicy::External(native) = request
            .policy()
            .policy()
            .borrow_native_with::<NativePolicy, _>(profile, &mut allow)
            .unwrap()
        else {
            panic!("actual retained native policy")
        };
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request,
            native.allows_inline(),
        ))
    })
    .unwrap()
}
fn accept(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    input: &mut Option<DomParsedHtmlStyleInlinePolicyObservation>,
) -> DomParsedHtmlStylePhase {
    session
        .style_execution()
        .accept_parsed_style_inline_policy(&mut session.dom(), id, input, envelope(), &cancel())
        .unwrap()
}
fn render(session: &mut DomSession, q: &DomParsedHtmlStyleQualification) {
    session
        .style_execution()
        .exclude_parsed_style_script_bookkeeping(&mut session.dom(), q, envelope(), &cancel())
        .unwrap();
    assert!(matches!(
        session
            .style_execution()
            .process_parsed_style_rendering(&mut session.dom(), q, envelope(), &cancel())
            .unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
}
fn queue(
    session: &mut DomSession,
    q: &DomParsedHtmlStyleQualification,
) -> DomStyleNetworkingTaskId {
    match session
        .style_execution()
        .queue_parsed_style_notification(&mut session.dom(), q, envelope(), &cancel())
        .unwrap()
    {
        DomStyleParsedNotificationProgress::Queued { task } => task,
        other => panic!("actual NoImports processing queues: {other:?}"),
    }
}

#[test]
fn original_allow_policy_drives_both_invocations_despite_final_canonical_deny() {
    let (report, processor, first_policy, pop_policy) = parsed(ALLOW, DENY);
    let (mut session, installation) = installed(report);
    let id = installation.pending_association().unwrap();
    let final_policy = associated(&session, installation.document());
    assert_ne!(first_policy.policy().id(), final_policy.policy().id());
    assert_ne!(first_policy.document(), final_policy.document());
    let HtmlCspNativePolicy::External(final_native) = final_policy
        .policy()
        .borrow_native_with::<NativePolicy, _>(&processor.profile, &mut allow)
        .unwrap()
    else {
        panic!("native final policy")
    };
    assert!(!final_native.allows_inline());
    let q = prepare(&mut session, id);
    let mut input = Some(decision(&session, id, &first_policy, &processor.profile));
    assert!(input.as_ref().unwrap().allowed());
    assert_eq!(
        accept(&mut session, id, &mut input),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    bind(&mut session, installation.document(), Default::default());
    assert!(
        session
            .parsed_html_style_prefix(id)
            .unwrap()
            .qualification()
            .is_some()
    );
    let first = create(&mut session, id, envelope()).unwrap();
    let first_sheet = first.completion().sheet().clone();
    drop(first);
    assert!(
        session.state.borrow().completions[0]
            .processing()
            .unwrap()
            .qualification
            == q
    );
    {
        let snapshot = session
            .style_execution()
            .snapshot(installation.document().node())
            .unwrap()
            .unwrap();
        assert!(
            snapshot
                .rules(snapshot.sheet(first_sheet.id()).unwrap().rules())
                .unwrap()
                .is_empty()
        );
    }
    render(&mut session, &q);
    let first_task = queue(&mut session, &q);
    let output = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    assert!(matches!(
        output.removal(),
        DomParsedHtmlStyleSuccessorRemoval::Removed { .. }
    ));
    let pop = output.qualification().clone();
    drop(output);
    assert_ne!(pop, q);
    {
        let view = session.parsed_html_style_prefix(id).unwrap();
        assert!(view.initial_policy_request().is_none());
        let request = view.inline_policy_request().unwrap();
        assert_eq!(request.text(), &DocumentString::from("p{color:red}"));
        assert_eq!(request.nonce(), Some(&DocumentString::from("original")));
    }
    let mut input = Some(decision(&session, id, &pop_policy, &processor.profile));
    assert_eq!(
        accept(&mut session, id, &mut input),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    let created = create(&mut session, id, envelope()).unwrap();
    let new_sheet = created.completion().sheet().clone();
    drop(created);
    assert_ne!(first_sheet.id(), new_sheet.id());
    {
        let snapshot = session
            .style_execution()
            .snapshot(installation.document().node())
            .unwrap()
            .unwrap();
        assert_eq!(
            snapshot
                .rules(snapshot.sheet(new_sheet.id()).unwrap().rules())
                .unwrap()
                .len(),
            1
        );
    }
    render(&mut session, &pop);
    let pop_task = queue(&mut session, &pop);
    let mut context = Context {
        styles: session.style_execution(),
        source: session.source_execution(),
        events: Vec::new(),
    };
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 16,
        nested_dispatches: 8,
    })
    .unwrap();
    dispatcher
        .add_listener(
            &mut session.dom(),
            first_sheet.association().element(),
            DomEventType::Load,
            DomEventCallback::new(|_, _, context: &mut Context, event| {
                context.events.push((
                    event.target().unwrap(),
                    event.current_target().unwrap().node().unwrap(),
                ));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    assert!(
        matches!(DomStyleRuntime::run_networking(&mut session.dom(), &mut dispatcher, &mut context, dispatch_limits(), &cancel()).unwrap(), DomStyleNetworkingProgress::OriginalSynchronousPending { task, .. } if task == first_task)
    );
    assert!(context.events.is_empty());
    drop(
        session
            .style_execution()
            .complete_parsed_html_synchronous_execution(
                &mut session.dom(),
                id,
                envelope(),
                &cancel(),
            )
            .unwrap(),
    );
    for expected in [first_task, pop_task] {
        assert!(
            matches!(DomStyleRuntime::run_networking(&mut session.dom(), &mut dispatcher, &mut context, dispatch_limits(), &cancel()).unwrap(), DomStyleNetworkingProgress::Complete { task, .. } if task == expected)
        );
    }
    assert_eq!(
        context.events,
        vec![
            (
                first_sheet.association().element(),
                first_sheet.association().element()
            );
            2
        ]
    );
    assert_eq!(
        processor.calls, 2,
        "installation and execution never replay original processing"
    );
    pending(&session, id);
}

#[test]
fn genuine_known_blocked_decisions_need_no_cssom_owner_and_keep_each_original_policy() {
    let (report, processor, first_policy, pop_policy) = parsed(DENY, ALLOW);
    let (mut session, installation) = installed(report);
    let id = installation.pending_association().unwrap();
    prepare(&mut session, id);
    let mut input = Some(decision(&session, id, &first_policy, &processor.profile));
    assert!(!input.as_ref().unwrap().allowed());
    assert_eq!(
        accept(&mut session, id, &mut input),
        DomParsedHtmlStylePhase::InlinePolicyBlocked
    );
    let output = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    assert!(matches!(
        output.removal(),
        DomParsedHtmlStyleSuccessorRemoval::NoAssociatedSheet
    ));
    let pop = output.qualification().clone();
    drop(output);
    let mut input = Some(decision(&session, id, &pop_policy, &processor.profile));
    assert!(!input.as_ref().unwrap().allowed());
    assert_eq!(
        accept(&mut session, id, &mut input),
        DomParsedHtmlStylePhase::InlinePolicyBlocked
    );
    assert_eq!(
        session
            .parsed_html_style_prefix(id)
            .unwrap()
            .qualification(),
        Some(&pop)
    );
    assert!(
        session
            .style_execution()
            .snapshot(installation.document().node())
            .unwrap()
            .is_none()
    );
    assert!(session.state.borrow().completions.is_empty());
    drop(
        session
            .style_execution()
            .complete_parsed_html_synchronous_execution(
                &mut session.dom(),
                id,
                envelope(),
                &cancel(),
            )
            .unwrap(),
    );
    assert_eq!(processor.calls, 2);
    pending(&session, id);
}

#[test]
fn known_decision_rejects_foreign_occurrence_and_preserves_input_on_cancel_and_raised_retry() {
    let (report, processor, first_policy, _) = parsed(ALLOW, DENY);
    let (mut a, installed_a) = installed(report.clone());
    let (mut b, installed_b) = installed(report);
    let a_id = installed_a.pending_association().unwrap();
    let b_id = installed_b.pending_association().unwrap();
    prepare(&mut a, a_id);
    prepare(&mut b, b_id);
    let mut foreign = Some(decision(&b, b_id, &first_policy, &processor.profile));
    assert!(matches!(
        a.style_execution().accept_parsed_style_inline_policy(
            &mut a.dom(),
            a_id,
            &mut foreign,
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::WrongRequest
        ))
    ));
    assert!(foreign.is_some());
    let mut input = Some(decision(&a, a_id, &first_policy, &processor.profile));
    assert!(
        a.style_execution()
            .accept_parsed_style_inline_policy(
                &mut a.dom(),
                a_id,
                &mut input,
                envelope(),
                &AtomicBool::new(true)
            )
            .is_err()
    );
    assert!(input.is_some());
    let mut runtime = a.style_execution();
    let owner = runtime.original().unwrap();
    let before = owner.borrow().work.get();
    runtime.set_limits(&a.dom(), limits(), &cancel()).unwrap();
    let cost = owner.borrow().work.get() - before;
    let mut small = limits();
    small.work = owner.borrow().work.get() + cost;
    runtime.set_limits(&a.dom(), small, &cancel()).unwrap();
    assert!(matches!(
        runtime.accept_parsed_style_inline_policy(
            &mut a.dom(),
            a_id,
            &mut input,
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedPrefixError::Runtime(
            DomStyleRuntimeError::Admission
        ))
    ));
    assert!(input.is_some());
    assert_eq!(
        a.parsed_html_style_prefix(a_id).unwrap().phase(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    runtime.set_limits(&a.dom(), limits(), &cancel()).unwrap();
    assert_eq!(
        accept(&mut a, a_id, &mut input),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert!(input.is_none());
    assert_eq!(processor.calls, 2);
}
