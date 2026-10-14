//! Fixed numerical requests and caller-owned finite continuations.
use crate::format::bounded::{
    FiniteFormatCursor, FiniteFormatSummary, FormatDigit, FormatReply, FormatRequest,
};
use crate::format::{FormatError, RoundingMode};
use crate::serialization::{
    SerializationAction, SerializationCursor, SerializationError as CursorError,
    SerializationKernel,
};
use crate::{
    ActionDescription as Action, ActionLayout, Color, ColorError, ColorSpace, DigitSlot,
    InterpolationError, InterpolationMethod, Kernel, OperationOwner, ProtocolError,
    SerializationFamily, SerializationRequest, Transformation,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumericalRequest {
    Construct {
        space: ColorSpace,
        channels: [Option<f64>; 3],
        alpha: Option<f64>,
    },
    Convert {
        value: Color,
        destination: ColorSpace,
    },
    Prepare {
        value: Color,
        destination: ColorSpace,
    },
    Interpolate {
        first: Color,
        second: Color,
        method: InterpolationMethod,
        progress: f64,
    },
    Transform {
        value: Color,
        transformation: Transformation,
    },
    WithAlpha {
        value: Color,
        alpha: Option<f64>,
    },
    WithFamily {
        value: Color,
        family: SerializationFamily,
    },
    BlackWhiteContrast {
        value: Color,
    },
}
impl NumericalRequest {
    fn kernel(self) -> Kernel {
        match self {
            Self::Construct { .. } => Kernel::Construct,
            Self::Convert { destination, .. } => Kernel::Convert(destination),
            Self::Prepare { destination, .. } => Kernel::Prepare(destination),
            Self::Interpolate { .. } => Kernel::Interpolate,
            Self::Transform { transformation, .. } => Kernel::Transform(transformation),
            Self::WithAlpha { .. } => Kernel::WithAlpha,
            Self::WithFamily { .. } => Kernel::WithFamily,
            Self::BlackWhiteContrast { .. } => Kernel::BlackWhiteContrast,
        }
    }
    fn execute(self) -> Result<Color, ComputationError> {
        Ok(match self {
            Self::Construct {
                space,
                channels,
                alpha,
            } => Color::try_new(space, channels, alpha)?,
            Self::Convert { value, destination } => value.converted(destination)?,
            Self::Prepare { value, destination } => {
                value.prepared_for_interpolation(destination)?
            }
            Self::Interpolate {
                first,
                second,
                method,
                progress,
            } => first
                .interpolated(second, method, progress)
                .map_err(ComputationError::Interpolation)?,
            Self::Transform {
                value,
                transformation,
            } => value.transformed(transformation)?,
            Self::WithAlpha { value, alpha } => value.with_alpha(alpha)?,
            Self::WithFamily { value, family } => value.with_serialization_family(family)?,
            Self::BlackWhiteContrast { value } => value.black_white_contrast()?,
        })
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ComputationError {
    Numerical(ColorError),
    Interpolation(InterpolationError),
    Protocol(ProtocolError),
}
impl From<ColorError> for ComputationError {
    fn from(e: ColorError) -> Self {
        Self::Numerical(e)
    }
}
impl From<ProtocolError> for ComputationError {
    fn from(e: ProtocolError) -> Self {
        Self::Protocol(e)
    }
}
impl std::fmt::Display for ComputationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Numerical(e) => e.fmt(f),
            Self::Interpolation(e) => e.fmt(f),
            Self::Protocol(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for ComputationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Numerical(e) => e,
            Self::Interpolation(e) => e,
            Self::Protocol(e) => e,
        })
    }
}
#[derive(Clone, Copy)]
enum Status {
    Pending,
    Ready,
    Failed,
    Abandoned,
}
pub struct Computation<'owner> {
    _owner: &'owner OperationOwner,
    original: NumericalRequest,
    status: Status,
    value: Option<Color>,
    failure: Option<ComputationError>,
}
impl<'owner> Computation<'owner> {
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    pub const fn action_bytes() -> usize {
        std::mem::size_of::<ComputationAction<'_, '_>>()
    }
    pub fn new(
        owner: &'owner mut OperationOwner,
        request: NumericalRequest,
    ) -> Result<Self, ProtocolError> {
        owner.begin()?;
        Ok(Self {
            _owner: owner,
            original: request,
            status: Status::Pending,
            value: None,
            failure: None,
        })
    }
    pub const fn original(&self) -> NumericalRequest {
        self.original
    }
    pub fn action(&self) -> Action {
        match self.status {
            Status::Pending => Action::Compute(self.original.kernel()),
            Status::Ready => Action::Ready,
            Status::Failed => Action::Failed,
            Status::Abandoned => Action::Abandoned,
        }
    }
    pub fn next_action(&mut self) -> ComputationAction<'_, 'owner> {
        ComputationAction { operation: self }
    }
    pub fn failure(&self) -> Option<&ComputationError> {
        self.failure.as_ref()
    }
    pub fn abandon(&mut self) {
        self.status = Status::Abandoned;
    }
    pub fn take(slot: &mut Option<Self>) -> Result<Color, ComputationError> {
        let operation = slot.as_ref().ok_or(ProtocolError::EmptySlot)?;
        match operation.status {
            Status::Pending => return Err(ProtocolError::NotReady.into()),
            Status::Failed => {
                return Err(operation
                    .failure
                    .expect("failed computation retains its first error"));
            }
            Status::Abandoned => return Err(ProtocolError::Terminated.into()),
            Status::Ready => {}
        }
        Ok(slot
            .take()
            .expect("shared preflight proved ready")
            .value
            .expect("ready computation has a value"))
    }
    pub fn recover(slot: &mut Option<Self>) -> Result<NumericalRequest, ProtocolError> {
        Ok(slot.take().ok_or(ProtocolError::EmptySlot)?.original)
    }
}
pub struct ComputationAction<'step, 'owner> {
    operation: &'step mut Computation<'owner>,
}
impl ComputationAction<'_, '_> {
    pub fn description(&self) -> Action {
        self.operation.action()
    }
    pub fn layout(&self) -> ActionLayout {
        crate::protocol::layout(self.description())
    }
    pub fn compute(self) -> Result<(), ComputationError> {
        if !matches!(self.operation.status, Status::Pending) {
            return Err(ProtocolError::WrongAction.into());
        }
        match self.operation.original.execute() {
            Ok(value) => {
                self.operation.value = Some(value);
                self.operation.status = Status::Ready;
                Ok(())
            }
            Err(error) => {
                self.operation.failure = Some(error);
                self.operation.status = Status::Failed;
                Err(error)
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerializationResource {
    Actions,
    Digits,
    OutputBytes,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SerializationLimits {
    pub actions: usize,
    pub digits: usize,
    pub output_bytes: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SerializationError {
    Numerical(ColorError),
    Protocol(ProtocolError),
    Limit {
        resource: SerializationResource,
        required: usize,
        allowed: usize,
    },
    Allocation {
        resource: SerializationResource,
    },
    CountOverflow,
}
impl From<ProtocolError> for SerializationError {
    fn from(e: ProtocolError) -> Self {
        Self::Protocol(e)
    }
}
impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Numerical(e) => e.fmt(f),
            Self::Protocol(e) => e.fmt(f),
            _ => write!(f, "color serialization: {self:?}"),
        }
    }
}
impl std::error::Error for SerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Numerical(e) => Some(e),
            Self::Protocol(e) => Some(e),
            _ => None,
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) enum Original {
    Serialization(Color, SerializationRequest),
    Format(f64, RoundingMode),
}
#[expect(
    clippy::large_enum_variant,
    reason = "Resolved serialization planning and finite formatting are exclusive inline numerical continuations; their actual register layout is admitted by callers without an unadmitted heap allocation"
)]
enum Cursor {
    Serialization(SerializationCursor),
    Format(FiniteFormatCursor),
}
pub(crate) struct Stream<'owner> {
    owner: &'owner OperationOwner,
    phase: usize,
    pub(crate) original: Original,
    cursor: Cursor,
    status: Status,
    failure: Option<SerializationError>,
    pub(crate) emitted: usize,
}
impl<'owner> Stream<'owner> {
    pub(crate) fn serialization(
        owner: &'owner mut OperationOwner,
        value: Color,
        request: SerializationRequest,
    ) -> Result<Self, ProtocolError> {
        let phase = owner.begin()?;
        Ok(Self {
            owner,
            phase,
            original: Original::Serialization(value, request),
            cursor: Cursor::Serialization(SerializationCursor::new(value, request)),
            status: Status::Pending,
            failure: None,
            emitted: 0,
        })
    }
    pub(crate) fn format(
        owner: &'owner mut OperationOwner,
        value: f64,
        rounding: RoundingMode,
    ) -> Result<Self, FormatError> {
        let cursor =
            FiniteFormatCursor::new(value, rounding).map_err(|_| FormatError::NonFinite)?;
        let phase = owner.begin().map_err(FormatError::Protocol)?;
        Ok(Self {
            owner,
            phase,
            original: Original::Format(value, rounding),
            cursor: Cursor::Format(cursor),
            status: Status::Pending,
            failure: None,
            emitted: 0,
        })
    }
    pub(crate) fn action(&self) -> Action {
        match self.status {
            Status::Failed => return Action::Failed,
            Status::Abandoned => return Action::Abandoned,
            Status::Ready => return Action::Ready,
            Status::Pending => {}
        }
        match &self.cursor {
            Cursor::Serialization(cursor) => match cursor.action() {
                SerializationAction::Plan | SerializationAction::Compute => {
                    Action::Compute(match cursor.kernel() {
                        SerializationKernel::Plan => Kernel::SerializationPlan,
                        SerializationKernel::Format => cursor
                            .format_kernel()
                            .map_or(Kernel::SerializationPiece, Into::into),
                        SerializationKernel::Transition => Kernel::SerializationPiece,
                    })
                }
                SerializationAction::ReadDigit { index } => Action::ReadDigit { index },
                SerializationAction::AppendDigit { index } => Action::AppendDigit { index },
                SerializationAction::ReplaceDigit { index } => Action::ReplaceDigit { index },
                SerializationAction::Emit { unit } => Action::EmitByte { unit },
                SerializationAction::Done => Action::Ready,
                SerializationAction::Stopped => Action::Abandoned,
            },
            Cursor::Format(cursor) => match cursor.next_request() {
                FormatRequest::Compute(kernel) => Action::Compute(kernel.into()),
                FormatRequest::ReadDigit { index } => Action::ReadDigit { index },
                FormatRequest::AppendDigit { index, .. } => Action::AppendDigit { index },
                FormatRequest::ReplaceDigit { index, .. } => Action::ReplaceDigit { index },
                FormatRequest::EmitByte { unit } => Action::EmitByte { unit },
                FormatRequest::Done(_) => Action::Ready,
            },
        }
    }
    pub(crate) fn ready(&self) -> Result<(), SerializationError> {
        match self.status {
            Status::Failed => Err(self.failure.expect("failed stream retains first error")),
            Status::Abandoned => Err(ProtocolError::Terminated.into()),
            _ if self.action() == Action::Ready => {
                if let Cursor::Serialization(c) = &self.cursor
                    && c.completed_length()? != self.emitted
                {
                    return Err(ProtocolError::WrongIndex.into());
                }
                Ok(())
            }
            _ => Err(ProtocolError::NotReady.into()),
        }
    }
    pub(crate) fn abandon(&mut self) {
        if let Cursor::Serialization(c) = &mut self.cursor {
            c.abandon();
        }
        self.status = Status::Abandoned;
    }
    pub(crate) fn planned_length(&self) -> Option<usize> {
        match &self.cursor {
            Cursor::Format(c) => c.planned_length(),
            _ => None,
        }
    }
    pub(crate) fn summary(&self) -> Result<FiniteFormatSummary, FormatError> {
        self.ready().map_err(|e| match e {
            SerializationError::Protocol(p) => FormatError::Protocol(p),
            _ => FormatError::Protocol(ProtocolError::WrongReply),
        })?;
        match &self.cursor {
            Cursor::Format(c) => match c.next_request() {
                FormatRequest::Done(s) => Ok(s),
                _ => Err(FormatError::Protocol(ProtocolError::NotReady)),
            },
            _ => Err(FormatError::Protocol(ProtocolError::WrongAction)),
        }
    }
    fn compute(&mut self) -> Result<(), SerializationError> {
        if !matches!(self.action(), Action::Compute(_)) {
            return Err(ProtocolError::WrongAction.into());
        }
        let result = match &mut self.cursor {
            Cursor::Serialization(c) => c.compute().map_err(|e| match e {
                CursorError::Numerical(e) => SerializationError::Numerical(e),
                CursorError::Protocol(e) => SerializationError::Protocol(e),
            }),
            Cursor::Format(c) => {
                let FormatRequest::Compute(k) = c.next_request() else {
                    unreachable!()
                };
                c.complete(FormatReply::Computed(k))
                    .map_err(|e| SerializationError::Protocol(e.into()))
            }
        };
        if let Err(e @ SerializationError::Numerical(_)) = result {
            self.failure = Some(e);
            self.status = Status::Failed;
        }
        result
    }
    fn digit(&self, index: usize) -> Result<FormatDigit, ProtocolError> {
        match &self.cursor {
            Cursor::Serialization(c) => c.digit_record(index),
            Cursor::Format(c) => match c.next_request() {
                FormatRequest::AppendDigit { index: i, digit }
                | FormatRequest::ReplaceDigit { index: i, digit }
                    if index == i =>
                {
                    Ok(digit)
                }
                _ => Err(ProtocolError::WrongIndex),
            },
        }
    }
}
pub struct StreamAction<'step, 'owner> {
    pub(crate) stream: &'step mut Stream<'owner>,
}
impl<'owner> StreamAction<'_, 'owner> {
    pub fn description(&self) -> Action {
        self.stream.action()
    }
    pub fn layout(&self) -> ActionLayout {
        crate::protocol::layout(self.description())
    }
    pub fn compute(self) -> Result<(), SerializationError> {
        self.stream.compute()
    }
    pub fn read_digit(self, index: usize, slot: &DigitSlot<'_>) -> Result<(), ProtocolError> {
        if self.description() != (Action::ReadDigit { index }) {
            return Err(ProtocolError::WrongAction);
        }
        let digit = slot.qualify(self.stream.owner, self.stream.phase, index)?;
        match &mut self.stream.cursor {
            Cursor::Serialization(c) => c.read_digit(index, digit),
            Cursor::Format(c) => c
                .complete(FormatReply::Digit { index, digit })
                .map_err(Into::into),
        }
    }
    pub fn record(&mut self) -> Result<DigitSlot<'owner>, ProtocolError> {
        let index = match self.description() {
            Action::AppendDigit { index } | Action::ReplaceDigit { index } => index,
            _ => return Err(ProtocolError::WrongAction),
        };
        let digit = self.stream.digit(index)?;
        Ok(DigitSlot::new(
            self.stream.owner,
            self.stream.phase,
            index,
            digit,
        ))
    }
    pub fn written(self, index: usize, slot: &DigitSlot<'_>) -> Result<(), ProtocolError> {
        let action = self.description();
        let digit = slot.qualify(self.stream.owner, self.stream.phase, index)?;
        match &mut self.stream.cursor {
            Cursor::Serialization(c) => c.digit_written(index, digit),
            Cursor::Format(c) => {
                let reply = match action {
                    Action::AppendDigit { index: i } if i == index => {
                        FormatReply::Appended { index, digit }
                    }
                    Action::ReplaceDigit { index: i } if i == index => {
                        FormatReply::Replaced { index, digit }
                    }
                    _ => return Err(ProtocolError::WrongAction),
                };
                c.complete(reply).map_err(Into::into)
            }
        }
    }
    pub fn emitted(self, unit: u8) -> Result<(), ProtocolError> {
        if self.description() != (Action::EmitByte { unit }) {
            return Err(ProtocolError::WrongReply);
        }
        let next = self
            .stream
            .emitted
            .checked_add(1)
            .ok_or(ProtocolError::Overflow)?;
        match &mut self.stream.cursor {
            Cursor::Serialization(c) => c.emitted(unit)?,
            Cursor::Format(c) => c
                .complete(FormatReply::Emitted { unit })
                .map_err(ProtocolError::from)?,
        };
        self.stream.emitted = next;
        Ok(())
    }
}
pub struct Serialization<'owner> {
    stream: Stream<'owner>,
}
#[derive(Debug)]
pub struct SerializationReceipt {
    value: Color,
    length: usize,
}
impl SerializationReceipt {
    pub const fn value(&self) -> Color {
        self.value
    }
    pub const fn length(&self) -> usize {
        self.length
    }
}
#[derive(Debug)]
pub struct SerializationRecovery {
    value: Color,
    request: SerializationRequest,
    length: usize,
}
impl SerializationRecovery {
    pub const fn value(&self) -> Color {
        self.value
    }
    pub const fn request(&self) -> SerializationRequest {
        self.request
    }
    pub const fn length(&self) -> usize {
        self.length
    }
}
impl<'owner> Serialization<'owner> {
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    pub const fn action_bytes() -> usize {
        std::mem::size_of::<StreamAction<'_, '_>>()
    }
    pub fn new(
        owner: &'owner mut OperationOwner,
        value: Color,
        request: SerializationRequest,
    ) -> Result<Self, ProtocolError> {
        Ok(Self {
            stream: Stream::serialization(owner, value, request)?,
        })
    }
    pub fn action(&self) -> Action {
        self.stream.action()
    }
    pub fn next_action(&mut self) -> StreamAction<'_, 'owner> {
        StreamAction {
            stream: &mut self.stream,
        }
    }
    pub fn failure(&self) -> Option<&SerializationError> {
        self.stream.failure.as_ref()
    }
    pub fn abandon(&mut self) {
        self.stream.abandon();
    }
    pub fn take(slot: &mut Option<Self>) -> Result<SerializationReceipt, SerializationError> {
        slot.as_ref()
            .ok_or(ProtocolError::EmptySlot)?
            .stream
            .ready()?;
        let s = slot.take().expect("shared preflight proved ready").stream;
        let Original::Serialization(value, _) = s.original else {
            unreachable!()
        };
        Ok(SerializationReceipt {
            value,
            length: s.emitted,
        })
    }
    pub fn recover(slot: &mut Option<Self>) -> Result<SerializationRecovery, ProtocolError> {
        let s = slot.take().ok_or(ProtocolError::EmptySlot)?.stream;
        let Original::Serialization(value, request) = s.original else {
            unreachable!()
        };
        Ok(SerializationRecovery {
            value,
            request,
            length: s.emitted,
        })
    }
}
pub fn serialize(
    value: Color,
    request: SerializationRequest,
    limits: SerializationLimits,
) -> Result<String, SerializationError> {
    fn admit(
        resource: SerializationResource,
        required: usize,
        allowed: usize,
    ) -> Result<(), SerializationError> {
        if required > allowed {
            Err(SerializationError::Limit {
                resource,
                required,
                allowed,
            })
        } else {
            Ok(())
        }
    }
    let mut owner = OperationOwner::new();
    let mut operation = Some(Serialization::new(&mut owner, value, request)?);
    let mut digits = Vec::new();
    let mut output = Vec::new();
    let mut actions = 0usize;
    loop {
        let op = operation.as_mut().expect("live serialization");
        if op.action() == Action::Ready {
            break;
        }
        actions = actions
            .checked_add(1)
            .ok_or(SerializationError::CountOverflow)?;
        admit(SerializationResource::Actions, actions, limits.actions)?;
        let mut guard = op.next_action();
        match guard.description() {
            Action::Compute(_) => guard.compute()?,
            Action::ReadDigit { index } => {
                guard.read_digit(index, digits.get(index).ok_or(ProtocolError::WrongIndex)?)?
            }
            Action::AppendDigit { index } => {
                if index != digits.len() {
                    return Err(ProtocolError::WrongIndex.into());
                }
                admit(
                    SerializationResource::Digits,
                    index
                        .checked_add(1)
                        .ok_or(SerializationError::CountOverflow)?,
                    limits.digits,
                )?;
                digits
                    .try_reserve(1)
                    .map_err(|_| SerializationError::Allocation {
                        resource: SerializationResource::Digits,
                    })?;
                digits.push(guard.record()?);
                guard.written(index, &digits[index])?
            }
            Action::ReplaceDigit { index } => {
                let actual = digits.get_mut(index).ok_or(ProtocolError::WrongIndex)?;
                *actual = guard.record()?;
                guard.written(index, actual)?
            }
            Action::EmitByte { unit } => {
                admit(
                    SerializationResource::OutputBytes,
                    output
                        .len()
                        .checked_add(1)
                        .ok_or(SerializationError::CountOverflow)?,
                    limits.output_bytes,
                )?;
                output
                    .try_reserve(1)
                    .map_err(|_| SerializationError::Allocation {
                        resource: SerializationResource::OutputBytes,
                    })?;
                output.push(unit);
                guard.emitted(unit)?
            }
            _ => return Err(ProtocolError::Terminated.into()),
        }
    }
    let receipt = Serialization::take(&mut operation)?;
    if receipt.length() != output.len() {
        return Err(ProtocolError::WrongIndex.into());
    }
    String::from_utf8(output).map_err(|_| ProtocolError::WrongReply.into())
}
