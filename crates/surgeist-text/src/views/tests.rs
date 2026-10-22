use super::*;
use crate::{
    Alignment, Decoration, ErrorDetail, Font, FontCatalogueMode, Id, Indent, Options, Slant,
    Source, Style, System, SystemOptions, Weight,
};

const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
use crate::test_fonts::{replace_tables, table, variable_ahem};

fn provider_face<'a>(data: &'a crate::FontData) -> FontFace<'a> {
    data.face()
}

fn owned_font(
    data: linebender_resource_handle::Blob<u8>,
    index: u32,
    catalogue: &crate::FontCatalogueIdentity,
) -> crate::FontData {
    crate::FontData::from_blob(data, index, catalogue.clone())
}

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
        size: 20.0,
        ..Style::default()
    }
}

#[test]
fn actual_borrowed_runs_match_registered_glyphs_placement_and_decorations() {
    let mut system = system(AHEM.to_vec());
    let style = Style {
        underline: Decoration::solid(),
        strikethrough: Decoration::solid(),
        ..style()
    };
    let layout = system
        .layout(
            Source::identified("AX\nX", Id::from_u64(7), 1),
            style,
            Options {
                width: Some(100.0),
                alignment: Alignment::Center,
                indent: Indent {
                    amount: 10.0,
                    ..Indent::default()
                },
                quantize: false,
                ..Options::default()
            },
        )
        .unwrap();
    let owned = layout.glyph_runs();
    let views: Vec<_> = layout.run_views().collect();
    assert_eq!(views.len(), owned.len());
    assert!(views.len() >= 2);
    assert_eq!(views[0].face().bytes(), AHEM);
    assert_eq!(views[0].face().index(), 0);
    for (view, owned) in views.iter().zip(&owned) {
        assert_eq!(view.line(), owned.line());
        assert_eq!(view.range(), owned.range());
        assert_eq!(view.brush(), owned.brush());
        assert_eq!(view.font_size(), 20.0);
        assert_eq!(view.origin(), owned.origin());
        let instance = view.effective_instance().unwrap();
        assert_eq!(instance.font_size(), 20.0);
        assert_eq!(instance.axis_count(), 0);
        assert_eq!(instance.normalized_coords().len(), 0);
        assert!(instance.face().is_same_face(&view.face()));
        let glyphs: Vec<_> = view.glyphs().collect();
        assert_eq!(glyphs.len(), owned.glyphs().len());
        for (borrowed, glyph) in glyphs.iter().zip(owned.glyphs()) {
            assert_eq!(borrowed.pen(), glyph.pen());
            assert_eq!(
                (
                    borrowed.id(),
                    borrowed.x(),
                    borrowed.y(),
                    borrowed.advance()
                ),
                (glyph.id(), glyph.x(), glyph.y(), glyph.advance())
            );
        }
    }
    assert_eq!(
        views[0]
            .glyphs()
            .map(|g| g.id())
            .take(2)
            .collect::<Vec<_>>(),
        [35, 58]
    );
    assert_eq!(views[0].glyphs().next().unwrap().x(), 0.0);
    assert_eq!(views[0].origin().x, 35.0);
    assert_eq!(
        views[0]
            .glyphs()
            .take(2)
            .map(|glyph| (glyph.pen(), glyph.x(), glyph.y(), glyph.advance()))
            .collect::<Vec<_>>(),
        [(0.0, 0.0, 0.0, 20.0), (20.0, 0.0, 0.0, 20.0)]
    );
    assert!(views[0].face().is_same_face(&views.last().unwrap().face()));
    assert_eq!(
        layout.decoration_views().collect::<Vec<_>>(),
        layout.decorations()
    );
    assert_eq!(layout.decoration_views().count(), views.len() * 2);
    drop(system);
    assert_eq!(views[0].face().bytes(), AHEM);
}

#[test]
fn actual_variable_default_expands_complete_axes_and_nondefault_bits_stay_exact() {
    let mut system = system(variable_ahem());
    let default = system
        .layout(
            Source::new("X"),
            Style {
                font: Font::new().family("Ahem").variation("\"wght\" 0"),
                ..style()
            },
            Options::default(),
        )
        .unwrap();
    let run = default.run_views().next().unwrap();
    assert!(run.shaped.coords.iter().all(|bits| *bits == 0));
    let instance = run.effective_instance().unwrap();
    assert_eq!(instance.axis_count(), 1);
    assert_eq!(instance.normalized_coords().collect::<Vec<_>>(), [0]);
    let nondefault = system
        .layout(
            Source::new("X"),
            Style {
                font: Font::new().family("Ahem").variation("\"wght\" 0.5"),
                ..style()
            },
            Options::default(),
        )
        .unwrap();
    let run = nondefault.run_views().next().unwrap();
    assert_eq!(run.shaped.coords, [8192]);
    let instance = run.effective_instance().unwrap();
    assert_eq!(instance.normalized_coords().collect::<Vec<_>>(), [8192]);
    assert_eq!(instance.axis_count(), 1);
    assert!(
        instance
            .face()
            .is_same_face(&default.run_views().next().unwrap().face())
    );
}

