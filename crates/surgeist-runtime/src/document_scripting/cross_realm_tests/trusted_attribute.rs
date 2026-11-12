use super::*;

pub(super) fn guest_verification_without_callback_retains_the_native_continuation() {
    use crate::document_scripting::guest::tests::{
        attach_public_nodes, expose_public_node_for_source_test,
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let document = target.document();
    let snapshot = session.snapshot();
    let root = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(root).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let script = batch.create_element_ns(document, Namespace::html(), "script");
    batch.insert_before(&script, body, None);
    let script = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&script)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, script, "scriptTarget");
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "scriptTarget.setAttribute('src', 'example.js')").map(drop)
    });
    assert!(matches!(
        result,
        Err(DocumentScriptError::Script(Error::HostFailure(_)))
    ));
    let parked = host
        .with_native(|dom, _, _| dom.pending_source_edit())
        .unwrap()
        .unwrap();
    assert_eq!(
        parked.phase(),
        surgeist_dom::DomSourcePhase::ScriptPreparation
    );
    let progress = host
        .resume_pending_attribute_write(&permit, SOURCE)
        .unwrap()
        .unwrap();
    assert!(matches!(
        progress,
        crate::DomSourceDriveProgress::Pending {request: DomSourceRequest::Script(request), ..}
            if request.operation() == parked.operation()
    ));
    assert!(
        host.snapshot()
            .unwrap()
            .node(script)
            .unwrap()
            .attribute(&"src".into())
            .is_none()
    );
}

pub(super) fn guest_attribute_policy_verifies_html_and_svg_sinks() {
    use crate::document_scripting::guest::tests::{
        attach_public_nodes, expose_public_node_for_source_test,
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits.with_meta_processing_work(1_000_000)
    });
    session.enable_trusted_types_csp(&cancel).unwrap();
    let document = target.document();
    let snapshot = session.snapshot();
    let root = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let head = snapshot.node(root).unwrap().children()[0];
    let hosted = session.hosted_execution();
    let context = hosted
        .with_current_top_level(&mut session.dom(), target, &cancel, |source, dom| {
            source
                .document_constructor_context(dom.raw(), target.global(), &cancel)
                .map_err(crate::DomHostedRuntimeError::Dom)
        })
        .unwrap();
    let mut batch = DomBatch::new();
    let detached = batch.construct_document(context);
    let meta = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "content-security-policy");
    batch.set_attribute(&meta, "content", "require-trusted-types-for 'script'");
    let iframe = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    let script = batch.create_element_ns(document, Namespace::html(), "script");
    let svg = batch.create_element_ns(&detached, Namespace::svg(), "script");
    let formed = session.dom().apply(batch, &cancel).unwrap();
    let mut adopt = DomBatch::new();
    for ticket in [&iframe, &svg] {
        adopt.adopt(formed.created(ticket).unwrap().unwrap(), document);
    }
    session.dom().apply(adopt, &cancel).unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, head, "sinkHead");
    for (ticket, key) in [
        (meta, "sinkPolicy"),
        (iframe, "iframeSink"),
        (script, "scriptSink"),
        (svg, "svgSink"),
    ] {
        let node = formed.created(&ticket).unwrap().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, node, key);
    }
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const xlink = 'http://www.w3.org/1999/xlink';
                const calls = [];
                sinkHead.appendChild(sinkPolicy);
                const accept = (value, type, sink) => {
                    calls.push([value, type, sink]);
                    return value + '!\udfff';
                };
                trustedTypes.createPolicy('default', {createHTML: accept, createScriptURL: accept});
                iframeSink.setAttribute('srcdoc', 'html\ud800\0');
                scriptSink.setAttributeNS(null, 'src', 'html script\ud800');
                svgSink.setAttributeNS(xlink, 'xlink:href', 'svg xlink\ud800');
                svgSink.setAttributeNS(null, 'href', 'svg plain\ud800');
                const expected = [
                    ['html\ud800\0', 'TrustedHTML', 'HTMLIFrameElement srcdoc'],
                    ['html script\ud800', 'TrustedScriptURL', 'HTMLScriptElement src'],
                    ['svg xlink\ud800', 'TrustedScriptURL', 'SVGScriptElement href'],
                    ['svg plain\ud800', 'TrustedScriptURL', 'SVGScriptElement href']
                ];
                if (JSON.stringify(calls) !== JSON.stringify(expected)) return JSON.stringify(calls);
                if (iframeSink.getAttribute('srcdoc') !== 'html\ud800\0!\udfff' ||
                    scriptSink.getAttribute('src') !== 'html script\ufffd!\ufffd' ||
                    svgSink.getAttributeNS(xlink, 'href') !== 'svg xlink\ufffd!\ufffd' ||
                    svgSink.getAttributeNS(null, 'href') !== 'svg plain\ufffd!\ufffd')
                    return 'wrong default callback string result';
                const identity = value => value;
                const named = trustedTypes.createPolicy('matching', {
                    createHTML: identity, createScriptURL: identity
                });
                iframeSink.setAttribute('srcdoc', named.createHTML('trusted\ud800'));
                scriptSink.setAttribute('src', named.createScriptURL('script\ud800'));
                svgSink.setAttributeNS(xlink, 'xlink:href', named.createScriptURL('svg\ud800'));
                return calls.length === 4 && iframeSink.getAttribute('srcdoc') === 'trusted\ud800'
                    && scriptSink.getAttribute('src') === 'script\ufffd'
                    && svgSink.getAttributeNS(xlink, 'href') === 'svg\ufffd'
                    ? 'ok' : 'matching brand did not bypass the default callback';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

