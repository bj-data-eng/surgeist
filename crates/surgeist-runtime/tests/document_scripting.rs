//! Real entry-thread V8 ownership over an original native Runtime Session.

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use surgeist_bindings_js::dom_exception::DomExceptionName;
use surgeist_bindings_js::{
    Callback, Completion, EngineKind, Error, Primitive, Process, PropertyKey, Scope, ValueKind,
};
use surgeist_css::{
    query::media::CssMediaScripting,
    serialization::{CssSerializationError, CssomIdentifierLimits},
    source::CssUtf16InputLimits,
};
use surgeist_dom::*;
use surgeist_html::scripting::HtmlScriptingPolicy;
use surgeist_html_semantics::{HtmlCaptureLimits, HtmlCapturedNodeKind, HtmlScriptingStatus};
use surgeist_runtime::{document_scripting::*, *};

#[path = "document_scripting/source_events.rs"]
mod source_events;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}

fn limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: surgeist_runtime::DomSourceEventRuntimeLimits {
            jobs: 0,
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
    }
}
fn session(policy: DomSessionLimits) -> DomSession {
    let dom = Dom::new(
        DomLimits::new(
            256,
            200_000,
            1024,
            1024,
            1024,
            32,
            4_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        ),
    )
    .unwrap();
    DomSession::new(dom, policy).unwrap()
}
fn create(session: &mut DomSession) -> (HostedRuntime, DomHostedTopLevel) {
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel()).unwrap();
    // Initial no-opener creation reaches no URL parsing service.
    let top = hosted.create_top_level(&mut dom, host, &cancel()).unwrap();
    (hosted, top)
}

const ALLOW: HtmlScriptingPolicy = HtmlScriptingPolicy {
    user_allows: true,
    automation_allows: true,
};
const STORAGE: usize = 1_000_000;

fn hooks(child_count: &[u16]) -> DocumentScriptHooks<'_> {
    const DETACH: [u16; 8] = [100, 101, 116, 97, 99, 104, 73, 102];
    DocumentScriptHooks {
        child_count,
        detach_source_node_if: &DETACH,
    }
}

fn hook() -> Vec<u16> {
    "readChildCount".encode_utf16().collect()
}

fn setup() -> (DomSession, DomHostedTopLevel) {
    let mut session = session(limits());
    let (_, top) = create(&mut session);
    (session, top)
}

fn prepare(session: DomSession, cancellation: Arc<AtomicBool>) -> PreparedDocumentScriptHost {
    PreparedDocumentScriptHost::prepare(session, STORAGE, cancellation).unwrap()
}

fn captured_status(capture: &DomHtmlCapture, document: DocumentId) -> HtmlScriptingStatus {
    let subject = capture.subject(document.node()).unwrap();
    match capture.capture().node(&subject).unwrap().kind() {
        HtmlCapturedNodeKind::Document(document) => document.scripting(),
        other => panic!("unexpected captured subject: {other:?}"),
    }
}

fn capture_limits() -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(32, 96, 32, 4096, 16, 8, 8, 4096)
}

fn assert_script_number(scope: &mut Scope<'_>, script: &str, expected: f64) -> Result<(), Error> {
    let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
    let number = scope.primitive(Primitive::Number(expected))?;
    assert!(scope.strict_equals(&value, &number)?);
    Ok(())
}

fn eval(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit, script: &str, expected: f64) {
    host.in_document(permit, |scope| {
        assert_script_number(scope, script, expected)
    })
    .unwrap();
}

fn native_window_exposes_dom_exception() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());

    let kind = host
        .in_document(&permit, |scope| {
            let script: Vec<_> = "typeof globalThis.DOMException".encode_utf16().collect();
            let value = scope.evaluate(&script)?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(
        String::from_utf16(&kind).unwrap(),
        "function",
        "native Window DOMException constructor is missing"
    );

    // The selected Web IDL DOMException IDL and the pinned WPT custom bindings
    // define these descriptor, default and receiver observations.
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const constructor = DOMException;
            const prototype = constructor.prototype;
            const global = Object.getOwnPropertyDescriptor(globalThis, "DOMException");
            const exception = new constructor(undefined, undefined);
            const descriptor = Object.getOwnPropertyDescriptor(prototype, "name");
            let brandRefused = false;
            try { descriptor.get.call(Object.create(prototype)); }
            catch (error) { brandRefused = error instanceof TypeError; }
            let callRefused = false;
            try { constructor(); }
            catch (error) { callRefused = error instanceof TypeError; }
            return global.value === constructor && global.writable && global.configurable
                && !global.enumerable && constructor.length === 0
                && Object.getPrototypeOf(constructor) === Function.prototype
                && Object.getPrototypeOf(prototype) === Error.prototype
                && typeof descriptor.get === "function" && descriptor.set === undefined
                && descriptor.enumerable && descriptor.configurable
                && exception.message === "" && exception.name === "Error" && exception.code === 0
                && !Object.hasOwn(exception, "message") && !Object.hasOwn(exception, "name")
                && !Object.hasOwn(exception, "code") && brandRefused && callRefused
                && Object.prototype.toString.call(exception) === "[object DOMException]";
        })())"#,
        1.0,
    );
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const exception = new DOMException("\ud800", "InvalidCharacterError");
            Object.defineProperty(exception, "name", {value: "WrongDocumentError"});
            return exception.message === "\ud800"
                && DOMException.INVALID_CHARACTER_ERR === 5
                && DOMException.prototype.INVALID_CHARACTER_ERR === 5
                && exception.code === 5;
        })())"#,
        1.0,
    );
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            class Derived extends DOMException {
                constructor(message) { super(message, "InvalidStateError"); }
            }
            const exception = new Derived("derived");
            const name = Object.getOwnPropertyDescriptor(DOMException.prototype, "name").get;
            return exception instanceof Derived
                && exception instanceof DOMException && exception instanceof Error
                && Object.getPrototypeOf(exception) === Derived.prototype
                && Error.isError(exception) && name.call(exception) === "InvalidStateError"
                && exception.message === "derived" && exception.code === 11;
        })())"#,
        1.0,
    );
}

