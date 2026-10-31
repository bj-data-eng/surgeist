//! Checked current-monitor enrichment. No renderer capabilities are inferred.

#[cfg(any(target_os = "macos", test))]
use crate::Size;
use crate::{Point, Rect, ScaleFactor};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct MonitorFacts {
    pub(crate) available_area: Option<Rect>,
    pub(crate) color_depth: Option<u16>,
    pub(crate) color_component_bits: Option<u16>,
}

pub(crate) fn observe(
    monitor: &winit::monitor::MonitorHandle,
    origin: Point,
    scale: ScaleFactor,
) -> MonitorFacts {
    #[cfg(target_os = "macos")]
    {
        use objc2::MainThreadMarker;
        use objc2_app_kit::{
            NSBitsPerSampleFromDepth, NSColorSpaceFromDepth, NSNumberOfColorComponents, NSScreen,
        };
        use winit::platform::macos::MonitorHandleExtMacOS;
        let Some(main_thread) = MainThreadMarker::new() else {
            return MonitorFacts::default();
        };
        let Some(id) = DisplayId::new(monitor.native_id()) else {
            return MonitorFacts::default();
        };
        let screens = NSScreen::screens(main_thread);
        let mut candidates = Vec::new();
        for index in 0..screens.count() {
            let screen = screens.objectAtIndex(index);
            let frame = screen.frame();
            let visible = screen.visibleFrame();
            let depth = screen.depth();
            let color = (depth.0 > 0)
                .then(|| {
                    let space = NSColorSpaceFromDepth(depth)?;
                    color_estimate(
                        NSBitsPerSampleFromDepth(depth),
                        NSNumberOfColorComponents(&space),
                        &space.to_string(),
                    )
                })
                .flatten();
            candidates.push(ScreenObservation {
                id: screen_number(&screen.deviceDescription()),
                frame: Rect {
                    origin: Point {
                        x: frame.origin.x,
                        y: frame.origin.y,
                    },
                    size: Size {
                        width: frame.size.width,
                        height: frame.size.height,
                    },
                },
                visible: Rect {
                    origin: Point {
                        x: visible.origin.x,
                        y: visible.origin.y,
                    },
                    size: Size {
                        width: visible.size.width,
                        height: visible.size.height,
                    },
                },
                backing_scale: screen.backingScaleFactor(),
                color,
            });
        }
        facts_for(Some(id), &candidates, origin, scale)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (monitor, origin, scale);
        MonitorFacts::default()
    }
}

#[cfg(any(target_os = "macos", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DisplayId(std::num::NonZeroU32);

#[cfg(any(target_os = "macos", test))]
impl DisplayId {
    fn new(value: u32) -> Option<Self> {
        std::num::NonZeroU32::new(value).map(Self)
    }
}

#[cfg(target_os = "macos")]
fn screen_number(
    description: &objc2_foundation::NSDictionary<
        objc2_foundation::NSString,
        objc2::runtime::AnyObject,
    >,
) -> Option<DisplayId> {
    use objc2::encode::Encoding;
    use objc2_foundation::{NSNumber, NSString};
    let value = description
        .objectForKey(&NSString::from_str("NSScreenNumber"))?
        .downcast::<NSNumber>()
        .ok()?;
    let id = match value.encoding() {
        Encoding::Char | Encoding::Short | Encoding::Int | Encoding::Long | Encoding::LongLong => {
            u32::try_from(value.as_i64()).ok()?
        }
        Encoding::UChar
        | Encoding::UShort
        | Encoding::UInt
        | Encoding::ULong
        | Encoding::ULongLong => u32::try_from(value.as_u64()).ok()?,
        _ => return None,
    };
    DisplayId::new(id)
}

#[cfg(any(target_os = "macos", test))]
#[derive(Clone, Copy, Debug)]
struct ColorEstimate {
    depth: u16,
    rgb_component_bits: Option<u16>,
}

#[cfg(any(target_os = "macos", test))]
fn color_estimate(bits: isize, components: isize, space: &str) -> Option<ColorEstimate> {
    let bits = u16::try_from(bits).ok().filter(|value| *value > 0)?;
    let components = u16::try_from(components).ok().filter(|value| *value > 0)?;
    let rgb_component_bits = match space {
        "NSCalibratedRGBColorSpace" | "NSDeviceRGBColorSpace" if components == 3 => Some(bits),
        "NSCalibratedWhiteColorSpace"
        | "NSDeviceWhiteColorSpace"
        | "NSCalibratedBlackColorSpace"
        | "NSDeviceBlackColorSpace"
            if components == 1 =>
        {
            None
        }
        _ => return None,
    };
    Some(ColorEstimate {
        depth: bits.checked_mul(components)?,
        rgb_component_bits,
    })
}