pub(super) fn guest_attribute_policy_keeps_original_authority_after_adoption() {
    use crate::document_scripting::guest::tests::{
        assert_queued_trusted_types_violations, attach_public_nodes,
        expose_public_node_for_source_test,
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits.with_meta_processing_work(1_000_000)
    });
    let other = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let accepted = insert_hosted_element(&mut session, target, "div", &cancel);
    let rejected = insert_hosted_element(&mut session, target, "div", &cancel);
    let destination = insert_hosted_element(&mut session, other, "div", &cancel);
    let original = session.snapshot();
    let root = original
        .document(target.document())
        .unwrap()
        .document_element()
        .unwrap();
    let head = original.node(root).unwrap().children()[0];
    let mut batch = DomBatch::new();
    let receiver = batch.create_element_ns(
        other.document(),
        Namespace::from_uri("urn:fixture"),
        "receiver",
    );
    batch.insert_before(&receiver, destination, None);
    let accepted_attr = batch.create_attribute(other.document(), "onclick");
    batch.set_attribute(rejected, "onclick", "old");
    let meta = batch.create_element_ns(target.document(), Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "content-security-policy");
    batch.set_attribute(&meta, "content", "require-trusted-types-for 'script'");
    let commit = session.dom().apply(batch, &cancel).unwrap();
    let receiver = commit.created(&receiver).unwrap().unwrap();
    let meta = commit.created(&meta).unwrap().unwrap();
    let snapshot = session.snapshot();
    let accepted_attr = commit.created(&accepted_attr).unwrap().unwrap();
    let accepted_identity = snapshot.attr(accepted_attr).unwrap().id();
    assert_eq!(
        snapshot.relevant_global(accepted_attr).unwrap(),
        Some(other.global())
    );
    let rejected_attr = snapshot
        .node(rejected)
        .unwrap()
        .get_attribute_node(&"onclick".into())
        .unwrap()
        .id()
        .node();
    // The sink's initial policy comes from its owner Element even when the
    // Attr already belongs to a different node document.
    let mut adopt = DomBatch::new();
    adopt.attach_native_attribute(accepted, accepted_identity);
    adopt.adopt(accepted_attr, other.document());
    session.dom().apply(adopt, &cancel).unwrap();
    session.enable_trusted_types_csp(&cancel).unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, other).unwrap();
    let origin_prototype = {
        let engine = host.engine.as_mut().unwrap();
        let realm = engine
            .creations
            .find(other.global(), || Ok::<(), Error>(()))
            .unwrap()
            .unwrap();
        engine
            .process
            .execution()
            .in_realm(realm, |scope| {
                let prototype = evaluate(scope, "Attr.prototype")?;
                Ok(scope.retain(&prototype))
            })
            .unwrap()
    };
    for (node, name) in [
        (head, "adoptionHead"),
        (meta, "adoptionMeta"),
        (accepted, "acceptedOwner"),
        (accepted_attr, "acceptedAttr"),
        (rejected, "rejectedOwner"),
        (rejected_attr, "rejectedAttr"),
        (destination, "adoptionTarget"),
        (receiver, "replacementOwner"),
    ] {
        expose_public_node_for_source_test(&mut host, &permit, node, name);
    }
    let result = host.in_document(&permit, |scope| {
        let prototype = scope.import(&origin_prototype)?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&"originAttrPrototype".encode_utf16().collect::<Vec<_>>()), &prototype)?;
        evaluate(scope, r#"
            (() => {
                const check = (ok, label) => { if (!ok) throw new Error(label); };
                adoptionHead.appendChild(adoptionMeta);
                const originalDocument = document;
                const destinationDocument = adoptionTarget.ownerDocument;
                check(originalDocument !== destinationDocument, 'distinct hosted documents');
                check(acceptedAttr.ownerDocument === destinationDocument && acceptedAttr.ownerElement === acceptedOwner,
                    'distinct initial Attr and owner documents');
                const creationPrototype = Object.getPrototypeOf(acceptedAttr);
                check(creationPrototype === originAttrPrototype && creationPrototype !== Attr.prototype
                    && Object.getPrototypeOf(rejectedAttr) === Attr.prototype,
                    'distinct creation realms');
                const throws = (action, name) => {
                    try { action(); } catch (error) { return error.name === name; }
                    return false;
                };
                const root = acceptedOwner.attachShadow({mode: 'closed'});
                check(throws(() => destinationDocument.adoptNode(originalDocument), 'NotSupportedError')
                    && throws(() => destinationDocument.adoptNode(root), 'HierarchyRequestError')
                    && throws(() => destinationDocument.adoptNode(), 'TypeError')
                    && throws(() => destinationDocument.adoptNode(new Proxy(acceptedAttr, {})), 'TypeError')
                    && throws(() => Document.prototype.adoptNode.call({}, acceptedAttr), 'TypeError'),
                    'adoptNode brand, arity and native errors');
                const calls = [];
                trustedTypes.createPolicy('default', {createScript(value, type, sink) {
                    calls.push(value);
                    check(type === 'TrustedScript' && sink === 'Element onclick', 'original selected sink');
                    const [owner, attr] = value === 'move\ud800'
                        ? [acceptedOwner, acceptedAttr] : [rejectedOwner, rejectedAttr];
                    check(owner.ownerDocument === originalDocument && attr.ownerElement === owner, 'original owner');
                    check(adoptionTarget.appendChild(owner) === owner, 'same adopted owner');
                    check(owner.ownerDocument === destinationDocument && attr.ownerDocument === destinationDocument
                        && attr.ownerElement === owner, 'live adoption');
                    check(originalDocument.adoptNode(attr) === attr && attr.ownerDocument === originalDocument
                        && attr.ownerElement === owner, 'direct adoption changes only the Attr document');
                    check(Document.prototype.adoptNode.call(destinationDocument, attr) === attr
                        && attr.ownerDocument === destinationDocument && attr.ownerElement === owner,
                        'borrowed adoption uses receiving Document');
                    check(owner.removeAttributeNode(attr) === attr && attr.ownerElement === null, 'exact removal');
                    if (value === 'reject') return null;
                    check(replacementOwner.setAttributeNode(attr) === null && attr.ownerElement === replacementOwner, 'new nonsink owner');
                    return value + '!';
                }});
                acceptedAttr.value = 'move\ud800';
                check(acceptedAttr.value === 'move\ud800!' && replacementOwner.getAttributeNode('onclick') === acceptedAttr
                    && acceptedOwner.getAttributeNode('onclick') === null && Object.getPrototypeOf(acceptedAttr) === creationPrototype,
                    'verified write uses new owner and preserves creation prototype');
                let blocked = false;
                try { rejectedAttr.value = 'reject'; }
                catch (error) { blocked = error instanceof TypeError; }
                check(blocked && rejectedAttr.value === 'old' && rejectedAttr.ownerElement === null, 'original policy rejects after adoption');
                check(calls.length === 2 && calls[0] === 'move\ud800' && calls[1] === 'reject', 'callbacks run once');
            })()
        "#).map(drop)
    });
    if let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = &result {
        panic!(
            "adoption callback refused: {:?}, {:?}",
            failure.cause::<DocumentGuestEventError>(),
            failure.cause::<crate::DomSourceDriveError>()
        );
    }
    result.unwrap();
    assert_queued_trusted_types_violations(
        &host,
        &[(0, "trusted-types-sink", "Element onclick|reject")],
    );
    let snapshot = host.snapshot().unwrap();
    assert_eq!(
        snapshot.node(destination).unwrap().children(),
        &[receiver, accepted, rejected]
    );
    let accepted = snapshot.attr(accepted_attr).unwrap();
    assert_eq!(accepted.owner_document(), other.document());
    assert_eq!(accepted.owner_element(), Some(receiver));
    assert_eq!(
        accepted.value().code_units(),
        &[109, 111, 118, 101, 0xd800, 33]
    );
    let rejected = snapshot.attr(rejected_attr).unwrap();
    assert_eq!(rejected.owner_document(), other.document());
    assert_eq!(rejected.owner_element(), None);
    assert_eq!(rejected.value(), &"old".into());
}

