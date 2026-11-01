//! Explicit native preference and monitor probe; requires a desktop session.
//!
//! Set SURGEIST_RUN_NATIVE_PREFERENCES_PROBE=1 to open an invisible window and
//! exercise the production event loop. Ordinary headless checks report a skip.

#![forbid(unsafe_code)]

#[cfg(target_os = "macos")]
fn main() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{
        NSBitsPerSampleFromDepth, NSColorSpaceFromDepth, NSNumberOfColorComponents, NSScreen,
        NSWorkspace,
    };
    use objc2_foundation::{NSNumber, NSString};
    use std::{cell::RefCell, rc::Rc};
    use surgeist_window::{
        ContrastPreference, Handler, Point, Ready, Rect, ReductionPreference, Result, Size,
        WindowSnapshot, app, open,
    };

    if std::env::var("SURGEIST_RUN_NATIVE_PREFERENCES_PROBE").as_deref() != Ok("1") {
        eprintln!(
            "SKIP native_preferences_native: set SURGEIST_RUN_NATIVE_PREFERENCES_PROBE=1 in a macOS WindowServer session"
        );
        return;
    }

    struct Probe {
        snapshot: Rc<RefCell<Option<WindowSnapshot>>>,
    }
    impl Handler for Probe {
        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            self.snapshot.replace(Some(ready.state().clone()));
            ready.exit();
            Ok(())
        }
    }

    let snapshot = Rc::new(RefCell::new(None));
    app(Probe {
        snapshot: snapshot.clone(),
    })
    .open(open("native-preferences-probe").visible(false))
    .run()
    .expect("the hidden native loop exits cleanly after Ready");
    let snapshot = snapshot
        .borrow_mut()
        .take()
        .expect("Ready captured the production environment");
    let captured = &snapshot.environment().preferences;
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

    // No native monitor ID is public. Only a single checked SDK screen supplies
    // an unambiguous hardware witness here; selection by exact ID is exercised
    // by the owning module's deterministic tests. Multiple screens are reported,
    // never reduced to main/first/name/geometry guesses.
    let main_thread =
        MainThreadMarker::new().expect("native probe runs on the process main thread");
    let screens = NSScreen::screens(main_thread);
    let candidates = (0..screens.count())
        .map(|index| screens.objectAtIndex(index))
        .collect::<Vec<_>>();
    let [candidate] = candidates.as_slice() else {
        eprintln!(
            "SKIP monitor comparison: {} SDK screens; public snapshot has no native ID to resolve ambiguity",
            candidates.len()
        );
        return;
    };
    let Some(number) = candidate
        .deviceDescription()
        .objectForKey(&NSString::from_str("NSScreenNumber"))
        .and_then(|value| value.downcast::<NSNumber>().ok())
    else {
        eprintln!("SKIP monitor comparison: SDK NSScreenNumber is not an NSNumber");
        return;
    };
    let sdk_id = match number.encoding() {
        objc2::encode::Encoding::Char
        | objc2::encode::Encoding::Short
        | objc2::encode::Encoding::Int
        | objc2::encode::Encoding::Long
        | objc2::encode::Encoding::LongLong => u32::try_from(number.as_i64()).ok(),
        objc2::encode::Encoding::UChar
        | objc2::encode::Encoding::UShort
        | objc2::encode::Encoding::UInt
        | objc2::encode::Encoding::ULong
        | objc2::encode::Encoding::ULongLong => u32::try_from(number.as_u64()).ok(),
        _ => None,
    }
    .filter(|value| *value != 0);
    let Some(sdk_id) = sdk_id else {
        eprintln!("SKIP monitor comparison: SDK screen ID is not a nonzero display identifier");
        return;
    };
    let frame = candidate.frame();
    let visible = candidate.visibleFrame();
    let ratio = candidate.backingScaleFactor() / snapshot.metrics().scale_factor();
    let Some(observed) = snapshot.environment().screen.as_ref() else {
        eprintln!("SKIP monitor comparison: production snapshot has no selected screen");
        return;
    };
    let values = [
        frame.origin.x,
        frame.origin.y,
        frame.size.width,
        frame.size.height,
        visible.origin.x,
        visible.origin.y,
        visible.size.width,
        visible.size.height,
        ratio,
    ];
    if values.into_iter().any(|value| !value.is_finite())
        || ratio <= 0.0
        || frame.size.width <= 0.0
        || frame.size.height <= 0.0
        || visible.size.width <= 0.0
        || visible.size.height <= 0.0
    {
        eprintln!("SKIP monitor comparison: invalid SDK frame, visible frame or scale");
        return;
    }
    let expected = Rect {
        origin: Point {
            x: observed.area().origin.x + ratio * (visible.origin.x - frame.origin.x),
            y: observed.area().origin.y
                + ratio
                    * ((frame.origin.y + frame.size.height)
                        - (visible.origin.y + visible.size.height)),
        },
        size: Size {
            width: ratio * visible.size.width,
            height: ratio * visible.size.height,
        },
    };
    let depth = candidate.depth();
    let bits = NSBitsPerSampleFromDepth(depth);
    let color_space = NSColorSpaceFromDepth(depth);
    let components = color_space
        .as_ref()
        .map(|space| NSNumberOfColorComponents(space));
    let color_space_name = color_space.map(|space| space.to_string());
    println!(
        "SDK screen ID={sdk_id}, frame={frame:?}, visible={visible:?}, ratio={ratio}, color space={color_space_name:?}, bits={bits}, components={components:?}; expected available={expected:?}, captured screen={observed:?}"
    );
    assert_eq!(
        observed.available_area(),
        Some(expected),
        "the native screen should expose the observed SDK available area"
    );
    if let Some(estimated) = u32::try_from(bits)
        .ok()
        .filter(|value| *value > 0)
        .zip(
            components
                .and_then(|count| u32::try_from(count).ok())
                .filter(|value| *value > 0),
        )
        .and_then(|(bits, components)| bits.checked_mul(components))
    {
        assert!(
            observed.color_depth() >= estimated,
            "color depth excludes alpha/storage padding and permits the public lower bound"
        );
    }
}

