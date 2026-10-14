//! Public specialized continuation over one canonical CSS color operation.
//!
//! The consumer owns input, all three real storage banks and its controller.
//! An action describes one finite observation/update. Nothing allocates,
//! releases capacity, executes abandoned work or publishes partial output here.

use super::operation as core;
use super::protocol::{CssColorBank, CssColorOperationOwner, CssColorProtocolError, CssColorSlot};
use super::{CssColorFactIdentity, CssColorFacts, CssColorKernel, CssUsedColor, CssUsedColorError};

/// Encoding of the original immutable input. UTF8 reads are bytes, not chars.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorInputKind {
    Utf8,
    Utf16,
}
impl CssColorInputKind {
    fn native(self) -> crate::tokenization::bounded::InputKind {
        match self {
            Self::Utf8 => crate::tokenization::bounded::InputKind::Utf8,
            Self::Utf16 => crate::tokenization::bounded::InputKind::Utf16,
        }
    }
}
/// Whether the actual parent requires one color or the entire declared input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorGrammarRequest {
    PrefixColor,
    WholeColor,
}
impl CssColorGrammarRequest {
    fn native(self) -> core::GrammarRequest {
        match self {
            Self::PrefixColor => core::GrammarRequest::PrefixColor,
            Self::WholeColor => core::GrammarRequest::WholeColor,
        }
    }
}
/// Original unit coordinates retained by the producer, never decoded offsets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorInputRange {
    pub start: usize,
    pub end: usize,
}
/// A reached unavailable fact differs from a standard-invalid color.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorUnavailable {
    CurrentColor,
    Palette(crate::CssSystemColor),
    Scheme,
    Profile,
    ProfileConversion,
    Device,
}
/// Reached capabilities not representable by the concrete fact carrier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorUnsupported {
    ProfileConversion,
    ReverseProfile,
    CustomInterpolation,
    ContextualUnit(crate::CssNumericUnit),
}
/// Malformed/ambiguous supplied facts never mint a standard-failure fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorFactError {
    Foreign,
    DuplicatePalette(crate::CssSystemColor),
    DuplicateProfile { first: usize, second: usize },
    InvalidProfileMetadata,
    DuplicateConversion,
    InvalidDevice,
}
/// Semantic or caller protocol failure. Resource/cancellation stays with the
/// original consumer's controller and is not reconstructed as this error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorOperationError {
    Protocol(CssColorProtocolError),
    Unavailable(CssColorUnavailable),
    Unsupported(CssColorUnsupported),
    Facts(CssColorFactError),
    Numerical(CssUsedColorError),
}
impl std::fmt::Display for CssColorOperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Protocol(error) => std::fmt::Display::fmt(error, f),
            Self::Unavailable(fact) => write!(f, "reached color fact is unavailable: {fact:?}"),
            Self::Unsupported(capability) => {
                write!(f, "reached color capability is unsupported: {capability:?}")
            }
            Self::Facts(error) => write!(f, "invalid color fact envelope: {error:?}"),
            Self::Numerical(error) => std::fmt::Display::fmt(error, f),
        }
    }
}
impl std::error::Error for CssColorOperationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Protocol(error) => Some(error),
            Self::Numerical(error) => Some(error),
            _ => None,
        }
    }
}
impl From<CssColorProtocolError> for CssColorOperationError {
    fn from(error: CssColorProtocolError) -> Self {
        Self::Protocol(error)
    }
}
impl From<core::Error> for CssColorOperationError {
    fn from(error: core::Error) -> Self {
        use super::{
            channels::Error as A,
            component_value::Error as V,
            device::DeviceError as D,
            interpolate::Error as I,
            palette::PaletteError as P,
            profile::{ComponentNameError as N, ProfileError as H},
            profile_conversion::Error as C,
            resolution::Error as R,
        };
        match error {
            core::Error::Protocol(error)
            | core::Error::Resolution(
                R::Protocol(error)
                | R::Component(V::Protocol(error))
                | R::Profile(H::Protocol(error))
                | R::ProfileComponent(N::Protocol(error))
                | R::Conversion(C::Protocol(error)),
            ) => Self::Protocol(error),
            core::Error::Resolution(error) => match error {
                R::Numerical(error)
                | R::Channel(A::Numerical(error))
                | R::Component(V::Channel(A::Numerical(error)))
                | R::Conversion(C::Numerical(error))
                | R::Device(D::Numerical(error))
                | R::Interpolation(I::Numerical(error)) => Self::Numerical(error),
                R::MissingScheme => Self::Unavailable(CssColorUnavailable::Scheme),
                R::ForeignFacts
                | R::Palette(P::ForeignFacts)
                | R::Profile(H::ForeignFacts)
                | R::ProfileComponent(N::ForeignFacts)
                | R::Conversion(C::ForeignFacts)
                | R::Device(D::ForeignFacts) => Self::Facts(CssColorFactError::Foreign),
                R::Palette(P::MissingCurrent) => {
                    Self::Unavailable(CssColorUnavailable::CurrentColor)
                }
                R::Palette(P::MissingPalette(keyword)) => {
                    Self::Unavailable(CssColorUnavailable::Palette(keyword))
                }
                R::Palette(P::Duplicate(keyword)) => {
                    Self::Facts(CssColorFactError::DuplicatePalette(keyword))
                }
                R::Profile(H::Missing) | R::ProfileComponent(N::Unavailable) => {
                    Self::Unavailable(CssColorUnavailable::Profile)
                }
                R::Profile(H::Duplicate { first, second }) => {
                    Self::Facts(CssColorFactError::DuplicateProfile { first, second })
                }
                R::Profile(H::InvalidMetadata)
                | R::ProfileComponent(N::InvalidMetadata)
                | R::Conversion(C::InvalidMetadata) => {
                    Self::Facts(CssColorFactError::InvalidProfileMetadata)
                }
                R::Conversion(C::Unavailable) => {
                    Self::Unavailable(CssColorUnavailable::ProfileConversion)
                }
                R::Conversion(C::Unsupported) => {
                    Self::Unsupported(CssColorUnsupported::ProfileConversion)
                }
                R::ReverseProfileUnsupported | R::Component(V::ReverseCustomOrigin { .. }) => {
                    Self::Unsupported(CssColorUnsupported::ReverseProfile)
                }
                R::Conversion(C::DuplicateConversion) => {
                    Self::Facts(CssColorFactError::DuplicateConversion)
                }
                R::Device(D::Unavailable) => Self::Unavailable(CssColorUnavailable::Device),
                R::Device(D::ConversionMismatch | D::InvalidFacts) => {
                    Self::Facts(CssColorFactError::InvalidDevice)
                }
                R::CustomInterpolationUnsupported => {
                    Self::Unsupported(CssColorUnsupported::CustomInterpolation)
                }
                R::Component(V::UnitBasis { unit, .. }) => {
                    Self::Unsupported(CssColorUnsupported::ContextualUnit(unit))
                }
                R::Interpolation(I::Overflow) => {
                    Self::Numerical(CssUsedColorError::Unrepresentable)
                }
                // These are refused dynamic call orders or standard-invalid
                // branches already converted by the canonical core. They cannot
                // manufacture a second fallback interpretation at this boundary.
                _ => Self::Protocol(CssColorProtocolError::WrongAction),
            },
        }
    }
}
/// One actual fixed fact observation, not a hidden array lookup/scan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorFactRead {
    Properties,
    PaletteLength,
    PaletteEntry { index: usize, earlier: bool },
    Scheme,
    ProfilesLength,
    ProfileHeader { binding: usize },
    ProfileNameByte { binding: usize, index: usize },
    SelectedProfileHeader,
    ComponentNameHeader { ordinal: usize },
    ComponentNameByte { ordinal: usize, index: usize },
    ConversionLength,
    ConversionHeader { ordinal: usize },
    ConversionChannel { conversion: usize, ordinal: usize },
    DeviceHeader,
    DeviceChannel { index: usize },
}
/// The precise action to admit before the guard's completion method.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorActionDescription {
    ReadInput { offset: usize },
    ReadSlot { bank: CssColorBank, index: usize },
    AppendSlot { bank: CssColorBank, index: usize },
    UpdateSlot { bank: CssColorBank, index: usize },
    ReadFact(CssColorFactRead),
    Compute(CssColorKernel),
    EmitUnit { unit: u8 },
    UsedReady,
    StandardFailed,
    Serialized,
    Stopped,
}
fn description(cursor: &core::ParseUsed<'_, '_>) -> CssColorActionDescription {
    use super::{
        device::DeviceAction as D,
        palette::PaletteAction as P,
        profile::{ComponentNameAction as N, ProfileAction as H},
        profile_conversion::{ConversionAction as C, InputAction as I},
        resolution::Action as R,
    };
    use core::{Action as A, StorageAction as S};
    if let Some(storage) = cursor.action().storage() {
        return match storage {
            S::Read { bank, index } => CssColorActionDescription::ReadSlot { bank, index },
            S::Append { bank, index } => CssColorActionDescription::AppendSlot { bank, index },
            S::Replace { bank, index } => CssColorActionDescription::UpdateSlot { bank, index },
        };
    }
    let fact = match cursor.action() {
        A::Lexical(super::protocol::ActionDescription::ReadInput { offset }) => {
            return CssColorActionDescription::ReadInput { offset };
        }
        A::UsedReady => return CssColorActionDescription::UsedReady,
        A::StandardFailed => return CssColorActionDescription::StandardFailed,
        A::Stopped => return CssColorActionDescription::Stopped,
        A::Resolution(R::ReadScheme) => Some(CssColorFactRead::Scheme),
        A::Resolution(R::Palette(P::ReadProperties)) => Some(CssColorFactRead::Properties),
        A::Resolution(R::Palette(P::ReadLength)) => Some(CssColorFactRead::PaletteLength),
        A::Resolution(R::Palette(P::ReadEntry { index })) => Some(CssColorFactRead::PaletteEntry {
            index,
            earlier: false,
        }),
        A::Resolution(R::Palette(P::CompareEarlier { index })) => {
            Some(CssColorFactRead::PaletteEntry {
                index,
                earlier: true,
            })
        }
        A::Resolution(R::Profile(H::ReadLength)) => Some(CssColorFactRead::ProfilesLength),
        A::Resolution(R::Profile(
            H::ReadHeader { binding } | H::ReadSelectedHeader { binding },
        )) => Some(CssColorFactRead::ProfileHeader { binding }),
        A::Resolution(R::Profile(H::ReadFactByte { binding, index })) => {
            Some(CssColorFactRead::ProfileNameByte { binding, index })
        }
        A::Resolution(R::ProfileName(N::ReadProfileHeader) | R::ProfileInput(I::ReadHeader)) => {
            Some(CssColorFactRead::SelectedProfileHeader)
        }
        A::Resolution(R::ProfileName(N::ReadNameHeader { ordinal })) => {
            Some(CssColorFactRead::ComponentNameHeader { ordinal })
        }
        A::Resolution(R::ProfileName(N::ReadNameByte { ordinal, index })) => {
            Some(CssColorFactRead::ComponentNameByte { ordinal, index })
        }
        A::Resolution(R::Conversion(C::ReadLength)) => Some(CssColorFactRead::ConversionLength),
        A::Resolution(R::Conversion(C::ReadHeader { ordinal })) => {
            Some(CssColorFactRead::ConversionHeader { ordinal })
        }
        A::Resolution(R::Conversion(C::ReadFactChannel {
            conversion,
            ordinal,
        })) => Some(CssColorFactRead::ConversionChannel {
            conversion,
            ordinal,
        }),
        A::Resolution(R::Device(D::ReadHeader)) => Some(CssColorFactRead::DeviceHeader),
        A::Resolution(R::Device(D::ReadChannel { index })) => {
            Some(CssColorFactRead::DeviceChannel { index })
        }
        _ => None,
    };
    fact.or_else(|| cursor.additional_fact_read()).map_or_else(
        || CssColorActionDescription::Compute(cursor.kernel()),
        CssColorActionDescription::ReadFact,
    )
}
/// Actual fixed records/units which a fact action can inspect. Zero entries
/// are absent observations. These are compiled record layouts, not lengths of
/// referenced names, channels or arrays; those units have separate actions.
impl CssColorFactRead {
    #[must_use]
    pub fn record_layouts(self) -> [usize; 3] {
        use std::mem::size_of;
        let identity = size_of::<CssColorFactIdentity>();
        let profile = size_of::<super::CssColorProfileBinding<'_>>();
        let conversion = size_of::<super::CssColorProfileConversion<'_>>();
        match self {
            Self::Properties => [identity, size_of::<super::CssColorPropertyFacts>(), 0],
            Self::Scheme => [identity, size_of::<super::CssColorSchemeFact>(), 0],
            Self::PaletteLength | Self::ProfilesLength => [identity, size_of::<usize>(), 0],
            Self::PaletteEntry { .. } => [identity, size_of::<super::CssColorPaletteEntry>(), 0],
            Self::ProfileHeader { .. } | Self::SelectedProfileHeader => [identity, profile, 0],
            Self::ProfileNameByte { .. } => [identity, profile, size_of::<u8>()],
            Self::ComponentNameHeader { .. } => [
                identity,
                profile,
                size_of::<crate::CssColorProfileComponentName>(),
            ],
            Self::ComponentNameByte { .. } => [
                identity,
                profile,
                size_of::<crate::CssColorProfileComponentName>() + size_of::<u8>(),
            ],
            Self::ConversionLength => [identity, profile, size_of::<usize>()],
            Self::ConversionHeader { .. } => [identity, conversion, 0],
            Self::ConversionChannel { .. } => [identity, conversion, size_of::<f64>()],
            Self::DeviceHeader => [identity, size_of::<super::CssDeviceColorFact<'_>>(), 0],
            Self::DeviceChannel { .. } => [
                identity,
                size_of::<super::CssDeviceColorFact<'_>>(),
                size_of::<f64>(),
            ],
        }
    }
}
/// Payload observations/writes for one finite action. The consumer additionally
/// admits the cached planning/header work and its own actual vector/output
/// capacity/copy events. CSS never converts these layouts into guessed weights.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorActionLayout {
    pub input_unit_bytes: usize,
    pub slot_read_bytes: usize,
    pub slot_write_bytes: usize,
    pub fact_records: [usize; 3],
    pub output_unit_bytes: usize,
    pub kernel: CssColorKernel,
}
fn layout(
    description: CssColorActionDescription,
    kind: CssColorInputKind,
    kernel: CssColorKernel,
) -> CssColorActionLayout {
    use std::mem::size_of;
    let mut layout = CssColorActionLayout {
        input_unit_bytes: 0,
        slot_read_bytes: 0,
        slot_write_bytes: 0,
        fact_records: [0; 3],
        output_unit_bytes: 0,
        kernel,
    };
    match description {
        CssColorActionDescription::ReadInput { .. } => {
            layout.input_unit_bytes = match kind {
                CssColorInputKind::Utf8 => size_of::<u8>(),
                CssColorInputKind::Utf16 => size_of::<u16>(),
            }
        }
        CssColorActionDescription::ReadSlot { .. } => {
            layout.slot_read_bytes = size_of::<CssColorSlot<'_>>()
        }
        CssColorActionDescription::AppendSlot { .. }
        | CssColorActionDescription::UpdateSlot { .. } => {
            // Generated opaque payload plus actual stored acknowledgement.
            layout.slot_write_bytes = size_of::<CssColorSlot<'_>>();
            layout.slot_read_bytes = size_of::<CssColorSlot<'_>>();
        }
        CssColorActionDescription::ReadFact(fact) => layout.fact_records = fact.record_layouts(),
        CssColorActionDescription::EmitUnit { .. } => layout.output_unit_bytes = size_of::<u8>(),
        _ => {}
    }
    layout
}
/// Actual fixed initialization observations and continuation layout. The
/// original consumer owns admission and any actual backing allocation/copy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorInitializationLayout {
    pub owner_bytes: usize,
    pub register_bytes: usize,
    pub fact_identity_bytes: usize,
}
/// Fixed continuation. It never takes the original input or caller banks.
pub struct CssColorOperation<'owner, 'facts> {
    cursor: core::ParseUsed<'owner, 'facts>,
}
impl<'owner, 'facts> CssColorOperation<'owner, 'facts> {
    /// Fixed metadata admission precedes this call; no array/input is scanned.
    pub fn parse_used(
        owner: &'owner mut CssColorOperationOwner,
        kind: CssColorInputKind,
        length: usize,
        request: CssColorGrammarRequest,
        facts: &'facts CssColorFacts<'facts>,
        expected: CssColorFactIdentity,
    ) -> Result<Self, CssColorOperationError> {
        if facts.identity != expected {
            return Err(CssColorOperationError::Facts(CssColorFactError::Foreign));
        }
        Ok(Self {
            cursor: core::ParseUsed::start(
                owner,
                kind.native(),
                length,
                request.native(),
                facts,
                expected,
            )?,
        })
    }
    /// Fixed owner/header reads for initialization, separate from the
    /// continuation's own compiled register allocation. No array is traversed.
    #[must_use]
    pub const fn initialization_layout() -> CssColorInitializationLayout {
        CssColorInitializationLayout {
            owner_bytes: std::mem::size_of::<CssColorOperationOwner>(),
            register_bytes: Self::register_bytes(),
            fact_identity_bytes: std::mem::size_of::<CssColorFactIdentity>(),
        }
    }
    /// Fixed cached planning only. The adapter admits this header observation.
    #[must_use]
    pub fn action(&self) -> CssColorActionDescription {
        description(&self.cursor)
    }
    /// Actual compiled fixed continuation layout, admitted before initialization.
    #[must_use]
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    /// Actual compiled outstanding guard layout, admitted before next_action.
    #[must_use]
    pub const fn action_bytes() -> usize {
        std::mem::size_of::<CssColorAction<'_, '_, '_>>()
    }
    /// The exact acknowledged logical extent; capacity remains caller-owned.
    #[must_use]
    pub const fn bank_length(&self, bank: CssColorBank) -> usize {
        self.cursor.bank_length(bank)
    }
    /// Exclusive finite action; dropping it keeps the actual request pending.
    pub fn next_action(&mut self) -> CssColorAction<'_, 'owner, 'facts> {
        let description = self.action();
        CssColorAction {
            operation: self,
            description,
            issued: false,
        }
    }
    /// Abandon never executes pending work or refunds/releases caller storage.
    pub fn abandon(&mut self) {
        self.cursor.abandon();
    }
    /// Extract only a complete fixed checkpoint; no scratch index escapes.
    pub fn take(self) -> Result<CssColorParseOutcome<'owner, 'facts>, CssColorProtocolError> {
        Ok(match self.cursor.take()? {
            core::ParseOutcome::Used(checkpoint) => {
                CssColorParseOutcome::Used(CssUsedColorCheckpoint { checkpoint })
            }
            core::ParseOutcome::StandardFailure(checkpoint) => {
                CssColorParseOutcome::StandardFailure(CssColorFailureCheckpoint { checkpoint })
            }
        })
    }
}
/// Only one action can borrow a color operation at a time.
pub struct CssColorAction<'step, 'owner, 'facts> {
    operation: &'step mut CssColorOperation<'owner, 'facts>,
    description: CssColorActionDescription,
    issued: bool,
}
impl<'owner> CssColorAction<'_, 'owner, '_> {
    #[must_use]
    pub const fn description(&self) -> CssColorActionDescription {
        self.description
    }
    #[must_use]
    pub fn layout(&self) -> CssColorActionLayout {
        layout(
            self.description,
            match self.operation.cursor.input_kind() {
                crate::tokenization::bounded::InputKind::Utf8 => CssColorInputKind::Utf8,
                crate::tokenization::bounded::InputKind::Utf16 => CssColorInputKind::Utf16,
            },
            self.operation.cursor.kernel(),
        )
    }
    pub fn read_input(self, offset: usize, unit: u16) -> Result<(), CssColorOperationError> {
        if self.description != (CssColorActionDescription::ReadInput { offset }) {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        self.operation
            .cursor
            .read_input(offset, unit)
            .map_err(Into::into)
    }
    pub fn read_slot(
        self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorOperationError> {
        if self.description != (CssColorActionDescription::ReadSlot { bank, index }) {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        self.operation
            .cursor
            .read_slot(bank, index, slot)
            .map_err(Into::into)
    }
    /// Mint one fixed pending payload. Generate only after the write/reserve
    /// admission. Advancing still requires acknowledgement of actual storage.
    pub fn record(&mut self) -> Result<CssColorSlot<'owner>, CssColorOperationError> {
        if self.issued {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        let (bank, index) = match self.description {
            CssColorActionDescription::AppendSlot { bank, index }
            | CssColorActionDescription::UpdateSlot { bank, index } => (bank, index),
            _ => return Err(CssColorProtocolError::WrongAction.into()),
        };
        let record = self.operation.cursor.pending_record(bank, index)?;
        self.issued = true;
        Ok(record)
    }
    pub fn written(
        self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorOperationError> {
        if !self.issued
            || !matches!(self.description,CssColorActionDescription::AppendSlot {bank: b,index:i}|CssColorActionDescription::UpdateSlot {bank:b,index:i} if bank==b&&index==i)
        {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        self.operation
            .cursor
            .written(bank, index, slot)
            .map_err(Into::into)
    }
    pub fn compute(self) -> Result<(), CssColorOperationError> {
        if !matches!(
            self.description,
            CssColorActionDescription::Compute(_) | CssColorActionDescription::ReadFact(_)
        ) {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        self.operation.cursor.compute().map_err(Into::into)
    }
}
/// Standard failure is the only parse result that admits an explicit fallback.
pub enum CssColorParseOutcome<'owner, 'facts> {
    Used(CssUsedColorCheckpoint<'owner, 'facts>),
    StandardFailure(CssColorFailureCheckpoint<'owner, 'facts>),
}
/// Fixed used value plus original source/fact qualification, no recipe backing.
pub struct CssUsedColorCheckpoint<'owner, 'facts> {
    checkpoint: core::UsedCheckpoint<'owner, 'facts>,
}
/// Actual standard-invalid branch, distinct from reached capability failure.
pub struct CssColorFailureCheckpoint<'owner, 'facts> {
    checkpoint: core::FailureCheckpoint<'owner, 'facts>,
}
/// The canonical reason and original input range for standard failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorStandardFailureKind {
    ColorGrammar,
    ComponentGrammar,
    NumericGrammar,
    InvalidProfile,
    InvalidComponents,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorStandardFailure {
    pub kind: CssColorStandardFailureKind,
    pub source: CssColorInputRange,
}
fn failure(value: core::StandardFailure) -> CssColorStandardFailure {
    use core::StandardReason as R;
    CssColorStandardFailure {
        kind: match value.reason {
            R::ColorGrammar => CssColorStandardFailureKind::ColorGrammar,
            R::ComponentGrammar => CssColorStandardFailureKind::ComponentGrammar,
            R::NumericGrammar => CssColorStandardFailureKind::NumericGrammar,
            R::InvalidProfile => CssColorStandardFailureKind::InvalidProfile,
            R::InvalidComponents => CssColorStandardFailureKind::InvalidComponents,
        },
        source: CssColorInputRange {
            start: value.source.start,
            end: value.source.end,
        },
    }
}
impl<'owner, 'facts> CssUsedColorCheckpoint<'owner, 'facts> {
    #[must_use]
    pub const fn value(&self) -> CssUsedColor {
        self.checkpoint.value()
    }
    #[must_use]
    pub fn consumed(&self) -> CssColorInputRange {
        let range = self.checkpoint.context().consumed;
        CssColorInputRange {
            start: range.start,
            end: range.end,
        }
    }
    #[must_use]
    pub const fn fact_identity(&self) -> CssColorFactIdentity {
        self.checkpoint.context().expected
    }
    /// Lends the actual immutable envelope retained at the used checkpoint.
    /// The consumer admits any later header/unit observations; this getter
    /// neither scans arrays nor resolves additional context.
    #[must_use]
    pub const fn facts(&self) -> &'facts CssColorFacts<'facts> {
        self.checkpoint.context().facts
    }
    #[must_use]
    pub fn standard_failure(&self) -> Option<CssColorStandardFailure> {
        self.checkpoint.standard_failure().map(failure)
    }
}
impl<'owner, 'facts> CssColorFailureCheckpoint<'owner, 'facts> {
    #[must_use]
    pub fn failure(&self) -> CssColorStandardFailure {
        failure(self.checkpoint.failure())
    }
    /// Consumer policy supplies the already checked value; no unavailable,
    /// numerical, protocol or resource failure has this transition.
    pub fn fallback(self, value: CssUsedColor) -> CssUsedColorCheckpoint<'owner, 'facts> {
        CssUsedColorCheckpoint {
            checkpoint: self.checkpoint.fallback(value),
        }
    }
}

/// Observed logical bank lengths at a later phase boundary. These fixed checks
/// supplement, and cannot prove, the consumer controller's real ticket disposal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorBankLengths {
    algorithm: usize,
    arithmetic: usize,
    evaluation: usize,
}
impl CssColorBankLengths {
    #[must_use]
    pub const fn new(algorithm: usize, arithmetic: usize, evaluation: usize) -> Self {
        Self {
            algorithm,
            arithmetic,
            evaluation,
        }
    }
    fn empty(self) -> Result<(), CssColorProtocolError> {
        if self.algorithm == 0 && self.arithmetic == 0 && self.evaluation == 0 {
            Ok(())
        } else {
            Err(CssColorProtocolError::StorageNotEmpty)
        }
    }
}
/// The fixed original input metadata. It never proves an HTML/DOM issuer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorSourceDescriptor {
    pub kind: CssColorInputKind,
    pub length: usize,
    pub request: CssColorGrammarRequest,
    pub consumed: CssColorInputRange,
    pub facts: CssColorFactIdentity,
}
fn source(context: &core::CheckpointContext<'_, '_>) -> CssColorSourceDescriptor {
    CssColorSourceDescriptor {
        kind: match context.input_kind {
            crate::tokenization::bounded::InputKind::Utf8 => CssColorInputKind::Utf8,
            crate::tokenization::bounded::InputKind::Utf16 => CssColorInputKind::Utf16,
        },
        length: context.input_length,
        request: match context.request {
            core::GrammarRequest::PrefixColor => CssColorGrammarRequest::PrefixColor,
            core::GrammarRequest::WholeColor => CssColorGrammarRequest::WholeColor,
        },
        consumed: CssColorInputRange {
            start: context.consumed.start,
            end: context.consumed.end,
        },
        facts: context.expected,
    }
}
fn transformation_error(error: core::TransformError) -> CssColorOperationError {
    match error {
        core::TransformError::Protocol(error) => error.into(),
        core::TransformError::Numerical(error) => CssColorOperationError::Numerical(error),
    }
}
/// One fixed transformation under an advanced checked phase, with no output.
pub struct CssColorTransformation<'owner, 'facts> {
    cursor: core::Transformation<'owner, 'facts>,
}
impl<'owner, 'facts> CssColorTransformation<'owner, 'facts> {
    #[must_use]
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    #[must_use]
    pub const fn action_bytes() -> usize {
        std::mem::size_of::<CssColorTransformAction<'_, '_, '_>>()
    }
    #[must_use]
    pub fn action(&self) -> CssColorActionDescription {
        match self.cursor.action() {
            core::TransformationAction::Compute(transformation) => {
                CssColorActionDescription::Compute(CssColorKernel::Transform(transformation))
            }
            core::TransformationAction::Done => CssColorActionDescription::UsedReady,
            core::TransformationAction::Stopped => CssColorActionDescription::Stopped,
        }
    }
    pub fn next_action(&mut self) -> CssColorTransformAction<'_, 'owner, 'facts> {
        CssColorTransformAction { operation: self }
    }
    pub fn abandon(&mut self) {
        self.cursor.abandon();
    }
    /// Refusal leaves the actual transformation in the caller's slot.
    pub fn take(
        slot: &mut Option<Self>,
    ) -> Result<CssUsedColorCheckpoint<'owner, 'facts>, CssColorOperationError> {
        slot.as_ref()
            .ok_or(CssColorProtocolError::EmptySlot)?
            .cursor
            .check_completed()
            .map_err(transformation_error)?;
        let Some(transformation) = slot.take() else {
            unreachable!("slot preflight proved Some")
        };
        Ok(CssUsedColorCheckpoint {
            checkpoint: transformation.cursor.into_completed(),
        })
    }
    /// Recover the latest committed value and its original context. A numerical
    /// refusal preserves the input to that transformation. This does not restart
    /// a stopped consumer controller.
    pub fn recover(self) -> CssUsedColorCheckpoint<'owner, 'facts> {
        CssUsedColorCheckpoint {
            checkpoint: self.cursor.recover(),
        }
    }
}
/// Exclusive transformation action. Dropping it leaves the computation pending.
pub struct CssColorTransformAction<'step, 'owner, 'facts> {
    operation: &'step mut CssColorTransformation<'owner, 'facts>,
}
impl CssColorTransformAction<'_, '_, '_> {
    #[must_use]
    pub fn layout(&self) -> CssColorActionLayout {
        let description = self.description();
        layout(
            description,
            CssColorInputKind::Utf8,
            match description {
                CssColorActionDescription::Compute(kernel) => kernel,
                _ => CssColorKernel::Transition,
            },
        )
    }
    #[must_use]
    pub fn description(&self) -> CssColorActionDescription {
        self.operation.action()
    }
    pub fn compute(self) -> Result<(), CssColorOperationError> {
        self.operation
            .cursor
            .compute()
            .map_err(transformation_error)
    }
}
impl<'owner, 'facts> CssUsedColorCheckpoint<'owner, 'facts> {
    #[must_use]
    pub fn source(&self) -> CssColorSourceDescriptor {
        source(self.checkpoint.context())
    }
    /// Retired banks must have been disposed through the actual original owner
    /// before this fixed header check. CSS itself owns no allocation tickets.
    pub fn transform(
        slot: &mut Option<Self>,
        transformation: super::CssUsedColorTransformation,
        banks: CssColorBankLengths,
    ) -> Result<CssColorTransformation<'owner, 'facts>, CssColorOperationError> {
        let checkpoint = slot.as_ref().ok_or(CssColorProtocolError::EmptySlot)?;
        banks.empty()?;
        let phase = checkpoint
            .checkpoint
            .next_phase()
            .map_err(transformation_error)?;
        let Some(checkpoint) = slot.take() else {
            unreachable!("slot preflight proved Some")
        };
        Ok(CssColorTransformation {
            cursor: checkpoint
                .checkpoint
                .transform_committed(transformation, phase),
        })
    }
    /// Begin only final selected serialization. Refusal preserves the actual
    /// caller slot; success consumes once after all fallible preflight.
    pub fn serialize(
        slot: &mut Option<Self>,
        request: super::CssColorSerializationRequest,
        banks: CssColorBankLengths,
    ) -> Result<CssColorSerialization<'owner, 'facts>, CssColorOperationError> {
        let checkpoint = slot.as_ref().ok_or(CssColorProtocolError::EmptySlot)?;
        banks.empty()?;
        let phase = checkpoint
            .checkpoint
            .next_phase()
            .map_err(transformation_error)?;
        let Some(checkpoint) = slot.take() else {
            unreachable!("slot preflight proved Some")
        };
        Ok(CssColorSerialization {
            inner: SerializationInner::Parsed(
                checkpoint.checkpoint.serialize_committed(request, phase),
            ),
        })
    }
}
enum SerializationInner<'owner, 'facts> {
    Parsed(core::Serialization<'owner, 'facts>),
    Native {
        cursor: super::serialize::UsedSerializer<'owner>,
        value: CssUsedColor,
    },
}
/// Canonical finite serialization from either a parsed checkpoint or a checked
/// native value. Both entries use the same CSS-owned digit/format continuation.
pub struct CssColorSerialization<'owner, 'facts> {
    inner: SerializationInner<'owner, 'facts>,
}
impl<'owner, 'facts> CssColorSerialization<'owner, 'facts> {
    #[must_use]
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    #[must_use]
    pub const fn action_bytes() -> usize {
        std::mem::size_of::<CssColorSerializationAction<'_, '_, '_>>()
    }
    /// A native value carries no invented parsed source or resolved fact proof.
    pub fn from_used(
        owner: &'owner mut CssColorOperationOwner,
        value: CssUsedColor,
        request: super::CssColorSerializationRequest,
        banks: CssColorBankLengths,
    ) -> Result<Self, CssColorProtocolError> {
        banks.empty()?;
        Ok(Self {
            inner: SerializationInner::Native {
                cursor: super::serialize::UsedSerializer::start(owner, value, request)?,
                value,
            },
        })
    }
    fn native_action(&self) -> super::serialize::SerializationAction {
        match &self.inner {
            SerializationInner::Parsed(cursor) => cursor.action(),
            SerializationInner::Native { cursor, .. } => cursor.action(),
        }
    }
    #[must_use]
    pub fn action(&self) -> CssColorActionDescription {
        use super::serialize::SerializationAction as A;
        match self.native_action() {
            A::Plan | A::Compute => CssColorActionDescription::Compute(match &self.inner {
                SerializationInner::Parsed(cursor) => cursor.kernel(),
                SerializationInner::Native { cursor, .. } => cursor.kernel(),
            }),
            A::ReadDigit { index } => CssColorActionDescription::ReadSlot {
                bank: CssColorBank::Algorithm,
                index,
            },
            A::AppendDigit { index } => CssColorActionDescription::AppendSlot {
                bank: CssColorBank::Algorithm,
                index,
            },
            A::ReplaceDigit { index } => CssColorActionDescription::UpdateSlot {
                bank: CssColorBank::Algorithm,
                index,
            },
            A::Emit { unit } => CssColorActionDescription::EmitUnit { unit },
            A::Done => CssColorActionDescription::Serialized,
            A::Stopped => CssColorActionDescription::Stopped,
        }
    }
    pub fn next_action(&mut self) -> CssColorSerializationAction<'_, 'owner, 'facts> {
        let description = self.action();
        CssColorSerializationAction {
            operation: self,
            description,
            issued: false,
        }
    }
    pub fn abandon(&mut self) {
        match &mut self.inner {
            SerializationInner::Parsed(cursor) => cursor.abandon(),
            SerializationInner::Native { cursor, .. } => cursor.abandon(),
        }
    }
    pub fn take(
        self,
    ) -> Result<CssColorSerializationReceipt<'owner, 'facts>, CssColorProtocolError> {
        Ok(CssColorSerializationReceipt {
            inner: match self.inner {
                SerializationInner::Parsed(cursor) => ReceiptInner::Parsed(cursor.take()?),
                SerializationInner::Native { cursor, value } => ReceiptInner::Native {
                    value,
                    length: cursor.completed_length()?,
                },
            },
        })
    }
}
/// The original output buffer remains caller-owned throughout this action.
pub struct CssColorSerializationAction<'step, 'owner, 'facts> {
    operation: &'step mut CssColorSerialization<'owner, 'facts>,
    description: CssColorActionDescription,
    issued: bool,
}
fn serialization_error(error: super::serialize::SerializationError) -> CssColorOperationError {
    match error {
        super::serialize::SerializationError::Protocol(error) => error.into(),
        super::serialize::SerializationError::Numerical(error) => {
            CssColorOperationError::Numerical(error)
        }
    }
}
impl<'owner> CssColorSerializationAction<'_, 'owner, '_> {
    #[must_use]
    pub const fn description(&self) -> CssColorActionDescription {
        self.description
    }
    #[must_use]
    pub fn layout(&self) -> CssColorActionLayout {
        layout(
            self.description,
            CssColorInputKind::Utf8,
            match self.description {
                CssColorActionDescription::Compute(kernel) => kernel,
                _ => CssColorKernel::Transition,
            },
        )
    }
    pub fn compute(self) -> Result<(), CssColorOperationError> {
        if !matches!(self.description, CssColorActionDescription::Compute(_)) {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        match &mut self.operation.inner {
            SerializationInner::Parsed(cursor) => cursor.compute(),
            SerializationInner::Native { cursor, .. } => cursor.compute(),
        }
        .map_err(serialization_error)
    }
    pub fn read_slot(
        self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        if self.description != (CssColorActionDescription::ReadSlot { bank, index }) {
            return Err(CssColorProtocolError::WrongAction);
        }
        match &mut self.operation.inner {
            SerializationInner::Parsed(cursor) => cursor.read_digit(index, slot),
            SerializationInner::Native { cursor, .. } => cursor.read_digit(index, slot),
        }
    }
    pub fn record(&mut self) -> Result<CssColorSlot<'owner>, CssColorProtocolError> {
        if self.issued {
            return Err(CssColorProtocolError::WrongAction);
        }
        let index = match self.description {
            CssColorActionDescription::AppendSlot {
                bank: CssColorBank::Algorithm,
                index,
            }
            | CssColorActionDescription::UpdateSlot {
                bank: CssColorBank::Algorithm,
                index,
            } => index,
            _ => return Err(CssColorProtocolError::WrongAction),
        };
        let slot = match &self.operation.inner {
            SerializationInner::Parsed(cursor) => cursor.pending_record(index),
            SerializationInner::Native { cursor, .. } => cursor.digit_record(index),
        }?;
        self.issued = true;
        Ok(slot)
    }
    pub fn written(
        self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        if !self.issued
            || !matches!(self.description,CssColorActionDescription::AppendSlot {bank:b,index:i}|CssColorActionDescription::UpdateSlot {bank:b,index:i} if bank==b&&index==i)
        {
            return Err(CssColorProtocolError::WrongAction);
        }
        match &mut self.operation.inner {
            SerializationInner::Parsed(cursor) => cursor.written(index, slot),
            SerializationInner::Native { cursor, .. } => cursor.digit_written(index, slot),
        }
    }
    pub fn emitted(self, unit: u8) -> Result<(), CssColorProtocolError> {
        if self.description != (CssColorActionDescription::EmitUnit { unit }) {
            return Err(CssColorProtocolError::WrongAction);
        }
        match &mut self.operation.inner {
            SerializationInner::Parsed(cursor) => cursor.emitted(unit),
            SerializationInner::Native { cursor, .. } => cursor.emitted(unit),
        }
    }
}
enum ReceiptInner<'owner, 'facts> {
    Parsed(core::SerializationReceipt<'owner, 'facts>),
    Native { value: CssUsedColor, length: usize },
}
/// Fixed success receipt. The consumer still owns its final buffer and commit.
pub struct CssColorSerializationReceipt<'owner, 'facts> {
    inner: ReceiptInner<'owner, 'facts>,
}
impl CssColorSerializationReceipt<'_, '_> {
    #[must_use]
    pub fn value(&self) -> CssUsedColor {
        match &self.inner {
            ReceiptInner::Parsed(receipt) => receipt.value,
            ReceiptInner::Native { value, .. } => *value,
        }
    }
    #[must_use]
    pub fn length(&self) -> usize {
        match &self.inner {
            ReceiptInner::Parsed(receipt) => receipt.output_length,
            ReceiptInner::Native { length, .. } => *length,
        }
    }
    /// None is the native entry, not missing parsed-source metadata.
    #[must_use]
    pub fn source(&self) -> Option<CssColorSourceDescriptor> {
        match &self.inner {
            ReceiptInner::Parsed(receipt) => Some(source(&receipt.context)),
            ReceiptInner::Native { .. } => None,
        }
    }
    #[must_use]
    pub fn standard_failure(&self) -> Option<CssColorStandardFailure> {
        match &self.inner {
            ReceiptInner::Parsed(receipt) => receipt.standard_failure.map(failure),
            ReceiptInner::Native { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        CssColorPropertyFacts, CssColorSchemeFact, CssColorSerializationRequest,
        CssDeviceColorFact, CssUsedColorTransformation,
    };
    use super::*;
    fn facts() -> CssColorFacts<'static> {
        CssColorFacts::new(
            CssColorFactIdentity::new(10, 20),
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &[],
            &[],
            CssDeviceColorFact::Unknown,
        )
    }
    fn bank(bank: CssColorBank) -> usize {
        match bank {
            CssColorBank::Algorithm => 0,
            CssColorBank::Arithmetic => 1,
            CssColorBank::Evaluation => 2,
        }
    }
    fn parse<'owner, 'facts>(
        owner: &'owner mut CssColorOperationOwner,
        input: &str,
        facts: &'facts CssColorFacts<'facts>,
    ) -> Result<(CssColorParseOutcome<'owner, 'facts>, usize), CssColorOperationError> {
        let mut operation = CssColorOperation::parse_used(
            owner,
            CssColorInputKind::Utf8,
            input.len(),
            CssColorGrammarRequest::WholeColor,
            facts,
            facts.identity,
        )?;
        let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
        let mut folds = 0;
        loop {
            let mut action = operation.next_action();
            let actual = action.layout();
            match action.description() {
                CssColorActionDescription::ReadInput { .. } => {
                    assert_eq!(actual.input_unit_bytes, std::mem::size_of::<u8>())
                }
                CssColorActionDescription::ReadSlot { .. } => assert_eq!(
                    actual.slot_read_bytes,
                    std::mem::size_of::<CssColorSlot<'_>>()
                ),
                CssColorActionDescription::AppendSlot { .. }
                | CssColorActionDescription::UpdateSlot { .. } => {
                    assert_eq!(
                        actual.slot_read_bytes,
                        std::mem::size_of::<CssColorSlot<'_>>()
                    );
                    assert_eq!(
                        actual.slot_write_bytes,
                        std::mem::size_of::<CssColorSlot<'_>>()
                    );
                }
                CssColorActionDescription::ReadFact(fact) => {
                    assert_eq!(actual.fact_records, fact.record_layouts())
                }
                CssColorActionDescription::Compute(kernel) => assert_eq!(actual.kernel, kernel),
                _ => {}
            }
            match action.description() {
                CssColorActionDescription::ReadInput { offset } => {
                    action.read_input(offset, u16::from(input.as_bytes()[offset]))?
                }
                CssColorActionDescription::ReadSlot { bank: which, index } => {
                    action.read_slot(which, index, &banks[bank(which)][index])?
                }
                CssColorActionDescription::AppendSlot { bank: which, index } => {
                    assert_eq!(index, banks[bank(which)].len());
                    let record = action.record()?;
                    banks[bank(which)].push(record);
                    action.written(which, index, &banks[bank(which)][index])?;
                }
                CssColorActionDescription::UpdateSlot { bank: which, index } => {
                    banks[bank(which)][index] = action.record()?;
                    action.written(which, index, &banks[bank(which)][index])?;
                }
                CssColorActionDescription::Compute(kernel) => {
                    folds += kernel.variadic_fold_count();
                    action.compute()?;
                }
                CssColorActionDescription::ReadFact(_) => action.compute()?,
                CssColorActionDescription::UsedReady
                | CssColorActionDescription::StandardFailed => {
                    break;
                }
                other => panic!("unexpected parse action {other:?}"),
            }
        }
        let outcome = operation.take()?;
        // This ordinary owner clears its vectors. It does not simulate real
        // HTML capacity/ticket cleanup or claim foreign-controller admission.
        banks.iter_mut().for_each(Vec::clear);
        drop(banks);
        Ok((outcome, folds))
    }
    fn serialize(
        mut serializer: CssColorSerialization<'_, '_>,
    ) -> (
        String,
        CssUsedColor,
        Option<CssColorSourceDescriptor>,
        Option<CssColorStandardFailure>,
    ) {
        let mut digits = Vec::new();
        let mut output = Vec::new();
        loop {
            let mut action = serializer.next_action();
            match action.description() {
                CssColorActionDescription::Compute(_) => action.compute().unwrap(),
                CssColorActionDescription::ReadSlot { bank, index } => {
                    action.read_slot(bank, index, &digits[index]).unwrap()
                }
                CssColorActionDescription::AppendSlot { bank, index } => {
                    assert_eq!(bank, CssColorBank::Algorithm);
                    assert_eq!(index, digits.len());
                    digits.push(action.record().unwrap());
                    action.written(bank, index, &digits[index]).unwrap();
                }
                CssColorActionDescription::UpdateSlot { bank, index } => {
                    digits[index] = action.record().unwrap();
                    action.written(bank, index, &digits[index]).unwrap();
                }
                CssColorActionDescription::EmitUnit { unit } => {
                    output.push(unit);
                    action.emitted(unit).unwrap();
                }
                CssColorActionDescription::Serialized => {
                    break;
                }
                other => panic!("unexpected serialization action {other:?}"),
            }
        }
        let receipt = serializer.take().unwrap();
        digits.clear();
        drop(digits);
        (
            String::from_utf8(output).unwrap(),
            receipt.value(),
            receipt.source(),
            receipt.standard_failure(),
        )
    }
    #[test]
    fn public_guard_drives_actual_banks_and_exposes_three_folds_per_argument() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let (outcome, folds) = parse(&mut owner, "rgb(calc(min(3, 2, 1)) 0 0)", &facts).unwrap();
        let CssColorParseOutcome::Used(checkpoint) = outcome else {
            panic!("valid numerical color")
        };
        assert_eq!(
            checkpoint.value().channels(),
            [Some(1.0 / 255.0), Some(0.0), Some(0.0)]
        );
        // Three min operands plus the enclosing calc operand each update all
        // three defining cached aggregates; finishing visits no prior prefix.
        assert_eq!(folds, 12);
        assert!(std::ptr::eq(checkpoint.facts(), &facts));
        let source = checkpoint.source();
        let value = checkpoint.value();
        let serializer = CssUsedColorCheckpoint::serialize(
            &mut Some(checkpoint),
            CssColorSerializationRequest::Css,
            CssColorBankLengths::new(0, 0, 0),
        )
        .ok()
        .unwrap();
        assert_eq!(
            serialize(serializer),
            ("rgb(1, 0, 0)".to_owned(), value, Some(source), None)
        );
    }
    #[test]
    fn public_checkpoint_rejects_live_banks_without_losing_value_and_retains_fallback_reason() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let (outcome, _) = parse(&mut owner, "rgb(1deg 0 0)", &facts).unwrap();
        let CssColorParseOutcome::StandardFailure(failed) = outcome else {
            panic!("degree is not an RGB scalar")
        };
        let expected = failed.failure();
        let checkpoint = failed.fallback(CssUsedColor::opaque_black());
        let mut slot = Some(checkpoint);
        assert_eq!(
            CssUsedColorCheckpoint::transform(
                &mut slot,
                CssUsedColorTransformation::ForceOpaqueAlpha,
                CssColorBankLengths::new(1, 0, 0)
            )
            .err(),
            Some(CssColorOperationError::Protocol(
                CssColorProtocolError::StorageNotEmpty
            ))
        );
        assert_eq!(slot.as_ref().unwrap().standard_failure(), Some(expected));
        let mut transform = CssUsedColorCheckpoint::transform(
            &mut slot,
            CssUsedColorTransformation::ForceOpaqueAlpha,
            CssColorBankLengths::new(0, 0, 0),
        )
        .unwrap();
        assert!(slot.is_none());
        assert_eq!(
            transform.next_action().description(),
            CssColorActionDescription::Compute(CssColorKernel::Transform(
                CssUsedColorTransformation::ForceOpaqueAlpha
            ))
        );
        transform.next_action().compute().unwrap();
        let checkpoint = CssColorTransformation::take(&mut Some(transform)).unwrap();
        let source = checkpoint.source();
        let value = checkpoint.value();
        assert_eq!(
            serialize(
                CssUsedColorCheckpoint::serialize(
                    &mut Some(checkpoint),
                    CssColorSerializationRequest::HtmlCompatible,
                    CssColorBankLengths::new(0, 0, 0)
                )
                .ok()
                .unwrap()
            ),
            ("#000000".to_owned(), value, Some(source), Some(expected))
        );
    }
    #[test]
    fn reached_profile_metadata_precedes_distinct_forward_mix_and_reverse_capabilities() {
        use super::super::{
            CssColorProfileAvailability as A, CssColorProfileBinding, CssColorProfileConversion,
        };
        use crate::{
            CssColorProfileComponentName, CssColorProfileName, CssColorProfileRenderingIntent,
        };
        let name = CssColorProfileName::try_new("--p").unwrap();
        let component = CssColorProfileComponentName::try_new("one").unwrap();
        let resource = CssColorFactIdentity::new(11, 12);
        let names = [component];
        let bindings = [CssColorProfileBinding::new(
            &name,
            resource,
            A::Loaded,
            CssColorProfileRenderingIntent::RelativeColorimetric,
            1,
            &names,
        )];
        let conversions = [CssColorProfileConversion::Unsupported { profile: resource }];
        let facts = CssColorFacts::new(
            CssColorFactIdentity::new(10, 20),
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &bindings,
            &conversions,
            CssDeviceColorFact::Unknown,
        );
        for (input, expected) in [
            ("color(--p .2)", CssColorUnsupported::ProfileConversion),
            (
                "color-mix(in --p, red, blue)",
                CssColorUnsupported::CustomInterpolation,
            ),
            (
                "color(from red --p one)",
                CssColorUnsupported::ReverseProfile,
            ),
        ] {
            let mut owner = CssColorOperationOwner::new();
            let error = parse(&mut owner, input, &facts)
                .err()
                .expect("actual reached capability");
            assert_eq!(
                error,
                CssColorOperationError::Unsupported(expected),
                "{input}"
            );
        }
        let unknown = [CssColorProfileBinding::new(
            &name,
            resource,
            A::Unknown,
            CssColorProfileRenderingIntent::RelativeColorimetric,
            1,
            &names,
        )];
        let facts = CssColorFacts::new(
            facts.identity,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &unknown,
            &[],
            CssDeviceColorFact::Unknown,
        );
        let mut owner = CssColorOperationOwner::new();
        assert!(matches!(
            parse(&mut owner, "color-mix(in --p, red, blue)", &facts),
            Err(CssColorOperationError::Unavailable(
                CssColorUnavailable::ProfileConversion
            ))
        ));
    }
    #[test]
    fn native_serializer_does_not_invent_source_and_dropped_guard_keeps_exact_action() {
        let mut owner = CssColorOperationOwner::new();
        let value = CssUsedColor::from_srgba8([255, 0, 0, 236]);
        let mut serializer = CssColorSerialization::from_used(
            &mut owner,
            value,
            CssColorSerializationRequest::Css,
            CssColorBankLengths::new(0, 0, 0),
        )
        .unwrap();
        let before = serializer.action();
        {
            let guard = serializer.next_action();
            assert_eq!(guard.description(), before);
        }
        assert_eq!(serializer.action(), before);
        assert_eq!(
            serialize(serializer),
            ("rgba(255, 0, 0, 0.925)".to_owned(), value, None, None)
        );
    }
    #[test]
    fn foreign_pending_record_and_wrong_reply_cannot_advance_public_guard() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let mut other = CssColorOperationOwner::new();
        let mut first = CssColorOperation::parse_used(
            &mut owner,
            CssColorInputKind::Utf8,
            1,
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity,
        )
        .unwrap();
        let mut second = CssColorOperation::parse_used(
            &mut other,
            CssColorInputKind::Utf8,
            1,
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity,
        )
        .unwrap();
        first.next_action().read_input(0, 114).unwrap();
        second.next_action().read_input(0, 114).unwrap();
        let foreign = first.next_action().record().unwrap();
        let pending = second.action();
        assert_eq!(
            second.next_action().read_input(0, 114),
            Err(CssColorOperationError::Protocol(
                CssColorProtocolError::WrongAction
            ))
        );
        assert_eq!(second.action(), pending);
        let mut action = second.next_action();
        let local = action.record().unwrap();
        assert_eq!(
            action.written(CssColorBank::Algorithm, 0, &foreign),
            Err(CssColorOperationError::Protocol(
                CssColorProtocolError::WrongSlot
            ))
        );
        assert_eq!(second.action(), pending);
        assert_eq!(second.bank_length(CssColorBank::Algorithm), 0);
        let mut action = second.next_action();
        let recovered = action.record().unwrap();
        action
            .written(CssColorBank::Algorithm, 0, &recovered)
            .unwrap();
        assert_eq!(second.bank_length(CssColorBank::Algorithm), 1);
        assert_eq!(
            local.kind(),
            super::super::protocol::CssColorSlotKind::Scalar
        );
        second.abandon();
        assert!(second.take().is_err());
        first.abandon();
        assert!(first.take().is_err());
    }
    #[test]
    fn fixed_initialization_and_utf16_observation_are_exposed_before_action() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let layout = CssColorOperation::initialization_layout();
        assert_eq!(
            layout.owner_bytes,
            std::mem::size_of::<CssColorOperationOwner>()
        );
        assert_eq!(
            layout.register_bytes,
            std::mem::size_of::<CssColorOperation<'_, '_>>()
        );
        assert_eq!(
            layout.fact_identity_bytes,
            std::mem::size_of::<CssColorFactIdentity>()
        );
        assert_eq!(
            CssColorOperation::action_bytes(),
            std::mem::size_of::<CssColorAction<'_, '_, '_>>()
        );
        eprintln!(
            "compiled CSS color owner={} slot={} operation={} operation_action={} checkpoint={} checkpoint_option={} transformation={} transformation_option={} serialization={}",
            std::mem::size_of::<CssColorOperationOwner>(),
            std::mem::size_of::<CssColorSlot<'_>>(),
            CssColorOperation::register_bytes(),
            CssColorOperation::action_bytes(),
            std::mem::size_of::<CssUsedColorCheckpoint<'_, '_>>(),
            std::mem::size_of::<Option<CssUsedColorCheckpoint<'_, '_>>>(),
            CssColorTransformation::register_bytes(),
            std::mem::size_of::<Option<CssColorTransformation<'_, '_>>>(),
            CssColorSerialization::register_bytes()
        );
        let expected = CssColorFactIdentity::new(90, 91);
        let refused = CssColorOperation::parse_used(
            &mut owner,
            CssColorInputKind::Utf16,
            3,
            CssColorGrammarRequest::WholeColor,
            &facts,
            expected,
        )
        .err();
        assert_eq!(
            refused,
            Some(CssColorOperationError::Facts(CssColorFactError::Foreign))
        );
        assert_eq!(owner.current_phase(), 0);
        let mut operation = CssColorOperation::parse_used(
            &mut owner,
            CssColorInputKind::Utf16,
            3,
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity(),
        )
        .unwrap();
        let pending = operation.action();
        assert_eq!(pending, CssColorActionDescription::ReadInput { offset: 0 });
        {
            let action = operation.next_action();
            assert_eq!(action.layout().input_unit_bytes, std::mem::size_of::<u16>());
            assert_eq!(action.layout().slot_read_bytes, 0);
            assert_eq!(action.layout().slot_write_bytes, 0);
            assert_eq!(action.layout().fact_records, [0; 3]);
            // Dropping the guard executes no observation or transition.
        }
        assert_eq!(operation.action(), pending);
        operation.abandon();
        assert_eq!(operation.action(), CssColorActionDescription::Stopped);
    }

    #[test]
    fn completed_intrinsic_kernels_distinguish_contrast_and_each_mix_fold() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let text = "color-mix(in srgb, contrast-color(black) 25%, black 25%, white 50%)";
        let mut operation = CssColorOperation::parse_used(
            &mut owner,
            CssColorInputKind::Utf8,
            text.len(),
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity(),
        )
        .unwrap();
        let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
        let mut contrast = 0;
        let mut interpolation = 0;
        loop {
            let mut action = operation.next_action();
            let layout = action.layout();
            match action.description() {
                CssColorActionDescription::ReadInput { offset } => action
                    .read_input(offset, u16::from(text.as_bytes()[offset]))
                    .unwrap(),
                CssColorActionDescription::ReadSlot { bank: which, index } => action
                    .read_slot(which, index, &banks[bank(which)][index])
                    .unwrap(),
                CssColorActionDescription::AppendSlot { bank: which, index } => {
                    assert_eq!(index, banks[bank(which)].len());
                    banks[bank(which)].push(action.record().unwrap());
                    action
                        .written(which, index, &banks[bank(which)][index])
                        .unwrap();
                }
                CssColorActionDescription::UpdateSlot { bank: which, index } => {
                    banks[bank(which)][index] = action.record().unwrap();
                    action
                        .written(which, index, &banks[bank(which)][index])
                        .unwrap();
                }
                CssColorActionDescription::Compute(kernel) => {
                    assert_eq!(layout.kernel, kernel);
                    contrast += usize::from(kernel == CssColorKernel::Contrast);
                    interpolation += usize::from(kernel == CssColorKernel::Interpolation);
                    action.compute().unwrap();
                }
                CssColorActionDescription::ReadFact(_) => action.compute().unwrap(),
                CssColorActionDescription::UsedReady => {
                    break;
                }
                other => panic!("unexpected {other:?}"),
            }
        }
        let CssColorParseOutcome::Used(value) = operation.take().unwrap() else {
            panic!("valid intrinsic operation")
        };
        banks.iter_mut().for_each(Vec::clear);
        drop(banks);
        // One contrast branch; the first member only initializes/converts the
        // retained aggregate, then two actual two-color interpolation folds.
        assert_eq!(contrast, 1);
        assert_eq!(interpolation, 2);
        assert_eq!(
            value.value().channels(),
            [Some(0.75), Some(0.75), Some(0.75)]
        );
    }

    #[test]
    fn relative_and_mix_cylindrical_results_preserve_the_color5_serialization_family() {
        let facts = facts();
        for text in [
            "hsl(from red h s l)",
            "hwb(from red h w b)",
            "color-mix(in hsl, red, red)",
            "color-mix(in hwb, red, red)",
        ] {
            let mut owner = CssColorOperationOwner::new();
            let (outcome, _) = parse(&mut owner, text, &facts).unwrap();
            let CssColorParseOutcome::Used(value) = outcome else {
                panic!("source-valid selected intrinsic color")
            };
            let serializer = CssUsedColorCheckpoint::serialize(
                &mut Some(value),
                CssColorSerializationRequest::Css,
                CssColorBankLengths::new(0, 0, 0),
            )
            .ok()
            .unwrap();
            // Color5 serialization converts present cylindrical results into
            // modern color(srgb ...); absolute hsl/hwb remain Color4 rgb(...).
            assert_eq!(serialize(serializer).0, "color(srgb 1 0 0)", "{text}");
        }
        for (text, expected) in [
            ("hsl(0 100% 50%)", "rgb(255, 0, 0)"),
            ("hwb(0 0% 0%)", "rgb(255, 0, 0)"),
            ("hsl(from red none s l)", "hsl(none 100% 50%)"),
        ] {
            let mut owner = CssColorOperationOwner::new();
            let (outcome, _) = parse(&mut owner, text, &facts).unwrap();
            let CssColorParseOutcome::Used(value) = outcome else {
                panic!("selected intrinsic color")
            };
            let serializer = CssUsedColorCheckpoint::serialize(
                &mut Some(value),
                CssColorSerializationRequest::Css,
                CssColorBankLengths::new(0, 0, 0),
            )
            .ok()
            .unwrap();
            assert_eq!(serialize(serializer).0, expected, "{text}");
        }
    }

    #[test]
    fn every_real_bank_refuses_wrong_replies_and_retired_slots_cannot_cross_phase() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let text = "rgb(calc(1 + 2) 0 0)";
        let mut operation = CssColorOperation::parse_used(
            &mut owner,
            CssColorInputKind::Utf8,
            text.len(),
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity(),
        )
        .unwrap();
        let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
        let mut read_banks = [false; 3];
        loop {
            let pending = operation.action();
            let mut action = operation.next_action();
            match pending {
                CssColorActionDescription::ReadInput { offset } => action
                    .read_input(offset, u16::from(text.as_bytes()[offset]))
                    .unwrap(),
                CssColorActionDescription::ReadSlot { bank: which, index } => {
                    let other = match which {
                        CssColorBank::Algorithm => CssColorBank::Arithmetic,
                        _ => CssColorBank::Algorithm,
                    };
                    assert_eq!(
                        action.read_slot(other, index, &banks[bank(which)][index]),
                        Err(CssColorOperationError::Protocol(
                            CssColorProtocolError::WrongAction
                        ))
                    );
                    assert_eq!(operation.action(), pending);
                    assert_eq!(
                        operation.next_action().read_slot(
                            which,
                            index + 1,
                            &banks[bank(which)][index]
                        ),
                        Err(CssColorOperationError::Protocol(
                            CssColorProtocolError::WrongAction
                        ))
                    );
                    assert_eq!(operation.action(), pending);
                    operation
                        .next_action()
                        .read_slot(which, index, &banks[bank(which)][index])
                        .unwrap();
                    read_banks[bank(which)] = true;
                }
                CssColorActionDescription::AppendSlot { bank: which, index } => {
                    assert_eq!(index, banks[bank(which)].len());
                    let record = action.record().unwrap();
                    assert_eq!(record.bank(), which);
                    assert!(matches!(
                        action.record(),
                        Err(CssColorOperationError::Protocol(
                            CssColorProtocolError::WrongAction
                        ))
                    ));
                    banks[bank(which)].push(record);
                    action
                        .written(which, index, &banks[bank(which)][index])
                        .unwrap();
                }
                CssColorActionDescription::UpdateSlot { bank: which, index } => {
                    banks[bank(which)][index] = action.record().unwrap();
                    action
                        .written(which, index, &banks[bank(which)][index])
                        .unwrap();
                }
                CssColorActionDescription::Compute(_) | CssColorActionDescription::ReadFact(_) => {
                    action.compute().unwrap()
                }
                CssColorActionDescription::UsedReady => break,
                other => panic!("unexpected parse action {other:?}"),
            }
        }
        let CssColorParseOutcome::Used(checkpoint) = operation.take().unwrap() else {
            panic!("valid intrinsic arithmetic")
        };
        assert_eq!(read_banks, [true; 3]);
        assert_eq!(
            checkpoint.value().channels(),
            [Some(3.0 / 255.0), Some(0.0), Some(0.0)]
        );
        // Logical lengths supplement actual disposal; they cannot make a
        // retained old phase record usable in a later serialization operation.
        // This ordinary fixture retains the actual old records to prove that
        // qualification, without pretending they have physical ledger tickets.
        let mut serializer = CssUsedColorCheckpoint::serialize(
            &mut Some(checkpoint),
            CssColorSerializationRequest::Css,
            CssColorBankLengths::new(0, 0, 0),
        )
        .ok()
        .unwrap();
        let mut digits = Vec::new();
        let mut output = Vec::new();
        let mut refused = false;
        loop {
            let pending = serializer.action();
            let mut action = serializer.next_action();
            match pending {
                CssColorActionDescription::Compute(_) => action.compute().unwrap(),
                CssColorActionDescription::ReadSlot { bank: which, index } => {
                    if !refused {
                        assert_eq!(
                            action.read_slot(which, index, &banks[0][0]),
                            Err(CssColorProtocolError::ForeignSlot)
                        );
                        assert_eq!(serializer.action(), pending);
                        serializer
                            .next_action()
                            .read_slot(which, index, &digits[index])
                            .unwrap();
                        refused = true;
                    } else {
                        action.read_slot(which, index, &digits[index]).unwrap();
                    }
                }
                CssColorActionDescription::AppendSlot { bank: which, index } => {
                    assert_eq!(index, digits.len());
                    digits.push(action.record().unwrap());
                    action.written(which, index, &digits[index]).unwrap();
                }
                CssColorActionDescription::UpdateSlot { bank: which, index } => {
                    digits[index] = action.record().unwrap();
                    action.written(which, index, &digits[index]).unwrap();
                }
                CssColorActionDescription::EmitUnit { unit } => {
                    output.push(unit);
                    action.emitted(unit).unwrap();
                }
                CssColorActionDescription::Serialized => break,
                other => panic!("unexpected serialization action {other:?}"),
            }
        }
        assert!(refused);
        assert_eq!(output, b"rgb(3, 0, 0)");
        assert_eq!(serializer.take().unwrap().length(), output.len());
        banks.iter_mut().for_each(Vec::clear);
        digits.clear();
    }

    #[test]
    fn computed_but_untaken_transformation_recovers_the_original_checkpoint() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let (outcome, _) = parse(&mut owner, "rgb(100% 0 0 / .5)", &facts).unwrap();
        let CssColorParseOutcome::Used(checkpoint) = outcome else {
            panic!("valid resolved color")
        };
        let original = checkpoint.value();
        let source = checkpoint.source();
        assert_eq!(original.srgb_bytes(), None);
        let mut slot = Some(checkpoint);
        let mut transform = CssUsedColorCheckpoint::transform(
            &mut slot,
            CssUsedColorTransformation::QuantizeSrgb,
            CssColorBankLengths::new(0, 0, 0),
        )
        .unwrap();
        transform.next_action().compute().unwrap();
        transform.abandon();
        let recovered = transform.recover();
        // Numerical computation alone does not publish a checkpoint. Only a
        // successful take commits its result; abandonment preserves the input.
        assert!(recovered.value().identical(original));
        assert_eq!(recovered.value().srgb_bytes(), None);
        assert_eq!(recovered.value().alpha(), Some(0.5));
        assert_eq!(recovered.source(), source);
        assert!(std::ptr::eq(recovered.facts(), &facts));
    }

    #[test]
    fn caller_checkpoint_slots_keep_refused_and_pending_owners_and_recover_committed_values() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let (outcome, _) = parse(&mut owner, "rgba(255 0 0 / .5)", &facts).unwrap();
        let CssColorParseOutcome::Used(checkpoint) = outcome else {
            panic!("valid resolved color")
        };
        let source = checkpoint.source();
        let original = checkpoint.value();
        let mut slot = Some(checkpoint);
        let banks = CssColorBankLengths::new(0, 0, 1);
        assert_eq!(
            CssUsedColorCheckpoint::transform(
                &mut slot,
                CssUsedColorTransformation::ForceOpaqueAlpha,
                banks
            )
            .err(),
            Some(CssColorOperationError::Protocol(
                CssColorProtocolError::StorageNotEmpty
            ))
        );
        assert_eq!(slot.as_ref().unwrap().source(), source);
        assert_eq!(slot.as_ref().unwrap().value(), original);
        assert!(std::ptr::eq(slot.as_ref().unwrap().facts(), &facts));
        assert_eq!(
            CssUsedColorCheckpoint::serialize(&mut slot, CssColorSerializationRequest::Css, banks)
                .err(),
            Some(CssColorOperationError::Protocol(
                CssColorProtocolError::StorageNotEmpty
            ))
        );
        assert_eq!(slot.as_ref().unwrap().source(), source);
        assert_eq!(slot.as_ref().unwrap().value(), original);
        let transformation = CssUsedColorCheckpoint::transform(
            &mut slot,
            CssUsedColorTransformation::ForceOpaqueAlpha,
            CssColorBankLengths::new(0, 0, 0),
        )
        .unwrap();
        assert!(slot.is_none());
        assert_eq!(
            CssUsedColorCheckpoint::transform(
                &mut slot,
                CssUsedColorTransformation::ForceOpaqueAlpha,
                CssColorBankLengths::new(0, 0, 0)
            )
            .err(),
            Some(CssColorOperationError::Protocol(
                CssColorProtocolError::EmptySlot
            ))
        );
        let mut transform = Some(transformation);
        let pending = transform.as_ref().unwrap().action();
        assert_eq!(
            CssColorTransformation::take(&mut transform).err(),
            Some(CssColorOperationError::Protocol(
                CssColorProtocolError::Terminal
            ))
        );
        assert_eq!(transform.as_ref().unwrap().action(), pending);
        transform.as_mut().unwrap().next_action().compute().unwrap();
        let checkpoint = CssColorTransformation::take(&mut transform).unwrap();
        assert!(transform.is_none());
        assert_eq!(
            CssColorTransformation::take(&mut transform).err(),
            Some(CssColorOperationError::Protocol(
                CssColorProtocolError::EmptySlot
            ))
        );
        assert_eq!(checkpoint.value().alpha(), Some(1.0));
        assert_eq!(checkpoint.source(), source);
        let mut slot = Some(checkpoint);
        // Once computation commits, an explicit abandon retains that latest
        // committed value rather than restoring the input checkpoint.
        let mut transform = CssUsedColorCheckpoint::transform(
            &mut slot,
            CssUsedColorTransformation::QuantizeSrgb,
            CssColorBankLengths::new(0, 0, 0),
        )
        .unwrap();
        transform.next_action().compute().unwrap();
        transform.abandon();
        let mut transform = Some(transform);
        assert_eq!(
            CssColorTransformation::take(&mut transform).err(),
            Some(CssColorOperationError::Protocol(
                CssColorProtocolError::Terminal
            ))
        );
        let checkpoint = transform.take().unwrap().recover();
        assert_eq!(checkpoint.value().srgb_bytes(), Some([255, 0, 0])); // Channel quantization does not invent alpha byte provenance.
        assert_eq!(checkpoint.value().alpha(), Some(1.0));
        assert_eq!(checkpoint.value().alpha_byte(), None);
        assert_eq!(checkpoint.source(), source);
        let mut slot = Some(checkpoint);
        let serializer = CssUsedColorCheckpoint::serialize(
            &mut slot,
            CssColorSerializationRequest::HtmlCompatible,
            CssColorBankLengths::new(0, 0, 0),
        )
        .unwrap();
        assert!(slot.is_none());
        assert_eq!(
            CssUsedColorCheckpoint::serialize(
                &mut slot,
                CssColorSerializationRequest::Css,
                CssColorBankLengths::new(0, 0, 0)
            )
            .err(),
            Some(CssColorOperationError::Protocol(
                CssColorProtocolError::EmptySlot
            ))
        );
        assert_eq!(serialize(serializer).0, "#ff0000");

        let mut owner = CssColorOperationOwner::new();
        let (outcome, _) = parse(
            &mut owner,
            "color(srgb-linear 1.7e308 1.7e308 1.7e308)",
            &facts,
        )
        .unwrap();
        let CssColorParseOutcome::Used(checkpoint) = outcome else {
            panic!("finite intrinsic channels")
        };
        let before = checkpoint.value();
        let source = checkpoint.source();
        let mut slot = Some(checkpoint);
        let mut transform = CssUsedColorCheckpoint::transform(
            &mut slot,
            CssUsedColorTransformation::Convert(crate::CssColorInterpolationSpace::Predefined(
                crate::CssPredefinedColorSpace::XyzD65,
            )),
            CssColorBankLengths::new(0, 0, 0),
        )
        .unwrap();
        assert_eq!(
            transform.next_action().compute(),
            Err(CssColorOperationError::Numerical(
                CssUsedColorError::Unrepresentable
            ))
        );
        transform.abandon();
        let checkpoint = transform.recover();
        assert!(checkpoint.value().identical(before));
        assert_eq!(checkpoint.source(), source);
        assert_eq!(checkpoint.standard_failure(), None);
    }

    #[test]
    fn final_cylindrical_conversion_is_advertised_pending_and_abandonable() {
        let facts = facts();
        for abandon in [false, true] {
            let mut owner = CssColorOperationOwner::new();
            let text = "hsl(0 100% 50%)";
            let mut operation = CssColorOperation::parse_used(
                &mut owner,
                CssColorInputKind::Utf8,
                text.len(),
                CssColorGrammarRequest::WholeColor,
                &facts,
                facts.identity(),
            )
            .unwrap();
            let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
            let mut conversion = false;
            loop {
                let pending = operation.action();
                if pending == CssColorActionDescription::Compute(CssColorKernel::ColorConversion) {
                    conversion = true;
                    {
                        let action = operation.next_action();
                        let layout = action.layout();
                        assert_eq!(layout.kernel, CssColorKernel::ColorConversion);
                        assert_eq!(layout.input_unit_bytes, 0);
                        assert_eq!(layout.slot_read_bytes, 0);
                        assert_eq!(layout.slot_write_bytes, 0);
                        assert_eq!(layout.output_unit_bytes, 0);
                        assert_eq!(layout.fact_records, [0; 3]);
                        // The caller withholds admission. Dropping this actual
                        // guard must neither convert nor publish the value.
                    }
                    assert_eq!(operation.action(), pending);
                    assert_eq!(
                        operation.next_action().read_input(0, 0),
                        Err(CssColorOperationError::Protocol(
                            CssColorProtocolError::WrongAction
                        ))
                    );
                    assert_eq!(operation.action(), pending);
                    if abandon {
                        // CSS cancellation is the original caller's explicit
                        // abandon, not a second ledger or invented HTML ticket.
                        let lengths = banks.each_ref().map(Vec::len);
                        assert!(lengths[0] > 0);
                        operation.abandon();
                        assert_eq!(operation.action(), CssColorActionDescription::Stopped);
                        assert_eq!(banks.each_ref().map(Vec::len), lengths);
                        assert!(matches!(
                            operation.take(),
                            Err(CssColorProtocolError::Terminal)
                        ));
                        break;
                    }
                }
                let mut action = operation.next_action();
                match pending {
                    CssColorActionDescription::ReadInput { offset } => action
                        .read_input(offset, u16::from(text.as_bytes()[offset]))
                        .unwrap(),
                    CssColorActionDescription::ReadSlot { bank: which, index } => action
                        .read_slot(which, index, &banks[bank(which)][index])
                        .unwrap(),
                    CssColorActionDescription::AppendSlot { bank: which, index } => {
                        assert_eq!(index, banks[bank(which)].len());
                        banks[bank(which)].push(action.record().unwrap());
                        action
                            .written(which, index, &banks[bank(which)][index])
                            .unwrap();
                    }
                    CssColorActionDescription::UpdateSlot { bank: which, index } => {
                        banks[bank(which)][index] = action.record().unwrap();
                        action
                            .written(which, index, &banks[bank(which)][index])
                            .unwrap();
                    }
                    CssColorActionDescription::ReadFact(_)
                    | CssColorActionDescription::Compute(_) => action.compute().unwrap(),
                    CssColorActionDescription::UsedReady => {
                        let CssColorParseOutcome::Used(value) = operation.take().unwrap() else {
                            panic!("used checkpoint")
                        };
                        assert_eq!(
                            value.value().space(),
                            crate::CssColorInterpolationSpace::Predefined(
                                crate::CssPredefinedColorSpace::Srgb
                            )
                        );
                        assert_eq!(value.value().channels(), [Some(1.0), Some(0.0), Some(0.0)]);
                        assert!(!value.value().uses_color_function());
                        assert_eq!(value.value().srgb_bytes(), None);
                        assert!(std::ptr::eq(value.facts(), &facts));
                        break;
                    }
                    other => panic!("unexpected parse action {other:?}"),
                }
            }
            banks.iter_mut().for_each(Vec::clear);
            drop(banks);
            assert!(
                conversion,
                "actual root conversion must be admitted before Used"
            );
        }
    }
}
