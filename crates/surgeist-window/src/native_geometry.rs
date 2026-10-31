//! Host-authorized CSS-pixel native geometry requests.
//!
//! The retained CSSOM View §4 browser algorithms require a script-created
//! auxiliary browsing context. Ordinary native windows do not meet that
//! predicate. This module instead uses an explicit native host policy, as
//! selected for #858; a future browser binding must still enforce the predicate.
//! Resolution captures the supplied snapshot and coordinate basis. It does not
//! reread the window when its resulting command is eventually dispatched, and
//! approval is never evidence that the operating system applied the geometry.
//!
//! Source: [CSSOM View, 16 September 2025, §4 and §4.1](https://www.w3.org/TR/2025/WD-cssom-view-1-20250916/#extensions-to-the-window-interface).

use super::geometry::{normalize_nonnegative_size, normalize_outer_position};
use super::{
    Command, Error, ErrorCode, Point, Rect, Result, Size, WindowRequestBuilder, WindowSnapshot,
};

/// Captured mapping from the selected output device's CSS pixels to desktop
/// logical coordinates. Positive axes point rightward and downward.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeGeometryBasis {
    origin: Point,
    units_per_css_pixel: f64,
}

impl NativeGeometryBasis {
    /// Validates an output-device origin and a finite positive CSS-pixel scale.
    /// Neither display scale nor page zoom is implicitly substituted for it.
    pub fn new(origin: Point, logical_units_per_css_pixel: f64) -> Result<Self> {
        let origin = normalize_outer_position(origin)?;
        if !logical_units_per_css_pixel.is_finite() || logical_units_per_css_pixel <= 0.0 {
            return Err(invalid("CSS-pixel scale must be finite and positive"));
        }
        Ok(Self {
            origin,
            units_per_css_pixel: logical_units_per_css_pixel,
        })
    }
}

/// Explicit native host admission policy, independent of backend capability.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeGeometryAuthorization(Admission);

#[derive(Clone, Copy, Debug, PartialEq)]
enum Admission {
    Deny,
    Allow,
    Clamp(Rect),
}

impl NativeGeometryAuthorization {
    /// Refuses the operation without producing a command.
    #[must_use]
    pub const fn deny() -> Self {
        Self(Admission::Deny)
    }

    /// Admits geometry without an available-area constraint. Negative requested
    /// viewport dimensions are clamped to zero by native policy.
    #[must_use]
    pub const fn allow() -> Self {
        Self(Admission::Allow)
    }

    /// Admits requests bounded by this desktop-logical available rectangle.
    /// Movement keeps the observed outer extent inside it where possible;
    /// resizing preserves the observed decoration extent when bounding size.
    /// A window larger than the area is aligned to its leading edge.
    pub fn clamp_to(area: Rect) -> Result<Self> {
        Ok(Self(Admission::Clamp(valid_rect(area)?)))
    }
}

/// A typed native request using signed CSS-pixel integer arguments.
///
/// `resize_to` targets viewport dimensions, as specified by the retained CSSOM
/// View §4 step 5. `resize_by` moves right/bottom edges by deltas, represented as
/// an equal inner-size delta with the captured decoration extent held fixed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeGeometryRequest {
    /// Absolute outer position relative to the selected output-device origin.
    MoveTo {
        /// Horizontal CSS coordinate.
        x: i32,
        /// Vertical CSS coordinate.
        y: i32,
    },
    /// Outer-position displacement from the captured observed position.
    MoveBy {
        /// Rightward CSS displacement.
        x: i32,
        /// Downward CSS displacement.
        y: i32,
    },
    /// Absolute viewport dimensions.
    ResizeTo {
        /// CSS viewport width.
        width: i32,
        /// CSS viewport height.
        height: i32,
    },
    /// Viewport-size displacement from the captured observed inner size.
    ResizeBy {
        /// CSS width displacement.
        x: i32,
        /// CSS height displacement.
        y: i32,
    },
}

