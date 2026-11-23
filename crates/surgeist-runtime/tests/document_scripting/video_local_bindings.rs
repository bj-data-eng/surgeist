//! HTMLVideoElement own reflections and unloaded native dimensions.

use super::{
    window_length::{edit, observe},
    *,
};
use surgeist_html::elements::media::HtmlInitialMediaState;

fn attached_video(source_work: u64) -> (DocumentScriptHost, NodeId, NodeId, NodeId) {
    let (mut session, top) = setup();
    let document = top.document();
    let snapshot = session.snapshot();
    let html = snapshot.node(document.node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let video = batch.create_element_ns(document, Namespace::html(), "video");
    batch.set_attribute(&video, "id", "nativeVideo");
    batch.set_attribute(&video, "width", "017");
    batch.insert_before(&video, body, None);
    let audio = batch.create_element_ns(document, Namespace::html(), "audio");
    batch.set_attribute(&audio, "id", "nativeAudio");
    batch.insert_before(&audio, body, None);
    let detached = batch.create_html_document(document, None);
    let frame = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    let receipt = session.dom().apply(batch, &cancel()).unwrap();
    let [video, frame] = [video, frame].map(|node| receipt.created(&node).unwrap().unwrap());
    let (host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 400_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 1_000_000, source_work, 128, 4096, 4096, 4),
        |_, _, _| {},
    );
    (host, video, frame, body)
}

