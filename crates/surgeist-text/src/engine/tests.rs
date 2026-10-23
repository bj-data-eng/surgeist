use crate::shaping_test_fonts::{arabic_ahem, ligature_ahem, mark_ahem};
use crate::{
    ErrorCode, ErrorDetail, Font, FontCatalogueMode, Glyph, Id, InlineBox, InlineBoxKind,
    LineHeight, Options, Range, ShapingRefusalReason, Size, Source, Style, System, SystemOptions,
};

const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn system(bytes: Vec<u8>) -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    system
}

fn style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::FontSizeRelative(1.0),
        ..Style::default()
    }
}

fn options() -> Options {
    Options {
        quantize: false,
        ..Options::default()
    }
}

fn glyphs(layout: &crate::Layout) -> Vec<Glyph> {
    layout
        .glyph_runs()
        .iter()
        .flat_map(|run| run.glyphs().iter().copied())
        .collect()
}

#[test]
fn registered_ligature_keeps_literal_glyph_source_extent_and_historical_resource() {
    let bytes = ligature_ahem();
    let mut system = system(bytes.clone());
    let source = Source::identified("AA", Id::from_u64(17), 4);
    let layout = system.layout(source.clone(), style(), options()).unwrap();
    let glyphs = glyphs(&layout);
    assert_eq!(glyphs.len(), 1);
    assert_eq!(glyphs[0].id(), 58);
    assert_eq!(glyphs[0].range(), Range::new(0, 2));
    assert_eq!(glyphs[0].advance(), 10.0);
    assert_eq!(layout.metrics().full_width(), 10.0);
    assert_eq!(layout.source(), &source);
    let catalogue = layout.key().font_catalogue().clone();
    system.refresh_fonts().unwrap();
    assert_ne!(system.font_catalogue_identity(), &catalogue);
    drop(system);
    let view = layout.run_views().next().unwrap();
    assert_eq!(view.face().bytes(), bytes);
    assert_eq!(view.face().catalogue_identity(), &catalogue);
}

#[test]
fn registered_mark_attachment_uses_literal_anchors_and_zero_mark_advance() {
    let mut system = system(mark_ahem());
    let layout = system
        .layout(Source::new("A\u{301}"), style(), options())
        .unwrap();
    let rows = glyphs(&layout);
    assert_eq!(
        rows.iter().map(|glyph| glyph.id()).collect::<Vec<_>>(),
        [35, 100]
    );
    let mark = rows[1];
    assert_eq!(mark.advance(), 0.0);
    // Base anchor(400,600) minus mark anchor(100,200), at size10/UPEM1000.
    assert_eq!(mark.pen() + f64::from(mark.x()), 3.0);
    assert_eq!(mark.y(), -4.0);
    assert_eq!(layout.metrics().full_width(), 10.0);
    assert_eq!(layout.source().text().code_units(), [0x41, 0x301]);
}

#[test]
fn actual_arabic_joining_selects_literal_isolated_initial_medial_and_final_forms() {
    let mut system = system(arabic_ahem());
    let mut style = style();
    style.locale = Some("ar".to_owned());
    for (source, expected) in [
        ("ب", vec![35]),
        ("بب", vec![82, 100]),
        ("ببب", vec![82, 277, 100]),
    ] {
        let layout = system
            .layout(Source::new(source), style.clone(), options())
            .unwrap();
        let rows = glyphs(&layout);
        assert_eq!(
            rows.iter().map(|glyph| glyph.id()).collect::<Vec<_>>(),
            expected,
            "{source}"
        );
        assert_eq!(
            layout.source().text().code_units(),
            source.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            layout.metrics().full_width(),
            source.chars().count() as f64 * 10.0
        );
    }
    let disconnected = system
        .layout(Source::new("ب\u{200c}ب"), style, options())
        .unwrap();
    assert_eq!(
        glyphs(&disconnected)
            .iter()
            .filter(|glyph| glyph.id() == 35)
            .count(),
        2
    );
}

