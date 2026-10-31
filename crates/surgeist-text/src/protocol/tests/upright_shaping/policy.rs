//! Public qualification of the adopted source-context and normal-gap policies.
use super::*;
use icu_properties::{
    CodePointMapData,
    props::{EastAsianWidth, GraphemeClusterBreak, Script},
    script::ScriptWithExtensions,
};

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

fn role_font() -> Vec<u8> {
    // The actual automatic ccmp lookup ligates two mapped bases58 -> 82.
    // IgnoreMarks keeps a real acute100 inside the input context. The separate
    // explicit vkrn lookup increases the first advance height by200, from1200
    // to1400; it also ignores the actual mark. OpenType GPOS Overview defines
    // positive YAdvance as increasing TTB advance height (raw y becomes-1400):
    // https://learn.microsoft.com/en-us/typography/opentype/spec/gpos
    let gsub = ot_layout(
        *b"DFLT",
        vec![(*b"ccmp", vec![0])],
        vec![(4, 8, words(&[1, 18, 1, 8, 1, 4, 82, 2, 58, 1, 1, 58]))],
    );
    let mark = words(&[
        1, 42, 48, 1, 12, 24, 1, 0, 6, 1, 100, 200, 2, 6, 12, 1, 400, 600, 1, 400, 600, 1, 1, 100,
        1, 2, 58, 82,
    ]);
    let pair = words(&[1, 18, 8, 0, 1, 12, 1, 58, 200, 1, 1, 58]);
    let gpos = ot_layout(
        *b"DFLT",
        vec![(*b"mark", vec![0]), (*b"vkrn", vec![1])],
        vec![(4, 0, mark), (2, 8, pair)],
    );
    let gdef = words(&[1, 0, 12, 0, 0, 0, 2, 3, 58, 58, 1, 82, 82, 1, 100, 100, 3]);
    with_native_metrics(&sfnt::replace_tables(
        AHEM,
        vec![
            (
                *b"cmap",
                cmap(&[
                    (0xb7, 58),
                    (0x301, 100),
                    (0x202f, 58),
                    (0xff21, 58),
                    (0x3008, 58),
                    (0xff76, 58),
                ]),
            ),
            (*b"GSUB", gsub),
            (*b"GPOS", gpos),
            (*b"GDEF", gdef),
        ],
    ))
}

fn style(locale: Option<&str>, height: LineHeight) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: height,
        locale: locale.map(str::to_owned),
        ..Style::default()
    }
}