pub(super) fn video_reflects_canonical_attributes_and_its_unloaded_dimensions() {
    let (mut host, video, frame, body) = attached_video(1_000_000);
    let permit = host.permit().unwrap();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const proto = HTMLVideoElement.prototype;
                if (typeof Object.getOwnPropertyDescriptor(proto, 'width')?.get !== 'function')
                    return 'missing HTMLVideoElement.prototype.width getter';
                const video = document.getElementById('nativeVideo');
                const audio = document.getElementById('nativeAudio');
                const html = 'http://www.w3.org/1999/xhtml';
                const prefixed = document.createElementNS(html, 'p:video');
                const foreign = document.createElementNS('urn:video-reflection', 'video');
                const wrongCase = document.createElementNS(html, 'VIDEO');
                Object.setPrototypeOf(foreign, proto);
                const order = ['width', 'height', 'videoWidth', 'videoHeight', 'poster', 'playsInline', 'constructor'];
                const installed = order.filter(name => name !== 'poster' || Object.hasOwn(proto, name));
                if (Object.getOwnPropertyNames(proto).filter(name => installed.includes(name)).join() !== installed.join() ||
                    Object.getPrototypeOf(proto) !== HTMLMediaElement.prototype ||
                    Object.getPrototypeOf(HTMLVideoElement) !== HTMLMediaElement ||
                    Object.getPrototypeOf(video) !== proto || Object.getPrototypeOf(prefixed) !== proto ||
                    prefixed.prefix !== 'p' || Object.getPrototypeOf(audio) !== HTMLAudioElement.prototype)
                    return 'authored order and native Video inheritance';
                const typeError = action => {
                    try { action(); } catch (error) {
                        return Object.getPrototypeOf(error) === TypeError.prototype;
                    }
                    return false;
                };
                for (const name of ['width', 'height', 'videoWidth', 'videoHeight', 'playsInline']) {
                    const d = Object.getOwnPropertyDescriptor(proto, name);
                    const readonly = name.startsWith('video');
                    if (typeof d?.get !== 'function' || d.get.name !== 'get ' + name ||
                        d.get.length !== 0 || !d.configurable || !d.enumerable ||
                        'value' in d || 'writable' in d ||
                        (readonly ? d.set !== undefined :
                            typeof d.set !== 'function' || d.set.length !== 1 || d.set.name !== 'set ' + name) ||
                        name in audio || Object.hasOwn(HTMLMediaElement.prototype, name))
                        return 'exact own Video descriptor ' + name;
                    for (const accessor of readonly ? [d.get] : [d.get, d.set]) {
                        if (Object.hasOwn(accessor, 'prototype') ||
                            !typeError(() => Reflect.construct(function () {}, [], accessor)))
                            return 'nonconstructible Video accessor ' + name;
                        for (const bad of [audio, foreign, wrongCase, Object.create(proto), new Proxy(video, {})]) {
                            let coerced = false;
                            const input = {[Symbol.toPrimitive]() { coerced = true; return 3; }};
                            if (!typeError(() => Reflect.apply(accessor, bad, [input])) || coerced)
                                return 'native Video brand before coercion ' + name;
                        }
                    }
                }

                if (video.width !== 17 || video.height !== 0 || prefixed.width !== 0 ||
                    video.readyState !== HTMLMediaElement.HAVE_NOTHING ||
                    video.videoWidth !== 0 || video.videoHeight !== 0)
                    return 'content dimensions are separate from native HAVE_NOTHING dimensions';
                for (const [name, input] of [['width', '-2'], ['height', '2147483648']]) {
                    video.setAttribute(name, input);
                    if (video[name] !== 0 || video.getAttribute(name) !== input)
                        return 'out-of-range native attribute readback ' + name;
                }
                for (const [input, expected] of [[17.75, 17], [4294967303, 7], [-1, 0], [2147483647, 2147483647]]) {
                    video.width = input;
                    if (video.width !== expected || video.getAttribute('width') !== String(expected))
                        return 'unsigned-long conversion and reflection ' + input;
                }
                video.setAttributeNS('urn:video-reflection', 'p:playsinline', 'untouched');
                if (video.playsInline) return 'foreign namespaced playsinline must be ignored';
                video.playsInline = {valueOf() { throw Error('Boolean coercion ran valueOf'); }};
                if (!video.playsInline || video.getAttributeNS(null, 'playsinline') !== '')
                    return 'Boolean presence';
                video.playsInline = false;
                if (video.playsInline || video.hasAttributeNS(null, 'playsinline') ||
                    video.getAttributeNS('urn:video-reflection', 'playsinline') !== 'untouched')
                    return 'Boolean reversal preserves namespaced attribute';

                const width = video.getAttributeNode('width');
                const hints = [];
                video.width = {[Symbol.toPrimitive](hint) {
                    hints.push(hint);
                    if (video.width !== 2147483647) throw Error('write before conversion');
                    video.height = 23;
                    video.parentNode.removeChild(video);
                    return 41;
                }};
                const sentinel = {};
                try { video.width = {valueOf() { throw sentinel; }}; return 'swallowed conversion'; }
                catch (error) { if (error !== sentinel) return 'changed thrown value'; }
                if (hints.join() !== 'number' || video.parentNode !== null ||
                    video.width !== 41 || width !== video.getAttributeNode('width') ||
                    width.value !== '41' || video.height !== 23 || video.videoWidth !== 0 || video.videoHeight !== 0)
                    return 'reentry must keep the original native Video and Attr';
                globalThis.savedVideo = video;
                return 'ok';
            })()"#,
        ),
        "ok"
    );
    let snapshot = host.snapshot().unwrap();
    let native = snapshot.node(video).unwrap();
    let state = native.as_element().unwrap().initial_media_state().unwrap();
    assert_eq!(state, HtmlInitialMediaState::Video);
    assert!(!state.has_media_data());
    assert_eq!(native.attribute(&"width".into()), Some(&"41".into()));
    assert_eq!(native.attribute(&"height".into()), Some(&"23".into()));
    edit(
        &mut host,
        DomSourceEdit::set_attribute_ns(video, Namespace::none(), "height", "00032"),
    );
    edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                if (savedVideo.height !== 32 || savedVideo.videoHeight !== 0)
                    return 'native content readback must leave natural dimensions alone';
                const child = window[0];
                const local = Object.getOwnPropertyDescriptor(HTMLVideoElement.prototype, 'width');
                const childProto = child.HTMLVideoElement.prototype;
                const other = Object.getOwnPropertyDescriptor(childProto, 'width');
                const video = child.document.createElement('video');
                if (other.get === local.get || Object.getPrototypeOf(other.set) !== child.Function.prototype ||
                    Object.getPrototypeOf(video) !== childProto)
                    return 'distinct original Video creation realm';
                local.set.call(video, 91);
                if (other.get.call(video) !== 91 || document.adoptNode(video) !== video ||
                    video.ownerDocument !== document || Object.getPrototypeOf(video) !== childProto)
                    return 'borrowed setter or adoption lost original Video';
                other.set.call(video, 62);
                return local.get.call(video) === 62 && video.videoWidth === 0
                    ? 'ok' : 'adopted Video did not keep its native reflection and media state';
            })()"#,
        ),
        "ok"
    );

    host.checkpoint().unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(video);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let refusal = native_failure(host.in_document(&permit, |scope| {
        let source = "savedVideo.videoWidth".encode_utf16().collect::<Vec<_>>();
        scope.evaluate(&source).map(drop)
    }));
    assert!(matches!(
        refusal.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}

pub(super) fn video_width_retains_the_original_source_work_refusal() {
    let (mut host, video, _, _) = attached_video(0);
    let permit = host.permit().unwrap();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            "typeof Object.getOwnPropertyDescriptor(HTMLVideoElement.prototype, 'width')?.set === 'function' ? 'ok' : 'missing HTMLVideoElement.prototype.width setter'"
        ),
        "ok"
    );
    let before = host.snapshot().unwrap();
    let refusal = native_failure(host.in_document(&permit, |scope| {
        let source = "Object.getOwnPropertyDescriptor(HTMLVideoElement.prototype, 'width').set.call(document.getElementById('nativeVideo'), 320)";
        scope.evaluate(&source.encode_utf16().collect::<Vec<_>>()).map(drop)
    }));
    assert!(matches!(
        refusal.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::SourceWorkLimit { limit: 0 }
        )))
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(
        after.node(video).unwrap().attribute(&"width".into()),
        Some(&"017".into())
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}
pub(super) fn borrowed_video_accessors_keep_original_realms_and_have_nothing() {
    let (session, top) = setup();
    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 300_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 1_000_000, 1_000_000, 128, 4096, 4096, 4),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r"(() => {
        if (typeof Object.getOwnPropertyDescriptor(HTMLVideoElement.prototype, 'width')?.get !== 'function')
            return 'missing HTMLVideoElement.prototype.width getter';
        globalThis.realmVideo = document.body.appendChild(document.createElement('video'));
        realmVideo.id = 'borrowed-video';
        globalThis.realmAudio = document.body.appendChild(document.createElement('audio'));
        return 'ok';
    })()"
        ),
        "ok"
    );
    let (video, frame, body) = host
        .with_native(|dom, _, _| {
            let snapshot = dom.snapshot();
            let video = snapshot
                .nodes()
                .find(|&id| {
                    snapshot
                        .node(id)
                        .unwrap()
                        .attribute(&"id".into())
                        .is_some_and(|value| value == &DocumentString::from("borrowed-video"))
                })
                .unwrap();
            let html = snapshot
                .document(permit.document())
                .unwrap()
                .document_element()
                .unwrap();
            let body = snapshot.node(html).unwrap().children()[1];
            assert_eq!(
                snapshot
                    .node(video)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .initial_media_state(),
                Some(HtmlInitialMediaState::Video)
            );
            let mut batch = DomBatch::new();
            batch.set_attribute(video, "width", "0320");
            batch.set_attribute(video, "height", "0240");
            let owner = batch.create_html_document(permit.document(), None);
            let frame = batch.create_element_ns(&owner, Namespace::html(), "iframe");
            let receipt = dom.apply(batch, &cancel()).unwrap();
            (video, receipt.created(&frame).unwrap().unwrap(), body)
        })
        .unwrap();
    edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r"(() => {
        const child = window[0], prototype = child.HTMLVideoElement.prototype;
        const width = Object.getOwnPropertyDescriptor(prototype, 'width');
        const localWidth = Object.getOwnPropertyDescriptor(HTMLVideoElement.prototype, 'width');
        const childVideo = child.document.createElement('video');
        const childAudio = child.document.createElement('audio');
        if (width.set === localWidth.set || Object.getPrototypeOf(width.set) !== child.Function.prototype ||
            Object.getPrototypeOf(childVideo) !== prototype || child.TypeError.prototype === TypeError.prototype)
            return 'distinct original Video setter realm';
        if (width.get.call(realmVideo) !== 320 || realmVideo.height !== 240)
            return 'borrowed child getter missed native top Video attributes';
        width.set.call(realmVideo, 73);
        let rejected = false;
        try { width.set.call(realmVideo, 1n); }
        catch (error) { rejected = Object.getPrototypeOf(error) === child.TypeError.prototype; }
        if (!rejected || realmVideo.width !== 73) return 'borrowed Video setter conversion error realm';
        for (const name of ['videoWidth', 'videoHeight']) {
            const get = Object.getOwnPropertyDescriptor(prototype, name).get;
            const topGet = Object.getOwnPropertyDescriptor(HTMLVideoElement.prototype, name).get;
            if (get === topGet || Object.getPrototypeOf(get) !== child.Function.prototype ||
                get.call(realmVideo) !== 0 || topGet.call(childVideo) !== 0 ||
                realmVideo.readyState !== HTMLMediaElement.HAVE_NOTHING ||
                childVideo.readyState !== child.HTMLMediaElement.HAVE_NOTHING)
                return 'borrowed Video dimensions require actual native HAVE_NOTHING';
            let wrong = false;
            try { get.call(realmAudio); }
            catch (error) { wrong = Object.getPrototypeOf(error) === child.TypeError.prototype; }
            if (!wrong) return 'borrowed child Video getter must reject native Audio in its own realm';
            wrong = false;
            try { topGet.call(childAudio); }
            catch (error) { wrong = Object.getPrototypeOf(error) === TypeError.prototype; }
            if (!wrong) return 'borrowed top Video getter must preserve the top TypeError realm';
        }
        if (document.adoptNode(childVideo) !== childVideo || Object.getPrototypeOf(childVideo) !== prototype)
            return 'adoption lost the original child Video projection';
        localWidth.set.call(childVideo, 25);
        return childVideo.width === 25 && childVideo.videoWidth === 0 && childVideo.videoHeight === 0
            ? 'ok' : 'adopted Video retains actual unloaded dimensions';
    })()"
        ),
        "ok"
    );
    let snapshot = host.snapshot().unwrap();
    assert_eq!(
        snapshot.node(video).unwrap().attribute(&"width".into()),
        Some(&"73".into())
    );
    assert_eq!(
        snapshot.node(video).unwrap().attribute(&"height".into()),
        Some(&"0240".into())
    );
    for id in snapshot.nodes() {
        if let Some(state) = snapshot
            .node(id)
            .unwrap()
            .as_element()
            .and_then(|e| e.initial_media_state())
        {
            assert!(!state.has_media_data());
            assert!(state.network_is_empty());
            assert!(state.current_src().is_empty());
        }
    }
    host.stop().into_session().unwrap();
}
