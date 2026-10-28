//! Actual used-Alphabetic Text supplies Layout's named reference and child facts.
use super::*;

const ALPHA: t::FontBaseline = t::FontBaseline::Alphabetic;
const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
));

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let at = (0..usize::from(u16_at(bytes, 4)))
        .map(|i| 12 + 16 * i)
        .find(|at| bytes[*at..*at + 4] == tag)
        .unwrap();
    let start = usize::try_from(u32_at(bytes, at + 8)).unwrap();
    let len = usize::try_from(u32_at(bytes, at + 12)).unwrap();
    &bytes[start..start + len]
}
fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}
fn font() -> Vec<u8> {
    // BASE1.0 -> horizontal Axis -> romn tag -> latn BaseScript ->
    // BaseValues(index0) -> format1 signed coordinate100. All offsets are
    // relative to their owning OpenType table, not a production parser helper.
    let base: Vec<u8> = vec![
        0, 1, 0, 0, 0, 8, 0, 0, 0, 4, 0, 10, 0, 1, b'r', b'o', b'm', b'n', 0, 1, b'l', b'a', b't',
        b'n', 0, 8, 0, 6, 0, 0, 0, 0, 0, 0, 0, 1, 0, 6, 0, 1, 0, 100,
    ];
    let mut tables = (0..usize::from(u16_at(AHEM, 4)))
        .map(|i| {
            let tag: [u8; 4] = AHEM[12 + 16 * i..16 + 16 * i].try_into().unwrap();
            assert_ne!(tag, *b"BASE");
            (tag, table(AHEM, tag).to_vec())
        })
        .collect::<Vec<_>>();
    tables.push((*b"BASE", base));
    tables.sort_by_key(|(tag, _)| *tag);
    let count = u16::try_from(tables.len()).unwrap();
    let power = 1_u16 << count.ilog2();
    let mut output = vec![0; 12 + 16 * tables.len()];
    output[..4].copy_from_slice(&AHEM[..4]);
    output[4..6].copy_from_slice(&count.to_be_bytes());
    output[6..8].copy_from_slice(&(power * 16).to_be_bytes());
    output[8..10].copy_from_slice(&u16::try_from(count.ilog2()).unwrap().to_be_bytes());
    output[10..12].copy_from_slice(&(count * 16 - power * 16).to_be_bytes());
    let mut head = 0;
    for (index, (tag, mut data)) in tables.into_iter().enumerate() {
        while !output.len().is_multiple_of(4) {
            output.push(0);
        }
        if tag == *b"head" {
            data[8..12].fill(0);
            head = output.len();
        }
        let at = 12 + 16 * index;
        output[at..at + 4].copy_from_slice(&tag);
        output[at + 4..at + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        let offset = u32::try_from(output.len()).unwrap();
        output[at + 8..at + 12].copy_from_slice(&offset.to_be_bytes());
        output[at + 12..at + 16].copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        output.extend(data);
    }
    while !output.len().is_multiple_of(4) {
        output.push(0);
    }
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&output));
    output[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    output
}
fn cmap_glyph(bytes: &[u8], cp: u16) -> u16 {
    let cmap = table(bytes, *b"cmap");
    let record = (0..usize::from(u16_at(cmap, 2)))
        .map(|i| 4 + 8 * i)
        .find(|at| u16_at(cmap, *at) == 3 && u16_at(cmap, *at + 2) == 1)
        .unwrap();
    let sub = &cmap[usize::try_from(u32_at(cmap, record + 4)).unwrap()..];
    assert_eq!(u16_at(sub, 0), 4);
    let count = usize::from(u16_at(sub, 6)) / 2;
    let i = (0..count).find(|i| cp <= u16_at(sub, 14 + 2 * i)).unwrap();
    let start = u16_at(sub, 16 + 2 * count + 2 * i);
    assert!(cp >= start);
    let delta = u16_at(sub, 16 + 4 * count + 2 * i);
    let range_at = 16 + 6 * count + 2 * i;
    let range = u16_at(sub, range_at);
    if range == 0 {
        cp.wrapping_add(delta)
    } else {
        let glyph = u16_at(
            sub,
            range_at + usize::from(range) + 2 * usize::from(cp - start),
        );
        if glyph == 0 {
            0
        } else {
            glyph.wrapping_add(delta)
        }
    }
}
fn qualify_font(bytes: &[u8]) {
    assert_eq!(checksum(bytes), 0xb1b0_afba);
    for tag in [
        *b"hhea", *b"OS/2", *b"hmtx", *b"cmap", *b"glyf", *b"loca", *b"maxp",
    ] {
        assert_eq!(table(bytes, tag), table(AHEM, tag));
    }
    assert_eq!(u16_at(table(bytes, *b"head"), 18), 1000);
    assert_eq!(
        [4, 6, 8]
            .map(|at| i16::from_be_bytes(table(bytes, *b"hhea")[at..at + 2].try_into().unwrap())),
        [800, -200, 0]
    );
    assert_eq!(u16_at(table(bytes, *b"OS/2"), 62) & 0x80, 0);
    assert_eq!((cmap_glyph(bytes, 65), cmap_glyph(bytes, 66)), (35, 36));
    assert_eq!(u16_at(table(bytes, *b"head"), 50), 0);
    for glyph in [35, 36] {
        assert_eq!(u16_at(table(bytes, *b"hmtx"), glyph * 4), 1000);
        let start = usize::from(u16_at(table(bytes, *b"loca"), glyph * 2)) * 2;
        let outline = &table(bytes, *b"glyf")[start..];
        assert_eq!(
            [2, 4, 6, 8].map(|at| i16::from_be_bytes(outline[at..at + 2].try_into().unwrap())),
            [0, -200, 1000, 800]
        );
    }
    let base = table(bytes, *b"BASE");
    assert_eq!((u16_at(base, 0), u16_at(base, 2)), (1, 0));
    assert_eq!(u16_at(base, 6), 0);
    let axis = usize::from(u16_at(base, 4));
    let tags = axis + usize::from(u16_at(base, axis));
    assert_eq!(u16_at(base, tags), 1);
    assert_eq!(&base[tags + 2..tags + 6], b"romn");
    let scripts = axis + usize::from(u16_at(base, axis + 2));
    assert_eq!(u16_at(base, scripts), 1);
    assert_eq!(&base[scripts + 2..scripts + 6], b"latn");
    let script = scripts + usize::from(u16_at(base, scripts + 6));
    assert_eq!((u16_at(base, script + 2), u16_at(base, script + 4)), (0, 0));
    let values = script + usize::from(u16_at(base, script));
    assert_eq!((u16_at(base, values), u16_at(base, values + 2)), (0, 1));
    let coord = values + usize::from(u16_at(base, values + 4));
    assert_eq!(u16_at(base, coord), 1);
    assert_eq!(
        i16::from_be_bytes(base[coord + 2..coord + 4].try_into().unwrap()),
        100
    );
}
fn raw_instance(
    instance: t::EffectiveFontInstance<'_>,
    catalogue: &t::FontCatalogueIdentity,
    size: f32,
) {
    assert_eq!(instance.font_size(), size);
    assert_eq!(instance.axis_count(), 0);
    let metrics = instance.metrics(catalogue).unwrap();
    let scale = f64::from(size) / 10.0;
    assert_eq!(
        (metrics.ascent(), metrics.descent(), metrics.leading()),
        (8.0 * scale, -2.0 * scale, 0.0)
    );
    let measured = instance
        .baseline(
            catalogue,
            t::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
            t::FontBaselineAxis::HorizontalY,
            ALPHA,
        )
        .unwrap();
    let t::FontBaselineOutcome::Available(value) = measured.outcome() else {
        panic!("actual romn100 must be available: {measured:?}")
    };
    assert_eq!(value.coordinate(), scale);
    assert_eq!(
        value.provenance(),
        t::FontBaselineProvenance::BaseCoordinate
    );
    assert_eq!(value.base_script(), Some(*b"latn"));
    assert_eq!(value.absence(), None);
}

