//! The original Form reset event and canonical form-owner path.

use super::{window_length::observe, *};

pub(super) fn attached_form_reset_uses_live_post_event_owners_and_a_per_form_lock() {
    let (session, top) = setup();
    let cancellation = Arc::new(cancel());
    let (mut host, _) =
        attach_event_window_with_work(session, top, cancellation, 160_000, |_, _, _| {});
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
