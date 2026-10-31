//! Explicit main-thread native probe; requires a macOS WindowServer session.
//!
//! Set SURGEIST_RUN_NATIVE_PREFERENCES_PROBE=1 to open an invisible window and
//! exercise the production event loop. Ordinary headless checks report a skip.

#![forbid(unsafe_code)]

#[cfg(target_os = "macos")]
fn main() {
    use objc2_app_kit::NSWorkspace;
    use std::{cell::RefCell, rc::Rc};
    use surgeist_window::{
        ContrastPreference, Handler, Ready, ReductionPreference, Result, UserPreferences, app, open,
    };

    if std::env::var("SURGEIST_RUN_NATIVE_PREFERENCES_PROBE").as_deref() != Ok("1") {
        eprintln!(
            "SKIP native_preferences_native: set SURGEIST_RUN_NATIVE_PREFERENCES_PROBE=1 in a macOS WindowServer session"
        );
        return;
    }

    struct Probe {
        preferences: Rc<RefCell<Option<UserPreferences>>>,
    }
    impl Handler for Probe {
        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            self.preferences
                .replace(Some(ready.state().environment().preferences.clone()));
            ready.exit();
            Ok(())
        }
    }

    let preferences = Rc::new(RefCell::new(None));
    app(Probe {
        preferences: preferences.clone(),
    })
    .open(open("native-preferences-probe").visible(false))
    .run()
    .expect("the hidden native loop exits cleanly after Ready");
    let captured = preferences
        .borrow_mut()
        .take()
        .expect("Ready captured the production baseline");
    // Assertions run after native teardown, never unwinding through AppKit.
    let workspace = NSWorkspace::sharedWorkspace();
    let motion = workspace.accessibilityDisplayShouldReduceMotion();
    let transparency = workspace.accessibilityDisplayShouldReduceTransparency();
    let contrast = workspace.accessibilityDisplayShouldIncreaseContrast();
    println!(
        "captured={captured:?}; AppKit motion={motion}, transparency={transparency}, contrast={contrast}"
    );
    assert_eq!(
        captured.reduced_motion,
        Some(if motion {
            ReductionPreference::Reduce
        } else {
            ReductionPreference::NoPreference
        })
    );
    assert_eq!(
        captured.reduced_transparency,
        Some(if transparency {
            ReductionPreference::Reduce
        } else {
            ReductionPreference::NoPreference
        })
    );
    assert_eq!(
        captured.contrast,
        Some(if contrast {
            ContrastPreference::More
        } else {
            ContrastPreference::NoPreference
        })
    );
    assert_eq!(captured.reduced_data, None);
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("SKIP native_preferences_native: the AppKit provider is macOS-only");
}
