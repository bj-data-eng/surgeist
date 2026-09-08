use super::*;
use kurbo::Shape as KurboShape;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PathBuilder {
    commands: Vec<Command>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    commands: Vec<Command>,
}

impl PathBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_to(&mut self, point: Point) -> &mut Self {
        self.commands.push(Command::MoveTo(point));
        self
    }

    pub fn line_to(&mut self, point: Point) -> &mut Self {
        self.commands.push(Command::LineTo(point));
        self
    }

    pub fn quad_to(&mut self, control: Point, end: Point) -> &mut Self {
        self.commands.push(Command::QuadTo { control, end });
        self
    }

    pub fn cubic_to(&mut self, control_a: Point, control_b: Point, end: Point) -> &mut Self {
        self.commands.push(Command::CubicTo {
            control_a,
            control_b,
            end,
        });
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.commands.push(Command::Close);
        self
    }

    pub fn build(self) -> Result<Path> {
        Path::from_commands(self.commands)
    }
}

impl Path {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn from_commands(commands: Vec<Command>) -> Result<Self> {
        validate_commands(&commands)?;
        Ok(Self { commands })
    }

    pub(crate) fn from_commands_unchecked(commands: Vec<Command>) -> Self {
        Self { commands }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    #[must_use]
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn validate(&self) -> Result<()> {
        validate_commands(&self.commands)
    }

    #[must_use]
    pub fn bounds(&self) -> Rect {
        if self.commands.is_empty() {
            return Rect::empty();
        }
        Rect::try_from(self.to_kurbo().bounding_box()).unwrap_or_default()
    }

    #[must_use]
    pub fn contains(&self, point: Point, fill_rule: FillRule) -> bool {
        let winding = self.to_kurbo().winding(point.to_kurbo());
        match fill_rule {
            FillRule::NonZero => winding != 0,
            FillRule::EvenOdd => winding % 2 != 0,
        }
    }

    #[must_use]
    pub fn to_kurbo(&self) -> kurbo::BezPath {
        let mut path = kurbo::BezPath::new();
        for command in &self.commands {
            match *command {
                Command::MoveTo(point) => path.move_to(point.to_kurbo()),
                Command::LineTo(point) => path.line_to(point.to_kurbo()),
                Command::QuadTo { control, end } => {
                    path.quad_to(control.to_kurbo(), end.to_kurbo());
                }
                Command::CubicTo {
                    control_a,
                    control_b,
                    end,
                } => path.curve_to(control_a.to_kurbo(), control_b.to_kurbo(), end.to_kurbo()),
                Command::Close => path.close_path(),
            }
        }
        path
    }

    #[must_use]
    pub fn key(&self, fill_rule: FillRule) -> Key {
        let mut state = StableHasher::new(0xaf63_bd4c_8601_b7df);
        self.hash_with_rule(&mut state, fill_rule);
        Key::from_parts(
            state.finish(),
            state.finish_with_seed(0x517c_c1b7_2722_0a95),
        )
    }

    pub(crate) fn from_kurbo(path: kurbo::BezPath) -> Self {
        let mut commands = Vec::new();
        for element in path.elements() {
            match *element {
                kurbo::PathEl::MoveTo(point) => {
                    commands.push(Command::MoveTo(Point::new_unchecked(point.x, point.y)));
                }
                kurbo::PathEl::LineTo(point) => {
                    commands.push(Command::LineTo(Point::new_unchecked(point.x, point.y)));
                }
                kurbo::PathEl::QuadTo(control, end) => {
                    commands.push(Command::QuadTo {
                        control: Point::new_unchecked(control.x, control.y),
                        end: Point::new_unchecked(end.x, end.y),
                    });
                }
                kurbo::PathEl::CurveTo(a, b, end) => {
                    commands.push(Command::CubicTo {
                        control_a: Point::new_unchecked(a.x, a.y),
                        control_b: Point::new_unchecked(b.x, b.y),
                        end: Point::new_unchecked(end.x, end.y),
                    });
                }
                kurbo::PathEl::ClosePath => {
                    commands.push(Command::Close);
                }
            }
        }
        match commands.first() {
            None => Self::empty(),
            Some(Command::MoveTo(_)) => Self::from_commands_unchecked(commands),
            Some(_) => Self::from_commands(commands).expect("kurbo path elements are ordered"),
        }
    }

    pub(crate) fn hash_with_rule(&self, state: &mut StableHasher, fill_rule: FillRule) {
        state.write_u8(match fill_rule {
            FillRule::NonZero => 1,
            FillRule::EvenOdd => 2,
        });
        state.write_usize(self.commands.len());
        for command in &self.commands {
            match *command {
                Command::MoveTo(point) => {
                    state.write_u8(1);
                    hash_point(state, point);
                }
                Command::LineTo(point) => {
                    state.write_u8(2);
                    hash_point(state, point);
                }
                Command::QuadTo { control, end } => {
                    state.write_u8(3);
                    hash_point(state, control);
                    hash_point(state, end);
                }
                Command::CubicTo {
                    control_a,
                    control_b,
                    end,
                } => {
                    state.write_u8(4);
                    hash_point(state, control_a);
                    hash_point(state, control_b);
                    hash_point(state, end);
                }
                Command::Close => state.write_u8(5),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    MoveTo(Point),
    LineTo(Point),
    QuadTo {
        control: Point,
        end: Point,
    },
    CubicTo {
        control_a: Point,
        control_b: Point,
        end: Point,
    },
    Close,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

fn validate_commands(commands: &[Command]) -> Result<()> {
    let mut moved = false;
    for command in commands {
        match *command {
            Command::MoveTo(point) => {
                point.validate("path move")?;
                moved = true;
            }
            Command::LineTo(point) => {
                if !moved {
                    return Err(Error::new(ErrorCode::InvalidPath, "path line before move"));
                }
                point.validate("path line")?;
            }
            Command::QuadTo { control, end } => {
                if !moved {
                    return Err(Error::new(ErrorCode::InvalidPath, "path quad before move"));
                }
                control.validate("path quad control")?;
                end.validate("path quad end")?;
            }
            Command::CubicTo {
                control_a,
                control_b,
                end,
            } => {
                if !moved {
                    return Err(Error::new(ErrorCode::InvalidPath, "path cubic before move"));
                }
                control_a.validate("path cubic control a")?;
                control_b.validate("path cubic control b")?;
                end.validate("path cubic end")?;
            }
            Command::Close => {
                if !moved {
                    return Err(Error::new(ErrorCode::InvalidPath, "path close before move"));
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn path_from_points(points: &[Point]) -> Path {
    let Some(first) = points.first() else {
        return Path::empty();
    };
    let mut commands = Vec::with_capacity(points.len());
    commands.push(Command::MoveTo(*first));
    for point in &points[1..] {
        commands.push(Command::LineTo(*point));
    }
    Path::from_commands_unchecked(commands)
}

pub(crate) fn path_polyline_length(path: &Path) -> f64 {
    let mut length = 0.0;
    let mut current = None;
    let mut start = None;
    for command in path.commands() {
        match *command {
            Command::MoveTo(point) => {
                current = Some(point);
                start = Some(point);
            }
            Command::LineTo(point) => {
                if let Some(prev) = current {
                    length += prev.distance_to(point);
                }
                current = Some(point);
            }
            Command::QuadTo { end, .. } | Command::CubicTo { end, .. } => {
                if let Some(prev) = current {
                    length += prev.distance_to(end);
                }
                current = Some(end);
            }
            Command::Close => {
                if let (Some(prev), Some(start)) = (current, start) {
                    length += prev.distance_to(start);
                }
            }
        }
    }
    length
}
