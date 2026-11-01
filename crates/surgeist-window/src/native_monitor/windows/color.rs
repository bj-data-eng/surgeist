//! Query-only wire precision for one exactly identified native monitor.

use super::ColorEstimate;
use windows::Devices::Display::Core::{
    DisplayManager, DisplayManagerOptions, DisplayManagerResult, DisplayPathStatus,
    DisplayWireFormatPixelEncoding,
};

// Bounds limit host query work. Hitting a bound never publishes a partial result.
const MAX_INTERFACES: usize = 128;
const MAX_TARGETS: u32 = 1024;

pub(super) fn observe(device_name: &str) -> Option<ColorEstimate> {
    use winsafe as w;

    if device_name.trim().is_empty() {
        return None;
    }
    let mut devices = w::EnumDisplayDevices(
        Some(device_name),
        Some(w::co::EDD::GET_DEVICE_INTERFACE_NAME),
    );
    // Own every returned string before any WinRT calls. The iterator's native
    // buffer is reused, and its terminal FALSE consults the thread's error slot.
    let interfaces = collect_interfaces(std::iter::from_fn(|| {
        w::SetLastError(w::co::ERROR::SUCCESS);
        devices
            .next()
            .map(|device| device.map(|device| device.DeviceID()))
    }))?;
    // A cloned GDI output can identify multiple physical monitors. Do not pick
    // one physical target's precision as the logical output's observation.
    if interfaces.len() != 1 {
        return None;
    }
    let manager = DisplayManager::Create(DisplayManagerOptions::None).ok()?;
    // Query returns only Rust values; all state/path/target objects are released
    // on this sampling thread before Close, including every query failure path.
    let estimate = query(&manager, &interfaces);
    if manager.Close().is_err() {
        return None;
    }
    estimate
}

fn collect_interfaces<E>(
    mut devices: impl Iterator<Item = Result<String, E>>,
) -> Option<Vec<String>> {
    let mut interfaces = Vec::new();
    let mut seen = std::collections::HashSet::new();
    loop {
        match devices.next() {
            None => return (!interfaces.is_empty()).then_some(interfaces),
            Some(device) => {
                let interface = device.ok()?;
                if interfaces.len() == MAX_INTERFACES
                    || interface.trim().is_empty()
                    || !seen.insert(interface.clone())
                {
                    return None;
                }
                interfaces.push(interface);
            }
        }
    }
}

struct WireObservation {
    encoding: DisplayWireFormatPixelEncoding,
    bits_per_channel: i32,
}

struct TargetObservation {
    interface: String,
    connected: bool,
    stale: bool,
    wire: Option<WireObservation>,
}

struct StateObservation {
    read_only: bool,
    stale: bool,
    targets: Vec<TargetObservation>,
}

fn query(manager: &DisplayManager, interfaces: &[String]) -> Option<ColorEstimate> {
    let result = manager.TryReadCurrentStateForAllTargets().ok()?;
    if result.ErrorCode().ok()? != DisplayManagerResult::Success
        || result.ExtendedErrorCode().ok()?.is_err()
    {
        return None;
    }
    let state = result.State().ok()?;
    let read_only = state.IsReadOnly().ok()?;
    let stale_before = state.IsStale().ok()?;
    if !read_only || stale_before {
        return None;
    }
    let native_targets = state.Targets().ok()?;
    let count = native_targets.Size().ok()?;
    if count > MAX_TARGETS {
        return None;
    }
    let mut targets = Vec::new();
    for index in 0..count {
        let target = native_targets.GetAt(index).ok()?;
        if !target.IsConnected().ok()? {
            continue;
        }
        let stale_before = target.IsStale().ok()?;
        if stale_before {
            return None;
        }
        let interface = target.DeviceInterfacePath().ok()?.to_string();
        let wire = if interfaces.contains(&interface) {
            let path = state.GetPathForTarget(&target).ok()?;
            // Unknown is a valid observed status for a read-only snapshot.
            // Pending or failed configurations cannot establish precision.
            if !matches!(
                path.Status().ok()?,
                DisplayPathStatus::Unknown | DisplayPathStatus::Succeeded
            ) {
                return None;
            }
            let wire = path.WireFormat().ok()?;
            Some(WireObservation {
                encoding: wire.PixelEncoding().ok()?,
                bits_per_channel: wire.BitsPerChannel().ok()?,
            })
        } else {
            None
        };
        targets.push(TargetObservation {
            interface,
            connected: target.IsConnected().ok()?,
            stale: stale_before || target.IsStale().ok()?,
            wire,
        });
    }
    select_color(
        interfaces,
        &StateObservation {
            read_only,
            stale: stale_before || state.IsStale().ok()?,
            targets,
        },
    )
}

fn select_color(interfaces: &[String], state: &StateObservation) -> Option<ColorEstimate> {
    let [interface] = interfaces else {
        return None;
    };
    if interface.trim().is_empty() || !state.read_only || state.stale {
        return None;
    }
    let mut selected = None;
    let mut seen = std::collections::HashSet::new();
    for target in &state.targets {
        if target.interface.trim().is_empty()
            || !seen.insert(target.interface.as_str())
            || !target.connected
            || target.stale
        {
            return None;
        }
        if &target.interface == interface {
            selected = Some(target);
        }
    }
    let wire = selected?.wire.as_ref()?;
    estimate(wire.encoding, wire.bits_per_channel)
}

