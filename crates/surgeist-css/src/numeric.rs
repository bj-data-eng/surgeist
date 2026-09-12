//! Exact, context-checked authored numeric expressions shared by CSS consumers.
use crate::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueErrorKind,
    CssComponentValueLimits, CssComponentValueRef, CssComponentValues, CssLengthUnit,
    CssNumericTokenKind, CssNumericTokenRef, CssRelativeColorChannel, CssSerializedValue,
    CssSourcePosition, CssValueOrigin, CssValueTokenRef,
};
use std::fmt;

/// Explicit provenance for an existing parser cursor; never ambient parser state.
pub(crate) enum NumericInputContext<'a> {
    Parsed(&'a crate::CssSourceSnapshot),
    Components(&'a CssComponentValues, &'a CssSerializedValue),
}
impl<'a> NumericInputContext<'a> {
    pub(crate) fn error_location(
        &self,
        error: &CssNumericConstructionError,
        fallback: cssparser::SourceLocation,
        root_offset: usize,
    ) -> cssparser::SourceLocation {
        match self {
            Self::Parsed(source) => {
                let Some(CssValueOrigin::Parsed(origin)) = error.origin() else {
                    return fallback;
                };
                if !source.same_snapshot(origin.source()) {
                    return fallback;
                }
                let position = origin.span().start();
                cssparser::SourceLocation {
                    line: position.line().value(),
                    column: position.column().value() + 1,
                }
            }
            Self::Components(_, serialized) => {
                let Some(relative) = error.path.as_deref() else {
                    return fallback;
                };
                let Some(base) = serialized.component_path_at(root_offset) else {
                    return fallback;
                };
                let mut path = base.to_vec();
                path.extend_from_slice(&relative[1..]);
                let Some(offset) = serialized.component_offset_for_path(&path) else {
                    return fallback;
                };
                let mut input = cssparser::ParserInput::new(&serialized.as_css()[..offset]);
                let mut parser = cssparser::Parser::new(&mut input);
                while parser.next_including_whitespace_and_comments().is_ok() {}
                parser.current_source_location()
            }
        }
    }
    pub(crate) fn admit(
        &self,
        values: CssComponentValues,
        root: CalculationRoot,
    ) -> Result<CssCalculationExpression> {
        self.admit_with_limits(values, root, CssComponentValueLimits::default())
    }
    pub(crate) fn admit_with_limits(
        &self,
        values: CssComponentValues,
        root: CalculationRoot,
        limits: CssComponentValueLimits,
    ) -> Result<CssCalculationExpression> {
        let policy = match self {
            Self::Parsed(_) => AdmissionPolicy::RecoveredSyntax,
            Self::Components(..) => AdmissionPolicy::Strict,
        };
        construct_with_policy(values, root, limits, policy)
    }
    pub(crate) fn origin_at(&self, offset: usize) -> Option<CssValueOrigin> {
        match self {
            Self::Components(_, serialized) => serialized.value_origin_at(offset).cloned(),
            Self::Parsed(source) => {
                let suffix = source.as_str().get(offset..)?;
                let mut input = cssparser::ParserInput::new(suffix);
                let mut parser = cssparser::Parser::new(&mut input);
                let _ = parser.next_including_whitespace_and_comments();
                let end = offset.checked_add(parser.position().byte_index())?;
                crate::CssParsedOrigin::from_range(source, offset..end).map(CssValueOrigin::Parsed)
            }
        }
    }
    pub(crate) fn parsed(source: &'a crate::CssSourceSnapshot) -> Self {
        Self::Parsed(source)
    }
    pub(crate) fn components(
        values: &'a CssComponentValues,
        serialized: &'a CssSerializedValue,
    ) -> Self {
        Self::Components(values, serialized)
    }
    pub(crate) fn collect(
        &self,
        input: &mut cssparser::Parser<'_, '_>,
    ) -> Result<CssComponentValue> {
        input.skip_whitespace();
        match self {
            Self::Parsed(source) => CssComponentValue::collect_from_parser(input, source)
                .map_err(CssNumericConstructionError::component),
            Self::Components(values, serialized) => {
                input.skip_whitespace();
                let offset = input.position().byte_index();
                let path = serialized.component_path_at(offset).ok_or_else(|| {
                    CssNumericConstructionError::at(
                        CssNumericConstructionErrorKind::MalformedExpression,
                        None,
                    )
                })?;
                let mut items = values.items();
                let mut found = None;
                for index in path {
                    let c = &items[*index];
                    found = Some(c);
                    items = match c.view() {
                        CssComponentValueRef::Function(f) => f.values().items(),
                        CssComponentValueRef::Block(b) => b.values().items(),
                        _ => &[],
                    };
                }
                let component = found
                    .expect("serialized component path is nonempty")
                    .clone();
                let nested = matches!(
                    component.view(),
                    CssComponentValueRef::Function(_) | CssComponentValueRef::Block(_)
                );
                input.next().map_err(|_| {
                    CssNumericConstructionError::at(
                        CssNumericConstructionErrorKind::MalformedExpression,
                        Some(&component),
                    )
                })?;
                if nested {
                    input
                        .parse_nested_block(|p| {
                            while p.next_including_whitespace_and_comments().is_ok() {}
                            Ok::<_, cssparser::ParseError<'_, ()>>(())
                        })
                        .map_err(|_| {
                            CssNumericConstructionError::at(
                                CssNumericConstructionErrorKind::MalformedExpression,
                                Some(&component),
                            )
                        })?;
                }
                Ok(component)
            }
        }
    }
}

/// A dimensional axis in the CSS numeric type algebra.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssNumericDimension {
    Length,
    Angle,
    Time,
    Frequency,
    Resolution,
    Flex,
    Percentage,
}
const DIMENSIONS: [CssNumericDimension; 7] = [
    CssNumericDimension::Length,
    CssNumericDimension::Angle,
    CssNumericDimension::Time,
    CssNumericDimension::Frequency,
    CssNumericDimension::Resolution,
    CssNumericDimension::Flex,
    CssNumericDimension::Percentage,
];
impl CssNumericDimension {
    const fn index(self) -> usize {
        self as usize
    }
}

/// Immutable dimensional exponents and the context's percentage hint.
///
/// Type metadata can only be obtained from a checked expression.
/// ```compile_fail
/// use surgeist_css::CssNumericType;
/// let forged = CssNumericType { powers: [0; 7], hint: None };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssNumericType {
    powers: [i32; 7],
    hint: Option<CssNumericDimension>,
}
impl CssNumericType {
    const NUMBER: Self = Self {
        powers: [0; 7],
        hint: None,
    };
    fn dimension(d: CssNumericDimension) -> Self {
        let mut t = Self::NUMBER;
        t.powers[d.index()] = 1;
        t
    }
    /// Returns a dimensional exponent, including intermediate negative powers.
    pub const fn exponent(self, d: CssNumericDimension) -> i32 {
        self.powers[d.index()]
    }
    /// Returns the unresolved percentage basis.
    pub const fn percent_hint(self) -> Option<CssNumericDimension> {
        self.hint
    }
    fn hinted(mut self, h: CssNumericDimension) -> Option<Self> {
        if self.hint.is_some_and(|old| old != h) {
            return None;
        }
        self.powers[h.index()] = self.powers[h.index()].checked_add(self.powers[6])?;
        self.powers[6] = 0;
        self.hint = Some(h);
        Some(self)
    }
    fn consistent(self, other: Self) -> Option<(Self, Self)> {
        match (self.hint, other.hint) {
            (Some(a), Some(b)) if a != b => None,
            (Some(h), _) => Some((self, other.hinted(h)?)),
            (_, Some(h)) => Some((self.hinted(h)?, other)),
            _ => Some((self, other)),
        }
    }
    fn add(self, other: Self) -> Option<Self> {
        let (a, b) = self.consistent(other)?;
        if a.powers == b.powers {
            return Some(a);
        }
        if a.hint.is_none() && (a.powers[6] != 0 || b.powers[6] != 0) {
            for d in &DIMENSIONS[..6] {
                let x = a.hinted(*d)?;
                let y = b.hinted(*d)?;
                if x.powers == y.powers {
                    return Some(x);
                }
            }
        }
        None
    }
    fn product(self, other: Self, divide: bool) -> Option<Self> {
        let (mut a, b) = self.consistent(other)?;
        for i in 0..7 {
            a.powers[i] = if divide {
                a.powers[i].checked_sub(b.powers[i])?
            } else {
                a.powers[i].checked_add(b.powers[i])?
            };
        }
        Some(a)
    }
    fn simple(self) -> bool {
        self.powers.iter().all(|p| *p == 0)
            || self.powers.iter().filter(|p| **p == 1).count() == 1
                && self.powers.iter().all(|p| *p == 0 || *p == 1)
    }
    fn is_number(self) -> bool {
        self.powers == [0; 7]
    }
    fn is(self, d: CssNumericDimension) -> bool {
        self.powers == Self::dimension(d).powers
    }
}

