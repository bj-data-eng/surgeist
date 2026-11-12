//! Trusted Types factory on the registered Window.

mod methods;

use super::super::reports::{ViolationBacking, ViolationOrigin, ViolationQueue, violation_error};
use super::*;
use crate::trusted_types_csp::{PolicyCreationError, PolicyFactory};
use surgeist_bindings_js::{
    NativeObject, TrustedPolicyCallback, TrustedTypeKind, ValueKind,
    conversion::{Converted, types},
};
use surgeist_dom::{DocumentString, DomDocumentCspState};

struct Factory {
    state: RefCell<PolicyFactory<RetainedPolicy, GuestBacking>>,
    policy_prototype: RetainedCallbackObject,
}

struct RetainedPolicy {
    object: RetainedCallbackObject,
    _backing: GuestBacking,
}

struct PolicyObject {
    name: DocumentString,
    callbacks: [Option<TrustedPolicyCallback>; 4],
}

impl PolicyObject {
    fn callback(&self, kind: TrustedTypeKind) -> Option<&TrustedPolicyCallback> {
        let [html, _, script, script_url] = &self.callbacks;
        match kind {
            TrustedTypeKind::Html => html,
            TrustedTypeKind::Script => script,
            TrustedTypeKind::ScriptUrl => script_url,
        }
        .as_ref()
    }
}

pub(super) struct DefaultPolicy(Rc<NativeObject<PolicyObject>>);

impl DefaultPolicy {
    pub(super) fn callback(&self, kind: TrustedTypeKind) -> Option<&TrustedPolicyCallback> {
        self.0.payload().callback(kind)
    }
}

pub(super) fn current_default_policy(
    scope: &mut Scope<'_>,
    events: &GuestEvents,
) -> Result<Option<DefaultPolicy>, DocumentGuestEventError> {
    let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
    let factory = scope.open_callback_object(&public.trusted_types)?;
    let factory = scope
        .native_object::<Factory>(&factory)?
        .ok_or(Error::ExpectedNativeObject)?;
    let policy = factory
        .payload()
        .state
        .try_borrow()
        .map_err(|_| DocumentScriptNativeError::Busy)?
        .default_policy()
        .cloned();
    let Some(policy) = policy else {
        return Ok(None);
    };
    let object = scope.open_callback_object(&policy.object)?;
    let object = scope
        .native_object::<PolicyObject>(&object)?
        .ok_or(Error::ExpectedNativeObject)?;
    Ok(Some(DefaultPolicy(object)))
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    window: &Value<'scope>,
    capture: CaptureFunding<'_>,
    fund: &dyn Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
    let policy_prototype = scope.object(ObjectPrototype::Default)?;
    let name_capture = capture(size_of::<Capture>())?;
    let name = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| policy_name(&name_capture, scope, arguments)),
        FunctionMetadata {
            name: &ascii(b"get name"),
            length: 0,
        },
    )?;
    define_accessor(scope, &policy_prototype, &ascii(b"name"), &name, None, true)?;
    methods::install(scope, &policy_prototype, capture)?;
    let observed = scope.callback_object(&policy_prototype)?;
    let policy_prototype = scope.retain_callback_object(observed, fund)?;

    let prototype = scope.object(ObjectPrototype::Default)?;
    let create_capture = capture(size_of::<Capture>())?;
    let create = scope.bounded_function(
        &Callback::new(move |scope, arguments| create_policy(&create_capture, scope, arguments)),
        FunctionMetadata {
            name: &ascii(b"createPolicy"),
            length: 1,
        },
        2,
    )?;
    define_data(
        scope,
        &prototype,
        PropertyKey::String(&ascii(b"createPolicy")),
        &create,
        true,
        true,
        true,
    )?;
    let default_capture = capture(size_of::<Capture>())?;
    let default = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| default_policy(&default_capture, scope, arguments)),
        FunctionMetadata {
            name: &ascii(b"get defaultPolicy"),
            length: 0,
        },
    )?;
    define_accessor(
        scope,
        &prototype,
        &ascii(b"defaultPolicy"),
        &default,
        None,
        true,
    )?;
    let factory = scope.project_native_object(
        ObjectPrototype::Object(&prototype),
        Factory {
            state: RefCell::new(PolicyFactory::default()),
            policy_prototype,
        },
        fund,
    )?;
    let observed = scope.callback_object(&factory)?;
    let factory = scope.retain_callback_object(observed, fund)?;
    let window_capture = capture(size_of::<Capture>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            let global = scope.global()?;
            if !scope.strict_equals(arguments.receiver(), &global)? {
                return type_error(scope, &ascii(b"Window required"));
            }
            let (owner, events) = window_capture.active(scope)?;
            if let Err(error) = qualify(&window_capture, &owner, &events) {
                return refuse(scope, &events.refusal, error);
            }
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            Ok(Completion::Return(
                scope.open_callback_object(&public.trusted_types)?,
            ))
        }),
        FunctionMetadata {
            name: &ascii(b"get trustedTypes"),
            length: 0,
        },
    )?;
    define_accessor(scope, window, &ascii(b"trustedTypes"), &getter, None, true)?;
    Ok(factory)
}