pub(super) fn guest_meta_retry_preserves_accepted_first_policy() {
    use crate::{DomSourceDriveError, DomSourceRuntimeError};
    use surgeist_dom::{DomResource, DomSourceMetaCspError, DomSourcePhase};

    let cancel = Arc::new(AtomicBool::new(false));
    let limits = |units| scripting_dom_limits(units, 0).with_meta_processing_work(1_000_000);
    let (mut session, target) =
        hosted_session_with_dom(&cancel, 0, Dom::new(limits(200_000)).unwrap());
    let document = target.document();
    let snapshot = session.snapshot();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let head = snapshot.node(html).unwrap().children()[0];
    let mut batch = DomBatch::new();
    let fragment = batch.create_fragment(document);
    let larger = format!(
        "require-trusted-types-for 'script'; extension {}",
        "x".repeat(512)
    );
    let policies = ["trusted-types default", larger.as_str()];
    let metas = policies.map(|policy| {
        let meta = batch.create_element_ns(document, Namespace::html(), "meta");
        batch.set_attribute(&meta, "http-equiv", "content-security-policy");
        batch.set_attribute(&meta, "content", policy);
        batch.insert_before(&meta, &fragment, None);
        meta
    });
    let commit = session.dom().apply(batch, &cancel).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let [first, second] = metas.map(|meta| commit.created(&meta).unwrap().unwrap());
    session.enable_trusted_types_csp(&cancel).unwrap();
    let policy_owner = session.trusted_types_csp.as_ref().unwrap().clone();
    let baseline = session.dom().usage().string_code_units();
    // Admit the short first policy in the draft and refuse the larger list
    // when the second policy is installed.
    let ceiling = baseline + 128;
    session.dom().raw_mut().set_limits(limits(ceiling)).unwrap();
    let mut host = super::super::guest::tests::attach_public_nodes(
        session,
        target,
        cancel.clone(),
        5_000,
        20_000,
    );
    let permit = host.permit().unwrap();
    for (node, name) in [(head, "retryHead"), (fragment, "retryFragment")] {
        super::super::guest::tests::expose_public_node_for_source_test(
            &mut host, &permit, node, name,
        );
    }
    let before = host.snapshot().unwrap();
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "retryHead.appendChild(retryFragment)").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("expected the original second policy admission refusal: {result:?}");
    };
    assert!(
        matches!(failure.cause::<DomSourceDriveError>(), Some(DomSourceDriveError::Source(DomSourceRuntimeError::MetaCsp(DomSourceMetaCspError::Dom(DomError::ResourceLimit { resource: DomResource::StringCodeUnits, limit })))) if *limit == ceiling),
        "{:?}",
        failure.cause::<DomSourceDriveError>()
    );
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    assert_eq!(
        host.snapshot().unwrap().node(fragment).unwrap().children(),
        &[first, second]
    );
    let parked = host
        .with_native(|dom, _, _| dom.pending_source_edit().unwrap())
        .unwrap();
    assert_eq!(parked.phase(), DomSourcePhase::MetaCsp);
    let mut session = host.stop().into_session().unwrap();
    let mut source = session.source_execution();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::MetaCsp(request),
        ..
    } = source
        .resume(
            &mut session.dom(),
            parked.operation(),
            SOURCE,
            &mut services,
            &cancel,
        )
        .unwrap()
    else {
        panic!("the original second policy request must remain parked");
    };
    assert_eq!(request.subject(), Some(second));
    assert_eq!(request.document(), Some(document));
    assert_eq!(request.operation(), parked.operation());
    session.dom().raw_mut().set_limits(limits(200_000)).unwrap();
    assert!(
        matches!(source.resume(&mut session.dom(), parked.operation(), SOURCE, &mut services, &cancel).unwrap(), DomSourceDriveProgress::Pending { request: DomSourceRequest::MetaCsp(reissued), .. } if reissued == request)
    );
    session
        .service_trusted_types_meta_csp(request, SOURCE, &cancel)
        .unwrap();
    assert!(
        matches!(source.resume(&mut session.dom(), parked.operation(), SOURCE, &mut services, &cancel).unwrap(), DomSourceDriveProgress::Complete(done) if done.operation() == parked.operation())
    );
    assert!(session.dom().pending_source_edit().is_none());
    let after = session.snapshot();
    assert_eq!(after.node(head).unwrap().children(), &[first, second]);
    assert!(after.node(fragment).unwrap().children().is_empty());
    let document = after.document(document).unwrap();
    let processor = policy_owner.processor.borrow();
    let list = processor
        .document_policies_with(document.metadata().policy_container().csp(), &mut |_| {
            Ok::<(), Infallible>(())
        })
        .unwrap()
        .unwrap();
    assert_eq!(list.len(), 2);
    for (policy, expected) in list.iter().zip(policies) {
        assert_eq!(policy.serialized(), &expected.into());
    }
}

pub(super) fn guest_meta_without_processor_preserves_original_source() {
    use crate::{DomSourceDriveError, DomSourceRuntimeError};
    use surgeist_dom::{DomSourceMetaCspError, DomSourcePhase};

    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits.with_meta_processing_work(1_000_000)
    });
    let document = target.document();
    let snapshot = session.snapshot();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let head = snapshot.node(html).unwrap().children()[0];
    let mut batch = DomBatch::new();
    let meta = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "content-security-policy");
    batch.set_attribute(&meta, "content", "trusted-types default");
    let meta = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&meta)
        .unwrap()
        .unwrap();
    let mut host = super::super::guest::tests::attach_public_nodes(
        session,
        target,
        cancel.clone(),
        5_000,
        20_000,
    );
    let permit = host.permit().unwrap();
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host, &permit, head, "metaHead",
    );
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host,
        &permit,
        meta,
        "pendingMeta",
    );
    let original = host.snapshot().unwrap().stamp();
    let failure = host.in_document(&permit, |scope| {
        evaluate(scope, "metaHead.appendChild(pendingMeta)").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = failure else {
        panic!("missing policy authority must remain a host refusal: {failure:?}");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::MetaCsp(
            DomSourceMetaCspError::PolicyUnavailable
        )))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), original);
    assert_eq!(host.snapshot().unwrap().node(meta).unwrap().parent(), None);
    let status = host
        .with_native(|dom, _, _| dom.pending_source_edit().unwrap())
        .unwrap();
    assert_eq!(status.phase(), DomSourcePhase::MetaCsp);
    assert!(matches!(
        host.in_document(&permit, |_| Ok(())),
        Err(DocumentScriptError::Script(Error::HostFailure(_)))
    ));
    let mut session = host.stop().into_session().unwrap();
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        status.operation()
    );
    session.enable_trusted_types_csp(&cancel).unwrap();
    let mut source = session.source_execution();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::MetaCsp(request),
        ..
    } = source
        .resume(
            &mut session.dom(),
            status.operation(),
            SOURCE,
            &mut services,
            &cancel,
        )
        .unwrap()
    else {
        panic!("the original Meta request must be recoverable");
    };
    assert_eq!(request.operation(), status.operation());
    assert_eq!(request.subject(), Some(meta));
    session
        .service_trusted_types_meta_csp(request, SOURCE, &cancel)
        .unwrap();
    assert!(
        matches!(source.resume(&mut session.dom(), status.operation(), SOURCE, &mut services, &cancel).unwrap(), DomSourceDriveProgress::Complete(done) if done.operation() == status.operation())
    );
    assert_eq!(session.snapshot().node(meta).unwrap().parent(), Some(head));
    assert!(session.dom().pending_source_edit().is_none());
}

pub(super) fn guest_tree_edits_deliver_original_meta_policies() {
    use crate::trusted_types_csp::PolicyDelivery;
    use surgeist_dom::DomDocumentCspState;

    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits.with_meta_processing_work(1_000_000)
    });
    let element = insert_hosted_element(&mut session, target, "div", &cancel);
    let document = target.document();
    let snapshot = session.snapshot();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let head = snapshot.node(html).unwrap().children()[0];
    let mut batch = DomBatch::new();
    let first = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&first, "http-equiv", "content-security-policy");
    batch.set_attribute(&first, "content", "trusted-types default");
    let second = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&second, "http-equiv", "content-security-policy");
    batch.set_attribute(&second, "content", "require-trusted-types-for 'script'");
    let fragment = batch.create_fragment(document);
    batch.insert_before(&first, &fragment, None);
    batch.insert_before(&second, &fragment, None);
    let commit = session.dom().apply(batch, &cancel).unwrap();
    let first = commit.created(&first).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    session.enable_trusted_types_csp(&cancel).unwrap();
    let policy_owner = session.trusted_types_csp.as_ref().unwrap().clone();
    let mut host =
        super::super::guest::tests::attach_public_nodes(session, target, cancel, 5_000, 20_000);
    let permit = host.permit().unwrap();
    for (node, name) in [
        (head, "policyHead"),
        (element, "policyTarget"),
        (fragment, "policyFragment"),
        (first, "firstMeta"),
        (second, "secondMeta"),
    ] {
        super::super::guest::tests::expose_public_node_for_source_test(
            &mut host, &permit, node, name,
        );
    }
    host.in_document(&permit, |scope| {
        evaluate(scope, r#"
            (() => {
                const check = (ok, label) => { if (!ok) throw new Error(label); };
                check(policyHead.insertBefore(policyFragment, null) === policyFragment, 'exact fragment return');
                check(firstMeta.parentNode === policyHead && secondMeta.parentNode === policyHead
                    && policyFragment.textContent === '', 'native fragment insertion');
                let blocked = false;
                try { policyTarget.setAttribute('onclick', 'blocked'); }
                catch (error) { blocked = error instanceof TypeError; }
                check(blocked, 'the original Meta policy must already be active');
                trustedTypes.createPolicy('default', {createScript(value) {
                    if (globalThis.insertNestedMeta) {
                        if (value === 'abandoned') policyTarget.setAttribute('onload', 'inner');
                        else if (value === 'inner') policyHead.appendChild(nestedMeta);
                    }
                    return value + '!';
                }});
                policyTarget.setAttribute('onclick', 'exact\ud800');
                const text = document.createTextNode('text');
                check(policyTarget.appendChild(text) === text, 'exact append return');
                const replacement = document.createComment('replacement');
                check(policyTarget.replaceChild(replacement, text) === text
                    && text.parentNode === null && replacement.parentNode === policyTarget, 'exact replace return');
                const throws = (action, name) => {
                    try { action(); } catch (error) { return error.name === name; }
                    return false;
                };
                check(throws(() => policyHead.removeChild(replacement), 'NotFoundError')
                    && throws(() => policyTarget.appendChild(policyTarget), 'HierarchyRequestError')
                    && throws(() => policyTarget.insertBefore(text), 'TypeError')
                    && throws(() => policyTarget.appendChild(new Proxy(text, {})), 'TypeError'), 'precise operation errors');
                check(replacement.parentNode === policyTarget, 'wrong-parent removal does not detach');
                check(policyTarget.removeChild(replacement) === replacement && replacement.parentNode === null, 'exact remove return');
            })()
        "#).map(drop)
    }).unwrap();
    let snapshot = host.snapshot().unwrap();
    assert_eq!(snapshot.node(head).unwrap().children(), &[first, second]);
    let actual = snapshot
        .node(element)
        .unwrap()
        .attribute(&"onclick".into())
        .unwrap();
    assert_eq!(
        actual.code_units(),
        &"exact"
            .encode_utf16()
            .chain([0xd800, u16::from(b'!')])
            .collect::<Vec<_>>()
    );
    let document = snapshot.document(document).unwrap();
    let state = document.metadata().policy_container().csp();
    assert!(matches!(state, DomDocumentCspState::Associated(_)));
    let processor = policy_owner.processor.borrow();
    let list = processor
        .document_policies_with(state, &mut |_| Ok::<(), Infallible>(()))
        .unwrap()
        .unwrap();
    assert_eq!(list.len(), 2);
    assert!(
        list.iter()
            .all(|policy| policy.delivery() == PolicyDelivery::Meta)
    );
    drop(processor);
    nested_meta_refusal_preserves_original_parent(host, permit, policy_owner, head, element);
}

