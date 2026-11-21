use super::{DocumentScriptError, Error, evaluate, hosted_session_with_dom_limits};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_html_semantics::{HtmlCaptureLimits, HtmlSemanticLimits};

pub(super) fn select_and_option_strings_follow_canonical_text_and_value() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits
            .with_select_processing(8, 4_000_000, 100_000)
            .unwrap()
            .with_select_value_queries(
                HtmlCaptureLimits::new(512, 4096, 512, 32768, 512, 0, 0, 16_777_216),
                HtmlSemanticLimits::new(100_000, 4, 512, 0, 4_000_000, 512),
            )
    });
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const select = document.createElement('select');
                const value = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value');
                if (!value) return 'missing native Select value accessor';
                for (const name of ['value', 'label', 'text']) {
                    const optionProperty = Object.getOwnPropertyDescriptor(HTMLOptionElement.prototype, name);
                    if (!optionProperty?.get || !optionProperty?.set ||
                        !optionProperty.enumerable || !optionProperty.configurable)
                        return 'Option accessor ' + name;
                }
                if (value.get.length !== 0 || value.set.length !== 1 ||
                    !value.enumerable || !value.configurable || select.value !== '')
                    return 'Select descriptor and empty value';
                const selectMembers = ['multiple', 'selectedIndex', 'value'];
                const optionMembers = ['label', 'defaultSelected', 'selected', 'value', 'text', 'index'];
                if (Object.keys(HTMLSelectElement.prototype).filter(name => selectMembers.includes(name)).join() !==
                        selectMembers.join() ||
                    Object.keys(HTMLOptionElement.prototype).filter(name => optionMembers.includes(name)).join() !==
                        optionMembers.join())
                    return 'supported declaration order';
                select.multiple = true;
                const first = document.createElement('option');
                const text = 'A\u00a0\ud800 \udc00';
                first.appendChild(document.createTextNode('   A\u00a0\ud800\t\n'));
                const script = document.createElement('script');
                script.textContent = ' excluded ';
                first.appendChild(script);
                const foreignScript = document.createElementNS('urn:foreign', 'script');
                foreignScript.textContent = '\udc00  ';
                first.appendChild(foreignScript);
                first.setAttributeNS('urn:foreign', 'p:value', 'foreign value');
                first.setAttributeNS('urn:foreign', 'p:label', 'foreign label');
                if (first.value !== text || first.label !== text || first.text !== text)
                    return 'HTML-aware text, namespace and exact UTF-16';
                first.label = '';
                first.value = '';
                if (first.label !== '' || first.value !== '' || first.text !== text ||
                    first.getAttributeNS(null, 'label') !== '' || first.getAttributeNS(null, 'value') !== '')
                    return 'present empty native attributes';
                first.removeAttribute('value');
                first.removeAttribute('label');
                const second = document.createElement('option');
                second.value = text;
                select.appendChild(first);
                select.appendChild(second);
                select.value = text;
                if (select.selectedIndex !== 0 || select.value !== text || !first.selected || second.selected)
                    return 'first matching native Option';
                select.value = {toString() {select.insertBefore(second, first); return text;}};
                if (select.selectedIndex !== 0 || select.value !== text || first.selected || !second.selected)
                    return 'conversion observes reordered matching Options';
                const abrupt = {};
                try { value.set.call(select, {toString() {throw abrupt;}}); return 'dropped exception'; }
                catch (error) { if (error !== abrupt || !second.selected) return 'original abrupt completion'; }
                let converted = false;
                try { value.set.call(first, {toString() {converted = true; return text;}}); return 'wrong brand'; }
                catch (error) { if (!(error instanceof TypeError) || converted) return 'brand before conversion'; }
                second.value = '';
                select.value = '';
                if (select.selectedIndex !== 0 || select.value !== '') return 'selected empty value';
                select.value = 'unmatched';
                if (select.selectedIndex !== -1 || select.value !== '') return 'unmatched clears selection';
                const raw = '\t New\ud800  text\0  ';
                first.text = raw;
                if (first.textContent !== raw || first.firstChild === null || first.firstChild !== first.lastChild ||
                    first.text !== 'New\ud800 text\0' || first.label !== first.text || first.value !== first.text)
                    return 'native string replace-all and subsequent text read';
                const calls = [];
                class ObservedOption extends HTMLOptionElement {
                    static observedAttributes = ['value'];
                    attributeChangedCallback(name, oldValue, value) {
                        calls.push([this, name, oldValue, value, this.text, this.label, this.value]);
                    }
                }
                customElements.define('x-option-string-read', ObservedOption, {extends: 'option'});
                const observed = document.createElement('option', {is: 'x-option-string-read'});
                observed.text = '   Ready\udfff \t  ';
                observed.value = '\ud800\0';
                const reaction = [observed, 'value', null, '\ud800\0', 'Ready\udfff', 'Ready\udfff', '\ud800\0'];
                if (calls.length !== 1 || !calls[0].every((result, index) => result === reaction[index]))
                    return 'published strings and output custody during the original reaction';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap_or_else(|error| {
        if let DocumentScriptError::Script(Error::HostFailure(failure)) = &error {
            panic!("Select and Option string host refusal: {:?}", std::error::Error::source(failure));
        }
        panic!("Select and Option string evaluation: {error:?}");
    });
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}