fn native_window_dom_exception_uses_error_data_and_preserves_abrupt_completion() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let callback = Callback::new(|scope, _| {
            Ok(Completion::Throw(scope.dom_exception(
                DomExceptionName::InvalidStateError,
                &[0xd800, 0, 0xdc00],
            )?))
        });
        let function = scope.function(&callback)?;
        let global = scope.global()?;
        scope.set(
            &global,
            PropertyKey::String(&"throwNative".encode_utf16().collect::<Vec<_>>()),
            &function,
        )
    })
    .unwrap();

    // The selected Web IDL exception binding requires ErrorData/Stack and an
    // implementation-checked receiver. V8's Error.isError checks ErrorData.
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const Constructor = DOMException;
            const prototype = Constructor.prototype;
            const errorIsError = Error.isError;
            const stackType = typeof new Error().stack;
            const exception = new Constructor("message", "TimeoutError");
            const own = Object.getOwnPropertyDescriptor;
            const data = (object, key, value, w, e, c) => {
                const d = own(object, key);
                return d.value === value && d.writable === w && d.enumerable === e
                    && d.configurable === c;
            };
            const accessors = ["name", "message", "code"].map(key => own(prototype, key));
            const refuses = (getter, receiver) => {
                try { getter.call(receiver); return false; }
                catch (error) { return error instanceof TypeError; }
            };
            if (!errorIsError(new Error()) || !errorIsError(prototype)
                || !errorIsError(exception) || errorIsError(Object.create(prototype))
                || errorIsError(new Proxy(exception, {}))
                || typeof exception.stack !== stackType || typeof prototype.stack !== stackType
                || Reflect.ownKeys(exception).some(key => key !== "stack")
                || !data(Constructor, "name", "DOMException", false, false, true)
                || !data(Constructor, "length", 0, false, false, true)
                || !data(Constructor, "prototype", prototype, false, false, false)
                || Object.hasOwn(Constructor, "arguments") || Object.hasOwn(Constructor, "caller")
                || Object.getOwnPropertyNames(prototype).at(-1) !== "constructor"
                || Object.keys(prototype).slice(0, 3).join(",") !== "name,message,code"
                || !data(prototype, "constructor", Constructor, true, false, true)
                || !data(prototype, Symbol.toStringTag, "DOMException", false, false, true)
                || !data(Constructor, "DATA_CLONE_ERR", 25, false, true, false)
                || !data(prototype, "DOMSTRING_SIZE_ERR", 2, false, true, false)
                || new Constructor("", "DOMStringSizeError").code !== 0
                || new Constructor("", "NoDataAllowedError").code !== 0
                || new Constructor("", "ValidationError").code !== 0
                || new Constructor("", "OperationError").code !== 0
                || new Constructor("", "timeoutError").code !== 0) return false;

            const attempted = [];
            const proxy = new Proxy(exception, {
                get() { attempted.push("proxy get"); },
                getOwnPropertyDescriptor() { attempted.push("proxy descriptor"); }
            });
            for (const [index, key] of ["name", "message", "code"].entries()) {
                const {get, set, enumerable, configurable} = accessors[index];
                if (get.name !== "get " + key || get.length !== 0 || set !== undefined
                    || !enumerable || !configurable
                    || !refuses(get, prototype) || !refuses(get, new Error())
                    || !refuses(get, Object.create(exception)) || !refuses(get, proxy)
                    || !refuses(get, null)) return false;
                try { new get(); return false; }
                catch (error) { if (!(error instanceof TypeError)) return false; }
            }
            if (attempted.length !== 0) return false;
            Object.setPrototypeOf(exception, null);
            if (accessors[2].get.call(exception) !== 23) return false;

            const abrupt = {};
            const log = [];
            const get = (label, value) => ({toString() {
                log.push(label);
                if (value === abrupt) throw abrupt;
                return value;
            }});
            let failPrototype = false;
            const custom = {};
            const target = new Proxy(function Target(){}, {
                get(object, key, receiver) {
                    if (key !== "prototype") return Reflect.get(object, key, receiver);
                    log.push("prototype");
                    if (failPrototype) throw abrupt;
                    return custom;
                }
            });
            const construct = (...args) => Reflect.construct(Constructor, args, target);
            const result = construct(get("message", "\udc00"), get("name", "\ud800"), get("unused", abrupt));
            if (log.join(",") !== "message,name,prototype" || !errorIsError(result)
                || Object.getPrototypeOf(result) !== custom
                || accessors[0].get.call(result) !== "\ud800"
                || accessors[1].get.call(result) !== "\udc00"
                || accessors[2].get.call(result) !== 0) return false;
            for (const [message, name, fail, expected] of [
                [abrupt, "n", false, "message"],
                ["m", abrupt, false, "message,name"],
                ["m", "n", true, "message,name,prototype"]
            ]) {
                log.length = 0;
                failPrototype = fail;
                try { construct(get("message", message), get("name", name)); return false; }
                catch (error) { if (error !== abrupt || log.join(",") !== expected) return false; }
            }
            log.length = 0;
            try { Constructor(get("message", abrupt)); return false; }
            catch (error) { if (!(error instanceof TypeError) || log.length !== 0) return false; }

            const reentered = new Constructor({toString() {
                return new Constructor("inner", "AbortError").message;
            }}, "NotFoundError");
            if (reentered.message !== "inner" || reentered.code !== 8) return false;

            let lookups = 0;
            const trap = {get() { lookups++; throw abrupt; }};
            Object.defineProperty(globalThis, "Error", trap);
            Object.defineProperty(globalThis, "DOMException", trap);
            const ownInstance = new Constructor(null, "SyntaxError");
            if (ownInstance.message !== "null" || ownInstance.code !== 12) return false;
            try { throwNative(); return false; }
            catch (error) {
                return errorIsError(error) && Object.getPrototypeOf(error) === prototype
                    && error.name === "InvalidStateError" && error.code === 11
                    && error.message === "\ud800\0\udc00" && lookups === 0;
            }
        })())"#,
        1.0,
    );
}

