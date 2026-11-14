use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, projected_node_id,
};

pub(super) fn structural_alignment_families_preserve_native_and_web_idl_contracts() {
    // Each actual-name case owns its live tree and cumulative semantic work.
    for tag in ["div", "p", "h1", "h2", "h3", "h4", "h5", "h6"] {
        structural_brands_precede_conversion(tag);
        structural_strings_reflect_exact_expanded_names(tag);
        structural_customized_constructors_and_reactions_keep_the_actual_tag(tag);
        structural_reads_follow_native_edits_and_retirement(tag);
    }
    for tag in ["div", "p", "h6"] {
        structural_accessors_keep_registered_realms_after_adoption(tag);
        structural_setters_admit_source_outputs_before_publication(tag);
    }
}

fn set_alignment_tag(scope: &mut Scope<'_>, tag: &str) -> Result<(), Error> {
    let global = scope.global()?;
    let value = scope.string(&tag.encode_utf16().collect::<Vec<_>>())?;
    scope.set(
        &global,
        PropertyKey::String(&"alignmentTag".encode_utf16().collect::<Vec<_>>()),
        &value,
    )
}

fn check_alignment_script(tag: &str, script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_alignment_tag(scope, tag)?;
        let result = evaluate(scope, script)?;
        assert_eq!(
            String::from_utf16_lossy(&scope.to_utf16(&result)?),
            "ok",
            "{tag}"
        );
        Ok(())
    })
    .unwrap_or_else(|error| match &error {
        DocumentScriptError::Script(Error::HostFailure(failure)) => panic!(
            "alignment {tag}: guest={:?}; source={:?}",
            failure.cause::<DocumentGuestEventError>(),
            failure.cause::<crate::DomSourceDriveError>()
        ),
        _ => panic!("alignment {tag}: {error:?}"),
    });
}

fn structural_brands_precede_conversion(tag: &str) {
    check_alignment_script(
        tag,
        r#"
        (() => {
            const Type = alignmentTag === 'div' ? HTMLDivElement :
                alignmentTag === 'p' ? HTMLParagraphElement : HTMLHeadingElement;
            const element = document.createElement(alignmentTag.toUpperCase());
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:' + alignmentTag);
            const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, 'align');
            if (Object.getPrototypeOf(Type) !== HTMLElement || Object.getPrototypeOf(Type.prototype) !== HTMLElement.prototype ||
                Object.getPrototypeOf(element) !== Type.prototype || element.localName !== alignmentTag ||
                !(prefixed instanceof Type) || Object.keys(Type.prototype).join() !== 'align' ||
                descriptor.get.name !== 'get align' || descriptor.set.name !== 'set align') return 'inheritance and descriptors';
            try { new Type(); return 'unregistered HTMLConstructor'; }
            catch (error) { if (!(error instanceof TypeError)) return 'HTMLConstructor error'; }
            descriptor.set.call(prefixed, 'prefixed');
            if (descriptor.get.call(prefixed) !== 'prefixed') return 'prefixed authentic receiver';
            const foreign = document.createElementNS('urn:foreign', alignmentTag);
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', alignmentTag.toUpperCase());
            const other = document.createElement(alignmentTag === 'div' ? 'p' : 'div');
            const hgroup = document.createElement('hgroup');
            if (foreign instanceof Type || upper instanceof Type || hgroup instanceof HTMLHeadingElement)
                return 'namespace case and hgroup exclusion';
            Object.setPrototypeOf(other, Type.prototype);
            for (const receiver of [null, {}, document, foreign, upper, other, hgroup,
                Object.create(element), new Proxy(element, {})]) {
                let converted = false;
                try { descriptor.get.call(receiver); return 'getter accepted wrong native family'; }
                catch (error) { if (!(error instanceof TypeError)) return 'getter brand error'; }
                try { descriptor.set.call(receiver, {toString() { converted = true; return 'bad'; }});
                    return 'setter accepted wrong native family'; }
                catch (error) { if (!(error instanceof TypeError) || converted) return 'native brand before conversion'; }
            }
            return 'ok';
        })()
    "#,
    );
}

