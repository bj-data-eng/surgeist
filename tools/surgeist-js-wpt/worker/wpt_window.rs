//! Drive one pinned page through its native parser, readiness and task owners.

use std::{
    cell::RefCell, error::Error, num::NonZeroUsize, rc::Rc, sync::atomic::AtomicBool, thread,
};

use surgeist_dom::{DomError, identity::NodeId};
use surgeist_html_parser::live::{HtmlLiveParseError, HtmlLiveParseProgress};
use surgeist_js_wpt::{GuestDiagnostic, WorkerFailure, WorkerResult};
use surgeist_runtime::{
    document_scripting::{
        DocumentHtmlCompletionError, DocumentHtmlCompletionProgress, DocumentScriptError,
        DocumentScriptEventError, DocumentScriptHost, DocumentScriptPermit, WindowTimerProgress,
    },
    hosted::{DomHostedHtmlError, DomHostedHtmlParser},
};

use super::{
    fixtures::{HtmlFixture, LocalScript},
    guest_event_failure, guest_exception, native_refusal, observe, policy, refusal, script_failure,
    unavailable,
    wpt_completion::{self, Captured},
};

const TOKENS_PER_STEP: NonZeroUsize = NonZeroUsize::new(32).unwrap();

pub(super) fn run(
    host: &mut DocumentScriptHost,
    mut parser: DomHostedHtmlParser<'_>,
    fixture: &HtmlFixture,
    cancel: &AtomicBool,
) -> Result<WorkerResult, Box<dyn Error>> {
    let permit = match host.permit() {
        Ok(permit) => permit,
        Err(error) => return Ok(native_refusal(&error)),
    };
    let report = Rc::new(RefCell::new(Captured::Pending));
    let mut continuation = loop {
        let progress = host.with_native(|dom, hosted, _| {
            parser
                .advance(&hosted, dom, TOKENS_PER_STEP, cancel)
                .map_err(parser_failure)
        });
        match progress {
            Ok(Ok(HtmlLiveParseProgress::Paused)) => {}
            Ok(Ok(HtmlLiveParseProgress::AwaitingEof)) => parser.accept_eof(),
            Ok(Ok(HtmlLiveParseProgress::Script)) => {
                let Some(node) = parser.pending_script() else {
                    return Ok(unavailable("canonical parser-inserted script"));
                };
                let script = match script_source(host, fixture, node) {
                    Ok(source) => source,
                    Err(result) => return Ok(result),
                };
                let observed = observe(host, &permit, |scope| {
                    scope.evaluate(&script.units)?;
                    if script.installs_reporter {
                        wpt_completion::install(scope, Rc::clone(&report))?;
                    }
                    Ok(())
                });
                match observed {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) => return Ok(WorkerResult::Failure(error)),
                    Err(error) => return script_failure(error),
                }
                if let Err(error) = host.checkpoint() {
                    return script_failure(error);
                }
                match host.with_native(|dom, hosted, _| {
                    parser
                        .complete_script(&hosted, dom, node, cancel)
                        .map_err(parser_failure)
                }) {
                    Ok(Ok(())) => {}
                    Ok(Err(result)) => return Ok(result),
                    Err(error) => return Ok(native_refusal(&error)),
                }
            }
            Ok(Ok(HtmlLiveParseProgress::StopParsing)) => {
                match host.with_native(|dom, hosted, _| {
                    parser
                        .take_stop_parsing(&hosted, dom, cancel)
                        .map_err(parser_failure)
                }) {
                    Ok(Ok(Some(continuation))) => break Some(continuation),
                    Ok(Ok(None)) => {
                        return Ok(unavailable("native HTML stop-parsing continuation"));
                    }
                    Ok(Err(result)) => return Ok(result),
                    Err(error) => return Ok(native_refusal(&error)),
                }
            }
            Ok(Err(result)) => return Ok(result),
            Err(error) => return Ok(native_refusal(&error)),
        }
    };

    if let Err(error) = host.begin_html_completion(&permit, &mut continuation) {
        return completion_failure(host, &permit, "begin-html-completion", error);
    }
    loop {
        match host.service_html_completion(&permit, policy::WINDOW_DISPATCH) {
            Ok(
                DocumentHtmlCompletionProgress::Interactive
                | DocumentHtmlCompletionProgress::DomContentLoaded,
            ) => {}
            Ok(DocumentHtmlCompletionProgress::Loaded) => break,
            Ok(DocumentHtmlCompletionProgress::Idle) => {
                return Ok(unavailable("native HTML Window load task"));
            }
            Ok(DocumentHtmlCompletionProgress::Retained(outcome)) => {
                return Ok(refusal(
                    "runtime",
                    "html-completion-retained",
                    &format!("{outcome:?}"),
                ));
            }
            Ok(_) => return Ok(unavailable("native HTML completion task phase")),
            Err(error) => {
                return completion_failure(host, &permit, "service-html-completion", error);
            }
        }
    }
    service_timers(host, &permit, &report)
}

