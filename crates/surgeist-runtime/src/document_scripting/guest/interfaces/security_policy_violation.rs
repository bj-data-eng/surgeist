//! The standard SecurityPolicyViolationEvent interface on the attached Window.

use super::*;
use crate::dom_event::{AuthoredCspViolationInit, CspEventText};
use surgeist_bindings_js::conversion::SecurityPolicyViolationEventDisposition;

#[derive(Clone, Copy)]
enum Member {
    Text(CspEventText),
    Disposition,
    StatusCode,
    LineNumber,
    ColumnNumber,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    event: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<Value<'scope>, DocumentGuestEventError> {
    let event_constructor = scope.get(event, PropertyKey::String(CONSTRUCTOR))?;
    let fallback = capture(size_of::<Capture>())?;
    let constructor = constructor_capture(capture, size_of::<Capture>())?;
    let callback = ConstructorCallback::bounded(
        2,
        move |scope| {
            let (_, events) = fallback.active(scope)?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            scope.open_callback_object(&public.security_policy_violation)
        },
        move |scope, arguments| construct(scope, arguments, &constructor),
    );
    let (_, prototype) = document::install_derived_interface(
        scope,
        event,
        &event_constructor,
        FunctionMetadata {
            name: &ascii(b"SecurityPolicyViolationEvent"),
            length: 1,
        },
        &callback,
    )?;
    for (member, key, name) in [
        (
            Member::Text(CspEventText::DocumentUri),
            &ascii(b"documentURI")[..],
            &ascii(b"get documentURI")[..],
        ),
        (
            Member::Text(CspEventText::Referrer),
            &ascii(b"referrer")[..],
            &ascii(b"get referrer")[..],
        ),
        (
            Member::Text(CspEventText::BlockedUri),
            &ascii(b"blockedURI")[..],
            &ascii(b"get blockedURI")[..],
        ),
        (
            Member::Text(CspEventText::EffectiveDirective),
            &ascii(b"effectiveDirective")[..],
            &ascii(b"get effectiveDirective")[..],
        ),
        (
            Member::Text(CspEventText::ViolatedDirective),
            &ascii(b"violatedDirective")[..],
            &ascii(b"get violatedDirective")[..],
        ),
        (
            Member::Text(CspEventText::OriginalPolicy),
            &ascii(b"originalPolicy")[..],
            &ascii(b"get originalPolicy")[..],
        ),
        (
            Member::Text(CspEventText::SourceFile),
            &ascii(b"sourceFile")[..],
            &ascii(b"get sourceFile")[..],
        ),
        (
            Member::Text(CspEventText::Sample),
            &ascii(b"sample")[..],
            &ascii(b"get sample")[..],
        ),
        (
            Member::Disposition,
            &ascii(b"disposition")[..],
            &ascii(b"get disposition")[..],
        ),
        (
            Member::StatusCode,
            &ascii(b"statusCode")[..],
            &ascii(b"get statusCode")[..],
        ),
        (
            Member::LineNumber,
            &ascii(b"lineNumber")[..],
            &ascii(b"get lineNumber")[..],
        ),
        (
            Member::ColumnNumber,
            &ascii(b"columnNumber")[..],
            &ascii(b"get columnNumber")[..],
        ),
    ] {
        let capture = capture(size_of::<(Capture, Member)>())?;
        let getter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| get(scope, arguments, &capture, member)),
            FunctionMetadata { name, length: 0 },
        )?;
        define_accessor(scope, &prototype, key, &getter, None, true)?;
    }
    Ok(prototype)
}

fn construct<'scope>(
    scope: &mut Scope<'scope>,
    arguments: ConstructorArguments<'scope>,
    capture: &Capture,
) -> Result<ConstructorCompletion<'scope>, Error> {
    let (owner, events) = capture.active(scope)?;
    let result = (|| -> Result<_, DocumentGuestEventError> {
        let ty = event_constructor_0_type();
        let (ty, init) = capture
            .callbacks
            .conversion
            .run(scope, |conversion| {
                let ty = conversion.argument(&ty, arguments.get(ty.index()))?;
                let init = conversion.security_policy_violation_event_init(arguments.get(1))?;
                Ok((ty, init))
            })
            .map_err(Error::from)?;
        let ArgumentValue::Present(ty) = ty.get() else {
            return Err(Error::BackendUnavailable.into());
        };
        let selected = arguments.select_prototype(scope)?;
        scope.check_active()?;
        events.step()?;
        let native = capture.callbacks.with_native(&owner, |session, global| {
            let init = AuthoredCspViolationInit {
                event: DomEventInit {
                    bubbles: init.bubbles,
                    cancelable: init.cancelable,
                    composed: init.composed,
                },
                text: [
                    init.document_uri.get().as_code_units(),
                    init.referrer.get().as_code_units(),
                    init.blocked_uri.get().as_code_units(),
                    init.effective_directive.get().as_code_units(),
                    init.violated_directive.get().as_code_units(),
                    init.original_policy.get().as_code_units(),
                    init.source_file.get().as_code_units(),
                    init.sample.get().as_code_units(),
                ],
                report_only: init.disposition == SecurityPolicyViolationEventDisposition::Report,
                status_code: init.status_code,
                line_number: init.line_number,
                column_number: init.column_number,
            };
            session
                .hosted_execution()
                .create_csp_violation_event(
                    &mut session.dom(),
                    global,
                    ty.as_code_units(),
                    init,
                    &owner.cancellation,
                )
                .map_err(DocumentGuestEventError::Construction)
        })?;
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let trusted = public.trusted_getter(scope)?;
        let value = scope.project_native_object(
            ObjectPrototype::Object(selected.prototype()),
            GuestEventProjection(native),
            |bytes| events.reserve(&owner, bytes),
        )?;
        expose_trust(scope, &value, &trusted)?;
        Ok(value)
    })();
    match result {
        Ok(value) => Ok(ConstructorCompletion::Return(value)),
        Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => Err(error),
        Err(error) => refuse(scope, &events.refusal, error),
    }
}

fn get<'scope>(
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
    capture: &Capture,
    member: Member,
) -> Result<Completion<'scope>, Error> {
    let (_, events) = capture.active(scope)?;
    let projection = scope.native_object::<GuestEventProjection>(arguments.receiver())?;
    let Some(payload) = projection
        .as_ref()
        .and_then(|projection| projection.payload().0.csp_violation())
    else {
        return Ok(Completion::Throw(
            scope.type_error(&ascii(b"SecurityPolicyViolationEvent required"))?,
        ));
    };
    let result = (|| {
        events.step()?;
        Ok(match member {
            Member::Text(member) => capture
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.copy_to_js(payload.text(member))
                })
                .map_err(Error::from)?,
            Member::Disposition => {
                const REPORT: &[u16] = &ascii(b"report");
                const ENFORCE: &[u16] = &ascii(b"enforce");
                scope.string(if payload.report_only { REPORT } else { ENFORCE })?
            }
            Member::StatusCode => {
                scope.primitive(Primitive::Number(f64::from(payload.status_code)))?
            }
            Member::LineNumber => {
                scope.primitive(Primitive::Number(f64::from(payload.line_number)))?
            }
            Member::ColumnNumber => {
                scope.primitive(Primitive::Number(f64::from(payload.column_number)))?
            }
        })
    })();
    document::complete(scope, &events, result)
}
