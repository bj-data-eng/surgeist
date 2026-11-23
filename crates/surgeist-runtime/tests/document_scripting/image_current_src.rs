//! Read the original unavailable request on a native inactive Image.

use super::{window_length::observe, *};
use surgeist_html::elements::image::HtmlImgFreshRequestState;

pub(super) fn inactive_image_current_src_reads_its_native_request() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 200_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    // The attached Document is active: create the Img in a never-hosted
    // factory Document, projected by the attached guest's ordinary public API.
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const owner = document.implementation.createHTMLDocument();
                const image = owner.createElement('img');
                image.setAttribute('src', 'authored-image.png');
                image.setAttribute('width', '640');
                image.setAttribute('height', '480');
                owner.body.appendChild(image);
                globalThis.imageCurrentSrcProbe = {owner, image};
                return image.ownerDocument === owner && owner.defaultView === null &&
                    image.getAttribute('src') === 'authored-image.png' ? 'ok' : 'inactive Img setup';
            })()"#,
        ),
        "ok",
    );

    // Authenticate the real native precondition before the missing-member RED.
    let snapshot = host.snapshot().unwrap();
    let images = snapshot
        .nodes()
        .filter(|&id| snapshot.node(id).unwrap().as_img().is_some())
        .collect::<Vec<_>>();
    let [id]: [NodeId; 1] = images.try_into().unwrap();
    let node = snapshot.node(id).unwrap();
    let owner = node.owner_document().unwrap();
    assert_ne!(owner, top.document());
    assert!(
        !snapshot
            .document(owner)
            .unwrap()
            .metadata()
            .document_environment()
            .has_ever_had_navigable()
    );
    assert_eq!(
        node.attribute(&"src".into()).unwrap().code_units(),
        "authored-image.png".encode_utf16().collect::<Vec<_>>()
    );
    let image = node.as_img().unwrap();
    assert_eq!(image.suspended_update().unwrap().unwrap().document(), owner);
    assert_eq!(
        image.state().current_request().state(),
        HtmlImgFreshRequestState::Unavailable
    );
    assert!(image.state().current_request().current_url().is_empty());
    let original_request = image.state().clone();

    // HTML877d embedded:647,1263: readonly USVString currentSrc returns the
    // current request's current URL, independently of the nonempty src attribute.
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const {owner, image} = imageCurrentSrcProbe;
                const prototype = Object.getPrototypeOf(image);
                const d = Object.getOwnPropertyDescriptor(prototype, 'currentSrc');
                if (typeof d?.get !== 'function')
                    return 'missing HTMLImageElement.prototype.currentSrc getter';
                if (prototype !== HTMLImageElement.prototype ||
                    !d.enumerable || !d.configurable || d.set !== undefined ||
                    'value' in d || 'writable' in d || d.get.name !== 'get currentSrc' ||
                    d.get.length !== 0 || Object.hasOwn(d.get, 'prototype') ||
                    Object.hasOwn(image, 'currentSrc'))
                    return 'currentSrc must be the own readonly Image prototype accessor';
                if (image.currentSrc !== '' || d.get.call(image) !== '' ||
                    image.getAttribute('src') !== 'authored-image.png')
                    return 'currentSrc must read the native current request, not authored src';

                // Keep the currentSrc missing-descriptor RED above first.
                const getters = [['currentSrc', d.get]];
                for (const [name, expected] of [['naturalWidth', 0], ['naturalHeight', 0], ['complete', false]]) {
                    const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
                    if (typeof descriptor?.get !== 'function')
                        return 'missing HTMLImageElement.prototype.' + name + ' getter';
                    if (!descriptor.enumerable || !descriptor.configurable || descriptor.set !== undefined ||
                        'value' in descriptor || 'writable' in descriptor ||
                        descriptor.get.name !== 'get ' + name || descriptor.get.length !== 0 ||
                        Object.hasOwn(descriptor.get, 'prototype') || Object.hasOwn(image, name))
                        return name + ' must be an own readonly Image prototype accessor';
                    if (image[name] !== expected || descriptor.get.call(image) !== expected)
                        return name + ' must read the original unavailable request';
                    getters.push([name, descriptor.get]);
                }

                const foreign = owner.createElementNS('urn:foreign', 'img');
                Object.setPrototypeOf(foreign, prototype);
                const upper = owner.createElementNS('http://www.w3.org/1999/xhtml', 'IMG');
                for (const [name, getter] of getters) {
                    for (const receiver of [null, {}, document.body, Object.create(prototype),
                        Object.create(image), new Proxy(image, {}), foreign, upper]) {
                        try { getter.call(receiver); return name + ' accepted a wrong native Img brand'; }
                        catch (error) {
                            if (Object.getPrototypeOf(error) !== TypeError.prototype) throw error;
                        }
                    }
                }
                return 'ok';
            })()"#,
        ),
        "ok",
    );

    // Independent HTML877d complete cases: attribute presence is meaningful,
    // including an explicitly empty srcset. No selected or loaded image exists.
    for (src, srcset, complete) in [
        (None, None, true),
        (Some(""), None, true),
        (None, Some(""), false),
        (Some(""), Some(""), false),
        (Some("held-authored.png"), None, false),
        (Some("held-authored.png"), Some(""), false),
        (Some("held-authored.png"), Some("candidate.png 1x"), false),
        (None, None, true),
    ] {
        host.with_native(|dom, _, _| {
            let mut edit = DomBatch::new();
            for (name, value) in [("src", src), ("srcset", srcset)] {
                match value {
                    Some(value) => edit.set_attribute(id, name, value),
                    None => edit.remove_attribute(id, name),
                }
            }
            dom.apply(edit, &cancel()).unwrap();
        })
        .unwrap();
        let current = host.snapshot().unwrap();
        let image = current.node(id).unwrap().as_img().unwrap();
        assert_eq!(image.state(), &original_request);
        assert_eq!(
            image.state().current_request().state(),
            HtmlImgFreshRequestState::Unavailable
        );
        assert!(image.state().current_request().current_url().is_empty());
        assert!(image.state().pending_request_is_null());
        assert_eq!(image.suspended_update().unwrap().unwrap().document(), owner);
        for (name, expected) in [("src", src), ("srcset", srcset)] {
            assert_eq!(
                current
                    .node(id)
                    .unwrap()
                    .attribute(&name.into())
                    .map(|raw| raw.code_units()),
                expected
                    .map(|value| value.encode_utf16().collect::<Vec<_>>())
                    .as_deref(),
            );
        }
        assert_eq!(
            observe(
                &mut host,
                &permit,
                &format!(
                    r"(() => {{
            const image = imageCurrentSrcProbe.image;
            return image.currentSrc === '' && image.naturalWidth === 0 && image.naturalHeight === 0 &&
                image.complete === {complete} && image.getAttribute('width') === '640' &&
                image.getAttribute('height') === '480' ? 'ok' : 'incorrect live request readbacks';
        }})()"
                )
            ),
            "ok",
            "src={src:?}, srcset={srcset:?}"
        );
    }
    host.stop().into_session().unwrap();
}
