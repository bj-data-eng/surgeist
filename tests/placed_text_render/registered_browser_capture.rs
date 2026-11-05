#![cfg(not(test))]
#![forbid(unsafe_code)]

//! Headless Native Render qualification of a registered Layout browser case.

use std::{collections::BTreeSet, path::Path};

use support::capture_authority::SpecializedProfile;
use surgeist::render;

#[path = "../../crates/surgeist-layout/tests/layout/browser_parity/atomic_paragraph_metrics.rs"]
pub mod atomic_paragraph_metrics;
#[path = "../../crates/surgeist-layout/tests/layout/browser_parity/fixture_variants.rs"]
pub mod fixture_variants;
#[path = "../../crates/surgeist-layout/tests/layout/browser_parity/grid_alignment_reference.rs"]
pub mod grid_alignment_reference;
#[path = "../../crates/surgeist-layout/tests/bin/surgeist-layout-generate/measurement/horizontal_br_font_band_profile.rs"]
pub mod horizontal_br_font_band_profile;
#[path = "../../crates/surgeist-layout/tests/bin/surgeist-layout-generate/measurement/original_subgrid_body_profile.rs"]
pub mod original_subgrid_body_profile;
#[path = "../../crates/surgeist-layout/tests/layout/browser_parity/pinned_font.rs"]
pub mod pinned_font;
#[path = "../../crates/surgeist-layout/tests/layout/browser_parity/portable_qualification.rs"]
pub mod portable_qualification;
#[path = "../../crates/surgeist-layout/tests/bin/surgeist-layout-generate/measurement/prepared_atomic_percentage_profile.rs"]
pub mod prepared_atomic_percentage_profile;
#[path = "../../crates/surgeist-layout/tests/layout/browser_parity/support.rs"]
pub mod support;

fn main() {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR")).join("crates/surgeist-layout");
    let registry = portable_qualification::admit_registered(&owner).unwrap();
    let root = SpecializedProfile::PreparedAtomicPercentage.artifact_root();
    let output =
        format!("{root}/xml/block/fri06_atomic_inline_percentage_block_size__border_box_ltr.xml");
    let mut observed = 0;
    let result = registry
        .execute_selected_with_placement(&BTreeSet::from([output]), &mut |placed| {
            observed += 1;
            assert_eq!(placed.container(), 0);
            assert_eq!(placed.nodes(), [1, 2, 3]);
            verify_render(placed.layout());
            Ok(())
        })
        .unwrap();
    assert_eq!(
        observed, 1,
        "the selected case must lend one actual placement"
    );
    assert_eq!(result.cases.len(), 1);
    assert!(
        result.selected_cases_passed(),
        "{:?}",
        result.cases[0].qualified,
    );
    assert!(result.cases[0].raw.as_ref().unwrap().is_empty());
    println!("Qualified one registered browser capture through Native Render");
}

fn verify_render(placed: &surgeist_text::Layout) {
    let units: Vec<_> = "\n  basis  definite\n".encode_utf16().collect();
    assert_eq!(placed.source().text().code_units(), units);
    let prepared = render::prepare_text_layout(
        placed,
        render::Transform::translation(7.0, 9.0).unwrap(),
        render::TextLayoutLimits::new(
            render::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
            8,
            32,
            0,
        ),
    )
    .unwrap();
    assert!(std::ptr::eq(prepared.layout(), placed));

    // The Ahem cmap maps the two source runs to these glyph IDs. Its 1000
    // units per em and 1000-unit advances give each glyph 16px. All letters
    // here have the same outline (0..1000, -200..800), and SPACE has no ink.
    // The atomic bottom sets the first baseline at 40px. The 20px line
    // strut sets the next baseline at 60px before the 9px scene translation.
    let expected = [
        (&[68, 67, 85, 75, 85, 3][..], 80.0, 49.0),
        (&[70, 71, 72, 75, 80, 75, 86, 71][..], 128.0, 69.0),
    ];
    let top = f64::from(-12.8_f32);
    let bottom = f64::from(3.2_f32);
    let color = render::Color::try_rgba(0.0, 128.0 / 255.0, 0.0, 1.0).unwrap();
    let mut draws = prepared.glyph_draws();
    let mut source_runs = placed
        .run_views()
        .filter(|run| run.glyphs().next().is_some());
    for (ids, ink_width, baseline) in expected {
        let draw = draws.next().expect("admitted glyph draw");
        let source = source_runs.next().expect("actual placed source run");
        let run = draw.run();
        let face = run.font().admitted_face();
        assert_eq!(
            face.resource().identity(),
            source.face().admitted_face().resource().identity()
        );
        assert_eq!(face.resource().bytes(), pinned_font::BYTES);
        assert_eq!(face.index(), 0);
        assert_eq!(run.instance().size(), 16.0);
        assert!(run.instance().coordinates().is_empty());
        assert_eq!(run.instance().synthesis(), render::GlyphSynthesis::none());
        let glyphs: Vec<_> = ids
            .iter()
            .zip(0_u32..)
            .map(|(&id, column)| {
                render::TextGlyph::try_new(id, f64::from(column) * 16.0, 0.0, 16.0).unwrap()
            })
            .collect();
        assert_eq!(run.glyphs(), glyphs);
        assert_eq!(
            draw.transform(),
            render::Transform::translation(7.0, baseline).unwrap()
        );
        assert_eq!(draw.paint().fill(), &color.into());
        assert_eq!(
            run.bounds().ink_rect(),
            Some(render::Rect::try_new(0.0, top, ink_width, bottom - top).unwrap())
        );
    }
    assert!(draws.next().is_none());
    assert!(source_runs.next().is_none());
    assert_eq!(prepared.scene().len(), expected.len());
}
