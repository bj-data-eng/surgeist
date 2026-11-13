mod authored_toggle;
mod declarative_shadow;
mod element_tasks;
mod inline_style;
mod nested_dispatch;
mod trusted_attribute;

use super::*;
use crate::{
    DomElementTaskRuntimeLimits, DomEventCallback, DomEventDispatchError, DomEventDispatchLimits,
    DomEventDispatcher, DomEventHostError, DomEventListenerOptions, DomEventRegistryLimits,
    DomEventStorageLimits, DomEventType, DomHostedRuntimeLimits, DomHostedTaskProgress,
    DomSessionLimits, DomShutdownOutcome, DomSourceDriveProgress, DomSourceEventRuntimeError,
    DomSourceEventRuntimeLimits, DomSourceEventServiceLimits, DomSourceEventServiceProgress,
    DomSourceExecutionScope, DomSourceRuntimeError, DomSourceServices, DomSourceUnavailable,
    DomStyleRuntimeLimits, NativeDomEvent,
};
use surgeist_bindings_js::{Root, Value};
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_dom::{
    Dom, DomBatch, DomDispatchCustodyLimits, DomHostedLimits, DomHostedRecordLimits, DomLimits,
    DomNameLimits, DomSourceEdit, DomSourceEventRequest, DomSourceLimits, DomSourceRequest, NodeId,
};
use surgeist_html::values::Namespace;

pub(super) fn hosted_session(
    cancel: &AtomicBool,
    source_jobs: usize,
) -> (DomSession, DomHostedTopLevel) {
    hosted_session_with_details(cancel, source_jobs, 0)
}

fn hosted_session_with_details(
    cancel: &AtomicBool,
    source_jobs: usize,
    details_intentions: usize,
) -> (DomSession, DomHostedTopLevel) {
    hosted_session_with_dom_limits(cancel, source_jobs, details_intentions, |limits| limits)
}

fn hosted_session_with_dom_limits(
    cancel: &AtomicBool,
    source_jobs: usize,
    details_intentions: usize,
    configure: impl FnOnce(DomLimits) -> DomLimits,
) -> (DomSession, DomHostedTopLevel) {
    let dom = Dom::new(configure(scripting_dom_limits(200_000, details_intentions))).unwrap();
    hosted_session_with_dom(cancel, source_jobs, dom)
}