fn qualify(
    capture: &Capture,
    owner: &NativeOwner,
    events: &GuestEvents,
) -> Result<(), DocumentGuestEventError> {
    capture
        .callbacks
        .with_read(owner, events, |read, _| read.step())
}

fn type_error<'scope>(
    scope: &mut Scope<'scope>,
    message: &[u16],
) -> Result<Completion<'scope>, Error> {
    Ok(Completion::Throw(scope.type_error(message)?))
}

fn policy_name<'scope>(
    capture: &Capture,
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let (owner, events) = capture.active(scope)?;
    let Some(policy) = scope.native_object::<PolicyObject>(arguments.receiver())? else {
        return type_error(scope, &ascii(b"TrustedTypePolicy required"));
    };
    if let Err(error) = qualify(capture, &owner, &events) {
        return refuse(scope, &events.refusal, error);
    }
    let value = capture.callbacks.conversion.run(scope, |conversion| {
        conversion.copy_to_js(policy.payload().name.code_units())
    })?;
    Ok(Completion::Return(value))
}

fn default_policy<'scope>(
    capture: &Capture,
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let (owner, events) = capture.active(scope)?;
    let Some(factory) = scope.native_object::<Factory>(arguments.receiver())? else {
        return type_error(scope, &ascii(b"TrustedTypePolicyFactory required"));
    };
    if let Err(error) = qualify(capture, &owner, &events) {
        return refuse(scope, &events.refusal, error);
    }
    let result = factory
        .payload()
        .state
        .try_borrow()
        .map(|state| state.default_policy().cloned());
    let Ok(default) = result else {
        return refuse(
            scope,
            &events.refusal,
            DocumentScriptNativeError::Busy.into(),
        );
    };
    let value = match default {
        Some(policy) => scope.open_callback_object(&policy.object)?,
        None => scope.primitive(Primitive::Null)?,
    };
    Ok(Completion::Return(value))
}

fn create_policy<'scope>(
    capture: &Capture,
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let (owner, events) = capture.active(scope)?;
    let Some(factory) = scope.native_object::<Factory>(arguments.receiver())? else {
        return type_error(scope, &ascii(b"TrustedTypePolicyFactory required"));
    };
    let Some(name) = arguments.get(0) else {
        return type_error(scope, &ascii(b"Policy name required"));
    };
    let name = capture
        .callbacks
        .conversion
        .run(scope, |cx| cx.value(&types::DOM_STRING, name))?;
    let mut options = [const { None }; 4];
    if let Some(dictionary) = arguments.get(1) {
        match scope.kind(dictionary)? {
            ValueKind::Null | ValueKind::Undefined => {}
            ValueKind::Object => {
                for (slot, key) in options.iter_mut().zip([
                    &ascii(b"createHTML")[..],
                    &ascii(b"createParserOptions")[..],
                    &ascii(b"createScript")[..],
                    &ascii(b"createScriptURL")[..],
                ]) {
                    if let Err(error) = events.step() {
                        return refuse(scope, &events.refusal, error);
                    }
                    let value = scope.get(dictionary, PropertyKey::String(key))?;
                    if scope.kind(&value)? == ValueKind::Undefined {
                        continue;
                    }
                    if !scope.is_callable(&value)? {
                        return type_error(scope, &ascii(b"Policy callback must be callable"));
                    }
                    *slot = Some(value);
                }
            }
            _ => return type_error(scope, &ascii(b"Policy options must be a dictionary")),
        }
    }
    let result = publish(
        capture,
        &owner,
        &events,
        scope,
        factory.payload(),
        name,
        options,
    );
    match result {
        Ok(value) => Ok(value),
        Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => Err(error),
        Err(error) => refuse(scope, &events.refusal, error),
    }
}

