use surgeist::render;
use surgeist::retained;
use surgeist::shape;
use surgeist::text;

pub const SCENARIO_COUNT: usize = 7;
const TAB_X: f64 = 42.0;
const TAB_Y: f64 = 130.0;
const TAB_WIDTH: f64 = 136.0;
const TAB_HEIGHT: f64 = 34.0;
const TAB_STEP: f64 = 150.0;
const RETAINED_ACTION_X: f64 = 76.0;
const RETAINED_ACTION_Y: f64 = 214.0;
const RETAINED_ACTION_WIDTH: f64 = 126.0;
const RETAINED_ACTION_HEIGHT: f64 = 30.0;
const RETAINED_ACTION_STEP: f64 = 138.0;
const RETAINED_TREE_X: f64 = 76.0;
const RETAINED_TREE_Y: f64 = 286.0;
const RETAINED_ROW_HEIGHT: f64 = 24.0;
const RETAINED_VISIBLE_ROWS: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scenario {
    TextBasics,
    BidiSelection,
    InlineBoxes,
    RenderPrimitives,
    ShapeGeometry,
    RetainedModel,
    WindowState,
}

impl Scenario {
    pub const ALL: [Self; SCENARIO_COUNT] = [
        Self::TextBasics,
        Self::BidiSelection,
        Self::InlineBoxes,
        Self::RenderPrimitives,
        Self::ShapeGeometry,
        Self::RetainedModel,
        Self::WindowState,
    ];

    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::TextBasics => "Text basics",
            Self::BidiSelection => "Bidi, cursor, selection",
            Self::InlineBoxes => "Inline boxes",
            Self::RenderPrimitives => "Render primitives",
            Self::ShapeGeometry => "Shape geometry",
            Self::RetainedModel => "Retained model",
            Self::WindowState => "Window and input state",
        }
    }

    #[must_use]
    pub const fn from_index(index: usize) -> Self {
        Self::ALL[index % SCENARIO_COUNT]
    }
}

#[derive(Debug, Default)]
pub struct DevState {
    pub active: usize,
    pub event_log: Vec<String>,
    pub retained: RetainedHarness,
}

impl DevState {
    #[must_use]
    pub fn active_scenario(&self) -> Scenario {
        Scenario::from_index(self.active)
    }

    pub fn next(&mut self) {
        self.active = (self.active + 1) % SCENARIO_COUNT;
    }

    pub fn previous(&mut self) {
        self.active = (self.active + SCENARIO_COUNT - 1) % SCENARIO_COUNT;
    }

    pub fn select(&mut self, index: usize) {
        self.active = index.min(SCENARIO_COUNT - 1);
    }

