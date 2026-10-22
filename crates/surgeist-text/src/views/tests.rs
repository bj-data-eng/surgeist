use super::*;
use crate::{
    Alignment, Decoration, Font, FontCatalogueMode, Id, Indent, Options, Slant, Source, Style,
    System, SystemOptions, Weight,
};

const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
use crate::test_fonts::{replace_tables, table, variable_ahem};

fn provider_face<'a>(
    data: &'a parley::FontData,
    catalogue: &'a crate::FontCatalogueIdentity,
) -> FontFace<'a> {
    FontFace::new(&data.data, data.index, catalogue)
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
        let instance = view.effective_instance().unwrap();
        assert_eq!(instance.font_size(), 20.0);
        assert_eq!(instance.axis_count(), 0);
        assert_eq!(instance.normalized_coords().len(), 0);
        assert!(instance.face().is_same_face(&view.face()));
        let glyphs: Vec<_> = view.glyphs().collect();
        assert_eq!(glyphs.len(), owned.glyphs().len());
        for (borrowed, glyph) in glyphs.iter().zip(owned.glyphs()) {
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
    assert_eq!(views[0].glyphs().next().unwrap().x(), 35.0);
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
    assert!(
        run.run.run().normalized_coords().is_empty(),
        "actual provider clears all-zero coordinates"
    );
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
    assert_eq!(run.run.run().normalized_coords(), [8192]);
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
fn actual_unresolved_faux_bold_and_skew_refuse_separately() {
    for (font, expected) in [
        (
            Font::new().family("Ahem").weight(Weight::Bold),
            FontSynthesisKind::Embolden,
        ),
        (
            Font::new().family("Ahem").style(Slant::Italic),
            FontSynthesisKind::Skew,
        ),
    ] {
        let mut system = system(AHEM.to_vec());
        let layout = system
            .layout(
                Source::new("X"),
                Style { font, ..style() },
                Options::default(),
            )
            .unwrap();
        let run = layout.run_views().next().unwrap();
        assert!(run.run.run().synthesis().any());
        assert_eq!(
            run.effective_instance().unwrap_err().detail(),
            Some(&ErrorDetail::UnresolvedFontSynthesis { kind: expected })
        );
    }
}

#[test]
fn actual_allocation_identity_is_not_a_font_id_or_content_comparison() {
    let system = system(AHEM.to_vec());
    let catalogue = system.font_catalogue_identity();
    let blob: linebender_resource_handle::Blob<u8> = AHEM.to_vec().into();
    let (allocation, id) = blob.clone().into_raw_parts();
    let same = parley::FontData {
        data: linebender_resource_handle::Blob::from_raw_parts(allocation, id + 1),
        index: 0,
    };
    let original = parley::FontData {
        data: blob,
        index: 0,
    };
    let (different_allocation, _) =
        linebender_resource_handle::Blob::from(AHEM.to_vec()).into_raw_parts();
    let different = parley::FontData {
        data: linebender_resource_handle::Blob::from_raw_parts(different_allocation, id),
        index: 0,
    };
    assert!(provider_face(&original, catalogue).is_same_face(&provider_face(&same, catalogue)));
    assert!(
        !provider_face(&original, catalogue).is_same_face(&provider_face(&different, catalogue))
    );
    let other_index = parley::FontData {
        data: original.data.clone(),
        index: 1,
    };
    assert!(
        !provider_face(&original, catalogue).is_same_face(&provider_face(&other_index, catalogue))
    );
}

#[test]
fn present_malformed_fvar_never_becomes_static_and_coordinate_validation_is_exact() {
    let system = system(AHEM.to_vec());
    let catalogue = system.font_catalogue_identity();
    let bytes = variable_ahem();
    let valid = parley::FontData {
        data: bytes.clone().into(),
        index: 0,
    };
    let face = provider_face(&valid, catalogue);
    let mut two_axes = table(&bytes, *b"fvar").to_vec();
    let mut second_axis = two_axes[16..36].to_vec();
    second_axis[..4].copy_from_slice(b"wdth");
    two_axes.extend(second_axis);
    two_axes[8..10].copy_from_slice(&2u16.to_be_bytes());
    two_axes[14..16].copy_from_slice(&12u16.to_be_bytes());
    let two_axis_font = parley::FontData {
        data: replace_tables(&bytes, vec![(*b"fvar", two_axes)]).into(),
        index: 0,
    };
    assert_eq!(
        EffectiveFontInstance::qualify(provider_face(&two_axis_font, catalogue), 20.0, &[0])
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
        let bad = parley::FontData {
            data: replace_tables(&bytes, vec![(*b"fvar", corrupted)]).into(),
            index: 0,
        };
        assert_eq!(
            EffectiveFontInstance::qualify(provider_face(&bad, catalogue), 20.0, &[])
                .unwrap_err()
                .detail(),
            Some(&ErrorDetail::MalformedFontVariations)
        );
    }
    let static_font = parley::FontData {
        data: AHEM.to_vec().into(),
        index: 0,
    };
    assert_eq!(
        EffectiveFontInstance::qualify(provider_face(&static_font, catalogue), 20.0, &[0])
            .unwrap_err()
            .detail(),
        Some(&ErrorDetail::FontCoordinateCount {
            expected: 0,
            actual: 1
        })
    );
}