fn analysis(
    source: Source,
    base_style: Style,
    strut: ParagraphStrut,
    tab_reference: Option<crate::ParagraphTabReference>,
) -> AnalyzedSource {
    let len = source.text().len();
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            ParagraphInput {
                source: source.clone(),
                base_style,
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: strut,
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference,
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
                anchor_order: if len == 0 {
                    vec![crate::ParagraphAnchor {
                        item: crate::ParagraphAnchorItem::EmptyOrigin(0),
                        affinity: crate::Affinity::After,
                    }]
                } else {
                    vec![]
                },
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
    analyzed
}

fn prepare(system: &mut System, source: Source, style: Style) -> crate::Result<PreparedText> {
    system.prepare(
        &analysis(source, style, ParagraphStrut::DefaultFont, None),
        PreparationOptions::default().with_orientation(LineOrientation::Vertical),
    )
}

fn finish(prepared: &PreparedText, tabs: TabContext) -> FinalizedLine {
    let mut input = request(prepared);
    input.context.orientation = LineOrientation::Vertical;
    input.context.base_direction = ParagraphBaseDirection::LeftToRight;
    input.context.line_box_direction = LineBoxDirection::LeftToRight;
    input.context.available_inline = None;
    input.context.alignment = crate::Alignment::Start;
    input.context.spacing = SpacingPolicy::None;
    input.context.tabs = tabs;
    prepared
        .select(input)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}

fn role_line(
    system: &mut System,
    source: Source,
    mut style: Style,
    positioning: bool,
) -> FinalizedLine {
    if positioning {
        style.font = style.font.feature(r#""ccmp" 0"#).feature(r#""vkrn" 1"#);
    }
    let prepared = prepare(system, source, style).unwrap();
    finish(&prepared, TabContext::NoTabs)
}

fn role_premises(bytes: &[u8]) {
    let set = ScriptWithExtensions::new().get_script_extensions_val32(0xb7);
    assert!(set.contains(&Script::Latin));
    assert!(set.contains(&Script::Han));
    for intrinsic in [
        Script::Mongolian,
        Script::PhagsPa,
        Script::OldTurkic,
        Script::OldHungarian,
    ] {
        assert!(!set.contains(&intrinsic));
    }
    assert_eq!(
        CodePointMapData::<EastAsianWidth>::new().get32(0xb7),
        EastAsianWidth::Ambiguous
    );
    assert_eq!(
        CodePointMapData::<Script>::new().get32(0x301),
        Script::Inherited
    );
    assert_eq!(
        CodePointMapData::<GraphemeClusterBreak>::new().get32(0x301),
        GraphemeClusterBreak::Extend
    );
    qualify(
        bytes,
        &[
            (0xb7, 58),
            (0x301, 100),
            (0x202f, 58),
            (0xff21, 58),
            (0x3008, 58),
            (0xff76, 58),
        ],
    );
    assert!(
        skrifa::FontRef::new(bytes)
            .unwrap()
            .gpos()
            .unwrap()
            .lookup_list()
            .is_ok()
    );
}

#[test]
fn mixed_supported_roles_execute_fallback_isolation_or_explicit_normal_context() {
    let bytes = role_font();
    role_premises(&bytes);
    let mut system = system_with(&bytes);
    // Revisit no-context after explicit Hani in the same System: neither the
    // neighboring query nor a warm provider plan can decide the source role.
    for (locale, normal) in [
        (None, false),
        (Some("und-Hani"), true),
        (None, false),
        (Some("fr"), false),
        (Some("zh"), false),
        (Some("und-Jpan"), false),
        (Some("und-Mong"), false),
        (Some("und-Latn"), false),
    ] {
        let own = style(locale, LineHeight::Absolute(12.0));
        let line = role_line(&mut system, Source::new("··"), own.clone(), false);
        assert_eq!(
            source_glyphs(&line),
            if normal {
                vec![(Range::new(0, 2), 82)]
            } else {
                vec![(Range::new(0, 1), 58), (Range::new(1, 2), 58)]
            },
            "{locale:?}"
        );
        assert_eq!(line.feedback().advance, if normal { 12.0 } else { 24.0 });
        let positioned = role_line(&mut system, Source::new("··"), own, true);
        assert_eq!(
            source_glyphs(&positioned),
            [(Range::new(0, 1), 58), (Range::new(1, 2), 58)]
        );
        assert_eq!(
            positioned.feedback().advance,
            if normal { 26.0 } else { 24.0 },
            "{locale:?}"
        );
        let cold = role_line(
            &mut system_with(&bytes),
            Source::new("··"),
            style(locale, LineHeight::Absolute(12.0)),
            false,
        );
        assert_eq!(source_glyphs(&cold), source_glyphs(&line));
    }
}

#[test]
fn complete_mark_cell_preserves_attachment_while_governing_context_controls_both_edges() {
    let bytes = role_font();
    role_premises(&bytes);
    let mut system = system_with(&bytes);
    for (base_locale, mark_locale, normal) in [
        (Some("und-Hani"), Some("und-Hani"), true),
        (Some("und-Latn"), Some("und-Latn"), false),
        (Some("und-Hani"), Some("und-Latn"), false),
        (Some("und-Hani"), None, false),
    ] {
        let compatible = base_locale == mark_locale;
        let mut own = style(base_locale, LineHeight::Absolute(12.0));
        own.font = own.font.feature(r#""ccmp" 0"#).feature(r#""vkrn" 1"#);
        let mut mark_style = own.clone();
        mark_style.locale = mark_locale.map(str::to_owned);
        if compatible {
            // This actual formatting difference is glyph-neutral: the mark
            // owns height16 while the base owns12, retaining joint GPOS.
            mark_style.line_height = LineHeight::Absolute(16.0);
        }
        let mut source = Source::new("·\u{301}·");
        source.span(Range::new(1, 2), mark_style);
        let captured = analysis(source.clone(), own, ParagraphStrut::DefaultFont, None);
        assert_eq!(
            captured.graphemes().collect::<Vec<_>>(),
            [Range::new(0, 2), Range::new(2, 3)]
        );
        let prepared = system
            .prepare(
                &captured,
                PreparationOptions::default().with_orientation(LineOrientation::Vertical),
            )
            .unwrap();
        let line = finish(&prepared, TabContext::NoTabs);
        // A glyph's original association is not its editing EGC. Equal
        // locales preserve the shared provider cluster; an actual locale/style
        // intersection retains its separate base and mark source components.
        // The editing EGC stays whole in every case. Exact shared attachment
        // is qualified below only for the existing compatible shaping profile.
        let (base_range, mark_range) = if compatible {
            (Range::new(0, 2), Range::new(0, 2))
        } else {
            (Range::new(0, 1), Range::new(1, 2))
        };
        assert_eq!(
            source_glyphs(&line),
            [(base_range, 58), (mark_range, 100), (Range::new(2, 3), 58)],
            "{base_locale:?}/{mark_locale:?}"
        );
        assert_eq!(
            line.feedback().advance,
            if normal { 26.0 } else { 24.0 },
            "{base_locale:?}/{mark_locale:?}"
        );
        assert_eq!(
            line.feedback().line_height,
            if compatible { 16.0 } else { 12.0 }
        );
        let layout = place_upright(prepared, line);
        assert_eq!(layout.source(), &source);
        let mut points = Vec::new();
        for run in layout.run_views() {
            assert_eq!(run.face().bytes(), bytes);
            for glyph in run.glyphs() {
                points.push((
                    glyph.id(),
                    Point::new(
                        run.origin().x + f64::from(glyph.x()),
                        run.origin().y + glyph.pen() + f64::from(glyph.y()),
                    ),
                ));
            }
        }
        if compatible {
            let base = points.iter().find(|(id, _)| *id == 58).unwrap().1;
            let mark = points.iter().find(|(id, _)| *id == 100).unwrap().1;
            assert_eq!((mark.x - base.x, mark.y - base.y), (3.0, -4.0));
        }
    }
}

#[test]
fn actual_width_precedence_controls_shaping_and_intrinsic_gate_does_not_fallback() {
    let bytes = role_font();
    role_premises(&bytes);
    let mut system = system_with(&bytes);
    for (cp, width, normal) in [
        (0xff21, EastAsianWidth::Fullwidth, true),
        (0x3008, EastAsianWidth::Wide, true),
        (0xff76, EastAsianWidth::Halfwidth, false),
    ] {
        assert_eq!(CodePointMapData::<EastAsianWidth>::new().get32(cp), width);
        let text = char::from_u32(cp).unwrap().to_string().repeat(2);
        let line = role_line(
            &mut system,
            Source::new(text),
            style(Some("und-Mong"), LineHeight::Absolute(12.0)),
            false,
        );
        assert_eq!(
            source_glyphs(&line),
            if normal {
                vec![(Range::new(0, 2), 82)]
            } else {
                vec![(Range::new(0, 1), 58), (Range::new(1, 2), 58)]
            }
        );
    }
    let set = ScriptWithExtensions::new().get_script_extensions_val32(0x202f);
    assert!(set.contains(&Script::Latin));
    assert!(set.contains(&Script::Mongolian));
    assert!(set.contains(&Script::PhagsPa));
    let error = prepare(
        &mut system,
        Source::new("\u{202f}"),
        style(None, LineHeight::Absolute(12.0)),
    )
    .unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::UprightScript {
            reason: crate::UprightScriptFailure::UnresolvedContext { .. },
            ..
        })
    ));
}

