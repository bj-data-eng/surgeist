use super::{evaluate, hosted_session_with_dom_limits};
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_bindings_js::{Error, PropertyKey};
use surgeist_dom::{
    DomBatch,
    form::select::DomSelectTaskOutcome,
    shadow::DomSlotQueryLimits,
    task::{DomNativeCheckpoint, DomNativeCheckpointLimits},
};
use surgeist_html::values::Namespace;

pub(super) fn selectedness_uses_live_native_option_order() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits
            .with_select_processing(8, 4_000_000, 100_000)
            .unwrap()
    });
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
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
                for (const [type, member, writable] of [
                    [HTMLSelectElement, 'multiple', true],
                    [HTMLSelectElement, 'selectedIndex', true],
                    [HTMLOptionElement, 'defaultSelected', true],
                    [HTMLOptionElement, 'selected', true],
                    [HTMLOptionElement, 'index', false],
                ]) {
                    const descriptor = Object.getOwnPropertyDescriptor(type.prototype, member);
                    if (!descriptor || !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get?.length !== 0 || descriptor.get?.name !== 'get ' + member ||
                        (writable ? descriptor.set?.length !== 1 || descriptor.set?.name !== 'set ' + member :
                            descriptor.set !== undefined)) return member + ' accessor descriptor';
                }
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
                try { index.set.call(first, {valueOf() {coerced = true; return 0;}}); return 'wrong native brand'; }
                catch (error) { if (!(error instanceof TypeError) || coerced) return 'brand before coercion'; }
                const abrupt = {};
                try { select.selectedIndex = {valueOf() {throw abrupt;}}; return 'dropped guest exception'; }
                catch (error) { if (error !== abrupt || select.selectedIndex !== -1) return 'coercion first failure'; }
                index.set.call(select);
                if (select.selectedIndex !== 0 || !second.selected || first.selected) return 'undefined long conversion';
                group.removeChild(first);
                if (first.index !== 0 || !first.defaultSelected) return 'detached index and reflection';

                const construct = [];
                const reactions = [];
                class WatchedSelect extends HTMLSelectElement {
                    constructor() {
                        super();
                        construct.push([this, this.selectedIndex, this.multiple]);
                    }
                }
                class WatchedOption extends HTMLOptionElement {
                    constructor() {
                        super();
                        construct.push([this, this.index, this.selected]);
                    }
                    static observedAttributes = ['selected'];
                    attributeChangedCallback(name, oldValue, value) {
                        reactions.push([this, name, oldValue, value, this.index, this.selected, this.defaultSelected]);
                    }
                }
                customElements.define('x-selectedness-select', WatchedSelect, {extends: 'select'});
                customElements.define('x-selectedness-option', WatchedOption, {extends: 'option'});
                const watchedSelect = document.createElement('select', {is: 'x-selectedness-select'});
                const watchedOption = document.createElement('option', {is: 'x-selectedness-option'});
                const expected = [[watchedSelect, -1, false], [watchedOption, 0, false]];
                if (construct.length !== expected.length ||
                    !construct.every((row, index) => row.every((value, column) => value === expected[index][column])))
                    return 'current publication in qualified constructors';
                watchedSelect.multiple = true;
                watchedSelect.appendChild(document.createElement('option'));
                watchedSelect.appendChild(watchedOption);
                watchedOption.defaultSelected = true;
                const reaction = [watchedOption, 'selected', null, '', 1, true, true];
                if (reactions.length !== 1 || !reactions[0].every((value, column) => value === reaction[column]) ||
                    watchedSelect.selectedIndex !== 1) return 'selected reflection reaction observes published state';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        assert!(dom.pending_source_edit().is_none());
        // Removing the Option schedules the native selectedcontent checkpoint.
        let limits = DomNativeCheckpointLimits::new(
            4_000_000,
            100_000,
            DomSlotQueryLimits::new(100, 100).unwrap(),
        )
        .unwrap();
        assert_eq!(dom.raw().pending_native_tasks(), 1);
        let DomNativeCheckpoint::Select(task) = dom
            .raw_mut()
            .native_checkpoint_step(limits, &cancel)
            .unwrap()
        else {
            panic!("option removal must schedule a Select task")
        };
        let completion = dom
            .raw_mut()
            .complete_select_task(&task, limits, &cancel)
            .unwrap();
        assert_eq!(completion.outcome(), DomSelectTaskOutcome::Updated);
        assert_eq!(dom.raw().pending_native_tasks(), 0);
    })
    .unwrap();
    host.stop().into_session().unwrap();
}

pub(super) fn selectedness_accessors_use_their_creation_realm() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits
            .with_select_processing(8, 4_000_000, 100_000)
            .unwrap()
    });
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let mut batch = DomBatch::new();
    let select = batch.create_element_ns(origin.document(), Namespace::html(), "select");
    let option = batch.create_element_ns(origin.document(), Namespace::html(), "option");
    batch.insert_before(&option, &select, None);
    let commit = session.dom().apply(batch, &cancel).unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    for (name, ticket) in [("foreignSelect", select), ("foreignOption", option)] {
        expose_public_node_for_source_test(
            &mut host,
            &permit,
            commit.created(&ticket).unwrap().unwrap(),
            name,
        );
    }
    let foreign_type_error = {
        let engine = host.engine.as_mut().unwrap();
        let realm = engine
            .creations
            .find(origin.global(), || Ok::<_, Error>(()))
            .unwrap()
            .unwrap();
        engine
            .process
            .execution()
            .in_realm(realm, |scope| {
                let error = evaluate(scope, "TypeError")?;
                Ok(scope.retain(&error))
            })
            .unwrap()
    };
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        let name: Vec<_> = "ForeignTypeError".encode_utf16().collect();
        scope.set(&global, PropertyKey::String(&name), &error)?;
        let result = evaluate(scope, r#"
            (() => {
                const descriptor = (node, name) => Object.getOwnPropertyDescriptor(Object.getPrototypeOf(node), name);
                const index = descriptor(foreignSelect, 'selectedIndex');
                const selected = descriptor(foreignOption, 'selected');
                if (foreignSelect instanceof HTMLSelectElement || foreignOption instanceof HTMLOptionElement)
                    return 'creation prototypes';
                try { index.set.call(foreignOption, 0); return 'foreign invalid receiver'; }
                catch (error) {
                    if (!(error instanceof ForeignTypeError) || error instanceof TypeError) return 'accessor error realm';
                }
                index.set.call(foreignSelect, -1);
                if (selected.get.call(foreignOption) || index.get.call(foreignSelect) !== -1)
                    return 'foreign mutation';
                selected.set.call(foreignOption, true);
                return index.get.call(foreignSelect) === 0 && foreignOption.index === 0 &&
                    foreignSelect.firstChild === foreignOption ? 'ok' : 'foreign observation and identity';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    drop(foreign_type_error);
    host.stop().into_session().unwrap();
}