fn nested_meta_refusal_preserves_original_parent(
    mut host: DocumentScriptHost,
    permit: DocumentScriptPermit,
    policy_owner: Rc<HostedPolicyProcessor>,
    head: NodeId,
    element: NodeId,
) {
    use crate::{DomSourceDriveError, DomSourceRuntimeError};
    use surgeist_dom::{DomSourceMetaCspError, DomSourcePhase};

    let cancel = host.native.cancellation.clone();
    let meta = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let meta = batch.create_element_ns(permit.document(), Namespace::html(), "meta");
            batch.set_attribute(&meta, "http-equiv", "content-security-policy");
            batch.set_attribute(&meta, "content", "trusted-types default");
            dom.apply(batch, &cancel)
                .unwrap()
                .created(&meta)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host,
        &permit,
        meta,
        "nestedMeta",
    );
    let before = host.snapshot().unwrap();
    let occupied = policy_owner.processor.borrow();
    let failure = host.in_document(&permit, |scope| {
        evaluate(
            scope,
            "globalThis.insertNestedMeta = true; policyTarget.setAttribute('onclick', 'abandoned')",
        )
        .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = failure else {
        panic!("expected exact nested processor refusal: {failure:?}");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::MetaCsp(
            DomSourceMetaCspError::IssuerBusy
        )))
    ));
    drop(occupied);
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    let mut session = host.stop().into_session().unwrap();
    let child = session.dom().pending_source_edit().unwrap();
    assert_eq!(child.phase(), DomSourcePhase::MetaCsp);
    let parent = child.parent_operation().unwrap();
    let mut source = session.source_execution();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::MetaCsp(request),
        ..
    } = source
        .resume(
            &mut session.dom(),
            child.operation(),
            SOURCE,
            &mut services,
            &cancel,
        )
        .unwrap()
    else {
        panic!("expected original nested Meta request");
    };
    session
        .service_trusted_types_meta_csp(request, SOURCE, &cancel)
        .unwrap();
    assert!(
        matches!(source.resume(&mut session.dom(), child.operation(), SOURCE, &mut services, &cancel).unwrap(), DomSourceDriveProgress::Complete(done) if done.operation() == child.operation())
    );
    assert!(
        matches!(source.resume(&mut session.dom(), parent, SOURCE, &mut services, &cancel).unwrap(), DomSourceDriveProgress::Complete(done) if done.operation() == parent && done.cancellation_requested())
    );
    let outer = session.dom().pending_source_edit().unwrap().operation();
    assert!(
        matches!(source.resume(&mut session.dom(), outer, SOURCE, &mut services, &cancel).unwrap(), DomSourceDriveProgress::Complete(done) if done.operation() == outer && done.cancellation_requested())
    );
    let after = session.snapshot();
    assert_eq!(after.node(meta).unwrap().parent(), Some(head));
    assert_eq!(
        after.node(element).unwrap().attribute(&"onclick".into()),
        before.node(element).unwrap().attribute(&"onclick".into())
    );
    assert_eq!(
        after.node(element).unwrap().attribute(&"onload".into()),
        None
    );
    assert!(session.dom().pending_source_edit().is_none());
}

use crate::trusted_types_csp::{AttributePolicies, AttributePolicyError, PolicyFactory};
use std::convert::Infallible;
use surgeist_bindings_js::conversion::{ConversionAccount, ResourceTable, WorkTable};
use surgeist_bindings_js::{TrustedPolicyCallback, TrustedTypeKind};
use surgeist_dom::{DocumentString, DomSourceAttributeVerificationOutcome, DomSourceProgress};
use surgeist_html::security::csp::{
    HtmlCspError, HtmlCspLayout, HtmlCspProcessorProfile, HtmlCspStorage, HtmlMetaCspOutcome,
    HtmlMetaCspProcessor, HtmlMetaCspRequest,
};

struct StopAfterCspPreparation<'a> {
    processor: &'a mut crate::trusted_types_csp::PolicyProcessor,
    cancel: &'a AtomicBool,
    calls: usize,
}

impl HtmlMetaCspProcessor for StopAfterCspPreparation<'_> {
    type Error = crate::trusted_types_csp::PolicyListCapacity;

    fn profile(&self) -> &HtmlCspProcessorProfile {
        self.processor.profile()
    }

    fn prepare<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<Self::Error>, HtmlCspError<E>> {
        self.calls += 1;
        let outcome = self.processor.prepare(request, admit, release)?;
        self.cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(outcome)
    }
}