fn structural_strings_reflect_exact_expanded_names(tag: &str) {
    check_alignment_script(
        tag,
        r#"
        (() => {
            const element = document.createElement(alignmentTag);
            if (element.align !== '') return 'absent default';
            element.setAttributeNS('urn:foreign', 'align', 'foreign');
            if (element.align !== '') return 'foreign attribute ignored';
            element.align = null;
            if (element.align !== 'null' || element.getAttributeNS(null, 'align') !== 'null') return 'ordinary null DOMString';
            element.align = {toString() { return null; }};
            if (element.align !== 'null') return 'object null DOMString';
            element.align = undefined;
            if (element.align !== 'undefined') return 'undefined DOMString';
            const exact = ' Raw\ud800\0\udc00 ';
            let count = 0;
            element.align = {toString() {
                count++;
                element.setAttributeNS(null, 'align', 'nested');
                if (element.align !== 'nested') throw 'nested reflection unavailable';
                return exact;
            }};
            if (count !== 1 || element.align !== exact || element.getAttributeNS(null, 'align') !== exact ||
                element.getAttributeNS('urn:foreign', 'align') !== 'foreign') return 'UTF16 reentry and namespace isolation';
            const original = {};
            try { element.align = {toString() { throw original; }}; return 'throwing value accepted'; }
            catch (error) { if (error !== original || element.align !== exact) return 'original exception and state'; }
            for (const value of [Symbol(), {toString() { return Symbol(); }}]) {
                try { element.align = value; return 'Symbol value accepted'; }
                catch (error) { if (!(error instanceof TypeError) || element.align !== exact) return 'Symbol state'; }
            }
            element.setAttributeNS(null, 'align', ' LEFT center invalid Justify ');
            if (element.align !== ' LEFT center invalid Justify ') return 'raw content without keyword normalization';
            element.removeAttributeNS(null, 'align');
            return element.align === '' && element.getAttributeNS('urn:foreign', 'align') === 'foreign'
                ? 'ok' : 'expanded-name removal';
        })()
    "#,
    );
}

fn structural_customized_constructors_and_reactions_keep_the_actual_tag(tag: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_alignment_tag(scope, tag)?;
        let result = evaluate(scope, r#"
            (() => {
                const Type = alignmentTag === 'div' ? HTMLDivElement :
                    alignmentTag === 'p' ? HTMLParagraphElement : HTMLHeadingElement;
                const calls = [];
                class Watched extends Type {
                    static observedAttributes = ['align'];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                        if (newValue === 'outer') this.align = 'inner';
                    }
                }
                customElements.define('x-align-' + alignmentTag, Watched, {extends: alignmentTag});
                const direct = new Watched();
                const element = document.createElement(alignmentTag, {is: 'x-align-' + alignmentTag});
                if (!(direct instanceof Watched) || !(element instanceof Watched) ||
                    direct.localName !== alignmentTag || element.localName !== alignmentTag || direct === element)
                    return 'customized actual extends tag';
                element.align = null;
                if (calls.length !== 1 || calls[0][0] !== element || calls[0][1] !== 'align' ||
                    calls[0][2] !== null || calls[0][3] !== 'null' || calls[0][4] !== null || calls[0][5] !== 'null')
                    return 'original synchronous DOMString reaction';
                element.align = 'outer';
                if (calls.length !== 3 || calls[1][2] !== 'null' || calls[1][3] !== 'outer' || calls[1][5] !== 'outer' ||
                    calls[2][2] !== 'outer' || calls[2][3] !== 'inner' || calls[2][5] !== 'inner' || element.align !== 'inner')
                    return 'publication before callback reentry';
                const original = {};
                try { element.align = {toString() { direct.align = '\ud800\0'; throw original; }};
                    return 'conversion throw'; }
                catch (error) { if (error !== original) return 'nested Source original throw identity'; }
                if (calls.length !== 4 || calls[3][0] !== direct || calls[3][2] !== null ||
                    calls[3][3] !== '\ud800\0' || calls[3][5] !== '\ud800\0' || element.align !== 'inner')
                    return 'nested Source reaction survives outer conversion throw';
                class Throwing extends Type {
                    constructor() { super(); globalThis.constructedAlignment = this;
                        throw globalThis.alignmentConstructorThrow = {}; }
                }
                customElements.define('x-failed-' + alignmentTag, Throwing, {extends: alignmentTag});
                const failed = document.createElement(alignmentTag, {is: 'x-failed-' + alignmentTag});
                if (failed !== constructedAlignment || !(failed instanceof Type) || !(failed instanceof Throwing) ||
                    failed.localName !== alignmentTag || failed instanceof HTMLUnknownElement)
                    return 'customized failure retains actual native interface';
                const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, 'align');
                descriptor.set.call(failed, 'authentic');
                return descriptor.get.call(failed) === 'authentic' ? 'ok' : 'authentic failed construction receiver';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok", "{tag}");
        Ok(())
    }).unwrap();
    let report = host
        .take_guest_constructor_exception(&permit)
        .unwrap()
        .unwrap();
    assert!(
        host.take_guest_constructor_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.in_document(&permit, |scope| {
        let original = evaluate(scope, "alignmentConstructorThrow")?;
        let reported = scope.import(report.exception.value())?;
        assert!(scope.strict_equals(&original, &reported)?);
        Ok(())
    })
    .unwrap();
}