#[cfg(any(target_os = "macos", test))]
#[derive(Clone, Copy, Debug)]
struct ScreenObservation {
    id: Option<DisplayId>,
    frame: Rect,
    visible: Rect,
    backing_scale: f64,
    color: Option<ColorEstimate>,
}

#[cfg(any(target_os = "macos", test))]
fn facts_for(
    id: Option<DisplayId>,
    screens: &[ScreenObservation],
    origin: Point,
    scale: ScaleFactor,
) -> MonitorFacts {
    let Some(id) = id else {
        return MonitorFacts::default();
    };
    let mut matches = screens.iter().filter(|screen| screen.id == Some(id));
    let Some(screen) = matches.next() else {
        return MonitorFacts::default();
    };
    if matches.next().is_some() {
        return MonitorFacts::default();
    }
    MonitorFacts {
        available_area: available_area(
            screen.frame,
            screen.visible,
            screen.backing_scale,
            origin,
            scale,
        ),
        color_depth: screen.color.map(|color| color.depth),
        color_component_bits: screen.color.and_then(|color| color.rgb_component_bits),
    }
}

#[cfg(any(target_os = "macos", test))]
fn available_area(
    frame: Rect,
    visible: Rect,
    backing_scale: f64,
    origin: Point,
    scale: ScaleFactor,
) -> Option<Rect> {
    let frame_max_x = frame.origin.x + frame.size.width;
    let frame_max_y = frame.origin.y + frame.size.height;
    let visible_max_x = visible.origin.x + visible.size.width;
    let visible_max_y = visible.origin.y + visible.size.height;
    let ratio = backing_scale / scale.get();
    if [
        frame.origin.x,
        frame.origin.y,
        frame.size.width,
        frame.size.height,
        visible.origin.x,
        visible.origin.y,
        visible.size.width,
        visible.size.height,
        frame_max_x,
        frame_max_y,
        visible_max_x,
        visible_max_y,
        origin.x,
        origin.y,
        ratio,
    ]
    .into_iter()
    .any(|value| !value.is_finite())
        || ratio <= 0.0
        || frame.size.width <= 0.0
        || frame.size.height <= 0.0
        || visible.size.width < 0.0
        || visible.size.height < 0.0
        || visible.origin.x < frame.origin.x
        || visible.origin.y < frame.origin.y
        || visible_max_x > frame_max_x
        || visible_max_y > frame_max_y
    {
        return None;
    }
    let area = Rect {
        origin: Point {
            x: origin.x + ratio * (visible.origin.x - frame.origin.x),
            y: origin.y + ratio * (frame_max_y - visible_max_y),
        },
        size: Size {
            width: ratio * visible.size.width,
            height: ratio * visible.size.height,
        },
    };
    [
        area.origin.x,
        area.origin.y,
        area.size.width,
        area.size.height,
    ]
    .into_iter()
    .all(f64::is_finite)
    .then_some(area)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScreenAreaSource, ScreenMetrics};
    fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect {
            origin: Point { x, y },
            size: Size { width, height },
        }
    }
    fn scale(value: f64) -> ScaleFactor {
        ScaleFactor::new(value).unwrap()
    }
    fn screen(id: u32) -> ScreenObservation {
        ScreenObservation {
            id: DisplayId::new(id),
            frame: rect(0.0, 0.0, 1000.0, 800.0),
            visible: rect(0.0, 20.0, 1000.0, 750.0),
            backing_scale: 2.0,
            color: color_estimate(8, 3, "NSDeviceRGBColorSpace"),
        }
    }

    #[test]
    fn exact_display_identity_does_not_use_equal_geometry_or_first_screen() {
        let mut second = screen(2);
        second.visible = rect(40.0, 0.0, 960.0, 800.0);
        let records = [screen(1), second];
        assert_eq!(
            facts_for(DisplayId::new(2), &records, Point::default(), scale(2.0)).available_area,
            Some(rect(40.0, 0.0, 960.0, 800.0))
        );
        for id in [None, DisplayId::new(0), DisplayId::new(3)] {
            assert_eq!(
                facts_for(id, &records, Point::default(), scale(2.0)),
                MonitorFacts::default()
            );
        }
        assert_eq!(
            facts_for(
                DisplayId::new(1),
                &[screen(1), screen(1)],
                Point::default(),
                scale(2.0)
            ),
            MonitorFacts::default()
        );
    }

    #[test]
    fn work_area_uses_relative_cocoa_edges_and_captured_scale() {
        let frame = rect(-2000.0, 400.0, 1200.0, 900.0);
        let visible = rect(-1970.0, 470.0, 1100.0, 805.0);
        assert_eq!(
            available_area(
                frame,
                visible,
                2.0,
                Point {
                    x: -1600.0,
                    y: -300.0
                },
                scale(1.25)
            ),
            Some(rect(-1552.0, -260.0, 1760.0, 1288.0))
        );
        assert_eq!(
            available_area(
                rect(500.0, -900.0, 200.0, 100.0),
                rect(500.25, -899.5, 190.5, 98.25),
                1.5,
                Point { x: 10.0, y: 20.0 },
                scale(2.0)
            ),
            Some(rect(10.1875, 20.9375, 142.875, 73.6875))
        );
    }

    #[test]
    fn invalid_work_area_inputs_and_arithmetic_remain_unavailable() {
        let frame = rect(0.0, 0.0, 1000.0, 800.0);
        for visible in [
            rect(-1.0, 0.0, 1000.0, 800.0),
            rect(0.0, 0.0, 1001.0, 800.0),
            rect(0.0, 0.0, -1.0, 2.0),
            rect(0.0, f64::NAN, 1.0, 1.0),
        ] {
            assert_eq!(
                available_area(frame, visible, 2.0, Point::default(), scale(1.0)),
                None
            );
        }
        for backing in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            assert_eq!(
                available_area(frame, frame, backing, Point::default(), scale(1.0)),
                None
            );
        }
        assert_eq!(
            available_area(frame, frame, f64::MAX, Point::default(), scale(1.0)),
            None
        );
        assert_eq!(
            available_area(
                frame,
                rect(0.0, 0.0, 0.0, 0.0),
                1.0,
                Point::default(),
                scale(1.0)
            ),
            Some(rect(0.0, 800.0, 0.0, 0.0))
        );
    }

    #[test]
    fn color_estimate_counts_color_channels_and_preserves_gray_unknown_and_floor() {
        let rgb = color_estimate(8, 3, "NSDeviceRGBColorSpace").unwrap();
        assert_eq!(rgb.depth, 24); // Not the common 32-bit storage layout.
        assert_eq!(rgb.rgb_component_bits, Some(8));
        let gray = color_estimate(8, 1, "NSDeviceWhiteColorSpace").unwrap();
        let metrics = ScreenMetrics::new(
            rect(0.0, 0.0, 1.0, 1.0),
            None,
            ScreenAreaSource::OutputDevice,
            Some(gray.depth),
            gray.rgb_component_bits,
        )
        .unwrap();
        assert_eq!(metrics.color_depth(), 8);
        for (bits, components, space) in [
            (0, 3, "NSDeviceRGBColorSpace"),
            (-1, 3, "NSDeviceRGBColorSpace"),
            (8, 0, "NSDeviceRGBColorSpace"),
            (65535, 3, "NSDeviceRGBColorSpace"),
            (8, 4, "NSDeviceRGBColorSpace"),
            (8, 3, "unknown"),
        ] {
            assert!(color_estimate(bits, components, space).is_none());
        }
        assert_eq!(
            ScreenMetrics::new(
                rect(0.0, 0.0, 1.0, 1.0),
                None,
                ScreenAreaSource::OutputDevice,
                None,
                None
            )
            .unwrap()
            .color_depth(),
            24
        );
        assert_eq!(
            ScreenMetrics::new(
                rect(0.0, 0.0, 1.0, 1.0),
                None,
                ScreenAreaSource::OutputDevice,
                Some(24),
                Some(10)
            )
            .unwrap()
            .color_depth(),
            30
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn sdk_dictionary_identity_requires_exact_key_number_type_and_checked_nonzero_integer() {
        use objc2::{rc::Retained, runtime::AnyObject};
        use objc2_foundation::{NSDictionary, NSNumber, NSString};
        fn dictionary(key: &str, value: &AnyObject) -> Retained<NSDictionary<NSString, AnyObject>> {
            NSDictionary::from_slices(&[&*NSString::from_str(key)], &[value])
        }
        assert_eq!(
            screen_number(&dictionary("NSScreenNumber", &NSNumber::new_u32(17))),
            DisplayId::new(17)
        );
        assert_eq!(
            screen_number(&dictionary("wrong-key", &NSNumber::new_u32(17))),
            None
        );
        assert_eq!(
            screen_number(&dictionary("NSScreenNumber", &NSString::from_str("17"))),
            None
        );
        for number in [
            NSNumber::new_i64(-1),
            NSNumber::new_u32(0),
            NSNumber::new_u64(u64::MAX),
            NSNumber::new_f64(17.5),
        ] {
            assert_eq!(screen_number(&dictionary("NSScreenNumber", &number)), None);
        }
    }
}
