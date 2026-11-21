//! Attached form-associated elements deliver the owning native transition reactions.

use super::*;
use surgeist_html::values::HtmlValueInterpreter;
use surgeist_html_semantics::{
    HtmlFormOwnerOrigin, HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticLimits,
    HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticValue,
};

pub(super) fn attached_form_associated_custom_elements_deliver_native_reactions() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const registry = customElements;
        const name = 'x-attached-face';
        const add = (parent, local) => parent.appendChild(document.createElement(local));
        const form = add(document.body, 'form');
        const alternative = add(document.body, 'form');
        alternative.setAttribute('id', 'alternative');
        const fieldset = add(form, 'fieldset');
        fieldset.disabled = true;
        const original = add(fieldset, name);
        original.setAttribute('data-second', 'two');
        original.setAttribute('data-first', 'one');
        const calls = [];
        let flagReads = 0;
        let reenter = false;
        class Face extends HTMLElement {
            static get formAssociated() { flagReads++; return true; }
            static get observedAttributes() { return ['data-first', 'data-second', 'data-reentry']; }
            constructor() {
                super();
                calls.push(['constructor', this, this.isConnected]);
            }
            attributeChangedCallback(name, old, value, namespace) {
                calls.push(['attribute', this, name, old, value, namespace]);
            }
            connectedCallback() { calls.push(['connected', this, this.isConnected]); }
            formAssociatedCallback(owner) {
                calls.push(['owner', this, owner, arguments.length]);
                if (reenter && this === original && owner === alternative) {
                    reenter = false;
                    this.setAttribute('data-reentry', 'nested');
                    calls.push(['resumed', this]);
                }
            }
            formDisabledCallback(disabled) {
                calls.push(['disabled', this, disabled, arguments.length]);
            }
        }
        try {
            registry.define(name, Face);
        } catch (error) {
            if (error instanceof DOMException && error.name === 'NotSupportedError' &&
                registry.get(name) === undefined && flagReads === 1 && calls.length === 0) {
                return 'CustomElementRegistry.define refused a formAssociated definition';
            }
            throw error;
        }
        const same = expected => calls.length === expected.length && calls.every((entry, i) =>
            entry.length === expected[i].length && entry.every((value, j) => value === expected[i][j]));
        if (registry.get(name) !== Face || flagReads !== 1 ||
            Object.getPrototypeOf(original) !== Face.prototype || !same([
                ['constructor', original, true],
                ['attribute', original, 'data-second', null, 'two', null],
                ['attribute', original, 'data-first', null, 'one', null],
                ['connected', original, true],
                ['owner', original, form, 1],
                ['disabled', original, true, 1],
            ])) return 'connected FACE upgrade callbacks in native order on the original receiver';

        calls.length = 0;
        const detached = document.createElement(name);
        if (detached === original || !(detached instanceof Face) || detached.parentNode !== null ||
            detached.ownerDocument !== document || !same([['constructor', detached, false]])) {
            return 'detached factory has no owner or disabled callback';
        }
        globalThis.faceProbe = {
            form, alternative, fieldset, original, calls, same,
            armReentry() { reenter = true; },
        };
        return 'ok';
    })()"#,
    );

    let initial = host.snapshot().unwrap();
    let html = initial
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = initial.node(html).unwrap().children()[1];
    let [form, alternative]: [NodeId; 2] =
        initial.node(body).unwrap().children().try_into().unwrap();
    let [fieldset]: [NodeId; 1] = initial.node(form).unwrap().children().try_into().unwrap();
    let [original]: [NodeId; 1] = initial
        .node(fieldset)
        .unwrap()
        .children()
        .try_into()
        .unwrap();
    let detached = initial
        .nodes()
        .find(|&id| {
            let node = initial.node(id).unwrap();
            node.parent().is_none()
                && node.as_element().is_some_and(|element| {
                    element.name().local_name() == &DocumentString::from("x-attached-face")
                })
        })
        .unwrap();
    assert_face_state(&initial, top.document(), original, Some(form), true);
    assert_face_state(&initial, top.document(), detached, None, false);

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const name = 'x-detached-form-upgrade';
        const owner = document.createElement('form');
        const candidate = owner.appendChild(document.createElement(name));
        const called = [];
        class DetachedFace extends HTMLElement {
            static formAssociated = true;
            formAssociatedCallback(form) { called.push([this, form]); }
        }
        customElements.define(name, DetachedFace);
        if (called.length !== 0 || candidate instanceof DetachedFace) {
            return 'detached candidate waits for its explicit upgrade';
        }
        customElements.upgrade(owner);
        return candidate instanceof DetachedFace && !candidate.isConnected && called.length === 1 &&
            called[0][0] === candidate && called[0][1] === owner ? 'ok' :
            'detached FACE upgrade delivers the Form to its otherwise empty callback queue';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {form, alternative, fieldset, original, calls, same, armReentry} = faceProbe;
        const observe = (action, expected) => {
            calls.length = 0;
            action();
            return same(expected);
        };
        armReentry();
        if (!observe(() => original.setAttribute('form', 'alternative'), [
            ['owner', original, alternative, 1],
            ['attribute', original, 'data-reentry', null, 'nested', null],
            ['resumed', original],
        ]) || original.getAttribute('data-reentry') !== 'nested') {
            return 'form reassociation delivers the current Form and admits a nested authored edit';
        }
        if (!observe(() => alternative.setAttribute('id', 'different'), [['owner', original, null, 1]]) ||
            !observe(() => original.removeAttribute('form'), [['owner', original, form, 1]])) {
            return 'ID mutation loses the owner and removing form restores the nearest Form';
        }
        if (!observe(() => { fieldset.disabled = false; }, [['disabled', original, false, 1]]) ||
            !observe(() => original.setAttribute('disabled', ''), [['disabled', original, true, 1]]) ||
            !observe(() => { fieldset.disabled = true; }, []) ||
            !observe(() => original.removeAttribute('disabled'), []) ||
            !observe(() => { fieldset.disabled = false; }, [['disabled', original, false, 1]])) {
            return 'owner emits only effective disabled transitions for own and inherited changes';
        }
        return 'ok';
    })()"#,
    );
    let transitioned = host.snapshot().unwrap();
    assert_eq!(
        transitioned
            .node(alternative)
            .unwrap()
            .attribute(&"id".into()),
        Some(&"different".into())
    );
    assert_eq!(
        transitioned
            .node(original)
            .unwrap()
            .attribute(&"data-reentry".into()),
        Some(&"nested".into())
    );
    assert_face_state(&transitioned, top.document(), original, Some(form), false);
    assert_face_state(&transitioned, top.document(), detached, None, false);

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {fieldset} = faceProbe;
        const name = 'x-throwing-face';
        fieldset.disabled = true;
        const failed = fieldset.appendChild(document.createElement(name));
        failed.setAttribute('data-start', 'before upgrade');
        const calls = [];
        const failure = new Error('original FACE constructor throw');
        globalThis.faceConstructorFailure = failure;
        class Failing extends HTMLElement {
            static formAssociated = true;
            static observedAttributes = ['data-start'];
            constructor() { super(); calls.push(['constructor', this]); throw failure; }
            attributeChangedCallback() { calls.push(['attribute', this]); }
            connectedCallback() { calls.push(['connected', this]); }
            formAssociatedCallback() { calls.push(['owner', this]); }
            formDisabledCallback() { calls.push(['disabled', this]); }
        }
        customElements.define(name, Failing);
        return customElements.get(name) === Failing && calls.length === 1 &&
            calls[0][0] === 'constructor' && calls[0][1] === failed ? 'ok' :
            'throwing FACE constructor clears the original receiver callback queue';
    })()"#,
    );
    let failed = host.snapshot().unwrap();
    let [same, rejected]: [NodeId; 2] = failed
        .node(fieldset)
        .unwrap()
        .children()
        .try_into()
        .unwrap();
    assert_eq!(same, original);
    assert_eq!(
        failed
            .node(rejected)
            .unwrap()
            .as_element()
            .unwrap()
            .custom_element_definition(),
        surgeist_dom::custom_element::DomElementDefinition::Null
    );
    let report = host
        .take_guest_constructor_exception(&permit)
        .unwrap()
        .unwrap();
    host.in_document(&permit, |scope| {
        let original =
            scope.evaluate(&"faceConstructorFailure".encode_utf16().collect::<Vec<_>>())?;
        let reported = scope.import(report.exception.value())?;
        assert!(scope.strict_equals(&original, &reported)?);
        Ok(())
    })
    .unwrap();
    assert!(
        host.take_guest_constructor_exception(&permit)
            .unwrap()
            .is_none()
    );
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {form, fieldset} = faceProbe;
        const name = 'x-face-callback-throws';
        const original = fieldset.appendChild(document.createElement(name));
        const calls = [];
        const failure = {};
        globalThis.faceCallbackFailure = failure;
        class ThrowingFormCallback extends HTMLElement {
            static formAssociated = true;
            formAssociatedCallback(owner) {
                calls.push(['owner', this, owner]);
                throw failure;
            }
            formDisabledCallback(disabled) {
                calls.push(['disabled', this, disabled]);
            }
        }
        customElements.define(name, ThrowingFormCallback);
        return calls.length === 2 && calls[0][0] === 'owner' &&
            calls[0][1] === original && calls[0][2] === form &&
            calls[1][0] === 'disabled' && calls[1][1] === original && calls[1][2] === true ?
            'ok' : 'throwing Form association callback must leave the following disabled reaction live';
    })()"#,
    );
    let reported = host
        .take_guest_custom_element_callback_exception(&permit)
        .unwrap()
        .unwrap();
    assert_eq!(
        reported.callback,
        surgeist_runtime::document_scripting::DocumentGuestCustomElementCallback::FormAssociated
    );
    let with_callback = host.snapshot().unwrap();
    let receiver = with_callback.node(fieldset).unwrap().children()[2];
    assert_eq!(reported.receiver, receiver);
    assert_face_state(&with_callback, top.document(), receiver, Some(form), true);
    host.in_document(&permit, |scope| {
        let original = scope.evaluate(&"faceCallbackFailure".encode_utf16().collect::<Vec<_>>())?;
        let report = scope.import(reported.exception.value())?;
        assert!(scope.strict_equals(&original, &report)?);
        Ok(())
    })
    .unwrap();
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
}

