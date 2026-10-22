//! Successful font admission retains the selected instance and effective size.
use super::*;

fn font_style(font: Font) -> Style {
    Style {
        font,
        size: 360.0,
        ..style()
    }
}

fn assert_ahem_glyphs(output: &Layout) {
    let glyphs: Vec<_> = output
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| (glyph.id(), f64::from(glyph.advance())))
                .collect::<Vec<_>>()
        })
        .collect();
    // The retained fixture defines A and X as glyphs35/58, UPEM1000,
    // with advance1000. At canonical size360 both advances are360.
    assert_eq!(glyphs, [(35, 360.0), (58, 360.0)]);
    assert_eq!(output.source().text().code_units(), [0x0041, 0x0058]);
}

#[test]
fn ordinary_selected_face_has_complete_instance_and_fixture_advances() {
    let mut system = registered_ahem();
    let output = system
        .layout(
            Source::new("AX"),
            font_style(Font::new().family("Ahem")),
            options(),
        )
        .unwrap();
    assert_ahem_glyphs(&output);
    let run = output.run_views().next().unwrap();
    let instance = run.effective_instance().unwrap();
    assert_eq!(instance.font_size(), 360.0);
    assert_eq!(instance.face().bytes(), AHEM);
    assert_eq!(instance.axis_count(), 0);
}

fn assert_selected_synthesis_is_executable(font: Font) {
    let mut system = registered_ahem();
    let plain = system
        .layout(
            Source::new("AX"),
            font_style(Font::new().family("Ahem")),
            options(),
        )
        .unwrap();
    let selected = system
        .layout(Source::new("AX"), font_style(font), options())
        .unwrap();
    // The selected outline operation changes ink, not the Ahem design advances
    // or font line metrics. Original positioned glyph facts remain authentic.
    assert_ahem_glyphs(&selected);
    assert_eq!(selected.metrics().size(), plain.metrics().size());
    let catalogue = system.font_catalogue_identity().clone();
    drop(system);
    let runs: Vec<_> = selected.run_views().collect();
    assert_eq!(runs.len(), 1);
    let instance = runs[0]
        .effective_instance()
        .expect("selected synthesis must resolve to an executable effective instance");
    assert_eq!(instance.font_size(), 360.0);
    assert_eq!(instance.face().bytes(), AHEM);
    assert_eq!(instance.axis_count(), 0);
    assert_eq!(selected.key().font_catalogue(), &catalogue);
}

#[test]
fn selected_faux_bold_retains_a_complete_executable_instance() {
    assert_selected_synthesis_is_executable(Font::new().family("Ahem").weight(Weight::Bold));
}

#[test]
fn selected_faux_italic_retains_a_complete_executable_instance() {
    assert_selected_synthesis_is_executable(Font::new().family("Ahem").style(Slant::Italic));
}

fn assert_effective_size_failure_is_atomic(size: f32, scaled_options: Options) {
    let mut system = registered_ahem();
    let warm = layout(&mut system, "AX");
    let before = system.stats();
    let catalogue = system.font_catalogue_identity().clone();
    let source = Source::new("AX");
    let _error = system
        .layout(source.clone(), Style { size, ..style() }, scaled_options)
        .expect_err("finite authored inputs must not publish a zero or infinite effective size");
    // The shared numeric contract rejects category loss before publishing a
    // layout/cache entry. It does not invent a general allocator guarantee.
    assert_eq!(system.stats(), before);
    assert_eq!(system.font_catalogue_identity(), &catalogue);
    assert_eq!(source.text().code_units(), [0x0041, 0x0058]);
    assert_eq!(layout(&mut system, "AX").key(), warm.key());
}

#[test]
fn positive_effective_size_cannot_round_to_zero() {
    // Each input is positive and finite; their product is below the smallest
    // positive binary32 value. The required canonical font size cannot be0.
    assert_effective_size_failure_is_atomic(
        f32::from_bits(1),
        Options {
            scale: 1e-45,
            ..options()
        },
    );
}

#[test]
fn finite_effective_size_cannot_overflow_binary32() {
    // A finite logical product need not fit the provider's canonical binary32.
    assert_effective_size_failure_is_atomic(
        f32::MAX,
        Options {
            scale: 2.0,
            ..options()
        },
    );
}

#[test]
fn renderable_text_without_a_usable_face_refuses_without_publishing_layout() {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .expect("an empty registered-only catalogue is valid");
    let source = Source::new("AX");
    let before = system.stats();
    let catalogue = system.font_catalogue_identity().clone();
    system
        .layout(
            source.clone(),
            font_style(Font::new().family("Ahem")),
            options(),
        )
        .expect_err("renderable text requires a usable admitted face");
    assert_eq!(system.stats(), before);
    assert_eq!(system.font_catalogue_identity(), &catalogue);
    assert_eq!(source.text().code_units(), [0x0041, 0x0058]);
    system
        .register_fonts(AHEM.to_vec())
        .expect("a refused request does not prevent later real admission");
    let admitted = system
        .layout(source, font_style(Font::new().family("Ahem")), options())
        .expect("the same authored request succeeds with an actual face");
    assert_ahem_glyphs(&admitted);
}

#[test]
fn refused_combining_reorder_does_not_publish_partial_shaping() {
    let mut system = registered_ahem();
    let warm = layout(&mut system, "AX");
    let before = system.stats();
    let catalogue = system.font_catalogue_identity().clone();
    // The maintained provider's normalization contract permits sorting 32
    // marks, but refuses the required reorder for 33. CCC232 precedes CCC230;
    // the provider returns a nonempty partial buffer with ContextLimit.
    let source = Source::new(format!("B\u{0315}{}", "\u{0300}".repeat(32)));
    system
        .layout(source.clone(), style(), options())
        .expect_err("a real provider refusal cannot publish its partial glyph buffer");
    assert_eq!(system.stats(), before);
    assert_eq!(system.font_catalogue_identity(), &catalogue);
    assert_eq!(source.text().len(), 34);
    assert_eq!(layout(&mut system, "AX").key(), warm.key());
}
