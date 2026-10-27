//! Fixed numerical requests and caller-owned finite continuations.
//!
//! Each continuation is a typestate: a transition consumes it and returns the
//! next state, and only the action a step issues exposes that action's single
//! completion method. Dropping a continuation or an action abandons it.
use crate::format::RoundingMode;
use crate::format::bounded::{
    FiniteFormatCursor, FiniteFormatSummary, FormatDigit, FormatReply, FormatRequest,
};
use crate::serialization::{SerializationCursor, SerializationKernel};
use crate::{
    ActionLayout, Color, ColorSpace, DigitSlot, Finite, InterpolationMethod, Kernel,
    OperationOwner, SerializationRequest, Transformation, UnitInterval,
};

/// One numerical operation over proven operands; every request is total.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumericalRequest {
    Construct {
        space: ColorSpace,
        channels: [Option<Finite>; 3],
        alpha: Option<UnitInterval>,
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
        progress: UnitInterval,
    },
    WithAlpha {
        value: Color,
        alpha: Option<UnitInterval>,
    },
    WithColorFunction {
        value: Color,
    },
    ToLegacySrgb {
        value: Color,
    },
    BlackWhiteContrast {
        value: Color,
    },
    Transform {
        value: Color,
        transformation: Transformation,
    },
}
impl NumericalRequest {
    /// The fixed kernel this request executes.
    #[must_use]
    pub const fn kernel(self) -> Kernel {
        match self {
            Self::Construct { .. } => Kernel::Construct,
            Self::Convert { destination, .. } => Kernel::Convert(destination),
            Self::Prepare { destination, .. } => Kernel::Prepare(destination),
            Self::Interpolate { .. } => Kernel::Interpolate,
            Self::WithAlpha { .. } => Kernel::WithAlpha,
            Self::WithColorFunction { .. } => Kernel::WithColorFunction,
            Self::ToLegacySrgb { .. } => Kernel::ToLegacySrgb,
            Self::BlackWhiteContrast { .. } => Kernel::BlackWhiteContrast,
            Self::Transform { transformation, .. } => Kernel::Transform(transformation),
        }
    }
    /// Executes the request with the same kernels as the `Color` methods.
    #[must_use]
    pub fn execute(self) -> Color {
        match self {
            Self::Construct {
                space,
                channels,
                alpha,
            } => Color::new(space, channels, alpha),
            Self::Convert { value, destination } => value.converted(destination),
            Self::Prepare { value, destination } => value.prepared_for_interpolation(destination),
            Self::Interpolate {
                first,
                second,
                method,
                progress,
            } => first.interpolated(second, method, progress),
            Self::WithAlpha { value, alpha } => value.with_alpha(alpha),
            Self::WithColorFunction { value } => value.with_color_function(),
            Self::ToLegacySrgb { value } => value.to_legacy_srgb(),
            Self::BlackWhiteContrast { value } => value.black_white_contrast(),
            Self::Transform {
                value,
                transformation,
            } => value.transformed(transformation),
        }
    }
}

/// One pending fixed-register numerical computation over an exclusive owner.
///
/// Computing consumes the continuation and returns the total result, which no
/// longer borrows the owner. Dropping it abandons the request.
pub struct Computation<'owner> {
    _owner: &'owner OperationOwner,
    request: NumericalRequest,
}
impl<'owner> Computation<'owner> {
    /// Begins one computation; the owner stays borrowed until it ends.
    ///
    /// # Panics
    ///
    /// If `owner` has already begun `usize::MAX` operations.
    pub fn new(owner: &'owner mut OperationOwner, request: NumericalRequest) -> Self {
        owner.begin();
        Self {
            _owner: owner,
            request,
        }
    }
    /// Actual compiled continuation layout, admitted before construction.
    #[must_use]
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    /// The fixed kernel [`compute`](Self::compute) executes.
    #[must_use]
    pub const fn kernel(&self) -> Kernel {
        self.request.kernel()
    }
    /// The computation observes no record and emits nothing.
    #[must_use]
    pub const fn layout(&self) -> ActionLayout {
        ActionLayout::COMPUTE
    }
    /// Executes the kernel; the result is total over proven operands.
    #[must_use]
    pub fn compute(self) -> Color {
        self.request.execute()
    }
    /// Declines the computation and returns its unexecuted request.
    #[must_use]
    pub const fn into_request(self) -> NumericalRequest {
        self.request
    }
}
impl std::fmt::Debug for Computation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Computation")
            .field("request", &self.request)
            .finish_non_exhaustive()
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
/// A refused ordinary serialization resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SerializationError {
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
impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "color serialization: {self:?}")
    }
}
impl std::error::Error for SerializationError {}

