use surgeist_html::*;

fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(100_000)
        .auxiliary_entries(100_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}

fn context() -> HtmlFragmentContext {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_document_template_facts(
            &document,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    let div = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&div).unwrap();
    builder.establish_no_shadow_root(&div).unwrap();
    let capture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap()
}

fn parsed(local_host: bool, mode: &str) -> (HtmlParseReport, HtmlParseCancellation) {
    let source = if local_host {
        format!("<div><template shadowrootmode={mode}><p>A</p></template></div>")
    } else {
        format!("<template shadowrootmode={mode}><p>A</p></template>")
    };
    let request = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8(source),
        context(),
        limits(),
        HtmlParserScriptingMode::Inert,
        true,
    )
    .unwrap();
    let cancellation = request.cancellation().clone();
    (request.parse().unwrap(), cancellation)
}

fn template(report: &HtmlParseReport) -> (HtmlNode<'_>, HtmlNode<'_>) {
    let mut metadata = report.metadata();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if let Some(template) = record.template() {
                    return (
                        record.node(),
                        report
                            .fragment()
                            .unwrap()
                            .node(&template.contents().id())
                            .unwrap(),
                    );
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => panic!("actual parser template metadata"),
        }
    }
}

fn shadow(report: &HtmlParseReport) -> HtmlNode<'_> {
    let effects = report.effects().unwrap();
    for index in 0..effects.len() {
        if let Some(HtmlParseEffect::AttachShadow { contents, .. }) = effects.effect(index) {
            return report.fragment().unwrap().node(&contents.id()).unwrap();
        }
    }
    panic!("actual declarative shadow effect")
}

fn assert_text_a(report: &HtmlParseReport, root: HtmlNode<'_>) {
    let fragment = report.fragment().unwrap();
    let p = fragment.node(&root.children().next().unwrap()).unwrap();
    let text = fragment.node(&p.children().next().unwrap()).unwrap();
    assert!(matches!(text.kind(), HtmlNodeKind::Text(value)
        if value.code_units() == [65]));
}

fn assert_shadow_readback_source_and_native_alias(local_host: bool) {
    let (report, cancellation) = parsed(local_host, "open");
    let usage = report.usage().clone();
    cancellation.cancel();
    let (template, target) = template(&report);
    let shadow = shadow(&report);
    assert_eq!(target.id(), shadow.id());
    assert_eq!(template.template_contents(), Some(shadow.id()));
    assert_text_a(&report, target);
    assert!(template.parent().is_none());
    assert_eq!(template.children().len(), 0);
    let fragment = report.fragment().unwrap();
    let p = fragment.node(&shadow.children().next().unwrap()).unwrap();
    let text = fragment.node(&p.children().next().unwrap()).unwrap();
    if local_host {
        let Some(HtmlFragmentOwner::ShadowHost(host)) = shadow.fragment_owner() else {
            panic!("actual local shadow host owner")
        };
        let host = fragment.node(host).unwrap();
        assert_eq!(
            host.declarative_shadow_root().unwrap().contents(),
            &shadow.id()
        );
    } else {
        assert!(shadow.fragment_owner().is_none());
    }
    let mut effect_seen = false;
    let effects = report.effects().unwrap();
    for index in 0..effects.len() {
        if let Some(HtmlParseEffect::AttachShadow {
            host,
            contents,
            source,
            ..
        }) = effects.effect(index)
        {
            effect_seen = true;
            assert_eq!(contents.id(), shadow.id());
            assert_eq!(source.source(), report.source().identity());
            assert_eq!(source.coordinates(), report.source().coordinates());
            let start = if local_host { 5 } else { 0 };
            assert_eq!(
                (source.start(), source.end()),
                (start, start + "<template shadowrootmode=open>".len())
            );
            assert_eq!(
                report.source().text(source).unwrap(),
                DocumentString::from("<template shadowrootmode=open>")
            );
            match host {
                HtmlParseEffectHost::Owned(host) => {
                    assert!(local_host);
                    assert_eq!(
                        host.declarative_shadow_root().unwrap().contents(),
                        &shadow.id()
                    );
                }
                HtmlParseEffectHost::Captured(host) => {
                    assert!(!local_host);
                    assert_eq!(host, report.context().unwrap().context_element());
                }
            }
        }
    }
    assert!(effect_seen);
    let rows = report.associations();
    let mut text_origin = false;
    for index in 0..rows.len() {
        let row = rows.row(index).unwrap();
        if let HtmlParseDestination::Field {
            node,
            field: HtmlParseField::Text,
        } = row.destination()
            && node.id() == text.id()
            && let Some(source) = row.source()
        {
            assert_eq!(source.source(), report.source().identity());
            assert_eq!(report.source().text(source).unwrap().code_units(), [65]);
            assert_eq!((row.start(), row.end()), (0, 1));
            text_origin = true;
        }
    }
    assert!(text_origin);
    assert_eq!(report.usage(), &usage);
    let template_id = template.id();
    let shadow_id = shadow.id();
    let text_id = text.id();
    let alias = fragment.clone();
    drop(report);
    assert_eq!(
        alias.node(&template_id).unwrap().template_contents(),
        Some(shadow_id.clone())
    );
    assert!(
        matches!(alias.node(&text_id).unwrap().kind(), HtmlNodeKind::Text(t) if t.code_units()==[65])
    );
    let (foreign, _) = parsed(local_host, "open");
    assert!(foreign.fragment().unwrap().node(&shadow_id).is_err());
}

#[test]
fn captured_shadow_reference_preserves_readback_source_and_native_alias() {
    assert_shadow_readback_source_and_native_alias(false);
}

#[test]
fn local_shadow_reference_preserves_readback_source_and_native_alias() {
    assert_shadow_readback_source_and_native_alias(true);
}

#[test]
fn fallback_template_retains_identical_native_and_parser_contents() {
    let (report, _) = parsed(false, "invalid");
    let (template, target) = template(&report);
    assert_eq!(template.template_contents(), Some(target.id()));
    assert!(
        matches!(target.fragment_owner(), Some(HtmlFragmentOwner::Template(owner))
        if owner == &template.id())
    );
    assert_text_a(&report, target);
}