#[test]
fn actual_shaping_refusal_preserves_original_request_prior_layout_and_warm_cache() {
    let mut system = system(AHEM.to_vec());
    let warm_source = Source::new("AX");
    let old = system
        .layout(warm_source.clone(), style(), options())
        .unwrap();
    let before = system.stats();
    let catalogue = system.font_catalogue_identity().clone();
    let source = Source::identified(
        format!("B\u{315}{}", "\u{300}".repeat(32)),
        Id::from_u64(18),
        9,
    );
    let error = system
        .layout(source.clone(), style(), options())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::ShapingRefused);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::ShapingRefused {
            reason: ShapingRefusalReason::ContextLimit
        })
    );
    assert_eq!(system.stats(), before);
    assert_eq!(system.font_catalogue_identity(), &catalogue);
    assert_eq!(source.revision(), 9);
    assert_eq!(source.text().len(), 34);
    assert_eq!(old.source(), &warm_source);
    assert_eq!(
        glyphs(&old)
            .iter()
            .map(|glyph| glyph.id())
            .collect::<Vec<_>>(),
        [35, 58]
    );
    let retry = system.layout(warm_source, style(), options()).unwrap();
    assert_eq!(retry.key(), old.key());
    assert_eq!(system.stats().layout_hits(), before.layout_hits() + 1);
}

#[test]
fn logical_spacing_retains_precision_without_rewriting_provider_advance() {
    let mut system = system(AHEM.to_vec());
    let spacing = 2.0_f64.powi(-40);
    let style = Style {
        size: 1.0,
        letter_spacing: spacing,
        ..style()
    };
    let layout = system.layout(Source::new("AA"), style, options()).unwrap();
    let rows = glyphs(&layout);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].advance(), 1.0_f32);
    assert_eq!(rows[1].advance(), 1.0_f32);
    assert_eq!(rows[1].pen(), 1.0 + spacing);
    assert_eq!(layout.metrics().full_width(), 2.0 + 2.0 * spacing);
}

#[test]
fn empty_source_has_real_font_strut_and_zero_width_while_authored_space_has_advance() {
    let mut system = system(AHEM.to_vec());
    let style = Style {
        size: 20.0,
        ..style()
    };
    let empty = system
        .layout(Source::new(""), style.clone(), options())
        .unwrap();
    assert_eq!(empty.metrics().size(), Size::new(0.0, 20.0));
    assert_eq!(empty.metrics().full_width(), 0.0);
    assert_eq!(empty.lines().len(), 1);
    assert_eq!(empty.lines()[0].range(), Range::new(0, 0));
    assert_eq!(empty.lines()[0].baseline(), 16.0);
    assert!(empty.clusters().is_empty());
    assert!(glyphs(&empty).is_empty());
    assert!(empty.source().text().is_empty());
    let space = system.layout(Source::new(" "), style, options()).unwrap();
    assert_eq!(space.metrics().full_width(), 20.0);
    assert_eq!(space.source().text().code_units(), [0x20]);
    assert_eq!(space.clusters().len(), 1);
    assert_eq!(glyphs(&space)[0].advance(), 20.0);
}

#[test]
fn fontless_actual_boxes_wrap_by_occurrence_without_inventing_font_or_text_rows() {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    let mut source = Source::new("");
    source.inline_box(InlineBox::new(
        Id::from_u64(1),
        InlineBoxKind::InFlow,
        0,
        Size::new(6.0, 10.0),
    ));
    source.inline_box(InlineBox::new(
        Id::from_u64(2),
        InlineBoxKind::InFlow,
        0,
        Size::new(6.0, 20.0),
    ));
    let layout = system
        .layout(
            source.clone(),
            style(),
            Options {
                width: Some(8.0),
                ..options()
            },
        )
        .unwrap();
    assert_eq!(layout.source(), &source);
    assert!(layout.run_views().next().is_none());
    assert!(layout.clusters().is_empty());
    let boxes = layout.inline_boxes();
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[0].id(), Id::from_u64(1));
    assert_eq!(boxes[1].id(), Id::from_u64(2));
    assert_eq!(boxes[0].line(), 0);
    assert_eq!(boxes[1].line(), 1);
    assert_eq!(boxes[0].rect().size, Size::new(6.0, 10.0));
    assert_eq!(boxes[1].rect().size, Size::new(6.0, 20.0));
    assert_eq!(layout.metrics().size(), Size::new(6.0, 30.0));
}
