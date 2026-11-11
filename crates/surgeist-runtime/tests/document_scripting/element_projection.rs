//! Element interfaces over a live canonical document root.

use super::*;
use surgeist_html::values::Namespace;

pub(super) fn document_projects_its_current_native_element() {
    let (mut host, top) = prepare_event_window();
    let permit = host.permit().unwrap();
    let empty = host
        .in_document(&permit, |scope| {
            let script = r#"(() => {
                globalThis.projectedDocument = new Document();
                const root = projectedDocument.documentElement;
                return root === null ? "null" : typeof root;
            })()"#;
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&empty).unwrap(), "null");

    let snapshot = host.snapshot().unwrap();
    let document = snapshot
        .nodes()
        .filter_map(|id| snapshot.node(id).unwrap().as_document())
        .map(|view| view.id())
        .find(|document| *document != top.document())
        .expect("canonical document created by the constructor");
    let native_root = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let preceding = batch.create_comment(document, "before the root");
            let root = batch.create_element_ns(document, Namespace::html(), "html");
            batch.insert_before(&preceding, document, None);
            batch.insert_before(&root, document, None);
            dom.apply(batch, &cancel())
                .unwrap()
                .created(&root)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    assert_eq!(
        host.snapshot()
            .unwrap()
            .document(document)
            .unwrap()
            .document_element(),
        Some(native_root)
    );
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const getter = Object.getOwnPropertyDescriptor(Document.prototype, "documentElement").get;
            let branded = false;
            try { Reflect.apply(getter, Object.create(Document.prototype), []); }
            catch (error) { branded = error instanceof TypeError; }
            const root = projectedDocument.documentElement;
            return branded && root === projectedDocument.documentElement
                && Object.getPrototypeOf(root) === HTMLHtmlElement.prototype
                && root instanceof HTMLElement && root instanceof Element
                && root instanceof Node && root instanceof EventTarget
                && root.ownerDocument === projectedDocument && root.parentNode === projectedDocument
                && root.nodeName === "html" && root.localName === "html" && root.tagName === "html"
                && root.namespaceURI === "http://www.w3.org/1999/xhtml" && root.prefix === null;
        })())"#,
        1.0,
    );
}

pub(super) fn html_roots_select_their_exact_interfaces() {
    let (mut host, permit, document) = interface_document();
    let mut previous = None;
    for (name, interface, parent) in [
        ("h2", "HTMLHeadingElement", "HTMLElement"),
        ("article", "HTMLElement", "Element"),
        ("widget-test", "HTMLElement", "Element"),
        ("undefinedtag", "HTMLUnknownElement", "HTMLElement"),
        ("applet", "HTMLUnknownElement", "HTMLElement"),
        ("listing", "HTMLPreElement", "HTMLElement"),
        ("font", "HTMLFontElement", "HTMLElement"),
        ("audio", "HTMLAudioElement", "HTMLMediaElement"),
        ("video", "HTMLVideoElement", "HTMLMediaElement"),
    ] {
        let native_root = host
            .with_native(|dom, _, _| {
                let mut batch = DomBatch::new();
                if let Some(previous) = previous {
                    batch.detach(previous);
                    batch.destroy(previous);
                }
                let root = batch.create_element_ns(document, Namespace::html(), name);
                batch.insert_before(&root, document, None);
                dom.apply(batch, &cancel())
                    .unwrap()
                    .created(&root)
                    .unwrap()
                    .unwrap()
            })
            .unwrap();
        previous = Some(native_root);
        let source = format!(
            r#"(() => {{
                const type = globalThis["{interface}"];
                const parent = globalThis["{parent}"];
                if (typeof type !== "function") return "missing {interface}";
                if (typeof parent !== "function") return "missing {parent}";
                const root = interfaceDocument.documentElement;
                return Object.getPrototypeOf(root) === type.prototype
                    && Object.getPrototypeOf(type) === parent
                    && Object.getPrototypeOf(type.prototype) === parent.prototype
                    && root.localName === "{name}" && root.ownerDocument === interfaceDocument
                    ? "ok" : "wrong {interface} for {name}";
            }})()"#,
        );
        let result = host
            .in_document(&permit, |scope| {
                let value = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
                scope.to_utf16(&value)
            })
            .unwrap();
        assert_eq!(String::from_utf16(&result).unwrap(), "ok", "{name}");
    }
    eval(
        &mut host,
        &permit,
        r#"Number(Object.getPrototypeOf(HTMLMediaElement) === HTMLElement &&
            [HTMLAudioElement, HTMLVideoElement].every(type =>
                Object.getPrototypeOf(type) === HTMLMediaElement &&
                Object.getPrototypeOf(type.prototype) === HTMLMediaElement.prototype))"#,
        1.0,
    );
}