pub(super) fn hosted_meta_source_publishes_ordered_csp_for_the_registered_attribute_verifier() {
    use crate::trusted_types_csp::{PolicyDelivery, PolicyProcessor};
    use crate::{DomSourceDriveProgress, DomSourceServices, DomSourceUnavailable};
    use surgeist_dom::{DomDocumentCspState, DomSourceMetaCspError};

    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits.with_meta_processing_work(1_000_000)
    });
    let element = insert_hosted_element(&mut session, target, "div", &cancel);
    let snapshot = session.snapshot();
    let document = target.document();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let head = snapshot.node(html).unwrap().children()[0];
    let mut batch = DomBatch::new();
    let mut metas = Vec::new();
    for policy in [
        "trusted-types default; report-uri /ignored",
        "require-trusted-types-for 'script'",
    ] {
        let meta = batch.create_element_ns(document, Namespace::html(), "meta");
        batch.set_attribute(&meta, "http-equiv", "content-security-policy");
        batch.set_attribute(&meta, "content", policy);
        metas.push(meta);
    }
    let formed = session.dom().apply(batch, &cancel).unwrap();
    let mut allow = |_| Ok::<(), Infallible>(());
    session.enable_trusted_types_csp(&cancel).unwrap();
    let selected = session.trusted_types_csp.as_ref().unwrap().clone();
    let mut other = PolicyProcessor::new_with(&mut allow, &mut |_| unreachable!()).unwrap();
    let mut source = session.source_execution();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let mut policy_units = 0;
    for (index, ticket) in metas.iter().enumerate() {
        let meta = formed.created(ticket).unwrap().unwrap();
        let before = session.snapshot().stamp();
        let before_units = session.dom().usage().string_code_units();
        let DomSourceDriveProgress::Pending {
            request: DomSourceRequest::MetaCsp(request),
            reason: DomSourceUnavailable::MetaCsp,
        } = source
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(meta, head, None),
                SOURCE,
                &mut services,
                &cancel,
            )
            .unwrap()
        else {
            panic!("expected original Meta policy request");
        };
        assert_eq!(session.snapshot().stamp(), before);
        if index == 0 {
            let mut processor = selected.processor.borrow_mut();
            let mut stopped = StopAfterCspPreparation {
                processor: &mut processor,
                cancel: &cancel,
                calls: 0,
            };
            let mut view = session.dom();
            {
                let mut lease = view
                    .resume_source_edit(request.operation(), SOURCE, &cancel)
                    .unwrap();
                assert!(matches!(
                    lease.process_meta_csp(request, &mut stopped, &cancel),
                    Err(DomSourceMetaCspError::Dom(DomError::Cancelled))
                ));
                assert_eq!(lease.snapshot().stamp(), before);
            }
            cancel.store(false, std::sync::atomic::Ordering::Relaxed);
            let mut lease = view
                .resume_source_edit(request.operation(), SOURCE, &cancel)
                .unwrap();
            lease
                .process_meta_csp(request, &mut stopped, &cancel)
                .unwrap();
            assert_eq!(stopped.calls, 1);
        } else {
            assert!(matches!(
                source.service_meta_csp(&mut session.dom(), request, &mut other, SOURCE, &cancel),
                Err(crate::DomSourceRuntimeError::MetaCsp(
                    DomSourceMetaCspError::Protocol(_)
                ))
            ));
            assert_eq!(session.snapshot().stamp(), before);
            session
                .service_trusted_types_meta_csp(request, SOURCE, &cancel)
                .unwrap();
        }
        assert_eq!(session.snapshot().stamp(), before);
        assert!(matches!(
            source
                .resume(
                    &mut session.dom(),
                    request.operation(),
                    SOURCE,
                    &mut services,
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let snapshot = session.snapshot();
        assert_eq!(snapshot.node(meta).unwrap().parent(), Some(head));
        let document = snapshot.document(document).unwrap();
        let state = document.metadata().policy_container().csp();
        let DomDocumentCspState::Associated(association) = state else {
            panic!("the real published document must retain the policy");
        };
        assert!(association.latest().is_some());
        let next_units = association
            .retained_layout()
            .unwrap()
            .storage()
            .utf16_units();
        assert_eq!(
            session.dom().usage().string_code_units(),
            before_units - policy_units + next_units
        );
        policy_units = next_units;
        let list = selected
            .processor
            .borrow()
            .document_policies_with(state, &mut allow)
            .unwrap()
            .unwrap();
        assert_eq!(list.len(), index + 1);
        assert!(
            list.iter()
                .all(|policy| policy.delivery() == PolicyDelivery::Meta)
        );
    }

    let host = enable(session, target, cancel.clone());
    let registration = &host.engine.as_ref().unwrap().registration;
    let mut session = host.native.session.borrow_mut();
    let mut view = session.dom();
    let mut lease = view
        .begin_source_edit(
            DomSourceEdit::set_attribute(element, "onclick", "input"),
            SOURCE,
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::AttributeVerification(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("expected actual registered attribute verifier");
    };
    let processor = selected.processor.borrow();
    let policies =
        AttributePolicies::read(&lease, request, registration, Some(&processor), &cancel).unwrap();
    assert!(policies.requires_default_policy(&cancel).unwrap());
    let mut mismatch = policies.mismatch();
    let violation = mismatch.next_violation(&cancel).unwrap().unwrap();
    assert_eq!(violation.policy_index(), 1);
    assert_eq!(mismatch.blocked(), Some(true));
    drop(processor);
    lease.request_cancel();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    drop(session);

    let session = host.stop().into_session().unwrap();
    let mut host = super::super::guest::tests::attach_public_nodes(
        session,
        target,
        cancel.clone(),
        5_000,
        20_000,
    );
    let permit = host.permit().unwrap();
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host,
        &permit,
        element,
        "protectedTarget",
    );
    let result = host.in_document(&permit, |scope| {
        let expression: Vec<_> = r#"
            (() => {
                const marker = {};
                globalThis.cspEvents = [];
                document.addEventListener('securitypolicyviolation', event => {
                    if (!(event instanceof SecurityPolicyViolationEvent) || !event.isTrusted
                        || !event.bubbles || !event.composed || event.cancelable
                        || event.target !== document || event.currentTarget !== document) {
                        throw new Error('incorrect native CSP event');
                    }
                    cspEvents.push(event);
                });
                globalThis.cspBubbles = 0;
                EventTarget.prototype.addEventListener.call(window, 'securitypolicyviolation', event => {
                    if (cspEvents[cspBubbles] !== event || event.currentTarget !== window
                        || event.target !== document) throw new Error('incorrect CSP path or identity');
                    cspBubbles++;
                });
                try { protectedTarget.setAttribute('onclick', 'missing'); return false; }
                catch (error) { if (!(error instanceof TypeError)) return false; }
                const policy = trustedTypes.createPolicy('default', {
                    createScript(value, type, sink) {
                        if (type !== 'TrustedScript' || sink !== 'Element onclick') throw new Error('incorrect sink');
                        if (protectedTarget.nodeType !== 1) throw new Error('unavailable native read');
                        if (value === 'expose-owner') {
                            globalThis.firstOwner = hiddenOwnerAttr.ownerElement;
                            if (!(firstOwner instanceof Element)) throw new Error('unavailable Attr owner');
                        }
                        if (globalThis.attachmentProbe) globalThis.attachmentProbe(value);
                        protectedTarget.setAttribute('data-entered', value);
                        if (value === 'throw') throw marker;
                        if (value === 'empty') return null;
                        return {toString() {
                            protectedTarget.setAttribute('data-coerced', 'yes');
                            if (value === 'later') stopAfterAttributeCallback();
                            return value + '!';
                        }};
                    }
                });
                if (policy.name !== 'default' || trustedTypes.defaultPolicy !== policy) return false;
                protectedTarget.setAttribute('onclick', 'first\ud800');
                try { protectedTarget.setAttributeNS(null, 'onclick', 'throw'); return false; }
                catch (error) { if (error !== marker) return false; }
                try { protectedTarget.setAttributeNS(null, 'onclick', 'empty'); return false; }
                catch (error) { if (!(error instanceof TypeError)) return false; }
                try { trustedTypes.createPolicy('denied'); }
                catch (error) { return error instanceof TypeError; }
                return false;
            })()
        "#.encode_utf16().collect();
        let result = scope.evaluate(&expression)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    });
    if let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = &result {
        panic!(
            "the guest attribute verifier refused: {:?}",
            failure.cause::<DocumentGuestEventError>()
        );
    }
    result.expect("the actual guest factory and setter must use the hosted Document's CSP");
    let snapshot = host.snapshot().unwrap();
    let node = snapshot.node(element).unwrap();
    assert_eq!(
        node.attribute(&"onclick".into()).unwrap().code_units(),
        &"first"
            .encode_utf16()
            .chain([0xd800, u16::from(b'!')])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        node.attribute(&"data-entered".into()),
        Some(&"empty".into())
    );
    assert_eq!(node.attribute(&"data-coerced".into()), Some(&"yes".into()));
    super::super::guest::tests::assert_queued_trusted_types_violations(
        &host,
        &[
            (1, "trusted-types-sink", "Element onclick|missing"),
            (1, "trusted-types-sink", "Element onclick|empty"),
            (0, "trusted-types-policy", "denied"),
        ],
    );

    host.in_document(&permit, |scope| {
        let source = "globalThis.detachedDocument = new Document()";
        scope
            .evaluate(&source.encode_utf16().collect::<Vec<_>>())
            .map(drop)
    })
    .unwrap();
    let after_constructor = host.snapshot().unwrap();
    let detached = after_constructor
        .nodes()
        .filter_map(|node| after_constructor.node(node).unwrap().as_document())
        .map(|document| document.id())
        .find(|document| snapshot.document(*document).is_err())
        .expect("the guest constructor must publish its own native Document");
    let detached_element = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let ticket = batch.create_element_ns(detached, Namespace::html(), "div");
            dom.apply(batch, &cancel)
                .unwrap()
                .created(&ticket)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    assert_eq!(
        host.snapshot()
            .unwrap()
            .relevant_global(detached_element)
            .unwrap(),
        Some(target.global())
    );
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host,
        &permit,
        detached_element,
        "detachedProtectedTarget",
    );
    host.in_document(&permit, |scope| {
        let source = r#"
            (() => {
                if (detachedProtectedTarget.ownerDocument !== detachedDocument) return false;
                try { detachedProtectedTarget.setAttribute('onclick', 'empty'); }
                catch (error) { return error instanceof TypeError; }
                return false;
            })()
        "#;
        let result = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    super::super::guest::tests::assert_queued_trusted_types_violations(
        &host,
        &[
            (1, "trusted-types-sink", "Element onclick|missing"),
            (1, "trusted-types-sink", "Element onclick|empty"),
            (0, "trusted-types-policy", "denied"),
            (1, "trusted-types-sink", "Element onclick|empty"),
        ],
    );
    assert_csp_event_counts(&mut host, &permit, 0);
    let first_slice = DomEventDispatchLimits {
        callbacks: 1,
        ..EVENT.dispatch
    };
    let progress = host
        .run_attached_csp_violation_task(&permit, first_slice)
        .unwrap();
    assert!(
        matches!(
            progress,
            DocumentCspViolationProgress::Retained(crate::DomEventDispatchOutcome::Retained(
                crate::DomEventResource::Callbacks
            ))
        ),
        "{progress:?}"
    );
    for count in 1..=4 {
        let progress = host
            .run_attached_csp_violation_task(&permit, EVENT.dispatch)
            .unwrap();
        let DocumentCspViolationProgress::Complete(receipt) = progress else {
            panic!("expected one native CSP violation task to complete: {progress:?}");
        };
        assert!(receipt.failures().is_empty());
        assert_csp_event_counts(&mut host, &permit, count);
    }
    assert!(matches!(
        host.run_attached_csp_violation_task(&permit, EVENT.dispatch)
            .unwrap(),
        DocumentCspViolationProgress::Empty
    ));
    host.in_document(&permit, |scope| {
        let source = r#"
            (() => {
                const expected = [
                    ['trusted-types-sink', 'require-trusted-types-for', 'Element onclick|missing'],
                    ['trusted-types-sink', 'require-trusted-types-for', 'Element onclick|empty'],
                    ['trusted-types-policy', 'trusted-types', 'denied'],
                    ['trusted-types-sink', 'require-trusted-types-for', 'Element onclick|empty']
                ];
                if (!cspEvents.every((event, index) => {
                    const [resource, directive, sample] = expected[index];
                    const policy = index === 2 ? 'trusted-types default' : "require-trusted-types-for 'script'";
                    return event.blockedURI === resource && event.effectiveDirective === directive
                        && event.violatedDirective === directive && event.sample === sample
                        && event.originalPolicy === policy && event.documentURI === 'about'
                        && event.referrer === '' && event.sourceFile === '' && event.statusCode === 0
                        && event.lineNumber === 0 && event.columnNumber === 0 && event.disposition === 'enforce';
                })) return false;
                const authored = new SecurityPolicyViolationEvent('authored', {
                    documentURI: '\ud800', sample: '\ud800', statusCode: -1, lineNumber: -1,
                    disposition: 'report', effectiveDirective: 'effective', violatedDirective: 'violated'
                });
                return authored instanceof Event && !authored.isTrusted
                    && authored.type === 'authored' && authored.documentURI === '\ufffd'
                    && authored.sample === '\ud800' && authored.statusCode === 65535
                    && authored.lineNumber === 4294967295 && authored.disposition === 'report'
                    && authored.effectiveDirective === 'effective' && authored.violatedDirective === 'violated';
            })()
        "#;
        let value = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
        assert!(scope.boolean(&value)?);
        Ok(())
    }).unwrap();

    let snapshot = host.snapshot().unwrap();
    let handler = snapshot
        .node(element)
        .unwrap()
        .attributes()
        .unwrap()
        .get_named_item(&"onclick".into())
        .unwrap()
        .id()
        .node();
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host,
        &permit,
        handler,
        "protectedAttr",
    );
    host.in_document(&permit, |scope| {
        let source = r#"
            protectedAttr.value = 'attr-write\ud800';
            protectedAttr.ownerElement === protectedTarget && protectedAttr.value === 'attr-write\ud800!'
        "#;
        let value = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
        assert!(scope.boolean(&value)?);
        Ok(())
    }).unwrap();
    assert_eq!(
        host.snapshot()
            .unwrap()
            .attr(handler)
            .unwrap()
            .value()
            .code_units(),
        &"attr-write"
            .encode_utf16()
            .chain([0xd800, u16::from(b'!')])
            .collect::<Vec<_>>(),
    );
    let hidden_owner = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let ticket = batch.create_element_ns(document, Namespace::html(), "span");
            batch.set_attribute(&ticket, "onclick", "original");
            dom.apply(batch, &cancel)
                .unwrap()
                .created(&ticket)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    let snapshot = host.snapshot().unwrap();
    let hidden_attr = snapshot
        .node(hidden_owner)
        .unwrap()
        .attributes()
        .unwrap()
        .get_named_item(&"onclick".into())
        .unwrap()
        .id()
        .node();
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host,
        &permit,
        hidden_attr,
        "hiddenOwnerAttr",
    );
    let result = host.in_document(&permit, |scope| {
        let source = r#"
            hiddenOwnerAttr.value = 'expose-owner';
            hiddenOwnerAttr.ownerElement === firstOwner && hiddenOwnerAttr.value === 'expose-owner!'
        "#;
        let value = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
        assert!(scope.boolean(&value)?);
        Ok(())
    });
    if let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = &result {
        panic!(
            "the original policy could not project the live Attr owner: {:?}",
            failure.cause::<DocumentGuestEventError>()
        );
    }
    result.unwrap();

    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        let inner = batch.create_element_ns(document, Namespace::html(), "span");
        let first = batch.create_text(document, "outer");
        let last = batch.create_text(document, DocumentString::from_utf16([0xdfff, 0]));
        batch.insert_before(&first, element, None);
        batch.insert_before(&inner, element, None);
        batch.insert_before(&last, &inner, None);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let source = r#"
            (() => {
                const inputs = [];
                globalThis.attachmentProbe = value => {
                    inputs.push(value);
                    if (protectedTarget.textContent !== 'outer\udfff\0')
                        throw new Error('wrong live descendant text');
                };
                protectedAttr.nodeValue = 'node\ud800';
                if (protectedAttr.textContent !== 'node\ud800!') return false;
                protectedAttr.textContent = 'content\0';
                delete globalThis.attachmentProbe;
                return protectedAttr.value === 'content\0!' && protectedAttr.nodeValue === protectedAttr.value
                    && inputs.length === 2 && inputs[0] === 'node\ud800' && inputs[1] === 'content\0';
            })()
        "#;
        let result = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();

    host.in_document(&permit, |scope| {
        let script = r#"
            (() => {
                const inputs = [];
                const original = protectedAttr.value;
                const moving = hiddenOwnerAttr.value;
                globalThis.attachmentProbe = value => inputs.push(value);
                if (protectedTarget.setAttributeNode(protectedAttr) !== protectedAttr
                    || protectedAttr.value !== original) return false;
                try { firstOwner.setAttributeNodeNS(protectedAttr); return false; }
                catch (error) { if (error.name !== 'InUseAttributeError') return false; }
                if (inputs.length !== 2 || inputs[0] !== original || inputs[1] !== original
                    || protectedAttr.ownerElement !== protectedTarget) return false;
                globalThis.attachmentProbe = value => {
                    inputs.push(value);
                    try {
                        protectedTarget.removeAttributeNode(hiddenOwnerAttr);
                        throw new Error('accepted removal from a different owner');
                    } catch (error) {
                        if (error.name !== 'NotFoundError') throw error;
                    }
                    if (firstOwner.getAttributeNode('ONCLICK') !== hiddenOwnerAttr
                        || firstOwner.removeAttributeNode(hiddenOwnerAttr) !== hiddenOwnerAttr)
                        throw new Error('nested removal lost the original Attr');
                };
                const replaced = protectedTarget.setAttributeNodeNS(hiddenOwnerAttr);
                delete globalThis.attachmentProbe;
                return replaced === protectedAttr && protectedAttr.ownerElement === null
                    && hiddenOwnerAttr.ownerElement === protectedTarget
                    && hiddenOwnerAttr.value === moving + '!'
                    && firstOwner.getAttributeNode('onclick') === null
                    && protectedTarget.getAttributeNode('onclick') === hiddenOwnerAttr
                    && inputs.length === 3 && inputs[2] === moving;
            })()
        "#;
        let result = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();

    let factory_result = host.in_document(&permit, |scope| {
        let script = r#"
            (() => {
                let replacement;
                let calls = 0;
                const policy = trustedTypes.defaultPolicy;
                globalThis.attachmentProbe = value => {
                    if (value !== 'factory') return;
                    calls++;
                    const plain = document.createAttribute('DATA-NESTED');
                    const fragment = document.createDocumentFragment();
                    const text = document.createTextNode('nested\ud800');
                    const comment = document.createComment('comment');
                    fragment.appendChild(text);
                    fragment.appendChild(comment);
                    plain.value = fragment.textContent;
                    firstOwner.setAttributeNode(plain);
                    replacement = document.createAttributeNS(null, 'onclick');
                    replacement.value = policy.createScript('nested\udfff', 'TrustedScript', 'Element onclick');
                    if (protectedTarget.setAttributeNode(replacement) !== hiddenOwnerAttr)
                        throw new Error('nested attachment selected the wrong Attr');
                };
                try {
                    protectedTarget.setAttribute('onclick', 'factory');
                    return calls === 1 && replacement instanceof Attr
                        && protectedTarget.getAttributeNode('onclick') === replacement
                        && replacement.value === 'factory!'
                        && hiddenOwnerAttr.ownerElement === null
                        && firstOwner.getAttribute('data-nested') === 'nested\ud800';
                } finally {
                    delete globalThis.attachmentProbe;
                }
            })()
        "#;
        let result = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
        assert!(scope.boolean(&result)?);
        Ok(())
    });
    if let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = &factory_result {
        panic!(
            "the original policy could not create a detached node: {:?}",
            failure.cause::<DocumentGuestEventError>()
        );
    }
    factory_result.unwrap();

    let calls = pause_guest_attribute(
        &mut host,
        &permit,
        &cancel,
        "protectedTarget.setAttribute('onclick', 'later')",
    );
    assert!(matches!(
        host.resume_pending_attribute_write(&permit, SOURCE),
        Err(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::Cancelled))
        ))
    ));
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    let parked = host
        .with_native(|dom, _, _| dom.pending_source_edit())
        .unwrap()
        .expect("the converted result must remain available after repeated refusal");
    let completed = complete_guest_attribute(host.resume_pending_attribute_write(&permit, SOURCE));
    assert_eq!(completed.operation(), parked.operation());
    assert_eq!(calls.get(), 1);
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(element)
            .unwrap()
            .attribute(&"onclick".into()),
        Some(&"later!".into())
    );
    assert!(
        host.with_native(|dom, _, _| dom.pending_source_edit())
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        host.in_document(&permit, |_| Ok(())),
        Err(DocumentScriptError::Script(Error::HostFailure(_)))
    ));

    let session = host.stop().into_session().unwrap();
    let (mut host, permit) =
        attach_attribute_recovery_fixture(session, target, element, &cancel, 4_500);
    let result = host.in_document(&permit, |scope| {
        let script = "protectedTarget.setAttribute('onclick', 'outer')";
        scope
            .evaluate(&script.encode_utf16().collect::<Vec<_>>())
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("expected Source work refusal after the nested result, got {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(
                DomError::SourceWorkLimit { limit: 4_500 }
            ))
        ))
    ));
    let child = host
        .with_native(|dom, _, _| dom.pending_source_edit())
        .unwrap()
        .unwrap();
    let parent = child
        .parent_operation()
        .expect("the child is inside the original verifier");
    assert!(matches!(
        host.with_native(|dom, _, _| dom
            .resume_source_edit(child.operation(), SOURCE, &cancel)
            .map(drop))
            .unwrap(),
        Err(DomError::SourceWorkLimit { limit: 4_500 })
    ));
    let completed = complete_guest_attribute(host.resume_pending_attribute_write(&permit, SOURCE));
    assert_eq!(completed.operation(), child.operation());
    let abandoned = complete_guest_attribute(host.resume_pending_attribute_write(&permit, SOURCE));
    assert_eq!(abandoned.operation(), parent);
    assert!(abandoned.cancellation_requested());
    let expected_nested = DocumentString::from_utf16(vec![u16::from(b'z'); 7_500]);
    let snapshot = host.snapshot().unwrap();
    let node = snapshot.node(element).unwrap();
    assert_eq!(node.attribute(&"onclick".into()), Some(&"later!".into()));
    assert_eq!(node.attribute(&"onload".into()), Some(&expected_nested));

    let session = host.stop().into_session().unwrap();
    let (mut host, permit) =
        attach_attribute_recovery_fixture(session, target, element, &cancel, 20_000);
    let calls = pause_guest_attribute(
        &mut host,
        &permit,
        &cancel,
        "protectedTarget.setAttribute('onload', 'unpublished')",
    );
    let mut stopped = host.stop().into_session().unwrap_err();
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    let abandoned = complete_guest_attribute(stopped.abandon_pending_attribute_write(SOURCE));
    assert!(abandoned.cancellation_requested());
    let mut session = stopped.into_session().unwrap();
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(
        session
            .snapshot()
            .node(element)
            .unwrap()
            .attribute(&"onload".into()),
        Some(&expected_nested)
    );
    assert_eq!(calls.get(), 1);

    let mut batch = DomBatch::new();
    let meta = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "content-security-policy");
    let policy = format!(
        "require-trusted-types-for 'script'; extension {}",
        "x".repeat(2_000)
    );
    batch.set_attribute(&meta, "content", policy.as_str());
    let meta = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&meta)
        .unwrap()
        .unwrap();
    let mut source = session.source_execution();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::MetaCsp(request),
        ..
    } = source
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(meta, head, None),
            SOURCE,
            &mut services,
            &cancel,
        )
        .unwrap()
    else {
        panic!("the third real Meta policy must reach its native service");
    };
    session
        .service_trusted_types_meta_csp(request, SOURCE, &cancel)
        .unwrap();
    assert!(matches!(
        source
            .resume(
                &mut session.dom(),
                request.operation(),
                SOURCE,
                &mut services,
                &cancel
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));

    let (mut host, permit) =
        attach_attribute_recovery_fixture(session, target, element, &cancel, 20_000);
    let calls = pause_guest_attribute(
        &mut host,
        &permit,
        &cancel,
        "protectedTarget.setAttribute('onclick', 'null-later')",
    );
    super::super::guest::tests::assert_queued_trusted_types_violations(&host, &[]);
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    let parked = host
        .with_native(|dom, _, _| dom.pending_source_edit())
        .unwrap()
        .expect("a null callback result must survive refusal before its mismatch reports");
    host.with_native(|dom, _, _| {
        let stamp = dom.snapshot().stamp();
        let mut lease = dom
            .resume_source_edit(parked.operation(), SOURCE, &cancel)
            .unwrap();
        assert!(matches!(
            lease.process_meta_csp(request, &mut other, &cancel),
            Err(DomSourceMetaCspError::Dom(DomError::InvalidSourceOperation))
        ));
        assert_eq!(lease.status().operation(), parked.operation());
        assert_eq!(lease.snapshot().stamp(), stamp);
    })
    .unwrap();
    let partial = DomSourceLimits::new(32_000_000, 1_000_000, 1_000, 128, 4096, 4096, 8);
    let refusal = host.resume_pending_attribute_write(&permit, partial);
    assert!(
        matches!(
            refusal,
            Err(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(
                    DomError::SourceWorkLimit { limit: 1_000 }
                ))
            ))
        ),
        "{refusal:?}"
    );
    super::super::guest::tests::assert_queued_trusted_types_violations(
        &host,
        &[(1, "trusted-types-sink", "Element onclick|null-later")],
    );
    let progress = host
        .resume_pending_attribute_write(&permit, SOURCE)
        .unwrap()
        .unwrap();
    let DomSourceDriveProgress::Failed(failure) = progress else {
        panic!("expected the original enforcing CSP rejection, got {progress:?}");
    };
    assert_eq!(failure.operation(), parked.operation());
    assert_eq!(
        failure.error(),
        &DomError::SourceAttributeVerificationRejected
    );
    assert_eq!(calls.get(), 1);
    assert!(
        host.with_native(|dom, _, _| dom.pending_source_edit())
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        host.resume_pending_attribute_write(&permit, SOURCE),
        Err(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::SourceAccess)
        ))
    ));
    super::super::guest::tests::assert_queued_trusted_types_violations(
        &host,
        &[
            (1, "trusted-types-sink", "Element onclick|null-later"),
            (2, "trusted-types-sink", "Element onclick|null-later"),
        ],
    );
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(element)
            .unwrap()
            .attribute(&"onclick".into()),
        Some(&"later!".into()),
    );
}

