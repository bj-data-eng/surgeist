//! Standard Element setters over the original native Source operation.

use super::{super::character_data::copy_native_data, *};
use crate::{
    DomSourceDriveError, DomSourceDriveProgress, DomSourceRuntimeError, DomSourceServices,
    trusted_types_csp::{AttributePolicies, AttributePolicyError},
};
use surgeist_bindings_js::{
    TrustedAttributeValue, TrustedTypeKind,
    conversion::{ArgumentValue, declarations},
    dom_exception::DomExceptionName,
};
use surgeist_dom::{
    DomSourceAttributeSink, DomSourceAttributeVerificationOutcome,
    DomSourceAttributeVerificationRequest, DomSourceEdit, DomSourceLease, DomSourceRequest,
    Namespace,
};

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    element: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (name, namespaced, length) in [
        (&ascii(b"setAttribute")[..], false, 2),
        (&ascii(b"setAttributeNS")[..], true, 3),
    ] {
        let handler = capture(size_of::<(Capture, bool)>())?;
        let function = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.write_attribute(scope, arguments, namespaced)
            }),
            FunctionMetadata { name, length },
        )?;
        define_data(
            scope,
            element,
            PropertyKey::String(name),
            &function,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

impl Capture {
    fn write_attribute<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        namespaced: bool,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::Element(element)) = receiver.and_then(|node| node.payload().id())
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Element required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let (namespace, name, value) = self
                .callbacks
                .conversion
                .run(scope, |cx| {
                    let (namespace, name, value) = if namespaced {
                        let namespace = declarations::element_set_attribute_ns_0_namespace();
                        (
                            Some(cx.argument(&namespace, arguments.get(namespace.index()))?),
                            declarations::element_set_attribute_ns_0_qualified_name(),
                            declarations::element_set_attribute_ns_0_value(),
                        )
                    } else {
                        (
                            None,
                            declarations::element_set_attribute_0_qualified_name(),
                            declarations::element_set_attribute_0_value(),
                        )
                    };
                    Ok((
                        namespace,
                        cx.argument(&name, arguments.get(name.index()))?,
                        cx.argument(&value, arguments.get(value.index()))?,
                    ))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let (ArgumentValue::Present(name), ArgumentValue::Present(value)) =
                (name.get(), value.get())
            else {
                return Err(Error::BackendUnavailable.into());
            };
            let (name, _name_backing) = copy_native_data(&owner, &events, name.as_code_units())?;
            let (text, _value_backing) = copy_native_data(&owner, &events, value.as_code_units())?;
            let uri = match namespace.as_ref().map(|namespace| namespace.get()) {
                Some(ArgumentValue::Present(Some(uri))) => {
                    Some(copy_native_data(&owner, &events, uri.as_code_units())?)
                }
                Some(ArgumentValue::Missing) => return Err(Error::BackendUnavailable.into()),
                _ => None,
            };
            let edit = if namespaced {
                let namespace = uri
                    .as_ref()
                    .map_or_else(Namespace::none, |(uri, _)| Namespace::from_uri(uri.clone()));
                DomSourceEdit::set_attribute_ns(element, namespace, name, text)
            } else {
                DomSourceEdit::set_attribute(element, name, text)
            };
            self.apply_attribute(scope, &owner, &events, edit, value)
        })();
        match result {
            Ok(completion) => Ok(completion),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn apply_attribute<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        edit: DomSourceEdit,
        value: &TrustedAttributeValue,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let (result, pending) = self.with_native(owner, |session, _| {
            let mut source = session.source_execution();
            let mut dom = session.dom();
            let limits = events.limits.source_edits;
            let cancel = &owner.cancellation;
            let mut services = DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            };
            let result = source.apply(&mut dom, edit, limits, &mut services, cancel);
            let result = match result {
                Ok(DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::AttributeVerification(request),
                    ..
                }) => {
                    let mut lease = dom.resume_source_edit(request.operation(), limits, cancel)?;
                    let verified = verify(owner, attachment, &mut lease, request, value);
                    if let Err(error) = verified {
                        lease.request_cancel();
                        // Finish the cancelled prefix even if the original caller stopped.
                        let _ = lease.advance_for_runtime(&AtomicBool::new(false));
                        return Err(error);
                    }
                    drop(lease);
                    source.resume(&mut dom, request.operation(), limits, &mut services, cancel)
                }
                result => result,
            };
            Ok((result, dom.pending_source_edit()))
        })?;
        match result {
            Ok(DomSourceDriveProgress::Complete(done)) if done.cancellation_requested() => {
                Err(DomError::Cancelled.into())
            }
            Ok(DomSourceDriveProgress::Complete(_)) => Ok(Completion::Undefined),
            Ok(DomSourceDriveProgress::Failed(failure)) => {
                attribute_failure(scope, failure.into_parts().1)
            }
            Err(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(error)))
                if pending.is_none() && attribute_exception(&error).is_some() =>
            {
                attribute_failure(scope, error)
            }
            Err(error) => refuse(scope, &attachment.source_failures, error).map_err(Into::into),
            Ok(_) => Err(
                pending.map_or(DocumentGuestEventError::HostRouteUnavailable, |status| {
                    DocumentGuestEventError::SourcePending(status.operation())
                }),
            ),
        }
    }
}

fn verify(
    owner: &NativeOwner,
    attachment: &Attachment,
    lease: &mut DomSourceLease<'_>,
    request: DomSourceAttributeVerificationRequest,
    value: &TrustedAttributeValue,
) -> Result<(), DocumentGuestEventError> {
    let registration = attachment
        .registration
        .upgrade()
        .ok_or(DocumentScriptNativeError::Revoked)?;
    let cancel = &owner.cancellation;
    let expected = match request.sink() {
        DomSourceAttributeSink::EventHandler(_) => TrustedTypeKind::Script,
        DomSourceAttributeSink::HtmlIframeSrcdoc => TrustedTypeKind::Html,
        DomSourceAttributeSink::HtmlScriptSrc | DomSourceAttributeSink::SvgScriptHref => {
            TrustedTypeKind::ScriptUrl
        }
    };
    if !matches!(value, TrustedAttributeValue::Trusted(brand) if brand.kind() == expected) {
        let policies = AttributePolicies::read(lease, request, &registration, None, cancel)
            .map_err(|error| match error {
                AttributePolicyError::Dom(error) => DocumentGuestEventError::from(error),
                _ => DocumentScriptNativeError::TrustedTypesPolicyUnavailable.into(),
            })?;
        if policies.requires_default_policy(cancel)? {
            return Err(DocumentScriptNativeError::TrustedTypesPolicyUnavailable.into());
        }
    }
    let input = lease
        .attribute_verification_input(request, &registration, cancel)?
        .clone();
    let mut outcome = Some(DomSourceAttributeVerificationOutcome::Value(input));
    lease.accept_attribute_verification(request, &registration, &mut outcome, cancel)?;
    Ok(())
}

fn attribute_exception(error: &DomError) -> Option<DomExceptionName> {
    match error {
        DomError::InvalidCharacter => Some(DomExceptionName::InvalidCharacterError),
        DomError::Namespace => Some(DomExceptionName::NamespaceError),
        _ => None,
    }
}

fn attribute_failure<'scope>(
    scope: &mut Scope<'scope>,
    error: DomError,
) -> Result<Completion<'scope>, DocumentGuestEventError> {
    match attribute_exception(&error) {
        Some(name) => Ok(Completion::Throw(scope.dom_exception(name, &[])?)),
        None => Err(error.into()),
    }
}