/// The single request a digit-banked continuation waits on.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Pending {
    Compute(Kernel),
    ReadDigit { index: usize },
    AppendDigit { index: usize, digit: FormatDigit },
    ReplaceDigit { index: usize, digit: FormatDigit },
    Emit { unit: u8 },
    Complete,
}
#[expect(
    clippy::large_enum_variant,
    reason = "Resolved serialization planning and finite formatting are exclusive inline numerical continuations; their actual register layout is admitted by callers without an unadmitted heap allocation"
)]
enum Cursor {
    Serialization(SerializationCursor),
    Format(FiniteFormatCursor),
}
/// The shared digit-banked continuation behind serialization and formatting.
///
/// Typed actions call only the completion of the request they were issued
/// for. Record identity is the caller's input and is checked; every other
/// mismatch is a defect in this crate and panics.
pub(crate) struct Stream<'owner> {
    owner: &'owner OperationOwner,
    phase: usize,
    cursor: Cursor,
    emitted: usize,
}
impl<'owner> Stream<'owner> {
    pub(crate) fn serialization(
        owner: &'owner mut OperationOwner,
        value: Color,
        request: SerializationRequest,
    ) -> Self {
        let phase = owner.begin();
        Self {
            owner,
            phase,
            cursor: Cursor::Serialization(SerializationCursor::new(value, request)),
            emitted: 0,
        }
    }
    pub(crate) fn format(
        owner: &'owner mut OperationOwner,
        value: Finite,
        rounding: RoundingMode,
    ) -> Self {
        let phase = owner.begin();
        Self {
            owner,
            phase,
            cursor: Cursor::Format(FiniteFormatCursor::new(value, rounding)),
            emitted: 0,
        }
    }
    pub(crate) fn pending(&self) -> Pending {
        match &self.cursor {
            Cursor::Serialization(cursor) => {
                use crate::serialization::SerializationAction as A;
                match cursor.action() {
                    A::Plan | A::Compute => Pending::Compute(match cursor.kernel() {
                        SerializationKernel::Plan => Kernel::SerializationPlan,
                        SerializationKernel::Format => cursor
                            .format_kernel()
                            .map_or(Kernel::SerializationPiece, Into::into),
                        SerializationKernel::Transition => Kernel::SerializationPiece,
                    }),
                    A::ReadDigit { index } => Pending::ReadDigit { index },
                    A::AppendDigit { index } => Pending::AppendDigit {
                        index,
                        digit: cursor.pending_digit(index),
                    },
                    A::ReplaceDigit { index } => Pending::ReplaceDigit {
                        index,
                        digit: cursor.pending_digit(index),
                    },
                    A::Emit { unit } => Pending::Emit { unit },
                    A::Done => {
                        assert_eq!(
                            cursor.emitted(),
                            self.emitted,
                            "the serialization plan and its acknowledged output agree"
                        );
                        Pending::Complete
                    }
                }
            }
            Cursor::Format(cursor) => match cursor.next_request() {
                FormatRequest::Compute(kernel) => Pending::Compute(kernel.into()),
                FormatRequest::ReadDigit { index } => Pending::ReadDigit { index },
                FormatRequest::AppendDigit { index, digit } => {
                    Pending::AppendDigit { index, digit }
                }
                FormatRequest::ReplaceDigit { index, digit } => {
                    Pending::ReplaceDigit { index, digit }
                }
                FormatRequest::EmitByte { unit } => Pending::Emit { unit },
                FormatRequest::Done(_) => Pending::Complete,
            },
        }
    }
    pub(crate) const fn emitted(&self) -> usize {
        self.emitted
    }
    pub(crate) fn serialized_value(&self) -> Color {
        match &self.cursor {
            Cursor::Serialization(cursor) => cursor.value(),
            Cursor::Format(_) => panic!("a finite formatter serializes no color"),
        }
    }
    pub(crate) fn planned_length(&self) -> Option<usize> {
        match &self.cursor {
            Cursor::Format(cursor) => cursor.planned_length(),
            Cursor::Serialization(_) => None,
        }
    }
    pub(crate) fn summary(&self) -> FiniteFormatSummary {
        match &self.cursor {
            Cursor::Format(cursor) => match cursor.next_request() {
                FormatRequest::Done(summary) => summary,
                request => panic!("a format summary was requested while {request:?} is pending"),
            },
            Cursor::Serialization(_) => panic!("a color serialization has no format summary"),
        }
    }
    pub(crate) fn compute(&mut self) {
        match &mut self.cursor {
            Cursor::Serialization(cursor) => cursor.compute(),
            Cursor::Format(cursor) => match cursor.next_request() {
                FormatRequest::Compute(kernel) => cursor
                    .complete(FormatReply::Computed(kernel))
                    .expect("a pending format kernel accepts its own completion"),
                request => panic!("format compute issued for pending {request:?}"),
            },
        }
    }
    /// Completes the pending read at `index` with the caller's record.
    ///
    /// # Panics
    ///
    /// If `slot` belongs to another operation or index.
    #[track_caller]
    pub(crate) fn read(&mut self, index: usize, slot: &DigitSlot<'_>) {
        let digit = slot.digit_for(self.owner, self.phase, index);
        match &mut self.cursor {
            Cursor::Serialization(cursor) => cursor.read_digit(index, digit),
            Cursor::Format(cursor) => cursor
                .complete(FormatReply::Digit { index, digit })
                .expect("a pending digit read accepts its own record"),
        }
    }
    /// Mints the record of the pending write at `index`.
    pub(crate) fn record(&self, index: usize, digit: FormatDigit) -> DigitSlot<'owner> {
        DigitSlot::new(self.owner, self.phase, index, digit)
    }
    /// Acknowledges that the caller stored the record minted for `index`.
    ///
    /// # Panics
    ///
    /// If `stored` belongs to another operation or index, or holds another digit.
    #[track_caller]
    pub(crate) fn store(&mut self, index: usize, digit: FormatDigit, stored: &DigitSlot<'_>) {
        assert!(
            stored.digit_for(self.owner, self.phase, index) == digit,
            "stored digit record is not the record minted for index {index}"
        );
        match &mut self.cursor {
            Cursor::Serialization(cursor) => cursor.digit_written(index, digit),
            Cursor::Format(cursor) => {
                let reply = match cursor.next_request() {
                    FormatRequest::AppendDigit { .. } => FormatReply::Appended { index, digit },
                    FormatRequest::ReplaceDigit { .. } => FormatReply::Replaced { index, digit },
                    request => panic!("digit write issued for pending {request:?}"),
                };
                cursor
                    .complete(reply)
                    .expect("a pending digit write accepts its own record");
            }
        }
    }
    pub(crate) fn emit(&mut self, unit: u8) {
        let next = self
            .emitted
            .checked_add(1)
            .expect("emitted output length fits in usize");
        match &mut self.cursor {
            Cursor::Serialization(cursor) => cursor.emitted_unit(unit),
            Cursor::Format(cursor) => cursor
                .complete(FormatReply::Emitted { unit })
                .expect("a pending emission accepts its own byte"),
        }
        self.emitted = next;
    }
}
/// Shows the operation phase, acknowledged output and pending request; the
/// owner anchor and the cursor's plan and registers stay private.
impl std::fmt::Debug for Stream<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stream")
            .field("phase", &self.phase)
            .field("emitted", &self.emitted)
            .field("pending", &self.pending())
            .finish_non_exhaustive()
    }
}

