//! Native pointer observations and the native content host's primary policy.

use crate::HostEnvironment;
#[cfg(any(target_os = "windows", test))]
use crate::InputCapabilities;
use crate::environment_native::CaptureProfile;

/// Distinguishes a native-owned observation (including unavailable) from facts
/// supplied independently by another host.
pub(crate) enum InputCapture {
    Preserve,
    #[cfg(any(target_os = "windows", test))]
    Observed(Option<InputCapabilities>),
}

impl InputCapture {
    pub(crate) fn apply(&self, environment: &mut HostEnvironment) {
        match self {
            Self::Preserve => {}
            #[cfg(any(target_os = "windows", test))]
            Self::Observed(input) => environment.input = input.clone(),
        }
        let _ = environment;
    }
}

/// Samples once at an existing event-loop opportunity. No native object escapes
/// this function; no device notification or independent wakeup is installed.
pub(crate) fn capture(profile: CaptureProfile) -> InputCapture {
    #[cfg(target_os = "windows")]
    if profile == CaptureProfile::NativeContent {
        // Microsoft documents package identity as a prerequisite. Treat this as
        // opportunistic capture: failure never establishes an absent device.
        return InputCapture::Observed(capture_windows());
    }
    let _ = profile;
    InputCapture::Preserve
}

#[cfg(target_os = "windows")]
fn capture_windows() -> Option<InputCapabilities> {
    use windows::Devices::Input::{PointerDevice, PointerDeviceType};

    let devices = PointerDevice::GetPointerDevices().ok()?;
    let count = devices.Size().ok()?;
    capabilities((0..count).map(|index| {
        let device = devices.GetAt(index).ok()?;
        match device.PointerDeviceType().ok()? {
            PointerDeviceType::Mouse => Some(NativePointer::Mouse),
            PointerDeviceType::Touchpad => Some(NativePointer::Touchpad),
            PointerDeviceType::Pen => Some(NativePointer::Pen {
                hover: device.MaxPointersWithZDistance().ok()? > 0,
            }),
            PointerDeviceType::Touch => Some(NativePointer::Touch),
            _ => None,
        }
    }))
}

#[cfg(any(target_os = "windows", test))]
#[derive(Clone, Copy)]
enum NativePointer {
    Mouse,
    Touchpad,
    Pen { hover: bool },
    Touch,
}

#[cfg(any(target_os = "windows", test))]
fn capabilities(
    devices: impl IntoIterator<Item = Option<NativePointer>>,
) -> Option<InputCapabilities> {
    use crate::{PointerAccuracy, PointerCapability};

    let mut pointers = Vec::new();
    let mut primary = PointerCapability {
        accuracy: PointerAccuracy::None,
        hover: false,
    };
    let mut primary_priority = (0, false);
    for device in devices {
        // Every available pointer contributes to the union. Unknown classes and
        // failed reads invalidate the whole observation rather than hiding one.
        let (pointer, priority) = match device? {
            NativePointer::Mouse | NativePointer::Touchpad => (
                PointerCapability {
                    accuracy: PointerAccuracy::Fine,
                    hover: true,
                },
                3,
            ),
            NativePointer::Pen { hover } => (
                PointerCapability {
                    accuracy: PointerAccuracy::Fine,
                    hover,
                },
                2,
            ),
            NativePointer::Touch => (
                PointerCapability {
                    accuracy: PointerAccuracy::Coarse,
                    hover: false,
                },
                1,
            ),
        };
        // Stable under enumeration order, including multiple pens with different
        // hover support. This is host policy, never the last input event.
        let priority = (priority, pointer.hover);
        if priority > primary_priority {
            primary = pointer;
            primary_priority = priority;
        }
        pointers.push(pointer);
    }
    InputCapabilities::new(primary, pointers).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NavigationControls, PointerAccuracy, PointerCapability};

    #[test]
    fn each_device_reports_its_accuracy_and_convenient_hover() {
        for (device, accuracy, hover) in [
            (NativePointer::Mouse, PointerAccuracy::Fine, true),
            (NativePointer::Touchpad, PointerAccuracy::Fine, true),
            (
                NativePointer::Pen { hover: true },
                PointerAccuracy::Fine,
                true,
            ),
            (
                NativePointer::Pen { hover: false },
                PointerAccuracy::Fine,
                false,
            ),
            (NativePointer::Touch, PointerAccuracy::Coarse, false),
        ] {
            let input = capabilities([Some(device)]).unwrap();
            assert_eq!(input.primary(), PointerCapability { accuracy, hover });
            assert_eq!(input.any_coarse(), accuracy == PointerAccuracy::Coarse);
            assert_eq!(input.any_fine(), accuracy == PointerAccuracy::Fine);
            assert_eq!(input.any_hover(), hover);
            assert!(!input.no_pointer());
        }
    }

    #[test]
    fn primary_and_complete_union_are_stable_under_device_permutations() {
        let devices = [
            NativePointer::Mouse,
            NativePointer::Touchpad,
            NativePointer::Pen { hover: false },
            NativePointer::Pen { hover: true },
            NativePointer::Touch,
        ];
        for first in 0..devices.len() {
            for second in 0..devices.len() {
                let mut order = devices;
                order.swap(0, first);
                order.swap(1, second);
                let input = capabilities(order.map(Some)).unwrap();
                assert_eq!(
                    input.primary(),
                    PointerCapability {
                        accuracy: PointerAccuracy::Fine,
                        hover: true,
                    }
                );
                assert!(input.any_coarse());
                assert!(input.any_fine());
                assert!(input.any_hover());
            }
        }
        for devices in [
            [
                NativePointer::Touch,
                NativePointer::Pen { hover: false },
                NativePointer::Pen { hover: true },
            ],
            [
                NativePointer::Pen { hover: true },
                NativePointer::Pen { hover: false },
                NativePointer::Touch,
            ],
        ] {
            let input = capabilities(devices.map(Some)).unwrap();
            assert_eq!(
                input.primary(),
                PointerCapability {
                    accuracy: PointerAccuracy::Fine,
                    hover: true,
                }
            );
            assert!(input.any_coarse());
            assert!(input.any_fine());
            assert!(input.any_hover());
        }
    }

    #[test]
    fn successful_empty_enumeration_is_distinct_from_an_unknown_or_failed_device() {
        let input = capabilities([]).unwrap();
        assert!(input.no_pointer());
        assert_eq!(
            input.primary(),
            PointerCapability {
                accuracy: PointerAccuracy::None,
                hover: false,
            }
        );
        assert!(!input.any_hover());
        assert_eq!(capabilities([None]), None);
        assert_eq!(capabilities([Some(NativePointer::Mouse), None]), None);
        assert_eq!(capabilities([None, Some(NativePointer::Touch)]), None);
    }

    #[test]
    fn preserve_and_browser_capture_retain_independent_host_facts() {
        let mut environment = HostEnvironment {
            input: capabilities([Some(NativePointer::Touch)]),
            navigation: Some(NavigationControls::Back),
            ..HostEnvironment::default()
        };
        let previous = environment.clone();
        InputCapture::Preserve.apply(&mut environment);
        assert_eq!(environment, previous);
        capture(CaptureProfile::BrowserCanvas).apply(&mut environment);
        assert_eq!(environment, previous);
    }
}
