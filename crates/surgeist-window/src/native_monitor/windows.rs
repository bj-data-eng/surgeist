//! Exact native monitor identity and checked desktop work-area conversion.

use super::{ColorEstimate, MonitorFacts};
use crate::{Point, Rect, ScaleFactor, Size};

#[cfg(target_os = "windows")]
mod color;

#[cfg(target_os = "windows")]
pub(super) fn observe(
    monitor: &winit::monitor::MonitorHandle,
    origin: Point,
    scale: ScaleFactor,
) -> MonitorFacts {
    use winit::platform::windows::MonitorHandleExtWindows;

    let id = monitor.native_id();
    let Ok(monitors) = surgeist_typed_winsafe::monitors() else {
        return MonitorFacts::default();
    };
    let observations = monitors
        .into_iter()
        .map(|monitor| Observation {
            id: monitor.device_name,
            frame: NativeRect::from(monitor.frame),
            work: NativeRect::from(monitor.work_area),
        })
        .collect::<Vec<_>>();
    facts_for(&id, &observations, origin, scale, color::observe(&id))
}

#[derive(Clone, Copy)]
struct NativeRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[cfg(target_os = "windows")]
impl From<surgeist_typed_winsafe::Rect> for NativeRect {
    fn from(rect: surgeist_typed_winsafe::Rect) -> Self {
        Self {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        }
    }
}

struct Observation {
    id: String,
    frame: NativeRect,
    work: NativeRect,
}

fn facts_for(
    id: &str,
    observations: &[Observation],
    origin: Point,
    scale: ScaleFactor,
    color: Option<ColorEstimate>,
) -> MonitorFacts {
    if id.trim().is_empty() {
        return MonitorFacts::default();
    }
    let mut seen = std::collections::HashSet::new();
    let mut selected = None;
    for observation in observations {
        if observation.id.trim().is_empty() || !seen.insert(observation.id.as_str()) {
            return MonitorFacts::default();
        }
        if observation.id == id {
            selected = Some(observation);
        }
    }
    MonitorFacts {
        available_area: selected.and_then(|observation| {
            available_area(observation.frame, observation.work, origin, scale)
        }),
        color_depth: selected.and(color).map(|color| color.depth),
        color_component_bits: selected
            .and(color)
            .and_then(|color| color.rgb_component_bits),
    }
}

