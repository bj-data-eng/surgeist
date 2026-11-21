use super::{DocumentScriptError, Error, evaluate, hosted_session_with_dom_limits};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn select_and_option_form_follow_native_owner() {
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
                const selectMember = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'form');
                if (!selectMember?.get) return 'missing native Select form accessor';
                const optionMember = Object.getOwnPropertyDescriptor(HTMLOptionElement.prototype, 'form');
                for (const member of [selectMember, optionMember]) {
                    if (!member?.get || member.get.name !== 'get form' || member.get.length !== 0 ||
                        member.set !== undefined || !member.enumerable || !member.configurable)
                        return 'readonly form descriptors';
                }
                if (Object.keys(HTMLSelectElement.prototype).join() !==
                        'disabled,form,multiple,name,required,size,type,selectedIndex,value' ||
                    Object.keys(HTMLOptionElement.prototype).join() !==
                        'disabled,form,label,defaultSelected,selected,value,text,index')
                    return 'form declaration order';
                const root = document.body;
                const outer = root.appendChild(document.createElement('form'));
                const named = root.appendChild(document.createElement('form'));
                named.setAttribute('id', 'form-\ud800');
                const select = document.createElement('select');
                const option = select.appendChild(document.createElement('option'));
                if (select.form !== null || option.form !== null) return 'detached owner';
                outer.appendChild(select);
                if (select.form !== outer || option.form !== outer || !(select.form instanceof HTMLFormElement))
                    return 'ancestor Form identity';
                select.setAttribute('form', 'form-\ud800');
                if (select.form !== named || option.form !== named) return 'explicit exact UTF16 owner';
                named.setAttribute('id', 'renamed');
                if (select.form !== null || option.form !== null) return 'missing explicit owner';
                select.removeAttribute('form');
                if (select.form !== outer || option.form !== outer) return 'restored ancestor owner';
                const barrier = select.appendChild(document.createElement('hr'));
                const blocked = barrier.appendChild(document.createElement('option'));
                if (blocked.form !== null) return 'Option ancestor barrier';
                for (const [getter, wrong] of [[selectMember.get, option], [optionMember.get, select]]) {
                    try { getter.call(wrong); return 'wrong native brand'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'wrong brand refusal'; }
                }

                const observed = [];
                let duringConstruction;
                class ObservedSelect extends HTMLSelectElement {
                    constructor() {
                        super();
                        duringConstruction = [this.form, select.form, option.form];
                    }
                    static observedAttributes = ['form'];
                    attributeChangedCallback() {
                        observed.push([this.form, this.firstChild.form]);
                    }
                }
                customElements.define('x-observed-select-form', ObservedSelect, {extends: 'select'});
                const watching = outer.appendChild(document.createElement('select', {is: 'x-observed-select-form'}));
                if (duringConstruction?.[0] !== null || duringConstruction?.[1] !== outer ||
                    duringConstruction?.[2] !== outer) return 'published associations during construction';
                watching.appendChild(document.createElement('option'));
                watching.setAttribute('form', 'renamed');
                if (observed.length !== 1 || observed[0][0] !== named || observed[0][1] !== named)
                    return 'published association inside the original callback';
                const destination = new Document();
                if (destination.adoptNode(select) !== select || option.ownerDocument !== destination ||
                    select.form !== null || option.form !== null) return 'adopted detached controls';
                const adoptedForm = destination.createElementNS('http://www.w3.org/1999/xhtml', 'form');
                destination.appendChild(adoptedForm);
                adoptedForm.appendChild(select);
                if (select.form !== adoptedForm || option.form !== adoptedForm)
                    return 'canonical owner after adoption';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap_or_else(|error| {
        if let DocumentScriptError::Script(Error::HostFailure(failure)) = &error {
            panic!("Select and Option form host refusal: {:?}", std::error::Error::source(failure));
        }
        panic!("Select and Option form evaluation: {error:?}");
    });
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}
