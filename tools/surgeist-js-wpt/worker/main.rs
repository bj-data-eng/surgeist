//! One native-backed qualification case per process.

use std::{
    error::Error as StdError,
    io::{self, Write},
    process::ExitCode,
    sync::{Arc, atomic::AtomicBool},
};

use surgeist_bindings_js::{Error, Exception, ExceptionPhase, Primitive, Scope, ValueKind};
use surgeist_dom::{DomError, hosted::DomHostedError};
use surgeist_html::scripting::HtmlScriptingPolicy;
use surgeist_js_wpt::{
    GuestDiagnostic, HarnessResult, HarnessStatus, Subtest, SubtestStatus, WorkerFailure,
    WorkerResult, read_request, write_report,
};
use surgeist_runtime::{
    DomSession,
    document_scripting::{
        DocumentGuestEventError, DocumentScriptError, DocumentScriptHost,
        DocumentScriptNativeError, DocumentScriptPermit, PreparedDocumentScriptHost,
    },
    event::DomEventDispatcher,
    hosted::DomHostedRuntimeError,
    source::{DomSourceDriveError, DomSourceRuntimeError},
};

mod fixtures;
mod policy;

const REQUEST_PAYLOAD_BYTES: u32 = 4_092;
const RESPONSE_PAYLOAD_BYTES: u32 = 32_764;
const MAX_DIAGNOSTIC_UNITS: usize = 1_024;

fn main() -> ExitCode {
    match serve() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "native Window worker: {error}");
            ExitCode::FAILURE
        }
    }
}

fn serve() -> Result<(), Box<dyn StdError>> {
    let identity = read_request(&mut io::stdin().lock(), REQUEST_PAYLOAD_BYTES)?;
    let result = match fixtures::resolve(&identity) {
        Some(fixture) => run_case(fixture)?,
        None => unavailable("fixture identity absent from worker inventory"),
    };
    let mut output = io::stdout().lock();
    write_report(&mut output, &identity, &result, RESPONSE_PAYLOAD_BYTES)?;
    output.flush()?;
    Ok(())
}

fn run_case(fixture: fixtures::Fixture) -> Result<WorkerResult, Box<dyn StdError>> {
    let cancellation = Arc::new(AtomicBool::new(false));
    let mut session = DomSession::new(policy::dom()?, policy::session())?;
    let mut hosted = session.hosted_execution();
    let host = hosted.create_host(&mut session.dom(), &cancellation)?;
    let top = hosted.create_top_level(&mut session.dom(), host, &cancellation)?;
    let mut registry = DomEventDispatcher::new(policy::REGISTRY)?;
    registry.bind(&mut session.dom(), &cancellation)?;
    let mut registry = Some(registry);

    let prepared = match PreparedDocumentScriptHost::prepare(
        session,
        policy::SCRIPT_STORAGE_BYTES,
        cancellation,
    ) {
        Ok(prepared) => prepared,
        Err(failure) => return Ok(native_refusal(&failure.error)),
    };
    let mut script = match prepared.enable_with_guest_event_registry(
        top,
        HtmlScriptingPolicy {
            user_allows: !matches!(fixture.admission, fixtures::Admission::ScriptingDisabled),
            automation_allows: true,
        },
        None,
        &mut registry,
        policy::guest(fixture.admission),
    ) {
        Ok(script) => script,
        Err(failure) => match failure.error {
            DocumentGuestEventError::Document(error) => return script_failure(error),
            other => return Ok(guest_event_failure(&other)),
        },
    };

    let result = evaluate(&mut script, fixture.source);
    drop(script.stop());
    match result {
        Ok(Some(result)) => Ok(result),
        Ok(None) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "qualification source must return a callable with a Boolean result",
        )
        .into()),
        Err(error) => script_failure(error),
    }
}

fn evaluate(
    host: &mut DocumentScriptHost,
    source: &str,
) -> Result<Option<WorkerResult>, DocumentScriptError> {
    let permit = host.permit()?;
    let source = source.encode_utf16().collect::<Vec<_>>();
    let completion = match observe(host, &permit, |scope| {
        let value = scope.evaluate(&source)?;
        if !scope.is_callable(&value)? {
            return Ok(None);
        }
        Ok(Some(scope.retain(&value)))
    })? {
        Ok(Some(completion)) => completion,
        Ok(None) => return Ok(None),
        Err(failure) => return Ok(Some(WorkerResult::Failure(failure))),
    };

    host.checkpoint()?;
    let passed = match observe(host, &permit, |scope| {
        let function = scope.import(&completion)?;
        let receiver = scope.primitive(Primitive::Undefined)?;
        let value = scope.call(&function, &receiver, &[])?;
        if scope.kind(&value)? != ValueKind::Boolean {
            return Ok(None);
        }
        scope.boolean(&value).map(Some)
    })? {
        Ok(Some(passed)) => passed,
        Ok(None) => return Ok(None),
        Err(failure) => return Ok(Some(WorkerResult::Failure(failure))),
    };

    Ok(Some(WorkerResult::Harness(HarnessResult {
        status: HarnessStatus::Ok,
        diagnostic: empty_diagnostic(),
        subtests: vec![Subtest {
            index: 0,
            name: "qualification assertion".encode_utf16().collect(),
            status: if passed {
                SubtestStatus::Pass
            } else {
                SubtestStatus::Fail
            },
            diagnostic: empty_diagnostic(),
        }],
    })))
}

