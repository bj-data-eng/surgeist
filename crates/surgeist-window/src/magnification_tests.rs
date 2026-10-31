use crate::{
    Access, EventKind, Handler, HostEnvironment, Id, InputEvent, Instant, MagnificationDelta,
    MagnificationEvent, MagnificationGesture, Metrics, ModifierState, NativeEventTransition,
    PhysicalPoint, PhysicalSize, Point, Result, ScaleFactor, Size, TouchPhase, ViewportMetrics,
    ViewportSizes, VisualViewportMetrics, WindowSnapshot, open, testing::Runner,
    winit_adapter::magnification_from_winit,
};
use winit::event::{DeviceId, TouchPhase as NativePhase, WindowEvent};

fn pinch(delta: f64, phase: NativePhase) -> WindowEvent {
    WindowEvent::PinchGesture {
        device_id: DeviceId::dummy(),
        delta,
        phase,
    }
}

fn smart() -> WindowEvent {
    WindowEvent::DoubleTapGesture {
        device_id: DeviceId::dummy(),
    }
}

fn modifiers() -> ModifierState {
    ModifierState {
        shift: true,
        control: false,
        alt: true,
        super_key: true,
    }
}

#[test]
fn magnification_delta_retains_finite_bits_and_rejects_nonfinite_values() {
    for value in [
        0.125,
        -0.375,
        0.0,
        -0.0,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
    ] {
        assert_eq!(
            MagnificationDelta::new(value).unwrap().get().to_bits(),
            value.to_bits()
        );
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(MagnificationDelta::new(value).is_none());
    }
}

#[test]
fn magnification_native_pinch_preserves_every_phase_fraction_zero_and_receipt_context() {
    let id = Id::from_u64(12);
    let timestamp = Instant::now();
    for (native_phase, expected_phase) in [
        (NativePhase::Started, TouchPhase::Started),
        (NativePhase::Moved, TouchPhase::Moved),
        (NativePhase::Ended, TouchPhase::Ended),
        (NativePhase::Cancelled, TouchPhase::Cancelled),
    ] {
        for value in [0.125, -0.375, 0.0, -0.0] {
            for position in [Some(Point { x: 12.25, y: -3.5 }), None] {
                let event = magnification_from_winit(
                    id,
                    &pinch(value, native_phase),
                    position,
                    modifiers(),
                    Some(timestamp),
                )
                .unwrap();
                let MagnificationGesture::Pinch {
                    delta: Some(delta),
                    phase,
                } = event.gesture
                else {
                    panic!("finite native pinch retains a known delta");
                };
                assert_eq!(delta.get().to_bits(), value.to_bits());
                assert_eq!(phase, expected_phase);
                assert_eq!(event.id, id);
                assert_eq!(InputEvent::Magnification(event.clone()).id(), id);
                assert_eq!(event.position, position);
                assert_eq!(event.modifiers, modifiers());
                assert_eq!(event.timestamp, Some(timestamp));
            }
        }
    }
}

#[test]
fn magnification_native_nonfinite_pinch_stays_unavailable_with_its_phase() {
    let id = Id::from_u64(13);
    let timestamp = Instant::now();
    for (native_phase, expected_phase) in [
        (NativePhase::Started, TouchPhase::Started),
        (NativePhase::Moved, TouchPhase::Moved),
        (NativePhase::Ended, TouchPhase::Ended),
        (NativePhase::Cancelled, TouchPhase::Cancelled),
    ] {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let event = magnification_from_winit(
                id,
                &pinch(value, native_phase),
                None,
                modifiers(),
                Some(timestamp),
            )
            .unwrap();
            assert_eq!(
                event.gesture,
                MagnificationGesture::Pinch {
                    delta: None,
                    phase: expected_phase
                }
            );
            assert_eq!(event.position, None);
            assert_eq!(event.modifiers, modifiers());
            assert_eq!(event.timestamp, Some(timestamp));
        }
    }
}

#[test]
fn magnification_native_smart_is_distinct_and_does_not_invent_pointer_or_time() {
    let id = Id::from_u64(14);
    let timestamp = Instant::now();
    for position in [Some(Point { x: -2.75, y: 4.125 }), None] {
        for timestamp in [Some(timestamp), None] {
            let event =
                magnification_from_winit(id, &smart(), position, modifiers(), timestamp).unwrap();
            assert_eq!(event.gesture, MagnificationGesture::Smart);
            assert_eq!(event.position, position);
            assert_eq!(event.modifiers, modifiers());
            assert_eq!(event.timestamp, timestamp);
            assert_eq!(InputEvent::Magnification(event).id(), id);
        }
    }
    assert!(
        magnification_from_winit(id, &WindowEvent::Focused(true), None, modifiers(), None)
            .is_none()
    );
}

#[derive(Default)]
struct Observer {
    inputs: Vec<InputEvent>,
    positions: Vec<Option<Point>>,
    modifiers: Vec<ModifierState>,
    input_states: Vec<WindowSnapshot>,
    snapshot: Option<WindowSnapshot>,
    generic_events: Vec<EventKind>,
    resize_count: usize,
}