fn native_window_dom_exception_admission_and_hook_collision() {
    let (session, top) = setup();
    let before = session.snapshot();
    let mut prepared = prepare(session, Arc::new(cancel()));
    let name: Vec<_> = "DOMException".encode_utf16().collect();
    for names in [
        hooks(&name),
        DocumentScriptHooks {
            child_count: &[65],
            detach_source_node_if: &name,
        },
    ] {
        let failure = match prepared.enable(top, ALLOW, names) {
            Err(failure) => failure,
            Ok(_) => panic!("DOMException hook collision published the Window"),
        };
        assert!(matches!(
            failure.error,
            DocumentScriptError::Script(Error::PropertyRejected)
        ));
        prepared = failure.owner;
    }
    let recovered = prepared.into_session().unwrap();
    assert_eq!(recovered.snapshot().revision(), before.revision());

    // Default "Error" is five UTF-16 units and ten live Rust bytes. The
    // two-unit message and default name cost seven units inward, then five
    // and two outward. Both Rust buffers remain live during the output copies.
    for (limits, accepted) in [
        (DomExceptionLimits::new(2, 14, 14), true),
        (DomExceptionLimits::new(1, 14, 14), false),
        (DomExceptionLimits::new(2, 6, 14), false),
        (DomExceptionLimits::new(2, 11, 14), false),
        (DomExceptionLimits::new(2, 13, 14), false),
        (DomExceptionLimits::new(2, 14, 13), false),
    ] {
        let (session, top) = setup();
        let mut host = prepare(session, Arc::new(cancel()))
            .with_dom_exception_limits(limits)
            .enable(top, ALLOW, hooks(&hook()))
            .unwrap();
        let permit = host.permit().unwrap();
        if accepted {
            eval(
                &mut host,
                &permit,
                r#"Number(new DOMException("\ud800a", undefined, Symbol()).name === "Error")"#,
                1.0,
            );
        }
        let failure = host.in_document(&permit, |scope| {
            let source: Vec<_> = r#"try { new DOMException("\ud800a"); } catch (_) {} 42"#
                .encode_utf16()
                .collect();
            scope.evaluate(&source).map(drop)
        });
        let Err(DocumentScriptError::Script(Error::ConversionLedger(first))) = failure else {
            panic!("constructor did not report the configured conversion refusal");
        };
        let Err(DocumentScriptError::Script(Error::ConversionLedger(checkpoint))) =
            host.checkpoint()
        else {
            panic!("checkpoint did not retain the terminal conversion refusal");
        };
        assert_eq!(checkpoint, first);
        host.stop().into_session().unwrap();
    }
}

fn native_window_dom_exception_meters_typed_diagnostics() {
    // Ten UTF-16 units for AbortError and three for the typed message.
    for (units, accepted) in [(13, true), (12, false), (9, false)] {
        let (session, top) = setup();
        let mut host = prepare(session, Arc::new(cancel()))
            .with_dom_exception_limits(DomExceptionLimits::new(0, units, 0))
            .enable(top, ALLOW, hooks(&hook()))
            .unwrap();
        let permit = host.permit().unwrap();
        if accepted {
            host.in_document(&permit, |scope| {
                let error =
                    scope.dom_exception(DomExceptionName::AbortError, &[0xd800, 0, 0xdc00])?;
                let message = scope.get(
                    &error,
                    PropertyKey::String(&[109, 101, 115, 115, 97, 103, 101]),
                )?;
                assert_eq!(scope.to_utf16(&message)?, [0xd800, 0, 0xdc00]);
                Ok(())
            })
            .unwrap();
        }
        let failure = host.in_document(&permit, |scope| {
            scope
                .dom_exception(DomExceptionName::AbortError, &[0xd800, 0, 0xdc00])
                .map(drop)
        });
        let Err(DocumentScriptError::Script(Error::ConversionLedger(first))) = failure else {
            panic!("typed diagnostic did not report the configured conversion refusal");
        };
        let Err(DocumentScriptError::Script(Error::ConversionLedger(checkpoint))) =
            host.checkpoint()
        else {
            panic!("checkpoint did not retain the typed diagnostic refusal");
        };
        assert_eq!(checkpoint, first);
        host.stop().into_session().unwrap();
    }
}

fn native_window_exposes_css_escape() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());

    let kind = host
        .in_document(&permit, |scope| {
            let script: Vec<_> = "globalThis.CSS".encode_utf16().collect();
            let value = scope.evaluate(&script)?;
            scope.kind(&value)
        })
        .unwrap();
    assert_eq!(
        kind,
        ValueKind::Object,
        "native Window CSS namespace is missing"
    );

    for check in [
        "Number(Object.getPrototypeOf(CSS) === Object.prototype)",
        "Number(Object.prototype.toString.call(CSS) === '[object CSS]')",
        "Number(CSS.escape.name === 'escape' && CSS.escape.length === 1)",
        r#"Number((() => {
            function data(object, key, value, writable, enumerable, configurable) {
                const descriptor = Object.getOwnPropertyDescriptor(object, key);
                return descriptor.value === value && descriptor.writable === writable
                    && descriptor.enumerable === enumerable
                    && descriptor.configurable === configurable;
            }
            const namespace = CSS;
            const escape = CSS.escape;
            return data(globalThis, "CSS", namespace, true, false, true)
                && data(namespace, "escape", escape, true, true, true)
                && data(namespace, Symbol.toStringTag, "CSS", false, false, true)
                && data(escape, "name", "escape", false, false, true)
                && data(escape, "length", 1, false, false, true)
                && Object.getPrototypeOf(escape) === Function.prototype
                && !Object.hasOwn(escape, "prototype")
                && namespace === globalThis.CSS && escape === CSS.escape;
        })())"#,
        r#"Number((() => {
            const escape = CSS.escape;
            const sentinel = {};
            const surplus = { toString() { throw sentinel; } };
            let coercions = 0;
            const result = escape.call({}, {
                [Symbol.toPrimitive](hint) {
                    if (hint !== "string") throw sentinel;
                    coercions++;
                    return "0a\u00a0\u00d7\ud800_\ud83d\ude42\udc00\0";
                }
            }, surplus);
            let originalThrow = false;
            try { escape({ toString() { throw sentinel; } }); }
            catch (error) { originalThrow = error === sentinel; }
            let symbolTypeError = false;
            try { escape(Symbol()); }
            catch (error) { symbolTypeError = error instanceof TypeError; }
            return result === "\\30 a\u00a0\u00d7\ud800_\ud83d\ude42\udc00\ufffd"
                && coercions === 1 && originalThrow && symbolTypeError
                && escape(undefined) === "undefined" && escape(null) === "null";
        })())"#,
        r"Number(CSS.escape('a\u00a0\ud800\0') === 'a\u00a0\ud800\ufffd')",
        "Number((() => { try { CSS.escape(); } catch (error) { return error instanceof TypeError; } return false; })())",
        "Number((() => { try { Reflect.construct(function() {}, [], CSS.escape); } catch (error) { return error instanceof TypeError; } return false; })())",
    ] {
        eval(&mut host, &permit, check, 1.0);
    }

    let (original_realm, original_css) = host
        .in_document(&permit, |scope| {
            let css = scope.evaluate(&"CSS".encode_utf16().collect::<Vec<_>>())?;
            Ok((scope.realm_authority()?.identity(), scope.retain(&css)))
        })
        .unwrap();
    let session = host.stop().into_session().unwrap();
    let mut replacement = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = replacement.permit().unwrap();
    replacement
        .in_document(&permit, |scope| {
            assert_ne!(scope.realm_authority()?.identity(), original_realm);
            assert!(matches!(
                scope.import(&original_css),
                Err(Error::ForeignOwner)
            ));
            Ok(())
        })
        .unwrap();
    eval(
        &mut replacement,
        &permit,
        "Number(CSS.escape('b') === 'b')",
        1.0,
    );
}

