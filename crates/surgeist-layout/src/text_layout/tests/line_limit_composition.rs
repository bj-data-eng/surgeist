//! Block-owned limits composed through the public Text/Layout boundary.
use super::*;
use surgeist_text::{
    HyphenateCharacter, HyphenationCounts, HyphenationProvenance, HyphenationResource, Hyphens,
    PreparedAlternativeDefinition, Range, Text,
};

fn paragraph(node: u32) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.hyphens = Hyphens::Auto;
    style.locale = Some("en-US".into());
    style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
    let paragraph = PreparedParagraph::plain(node, Source::new("cooperate"), style);
    // Independent pinned pattern oracle: 9 letters, cuts after 2/4/6. The
    // retained Ahem letters and indication each advance one em (10px).
    let words = paragraph.prepared.word_views().collect::<Vec<_>>();
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].query(), Some("cooperate"));
    assert_eq!(words[0].resource(), HyphenationResource::EnUs2005);
    assert_eq!(
        paragraph
            .prepared
            .hyphenation_cut_views()
            .map(|cut| {
                assert_eq!(cut.provenance(), HyphenationProvenance::Pattern);
                assert!(cut.opportunity().is_some());
                cut.counts().unwrap()
            })
            .collect::<Vec<_>>(),
        [2, 4, 6].map(|before| HyphenationCounts {
            total: 9,
            before,
            after: 9 - before
        })
    );
    paragraph
}

fn block<S: LayoutScalar>(limit: HyphenateLimitLines) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        hyphenate_limit_lines: limit,
        ..NodeInputOf::default()
    }
}

fn tree<S: LayoutScalar>(
    first: &PreparedParagraph,
    second: &PreparedParagraph,
    first_limit: HyphenateLimitLines,
    second_limit: HyphenateLimitLines,
) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(0, [10, 20])
        .children(10, [1])
        .children(20, [2])
        .children(1, [])
        .children(2, [])
        // Native inputs are already computed: the actual child container's
        // value controls its lines, rather than borrowing the parent's zero.
        .style(0, block(HyphenateLimitLines::Count(0)))
        .style(10, block(first_limit))
        .style(20, block(second_limit))
        .input(1, first.text_input(1))
        .input(2, second.text_input(2))
}

fn assert_block<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
    container: u32,
    node: u32,
    expected: &[(Range, f64)],
) {
    let placed = batch
        .placed_text()
        .iter()
        .find(|p| p.container() == container)
        .unwrap();
    assert_eq!(placed.nodes(), &[node]);
    assert_eq!(
        placed.layout().source().text().code_units(),
        "cooperate".encode_utf16().collect::<Vec<_>>()
    );
    let lines = placed.layout().lines();
    assert_eq!(lines.len(), expected.len());
    for (index, ((range, advance), line)) in expected.iter().zip(lines).enumerate() {
        assert_eq!(line.range(), *range);
        let fragment = batch
            .final_inline_fragments()
            .iter()
            .find(|entry| {
                entry.owner() == &crate::InlineFragmentOwner::Node(node)
                    && entry.fragment().line_index() == index
            })
            .unwrap();
        let finalized = fragment.fragment().finalized_fragment().finalized_line();
        assert_eq!(finalized.prepared_text(), paragraph.prepared);
        assert_eq!(finalized.feedback().advance, *advance);
        let selected = finalized.selected_line();
        let ending = selected.request().alternative.as_ref();
        if range.end < 9 {
            let definition = ending
                .expect("an actual automatic ending is selected")
                .view()
                .definition();
            let PreparedAlternativeDefinition::Automatic { cut } = definition else {
                panic!("the fixture contains no authored discretionary ending");
            };
            assert_eq!(cut.view().counts().unwrap().before, range.end);
        } else {
            assert!(ending.is_none());
            assert_eq!(
                selected.request().context.end,
                surgeist_text::LineEnd::Paragraph
            );
        }
    }
}

fn assert_nested_container_limits<S: LayoutScalar>() {
    let first = paragraph(1);
    let second = paragraph(2);
    let capped = [(Range::new(0, 2), 30.0), (Range::new(2, 9), 70.0)];
    let uncapped = [
        (Range::new(0, 2), 30.0),
        (Range::new(2, 4), 30.0),
        (Range::new(4, 6), 30.0),
        (Range::new(6, 9), 30.0),
    ];
    let original = tree::<S>(
        &first,
        &second,
        HyphenateLimitLines::Count(1),
        HyphenateLimitLines::NoLimit,
    );
    let capture = LayoutFlowInput::try_new(
        &original,
        0,
        vec![first.binding(10), second.binding(20)],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let swapped = tree::<S>(
        &first,
        &second,
        HyphenateLimitLines::NoLimit,
        HyphenateLimitLines::Count(1),
    );
    let swapped_capture = LayoutFlowInput::try_new(
        &swapped,
        0,
        vec![first.binding(10), second.binding(20)],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    // Both computations reuse the same genuine preparations. Changing which
    // block owns the cap must not rewrite dictionary offers or leak state.
    for (input, first_expected, second_expected) in [
        (&capture, capped.as_slice(), uncapped.as_slice()),
        (&swapped_capture, uncapped.as_slice(), capped.as_slice()),
        (&capture, capped.as_slice(), uncapped.as_slice()),
    ] {
        let batch = compute_layout_with_flow(input, 0, request(30.0)).unwrap();
        assert_eq!(batch.placed_text().len(), 2);
        assert_block(&batch, &first, 10, 1, first_expected);
        assert_block(&batch, &second, 20, 2, second_expected);
        let root = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(root.size.height, S::from_f64(60.0));
    }
}

#[test]
fn nested_block_limits_use_actual_container_values_and_fresh_computation_state() {
    assert_nested_container_limits::<f32>();
    assert_nested_container_limits::<f64>();
}
