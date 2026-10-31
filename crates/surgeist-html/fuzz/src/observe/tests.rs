//! Oracle checks for the literal observation, using handwritten spec-derived sources.
use super::*;

fn parsed_nodes(source: &str) -> Vec<NativeNode> {
    let limits = crate::codec::limits(crate::codec::CAPS).expect("documented envelope");
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source.to_owned()), limits)
        .parse()
        .expect("literal parses inside the generous envelope");
    native(&report).expect("bounded native tree")
}

fn with_instruction_data(nodes: &[NativeNode], data: &str) -> Vec<NativeNode> {
    nodes
        .iter()
        .cloned()
        .map(|mut node| {
            if let HtmlNodeKind::ProcessingInstruction(instruction) = &node.kind {
                node.kind = HtmlNodeKind::ProcessingInstruction(HtmlProcessingInstruction::new(
                    instruction.target().clone(),
                    data,
                ));
            }
            node
        })
        .collect()
}

/// HTML 877d, 13.2.5.73-76: the target ends at the space, optional whitespace is
/// skipped, data accumulates until `>`, and a `?` directly before `>` is not data.
/// Both endings therefore yield the data `data`.
fn assert_literal_requires_instruction_data(id: u8, source: &str) {
    let nodes = parsed_nodes(source);
    assert!(check_literal(id, &nodes), "literal {id} on {source:?}");
    for wrong in ["", "dat", "data?", "datum", "target data"] {
        assert!(
            !check_literal(id, &with_instruction_data(&nodes, wrong)),
            "literal {id} accepted instruction data {wrong:?}"
        );
    }
}

#[test]
fn question_mark_terminated_instruction_literal_requires_its_tokenizer_data() {
    assert_literal_requires_instruction_data(5, "<?target data?><p>P</p>");
}

#[test]
fn greater_than_terminated_instruction_literal_requires_its_tokenizer_data() {
    assert_literal_requires_instruction_data(8, "<?target data><p>P</p>");
}