fn native_window_css_admission() {
    fn configured(limits: CssNamespaceLimits) -> DocumentScriptHost {
        let (session, top) = setup();
        prepare(session, Arc::new(cancel()))
            .with_css_namespace_limits(limits)
            .enable(top, ALLOW, hooks(&hook()))
            .unwrap()
    }

    const ESCAPE: &str = r#"CSS.escape("1\ud800_\ud83d\ude42\udc00")"#;
    const EXACT: CssomIdentifierLimits = CssomIdentifierLimits::new(6, 18);
    let mut host = configured(CssNamespaceLimits::new(2, 12, 12, EXACT));
    let permit = host.permit().unwrap();
    for _ in 0..2 {
        eval(
            &mut host,
            &permit,
            r#"Number(CSS.escape("1\ud800_\ud83d\ude42\udc00") === "\\31 \ud800_\ud83d\ude42\udc00")"#,
            1.0,
        );
    }
    assert!(matches!(
        host.in_document(&permit, |scope| scope
            .evaluate(&ESCAPE.encode_utf16().collect::<Vec<_>>())
            .map(drop)),
        Err(DocumentScriptError::Script(Error::ConversionLedger(_)))
    ));
    host.stop().into_session().unwrap();

    for limits in [
        CssNamespaceLimits::new(1, 5, 12, EXACT),
        CssNamespaceLimits::new(1, 6, 11, EXACT),
    ] {
        let mut host = configured(limits);
        let permit = host.permit().unwrap();
        assert!(matches!(
            host.in_document(&permit, |scope| scope
                .evaluate(&ESCAPE.encode_utf16().collect::<Vec<_>>())
                .map(drop)),
            Err(DocumentScriptError::Script(Error::ConversionLedger(_)))
        ));
        host.stop().into_session().unwrap();
    }

    for (identifier, input_refused) in [
        (CssomIdentifierLimits::new(5, 17), true),
        (CssomIdentifierLimits::new(6, 17), false),
    ] {
        let mut host = configured(CssNamespaceLimits::new(1, 6, 12, identifier));
        let permit = host.permit().unwrap();
        let script = format!("try {{ {ESCAPE}; }} catch (_) {{}} 42");
        let failure = native_failure(host.in_document(&permit, |scope| {
            scope
                .evaluate(&script.encode_utf16().collect::<Vec<_>>())
                .map(drop)
        }));
        let cause = failure.cause::<CssSerializationError>().unwrap();
        assert!(cause.is_resource_limit());
        assert_eq!(cause.is_input_unit_limit(), input_refused);
        assert_eq!(cause.is_byte_limit(), !input_refused);
        let first = native_failure(host.checkpoint());
        assert!(std::ptr::eq(
            cause,
            first.cause::<CssSerializationError>().unwrap()
        ));
        host.stop().into_session().unwrap();
        assert_eq!(cause.is_input_unit_limit(), input_refused);
    }
}

fn native_window_css_hook_collision_preserves_the_original_session() {
    const CSS: &[u16] = &[67, 83, 83];
    let global_this: Vec<_> = "globalThis".encode_utf16().collect();
    let (session, top) = setup();
    let before = session.snapshot();
    let mut prepared = prepare(session, Arc::new(cancel()));
    for names in [
        DocumentScriptHooks {
            child_count: CSS,
            detach_source_node_if: &[65],
        },
        DocumentScriptHooks {
            child_count: &[65],
            detach_source_node_if: CSS,
        },
        DocumentScriptHooks {
            child_count: &global_this,
            detach_source_node_if: &[65],
        },
        DocumentScriptHooks {
            child_count: &[65],
            detach_source_node_if: &global_this,
        },
    ] {
        let failure = match prepared.enable(top, ALLOW, names) {
            Err(failure) => failure,
            Ok(mut host) => {
                let permit = host.permit().unwrap();
                let observed = host
                    .in_document(&permit, |scope| {
                        let source: Vec<_> = "typeof globalThis.CSS + '/' + typeof CSS.escape"
                            .encode_utf16()
                            .collect();
                        let value = scope.evaluate(&source)?;
                        scope.to_utf16(&value)
                    })
                    .unwrap();
                panic!(
                    "conflicting hook published the attachment, guest globalThis.CSS/CSS.escape: {}",
                    String::from_utf16(&observed).unwrap()
                );
            }
        };
        assert!(matches!(
            failure.error,
            DocumentScriptError::Script(Error::PropertyRejected)
        ));
        prepared = failure.owner;
    }
    let session = prepared.into_session().unwrap();
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .scripting(),
        HtmlScriptingStatus::Disabled,
    );
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    eval(&mut host, &permit, "Number(CSS.escape('a') === 'a')", 1.0);
    eval(
        &mut host,
        &permit,
        "Number(Object.getOwnPropertyDescriptor(globalThis, 'globalThis').configurable)",
        1.0,
    );
}

