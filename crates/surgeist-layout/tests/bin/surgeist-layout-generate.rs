#[path = "surgeist-layout-generate/adapter.rs"]
mod adapter;
#[path = "../layout/browser_parity/atomic_paragraph_metrics.rs"]
mod atomic_paragraph_metrics;
#[path = "surgeist-layout-generate/browser_profile.rs"]
mod browser_profile;
#[path = "surgeist-layout-generate/measurement/capture_helper_identity.rs"]
mod capture_helper_identity;
#[path = "surgeist-layout-generate/cli.rs"]
mod cli;
#[path = "surgeist-layout-generate/measurement/empty_line_br_font_band_profile.rs"]
mod empty_line_br_font_band_profile;
#[path = "surgeist-layout-generate/envelope.rs"]
mod envelope;
#[path = "../layout/browser_parity/fixture_variants.rs"]
mod fixture_variants;
#[path = "../layout/browser_parity/grid_alignment_reference.rs"]
mod grid_alignment_reference;
#[path = "surgeist-layout-generate/measurement/horizontal_br_font_band_profile.rs"]
mod horizontal_br_font_band_profile;
#[path = "surgeist-layout-generate/measurement.rs"]
mod measurement;
#[path = "surgeist-layout-generate/measurement/original_block_axes_profile.rs"]
mod original_block_axes_profile;
#[path = "surgeist-layout-generate/measurement/original_flex_axes_profile.rs"]
mod original_flex_axes_profile;
#[path = "surgeist-layout-generate/measurement/original_flex_parent_profile.rs"]
mod original_flex_parent_profile;
#[path = "surgeist-layout-generate/measurement/original_grid_axes_profile.rs"]
mod original_grid_axes_profile;
#[path = "surgeist-layout-generate/measurement/original_subgrid_axes_profile.rs"]
mod original_subgrid_axes_profile;
#[path = "surgeist-layout-generate/measurement/original_subgrid_body_profile.rs"]
mod original_subgrid_body_profile;
#[path = "../layout/browser_parity/pinned_font.rs"]
mod pinned_font;
#[path = "surgeist-layout-generate/measurement/prepared_atomic_percentage_profile.rs"]
mod prepared_atomic_percentage_profile;
#[path = "surgeist-layout-generate/measurement/vertical_br_font_band_profile.rs"]
mod vertical_br_font_band_profile;
#[path = "surgeist-layout-generate/xml.rs"]
mod xml;

#[cfg(test)]
#[path = "surgeist-layout-generate/helper_protocol_tests.rs"]
mod helper_protocol_tests;

fn main() {
    let result = match surgeist_generator::browser::run_supervisor_from_env() {
        Some(result) => result.map_err(|error| error.to_string()),
        None => cli::run_from_env(),
    };
    if let Err(error) = result {
        eprintln!("surgeist-layout-generate: {error}");
        std::process::exit(1);
    }
}
