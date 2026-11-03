//! Finite real-browser qualification of Layout's source-qualified root capture.
//! Run explicitly with the existing layout-golden-generate feature and cached browser.

#[path = "../tests/bin/surgeist-layout-generate/adapter.rs"]
mod adapter;
#[path = "../tests/bin/surgeist-layout-generate/browser_profile.rs"]
mod browser_profile;
#[path = "../tests/bin/surgeist-layout-generate/envelope.rs"]
mod envelope;
#[path = "../tests/bin/surgeist-layout-generate/measurement.rs"]
mod measurement;
#[path = "../tests/layout/browser_parity/support.rs"]
pub mod parity_support;
#[path = "../tests/layout/browser_parity/pinned_font.rs"]
mod pinned_font;
#[path = "qualify-root-context/support.rs"]
mod support;
#[path = "../tests/bin/surgeist-layout-generate/xml.rs"]
mod xml;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(result) = surgeist_generator::browser::run_supervisor_from_env() {
        return result.map_err(Into::into);
    }
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let fixture = match arguments.as_slice() {
        [] => None,
        [flag, source] if flag == "--fixture" => Some(source.as_str()),
        _ => return Err("usage: qualify-root-context [--fixture <declared-source>]".into()),
    };
    support::run(fixture)
}