fn scripting_dom_limits(units: usize, details_intentions: usize) -> DomLimits {
    DomLimits::new(
        128,
        units,
        256,
        256,
        256,
        32,
        1_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_dispatch_custody(
        DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
    )
    .with_details_toggle_intentions(details_intentions)
    .unwrap()
}

fn hosted_session_with_dom(
    cancel: &AtomicBool,
    source_jobs: usize,
    dom: Dom,
) -> (DomSession, DomHostedTopLevel) {
    let mut session = DomSession::new(
        dom,
        DomSessionLimits {
            authored_events: DomEventStorageLimits {
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            source_events: DomSourceEventRuntimeLimits {
                jobs: source_jobs,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 4,
                completions: 32,
                reports: 32,
                storage_bytes: 16_000_000,
                work: 1_000_000,
                css_input: CssUtf16InputLimits::default(),
            },
            element_tasks: DomElementTaskRuntimeLimits {
                tasks: 32,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            hosted: DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                    16_000_000,
                    16_000_000,
                    1_000_000,
                ),
                tasks: 32,
                storage_bytes: 16_000_000,
                scratch_bytes: 16_000_000,
                work: 1_000_000,
            },
        },
    )
    .unwrap();
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let owner = hosted.create_host(&mut dom, cancel).unwrap();
    let target = hosted.create_top_level(&mut dom, owner, cancel).unwrap();
    (session, target)
}

#[test]
fn an_installed_hook_delegates_its_document_to_a_direct_cross_realm_call() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let read_name = "readChildCount".encode_utf16().collect::<Vec<_>>();
    let mut host = enable(session, target, cancel.clone());
    let permit = host.permit().unwrap();
    let (hook, escape, window_type_error) = host
        .in_document(&permit, |scope| {
            let global = scope.global()?;
            let callback = scope.get(&global, PropertyKey::String(&read_name))?;
            let escape = scope.evaluate(&"CSS.escape".encode_utf16().collect::<Vec<_>>())?;
            let type_error = scope.evaluate(&"TypeError".encode_utf16().collect::<Vec<_>>())?;
            Ok((
                scope.retain(&callback),
                scope.retain(&escape),
                scope.retain(&type_error),
            ))
        })
        .unwrap();

    // Only the test enters another realm through the private Process. Give
    // that realm the actual installed function, not a new native callback.
    let caller = {
        let engine = host.engine.as_mut().unwrap();
        let mut execution = engine.process.execution();
        let caller = execution.create_realm().unwrap();
        execution
            .in_realm(&caller, |scope| {
                assert_ne!(scope.realm_authority()?.identity(), permit.realm);
                let global = scope.global()?;
                let delegate = scope.import(&hook)?;
                scope.set(&global, PropertyKey::String(&read_name), &delegate)?;
                let escape = scope.import(&escape)?;
                let window_type_error = scope.import(&window_type_error)?;
                scope.set(
                    &global,
                    PropertyKey::String(&"windowEscape".encode_utf16().collect::<Vec<_>>()),
                    &escape,
                )?;
                scope.set(
                    &global,
                    PropertyKey::String(&"windowTypeError".encode_utf16().collect::<Vec<_>>()),
                    &window_type_error,
                )?;
                let observed = scope.evaluate(
                    &r#"(() => {
                        let fromWindow = false;
                        try { windowEscape(); }
                        catch (error) {
                            fromWindow = error instanceof windowTypeError
                                && !(error instanceof TypeError);
                        }
                        return !Object.hasOwn(globalThis, "CSS") && fromWindow
                            && windowEscape("\ud800") === "\ud800";
                    })()"#
                        .encode_utf16()
                        .collect::<Vec<_>>(),
                )?;
                let expected = scope.primitive(Primitive::Boolean(true))?;
                assert!(scope.strict_equals(&observed, &expected)?);
                Ok(())
            })
            .unwrap();
        caller
    };

    for expected in 1..=2 {
        let engine = host.engine.as_mut().unwrap();
        engine
            .process
            .execution()
            .in_realm(&caller, |scope| {
                assert_ne!(scope.realm_authority()?.identity(), permit.realm);
                let count =
                    scope.evaluate(&"readChildCount()".encode_utf16().collect::<Vec<_>>())?;
                let value = scope.primitive(Primitive::Number(f64::from(expected)))?;
                assert!(scope.strict_equals(&count, &value)?);
                Ok(())
            })
            .unwrap();
        if expected == 1 {
            host.with_native(|dom, _, _| {
                let mut batch = DomBatch::new();
                let document = target.document().node();
                let comment = batch.create_comment(document, "native child");
                batch.insert_before(&comment, document, None);
                dom.apply(batch, &cancel).unwrap();
            })
            .unwrap();
        }
    }

    drop(caller);
    drop(window_type_error);
    drop(escape);
    drop(hook);
    let session = host.stop().into_session().unwrap();
    assert_eq!(
        session
            .snapshot()
            .node(target.document().node())
            .unwrap()
            .children()
            .len(),
        2
    );
    drop(session);
    projected_comment_custody_outlives_collection_and_the_origin_realm_token();
    rejected_comment_requires_each_diagnostic_utf16_work_unit();
    window_binding_storage_admits_exact_and_refuses_one_byte_short();
    super::guest::tests::adopted_element_uses_its_registered_creation_realm();
    super::guest::tests::registered_origin::native_document_and_fragment_use_their_registered_realm(
    );
    super::guest::tests::registered_origin::registered_origin_refusals_preserve_state();
    super::guest::tests::registered_origin::character_data_observation_admits_native_lookup();
    super::guest::tests::registered_origin::registered_origin_character_factories_use_receiver_realm();
    super::guest::tests::registered_origin::document_element_factories_use_native_names_and_identity();
    super::guest::tests::registered_origin::borrowed_element_factory_keeps_dom_on_wrapper_refusal();
    super::guest::tests::registered_origin::element_factory_admits_exact_projection_work();
    super::guest::tests::custom_element_registry_identity_is_canonical();
    super::guest::tests::element_factory_qualifies_the_explicit_registry();
    super::guest::tests::registry_definition_preserves_guest_identity_and_failure();
    super::guest::tests::document_factories_execute_the_reached_custom_constructor();
    super::guest::tests::registered_origin::custom_element_factories_use_the_correct_realm();
    super::guest::tests::registered_origin::registered_origin_registry_keeps_exact_wrapper();
    super::guest::tests::registered_origin::registered_character_factory_refuses_without_publication(
    );
    public_and_source_comment_projection_share_identity();
    attached_registry_delivers_the_original_native_cancel_event();
    attached_registry_keeps_preparation_refusal_on_retry();
    attached_registry_preserves_the_original_terminal_refusal();
    attached_beforetoggle_uses_the_original_toggle_event();
    element_tasks::verify_attached_element_tasks();
    nested_dispatch::verify_nested_authored_dispatch();
    authored_toggle::verify_authored_toggle_constructor();
    super::guest::tests::attributes::detached_attribute_keeps_its_live_native_identity();
    super::guest::tests::attributes::element_methods_return_the_exact_attribute();
    super::guest::tests::attributes::refused_verified_replacement_keeps_its_exact_output();
    super::guest::tests::attributes::inherited_node_values_use_canonical_text();
    trusted_attribute::hosted_attribute_policy_reads_charge_the_parked_source_and_preserve_exact_input();
    trusted_attribute::hosted_meta_source_publishes_ordered_csp_for_the_registered_attribute_verifier();
    trusted_attribute::guest_tree_edits_deliver_original_meta_policies();
    inline_style::guest_insertion_publishes_a_sheet_under_the_known_empty_policy();
    inline_style::style_runtime_refusal_preserves_the_original_guest_edit();
    inline_style::guest_style_preserves_foreign_and_unavailable_policy();
    inline_style::guest_style_reports_to_current_window_for_another_document();
    inline_style::guest_insertion_uses_the_effective_meta_style_directive();
    inline_style::guest_style_update_matches_the_exact_nonce();
    inline_style::guest_style_hashes_exact_utf16_source();
    inline_style::guest_style_violations_retarget_at_dispatch();
    inline_style::character_data_edit_in_a_native_listener_settles_style_before_return();
    trusted_attribute::guest_meta_without_processor_preserves_original_source();
    trusted_attribute::guest_meta_retry_preserves_accepted_first_policy();
    trusted_attribute::guest_attribute_policy_keeps_original_authority_after_adoption();
    trusted_attribute::guest_attribute_policy_verifies_html_and_svg_sinks();
    trusted_attribute::guest_verification_without_callback_retains_the_native_continuation();
    trusted_attribute::default_factory_retains_the_real_guest_callback();
    super::guest::tests::trusted_policy::window_factory_creates_the_exact_default_policy();
    super::guest::tests::trusted_policy::guest_attribute_writes_reach_the_native_element();
    declarative_shadow::verify_declarative_shadow_reuse();
    source_comment_qualification_precedes_foreign_handler();
    super::guest::tests::registered_origin::borrowed_fragment_factory_keeps_dom_on_wrapper_refusal(
    );
    super::guest::tests::shadow_root::native_open_shadow_root_uses_the_canonical_interface();
    super::guest::tests::move_before::parent_node_move_uses_the_live_native_identities();
    super::guest::tests::shadow_root::scripted_attach_publishes_the_exact_native_shadow_root();
    super::guest::tests::shadow_root::scripted_shadow_conversion_preserves_order_and_identity();
    super::guest::tests::shadow_root::scripted_shadow_revalidates_after_reentry();
    super::guest::tests::shadow_root::scripted_shadow_source_refusal_keeps_the_original_state();
    super::guest::tests::shadow_root::scripted_shadow_selects_the_hosted_registry_and_clones_null();
    super::guest::tests::shadow_root::closed_root_keeps_identity_and_native_event_visibility();
    super::guest::tests::shadow_root::shadow_projection_admits_exact_work_and_backing();
    super::guest::tests::shadow_root::closed_root_visibility_admits_the_canonical_lookup();
    super::guest::tests::registered_origin::shadow_root_keeps_its_registered_creation_realm();
    super::guest::tests::registered_origin::scripted_shadow_uses_the_current_document_creation_realm();
    super::guest::tests::registered_origin::native_document_constructor_refusal_releases_private_wrapper();
    super::guest::tests::element_projection_admits_exact_work_and_backing();
    super::guest::tests::refused_element_realm_installation_releases_guest_backing();
    super::guest::tests::native_element_ancestors_are_projected_and_metered();
    super::guest::tests::qualified_tag_presentation_admits_its_utf16_work();
    super::guest::tests::unpublished_window_requires_its_original_bound_registry();
    super::guest::tests::fragment_custody_survives_collection_and_releases_on_retirement();
    super::guest::tests::fragment_watch_refusal_preserves_native_state();
    super::guest::tests::character_data_source_refusal_preserves_native_state();
    super::guest::tests::refused_first_target_releases_prototype_capture();
    super::guest::tests::refused_projection_releases_native_backing_before_retry();
    super::guest::tests::attached_registered_function_uses_the_exact_native_invocation();
    super::guest::tests::attached_proxy_uses_its_own_realm();
    super::guest::tests::attached_object_listeners_look_up_handle_event_when_invoked();
    super::guest::tests::attached_window_event_follows_the_listener_invocation();
    super::guest::tests::attached_window_event_clears_on_terminal_listener_refusal();
    super::guest::tests::attached_guest_dispatch_restores_the_outer_window_event();
    super::guest::tests::direct_guest_dispatch_preserves_reports_and_object_custody();
    super::guest::tests::nested_guest_reports_keep_observation_order_and_depth();
    super::guest::tests::full_guest_report_reservation_keeps_exact_invalid_state();
    super::guest::tests::full_guest_report_bank_preserves_original_registry_work();
    super::window_timers::tests::run_timer_tests();
    super::guest::tests::document_factories_upgrade_the_reached_customized_builtin();
}

