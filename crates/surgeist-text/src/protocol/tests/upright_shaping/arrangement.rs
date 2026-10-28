//! Real provider and selected-presentation witnesses for upright arrangement.
use super::*;
use crate::{
    FinalizedFragmentOwner, GlyphBasis, HyphenationPlacement, Hyphens, NativeHyphenationConvention,
    Text,
};
use icu_properties::{CodePointMapData, props::LineBreak as UnicodeLineBreak};

fn cmap(mappings: &[(u32, u32)]) -> Vec<u8> {
    let mut rows = mappings.to_vec();
    rows.sort_unstable();
    let mut table = words(&[0, 1, 3, 10]);
    table.extend(12_u32.to_be_bytes());
    table.extend(words(&[12, 0]));
    table.extend(u32::try_from(16 + 12 * rows.len()).unwrap().to_be_bytes());
    table.extend(0_u32.to_be_bytes());
    table.extend(u32::try_from(rows.len()).unwrap().to_be_bytes());
    for (cp, glyph) in rows {
        table.extend(cp.to_be_bytes());
        table.extend(cp.to_be_bytes());
        table.extend(glyph.to_be_bytes());
    }
    table
}

fn native(font: &[u8], tags: &[[u8; 4]]) -> Vec<u8> {
    let font = with_native_metrics(font);
    let mut tags = tags.to_vec();
    tags.sort_unstable();
    sfnt::replace_tables(
        &font,
        vec![(
            *b"BASE",
            base::base(
                0,
                None,
                Some(base::Axis {
                    tags: vec![*b"ideo", *b"idtp"],
                    scripts: tags
                        .into_iter()
                        .map(|tag| base::Script {
                            tag,
                            default_index: 0,
                            coordinates: vec![base::Coord::Design(0), base::Coord::Design(1200)],
                        })
                        .collect(),
                }),
                None,
            ),
        )],
    )
}

fn context(input: &[u16], nested: u16) -> Vec<u8> {
    // ContextSubst3, substitute the first input only. The nested single
    // substitution is not independently enabled by a feature.
    let start = 10 + 2 * input.len();
    let mut table = words(&[3, offset(input.len()), 1]);
    for index in 0..input.len() {
        table.extend(words(&[offset(start + 6 * index)]));
    }
    table.extend(words(&[0, nested]));
    for &glyph in input {
        table.extend(words(&[1, 1, glyph]));
    }
    table
}

fn mark_positioning(bases: &[u16]) -> Vec<u8> {
    let mark_coverage = 26 + 8 * bases.len();
    let mut table = words(&[
        1,
        offset(mark_coverage),
        offset(mark_coverage + 6),
        1,
        12,
        24,
    ]);
    table.extend(words(&[1, 0, 6, 1, 100, 200]));
    table.extend(words(&[offset(bases.len())]));
    for index in 0..bases.len() {
        table.extend(words(&[offset(2 + 2 * bases.len() + 6 * index)]));
    }
    for _ in bases {
        table.extend(words(&[1, 400, 600]));
    }
    table.extend(words(&[1, 1, 100, 1, offset(bases.len())]));
    table.extend(words(bases));
    table
}

fn tibetan_font() -> Vec<u8> {
    // Whole Ka+tsek+shad sees 35->277. The genuine cut after tsek sees only
    // Ka+tsek and takes35->274. A shad+Kha leak would incorrectly take36->82.
    // Global vert is intentional: the USE owner's ccmp is per-syllable and
    // cannot independently witness the larger arrangement-context contract.
    let gsub = ot_layout(
        *b"tibt",
        vec![(*b"vert", vec![0, 1, 2])],
        vec![
            (5, 8, context(&[35, 58, 36], 3)),
            (5, 8, context(&[35, 58], 4)),
            (5, 8, context(&[36, 37], 5)),
            (1, 0, single(35, 277)),
            (1, 0, single(35, 274)),
            (1, 0, single(36, 82)),
        ],
    );
    let gpos = ot_layout(
        *b"tibt",
        vec![(*b"mark", vec![0])],
        vec![(4, 0, mark_positioning(&[35, 274, 277]))],
    );
    let gdef = words(&[
        1, 0, 12, 0, 0, 0, 2, 6, 35, 37, 1, 58, 58, 1, 82, 82, 1, 100, 100, 3, 274, 274, 1, 277,
        277, 1,
    ]);
    let mut hmtx = sfnt::table(AHEM, *b"hmtx").to_vec();
    hmtx[100 * 4..100 * 4 + 2].copy_from_slice(&0_u16.to_be_bytes());
    native(
        &sfnt::replace_tables(
            AHEM,
            vec![
                (
                    *b"cmap",
                    cmap(&[
                        (0x20, 3),
                        (0xf08, 36),
                        (0xf0b, 58),
                        (0xf0c, 58),
                        (0xf0d, 36),
                        (0xf40, 35),
                        (0xf41, 37),
                        (0xf72, 100),
                    ]),
                ),
                (*b"GSUB", gsub),
                (*b"GPOS", gpos),
                (*b"GDEF", gdef),
                (*b"hmtx", hmtx),
            ],
        ),
        &[*b"latn", *b"tibt"],
    )
}

