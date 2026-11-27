//! The original Form reset event and canonical form-owner path.

use super::{
    standard_window::{guest_limits, new_registry},
    window_length::observe,
    *,
};

fn reset_session() -> (DomSession, DomHostedTopLevel) {
    let mut policy = limits();
    policy.source_events.jobs = 8;
    let mut dom = native_dom();
    dom.set_limits(
        dom.limits()
            .with_select_processing(32, 1_000_000, 1_000_000)
            .unwrap(),
    )
    .unwrap();
    let mut session = DomSession::new(dom, policy).unwrap();
    let (_, top) = create(&mut session);
    (session, top)
}

fn attach_reset_guest(
    mut session: DomSession,
    top: DomHostedTopLevel,
    cancel: Arc<AtomicBool>,
) -> DocumentScriptHost {
    let mut registry = new_registry();
    registry.bind(&mut session.dom(), &cancel).unwrap();
    let mut registry = Some(registry);
    let mut limits = guest_limits();
    limits.storage_bytes = 6_000_000;
    limits.work = 200_000;
    limits.source_edits = DomSourceLimits::new(32_000_000, 1_000_000, 1_000_000, 32, 512, 512, 16);
    limits.dispatch.path_items = 16;
    PreparedDocumentScriptHost::prepare(session, 8_000_000, cancel)
        .unwrap()
        .enable_with_guest_event_registry(top, ALLOW, None, &mut registry, limits)
        .unwrap()
}

