//! The native initial zero zone requires a strict positive natural gap.
//! Zero-flow retained text supplies a genuine ordinary fallback before the word;
//! automatic empty indications remain automatic and are subject to that gate.
use super::*;
use surgeist_text::{
    HyphenateCharacter, HyphenationCounts, HyphenationProvenance, HyphenationResource, Hyphens,
    LineEnd, OverflowWrap, PreparedBreakKind, PreparedTermination, Range, SpacingAllocations, Text,
    WhiteSpace,
};
const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
fn style() -> surgeist_text::Style {
    let mut value = ahem_style(10.0, 10.0);
    value.locale = Some("en-US".into());
    value.hyphens = Hyphens::Auto;
    value.hyphenate_character = HyphenateCharacter::String(Text::from(""));
    value.white_space = WhiteSpace::Collapse;
    value.wrap = Wrap::Word;
    value.overflow_wrap = OverflowWrap::Normal;
    value
}
fn trial(
    p: &surgeist_text::PreparedText,
    offer: surgeist_text::PreparedOpportunity,
    alternative: Option<surgeist_text::PreparedAlternative>,
) -> surgeist_text::FinalizedLine {
    let whole = crate::test_support::text_fragment::fragment_for(p).finalized_line();
    let mut request = whole.selected_line().request().clone();
    request.start = p.boundary_views().next().unwrap().handle();
    request.end = offer.view().boundary();
    request.participants = p
        .participants()
        .take(request.end.view().participant_offset())
        .collect();
    request.termination = PreparedTermination::Ordinary(offer);
    request.alternative = alternative;
    request.context.available_inline = Some(0.0);
    request.context.end = LineEnd::Soft;
    request.context.alignment = surgeist_text::Alignment::Start;
    request.context.spacing = surgeist_text::SpacingPolicy::None;
    p.select(request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn report(label: &str, line: &surgeist_text::FinalizedLine, bytes: &[u8]) {
    let selected = line.selected_line();
    let request = selected.request();
    let fit = line.measurements().fit();
    let retained = line
        .prefix_at(&surgeist_text::TextSeam::AfterParticipant(
            request.participants.last().unwrap().clone(),
        ))
        .unwrap()
        .has_retained_flow_participation();
    println!(
        "ZONE_OBSERVATION {label} source={}..{} raw={} fit={} occupation={}..{} height={} baseline={} retained={} alternative={:?}",
        request.start.view().source_after().get(),
        request.end.view().source_before().get(),
        line.feedback().advance,
        fit.advance(),
        fit.occupied_start(),
        fit.occupied_end(),
        line.feedback().line_height,
        line.feedback().baseline,
        retained,
        request
            .alternative
            .as_ref()
            .map(|alt| alt.view().definition())
    );
    assert!(
        line.run_views().all(|run| run.face().bytes() == bytes
            && run.effective_instance().unwrap().font_size() == 10.0)
    );
}
fn qualify<S: LayoutScalar>() -> (Range, Option<PreparedBreakKind>, bool, f64, f64) {
    println!("ZONE_SCALAR {}", std::any::type_name::<S>());
    let font = super::canonical::fonts::positioned_ahem(&[
        (58, -1000),
        (3, -1000),
        (67, -1000),
        (85, -1000),
    ]);
    let mut normal = style();
    normal.font = normal.font.feature("\"kern\" 0");
    let mut collapsed = normal.clone();
    collapsed.font = surgeist_text::Font::new()
        .family("Ahem")
        .feature("\"kern\" 1");
    let paragraph = PreparedParagraph::with_registered_font(
        vec![
            ParagraphPart::text(1, Source::new("X as"), collapsed),
            ParagraphPart::text(2, Source::new("sociate"), normal.clone()),
        ],
        normal.clone(),
        ParagraphBaseDirection::LeftToRight,
        &font,
    );
    let p = &paragraph.prepared;
    // Same font and literal glyphs: kern-on prefix has four actual zero cells;
    // kern-off residual has seven10px cells, including repeated s/a glyphs.
    let whole = crate::test_support::text_fragment::fragment_for(p).finalized_line();
    let actual = whole
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), font);
            run.glyphs()
                .map(|g| (g.range(), g.id(), g.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual.iter().map(|g| g.1).collect::<Vec<_>>(),
        [58, 3, 67, 85, 85, 81, 69, 75, 67, 86, 71]
    );
    for (range, _, advance) in actual {
        assert_eq!(advance, if range.end <= 4 { 0.0 } else { 10.0 });
    }
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        "X associate".encode_utf16().collect::<Vec<_>>()
    );
    let word = p
        .word_views()
        .find(|word| word.query() == Some("associate"))
        .unwrap();
    assert_eq!(word.resource(), HyphenationResource::EnUs2005);
    assert_eq!(
        word.provenance(),
        Some(HyphenationProvenance::SourceException { entry: "associate" })
    );
    let cuts = word.cuts().collect::<Vec<_>>();
    assert_eq!(
        cuts.iter()
            .map(|cut| cut.boundary().view().source_before().get())
            .collect::<Vec<_>>(),
        [4, 6]
    );
    assert_eq!(
        cuts.iter()
            .map(|cut| cut.counts().unwrap())
            .collect::<Vec<_>>(),
        [
            HyphenationCounts {
                total: 9,
                before: 2,
                after: 7
            },
            HyphenationCounts {
                total: 9,
                before: 4,
                after: 5
            }
        ]
    );
    assert!(cuts.iter().all(|cut| cut.opportunity().is_some()));
    let prefix = p
        .opportunity_views()
        .find(|row| {
            row.kind() == PreparedBreakKind::Allowed
                && row.boundary().view().source_before().get() == 2
        })
        .unwrap();
    let prefix = trial(p, prefix.handle(), None);
    report("ordinary-prefix", &prefix, &font);
    assert!(prefix.edge_units().any(|row| row.disposition()
        == surgeist_text::LineEdgeDisposition::Removed
        && row.source_pieces().collect::<Vec<_>>() == [Range::new(1, 2)]));
    assert_eq!(prefix.measurements().alignment().advance(), 0.0);

    let first = cuts[0].opportunity().unwrap();
    let alt = first.view().alternatives().next().unwrap().handle();
    assert!(alt.view().text().code_units().is_empty());
    let first = trial(p, first, Some(alt));
    report("automatic-two", &first, &font);
    let end = p
        .opportunity_views()
        .find(|row| row.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let mut residual_request = first.selected_line().request().clone();
    residual_request.start = first.selected_line().request().end.clone();
    residual_request.end = end.view().boundary();
    let from = residual_request.start.view().participant_offset();
    let to = residual_request.end.view().participant_offset();
    residual_request.participants = p.participants().skip(from).take(to - from).collect();
    residual_request.termination = PreparedTermination::Ordinary(end);
    residual_request.alternative = None;
    residual_request.context.start = surgeist_text::LineStart::Continuation;
    residual_request.context.end = LineEnd::Paragraph;
    let residual_line = p
        .select(residual_request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    report("positive-original-residual", &residual_line, &font);
    assert_eq!(residual_line.feedback().advance, 70.0);
    assert_eq!(
        (
            residual_line.measurements().fit().occupied_start(),
            residual_line.measurements().fit().occupied_end()
        ),
        (0.0, 70.0)
    );

    let second = cuts[1].opportunity().unwrap();
    let alt = second.view().alternatives().next().unwrap().handle();
    let second = trial(p, second, Some(alt));
    report("automatic-four", &second, &font);
    for line in [&prefix, &first] {
        assert_eq!(line.feedback().advance, 0.0);
        assert_eq!(line.measurements().fit().advance(), 0.0);
        assert_eq!(
            (
                line.measurements().fit().occupied_start(),
                line.measurements().fit().occupied_end()
            ),
            (0.0, 0.0)
        );
        assert!(
            line.prefix_at(&surgeist_text::TextSeam::AfterParticipant(
                line.selected_line()
                    .request()
                    .participants
                    .last()
                    .unwrap()
                    .clone()
            ))
            .unwrap()
            .has_retained_flow_participation()
        );
    }
    assert_eq!(second.feedback().advance, 20.0);
    let residual = word.portions().unwrap();
    assert_eq!(residual.len(), 1);
    // The independently summed original word is seven normal letters plus two
    // actual zero-advance letters. The registered font control below fixes em.
    let control = PreparedParagraph::plain(1, Source::new("X associate"), normal);
    let whole =
        crate::test_support::text_fragment::fragment_for(&control.prepared).finalized_line();
    assert_eq!(whole.feedback().advance, 110.0);
    report("font-control", &whole, AHEM);
    let mut glyphs = whole
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    glyphs.sort_by_key(|row| row.0.start);
    assert_eq!(
        glyphs.iter().map(|row| row.1).collect::<Vec<_>>(),
        [58, 3, 67, 85, 85, 81, 69, 75, 67, 86, 71]
    );
    assert!(glyphs.iter().all(|row| row.2 == 10.0));
    let binding = paragraph.binding(0);
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(S::ZERO), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1))
        .input(2, paragraph.text_input(2));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(0.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(
        layout.source().text().code_units(),
        "X associate".encode_utf16().collect::<Vec<_>>()
    );
    let mut lines = vec![None; layout.lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        lines[fragment.line_index()] = Some(fragment.finalized_fragment().finalized_line());
    }
    for (index, line) in lines.iter().enumerate() {
        let line = line.as_ref().unwrap();
        report(&format!("current-chosen-{index}"), line, &font);
        println!(
            "ZONE_CHOICE {} {:?} {}",
            index,
            layout.lines()[index].range(),
            layout.lines()[index].baseline()
        );
    }
    assert_eq!(layout.lines().last().unwrap().range().end, 11);
    assert_eq!(paragraph.origin(1).view().local_range(), Range::new(0, 4));
    let first = lines[0].as_ref().unwrap();
    let selected = first.selected_line();
    let kind = match &selected.request().termination {
        PreparedTermination::Ordinary(offer) => Some(offer.view().kind()),
        _ => None,
    };
    (
        layout.lines()[0].range(),
        kind,
        selected.request().alternative.is_none(),
        first.feedback().advance,
        layout.lines()[0].baseline(),
    )
}
#[test]
fn initial_zero_zone_uses_the_genuine_zero_flow_ordinary_fallback() {
    // Both complete scalar paths qualify resources, fonts, ordinary/native
    // receipts and source custody before the intended semantic RED assertion.
    let actual = [qualify::<f32>(), qualify::<f64>()];
    let expected = (
        Range::new(0, 2),
        Some(PreparedBreakKind::Allowed),
        true,
        0.0,
        8.0,
    );
    assert_eq!(actual, [expected, expected]);
}