struct Captured {
    prepared: t::PreparedText,
    source: t::Source,
}
fn style(size: f32, height: f64) -> t::Style {
    t::Style {
        font: t::Font::new().family("Ahem"),
        size,
        line_height: t::LineHeight::Absolute(height),
        white_space: t::WhiteSpace::Preserve,
        word_break: t::WordBreak::BreakAll,
        wrap: t::Wrap::Word,
        ..t::Style::default()
    }
}
fn capture(
    system: &mut t::System,
    raw: &str,
    style: t::Style,
    orientation: t::LineOrientation,
    atomic: bool,
) -> Captured {
    let mut source = t::Source::new(raw);
    if atomic {
        source.inline_box(t::InlineBox::new(
            t::Id::from_u64(2),
            t::InlineBoxKind::InFlow,
            1,
            t::Size::new(6.0, 14.0),
        ));
    }
    let ranges = if atomic {
        vec![t::Range::new(0, 1), t::Range::new(1, 2)]
    } else {
        vec![t::Range::new(0, raw.encode_utf16().count())]
    };
    use t::SourceProcessingEvent as E;
    let mut events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Origin {
            origin: 0,
            affinity: t::Affinity::After,
        },
    ];
    let mut boundaries = Vec::new();
    if atomic {
        events.extend([
            E::Atomic {
                atomic: 0,
                affinity: t::Affinity::After,
            },
            E::Connection(0),
            E::Origin {
                origin: 1,
                affinity: t::Affinity::After,
            },
        ]);
        boundaries.extend([
            t::ParagraphBoundaryDefinition::AtomicBefore(0),
            t::ParagraphBoundaryDefinition::AtomicAfter(0),
            t::ParagraphBoundaryDefinition::Connection(0),
        ]);
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    let processed = Arc::new(
        t::process_contextual_source(t::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::SourceProcessingEnvelope::Complete,
            origins: ranges
                .into_iter()
                .map(|range| t::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: range,
                    aggregate: range,
                    kind: t::ParagraphOriginKind::Authored,
                })
                .collect(),
            controls: vec![],
            connections: if atomic {
                vec![t::SourceProcessingConnection {
                    before_origin: 0,
                    after_origin: 1,
                    kind: t::ParagraphConnectionKind::Continuous,
                }]
            } else {
                vec![]
            },
            boundary_policies: boundaries
                .into_iter()
                .map(|definition| t::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: t::Wrap::Word,
                    following_wrap: t::Wrap::Word,
                    ancestor_wrap: t::Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            computed_line_boundaries: vec![],
            scopes: vec![t::SourceProcessingScope::Block {
                topology: t::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: t::ParagraphBaseDirection::LeftToRight,
            }],
            paragraphs: vec![t::SourceProcessingParagraph {
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: t::ParagraphStrut::DefaultFont,
            }],
            events,
        })
        .unwrap(),
    );
    let analysis = t::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        t::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analysis
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| t::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap: t::Wrap::Word,
        })
        .collect();
    let analysis = analysis.with_wrapping_policies(policies).unwrap();
    let options = t::PreparationOptions::default()
        .with_orientation(orientation)
        .with_baseline_reference(ALPHA);
    let prepared = system
        .prepare(&analysis, options)
        .expect("the complete genuine used-Alphabetic producer must succeed before Layout");
    assert_eq!(prepared.baseline_reference(), Some(ALPHA));
    assert_eq!(
        prepared
            .preparation_options()
            .requested_baseline_reference(),
        Some(ALPHA)
    );
    assert_eq!(prepared.preparation_options(), options);
    assert_eq!(prepared.analyzed_source().identity(), analysis.identity());
    assert_eq!(
        prepared.captured_paragraph(),
        processed.captured_paragraph(0)
    );
    assert_eq!(prepared.orientation(), orientation);
    assert_eq!(prepared.atomic_views().count(), usize::from(atomic));
    Captured { prepared, source }
}
fn select(
    parent: &Captured,
    start: t::PreparedBoundary,
    offer: t::PreparedOpportunity,
    atomics: Vec<t::AtomicAllocation>,
    available: f64,
) -> t::FinalizedLine {
    let from = start.view().participant_offset();
    let end = offer.view().boundary();
    let to = end.view().participant_offset();
    let terminal = offer.view().kind() == t::PreparedBreakKind::ParagraphEnd;
    parent
        .prepared
        .select(t::LineSelectionRequest {
            start,
            end,
            termination: t::PreparedTermination::Ordinary(offer),
            participants: parent
                .prepared
                .participants()
                .skip(from)
                .take(to - from)
                .collect(),
            alternative: None,
            incoming: None,
            atomics,
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: t::LineContext {
                alignment_insets: t::LineAlignmentInsets::ZERO,
                formatted_line: t::FormattedLineContext::default(),
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                line_box_direction: t::LineBoxDirection::LeftToRight,
                orientation: parent.prepared.orientation(),
                start: if from == 0 {
                    t::LineStart::Paragraph
                } else {
                    t::LineStart::Continuation
                },
                end: if terminal {
                    t::LineEnd::Paragraph
                } else {
                    t::LineEnd::Forced
                },
                available_inline: Some(available),
                indent: 0.0,
                alignment: t::Alignment::Start,
                tabs: t::TabContext::NoTabs,
                spacing: t::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(t::SpacingAllocations::None)
        .unwrap()
}
fn terminal(p: &t::PreparedText) -> t::PreparedOpportunity {
    p.opportunity_views()
        .find(|row| row.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle()
}
fn placed<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    container: u32,
) -> &t::Layout {
    batch
        .placed_text()
        .iter()
        .find(|row| row.container() == container)
        .unwrap()
        .layout()
}
fn space<S: l::LayoutScalar>(width: f64, height: f64) -> l::Size<l::AvailableOf<S>> {
    l::Size::new(
        l::AvailableOf::definite(n(width)),
        l::AvailableOf::definite(n(height)),
    )
}
fn solve<S: l::LayoutScalar>(
    host: &Host<S>,
    bindings: Vec<l::TextParagraphBinding<u32>>,
    width: f64,
    height: f64,
) -> l::CompletedLayoutBatchOf<u32, S> {
    let flow = l::LayoutFlowInput::try_new(host, 0, bindings, l::WrappingInput::default())
        .expect("actual issued origins/atomic must bind to real tree members");
    l::compute_layout_with_flow(
        &flow,
        0,
        l::LayoutRootRequestOf::viewport(space(width, height)).unwrap(),
    )
    .expect("genuine used Alphabetic preparation must supply Layout Named(Alphabetic)")
}
fn child_input<S: l::LayoutScalar>(
    cw: bool,
    context: l::InlineBaselineContext,
) -> l::NodeInputOf<S> {
    let mut input = root(
        if cw {
            l::WritingMode::SidewaysRl
        } else {
            l::WritingMode::HorizontalTb
        },
        l::Direction::Ltr,
        context,
    );
    input.display = l::Display::InlineBlock;
    input.size = l::Size::new(px(6.0), px(14.0));
    input
}
fn child_binding(child: &Captured, container: u32, text: u32) -> l::TextParagraphBinding<u32> {
    l::TextParagraphBinding {
        prepared: child.prepared.clone(),
        container,
        nodes: vec![text],
        origins: vec![l::TextOriginBinding {
            origin: child.prepared.origin_views().next().unwrap().handle(),
            node: text,
        }],
        atomics: vec![],
        controls: vec![],
        line_strut: None,
    }
}
fn unknown<S: l::LayoutScalar>(facts: l::NamedBaselinesOf<S>) {
    for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
        let pair = facts.get(name);
        assert_eq!(
            (pair.first(), pair.last()),
            (l::BaselineFactOf::Unknown, l::BaselineFactOf::Unknown)
        );
    }
}
fn alpha<S: l::LayoutScalar>(
    facts: l::NamedBaselinesOf<S>,
    axis: l::PhysicalAxis,
    first: f64,
    last: f64,
) {
    let pair = facts.get(l::BaselineName::Alphabetic);
    assert_eq!(
        pair.first(),
        l::BaselineFactOf::Available(l::BaselineCoordinateOf::try_new(axis, n(first)).unwrap())
    );
    assert_eq!(
        pair.last(),
        l::BaselineFactOf::Available(l::BaselineCoordinateOf::try_new(axis, n(last)).unwrap())
    );
    let other = facts.get(l::BaselineName::Central);
    assert_eq!(
        (other.first(), other.last()),
        (l::BaselineFactOf::Unknown, l::BaselineFactOf::Unknown)
    );
}
fn qualify_child_text(child: &Captured, bytes: &[u8], cw: bool) {
    let p = &child.prepared;
    let forced = p
        .opportunity_views()
        .find(|row| row.kind() == t::PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let first = select(
        child,
        p.boundary_views().next().unwrap().handle(),
        forced.clone(),
        vec![],
        if cw { 14.0 } else { 6.0 },
    );
    let last = select(
        child,
        forced.view().boundary(),
        terminal(p),
        vec![],
        if cw { 14.0 } else { 6.0 },
    );
    for line in [&first, &last] {
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (5.0, 4.0, 6.0)
        );
    }
    let refs = if cw { [2.0, -4.0] } else { [4.0, 10.0] };
    let text = t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: p.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: [(first, refs[0]), (last, refs[1])]
            .into_iter()
            .map(|(line, baseline)| {
                t::PlacementEntry::Formatted(t::LinePlacement {
                    line,
                    anchor: t::LineAnchor {
                        inline_origin: 0.0,
                        baseline,
                    },
                    orientation: p.orientation(),
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap();
    assert_eq!(
        text.lines()
            .iter()
            .map(|line| line.baseline())
            .collect::<Vec<_>>(),
        refs
    );
    assert_eq!(text.source(), &child.source);
    let origins = text
        .run_views()
        .map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            raw_instance(
                run.effective_instance().unwrap(),
                p.font_catalogue_identity(),
                5.0,
            );
            let raw = run
                .baseline(
                    p.font_catalogue_identity(),
                    t::FontBaselineAxis::HorizontalY,
                    ALPHA,
                )
                .unwrap();
            assert_eq!(
                raw.script(),
                t::FontBaselineScript::from_iso15924(*b"Latn").unwrap()
            );
            let t::FontBaselineOutcome::Available(value) = raw.outcome() else {
                panic!("actual child Latin romn is available")
            };
            assert_eq!(value.coordinate(), 0.5);
            assert_eq!(
                run.glyph_basis(),
                if cw {
                    t::GlyphBasis::Clockwise
                } else {
                    t::GlyphBasis::Unrotated
                }
            );
            if cw { run.origin().x } else { run.origin().y }
        })
        .collect::<Vec<_>>();
    assert_eq!(origins, if cw { vec![1.5, -4.5] } else { vec![4.5, 10.5] });
}
fn qualify_physical_child<S: l::LayoutScalar>(child: &Captured, cw: bool) {
    let mut host = Host::<S>::new(child_input(cw, l::InlineBaselineContext::Physical));
    let binding = child_binding(child, 0, 1);
    host.push(
        Some(0),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(binding.origins[0].origin.clone())),
    );
    let batch = solve(&host, vec![binding], 6.0, 14.0);
    let text = placed(&batch, 0);
    assert_eq!(
        text.lines()
            .iter()
            .map(|line| line.baseline())
            .collect::<Vec<_>>(),
        if cw { vec![2.0, -4.0] } else { vec![4.0, 10.0] }
    );
    assert_eq!(
        node(batch.unrounded_entries(), 0).size,
        l::Size::new(n(6.0), n(14.0))
    );
    unknown(node(batch.unrounded_entries(), 0).named_baselines);
    unknown(node(batch.final_entries(), 0).named_baselines);
    assert_eq!(text.source(), &child.source);
    assert_eq!(
        text.run_views()
            .map(|run| if cw { run.origin().x } else { run.origin().y })
            .collect::<Vec<_>>(),
        if cw { vec![1.5, -4.5] } else { vec![4.5, 10.5] }
    );
}

#[derive(Clone, Copy)]
struct Case {
    mode: l::WritingMode,
    source: l::BaselineSource,
    baseline: f64,
    before: f64,
    height: f64,
    child_x: f64,
    child_y: f64,
    font_zero: f64,
    child_reference: f64,
}
const CASES: [Case; 8] = [
    Case {
        mode: l::WritingMode::HorizontalTb,
        source: l::BaselineSource::First,
        baseline: 7.0,
        before: 7.0,
        height: 17.0,
        child_x: 10.0,
        child_y: 3.0,
        font_zero: 8.0,
        child_reference: 4.0,
    },
    Case {
        mode: l::WritingMode::HorizontalTb,
        source: l::BaselineSource::Last,
        baseline: 10.0,
        before: 10.0,
        height: 14.0,
        child_x: 10.0,
        child_y: 0.0,
        font_zero: 11.0,
        child_reference: 10.0,
    },
    Case {
        mode: l::WritingMode::SidewaysRl,
        source: l::BaselineSource::First,
        baseline: 33.0,
        before: 7.0,
        height: 10.0,
        child_x: 31.0,
        child_y: 10.0,
        font_zero: 32.0,
        child_reference: 2.0,
    },
    Case {
        mode: l::WritingMode::SidewaysRl,
        source: l::BaselineSource::Last,
        baseline: 30.0,
        before: 10.0,
        height: 13.0,
        child_x: 34.0,
        child_y: 10.0,
        font_zero: 29.0,
        child_reference: -4.0,
    },
    Case {
        mode: l::WritingMode::VerticalRl,
        source: l::BaselineSource::First,
        baseline: 33.0,
        before: 7.0,
        height: 10.0,
        child_x: 31.0,
        child_y: 10.0,
        font_zero: 32.0,
        child_reference: 2.0,
    },
    Case {
        mode: l::WritingMode::VerticalRl,
        source: l::BaselineSource::Last,
        baseline: 30.0,
        before: 10.0,
        height: 13.0,
        child_x: 34.0,
        child_y: 10.0,
        font_zero: 29.0,
        child_reference: -4.0,
    },
    Case {
        mode: l::WritingMode::VerticalLr,
        source: l::BaselineSource::First,
        baseline: 3.0,
        before: 7.0,
        height: 10.0,
        child_x: 1.0,
        child_y: 10.0,
        font_zero: 2.0,
        child_reference: 2.0,
    },
    Case {
        mode: l::WritingMode::VerticalLr,
        source: l::BaselineSource::Last,
        baseline: 3.0,
        before: 10.0,
        height: 13.0,
        child_x: 7.0,
        child_y: 10.0,
        font_zero: 2.0,
        child_reference: -4.0,
    },
];
fn parent_allocation(parent: &Captured, case: Case) -> t::AtomicAllocation {
    let cw = case.mode != l::WritingMode::HorizontalTb;
    let p = case.child_reference;
    let before = if cw { 6.0 - p } else { p };
    let rect = if cw {
        t::Rect::new(0.0, -before, 14.0, 6.0)
    } else {
        t::Rect::new(0.0, -before, 6.0, 14.0)
    };
    t::AtomicAllocation {
        occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
        allocation: t::AtomicLineAllocation::InFlow {
            flow: t::AtomicFlowAllocation {
                advance: if cw { 14.0 } else { 6.0 },
                before_baseline: before,
                after_baseline: if cw { p } else { 14.0 - p },
                occupied_start: 0.0,
                occupied_end: if cw { 14.0 } else { 6.0 },
            },
            box_geometry: t::AtomicBoxAllocation {
                border: rect,
                content: rect,
                baseline: 0.0,
            },
            alignment: t::LineBoxAlignment::Baseline,
        },
    }
}
fn qualify_parent_text(parent: &Captured, bytes: &[u8], case: Case) {
    let cw = case.mode != l::WritingMode::HorizontalTb;
    let line = select(
        parent,
        parent.prepared.boundary_views().next().unwrap().handle(),
        terminal(&parent.prepared),
        vec![parent_allocation(parent, case)],
        40.0,
    );
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (if cw { 34.0 } else { 26.0 }, case.before, case.height)
    );
    let reference = if cw { 100.0 } else { 70.0 };
    let border = if cw {
        t::Rect::new(reference - case.child_reference, 30.0, 6.0, 14.0)
    } else {
        t::Rect::new(30.0, reference - case.child_reference, 6.0, 14.0)
    };
    let text = t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: parent.prepared.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: vec![t::PlacementEntry::Formatted(t::LinePlacement {
            line,
            anchor: t::LineAnchor {
                inline_origin: 20.0,
                baseline: reference,
            },
            orientation: parent.prepared.orientation(),
            atomics: vec![t::AtomicPlacement {
                occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                border,
                content: border,
                baseline: reference,
            }],
        })],
    })
    .unwrap();
    assert_eq!(text.lines()[0].baseline(), reference);
    assert_eq!(text.inline_boxes()[0].rect(), border);
    for run in text.run_views() {
        assert_eq!(run.face().bytes(), bytes);
        raw_instance(
            run.effective_instance().unwrap(),
            parent.prepared.font_catalogue_identity(),
            10.0,
        );
        let raw = run
            .baseline(
                parent.prepared.font_catalogue_identity(),
                t::FontBaselineAxis::HorizontalY,
                ALPHA,
            )
            .unwrap();
        assert_eq!(
            raw.script(),
            t::FontBaselineScript::from_iso15924(*b"Latn").unwrap()
        );
        let t::FontBaselineOutcome::Available(value) = raw.outcome() else {
            panic!("actual parent Latin romn is available")
        };
        assert_eq!(value.coordinate(), 1.0);
        assert_eq!(
            if cw { run.origin().x } else { run.origin().y },
            if cw { 99.0 } else { 71.0 }
        );
    }
}
fn actual<S: l::LayoutScalar>(bytes: &[u8], parent: &Captured, child: &Captured, case: Case) {
    let cw = case.mode != l::WritingMode::HorizontalTb;
    let context = l::InlineBaselineContext::Named(l::BaselineName::Alphabetic);
    let mut input = root(case.mode, l::Direction::Ltr, context);
    input.size = l::Size::new(px(40.0), px(40.0));
    let mut host = Host::<S>::new(input);
    let origins = parent
        .prepared
        .origin_views()
        .map(|row| row.handle())
        .collect::<Vec<_>>();
    host.push(
        Some(0),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origins[0].clone())),
    );
    let mut input = child_input(cw, context);
    input.baseline_source = case.source;
    assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(input)), 2);
    host.push(
        Some(0),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origins[1].clone())),
    );
    let inner = child_binding(child, 2, 4);
    host.push(
        Some(2),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(inner.origins[0].origin.clone())),
    );
    let outer = l::TextParagraphBinding {
        prepared: parent.prepared.clone(),
        container: 0,
        nodes: vec![1, 2, 3],
        origins: origins
            .into_iter()
            .zip([1, 3])
            .map(|(origin, node)| l::TextOriginBinding { origin, node })
            .collect(),
        atomics: vec![l::TextAtomicBinding {
            occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
            node: 2,
        }],
        controls: vec![],
        line_strut: None,
    };
    let batch = solve(&host, vec![outer, inner], 40.0, 40.0);
    let axis = if cw {
        l::PhysicalAxis::Horizontal
    } else {
        l::PhysicalAxis::Vertical
    };
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        alpha(
            node(entries, 0).named_baselines,
            axis,
            case.baseline,
            case.baseline,
        );
        let output = node(entries, 2);
        alpha(
            output.named_baselines,
            axis,
            if cw { 2.0 } else { 4.0 },
            if cw { -4.0 } else { 10.0 },
        );
        assert_eq!(
            output.location,
            l::Point::new(n(case.child_x), n(case.child_y))
        );
        assert_eq!(output.size, l::Size::new(n(6.0), n(14.0)));
        assert_eq!(
            if cw {
                output.location.x.to_f64()
            } else {
                output.location.y.to_f64()
            } + case.child_reference,
            case.baseline
        );
    }
    let text = placed(&batch, 0);
    assert_eq!(text.source(), &parent.source);
    assert_eq!(
        text.analyzed_source().identity(),
        parent.prepared.analyzed_source().identity()
    );
    assert_eq!(text.lines().len(), 1);
    assert_eq!(text.lines()[0].baseline(), case.baseline);
    let border = t::Rect::new(case.child_x, case.child_y, 6.0, 14.0);
    assert_eq!(text.inline_boxes()[0].rect(), border);
    let pens = text
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(
                run.glyph_basis(),
                if cw {
                    t::GlyphBasis::Clockwise
                } else {
                    t::GlyphBasis::Unrotated
                }
            );
            assert_eq!(
                if cw { run.origin().x } else { run.origin().y },
                case.font_zero
            );
            run.glyphs()
                .map(|g| {
                    assert_eq!((g.x(), g.y(), g.advance()), (0.0, 0.0, 10.0));
                    (
                        g.id(),
                        if cw { run.origin().y } else { run.origin().x } + g.pen(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(pens, [(35, 0.0), (36, if cw { 24.0 } else { 16.0 })]);
    let child_text = placed(&batch, 2);
    assert_eq!(child_text.source(), &child.source);
    let child_origins = child_text
        .run_views()
        .map(|run| if cw { run.origin().x } else { run.origin().y })
        .collect::<Vec<_>>();
    assert_eq!(
        child_origins,
        if cw {
            vec![case.child_x + 1.5, case.child_x - 4.5]
        } else {
            vec![case.child_y + 4.5, case.child_y + 10.5]
        }
    );
    let line = batch
        .unrounded_inline_fragments()
        .iter()
        .find(|row| matches!(row.owner(), l::InlineFragmentOwner::Node(1)))
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert_eq!(line.prepared_text(), parent.prepared);
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (case.before, case.height)
    );
    assert_eq!(
        line.atomic_slots().next().unwrap().occurrence,
        parent.prepared.atomic_views().next().unwrap().handle()
    );
}
fn core<S: l::LayoutScalar>() {
    let bytes = font();
    qualify_font(&bytes);
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let face = system.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(face.bytes(), bytes);
    for size in [5.0, 10.0] {
        raw_instance(
            face.unsynthesized_instance(size, &[]).unwrap(),
            system.font_catalogue_identity(),
            size,
        );
    }
    let child_h = capture(
        &mut system,
        "A\nB",
        style(5.0, 6.0),
        t::LineOrientation::Horizontal,
        false,
    );
    let child_cw = capture(
        &mut system,
        "A\nB",
        style(5.0, 6.0),
        t::LineOrientation::Sideways,
        false,
    );
    qualify_child_text(&child_h, &bytes, false);
    qualify_child_text(&child_cw, &bytes, true);
    // Physical readback is already supported and truthfully remains Unknown.
    // It qualifies actual child placement before the old Named gate is called.
    qualify_physical_child::<S>(&child_h, false);
    qualify_physical_child::<S>(&child_cw, true);
    let parent_h = capture(
        &mut system,
        "AB",
        style(10.0, 10.0),
        t::LineOrientation::Horizontal,
        true,
    );
    let parent_cw = capture(
        &mut system,
        "AB",
        style(10.0, 10.0),
        t::LineOrientation::Sideways,
        true,
    );
    // All standalone new-producer controls precede the intended Layout RED.
    for case in CASES {
        qualify_parent_text(
            if case.mode == l::WritingMode::HorizontalTb {
                &parent_h
            } else {
                &parent_cw
            },
            &bytes,
            case,
        );
    }
    for case in CASES {
        let cw = case.mode != l::WritingMode::HorizontalTb;
        actual::<S>(
            &bytes,
            if cw { &parent_cw } else { &parent_h },
            if cw { &child_cw } else { &child_h },
            case,
        );
    }
}

#[test]
fn actual_used_alphabetic_matches_real_child_first_last_h_and_clockwise_f32() {
    core::<f32>();
}
#[test]
fn actual_used_alphabetic_matches_real_child_first_last_h_and_clockwise_f64() {
    core::<f64>();
}