/// Classified root types, with complete algebra available for intermediate nodes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCalculationType {
    Number,
    Percentage,
    Length,
    LengthPercentage,
    Angle,
    AnglePercentage,
    Time,
    TimePercentage,
    Frequency,
    FrequencyPercentage,
    Resolution,
    Flex,
    Algebra(CssNumericType),
}
impl CssCalculationType {
    pub(crate) fn from_numeric(t: CssNumericType) -> Self {
        if t.hint.is_none() {
            if t.is_number() {
                return Self::Number;
            }
            for (d, v) in [
                (CssNumericDimension::Length, Self::Length),
                (CssNumericDimension::Angle, Self::Angle),
                (CssNumericDimension::Time, Self::Time),
                (CssNumericDimension::Frequency, Self::Frequency),
                (CssNumericDimension::Resolution, Self::Resolution),
                (CssNumericDimension::Flex, Self::Flex),
                (CssNumericDimension::Percentage, Self::Percentage),
            ] {
                if t.is(d) {
                    return v;
                }
            }
        }
        if t.hint == Some(CssNumericDimension::Length) && t.is(CssNumericDimension::Length) {
            return Self::LengthPercentage;
        }
        Self::Algebra(t)
    }
    pub(crate) fn numeric(self) -> CssNumericType {
        match self {
            Self::Number => CssNumericType::NUMBER,
            Self::Percentage => CssNumericType::dimension(CssNumericDimension::Percentage),
            Self::Length => CssNumericType::dimension(CssNumericDimension::Length),
            Self::Angle => CssNumericType::dimension(CssNumericDimension::Angle),
            Self::Time => CssNumericType::dimension(CssNumericDimension::Time),
            Self::Frequency => CssNumericType::dimension(CssNumericDimension::Frequency),
            Self::Resolution => CssNumericType::dimension(CssNumericDimension::Resolution),
            Self::Flex => CssNumericType::dimension(CssNumericDimension::Flex),
            Self::LengthPercentage => CssNumericType::dimension(CssNumericDimension::Length)
                .hinted(CssNumericDimension::Length)
                .unwrap(),
            Self::AnglePercentage => CssNumericType::dimension(CssNumericDimension::Angle)
                .hinted(CssNumericDimension::Angle)
                .unwrap(),
            Self::TimePercentage => CssNumericType::dimension(CssNumericDimension::Time)
                .hinted(CssNumericDimension::Time)
                .unwrap(),
            Self::FrequencyPercentage => CssNumericType::dimension(CssNumericDimension::Frequency)
                .hinted(CssNumericDimension::Frequency)
                .unwrap(),
            Self::Algebra(t) => t,
        }
    }
    /// Returns the exponent of one dimension.
    pub fn exponent(self, d: CssNumericDimension) -> i32 {
        self.numeric().exponent(d)
    }
    /// Returns the percentage basis retained by type checking.
    pub fn percent_hint(self) -> Option<CssNumericDimension> {
        self.numeric().hint
    }
}

/// Intrinsic rejection of checked numeric construction.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNumericConstructionErrorKind {
    EmptyValue,
    MultipleValues,
    RecoveredComponent,
    SubstitutionRequired,
    MalformedExpression,
    UnknownFunction,
    Arity,
    InvalidArgumentType,
    IncompatibleTypes,
    RootDomainMismatch,
    ResourceLimit,
    Component(CssComponentValueErrorKind),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNumericConstructionError {
    kind: CssNumericConstructionErrorKind,
    origin: Option<CssValueOrigin>,
    source: Option<Box<CssComponentValueError>>,
    path: Option<Box<[usize]>>,
}
impl CssNumericConstructionError {
    pub fn kind(&self) -> &CssNumericConstructionErrorKind {
        &self.kind
    }
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.origin.as_ref()
    }
    pub(crate) fn component_error(&self) -> Option<&CssComponentValueError> {
        self.source.as_deref()
    }
    fn at(kind: CssNumericConstructionErrorKind, c: Option<&CssComponentValue>) -> Self {
        Self {
            kind,
            origin: c.map(|c| c.origin().clone()),
            source: None,
            path: None,
        }
    }
    fn component(e: CssComponentValueError) -> Self {
        Self {
            kind: match e.kind() {
                CssComponentValueErrorKind::NestingLimit
                | CssComponentValueErrorKind::ComponentLimit
                | CssComponentValueErrorKind::ByteLimit
                | CssComponentValueErrorKind::CapacityOverflow => {
                    CssNumericConstructionErrorKind::ResourceLimit
                }
                kind => CssNumericConstructionErrorKind::Component(kind),
            },
            origin: Some(e.origin().clone()),
            source: Some(Box::new(e)),
            path: None,
        }
    }
}
impl CssNumericConstructionError {
    fn with_path(mut self, path: Option<Box<[usize]>>) -> Self {
        if self.path.is_none() {
            self.path = path;
        }
        self
    }
}
impl fmt::Display for CssNumericConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid CSS numeric value: {:?}", self.kind)
    }
}
impl std::error::Error for CssNumericConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref().map(|e| e as _)
    }
}
type Result<T> = std::result::Result<T, CssNumericConstructionError>;

/// A checked canonical unit identity, independent of authored unit spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNumericUnit {
    Length(CssLengthUnit),
    Angle(crate::CssAngleUnit),
    Time(crate::CssTimeUnit),
    Frequency(crate::CssFrequencyUnit),
    Resolution(crate::CssResolutionUnit),
    Flex,
}

