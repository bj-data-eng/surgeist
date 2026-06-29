use std::{collections::HashMap, env};

use keyboard_types::Code;
use surgeist::render;
use surgeist::text;
use surgeist::window;
use surgeist_dev::{
    DevState, Scenario, WindowFacts, build_scene, retained_action_at_point, retained_node_at_point,
    scenario_at_point,
};
use window::{Context, Handler, InputEvent, KeyState, PointerButton, PointerPhase, Result};

struct DevWindow {
    mode: HarnessMode,
    renderer: Option<render::Renderer>,
    text_system: Option<text::System>,
    state: DevState,
    windows: HashMap<window::Id, LiveWindow>,
    opened: bool,
}

struct LiveWindow {
    surface: Option<render::Surface>,
    metrics: window::Metrics,
    facts: WindowFacts,
    resizing: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HarnessMode {
    Full,
    Render,
    EmptyRender,
    CpuFull,
    CpuRender,
    CpuEmptyRender,
    TextLayout,
    TextSystem,
    Surface,
    Renderer,
    Window,
}

fn render_point(x: f64, y: f64) -> render::Point {
    render::Point::try_new(x, y).expect("dev harness render point should be valid")
}

fn render_size(width: f64, height: f64) -> render::Size {
    render::Size::try_new(width, height).expect("dev harness render size should be valid")
}

fn render_rect(x: f64, y: f64, width: f64, height: f64) -> render::Rect {
    render::Rect::try_new(x, y, width, height).expect("dev harness render rect should be valid")
}

fn render_color(r: f32, g: f32, b: f32, a: f32) -> render::Color {
    render::Color::try_rgba(r, g, b, a).expect("dev harness color should be valid")
}

fn render_stroke(width: f64) -> render::Stroke {
    render::Stroke::try_new(width).expect("dev harness render stroke should be valid")
}

fn render_circle(center: render::Point, radius: f64) -> render::Shape {
    render::Shape::try_circle(center, radius).expect("dev harness render circle should be valid")
}

impl HarnessMode {
    fn from_env() -> Self {
        match env::var("SURGEIST_HARNESS_MODE") {
            Ok(value) => Self::parse(&value).unwrap_or_else(|| {
                eprintln!("unknown SURGEIST_HARNESS_MODE={value:?}; using full");
                Self::Full
            }),
            Err(_) => Self::Full,
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "full" => Some(Self::Full),
            "render" | "primitive" | "primitives" => Some(Self::Render),
            "empty-render" | "empty" | "present" => Some(Self::EmptyRender),
            "cpu-full" | "vello-cpu-full" => Some(Self::CpuFull),
            "cpu-render" | "vello-cpu-render" => Some(Self::CpuRender),
            "cpu-empty-render" | "cpu-empty" | "vello-cpu-empty-render" => {
                Some(Self::CpuEmptyRender)
            }
            "text-layout" | "layout-text" | "layout" => Some(Self::TextLayout),
            "text-system" | "text" | "font" | "fonts" => Some(Self::TextSystem),
            "surface" => Some(Self::Surface),
            "renderer" | "device" => Some(Self::Renderer),
            "window" | "native" => Some(Self::Window),
            _ => None,
        }
    }

    const fn needs_renderer(self) -> bool {
        !matches!(self, Self::Window | Self::TextLayout | Self::TextSystem)
    }

    const fn needs_text(self) -> bool {
        matches!(
            self,
            Self::Full | Self::CpuFull | Self::TextLayout | Self::TextSystem
        )
    }

    const fn needs_surface(self) -> bool {
        matches!(
            self,
            Self::Full
                | Self::Render
                | Self::EmptyRender
                | Self::CpuFull
                | Self::CpuRender
                | Self::CpuEmptyRender
                | Self::Surface
        )
    }

    const fn uses_vello_cpu(self) -> bool {
        matches!(self, Self::CpuFull | Self::CpuRender | Self::CpuEmptyRender)
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Render => "render",
            Self::EmptyRender => "empty-render",
            Self::CpuFull => "cpu-full",
            Self::CpuRender => "cpu-render",
            Self::CpuEmptyRender => "cpu-empty-render",
            Self::TextLayout => "text-layout",
            Self::TextSystem => "text-system",
            Self::Surface => "surface",
            Self::Renderer => "renderer",
            Self::Window => "window",
        }
    }
}

impl DevWindow {
    fn new(mode: HarnessMode, renderer: Option<render::Renderer>) -> Self {
        Self {
            mode,
            renderer,
            text_system: mode.needs_text().then(text::System::default),
            state: DevState::default(),
            windows: HashMap::new(),
            opened: false,
        }
    }

