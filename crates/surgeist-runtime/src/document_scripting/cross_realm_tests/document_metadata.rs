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
    HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
    HtmlBaseUrlScheme,
};

#[test]
fn projected_parsed_documents_keep_original_metadata() {
    let cancel = Arc::new(AtomicBool::new(false));
    let mut dom = Dom::new(scripting_dom_limits(200_000, 0)).unwrap();
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
        let report = HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(format!("{doctype}<p>déjà</p>").as_str().into()),
            super::declarative_shadow::parse_limits(),
        )
        .parse()
        .unwrap();
        let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
        let url = HtmlBaseUrlRecord::observed(
            revision,
            "https://example.test/original/".into(),
            HtmlBaseUrlScheme::Other,
            false,
            None,
        )
        .unwrap();
        let environment = DomParsedDocumentEnvironment::headless(
            origin.clone(),
            url,
            HtmlBaseEncoding::fresh(),
            "windows-1252".into(),
            DomNativeInputPlatform::headless(),
        );
        let installed = dom
            .install_parsed_html_document(
                dom.snapshot().stamp(),
                Arc::new(report),
                environment,
                DomParsedHtmlInstallLimits::new(64, 1_000_000, 1_000_000, 1_000_000),
                &cancel,
            )
            .unwrap();
        (name, installed.document())
    });
    let (session, target) = hosted_session_with_dom(&cancel, 0, dom);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    for (name, document) in documents {
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
            return 'ok';
        })()")?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&observed)?), "ok");
        Ok(())
    }).unwrap();
    host.stop().into_session().unwrap();
}