#[test]
fn actual_selected_bold_and_skew_are_executable_without_advance_or_metric_changes() {
    let mut system = system(AHEM.to_vec());
    let plain = system
        .layout(
            Source::new("AX"),
            Style {
                size: 360.0,
                ..style()
            },
            Options {
                quantize: false,
                ..Options::default()
            },
        )
        .unwrap();
    for (font, bold, skew) in [
        (Font::new().family("Ahem").weight(Weight::Bold), true, false),
        (Font::new().family("Ahem").style(Slant::Italic), false, true),
        (
            Font::new()
                .family("Ahem")
                .weight(Weight::Bold)
                .style(Slant::Italic),
            true,
            true,
        ),
    ] {
        let selected = system
            .layout(
                Source::new("AX"),
                Style {
                    font,
                    size: 360.0,
                    ..style()
                },
                Options {
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        assert_eq!(selected.metrics().size(), plain.metrics().size());
        let view = selected.run_views().next().unwrap();
        let instance = view.effective_instance().unwrap();
        assert_eq!(instance.font_size(), 360.0);
        assert_eq!(instance.embolden(), if bold { 10.0 } else { 0.0 });
        if skew {
            // The selected static Ahem face receives Fontique's documented 14-degree
            // faux italic request. tan(14 degrees) is approximately0.249328003.
            assert!((instance.x_shear() - 0.249_328).abs() < 0.000_000_1);
        } else {
            assert_eq!(instance.x_shear(), 0.0);
        }
        assert_eq!(
            view.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>(),
            [(35, 360.0), (58, 360.0)]
        );
        assert_eq!(instance.normalized_coords().len(), 0);
    }
}

#[test]
fn actual_allocation_identity_is_not_a_font_id_or_content_comparison() {
    let system = system(AHEM.to_vec());
    let catalogue = system.font_catalogue_identity();
    let blob: linebender_resource_handle::Blob<u8> = AHEM.to_vec().into();
    let (allocation, id) = blob.clone().into_raw_parts();
    let same = owned_font(
        linebender_resource_handle::Blob::from_raw_parts(allocation, id + 1),
        0,
        catalogue,
    );
    let original = owned_font(blob.clone(), 0, catalogue);
    let (different_allocation, _) =
        linebender_resource_handle::Blob::from(AHEM.to_vec()).into_raw_parts();
    let different = owned_font(
        linebender_resource_handle::Blob::from_raw_parts(different_allocation, id),
        0,
        catalogue,
    );
    assert!(provider_face(&original).is_same_face(&provider_face(&same)));
    assert!(!provider_face(&original).is_same_face(&provider_face(&different)));
    let other_index = owned_font(blob.clone(), 1, catalogue);
    assert!(!provider_face(&original).is_same_face(&provider_face(&other_index)));
}

#[test]
fn present_malformed_fvar_never_becomes_static_and_coordinate_validation_is_exact() {
    let system = system(AHEM.to_vec());
    let catalogue = system.font_catalogue_identity();
    let bytes = variable_ahem();
    let valid = owned_font(bytes.clone().into(), 0, catalogue);
    let face = provider_face(&valid);
    let mut two_axes = table(&bytes, *b"fvar").to_vec();
    let mut second_axis = two_axes[16..36].to_vec();
    second_axis[..4].copy_from_slice(b"wdth");
    two_axes.extend(second_axis);
    two_axes[8..10].copy_from_slice(&2u16.to_be_bytes());
    two_axes[14..16].copy_from_slice(&12u16.to_be_bytes());
    let two_axis_font = owned_font(
        replace_tables(&bytes, vec![(*b"fvar", two_axes)]).into(),
        0,
        catalogue,
    );
    assert_eq!(
        EffectiveFontInstance::qualify(provider_face(&two_axis_font), 20.0, &[0])
            .unwrap_err()
            .detail(),
        Some(&ErrorDetail::FontCoordinateCount {
            expected: 2,
            actual: 1
        })
    );
    for coordinates in [&[0i16, 0][..], &[16385i16][..]] {
        assert!(EffectiveFontInstance::qualify(face, 20.0, coordinates).is_err());
    }
    for bits in [-16384, 0, 16384] {
        assert_eq!(
            EffectiveFontInstance::qualify(face, 20.0, &[bits])
                .unwrap()
                .normalized_coords()
                .collect::<Vec<_>>(),
            [bits]
        );
    }
    for corrupted in [
        vec![0],
        {
            let mut fvar = table(&bytes, *b"fvar").to_vec();
            fvar[8..10].copy_from_slice(&2u16.to_be_bytes());
            fvar
        },
        {
            let mut fvar = table(&bytes, *b"fvar").to_vec();
            fvar[24..28].copy_from_slice(&131072i32.to_be_bytes());
            fvar
        },
    ] {
        let bad = owned_font(
            replace_tables(&bytes, vec![(*b"fvar", corrupted)]).into(),
            0,
            catalogue,
        );
        assert_eq!(
            EffectiveFontInstance::qualify(provider_face(&bad), 20.0, &[])
                .unwrap_err()
                .detail(),
            Some(&ErrorDetail::MalformedFontVariations)
        );
    }
    let static_font = owned_font(AHEM.to_vec().into(), 0, catalogue);
    assert_eq!(
        EffectiveFontInstance::qualify(provider_face(&static_font), 20.0, &[0])
            .unwrap_err()
            .detail(),
        Some(&ErrorDetail::FontCoordinateCount {
            expected: 0,
            actual: 1
        })
    );
}