fn window_binding_storage_admits_exact_and_refuses_one_byte_short() {
    use surgeist_bindings_js::conversion::{CleanupStatus, ConversionAccount};
    use surgeist_css::serialization::CssomIdentifierLimits;

    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let probe = enable(session, target, cancel.clone());
    let permit = probe.permit().unwrap();
    let read = child_count_callback(Rc::downgrade(&probe.native), permit.clone());
    let detach = source::detach_callback(Rc::downgrade(&probe.native), permit);

    // Independent layout oracle for the native budget. The Bindings native
    // qualification checks these account field shapes against the actual
    // retained Rc<Account> targets. The hosted budget counts requested Rust
    // payload bytes and excludes allocator and V8 overhead.
    let account = size_of::<(Weak<()>, Arc<()>, (u64, usize), Cell<CleanupStatus>)>();
    let dom_exception = size_of::<(Rc<ConversionAccount>, DomHostedRuntimeRetention)>()
        + size_of::<ConversionAccount>()
        + size_of::<Weak<ConversionAccount>>()
        + account;
    let escape = size_of::<(
        ConversionAccount,
        CssomIdentifierLimits,
        HostFailureSlot<CssSerializationError, DomHostedRuntimeRetention>,
        DomHostedRuntimeRetention,
    )>() + account;
    let window_event =
        size_of::<(RefCell<Option<Root>>, DomHostedRuntimeRetention)>() + 2 * size_of::<Rc<()>>();
    // The four relations each have a getter. Only window has no setter.
    let window_relations = size_of::<(
        Weak<NativeOwner>,
        DocumentScriptPermit,
        DomHostedRuntimeRetention,
    )>() + 7 * size_of::<(Rc<()>, &'static ())>();
    let exact = size_of::<NativeOwner>()
        + size_of_val(&read)
        + size_of_val(&detach)
        + FailureSlot::storage_bytes()
        + SourceFailureSlot::storage_bytes()
        + DomDocumentScriptRegistration::storage_bytes()
        + REGISTRATION_ACCESS_BYTES
        + escape
        + HostFailureSlot::<CssSerializationError, DomHostedRuntimeRetention>::storage_bytes()
        + dom_exception
        + window_relations
        + window_event;
    drop(read);
    drop(detach);

    let policy = HtmlScriptingPolicy {
        user_allows: true,
        automation_allows: true,
    };
    let child_count = "readChildCount".encode_utf16().collect::<Vec<_>>();
    let detach_source_node_if = "detachIf".encode_utf16().collect::<Vec<_>>();
    let hooks = || DocumentScriptHooks {
        child_count: &child_count,
        detach_source_node_if: &detach_source_node_if,
    };
    let native_bindings_are_live = |host: &mut DocumentScriptHost| {
        let permit = host.permit().unwrap();
        host.in_document(&permit, |scope| {
            let script = r#"(() => {
                const current = Object.getOwnPropertyDescriptor(globalThis, "event");
                return new DOMException().name === "Error" && CSS.escape("0") === "\\30 " &&
                       window === globalThis && self === window && parent === window && opener === null &&
                       globalThis.event === undefined && current.enumerable && current.configurable &&
                       current.get.name === "get event" && current.get.length === 0 &&
                       current.set.name === "set event" && current.set.length === 1;
            })()"#
            .encode_utf16()
            .collect::<Vec<_>>();
            let value = scope.evaluate(&script)?;
            assert_eq!(scope.to_utf16(&value)?, [116, 114, 117, 101]);
            Ok(())
        })
        .unwrap();
    };

    let session = probe.stop().into_session().unwrap();
    let mut host = PreparedDocumentScriptHost::prepare(session, exact, cancel.clone())
        .unwrap()
        .enable(target, policy, hooks())
        .unwrap();
    native_bindings_are_live(&mut host);
    let session = host.stop().into_session().unwrap();
    let original = session.snapshot().stamp();
    let one_short = exact - 1;
    let refused = match PreparedDocumentScriptHost::prepare(session, one_short, cancel.clone())
        .unwrap()
        .enable(target, policy, hooks())
    {
        Err(refused) => refused,
        Ok(mut host) => {
            native_bindings_are_live(&mut host);
            let charged = host.engine.as_ref().unwrap().retained_bytes;
            panic!(
                "one-byte-short native Window published bindings: limit={one_short}, required={exact}, charged={charged}, account={account}"
            );
        }
    };
    assert!(matches!(
        refused.error,
        DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
    ));
    let session = refused.owner.into_session().unwrap();
    let snapshot = session.snapshot();
    assert_eq!(snapshot.stamp(), original);
    assert_eq!(
        snapshot.document(target.document()).unwrap().scripting(),
        surgeist_html_semantics::HtmlScriptingStatus::Disabled
    );
    let mut restored = PreparedDocumentScriptHost::prepare(session, exact, cancel)
        .unwrap()
        .enable(target, policy, hooks())
        .unwrap();
    assert_eq!(restored.engine.as_ref().unwrap().retained_bytes, exact);
    native_bindings_are_live(&mut restored);
    restored.stop().into_session().unwrap();
}

fn enable(
    session: DomSession,
    target: DomHostedTopLevel,
    cancel: Arc<AtomicBool>,
) -> DocumentScriptHost {
    PreparedDocumentScriptHost::prepare(session, 1_000_000, cancel)
        .unwrap()
        .enable(
            target,
            HtmlScriptingPolicy {
                user_allows: true,
                automation_allows: true,
            },
            DocumentScriptHooks {
                child_count: &"readChildCount".encode_utf16().collect::<Vec<_>>(),
                detach_source_node_if: &"detachIf".encode_utf16().collect::<Vec<_>>(),
            },
        )
        .unwrap()
}

const SOURCE: DomSourceLimits =
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 8);
const EVENT: DomSourceEventServiceLimits = DomSourceEventServiceLimits {
    source: SOURCE,
    dispatch: DomEventDispatchLimits {
        path_items: 16,
        snapshot_listeners: 16,
        failure_records: 16,
        callbacks: 16,
        work: 100_000,
    },
};

struct Projection {
    host: DocumentScriptHost,
    permit: DocumentScriptPermit,
    execution: DomSourceExecutionScope,
    dispatcher: DomEventDispatcher<()>,
    dialog: NodeId,
    comment: NodeId,
}

impl Projection {
    fn expose(&mut self, handler: &str, cancel: &AtomicBool) {
        let (request, ()) = self.invoke(handler, SOURCE, cancel, |outcome| {
            assert!(
                matches!(outcome, DocumentScriptGuestResult::Returned),
                "{outcome:?}"
            );
        });
        self.finish(&request, cancel);
    }

    fn invoke<T>(
        &mut self,
        handler: &str,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
        observe: impl FnOnce(&DocumentScriptGuestResult) -> T,
    ) -> (DomSourceEventRequest, T) {
        let handler: Root = self
            .host
            .in_document(&self.permit, |scope| {
                let handler = evaluate(scope, handler)?;
                Ok(scope.retain(&handler))
            })
            .unwrap();
        self.invoke_root(&handler, limits, cancel, observe)
    }

    fn invoke_root<T>(
        &mut self,
        handler: &Root,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
        observe: impl FnOnce(&DocumentScriptGuestResult) -> T,
    ) -> (DomSourceEventRequest, T) {
        let mut services = DomSourceServices {
            meta: None,
            execution_scope: Some(&self.execution),
            style_policy: &mut |_| None,
            url: None,
        };
        let request = self
            .host
            .with_native(|dom, _, mut source| {
                assert!(matches!(
                    source
                        .apply(
                            dom,
                            DomSourceEdit::set_native_attribute(self.dialog, "open", ""),
                            SOURCE,
                            &mut services,
                            cancel,
                        )
                        .unwrap(),
                    DomSourceDriveProgress::Complete(_)
                ));
                let progress = source
                    .apply(
                        dom,
                        DomSourceEdit::close_dialog(self.dialog, None),
                        limits,
                        &mut services,
                        cancel,
                    )
                    .unwrap();
                let DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::Event(request),
                    reason: DomSourceUnavailable::Event,
                } = progress
                else {
                    panic!("actual Dialog Close must reach beforetoggle: {progress:?}");
                };
                request
            })
            .unwrap();
        let progress = self
            .host
            .select_source_event(
                &self.permit,
                &request,
                &mut self.dispatcher,
                &mut (),
                DomSourceEventServiceLimits {
                    source: limits,
                    ..EVENT
                },
            )
            .unwrap();
        let DocumentScriptEventProgress::Selected(selected) = progress else {
            panic!("actual native listener must be selected");
        };
        let mut report = selected.invoke(handler, self.comment, limits);
        let outcome = observe(report.outcome());
        report.report(&self.dispatcher).unwrap();
        (request, outcome)
    }

    fn finish(&mut self, request: &DomSourceEventRequest, cancel: &AtomicBool) {
        {
            let progress = self
                .host
                .select_source_event(&self.permit, request, &mut self.dispatcher, &mut (), EVENT)
                .unwrap();
            let DocumentScriptEventProgress::Native(DomSourceEventServiceProgress::Serviced {
                receipt,
            }) = progress
            else {
                panic!("the original dispatcher must publish the event receipt");
            };
            assert!(receipt.failures().is_empty());
        }
        let mut services = DomSourceServices {
            meta: None,
            execution_scope: Some(&self.execution),
            style_policy: &mut |_| None,
            url: None,
        };
        self.host
            .with_native(|dom, _, mut source| {
                let progress = source
                    .resume(dom, request.operation(), SOURCE, &mut services, cancel)
                    .unwrap();
                assert!(matches!(
                    progress,
                    DomSourceDriveProgress::Complete(done)
                        if done.operation() == request.operation()
                ));
                assert!(dom.pending_source_edit().is_none());
            })
            .unwrap();
    }

    fn parent(&self) -> Option<NodeId> {
        self.host
            .snapshot()
            .unwrap()
            .node(self.comment)
            .unwrap()
            .parent()
    }
}

