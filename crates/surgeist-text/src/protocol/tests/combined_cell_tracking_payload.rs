//! Real width packets stay horizontal and untracked inside the outer Cell.
//! The adopted predecessor-owned boundary means affect flow, not its E10 square.

#[path = "combined_cell_tracking_numeric.rs"]
mod numeric;

use super::*;

#[path = "combined_cell_observation_mixed.rs"]
mod observation_mixed;

#[path = "combined_cell_allocated_source.rs"]
mod allocated_source;

#[path = "combined_cell_sideways_tracking.rs"]
mod sideways;

struct Fixture {
    processed: Arc<ProcessedSource>,
    analyzed: AnalyzedSource,
    parent: FontData,
    body: FontData,
    inner: Range,
    leading_b: bool,
    trailing_b: bool,
}

fn public_packet(
    bytes: &[u8],
    text: &str,
    request: Option<([u8; 4], surgeist_shaper::WidthFeature)>,
    glyph: u32,
    advance: i32,
) {
    public_packet_with_outcome(
        bytes,
        text,
        request,
        glyph,
        advance,
        surgeist_shaper::WidthInputOutcome::Covered,
    );
}

fn public_packet_with_outcome(
    bytes: &[u8],
    text: &str,
    request: Option<([u8; 4], surgeist_shaper::WidthFeature)>,
    glyph: u32,
    advance: i32,
    outcome: surgeist_shaper::WidthInputOutcome,
) {
    use surgeist_shaper::{
        Direction, Feature, FontRef, ShaperData, ShapingStatus, Tag, UnicodeBuffer,
        WidthObservationCoverage, script,
    };
    let font = FontRef::new(bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    let mut input = UnicodeBuffer::new();
    for (index, character) in text.chars().enumerate() {
        input.add(character, u32::try_from(index).unwrap());
    }
    input.set_direction(Direction::LeftToRight);
    input.set_script(script::LATIN);
    input.set_language("en".parse().unwrap());
    let features = if let Some((tag, observation)) = request {
        input.observe_width_feature(observation);
        vec![Feature::new(Tag::new(&tag), 1, ..)]
    } else {
        vec![]
    };
    let buffer = shaper.shape(input, &features);
    assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
    assert_eq!(
        buffer
            .glyph_infos()
            .iter()
            .map(|info| (info.glyph_id, info.cluster))
            .collect::<Vec<_>>(),
        (0..u32::try_from(text.len()).unwrap())
            .map(|index| (glyph, index))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        buffer
            .glyph_positions()
            .iter()
            .map(|position| (
                position.x_advance,
                position.y_advance,
                position.x_offset,
                position.y_offset
            ))
            .collect::<Vec<_>>(),
        vec![(advance, 0, 0, 0); text.len()]
    );
    if let Some((_, feature)) = request {
        let receipt = buffer.width_feature_receipt().unwrap().unwrap();
        assert_eq!(receipt.feature(), feature);
        assert_eq!(receipt.input_len(), text.len());
        assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
        for index in 0..text.len() {
            assert_eq!(receipt.input(index).unwrap().outcome(), outcome);
        }
    }
}

fn fixture(
    inner: &str,
    tracking: f64,
    leading_b: bool,
    trailing_b: bool,
    size: f32,
) -> (System, Fixture) {
    assert!(inner.chars().all(|character| character == 'A'));
    fixture_with_processing(
        inner,
        tracking,
        leading_b,
        trailing_b,
        size,
        WhiteSpace::Preserve,
        false,
    )
}

fn fixture_with_processing(
    inner: &str,
    tracking: f64,
    leading_b: bool,
    trailing_b: bool,
    size: f32,
    white_space: WhiteSpace,
    uppercase: bool,
) -> (System, Fixture) {
    fixture_with_profile(
        inner,
        tracking,
        leading_b,
        trailing_b,
        size,
        FixtureProfile {
            white_space,
            uppercase,
            typography: AnalysisTypography::Mixed,
            reference: font_reference(),
            exterior_spaces: false,
        },
    )
}

struct FixtureProfile {
    white_space: WhiteSpace,
    uppercase: bool,
    typography: AnalysisTypography,
    reference: ScopeFontReference,
    exterior_spaces: bool,
}

fn fixture_with_profile(
    inner: &str,
    tracking: f64,
    leading_b: bool,
    trailing_b: bool,
    size: f32,
    profile: FixtureProfile,
) -> (System, Fixture) {
    let FixtureProfile {
        white_space,
        uppercase,
        typography,
        reference,
        exterior_spaces,
    } = profile;
    assert!(!exterior_spaces || (leading_b && trailing_b));
    assert!(
        inner
            .chars()
            .all(|character| matches!(character, 'A' | ' ' | 'ß'))
    );
    assert!(!inner.is_empty());
    // This deliberately small fixture admits plain edge SPACE plus uniform
    // visible A, or the stable Unicode uppercase expansion ß -> SS. It does
    // not assert generic scalar counts for arbitrary EGCs or fonts.
    let transformed = if uppercase {
        inner.to_uppercase()
    } else {
        inner.to_owned()
    };
    let used = transformed.trim_matches(' ');
    assert!(!used.is_empty());
    assert!(used.chars().all(|ch| ch == 'A') || used.chars().all(|ch| ch == 'S'));
    assert!(matches!(
        white_space,
        WhiteSpace::Preserve | WhiteSpace::Collapse
    ));
    assert!(white_space == WhiteSpace::Collapse || !inner.contains(' '));
    let parent = parent_font();
    let body = composition_font();
    glyph_frame_font_premises(&parent);
    glyph_frame_font_premises(&body);
    // Qualify these actual native-table font bytes through the public provider,
    // independently of Text's candidate choice and of every tracking result.
    public_packet(&parent, "B", None, 36, 1000);
    use skrifa::MetadataProvider;
    let parsed = skrifa::FontRef::new(&body).unwrap();
    let scalar = used.chars().next().unwrap();
    let glyph = parsed.charmap().map(scalar).unwrap().to_u32();
    assert_eq!(
        sfnt::u16_at(
            sfnt::table(&body, *b"hmtx"),
            usize::try_from(glyph).unwrap() * 4
        ),
        1000
    );
    public_packet(&body, used, None, glyph, 1000);
    for (tag, feature, glyph, advance) in [
        (*b"hwid", surgeist_shaper::WidthFeature::HalfWidth, 36, 500),
        (*b"twid", surgeist_shaper::WidthFeature::ThirdWidth, 58, 400),
        (
            *b"qwid",
            surgeist_shaper::WidthFeature::QuarterWidth,
            16,
            300,
        ),
    ] {
        if scalar == 'A' {
            public_packet(&body, used, Some((tag, feature)), glyph, advance);
        } else {
            // The maintained derivative edits width substitutions only for A.
            // Qualify actual S/SS NoAction on each exact executed width plan.
            let source_glyph = parsed.charmap().map('S').unwrap().to_u32();
            public_packet_with_outcome(
                &body,
                used,
                Some((tag, feature)),
                source_glyph,
                1000,
                surgeist_shaper::WidthInputOutcome::NoAction,
            );
        }
    }
    let outside = Style {
        font: Font::new().family("Ahem"),
        size,
        line_height: LineHeight::Absolute(f64::from(size)),
        white_space: WhiteSpace::Preserve,
        text_transform: TextTransform::none(),
        text_combine_upright: TextCombineUpright::None,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        wrap: Wrap::Word,
        ..Style::default()
    };
    let inside = Style {
        font: Font::new().family(crate::shaping_test_fonts::COMBINED_WIDTH_FAMILY),
        text_combine_upright: TextCombineUpright::All,
        letter_spacing: tracking,
        white_space,
        text_transform: if uppercase {
            TextTransform::Set(TextTransformSet::none().with_case(CaseTransform::Uppercase))
        } else {
            TextTransform::none()
        },
        ..outside.clone()
    };
    let prefix = if leading_b {
        if exterior_spaces { "B " } else { "B" }
    } else {
        ""
    };
    let suffix = if trailing_b {
        if exterior_spaces { " B" } else { "B" }
    } else {
        ""
    };
    let text = format!("{prefix}{inner}{suffix}");
    let start = prefix.encode_utf16().count();
    let range = Range::new(start, start + inner.encode_utf16().count());
    let text_len = text.encode_utf16().count();
    let mut source = Source::identified(text.as_str(), Id::from_u64(784_944), 1);
    if leading_b {
        source.span(Range::new(0, range.start), outside.clone());
    }
    source.span(range, inside.clone());
    if trailing_b {
        source.span(Range::new(range.end, text_len), outside.clone());
    }
    let mut events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
    ];
    for index in 0..text_len {
        if index > 0 {
            events.push(SourceProcessingEvent::Connection(index - 1));
        }
        if index == range.start {
            events.push(SourceProcessingEvent::EnterScope(1));
        }
        events.push(SourceProcessingEvent::Origin {
            origin: index,
            affinity: Affinity::After,
        });
        if index + 1 == range.end {
            events.push(SourceProcessingEvent::ExitScope(1));
        }
    }
    events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: (0..text_len)
                .map(|index| ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(index, index + 1),
                    aggregate: Range::new(index, index + 1),
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..text_len - 1)
                .map(|index| SourceProcessingConnection {
                    before_origin: index,
                    after_origin: index + 1,
                    kind: ParagraphConnectionKind::Continuous,
                })
                .collect(),
            boundary_policies: (0..text_len - 1)
                .map(|index| ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(index),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            controls: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows: vec![],
            scope_alignment_limits: Some(ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            scopes: vec![
                SourceProcessingScope::Block {
                    topology: SourceProcessingTopology::Complete,
                    trim: WhiteSpaceTrim::default(),
                    base_style: outside,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(reference),
                },
                SourceProcessingScope::Inline {
                    trim: WhiteSpaceTrim::default(),
                    style: Some(inside),
                    boundaries: None,
                    alignment_boundary: None,
                    font_reference: Some(reference),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            events,
        })
        .unwrap(),
    );
    assert_eq!(processed.source(), &source);
    let mut phase_one_inner = String::new();
    let mut previous_space = false;
    for ch in inner.chars() {
        if white_space != WhiteSpace::Collapse || ch != ' ' || !previous_space {
            phase_one_inner.push(ch);
        }
        previous_space = ch == ' ';
    }
    let phase_one = format!("{prefix}{phase_one_inner}{suffix}");
    assert_eq!(processed.processed_text(), &Text::from(phase_one.as_str()));
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        typography,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    let owner = analyzed.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    assert!(owner.joint_flow().is_none());
    assert_eq!(analyzed.combined_cells().len(), 1);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.selection_unit_count(), transformed.chars().count());
    assert_eq!(cell.used_unit_count(), used.chars().count());
    assert_eq!(
        cell.used_code_points().collect::<Vec<_>>(),
        used.chars().map(u32::from).collect::<Vec<_>>()
    );
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    let ancestry = inner
        .chars()
        .enumerate()
        .flat_map(|(index, ch)| {
            std::iter::repeat_n(
                (range.start + index, ch != ' '),
                if uppercase && ch == 'ß' { 2 } else { 1 },
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(pieces.len(), ancestry.len());
    for (piece, (index, retained)) in pieces.iter().zip(ancestry) {
        assert_eq!(piece.source(), processed.source());
        assert_eq!(piece.local(), Range::new(index, index + 1));
        assert_eq!(piece.aggregate(), Range::new(index, index + 1));
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(index).unwrap()
        );
        assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
        assert_eq!(piece.is_used(), retained);
        assert_eq!(piece.has_presentation_emission(), retained);
    }
    let scope = processed.captured_scope(1).unwrap();
    assert_eq!(scope.style().unwrap().letter_spacing, tracking);
    assert_eq!(scope.style().unwrap().word_spacing, 0.0);
    assert!(scope.style().unwrap().font.features.is_empty());
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(parent.clone()).unwrap();
    system.register_fonts(body.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 2);
    let parent = system
        .registered_font_faces()
        .find(|face| face.bytes() == parent.as_slice())
        .unwrap()
        .to_owned();
    let body = system
        .registered_font_faces()
        .find(|face| face.bytes() == body.as_slice())
        .unwrap()
        .to_owned();
    assert!(!parent.face().is_same_face(&body.face()));
    (
        system,
        Fixture {
            processed,
            analyzed,
            parent,
            body,
            inner: range,
            leading_b,
            trailing_b,
        },
    )
}

// (line advance, square start, cell flow end), then the actual inner winning
// glyph/advance/scale. All expected values come from input/font/spec operands.
fn output(
    fixture: &Fixture,
    prepared: &PreparedText,
    line: &FinalizedLine,
    outer: (f64, f64, f64),
    packet: (u32, f32, f64),
) {
    let (advance, square_start, flow_end) = outer;
    let (glyph, raw_advance, scale) = packet;
    assert_eq!(line.feedback().advance, advance);
    assert_eq!(line.measurements().fit().advance(), advance);
    assert_eq!(line.measurements().alignment().advance(), advance);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual cell");
    };
    assert_eq!(cell.square().origin.x, square_start);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(cell.square().origin.y, cell.reference() - 5.0);
    assert_eq!(cell.scope_displacement().get(), 0.0);
    assert_eq!(
        cell.source_cell().scope(),
        fixture.processed.captured_scope(1).unwrap()
    );
    let mut inner = Vec::new();
    let mut ordinary = Vec::new();
    for run in line.run_views() {
        let instance = run.effective_instance().unwrap();
        assert_eq!(instance.font_size(), 10.0);
        assert_eq!(instance.axis_count(), 0);
        if run.cell().is_some() {
            assert!(run.face().is_same_face(&fixture.body.face()));
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            for value in run.glyphs() {
                inner.push((
                    value.range(),
                    value.id(),
                    run.origin().x + value.pen(),
                    value.x(),
                    value.y(),
                    value.advance(),
                ));
            }
        } else {
            assert!(run.face().is_same_face(&fixture.parent.face()));
            assert_eq!(run.glyph_basis(), GlyphBasis::Clockwise);
            for value in run.glyphs() {
                ordinary.push((
                    value.range(),
                    value.id(),
                    run.origin().x + value.pen(),
                    value.advance(),
                ));
            }
        }
    }
    assert_eq!(
        inner,
        (fixture.inner.start..fixture.inner.end)
            .enumerate()
            .map(|(index, source)| (
                Range::new(source, source + 1),
                glyph,
                f64::from(u32::try_from(index).unwrap()) * f64::from(raw_advance),
                0.0,
                0.0,
                raw_advance
            ))
            .collect::<Vec<_>>()
    );
    let mut expected_ordinary = Vec::new();
    if fixture.leading_b {
        expected_ordinary.push((Range::new(0, 1), 36, 0.0, 10.0));
    }
    if fixture.trailing_b {
        expected_ordinary.push((
            Range::new(fixture.inner.end, fixture.inner.end + 1),
            36,
            flow_end,
            10.0,
        ));
    }
    assert_eq!(ordinary, expected_ordinary);
    let layout = placed_vertical_cell(prepared, line.clone());
    assert_eq!(layout.source(), fixture.processed.source());
    let mut body_points = Vec::new();
    let mut outside_points = Vec::new();
    for run in layout.glyph_runs() {
        let frame = run.glyph_frame();
        let body = run.font().face().is_same_face(&fixture.body.face());
        if body {
            assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
            close(frame.outline_x().x, scale);
            assert_eq!(frame.outline_x().y, 0.0);
            assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
            assert_eq!(run.font_size(), 10.0);
        } else {
            assert!(run.font().face().is_same_face(&fixture.parent.face()));
        }
        let pen = frame.pen_direction();
        for value in run.glyphs() {
            let x = f64::from(value.x()) + value.pen() * pen.x;
            let y = f64::from(value.y()) + value.pen() * pen.y;
            let point = Point::new(
                frame.origin().x + frame.outline_x().x * x + frame.outline_y().x * y,
                frame.origin().y + frame.outline_x().y * x + frame.outline_y().y * y,
            );
            if body {
                body_points.push(point);
            } else {
                outside_points.push((value.range(), point.y));
            }
        }
    }
    assert_eq!(body_points.len(), fixture.inner.len());
    for (index, point) in body_points.iter().enumerate() {
        close(
            point.x,
            95.0 + scale * f64::from(u32::try_from(index).unwrap()) * f64::from(raw_advance),
        );
        close(point.y, 28.0 + square_start);
    }
    let mut expected_outside = Vec::new();
    if fixture.leading_b {
        expected_outside.push((Range::new(0, 1), 20.0));
    }
    if fixture.trailing_b {
        expected_outside.push((
            Range::new(fixture.inner.end, fixture.inner.end + 1),
            20.0 + flow_end,
        ));
    }
    assert_eq!(outside_points, expected_outside);
    let start = Cursor::new(fixture.inner.start, Affinity::After);
    let end = Cursor::new(fixture.inner.end, Affinity::Before);
    assert_eq!(
        layout.cursor(start).unwrap().rect(),
        Rect::new(95.0, 20.0 + square_start, 10.0, 1.0)
    );
    assert_eq!(
        layout.cursor(end).unwrap().rect(),
        Rect::new(95.0, 20.0 + flow_end, 10.0, 1.0)
    );
    let expected_selection = SelectionGeometry::new(vec![SelectionRect::new(
        Rect::new(95.0, 20.0 + square_start, 10.0, flow_end - square_start),
        0,
    )]);
    assert_eq!(
        layout.selection(Selection::new(start, end)).unwrap(),
        expected_selection
    );
    let positions = layout.positions().collect::<Vec<_>>();
    let first = positions
        .iter()
        .find(|position| position.source_cursor() == start)
        .unwrap();
    let last = positions
        .iter()
        .find(|position| position.source_cursor() == end)
        .unwrap();
    let selection = layout.position_selection(first, last).unwrap();
    assert_eq!(selection.geometry(), &expected_selection);
    let expected_text = Text::from("A".repeat(fixture.inner.len()));
    assert_eq!(selection.source_text(), &expected_text);
    assert_eq!(selection.plaintext(), &expected_text);
    assert_eq!(selection.source_pieces().len(), fixture.inner.len());
    for (piece, index) in selection
        .source_pieces()
        .iter()
        .zip(fixture.inner.start..fixture.inner.end)
    {
        assert_eq!(piece.local_range(), Range::new(index, index + 1));
        assert_eq!(piece.aggregate_range(), Range::new(index, index + 1));
        assert_eq!(
            piece.origin().view().capture_position(),
            Some(fixture.processed.captured_origin(index).unwrap())
        );
    }
}

fn case(
    inner: &str,
    tracking: f64,
    leading_b: bool,
    trailing_b: bool,
    size: f32,
    outer: (f64, f64, f64),
    packet: (u32, f32, f64),
) {
    let (mut system, fixture) = fixture(inner, tracking, leading_b, trailing_b, size);
    let prepared = system
        .prepare(
            &fixture.analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(
        prepared.analyzed_source().identity(),
        fixture.analyzed.identity()
    );
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one prepared cell");
    };
    assert_eq!(cell.extent(), 10.0);
    assert_eq!((cell.before(), cell.after()), (5.0, 5.0));
    assert!(cell.has_provider_input());
    let metrics = prepared
        .scope_metrics(&fixture.processed.captured_scope(1).unwrap())
        .unwrap();
    assert!(
        metrics
            .effective_instance()
            .unwrap()
            .face()
            .is_same_face(&fixture.body.face())
    );
    let line = finalize(&prepared);
    output(&fixture, &prepared, &line, outer, packet);
    drop(system);
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    output(&fixture, &prepared, &replay, outer, packet);
}

#[test]
fn positive_outer_tracking_keeps_the_real_hwid_packet_and_fixed_square() {
    // Boundary means(0+2)/2=1. B10+1+Cell10+1+B10=32;
    // content11..21, flow end22. Inner hwid remains two5-unit glyphs, scale1.
    case(
        "AA",
        2.0,
        true,
        true,
        10.0,
        (32.0, 11.0, 22.0),
        (36, 5.0, 1.0),
    );
}

#[test]
fn negative_outer_tracking_overlaps_flow_without_shrinking_the_inner_hwid_packet() {
    // Boundary means(0-2)/2=-1. B10-1+Cell10-1+B10=28;
    // content9..19, flow end18 (placed caret38), not content end19.
    case(
        "AA",
        -2.0,
        true,
        true,
        10.0,
        (28.0, 9.0, 18.0),
        (36, 5.0, 1.0),
    );
}

#[test]
fn line_edge_tracking_adds_no_outer_term_to_the_compressed_qwid_packet() {
    // One outer unit has no boundary mean, even with authored2. Normal40,
    // hwid20, twid16, qwid12 leave the qualified qwid packet at scale5/6.
    case(
        "AAAA",
        2.0,
        false,
        false,
        10.0,
        (10.0, 0.0, 10.0),
        (16, 3.0, 5.0 / 6.0),
    );
}
