use super::{evaluate, hosted_session};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::{sync::Arc, sync::atomic::AtomicBool};

pub(super) fn table_ranged_attributes_read_from_and_write_to_the_native_elements() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const td = document.createElement('td');
                const th = document.createElement('th');
                const col = document.createElement('col');
                const group = document.createElement('colgroup');
                if (!(td instanceof HTMLTableCellElement) || !(th instanceof HTMLTableCellElement) ||
                    !(col instanceof HTMLTableColElement) || !(group instanceof HTMLTableColElement))
                    return 'native table fixture';
                const span = Object.getOwnPropertyDescriptor(HTMLTableCellElement.prototype, 'colSpan');
                if (typeof span?.get !== 'function' || typeof span?.set !== 'function')
                    return 'table cell missing colSpan accessor';
                if (td.colSpan !== 1 || th.rowSpan !== 1 || col.span !== 1 || group.span !== 1)
                    return 'range defaults';
                td.colSpan = 1001;
                th.rowSpan = 0;
                col.span = 0;
                group.span = -1;
                if (td.colSpan !== 1000 || td.getAttribute('colspan') !== '1001' ||
                    th.rowSpan !== 0 || th.getAttribute('rowspan') !== '0' ||
                    col.span !== 1 || col.getAttribute('span') !== '0' ||
                    group.span !== 1 || group.getAttribute('span') !== '1')
                    return 'getter clamp and setter serialization';
                const row = document.createElement('tr');
                if (td.cellIndex !== -1) return 'detached cell index';
                row.appendChild(td);
                row.appendChild(document.createElement('div'));
                row.appendChild(th);
                return td.cellIndex === 0 && th.cellIndex === 1 ? 'ok' : 'native row cell order';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