fn native_projection(cancel: Arc<AtomicBool>) -> (Projection, DomHostedTopLevel) {
    native_projection_with_registry_route(cancel, false, true)
}

fn native_projection_with_registry_route(
    cancel: Arc<AtomicBool>,
    public: bool,
    register_origin: bool,
) -> (Projection, DomHostedTopLevel) {
    let (mut session, receiving) = hosted_session(&cancel, 8);
    let execution = session
        .source_execution_scope(receiving.host(), &cancel)
        .unwrap();
    let document = receiving.document();
    let html = session
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = session.snapshot().node(html).unwrap().children()[1];
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let mut create = DomBatch::new();
    let dialog = create.create_element_ns(document.node(), Namespace::html(), "dialog");
    let comment = create.create_comment(origin.document().node(), "A's native Comment");
    create.insert_before(&dialog, body, None);
    create.insert_before(&comment, origin.document().node(), None);
    let commit = session.dom().apply(create, &cancel).unwrap();
    let dialog = commit.created(&dialog).unwrap().unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    assert_eq!(
        session.snapshot().node(comment).unwrap().owner_document(),
        Some(origin.document())
    );
    let mut adopt = DomBatch::new();
    adopt.insert_before(comment, document.node(), None);
    session.dom().apply(adopt, &cancel).unwrap();
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.node(comment).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(
        snapshot.relevant_global(comment).unwrap(),
        Some(origin.global())
    );
    let registry_limits = DomEventRegistryLimits {
        registrations: 4,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    };
    let mut dispatcher = DomEventDispatcher::new(registry_limits).unwrap();
    dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BEFORE_TOGGLE,
            DomEventCallback::host(77),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let (mut host, dispatcher) = if public {
        let mut registry = Some(dispatcher);
        let host = super::guest::tests::attach_public_nodes_with_registry(
            session,
            receiving,
            cancel,
            10_000,
            100_000,
            &mut registry,
        );
        let replacement = DomEventDispatcher::new(registry_limits).unwrap();
        let dispatcher =
            super::guest::tests::move_public_registry_for_source_test(&host, replacement);
        (host, dispatcher)
    } else {
        (enable(session, receiving, cancel), dispatcher)
    };
    let permit = host.permit().unwrap();
    if register_origin {
        host.register_node_creation_realm(&permit, origin).unwrap();
    }
    (
        Projection {
            host,
            permit,
            execution,
            dispatcher,
            dialog,
            comment,
        },
        origin,
    )
}

fn public_and_source_comment_projection_share_identity() {
    for public_first in [true, false] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut projection, _) = native_projection_with_registry_route(cancel.clone(), true, true);
        let expose_public = |projection: &mut Projection| {
            super::guest::tests::expose_public_node_for_source_test(
                &mut projection.host,
                &projection.permit,
                projection.comment,
                "publicComment",
            );
        };
        if public_first {
            expose_public(&mut projection);
            projection
                .host
                .in_document(&projection.permit, |scope| {
                    evaluate(scope, "publicComment.marker = 42").map(drop)
                })
                .unwrap();
            projection.expose(
                r#"comment => {
                    if (comment !== publicComment || comment.marker !== 42) {
                        throw new Error("Source must delegate the original public Comment");
                    }
                    if (comment.nodeValue !== "A's native Comment") throw new Error("wrong Source nodeValue");
                    comment.textContent = 'new\ud800';
                    if (comment.nodeValue !== 'new\ud800') throw new Error("wrong Source textContent");
                    detachIf(1, publicComment);
                }"#,
                &cancel,
            );
            assert_eq!(projection.parent(), None);
        } else {
            projection.expose("comment => { globalThis.fromSource = comment; }", &cancel);
            expose_public(&mut projection);
            projection
                .host
                .in_document(&projection.permit, |scope| {
                    let actual = evaluate(scope, "fromSource === publicComment")?;
                    let expected = scope.primitive(Primitive::Boolean(true))?;
                    assert!(scope.strict_equals(&actual, &expected)?);
                    Ok(())
                })
                .unwrap();
            projection
                .host
                .with_native(|dom, _, _| {
                    let mut retire = DomBatch::new();
                    retire.detach(projection.comment);
                    retire.destroy(projection.comment);
                    dom.apply(retire, &cancel).unwrap();
                })
                .unwrap();
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) =
                projection.host.in_document(&projection.permit, |scope| {
                    evaluate(scope, "publicComment.length").map(drop)
                })
            else {
                panic!("the Source-first Comment must observe native retirement");
            };
            assert!(matches!(
                failure.cause::<super::DocumentGuestEventError>(),
                Some(super::DocumentGuestEventError::Document(
                    DocumentScriptError::Native(DocumentScriptNativeError::Dom(
                        surgeist_dom::DomError::StaleNode
                    ))
                ))
            ));
        }
    }
}

fn source_comment_qualification_precedes_foreign_handler() {
    let cancel = Arc::new(AtomicBool::new(false));
    let foreign_handler = {
        let (mut projection, _) = native_projection(cancel.clone());
        projection
            .host
            .in_document(&projection.permit, |scope| {
                let handler = evaluate(scope, "() => {}")?;
                Ok(scope.retain(&handler))
            })
            .unwrap()
    };

    for public in [false, true] {
        let (mut projection, _) =
            native_projection_with_registry_route(cancel.clone(), public, false);
        let (request, ()) = projection.invoke_root(&foreign_handler, SOURCE, &cancel, |outcome| {
            let DocumentScriptGuestResult::Refused(Error::HostFailure(failure)) = outcome else {
                panic!("the unavailable Comment realm must refuse first: {outcome:?}");
            };
            assert!(matches!(
                failure.cause::<DocumentScriptNativeError>(),
                Some(DocumentScriptNativeError::CreationGlobalUnavailable)
            ));
        });
        assert_eq!(
            projection.parent(),
            Some(projection.permit.document().node())
        );
        let mut session = projection.host.stop().into_session().unwrap();
        assert_eq!(
            session.dom().pending_source_edit().unwrap().operation(),
            request.operation()
        );
    }
}

struct AttachedCancel {
    host: DocumentScriptHost,
    dialog: NodeId,
    observations: Rc<RefCell<Vec<NativeDomEvent>>>,
}

