use crate::{Error, ErrorCode, Id, Point, PointerEvent, Result, WindowSnapshot};

/// Native logical coordinates observed for one window's pointer event.
///
/// Values use the host's logical window units: native pixels divided by the
/// window's display scale, with axes increasing rightward and downward. They
/// retain fractional positions and may be negative. They are not DOM MouseEvent
/// coordinates or a conversion to CSS pixels at arbitrary page zoom.
///
/// The client and selected screen origins must come from the same native
/// coordinate space as the event. The default native host supplies this space
/// at page zoom one. Page zoom, visual magnification, document scrolling,
/// dispatch state and target geometry do not participate in this observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativePointerCoordinates {
    id: Id,
    client: Option<Point>,
    desktop: Option<Point>,
    screen: Option<Point>,
}

impl NativePointerCoordinates {
    /// Returns the window identity shared by the event and snapshot.
    #[must_use]
    pub const fn id(self) -> Id {
        self.id
    }

    /// Returns the event position relative to the native client area's origin.
    ///
    /// An event without a logical position returns `None`; integer physical
    /// coordinates are not substituted for the missing observation.
    #[must_use]
    pub const fn client(self) -> Option<Point> {
        self.client
    }

    /// Returns the position relative to the native desktop origin.
    ///
    /// This adds the observed client-area origin to the client position. It is
    /// unavailable if either observation is absent. Decorated outer-window
    /// geometry never supplies a missing client origin.
    #[must_use]
    pub const fn desktop(self) -> Option<Point> {
        self.desktop
    }

    /// Returns the position relative to the selected native screen area's origin.
    ///
    /// This subtracts the captured screen origin from the desktop position; it
    /// does not clamp the pointer to the selected screen. Missing desktop or
    /// selected-screen observations return `None`.
    #[must_use]
    pub const fn screen(self) -> Option<Point> {
        self.screen
    }
}

impl PointerEvent {
    /// Observes native logical coordinates against this event's window snapshot.
    ///
    /// Supply the snapshot captured with the event and native origins expressed
    /// in the event's logical units. Window identity is checked; identity alone
    /// does not establish that a separately retained snapshot is contemporaneous.
    /// The event's floating-point position is used directly, without rounding or
    /// conversion through its optional integer physical position.
    ///
    /// Missing position or origins remain explicit in the returned observation.
    /// A mismatched window, nonfinite client position or nonfinite arithmetic
    /// result returns [`ErrorCode::InvalidRequest`] with the event's window id.
    pub fn native_coordinates(
        &self,
        snapshot: &WindowSnapshot,
    ) -> Result<NativePointerCoordinates> {
        if self.id != snapshot.id() {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                "pointer event and coordinate snapshot identify different windows",
            )
            .with_id(self.id));
        }
        let client = self
            .position
            .map(|point| finite_point(self.id, point))
            .transpose()?;
        let environment = snapshot.environment();
        let desktop = client
            .zip(environment.client_area.as_ref())
            .map(|(point, area)| {
                let origin = area.area().origin;
                finite_point(
                    self.id,
                    Point {
                        x: origin.x + point.x,
                        y: origin.y + point.y,
                    },
                )
            })
            .transpose()?;
        let screen = desktop
            .zip(environment.screen.as_ref())
            .map(|(point, screen)| {
                let origin = screen.area().origin;
                finite_point(
                    self.id,
                    Point {
                        x: point.x - origin.x,
                        y: point.y - origin.y,
                    },
                )
            })
            .transpose()?;
        Ok(NativePointerCoordinates {
            id: self.id,
            client,
            desktop,
            screen,
        })
    }
}

