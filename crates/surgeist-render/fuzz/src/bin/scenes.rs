#![forbid(unsafe_code)]
#![cfg_attr(feature = "coverage", no_main)]

#[cfg(not(feature = "coverage"))]
fn main() -> surgeist_fuzz::Result<()> {
    surgeist_fuzz::worker::serve(&surgeist_render_fuzz::SceneTarget::new())
}

#[cfg(feature = "coverage")]
static TARGET: std::sync::OnceLock<surgeist_render_fuzz::SceneTarget> = std::sync::OnceLock::new();
#[cfg(feature = "coverage")]
std::thread_local! {
    static DRIVER: std::cell::RefCell<surgeist_fuzz::engine::CoverageDriver<'static, surgeist_render_fuzz::SceneTarget>> =
        std::cell::RefCell::new(surgeist_fuzz::engine::CoverageDriver::for_campaign(
            TARGET.get_or_init(surgeist_render_fuzz::SceneTarget::new), None)
            .expect("Render campaign configuration"));
}
#[cfg(feature = "coverage")]
libfuzzer_sys::fuzz_target!(|input: &[u8]| {
    DRIVER.with(|driver| {
        driver
            .borrow_mut()
            .libfuzzer_entry_framed(input, &mut std::io::stdout().lock(), 1_048_576)
            .expect("Render coverage evaluation");
    });
});