fn failed_enable_preserves_original_owner() {
    let (original, target) = setup();
    let (foreign, other_target) = setup();
    let stamp = original.snapshot().stamp();
    let cancellation = Arc::new(cancel());
    let refused =
        PreparedDocumentScriptHost::prepare(original, 0, cancellation.clone()).unwrap_err();
    assert!(matches!(
        refused.error,
        DocumentScriptNativeError::StorageLimit
    ));
    assert_eq!(refused.session.snapshot().stamp(), stamp);
    let mut prepared = prepare(refused.session, cancellation);
    let refused = prepared
        .enable(other_target, ALLOW, hooks(&hook()))
        .unwrap_err();
    assert!(matches!(
        refused.error,
        DocumentScriptError::Native(DocumentScriptNativeError::Hosted(
            DomHostedRuntimeError::TargetNotFound
        ))
    ));
    prepared = refused.owner;
    drop(foreign);

    let occupied = Process::bootstrap(EngineKind::V8).unwrap();
    let refused = prepared.enable(target, ALLOW, hooks(&hook())).unwrap_err();
    assert!(matches!(
        refused.error,
        DocumentScriptError::Script(Error::OwnerAlreadyActive)
    ));
    drop(occupied);
    prepared = refused.owner;
    for policy in [
        HtmlScriptingPolicy {
            user_allows: false,
            ..ALLOW
        },
        HtmlScriptingPolicy {
            automation_allows: false,
            ..ALLOW
        },
    ] {
        let refused = prepared.enable(target, policy, hooks(&hook())).unwrap_err();
        assert!(matches!(
            refused.error,
            DocumentScriptError::Native(DocumentScriptNativeError::Hosted(
                DomHostedRuntimeError::Dom(DomHostedError::ScriptingDisabled)
            ))
        ));
        prepared = refused.owner;
    }
    let fixed_global = "Infinity".encode_utf16().collect::<Vec<_>>();
    let refused = prepared
        .enable(target, ALLOW, hooks(&fixed_global))
        .unwrap_err();
    assert!(matches!(
        refused.error,
        DocumentScriptError::Script(Error::PropertyRejected)
    ));
    let mut original = refused.owner.into_session().unwrap();
    let no_script = original.snapshot();
    assert_eq!(no_script.stamp(), stamp);
    let status = no_script.document(target.document()).unwrap().scripting();
    assert_eq!(status, HtmlScriptingStatus::Disabled);
    assert_eq!(
        media_scripting_for_status(status),
        Some(CssMediaScripting::None)
    );
    let mut native = DomBatch::new();
    let independent = native.create_document();
    let new = original
        .dom()
        .apply(native, &cancel())
        .unwrap()
        .document(&independent)
        .unwrap();
    let snapshot = original.snapshot();
    let environment = snapshot
        .document(new)
        .unwrap()
        .metadata()
        .document_environment();
    assert_eq!(
        environment.browsing_context(),
        surgeist_html::security::nonce::HtmlBrowsingContextPresence::Absent
    );
    assert_eq!(
        snapshot.document(new).unwrap().scripting(),
        HtmlScriptingStatus::Disabled
    );
}

fn native_read_snapshot_and_explicit_queue() -> DocumentScriptPermit {
    let (session, top) = setup();
    let original = session.snapshot();
    let native_only = original
        .html_capture(
            top.document(),
            None,
            capture_limits(),
            DomHtmlCaptureLimits::new(32, 10_000),
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        captured_status(&native_only, top.document()),
        HtmlScriptingStatus::Disabled
    );
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());

    let attached = host.snapshot().unwrap();
    let observation = host.observe_scripting(&permit).unwrap();
    assert_eq!(observation.status(), HtmlScriptingStatus::Enabled);
    assert_eq!(
        media_scripting_for_status(observation.status()),
        Some(CssMediaScripting::Enabled)
    );
    assert_eq!(
        media_scripting_for_status(HtmlScriptingStatus::Unavailable),
        None
    );
    assert!(matches!(
        original.with_document_scripting(observation),
        Err(DomHtmlCaptureError::StaleScriptingObservation)
    ));
    let adapter_limits = DomHtmlCaptureLimits::new(32, 10_000);
    let unqualified = attached
        .html_capture(
            top.document(),
            None,
            capture_limits(),
            adapter_limits,
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        captured_status(&unqualified, top.document()),
        HtmlScriptingStatus::Unavailable
    );
    let capture = attached
        .with_document_scripting(observation)
        .unwrap()
        .html_capture(None, capture_limits(), adapter_limits, &cancel())
        .unwrap();
    assert_eq!(
        captured_status(&capture, top.document()),
        HtmlScriptingStatus::Enabled
    );

    let thrown = host.in_document(&permit, |scope| {
        let source = "globalThis.thrown = {}; throw thrown"
            .encode_utf16()
            .collect::<Vec<_>>();
        scope.evaluate(&source)?;
        Ok(())
    });
    let Err(DocumentScriptError::Script(Error::Exception(exception))) = thrown else {
        panic!("expected the original guest exception: {thrown:?}");
    };
    host.in_document(&permit, |scope| {
        let thrown = scope.import(exception.value())?;
        let original = scope.evaluate(&"thrown".encode_utf16().collect::<Vec<_>>())?;
        assert!(scope.strict_equals(&thrown, &original)?);
        Ok(())
    })
    .unwrap();

    eval(&mut host, &permit, "readChildCount()", 1.0);
    eval(
        &mut host,
        &permit,
        "globalThis.job = 0; Promise.resolve().then(() => { job = readChildCount(); }); job",
        0.0,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        let comment = batch.create_comment(top.document().node(), "new native child");
        batch.insert_before(&comment, top.document().node(), None);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let after = host.snapshot().unwrap();
    assert!(matches!(
        after.with_document_scripting(observation),
        Err(DomHtmlCaptureError::StaleScriptingObservation)
    ));
    eval(&mut host, &permit, "readChildCount()", 2.0);
    eval(&mut host, &permit, "job", 0.0);
    host.checkpoint().unwrap();
    eval(&mut host, &permit, "job", 2.0);

    let queued_calls = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, queued_calls.clone());
    let restored = host.stop().into_session().unwrap();
    assert_eq!(restored.snapshot().stamp(), after.stamp());
    let fresh = restored.snapshot();
    assert!(matches!(
        fresh.with_document_scripting(observation),
        Err(DomHtmlCaptureError::StaleScriptingObservation)
    ));
    let capture = fresh
        .html_capture(
            top.document(),
            None,
            capture_limits(),
            adapter_limits,
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        captured_status(&capture, top.document()),
        HtmlScriptingStatus::Disabled
    );
    assert_eq!(queued_calls.load(Ordering::Relaxed), 0);
    assert_eq!(Arc::strong_count(&queued_calls), 1);
    let recapture = attached
        .with_document_scripting(observation)
        .unwrap()
        .html_capture(None, capture_limits(), adapter_limits, &cancel())
        .unwrap();
    assert_eq!(
        captured_status(&recapture, top.document()),
        HtmlScriptingStatus::Enabled
    );
    permit
}

fn queue_observable_job(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    calls: Arc<AtomicUsize>,
) {
    host.in_document(permit, |scope| {
        let callback = Callback::new(move |_, _| {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok(Completion::Undefined)
        });
        let global = scope.global()?;
        let function = scope.function(&callback)?;
        let key = "nativeJob".encode_utf16().collect::<Vec<_>>();
        scope.set(&global, PropertyKey::String(&key), &function)?;
        let script = "Promise.resolve().then(nativeJob);"
            .encode_utf16()
            .collect::<Vec<_>>();
        scope.evaluate(&script)?;
        Ok(())
    })
    .unwrap();
}

fn destroy_native_top(
    dom: &mut DomEventDom<'_>,
    mut hosted: HostedRuntime,
    top: DomHostedTopLevel,
) {
    let cancellation = cancel();
    let queued = hosted
        .destroy_top_level(dom, top.navigable(), &cancellation)
        .unwrap();
    for _ in 0..256 {
        if matches!(
            hosted.run_next(dom, queued.traversable(), &cancellation).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == queued.operation()
        ) {
            return;
        }
    }
    panic!("the original native retirement never completed");
}