/// A pending resolved serialization over an exclusive owner.
///
/// The caller owns every digit record and output byte. [`step`](Self::step)
/// consumes the serialization and yields either its next action or its
/// receipt; dropping it, or an action, abandons the serialization.
pub struct Serialization<'owner> {
    stream: Stream<'owner>,
}
impl<'owner> Serialization<'owner> {
    /// Begins one serialization; the owner stays borrowed until it ends and
    /// every record it minted is dropped.
    ///
    /// # Panics
    ///
    /// If `owner` has already begun `usize::MAX` operations.
    pub fn new(
        owner: &'owner mut OperationOwner,
        value: Color,
        request: SerializationRequest,
    ) -> Self {
        Self {
            stream: Stream::serialization(owner, value, request),
        }
    }
    /// Actual compiled continuation layout, admitted before construction.
    #[must_use]
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    /// Actual compiled layout of one issued action, which owns the continuation.
    #[must_use]
    pub const fn action_bytes() -> usize {
        std::mem::size_of::<SerializationAction<'static>>()
    }
    /// Consumes the serialization and yields its next action or its receipt.
    #[must_use]
    pub fn step(self) -> SerializationStep<'owner> {
        let stream = self.stream;
        SerializationStep::Pending(match stream.pending() {
            Pending::Compute(kernel) => {
                SerializationAction::Compute(SerializationCompute { stream, kernel })
            }
            Pending::ReadDigit { index } => {
                SerializationAction::ReadDigit(ReadDigit { stream, index })
            }
            Pending::AppendDigit { index, digit } => SerializationAction::AppendDigit(WriteDigit {
                stream,
                index,
                digit,
            }),
            Pending::ReplaceDigit { index, digit } => {
                SerializationAction::ReplaceDigit(WriteDigit {
                    stream,
                    index,
                    digit,
                })
            }
            Pending::Emit { unit } => SerializationAction::Emit(Emit { stream, unit }),
            Pending::Complete => {
                return SerializationStep::Complete(SerializationReceipt {
                    value: stream.serialized_value(),
                    length: stream.emitted(),
                });
            }
        })
    }
}
impl std::fmt::Debug for Serialization<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Serialization")
            .field("stream", &self.stream)
            .finish()
    }
}
/// The next action of a pending serialization, or its completed receipt.
#[derive(Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "A pending action owns the inline serialization continuation, whose register layout callers admit; boxing it would add an unadmitted heap allocation per step"
)]
pub enum SerializationStep<'owner> {
    Pending(SerializationAction<'owner>),
    Complete(SerializationReceipt),
}
/// Exactly one pending action; it owns the serialization until completion.
#[derive(Debug)]
pub enum SerializationAction<'owner> {
    Compute(SerializationCompute<'owner>),
    ReadDigit(ReadDigit<'owner>),
    AppendDigit(WriteDigit<'owner>),
    ReplaceDigit(WriteDigit<'owner>),
    Emit(Emit<'owner>),
}
impl<'owner> SerializationAction<'owner> {
    /// Payload observations and writes to admit before completing the action.
    #[must_use]
    pub const fn layout(&self) -> ActionLayout {
        match self {
            Self::Compute(_) => ActionLayout::COMPUTE,
            Self::ReadDigit(_) => ActionLayout::READ_DIGIT,
            Self::AppendDigit(_) | Self::ReplaceDigit(_) => ActionLayout::WRITE_DIGIT,
            Self::Emit(_) => ActionLayout::EMIT,
        }
    }
    /// Declines the action without effect; the serialization stays pending.
    #[must_use]
    pub fn into_serialization(self) -> Serialization<'owner> {
        let stream = match self {
            Self::Compute(SerializationCompute { stream, .. })
            | Self::ReadDigit(ReadDigit { stream, .. })
            | Self::AppendDigit(WriteDigit { stream, .. })
            | Self::ReplaceDigit(WriteDigit { stream, .. })
            | Self::Emit(Emit { stream, .. }) => stream,
        };
        Serialization { stream }
    }
}
/// One pending fixed-register planning or formatting kernel.
pub struct SerializationCompute<'owner> {
    stream: Stream<'owner>,
    kernel: Kernel,
}
impl<'owner> SerializationCompute<'owner> {
    /// The fixed kernel [`compute`](Self::compute) executes.
    #[must_use]
    pub const fn kernel(&self) -> Kernel {
        self.kernel
    }
    /// Executes the kernel.
    #[must_use]
    pub fn compute(mut self) -> Serialization<'owner> {
        self.stream.compute();
        Serialization {
            stream: self.stream,
        }
    }
}
impl std::fmt::Debug for SerializationCompute<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerializationCompute")
            .field("kernel", &self.kernel)
            .finish_non_exhaustive()
    }
}
/// One pending read of a caller-owned digit record this serialization minted.
pub struct ReadDigit<'owner> {
    stream: Stream<'owner>,
    index: usize,
}
impl<'owner> ReadDigit<'owner> {
    /// The caller-owned record index to read.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }
    /// Reads the caller's record stored at [`index`](Self::index).
    ///
    /// # Panics
    ///
    /// If `slot` was minted by another operation or for another index.
    #[must_use]
    #[track_caller]
    pub fn read(mut self, slot: &DigitSlot<'_>) -> Serialization<'owner> {
        self.stream.read(self.index, slot);
        Serialization {
            stream: self.stream,
        }
    }
}
impl std::fmt::Debug for ReadDigit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadDigit")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}
/// One pending append or replacement of a caller-owned digit record.
pub struct WriteDigit<'owner> {
    stream: Stream<'owner>,
    index: usize,
    digit: FormatDigit,
}
impl<'owner> WriteDigit<'owner> {
    /// The caller-owned record index to write.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }
    /// Mints the record to store at [`index`](Self::index). Mint it only after
    /// the storage admission; the receipt acknowledges actual storage.
    #[must_use]
    pub fn record(self) -> (DigitSlot<'owner>, DigitReceipt<'owner>) {
        (
            self.stream.record(self.index, self.digit),
            DigitReceipt {
                stream: self.stream,
                index: self.index,
                digit: self.digit,
            },
        )
    }
}
impl std::fmt::Debug for WriteDigit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteDigit")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}
/// A minted digit record awaiting its storage acknowledgement.
pub struct DigitReceipt<'owner> {
    stream: Stream<'owner>,
    index: usize,
    digit: FormatDigit,
}
impl<'owner> DigitReceipt<'owner> {
    /// Acknowledges that `stored` now holds the minted record.
    ///
    /// The caller must pass the record it actually stored at this index. The
    /// check covers only the record's operation, index and digit, so a
    /// substituted record with the same identity is accepted, and a later
    /// read of a different stored record yields wrong output that the fixed
    /// registers cannot detect.
    ///
    /// # Panics
    ///
    /// If `stored` is not the record minted by the matching `record`.
    #[must_use]
    #[track_caller]
    pub fn stored(mut self, stored: &DigitSlot<'_>) -> Serialization<'owner> {
        self.stream.store(self.index, self.digit, stored);
        Serialization {
            stream: self.stream,
        }
    }
    /// Storage was refused: the minted record is discarded and the
    /// serialization waits at the same write.
    #[must_use]
    pub fn into_serialization(self) -> Serialization<'owner> {
        Serialization {
            stream: self.stream,
        }
    }
}
impl std::fmt::Debug for DigitReceipt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DigitReceipt")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}
/// One pending output byte.
pub struct Emit<'owner> {
    stream: Stream<'owner>,
    unit: u8,
}
impl<'owner> Emit<'owner> {
    /// The ASCII byte to append to the caller's output.
    #[must_use]
    pub const fn unit(&self) -> u8 {
        self.unit
    }
    /// Acknowledges that the caller appended [`unit`](Self::unit).
    #[must_use]
    pub fn emitted(mut self) -> Serialization<'owner> {
        self.stream.emit(self.unit);
        Serialization {
            stream: self.stream,
        }
    }
}
impl std::fmt::Debug for Emit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Emit")
            .field("unit", &self.unit)
            .finish_non_exhaustive()
    }
}
/// A completed serialization. It no longer borrows the numerical owner.
#[derive(Debug)]
pub struct SerializationReceipt {
    value: Color,
    length: usize,
}
impl SerializationReceipt {
    /// The serialized value.
    #[must_use]
    pub const fn value(&self) -> Color {
        self.value
    }
    /// Count of acknowledged output bytes.
    #[must_use]
    pub const fn length(&self) -> usize {
        self.length
    }
}
/// Serializes `value` through the same controlled continuation, with explicit
/// action, digit and output limits and fallible allocation.
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
    let mut serialization = Serialization::new(&mut owner, value, request);
    let mut digits: Vec<DigitSlot<'_>> = Vec::new();
    let mut output = Vec::new();
    let mut actions = 0usize;
    let receipt = loop {
        let action = match serialization.step() {
            SerializationStep::Complete(receipt) => break receipt,
            SerializationStep::Pending(action) => action,
        };
        actions = actions
            .checked_add(1)
            .ok_or(SerializationError::CountOverflow)?;
        admit(SerializationResource::Actions, actions, limits.actions)?;
        serialization = match action {
            SerializationAction::Compute(compute) => compute.compute(),
            SerializationAction::ReadDigit(read) => {
                let slot = &digits[read.index()];
                read.read(slot)
            }
            SerializationAction::AppendDigit(write) => {
                admit(
                    SerializationResource::Digits,
                    write
                        .index()
                        .checked_add(1)
                        .ok_or(SerializationError::CountOverflow)?,
                    limits.digits,
                )?;
                digits
                    .try_reserve(1)
                    .map_err(|_| SerializationError::Allocation {
                        resource: SerializationResource::Digits,
                    })?;
                let (record, receipt) = write.record();
                digits.push(record);
                receipt.stored(digits.last().expect("the record was just appended"))
            }
            SerializationAction::ReplaceDigit(write) => {
                let index = write.index();
                let (record, receipt) = write.record();
                digits[index] = record;
                receipt.stored(&digits[index])
            }
            SerializationAction::Emit(emit) => {
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
                output.push(emit.unit());
                emit.emitted()
            }
        };
    };
    debug_assert_eq!(receipt.length(), output.len());
    Ok(String::from_utf8(output).expect("color serialization emits ASCII"))
}