pub(super) fn generic_and_mathml_roots_keep_exact_namespaces() {
    let (mut host, permit, document) = interface_document();
    let mut previous = None;
    for (namespace, name, expression) in [
        (
            Namespace::none(),
            "Root",
            r#"root.namespaceURI === null && root.prefix === null && root.localName === "Root"
                && root.tagName === "Root" && Object.getPrototypeOf(root) === Element.prototype"#,
        ),
        (
            Namespace::from_uri("urn:surgeist:example"),
            "p:Ω",
            r#"root.namespaceURI === "urn:surgeist:example" && root.prefix === "p"
                && root.localName === "Ω" && root.tagName === "p:Ω"
                && Object.getPrototypeOf(root) === Element.prototype"#,
        ),
        (
            Namespace::mathml(),
            "math",
            r#"root.namespaceURI === "http://www.w3.org/1998/Math/MathML"
                && root.localName === "math" && Object.getPrototypeOf(root) === MathMLElement.prototype
                && Object.getPrototypeOf(MathMLElement.prototype) === Element.prototype"#,
        ),
    ] {
        previous = Some(
            host.with_native(|dom, _, _| {
                let mut batch = DomBatch::new();
                if let Some(previous) = previous {
                    batch.detach(previous);
                    batch.destroy(previous);
                }
                let root = batch.create_element_ns(document, namespace, name);
                batch.insert_before(&root, document, None);
                dom.apply(batch, &cancel())
                    .unwrap()
                    .created(&root)
                    .unwrap()
                    .unwrap()
            })
            .unwrap(),
        );
        let script = format!(
            "Number((() => {{ const root = interfaceDocument.documentElement; return {expression}; }})())"
        );
        host.in_document(&permit, |scope| assert_script_number(scope, &script, 1.0))
            .unwrap_or_else(|error| panic!("qualified foreign root {name} refused: {error:?}"));
    }
}

pub(super) fn svg_roots_select_exact_interfaces_and_case() {
    let (mut host, permit, document) = interface_document();
    let mut previous = None;
    for (name, interface, parent) in [
        ("svg", "SVGSVGElement", "SVGGraphicsElement"),
        ("rect", "SVGRectElement", "SVGGeometryElement"),
        (
            "linearGradient",
            "SVGLinearGradientElement",
            "SVGGradientElement",
        ),
        (
            "feFuncR",
            "SVGFEFuncRElement",
            "SVGComponentTransferFunctionElement",
        ),
        ("textPath", "SVGTextPathElement", "SVGTextContentElement"),
        (
            "animateMotion",
            "SVGAnimateMotionElement",
            "SVGAnimationElement",
        ),
        ("lineargradient", "SVGElement", "Element"),
        ("unknown-test", "SVGElement", "Element"),
    ] {
        previous = Some(
            host.with_native(|dom, _, _| {
                let mut batch = DomBatch::new();
                if let Some(previous) = previous {
                    batch.detach(previous);
                    batch.destroy(previous);
                }
                let root = batch.create_element_ns(document, Namespace::svg(), name);
                batch.insert_before(&root, document, None);
                dom.apply(batch, &cancel())
                    .unwrap()
                    .created(&root)
                    .unwrap()
                    .unwrap()
            })
            .unwrap(),
        );
        let script = format!(
            r#"Number((() => {{
                const root = interfaceDocument.documentElement;
                const type = globalThis["{interface}"];
                const parent = globalThis["{parent}"];
                return Object.getPrototypeOf(root) === type.prototype
                    && Object.getPrototypeOf(type) === parent
                    && Object.getPrototypeOf(type.prototype) === parent.prototype
                    && root.localName === "{name}"
                    && root.namespaceURI === "http://www.w3.org/2000/svg";
            }})())"#,
        );
        host.in_document(&permit, |scope| assert_script_number(scope, &script, 1.0))
            .unwrap_or_else(|error| panic!("qualified SVG root {name} refused: {error:?}"));
    }
}

