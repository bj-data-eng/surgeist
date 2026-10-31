use crate::*;

fn extent(width: f64, height: f64) -> Size {
    Size { width, height }
}
fn scale(value: f64) -> ScaleFactor {
    ScaleFactor::new(value).unwrap()
}
fn pointer(accuracy: PointerAccuracy, hover: bool) -> PointerCapability {
    PointerCapability { accuracy, hover }
}

#[test]
fn page_zoom_changes_resolution_but_visual_magnification_keeps_layout_space() {
    let layout = extent(800.0, 600.0);
    let environment = HostEnvironment {
        native_scale: Some(scale(2.0)),
        page_zoom: Some(scale(1.5)),
        output: OutputDevice::Present(OutputResolution::Raster(scale(2.0))),
        viewport: Some(
            ViewportMetrics::new(layout, ViewportSizes::fixed(layout).unwrap())
                .unwrap()
                .with_visual(
                    VisualViewportMetrics::new(
                        Point { x: 10.0, y: 20.0 },
                        Point { x: 50.0, y: 80.0 },
                        extent(400.0, 300.0),
                        scale(2.0),
                    )
                    .unwrap(),
                ),
        ),
        ..HostEnvironment::default()
    };
    assert_eq!(
        environment.resolution().unwrap(),
        Some(OutputResolution::Raster(scale(3.0)))
    );
    let viewport = environment.viewport.unwrap();
    assert_eq!(viewport.layout(), layout);
    assert_eq!(viewport.visual().unwrap().scale().get(), 2.0);
    assert_eq!(
        viewport.visual().unwrap().page_position(),
        Point { x: 50.0, y: 80.0 }
    );
}

#[test]
fn absent_device_and_unavailable_device_have_different_resolution_results() {
    let mut environment = HostEnvironment::default();
    assert_eq!(environment.resolution().unwrap(), None);
    environment.output = OutputDevice::Absent;
    assert_eq!(
        environment.resolution().unwrap(),
        Some(OutputResolution::Raster(ScaleFactor::ONE))
    );
    environment.output = OutputDevice::Present(OutputResolution::Infinite);
    environment.page_zoom = Some(scale(100.0));
    assert_eq!(
        environment.resolution().unwrap(),
        Some(OutputResolution::Infinite)
    );
}

#[test]
fn scales_reject_nonpositive_nonfinite_and_overflowed_density() {
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            ScaleFactor::new(value).unwrap_err().code,
            ErrorCode::InvalidRequest
        );
    }
    assert!(
        OutputResolution::Raster(scale(f64::MAX))
            .at_page_zoom(scale(2.0))
            .is_err()
    );
}

#[test]
fn scroll_driven_dynamic_viewport_changes_preserve_stable_extrema() {
    let sizes = ViewportSizes::new(
        extent(400.0, 800.0),
        extent(400.0, 600.0),
        extent(400.0, 700.0),
    )
    .unwrap();
    let changed = sizes.with_dynamic(extent(400.0, 800.0)).unwrap();
    assert_eq!(changed.large(), extent(400.0, 800.0));
    assert_eq!(changed.small(), extent(400.0, 600.0));
    assert_eq!(changed.dynamic(), extent(400.0, 800.0));
    assert!(sizes.with_dynamic(extent(400.0, 801.0)).is_err());
    assert!(ViewportSizes::fixed(extent(-1.0, 0.0)).is_err());
    assert_eq!(
        ViewportSizes::fixed(extent(0.0, 0.0)).unwrap().small(),
        extent(0.0, 0.0)
    );
}

#[test]
fn viewport_keeps_scrollbars_paged_area_and_initial_block_independent() {
    let viewport = ViewportMetrics::new(
        extent(800.0, 600.0),
        ViewportSizes::fixed(extent(800.0, 600.0)).unwrap(),
    )
    .unwrap()
    .with_layout_observations(
        Some(extent(785.0, 600.0)),
        Some(extent(700.0, 900.0)),
        Some(Point { x: -40.0, y: 25.0 }),
        Some(Insets {
            right: 15.0,
            ..Insets::default()
        }),
    )
    .unwrap();
    assert_eq!(viewport.layout(), extent(800.0, 600.0));
    assert_eq!(
        viewport.initial_containing_block(),
        Some(extent(785.0, 600.0))
    );
    assert_eq!(viewport.paged_area(), Some(extent(700.0, 900.0)));
    assert_eq!(viewport.scroll(), Some(Point { x: -40.0, y: 25.0 }));
    assert_eq!(viewport.root_scrollbars().unwrap().right, 15.0);
    assert!(
        viewport
            .with_layout_observations(
                None,
                None,
                None,
                Some(Insets {
                    top: -1.0,
                    ..Insets::default()
                })
            )
            .is_err()
    );
}

#[test]
fn mixed_pointer_union_preserves_both_accuracies_and_accessibility_primary() {
    let input = InputCapabilities::new(
        pointer(PointerAccuracy::Coarse, false),
        [
            pointer(PointerAccuracy::Coarse, false),
            pointer(PointerAccuracy::Fine, true),
        ],
    )
    .unwrap();
    assert_eq!(input.primary(), pointer(PointerAccuracy::Coarse, false));
    assert!(input.any_coarse());
    assert!(input.any_fine());
    assert!(input.any_hover());
    assert!(!input.no_pointer());
    let empty = InputCapabilities::new(pointer(PointerAccuracy::None, false), []).unwrap();
    assert!(empty.no_pointer());
    assert!(!empty.any_hover());
    let none = InputCapabilities::new(
        pointer(PointerAccuracy::None, false),
        [pointer(PointerAccuracy::None, false)],
    )
    .unwrap();
    assert!(none.no_pointer());
    assert!(InputCapabilities::new(pointer(PointerAccuracy::None, true), []).is_err());
}

