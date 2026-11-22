use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};
use surgeist_dom::{
    document::DomNativeInputPlatform,
    installation::{DomParsedDocumentEnvironment, DomParsedHtmlInstallLimits},
};
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlSource};
use surgeist_html_semantics::{
    HtmlAboutBaseUrl, HtmlBaseAllowed, HtmlBaseDocumentFacts, HtmlBaseEncoding,
    HtmlBaseFactsLimits, HtmlBaseFactsRevision, HtmlBaseParseObservation, HtmlBaseParseOutcome,
    HtmlBaseProducerId, HtmlBaseUrlRecord, HtmlBaseUrlScheme, HtmlParseBaseFacts,
};

pub(super) fn projected_parsed_documents_keep_original_metadata() {
    let cancel = Arc::new(AtomicBool::new(false));
    let mut dom = Dom::new(
        scripting_dom_limits(200_000, 0)
            .with_base_processing(8, 100_000)
            .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let creator = batch.create_document();
    let commit = dom.apply(batch, &cancel).unwrap();
    let origin = dom
        .snapshot()
        .document(commit.document(&creator).unwrap())
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let fixtures = [
        ("parsedQuirks", ""),
        (
            "parsedLimited",
            "<!doctype html PUBLIC '-//W3C//DTD XHTML 1.0 Transitional//EN' 'http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd'>",
        ),
        ("parsedStandards", "<!doctype html>"),
    ];
    let documents = fixtures.map(|(name, doctype)| {
        let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
        let url = HtmlBaseUrlRecord::observed(
            revision.clone(),
            "https://example.test/original/".into(),
            HtmlBaseUrlScheme::Other,
            false,
            None,
        )
        .unwrap();
        let encoding = HtmlBaseEncoding::fresh();
        let base = if name == "parsedLimited" {
            "<base href='next/'>"
        } else {
            ""
        };
        let mut request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(format!("{doctype}{base}<p>déjà</p>").as_str().into()),
            super::declarative_shadow::parse_limits(),
        );
        if !base.is_empty() {
            let frozen = HtmlBaseUrlRecord::observed(
                revision.clone(),
                "https://example.test/original/next/".into(),
                HtmlBaseUrlScheme::Other,
                false,
                None,
            )
            .unwrap();
            let observation = HtmlBaseParseObservation::new(
                "next/".into(),
                url.clone(),
                encoding.clone(),
                revision.clone(),
                HtmlBaseParseOutcome::Parsed {
                    record: frozen,
                    base_allowed: HtmlBaseAllowed::Allowed,
                },
            )
            .unwrap();
            let facts = HtmlParseBaseFacts::new(
                HtmlBaseDocumentFacts::new(
                    revision,
                    Some(url.clone()),
                    HtmlAboutBaseUrl::KnownNull,
                    false,
                    Some(encoding.clone()),
                )
                .unwrap(),
                vec![observation],
                HtmlBaseFactsLimits::new(8, 4096, 100_000),
            )
            .unwrap();
            request = request.with_base_facts(facts).unwrap();
        }
        let environment = DomParsedDocumentEnvironment::headless(
            origin.clone(),
            url,
            encoding,
            "windows-1252".into(),
            DomNativeInputPlatform::headless(),
        );
        let installed = dom
            .install_parsed_html_document(
                dom.snapshot().stamp(),
                Arc::new(request.parse().unwrap()),
                environment,
                DomParsedHtmlInstallLimits::new(64, 1_000_000, 1_000_000, 1_000_000),
                &cancel,
            )
            .unwrap();
        let identity = installed.document();
        let selected_base = installed
            .snapshot()
            .document(identity)
            .unwrap()
            .first_base()
            .unwrap();
        (name, identity, selected_base)
    });
    let (session, target) = hosted_session_with_dom(&cancel, 0, dom);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    for (name, document, _) in documents {
        expose_public_node_for_source_test(&mut host, &permit, document.node(), name);
    }
    host.in_document(&permit, |scope| {
        let observed = evaluate(scope, r"(() => {
            const cases = [[parsedQuirks, 'BackCompat'], [parsedLimited, 'CSS1Compat'], [parsedStandards, 'CSS1Compat']];
            for (const [doc, mode] of cases) {
                if (!(doc instanceof Document) || doc === document) return 'missing parsed Document';
                if (doc.URL !== 'https://example.test/original/' || doc.documentURI !== doc.URL)
                    return 'parsed Document must retain its original URL';
                if (doc.compatMode !== mode) return 'incorrect parsed compatibility mode';
                if (doc.characterSet !== 'windows-1252' || doc.charset !== doc.characterSet ||
                    doc.inputEncoding !== doc.characterSet || doc.contentType !== 'text/html')
                    return 'parsed Document must retain its selected encoding and content type';
            }
            if (parsedLimited.baseURI !== 'https://example.test/original/next/')
                return 'parsed Node base URI must use the actual frozen Base';
            const node = document.createElementNS(null, 'witness');
            const attribute = document.createAttribute('data-base');
            node.setAttributeNode(attribute);
            document.body.appendChild(node);
            if (node.baseURI !== 'about:blank' || attribute.baseURI !== 'about:blank')
                return 'the attached Node and Attr must initially use the attached Document';
            const section = document.body.appendChild(document.createElement('section'));
            section.appendChild(node);
            if (node.parentNode !== section || node.baseURI !== 'about:blank' || attribute.baseURI !== 'about:blank')
                return 'ordinary movement must preserve the current Document base URL';
            parsedLimited.adoptNode(node);
            if (node.parentNode !== null || attribute.ownerElement !== node ||
                node.ownerDocument !== parsedLimited || attribute.ownerDocument !== parsedLimited ||
                node.baseURI !== parsedLimited.baseURI || attribute.baseURI !== parsedLimited.baseURI)
                return 'adopted Node and Attr must read the destination Document and its frozen Base';
            globalThis.baseUriProbe = {node, attribute};
            return 'ok';
        })()")?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&observed)?), "ok");
        Ok(())
    }).unwrap();
    let base = documents[1].2.unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.detach(base);
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let observed = evaluate(scope, r"(() => {
            const {node, attribute} = baseUriProbe;
            return parsedLimited.baseURI === 'https://example.test/original/' &&
                node.ownerDocument === parsedLimited && node.baseURI === parsedLimited.baseURI &&
                attribute.baseURI === parsedLimited.baseURI ? 'ok' : 'retained Nodes must follow the Base removal';
        })()")?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&observed)?), "ok");
        Ok(())
    }).unwrap();
    host.stop().into_session().unwrap();
}