    fn open_windows(&mut self, cx: &mut Context<'_>) {
        cx.open(
            window::open("surgeist-dev-main")
                .title(format!("Surgeist Dev ({})", self.mode.label()))
                .size(window::size(1240.0, 760.0)),
        );
        self.opened = true;
    }

    fn attach_surface(&mut self, handle: Option<window::Handle>, state: &window::WindowSnapshot) {
        if !self.mode.needs_surface() {
            let facts = facts_from_state(state);
            self.windows.insert(
                state.id(),
                LiveWindow {
                    surface: None,
                    metrics: state.metrics().clone(),
                    facts,
                    resizing: false,
                },
            );
            return;
        }

        let handle = handle.expect("created surface window should have a native handle");
        let facts = facts_from_state(state);
        let surface = self
            .renderer
            .as_mut()
            .expect("surface modes should initialize renderer")
            .create_surface(
                render::Attachment::from_window(handle),
                render::SurfaceOptions {
                    size: facts.logical_size,
                    scale: facts.scale_factor,
                    ..render::SurfaceOptions::default()
                },
            )
            .expect("native render surface should attach");
        self.windows.insert(
            state.id(),
            LiveWindow {
                surface: Some(surface),
                metrics: state.metrics().clone(),
                facts,
                resizing: false,
            },
        );
    }

    fn resize_surface(&mut self, metrics: window::Metrics) {
        if let Some(live) = self.windows.get_mut(&metrics.id) {
            if !live.resizing
                && let Some(surface) = &mut live.surface
            {
                self.renderer
                    .as_mut()
                    .expect("surface modes should initialize renderer")
                    .set_surface_resizing(surface, true)
                    .expect("native render surface should enter resize state");
                live.resizing = true;
            }
            live.metrics = metrics.clone();
            live.facts.logical_size =
                render_size(metrics.logical_size.width, metrics.logical_size.height);
            live.facts.scale_factor = metrics.scale_factor;
            if let Some(surface) = &mut live.surface {
                surface
                    .resize(live.facts.logical_size, metrics.scale_factor)
                    .expect("native render surface should resize");
            }
        }
    }

    fn log_input(&mut self, input: &InputEvent) -> bool {
        if !should_log_input(input) {
            return false;
        }
        self.state.push_event(short_input(input));
        true
    }

    fn draw_all(&self, cx: &mut Context<'_>) {
        for id in self.windows.keys().copied() {
            cx.draw(id);
        }
    }

    fn draw_for_log_change(&self, cx: &mut Context<'_>) {
        if self.state.active_scenario() == Scenario::WindowState {
            self.draw_all(cx);
        }
    }
}

impl Handler for DevWindow {
    fn resume(&mut self, cx: &mut Context<'_>) -> Result<()> {
        if !self.opened {
            self.open_windows(cx);
            return Ok(());
        }

        for (id, live) in &mut self.windows {
            let Some(surface) = &mut live.surface else {
                continue;
            };
            let handle = cx.handle(*id)?;
            self.renderer
                .as_mut()
                .expect("surface modes should initialize renderer")
                .resume_surface(surface, render::Attachment::from_window(handle))
                .expect("native render surface should resume");
        }
        Ok(())
    }

    fn suspend(&mut self, _cx: &mut Context<'_>) -> Result<()> {
        for live in self.windows.values_mut() {
            if let Some(surface) = &mut live.surface {
                surface
                    .suspend()
                    .expect("native render surface should suspend");
            }
        }
        Ok(())
    }

    fn ready(&mut self, win: &mut window::Ready<'_>) -> Result<()> {
        self.state
            .push_event(format!("created {}", win.state().title()));
        let handle = self
            .mode
            .needs_surface()
            .then(|| win.handle())
            .transpose()?;
        self.attach_surface(handle, win.state());
        win.draw();
        Ok(())
    }

    fn resize(&mut self, win: &mut window::Resize<'_>) -> Result<()> {
        let id = win.id();
        let metrics = win.metrics().clone();
        self.state.push_event(format!(
            "resize {} {:.0}x{:.0}",
            id.as_u64(),
            metrics.logical_size.width,
            metrics.logical_size.height
        ));
        self.resize_surface(metrics);
        win.draw();
        Ok(())
    }