fn assert_csp_event_counts(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    count: usize,
) {
    host.in_document(permit, |scope| {
        let source = format!("cspEvents.length === {count} && cspBubbles === {count}");
        let value = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
        assert!(scope.boolean(&value)?);
        Ok(())
    })
    .unwrap();
}

fn attach_attribute_recovery_fixture(
    session: DomSession,
    target: DomHostedTopLevel,
    element: surgeist_dom::NodeId,
    cancel: &Arc<AtomicBool>,
    source_work: u64,
) -> (DocumentScriptHost, DocumentScriptPermit) {
    let mut host = super::super::guest::tests::attach_public_nodes(
        session,
        target,
        cancel.clone(),
        5_000,
        source_work,
    );
    let permit = host.permit().unwrap();
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host,
        &permit,
        element,
        "protectedTarget",
    );
    host.in_document(&permit, |scope| {
        let script = r#"
            trustedTypes.createPolicy('default', {
                createScript(value) {
                    if (value === 'outer') {
                        protectedTarget.setAttribute('onload', 'budget');
                        return 'unreachable';
                    }
                    if (value === 'budget') return 'z'.repeat(7500);
                    if (value === 'null-later') {
                        stopAfterAttributeCallback();
                        return null;
                    }
                    return { toString() {
                        stopAfterAttributeCallback();
                        return value + '!';
                    }};
                }
            });
        "#;
        scope
            .evaluate(&script.encode_utf16().collect::<Vec<_>>())
            .map(drop)
    })
    .unwrap();
    (host, permit)
}