/// Reason a native request produced no command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeGeometryNoAction {
    /// Host admission was denied; no backend operation was requested.
    Denied,
    /// Relative movement or containment needs unavailable outer observations.
    UnavailableGeometry,
    /// Outer dimensions are smaller than the observed viewport dimensions.
    InconsistentGeometry,
    /// Resolved requested geometry equals the captured observation.
    Unchanged,
}

/// A resolved request's admission result, never an operating-system observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeGeometryDisposition {
    /// A command is available for the ordinary lifecycle/capability planner.
    Requested {
        /// Whether native policy changed requested geometry.
        clamped: bool,
    },
    /// No backend request is available.
    NoAction(NativeGeometryNoAction),
}

/// Captured geometry resolution. Commands still require ordinary normalization,
/// lifecycle validation, capability planning, and backend application.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeGeometryOutcome {
    disposition: NativeGeometryDisposition,
    command: Option<Command>,
}

impl NativeGeometryOutcome {
    /// Returns the admission result independently of eventual host observation.
    #[must_use]
    pub const fn disposition(&self) -> NativeGeometryDisposition {
        self.disposition
    }

    /// Returns approved logical geometry through the canonical command surface.
    #[must_use]
    pub const fn command(&self) -> Option<&Command> {
        self.command.as_ref()
    }

    /// Takes the approved command, or `None` for a no-action result.
    #[must_use]
    pub fn into_command(self) -> Option<Command> {
        self.command
    }

    fn no_action(reason: NativeGeometryNoAction) -> Self {
        Self {
            disposition: NativeGeometryDisposition::NoAction(reason),
            command: None,
        }
    }

    fn requested(command: Command, clamped: bool) -> Self {
        Self {
            disposition: NativeGeometryDisposition::Requested { clamped },
            command: Some(command),
        }
    }
}

impl NativeGeometryRequest {
    /// Authors an absolute CSS-pixel outer-position request.
    #[must_use]
    pub const fn move_to(x: i32, y: i32) -> Self {
        Self::MoveTo { x, y }
    }
    /// Authors a relative CSS-pixel outer-position request.
    #[must_use]
    pub const fn move_by(x: i32, y: i32) -> Self {
        Self::MoveBy { x, y }
    }
    /// Authors absolute CSS-pixel viewport dimensions.
    #[must_use]
    pub const fn resize_to(width: i32, height: i32) -> Self {
        Self::ResizeTo { width, height }
    }
    /// Authors relative CSS-pixel viewport-size deltas.
    #[must_use]
    pub const fn resize_by(x: i32, y: i32) -> Self {
        Self::ResizeBy { x, y }
    }