fn estimate(
    encoding: DisplayWireFormatPixelEncoding,
    bits_per_channel: i32,
) -> Option<ColorEstimate> {
    if encoding != DisplayWireFormatPixelEncoding::Rgb444 {
        return None;
    }
    let bits = u16::try_from(bits_per_channel)
        .ok()
        .filter(|bits| *bits > 0)?;
    Some(ColorEstimate {
        depth: bits.checked_mul(3)?,
        rgb_component_bits: Some(bits),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(interface: &str) -> TargetObservation {
        TargetObservation {
            interface: interface.to_owned(),
            connected: true,
            stale: false,
            wire: Some(WireObservation {
                encoding: DisplayWireFormatPixelEncoding::Rgb444,
                bits_per_channel: 10,
            }),
        }
    }

    fn state(targets: Vec<TargetObservation>) -> StateObservation {
        StateObservation {
            read_only: true,
            stale: false,
            targets,
        }
    }

    fn values(estimate: Option<ColorEstimate>) -> Option<(u16, Option<u16>)> {
        estimate.map(|color| (color.depth, color.rgb_component_bits))
    }

    #[test]
    fn rgb_wire_precision_counts_three_color_channels_without_alpha() {
        for (bits, depth, component) in [
            (6, 18, 6),
            (8, 24, 8),
            (10, 30, 10),
            (12, 36, 12),
            (16, 48, 16),
        ] {
            assert_eq!(
                values(estimate(DisplayWireFormatPixelEncoding::Rgb444, bits)),
                Some((depth, Some(component)))
            );
        }
        for bits in [i32::MIN, -1, 0, 21846, 65536, i32::MAX] {
            assert!(estimate(DisplayWireFormatPixelEncoding::Rgb444, bits).is_none());
        }
        for encoding in [
            DisplayWireFormatPixelEncoding::Ycc444,
            DisplayWireFormatPixelEncoding::Ycc422,
            DisplayWireFormatPixelEncoding::Ycc420,
            DisplayWireFormatPixelEncoding::Intensity,
            DisplayWireFormatPixelEncoding(-1),
            DisplayWireFormatPixelEncoding(i32::MAX),
        ] {
            assert!(estimate(encoding, 10).is_none());
        }
    }

    #[test]
    fn complete_interface_collection_rejects_errors_empty_duplicates_and_truncation() {
        assert_eq!(
            collect_interfaces::<()>([Ok("monitor".to_owned())].into_iter()),
            Some(vec!["monitor".to_owned()])
        );
        for records in [
            vec![],
            vec![Ok("".to_owned())],
            vec![Ok(" ".to_owned())],
            vec![Ok("monitor".to_owned()), Err(())],
            vec![Ok("monitor".to_owned()), Ok("monitor".to_owned())],
        ] {
            assert!(collect_interfaces(records.into_iter()).is_none());
        }
        assert!(
            collect_interfaces::<()>((0..=MAX_INTERFACES).map(|index| Ok(index.to_string())))
                .is_none()
        );
        assert_eq!(
            collect_interfaces::<()>((0..MAX_INTERFACES).map(|index| Ok(index.to_string())))
                .unwrap()
                .len(),
            MAX_INTERFACES
        );
    }

    #[test]
    fn exact_interface_selects_a_fresh_target_without_order_or_geometry_guesses() {
        let interfaces = ["second".to_owned()];
        assert_eq!(
            values(select_color(
                &interfaces,
                &state(vec![target("first"), target("second")])
            )),
            Some((30, Some(10)))
        );
        for interfaces in [
            vec![],
            vec!["unknown".to_owned()],
            vec!["SECOND".to_owned()],
            vec![" second".to_owned()],
            vec!["".to_owned()],
            vec!["first".to_owned(), "second".to_owned()],
        ] {
            assert!(select_color(&interfaces, &state(vec![target("second")])).is_none());
        }
    }

    #[test]
    fn stale_mutable_disconnected_duplicate_or_incomplete_results_are_unavailable() {
        let interfaces = ["monitor".to_owned()];
        let mut disconnected = target("monitor");
        disconnected.connected = false;
        let mut stale = target("monitor");
        stale.stale = true;
        let mut missing_wire = target("monitor");
        missing_wire.wire = None;
        for snapshot in [
            StateObservation {
                read_only: false,
                ..state(vec![target("monitor")])
            },
            StateObservation {
                stale: true,
                ..state(vec![target("monitor")])
            },
            state(vec![]),
            state(vec![disconnected]),
            state(vec![stale]),
            state(vec![missing_wire]),
            state(vec![target("monitor"), target("monitor")]),
            state(vec![target("monitor"), target("")]),
            state(vec![target("monitor"), target("other"), target("other")]),
        ] {
            assert!(select_color(&interfaces, &snapshot).is_none());
        }
    }
}