fn retire_attached(host: &mut DocumentScriptHost, top: DomHostedTopLevel) {
    host.with_native(|dom, hosted, _| destroy_native_top(dom, hosted, top))
        .unwrap();
}

fn retirement_unwind_disposes_the_revoked_guest_owner() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, calls.clone());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        host.with_native::<()>(|dom, hosted, _| {
            destroy_native_top(dom, hosted, top);
            panic!("native retirement unwind");
        })
        .unwrap();
    }));
    let original = result.expect_err("the actual host unwind must propagate");
    assert_eq!(
        original.downcast_ref::<&'static str>().copied(),
        Some("native retirement unwind")
    );
    assert_immediate_retirement(&host, &calls);
    let _session = host.stop().into_session().unwrap();
}

fn assert_immediate_retirement(host: &DocumentScriptHost, queued_calls: &Arc<AtomicUsize>) {
    let revoked = matches!(host.permit(), Err(DocumentScriptNativeError::Revoked));
    let calls = queued_calls.load(Ordering::Relaxed);
    let captures = Arc::strong_count(queued_calls);
    let replacement = Process::bootstrap(EngineKind::V8);
    let available = replacement.is_ok();
    drop(replacement);
    assert!(
        revoked && calls == 0 && captures == 1 && available,
        "retirement must return with revoked permit, abandoned queue and released process: revoked={revoked}, calls={calls}, captures={captures}, available={available}"
    );
}

fn retirement_refusals_follow_actual_canonical_publication() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let attempt = |host: &mut DocumentScriptHost, operations| {
        host.with_native(|dom, mut hosted, _| {
            let cancellation = cancel();
            let mut hosted_limits = limits().hosted;
            let canonical = hosted_limits.canonical;
            let records = canonical.records();
            hosted_limits.canonical = DomHostedLimits::new(
                DomHostedRecordLimits::new(
                    records.hosts(),
                    records.browsing_contexts(),
                    records.navigables(),
                    records.entries(),
                    records.globals(),
                    operations,
                    records.registries(),
                ),
                canonical.retained_bytes(),
                canonical.scratch_bytes(),
                canonical.work(),
            );
            hosted
                .set_limits(dom, hosted_limits, &cancellation)
                .unwrap();
            hosted.destroy_top_level(dom, top.navigable(), &cancellation)
        })
        .unwrap()
    };

    let before = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, before.clone());
    // Zero operation identities refuses the initial invocation itself.
    assert_eq!(
        attempt(&mut host, 0).unwrap_err(),
        DomHostedRuntimeError::Dom(DomHostedError::Limit {
            resource: DomHostedResource::Operations,
            limit: 0,
        })
    );
    assert!(host.permit().is_ok());
    assert_eq!(before.load(Ordering::Relaxed), 0);
    assert!(matches!(
        Process::bootstrap(EngineKind::V8),
        Err(Error::OwnerAlreadyActive)
    ));
    host.checkpoint().unwrap();
    assert_eq!(before.load(Ordering::Relaxed), 1);

    let after = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, after.clone());
    // One identity admits the invocation and its canonical retiring mark,
    // but refuses issuance of the distinct cleanup operation identity.
    assert_eq!(
        attempt(&mut host, 1).unwrap_err(),
        DomHostedRuntimeError::Dom(DomHostedError::Limit {
            resource: DomHostedResource::Operations,
            limit: 1,
        })
    );
    assert!(
        host.snapshot()
            .unwrap()
            .hosted_navigable(top.navigable())
            .unwrap()
            .is_retiring()
    );
    assert_immediate_retirement(&host, &after);
    host.with_native(|dom, mut hosted, _| {
        hosted.set_limits(dom, limits().hosted, &cancel()).unwrap();
        destroy_native_top(dom, hosted, top);
    })
    .unwrap();
    let _session = host.stop().into_session().unwrap();
}

fn real_retirement_revokes_and_abandons(foreign: &DocumentScriptPermit) {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    assert!(matches!(
        host.in_document(foreign, |_| panic!("foreign permit entered guest")),
        Err::<(), _>(DocumentScriptError::Native(
            DocumentScriptNativeError::ForeignPermit
        ))
    ));
    let permit = host.permit().unwrap();
    let queued_calls = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, queued_calls.clone());
    retire_attached(&mut host, top);
    assert_immediate_retirement(&host, &queued_calls);
    assert!(matches!(
        host.checkpoint(),
        Err(DocumentScriptError::Native(
            DocumentScriptNativeError::Hosted(
                DomHostedRuntimeError::TargetNotFound
                    | DomHostedRuntimeError::Inactive
                    | DomHostedRuntimeError::Dom(DomHostedError::StaleIdentity)
            )
        ))
    ));
    assert!(matches!(
        host.in_document(&permit, |_| panic!("revoked permit entered guest")),
        Err::<(), _>(DocumentScriptError::Native(
            DocumentScriptNativeError::Revoked
        ))
    ));
    assert_eq!(queued_calls.load(Ordering::Relaxed), 0);
    assert_eq!(Arc::strong_count(&queued_calls), 1);
    let replacement = Process::bootstrap(EngineKind::V8).unwrap();
    drop(replacement);
    let snapshot = host.snapshot().unwrap();
    assert_eq!(
        snapshot.document(top.document()).unwrap().scripting(),
        HtmlScriptingStatus::Disabled
    );
    let _session = host.stop().into_session().unwrap();
}

fn native_refusal_stays_typed_and_sticky() {
    let (session, top) = setup();
    let cancellation = Arc::new(cancel());
    let mut host = prepare(session, cancellation.clone())
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let queued_calls = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, queued_calls.clone());
    let caught = host.in_document(&permit, |scope| {
        cancellation.store(true, Ordering::Relaxed);
        let script = "try { readChildCount(); } catch (_) {} 42"
            .encode_utf16()
            .collect::<Vec<_>>();
        scope.evaluate(&script)?;
        Ok(())
    });
    let original = native_failure(caught);
    let cause = original.cause::<DocumentScriptNativeError>().unwrap();
    assert!(matches!(
        cause,
        DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Cancelled)
    ));
    cancellation.store(false, Ordering::Relaxed);
    let first = native_failure(host.checkpoint());
    assert!(std::ptr::eq(
        cause,
        first.cause::<DocumentScriptNativeError>().unwrap()
    ));
    retire_attached(&mut host, top);
    assert_immediate_retirement(&host, &queued_calls);
    let retained_first = native_failure(host.checkpoint());
    assert!(std::ptr::eq(
        cause,
        retained_first.cause::<DocumentScriptNativeError>().unwrap()
    ));
    let _session = host.stop().into_session().unwrap();
    assert!(std::ptr::eq(
        cause,
        original.cause::<DocumentScriptNativeError>().unwrap()
    ));
}