    /// Resolves against a captured immutable observation and explicit host policy.
    /// Clamping needs observed outer size to account for native decorations;
    /// missing observations yield no action instead of assuming zero decoration.
    /// Arithmetic outside finite logical geometry returns `InvalidRequest`.
    pub fn resolve_against(
        self,
        snapshot: &WindowSnapshot,
        basis: NativeGeometryBasis,
        authorization: NativeGeometryAuthorization,
    ) -> Result<NativeGeometryOutcome> {
        use NativeGeometryNoAction::{
            Denied, InconsistentGeometry, UnavailableGeometry, Unchanged,
        };
        if matches!(authorization.0, Admission::Deny) {
            return Ok(NativeGeometryOutcome::no_action(Denied));
        }
        let metrics = snapshot.metrics();
        let id = snapshot.id();
        let scale = basis.units_per_css_pixel;
        match self {
            Self::MoveTo { x, y } | Self::MoveBy { x, y } => {
                let origin = if matches!(self, Self::MoveTo { .. }) {
                    basis.origin
                } else if let Some(position) = metrics.outer_position() {
                    position
                } else {
                    return Ok(NativeGeometryOutcome::no_action(UnavailableGeometry));
                };
                let requested = normalize_outer_position(Point {
                    x: origin.x + f64::from(x) * scale,
                    y: origin.y + f64::from(y) * scale,
                })?;
                let mut position = requested;
                if let Admission::Clamp(area) = authorization.0 {
                    let Some(outer) = metrics.outer_size() else {
                        return Ok(NativeGeometryOutcome::no_action(UnavailableGeometry));
                    };
                    if outer.width < metrics.logical_size().width
                        || outer.height < metrics.logical_size().height
                    {
                        return Ok(NativeGeometryOutcome::no_action(InconsistentGeometry));
                    }
                    position = clamp_position(position, outer, area);
                }
                if Some(position) == metrics.outer_position() {
                    return Ok(NativeGeometryOutcome::no_action(Unchanged));
                }
                Ok(NativeGeometryOutcome::requested(
                    Command::SetPosition { id, position },
                    position != requested,
                ))
            }
            Self::ResizeTo {
                width: x,
                height: y,
            }
            | Self::ResizeBy { x, y } => {
                let current = metrics.logical_size();
                let origin = if matches!(self, Self::ResizeBy { .. }) {
                    current
                } else {
                    Size::default()
                };
                let width = origin.width + f64::from(x) * scale;
                let height = origin.height + f64::from(y) * scale;
                ensure_finite(width, height)?;
                let requested = Size { width, height };
                let mut size = Size {
                    width: width.max(0.0),
                    height: height.max(0.0),
                };
                if let Admission::Clamp(area) = authorization.0 {
                    let Some(outer) = metrics.outer_size() else {
                        return Ok(NativeGeometryOutcome::no_action(UnavailableGeometry));
                    };
                    if outer.width < current.width || outer.height < current.height {
                        return Ok(NativeGeometryOutcome::no_action(InconsistentGeometry));
                    }
                    size.width = size
                        .width
                        .min((area.size.width - (outer.width - current.width)).max(0.0));
                    size.height = size
                        .height
                        .min((area.size.height - (outer.height - current.height)).max(0.0));
                }
                if size == current {
                    return Ok(NativeGeometryOutcome::no_action(Unchanged));
                }
                Ok(NativeGeometryOutcome::requested(
                    Command::SetInnerSize { id, size },
                    size != requested,
                ))
            }
        }
    }
}

/// The four CSSOM View §4.1 features, parsed from an already-tokenized feature
/// map. HTML feature-string tokenization and alias handling belong to HTML.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NativeOpenFeatures {
    left: Option<f64>,
    top: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
}

impl NativeOpenFeatures {
    /// Parses only exact `left`, `top`, `width`, and `height` names. Unknown names
    /// are ignored. Repeated supported tokens replace their earlier value.
    /// HTML integer parsing skips ASCII whitespace, accepts an optional sign,
    /// and consumes the initial ASCII digit run; syntax errors become zero.
    /// Integers are lowered to logical `f64` precision. Values beyond finite
    /// logical representation are rejected when resolving, not treated as zero.
    #[must_use]
    pub fn from_tokens<'a>(tokens: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut result = Self::default();
        for (name, value) in tokens {
            let field = match name {
                "left" => &mut result.left,
                "top" => &mut result.top,
                "width" => &mut result.width,
                "height" => &mut result.height,
                _ => continue,
            };
            *field = Some(parse_integer(value));
        }
        result
    }

    /// Resolves initial features into a normal open command. `fallback` is the
    /// host-selected desktop-logical outer position and viewport size for
    /// unspecified axes; existing builder geometry takes precedence over it.
    /// Width/height zero skips that axis independently. Negative dimensions are
    /// clamped to zero. Available-area clamping bounds initial viewport size and
    /// leading position, without claiming knowledge of not-yet-created native
    /// decorations or guaranteeing final whole-window containment.
    pub fn resolve_open(
        self,
        mut builder: WindowRequestBuilder,
        fallback: Rect,
        basis: NativeGeometryBasis,
        authorization: NativeGeometryAuthorization,
    ) -> Result<NativeGeometryOutcome> {
        if matches!(authorization.0, Admission::Deny) {
            return Ok(NativeGeometryOutcome::no_action(
                NativeGeometryNoAction::Denied,
            ));
        }
        let fallback = valid_rect(fallback)?;
        let base_position = builder.request.position().unwrap_or(fallback.origin);
        let base_size = builder.request.inner_size().unwrap_or(fallback.size);
        let mut position = Point {
            x: self.left.map_or(base_position.x, |x| {
                basis.origin.x + x * basis.units_per_css_pixel
            }),
            y: self.top.map_or(base_position.y, |y| {
                basis.origin.y + y * basis.units_per_css_pixel
            }),
        };
        position = normalize_outer_position(position)?;
        let width = self.width.filter(|value| *value != 0.0);
        let height = self.height.filter(|value| *value != 0.0);
        let mut size = Size {
            width: width.map_or(base_size.width, |x| x * basis.units_per_css_pixel),
            height: height.map_or(base_size.height, |y| y * basis.units_per_css_pixel),
        };
        ensure_finite(size.width, size.height)?;
        let original = (position, size);
        if width.is_some() {
            size.width = size.width.max(0.0);
        }
        if height.is_some() {
            size.height = size.height.max(0.0);
        }
        if let Admission::Clamp(area) = authorization.0 {
            if width.is_some() {
                size.width = size.width.min(area.size.width);
            }
            if height.is_some() {
                size.height = size.height.min(area.size.height);
            }
            let bounded = clamp_position(position, size, area);
            if self.left.is_some() {
                position.x = bounded.x;
            }
            if self.top.is_some() {
                position.y = bounded.y;
            }
        }
        if self.left.is_some() || self.top.is_some() {
            builder = builder.position(position);
        }
        if width.is_some() || height.is_some() {
            builder = builder.inner_size(size);
        }
        Ok(NativeGeometryOutcome::requested(
            Command::Open {
                request: builder.build(),
            },
            original != (position, size),
        ))
    }
}

