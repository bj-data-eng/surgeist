use super::{evaluate, hosted_session_with_dom_limits};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn select_metadata_reflects_current_source_and_numeric_rules() {
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
                const option = document.createElement('option');
                const size = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'size');
                if (!size) return 'missing native Select size accessor';
                const type = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'type');
                if (!type?.get || type.set !== undefined || !type.enumerable || !type.configurable ||
                    size.get.length !== 0 || size.set.length !== 1 || !size.enumerable || !size.configurable)
                    return 'metadata descriptors';
                const selectMembers = 'disabled,multiple,name,required,size,type,selectedIndex,value'.split(',');
                const optionMembers = 'disabled,label,defaultSelected,selected,value,text,index'.split(',');
                if (Object.keys(HTMLSelectElement.prototype).filter(name => selectMembers.includes(name)).join() !==
                        selectMembers.join() ||
                    Object.keys(HTMLOptionElement.prototype).filter(name => optionMembers.includes(name)).join() !==
                        optionMembers.join())
                    return 'prototype declaration order';
                if (select.disabled || select.required || option.disabled || select.name !== '' ||
                    select.size !== 0 || select.type !== 'select-one') return 'initial metadata';
                select.setAttributeNS('urn:foreign', 'p:disabled', 'foreign');
                select.setAttributeNS('urn:foreign', 'p:multiple', 'foreign');
                if (select.disabled || select.type !== 'select-one') return 'own null-namespace metadata';
                select.multiple = true;
                if (select.type !== 'select-multiple') return 'multiple type';
                select.multiple = false;
                if (select.type !== 'select-one') return 'restored single type';
                select.disabled = {valueOf() {throw 'Boolean must not coerce objects';}};
                select.required = true;
                if (!select.disabled || !select.required || select.getAttributeNS(null, 'disabled') !== '' ||
                    select.getAttributeNS(null, 'required') !== '') return 'boolean Source reflection';
                select.disabled = false;
                select.required = false;
                if (select.hasAttributeNS(null, 'disabled') || select.hasAttributeNS(null, 'required'))
                    return 'boolean removal';
                const rawName = ' name\ud800\0\udc00 ';
                select.name = rawName;
                if (select.name !== rawName || select.getAttributeNS(null, 'name') !== rawName)
                    return 'exact UTF-16 name';
                const group = document.createElement('optgroup');
                group.disabled = true;
                group.appendChild(option);
                if (option.disabled) return 'Option own disabled presence';
                option.disabled = true;
                if (!option.disabled || option.getAttributeNS(null, 'disabled') !== '') return 'Option disabled';
                option.disabled = false;
                if (option.disabled || option.hasAttributeNS(null, 'disabled')) return 'Option disabled removal';

                for (const [raw, expected] of [
                    [' \t+7tail', 7], ['-1', 0], ['-0', 0], ['2147483647', 2147483647],
                    ['2147483648', 0], ['999999999999999999999', 0], ['word', 0],
                ]) {
                    select.setAttributeNS(null, 'size', raw);
                    if (select.size !== expected || select.getAttributeNS(null, 'size') !== raw)
                        return 'bounded numeric getter ' + raw;
                }
                for (const [input, expected] of [
                    [-1, 0], [2147483647, 2147483647], [2147483648, 0],
                    [4294967298, 2], [NaN, 0], [undefined, 0],
                ]) {
                    select.size = input;
                    if (select.size !== expected || select.getAttributeNS(null, 'size') !== String(expected))
                        return 'numeric conversion and shortest decimal';
                }
                let coerced = false;
                try { size.set.call(option, {valueOf() {coerced = true; return 1;}}); return 'wrong receiver'; }
                catch (error) { if (!(error instanceof TypeError) || coerced) return 'brand before coercion'; }
                const reset = document.createElement('select');
                reset.setAttributeNS(null, 'size', '2147483648');
                const first = reset.appendChild(document.createElement('option'));
                const second = reset.appendChild(document.createElement('option'));
                if (reset.size !== 0 || reset.selectedIndex !== -1)
                    return 'reflected size is independent of native display size';
                reset.size = {valueOf() {reset.insertBefore(second, first); return 1;}};
                if (reset.size !== 1 || reset.selectedIndex !== 0 || first.selected || !second.selected)
                    return 'original size hook after conversion reentry';
                reset.selectedIndex = -1;
                second.disabled = true;
                if (reset.selectedIndex !== 1 || !first.selected || second.selected)
                    return 'original disabledness hook';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}
