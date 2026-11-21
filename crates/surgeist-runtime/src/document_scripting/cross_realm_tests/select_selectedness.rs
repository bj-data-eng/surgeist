use super::{evaluate, hosted_session_with_dom_limits};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn selectedness_uses_live_native_option_order() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits
            .with_select_processing(8, 4_000_000, 100_000)
            .unwrap()
    });
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const select = document.createElement('select');
                const first = document.createElement('option');
                if (!(select instanceof HTMLSelectElement) || !(first instanceof HTMLOptionElement))
                    return 'canonical control fixture';
                const index = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'selectedIndex');
                if (!index) return 'missing native selectedIndex accessor';
                if (index.get.length !== 0 || index.set.length !== 1 || !index.enumerable || !index.configurable)
                    return 'selectedIndex descriptor';
                if (select.selectedIndex !== -1 || first.index !== 0 || first.selected || first.defaultSelected)
                    return 'initial native selectedness';
                first.defaultSelected = true;
                if (!first.selected || !first.hasAttribute('selected')) return 'clean default selectedness';
                first.selected = false;
                first.defaultSelected = false;
                first.defaultSelected = true;
                if (first.selected || !first.defaultSelected) return 'dirty selectedness independent of reflection';

                select.multiple = true;
                const group = select.appendChild(document.createElement('optgroup'));
                group.appendChild(first);
                const second = select.appendChild(document.createElement('option'));
                const barrier = select.appendChild(document.createElement('hr'));
                const excluded = barrier.appendChild(document.createElement('option'));
                if (!select.hasAttribute('multiple') || first.index !== 0 || second.index !== 1 ||
                    excluded.index !== 0 || select.selectedIndex !== -1) return 'canonical option list and barriers';
                second.selected = true;
                first.selected = true;
                if (!first.selected || !second.selected || select.selectedIndex !== 0)
                    return 'first of two selected options';

                select.selectedIndex = 4294967297;
                if (first.selected || !second.selected || select.selectedIndex !== 1)
                    return 'long conversion and native exclusive selection';
                select.selectedIndex = {valueOf() {select.insertBefore(second, group); return 1;}};
                if (second.index !== 0 || first.index !== 1 || !first.selected || second.selected ||
                    select.selectedIndex !== 1) return 'coercion observes reordered option list';
                select.selectedIndex = 999;
                if (first.selected || second.selected || select.selectedIndex !== -1)
                    return 'out-of-range clears selectedness';
                let coerced = false;
                try { index.set.call({}, {valueOf() {coerced = true; return 0;}}); return 'forged receiver'; }
                catch (error) { if (!(error instanceof TypeError) || coerced) return 'receiver before coercion'; }
                group.removeChild(first);
                return first.index === 0 && first.defaultSelected ? 'ok' : 'detached index and reflection';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