fn complete_guest_attribute(
    progress: Result<
        Result<crate::DomSourceDriveProgress, crate::DomSourceDriveError>,
        DocumentGuestEventError,
    >,
) -> surgeist_dom::DomSourceCompletion {
    let progress = progress.unwrap().unwrap();
    let crate::DomSourceDriveProgress::Complete(completion) = progress else {
        panic!("expected original attribute completion, got {progress:?}");
    };
    completion
}

fn pause_guest_attribute(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    cancel: &Arc<AtomicBool>,
    script: &str,
) -> Rc<Cell<usize>> {
    let calls = Rc::new(Cell::new(0));
    let result = host.in_document(permit, |scope| {
        let calls = calls.clone();
        let stop = cancel.clone();
        let function = scope.function(&Callback::new(move |_, _| {
            calls.set(calls.get() + 1);
            stop.store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(surgeist_bindings_js::Completion::Undefined)
        }))?;
        let global = scope.global()?;
        scope.set(
            &global,
            PropertyKey::String(
                &"stopAfterAttributeCallback"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            ),
            &function,
        )?;
        scope
            .evaluate(&script.encode_utf16().collect::<Vec<_>>())
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("expected terminal guest host failure, got {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::Cancelled))
        ))
    ));
    assert_eq!(calls.get(), 1);
    calls
}