fn metric_font(gap: i16, variable: bool) -> Vec<u8> {
    let native = vertical::native();
    let mut vhea = sfnt::table(&native, *b"vhea").to_vec();
    vhea[..4].copy_from_slice(&0x0001_1000_u32.to_be_bytes());
    for (at, value) in [(4, 500_i16), (6, -500), (8, gap)] {
        vhea[at..at + 2].copy_from_slice(&value.to_be_bytes());
    }
    let original = if variable {
        sfnt::variable_ahem()
    } else {
        AHEM.to_vec()
    };
    sfnt::replace_tables(
        &original,
        vec![
            (*b"vhea", vhea),
            (*b"vmtx", sfnt::table(&native, *b"vmtx").to_vec()),
            (
                *b"BASE",
                base::base(
                    0,
                    None,
                    Some(base::single_axis(&[*b"ideo", *b"idtp"], &[0, 1000])),
                    None,
                ),
            ),
        ],
    )
}

fn metric_premises(bytes: &[u8], gap: i16) {
    let font = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    let vhea = font.vhea().unwrap();
    assert_eq!(vhea.version().to_major_minor(), (1, 1));
    assert_eq!(
        (
            vhea.ascender().to_i16(),
            vhea.descender().to_i16(),
            vhea.line_gap().to_i16()
        ),
        (500, -500, gap)
    );
    let axis = font.base().unwrap().vert_axis().unwrap().unwrap();
    let scripts = axis.base_script_list().unwrap();
    let values = scripts.base_script_records()[0]
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
        [0, 1000]
    );
    assert_eq!(font.charmap().map('X'), Some(GlyphId::new(58)));
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

