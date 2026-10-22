use super::*;
use crate::{
    Alignment, Decoration, Font, FontCatalogueMode, Id, Indent, Options, Slant, Source, Style,
    System, SystemOptions, Weight,
};

const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
const X: u32 = 58;

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
    assert!(FontFace { data: &original }.is_same_face(&FontFace { data: &same }));
    assert!(!FontFace { data: &original }.is_same_face(&FontFace { data: &different }));
    let other_index = parley::FontData {
        data: original.data.clone(),
        index: 1,
    };
    assert!(!FontFace { data: &original }.is_same_face(&FontFace { data: &other_index }));
}

#[test]
fn present_malformed_fvar_never_becomes_static_and_coordinate_validation_is_exact() {
    let bytes = variable_ahem();
    let valid = parley::FontData {
        data: bytes.clone().into(),
        index: 0,
    };
    let face = FontFace { data: &valid };
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
        EffectiveFontInstance::qualify(
            FontFace {
                data: &two_axis_font
            },
            20.0,
            &[0]
        )
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
            EffectiveFontInstance::qualify(FontFace { data: &bad }, 20.0, &[])
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
        EffectiveFontInstance::qualify(FontFace { data: &static_font }, 20.0, &[0])
            .unwrap_err()
            .detail(),
        Some(&ErrorDetail::FontCoordinateCount {
            expected: 0,
            actual: 1
        })
    );
}

// Repository-authored variable Ahem fixture construction copied from Render's
// tests/prepared_outline.rs. The original licensed Ahem bytes are unchanged;
// see Render tests/fixtures/fonts/ahem/PROVENANCE.md for the actual one-axis
// contour-delta facts. This is real table data consumed by registered shaping,
// not a synthetic coordinate attestation or broad variable-font conformance.
fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let record = (0..usize::from(u16_at(bytes, 4)))
        .map(|i| 12 + i * 16)
        .find(|r| bytes[*r..*r + 4] == tag)
        .unwrap();
    let offset = u32_at(bytes, record + 8) as usize;
    let len = u32_at(bytes, record + 12) as usize;
    &bytes[offset..offset + len]
}
fn replace_tables(bytes: &[u8], replacements: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    let mut tables: Vec<_> = (0..usize::from(u16_at(bytes, 4)))
        .map(|i| {
            let r = 12 + i * 16;
            let tag: [u8; 4] = bytes[r..r + 4].try_into().unwrap();
            (tag, table(bytes, tag).to_vec())
        })
        .collect();
    for (tag, data) in replacements {
        if let Some(t) = tables.iter_mut().find(|t| t.0 == tag) {
            t.1 = data;
        } else {
            tables.push((tag, data));
        }
    }
    tables.sort_by_key(|t| t.0);
    let mut output = vec![0; 12 + tables.len() * 16];
    output[..4].copy_from_slice(&bytes[..4]);
    output[4..6].copy_from_slice(&u16::try_from(tables.len()).unwrap().to_be_bytes());
    let power =
        tables.len().next_power_of_two() / if tables.len().is_power_of_two() { 1 } else { 2 };
    output[6..8].copy_from_slice(&u16::try_from(power * 16).unwrap().to_be_bytes());
    output[8..10].copy_from_slice(&u16::try_from(power.ilog2()).unwrap().to_be_bytes());
    output[10..12].copy_from_slice(
        &u16::try_from((tables.len() - power) * 16)
            .unwrap()
            .to_be_bytes(),
    );
    for (i, (tag, data)) in tables.into_iter().enumerate() {
        while !output.len().is_multiple_of(4) {
            output.push(0);
        }
        let r = 12 + i * 16;
        let offset = u32::try_from(output.len()).unwrap();
        output[r..r + 4].copy_from_slice(&tag);
        output[r + 8..r + 12].copy_from_slice(&offset.to_be_bytes());
        output[r + 12..r + 16].copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        output.extend(data);
    }
    output
}

// Authored derivative of licensed Ahem. One wght axis [0,1], default0. At +1,
// X's two right-edge points move +100 font units, while all phantom points and
// y deltas stay zero. Thus width is 1000 + coordinate*100 at UPEM1000.
fn variable_ahem() -> Vec<u8> {
    variable_ahem_with_tuple(&[
        0, 1, 0, 10, 0, 11, 0xA0, 0, 0x40, 0, 0, 7, 0, 100, 100, 0, 0, 0, 0, 0, 0x87,
    ])
}
fn variable_ahem_with_tuple(tuple: &[u8]) -> Vec<u8> {
    let mut fvar = vec![];
    fvar.extend(0x00010000u32.to_be_bytes());
    for value in [16u16, 2, 1, 20, 0, 8] {
        fvar.extend(value.to_be_bytes());
    }
    fvar.extend(*b"wght");
    for value in [0i32, 0, 65536] {
        fvar.extend(value.to_be_bytes());
    }
    fvar.extend(0u16.to_be_bytes());
    fvar.extend(256u16.to_be_bytes());
    let count = usize::from(u16_at(table(AHEM, *b"maxp"), 4));
    let data_offset = 20 + (count + 1) * 4;
    let mut gvar = vec![];
    gvar.extend(0x00010000u32.to_be_bytes());
    gvar.extend(1u16.to_be_bytes());
    gvar.extend(0u16.to_be_bytes());
    gvar.extend(u32::try_from(data_offset).unwrap().to_be_bytes());
    gvar.extend(u16::try_from(count).unwrap().to_be_bytes());
    gvar.extend(1u16.to_be_bytes());
    gvar.extend(u32::try_from(data_offset).unwrap().to_be_bytes());
    for i in 0..=count {
        gvar.extend(
            (if i <= X as usize {
                0u32
            } else {
                tuple.len() as u32
            })
            .to_be_bytes(),
        );
    }
    gvar.extend_from_slice(tuple);
    replace_tables(AHEM, vec![(*b"fvar", fvar), (*b"gvar", gvar)])
}