pub(super) fn attached_form_reset_uses_live_post_event_owners_and_a_per_form_lock() {
    let (session, top) = reset_session();
    let cancellation = Arc::new(cancel());
    let mut host = attach_reset_guest(session, top, cancellation);
    let permit = host.permit().unwrap();
    let observed = observe(
        &mut host,
        &permit,
        r#"(() => {
            const proto = HTMLFormElement.prototype;
            const descriptor = Object.getOwnPropertyDescriptor(proto, 'reset');
            if (typeof descriptor?.value !== 'function') return 'missing HTMLFormElement.prototype.reset';
            const reset = descriptor.value;
            if (!descriptor.configurable || !descriptor.writable || !descriptor.enumerable ||
                reset.length !== 0 || reset.name !== 'reset') return 'reset method descriptor';
            try {
                reset.call(Object.create(proto));
                return 'reset accepted a forged Form';
            } catch (error) {
                if (!(error instanceof TypeError)) return 'wrong Form brand failure';
            }

            const attach = tag => document.body.appendChild(document.createElement(tag));
            const a = attach('form');
            const b = attach('form');
            a.setAttributeNS(null, 'id', 'original-reset-form');
            b.setAttributeNS(null, 'id', 'other-reset-form');
            const inside = a.appendChild(document.createElement('input'));
            const outside = attach('input');
            const other = b.appendChild(document.createElement('input'));
            outside.setAttributeNS(null, 'form', 'original-reset-form');
            for (const control of [inside, outside, other]) {
                control.defaultValue = 'default';
                control.value = 'dirty';
            }

            let phase = 'cancel';
            const events = [];
            a.addEventListener('reset', event => {
                events.push([event.target, event.currentTarget, event.isTrusted, event.bubbles,
                    event.cancelable, event.composed, inside.value, outside.value, other.value]);
                if (events.length > 2) return;
                // Same Form is already locked. A different Form can independently reset.
                reset.call(a);
                if (phase === 'cancel') {
                    event.preventDefault();
                    return;
                }
                reset.call(b);
                b.appendChild(inside);
                other.setAttributeNS(null, 'form', 'original-reset-form');
                other.value = 'post-event';
            });
            let bubbled = 0;
            document.body.addEventListener('reset', event => { if (event.target === a) bubbled++; });
            if (reset.call(a) !== undefined ||
                [inside, outside, other].some(control => control.value !== 'dirty'))
                return 'canceled reset mutated a control';
            phase = 'deliver';
            if (reset.call(a) !== undefined) return 'reset return value';
            if (events.length !== 2 || bubbled !== 2 ||
                events.some(row => row[0] !== a || row[1] !== a ||
                    !row[2] || !row[3] || !row[4] || row[5] ||
                    row.slice(6).join() !== 'dirty,dirty,dirty'))
                return 'reset source flags, event order or reentry';
            if (inside.form !== b || inside.value !== 'dirty' ||
                outside.form !== a || outside.value !== 'default' ||
                other.form !== a || other.value !== 'default')
                return 'reset must consume latest actual Form owners after listeners';
            return 'ok';
        })()"#,
    );
    assert_eq!(observed, "ok");
}
pub(super) fn attached_form_reset_restores_all_native_families_before_face_callbacks() {
    use surgeist_dom::file::{DomFile, DomFileList};

    let (mut session, top) = reset_session();
    let cancel = Arc::new(cancel());
    let input = {
        let snapshot = session.snapshot();
        let root = snapshot
            .document(top.document())
            .unwrap()
            .document_element()
            .unwrap();
        let body = *snapshot.node(root).unwrap().children().last().unwrap();
        let mut batch = DomBatch::new();
        let form = batch.create_element_ns(top.document(), Namespace::html(), "form");
        batch.set_attribute(&form, "id", "reset-families");
        let file = batch.create_element_ns(top.document(), Namespace::html(), "input");
        batch.set_attribute(&file, "id", "reset-file");
        batch.set_attribute(&file, "type", "file");
        batch.insert_before(&file, &form, None);
        batch.insert_before(&form, body, None);
        let commit = session.dom().apply(batch, &cancel).unwrap();
        commit.created(&file).unwrap().unwrap()
    };
    let file = DomFile::from_owned_parts("selected.txt".into(), "".into(), vec![42]).unwrap();
    let list = DomFileList::from_files(vec![file]).unwrap();
    let mut execution = session.source_execution();
    let outcome = execution
        .apply(
            &mut session.dom(),
            DomSourceEdit::assign_input_files(input, Some(list)),
            DomSourceLimits::new(16_000_000, 200_000, 100_000, 32, 512, 512, 2),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap();
    assert!(matches!(outcome, DomSourceDriveProgress::Complete(_)));

    let mut host = attach_reset_guest(session, top, cancel);
    let permit = host.permit().unwrap();
    let observed = observe(
        &mut host,
        &permit,
        r#"(() => {
            const form = document.getElementById('reset-families');
            const file = document.getElementById('reset-file');
            const add = (parent, name) => parent.appendChild(document.createElement(name));
            const text = add(form, 'textarea');
            text.defaultValue = 'text default';
            text.value = 'text dirty';
            const output = add(form, 'output');
            output.defaultValue = 'output default';
            output.value = 'output dirty';
            const nested = add(output, 'input');
            nested.defaultValue = 'nested default';
            nested.value = 'nested dirty';
            const select = add(form, 'select');
            add(select, 'option').defaultSelected = true;
            add(select, 'option');
            select.selectedIndex = 1;
            let userEvents = 0;
            for (const name of ['input', 'change']) {
                form.addEventListener(name, () => userEvents++);
            }
            let nestedEvents = 0;
            const received = [];
            class ResetControl extends HTMLElement {
                static formAssociated = true;
                formResetCallback(...args) {
                    received.push([this, args.length, file.value, text.value, output.value, select.selectedIndex]);
                    if (this === first && nestedEvents === 0) {
                        form.addEventListener('reset', event => {
                            nestedEvents++;
                            event.preventDefault();
                        }, {once: true});
                        form.reset();
                    }
                }
            }
            customElements.define('x-form-reset-probe', ResetControl);
            const second = document.createElement('x-form-reset-probe');
            const first = document.createElement('x-form-reset-probe');
            form.appendChild(first);
            form.appendChild(second);
            if (file.value !== String.raw`C:\fakepath\selected.txt` ||
                text.value !== 'text dirty' || output.value !== 'output dirty' ||
                select.selectedIndex !== 1) return 'reset preconditions';
            form.reset();
            if (nested.parentNode !== null || nested.value !== 'nested default')
                return 'reset lost a captured member removed by an earlier control';
            if (received.length !== 2 || received[0][0] !== first || received[1][0] !== second)
                return 'live FACE reset order';
            if (received.some(row => row[1] !== 0 || row[2] !== '' ||
                row[3] !== 'text default' || row[4] !== 'output default' || row[5] !== 0))
                return 'FACE must observe every completed native reset and no callback arguments';
            if (nestedEvents !== 1 || userEvents !== 0)
                return 'FACE must run after unlock and native reset must be silent';
            return 'ok';
        })()"#,
    );
    assert_eq!(observed, "ok");
}
