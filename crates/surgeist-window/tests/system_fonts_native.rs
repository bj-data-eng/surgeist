//! Runs on the process main thread to exercise the real native preference host.

#![forbid(unsafe_code)]

#[cfg(target_os = "macos")]
fn main() {
    use surgeist_window::{SystemFontEnvironment, SystemFontRole};

    let environment = SystemFontEnvironment::capture().expect("main-thread native font capture");
    for role in [
        SystemFontRole::Caption,
        SystemFontRole::Icon,
        SystemFontRole::Menu,
        SystemFontRole::MessageBox,
        SystemFontRole::SmallCaption,
        SystemFontRole::StatusBar,
    ] {
        let font = environment.font(role);
        assert!(
            !font.family().trim().is_empty(),
            "{role:?}: family is observed"
        );
        assert!(!font.face().trim().is_empty(), "{role:?}: face is observed");
        assert!(
            font.weight().appkit() <= 15,
            "{role:?}: native AppKit weight range"
        );
        let metrics = font.metrics();
        assert!(metrics.point_size().is_finite() && metrics.point_size() > 0.0);
        assert!(metrics.ascent().is_finite() && metrics.ascent() >= 0.0);
        assert!(metrics.descent().is_finite() && metrics.descent() <= 0.0);
        assert!(metrics.leading().is_finite());
        assert!(metrics.cap_height().is_finite() && metrics.cap_height() >= 0.0);
        assert!(metrics.x_height().is_finite() && metrics.x_height() >= 0.0);
        println!("{role:?}: {font:?}");
    }
    // Independent native defaults verify the declared host size policy. No
    // family names or point sizes are hard-coded from a particular machine.
    use objc2_app_kit::NSFont;
    assert_eq!(
        environment
            .font(SystemFontRole::Caption)
            .metrics()
            .point_size(),
        NSFont::systemFontSize()
    );
    assert_eq!(
        environment
            .font(SystemFontRole::SmallCaption)
            .metrics()
            .point_size(),
        NSFont::smallSystemFontSize()
    );
    assert_eq!(
        environment
            .font(SystemFontRole::StatusBar)
            .metrics()
            .point_size(),
        NSFont::labelFontSize()
    );
}

#[cfg(target_os = "windows")]
fn main() {
    if std::env::var_os("SURGEIST_RUN_NATIVE_FONT_PROBE").as_deref()
        != Some(std::ffi::OsStr::new("1"))
    {
        println!("native Windows font probe skipped; set SURGEIST_RUN_NATIVE_FONT_PROBE=1");
        return;
    }

    use surgeist_window::{SystemFontEnvironment, SystemFontRole};

    let environment = SystemFontEnvironment::capture().expect("native Windows UI font capture");
    for role in [
        SystemFontRole::Caption,
        SystemFontRole::Icon,
        SystemFontRole::Menu,
        SystemFontRole::MessageBox,
        SystemFontRole::SmallCaption,
        SystemFontRole::StatusBar,
    ] {
        println!("{role:?}: {:?}", environment.font(role));
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn main() {
    use surgeist_window::{SystemFontEnvironment, SystemFontError};
    assert_eq!(
        SystemFontEnvironment::capture(),
        Err(SystemFontError::UnsupportedPlatform)
    );
}
