use super::{
    Controls, DrawScheduler, Error, ErrorCode, Fullscreen, Level, Rect, Result, Role, Theme,
    WindowRequest,
    planning::{ResolvedImePurpose, ResolvedImeRequest},
};
use std::time::Instant;

pub(crate) fn window_attributes_from_request(
    request: &WindowRequest,
) -> Result<winit::window::WindowAttributes> {
    if !matches!(request.role(), Role::Root) {
        return Err(Error::new(
            ErrorCode::UnsupportedFeature,
            "native window roles require parent and modality wiring",
        ));
    }

    let mut attributes = winit::window::Window::default_attributes()
        .with_title(request.title().to_owned())
        .with_resizable(request.resizable())
        .with_enabled_buttons(request.controls().into())
        .with_decorations(request.decorations())
        .with_transparent(request.transparent())
        .with_visible(request.visible())
        .with_window_level(request.level().into())
        .with_theme(request.theme().map(Into::into));

    if let Some(position) = request.position() {
        attributes =
            attributes.with_position(winit::dpi::LogicalPosition::new(position.x, position.y));
    }
    if let Some(size) = request.inner_size() {
        attributes =
            attributes.with_inner_size(winit::dpi::LogicalSize::new(size.width, size.height));
    }
    if let Some(size) = request.min_inner_size() {
        attributes =
            attributes.with_min_inner_size(winit::dpi::LogicalSize::new(size.width, size.height));
    }
    if let Some(size) = request.max_inner_size() {
        attributes =
            attributes.with_max_inner_size(winit::dpi::LogicalSize::new(size.width, size.height));
    }

    attributes = match request.fullscreen() {
        Fullscreen::None => attributes.with_fullscreen(None),
        Fullscreen::Borderless => {
            attributes.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        }
        Fullscreen::Exclusive => {
            return Err(Error::new(
                ErrorCode::CommandFailed,
                "exclusive fullscreen requires a native video mode",
            ));
        }
    };

    Ok(attributes)
}

pub(crate) fn control_flow_from_draw_scheduler(
    draw: &DrawScheduler,
) -> winit::event_loop::ControlFlow {
    native_control_flow(draw.next_deadline().map_or(
        winit::event_loop::ControlFlow::Wait,
        winit::event_loop::ControlFlow::WaitUntil,
    ))
}

pub(crate) fn native_control_flow(
    control_flow: winit::event_loop::ControlFlow,
) -> winit::event_loop::ControlFlow {
    #[cfg(target_os = "macos")]
    {
        if matches!(control_flow, winit::event_loop::ControlFlow::Wait) {
            return winit::event_loop::ControlFlow::WaitUntil(
                Instant::now() + std::time::Duration::from_secs(60 * 60),
            );
        }
    }

    control_flow
}

/// One ordered native IME operation derived from a resolved request.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NativeImeOperation {
    Allowed(bool),
    Purpose(ResolvedImePurpose),
    CursorArea(Rect),
}

/// Lowers a capability-resolved IME request into the exact winit operation order.
pub(crate) fn native_ime_operations(request: &ResolvedImeRequest) -> Vec<NativeImeOperation> {
    let mut operations = Vec::new();
    match request {
        ResolvedImeRequest::Disable => operations.push(NativeImeOperation::Allowed(false)),
        ResolvedImeRequest::Enable(config) => {
            operations.push(NativeImeOperation::Allowed(true));
            append_ime_configuration(&mut operations, config);
        }
        ResolvedImeRequest::Update(config) => append_ime_configuration(&mut operations, config),
        ResolvedImeRequest::Restart(config) => {
            operations.push(NativeImeOperation::Allowed(false));
            operations.push(NativeImeOperation::Allowed(true));
            append_ime_configuration(&mut operations, config);
        }
    }
    operations
}

fn append_ime_configuration(
    operations: &mut Vec<NativeImeOperation>,
    config: &super::planning::ResolvedImeConfig,
) {
    operations.push(NativeImeOperation::Purpose(config.purpose));
    if let Some(cursor_area) = config.cursor_area {
        operations.push(NativeImeOperation::CursorArea(cursor_area));
    }
}

/// Applies one resolved IME request through winit without reopening authored semantics.
pub(crate) fn apply_native_ime_request(
    window: &winit::window::Window,
    request: &ResolvedImeRequest,
) {
    for operation in native_ime_operations(request) {
        match operation {
            NativeImeOperation::Allowed(allowed) => window.set_ime_allowed(allowed),
            NativeImeOperation::Purpose(purpose) => window.set_ime_purpose(match purpose {
                ResolvedImePurpose::Normal => winit::window::ImePurpose::Normal,
                ResolvedImePurpose::Password => winit::window::ImePurpose::Password,
                ResolvedImePurpose::Terminal => winit::window::ImePurpose::Terminal,
            }),
            NativeImeOperation::CursorArea(cursor_area) => window.set_ime_cursor_area(
                winit::dpi::LogicalPosition::new(cursor_area.origin.x, cursor_area.origin.y),
                winit::dpi::LogicalSize::new(cursor_area.size.width, cursor_area.size.height),
            ),
        }
    }
}

impl From<Controls> for winit::window::WindowButtons {
    fn from(controls: Controls) -> Self {
        let mut buttons = Self::empty();
        if controls.close {
            buttons |= Self::CLOSE;
        }
        if controls.minimize {
            buttons |= Self::MINIMIZE;
        }
        if controls.maximize {
            buttons |= Self::MAXIMIZE;
        }
        buttons
    }
}

impl From<Level> for winit::window::WindowLevel {
    fn from(level: Level) -> Self {
        match level {
            Level::Normal => Self::Normal,
            Level::AlwaysOnTop => Self::AlwaysOnTop,
            Level::AlwaysOnBottom => Self::AlwaysOnBottom,
        }
    }
}

impl From<Theme> for winit::window::Theme {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => Self::Light,
            Theme::Dark => Self::Dark,
        }
    }
}