    fn input(&mut self, input: &mut window::Input<'_>) -> Result<()> {
        let logged = self.log_input(input.event());
        match input.event().clone() {
            InputEvent::Key(key) if key.state == KeyState::Pressed => match key.physical_key {
                Code::Digit1 => {
                    self.state.select(0);
                    self.draw_all(input.context_mut());
                }
                Code::Digit2 => {
                    self.state.select(1);
                    self.draw_all(input.context_mut());
                }
                Code::Digit3 => {
                    self.state.select(2);
                    self.draw_all(input.context_mut());
                }
                Code::Digit4 => {
                    self.state.select(3);
                    self.draw_all(input.context_mut());
                }
                Code::Digit5 => {
                    self.state.select(4);
                    self.draw_all(input.context_mut());
                }
                Code::Digit6 => {
                    self.state.select(5);
                    self.draw_all(input.context_mut());
                }
                Code::Digit7 => {
                    self.state.select(6);
                    self.draw_all(input.context_mut());
                }
                Code::ArrowRight | Code::ArrowDown => {
                    self.state.next();
                    self.draw_all(input.context_mut());
                }
                Code::ArrowLeft | Code::ArrowUp => {
                    self.state.previous();
                    self.draw_all(input.context_mut());
                }
                Code::Escape => {
                    input.exit();
                }
                _ => {
                    if logged {
                        self.draw_for_log_change(input.context_mut());
                    }
                }
            },
            InputEvent::Pointer(pointer)
                if pointer.phase == PointerPhase::Pressed
                    && pointer.button == Some(PointerButton::Primary) =>
            {
                if let Some(position) = pointer.position
                    && let Some(index) = self.scenario_at_pointer(pointer.id, position)
                {
                    self.state.select(index);
                    self.draw_all(input.context_mut());
                    return Ok(());
                }
                if self.state.active_scenario() == Scenario::RetainedModel
                    && let Some(position) = pointer.position
                {
                    if let Some(action) = retained_action_at_point(position.x, position.y) {
                        self.state.retained.apply(action);
                        self.draw_all(input.context_mut());
                        return Ok(());
                    }
                    if let Some(id) = retained_node_at_point(&self.state, position.x, position.y) {
                        self.state.retained.select(id);
                        self.draw_all(input.context_mut());
                        return Ok(());
                    }
                }
                if logged {
                    self.draw_for_log_change(input.context_mut());
                }
            }
            _ if logged => {
                self.draw_for_log_change(input.context_mut());
            }
            _ => {}
        }
        Ok(())
    }

    fn close(&mut self, close: &mut window::Close<'_>) -> Result<()> {
        self.state
            .push_event(format!("close requested {}", close.id().as_u64()));
        close.close();
        Ok(())
    }