fn actual_glyph_source(error: &crate::Error) -> &crate::Error {
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::VerticalMetricSupplier { glyph_id: 58, .. })
    ));
    let original = error
        .source
        .as_deref()
        .unwrap()
        .downcast_ref::<crate::Error>()
        .unwrap();
    assert_eq!(error.code, original.code);
    original
}

#[test]
fn public_metrics_relative_uses_signed_native_gap_without_moving_cell_origins() {
    for (gap, expected) in [(200, 14.4), (-200, 9.6)] {
        let bytes = metric_font(gap, false);
        metric_premises(&bytes, gap);
        let mut system = system_with(&bytes);
        for text in ["X", ""] {
            let prepared = prepare(
                &mut system,
                Source::new(text),
                style(None, LineHeight::MetricsRelative(1.2)),
            )
            .unwrap();
            let line = finish(&prepared, TabContext::NoTabs);
            close(line.feedback().line_height, expected);
            close(line.feedback().ascent, expected / 2.0);
            close(line.feedback().descent, expected / 2.0);
            assert_eq!(
                line.feedback().advance,
                if text.is_empty() { 0.0 } else { 12.0 }
            );
            let layout = place_upright(prepared, line);
            if !text.is_empty() {
                let run = layout.run_views().next().unwrap();
                let glyph = run.glyphs().next().unwrap();
                assert_eq!((glyph.x(), glyph.y(), glyph.advance()), (-5.0, 9.0, 12.0));
                assert_eq!(run.origin(), Point::new(100.0, 20.0));
            }
        }
    }
    let bytes = metric_font(-1001, false);
    metric_premises(&bytes, -1001);
    let tiny = f64::from_bits(1);
    assert!(tiny > 0.0);
    let error = prepare(
        &mut system_with(&bytes),
        Source::new(""),
        style(None, LineHeight::MetricsRelative(tiny)),
    )
    .unwrap_err();
    assert!(
        matches!(error.detail(),Some(crate::ErrorDetail::InvalidTextNumeric { field: "upright normal extent", phase: crate::TextNumericPhase::Preparation, category: crate::TextNumericError::OutOfRange, value }) if *value == -1.0)
    );
}