fn hosted_element(cancel: &AtomicBool, name: &str) -> (DomSession, DomHostedTopLevel, NodeId) {
    let details_intentions = if name == "details" { 4 } else { 0 };
    let (mut session, target) = hosted_session_with_details(cancel, 8, details_intentions);
    let element = insert_hosted_element(&mut session, target, name, cancel);
    (session, target, element)
}

fn insert_hosted_element(
    session: &mut DomSession,
    target: DomHostedTopLevel,
    name: &str,
    cancel: &AtomicBool,
) -> NodeId {
    let document = target.document();
    let html = session
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = session.snapshot().node(html).unwrap().children()[1];
    let mut create = DomBatch::new();
    let element = create.create_element_ns(document, Namespace::html(), name);
    create.insert_before(&element, body, None);
    session
        .dom()
        .apply(create, cancel)
        .unwrap()
        .created(&element)
        .unwrap()
        .unwrap()
}

fn attached_cancel_host(cancel: Arc<AtomicBool>) -> AttachedCancel {
    attached_cancel_host_with_reports(cancel, 4)
}

fn attached_cancel_host_with_reports(cancel: Arc<AtomicBool>, reports: usize) -> AttachedCancel {
    let (mut session, target, dialog) = hosted_element(&cancel, "dialog");
    let document = target.document();
    let registry_limits = DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    };
    let mut registry = DomEventDispatcher::new(registry_limits).unwrap();
    let observations = Rc::new(RefCell::new(Vec::new()));
    let native_observed = observations.clone();
    registry
        .add_listener(
            &mut session.dom(),
            document.node(),
            DomEventType::CANCEL,
            DomEventCallback::new(move |_, _, _, event| {
                assert_eq!(event.event_type(), DomEventType::CANCEL);
                assert_eq!(event.target().unwrap().node(), Some(dialog));
                assert!(event.cancelable());
                assert!(event.toggle_event().is_none());
                native_observed.borrow_mut().push(event.clone());
                Ok(())
            }),
            DomEventListenerOptions {
                capture: true,
                ..Default::default()
            },
        )
        .unwrap();
    let mut registry = Some(registry);
    let mut host = super::guest::tests::attach_public_nodes_with_registry_and_reports(
        session,
        target,
        cancel.clone(),
        1_000,
        20_000,
        &mut registry,
        reports,
    );
    assert!(registry.is_none());
    let mut competing = DomEventDispatcher::<()>::new(registry_limits).unwrap();
    assert!(matches!(
        host.with_native(|dom, _, _| competing.bind(dom, &cancel))
            .unwrap(),
        Err(DomEventDispatchError::Dom(
            DomError::EventRegistryAlreadyBound
        ))
    ));
    AttachedCancel {
        host,
        dialog,
        observations,
    }
}

fn attached_registry_delivers_the_original_native_cancel_event() {
    let cancel = Arc::new(AtomicBool::new(false));
    let AttachedCancel {
        mut host,
        dialog,
        observations,
    } = attached_cancel_host(cancel.clone());
    let permit = host.permit().unwrap();
    super::guest::tests::expose_public_node_for_source_test(&mut host, &permit, dialog, "dialog");
    host.in_document(&permit, |scope| {
        evaluate(
            scope,
            r#"
            globalThis.sentinel = {};
            globalThis.cancelChecks = false;
            const cancelAttr = document.createAttribute('data-cancel');
            let removedCalls = 0;
            const removed = () => { removedCalls++; };
            dialog.addEventListener("cancel", function (event) {
                globalThis.originalCancel = event;
                throw sentinel;
            }, {once: true});
            dialog.addEventListener("cancel", function (event) {
                const attachedDocument = document;
                cancelChecks = event.target.nodeName === "DIALOG"
                    && event.target.localName === "dialog"
                    && event.target.ownerDocument === attachedDocument
                    && event.currentTarget.getRootNode() === attachedDocument
                    && this === dialog && event === originalCancel
                    && event.target === dialog && event.currentTarget === dialog
                    && event instanceof Event && event.isTrusted
                    && event.cancelable && !event.bubbles && !event.composed
                    && globalThis.event === event && readChildCount() === 1;
                const path = event.composedPath();
                cancelChecks &&= path[0] === dialog
                    && path[1] instanceof HTMLBodyElement && path.includes(attachedDocument);
                cancelChecks &&= dialog.setAttributeNode(cancelAttr) === null
                    && dialog.getAttributeNode('data-cancel') === cancelAttr
                    && dialog.removeAttributeNode(cancelAttr) === cancelAttr;
                dialog.textContent = 'native event\ud800';
                cancelChecks &&= dialog.textContent === 'native event\ud800';
                event.currentTarget.removeEventListener("cancel", removed);
                event.preventDefault();
            }, {once: true});
            dialog.addEventListener("cancel", removed);
        "#,
        )
        .map(drop)
    })
    .unwrap();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let request = start_attached_cancel(&mut host, dialog, &mut services, &cancel);
    let mut bounded = EVENT;
    bounded.dispatch.callbacks = 2;
    assert!(matches!(
        host.service_attached_source_event(&permit, &request, bounded)
            .unwrap(),
        DomSourceEventServiceProgress::Retained {
            dispatch: crate::DomEventDispatchOutcome::Retained(crate::DomEventResource::Callbacks)
        }
    ));
    assert_eq!(observations.borrow().len(), 1);
    assert!(
        host.snapshot()
            .unwrap()
            .node(dialog)
            .unwrap()
            .attribute(&"open".into())
            .is_some()
    );
    let progress = host
        .service_attached_source_event(&permit, &request, EVENT)
        .unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } = progress else {
        panic!("attached guest delivery must accept the original receipt: {progress:?}");
    };
    assert_eq!(observations.borrow().len(), 1);
    assert_eq!(receipt.callbacks(), 3);
    assert_eq!(receipt.failures().len(), 1);
    assert!(receipt.default_prevented());
    host.with_native(|dom, _, mut source| {
        assert!(matches!(
            source.resume(dom, request.operation(), SOURCE, &mut services, &cancel).unwrap(),
            DomSourceDriveProgress::Complete(done) if done.operation() == request.operation()
        ));
        assert!(dom.pending_source_edit().is_none());
    })
    .unwrap();
    assert!(
        host.snapshot()
            .unwrap()
            .node(dialog)
            .unwrap()
            .attribute(&"open".into())
            .is_some()
    );
    let reported = host
        .take_guest_listener_exception(&permit)
        .unwrap()
        .unwrap();
    host.in_document(&permit, |scope| {
        let exact = scope.import(reported.exception.value())?;
        let sentinel = evaluate(scope, "sentinel")?;
        assert!(scope.strict_equals(&exact, &sentinel)?);
        let actual = evaluate(scope, "cancelChecks && removedCalls === 0 && originalCancel.currentTarget === null && originalCancel.defaultPrevented")?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        Ok(())
    }).unwrap();
}