impl Handler for Observer {
    fn ready(&mut self, ready: &mut crate::Ready<'_>) -> Result<()> {
        self.snapshot = Some(ready.state().clone());
        Ok(())
    }

    fn resize(&mut self, resize: &mut crate::Resize<'_>) -> Result<()> {
        self.snapshot = Some(resize.state().clone());
        self.resize_count += 1;
        Ok(())
    }

    fn input(&mut self, input: &mut crate::Input<'_>) -> Result<()> {
        self.inputs.push(input.event().clone());
        self.positions.push(input.position());
        self.modifiers.push(input.modifiers());
        self.input_states.push(input.state().clone());
        Ok(())
    }

    fn event(&mut self, event: &mut crate::Event<'_>) -> Result<()> {
        self.generic_events.push(event.event().clone());
        Ok(())
    }
}

fn scaled_environment() -> HostEnvironment {
    let layout = Size {
        width: 400.0,
        height: 300.0,
    };
    HostEnvironment {
        native_scale: Some(ScaleFactor::new(2.0).unwrap()),
        page_zoom: Some(ScaleFactor::new(1.25).unwrap()),
        viewport: Some(
            ViewportMetrics::new(layout, ViewportSizes::new(layout, layout, layout).unwrap())
                .unwrap()
                .with_visual(
                    VisualViewportMetrics::new(
                        Point { x: 3.25, y: 4.5 },
                        Point { x: 10.0, y: 20.0 },
                        Size {
                            width: 120.0,
                            height: 90.0,
                        },
                        ScaleFactor::new(3.0).unwrap(),
                    )
                    .unwrap(),
                ),
        ),
        ..HostEnvironment::default()
    }
}

#[test]
fn magnification_runner_preserves_mixed_source_order_and_all_observed_state() {
    let mut runner = Runner::new(Observer::default());
    runner.startup(vec![open("magnification").into()]).unwrap();
    runner.resume();
    let id = runner.host().window_id("magnification").unwrap();
    let metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 800,
            height: 600,
        },
        2.0,
    )
    .unwrap();
    runner.transition(
        NativeEventTransition::scale_factor_changed(metrics).with_environment(scaled_environment()),
    );
    let before = runner.handler().snapshot.clone().unwrap();
    let before_commands = runner.host().commands().to_vec();
    let before_events = runner.host().events().len();
    let before_generic = runner.handler().generic_events.clone();
    let before_resizes = runner.handler().resize_count;
    let position = Point { x: 10.25, y: 20.75 };
    runner.transition(NativeEventTransition::mouse_moved(
        id,
        position,
        PhysicalPoint { x: 21, y: 42 },
        None,
        modifiers(),
    ));

    let timestamp = Instant::now();
    let native_events = [
        pinch(0.0, NativePhase::Started),
        pinch(0.125, NativePhase::Moved),
        pinch(-0.0, NativePhase::Ended),
        smart(),
        pinch(f64::NAN, NativePhase::Started),
        pinch(f64::NEG_INFINITY, NativePhase::Cancelled),
    ];
    let mut expected_magnification = Vec::new();
    for native in native_events {
        let event =
            magnification_from_winit(id, &native, Some(position), modifiers(), Some(timestamp))
                .unwrap();
        expected_magnification.push(InputEvent::Magnification(event.clone()));
        runner.transition(NativeEventTransition::magnification(event));
    }

    assert!(
        matches!(&runner.handler().inputs[0], InputEvent::Pointer(pointer)
        if pointer.id == id && pointer.position == Some(position))
    );
    assert_eq!(&runner.handler().inputs[1..], expected_magnification);
    assert_eq!(runner.handler().positions, vec![Some(position); 7]);
    assert_eq!(runner.handler().modifiers, vec![modifiers(); 7]);
    assert_eq!(runner.handler().input_states, vec![before.clone(); 7]);
    assert_eq!(runner.handler().generic_events, before_generic);
    assert_eq!(runner.handler().resize_count, before_resizes);
    assert_eq!(runner.host().commands(), before_commands);
    assert!(
        runner.host().events()[before_events..]
            .iter()
            .all(|event| matches!(event, crate::testing::Event::Input(_)))
    );
    let observed = runner.host().registry().get(id).unwrap();
    assert_eq!(&observed.metrics(), before.metrics());
    assert_eq!(observed.environment(), before.environment());
    assert!(runner.result().is_none());
}

#[test]
fn magnification_transition_rejects_wrong_target_without_mutating_snapshot() {
    let mut snapshot = WindowSnapshot::new(
        "target",
        Metrics::from_physical_size(
            Id::from_u64(1),
            PhysicalSize {
                width: 800,
                height: 600,
            },
            2.0,
        )
        .unwrap(),
    )
    .with_environment(scaled_environment());
    let before = snapshot.clone();
    let event: MagnificationEvent = magnification_from_winit(
        Id::from_u64(2),
        &pinch(0.125, NativePhase::Moved),
        None,
        modifiers(),
        Some(Instant::now()),
    )
    .unwrap();
    let error = NativeEventTransition::magnification(event)
        .apply(&mut snapshot)
        .unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::InvalidRequest);
    assert_eq!(snapshot, before);
}