    fn closed(&mut self, closed: &mut window::Closed<'_>) -> Result<()> {
        let id = closed.id();
        self.state.push_event(format!("destroyed {}", id.as_u64()));
        self.windows.remove(&id);
        if self.windows.is_empty() {
            closed.exit();
        } else {
            self.draw_all(closed.context_mut());
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut window::Frame<'_>) -> Result<()> {
        let id = frame.id();
        let Some(live) = self.windows.get_mut(&id) else {
            return Ok(());
        };

        match self.mode {
            HarnessMode::Full | HarnessMode::CpuFull => {
                let scene = build_scene(
                    self.text_system
                        .as_mut()
                        .expect("full mode should initialize text system"),
                    &self.state,
                    live.facts,
                );
                let Some(surface) = &mut live.surface else {
                    return Ok(());
                };
                self.renderer
                    .as_mut()
                    .expect("render modes should initialize renderer")
                    .render(surface, &scene, render::Parameters::default())
                    .expect("dev scene should render");
            }
            HarnessMode::Render
            | HarnessMode::EmptyRender
            | HarnessMode::CpuRender
            | HarnessMode::CpuEmptyRender => {
                let scene = if matches!(self.mode, HarnessMode::Render | HarnessMode::CpuRender) {
                    build_render_probe_scene(live.facts)
                } else {
                    render::Scene::new()
                };
                let Some(surface) = &mut live.surface else {
                    return Ok(());
                };
                self.renderer
                    .as_mut()
                    .expect("render modes should initialize renderer")
                    .render(surface, &scene, render::Parameters::default())
                    .expect("dev scene should render");
            }
            HarnessMode::TextLayout => {
                let _scene = build_scene(
                    self.text_system
                        .as_mut()
                        .expect("text layout mode should initialize text system"),
                    &self.state,
                    live.facts,
                );
            }
            HarnessMode::TextSystem
            | HarnessMode::Surface
            | HarnessMode::Renderer
            | HarnessMode::Window => {}
        }

        if live.resizing {
            if let Some(surface) = &mut live.surface {
                self.renderer
                    .as_mut()
                    .expect("surface modes should initialize renderer")
                    .set_surface_resizing(surface, false)
                    .expect("native render surface should leave resize state");
            }
            live.resizing = false;
        }
        Ok(())
    }
}

impl DevWindow {
    fn scenario_at_pointer(&self, _id: window::Id, position: window::Point) -> Option<usize> {
        scenario_at_point(position.x, position.y)
    }
}

fn should_log_input(input: &InputEvent) -> bool {
    !matches!(
        input,
        InputEvent::Pointer(pointer)
            if matches!(
                pointer.phase,
                PointerPhase::Entered | PointerPhase::Moved | PointerPhase::Left
            ) && pointer.button.is_none()
    )
}

fn facts_from_state(state: &window::WindowSnapshot) -> WindowFacts {
    let metrics = state.metrics();
    WindowFacts {
        logical_size: render_size(metrics.logical_size.width, metrics.logical_size.height),
        scale_factor: metrics.scale_factor,
        focused: state.is_focused(),
    }
}

fn build_render_probe_scene(facts: WindowFacts) -> render::Scene {
    let mut scene = render::Scene::new();
    let panel = render_rect(
        40.0,
        40.0,
        (facts.logical_size.width() - 80.0).max(120.0),
        (facts.logical_size.height() - 80.0).max(120.0),
    );
    scene
        .fill(panel, render_color(0.96, 0.97, 1.0, 1.0))
        .stroke(
            panel,
            render_stroke(2.0),
            render_color(0.15, 0.33, 0.64, 1.0),
        )
        .fill(
            render_circle(render_point(panel.x() + 92.0, panel.y() + 92.0), 54.0),
            render_color(0.96, 0.72, 0.24, 0.78),
        )
        .fill(
            render_circle(render_point(panel.x() + 148.0, panel.y() + 92.0), 54.0),
            render_color(0.25, 0.58, 0.96, 0.78),
        );
    scene
}

fn short_input(input: &InputEvent) -> String {
    match input {
        InputEvent::Pointer(pointer) => format!("pointer {:?}", pointer.phase),
        InputEvent::Wheel(wheel) => format!("wheel {:?}", wheel.delta),
        InputEvent::Key(key) => format!("key {:?} {:?}", key.physical_key, key.state),
        InputEvent::Modifiers { modifiers, .. } => format!("modifiers {modifiers:?}"),
        InputEvent::Ime(ime) => format!("ime {ime:?}"),
        InputEvent::StandardKeyBinding(binding) => format!("binding {}", binding.binding),
    }
}

fn main() -> Result<()> {
    let mode = HarnessMode::from_env();
    let renderer = mode.needs_renderer().then(|| {
        pollster::block_on(render::Renderer::new(render::Options {
            use_cpu: mode.uses_vello_cpu(),
            ..render::Options::default()
        }))
        .expect("renderer should initialize")
    });
    window::Loop::new(DevWindow::new(mode, renderer)).run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_mode_parses_memory_probe_modes() {
        assert_eq!(HarnessMode::parse("full"), Some(HarnessMode::Full));
        assert_eq!(HarnessMode::parse("render"), Some(HarnessMode::Render));
        assert_eq!(
            HarnessMode::parse("empty-render"),
            Some(HarnessMode::EmptyRender)
        );
        assert_eq!(HarnessMode::parse("cpu-full"), Some(HarnessMode::CpuFull));
        assert_eq!(
            HarnessMode::parse("cpu-render"),
            Some(HarnessMode::CpuRender)
        );
        assert_eq!(
            HarnessMode::parse("cpu-empty-render"),
            Some(HarnessMode::CpuEmptyRender)
        );
        assert_eq!(
            HarnessMode::parse("text-layout"),
            Some(HarnessMode::TextLayout)
        );
        assert_eq!(
            HarnessMode::parse("text-system"),
            Some(HarnessMode::TextSystem)
        );
        assert_eq!(HarnessMode::parse("surface"), Some(HarnessMode::Surface));
        assert_eq!(HarnessMode::parse("renderer"), Some(HarnessMode::Renderer));
        assert_eq!(HarnessMode::parse("window"), Some(HarnessMode::Window));
        assert_eq!(HarnessMode::parse("device"), Some(HarnessMode::Renderer));
        assert_eq!(HarnessMode::parse("bogus"), None);
    }

    #[test]
    fn pointer_motion_does_not_dirty_event_log() {
        assert!(!should_log_input(&InputEvent::Pointer(
            window::PointerEvent {
                id: window::Id::from_u64(1),
                phase: PointerPhase::Moved,
                kind: window::PointerKind::Mouse,
                pointer_id: None,
                position: Some(window::Point { x: 1.0, y: 2.0 }),
                physical_position: None,
                delta: None,
                button: None,
                modifiers: window::ModifierState::default(),
                device: window::PointerDeviceData::default(),
                timestamp: None,
            }
        )));
    }
}
