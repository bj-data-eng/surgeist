//! Runs on the process main thread to exercise the real native preference host.

#![forbid(unsafe_code)]

#[cfg(target_os = "macos")]
fn main() {
    use surgeist_window::{SystemFont, SystemFontEnvironment, SystemFontRole};

    let environment = SystemFontEnvironment::capture().expect("main-thread native font capture");
    for role in [
        SystemFontRole::Caption,
        SystemFontRole::Icon,
        SystemFontRole::Menu,
        SystemFontRole::MessageBox,
        SystemFontRole::SmallCaption,
        SystemFontRole::StatusBar,
    ] {
        let SystemFont::AppKit(font) = environment.font(role) else {
            panic!("{role:?}: expected AppKit native font");
        };
        assert!(
            !font.family().trim().is_empty(),
            "{role:?}: family is observed"
        );
        assert!(
            !font.postscript_name().trim().is_empty(),
            "{role:?}: face is observed"
        );
        assert!(
            font.weight().value() <= 15,
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
    let appkit = |role| {
        let SystemFont::AppKit(font) = environment.font(role) else {
            panic!("expected AppKit native font");
        };
        font
    };
    assert_eq!(
        appkit(SystemFontRole::Caption).metrics().point_size(),
        NSFont::systemFontSize()
    );
    assert_eq!(
        appkit(SystemFontRole::SmallCaption).metrics().point_size(),
        NSFont::smallSystemFontSize()
    );
    assert_eq!(
        appkit(SystemFontRole::StatusBar).metrics().point_size(),
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

    use surgeist_window::{SystemFont, SystemFontEnvironment, SystemFontRole};

    let environment = SystemFontEnvironment::capture().expect("native Windows UI font capture");
    use surgeist_typed_winsafe::{UiFontRole, ui_fonts_for_dpi};
    let dpi = std::num::NonZeroU32::new(96).unwrap();
    let native = ui_fonts_for_dpi(dpi).expect("independent native Windows UI font query");
    for (role, native_role) in [
        (SystemFontRole::Caption, UiFontRole::Caption),
        (SystemFontRole::Icon, UiFontRole::Icon),
        (SystemFontRole::Menu, UiFontRole::Menu),
        (SystemFontRole::MessageBox, UiFontRole::MessageBox),
        (SystemFontRole::SmallCaption, UiFontRole::SmallCaption),
        (SystemFontRole::StatusBar, UiFontRole::StatusBar),
    ] {
        let SystemFont::Windows(font) = environment.font(role) else {
            panic!("{role:?}: expected Windows logical font preference");
        };
        let expected = native.font(native_role);
        assert_eq!(
            font.reference_dpi(),
            dpi,
            "{role:?}: explicit reference DPI"
        );
        assert_eq!(font.face_name(), expected.face_name(), "{role:?}: typeface");
        assert_eq!(
            font.face_name_utf16(),
            expected.face_name_utf16(),
            "{role:?}: exact native name"
        );
        assert_eq!(font.height(), expected.height(), "{role:?}: signed height");
        assert_eq!(font.width(), expected.width(), "{role:?}: native width");
        assert_eq!(
            font.escapement(),
            expected.escapement(),
            "{role:?}: escapement"
        );
        assert_eq!(
            font.orientation(),
            expected.orientation(),
            "{role:?}: orientation"
        );
        assert_eq!(font.weight(), expected.weight(), "{role:?}: native weight");
        assert_eq!(font.italic(), expected.italic(), "{role:?}: italic byte");
        assert_eq!(
            font.underline(),
            expected.underline(),
            "{role:?}: underline byte"
        );
        assert_eq!(
            font.strike_out(),
            expected.strike_out(),
            "{role:?}: strikeout byte"
        );
        assert_eq!(
            font.char_set(),
            expected.char_set(),
            "{role:?}: character set"
        );
        assert_eq!(
            font.out_precision(),
            expected.out_precision(),
            "{role:?}: output precision"
        );
        assert_eq!(
            font.clip_precision(),
            expected.clip_precision(),
            "{role:?}: clip precision"
        );
        assert_eq!(font.quality(), expected.quality(), "{role:?}: quality");
        assert_eq!(
            font.pitch_and_family(),
            expected.pitch_and_family(),
            "{role:?}: pitch/family"
        );
        println!("{role:?}: {font:?}");
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