#[test]
fn screen_exposure_preserves_selected_coordinates_and_color_lower_bound() {
    let area = Rect {
        origin: Point { x: -1920.0, y: 0.0 },
        size: extent(1920.0, 1080.0),
    };
    let screen =
        ScreenMetrics::new(area, None, ScreenAreaSource::OutputDevice, None, None).unwrap();
    assert_eq!(screen.area(), area);
    assert_eq!(screen.available_area(), None);
    assert_eq!(screen.color_depth(), 24);
    let quantized = ScreenMetrics::new(
        Rect {
            origin: Point::default(),
            size: extent(1000.0, 1000.0),
        },
        Some(area),
        ScreenAreaSource::PrivacyAdjusted,
        Some(24),
        Some(10),
    )
    .unwrap();
    assert_eq!(quantized.color_depth(), 30);
    assert_eq!(quantized.source(), ScreenAreaSource::PrivacyAdjusted);
    assert!(
        ScreenMetrics::new(
            Rect {
                origin: Point::default(),
                size: extent(f64::NAN, 1.0)
            },
            None,
            ScreenAreaSource::Viewport,
            None,
            None
        )
        .is_err()
    );
}

#[test]
fn environment_and_metrics_commit_together_and_reject_foreign_identity() {
    let id = Id::from_u64(5);
    let metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 800,
            height: 600,
        },
        1.0,
    )
    .unwrap();
    let mut snapshot = WindowSnapshot::new("host", metrics);
    let environment = HostEnvironment {
        page_zoom: Some(scale(1.25)),
        preferences: UserPreferences {
            contrast: Some(ContrastPreference::Custom),
            reduced_motion: Some(ReductionPreference::Reduce),
            ..UserPreferences::default()
        },
        ..HostEnvironment::default()
    };
    let resized = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 1200,
            height: 900,
        },
        2.0,
    )
    .unwrap();
    let event = NativeEventTransition::resized(resized.clone())
        .with_environment(environment.clone())
        .apply(&mut snapshot)
        .unwrap();
    assert_eq!(event, EventKind::Resized(resized.clone()));
    assert_eq!(snapshot.metrics(), &resized);
    assert_eq!(snapshot.environment(), &environment);
    let committed = snapshot.clone();
    assert!(
        NativeEventTransition::environment_changed(Id::from_u64(6), HostEnvironment::default())
            .apply(&mut snapshot)
            .is_err()
    );
    assert_eq!(snapshot, committed);
    let event = NativeEventTransition::environment_changed(id, HostEnvironment::default())
        .apply(&mut snapshot)
        .unwrap();
    assert_eq!(
        event,
        EventKind::EnvironmentChanged {
            id,
            environment: HostEnvironment::default()
        }
    );
    let replacement = HostEnvironment {
        navigation: Some(NavigationControls::Back),
        ..HostEnvironment::default()
    };
    let event = NativeEventTransition::environment_changed(id, HostEnvironment::default())
        .with_environment(replacement.clone())
        .apply(&mut snapshot)
        .unwrap();
    assert_eq!(snapshot.environment(), &replacement);
    assert_eq!(
        event,
        EventKind::EnvironmentChanged {
            id,
            environment: replacement
        }
    );
}

#[test]
fn rejected_attached_environment_does_not_leak_into_snapshot() {
    let id = Id::from_u64(1);
    let metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 1,
            height: 1,
        },
        1.0,
    )
    .unwrap();
    let mut snapshot = WindowSnapshot::new("atomic", metrics);
    let before = snapshot.clone();
    let environment = HostEnvironment {
        display_mode: Some(DisplayMode::PictureInPicture),
        blending: Some(EnvironmentBlending::Subtractive),
        ..HostEnvironment::default()
    };
    assert!(
        NativeEventTransition::moved(
            id,
            Point {
                x: f64::NAN,
                y: 0.0
            }
        )
        .with_environment(environment)
        .apply(&mut snapshot)
        .is_err()
    );
    assert_eq!(snapshot, before);
}

#[test]
fn resize_callback_observes_exact_metrics_basis_and_committed_environment() {
    #[derive(Default)]
    struct Observer {
        observations: Vec<(Size, f64, Size, f64)>,
    }
    impl Handler for Observer {
        fn resize(&mut self, window: &mut Resize<'_>) -> Result<()> {
            let environment = window.state().environment();
            self.observations.push((
                window.size(),
                window.scale(),
                environment.viewport.as_ref().unwrap().layout(),
                window.access()?.environment().native_scale.unwrap().get(),
            ));
            Ok(())
        }
    }
    let mut runner = testing::Runner::new(Observer::default());
    runner.startup(vec![open("coherent").into()]).unwrap();
    runner.resume();
    let id = runner.host().window_id("coherent").unwrap();
    let metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 1800,
            height: 1200,
        },
        3.0,
    )
    .unwrap();
    let size = extent(600.0, 400.0);
    let environment = HostEnvironment {
        native_scale: Some(scale(3.0)),
        viewport: Some(ViewportMetrics::new(size, ViewportSizes::fixed(size).unwrap()).unwrap()),
        ..HostEnvironment::default()
    };
    runner.transition(
        NativeEventTransition::scale_factor_changed(metrics).with_environment(environment),
    );
    assert_eq!(runner.handler().observations, [(size, 3.0, size, 3.0)]);
    assert!(runner.result().is_none());
}