fn structural_reads_follow_native_edits_and_retirement(tag: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let element = host.in_document(&permit, |scope| {
        set_alignment_tag(scope, tag)?;
        evaluate(scope, r#"
            globalThis.alignmentContainer = document.createElement('div');
            globalThis.nativeAlignment = alignmentContainer.appendChild(document.createElement(alignmentTag));
            globalThis.savedAlignment = nativeAlignment;
            globalThis.alignmentPrototype = Object.getPrototypeOf(nativeAlignment);
        "#)?;
        let element = evaluate(scope, "nativeAlignment")?;
        projected_node_id(scope, &element)
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(
            element,
            Namespace::none(),
            "align",
            surgeist_dom::DocumentString::from_utf16([0xd800, 0, 0xdc00]),
        );
        batch.set_attribute_ns(
            element,
            Namespace::from_uri("urn:foreign"),
            "align",
            "foreign",
        );
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            nativeAlignment === savedAlignment && alignmentContainer.firstChild === nativeAlignment &&
            nativeAlignment.align === '\ud800\0\udc00' && nativeAlignment.getAttributeNS('urn:foreign', 'align') === 'foreign'
        "#)?;
        assert!(scope.boolean(&result)?, "{tag}");
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.in_document(&permit, |scope| {
        evaluate(scope, r#"
            globalThis.alignmentDestination = new Document();
            alignmentDestination.adoptNode(nativeAlignment);
        "#)?;
        let result = evaluate(scope, r#"
            nativeAlignment === savedAlignment && nativeAlignment.ownerDocument === alignmentDestination &&
            nativeAlignment.parentNode === null && Object.getPrototypeOf(nativeAlignment) === alignmentPrototype &&
            nativeAlignment.localName === alignmentTag && nativeAlignment.align === '\ud800\0\udc00'
        "#)?;
        assert!(scope.boolean(&result)?, "{tag}");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        // Source adoption detached this exact node before native retirement.
        batch.destroy(element);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    for expression in ["nativeAlignment.align", "nativeAlignment.align = 'retired'"] {
        let result = host.in_document(&permit, |scope| evaluate(scope, expression).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("retired native {tag} must refuse {expression}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
            ))
        ));
    }
}

fn structural_accessors_keep_registered_realms_after_adoption(tag: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let element = {
        let mut batch = DomBatch::new();
        let element = batch.create_element_ns(origin.document(), Namespace::html(), tag);
        session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&element)
            .unwrap()
            .unwrap()
    };
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(&mut host, &permit, element, "originAlignment");
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
                let value = evaluate(scope, "TypeError")?;
                Ok(scope.retain(&value))
            })
            .unwrap()
    };
    host.in_document(&permit, |scope| {
        set_alignment_tag(scope, tag)?;
        let value = evaluate(scope, "originAlignment")?;
        assert_ne!(scope.callback_object(&value)?.associated_realm().identity(), permit.realm);
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        scope.set(&global, PropertyKey::String(&"foreignAlignmentTypeError".encode_utf16().collect::<Vec<_>>()), &error)?;
        let result = evaluate(scope, r#"
            (() => {
                const Type = alignmentTag === 'div' ? HTMLDivElement :
                    alignmentTag === 'p' ? HTMLParagraphElement : HTMLHeadingElement;
                globalThis.savedOriginAlignment = originAlignment;
                globalThis.originAlignmentPrototype = Object.getPrototypeOf(originAlignment);
                if (originAlignment instanceof Type || originAlignmentPrototype.constructor.name !== Type.name)
                    return 'registered creation realm';
                const descriptor = Object.getOwnPropertyDescriptor(originAlignmentPrototype, 'align');
                for (const accessor of [descriptor.get, descriptor.set]) {
                    try { accessor.call({}, null); return 'borrowed receiver accepted'; }
                    catch (error) { if (!(error instanceof foreignAlignmentTypeError) || error instanceof TypeError)
                        return 'accessor function realm exception'; }
                }
                descriptor.set.call(originAlignment, null);
                if (descriptor.get.call(originAlignment) !== 'null') return 'borrowed ordinary null conversion';
                const original = {};
                try { descriptor.set.call(originAlignment, {toString() { throw original; }}); return 'borrowed conversion throw'; }
                catch (error) { if (error !== original) return 'borrowed original exception identity'; }
                descriptor.set.call(originAlignment, 'foreign\ud800\0');
                return descriptor.get.call(originAlignment) === 'foreign\ud800\0' ? 'ok' : 'borrowed native reflection';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok", "{tag}");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(element, target.document());
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            originAlignment === savedOriginAlignment && originAlignment.ownerDocument === document &&
            Object.getPrototypeOf(originAlignment) === originAlignmentPrototype &&
            originAlignment.localName === alignmentTag && originAlignment.align === 'foreign\ud800\0'
        "#)?;
        assert!(scope.boolean(&result)?, "{tag}");
        Ok(())
    }).unwrap();
}

fn structural_setters_admit_source_outputs_before_publication(tag: &str) {
    for outputs in [1, 2] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let element = batch.create_element_ns(target.document(), Namespace::html(), tag);
        let element = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&element)
            .unwrap()
            .unwrap();
        let mut host =
            attach_public_nodes_with_source_output_limit(session, target, cancel, outputs);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, element, "limitedAlignment");
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| {
            evaluate(scope, "limitedAlignment.align = null").map(drop)
        });
        let after = host.snapshot().unwrap();
        if outputs == 1 {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short {tag} Source output must refuse before publication: {result:?}");
            };
            assert!(matches!(
                failure.cause::<crate::DomSourceDriveError>(),
                Some(crate::DomSourceDriveError::Source(
                    crate::DomSourceRuntimeError::Dom(DomError::SourceOutputLimit)
                ))
            ));
            assert_eq!(after.stamp(), before.stamp());
            assert_eq!(after.nodes().count(), before.nodes().count());
            assert!(
                after
                    .node(element)
                    .unwrap()
                    .attributes()
                    .unwrap()
                    .is_empty()
            );
        } else {
            result.unwrap();
            let name = "align".into();
            let attr = after
                .node(element)
                .unwrap()
                .get_attribute_node_ns(&Namespace::none(), &name)
                .unwrap();
            assert_eq!(
                attr.value().code_units(),
                &[0x6e, 0x75, 0x6c, 0x6c],
                "{tag} admitted native value"
            );
        }
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

// HTML877d's modern interfaces and obsolete partial interfaces independently
// specify align as a CEReactions/Reflect DOMString for these three families.
pub(super) fn structural_alignment_reflects_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const families = [['div', HTMLDivElement], ['p', HTMLParagraphElement],
                    ...['h1', 'h2', 'h3', 'h4', 'h5', 'h6'].map(tag => [tag, HTMLHeadingElement])];
                for (const [tag, Type] of families) {
                    if (!(document.createElement(tag) instanceof Type)) return tag + ' interface';
                }
                if (document.createElement('hgroup') instanceof HTMLHeadingElement)
                    return 'hgroup is not a heading interface';
                for (const [tag, Type] of families) {
                    if (Object.keys(Type.prototype).join() !== 'align')
                        return tag + ' missing reflected align';
                    const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, 'align');
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return tag + ' align descriptor';
                    const element = document.createElement(tag);
                    if (element.align !== '') return tag + ' absent align default';
                    element.align = null;
                    if (element.align !== 'null' || element.getAttributeNS(null, 'align') !== 'null')
                        return tag + ' plain DOMString reflection';
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