pub(super) fn default_factory_retains_the_real_guest_callback() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = enable(session, target, cancel);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let mut factory = PolicyFactory::<TrustedPolicyCallback, ()>::default();
        let name = DocumentString::from("default");
        let attempt = factory
            .begin(&name, &[], &mut || Ok::<_, Infallible>(()))
            .unwrap();
        let admitted = attempt.admit(|_| Ok::<_, Infallible>(())).unwrap();
        let guest = scope.evaluate(
            &"(value, type, sink) => value + (type === 'TrustedScript' && sink === 'Element onclick' ? '!' : '?')"
                .encode_utf16()
                .collect::<Vec<_>>(),
        )?;
        let policy = Rc::new(
            scope
                .retain_trusted_policy_callback(&guest, |_| Ok::<_, Error>(()))?
                .unwrap(),
        );
        admitted.publish(&policy);
        let retained = factory.default_policy().unwrap();
        assert!(Rc::ptr_eq(retained, &policy));
        drop(policy);
        let account = ConversionAccount::standalone(
            scope,
            WorkTable::splat(1_000),
            ResourceTable::splat(1_000),
        )?;
        let result = account.run(scope, |conversion| {
            conversion.invoke_default_policy(
                retained,
                TrustedTypeKind::Script,
                &[0xd800, 0],
                &"Element onclick".encode_utf16().collect::<Vec<_>>(),
            )
        })?;
        assert_eq!(
            result.get().as_ref().unwrap().as_code_units(),
            &[0xd800, 0, b'!' as u16]
        );
        Ok(())
    })
    .unwrap();
}

pub(super) fn hosted_attribute_policy_reads_charge_the_parked_source_and_preserve_exact_input() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target, element) = hosted_element(&cancel, "div");
    let host = enable(session, target, cancel.clone());
    let registration = &host.engine.as_ref().unwrap().registration;
    let mut session = host.native.session.borrow_mut();
    let mut view = session.dom();
    let dom = view.raw_mut();
    let units = DocumentString::from_utf16([0xd800, 0, 0x61]);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_attribute(element, "onclick", units.clone()),
            SOURCE,
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::AttributeVerification(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("expected the actual registered Source sink");
    };
    let stopped = AtomicBool::new(true);
    let spent = lease.status().spent_work();
    assert!(matches!(
        AttributePolicies::read(&lease, request, registration, None, &stopped),
        Err(AttributePolicyError::Dom(DomError::Cancelled))
    ));
    assert_eq!(lease.status().spent_work(), spent);
    {
        let policies =
            AttributePolicies::read(&lease, request, registration, None, &cancel).unwrap();
        assert_eq!(policies.document(), target.document());
        assert!(!policies.requires_default_policy(&cancel).unwrap());
        let mut mismatch = policies.mismatch();
        assert!(matches!(
            mismatch.next_violation(&stopped),
            Err(DomError::Cancelled)
        ));
        assert!(mismatch.next_violation(&cancel).unwrap().is_none());
        assert_eq!(mismatch.blocked(), Some(false));
    }
    assert!(lease.status().spent_work() > spent);
    let original = lease
        .attribute_verification_input(request, registration, &cancel)
        .unwrap()
        .clone();
    assert_eq!(original, units);
    let mut reply = Some(DomSourceAttributeVerificationOutcome::Value(original));
    lease
        .accept_attribute_verification(request, registration, &mut reply, &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    assert_eq!(
        lease
            .snapshot()
            .node(element)
            .unwrap()
            .attribute(&"onclick".into()),
        Some(&units)
    );
}