fn parse_integer(value: &str) -> f64 {
    let value = value.trim_start_matches(['\t', '\n', '\u{000c}', '\r', ' ']);
    let (negative, digits) = if let Some(rest) = value.strip_prefix('-') {
        (true, rest)
    } else {
        (false, value.strip_prefix('+').unwrap_or(value))
    };
    let count = digits.bytes().take_while(u8::is_ascii_digit).count();
    if count == 0 {
        return 0.0;
    }
    let number: f64 = digits[..count].parse().unwrap_or(f64::INFINITY);
    if negative { -number } else { number }
}

fn invalid(message: &str) -> Error {
    Error::new(ErrorCode::InvalidRequest, message)
}
fn ensure_finite(x: f64, y: f64) -> Result<()> {
    if x.is_finite() && y.is_finite() {
        Ok(())
    } else {
        Err(invalid(
            "native CSS-pixel geometry exceeds finite logical coordinates",
        ))
    }
}
fn valid_rect(rect: Rect) -> Result<Rect> {
    let origin = normalize_outer_position(rect.origin)?;
    let size = normalize_nonnegative_size(rect.size, "available geometry size")?;
    ensure_finite(origin.x + size.width, origin.y + size.height)?;
    Ok(Rect { origin, size })
}
fn clamp_position(position: Point, size: Size, area: Rect) -> Point {
    Point {
        x: position.x.clamp(
            area.origin.x,
            area.origin.x + (area.size.width - size.width).max(0.0),
        ),
        y: position.y.clamp(
            area.origin.y,
            area.origin.y + (area.size.height - size.height).max(0.0),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Access, CapabilitySupport, FullscreenMode, Handler, HostCapabilities, Id, Metrics,
        PhysicalSize, RoleKind, WindowRequest, open, testing::Runner,
    };

    struct Quiet;
    impl Handler for Quiet {}

    fn basis() -> NativeGeometryBasis {
        NativeGeometryBasis::new(
            Point {
                x: -1000.0,
                y: 20.0,
            },
            2.0,
        )
        .unwrap()
    }
    fn snapshot() -> WindowSnapshot {
        let metrics = Metrics::from_physical_size(
            Id::from_u64(1),
            PhysicalSize {
                width: 800,
                height: 600,
            },
            2.0,
        )
        .unwrap()
        .with_outer_geometry(
            Some(Point {
                x: -900.0,
                y: 100.0,
            }),
            Some(Size {
                width: 416.0,
                height: 338.0,
            }),
        )
        .unwrap();
        WindowSnapshot::new("fixture", metrics)
    }
    fn runner() -> Runner<Quiet> {
        let mut runner = Runner::new(Quiet);
        runner
            .startup(vec![
                open("native")
                    .position(Point {
                        x: -900.0,
                        y: 100.0,
                    })
                    .inner_size(Size {
                        width: 400.0,
                        height: 300.0,
                    })
                    .into(),
            ])
            .unwrap();
        runner.resume();
        assert!(runner.result().is_none());
        runner
    }
    fn resolve(request: NativeGeometryRequest) -> NativeGeometryOutcome {
        request
            .resolve_against(&snapshot(), basis(), NativeGeometryAuthorization::allow())
            .unwrap()
    }
    fn apply(runner: &mut Runner<Quiet>, outcome: NativeGeometryOutcome) {
        if let Some(command) = outcome.into_command() {
            runner.dispatch(command);
        }
    }
    fn fallback() -> Rect {
        Rect {
            origin: Point {
                x: -900.0,
                y: 100.0,
            },
            size: Size {
                width: 400.0,
                height: 300.0,
            },
        }
    }
    fn open_features(tokens: &[(&str, &str)]) -> NativeGeometryOutcome {
        NativeOpenFeatures::from_tokens(tokens.iter().copied())
            .resolve_open(
                WindowRequest::builder("features"),
                fallback(),
                basis(),
                NativeGeometryAuthorization::allow(),
            )
            .unwrap()
    }
    fn open_request(outcome: &NativeGeometryOutcome) -> &WindowRequest {
        let Some(Command::Open { request }) = outcome.command() else {
            panic!("expected admitted open request")
        };
        request
    }

    #[test]
    fn move_absolute_and_relative_use_captured_css_basis_through_runner() {
        let cases = [
            (
                NativeGeometryRequest::move_to(20, 30),
                Point { x: -960.0, y: 80.0 },
            ),
            (
                NativeGeometryRequest::move_by(20, 30),
                Point {
                    x: -860.0,
                    y: 160.0,
                },
            ),
        ];
        for (request, expected) in cases {
            let mut runner = runner();
            let outcome = resolve(request);
            assert_eq!(
                outcome.command(),
                Some(&Command::SetPosition {
                    id: Id::from_u64(1),
                    position: expected
                })
            );
            apply(&mut runner, outcome);
            assert_eq!(
                runner
                    .host()
                    .registry()
                    .get(Id::from_u64(1))
                    .unwrap()
                    .metrics()
                    .outer_position(),
                Some(expected)
            );
            assert!(runner.result().is_none());
        }
    }

    #[test]
    fn resize_absolute_targets_viewport_while_relative_adds_to_it_through_runner() {
        let cases = [
            (
                NativeGeometryRequest::resize_to(200, 100),
                Size {
                    width: 400.0,
                    height: 200.0,
                },
            ),
            (
                NativeGeometryRequest::resize_by(200, 100),
                Size {
                    width: 800.0,
                    height: 500.0,
                },
            ),
            (
                NativeGeometryRequest::resize_to(-10, 100),
                Size {
                    width: 0.0,
                    height: 200.0,
                },
            ),
            (
                NativeGeometryRequest::resize_by(-250, -200),
                Size {
                    width: 0.0,
                    height: 0.0,
                },
            ),
        ];
        for (request, expected) in cases {
            let mut runner = runner();
            let outcome = resolve(request);
            assert_eq!(
                outcome.command(),
                Some(&Command::SetInnerSize {
                    id: Id::from_u64(1),
                    size: expected
                })
            );
            apply(&mut runner, outcome);
            assert_eq!(
                runner
                    .host()
                    .registry()
                    .get(Id::from_u64(1))
                    .unwrap()
                    .metrics()
                    .logical_size(),
                expected
            );
            assert!(runner.result().is_none());
        }
    }

    #[test]
    fn denied_and_unchanged_operations_have_no_runner_effect() {
        let mut runner = runner();
        let original_count = runner.host().commands().len();
        let original_metrics = runner
            .host()
            .registry()
            .get(Id::from_u64(1))
            .unwrap()
            .metrics();
        let requests = [
            NativeGeometryRequest::move_to(50, 40),
            NativeGeometryRequest::move_by(0, 0),
            NativeGeometryRequest::resize_to(200, 150),
            NativeGeometryRequest::resize_by(0, 0),
        ];
        for request in requests {
            let denied = request
                .resolve_against(&snapshot(), basis(), NativeGeometryAuthorization::deny())
                .unwrap();
            assert_eq!(
                denied.disposition(),
                NativeGeometryDisposition::NoAction(NativeGeometryNoAction::Denied)
            );
            apply(&mut runner, denied);
            let unchanged = resolve(request);
            assert_eq!(
                unchanged.disposition(),
                NativeGeometryDisposition::NoAction(NativeGeometryNoAction::Unchanged)
            );
            apply(&mut runner, unchanged);
        }
        assert_eq!(runner.host().commands().len(), original_count);
        assert_eq!(
            runner
                .host()
                .registry()
                .get(Id::from_u64(1))
                .unwrap()
                .metrics(),
            original_metrics
        );
        assert!(runner.result().is_none());
    }

    #[test]
    fn admitted_request_still_requires_backend_capability() {
        let report = HostCapabilities::builder()
            .role(RoleKind::Root, CapabilitySupport::Supported)
            .fullscreen(FullscreenMode::None, CapabilitySupport::Supported)
            .build();
        let mut runner = Runner::with_capabilities(Quiet, report);
        runner
            .startup(vec![Command::Open {
                request: WindowRequest::default(),
            }])
            .unwrap();
        runner.resume();
        assert!(runner.result().is_none());
        apply(&mut runner, resolve(NativeGeometryRequest::move_to(1, 2)));
        assert_eq!(
            runner.result().unwrap().unwrap_err().code,
            ErrorCode::UnsupportedFeature
        );
        assert_eq!(runner.host().commands().len(), 1);
    }

    #[test]
    fn canonical_planning_retains_size_constraints_after_geometry_admission() {
        let mut runner = runner();
        runner.dispatch(Command::SetMaxInnerSize {
            id: Id::from_u64(1),
            size: Some(Size {
                width: 500.0,
                height: 350.0,
            }),
        });
        apply(
            &mut runner,
            resolve(NativeGeometryRequest::resize_by(200, 100)),
        );
        assert_eq!(
            runner
                .host()
                .registry()
                .get(Id::from_u64(1))
                .unwrap()
                .metrics()
                .logical_size(),
            Size {
                width: 500.0,
                height: 350.0
            }
        );
    }

    #[test]
    fn clamping_accounts_for_observed_decorations_and_negative_screen_origin() {
        let area = Rect {
            origin: Point { x: -1000.0, y: 0.0 },
            size: Size {
                width: 600.0,
                height: 400.0,
            },
        };
        let authorization = NativeGeometryAuthorization::clamp_to(area).unwrap();
        let moved = NativeGeometryRequest::move_to(2000, -40)
            .resolve_against(&snapshot(), basis(), authorization)
            .unwrap();
        assert_eq!(
            moved.command(),
            Some(&Command::SetPosition {
                id: Id::from_u64(1),
                position: Point { x: -816.0, y: 0.0 }
            })
        );
        assert_eq!(
            moved.disposition(),
            NativeGeometryDisposition::Requested { clamped: true }
        );
        let resized = NativeGeometryRequest::resize_to(1000, 1000)
            .resolve_against(&snapshot(), basis(), authorization)
            .unwrap();
        assert_eq!(
            resized.command(),
            Some(&Command::SetInnerSize {
                id: Id::from_u64(1),
                size: Size {
                    width: 584.0,
                    height: 362.0
                }
            })
        );
        assert_eq!(
            resized.disposition(),
            NativeGeometryDisposition::Requested { clamped: true }
        );
    }

    #[test]
    fn absent_and_inconsistent_observations_are_explicit_no_actions() {
        let missing = WindowSnapshot::new(
            "missing",
            Metrics::from_physical_size(
                Id::from_u64(1),
                PhysicalSize {
                    width: 400,
                    height: 300,
                },
                1.0,
            )
            .unwrap(),
        );
        let area = NativeGeometryAuthorization::clamp_to(fallback()).unwrap();
        for request in [
            NativeGeometryRequest::move_by(1, 2),
            NativeGeometryRequest::move_to(1, 2),
            NativeGeometryRequest::resize_to(1, 2),
        ] {
            let result = request.resolve_against(&missing, basis(), area).unwrap();
            assert_eq!(
                result.disposition(),
                NativeGeometryDisposition::NoAction(NativeGeometryNoAction::UnavailableGeometry)
            );
        }
        assert_eq!(
            NativeGeometryRequest::move_by(1, 2)
                .resolve_against(&missing, basis(), NativeGeometryAuthorization::allow())
                .unwrap()
                .disposition(),
            NativeGeometryDisposition::NoAction(NativeGeometryNoAction::UnavailableGeometry)
        );
        let inconsistent = WindowSnapshot::new(
            "inconsistent",
            missing
                .metrics()
                .clone()
                .with_outer_geometry(
                    Some(Point::default()),
                    Some(Size {
                        width: 10.0,
                        height: 10.0,
                    }),
                )
                .unwrap(),
        );
        for request in [
            NativeGeometryRequest::move_to(1, 2),
            NativeGeometryRequest::resize_by(1, 2),
        ] {
            assert_eq!(
                request
                    .resolve_against(&inconsistent, basis(), area)
                    .unwrap()
                    .disposition(),
                NativeGeometryDisposition::NoAction(NativeGeometryNoAction::InconsistentGeometry)
            );
        }
    }

    #[test]
    fn feature_integer_parsing_follows_html_prefix_and_ascii_whitespace_rules() {
        for (input, expected) in [
            (" \t\n\r\u{c}+12tail", 12.0),
            ("-12.9", -12.0),
            ("12e3", 12.0),
            ("0x10", 0.0),
            ("+", 0.0),
            ("- 3", 0.0),
            ("", 0.0),
            ("garbage", 0.0),
            ("\u{a0}12", 0.0),
            ("\u{b}12", 0.0),
            ("１２", 0.0),
        ] {
            let outcome = open_features(&[("left", input), ("top", input)]);
            assert_eq!(
                open_request(&outcome).position(),
                Some(Point {
                    x: -1000.0 + expected * 2.0,
                    y: 20.0 + expected * 2.0
                })
            );
        }
    }

    #[test]
    fn only_four_exact_feature_names_are_consumed_and_later_tokens_replace_earlier() {
        let ignored = open_features(&[
            ("screenX", "17"),
            ("innerWidth", "18"),
            ("WIDTH", "19"),
            ("popup", "yes"),
        ]);
        assert_eq!(open_request(&ignored).position(), None);
        assert_eq!(open_request(&ignored).inner_size(), None);
        let replaced = open_features(&[("width", "20"), ("width", "31suffix"), ("height", "15")]);
        assert_eq!(
            open_request(&replaced).inner_size(),
            Some(Size {
                width: 62.0,
                height: 30.0
            })
        );
    }

    #[test]
    fn zero_and_invalid_feature_dimensions_skip_each_axis_independently() {
        for token in ["0", "-0", "invalid", ""] {
            let width = open_features(&[("width", token), ("height", "25")]);
            assert_eq!(
                open_request(&width).inner_size(),
                Some(Size {
                    width: 400.0,
                    height: 50.0
                })
            );
            let height = open_features(&[("width", "25"), ("height", token)]);
            assert_eq!(
                open_request(&height).inner_size(),
                Some(Size {
                    width: 50.0,
                    height: 300.0
                })
            );
            let both = open_features(&[("width", token), ("height", token)]);
            assert_eq!(open_request(&both).inner_size(), None);
        }
    }

    #[test]
    fn open_features_flow_through_runner_with_independent_axes_and_native_clamping() {
        let authorization = NativeGeometryAuthorization::clamp_to(fallback()).unwrap();
        let features = NativeOpenFeatures::from_tokens([
            ("left", "-2000"),
            ("top", "500"),
            ("width", "800"),
            ("height", "0"),
        ]);
        let outcome = features
            .resolve_open(
                WindowRequest::builder("features"),
                fallback(),
                basis(),
                authorization,
            )
            .unwrap();
        assert_eq!(
            outcome.disposition(),
            NativeGeometryDisposition::Requested { clamped: true }
        );
        let request = open_request(&outcome);
        assert_eq!(request.position(), Some(fallback().origin));
        assert_eq!(request.inner_size(), Some(fallback().size));
        let mut runner = Runner::new(Quiet);
        runner
            .startup(vec![outcome.into_command().unwrap()])
            .unwrap();
        runner.resume();
        let metrics = runner
            .host()
            .registry()
            .get(Id::from_u64(1))
            .unwrap()
            .metrics();
        assert_eq!(metrics.logical_size(), fallback().size);
        assert_eq!(metrics.outer_position(), Some(fallback().origin));
        assert!(runner.result().is_none());
    }

    #[test]
    fn denied_open_features_do_not_create_a_window() {
        let outcome = NativeOpenFeatures::from_tokens([("width", "500")])
            .resolve_open(
                WindowRequest::builder("denied"),
                fallback(),
                basis(),
                NativeGeometryAuthorization::deny(),
            )
            .unwrap();
        assert_eq!(
            outcome.disposition(),
            NativeGeometryDisposition::NoAction(NativeGeometryNoAction::Denied)
        );
        let mut runner = Runner::new(Quiet);
        runner
            .startup(outcome.into_command().into_iter().collect())
            .unwrap();
        runner.resume();
        assert!(runner.host().window_id("denied").is_none());
        assert!(runner.host().commands().is_empty());
    }

    #[test]
    fn finite_mapping_area_and_arithmetic_are_validated() {
        for scale in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            assert!(NativeGeometryBasis::new(Point::default(), scale).is_err());
        }
        assert!(
            NativeGeometryBasis::new(
                Point {
                    x: f64::NAN,
                    y: 0.0
                },
                1.0
            )
            .is_err()
        );
        for area in [
            Rect {
                origin: Point::default(),
                size: Size {
                    width: -1.0,
                    height: 2.0,
                },
            },
            Rect {
                origin: Point {
                    x: f64::MAX,
                    y: 0.0,
                },
                size: Size {
                    width: f64::MAX,
                    height: 1.0,
                },
            },
        ] {
            assert!(NativeGeometryAuthorization::clamp_to(area).is_err());
        }
        let enormous = NativeGeometryBasis::new(Point::default(), f64::MAX).unwrap();
        for request in [
            NativeGeometryRequest::move_to(2, 1),
            NativeGeometryRequest::resize_by(2, 1),
        ] {
            assert!(
                request
                    .resolve_against(&snapshot(), enormous, NativeGeometryAuthorization::allow())
                    .is_err()
            );
        }
        let huge = "9".repeat(400);
        let features = NativeOpenFeatures::from_tokens([("width", huge.as_str())]);
        assert!(
            features
                .resolve_open(
                    WindowRequest::builder("huge"),
                    fallback(),
                    basis(),
                    NativeGeometryAuthorization::allow()
                )
                .is_err()
        );
    }

    #[test]
    fn feature_zero_preserves_existing_builder_axis_instead_of_fallback() {
        let builder = WindowRequest::builder("authored")
            .position(Point { x: 12.0, y: 13.0 })
            .inner_size(Size {
                width: 70.0,
                height: 90.0,
            });
        let outcome =
            NativeOpenFeatures::from_tokens([("left", "3"), ("height", "10"), ("width", "0")])
                .resolve_open(
                    builder,
                    fallback(),
                    basis(),
                    NativeGeometryAuthorization::allow(),
                )
                .unwrap();
        assert_eq!(
            open_request(&outcome).position(),
            Some(Point { x: -994.0, y: 13.0 })
        );
        assert_eq!(
            open_request(&outcome).inner_size(),
            Some(Size {
                width: 70.0,
                height: 20.0
            })
        );
    }
}