fn finite_point(id: Id, point: Point) -> Result<Point> {
    if !point.x.is_finite() || !point.y.is_finite() {
        return Err(Error::new(
            ErrorCode::InvalidRequest,
            "native pointer coordinates must remain finite",
        )
        .with_id(id));
    }
    Ok(point)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ClientWindowMetrics, EventKind, Handler, HostEnvironment, InputEvent, Metrics,
        ModifierState, NativeEventTransition, PhysicalPoint, PhysicalSize, Rect, ScreenAreaSource,
        ScreenMetrics, Size, open, testing::Runner,
    };

    fn environment(client_origin: Point, screen_origin: Point) -> HostEnvironment {
        let size = Size {
            width: 800.0,
            height: 600.0,
        };
        HostEnvironment {
            client_area: Some(
                ClientWindowMetrics::new(Rect {
                    origin: client_origin,
                    size,
                })
                .unwrap(),
            ),
            screen: Some(
                ScreenMetrics::new(
                    Rect {
                        origin: screen_origin,
                        size,
                    },
                    None,
                    ScreenAreaSource::OutputDevice,
                    None,
                    None,
                )
                .unwrap(),
            ),
            ..HostEnvironment::default()
        }
    }

    fn snapshot(id: Id, environment: HostEnvironment) -> WindowSnapshot {
        let metrics = Metrics::from_physical_size(
            id,
            PhysicalSize {
                width: 1600,
                height: 1200,
            },
            2.0,
        )
        .unwrap()
        .with_outer_geometry(Some(Point { x: 900.0, y: 700.0 }), None)
        .unwrap();
        WindowSnapshot::new("pointer", metrics).with_environment(environment)
    }

    fn pointer(id: Id, position: Point) -> PointerEvent {
        let mut state = snapshot(id, HostEnvironment::default());
        let event = NativeEventTransition::mouse_moved(
            id,
            position,
            PhysicalPoint { x: 21, y: 41 },
            None,
            ModifierState::default(),
        )
        .apply(&mut state)
        .unwrap();
        let EventKind::Input(InputEvent::Pointer(pointer)) = event else {
            panic!("mouse movement must produce pointer input");
        };
        pointer
    }

    #[test]
    fn fractional_client_coordinates_use_client_origin_and_negative_screen_origin() {
        let id = Id::from_u64(1);
        let state = snapshot(
            id,
            environment(
                Point {
                    x: -1800.0,
                    y: 40.0,
                },
                Point {
                    x: -1920.0,
                    y: -100.0,
                },
            ),
        );
        let observed = pointer(id, Point { x: 10.25, y: 20.75 })
            .native_coordinates(&state)
            .unwrap();
        assert_eq!(observed.id(), id);
        assert_eq!(observed.client(), Some(Point { x: 10.25, y: 20.75 }));
        assert_eq!(
            observed.desktop(),
            Some(Point {
                x: -1789.75,
                y: 60.75
            })
        );
        assert_eq!(
            observed.screen(),
            Some(Point {
                x: 130.25,
                y: 160.75
            })
        );
    }

    #[test]
    fn absent_logical_position_does_not_use_physical_position() {
        let id = Id::from_u64(1);
        let state = snapshot(id, environment(Point::default(), Point::default()));
        let mut event = pointer(id, Point::default());
        event.position = None;
        let observed = event.native_coordinates(&state).unwrap();
        assert_eq!(observed.client(), None);
        assert_eq!(observed.desktop(), None);
        assert_eq!(observed.screen(), None);
    }

    #[test]
    fn missing_client_origin_keeps_client_and_does_not_use_outer_geometry() {
        let id = Id::from_u64(1);
        let mut environment = environment(Point::default(), Point::default());
        environment.client_area = None;
        let state = snapshot(id, environment);
        let observed = pointer(id, Point { x: -2.5, y: 0.25 })
            .native_coordinates(&state)
            .unwrap();
        assert_eq!(observed.client(), Some(Point { x: -2.5, y: 0.25 }));
        assert_eq!(observed.desktop(), None);
        assert_eq!(observed.screen(), None);
    }

    #[test]
    fn missing_selected_screen_keeps_desktop_coordinates() {
        let id = Id::from_u64(1);
        let mut environment = environment(Point { x: -10.0, y: 8.0 }, Point::default());
        environment.screen = None;
        let observed = pointer(id, Point { x: 1.5, y: -2.25 })
            .native_coordinates(&snapshot(id, environment))
            .unwrap();
        assert_eq!(observed.desktop(), Some(Point { x: -8.5, y: 5.75 }));
        assert_eq!(observed.screen(), None);
    }

    #[test]
    fn mismatched_snapshot_is_rejected_even_without_a_position() {
        let id = Id::from_u64(1);
        let mut event = pointer(id, Point::default());
        event.position = None;
        let error = event
            .native_coordinates(&snapshot(Id::from_u64(2), HostEnvironment::default()))
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidRequest);
        assert_eq!(error.id, Some(id));
    }

    #[test]
    fn nonfinite_client_coordinates_are_rejected_without_an_origin() {
        let id = Id::from_u64(1);
        let state = snapshot(id, HostEnvironment::default());
        for position in [
            Point {
                x: f64::NAN,
                y: 0.0,
            },
            Point {
                x: 0.0,
                y: f64::INFINITY,
            },
            Point {
                x: f64::NEG_INFINITY,
                y: 0.0,
            },
        ] {
            let error = pointer(id, position)
                .native_coordinates(&state)
                .unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidRequest);
            assert_eq!(error.id, Some(id));
        }
    }

    #[test]
    fn finite_inputs_that_overflow_desktop_or_screen_are_rejected() {
        let id = Id::from_u64(1);
        for (origin, screen, position) in [
            (
                Point {
                    x: f64::MAX,
                    y: 0.0,
                },
                Point::default(),
                Point {
                    x: f64::MAX,
                    y: 0.0,
                },
            ),
            (
                Point {
                    x: f64::MAX,
                    y: 0.0,
                },
                Point {
                    x: -f64::MAX,
                    y: 0.0,
                },
                Point::default(),
            ),
        ] {
            let state = snapshot(id, environment(origin, screen));
            let error = pointer(id, position)
                .native_coordinates(&state)
                .unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidRequest);
        }
    }

    #[test]
    fn runner_callback_observes_coordinates_from_the_committed_environment() {
        #[derive(Default)]
        struct Observer {
            coordinates: Option<NativePointerCoordinates>,
        }
        impl Handler for Observer {
            fn input(&mut self, input: &mut crate::Input<'_>) -> Result<()> {
                if let InputEvent::Pointer(pointer) = input.event() {
                    self.coordinates = Some(pointer.native_coordinates(input.state())?);
                }
                Ok(())
            }

            fn closed(&mut self, closed: &mut crate::Closed<'_>) -> Result<()> {
                closed.exit();
                Ok(())
            }
        }
        let mut runner = Runner::new(Observer::default());
        runner.startup(vec![open("pointer").into()]).unwrap();
        runner.resume();
        let id = runner.host().window_id("pointer").unwrap();
        runner.transition(
            NativeEventTransition::mouse_moved(
                id,
                Point { x: 10.25, y: 20.75 },
                PhysicalPoint { x: 21, y: 41 },
                None,
                ModifierState::default(),
            )
            .with_environment(environment(
                Point {
                    x: -1800.0,
                    y: 40.0,
                },
                Point {
                    x: -1920.0,
                    y: -100.0,
                },
            )),
        );
        let observed = runner.handler().coordinates.unwrap();
        assert_eq!(observed.client(), Some(Point { x: 10.25, y: 20.75 }));
        assert_eq!(
            observed.screen(),
            Some(Point {
                x: 130.25,
                y: 160.75
            })
        );
        runner.destroy(id);
        assert!(matches!(runner.result(), Some(Ok(()))));
    }
}