fn indication_font() -> Vec<u8> {
    native(
        &sfnt::replace_tables(
            &arabic_font(),
            vec![(
                *b"cmap",
                cmap(&[
                    (0x20, 3),
                    (0x28, 36),
                    (0x29, 37),
                    (0x31, 58),
                    (0x41, 35),
                    (0x42, 36),
                    (0x43, 37),
                    (0x58, 58),
                    (0x5d0, 82),
                    (0x628, 35),
                    (0x64e, 100),
                    (0x200c, 252),
                    (0x200d, 253),
                ]),
            )],
        ),
        &[*b"arab", *b"hebr", *b"latn"],
    )
}

fn qualify_native(bytes: &[u8], mappings: &[(u32, u32)], script: [u8; 4]) {
    let face = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(face.maxp().unwrap().num_glyphs(), 281);
    assert!(face.table_data(skrifa::Tag::new(b"fvar")).is_none());
    assert!(face.glyf().is_ok());
    for &(cp, glyph) in mappings {
        assert_eq!(face.charmap().map(cp), Some(GlyphId::new(glyph)));
    }
    let scripts = face.gsub().unwrap().script_list().unwrap();
    assert_eq!(scripts.script_records().len(), 1);
    assert_eq!(
        scripts.script_records()[0].script_tag().to_be_bytes(),
        script
    );
    let metrics = face.glyph_metrics(Size::unscaled(), LocationRef::default());
    for (glyph, ymax) in [(35, 800.0), (274, 600.0), (277, 800.0)] {
        assert_eq!(metrics.bounds(GlyphId::new(glyph)).unwrap().y_max, ymax);
        let native = &face.vmtx().unwrap().v_metrics()[usize::try_from(glyph).unwrap()];
        assert_eq!(
            (native.advance.get(), native.side_bearing.get()),
            (1200, 100)
        );
    }
    let axis = face.base().unwrap().vert_axis().unwrap().unwrap();
    let scripts = axis.base_script_list().unwrap();
    for record in scripts.base_script_records() {
        let values = record
            .base_script(scripts.offset_data())
            .unwrap()
            .base_values()
            .unwrap()
            .unwrap();
        assert_eq!(
            values
                .base_coords()
                .iter()
                .map(|v| v.unwrap().coordinate())
                .collect::<Vec<_>>(),
            [0, 1200]
        );
    }
    assert!(face.gpos().unwrap().lookup_list().is_ok());
}

fn own_style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

fn prepare(system: &mut System, text: &str, style: Style) -> PreparedText {
    let source = Source::new(text);
    let len = source.text().len();
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            ParagraphInput {
                source: source.clone(),
                base_style: style,
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                origins: vec![crate::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(0, len),
                    aggregate: Range::new(0, len),
                    kind: crate::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            AnalysisTypography::Upright {
                computed_direction: ParagraphBaseDirection::RightToLeft,
            },
        )
        .unwrap(),
        crate::Wrap::Word,
    );
    assert_eq!(analyzed.source(), &source);
    assert_eq!(analyzed.direction(), Direction::LeftToRight);
    system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap()
}

fn start(p: &PreparedText) -> PreparedBoundary {
    p.boundary_views().next().unwrap().handle()
}
fn terminal(p: &PreparedText) -> PreparedOpportunity {
    p.opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle()
}
fn select_request(
    p: &PreparedText,
    from: PreparedBoundary,
    offer: PreparedOpportunity,
) -> LineSelectionRequest {
    let mut r = request(p);
    r.start = from;
    r.end = offer.view().boundary();
    let first = r.start.view().participant_offset();
    let last = r.end.view().participant_offset();
    r.participants = p.participants().skip(first).take(last - first).collect();
    r.termination = PreparedTermination::Ordinary(offer.clone());
    r.context.orientation = LineOrientation::Vertical;
    r.context.base_direction = ParagraphBaseDirection::LeftToRight;
    r.context.line_box_direction = LineBoxDirection::LeftToRight;
    r.context.start = if first == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    r.context.end = if offer.view().kind() == PreparedBreakKind::ParagraphEnd {
        LineEnd::Paragraph
    } else {
        LineEnd::Soft
    };
    r.context.available_inline = None;
    r.context.alignment = crate::Alignment::Start;
    r.context.spacing = SpacingPolicy::None;
    r
}
fn finish(p: &PreparedText, r: LineSelectionRequest) -> FinalizedLine {
    p.select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn ids(line: &FinalizedLine) -> Vec<u32> {
    line.run_views()
        .flat_map(|run| run.glyphs().map(|g| g.id()).collect::<Vec<_>>())
        .collect()
}
fn place(p: &PreparedText, lines: Vec<FinalizedLine>) -> crate::Layout {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                crate::PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 20.0,
                        baseline: 100.0 - 20.0 * index as f64,
                    },
                    orientation: LineOrientation::Vertical,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap()
}