fn attached_registry_keeps_preparation_refusal_on_retry() {
    let cancel = Arc::new(AtomicBool::new(false));
    let AttachedCancel {
        mut host,
        dialog,
        observations,
    } = attached_cancel_host(cancel.clone());
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        evaluate(scope, "document.addEventListener('cancel', () => { throw 'unexpected'; }, {capture: true, once: true})").map(drop)
    }).unwrap();
    super::guest::tests::refuse_next_source_node_validation(&host);
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let request = start_attached_cancel(&mut host, dialog, &mut services, &cancel);
    let first = host
        .service_attached_source_event(&permit, &request, EVENT)
        .unwrap_err();
    let retry = host
        .service_attached_source_event(&permit, &request, EVENT)
        .unwrap_err();
    match (&first, &retry) {
        (
            DocumentScriptEventError::Access(DocumentScriptError::Script(Error::HostFailure(
                first,
            ))),
            DocumentScriptEventError::Access(DocumentScriptError::Script(Error::HostFailure(
                retry,
            ))),
        ) => {
            let first = first.cause::<DocumentGuestEventError>().unwrap();
            assert!(matches!(
                first,
                DocumentGuestEventError::Document(DocumentScriptError::Script(
                    Error::BackendUnavailable
                ))
            ));
            assert!(std::ptr::eq(
                first,
                retry.cause::<DocumentGuestEventError>().unwrap()
            ));
        }
        (
            DocumentScriptEventError::Access(DocumentScriptError::Script(
                Error::BackendUnavailable,
            )),
            DocumentScriptEventError::Access(DocumentScriptError::Script(
                Error::BackendUnavailable,
            )),
        ) => (),
        _ => panic!("preparation refusal changed on retry: first {first:?}, retry {retry:?}"),
    }
    assert_eq!(observations.borrow().len(), 1);
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_none()
    );
    assert!(host.native.attached_event_request().is_none());
}

fn attached_registry_preserves_the_original_terminal_refusal() {
    let cancel = Arc::new(AtomicBool::new(false));
    let AttachedCancel {
        mut host,
        dialog,
        observations,
    } = attached_cancel_host(cancel.clone());
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        evaluate(
            scope,
            r#"
            const attached = document;
            let first;
            attached.addEventListener("cancel", event => {
                first = event;
                if (!(event.target instanceof HTMLDialogElement)
                    || event.currentTarget !== attached || readChildCount() !== 1) return;
                throw { original: true };
            }, {capture: true, once: true});
            attached.addEventListener("cancel", {
                handleEvent(event) {
                    if (event === first && event === globalThis.event) detachIf();
                }
            }, {capture: true, once: true});
            attached.addEventListener("cancel", () => { throw { replayed: true }; },
                {capture: true, once: true});
        "#,
        )
        .map(drop)
    })
    .unwrap();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let request = start_attached_cancel(&mut host, dialog, &mut services, &cancel);
    let window_event = host.engine.as_ref().unwrap().window_event.clone();
    let Err(DocumentScriptEventError::Access(DocumentScriptError::Script(Error::HostFailure(
        first,
    )))) = host.service_attached_source_event(&permit, &request, EVENT)
    else {
        panic!("the selected guest listener must return the actual host refusal");
    };
    let cause = first.cause::<DocumentScriptNativeError>().unwrap();
    assert!(matches!(cause, DocumentScriptNativeError::SourceAccess));
    assert!(window_event.is_idle());
    assert!(host.native.attached_event_request().is_none());
    {
        let delivered = observations.borrow();
        let [event] = delivered.as_slice() else {
            panic!("the native callback must have observed the original event exactly once");
        };
        assert!(event.current_target().is_none());
        assert_eq!(event.composed_path().count(), 0);
    }
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_some()
    );
    let Err(DocumentScriptEventError::Access(DocumentScriptError::Script(Error::HostFailure(
        again,
    )))) = host.service_attached_source_event(&permit, &request, EVENT)
    else {
        panic!("retry must return the first refusal without reentering any listener");
    };
    assert!(std::ptr::eq(
        cause,
        again.cause::<DocumentScriptNativeError>().unwrap()
    ));
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_none()
    );
    let mut session = host.stop().into_session().unwrap();
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        request.operation()
    );
    assert!(
        session
            .snapshot()
            .node(dialog)
            .unwrap()
            .attribute(&"open".into())
            .is_some()
    );
}

fn attached_beforetoggle_uses_the_original_toggle_event() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target, dialog) = hosted_element(&cancel, "dialog");
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    registry.bind(&mut session.dom(), &cancel).unwrap();
    let mut registry = Some(registry);
    let mut host = super::guest::tests::attach_public_nodes_with_registry(
        session,
        target,
        cancel.clone(),
        1_000,
        20_000,
        &mut registry,
    );
    let permit = host.permit().unwrap();
    super::guest::tests::expose_public_node_for_source_test(&mut host, &permit, dialog, "dialog");
    host.in_document(&permit, |scope| {
        evaluate(
            scope,
            r#"
            const ordinary = new Event("ordinary");
            globalThis.toggleSentinel = {};
            globalThis.toggleChecks = false;
            dialog.addEventListener("beforetoggle", event => {
                globalThis.originalToggle = event;
                throw toggleSentinel;
            }, {once: true});
            dialog.addEventListener("beforetoggle", function (event) {
                const oldState = Object.getOwnPropertyDescriptor(ToggleEvent.prototype, "oldState");
                const newState = Object.getOwnPropertyDescriptor(ToggleEvent.prototype, "newState");
                const source = Object.getOwnPropertyDescriptor(ToggleEvent.prototype, "source");
                let rejectsOrdinary = false;
                try {
                    Reflect.apply(oldState.get, ordinary, []);
                } catch (error) {
                    rejectsOrdinary = error instanceof TypeError;
                }
                toggleChecks = event === originalToggle && event === globalThis.event
                    && this === dialog && event.target === dialog && event.currentTarget === dialog
                    && event.type === "beforetoggle" && event.isTrusted && event.cancelable
                    && event instanceof ToggleEvent && event instanceof Event
                    && Object.getPrototypeOf(event) === ToggleEvent.prototype
                    && Object.getPrototypeOf(ToggleEvent.prototype) === Event.prototype
                    && Object.getPrototypeOf(ToggleEvent) === Event
                    && ToggleEvent.length === 1
                    && Object.prototype.toString.call(event) === "[object ToggleEvent]"
                    && event.oldState === "closed" && event.newState === "open"
                    && event.source === null && event.oldState === "closed" && event.source === null
                    && [oldState, newState, source].every(attribute => attribute.enumerable
                        && attribute.configurable && attribute.set === undefined)
                    && rejectsOrdinary;
                event.preventDefault();
            }, {once: true});
        "#,
        )
        .map(drop)
    })
    .unwrap();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let request = host
        .with_native(|dom, _, mut source| {
            let progress = source
                .apply(
                    dom,
                    DomSourceEdit::show_dialog(dialog),
                    SOURCE,
                    &mut services,
                    &cancel,
                )
                .unwrap();
            let DomSourceDriveProgress::Pending {
                request: DomSourceRequest::Event(request),
                reason: DomSourceUnavailable::Event,
            } = progress
            else {
                panic!("native Dialog Show must reach beforetoggle: {progress:?}");
            };
            assert_eq!(
                request.kind(),
                surgeist_dom::DomSourceEventKind::BeforeToggle
            );
            let toggle = request.toggle_event().unwrap();
            assert_eq!(
                toggle.old_state(),
                surgeist_html::elements::dialog::HtmlDialogState::Closed
            );
            assert_eq!(
                toggle.new_state(),
                surgeist_html::elements::dialog::HtmlDialogState::Open
            );
            request
        })
        .unwrap();
    let result = host
        .service_attached_source_event(&permit, &request, EVENT)
        .unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } = result else {
        panic!(
            "native beforetoggle must reach the exact attached ToggleEvent listeners: {result:?}"
        );
    };
    assert_eq!(receipt.callbacks(), 2);
    assert_eq!(receipt.failures().len(), 1);
    assert!(receipt.default_prevented());
    host.with_native(|dom, _, mut source| {
        assert!(matches!(
            source.resume(dom, request.operation(), SOURCE, &mut services, &cancel).unwrap(),
            DomSourceDriveProgress::Complete(done) if done.operation() == request.operation()
        ));
    })
    .unwrap();
    assert!(
        host.snapshot()
            .unwrap()
            .node(dialog)
            .unwrap()
            .attribute(&"open".into())
            .is_none()
    );
    let reported = host
        .take_guest_listener_exception(&permit)
        .unwrap()
        .unwrap();
    assert_eq!(reported.realm, permit.realm);
    host.in_document(&permit, |scope| {
        let exact = scope.import(reported.exception.value())?;
        let sentinel = evaluate(scope, "toggleSentinel")?;
        assert!(scope.strict_equals(&exact, &sentinel)?);
        let actual = evaluate(scope, "toggleChecks && originalToggle.currentTarget === null && originalToggle.defaultPrevented && originalToggle.source === null")?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        Ok(())
    }).unwrap();
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_none()
    );
    assert!(host.stop().into_session().is_ok());
}