#[test]
fn glyph_free_default_strut_uses_actual_variable_instance_gap_and_refuses_actual_glyphs() {
    let mut mvar = words(&[1, 0, 0, 8, 1, 20]);
    mvar.extend(*b"vlgp");
    mvar.extend(words(&[0, 0]));
    mvar.extend(base::one_axis_store(100));
    let bytes = sfnt::replace_tables(&metric_font(200, true), vec![(*b"MVAR", mvar)]);
    metric_premises(&bytes, 200);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.fvar().unwrap().axis_count(), 1);
    assert_eq!(font.mvar().unwrap().value_record_count(), 1);
    let mut system = system_with(&bytes);
    for (coordinate, bits, expected) in [(0, 0, 14.4), (1, 16384, 15.6), (0, 0, 14.4)] {
        let mut own = style(None, LineHeight::MetricsRelative(1.2));
        own.font = own.font.variation(format!(r#""wght" {coordinate}"#));
        let prepared = prepare(&mut system, Source::new(""), own.clone()).unwrap();
        let line = finish(&prepared, TabContext::NoTabs);
        close(line.feedback().line_height, expected);
        assert_eq!(line.feedback().advance, 0.0);
        let runs = line.run_views().collect::<Vec<_>>();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].glyphs().count(), 0);
        assert_eq!(runs[0].face().bytes(), bytes);
        assert_eq!(
            runs[0]
                .effective_instance()
                .unwrap()
                .normalized_coords()
                .collect::<Vec<_>>(),
            [bits]
        );
        let error = prepare(&mut system, Source::new("X"), own).unwrap_err();
        assert!(matches!(
            actual_glyph_source(&error).detail(),
            Some(crate::ErrorDetail::UnsupportedVerticalGlyphVariation {
                glyph: 58,
                axis_count: 1
            })
        ));
    }
}

#[test]
fn explicit_zero_atom_heights_do_not_demand_gap_but_actual_glyph_tables_remain_strict() {
    let good = metric_font(200, false);
    metric_premises(&good, 200);
    for (tag, table, reason) in [
        (
            *b"vhea",
            crate::FontMetricTable::Vhea,
            crate::FontMetricTableError::Malformed,
        ),
        (
            *b"MVAR",
            crate::FontMetricTable::Mvar,
            crate::FontMetricTableError::VariationWithoutAxes,
        ),
    ] {
        let bytes = sfnt::replace_tables(&good, vec![(tag, vec![0])]);
        let mut system = system_with(&bytes);
        for (height, expected) in [
            (LineHeight::Absolute(12.0), 12.0),
            (LineHeight::FontSizeRelative(1.5), 15.0),
        ] {
            let prepared = prepare(&mut system, Source::new(""), style(None, height)).unwrap();
            let line = finish(&prepared, TabContext::NoTabs);
            assert_eq!(line.feedback().line_height, expected);
            assert_eq!(
                line.run_views().map(|r| r.glyphs().count()).sum::<usize>(),
                0
            );
        }
        let error = prepare(
            &mut system,
            Source::new(""),
            style(None, LineHeight::MetricsRelative(1.2)),
        )
        .unwrap_err();
        assert_eq!(
            error.detail(),
            Some(&crate::ErrorDetail::FontMetricTable { table, reason })
        );
        if tag == *b"vhea" {
            let error = prepare(
                &mut system,
                Source::new("X"),
                style(None, LineHeight::Absolute(12.0)),
            )
            .unwrap_err();
            assert_eq!(
                actual_glyph_source(&error).detail(),
                Some(&crate::ErrorDetail::FontMetricTable { table, reason })
            );
        }
    }
    // An unrelated malformed MVAR is not demanded for qualified vhea1.0 or
    // genuinely absent vhea. BASE still supplies the same1000-wide em box.
    for absent in [false, true] {
        let mut bytes = metric_font(0, false);
        if absent {
            bytes = sfnt::remove_tables(&bytes, &[*b"vhea", *b"vmtx"]);
        } else {
            let mut header = sfnt::table(&bytes, *b"vhea").to_vec();
            header[..4].copy_from_slice(&0x0001_0000_u32.to_be_bytes());
            bytes = sfnt::replace_tables(&bytes, vec![(*b"vhea", header)]);
        }
        bytes = sfnt::replace_tables(&bytes, vec![(*b"MVAR", vec![0])]);
        let prepared = prepare(
            &mut system_with(&bytes),
            Source::new(""),
            style(None, LineHeight::MetricsRelative(1.2)),
        )
        .unwrap();
        close(
            finish(&prepared, TabContext::NoTabs).feedback().line_height,
            12.0,
        );
    }
}