#[test]
fn tibetan_real_vert_context_survives_tsek_cut_and_rebuilds_the_selected_orphan() {
    // Actual Unicode17 data and pinned UAX14rev55 LB13 forbid BA->EX,
    // so a tsek followed by ordinary shad is not an offered-break witness.
    // LB12a permits BA->GL; LB12 still forbids GL->GL. SBRUL SHAD0F08
    // belongs to the adopted arrangement set and supplies that genuine case.
    // https://www.unicode.org/Public/17.0.0/ucd/LineBreak.txt
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB12a
    let classes = CodePointMapData::<UnicodeLineBreak>::new();
    assert_eq!(classes.get32(0xf08), UnicodeLineBreak::Glue);
    assert_eq!(classes.get32(0xf0b), UnicodeLineBreak::BreakAfter);
    assert_eq!(classes.get32(0xf0c), UnicodeLineBreak::Glue);
    assert_eq!(classes.get32(0xf0d), UnicodeLineBreak::Exclamation);
    let bytes = tibetan_font();
    qualify_native(
        &bytes,
        &[
            (0xf08, 36),
            (0xf40, 35),
            (0xf72, 100),
            (0xf0b, 58),
            (0xf0c, 58),
            (0xf0d, 36),
            (0xf41, 37),
        ],
        *b"tibt",
    );
    let mut system = system_with(&bytes);
    let shad = prepare(
        &mut system,
        "\u{f40}\u{f72}\u{f0b}\u{f0d}\u{f41}",
        own_style(),
    );
    let whole_shad = finish(&shad, select_request(&shad, start(&shad), terminal(&shad)));
    assert_eq!(ids(&whole_shad), [277, 100, 58, 36, 37]);
    assert_eq!(whole_shad.feedback().advance, 48.0);
    assert!(
        !shad
            .opportunity_views()
            .any(|o| o.kind() == PreparedBreakKind::Allowed
                && o.boundary().view().source_before().get() == 3)
    );
    let text = "\u{f40}\u{f72}\u{f0b}\u{f08}\u{f41}";
    let p = prepare(&mut system, text, own_style());
    let egcs = vec![
        Range::new(0, 2),
        Range::new(2, 3),
        Range::new(3, 4),
        Range::new(4, 5),
    ];
    assert_eq!(p.analyzed_source().graphemes().collect::<Vec<_>>(), egcs);
    let whole = finish(&p, select_request(&p, start(&p), terminal(&p)));
    assert_eq!(ids(&whole), [277, 100, 58, 36, 37]);
    assert_eq!(whole.feedback().advance, 48.0);
    let offered = p
        .opportunity_views()
        .find(|o| {
            o.kind() == PreparedBreakKind::Allowed && o.boundary().view().source_before().get() == 3
        })
        .expect("actual U+0F0B ordinary cut")
        .handle();
    let first = finish(&p, select_request(&p, start(&p), offered.clone()));
    let second = finish(
        &p,
        select_request(&p, offered.view().boundary(), terminal(&p)),
    );
    assert_eq!(
        first.selected_line().request().termination,
        PreparedTermination::Ordinary(offered.clone())
    );
    assert_eq!(
        second.selected_line().request().start,
        offered.view().boundary()
    );
    assert_eq!(
        ids(&first),
        [274, 100, 58],
        "actual selected prefix lacks the following sbrul-shad context for277"
    );
    assert_eq!(
        ids(&second),
        [36, 37],
        "leading shad cannot leak context into the next Kha cell"
    );
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (24.0, 24.0)
    );
    let layout = place(&p, vec![first, second]);
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(94.0, 20.0, 12.0, 24.0)
    );
    assert_eq!(
        layout.lines()[1].bounds(),
        Rect::new(74.0, 20.0, 12.0, 24.0)
    );
    let first_positions = layout
        .run_views()
        .filter(|r| r.line() == 0)
        .flat_map(|r| {
            r.glyphs()
                .map(|g| {
                    (
                        g.id(),
                        Point::new(
                            r.origin().x + f64::from(g.x()),
                            r.origin().y + g.pen() + f64::from(g.y()),
                        ),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        first_positions,
        [
            (274, Point::new(94.0, 27.0)),
            (100, Point::new(97.0, 23.0)),
            (58, Point::new(94.0, 41.0))
        ]
    );
    assert_eq!(
        layout.source().text().code_units(),
        text.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(p.analyzed_source().graphemes().collect::<Vec<_>>(), egcs);
    let nonbreaking = prepare(
        &mut system,
        "\u{f40}\u{f72}\u{f0c}\u{f08}\u{f41}",
        own_style(),
    );
    assert!(
        !nonbreaking
            .opportunity_views()
            .any(|o| o.kind() == PreparedBreakKind::Allowed
                && o.boundary().view().source_before().get() == 3)
    );
    assert_eq!(
        ids(&finish(
            &nonbreaking,
            select_request(&nonbreaking, start(&nonbreaking), terminal(&nonbreaking))
        )),
        [277, 100, 58, 36, 37]
    );
}

#[test]
fn real_before_after_and_both_indications_keep_local_ltr_glyphs_and_custody() {
    let bytes = indication_font();
    qualify_native(
        &bytes,
        &[(0x5d0, 82), (0x28, 36), (0x628, 35), (0x29, 37), (0x31, 58)],
        *b"arab",
    );
    let mut system = system_with(&bytes);
    let indication = "\u{5d0}(\u{628})1\u{202e}AB\u{202c}";
    let visible = vec![82, 36, 274, 37, 58, 35, 36];
    for mode in [
        HyphenationPlacement::LogicalBefore,
        HyphenationPlacement::LogicalAfter,
        HyphenationPlacement::BothSides,
    ] {
        let mut style = own_style();
        style.hyphens = Hyphens::Manual;
        style.locale = Some("und-Latn".into());
        style.native_hyphenation_convention = Some(
            NativeHyphenationConvention::language_script(
                "und-Latn",
                "Latn",
                Text::from(indication),
                mode,
            )
            .unwrap(),
        );
        let p = prepare(&mut system, "AB\u{ad}CCC", style);
        let offer = p
            .hyphenation_cut_views()
            .find(|cut| cut.boundary().view().source_before().get() == 3)
            .unwrap()
            .opportunity()
            .unwrap();
        let chosen = offer.view().alternatives().next().unwrap().handle();
        assert_eq!(
            chosen.view().text().code_units(),
            indication.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(chosen.view().source_range(), Some(Range::new(2, 3)));
        let mut r = select_request(&p, start(&p), offer.clone());
        r.alternative = Some(chosen.clone());
        let first = finish(&p, r);
        let incoming = first.hyphenation_continuation();
        assert_eq!(
            incoming.is_some(),
            mode != HyphenationPlacement::LogicalBefore
        );
        let mut r = select_request(&p, offer.view().boundary(), terminal(&p));
        r.incoming = incoming.clone();
        let second = finish(&p, r);
        let mut expected_first = vec![35, 36];
        if mode != HyphenationPlacement::LogicalAfter {
            expected_first.extend(&visible);
        }
        let mut expected_second = Vec::new();
        if mode != HyphenationPlacement::LogicalBefore {
            expected_second.extend(&visible);
        }
        expected_second.extend([37, 37, 37]);
        assert_eq!(ids(&first), expected_first, "{mode:?}");
        assert_eq!(ids(&second), expected_second, "{mode:?}");
        assert_eq!(first.feedback().advance, expected_first.len() as f64 * 12.0);
        assert_eq!(
            second.feedback().advance,
            expected_second.len() as f64 * 12.0
        );
        for (line, emitted) in [
            (&first, mode != HyphenationPlacement::LogicalAfter),
            (&second, mode != HyphenationPlacement::LogicalBefore),
        ] {
            // Supplemental exact owner observation after the real public
            // selection/finalization; glyph order and placement are primary.
            assert!(
                line.inner
                    .selected
                    .presentation
                    .analysis
                    .levels
                    .iter()
                    .all(|&level| level == 0)
            );
            if emitted {
                assert!(
                    line.fragment_views()
                        .any(|f| f.owner() == FinalizedFragmentOwner::Alternative(chosen.clone()))
                );
                assert!(
                    line.mapping_views()
                        .any(|m| m.alternative() == Some(chosen.clone()))
                );
            }
            assert!(line.run_views().all(|r| r.face().bytes() == bytes));
        }
        assert!(second.hyphenation_continuation().is_none());
        let layout = place(&p, vec![first, second]);
        assert_eq!(
            layout.source().text().code_units(),
            [65, 66, 173, 67, 67, 67]
        );
        for run in layout.run_views() {
            assert_eq!(run.orientation(), LineOrientation::Vertical);
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
        }
        for line in 0..2 {
            let points = layout
                .run_views()
                .filter(|r| r.line() == line)
                .flat_map(|r| {
                    r.glyphs()
                        .map(|g| r.origin().y + g.pen() + f64::from(g.y()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            assert!(
                points.windows(2).all(|p| p[0] < p[1]),
                "physical TTB order in {mode:?}"
            );
        }
    }
}