fn assert_face_state(
    snapshot: &DomSnapshot,
    document: DocumentId,
    node: NodeId,
    owner: Option<NodeId>,
    disabled: bool,
) {
    let captured = snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(64, 4096, 128, 32768, 64, 16, 16, 1_000_000),
            DomHtmlCaptureLimits::new(64, 1_000_000),
            &cancel(),
        )
        .unwrap();
    let subject = captured.subject(node).unwrap();
    let report = captured
        .query(
            &[
                HtmlSemanticRequest::new(subject.clone(), HtmlSemanticQuery::FormOwner),
                HtmlSemanticRequest::new(subject, HtmlSemanticQuery::Disabledness),
            ],
            HtmlValueInterpreter::new(4096),
            HtmlSemanticLimits::new(1_000_000, 4, 128, 64, 1_000_000, 64),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    let HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(association)) =
        report.results()[0].fact()
    else {
        panic!(
            "expected the FACE's published native Form owner, got {:?}",
            report.results()[0].fact()
        );
    };
    assert_eq!(
        association.origin(),
        HtmlFormOwnerOrigin::EstablishedCurrent
    );
    assert_eq!(
        association.owner().map(|id| captured.node(id).unwrap()),
        owner
    );
    assert_eq!(
        report.results()[1].fact(),
        &HtmlSemanticFact::Disabledness(HtmlSemanticValue::Known(disabled))
    );
}

fn observe(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit, script: &str) {
    let result = host
        .in_document(permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
}
