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
        ContrastPreference, Handler, InputCapabilities, Point, PointerAccuracy, PointerCapability,
        Ready, Rect, ReductionPreference, Result, Size, WindowSnapshot, app, open,
    };
    use windows::Devices::Display::Core::{
        DisplayManager, DisplayManagerOptions, DisplayManagerResult, DisplayWireFormatPixelEncoding,
    };
    use windows::Devices::Input::{PointerDevice, PointerDeviceType};
    use windows::UI::ViewManagement::{AccessibilitySettings, UISettings};

    if std::env::var("SURGEIST_RUN_NATIVE_PREFERENCES_PROBE").as_deref() != Ok("1") {
        eprintln!(
            "SKIP native_preferences_native: set SURGEIST_RUN_NATIVE_PREFERENCES_PROBE=1 in a Windows desktop session"
        );
        return;
    }

    struct NativeWitness {
        monitor: winsafe::MONITORINFOEX,
        client: winsafe::RECT,
        client_origin: winsafe::POINT,
        dpi: u32,
        input: std::result::Result<InputCapabilities, String>,
        color: std::result::Result<Option<u32>, String>,
    }
    fn expected_input() -> std::result::Result<InputCapabilities, String> {
        let devices = PointerDevice::GetPointerDevices().map_err(|error| error.to_string())?;
        let count = devices.Size().map_err(|error| error.to_string())?;
        let mut pointers = Vec::new();
        let mut mouse = false;
        let mut pen = false;
        let mut pen_hover = false;
        let mut touch = false;
        for index in 0..count {
            let device = devices.GetAt(index).map_err(|error| error.to_string())?;
            let kind = device
                .PointerDeviceType()
                .map_err(|error| error.to_string())?;
            let capability =
                if kind == PointerDeviceType::Mouse || kind == PointerDeviceType::Touchpad {
                    mouse = true;
                    PointerCapability {
                        accuracy: PointerAccuracy::Fine,
                        hover: true,
                    }
                } else if kind == PointerDeviceType::Pen {
                    pen = true;
                    let hover = device
                        .MaxPointersWithZDistance()
                        .map_err(|error| error.to_string())?
                        > 0;
                    pen_hover |= hover;
                    PointerCapability {
                        accuracy: PointerAccuracy::Fine,
                        hover,
                    }
                } else if kind == PointerDeviceType::Touch {
                    touch = true;
                    PointerCapability {
                        accuracy: PointerAccuracy::Coarse,
                        hover: false,
                    }
                } else {
                    return Err(format!("unknown Windows pointer kind {kind:?}"));
                };
            pointers.push(capability);
        }
        let primary = if mouse {
            PointerCapability {
                accuracy: PointerAccuracy::Fine,
                hover: true,
            }
        } else if pen {
            PointerCapability {
                accuracy: PointerAccuracy::Fine,
                hover: pen_hover,
            }
        } else if touch {
            PointerCapability {
                accuracy: PointerAccuracy::Coarse,
                hover: false,
            }
        } else {
            PointerCapability {
                accuracy: PointerAccuracy::None,
                hover: false,
            }
        };
        InputCapabilities::new(primary, pointers).map_err(|error| error.to_string())
    }
    fn expected_color(device_name: &str) -> std::result::Result<Option<u32>, String> {
        // Collect the HWND monitor's interface independently of winit monitor
        // selection. No WinRT call is interleaved with WinSafe enumeration.
        let mut interfaces = Vec::new();
        let mut devices = winsafe::EnumDisplayDevices(
            Some(device_name),
            Some(winsafe::co::EDD::GET_DEVICE_INTERFACE_NAME),
        );
        loop {
            winsafe::SetLastError(winsafe::co::ERROR::SUCCESS);
            let Some(device) = devices.next() else { break };
            interfaces.push(device.map_err(|error| error.to_string())?.DeviceID());
        }
        let [interface] = interfaces.as_slice() else {
            return Ok(None);
        };
        if interface.is_empty() {
            return Ok(None);
        }
        let manager = DisplayManager::Create(DisplayManagerOptions::None)
            .map_err(|error| error.to_string())?;
        let observed = (|| -> windows::core::Result<Option<u32>> {
            let result = manager.TryReadCurrentStateForAllTargets()?;
            if result.ErrorCode()? != DisplayManagerResult::Success {
                return Ok(None);
            }
            let state = result.State()?;
            if !state.IsReadOnly()? || state.IsStale()? {
                return Ok(None);
            }
            let targets = state.Targets()?;
            let mut selected = None;
            for index in 0..targets.Size()? {
                let target = targets.GetAt(index)?;
                if !target.IsConnected()? {
                    continue;
                }
                if target.IsStale()? {
                    return Ok(None);
                }
                if target.DeviceInterfacePath()? == interface.as_str() {
                    if selected.is_some() {
                        return Ok(None);
                    }
                    selected = Some(target);
                }
            }
            let Some(target) = selected else {
                return Ok(None);
            };
            let wire = state.GetPathForTarget(&target)?.WireFormat()?;
            if wire.PixelEncoding()? != DisplayWireFormatPixelEncoding::Rgb444 {
                return Ok(None);
            }
            let bits = wire.BitsPerChannel()?;
            let depth = u16::try_from(bits)
                .ok()
                .filter(|bits| *bits > 0)
                .and_then(|bits| bits.checked_mul(3))
                .map(u32::from);
            if state.IsStale()? || target.IsStale()? || !target.IsConnected()? {
                return Ok(None);
            }
            Ok(depth)
        })()
        .map_err(|error| error.to_string());
        // Close even when querying or validation failed; never acquire or apply
        // a writable display state for this read-only witness.
        manager.Close().map_err(|error| error.to_string())?;
        observed
    }
    fn native_witness(title: &str) -> std::result::Result<NativeWitness, String> {
        let mut matches = Vec::new();
        winsafe::EnumWindows(|window| {
            if window.GetWindowThreadProcessId().1 == std::process::id()
                && window.GetWindowText().is_ok_and(|text| text == title)
            {
                matches.push(window);
            }
            true
        })
        .map_err(|error| error.to_string())?;
        let [window] = matches.as_slice() else {
            return Err(format!(
                "expected one process-owned probe window, found {}",
                matches.len()
            ));
        };
        let monitor = window
            .MonitorFromWindow(winsafe::co::MONITOR::DEFAULTTONULL)
            .GetMonitorInfo()
            .map_err(|error| error.to_string())?;
        let color = expected_color(&monitor.szDevice());
        Ok(NativeWitness {
            monitor,
            client: window.GetClientRect().map_err(|error| error.to_string())?,
            client_origin: window
                .ClientToScreen(winsafe::POINT::default())
                .map_err(|error| error.to_string())?,
            dpi: window.GetDpiForWindow(),
            input: expected_input(),
            color,
        })
    }
    struct Capture {
        snapshot: WindowSnapshot,
        witness: std::result::Result<NativeWitness, String>,
    }
    struct Probe {
        capture: Rc<RefCell<Option<Capture>>>,
        title: String,
    }
    impl Handler for Probe {
        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            self.capture.replace(Some(Capture {
                snapshot: ready.state().clone(),
                witness: native_witness(&self.title),
            }));
            ready.exit();
            Ok(())
        }
    }

    let capture = Rc::new(RefCell::new(None));
    let title = format!("Surgeist native probe {}", std::process::id());
    app(Probe {
        capture: capture.clone(),
        title: title.clone(),
    })
    .open(open("native-preferences-probe").title(title).visible(false))
    .run()
    .expect("the hidden native loop exits cleanly after Ready");
    let Capture { snapshot, witness } = capture.borrow_mut().take().expect("Ready captured state");
    let witness = witness.expect("native window geometry capture");
    let scale = snapshot.metrics().scale_factor();
    let logical_rect = |rect: winsafe::RECT| Rect {
        origin: Point {
            x: f64::from(rect.left) / scale,
            y: f64::from(rect.top) / scale,
        },
        size: Size {
            width: (f64::from(rect.right) - f64::from(rect.left)) / scale,
            height: (f64::from(rect.bottom) - f64::from(rect.top)) / scale,
        },
    };
    let screen = snapshot
        .environment()
        .screen
        .as_ref()
        .expect("native monitor capture");
    let assert_rect_close = |actual: Rect, expected: Rect| {
        for (actual, expected) in [
            (actual.origin.x, expected.origin.x),
            (actual.origin.y, expected.origin.y),
            (actual.size.width, expected.size.width),
            (actual.size.height, expected.size.height),
        ] {
            // Absolute division and monitor-local offset projection may differ
            // by a few floating-point rounding steps on mixed-DPI desktops.
            let tolerance = 8.0 * f64::EPSILON * expected.abs().max(1.0);
            assert!(
                (actual - expected).abs() <= tolerance,
                "{actual} != {expected}"
            );
        }
    };
    assert_rect_close(screen.area(), logical_rect(witness.monitor.rcMonitor));
    assert_rect_close(
        screen.available_area().expect("captured native work area"),
        logical_rect(witness.monitor.rcWork),
    );
    assert_eq!(scale, f64::from(witness.dpi) / 96.0);
    let client = snapshot
        .environment()
        .client_area
        .as_ref()
        .expect("native client capture");
    assert_rect_close(
        client.area(),
        Rect {
            origin: Point {
                x: f64::from(witness.client_origin.x) / scale,
                y: f64::from(witness.client_origin.y) / scale,
            },
            size: logical_rect(witness.client).size,
        },
    );
    let viewport = snapshot
        .environment()
        .viewport
        .as_ref()
        .expect("native viewport capture");
    assert_eq!(viewport.layout(), client.area().size);
    assert_eq!(viewport.sizes().large(), client.area().size);
    assert_eq!(viewport.sizes().small(), client.area().size);
    assert_eq!(viewport.sizes().dynamic(), client.area().size);
    if let Err(error) = &witness.input {
        eprintln!("Windows input observations unavailable: {error}");
    }
    assert_eq!(
        snapshot.environment().input.as_ref(),
        witness.input.as_ref().ok()
    );
    println!("native input={:?}", snapshot.environment().input);
    if let Ok(Some(depth)) = witness.color {
        assert_eq!(screen.color_depth(), depth);
        println!("native RGB wire color depth={depth}");
    } else {
        eprintln!(
            "SKIP color comparison: no complete fresh RGB native witness: {:?}",
            witness.color
        );
    }
    println!(
        "native monitor {}: screen={screen:?}, client={client:?}, scale={scale}",
        witness.monitor.szDevice()
    );
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