fn publish<'scope>(
    capture: &Capture,
    owner: &NativeOwner,
    events: &GuestEvents,
    scope: &mut Scope<'scope>,
    factory: &Factory,
    name: Converted<surgeist_bindings_js::DomString>,
    options: [Option<Value<'scope>>; 4],
) -> Result<Completion<'scope>, DocumentGuestEventError> {
    let request = owner.attached_event_request();
    let (snapshot, processor, global) =
        capture
            .callbacks
            .with_qualified_session(owner, request.as_ref(), |session, global| {
                Ok((
                    session.snapshot(),
                    session.trusted_types_csp.clone(),
                    global,
                ))
            })?;
    let mut step = || {
        if let Some((request, registration)) = owner.active_attribute_registration()? {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session.dom().raw().source_attribute_callback_step(
                request,
                &registration,
                &owner.cancellation,
            )?;
        } else if let Some(request) = &request {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session
                .dom()
                .raw()
                .source_event_step(request, &owner.cancellation)?;
        }
        events.step()
    };
    step()?;
    let document = snapshot.document(capture.callbacks.permit.document)?;
    let origin = ViolationOrigin {
        global,
        document: document.id(),
    };
    let policies = match document.metadata().policy_container().csp() {
        DomDocumentCspState::ConstructorInitialEmpty => &[][..],
        DomDocumentCspState::Associated(policy) if policy.policy().profile().is_none() => &[],
        state @ DomDocumentCspState::Associated(_) => {
            let processor = processor
                .as_ref()
                .ok_or(DocumentScriptNativeError::TrustedTypesPolicyUnavailable)?;
            let processor = processor
                .processor
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let mut scratch = None;
            processor
                .document_policies_with(state, &mut |layout| {
                    drop(scratch.take());
                    layout
                        .charge_work(&mut crate::document_scripting::CspWork(|units| {
                            for _ in 0..units {
                                step()?;
                            }
                            Ok::<(), DocumentGuestEventError>(())
                        }))
                        .map_err(crate::document_scripting::csp_work_error)?;
                    scratch = Some(events.reserve(owner, layout.atomic_bytes())?);
                    Ok::<(), DocumentGuestEventError>(())
                })
                .map_err(|error| match error {
                    surgeist_html::security::csp::HtmlCspError::Interrupted(error) => error,
                    surgeist_html::security::csp::HtmlCspError::Protocol(_) => {
                        DocumentScriptNativeError::TrustedTypesPolicyUnavailable.into()
                    }
                })?
                .ok_or(DocumentScriptNativeError::TrustedTypesPolicyUnavailable)?
        }
        DomDocumentCspState::Unavailable => {
            return Err(DocumentScriptNativeError::TrustedTypesPolicyUnavailable.into());
        }
    };
    let units = name.get().as_code_units();
    let text_bytes = units
        .len()
        .checked_mul(size_of::<u16>())
        .and_then(|bytes| bytes.checked_add(size_of::<Vec<u16>>()))
        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
    let _text = events.reserve(owner, text_bytes)?;
    let copy_work = u64::try_from(units.len())
        .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
    events.charge_work(copy_work)?;
    let mut copy = Vec::new();
    copy.try_reserve_exact(units.len())
        .map_err(|_| Error::Allocation)?;
    let extra = copy
        .capacity()
        .saturating_sub(units.len())
        .checked_mul(size_of::<u16>())
        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
    let _extra = if extra == 0 {
        None
    } else {
        Some(events.reserve(owner, extra)?)
    };
    let text_bytes = text_bytes
        .checked_add(extra)
        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
    copy.extend_from_slice(units);
    let name = DocumentString::from_utf16(copy);
    let mut state = factory
        .state
        .try_borrow_mut()
        .map_err(|_| DocumentScriptNativeError::Busy)?;
    let mut attempt = state.begin(&name, policies, &mut step)?;
    let mut reports = owner
        .trusted_types_reports
        .try_borrow_mut()
        .map_err(|_| DocumentScriptNativeError::Busy)?;
    while let Some(report) = attempt
        .next_report(&mut step, |bytes| {
            ViolationQueue::admit_report(
                &mut reports,
                origin,
                bytes,
                |bytes, _| {
                    Ok(ViolationBacking::Guest {
                        _backing: events.reserve(owner, bytes)?,
                    })
                },
                |work| events.charge_work(work),
            )
        })
        .map_err(violation_error)?
    {
        reports
            .as_mut()
            .ok_or(DocumentScriptNativeError::Busy)?
            .push(report);
    }
    drop(reports);
    let admitted = match attempt.admit(|bytes| events.reserve(owner, bytes)) {
        Ok(admitted) => admitted,
        Err(PolicyCreationError::DefaultAlreadyExists) => {
            return Ok(type_error(scope, &ascii(b"Default policy already exists"))?);
        }
        Err(PolicyCreationError::Admission(error)) => return Err(error),
        Err(PolicyCreationError::RepresentationExhausted) => {
            return Err(DocumentScriptNativeError::RepresentationExhausted.into());
        }
        Err(PolicyCreationError::BlockedByCsp) => {
            return Ok(type_error(
                scope,
                &ascii(b"Trusted Types policy blocked by CSP"),
            )?);
        }
        Err(PolicyCreationError::PoliciesPending) => {
            return Err(DocumentScriptNativeError::TrustedTypesPolicyUnavailable.into());
        }
    };
    let mut retained = [const { None }; 4];
    for (slot, callback) in retained.iter_mut().zip(options) {
        if let Some(callback) = callback {
            *slot = scope
                .retain_trusted_policy_callback(&callback, |bytes| events.reserve(owner, bytes))?;
        }
    }
    let prototype = scope.open_callback_object(&factory.policy_prototype)?;
    let policy = scope.project_native_object(
        ObjectPrototype::Object(&prototype),
        PolicyObject {
            name: name.clone(),
            callbacks: retained,
        },
        |bytes| {
            events.reserve(
                owner,
                bytes
                    .checked_add(text_bytes)
                    .ok_or(DocumentScriptNativeError::RepresentationExhausted)?,
            )
        },
    )?;
    let backing = events.reserve(owner, size_of::<RetainedPolicy>())?;
    let observed = scope.callback_object(&policy)?;
    let object = scope.retain_callback_object(observed, |bytes| events.reserve(owner, bytes))?;
    let retained = Rc::new(RetainedPolicy {
        object,
        _backing: backing,
    });
    admitted.publish(&retained);
    Ok(Completion::Return(policy))
}