#[cfg(target_os = "windows")]
fn main() {
    use std::{cell::RefCell, rc::Rc};
    use surgeist_window::{
        ContrastPreference, Handler, Ready, ReductionPreference, Result, WindowSnapshot, app, open,
    };
    use windows::UI::ViewManagement::{AccessibilitySettings, UISettings};

    if std::env::var("SURGEIST_RUN_NATIVE_PREFERENCES_PROBE").as_deref() != Ok("1") {
        eprintln!(
            "SKIP native_preferences_native: set SURGEIST_RUN_NATIVE_PREFERENCES_PROBE=1 in a Windows desktop session"
        );
        return;
    }

    struct Probe(Rc<RefCell<Option<WindowSnapshot>>>);
    impl Handler for Probe {
        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            self.0.replace(Some(ready.state().clone()));
            ready.exit();
            Ok(())
        }
    }

    let snapshot = Rc::new(RefCell::new(None));
    app(Probe(snapshot.clone()))
        .open(open("native-preferences-probe").visible(false))
        .run()
        .expect("the hidden native loop exits cleanly after Ready");
    let snapshot = snapshot.borrow_mut().take().expect("Ready captured state");
    let captured = &snapshot.environment().preferences;
    let ui = UISettings::new().expect("native UI settings");
    let animations = ui.AnimationsEnabled().expect("animation observation");
    let transparency = ui
        .AdvancedEffectsEnabled()
        .expect("transparency observation");
    let contrast = AccessibilitySettings::new()
        .and_then(|settings| settings.HighContrast())
        .expect("contrast theme observation");
    println!(
        "captured={captured:?}; Windows animations={animations}, transparency={transparency}, contrast={contrast}"
    );
    assert_eq!(
        captured.reduced_motion,
        Some(if animations {
            ReductionPreference::NoPreference
        } else {
            ReductionPreference::Reduce
        })
    );
    assert_eq!(
        captured.reduced_transparency,
        Some(if transparency {
            ReductionPreference::NoPreference
        } else {
            ReductionPreference::Reduce
        })
    );
    assert_eq!(
        captured.contrast,
        Some(if contrast {
            ContrastPreference::Custom
        } else {
            ContrastPreference::NoPreference
        })
    );
    assert_eq!(captured.reduced_data, None);
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn main() {
    eprintln!("SKIP native_preferences_native: no native preference provider for this target");
}