fn start_attached_cancel(
    host: &mut DocumentScriptHost,
    dialog: NodeId,
    services: &mut DomSourceServices<'_>,
    cancel: &AtomicBool,
) -> DomSourceEventRequest {
    host.with_native(|dom, _, mut source| {
        assert!(matches!(
            source
                .apply(
                    dom,
                    DomSourceEdit::set_native_attribute(dialog, "open", ""),
                    SOURCE,
                    services,
                    cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let progress = source
            .apply(
                dom,
                DomSourceEdit::request_close_dialog(dialog, Some("uncommitted".into())),
                SOURCE,
                services,
                cancel,
            )
            .unwrap();
        let DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(request),
            reason: DomSourceUnavailable::Event,
        } = progress
        else {
            panic!("actual requestClose must reach native Cancel: {progress:?}");
        };
        assert_eq!(request.kind(), surgeist_dom::DomSourceEventKind::Cancel);
        request
    })
    .unwrap()
}

fn evaluate<'scope>(scope: &mut Scope<'scope>, text: &str) -> Result<Value<'scope>, Error> {
    scope.evaluate(&text.encode_utf16().collect::<Vec<_>>())
}

struct DropReceipt(Rc<Cell<usize>>);

impl Drop for DropReceipt {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn collect(projection: &mut Projection, marker: &Cell<usize>, expected: usize, phase: &str) {
    let witness = Rc::new(Cell::new(0));
    let receipt = DropReceipt(witness.clone());
    projection
        .host
        .in_document(&projection.permit, |scope| {
            let transient = Callback::new(move |_, _| {
                let _receipt = &receipt;
                Ok(Completion::Undefined)
            });
            scope.function(&transient)?;
            Ok(())
        })
        .unwrap();

    let engine = projection.host.engine.as_mut().unwrap();
    let mut execution = engine.process.execution();
    // Leave the last guest job before applying pressure. The transient
    // callback must actually collect before a surviving marker is evidence.
    for _ in 0..8 {
        execution.checkpoint().unwrap();
        execution.memory_pressure();
        if witness.get() == 1 && (expected == 0 || marker.get() == expected) {
            break;
        }
    }
    assert_eq!(
        witness.get(),
        1,
        "{phase}: transient capture was not collected"
    );
    assert_eq!(marker.get(), expected, "{phase}: projected capture drops");
}

fn install_origin_marker(
    projection: &mut Projection,
    origin: DomHostedTopLevel,
) -> Rc<Cell<usize>> {
    let marker_drops = Rc::new(Cell::new(0));
    let receipt = DropReceipt(marker_drops.clone());
    let engine = projection.host.engine.as_mut().unwrap();
    let origin_realm = engine
        .creations
        .find(origin.global(), || Ok::<_, Error>(()))
        .unwrap()
        .unwrap();
    let marker = engine
        .process
        .execution()
        .in_realm(origin_realm, |scope| {
            evaluate(scope, "globalThis.custodyMarker = 97")?;
            let identity = scope.realm_authority()?.identity();
            let callback = Callback::new(move |scope, _| {
                let _receipt = &receipt;
                assert_eq!(scope.realm_authority()?.identity(), identity);
                Ok(Completion::Return(evaluate(scope, "custodyMarker")?))
            });
            let marker = scope.function(&callback)?;
            Ok(scope.retain(&marker))
        })
        .unwrap();
    projection
        .host
        .in_document(&projection.permit, |scope| {
            evaluate(scope, "globalThis.custodyMarker = 19")?;
            let global = scope.global()?;
            let marker = scope.import(&marker)?;
            scope.set(
                &global,
                PropertyKey::String(&"projectionMarker".encode_utf16().collect::<Vec<_>>()),
                &marker,
            )
        })
        .unwrap();
    marker_drops
}

fn projected_comment_custody_outlives_collection_and_the_origin_realm_token() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut projection, origin) = native_projection(cancel.clone());
    let marker = install_origin_marker(&mut projection, origin);
    let document = projection.permit.document();
    projection
        .host
        .with_native(|dom, mut hosted, _| {
            let task = hosted
                .destroy_top_level(dom, origin.navigable(), &cancel)
                .unwrap();
            assert!((0..256).any(|_| matches!(
                hosted.run_next(dom, task.traversable(), &cancel).unwrap(),
                DomHostedTaskProgress::Complete { operation } if operation == task.operation()
            )));
            hosted.destroy_host(dom, origin.host(), &cancel).unwrap();
            let snapshot = dom.snapshot();
            assert!(
                !snapshot
                    .document(origin.document())
                    .unwrap()
                    .is_fully_active()
                    .unwrap()
            );
            assert!(
                snapshot
                    .document(document)
                    .unwrap()
                    .is_fully_active()
                    .unwrap()
            );
            assert_eq!(
                snapshot.relevant_global(projection.comment).unwrap(),
                Some(origin.global())
            );
        })
        .unwrap();

    projection.expose(
        "(comment) => {
            globalThis.projected = new WeakRef(comment);
            comment.marker = projectionMarker;
            if (!delete globalThis.projectionMarker || comment.marker() !== 97)
                throw Error('origin marker');
        }",
        &cancel,
    );
    assert_eq!(projection.parent(), Some(document.node()));
    collect(&mut projection, &marker, 0, "connected");

    projection.expose(
        "(comment) => {
            if (projected.deref() !== comment || comment.marker() !== 97)
                throw Error('connected expando');
            detachIf(1, comment);
        }",
        &cancel,
    );
    assert_eq!(projection.parent(), None);
    collect(&mut projection, &marker, 0, "detached");

    projection
        .host
        .with_native(|dom, _, _| {
            let mut insert = DomBatch::new();
            insert.insert_before(projection.comment, document.node(), None);
            dom.apply(insert, &cancel).unwrap();
        })
        .unwrap();
    projection.expose(
        "(comment) => {
            if (projected.deref() !== comment || comment.marker() !== 97)
                throw Error('detached expando');
            detachIf(1, comment);
        }",
        &cancel,
    );
    assert_eq!(projection.parent(), None);

    // Future first projection needs the production registration. After the
    // last exposure this private harness can release its sole Rust Realm
    // token while leaving the already admitted native pin in B's Process.
    let engine = projection.host.engine.as_mut().unwrap();
    let CreationRealm {
        next,
        global,
        realm,
        _backing: creation_backing,
    } = *engine.creations.0.take().unwrap();
    assert!(next.is_none());
    assert_eq!(global, origin.global());
    drop(realm);
    collect(&mut projection, &marker, 0, "origin token released");

    let engine = projection.host.engine.as_mut().unwrap();
    let key = engine
        .comments
        .find(projection.comment, || Ok(()))
        .unwrap()
        .unwrap();
    let holder = engine.comments.object(key).unwrap();
    engine
        .process
        .execution()
        .in_realm(&engine.realm, |scope| {
            let wrapper = scope.open_managed_object(holder)?;
            let weak = evaluate(scope, "projected.deref()")?;
            assert!(scope.strict_equals(&wrapper, &weak)?);
            let callback = scope.get(
                &wrapper,
                PropertyKey::String(&"marker".encode_utf16().collect::<Vec<_>>()),
            )?;
            let value = scope.call(&callback, &wrapper, &[])?;
            let expected = scope.primitive(Primitive::Number(97.0))?;
            assert!(scope.strict_equals(&value, &expected)?);
            Ok(())
        })
        .unwrap();

    projection
        .host
        .with_native(|dom, _, _| {
            let mut destroy = DomBatch::new();
            destroy.destroy(projection.comment);
            dom.apply(destroy, &cancel).unwrap();
            assert!(matches!(
                dom.snapshot().node(projection.comment),
                Err(DomError::StaleNode)
            ));
        })
        .unwrap();
    let engine = projection.host.engine.as_ref().unwrap();
    assert!(engine.comments.object(key).is_none());
    collect(&mut projection, &marker, 1, "native retirement");
    drop(creation_backing);
    projection
        .host
        .in_document(&projection.permit, |scope| {
            let count = evaluate(scope, "readChildCount()")?;
            let expected = scope.primitive(Primitive::Number(1.0))?;
            assert!(scope.strict_equals(&count, &expected)?);
            let wrapper = evaluate(scope, "projected.deref()")?;
            let undefined = scope.primitive(Primitive::Undefined)?;
            assert!(scope.strict_equals(&wrapper, &undefined)?);
            Ok(())
        })
        .unwrap();
}

