//! Shared HTMLMediaElement properties observe the admitted unloaded native family.

use super::*;
use surgeist_html::elements::media::HtmlInitialMediaState;

pub(super) fn attached_media_observes_unloaded_native_state() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const base = HTMLMediaElement;
        const prototype = base.prototype;
        const values = [['currentSrc', ''], ['networkState', 0], ['readyState', 0], ['paused', true]];
        const getters = [];
        for (const [name] of values) {
            const d = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof d?.get !== 'function') return 'missing HTMLMediaElement.prototype.' + name + ' getter';
            if (d.get.name !== 'get ' + name || d.get.length !== 0 ||
                d.set !== undefined || !d.enumerable || !d.configurable) return name + ' readonly Media descriptor';
            getters.push(d.get);
        }
        const constants = [
            ['NETWORK_EMPTY', 0], ['NETWORK_IDLE', 1], ['NETWORK_LOADING', 2], ['NETWORK_NO_SOURCE', 3],
            ['HAVE_NOTHING', 0], ['HAVE_METADATA', 1], ['HAVE_CURRENT_DATA', 2], ['HAVE_FUTURE_DATA', 3], ['HAVE_ENOUGH_DATA', 4],
        ];
        for (const [name, value] of constants) {
            for (const target of [base, prototype]) {
                const d = Object.getOwnPropertyDescriptor(target, name);
                if (d?.value !== value || d.writable !== false || !d.enumerable || d.configurable !== false) {
                    return name + ' constant descriptor';
                }
            }
        }
        const constantNames = constants.map(([name]) => name);
        const prototypeNames = [...values.map(([name]) => name), ...constantNames];
        if (Object.getOwnPropertyNames(base).filter(name => constantNames.includes(name)).join() !== constantNames.join() ||
            Object.getOwnPropertyNames(prototype).filter(name => prototypeNames.includes(name)).join() !== prototypeNames.join()) {
            return 'Web IDL Media prototype and constructor property order';
        }
        if (Object.getPrototypeOf(base) !== HTMLElement || Object.getPrototypeOf(prototype) !== HTMLElement.prototype) {
            return 'shared Media interface inheritance';
        }
        try {
            new HTMLMediaElement();
            return 'constructed the shared Media base';
        } catch (error) {
            if (!(error instanceof TypeError)) return 'Media base construction requires TypeError';
        }

        const html = 'http://www.w3.org/1999/xhtml';
        const audio = document.body.appendChild(document.createElement('audio'));
        const video = document.createElementNS(html, 'p:video');
        for (const [element, constructor] of [[audio, HTMLAudioElement], [video, HTMLVideoElement]]) {
            if (!(element instanceof base) || Object.getPrototypeOf(element) !== constructor.prototype ||
                Object.getPrototypeOf(constructor) !== base || Object.getPrototypeOf(constructor.prototype) !== prototype) {
                return constructor.name + ' native inheritance';
            }
            if (values.some(([name, value]) => element[name] !== value)) {
                return constructor.name + ' unloaded values';
            }
            for (const [name, value] of constants) {
                if (constructor[name] !== value || constructor.prototype[name] !== value || element[name] !== value ||
                    Object.hasOwn(constructor, name) || Object.hasOwn(constructor.prototype, name)) {
                    return constructor.name + ' inherited Media constant ' + name;
                }
            }
        }
        if (video.parentNode !== null) return 'Video retirement fixture must remain detached';
        const foreignAudio = document.createElementNS('urn:foreign', 'audio');
        const foreignVideo = document.createElementNS('urn:foreign', 'video');
        Object.setPrototypeOf(foreignAudio, HTMLAudioElement.prototype);
        Object.setPrototypeOf(foreignVideo, HTMLVideoElement.prototype);
        const wrong = [foreignAudio, foreignVideo, document.createElementNS(html, 'VIDEO'),
            document.createElement('source'), Object.create(prototype)];
        for (const getter of getters) {
            for (const receiver of wrong) {
                try {
                    getter.call(receiver);
                    return getter.name + ' accepted a wrong native brand';
                } catch (error) {
                    if (!(error instanceof TypeError)) return getter.name + ' wrong receiver requires TypeError';
                }
            }
        }

        globalThis.mediaStatusProbe = {audio, video};
        return 'ok';
    })()"#,
    );

    let initial = host.snapshot().unwrap();
    let html = initial
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = initial.node(html).unwrap().children()[1];
    let [audio]: [NodeId; 1] = initial.node(body).unwrap().children().try_into().unwrap();
    let video = initial
        .nodes()
        .find(|&node| media_state(&initial, node) == Some(HtmlInitialMediaState::Video))
        .unwrap();
    assert_unloaded(&initial, audio, HtmlInitialMediaState::Audio);
    assert_unloaded(&initial, video, HtmlInitialMediaState::Video);
    assert_eq!(initial.node(video).unwrap().parent(), None);

    host.with_native(|dom, _, _| {
        let mut edit = DomBatch::new();
        edit.destroy(video);
        dom.apply(edit, &cancel()).unwrap();
    })
    .unwrap();
    let current = host.snapshot().unwrap();
    assert!(matches!(current.node(video), Err(DomError::StaleNode)));
    assert_unloaded(&initial, video, HtmlInitialMediaState::Video);
    assert_unloaded(&current, audio, HtmlInitialMediaState::Audio);
    assert_eq!(current.incomplete_source_operation(), None);

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {audio} = mediaStatusProbe;
        return audio.parentNode === document.body && audio.currentSrc === '' &&
            audio.networkState === HTMLMediaElement.NETWORK_EMPTY &&
            audio.readyState === HTMLMediaElement.HAVE_NOTHING && audio.paused === true
            ? 'ok' : 'current Audio remains readable after independent Video retirement';
    })()"#,
    );

    let failure = native_failure(host.in_document(&permit, |scope| {
        let code = "mediaStatusProbe.video.currentSrc"
            .encode_utf16()
            .collect::<Vec<_>>();
        scope.evaluate(&code).map(drop)
    }));
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
    host.stop().into_session().unwrap();
}

fn media_state(snapshot: &DomSnapshot, id: NodeId) -> Option<HtmlInitialMediaState> {
    snapshot
        .node(id)
        .unwrap()
        .as_element()
        .and_then(|element| element.initial_media_state())
}

fn assert_unloaded(snapshot: &DomSnapshot, id: NodeId, expected: HtmlInitialMediaState) {
    let state = media_state(snapshot, id).unwrap();
    assert_eq!(state, expected);
    assert!(state.current_src().is_empty());
    assert!(state.network_is_empty());
    assert!(!state.has_media_data());
    assert!(state.paused());
}

fn observe(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit, script: &str) {
    let result = host
        .in_document(permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
}