    pub fn push_event(&mut self, event: impl Into<String>) {
        self.event_log.push(event.into());
        const MAX_EVENTS: usize = 10;
        if self.event_log.len() > MAX_EVENTS {
            self.event_log.remove(0);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetainedAction {
    Reset,
    Project,
    TogglePresence,
    DispatchClick,
    Focus,
    Stress10k,
}

impl RetainedAction {
    pub const ALL: [Self; 6] = [
        Self::Reset,
        Self::Project,
        Self::TogglePresence,
        Self::DispatchClick,
        Self::Focus,
        Self::Stress10k,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Reset => "Reset",
            Self::Project => "Project",
            Self::TogglePresence => "Hide/show",
            Self::DispatchClick => "Click route",
            Self::Focus => "Focus",
            Self::Stress10k => "10k stress",
        }
    }
}

#[derive(Debug)]
pub struct RetainedHarness {
    pub model: retained::Model,
    pub selected: retained::Id,
    pub projected: bool,
    pub flags_log: Vec<String>,
    pub route_log: Vec<String>,
    pub command_log: Vec<String>,
    pub note: String,
    pub large_count: Option<usize>,
}

impl Default for RetainedHarness {
    fn default() -> Self {
        let model = retained_fixture_model(false);
        let selected = retained_find_key(&model, "run").unwrap_or_else(|| model.root());
        Self {
            model,
            selected,
            projected: false,
            flags_log: vec![String::from("ready: fixture model retained")],
            route_log: Vec::new(),
            command_log: Vec::new(),
            note: String::from("select a row or press an action"),
            large_count: None,
        }
    }
}

impl RetainedHarness {
    pub fn select(&mut self, id: retained::Id) {
        if self.model.snapshot().get(id).is_some() {
            self.selected = id;
            self.note = format!("selected {id:?}");
        }
    }

    pub fn apply(&mut self, action: RetainedAction) {
        match action {
            RetainedAction::Reset => {
                *self = Self::default();
                self.note = String::from("reset fixture model");
            }
            RetainedAction::Project => self.project_fixture(),
            RetainedAction::TogglePresence => self.toggle_presence(),
            RetainedAction::DispatchClick => self.dispatch_click(),
            RetainedAction::Focus => self.focus_selected(),
            RetainedAction::Stress10k => self.stress_10k(),
        }
    }

    fn project_fixture(&mut self) {
        self.projected = !self.projected;
        let selected_path = self
            .model
            .snapshot()
            .get(self.selected)
            .map(|node| node.key_path().clone());
        let slot = retained::ProjectionSlot::default(self.model.root());
        let report = self
            .model
            .apply_projection(retained::ProjectionEdit::new(
                slot.clone(),
                retained::ProjectionSource::Elements(vec![retained_fixture_panel(self.projected)]),
                retained::ProjectionReplaceMode::PreserveCompatible,
            ))
            .and_then(|_| self.model.resolve_projection(slot));
        self.record_report("project", report);
        if let Some(path) = selected_path
            && let Some(id) = self.model.snapshot().find_key(&path)
        {
            self.selected = id;
        }
        self.large_count = None;
    }

    fn toggle_presence(&mut self) {
        let snapshot = self.model.snapshot();
        let Some(node) = snapshot.get(self.selected) else {
            self.note = String::from("selected node is stale");
            self.selected = self.model.root();
            return;
        };
        let next = if node.state().presence() == retained::Presence::Visible {
            retained::Presence::Hidden
        } else {
            retained::Presence::Visible
        };
        let report = self.model.apply(retained::Patch::SetState {
            id: self.selected,
            state: retained::StatePatch::default().presence(next),
        });
        self.record_report("presence", report);
    }

    fn dispatch_click(&mut self) {
        let event = retained::Event::new(self.selected, retained::EventKind::Click)
            .with_propagation(retained::Propagation::CaptureThenBubble)
            .with_pointer(retained::PointerId::new(1));

        match self.model.route(event.clone()) {
            Ok(route) => {
                self.route_log = route
                    .steps()
                    .iter()
                    .map(|step| format!("{:?} {:?}", step.phase, step.id))
                    .collect();
            }
            Err(error) => {
                self.route_log = vec![format!("route error: {}", error.message())];
            }
        }
        let report = self.model.dispatch(event);
        self.record_report("dispatch", report);
    }

    fn focus_selected(&mut self) {
        let report = self.model.focus(Some(self.selected));
        self.record_report("focus", report);
    }

    fn stress_10k(&mut self) {
        let children = (0..10_000)
            .map(|index| {
                retained::Element::tagged(retained_tag("div"))
                    .with_key(retained_key(&format!("node-{index}")))
                    .with_role(retained::Role::ListItem)
                    .with_class(retained_class("stress-row"))
            })
            .collect::<Vec<_>>();
        let mut model = retained::Model::new(retained::Element::root().with_children(children))
            .expect("retained stress model should be valid");
        let path = retained::KeyPath::root().canonical_key(&retained_key("node-9999"));
        let selected = model
            .snapshot()
            .find_key(&path)
            .unwrap_or_else(|| model.root());
        let report = model.apply(retained::Patch::SetText {
            id: selected,
            text: Some(retained_text("localized edit in 10k model")),
        });
        self.model = model;
        self.selected = selected;
        self.projected = false;
        self.large_count = Some(10_000);
        self.route_log.clear();
        self.command_log.clear();
        self.record_report("10k patch", report);
    }

    fn record_report(&mut self, label: &str, report: retained::Result<retained::Report>) {
        match report {
            Ok(report) => {
                self.note = format_report(label, &report);
                self.flags_log.insert(0, self.note.clone());
                const MAX_LOGS: usize = 6;
                if self.flags_log.len() > MAX_LOGS {
                    self.flags_log.truncate(MAX_LOGS);
                }
                self.command_log = report
                    .commands()
                    .iter()
                    .map(|command| {
                        format!(
                            "{} {:?} target {:?} route {}",
                            command.command().as_str(),
                            command.phase(),
                            command.target(),
                            command.route().steps().len()
                        )
                    })
                    .collect();
            }
            Err(error) => {
                self.note = format!("{label} error: {:?} {}", error.code(), error.message());
                self.flags_log.insert(0, self.note.clone());
            }
        }
    }
}

#[must_use]
pub fn retained_action_at_point(x: f64, y: f64) -> Option<RetainedAction> {
    RetainedAction::ALL
        .iter()
        .copied()
        .enumerate()
        .find_map(|(index, action)| {
            let left = RETAINED_ACTION_X + index as f64 * RETAINED_ACTION_STEP;
            ((left..=left + RETAINED_ACTION_WIDTH).contains(&x)
                && (RETAINED_ACTION_Y..=RETAINED_ACTION_Y + RETAINED_ACTION_HEIGHT).contains(&y))
            .then_some(action)
        })
}

#[must_use]
pub fn retained_node_at_point(state: &DevState, x: f64, y: f64) -> Option<retained::Id> {
    if state.active_scenario() != Scenario::RetainedModel
        || !(RETAINED_TREE_X..=RETAINED_TREE_X + 400.0).contains(&x)
        || y < RETAINED_TREE_Y
    {
        return None;
    }
    let row = ((y - RETAINED_TREE_Y) / RETAINED_ROW_HEIGHT).floor() as usize;
    displayed_retained_nodes(&state.retained.model, RETAINED_VISIBLE_ROWS)
        .get(row)
        .map(|(_, id)| *id)
}

#[must_use]
pub fn scenario_at_point(x: f64, y: f64) -> Option<usize> {
    Scenario::ALL.iter().enumerate().find_map(|(index, _)| {
        let rect = tab_rect(index);
        (x >= rect.x()
            && x <= rect.x() + rect.width()
            && y >= rect.y()
            && y <= rect.y() + rect.height())
        .then_some(index)
    })
}

fn retained_fixture_model(projected: bool) -> retained::Model {
    retained::Model::new(retained::Element::root().with_child(retained_fixture_panel(projected)))
        .expect("retained harness fixture should be valid")
}

fn retained_fixture_panel(projected: bool) -> retained::Element {
    let mut panel = retained::Element::tagged(retained_tag("section"))
        .with_key(retained_key("panel"))
        .with_role(retained::Role::Application)
        .with_class(retained_class("surface"))
        .with_child(
            retained::Element::tagged(retained_tag("button"))
                .with_key(retained_key("run"))
                .with_role(retained::Role::Button)
                .with_label(retained_text("Run retained command"))
                .with_class(retained_class("primary"))
                .with_hook(retained::Hook::new(
                    retained::Trigger::Event(retained::EventKind::Click),
                    retained_command("demo.run"),
                ))
                .with_child(retained::Element::text(retained_text("Run"))),
        )
        .with_child(
            retained::Element::tagged(retained_tag("div"))
                .with_key(retained_key("status"))
                .with_role(retained::Role::Text)
                .with_class(retained_class("muted"))
                .with_child(retained::Element::text(retained_text(
                    "retained fixture status",
                ))),
        )
        .with_child(
            retained::Element::tagged(retained_tag("canvas"))
                .with_key(retained_key("graph-canvas"))
                .with_role(retained::Role::Canvas)
                .with_class(retained_class("canvas-host")),
        );

    if projected {
        panel = panel.with_class(retained_class("projected")).with_child(
            retained::Element::tagged(retained_tag("button"))
                .with_key(retained_key("secondary"))
                .with_role(retained::Role::Button)
                .with_label(retained_text("Secondary command"))
                .with_hook(retained::Hook::new(
                    retained::Trigger::Intent(retained::Intent::Command),
                    retained_command("demo.secondary"),
                ))
                .with_child(retained::Element::text(retained_text("Secondary"))),
        );
    }

    panel
}

fn retained_find_key(model: &retained::Model, key: &str) -> Option<retained::Id> {
    displayed_retained_nodes(model, usize::MAX)
        .into_iter()
        .map(|(_, id)| id)
        .find(|id| {
            model
                .snapshot()
                .get(*id)
                .and_then(|node| node.key().cloned())
                .is_some_and(|node_key| node_key == retained_key(key))
        })
}

fn retained_key(value: &str) -> retained::Key {
    retained::Key::new(value).expect("retained harness key should be valid")
}

fn retained_tag(value: &str) -> retained::Tag {
    retained::Tag::new(value).expect("retained harness tag should be valid")
}

fn retained_class(value: &str) -> retained::Class {
    retained::Class::new(value).expect("retained harness class should be valid")
}

fn retained_text(value: &str) -> retained::Text {
    retained::Text::new(value).expect("retained harness text should be valid")
}

fn retained_command(value: &str) -> retained::CommandName {
    retained::CommandName::new(value).expect("retained harness command should be valid")
}

fn displayed_retained_nodes(model: &retained::Model, limit: usize) -> Vec<(usize, retained::Id)> {
    let snapshot = model.snapshot();
    let mut out = Vec::new();
    collect_retained_nodes(&snapshot, snapshot.root(), 0, limit, &mut out);
    out
}

fn collect_retained_nodes(
    snapshot: &retained::Snapshot<'_>,
    id: retained::Id,
    depth: usize,
    limit: usize,
    out: &mut Vec<(usize, retained::Id)>,
) {
    if out.len() >= limit {
        return;
    }
    out.push((depth, id));
    let children = match snapshot.projected_children(retained::ProjectionSlot::default(id)) {
        Ok(children) => children.collect::<Vec<_>>(),
        Err(_) => snapshot
            .children(id)
            .map(|children| children.collect::<Vec<_>>())
            .unwrap_or_default(),
    };
    for child in children {
        collect_retained_nodes(snapshot, child, depth + 1, limit, out);
        if out.len() >= limit {
            return;
        }
    }
}

fn format_report(label: &str, report: &retained::Report) -> String {
    format!(
        "{label}: +{} -{} changed {} moved {} commands {}",
        report.changes().inserted().len(),
        report.changes().removed().len(),
        report.changes().changed().count(),
        report.changes().moved().len(),
        report.commands().len()
    )
}

#[derive(Clone, Copy, Debug)]
pub struct WindowFacts {
    pub logical_size: render::Size,
    pub scale_factor: f64,
    pub focused: bool,
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

fn render_radii(
    top_left: f64,
    top_right: f64,
    bottom_right: f64,
    bottom_left: f64,
) -> render::Radii {
    render::Radii::try_new(top_left, top_right, bottom_right, bottom_left)
        .expect("dev harness render radii should be valid")
}

fn render_radii_all(radius: f64) -> render::Radii {
    render::Radii::try_all(radius).expect("dev harness render radii should be valid")
}

fn render_stroke_width(width: f64) -> render::Stroke {
    render::Stroke::try_new(width).expect("dev harness render stroke should be valid")
}

fn render_shadow(
    offset: render::Point,
    blur: f64,
    spread: f64,
    paint: render::Color,
) -> render::Shadow {
    render::Shadow::try_new(offset, blur, spread, paint)
        .expect("dev harness render shadow should be valid")
}

fn render_color_rgba(r: f32, g: f32, b: f32, a: f32) -> render::Color {
    render::Color::try_rgba(r, g, b, a).expect("dev harness color should be valid")
}

fn render_gradient_stop(offset: f32, color: render::Color) -> render::GradientStop {
    render::GradientStop::try_new(offset, color).expect("dev harness gradient stop should be valid")
}

fn render_linear_gradient(
    start: render::Point,
    end: render::Point,
    stops: Vec<render::GradientStop>,
) -> render::Gradient {
    render::Gradient::try_linear(start, end, stops)
        .expect("dev harness linear gradient should be valid")
}

fn render_translate(x: f64, y: f64) -> render::Transform {
    render::Transform::try_new([1.0, 0.0, 0.0, 1.0, x, y])
        .expect("dev harness render transform should be valid")
}

fn render_circle(center: render::Point, radius: f64) -> render::Shape {
    render::Shape::try_circle(center, radius).expect("dev harness render circle should be valid")
}

fn render_layer_blend(blend: render::BlendMode) -> render::Layer {
    render::Layer::new().blend(blend)
}

fn render_layer_opacity(opacity: f32) -> render::Layer {
    render::Layer::new()
        .try_opacity(opacity)
        .expect("dev harness layer opacity should be valid")
}

fn shape_point(x: f64, y: f64) -> shape::Point {
    shape::Point::try_new(x, y).expect("dev harness shape point should be valid")
}

fn shape_size(width: f64, height: f64) -> shape::Size {
    shape::Size::try_new(width, height).expect("dev harness shape size should be valid")
}

fn shape_rect(x: f64, y: f64, width: f64, height: f64) -> shape::Rect {
    shape::Rect::try_new(x, y, width, height).expect("dev harness shape rect should be valid")
}

fn shape_insets(top: f64, right: f64, bottom: f64, left: f64) -> shape::Insets {
    shape::Insets::try_new(top, right, bottom, left)
        .expect("dev harness shape insets should be valid")
}

fn shape_radii(top_left: f64, top_right: f64, bottom_right: f64, bottom_left: f64) -> shape::Radii {
    shape::Radii::try_new(top_left, top_right, bottom_right, bottom_left)
        .expect("dev harness shape radii should be valid")
}

fn shape_radii_all(radius: f64) -> shape::Radii {
    shape::Radii::try_all(radius).expect("dev harness shape radii should be valid")
}

fn shape_stroke_width(width: f64) -> shape::Stroke {
    shape::Stroke::try_new(width).expect("dev harness shape stroke should be valid")
}

fn shape_stroke_inside(width: f64) -> shape::Stroke {
    shape::Stroke::try_inside(width).expect("dev harness shape stroke should be valid")
}

fn shape_dash_density(dash: shape::Dash, density: f64) -> shape::Dash {
    dash.try_with_density(density)
        .expect("dev harness dash density should be valid")
}

fn shape_dash_sides(dash: shape::Dash, sides: shape::SideSet) -> shape::Dash {
    dash.with_sides(sides)
        .expect("dev harness dash side set should be valid")
}

fn shape_rect_shape(rect: shape::Rect) -> shape::Shape {
    shape::Shape::try_rect(rect).expect("dev harness shape rect should be valid")
}

fn shape_rounded_rect(rect: shape::Rect, radii: shape::Radii) -> shape::Shape {
    shape::Shape::try_rounded_rect(rect, radii).expect("dev harness rounded shape should be valid")
}

fn shape_circle(center: shape::Point, radius: f64) -> shape::Shape {
    shape::Shape::try_circle(center, radius).expect("dev harness circle should be valid")
}

fn shape_ellipse(center: shape::Point, radii: shape::Size) -> shape::Shape {
    shape::Shape::try_ellipse(center, radii).expect("dev harness ellipse should be valid")
}

fn shape_path(path: shape::Path, fill_rule: shape::FillRule) -> shape::Shape {
    shape::Shape::try_path(path, fill_rule).expect("dev harness path shape should be valid")
}

pub fn build_scene(
    text_system: &mut text::System,
    state: &DevState,
    facts: WindowFacts,
) -> render::Scene {
    let mut scene = render::Scene::new();
    draw_background(&mut scene, facts.logical_size);
    draw_chrome(&mut scene, text_system, state, facts);

    match state.active_scenario() {
        Scenario::TextBasics => draw_text_basics(&mut scene, text_system),
        Scenario::BidiSelection => draw_bidi_selection(&mut scene, text_system),
        Scenario::InlineBoxes => draw_inline_boxes(&mut scene, text_system),
        Scenario::RenderPrimitives => draw_render_primitives(&mut scene),
        Scenario::ShapeGeometry => draw_shape_geometry(&mut scene),
        Scenario::RetainedModel => draw_retained_model(&mut scene, text_system, &state.retained),
        Scenario::WindowState => draw_window_state(&mut scene, text_system, state, facts),
    }

    scene
}

fn draw_background(scene: &mut render::Scene, size: render::Size) {
    scene.fill(
        render_rect(0.0, 0.0, size.width(), size.height()),
        render_linear_gradient(
            render_point(0.0, 0.0),
            render_point(size.width().max(1.0), size.height().max(1.0)),
            vec![
                render_gradient_stop(0.0, color(0xF7, 0xF8, 0xFB, 0xFF)),
                render_gradient_stop(1.0, color(0xDF, 0xE7, 0xF3, 0xFF)),
            ],
        ),
    );
}

fn draw_chrome(
    scene: &mut render::Scene,
    text_system: &mut text::System,
    state: &DevState,
    facts: WindowFacts,
) {
    scene
        .shadow(
            render::Shape::rounded_rect(
                render_rect(24.0, 24.0, 960.0, 88.0),
                render_radii_all(18.0),
            ),
            render_shadow(
                render_point(0.0, 12.0),
                28.0,
                0.0,
                color(0x27, 0x38, 0x53, 0x33),
            ),
        )
        .fill(
            render::Shape::rounded_rect(
                render_rect(24.0, 24.0, 960.0, 88.0),
                render_radii_all(18.0),
            ),
            color(0xFF, 0xFF, 0xFF, 0xE8),
        );

    draw_text(
        scene,
        text_system,
        "Surgeist dev harness",
        render_point(48.0, 48.0),
        760.0,
        text_style(28.0, color(0x1E, 0x2A, 0x3A, 0xFF)),
        &[],
    );
    draw_text(
        scene,
        text_system,
        &format!(
            "{}  |  {}x{} @ {:.2}x  |  focused: {}",
            state.active_scenario().title(),
            facts.logical_size.width().round(),
            facts.logical_size.height().round(),
            facts.scale_factor,
            facts.focused
        ),
        render_point(48.0, 82.0),
        850.0,
        text_style(14.0, color(0x58, 0x67, 0x7A, 0xFF)),
        &[],
    );

    for (index, scenario) in Scenario::ALL.iter().enumerate() {
        let rect = tab_rect(index);
        let active = index == state.active;
        scene.fill(
            render::Shape::rounded_rect(rect, render_radii_all(10.0)),
            if active {
                color(0x20, 0x58, 0xA8, 0xFF)
            } else {
                color(0xEC, 0xF0, 0xF5, 0xFF)
            },
        );
        draw_text(
            scene,
            text_system,
            &format!("{} {}", index + 1, scenario.title()),
            render_point(rect.x() + 12.0, rect.y() + 8.0),
            116.0,
            text_style(
                12.0,
                if active {
                    color(0xFF, 0xFF, 0xFF, 0xFF)
                } else {
                    color(0x2F, 0x3B, 0x4A, 0xFF)
                },
            ),
            &[],
        );
    }
}

fn tab_rect(index: usize) -> render::Rect {
    render_rect(
        TAB_X + index as f64 * TAB_STEP,
        TAB_Y,
        TAB_WIDTH,
        TAB_HEIGHT,
    )
}

fn draw_text_basics(scene: &mut render::Scene, text_system: &mut text::System) {
    draw_panel(scene, 42.0, 190.0, 720.0, 330.0);
    let copy = "Parley-backed text through Surgeist render. Strong spans, color spans, underline, strikethrough, wrapping, and decoration projection are all visible in this paragraph.";
    let strong = range_of(copy, "Strong");
    let color_span = range_of(copy, "color spans");
    let underline = range_of(copy, "underline");
    let strike = range_of(copy, "strikethrough");
    let spans = [
        text::Span::new(
            strong,
            text_style(22.0, color(0x1E, 0x2A, 0x3A, 0xFF)).with_weight(text::Weight::Bold),
        ),
        text::Span::new(color_span, text_style(22.0, color(0xB7, 0x3A, 0x47, 0xFF))),
        text::Span::new(
            underline,
            text_style(22.0, color(0x1E, 0x2A, 0x3A, 0xFF))
                .with_underline(color(0x20, 0x58, 0xA8, 0xFF)),
        ),
        text::Span::new(
            strike,
            text_style(22.0, color(0x1E, 0x2A, 0x3A, 0xFF))
                .with_strikethrough(color(0xD9, 0x86, 0x24, 0xFF)),
        ),
    ];
    let layout = layout_text(
        text_system,
        copy,
        620.0,
        text_style(22.0, color(0x1E, 0x2A, 0x3A, 0xFF)),
        &spans,
        &[],
    );
    draw_layout(scene, &layout, render_point(76.0, 232.0));
    draw_layout_diagnostics(scene, &layout, render_point(76.0, 232.0));
}

fn draw_bidi_selection(scene: &mut render::Scene, text_system: &mut text::System) {
    draw_panel(scene, 42.0, 190.0, 820.0, 350.0);
    let copy = "English text, עברית באמצע, and Arabic العربية in one wrapped paragraph. Cursor movement and selection geometry should stay visually sane.";
    let layout = layout_text(
        text_system,
        copy,
        680.0,
        text_style(24.0, color(0x18, 0x22, 0x31, 0xFF)),
        &[],
        &[],
    );
    let origin = render_point(76.0, 232.0);
    draw_selection(
        scene,
        &layout,
        origin,
        text::Selection::new(
            text::Cursor::new(14, text::Affinity::After),
            text::Cursor::new(55, text::Affinity::Before),
        ),
    );
    draw_cursor(
        scene,
        &layout,
        origin,
        text::Cursor::new(34, text::Affinity::After),
    );
    draw_layout(scene, &layout, origin);
    draw_layout_diagnostics(scene, &layout, origin);
}

fn draw_inline_boxes(scene: &mut render::Scene, text_system: &mut text::System) {
    draw_panel(scene, 42.0, 190.0, 820.0, 350.0);
    let copy = "Text before  and text after an inline reserved box, with an out-of-flow marker anchored later.";
    let boxes = [
        text::InlineBox::new(
            text::Id::from_u64(100),
            text::InlineBoxKind::InFlow,
            "Text before ".len(),
            text::Size::new(72.0, 32.0),
        ),
        text::InlineBox::new(
            text::Id::from_u64(101),
            text::InlineBoxKind::OutOfFlow,
            range_of(copy, "out-of-flow").start,
            text::Size::new(18.0, 18.0),
        ),
    ];
    let layout = layout_text(
        text_system,
        copy,
        660.0,
        text_style(22.0, color(0x1E, 0x2A, 0x3A, 0xFF)),
        &[],
        &boxes,
    );
    let origin = render_point(76.0, 232.0);
    draw_layout(scene, &layout, origin);
    for box_ in layout.inline_boxes() {
        let rect = translated_rect(box_.rect(), origin);
        scene
            .fill(
                render::Shape::rounded_rect(rect, render_radii_all(8.0)),
                match box_.kind() {
                    text::InlineBoxKind::InFlow => color(0xF2, 0xC8, 0x5E, 0xDD),
                    text::InlineBoxKind::OutOfFlow => color(0x5A, 0xC8, 0xA0, 0xDD),
                },
            )
            .stroke(
                render::Shape::rounded_rect(rect, render_radii_all(8.0)),
                render_stroke_width(2.0),
                color(0x2F, 0x3B, 0x4A, 0xFF),
            );
    }
    draw_layout_diagnostics(scene, &layout, origin);
}

fn draw_render_primitives(scene: &mut render::Scene) {
    draw_panel(scene, 42.0, 190.0, 960.0, 420.0);
    scene
        .shadow(
            render::Shape::rounded_rect(
                render_rect(76.0, 230.0, 170.0, 96.0),
                render_radii_all(20.0),
            ),
            render_shadow(
                render_point(0.0, 16.0),
                30.0,
                2.0,
                color(0x16, 0x22, 0x34, 0x55),
            ),
        )
        .fill(
            render::Shape::rounded_rect(
                render_rect(76.0, 230.0, 170.0, 96.0),
                render_radii_all(20.0),
            ),
            render_linear_gradient(
                render_point(76.0, 230.0),
                render_point(246.0, 326.0),
                vec![
                    render_gradient_stop(0.0, color(0xFF, 0xFF, 0xFF, 0xFF)),
                    render_gradient_stop(1.0, color(0xB9, 0xD4, 0xFF, 0xFF)),
                ],
            ),
        );

    scene.stroke(
        render_rect(290.0, 230.0, 126.0, 86.0),
        render_stroke_width(10.0).align(render::StrokeAlign::Inside),
        color(0x20, 0x58, 0xA8, 0xFF),
    );
    scene.stroke(
        render_rect(450.0, 230.0, 126.0, 86.0),
        render_stroke_width(10.0).align(render::StrokeAlign::Outside),
        color(0xB7, 0x3A, 0x47, 0xFF),
    );

    let mut path = render::Path::new();
    path.move_to(render_point(650.0, 284.0)).cubic_to(
        render_point(690.0, 194.0),
        render_point(760.0, 374.0),
        render_point(810.0, 244.0),
    );
    scene.stroke(
        render::Shape::path(path),
        render_stroke_width(12.0)
            .join(render::LineJoin::Round)
            .caps(render::LineCap::Round, render::LineCap::Square),
        color(0x25, 0x8B, 0x73, 0xFF),
    );

    let image = checker_image().expect("checker image should be valid");
    scene.image(
        image
            .quality(render::ImageQuality::Low)
            .extend(render::Extend::Repeat),
        render_rect(78.0, 370.0, 160.0, 96.0),
        render::ImageFit::Stretch,
    );

    let multiply_clip = render_rect(284.0, 358.0, 252.0, 128.0);
    scene
        .stroke(
            multiply_clip,
            render_stroke_width(1.0),
            color(0xD4, 0xDE, 0xEC, 0xFF),
        )
        .clip(multiply_clip, |scene| {
            scene.fill(
                render_circle(render_point(360.0, 422.0), 58.0),
                color(0xFF, 0xC0, 0x52, 0xFF),
            );
            scene.layer(render_layer_blend(render::BlendMode::Multiply), |scene| {
                scene.fill(
                    render_circle(render_point(438.0, 422.0), 58.0),
                    color(0x5B, 0xA9, 0xFF, 0xFF),
                );
            });
        });

    scene.layer(render_layer_opacity(0.5), |scene| {
        scene.fill(
            render_circle(render_point(650.0, 422.0), 58.0),
            color(0xB7, 0x3A, 0x47, 0xFF),
        );
    });
    scene.fill(
        render_circle(render_point(710.0, 422.0), 58.0),
        color(0x25, 0x8B, 0x73, 0xB8),
    );

    scene.clip(render_rect(800.0, 358.0, 144.0, 96.0), |scene| {
        scene
            .fill(
                render_circle(render_point(800.0, 406.0), 64.0),
                color(0xF2, 0xC8, 0x5E, 0xDD),
            )
            .fill(
                render_circle(render_point(944.0, 406.0), 64.0),
                color(0x5B, 0xA9, 0xFF, 0xDD),
            );
    });
    scene.stroke(
        render_rect(800.0, 358.0, 144.0, 96.0),
        render_stroke_width(1.0),
        color(0xD4, 0xDE, 0xEC, 0xFF),
    );

    let corner_rect = render_rect(846.0, 226.0, 124.0, 86.0);
    let corner_radii = render_radii(0.0, 5.0, 10.0, 0.0);
    scene
        .shadow(
            render::Shape::rounded_rect(corner_rect, corner_radii),
            render_shadow(
                render_point(12.0, 14.0),
                24.0,
                0.0,
                color(0x16, 0x22, 0x34, 0x3A),
            ),
        )
        .fill(
            render::Shape::rounded_rect(corner_rect, corner_radii),
            color(0xF2, 0xC8, 0x5E, 0xFF),
        )
        .stroke(
            render::Shape::rounded_rect(corner_rect, corner_radii),
            render_stroke_width(2.0).align(render::StrokeAlign::Inside),
            color(0xB7, 0x8B, 0x2D, 0xFF),
        );
}

fn draw_shape_geometry(scene: &mut render::Scene) {
    draw_panel(scene, 42.0, 190.0, 1040.0, 500.0);

    let rounded = shape_rounded_rect(
        shape_rect(76.0, 230.0, 190.0, 110.0),
        shape_radii(0.0, 10.0, 28.0, 0.0),
    );
    draw_shape_fill(scene, &rounded, color(0xD8, 0xE8, 0xFF, 0xFF));
    draw_shape_stroke(
        scene,
        &rounded,
        shape_stroke_inside(3.0),
        color(0x20, 0x58, 0xA8, 0xFF),
    );
    draw_shape_rect(scene, rounded.bounds(), color(0x20, 0x58, 0xA8, 0x28));
    draw_shape_rect(
        scene,
        rounded
            .support_bounds(shape_insets(18.0, 26.0, 18.0, 10.0))
            .expect("shape support bounds should resolve"),
        color(0xB7, 0x3A, 0x47, 0x22),
    );

    let circle = shape_circle(shape_point(380.0, 286.0), 58.0);
    draw_shape_fill(scene, &circle, color(0xF2, 0xC8, 0x5E, 0xD8));
    draw_shape_dashes(
        scene,
        &circle,
        shape_stroke_width(5.0).with_dash(shape_dash_density(shape::Dash::dotted(), 1.35)),
        color(0x25, 0x8B, 0x73, 0xFF),
    );

    let ellipse = shape_ellipse(shape_point(560.0, 286.0), shape_size(76.0, 46.0));
    draw_shape_fill(scene, &ellipse, color(0xFF, 0xF7, 0xE0, 0xFF));
    draw_shape_dashes(
        scene,
        &ellipse,
        shape_stroke_width(4.0).with_dash(shape_dash_density(shape::Dash::dashed(), 1.6).rounded()),
        color(0xB7, 0x3A, 0x47, 0xFF),
    );

    let full_dash =
        shape_rounded_rect(shape_rect(76.0, 410.0, 230.0, 130.0), shape_radii_all(24.0));
    draw_shape_fill(scene, &full_dash, color(0xFF, 0xFF, 0xFF, 0xF0));
    draw_shape_dashes(
        scene,
        &full_dash,
        shape_stroke_width(6.0)
            .with_dash(shape_dash_density(shape::Dash::dashed(), 1.1).with_corner_anchors()),
        color(0x20, 0x58, 0xA8, 0xFF),
    );

    let side_dash = shape_rounded_rect(
        shape_rect(360.0, 410.0, 230.0, 130.0),
        shape_radii_all(30.0),
    );
    draw_shape_fill(scene, &side_dash, color(0xFF, 0xFF, 0xFF, 0xF0));
    draw_shape_dashes(
        scene,
        &side_dash,
        shape_stroke_width(6.0).with_dash(
            shape_dash_sides(
                shape_dash_density(shape::Dash::dashed(), 1.2).rounded(),
                shape::SideSet {
                    top: true,
                    right: true,
                    bottom: false,
                    left: false,
                },
            )
            .with_corner_anchors(),
        ),
        color(0x25, 0x8B, 0x73, 0xFF),
    );

    let dot_dash = shape_rounded_rect(
        shape_rect(644.0, 410.0, 230.0, 130.0),
        shape_radii_all(30.0),
    );
    draw_shape_fill(scene, &dot_dash, color(0xFF, 0xFF, 0xFF, 0xF0));
    draw_shape_dashes(
        scene,
        &dot_dash,
        shape_stroke_width(7.0).with_dash(
            shape_dash_sides(
                shape_dash_density(shape::Dash::dotted(), 1.15),
                shape::SideSet::horizontal(),
            )
            .with_corner_anchors(),
        ),
        color(0xB7, 0x3A, 0x47, 0xFF),
    );

    let sharp_dash = shape_rect_shape(shape_rect(928.0, 410.0, 120.0, 130.0));
    draw_shape_fill(scene, &sharp_dash, color(0xFF, 0xFF, 0xFF, 0xF0));
    draw_shape_dashes(
        scene,
        &sharp_dash,
        shape_stroke_width(6.0)
            .with_dash(shape_dash_density(shape::Dash::dashed(), 1.1).with_corner_anchors()),
        color(0x20, 0x58, 0xA8, 0xFF),
    );

    let mut path = shape::PathBuilder::new();
    path.move_to(shape_point(760.0, 270.0)).cubic_to(
        shape_point(805.0, 205.0),
        shape_point(845.0, 360.0),
        shape_point(910.0, 252.0),
    );
    let custom = shape_path(
        path.build().expect("dev harness path should be valid"),
        shape::FillRule::NonZero,
    );
    draw_shape_stroke(
        scene,
        &custom,
        shape_stroke_width(10.0)
            .with_join(shape::LineJoin::Round)
            .with_caps(shape::LineCap::Round, shape::LineCap::Round),
        color(0x2F, 0x3B, 0x4A, 0xFF),
    );
}

fn draw_retained_model(
    scene: &mut render::Scene,
    text_system: &mut text::System,
    harness: &RetainedHarness,
) {
    draw_panel(scene, 42.0, 190.0, 1120.0, 500.0);

    let node_count = harness.large_count.map_or_else(
        || {
            harness
                .model
                .snapshot()
                .descendants(harness.model.root())
                .map_or(1, |nodes| nodes.count() + 1)
        },
        |count| count + 1,
    );
    let summary = format!(
        "{} nodes{}  |  selected {:?}  |  {}",
        node_count,
        harness
            .large_count
            .map(|count| format!(" ({count} stress children)"))
            .unwrap_or_default(),
        harness.selected,
        harness.note
    );
    draw_text(
        scene,
        text_system,
        &summary,
        render_point(76.0, 258.0),
        980.0,
        text_style(14.0, color(0x58, 0x67, 0x7A, 0xFF)),
        &[],
    );

    for (index, action) in RetainedAction::ALL.iter().copied().enumerate() {
        let rect = render_rect(
            RETAINED_ACTION_X + index as f64 * RETAINED_ACTION_STEP,
            RETAINED_ACTION_Y,
            RETAINED_ACTION_WIDTH,
            RETAINED_ACTION_HEIGHT,
        );
        scene.fill(
            render::Shape::rounded_rect(rect, render_radii_all(8.0)),
            match action {
                RetainedAction::Stress10k => color(0x25, 0x8B, 0x73, 0xFF),
                RetainedAction::Reset => color(0xEC, 0xF0, 0xF5, 0xFF),
                _ => color(0x20, 0x58, 0xA8, 0xFF),
            },
        );
        draw_text(
            scene,
            text_system,
            action.label(),
            render_point(rect.x() + 12.0, rect.y() + 7.0),
            108.0,
            text_style(
                12.0,
                if action == RetainedAction::Reset {
                    color(0x2F, 0x3B, 0x4A, 0xFF)
                } else {
                    color(0xFF, 0xFF, 0xFF, 0xFF)
                },
            ),
            &[],
        );
    }

    draw_retained_box(scene, 68.0, 278.0, 420.0, 334.0);
    draw_text(
        scene,
        text_system,
        "model tree",
        render_point(82.0, 292.0),
        180.0,
        text_style(13.0, color(0x58, 0x67, 0x7A, 0xFF)),
        &[],
    );
    let rows = displayed_retained_nodes(&harness.model, RETAINED_VISIBLE_ROWS);
    for (row, (depth, id)) in rows.iter().enumerate() {
        let y = RETAINED_TREE_Y + row as f64 * RETAINED_ROW_HEIGHT;
        if *id == harness.selected {
            scene.fill(
                render::Shape::rounded_rect(
                    render_rect(RETAINED_TREE_X - 6.0, y - 2.0, 388.0, 22.0),
                    render_radii_all(6.0),
                ),
                color(0xD8, 0xE8, 0xFF, 0xFF),
            );
        }
        let label = retained_node_label(&harness.model, *id, *depth);
        draw_text(
            scene,
            text_system,
            &label,
            render_point(RETAINED_TREE_X + *depth as f64 * 14.0, y),
            370.0 - *depth as f32 * 14.0,
            text_style(13.0, color(0x1E, 0x2A, 0x3A, 0xFF)),
            &[],
        );
    }

    draw_retained_box(scene, 510.0, 278.0, 292.0, 334.0);
    draw_text(
        scene,
        text_system,
        "selected node",
        render_point(524.0, 292.0),
        180.0,
        text_style(13.0, color(0x58, 0x67, 0x7A, 0xFF)),
        &[],
    );
    draw_text(
        scene,
        text_system,
        &retained_inspector_text(harness),
        render_point(524.0, 318.0),
        248.0,
        text_style(13.0, color(0x1E, 0x2A, 0x3A, 0xFF)),
        &[],
    );

    draw_retained_box(scene, 824.0, 278.0, 300.0, 334.0);
    draw_text(
        scene,
        text_system,
        "routes, commands, flags",
        render_point(838.0, 292.0),
        250.0,
        text_style(13.0, color(0x58, 0x67, 0x7A, 0xFF)),
        &[],
    );
    draw_text(
        scene,
        text_system,
        &retained_log_text(harness),
        render_point(838.0, 318.0),
        252.0,
        text_style(12.0, color(0x1E, 0x2A, 0x3A, 0xFF)),
        &[],
    );
}

fn draw_retained_box(scene: &mut render::Scene, x: f64, y: f64, width: f64, height: f64) {
    scene
        .fill(
            render::Shape::rounded_rect(render_rect(x, y, width, height), render_radii_all(10.0)),
            color(0xFF, 0xFF, 0xFF, 0xC8),
        )
        .stroke(
            render::Shape::rounded_rect(render_rect(x, y, width, height), render_radii_all(10.0)),
            render_stroke_width(1.0),
            color(0xD4, 0xDE, 0xEC, 0xFF),
        );
}

fn retained_node_label(model: &retained::Model, id: retained::Id, depth: usize) -> String {
    let snapshot = model.snapshot();
    let Some(node) = snapshot.get(id) else {
        return format!("{id:?} stale");
    };
    let key = node.key().map_or("-", retained::Key::as_str);
    format!(
        "{} {:?} key={} {}",
        format_kind(node.kind()),
        node.role(),
        key,
        if depth == 0 { "root" } else { "" }
    )
}

fn retained_inspector_text(harness: &RetainedHarness) -> String {
    let snapshot = harness.model.snapshot();
    let Some(node) = snapshot.get(harness.selected) else {
        return String::from("selected node is stale");
    };
    let attrs = node
        .attributes()
        .iter()
        .map(|attribute| format!("{}={}", attribute.name.as_str(), attribute.value.as_str()))
        .collect::<Vec<_>>()
        .join(", ");
    let classes = node
        .classes()
        .iter()
        .map(retained::Class::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "id: {:?}\npath: {:?}\nkind: {}\nrole: {:?}\nkey: {}\nclasses: {}\nattrs: {}\ntext: {}\nstate: {}",
        node.id(),
        node.key_path(),
        format_kind(node.kind()),
        node.role(),
        node.key().map_or("-", retained::Key::as_str),
        empty_dash(&classes),
        empty_dash(&attrs),
        node.text().map_or("-", retained::Text::as_str),
        format_state(node.state())
    )
}

fn retained_log_text(harness: &RetainedHarness) -> String {
    let mut lines = Vec::new();
    lines.push(String::from("route:"));
    if harness.route_log.is_empty() {
        lines.push(String::from("-"));
    } else {
        lines.extend(harness.route_log.iter().take(5).cloned());
    }
    lines.push(String::from("commands:"));
    if harness.command_log.is_empty() {
        lines.push(String::from("-"));
    } else {
        lines.extend(harness.command_log.iter().take(3).cloned());
    }
    lines.push(String::from("changes:"));
    lines.extend(harness.flags_log.iter().take(5).cloned());
    lines.join("\n")
}

fn format_kind(kind: &retained::Kind) -> String {
    match kind {
        retained::Kind::Root => String::from("root"),
        retained::Kind::Element(tag) => tag.as_str().to_owned(),
        retained::Kind::Text => String::from("text"),
        retained::Kind::Canvas => String::from("canvas"),
        retained::Kind::Fragment => String::from("fragment"),
        retained::Kind::Slot(tag) => format!("slot:{}", tag.as_str()),
        retained::Kind::Widget(tag) => format!("widget:{}", tag.as_str()),
        _ => String::from("unknown"),
    }
}

fn format_state(state: &retained::State) -> String {
    format!(
        "{:?} dis={} hov={} act={} foc={} within={} cap={} sel={} press={}",
        state.presence(),
        state.disabled(),
        state.hovered(),
        state.active(),
        state.focused(),
        state.focus_within(),
        state.pointer_captured(),
        state.selected(),
        state.pressed()
    )
}

fn empty_dash(value: &str) -> &str {
    if value.is_empty() { "-" } else { value }
}

fn draw_window_state(
    scene: &mut render::Scene,
    text_system: &mut text::System,
    state: &DevState,
    facts: WindowFacts,
) {
    draw_panel(scene, 42.0, 190.0, 760.0, 390.0);
    let mut lines = vec![
        "Window integration facts".to_owned(),
        format!(
            "logical size: {:.0} x {:.0}",
            facts.logical_size.width(),
            facts.logical_size.height()
        ),
        format!("scale factor: {:.2}", facts.scale_factor),
        format!("focused: {}", facts.focused),
        "keys: 1-7 switch scenarios, arrows cycle, Escape exits".to_owned(),
        "recent events:".to_owned(),
    ];
    lines.extend(state.event_log.iter().cloned());
    draw_text(
        scene,
        text_system,
        &lines.join("\n"),
        render_point(76.0, 232.0),
        660.0,
        text_style(18.0, color(0x1E, 0x2A, 0x3A, 0xFF)),
        &[],
    );
}

fn draw_panel(scene: &mut render::Scene, x: f64, y: f64, width: f64, height: f64) {
    scene
        .shadow(
            render::Shape::rounded_rect(render_rect(x, y, width, height), render_radii_all(18.0)),
            render_shadow(
                render_point(0.0, 14.0),
                30.0,
                0.0,
                color(0x27, 0x38, 0x53, 0x28),
            ),
        )
        .fill(
            render::Shape::rounded_rect(render_rect(x, y, width, height), render_radii_all(18.0)),
            color(0xFF, 0xFF, 0xFF, 0xEA),
        );
}

fn draw_shape_fill(scene: &mut render::Scene, shape: &shape::Shape, paint: render::Color) {
    scene.fill(render_shape(shape), paint);
}

fn draw_shape_stroke(
    scene: &mut render::Scene,
    shape: &shape::Shape,
    stroke: shape::Stroke,
    paint: render::Color,
) {
    scene.stroke(render_shape(shape), render_stroke(stroke), paint);
}

fn draw_shape_dashes(
    scene: &mut render::Scene,
    shape: &shape::Shape,
    stroke: shape::Stroke,
    paint: render::Color,
) {
    let geometry = shape
        .dashed_stroke(stroke)
        .expect("shape dash geometry should resolve");
    for segment in geometry.segments() {
        scene.stroke(
            render::Shape::path(render_path(segment.path())),
            render_stroke_width(segment.width())
                .join(render::LineJoin::Round)
                .caps(
                    if segment.rounded() {
                        render::LineCap::Round
                    } else {
                        render::LineCap::Butt
                    },
                    if segment.rounded() {
                        render::LineCap::Round
                    } else {
                        render::LineCap::Butt
                    },
                ),
            paint,
        );
    }
}

fn draw_shape_rect(scene: &mut render::Scene, rect: shape::Rect, paint: render::Color) {
    scene.stroke(shape_rect_to_render(rect), render_stroke_width(1.0), paint);
}

fn render_shape(shape: &shape::Shape) -> render::Shape {
    if shape.to_kurbo_rect().is_some() {
        return render::Shape::rect(shape_rect_to_render(shape.bounds()));
    }
    if let Some(rounded) = shape.to_kurbo_rounded_rect() {
        let rect = rounded.rect();
        let radii = rounded.radii();
        return render::Shape::rounded_rect(
            render_rect(rect.x0, rect.y0, rect.width(), rect.height()),
            render_radii(
                radii.top_left,
                radii.top_right,
                radii.bottom_right,
                radii.bottom_left,
            ),
        );
    }
    render::Shape::path(render_path(
        &shape.to_path().expect("shape should convert to path"),
    ))
}

fn render_stroke(stroke: shape::Stroke) -> render::Stroke {
    render_stroke_width(stroke.width()).align(match stroke.align() {
        shape::StrokeAlign::Center => render::StrokeAlign::Center,
        shape::StrokeAlign::Inside => render::StrokeAlign::Inside,
        shape::StrokeAlign::Outside => render::StrokeAlign::Outside,
    })
}

fn render_path(path: &shape::Path) -> render::Path {
    let mut out = render::Path::new();
    for command in path.commands() {
        match *command {
            shape::Command::MoveTo(point) => {
                out.move_to(shape_point_to_render(point));
            }
            shape::Command::LineTo(point) => {
                out.line_to(shape_point_to_render(point));
            }
            shape::Command::QuadTo { control, end } => {
                out.quad_to(shape_point_to_render(control), shape_point_to_render(end));
            }
            shape::Command::CubicTo {
                control_a,
                control_b,
                end,
            } => {
                out.cubic_to(
                    shape_point_to_render(control_a),
                    shape_point_to_render(control_b),
                    shape_point_to_render(end),
                );
            }
            shape::Command::Close => {
                out.close();
            }
        }
    }
    out
}

fn shape_rect_to_render(rect: shape::Rect) -> render::Rect {
    render_rect(
        rect.origin().x(),
        rect.origin().y(),
        rect.size().width(),
        rect.size().height(),
    )
}

fn shape_point_to_render(point: shape::Point) -> render::Point {
    render_point(point.x(), point.y())
}

fn draw_text(
    scene: &mut render::Scene,
    text_system: &mut text::System,
    copy: &str,
    origin: render::Point,
    width: f32,
    style: text::Style,
    spans: &[text::Span],
) {
    let layout = layout_text(text_system, copy, width, style, spans, &[]);
    draw_layout(scene, &layout, origin);
}

fn layout_text(
    text_system: &mut text::System,
    copy: &str,
    width: f32,
    style: text::Style,
    spans: &[text::Span],
    boxes: &[text::InlineBox],
) -> text::Layout {
    let mut builder = text_system.builder(copy);
    builder.default_style(style).options(text::Options {
        width: Some(width),
        ..text::Options::default()
    });
    for span in spans {
        builder.span(span.range(), span.style().clone());
    }
    for box_ in boxes {
        builder.inline_box(*box_);
    }
    builder.build().expect("dev text layout should build")
}

fn draw_layout(scene: &mut render::Scene, layout: &text::Layout, origin: render::Point) {
    layout.push_render_text(scene, render_translate(origin.x(), origin.y()));
}

fn draw_selection(
    scene: &mut render::Scene,
    layout: &text::Layout,
    origin: render::Point,
    selection: text::Selection,
) {
    for rect in layout.selection(selection).rects() {
        scene.fill(
            translated_rect(rect.rect(), origin),
            color(0x86, 0xB7, 0xFF, 0x66),
        );
    }
}

fn draw_cursor(
    scene: &mut render::Scene,
    layout: &text::Layout,
    origin: render::Point,
    cursor: text::Cursor,
) {
    let rect = translated_rect(layout.cursor(cursor).rect(), origin);
    scene.fill(rect, color(0x20, 0x58, 0xA8, 0xFF));
}

fn draw_layout_diagnostics(
    scene: &mut render::Scene,
    layout: &text::Layout,
    origin: render::Point,
) {
    for line in layout.lines() {
        scene.stroke(
            translated_rect(line.bounds(), origin),
            render_stroke_width(1.0),
            color(0x20, 0x58, 0xA8, 0x24),
        );
    }
    for cluster in layout.clusters().into_iter().take(48) {
        scene.stroke(
            translated_rect(cluster.bounds(), origin),
            render_stroke_width(0.5),
            color(0xD9, 0x86, 0x24, 0x45),
        );
    }
}

fn translated_rect(rect: text::Rect, origin: render::Point) -> render::Rect {
    render_rect(
        origin.x() + f64::from(rect.origin().x()),
        origin.y() + f64::from(rect.origin().y()),
        f64::from(rect.size().width()),
        f64::from(rect.size().height()),
    )
}

fn text_style(size: f32, brush: render::Color) -> text::Style {
    text::Style {
        size,
        line_height: text::LineHeight::FontSizeRelative(1.25),
        brush: text::Brush::color(brush.r(), brush.g(), brush.b(), brush.a()),
        ..text::Style::default()
    }
}

trait StyleExt {
    fn with_weight(self, weight: text::Weight) -> Self;
    fn with_underline(self, brush: render::Color) -> Self;
    fn with_strikethrough(self, brush: render::Color) -> Self;
}

impl StyleExt for text::Style {
    fn with_weight(mut self, weight: text::Weight) -> Self {
        self.font.weight = weight;
        self
    }

    fn with_underline(mut self, brush: render::Color) -> Self {
        self.underline = text::Decoration::solid(Some(text::Brush::color(
            brush.r(),
            brush.g(),
            brush.b(),
            brush.a(),
        )));
        self
    }

    fn with_strikethrough(mut self, brush: render::Color) -> Self {
        self.strikethrough = text::Decoration::solid(Some(text::Brush::color(
            brush.r(),
            brush.g(),
            brush.b(),
            brush.a(),
        )));
        self
    }
}

fn color(r: u8, g: u8, b: u8, a: u8) -> render::Color {
    render_color_rgba(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    )
}

fn range_of(copy: &str, needle: &str) -> text::Range {
    let start = copy
        .find(needle)
        .expect("dev scenario range needle should exist");
    text::Range::new(start, start + needle.len())
}

fn checker_image() -> render::Result<render::Image> {
    let mut rgba = Vec::new();
    for y in 0..32 {
        for x in 0..32 {
            let light = (x / 8 + y / 8) % 2 == 0;
            rgba.extend_from_slice(if light {
                &[0xFF, 0xFF, 0xFF, 0xFF]
            } else {
                &[0x32, 0x49, 0x61, 0xFF]
            });
        }
    }
    render::Image::from_rgba(render_size(32.0, 32.0), rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenarios_are_indexed_stably() {
        assert_eq!(Scenario::from_index(0), Scenario::TextBasics);
        assert_eq!(Scenario::from_index(4), Scenario::ShapeGeometry);
        assert_eq!(Scenario::from_index(5), Scenario::RetainedModel);
        assert_eq!(Scenario::from_index(6), Scenario::WindowState);
        assert_eq!(Scenario::from_index(7), Scenario::TextBasics);
    }

    #[test]
    fn scenario_tabs_are_hit_testable() {
        assert_eq!(scenario_at_point(50.0, 140.0), Some(0));
        assert_eq!(scenario_at_point(50.0 + TAB_STEP * 2.0, 140.0), Some(2));
        assert_eq!(scenario_at_point(10.0, 140.0), None);
        assert_eq!(scenario_at_point(50.0, 190.0), None);
    }

    #[test]
    fn retained_harness_hit_targets_are_stable() {
        let mut state = DevState::default();
        state.select(5);
        assert_eq!(
            retained_action_at_point(RETAINED_ACTION_X + 4.0, RETAINED_ACTION_Y + 4.0),
            Some(RetainedAction::Reset)
        );
        assert_eq!(
            retained_node_at_point(&state, RETAINED_TREE_X + 8.0, RETAINED_TREE_Y + 4.0),
            Some(state.retained.model.root())
        );
    }

    #[test]
    fn scenes_build_for_all_scenarios() {
        let mut system = text::System::default();
        for index in 0..SCENARIO_COUNT {
            let mut state = DevState::default();
            state.select(index);
            state.push_event("test event");
            let _scene = build_scene(
                &mut system,
                &state,
                WindowFacts {
                    logical_size: render_size(1024.0, 720.0),
                    scale_factor: 1.0,
                    focused: true,
                },
            );
        }
    }

    #[test]
    fn shape_geometry_scene_renders_headless() {
        let mut system = text::System::default();
        let mut state = DevState::default();
        state.select(4);
        let scene = build_scene(
            &mut system,
            &state,
            WindowFacts {
                logical_size: render_size(1024.0, 720.0),
                scale_factor: 1.0,
                focused: true,
            },
        );
        let mut renderer = pollster::block_on(render::Renderer::new(render::Options::default()))
            .expect("dev harness renderer should initialize");
        let mut surface = renderer
            .create_headless(render_size(1024.0, 720.0), 1.0)
            .expect("dev harness surface should initialize");

        renderer
            .render(&mut surface, &scene, render::Parameters::default())
            .expect("shape geometry scene should render without unsupported path strokes");
    }
}
