//! Mixins over admitted creation realms and the original native Source budget.

use super::*;
use crate::{
    document_scripting::guest::tests::{
        attach_public_nodes, expose_public_node_for_source_test, fund_guest_operation_after_setup,
    },
    internal::dom_source::{DomSourceDriveError, DomSourceRuntimeError},
};

pub(super) fn borrowed_accessors_keep_native_includers_across_creation_realms() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let mut batch = DomBatch::new();
    let subjects = [
        (
            Namespace::html(),
            "div",
            "originMixinHtml",
            "localMixinHtml",
        ),
        (Namespace::svg(), "rect", "originMixinSvg", "localMixinSvg"),
        (
            Namespace::mathml(),
            "mi",
            "originMixinMath",
            "localMixinMath",
        ),
    ]
    .map(|(namespace, tag, origin_name, local_name)| {
        (
            batch.create_element_ns(origin.document(), namespace.clone(), tag),
            batch.create_element_ns(target.document(), namespace, tag),
            origin_name,
            local_name,
        )
    });
    let created = session.dom().apply(batch, &cancel).unwrap();
    let subjects = subjects.map(|(foreign, local, origin_name, local_name)| {
        (
            created.created(&foreign).unwrap().unwrap(),
            created.created(&local).unwrap().unwrap(),
            origin_name,
            local_name,
        )
    });
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    for (foreign, local, origin_name, local_name) in subjects {
        expose_public_node_for_source_test(&mut host, &permit, foreign, origin_name);
        expose_public_node_for_source_test(&mut host, &permit, local, local_name);
    }
    fund_guest_operation_after_setup(&host);
    host.in_document(&permit, |scope| {
        for (_, _, origin_name, local_name) in subjects {
            let foreign = evaluate(scope, origin_name)?;
            let local = evaluate(scope, local_name)?;
            assert_ne!(
                scope.callback_object(&foreign)?.associated_realm().identity(),
                permit.realm
            );
            assert_eq!(
                scope.callback_object(&local)?.associated_realm().identity(),
                permit.realm
            );
        }
        let result = evaluate(scope, r#"
            (() => {
                const originPrototype = node => {
                    for (let p = Object.getPrototypeOf(node); p; p = Object.getPrototypeOf(p)) {
                        if (Object.hasOwn(p, 'nonce')) return p;
                    }
                    return null;
                };
                const saved = [];
                for (const [foreign, local, type] of [
                    [originMixinHtml, localMixinHtml, HTMLElement],
                    [originMixinSvg, localMixinSvg, SVGElement],
                    [originMixinMath, localMixinMath, MathMLElement],
                ]) {
                    const prototype = originPrototype(foreign);
                    if (!prototype || prototype === type.prototype || foreign instanceof type ||
                        !(local instanceof type) || prototype.constructor.name !== type.name) {
                        return 'mixin accessor creation realm';
                    }
                    const foreignNonce = Object.getOwnPropertyDescriptor(prototype, 'nonce');
                    const localNonce = Object.getOwnPropertyDescriptor(type.prototype, 'nonce');
                    const foreignFocus = Object.getOwnPropertyDescriptor(prototype, 'autofocus');
                    const localFocus = Object.getOwnPropertyDescriptor(type.prototype, 'autofocus');
                    if (foreignNonce.get === localNonce.get || foreignFocus.set === localFocus.set) return 'separate realm functions';
                    foreign.setAttributeNS(null, 'nonce', 'public');
                    localNonce.set.call(foreign, 'origin\ud800\0');
                    foreignNonce.set.call(local, 'local\udc00\0');
                    localFocus.set.call(foreign, true);
                    foreignFocus.set.call(local, true);
                    if (localNonce.get.call(foreign) !== 'origin\ud800\0' ||
                        foreignNonce.get.call(local) !== 'local\udc00\0' ||
                        !localFocus.get.call(foreign) || !foreignFocus.get.call(local) ||
                        foreign.getAttributeNS(null, 'nonce') !== 'public' || local.hasAttributeNS(null, 'nonce') ||
                        foreign.getAttributeNS(null, 'autofocus') !== '' || local.getAttributeNS(null, 'autofocus') !== '') {
                        return 'borrowed mixin accessors in both directions';
                    }
                    saved.push([foreign, Object.getPrototypeOf(foreign), localNonce, foreignFocus]);
                }
                globalThis.savedMixinOrigins = saved;
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16(&scope.to_utf16(&result)?).unwrap(), "ok");
        Ok(())
    }).unwrap();

    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        for (foreign, _, _, _) in subjects {
            batch.adopt(foreign, target.document());
        }
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(
            scope,
            r#"
            savedMixinOrigins.every(([node, prototype, localNonce, foreignFocus], index) =>
                node === [originMixinHtml, originMixinSvg, originMixinMath][index] &&
                Object.getPrototypeOf(node) === prototype && node.ownerDocument === document &&
                localNonce.get.call(node) === 'origin\ud800\0' && foreignFocus.get.call(node))
            "#,
        )?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

pub(super) fn mixin_setters_preserve_native_state_on_source_refusal() {
    for script in [
        "limitedMixin.nonce = 'changed'",
        "limitedMixin.autofocus = true",
        "limitedMixin.tabIndex = -1",
    ] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let subject = batch.create_element_ns(target.document(), Namespace::html(), "div");
        batch.set_attribute_ns(&subject, Namespace::none(), "nonce", "public");
        batch.set_nonce(&subject, "private");
        batch.set_attribute_ns(&subject, Namespace::none(), "autofocus", "authored");
        batch.set_attribute_ns(&subject, Namespace::none(), "tabindex", "+7tail");
        let subject = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&subject)
            .unwrap()
            .unwrap();
        let mut host = attach_public_nodes(session, target, cancel, 200_000, 0);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, subject, "limitedMixin");
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| evaluate(scope, script).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("element mixin Source work refusal for {script}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DomSourceDriveError>(),
            Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                DomError::SourceWorkLimit { limit: 0 }
            )))
        ));
        let after = host.snapshot().unwrap();
        assert_eq!(after.stamp(), before.stamp());
        let node = after.node(subject).unwrap();
        assert_eq!(node.as_element().unwrap().nonce(), Some(&"private".into()));
        assert_eq!(node.attribute(&"nonce".into()), Some(&"public".into()));
        assert_eq!(
            node.attribute(&"autofocus".into()),
            Some(&"authored".into())
        );
        assert_eq!(node.attribute(&"tabindex".into()), Some(&"+7tail".into()));
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}
