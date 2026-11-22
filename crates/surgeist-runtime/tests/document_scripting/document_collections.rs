use super::{window_length::observe, *};

pub(super) fn document_collections_share_live_native_identity() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 180_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let form = host
        .with_native(|dom, _, _| {
            let document = top.document();
            let snapshot = dom.snapshot();
            let html = snapshot.node(document.node()).unwrap().children()[0];
            let body = snapshot.node(html).unwrap().children()[1];
            let mut edit = DomBatch::new();
            let form = edit.create_element_ns(document, Namespace::html(), "form");
            let script = edit.create_element_ns(document, Namespace::html(), "script");
            edit.set_attribute(&form, "id", "form");
            edit.set_attribute(&script, "id", "script");
            edit.insert_before(&form, body, None);
            edit.insert_before(&script, body, None);
            let receipt = dom.apply(edit, &cancel()).unwrap();
            receipt.created(&form).unwrap().unwrap()
        })
        .unwrap();

    let initial = observe(
        &mut host,
        &permit,
        r#"(() => {
            const names = ['images', 'embeds', 'plugins', 'links', 'forms', 'scripts', 'anchors', 'applets'];
            const want = [null, null, null, null, 'form', 'script', null, null];
            const collections = names.map(name => document[name]);
            if (collections.some(collection => !(collection instanceof HTMLCollection)))
                return 'the Document collection getters must expose native HTMLCollections';
            const empty = new Document();
            for (let n = 0; n < names.length; ++n) {
                const name = names[n], collection = collections[n], id = want[n];
                const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, name);
                const getter = descriptor?.get;
                if (typeof getter !== 'function' || getter.name !== 'get ' + name || getter.length !== 0
                    || !descriptor.configurable || !descriptor.enumerable || descriptor.set !== undefined)
                    return 'getters must preserve their interface descriptors';
                if (collection !== document[name] || collection !== getter.call(document)
                    || empty[name] !== empty[name] || empty[name] === collection || empty[name].length !== 0)
                    return 'collection identity belongs to its exact native Document';
                if (collection.length !== (id === null ? 0 : 1))
                    return 'the collection must select only its qualifying members';
                if (id !== null && (collection[0] !== document.getElementById(id)
                    || collection.namedItem(id) !== collection[0] || Reflect.ownKeys(collection).join() !== '0,' + id))
                    return 'the collection must return canonical members and indexed and named keys';
                try { getter.call(Object.create(document)); return 'accepted a forged Document receiver'; }
                catch (error) { if (!(error instanceof TypeError)) throw error; }
            }
            if (document.plugins !== document.embeds || empty.plugins !== empty.embeds
                || document.applets === document.images || empty.applets === empty.images)
                return 'identity must be shared only by the specified alias';

            const detached = document.implementation.createHTMLDocument();
            const {images, links, anchors, applets, embeds} = detached;
            const image = detached.createElement('img');
            const a = detached.createElement('a');
            const area = detached.createElement('area');
            const applet = detached.createElement('applet');
            const svgAnchor = detached.createElementNS('http://www.w3.org/2000/svg', 'a');
            const svgEmbed = detached.createElementNS('http://www.w3.org/2000/svg', 'embed');
            a.setAttribute('name', '');
            area.setAttribute('href', '');
            svgAnchor.setAttribute('href', '');
            for (const node of [image, a, area, applet, svgAnchor, svgEmbed])
                detached.body.appendChild(node);
            if (images !== detached.images || images.length !== 1 || images[0] !== image
                || links.length !== 1 || links[0] !== area || anchors.length !== 1 || anchors[0] !== a
                || applets.length !== 0 || embeds.length !== 0 || detached.plugins !== embeds)
                return 'filters must use ordinary HTML elements and present attributes';
            a.setAttribute('href', '');
            area.remove();
            image.remove();
            if (images.length !== 0 || links !== detached.links || links.length !== 1 || links[0] !== a)
                return 'the same collections must observe ordinary tree and attribute changes';
            globalThis.savedDocumentCollections = collections;
            return 'ok';
        })()"#,
    );
    assert_eq!(initial, "ok");

    host.with_native(|dom, _, _| {
        let mut edit = DomBatch::new();
        edit.detach(form);
        dom.apply(edit, &cancel()).unwrap();
    })
    .unwrap();
    let changed = observe(
        &mut host,
        &permit,
        r#"(() => {
            const [, embeds, plugins, , forms, scripts] = savedDocumentCollections;
            return forms === document.forms && forms.length === 0 && forms.form === undefined &&
                scripts === document.scripts && scripts.length === 1 && embeds === plugins
                ? 'ok' : 'retained native collections must reflect later native edits';
        })()"#,
    );
    assert_eq!(changed, "ok");
}