fn install_brand_work_control(
    projection: &mut Projection,
    work: u64,
    remaining: u64,
) -> Rc<Cell<(usize, usize)>> {
    let native = Rc::downgrade(&projection.host.native);
    let hits = Rc::new(Cell::new((0, 0)));
    let witness = hits.clone();
    projection
        .host
        .in_document(&projection.permit, |scope| {
            let burn = Callback::new(move |_, _| {
                let native = native.upgrade().unwrap();
                let active = native.active_source().unwrap();
                let mut session = native.session.borrow_mut();
                let dom = session.dom();
                let status = dom.pending_source_edit().unwrap();
                assert_eq!(status.operation(), active.request().operation());
                assert!(status.spent_work() < work - remaining);
                for _ in status.spent_work()..work - remaining {
                    dom.raw()
                        .source_event_step(active.request(), &native.cancellation)
                        .unwrap();
                }
                Ok(Completion::Undefined)
            });
            let mark = Callback::new(move |scope, args| {
                let (diagnostic, coercion) = witness.get();
                witness.set(if scope.boolean(args.get(0).unwrap())? {
                    (diagnostic + 1, coercion)
                } else {
                    (diagnostic, coercion + 1)
                });
                Ok(Completion::Undefined)
            });
            let global = scope.global()?;
            for (name, callback) in [("limitBrandWork", burn), ("recordBrandResult", mark)] {
                let function = scope.function(&callback)?;
                scope.set(
                    &global,
                    PropertyKey::String(&name.encode_utf16().collect::<Vec<_>>()),
                    &function,
                )?;
            }
            Ok(())
        })
        .unwrap();
    hits
}

fn rejected_comment_requires_each_diagnostic_utf16_work_unit() {
    const LIMIT: DomSourceLimits =
        DomSourceLimits::new(32_000_000, 1_000_000, 8_192, 128, 4096, 4096, 8);
    const MESSAGE: &str = "Delegated Comment required";
    const HANDLER: &str = "(comment) => {
        const forged = Object.create(Object.getPrototypeOf(comment));
        limitBrandWork();
        try {
            detachIf({ valueOf() { recordBrandResult(false); return 1; } }, forged);
        } catch (error) {
            if (!(error instanceof TypeError) || error.message !== 'Delegated Comment required')
                throw error;
            recordBrandResult(true);
            return;
        }
        throw Error('forged Comment accepted');
    }";
    // The existing entry spends one step. The diagnostic's authored UTF-16
    // code units each need a step before the V8 TypeError can be allocated.
    let exact = 1 + u64::try_from(MESSAGE.encode_utf16().count()).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    for remaining in [exact, exact - 1] {
        let (mut projection, _) = native_projection(cancel.clone());
        let hits = install_brand_work_control(&mut projection, LIMIT.work(), remaining);
        let (request, failure) = projection.invoke(HANDLER, LIMIT, &cancel, |outcome| {
            if remaining == exact {
                assert!(
                    matches!(outcome, DocumentScriptGuestResult::Returned),
                    "{outcome:?}"
                );
                return None;
            }
            let DocumentScriptGuestResult::Refused(Error::HostFailure(failure)) = outcome else {
                panic!("one diagnostic UTF-16 unit short must refuse before throwing: {outcome:?}");
            };
            let cause = failure.cause::<DocumentScriptNativeError>().unwrap();
            assert!(
                matches!(
                    cause,
                    DocumentScriptNativeError::Dom(DomError::SourceWorkLimit { limit })
                        if *limit == LIMIT.work()
                ),
                "{cause:?}"
            );
            Some(failure.clone())
        });
        assert_eq!(hits.get(), (usize::from(remaining == exact), 0));
        assert!(projection.host.native.active_source().is_none());
        let document = projection.permit.document();
        assert_eq!(projection.parent(), Some(document.node()));
        projection
            .host
            .with_native(|dom, _, _| {
                assert_eq!(
                    dom.pending_source_edit().unwrap().operation(),
                    request.operation()
                );
                dom.raw_mut()
                    .raise_source_event_limits(&request, SOURCE, &cancel)
                    .unwrap();
            })
            .unwrap();

        if let Some(failure) = failure {
            let Err(DocumentScriptError::Script(Error::HostFailure(checkpoint))) =
                projection.host.checkpoint()
            else {
                panic!("the original sticky refusal must survive cleanup");
            };
            assert!(std::ptr::eq(
                failure.cause::<DocumentScriptNativeError>().unwrap(),
                checkpoint.cause::<DocumentScriptNativeError>().unwrap(),
            ));
            assert_eq!(projection.host.engine.as_ref().unwrap().comments.len(), 0);
            projection
                .host
                .with_native(|dom, _, source| {
                    assert!(matches!(
                        source.service_event(
                            dom,
                            &request,
                            &mut projection.dispatcher,
                            &mut (),
                            EVENT,
                            &cancel
                        ),
                        Err(DomSourceRuntimeError::SourceEvent(
                            DomSourceEventRuntimeError::Dispatch(
                                DomEventDispatchError::HostRefused(DomEventHostError { code: 77 })
                            )
                        ))
                    ));
                    assert_eq!(
                        dom.pending_source_edit().unwrap().operation(),
                        request.operation()
                    );
                    assert!(
                        dom.snapshot()
                            .node(projection.dialog)
                            .unwrap()
                            .attribute(&"open".into())
                            .is_some()
                    );
                })
                .unwrap();
            drop(failure);
            drop(checkpoint);
            let mut session = projection.host.stop().into_session().unwrap();
            assert_eq!(
                session.snapshot().document(document).unwrap().scripting(),
                HtmlScriptingStatus::Disabled
            );
            assert_eq!(
                session.dom().pending_source_edit().unwrap().operation(),
                request.operation()
            );
            assert_eq!(Rc::strong_count(&hits), 1);
            assert!(matches!(
                session.try_shutdown(),
                DomShutdownOutcome::Pending { .. }
            ));
        } else {
            projection.finish(&request, &cancel);
            assert_eq!(projection.parent(), Some(document.node()));
        }
    }
}