fn interface_document() -> (DocumentScriptHost, DocumentScriptPermit, DocumentId) {
    let (mut host, top) = prepare_event_window();
    let permit = host.permit().unwrap();
    eval(
        &mut host,
        &permit,
        "Number((globalThis.interfaceDocument = new Document()) instanceof Document)",
        1.0,
    );
    let snapshot = host.snapshot().unwrap();
    let document = snapshot
        .nodes()
        .filter_map(|id| snapshot.node(id).unwrap().as_document())
        .map(|view| view.id())
        .find(|document| *document != top.document())
        .expect("canonical document created by the constructor");
    (host, permit, document)
}

pub(super) fn root_identity_survives_adoption_and_exact_retirement() {
    let (mut host, top) = prepare_event_window();
    let permit = host.permit().unwrap();
    eval(
        &mut host,
        &permit,
        "Number((globalThis.originDocument = new Document()) instanceof Document)",
        1.0,
    );
    let snapshot = host.snapshot().unwrap();
    let origin = snapshot
        .nodes()
        .filter_map(|id| snapshot.node(id).unwrap().as_document())
        .map(|view| view.id())
        .find(|document| *document != top.document())
        .unwrap();
    eval(
        &mut host,
        &permit,
        "Number((globalThis.destinationDocument = new Document()) instanceof Document)",
        1.0,
    );
    let snapshot = host.snapshot().unwrap();
    let destination = snapshot
        .nodes()
        .filter_map(|id| snapshot.node(id).unwrap().as_document())
        .map(|view| view.id())
        .find(|document| *document != top.document() && *document != origin)
        .unwrap();
    let root = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let root = batch.create_element_ns(origin, Namespace::html(), "div");
            batch.insert_before(&root, origin, None);
            dom.apply(batch, &cancel())
                .unwrap()
                .created(&root)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    let connected = host.in_document(&permit, |scope| {
        assert_script_number(
            scope,
            r#"Number((() => {
                globalThis.originalElement = originDocument.documentElement;
                originalElement.expando = 42;
                return originalElement instanceof HTMLDivElement
                    && originalElement.getRootNode() === originDocument;
            })())"#,
            1.0,
        )
    });
    if connected.is_err() {
        panic!(
            "connected Element root lookup refused: {:?}",
            native_failure(connected).cause::<DocumentGuestEventError>()
        );
    }
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(root, destination);
        batch.insert_before(root, destination, None);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    eval(
        &mut host,
        &permit,
        r#"Number(originDocument.documentElement === null
            && destinationDocument.documentElement === originalElement
            && originalElement.getRootNode() === destinationDocument
            && originalElement.ownerDocument === destinationDocument
            && originalElement.expando === 42)"#,
        1.0,
    );
    let replacement = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            batch.detach(root);
            batch.destroy(root);
            let replacement = batch.create_element_ns(destination, Namespace::html(), "div");
            batch.insert_before(&replacement, destination, None);
            dom.apply(batch, &cancel())
                .unwrap()
                .created(&replacement)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    assert_ne!(root, replacement);
    assert!(matches!(
        host.snapshot().unwrap().node(root),
        Err(DomError::StaleNode)
    ));
    eval(
        &mut host,
        &permit,
        r#"Number(destinationDocument.documentElement instanceof HTMLDivElement
            && destinationDocument.documentElement !== originalElement
            && destinationDocument.documentElement.expando === undefined)"#,
        1.0,
    );
    let failure = native_failure(host.in_document(&permit, |scope| {
        let code = "originalElement.localName"
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
}