fn observe<T>(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    action: impl for<'scope> FnOnce(&mut Scope<'scope>) -> Result<T, Error>,
) -> Result<Result<T, WorkerFailure>, DocumentScriptError> {
    match host.in_document(permit, action) {
        Err(DocumentScriptError::Script(Error::Exception(exception))) => {
            Ok(Err(guest_exception(host, permit, &exception)))
        }
        result => result.map(Ok),
    }
}

fn guest_exception(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    exception: &Exception,
) -> WorkerFailure {
    // Reading arbitrary error properties or coercing an object could reenter
    // guest code. Retain only an original string primitive, under the host's
    // preparation admission and the report's tighter size limit. Observe it
    // after the original exception is selected so a secondary refusal cannot
    // change the primary result.
    let message = host
        .in_document(permit, |scope| {
            let value = scope.import(exception.value())?;
            if scope.kind(&value)? != ValueKind::String {
                return Ok(None);
            }
            let admitted = scope.to_utf16(&value)?;
            let mut diagnostic = Vec::new();
            if admitted.len() > MAX_DIAGNOSTIC_UNITS
                || diagnostic.try_reserve_exact(admitted.len()).is_err()
            {
                return Ok(None);
            }
            diagnostic.extend_from_slice(&admitted);
            Ok(Some(diagnostic))
        })
        .ok()
        .flatten();
    let diagnostic = GuestDiagnostic {
        message,
        stack: None,
    };
    match exception.phase() {
        ExceptionPhase::Compile => WorkerFailure::GuestSyntax(diagnostic),
        _ => WorkerFailure::GuestRuntime(diagnostic),
    }
}

fn script_failure(error: DocumentScriptError) -> Result<WorkerResult, Box<dyn StdError>> {
    match error {
        DocumentScriptError::Native(error) => Ok(native_refusal(&error)),
        DocumentScriptError::Script(Error::HostFailure(ref failure)) => {
            if let Some(cause) = failure.cause::<DocumentScriptNativeError>() {
                Ok(native_refusal(cause))
            } else if let Some(cause) = failure.cause::<DocumentGuestEventError>() {
                Ok(guest_event_failure(cause))
            } else if let Some(cause) = failure.cause::<DomSourceDriveError>() {
                Ok(source_refusal(cause))
            } else {
                let cause = failure.source().unwrap_or(failure);
                Ok(refusal("runtime", "host-operation", &cause.to_string()))
            }
        }
        DocumentScriptError::Script(Error::ConversionLedger(ref cause)) => {
            Ok(refusal("bindings", "conversion-ledger", &cause.to_string()))
        }
        DocumentScriptError::Script(Error::ConversionAdmission(ref cause)) => Ok(refusal(
            "bindings",
            "conversion-admission",
            &cause.to_string(),
        )),
        other => Err(other.into()),
    }
}

fn guest_event_failure(error: &DocumentGuestEventError) -> WorkerResult {
    let code = match error {
        DocumentGuestEventError::HostRouteUnavailable => {
            return unavailable("native guest EventTarget host route");
        }
        DocumentGuestEventError::Timestamp(_) => return unavailable("native event timestamp"),
        DocumentGuestEventError::WorkLimit => "guest-work-limit",
        DocumentGuestEventError::Accounting(_) => "guest-accounting",
        DocumentGuestEventError::Dispatch(_) => "guest-dispatch",
        _ => "guest-operation",
    };
    refusal("runtime", code, &error.to_string())
}

fn source_refusal(error: &DomSourceDriveError) -> WorkerResult {
    if let DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
        cause @ DomError::SourceWorkLimit { .. },
    )) = error
    {
        return refusal("runtime", "source-work-limit", &cause.to_string());
    }
    refusal("runtime", "source-drive", &error.to_string())
}

fn unavailable(capability: &str) -> WorkerResult {
    WorkerResult::Failure(WorkerFailure::UnavailableCapability {
        capability: capability.to_owned(),
        diagnostic: empty_diagnostic(),
    })
}

fn native_refusal(error: &DocumentScriptNativeError) -> WorkerResult {
    let code = match error {
        DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Dom(
            DomHostedError::ScriptingDisabled,
        )) => "scripting-disabled",
        DocumentScriptNativeError::StorageLimit => "storage-limit",
        DocumentScriptNativeError::CreationRealmWorkLimit => "creation-realm-work-limit",
        _ => "native-operation",
    };
    refusal("runtime", code, &error.to_string())
}

fn refusal(owner: &str, code: &str, message: &str) -> WorkerResult {
    WorkerResult::Failure(WorkerFailure::NativeRefusal {
        owner: owner.to_owned(),
        code: code.to_owned(),
        diagnostic: GuestDiagnostic {
            message: Some(message.encode_utf16().take(MAX_DIAGNOSTIC_UNITS).collect()),
            stack: None,
        },
    })
}

const fn empty_diagnostic() -> GuestDiagnostic {
    GuestDiagnostic {
        message: None,
        stack: None,
    }
}