fn native_failure(result: Result<(), DocumentScriptError>) -> surgeist_bindings_js::HostFailure {
    match result {
        Err(DocumentScriptError::Script(Error::HostFailure(failure))) => failure,
        other => panic!("unexpected native refusal: {other:?}"),
    }
}

fn stopped_scripting_snapshots() -> (
    DocumentId,
    DomSnapshot,
    DomDocumentScriptingObservation,
    DomSnapshot,
) {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let observation = host.observe_scripting(&host.permit().unwrap()).unwrap();
    assert_eq!(observation.status(), HtmlScriptingStatus::Enabled);
    let historical = host.snapshot().unwrap();
    let fresh = host.stop().into_session().unwrap().snapshot();
    (top.document(), historical, observation, fresh)
}

fn stopped_host_captures_known_no_engine() {
    let (document, _, _, fresh) = stopped_scripting_snapshots();
    let capture = fresh
        .html_capture(
            document,
            None,
            capture_limits(),
            DomHtmlCaptureLimits::new(32, 10_000),
            &cancel(),
        )
        .unwrap();
    let status = captured_status(&capture, document);
    assert_eq!(status, HtmlScriptingStatus::Disabled);
    assert_eq!(
        media_scripting_for_status(status),
        Some(CssMediaScripting::None)
    );
}

fn stopped_host_rejects_the_last_live_observation() {
    let (document, historical, observation, fresh) = stopped_scripting_snapshots();
    let adapter = DomHtmlCaptureLimits::new(32, 10_000);
    let prior = historical
        .with_document_scripting(observation)
        .unwrap()
        .html_capture(None, capture_limits(), adapter, &cancel())
        .unwrap();
    assert_eq!(
        captured_status(&prior, document),
        HtmlScriptingStatus::Enabled
    );
    match fresh.with_document_scripting(observation) {
        Err(DomHtmlCaptureError::StaleScriptingObservation) => {}
        Err(error) => panic!("the old observation must be stale: {error:?}"),
        Ok(capture) => {
            let capture = capture
                .html_capture(None, capture_limits(), adapter, &cancel())
                .unwrap();
            panic!(
                "a fresh stopped snapshot accepted the prior live observation: {:?}",
                captured_status(&capture, document)
            );
        }
    }
}

fn stopped_session_reattaches_with_a_new_scripting_basis() {
    let (session, top) = setup();
    let cancellation = Arc::new(cancel());
    let mut host = prepare(session, cancellation.clone())
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let observation = host.observe_scripting(&host.permit().unwrap()).unwrap();
    let historical = host.snapshot().unwrap();
    cancellation.store(true, Ordering::Relaxed);
    let session = host.stop().into_session().unwrap();
    let stopped = session.snapshot();
    assert_eq!(historical.stamp(), stopped.stamp());
    assert_eq!(
        stopped.document(top.document()).unwrap().scripting(),
        HtmlScriptingStatus::Disabled
    );
    cancellation.store(false, Ordering::Relaxed);
    let mut replacement = prepare(session, cancellation)
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let current = replacement.snapshot().unwrap();
    assert_ne!(current.stamp(), stopped.stamp());
    assert!(matches!(
        current.with_document_scripting(observation),
        Err(DomHtmlCaptureError::StaleScriptingObservation)
    ));
    let new_observation = replacement
        .observe_scripting(&replacement.permit().unwrap())
        .unwrap();
    current.with_document_scripting(new_observation).unwrap();
    historical.with_document_scripting(observation).unwrap();
    assert_eq!(
        stopped.document(top.document()).unwrap().scripting(),
        HtmlScriptingStatus::Disabled
    );
    let _session = replacement.stop().into_session().unwrap();
}

fn enable_at_host_work(
    work: u64,
    child_count: &[u16],
    detach: &[u16],
) -> Result<DocumentScriptHost, DocumentScriptError> {
    let (mut session, top) = setup();
    let cancellation = Arc::new(cancel());
    let stamp = session.snapshot().stamp();
    let mut hosted = session.hosted_execution();
    let mut policy = limits().hosted;
    policy.work = work;
    hosted
        .set_limits(&mut session.dom(), policy, &cancellation)
        .map_err(DocumentScriptNativeError::from)?;
    let prepared = PreparedDocumentScriptHost::prepare(session, STORAGE, cancellation)
        .map_err(|failure| failure.error)?;
    match prepared.enable(
        top,
        ALLOW,
        DocumentScriptHooks {
            child_count,
            detach_source_node_if: detach,
        },
    ) {
        Ok(host) => Ok(host),
        Err(failure) => {
            let snapshot = failure.owner.into_session().unwrap().snapshot();
            assert_eq!(snapshot.stamp(), stamp);
            assert_eq!(
                snapshot.document(top.document()).unwrap().scripting(),
                HtmlScriptingStatus::Disabled
            );
            Err(failure.error)
        }
    }
}

fn hook_work_refused(result: Result<DocumentScriptHost, DocumentScriptError>) -> bool {
    match result {
        Ok(_) => false,
        Err(DocumentScriptError::Native(DocumentScriptNativeError::Hosted(
            DomHostedRuntimeError::Admission,
        ))) => true,
        Err(error) => panic!("unexpected hook setup result: {error:?}"),
    }
}

