//! Finite real-browser qualification of Layout's source-qualified root capture.
//! Run explicitly with the cached browser.

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    surgeist_layout_parity::run_qualification()
}
