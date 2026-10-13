use surgeist_html::*;

#[test]
fn color_input_preserves_red_default_and_publishes_sanitized_current_with_form_sibling() {
    // HTML877d #update-a-color-well-control-color and
    // #serialize-a-color-well-control-color: the fresh clean value uses the
    // authored attribute. CSS Color 4's red is opaque sRGB (255, 0, 0); absent
    // alpha and the default Limited sRGB state request lowercase HTML hex.
    let literal = "<!doctype html><form id=f><input type=color value=red>Z</form>";
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits());
    let report = request
        .parse()
        .expect("ordinary Color input must complete its source-defined value update");
    let state = report.document_state().unwrap();
    assert_eq!(state.readiness(), HtmlParseReadiness::Complete);
    assert_eq!(state.completion(), HtmlParseCompletion::Complete);
    assert!(state.ready_for_post_load_tasks());
    assert_eq!(report.source().as_utf8(), Some(literal));

    let document = report.document().unwrap();
    assert_eq!(document.mode(), HtmlDocumentMode::NoQuirks);
    let roots: Vec<_> = document.root().children().collect();
    assert_eq!(roots.len(), 2);
    assert!(matches!(
        document.node(&roots[0]).unwrap().kind(),
        HtmlNodeKind::Doctype(_)
    ));
    let html = document.node(&roots[1]).unwrap();
    let branches: Vec<_> = html.children().collect();
    assert_eq!(branches.len(), 2);
    let body = document.node(&branches[1]).unwrap();
    let forms: Vec<_> = body.children().collect();
    assert_eq!(forms.len(), 1);
    let form = document.node(&forms[0]).unwrap();
    let HtmlNodeKind::Element(form_element) = form.kind() else {
        panic!("native form")
    };
    assert_eq!(
        form_element.name().local_name(),
        &DocumentString::from("form")
    );
    assert_eq!(
        form_element.attributes().as_slice()[0].value(),
        &DocumentString::from("f")
    );
    let children: Vec<_> = form.children().collect();
    assert_eq!(children.len(), 2);
    let input = document.node(&children[0]).unwrap();
    let HtmlNodeKind::Element(element) = input.kind() else {
        panic!("native input")
    };
    assert_eq!(element.name().local_name(), &DocumentString::from("input"));
    assert_eq!(element.attributes().len(), 2);
    for (attribute, (name, value)) in element
        .attributes()
        .iter()
        .zip([("type", "color"), ("value", "red")])
    {
        assert_eq!(attribute.name().namespace().uri(), None);
        assert_eq!(attribute.name().local_name(), &DocumentString::from(name));
        assert_eq!(attribute.value(), &DocumentString::from(value));
    }
    assert_eq!(input.children().count(), 0);
    assert!(matches!(
        document.node(&children[1]).unwrap().kind(),
        HtmlNodeKind::Text(value) if value == &DocumentString::from("Z")
    ));

    let (mut inputs, mut owners) = (0, 0);
    let mut metadata = report.metadata();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if record.node().id() != input.id() {
                    continue;
                }
                if let Some(flags) = record.input_flags() {
                    inputs += 1;
                    assert_eq!(flags.kind().input_type(), HtmlInputType::Color);
                    assert_eq!(flags.kind().value_mode(), HtmlInputValueMode::Value);
                    assert!(!flags.dirty_value());
                    assert!(!flags.dirty_checkedness());
                    assert!(!flags.checkedness());
                    assert!(!flags.indeterminate());
                    assert!(!flags.user_validity());
                    assert_eq!(
                        record.input_raw_value(),
                        Some(&DocumentString::from("#ff0000"))
                    );
                    assert_eq!(record.input_creation_writeback(), Some(false));
                }
                if let Some(owner) = record.form_owner() {
                    owners += 1;
                    assert_eq!(owner.unwrap().id(), form.id());
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!((inputs, owners), (1, 1));
}

fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
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