fn verify_hook_keys(mut host: DocumentScriptHost, child: &[u16], detach: &[u16]) {
    use std::fmt::Write as _;

    fn literal(units: &[u16]) -> String {
        let mut text = String::new();
        text.push('"');
        for unit in units {
            write!(text, r"\u{unit:04x}").unwrap();
        }
        text.push('"');
        text
    }

    // The exact enable allowance is spent. Raise the same native owner's work
    // before entering V8. JS escapes read the authored keys independently of
    // the bindings' PropertyKey::String path that installed them.
    host.with_native(|dom, mut hosted, _| hosted.set_limits(dom, limits().hosted, &cancel()))
        .unwrap()
        .unwrap();
    host.in_document(&host.permit().unwrap(), |scope| {
        assert_script_number(scope, &format!("globalThis[{}]()", literal(child)), 1.0)?;
        let inspect = format!("typeof globalThis[{}] === 'function'", literal(detach));
        let result = scope.evaluate(&inspect.encode_utf16().collect::<Vec<_>>())?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
}

fn connected_script_parks_the_original_source_for_live_scripting() {
    let cancellation = Arc::new(cancel());
    let (mut session, top) = setup();
    let document = top.document();
    let snapshot = session.snapshot();
    let html = snapshot.node(document.node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];

    let mut batch = DomBatch::new();
    let script = batch.create_element_ns(document, Namespace::html(), "script");
    let text = batch.create_text(document, "globalThis.scriptRan = true");
    batch.insert_before(&text, &script, None);
    let commit = session.dom().apply(batch, &cancellation).unwrap();
    let script = commit.created(&script).unwrap().unwrap();
    assert_eq!(session.snapshot().node(script).unwrap().parent(), None);

    let mut host = prepare(session, cancellation.clone())
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    assert_eq!(
        host.observe_scripting(&permit).unwrap().status(),
        HtmlScriptingStatus::Enabled
    );
    let original = host.snapshot().unwrap().stamp();
    let limits = DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 8);
    host.with_native(|dom, _, mut source| {
        let mut services = DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        };
        let progress = source.apply(
            dom,
            DomSourceEdit::insert_before(script, body, None),
            limits,
            &mut services,
            &cancellation,
        );
        assert!(
            matches!(progress, Ok(DomSourceDriveProgress::Pending { .. })),
            "connecting a nonempty Script must retain its required live scripting request: {progress:?}"
        );
        let operation = dom.pending_source_edit().unwrap().operation();
        assert_eq!(dom.snapshot().stamp(), original);
        assert_eq!(dom.snapshot().node(script).unwrap().parent(), None);

        let resumed = source.resume(dom, operation, limits, &mut services, &cancellation);
        assert!(
            matches!(resumed, Ok(DomSourceDriveProgress::Pending { .. })),
            "resuming without a live scripting answer must still request it: {resumed:?}"
        );
        assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
        assert_eq!(dom.snapshot().stamp(), original);
    })
    .unwrap();
}

fn explicit_hook_keys_require_each_authored_utf16_work_unit() {
    let mut refused = 0;
    let mut baseline = limits().hosted.work;
    let ascii = [u16::from(b'r'), u16::from(b'1')];
    let lone = [0xd800, 0];
    assert!(!hook_work_refused(enable_at_host_work(
        baseline, &ascii, &lone
    )));
    while baseline - refused > 1 {
        let probe = refused + (baseline - refused) / 2;
        if hook_work_refused(enable_at_host_work(probe, &ascii, &lone)) {
            refused = probe;
        } else {
            baseline = probe;
        }
    }
    // Equal-length independent keys control for V8 representation, including
    // NUL and isolated surrogates. Only the authored UTF-16 extent varies.
    let other_ascii = [u16::from(b'r'), u16::from(b'2')];
    let other_lone = [0, 0xdc00];
    verify_hook_keys(
        enable_at_host_work(baseline, &other_lone, &other_ascii).unwrap(),
        &other_lone,
        &other_ascii,
    );
    assert!(hook_work_refused(enable_at_host_work(
        baseline - 1,
        &other_lone,
        &other_ascii
    )));

    let child = [0xd800, 0, u16::from(b'a'), 0xdc00, u16::from(b'r')].repeat(7);
    let detach = [u16::from(b'd'), u16::from(b'e'), u16::from(b't')].repeat(11);
    let exact =
        baseline + u64::try_from(child.len() + detach.len() - ascii.len() - lone.len()).unwrap();
    verify_hook_keys(
        enable_at_host_work(exact, &child, &detach).unwrap(),
        &child,
        &detach,
    );
    assert!(
        hook_work_refused(enable_at_host_work(exact - 1, &child, &detach)),
        "one UTF-16 unit short must refuse the authored hook keys"
    );
}

fn main() {
    if let Some(scenario) = std::env::args().nth(1) {
        match scenario.as_str() {
            "native_window_exposes_dom_exception" => native_window_exposes_dom_exception(),
            "dom-exception-error-data" => {
                native_window_dom_exception_uses_error_data_and_preserves_abrupt_completion()
            }
            "dom-exception-admission" => {
                native_window_dom_exception_admission_and_hook_collision();
                native_window_dom_exception_meters_typed_diagnostics();
            }
            "css-namespace" => native_window_exposes_css_escape(),
            "css-namespace-admission" => native_window_css_admission(),
            "css-namespace-collision" => {
                native_window_css_hook_collision_preserves_the_original_session()
            }
            "conversion-revocation" => source_events::run(),
            "creation-realms" => source_events::run_creation_realms(),
            "hook-utf16-work" => explicit_hook_keys_require_each_authored_utf16_work_unit(),
            "script-live-read" => connected_script_parks_the_original_source_for_live_scripting(),
            "native-retirement" => {
                let expired = native_read_snapshot_and_explicit_queue();
                real_retirement_revokes_and_abandons(&expired);
            }
            "retirement-after-refusal" => native_refusal_stays_typed_and_sticky(),
            "retirement-unwind" => retirement_unwind_disposes_the_revoked_guest_owner(),
            "retirement-admission" => retirement_refusals_follow_actual_canonical_publication(),
            "stop-known-no-engine" => stopped_host_captures_known_no_engine(),
            "stop-stale-observation" => stopped_host_rejects_the_last_live_observation(),
            "stop-reattach" => stopped_session_reattaches_with_a_new_scripting_basis(),
            _ => panic!("unknown scripting scenario: {scenario}"),
        }
        return;
    }
    native_window_exposes_dom_exception();
    native_window_dom_exception_uses_error_data_and_preserves_abrupt_completion();
    native_window_dom_exception_admission_and_hook_collision();
    native_window_dom_exception_meters_typed_diagnostics();
    native_window_exposes_css_escape();
    native_window_css_admission();
    native_window_css_hook_collision_preserves_the_original_session();
    connected_script_parks_the_original_source_for_live_scripting();
    failed_enable_preserves_original_owner();
    explicit_hook_keys_require_each_authored_utf16_work_unit();
    let expired_permit = native_read_snapshot_and_explicit_queue();
    stopped_host_captures_known_no_engine();
    stopped_host_rejects_the_last_live_observation();
    stopped_session_reattaches_with_a_new_scripting_basis();
    real_retirement_revokes_and_abandons(&expired_permit);
    retirement_unwind_disposes_the_revoked_guest_owner();
    retirement_refusals_follow_actual_canonical_publication();
    native_refusal_stays_typed_and_sticky();
    source_events::run();
    source_events::run_creation_realms();
}