fn renamed(font: &[u8]) -> Vec<u8> {
    let mut name = sfnt::table(font, *b"name").to_vec();
    for at in 0..name.len().saturating_sub(7) {
        if &name[at..at + 8] == b"\0A\0h\0e\0m" {
            name[at + 1] = b'B';
        }
    }
    let bytes = sfnt::replace_tables(font, vec![(*b"name", name)]);
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(
        face.localized_strings(skrifa::string::StringId::FAMILY_NAME)
            .english_or_first()
            .unwrap()
            .to_string(),
        "Bhem"
    );
    bytes
}

#[test]
fn actual_default_body_and_captured_tab_fonts_keep_independent_metrics() {
    let default = sfnt::replace_tables(
        &metric_font(200, false),
        vec![(*b"cmap", cmap(&[(0x20, 3), (0x30, 19)]))],
    );
    let body = renamed(&metric_font(0, false));
    let body = vertical::with_native(&body, 1800, 100, 281);
    let body = sfnt::replace_tables(
        &body,
        vec![
            (*b"cmap", cmap(&[(0x58, 58)])),
            (
                *b"BASE",
                base::base(
                    0,
                    None,
                    Some(base::single_axis(&[*b"ideo", *b"idtp"], &[0, 600])),
                    None,
                ),
            ),
        ],
    );
    let d = skrifa::FontRef::new(&default).unwrap();
    let b = skrifa::FontRef::new(&body).unwrap();
    assert!(d.charmap().map('X').is_none());
    assert_eq!(d.charmap().map(' '), Some(GlyphId::new(3)));
    assert_eq!(d.charmap().map('0'), Some(GlyphId::new(19)));
    assert_eq!(b.charmap().map('X'), Some(GlyphId::new(58)));
    assert_eq!(b.vmtx().unwrap().v_metrics()[58].advance.get(), 1800);
    let mut system = system_with(&default);
    system.register_fonts(body.clone()).unwrap();
    let mut own = style(None, LineHeight::MetricsRelative(1.2));
    own.font = Font::new().family("Ahem").family("Bhem");
    for (strut, expected) in [
        (ParagraphStrut::DefaultFont, 14.4),
        (ParagraphStrut::AllocatedContributions, 7.2),
    ] {
        let p = system
            .prepare(
                &analysis(Source::new("X"), own.clone(), strut, None),
                PreparationOptions::default().with_orientation(LineOrientation::Vertical),
            )
            .unwrap();
        let line = finish(&p, TabContext::NoTabs);
        close(line.feedback().line_height, expected);
        assert_eq!(line.feedback().advance, 18.0);
        assert!(
            line.run_views()
                .filter(|r| r.glyphs().count() > 0)
                .all(|r| r.face().bytes() == body)
        );
    }
    own.tab_size = crate::TabSize::spaces(2.0).unwrap();
    let mut block = style(None, LineHeight::Absolute(12.0));
    block.size = 20.0;
    let reference =
        crate::ParagraphTabReference::new(block, ParagraphBaseDirection::RightToLeft).unwrap();
    let captured = analysis(
        Source::new("X\tX"),
        own,
        ParagraphStrut::DefaultFont,
        Some(reference),
    );
    let p = system
        .prepare(
            &captured,
            PreparationOptions::try_new(1.0, crate::TabPreparation::CssStops)
                .unwrap()
                .with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let line = finish(&p, TabContext::CssStops { origin: Some(0.0) });
    close(line.feedback().line_height, 14.4);
    // Captured Ahem SPACE1200 at size20 gives24; two spaces give interval48.
    // Actual Bhem X18 ends at18, then tab reaches48 and the second X ends66.
    assert_eq!(line.feedback().advance, 66.0);
    assert_eq!(
        source_glyphs(&line),
        [(Range::new(0, 1), 58), (Range::new(2, 3), 58)]
    );
    let layout = place_upright(p, line);
    let mut ys = Vec::new();
    for run in layout.run_views() {
        for glyph in run.glyphs() {
            assert_eq!(run.face().bytes(), body);
            ys.push(run.origin().y + glyph.pen());
        }
    }
    assert_eq!(ys, [20.0, 68.0]);
}