fn available_area(
    frame: NativeRect,
    work: NativeRect,
    origin: Point,
    scale: ScaleFactor,
) -> Option<Rect> {
    let frame_width = frame.right.checked_sub(frame.left)?;
    let frame_height = frame.bottom.checked_sub(frame.top)?;
    let width = work.right.checked_sub(work.left)?;
    let height = work.bottom.checked_sub(work.top)?;
    if frame_width <= 0
        || frame_height <= 0
        || width < 0
        || height < 0
        || work.left < frame.left
        || work.top < frame.top
        || work.right > frame.right
        || work.bottom > frame.bottom
    {
        return None;
    }
    // Windows rectangles already use top-left origins and downward-positive Y.
    // Anchor physical insets to the caller's committed logical screen origin.
    let area = Rect {
        origin: Point {
            x: origin.x + f64::from(work.left.checked_sub(frame.left)?) / scale.get(),
            y: origin.y + f64::from(work.top.checked_sub(frame.top)?) / scale.get(),
        },
        size: Size {
            width: f64::from(width) / scale.get(),
            height: f64::from(height) / scale.get(),
        },
    };
    [
        origin.x,
        origin.y,
        area.origin.x,
        area.origin.y,
        area.size.width,
        area.size.height,
        area.origin.x + area.size.width,
        area.origin.y + area.size.height,
    ]
    .into_iter()
    .all(f64::is_finite)
    .then_some(area)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edges(left: i32, top: i32, right: i32, bottom: i32) -> NativeRect {
        NativeRect {
            left,
            top,
            right,
            bottom,
        }
    }

    fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect {
            origin: Point { x, y },
            size: Size { width, height },
        }
    }

    fn scale(value: f64) -> ScaleFactor {
        ScaleFactor::new(value).unwrap()
    }

    fn observation(id: &str, work: NativeRect) -> Observation {
        Observation {
            id: id.to_owned(),
            frame: edges(-2400, -1200, 0, 600),
            work,
        }
    }

    #[test]
    fn physical_insets_use_committed_origin_and_scale_on_negative_desktops() {
        let frame = edges(-2400, -1200, 0, 600);
        let work = edges(-2370, -1140, -90, 480);
        assert_eq!(
            available_area(
                frame,
                work,
                Point {
                    x: -1920.0,
                    y: -960.0
                },
                scale(1.25)
            ),
            Some(rect(-1896.0, -912.0, 1824.0, 1296.0))
        );
        assert_eq!(
            available_area(frame, work, Point { x: 100.0, y: 200.0 }, scale(2.0)),
            Some(rect(115.0, 230.0, 1140.0, 810.0))
        );
        assert_eq!(
            available_area(
                frame,
                frame,
                Point {
                    x: -1920.0,
                    y: -960.0
                },
                scale(1.0)
            ),
            Some(rect(-1920.0, -960.0, 2400.0, 1800.0))
        );
    }

    #[test]
    fn exact_device_id_selects_its_work_area_without_inferring_color() {
        let observations = [
            observation("DISPLAY1", edges(-2400, -1200, 0, 600)),
            observation("DISPLAY2", edges(-2370, -1140, -90, 480)),
        ];
        assert_eq!(
            facts_for(
                "DISPLAY2",
                &observations,
                Point::default(),
                scale(2.0),
                None
            ),
            MonitorFacts {
                available_area: Some(rect(15.0, 30.0, 1140.0, 810.0)),
                color_depth: None,
                color_component_bits: None,
            }
        );
        for id in ["", " ", "DISPLAY3", "display2", " DISPLAY2"] {
            assert_eq!(
                facts_for(id, &observations, Point::default(), scale(1.0), None),
                MonitorFacts::default()
            );
        }
        assert_eq!(
            facts_for("DISPLAY1", &[], Point::default(), scale(1.0), None),
            MonitorFacts::default()
        );
    }

    #[test]
    fn unavailable_color_preserves_valid_work_area_and_observed_color_composes_independently() {
        let observations = [observation("DISPLAY1", edges(-2370, -1140, -90, 480))];
        let without_color = facts_for(
            "DISPLAY1",
            &observations,
            Point::default(),
            scale(2.0),
            None,
        );
        assert_eq!(
            without_color.available_area,
            Some(rect(15.0, 30.0, 1140.0, 810.0))
        );
        assert_eq!(without_color.color_depth, None);
        assert_eq!(without_color.color_component_bits, None);
        let color = Some(ColorEstimate {
            depth: 30,
            rgb_component_bits: Some(10),
        });
        assert_eq!(
            facts_for(
                "DISPLAY1",
                &observations,
                Point::default(),
                scale(2.0),
                color
            ),
            MonitorFacts {
                available_area: without_color.available_area,
                color_depth: Some(30),
                color_component_bits: Some(10),
            }
        );
        assert_eq!(
            facts_for(
                "unknown",
                &observations,
                Point::default(),
                scale(2.0),
                color
            ),
            MonitorFacts::default()
        );
    }

    #[test]
    fn incomplete_or_duplicate_native_identities_leave_work_area_unavailable() {
        let frame = edges(-2400, -1200, 0, 600);
        for ids in [
            ["DISPLAY1", "DISPLAY1"],
            ["DISPLAY1", ""],
            ["DISPLAY1", " "],
        ] {
            let observations = ids.map(|id| observation(id, frame));
            assert_eq!(
                facts_for(
                    "DISPLAY1",
                    &observations,
                    Point::default(),
                    scale(1.0),
                    None
                ),
                MonitorFacts::default()
            );
        }
        let observations = ["DISPLAY1", "DISPLAY2", "DISPLAY2"].map(|id| observation(id, frame));
        assert_eq!(
            facts_for(
                "DISPLAY1",
                &observations,
                Point::default(),
                scale(1.0),
                None
            ),
            MonitorFacts::default()
        );
    }

    #[test]
    fn invalid_or_noncontained_native_rectangles_remain_unavailable() {
        let frame = edges(0, 0, 1000, 800);
        for work in [
            edges(-1, 0, 1000, 800),
            edges(0, -1, 1000, 800),
            edges(0, 0, 1001, 800),
            edges(0, 0, 1000, 801),
            edges(10, 0, 9, 800),
            edges(0, 10, 1000, 9),
        ] {
            assert!(available_area(frame, work, Point::default(), scale(1.0)).is_none());
        }
        for frame in [
            edges(0, 0, 0, 800),
            edges(0, 0, 1000, 0),
            edges(1000, 0, 0, 800),
            edges(i32::MIN, 0, i32::MAX, 800),
            edges(0, i32::MIN, 1000, i32::MAX),
        ] {
            assert!(available_area(frame, frame, Point::default(), scale(1.0)).is_none());
        }
        assert_eq!(
            available_area(frame, edges(30, 40, 30, 40), Point::default(), scale(1.0)),
            Some(rect(30.0, 40.0, 0.0, 0.0))
        );
    }

    #[test]
    fn nonfinite_conversion_or_output_edges_remain_unavailable() {
        let frame = edges(0, 0, 1000, 800);
        for origin in [
            Point {
                x: f64::NAN,
                y: 0.0,
            },
            Point {
                x: 0.0,
                y: f64::INFINITY,
            },
        ] {
            assert!(available_area(frame, frame, origin, scale(1.0)).is_none());
        }
        assert!(available_area(frame, frame, Point::default(), scale(f64::MIN_POSITIVE)).is_none());
        assert!(
            available_area(
                frame,
                frame,
                Point {
                    x: f64::MAX,
                    y: 0.0
                },
                scale(1e-305),
            )
            .is_none()
        );
    }
}
