#![forbid(unsafe_code)]
#![cfg(target_os = "windows")]

use std::num::NonZeroU32;
use surgeist_typed_winsafe::{self as bridge, UiFontRole};

#[test]
fn native_font_preferences_report_all_actual_roles_at_explicit_dpi() {
    let dpi = NonZeroU32::new(96).unwrap();
    let preferences = bridge::ui_fonts_for_dpi(dpi).unwrap();
    for role in [
        UiFontRole::Caption,
        UiFontRole::Icon,
        UiFontRole::Menu,
        UiFontRole::MessageBox,
        UiFontRole::SmallCaption,
        UiFontRole::StatusBar,
    ] {
        let font = preferences.font(role);
        assert_eq!(font.reference_dpi(), dpi);
        assert_eq!(
            font.face_name().encode_utf16().collect::<Vec<_>>(),
            font.face_name_utf16()
        );
        assert!((0..=1000).contains(&font.weight()));
    }
}

#[test]
fn native_monitor_and_window_queries_return_owned_values() {
    let monitors = bridge::monitors().unwrap();
    for monitor in monitors {
        assert!(!monitor.device_name.trim().is_empty());
        assert!(monitor.frame.right > monitor.frame.left);
        assert!(monitor.frame.bottom > monitor.frame.top);
        // Multiple and empty interface mappings remain valid query observations.
        winsafe::SetLastError(winsafe::co::ERROR::ACCESS_DENIED);
        let _interfaces = bridge::display_interfaces(&monitor.device_name).unwrap();
    }
    let windows = bridge::top_level_windows().unwrap();
    // Windows can disappear between enumeration and each query. Their errors
    // remain explicit results, rather than aborting or requiring raw handles.
    for window in windows {
        let _ = window.title();
        let _ = window.process_id();
        let _ = window.monitor();
        let _ = window.client_rect();
        let _ = window.client_origin();
        let _ = window.dpi();
    }
}