struct ScriptSource {
    units: Vec<u16>,
    installs_reporter: bool,
}

fn script_source(
    host: &DocumentScriptHost,
    fixture: &HtmlFixture,
    id: NodeId,
) -> Result<ScriptSource, WorkerResult> {
    let snapshot = host.snapshot().map_err(|error| native_refusal(&error))?;
    let node = snapshot
        .node(id)
        .map_err(|error| refusal("dom", "parser-script-source", &error.to_string()))?;
    if let Some(url) = node.attribute(&"src".into()) {
        let (source, installs_reporter) = match fixture.external_script(url.code_units()) {
            Some(LocalScript::Harness(source)) => (source, false),
            Some(LocalScript::HarnessReport(source)) => (source, true),
            None => return Err(unavailable("pinned local parser script URL")),
        };
        return Ok(ScriptSource {
            units: source.encode_utf16().collect(),
            installs_reporter,
        });
    }
    let mut source = Vec::new();
    node.visit_child_text_with::<DomError>(
        || Ok(()),
        |text| {
            source.extend_from_slice(text.code_units());
            Ok(())
        },
    )
    .map_err(|error| refusal("dom", "parser-script-source", &error.to_string()))?;
    Ok(ScriptSource {
        units: source,
        installs_reporter: false,
    })
}

fn service_timers(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    report: &RefCell<Captured>,
) -> Result<WorkerResult, Box<dyn Error>> {
    loop {
        match host.service_window_timer(permit) {
            Ok(WindowTimerProgress::Fired {
                exception: None, ..
            }) => match report.take() {
                Captured::Report(result) => return Ok(WorkerResult::Harness(result)),
                Captured::Invalid => {
                    return Ok(refusal(
                        "worker",
                        "harness-report",
                        "invalid or over-limit public completion report",
                    ));
                }
                Captured::Pending => {}
            },
            Ok(
                WindowTimerProgress::Fired {
                    exception: Some(exception),
                    ..
                }
                | WindowTimerProgress::CheckpointRefused {
                    exception: Some(exception),
                    ..
                },
            ) => {
                return Ok(WorkerResult::Failure(guest_exception(
                    host, permit, &exception,
                )));
            }
            Ok(WindowTimerProgress::Waiting(wait)) => thread::sleep(wait),
            Ok(WindowTimerProgress::Idle) => {
                return Ok(unavailable(
                    "public WPT completion from native Window tasks",
                ));
            }
            Ok(WindowTimerProgress::CheckpointRefused { error, .. }) | Err(error) => {
                return observed_script_failure(host, permit, error);
            }
        }
    }
}

fn completion_failure(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    stage: &str,
    error: DocumentHtmlCompletionError,
) -> Result<WorkerResult, Box<dyn Error>> {
    match error {
        DocumentHtmlCompletionError::Script(error)
        | DocumentHtmlCompletionError::Event(DocumentScriptEventError::Access(error)) => {
            observed_script_failure(host, permit, error)
        }
        DocumentHtmlCompletionError::Event(DocumentScriptEventError::Guest(error)) => {
            Ok(guest_event_failure(&error))
        }
        other => Ok(refusal("runtime", stage, &format!("{other:?}"))),
    }
}

fn observed_script_failure(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    error: DocumentScriptError,
) -> Result<WorkerResult, Box<dyn Error>> {
    match error {
        DocumentScriptError::Script(surgeist_bindings_js::Error::Exception(exception)) => Ok(
            WorkerResult::Failure(guest_exception(host, permit, &exception)),
        ),
        other => script_failure(other),
    }
}

pub(super) fn parser_failure(error: &DomHostedHtmlError) -> WorkerResult {
    match error {
        DomHostedHtmlError::Start(HtmlLiveParseError::Unsupported { reason, source })
        | DomHostedHtmlError::Parse(HtmlLiveParseError::Unsupported { reason, source }) => {
            WorkerResult::Failure(WorkerFailure::UnavailableCapability {
                capability: "local live HTML parser profile".into(),
                diagnostic: GuestDiagnostic {
                    message: Some(
                        format!(
                            "{reason:?} at {:?} offsets {}..{}",
                            source.coordinates(),
                            source.start(),
                            source.end(),
                        )
                        .encode_utf16()
                        .collect(),
                    ),
                    stack: None,
                },
            })
        }
        other => refusal("runtime", "live-html-parser", &format!("{other:?}")),
    }
}