/// A checked exact lexical numeric leaf.
#[derive(Clone, Copy, Debug)]
pub struct CssNumericLiteralRef<'a> {
    component: &'a CssComponentValue,
    ty: CssNumericType,
}
impl<'a> CssNumericLiteralRef<'a> {
    pub fn canonical_unit(self) -> Option<CssNumericUnit> {
        let unit = self.unit()?;
        if let Some(unit) = CssLengthUnit::from_css_unit(unit) {
            return Some(CssNumericUnit::Length(unit));
        }
        Some(match unit.to_ascii_lowercase().as_str() {
            "deg" => CssNumericUnit::Angle(crate::CssAngleUnit::Degrees),
            "grad" => CssNumericUnit::Angle(crate::CssAngleUnit::Gradians),
            "rad" => CssNumericUnit::Angle(crate::CssAngleUnit::Radians),
            "turn" => CssNumericUnit::Angle(crate::CssAngleUnit::Turns),
            "s" => CssNumericUnit::Time(crate::CssTimeUnit::Seconds),
            "ms" => CssNumericUnit::Time(crate::CssTimeUnit::Milliseconds),
            "hz" => CssNumericUnit::Frequency(crate::CssFrequencyUnit::Hertz),
            "khz" => CssNumericUnit::Frequency(crate::CssFrequencyUnit::Kilohertz),
            "dpi" => CssNumericUnit::Resolution(crate::CssResolutionUnit::Dpi),
            "dpcm" => CssNumericUnit::Resolution(crate::CssResolutionUnit::Dpcm),
            "dppx" | "x" => CssNumericUnit::Resolution(crate::CssResolutionUnit::Dppx),
            "fr" => CssNumericUnit::Flex,
            _ => unreachable!("checked numeric unit"),
        })
    }
    pub fn numeric_type(self) -> CssNumericType {
        self.ty
    }
    pub fn numeric(self) -> CssNumericTokenRef<'a> {
        match self.component.view() {
            CssComponentValueRef::Token(
                CssValueTokenRef::Number(n)
                | CssValueTokenRef::Percentage(n)
                | CssValueTokenRef::Dimension { number: n, .. },
            ) => n,
            _ => unreachable!("checked numeric leaf"),
        }
    }
    pub fn representation(self) -> &'a str {
        self.numeric().representation()
    }
    pub fn unit(self) -> Option<&'a str> {
        match self.component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) => Some(unit),
            _ => None,
        }
    }
    pub fn origin(self) -> &'a CssValueOrigin {
        self.component.origin()
    }
}
impl PartialEq for CssNumericLiteralRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.component == other.component && self.ty == other.ty
    }
}
impl Eq for CssNumericLiteralRef<'_> {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCalculationValueRef<'a> {
    Integer(CssNumericLiteralRef<'a>),
    Number(CssNumericLiteralRef<'a>),
    Percentage(CssNumericLiteralRef<'a>),
    Length(CssNumericLiteralRef<'a>),
    Angle(CssNumericLiteralRef<'a>),
    Time(CssNumericLiteralRef<'a>),
    Frequency(CssNumericLiteralRef<'a>),
    Resolution(CssNumericLiteralRef<'a>),
    Flex(CssNumericLiteralRef<'a>),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssNumericConstant {
    E,
    Pi,
    Infinity,
    NegativeInfinity,
    NaN,
}
impl CssNumericConstant {
    fn name(self) -> &'static str {
        match self {
            Self::E => "e",
            Self::Pi => "pi",
            Self::Infinity => "infinity",
            Self::NegativeInfinity => "-infinity",
            Self::NaN => "NaN",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssMathFunction {
    Calc,
    Min,
    Max,
    Clamp,
    Round,
    Mod,
    Rem,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    Pow,
    Sqrt,
    Hypot,
    Log,
    Exp,
    Abs,
    Sign,
}
impl CssMathFunction {
    fn parse(name: &str) -> Option<Self> {
        Some(match name.to_ascii_lowercase().as_str() {
            "calc" => Self::Calc,
            "min" => Self::Min,
            "max" => Self::Max,
            "clamp" => Self::Clamp,
            "round" => Self::Round,
            "mod" => Self::Mod,
            "rem" => Self::Rem,
            "sin" => Self::Sin,
            "cos" => Self::Cos,
            "tan" => Self::Tan,
            "asin" => Self::Asin,
            "acos" => Self::Acos,
            "atan" => Self::Atan,
            "atan2" => Self::Atan2,
            "pow" => Self::Pow,
            "sqrt" => Self::Sqrt,
            "hypot" => Self::Hypot,
            "log" => Self::Log,
            "exp" => Self::Exp,
            "abs" => Self::Abs,
            "sign" => Self::Sign,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Calc => "calc",
            Self::Min => "min",
            Self::Max => "max",
            Self::Clamp => "clamp",
            Self::Round => "round",
            Self::Mod => "mod",
            Self::Rem => "rem",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Atan2 => "atan2",
            Self::Pow => "pow",
            Self::Sqrt => "sqrt",
            Self::Hypot => "hypot",
            Self::Log => "log",
            Self::Exp => "exp",
            Self::Abs => "abs",
            Self::Sign => "sign",
        }
    }
}
pub(crate) fn is_math_function(name: &str) -> bool {
    CssMathFunction::parse(name).is_some()
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssRoundingStrategy {
    Nearest,
    Up,
    Down,
    ToZero,
}
impl CssRoundingStrategy {
    fn name(self) -> &'static str {
        match self {
            Self::Nearest => "nearest",
            Self::Up => "up",
            Self::Down => "down",
            Self::ToZero => "to-zero",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssCalculationSumOperator {
    Add,
    Subtract,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssCalculationProductOperator {
    Multiply,
    Divide,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum NodeKind {
    Value(Box<CssComponentValue>),
    Constant(CssNumericConstant),
    Variable(CssRelativeColorChannel),
    Sum(Vec<(Option<CssCalculationSumOperator>, CssCalculationExpression)>),
    Product(
        Vec<(
            Option<CssCalculationProductOperator>,
            CssCalculationExpression,
        )>,
    ),
    Group(Box<CssCalculationExpression>),
    Function {
        function: CssMathFunction,
        args: Vec<Option<CssCalculationExpression>>,
        strategy: Option<CssRoundingStrategy>,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssCalculationExpression {
    kind: NodeKind,
    ty: CssNumericType,
    origin: CssValueOrigin,
    components: Option<CssComponentValues>,
    syntax: Vec<CssComponentValue>,
    closing: Option<CssValueOrigin>,
}
impl CssCalculationExpression {
    fn canonical_len(&self, limit: usize) -> Option<usize> {
        let mut nodes = vec![self];
        let mut length = 0usize;
        while let Some(node) = nodes.pop() {
            let added = match &node.kind {
                NodeKind::Value(c) => {
                    let literal = CssNumericLiteralRef {
                        component: c,
                        ty: node.ty,
                    };
                    literal.representation().len().checked_add(match c.view() {
                        CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => 1,
                        CssComponentValueRef::Token(CssValueTokenRef::Dimension {
                            unit, ..
                        }) => unit.len(),
                        _ => 0,
                    })?
                }
                NodeKind::Constant(c) => c.name().len(),
                NodeKind::Variable(c) => {
                    if *c == CssRelativeColorChannel::Alpha {
                        5
                    } else {
                        1
                    }
                }
                NodeKind::Group(value) => {
                    nodes.push(value);
                    2
                }
                NodeKind::Sum(values) => {
                    nodes.extend(values.iter().map(|(_, v)| v));
                    values.len().checked_sub(1)?.checked_mul(3)?
                }
                NodeKind::Product(values) => {
                    nodes.extend(values.iter().map(|(_, v)| v));
                    values.len().checked_sub(1)?.checked_mul(3)?
                }
                NodeKind::Function {
                    function,
                    args,
                    strategy,
                } => {
                    nodes.extend(args.iter().flatten());
                    function
                        .name()
                        .len()
                        .checked_add(2)?
                        .checked_add(args.len().checked_sub(1)?.checked_mul(2)?)?
                        .checked_add(args.iter().filter(|a| a.is_none()).count().checked_mul(4)?)?
                        .checked_add(strategy.map_or(0, |s| s.name().len() + 2))?
                }
            };
            length = length.checked_add(added)?;
            if length > limit {
                return None;
            }
        }
        Some(length)
    }
    fn syntax_items(&self) -> &[CssComponentValue] {
        &self.syntax
    }
    fn closing_origin(&self) -> &CssValueOrigin {
        self.closing.as_ref().unwrap_or(&self.origin)
    }
    /// Emits canonical tokens without consuming call stack per expression node.
    /// Both string and provenance serialization use this one traversal.
    fn emit_canonical(&self, mut emit: impl FnMut(String, &CssValueOrigin)) {
        enum Work<'a> {
            Visit(&'a CssCalculationExpression),
            Token(String, &'a CssValueOrigin),
        }
        let mut pending = vec![Work::Visit(self)];
        while let Some(work) = pending.pop() {
            let node = match work {
                Work::Token(text, origin) => {
                    emit(text, origin);
                    continue;
                }
                Work::Visit(node) => node,
            };
            let mut next = Vec::new();
            match &node.kind {
                NodeKind::Value(component) => {
                    let literal = CssNumericLiteralRef {
                        component,
                        ty: node.ty,
                    };
                    let text = match component.view() {
                        CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => {
                            format!("{}%", literal.representation())
                        }
                        CssComponentValueRef::Token(CssValueTokenRef::Dimension {
                            unit, ..
                        }) => format!("{}{}", literal.representation(), unit.to_ascii_lowercase()),
                        _ => literal.representation().to_owned(),
                    };
                    emit(text, &node.origin);
                }
                NodeKind::Constant(constant) => emit(constant.name().to_owned(), &node.origin),
                NodeKind::Variable(channel) => {
                    emit(format!("{channel:?}").to_ascii_lowercase(), &node.origin)
                }
                NodeKind::Group(value) => {
                    emit("(".into(), &node.origin);
                    next.push(Work::Visit(value));
                    next.push(Work::Token(")".into(), node.closing_origin()));
                }
                NodeKind::Sum(values) => {
                    let mut operators = node.syntax_items().iter().filter(|component| {
                        matches!(
                            component.view(),
                            CssComponentValueRef::Token(CssValueTokenRef::Delim('+' | '-'))
                        )
                    });
                    for (operator, value) in values {
                        if let Some(operator) = operator {
                            let text = match operator {
                                CssCalculationSumOperator::Add => " + ",
                                CssCalculationSumOperator::Subtract => " - ",
                            };
                            next.push(Work::Token(
                                text.into(),
                                operators.next().expect("checked sum operator").origin(),
                            ));
                        }
                        next.push(Work::Visit(value));
                    }
                }
                NodeKind::Product(values) => {
                    let mut operators = node.syntax_items().iter().filter(|component| {
                        matches!(
                            component.view(),
                            CssComponentValueRef::Token(CssValueTokenRef::Delim('*' | '/'))
                        )
                    });
                    for (operator, value) in values {
                        if let Some(operator) = operator {
                            let text = match operator {
                                CssCalculationProductOperator::Multiply => " * ",
                                CssCalculationProductOperator::Divide => " / ",
                            };
                            next.push(Work::Token(
                                text.into(),
                                operators.next().expect("checked product operator").origin(),
                            ));
                        }
                        next.push(Work::Visit(value));
                    }
                }
                NodeKind::Function {
                    function,
                    args,
                    strategy,
                } => {
                    emit(format!("{}(", function.name()), &node.origin);
                    let mut commas = node.syntax_items().iter().filter(|component| {
                        matches!(
                            component.view(),
                            CssComponentValueRef::Token(CssValueTokenRef::Comma)
                        )
                    });
                    let mut names = node.syntax_items().iter().filter(|component| {
                        matches!(
                            component.view(),
                            CssComponentValueRef::Token(CssValueTokenRef::Ident(_))
                        )
                    });
                    if let Some(strategy) = strategy {
                        next.push(Work::Token(
                            strategy.name().into(),
                            names.next().expect("checked strategy").origin(),
                        ));
                        next.push(Work::Token(
                            ", ".into(),
                            commas.next().expect("checked strategy comma").origin(),
                        ));
                    }
                    for (index, argument) in args.iter().enumerate() {
                        if index > 0 {
                            next.push(Work::Token(
                                ", ".into(),
                                commas.next().expect("checked argument comma").origin(),
                            ));
                        }
                        if let Some(argument) = argument {
                            next.push(Work::Visit(argument));
                        } else {
                            next.push(Work::Token(
                                "none".into(),
                                names
                                    .find(|component| ident(component, "none"))
                                    .expect("checked absent bound")
                                    .origin(),
                            ));
                        }
                    }
                    next.push(Work::Token(")".into(), node.closing_origin()));
                }
            }
            pending.extend(next.into_iter().rev());
        }
    }
    fn canonical_tokens(&self, out: &mut Vec<(String, CssValueOrigin)>) {
        self.emit_canonical(|text, origin| out.push((text, origin.clone())));
    }
    pub(crate) fn references(&self) -> Vec<CssRelativeColorChannel> {
        let mut found = Vec::new();
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            match &node.kind {
                NodeKind::Variable(c) => found.push(*c),
                NodeKind::Sum(v) => stack.extend(v.iter().rev().map(|(_, e)| e)),
                NodeKind::Product(v) => stack.extend(v.iter().rev().map(|(_, e)| e)),
                NodeKind::Group(e) => stack.push(e),
                NodeKind::Function { args, .. } => stack.extend(args.iter().rev().flatten()),
                _ => {}
            }
        }
        found
    }
    pub(crate) fn result_type(&self) -> CssCalculationType {
        CssCalculationType::from_numeric(self.ty)
    }
    pub(crate) fn as_ref(&self) -> CssCalculationExpressionRef<'_> {
        match &self.kind {
            NodeKind::Value(c) => {
                let v = CssNumericLiteralRef {
                    component: c,
                    ty: self.ty,
                };
                let value = match c.view() {
                    CssComponentValueRef::Token(CssValueTokenRef::Number(n)) => {
                        if n.kind() == CssNumericTokenKind::Integer {
                            CssCalculationValueRef::Integer(v)
                        } else {
                            CssCalculationValueRef::Number(v)
                        }
                    }
                    CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => {
                        CssCalculationValueRef::Percentage(v)
                    }
                    _ => match unit_dimension(v.unit().unwrap()) {
                        Some(CssNumericDimension::Length) => CssCalculationValueRef::Length(v),
                        Some(CssNumericDimension::Angle) => CssCalculationValueRef::Angle(v),
                        Some(CssNumericDimension::Time) => CssCalculationValueRef::Time(v),
                        Some(CssNumericDimension::Frequency) => {
                            CssCalculationValueRef::Frequency(v)
                        }
                        Some(CssNumericDimension::Resolution) => {
                            CssCalculationValueRef::Resolution(v)
                        }
                        Some(CssNumericDimension::Flex) => CssCalculationValueRef::Flex(v),
                        _ => unreachable!("checked dimension"),
                    },
                };
                CssCalculationExpressionRef::Value(value)
            }
            NodeKind::Constant(_) => {
                CssCalculationExpressionRef::Constant(CssCalculationConstantRef { node: self })
            }
            NodeKind::Variable(_) => {
                CssCalculationExpressionRef::Variable(CssCalculationVariableRef { node: self })
            }
            NodeKind::Sum(_) => {
                CssCalculationExpressionRef::Sum(CssCalculationSumRef { node: self })
            }
            NodeKind::Product(_) => {
                CssCalculationExpressionRef::Product(CssCalculationProductRef { node: self })
            }
            NodeKind::Group(_) => {
                CssCalculationExpressionRef::Group(CssCalculationUnaryRef { node: self })
            }
            NodeKind::Function {
                function: CssMathFunction::Calc,
                ..
            } => CssCalculationExpressionRef::NestedCalc(CssCalculationUnaryRef { node: self }),
            NodeKind::Function { .. } => {
                CssCalculationExpressionRef::Function(CssCalculationFunctionRef { node: self })
            }
        }
    }
    pub(crate) fn to_css_fragment(&self) -> String {
        let mut output = String::new();
        self.emit_canonical(|text, _| output.push_str(&text));
        output
    }
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssCalculationExpressionRef<'a> {
    Value(CssCalculationValueRef<'a>),
    Constant(CssCalculationConstantRef<'a>),
    Variable(CssCalculationVariableRef<'a>),
    Sum(CssCalculationSumRef<'a>),
    Product(CssCalculationProductRef<'a>),
    Group(CssCalculationUnaryRef<'a>),
    NestedCalc(CssCalculationUnaryRef<'a>),
    Function(CssCalculationFunctionRef<'a>),
}
impl<'a> CssCalculationExpressionRef<'a> {
    pub fn origin(self) -> &'a CssValueOrigin {
        match self {
            Self::Value(v) => v.literal().origin(),
            Self::Constant(v) => v.origin(),
            Self::Variable(v) => v.origin(),
            Self::Sum(v) => v.origin(),
            Self::Product(v) => v.origin(),
            Self::Group(v) | Self::NestedCalc(v) => v.origin(),
            Self::Function(v) => v.origin(),
        }
    }
    pub fn numeric_type(self) -> CssNumericType {
        match self {
            Self::Value(v) => v.literal().numeric_type(),
            Self::Constant(v) => v.node.ty,
            Self::Variable(v) => v.node.ty,
            Self::Sum(v) => v.node.ty,
            Self::Product(v) => v.node.ty,
            Self::Group(v) | Self::NestedCalc(v) => v.node.ty,
            Self::Function(v) => v.node.ty,
        }
    }
}
impl<'a> CssCalculationValueRef<'a> {
    pub fn literal(self) -> CssNumericLiteralRef<'a> {
        match self {
            Self::Integer(v)
            | Self::Number(v)
            | Self::Percentage(v)
            | Self::Length(v)
            | Self::Angle(v)
            | Self::Time(v)
            | Self::Frequency(v)
            | Self::Resolution(v)
            | Self::Flex(v) => v,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationConstantRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationConstantRef<'a> {
    pub fn value(self) -> CssNumericConstant {
        let NodeKind::Constant(v) = self.node.kind else {
            unreachable!()
        };
        v
    }
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn numeric_type(self) -> CssNumericType {
        self.node.ty
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationVariableRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationVariableRef<'a> {
    pub fn channel(self) -> CssRelativeColorChannel {
        let NodeKind::Variable(v) = self.node.kind else {
            unreachable!()
        };
        v
    }
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn numeric_type(self) -> CssNumericType {
        self.node.ty
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationSumRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationSumRef<'a> {
    pub fn len(self) -> usize {
        match &self.node.kind {
            NodeKind::Sum(v) => v.len(),
            _ => unreachable!(),
        }
    }
    pub fn is_empty(self) -> bool {
        false
    }
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn term(self, i: usize) -> Option<CssCalculationSumTermRef<'a>> {
        let NodeKind::Sum(v) = &self.node.kind else {
            unreachable!()
        };
        v.get(i)
            .map(|(operator, expression)| CssCalculationSumTermRef {
                operator: *operator,
                expression,
                operator_origin: i
                    .checked_sub(1)
                    .and_then(|n| {
                        self.node
                            .syntax
                            .iter()
                            .filter(|c| {
                                matches!(
                                    c.view(),
                                    CssComponentValueRef::Token(CssValueTokenRef::Delim('+' | '-'))
                                )
                            })
                            .nth(n)
                    })
                    .map(CssComponentValue::origin),
            })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationSumTermRef<'a> {
    operator: Option<CssCalculationSumOperator>,
    expression: &'a CssCalculationExpression,
    operator_origin: Option<&'a CssValueOrigin>,
}
impl<'a> CssCalculationSumTermRef<'a> {
    pub fn operator_origin(self) -> Option<&'a CssValueOrigin> {
        self.operator_origin
    }
    pub fn operator(self) -> Option<CssCalculationSumOperator> {
        self.operator
    }
    pub fn expression(self) -> CssCalculationExpressionRef<'a> {
        self.expression.as_ref()
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationProductRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationProductRef<'a> {
    pub fn len(self) -> usize {
        match &self.node.kind {
            NodeKind::Product(v) => v.len(),
            _ => unreachable!(),
        }
    }
    pub fn is_empty(self) -> bool {
        false
    }
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn factor(self, i: usize) -> Option<CssCalculationProductFactorRef<'a>> {
        let NodeKind::Product(v) = &self.node.kind else {
            unreachable!()
        };
        v.get(i)
            .map(|(operator, expression)| CssCalculationProductFactorRef {
                operator: *operator,
                expression,
                operator_origin: i
                    .checked_sub(1)
                    .and_then(|n| {
                        self.node
                            .syntax
                            .iter()
                            .filter(|c| {
                                matches!(
                                    c.view(),
                                    CssComponentValueRef::Token(CssValueTokenRef::Delim('*' | '/'))
                                )
                            })
                            .nth(n)
                    })
                    .map(CssComponentValue::origin),
            })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationProductFactorRef<'a> {
    operator: Option<CssCalculationProductOperator>,
    expression: &'a CssCalculationExpression,
    operator_origin: Option<&'a CssValueOrigin>,
}
impl<'a> CssCalculationProductFactorRef<'a> {
    pub fn operator_origin(self) -> Option<&'a CssValueOrigin> {
        self.operator_origin
    }
    pub fn operator(self) -> Option<CssCalculationProductOperator> {
        self.operator
    }
    pub fn expression(self) -> CssCalculationExpressionRef<'a> {
        self.expression.as_ref()
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationUnaryRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationUnaryRef<'a> {
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn operand(self) -> CssCalculationExpressionRef<'a> {
        match &self.node.kind {
            NodeKind::Group(v) => v.as_ref().as_ref(),
            NodeKind::Function { args, .. } => args[0].as_ref().unwrap().as_ref(),
            _ => unreachable!(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationFunctionRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationFunctionRef<'a> {
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn function(self) -> CssMathFunction {
        let NodeKind::Function { function, .. } = &self.node.kind else {
            unreachable!()
        };
        *function
    }
    pub fn len(self) -> usize {
        let NodeKind::Function { args, .. } = &self.node.kind else {
            unreachable!()
        };
        args.len()
    }
    pub fn is_empty(self) -> bool {
        false
    }
    pub fn argument(self, i: usize) -> Option<Option<CssCalculationExpressionRef<'a>>> {
        let NodeKind::Function { args, .. } = &self.node.kind else {
            unreachable!()
        };
        args.get(i)
            .map(|a| a.as_ref().map(CssCalculationExpression::as_ref))
    }
    pub fn rounding_strategy(self) -> Option<CssRoundingStrategy> {
        let NodeKind::Function { strategy, .. } = &self.node.kind else {
            unreachable!()
        };
        *strategy
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CalculationRoot {
    NamedDimensionOrNumber,
    Number,
    Integer,
    Percentage,
    Length,
    LengthPercentage,
    Angle,
    Time,
    Frequency,
    Resolution,
    Relative(
        crate::CssRelativeColorEnvironment,
        crate::CssRelativeColorResultDomain,
    ),
}
impl CalculationRoot {
    fn hint(self) -> Option<CssNumericDimension> {
        (self == Self::LengthPercentage).then_some(CssNumericDimension::Length)
    }
    fn accepts(self, t: CssNumericType) -> bool {
        match self {
            Self::NamedDimensionOrNumber => {
                t.hint.is_none()
                    && (t.is_number() || DIMENSIONS[..6].iter().any(|dimension| t.is(*dimension)))
            }
            Self::Number | Self::Integer => t.is_number() && t.hint.is_none(),
            Self::Percentage => t.is(CssNumericDimension::Percentage) && t.hint.is_none(),
            Self::Length => t.is(CssNumericDimension::Length) && t.hint.is_none(),
            Self::LengthPercentage => t.is(CssNumericDimension::Length),
            Self::Angle => t.is(CssNumericDimension::Angle) && t.hint.is_none(),
            Self::Time => t.is(CssNumericDimension::Time) && t.hint.is_none(),
            Self::Frequency => t.is(CssNumericDimension::Frequency) && t.hint.is_none(),
            Self::Resolution => t.is(CssNumericDimension::Resolution) && t.hint.is_none(),
            Self::Relative(_, crate::CssRelativeColorResultDomain::Hue) => {
                t.hint.is_none() && (t.is_number() || t.is(CssNumericDimension::Angle))
            }
            Self::Relative(_, _) => {
                t.hint.is_none() && (t.is_number() || t.is(CssNumericDimension::Percentage))
            }
        }
    }
}

fn unit_dimension(unit: &str) -> Option<CssNumericDimension> {
    if CssLengthUnit::from_css_unit(unit).is_some() {
        return Some(CssNumericDimension::Length);
    }
    Some(match unit.to_ascii_lowercase().as_str() {
        "deg" | "grad" | "rad" | "turn" => CssNumericDimension::Angle,
        "s" | "ms" => CssNumericDimension::Time,
        "hz" | "khz" => CssNumericDimension::Frequency,
        "dpi" | "dpcm" | "dppx" | "x" => CssNumericDimension::Resolution,
        "fr" => CssNumericDimension::Flex,
        _ => return None,
    })
}
fn trivia(c: &CssComponentValue) -> bool {
    matches!(
        c.view(),
        CssComponentValueRef::Comment(_)
            | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
    )
}
fn direct_syntax(items: &[CssComponentValue]) -> Vec<CssComponentValue> {
    items
        .iter()
        .filter(|c| {
            matches!(
                c.view(),
                CssComponentValueRef::Token(
                    CssValueTokenRef::Ident(_)
                        | CssValueTokenRef::Comma
                        | CssValueTokenRef::Delim(_)
                )
            )
        })
        .cloned()
        .collect()
}
fn whitespace(c: &CssComponentValue) -> bool {
    matches!(
        c.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
    )
}
fn ident(c: &CssComponentValue, name: &str) -> bool {
    matches!(c.view(),CssComponentValueRef::Token(CssValueTokenRef::Ident(s))if s.eq_ignore_ascii_case(name))
}
fn exact_zero(n: CssNumericTokenRef<'_>) -> bool {
    n.representation()
        .split(['e', 'E'])
        .next()
        .unwrap()
        .chars()
        .all(|c| matches!(c, '+' | '-' | '.' | '0'))
}

// These keys identify occurrences in the borrowed component graph only. They
// never escape admission, and neither token equality nor origin equality is
// used to combine independently positioned components.
#[derive(Clone, Copy)]
struct ComponentKey<'a>(&'a CssComponentValue);
impl PartialEq for ComponentKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}
impl Eq for ComponentKey<'_> {}
impl std::hash::Hash for ComponentKey<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&std::ptr::from_ref(self.0), state);
    }
}

struct NumericParser<'a> {
    root: CalculationRoot,
    ready: std::collections::HashMap<ComponentKey<'a>, CssCalculationExpression>,
    paths: std::collections::HashMap<ComponentKey<'a>, Box<[usize]>>,
}
impl<'a> NumericParser<'a> {
    fn error(
        &self,
        kind: CssNumericConstructionErrorKind,
        component: Option<&'a CssComponentValue>,
    ) -> CssNumericConstructionError {
        CssNumericConstructionError::at(kind, component)
            .with_path(component.and_then(|c| self.paths.get(&ComponentKey(c)).cloned()))
    }
    fn take(&mut self, component: &'a CssComponentValue) -> Result<CssCalculationExpression> {
        if let Some(expression) = self.ready.remove(&ComponentKey(component)) {
            return Ok(expression);
        }
        if matches!(
            component.view(),
            CssComponentValueRef::Function(_) | CssComponentValueRef::Block(_)
        ) {
            return Err(self.error(
                CssNumericConstructionErrorKind::MalformedExpression,
                Some(component),
            ));
        }
        parse_node(component, self)
            .map_err(|e| e.with_path(self.paths.get(&ComponentKey(component)).cloned()))
    }
}

fn parse_tree(
    component: &CssComponentValue,
    root: CalculationRoot,
    root_index: usize,
) -> Result<CssCalculationExpression> {
    let mut parser = NumericParser {
        root,
        ready: std::collections::HashMap::new(),
        paths: std::collections::HashMap::new(),
    };
    let mut pending = vec![(component, vec![root_index], false)];
    while let Some((current, path, visited)) = pending.pop() {
        if visited {
            let expression = parse_node(current, &mut parser)
                .map_err(|e| e.with_path(Some(path.into_boxed_slice())))?;
            parser.ready.insert(ComponentKey(current), expression);
            continue;
        }
        parser
            .paths
            .insert(ComponentKey(current), path.clone().into_boxed_slice());
        pending.push((current, path.clone(), true));
        let children = match current.view() {
            CssComponentValueRef::Function(f) => f.values().items(),
            CssComponentValueRef::Block(b) => b.values().items(),
            _ => &[],
        };
        for (index, child) in children.iter().enumerate().rev() {
            let mut child_path = path.clone();
            child_path.push(index);
            parser
                .paths
                .insert(ComponentKey(child), child_path.clone().into_boxed_slice());
            if matches!(
                child.view(),
                CssComponentValueRef::Function(_) | CssComponentValueRef::Block(_)
            ) {
                pending.push((child, child_path, false));
            }
        }
    }
    Ok(parser
        .ready
        .remove(&ComponentKey(component))
        .expect("admitted root node"))
}

struct Cursor<'a, 'p> {
    items: &'a [CssComponentValue],
    i: usize,
    parser: &'p mut NumericParser<'a>,
}
impl<'a> Cursor<'a, '_> {
    fn skip(&mut self) -> bool {
        let mut ws = false;
        while self.i < self.items.len() && trivia(&self.items[self.i]) {
            ws |= whitespace(&self.items[self.i]);
            self.i += 1;
        }
        ws
    }
    fn err(&self, k: CssNumericConstructionErrorKind) -> CssNumericConstructionError {
        self.parser.error(
            k,
            self.items
                .get(self.i)
                .or_else(|| self.items.iter().rev().find(|c| !trivia(c))),
        )
    }
    fn sum(&mut self) -> Result<CssCalculationExpression> {
        let start = self.i;
        let first = self.product()?;
        let mut ty = first.ty;
        let origin = first.origin.clone();
        let mut terms = vec![(None, first)];
        loop {
            let mark = self.i;
            let ws = self.skip();
            let op = match self.items.get(self.i).map(CssComponentValue::view) {
                Some(CssComponentValueRef::Token(CssValueTokenRef::Delim('+'))) => {
                    CssCalculationSumOperator::Add
                }
                Some(CssComponentValueRef::Token(CssValueTokenRef::Delim('-'))) => {
                    CssCalculationSumOperator::Subtract
                }
                _ => {
                    self.i = mark;
                    break;
                }
            };
            if !ws {
                return Err(self.err(CssNumericConstructionErrorKind::MalformedExpression));
            }
            self.i += 1;
            if !self.skip() {
                return Err(self.err(CssNumericConstructionErrorKind::MalformedExpression));
            }
            let right_site = self.items.get(self.i);
            let right = self.product()?;
            ty = ty.add(right.ty).ok_or_else(|| {
                self.parser.error(
                    CssNumericConstructionErrorKind::IncompatibleTypes,
                    right_site,
                )
            })?;
            terms.push((Some(op), right));
        }
        if terms.len() == 1 {
            Ok(terms.pop().unwrap().1)
        } else {
            Ok(CssCalculationExpression {
                kind: NodeKind::Sum(terms),
                ty,
                origin,
                components: None,
                syntax: direct_syntax(&self.items[start..self.i]),
                closing: None,
            })
        }
    }
    fn product(&mut self) -> Result<CssCalculationExpression> {
        let start = self.i;
        let first = self.value()?;
        let mut ty = first.ty;
        let origin = first.origin.clone();
        let mut factors = vec![(None, first)];
        loop {
            let mark = self.i;
            self.skip();
            let op = match self.items.get(self.i).map(CssComponentValue::view) {
                Some(CssComponentValueRef::Token(CssValueTokenRef::Delim('*'))) => {
                    CssCalculationProductOperator::Multiply
                }
                Some(CssComponentValueRef::Token(CssValueTokenRef::Delim('/'))) => {
                    CssCalculationProductOperator::Divide
                }
                _ => {
                    self.i = mark;
                    break;
                }
            };
            self.i += 1;
            self.skip();
            let right_site = self.items.get(self.i);
            let right = self.value()?;
            if ty.hint.is_some() && right.ty.hint.is_some() && ty.hint != right.ty.hint {
                return Err(self.parser.error(
                    CssNumericConstructionErrorKind::IncompatibleTypes,
                    right_site,
                ));
            }
            ty = ty
                .product(right.ty, op == CssCalculationProductOperator::Divide)
                .ok_or_else(|| {
                    let error = self
                        .parser
                        .error(CssNumericConstructionErrorKind::ResourceLimit, right_site);
                    CssNumericConstructionError::component(CssComponentValueError::new(
                        CssComponentValueErrorKind::CapacityOverflow,
                        error
                            .origin
                            .clone()
                            .expect("a checked right operand has an origin"),
                    ))
                    .with_path(error.path)
                })?;
            factors.push((Some(op), right));
        }
        if factors.len() == 1 {
            Ok(factors.pop().unwrap().1)
        } else {
            Ok(CssCalculationExpression {
                kind: NodeKind::Product(factors),
                ty,
                origin,
                components: None,
                syntax: direct_syntax(&self.items[start..self.i]),
                closing: None,
            })
        }
    }
    fn value(&mut self) -> Result<CssCalculationExpression> {
        self.skip();
        let c = self
            .items
            .get(self.i)
            .ok_or_else(|| self.err(CssNumericConstructionErrorKind::MalformedExpression))?;
        self.i += 1;
        self.parser.take(c)
    }
}
fn sequence<'a>(
    items: &'a [CssComponentValue],
    parser: &mut NumericParser<'a>,
) -> Result<CssCalculationExpression> {
    let mut c = Cursor {
        items,
        i: 0,
        parser,
    };
    let result = c.sum()?;
    c.skip();
    if c.i != items.len() {
        return Err(c.err(CssNumericConstructionErrorKind::MalformedExpression));
    }
    Ok(result)
}
fn parse_node<'a>(
    c: &'a CssComponentValue,
    parser: &mut NumericParser<'a>,
) -> Result<CssCalculationExpression> {
    let root = parser.root;
    let error = |kind| CssNumericConstructionError::at(kind, Some(c));
    let (kind, mut ty) = match c.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(_)) => {
            (NodeKind::Value(Box::new(c.clone())), CssNumericType::NUMBER)
        }
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => (
            NodeKind::Value(Box::new(c.clone())),
            CssNumericType::dimension(CssNumericDimension::Percentage),
        ),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) => (
            NodeKind::Value(Box::new(c.clone())),
            CssNumericType::dimension(
                unit_dimension(unit)
                    .ok_or_else(|| error(CssNumericConstructionErrorKind::InvalidArgumentType))?,
            ),
        ),
        CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => {
            let constant = match name.to_ascii_lowercase().as_str() {
                "e" => Some(CssNumericConstant::E),
                "pi" => Some(CssNumericConstant::Pi),
                "infinity" => Some(CssNumericConstant::Infinity),
                "-infinity" => Some(CssNumericConstant::NegativeInfinity),
                "nan" => Some(CssNumericConstant::NaN),
                _ => None,
            };
            if let Some(v) = constant {
                (NodeKind::Constant(v), CssNumericType::NUMBER)
            } else if let CalculationRoot::Relative(environment, _) = root {
                let (channel, ty) = crate::parser::numeric_relative_channel(environment, name)
                    .ok_or_else(|| error(CssNumericConstructionErrorKind::InvalidArgumentType))?;
                (NodeKind::Variable(channel), ty.numeric())
            } else {
                return Err(error(CssNumericConstructionErrorKind::InvalidArgumentType));
            }
        }
        CssComponentValueRef::Block(b) if b.kind() == CssBlockKind::Parenthesis => {
            let child = sequence(b.values().items(), parser)?;
            let ty = child.ty;
            (NodeKind::Group(Box::new(child)), ty)
        }
        CssComponentValueRef::Function(f) => {
            if f.name().eq_ignore_ascii_case("var") {
                return Err(error(CssNumericConstructionErrorKind::SubstitutionRequired));
            }
            let function = CssMathFunction::parse(f.name())
                .ok_or_else(|| error(CssNumericConstructionErrorKind::UnknownFunction))?;
            parse_function(function, f.values(), parser, c)?
        }
        _ => return Err(error(CssNumericConstructionErrorKind::MalformedExpression)),
    };
    if let Some(h) = root.hint()
        && ty.is(CssNumericDimension::Percentage)
    {
        ty = ty.hinted(h).unwrap()
    }
    let (syntax, closing) = match c.view() {
        CssComponentValueRef::Function(f) => (
            direct_syntax(f.values().items()),
            Some(f.closing_origin().clone()),
        ),
        CssComponentValueRef::Block(b) => (Vec::new(), Some(b.closing_origin().clone())),
        _ => (Vec::new(), None),
    };
    Ok(CssCalculationExpression {
        kind,
        ty,
        origin: c.origin().clone(),
        components: None,
        syntax,
        closing,
    })
}
fn parse_function<'a>(
    function: CssMathFunction,
    values: &'a CssComponentValues,
    parser: &mut NumericParser<'a>,
    component: &'a CssComponentValue,
) -> Result<(NodeKind, CssNumericType)> {
    use CssMathFunction as F;
    let err = |kind| CssNumericConstructionError::at(kind, Some(component));
    let mut segments: Vec<&[CssComponentValue]> = values
        .items()
        .split(|c| {
            matches!(
                c.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Comma)
            )
        })
        .collect();
    let mut strategy = None;
    if function == F::Round {
        let significant: Vec<_> = segments[0].iter().filter(|c| !trivia(c)).collect();
        if significant.len() == 1 {
            strategy = if ident(significant[0], "nearest") {
                Some(CssRoundingStrategy::Nearest)
            } else if ident(significant[0], "up") {
                Some(CssRoundingStrategy::Up)
            } else if ident(significant[0], "down") {
                Some(CssRoundingStrategy::Down)
            } else if ident(significant[0], "to-zero") {
                Some(CssRoundingStrategy::ToZero)
            } else {
                None
            };
            if strategy.is_some() {
                segments.remove(0);
            }
        }
    }
    let valid_arity = match function {
        F::Min | F::Max | F::Hypot => !segments.is_empty(),
        F::Clamp => segments.len() == 3,
        F::Round | F::Log => (1..=2).contains(&segments.len()),
        F::Mod | F::Rem | F::Atan2 | F::Pow => segments.len() == 2,
        _ => segments.len() == 1,
    };
    if !valid_arity {
        return Err(err(CssNumericConstructionErrorKind::Arity));
    }
    let mut args = Vec::new();
    for (i, s) in segments.iter().enumerate() {
        let sig: Vec<_> = s.iter().filter(|c| !trivia(c)).collect();
        if sig.is_empty() {
            return Err(err(CssNumericConstructionErrorKind::Arity));
        }
        if function == F::Clamp && i != 1 && sig.len() == 1 && ident(sig[0], "none") {
            args.push(None)
        } else {
            args.push(Some(sequence(s, parser)?));
        }
    }
    let ts: Vec<_> = args.iter().flatten().map(|a| a.ty).collect();
    if ts.is_empty() {
        return Err(err(CssNumericConstructionErrorKind::Arity));
    }
    let common = || {
        ts.iter()
            .copied()
            .try_fold(None, |acc: Option<CssNumericType>, t| match acc {
                None => Some(Some(t)),
                Some(a) => a.add(t).map(Some),
            })
            .flatten()
            .ok_or_else(|| err(CssNumericConstructionErrorKind::IncompatibleTypes))
    };
    let output = match function {
        F::Calc | F::Abs => ts[0],
        F::Min | F::Max | F::Clamp | F::Hypot | F::Mod | F::Rem => common()?,
        F::Round => {
            if args.len() == 1 && !ts[0].is_number() {
                return Err(err(CssNumericConstructionErrorKind::InvalidArgumentType));
            }
            common()?
        }
        F::Sin | F::Cos | F::Tan => {
            if !ts[0].is_number() && !ts[0].is(CssNumericDimension::Angle) {
                return Err(err(CssNumericConstructionErrorKind::InvalidArgumentType));
            }
            CssNumericType {
                hint: ts[0].hint,
                ..CssNumericType::NUMBER
            }
        }
        F::Asin | F::Acos | F::Atan => {
            if !ts[0].is_number() {
                return Err(err(CssNumericConstructionErrorKind::InvalidArgumentType));
            }
            CssNumericType {
                hint: ts[0].hint,
                ..CssNumericType::dimension(CssNumericDimension::Angle)
            }
        }
        F::Atan2 => {
            let t = common()?;
            CssNumericType {
                hint: t.hint,
                ..CssNumericType::dimension(CssNumericDimension::Angle)
            }
        }
        F::Pow | F::Sqrt | F::Log | F::Exp => {
            if ts.iter().any(|t| !t.is_number()) {
                return Err(err(CssNumericConstructionErrorKind::InvalidArgumentType));
            }
            common()?
        }
        F::Sign => CssNumericType {
            hint: ts[0].hint,
            ..CssNumericType::NUMBER
        },
    };
    if !output.simple() || ts.iter().any(|t| !t.simple()) && function != F::Calc {
        return Err(err(CssNumericConstructionErrorKind::InvalidArgumentType));
    }
    Ok((
        NodeKind::Function {
            function,
            args,
            strategy,
        },
        output,
    ))
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum AdmissionPolicy {
    Strict,
    RecoveredSyntax,
}

fn validate_components(
    values: &CssComponentValues,
    limits: CssComponentValueLimits,
    policy: AdmissionPolicy,
) -> Result<()> {
    values
        .validate_with_limits(limits)
        .map_err(CssNumericConstructionError::component)?;
    if policy == AdmissionPolicy::Strict {
        let mut pending: Vec<_> = values.items().iter().collect();
        while let Some(component) = pending.pop() {
            let nested = match component.view() {
                CssComponentValueRef::Function(function) => {
                    Some((function.values(), function.closing_origin()))
                }
                CssComponentValueRef::Block(block) => {
                    Some((block.values(), block.closing_origin()))
                }
                _ => None,
            };
            if let Some((children, closing)) = nested {
                if matches!(closing, CssValueOrigin::ImplicitClosure { .. }) {
                    return Err(CssNumericConstructionError::at(
                        CssNumericConstructionErrorKind::RecoveredComponent,
                        Some(component),
                    ));
                }
                pending.extend(children.items());
            }
        }
    }
    Ok(())
}
pub(crate) fn construct(
    values: CssComponentValues,
    root: CalculationRoot,
    limits: CssComponentValueLimits,
) -> Result<CssCalculationExpression> {
    construct_with_policy(values, root, limits, AdmissionPolicy::Strict)
}
fn construct_with_policy(
    values: CssComponentValues,
    root: CalculationRoot,
    limits: CssComponentValueLimits,
    policy: AdmissionPolicy,
) -> Result<CssCalculationExpression> {
    validate_components(&values, limits, policy)?;
    let mut significant = values.items().iter().filter(|c| !trivia(c));
    let c = significant.next().ok_or_else(|| {
        CssNumericConstructionError::at(CssNumericConstructionErrorKind::EmptyValue, None)
    })?;
    if let Some(extra) = significant.next() {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::MultipleValues,
            Some(extra),
        ));
    }
    let literal = matches!(
        c.view(),
        CssComponentValueRef::Token(
            CssValueTokenRef::Number(_)
                | CssValueTokenRef::Dimension { .. }
                | CssValueTokenRef::Percentage(_)
        )
    );
    if !literal && !matches!(c.view(), CssComponentValueRef::Function(_)) {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::RootDomainMismatch,
            Some(c),
        ));
    }
    let root_index = values
        .items()
        .iter()
        .position(|value| std::ptr::eq(value, c))
        .expect("root component belongs to input");
    let mut result = parse_tree(c, root, root_index)?;
    if root == CalculationRoot::Integer
        && literal
        && !matches!(c.view(),CssComponentValueRef::Token(CssValueTokenRef::Number(n))if n.kind()==CssNumericTokenKind::Integer)
    {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::RootDomainMismatch,
            Some(c),
        ));
    }
    if matches!(
        root,
        CalculationRoot::Length | CalculationRoot::LengthPercentage
    ) && matches!(c.view(),CssComponentValueRef::Token(CssValueTokenRef::Number(n))if exact_zero(n))
    {
        result.ty = CssNumericType::dimension(CssNumericDimension::Length)
    }
    if !root.accepts(result.ty) {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::RootDomainMismatch,
            Some(c),
        ));
    }
    let canonical_bytes = result.canonical_len(usize::MAX).ok_or_else(|| {
        CssNumericConstructionError::component(CssComponentValueError::new(
            CssComponentValueErrorKind::CapacityOverflow,
            c.origin().clone(),
        ))
    })?;
    if canonical_bytes > limits.max_css_bytes() {
        return Err(CssNumericConstructionError::component(
            CssComponentValueError::new(CssComponentValueErrorKind::ByteLimit, c.origin().clone()),
        ));
    }
    result.components = Some(values);
    Ok(result)
}

macro_rules! root {
    ($name:ident,$kind:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            pub(crate) expression: Box<CssCalculationExpression>,
        }
        impl $name {
            pub fn try_from_components(values: CssComponentValues) -> Result<Self> {
                Self::try_from_components_with_limits(values, CssComponentValueLimits::default())
            }
            pub fn try_from_components_with_limits(
                values: CssComponentValues,
                limits: CssComponentValueLimits,
            ) -> Result<Self> {
                construct(values, CalculationRoot::$kind, limits).map(Self::from_expression)
            }
            pub(crate) fn from_expression(expression: CssCalculationExpression) -> Self {
                Self {
                    expression: Box::new(expression),
                }
            }
            pub fn expression(&self) -> CssCalculationExpressionRef<'_> {
                self.expression.as_ref().as_ref()
            }
            pub fn result_type(&self) -> CssCalculationType {
                self.expression.result_type()
            }
            pub fn numeric_type(&self) -> CssNumericType {
                self.expression.ty
            }
            pub fn components(&self) -> &CssComponentValues {
                self.expression
                    .components
                    .as_ref()
                    .expect("checked root component graph")
            }
            pub fn position(&self) -> Option<CssSourcePosition> {
                match &self.expression.origin {
                    CssValueOrigin::Parsed(v) => Some(v.span().start()),
                    _ => None,
                }
            }
            pub fn origin(&self) -> &CssValueOrigin {
                &self.expression.origin
            }
            pub fn serialize(
                &self,
            ) -> std::result::Result<CssSerializedValue, CssComponentValueError> {
                let mut tokens = Vec::new();
                self.expression.canonical_tokens(&mut tokens);
                CssSerializedValue::from_numeric_tokens(tokens)
            }
        }
    };
}
root!(CssNumberCalculation, Number);
root!(CssIntegerCalculation, Integer);
root!(CssPercentageCalculation, Percentage);
root!(CssLengthCalculation, Length);
root!(CssLengthPercentageCalculation, LengthPercentage);
root!(CssAngleCalculation, Angle);
root!(CssTimeCalculation, Time);
root!(CssFrequencyCalculation, Frequency);
root!(CssResolutionCalculation, Resolution);

/// Rechecks a mixed-context tree at a pure-length consumer boundary without
/// serialization or loss of original component provenance.
pub(crate) fn admit_pure_length(mut value: crate::CssLength) -> Option<crate::CssLength> {
    if !crate::syntax::length_has_valid_calc_shape(&value) {
        return None;
    }
    if let crate::CssLength::Calc(calculation) = &mut value {
        let mut pending = vec![calculation];
        while let Some(node) = pending.pop() {
            match node {
                crate::CssCalcLength::Typed(calculation)
                    if calculation.numeric_type()
                        != CssNumericType::dimension(CssNumericDimension::Length) =>
                {
                    // The input is already an admitted tree. A recovered closing
                    // origin remains recovered when changing its type context.
                    let expression = construct_with_policy(
                        calculation.components().clone(),
                        CalculationRoot::Length,
                        CssComponentValueLimits::default(),
                        AdmissionPolicy::RecoveredSyntax,
                    )
                    .ok()?;
                    *calculation = CssLengthPercentageCalculation::from_expression(expression);
                }
                crate::CssCalcLength::Sum(terms) => {
                    pending.extend(terms.iter_mut().map(crate::CssCalcLengthTerm::value_mut))
                }
                _ => {}
            }
        }
    }
    Some(value)
}
impl CssIntegerCalculation {
    pub fn requires_rounding(&self) -> bool {
        matches!(self.expression.kind, NodeKind::Function { .. })
    }
    pub fn literal(value: i32) -> Self {
        Self::try_from_components(
            CssComponentValues::try_new(vec![
                CssComponentValue::try_number(&value.to_string()).expect("integer spelling"),
            ])
            .expect("single component"),
        )
        .expect("integer literal")
    }
}
fn programmatic_number(value: f32) -> Option<CssComponentValues> {
    if !value.is_finite() {
        return None;
    }
    CssComponentValues::try_new(vec![
        CssComponentValue::try_number(&value.to_string()).ok()?,
    ])
    .ok()
}
impl CssNumberCalculation {
    pub fn try_literal(value: f32) -> Option<Self> {
        Self::try_from_components(programmatic_number(value)?).ok()
    }
}
impl CssPercentageCalculation {
    pub fn try_literal(value: f32) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        Self::try_from_components(
            CssComponentValues::try_new(vec![
                CssComponentValue::try_token(&format!("{value}%")).ok()?,
            ])
            .ok()?,
        )
        .ok()
    }
}
impl CssLengthCalculation {
    pub fn try_dimension(value: f32, unit: CssLengthUnit) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        Self::try_from_components(
            CssComponentValues::try_new(vec![
                CssComponentValue::try_dimension(&value.to_string(), unit.as_css_str()).ok()?,
            ])
            .ok()?,
        )
        .ok()
    }
}
fn programmatic_dimension(value: f32, unit: &str) -> Option<CssComponentValues> {
    if !value.is_finite() {
        return None;
    }
    CssComponentValues::try_new(vec![
        CssComponentValue::try_dimension(&value.to_string(), unit).ok()?,
    ])
    .ok()
}
/// Resource accounting for assembling already admitted roots. No child graph is
/// cloned until the complete iterator has passed these aggregate checks.
struct SumAssemblyBudget {
    limits: CssComponentValueLimits,
    components: usize,
    lexical: crate::component_values::CssCanonicalBuilder,
    canonical_bytes: usize,
}
impl SumAssemblyBudget {
    fn error(
        kind: CssComponentValueErrorKind,
        origin: &CssValueOrigin,
    ) -> CssNumericConstructionError {
        CssNumericConstructionError::component(CssComponentValueError::new(kind, origin.clone()))
    }
    fn add(
        total: &mut usize,
        amount: usize,
        limit: usize,
        kind: CssComponentValueErrorKind,
        origin: &CssValueOrigin,
    ) -> Result<()> {
        *total = total
            .checked_add(amount)
            .ok_or_else(|| Self::error(CssComponentValueErrorKind::CapacityOverflow, origin))?;
        if *total > limit {
            return Err(Self::error(kind, origin));
        }
        Ok(())
    }
    fn new(limits: CssComponentValueLimits) -> Result<Self> {
        let mut budget = Self {
            limits,
            components: 0,
            lexical: crate::component_values::CssCanonicalBuilder::counting(limits.max_css_bytes()),
            canonical_bytes: 0,
        };
        let origin = CssValueOrigin::Programmatic;
        Self::add(
            &mut budget.components,
            1,
            limits.max_components(),
            CssComponentValueErrorKind::ComponentLimit,
            &origin,
        )?;
        if limits.max_nesting_depth() == 0 {
            return Err(Self::error(
                CssComponentValueErrorKind::NestingLimit,
                &origin,
            ));
        }
        // Canonical numeric output includes both wrapper tokens. Lexical output
        // carries shared boundary state, including any inserted separators.
        budget.add_canonical_bytes(6, &origin)?;
        budget
            .lexical
            .push_grammar(
                crate::component_values::CssCanonicalToken::Function("calc"),
                &origin,
            )
            .map_err(CssNumericConstructionError::component)?;
        budget.reserve_closing()?;
        Ok(budget)
    }
    fn add_canonical_bytes(&mut self, canonical: usize, origin: &CssValueOrigin) -> Result<()> {
        Self::add(
            &mut self.canonical_bytes,
            canonical,
            self.limits.max_css_bytes(),
            CssComponentValueErrorKind::ByteLimit,
            origin,
        )
    }
    fn reserve_closing(&self) -> Result<()> {
        let mut bytes = self.lexical.byte_len();
        Self::add(
            &mut bytes,
            1,
            self.limits.max_css_bytes(),
            CssComponentValueErrorKind::ByteLimit,
            &CssValueOrigin::Programmatic,
        )
    }
    fn separator(&mut self, operator: CssCalculationSumOperator) -> Result<()> {
        use crate::component_values::CssCanonicalToken;
        let origin = CssValueOrigin::Programmatic;
        Self::add(
            &mut self.components,
            3,
            self.limits.max_components(),
            CssComponentValueErrorKind::ComponentLimit,
            &origin,
        )?;
        for token in [
            CssCanonicalToken::Whitespace,
            CssCanonicalToken::Delim(match operator {
                CssCalculationSumOperator::Add => '+',
                CssCalculationSumOperator::Subtract => '-',
            }),
            CssCanonicalToken::Whitespace,
        ] {
            self.lexical
                .push_grammar(token, &origin)
                .map_err(CssNumericConstructionError::component)?;
        }
        self.reserve_closing()?;
        self.add_canonical_bytes(3, &origin)
    }
    fn finish(mut self) -> Result<()> {
        self.lexical
            .push_grammar(
                crate::component_values::CssCanonicalToken::CloseParen,
                &CssValueOrigin::Programmatic,
            )
            .map_err(CssNumericConstructionError::component)
    }
    fn operand(&mut self, value: &CssLengthPercentageCalculation) -> Result<()> {
        // Iterator frames keep traversal storage proportional to nesting depth.
        let mut pending = vec![(value.components().items().iter(), 1u32)];
        while let Some((items, depth)) = pending.last_mut() {
            let Some(component) = items.next() else {
                pending.pop();
                continue;
            };
            Self::add(
                &mut self.components,
                1,
                self.limits.max_components(),
                CssComponentValueErrorKind::ComponentLimit,
                component.origin(),
            )?;
            let children = match component.view() {
                CssComponentValueRef::Function(function) => Some(function.values()),
                CssComponentValueRef::Block(block) => Some(block.values()),
                _ => None,
            };
            if let Some(children) = children {
                if *depth >= self.limits.max_nesting_depth() {
                    return Err(Self::error(
                        CssComponentValueErrorKind::NestingLimit,
                        component.origin(),
                    ));
                }
                let child_depth = *depth + 1;
                pending.push((children.items().iter(), child_depth));
            }
        }
        self.lexical
            .push_components(value.components().items())
            .map_err(CssNumericConstructionError::component)?;
        self.reserve_closing()?;
        let canonical = value.expression.canonical_len(usize::MAX).ok_or_else(|| {
            Self::error(CssComponentValueErrorKind::CapacityOverflow, value.origin())
        })?;
        self.add_canonical_bytes(canonical, value.origin())
    }
}

impl CssLengthPercentageCalculation {
    /// Assembles a symbolic sum while preserving every operand's original components.
    ///
    /// The first operand has no binary operator. Even a single operand receives
    /// a programmatic `calc()` wrapper. Root-only unitless zero is rechecked as
    /// an arithmetic operand and can fail ordinary numeric type admission.
    pub fn try_sum(
        first: Self,
        rest: impl IntoIterator<Item = (CssCalculationSumOperator, Self)>,
    ) -> Result<Self> {
        Self::try_sum_with_limits(first, rest, CssComponentValueLimits::default())
    }

    /// Assembles a sum under aggregate component, nesting and output-byte limits.
    ///
    /// Limits include inserted punctuation and whitespace. Admission stops
    /// consuming operands when a limit is exceeded, before cloning child graphs.
    /// Unlimited count and byte limits do not bound an arbitrary input iterator.
    /// Already admitted recovery origins remain intact; the public component
    /// constructor continues to reject recovered input.
    pub fn try_sum_with_limits(
        first: Self,
        rest: impl IntoIterator<Item = (CssCalculationSumOperator, Self)>,
        limits: CssComponentValueLimits,
    ) -> Result<Self> {
        let mut budget = SumAssemblyBudget::new(limits)?;
        budget.operand(&first)?;
        let mut operands = vec![(None, first)];
        for (operator, value) in rest {
            budget.separator(operator)?;
            budget.operand(&value)?;
            operands.push((Some(operator), value));
        }
        budget.finish()?;
        let mut components = Vec::new();
        for (operator, value) in &operands {
            if let Some(operator) = operator {
                for token in [
                    " ",
                    match operator {
                        CssCalculationSumOperator::Add => "+",
                        CssCalculationSumOperator::Subtract => "-",
                    },
                    " ",
                ] {
                    components.push(
                        CssComponentValue::try_token(token)
                            .map_err(CssNumericConstructionError::component)?,
                    );
                }
            }
            components.extend(value.components().items().iter().cloned());
        }
        let children = CssComponentValues::try_new(components)
            .map_err(CssNumericConstructionError::component)?;
        let function = CssComponentValue::try_function("calc", children)
            .map_err(CssNumericConstructionError::component)?;
        let values = CssComponentValues::try_new(vec![function])
            .map_err(CssNumericConstructionError::component)?;
        construct_with_policy(
            values,
            CalculationRoot::LengthPercentage,
            limits,
            AdmissionPolicy::RecoveredSyntax,
        )
        .map(Self::from_expression)
    }

    pub fn try_dimension(value: f32, unit: CssLengthUnit) -> Option<Self> {
        Self::try_from_components(programmatic_dimension(value, unit.as_css_str())?).ok()
    }
    pub fn try_percentage(value: f32) -> Option<Self> {
        Self::try_from_components(
            CssPercentageCalculation::try_literal(value)?
                .components()
                .clone(),
        )
        .ok()
    }
}
impl CssAngleCalculation {
    pub fn try_literal(value: f32, unit: crate::CssAngleUnit) -> Option<Self> {
        let unit = match unit {
            crate::CssAngleUnit::Degrees => "deg",
            crate::CssAngleUnit::Gradians => "grad",
            crate::CssAngleUnit::Radians => "rad",
            crate::CssAngleUnit::Turns => "turn",
        };
        Self::try_from_components(programmatic_dimension(value, unit)?).ok()
    }
}
impl CssTimeCalculation {
    pub fn try_literal(value: f32, unit: crate::CssTimeUnit) -> Option<Self> {
        let unit = match unit {
            crate::CssTimeUnit::Seconds => "s",
            crate::CssTimeUnit::Milliseconds => "ms",
        };
        Self::try_from_components(programmatic_dimension(value, unit)?).ok()
    }
}
impl CssFrequencyCalculation {
    pub fn try_literal(value: f32, unit: crate::CssFrequencyUnit) -> Option<Self> {
        let unit = match unit {
            crate::CssFrequencyUnit::Hertz => "hz",
            crate::CssFrequencyUnit::Kilohertz => "khz",
        };
        Self::try_from_components(programmatic_dimension(value, unit)?).ok()
    }
}

#[cfg(test)]
mod media_helpers_tests {
    use super::*;

    #[test]
    fn generic_media_numbers_admit_named_dimensions_but_not_percentages_or_products() {
        for source in [
            "calc(1)",
            "calc(1px)",
            "calc(1deg)",
            "calc(1s)",
            "calc(1Hz)",
            "calc(1dppx)",
            "calc(1fr)",
        ] {
            let values = crate::parse_component_values(source).unwrap();
            let serialized = values.serialize().unwrap();
            let context = NumericInputContext::components(&values, &serialized);
            let expression = context
                .admit_with_limits(
                    values.clone(),
                    CalculationRoot::NamedDimensionOrNumber,
                    CssComponentValueLimits::default(),
                )
                .unwrap();
            assert_eq!(expression.components.as_ref(), Some(&values));
        }
        for (source, expected) in [
            (
                "calc(1%)",
                CssNumericConstructionErrorKind::RootDomainMismatch,
            ),
            (
                "calc(1px * 1px)",
                CssNumericConstructionErrorKind::InvalidArgumentType,
            ),
        ] {
            let values = crate::parse_component_values(source).unwrap();
            let error = construct(
                values,
                CalculationRoot::NamedDimensionOrNumber,
                CssComponentValueLimits::default(),
            )
            .unwrap_err();
            assert_eq!(error.kind(), &expected, "{source}");
        }
    }

    #[test]
    fn numeric_resource_errors_retain_the_concrete_component_cause() {
        let values = crate::parse_component_values("calc(1+ 2)").unwrap();
        // Limits precede grammar admission, including malformed mathematics.
        for (limits, kind) in [
            (
                CssComponentValueLimits::try_new(0, 100, 100).unwrap(),
                CssComponentValueErrorKind::NestingLimit,
            ),
            (
                CssComponentValueLimits::try_new(1, 1, 100).unwrap(),
                CssComponentValueErrorKind::ComponentLimit,
            ),
            (
                CssComponentValueLimits::try_new(1, 100, 1).unwrap(),
                CssComponentValueErrorKind::ByteLimit,
            ),
        ] {
            let error = construct(values.clone(), CalculationRoot::Number, limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNumericConstructionErrorKind::ResourceLimit
            );
            assert_eq!(error.component_error().unwrap().kind(), kind);
            assert_eq!(
                error.origin(),
                Some(error.component_error().unwrap().origin())
            );
        }
        let compact = crate::parse_component_values("calc(1*2)").unwrap();
        let error = construct(
            compact,
            CalculationRoot::Number,
            CssComponentValueLimits::try_new(1, 10, 9).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error.component_error().unwrap().kind(),
            CssComponentValueErrorKind::ByteLimit
        );
    }
}
