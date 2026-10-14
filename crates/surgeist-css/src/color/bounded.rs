//! Public specialized continuation over one canonical CSS color operation.
//!
//! The consumer owns input, all three real storage banks and its controller.
//! An action describes one finite observation/update. Nothing allocates,
//! releases capacity, executes abandoned work or publishes partial output here.

use super::operation as core;
use super::protocol::{CssColorBank, CssColorOperationOwner, CssColorProtocolError, CssColorSlot};
use super::{Color, ColorError, CssColorFactIdentity, CssColorFacts, CssColorKernel};
use surgeist_color::{SerializationRequest, Transformation};

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
#[derive(Debug, Eq, PartialEq)]
pub enum CssColorOperationError {
    Protocol(CssColorProtocolError),
    Unavailable(CssColorUnavailable),
    Unsupported(CssColorUnsupported),
    Facts(CssColorFactError),
    Numerical {
        error: ColorError,
        site: CssColorResolutionSite,
    },
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
            Self::Numerical { error, .. } => std::fmt::Display::fmt(error, f),
        }
    }
}
impl std::error::Error for CssColorOperationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Protocol(error) => Some(error),
            Self::Numerical { error, .. } => Some(error),
            _ => None,
        }
    }
}
impl From<CssColorProtocolError> for CssColorOperationError {
    fn from(error: CssColorProtocolError) -> Self {
        Self::Protocol(error)
    }
}
/// Original source coordinates or the owned path/origin of the reached authored
/// operand. Authored backing transfers before a numerical error is returned.
#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorResolutionSite {
    Source(CssColorInputRange),
    Authored {
        path: super::authored_resolution::CssColorComponentPath,
        origin: Option<crate::CssValueOrigin>,
    },
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
            core::Error::Numerical { error, site } => {
                let site = site
                    .source()
                    .expect("source adapter consumes source operation errors");
                Self::Numerical {
                    error,
                    site: CssColorResolutionSite::Source(CssColorInputRange {
                        start: site.start,
                        end: site.end,
                    }),
                }
            }
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
                | R::Interpolation(I::Numerical(error)) => {
                    let _ = error;
                    unreachable!(
                        "numerical errors are decorated at the reached core recipe before abandonment"
                    )
                }
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
                    unreachable!("numerical mix overflow is decorated before leaving the core")
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
    ReadNumericalDigit { index: usize },
    AppendNumericalDigit { index: usize },
    UpdateNumericalDigit { index: usize },
    Compute(CssColorKernel),
    EmitUnit { unit: u8 },
    UsedReady,
    StandardFailed,
    Serialized,
    Stopped,
}
pub(super) fn description(cursor: &core::ColorOperation<'_, '_>) -> CssColorActionDescription {
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
    pub numerical_read_bytes: usize,
    pub numerical_write_bytes: usize,
    pub kernel: CssColorKernel,
}
fn layout(
    description: CssColorActionDescription,
    kind: CssColorInputKind,
    kernel: CssColorKernel,
) -> CssColorActionLayout {
    use std::mem::size_of;
    let mut layout = CssColorActionLayout {
        numerical_read_bytes: 0,
        numerical_write_bytes: 0,
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
            layout.slot_read_bytes = CssColorSlot::record_bytes()
        }
        CssColorActionDescription::AppendSlot { .. }
        | CssColorActionDescription::UpdateSlot { .. } => {
            // Generated opaque payload plus actual stored acknowledgement.
            layout.slot_write_bytes = CssColorSlot::record_bytes();
            layout.slot_read_bytes = CssColorSlot::record_bytes();
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
    cursor: core::ColorOperation<'owner, 'facts>,
}
impl<'owner, 'facts> CssColorOperation<'owner, 'facts> {
    /// Fixed metadata admission precedes this call; no array/input is scanned.
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
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
            cursor: core::ColorOperation::start(
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
    pub fn take(
        slot: &mut Option<Self>,
    ) -> Result<CssColorParseOutcome<'owner, 'facts>, CssColorProtocolError> {
        let operation = slot.as_ref().ok_or(CssColorProtocolError::EmptySlot)?;
        if !matches!(
            operation.cursor.action(),
            core::Action::UsedReady | core::Action::StandardFailed
        ) {
            return Err(CssColorProtocolError::Terminal);
        }
        let operation = slot
            .take()
            .expect("shared preflight proved a completed parse");
        Ok(match operation.cursor.take()? {
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
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn read_input(self, offset: usize, unit: u16) -> Result<(), CssColorOperationError> {
        if self.description != (CssColorActionDescription::ReadInput { offset }) {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        self.operation
            .cursor
            .read_input(offset, unit)
            .map_err(Into::into)
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
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
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
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
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
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
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
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
    pub(super) checkpoint: core::UsedCheckpoint<'owner, 'facts>,
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
    let source = value
        .site
        .source()
        .expect("source checkpoint retains source provenance");
    CssColorStandardFailure {
        kind: match value.reason {
            R::ColorGrammar => CssColorStandardFailureKind::ColorGrammar,
            R::ComponentGrammar => CssColorStandardFailureKind::ComponentGrammar,
            R::NumericGrammar => CssColorStandardFailureKind::NumericGrammar,
            R::InvalidProfile => CssColorStandardFailureKind::InvalidProfile,
            R::InvalidComponents => CssColorStandardFailureKind::InvalidComponents,
        },
        source: CssColorInputRange {
            start: source.start,
            end: source.end,
        },
    }
}
impl<'owner, 'facts> CssUsedColorCheckpoint<'owner, 'facts> {
    #[must_use]
    pub const fn value(&self) -> Color {
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
    pub fn fallback(self, value: Color) -> CssUsedColorCheckpoint<'owner, 'facts> {
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
fn numerical_error(
    error: surgeist_color::ComputationError,
    range: CssColorInputRange,
) -> CssColorOperationError {
    match error {
        surgeist_color::ComputationError::Numerical(error) => CssColorOperationError::Numerical {
            error,
            site: CssColorResolutionSite::Source(range),
        },
        surgeist_color::ComputationError::Interpolation(
            surgeist_color::InterpolationError::Numerical(error),
        ) => CssColorOperationError::Numerical {
            error,
            site: CssColorResolutionSite::Source(range),
        },
        surgeist_color::ComputationError::Protocol(error) => {
            CssColorProtocolError::NumericalProtocol(error).into()
        }
        _ => unreachable!(
            "CSS fixed transformations use checked numerical requests without interpolation policy errors"
        ),
    }
}
fn numerical_serialization_error(
    error: surgeist_color::SerializationError,
    range: CssColorInputRange,
) -> CssColorOperationError {
    match error {
        surgeist_color::SerializationError::Numerical(error) => CssColorOperationError::Numerical {
            error,
            site: CssColorResolutionSite::Source(range),
        },
        surgeist_color::SerializationError::Protocol(error) => {
            CssColorProtocolError::NumericalProtocol(error).into()
        }
        _ => unreachable!(
            "controlled serialization does not execute ordinary allocation or admission policy"
        ),
    }
}
fn numerical_description(action: surgeist_color::ActionDescription) -> CssColorActionDescription {
    use surgeist_color::ActionDescription as N;
    match action {
        N::Compute(k) => CssColorActionDescription::Compute(CssColorKernel::Numerical(k)),
        N::ReadDigit { index } => CssColorActionDescription::ReadNumericalDigit { index },
        N::AppendDigit { index } => CssColorActionDescription::AppendNumericalDigit { index },
        N::ReplaceDigit { index } => CssColorActionDescription::UpdateNumericalDigit { index },
        N::EmitByte { unit } => CssColorActionDescription::EmitUnit { unit },
        N::Ready => CssColorActionDescription::Serialized,
        N::Failed | N::Abandoned => CssColorActionDescription::Stopped,
    }
}

pub struct CssColorTransformation<'css, 'num, 'facts> {
    checkpoint: core::UsedCheckpoint<'css, 'facts>,
    numerical: Option<surgeist_color::Computation<'num>>,
}
impl<'css, 'num, 'facts> CssColorTransformation<'css, 'num, 'facts> {
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    pub const fn action_bytes() -> usize {
        std::mem::size_of::<CssColorTransformAction<'_, '_, '_, '_>>()
    }
    pub fn action(&self) -> CssColorActionDescription {
        match self
            .numerical
            .as_ref()
            .expect("owned numerical computation")
            .action()
        {
            surgeist_color::ActionDescription::Ready => CssColorActionDescription::UsedReady,
            a => numerical_description(a),
        }
    }
    pub fn next_action(&mut self) -> CssColorTransformAction<'_, 'css, 'num, 'facts> {
        CssColorTransformAction { operation: self }
    }
    pub fn abandon(&mut self) {
        self.numerical
            .as_mut()
            .expect("owned computation")
            .abandon();
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn take(
        slot: &mut Option<Self>,
    ) -> Result<CssUsedColorCheckpoint<'css, 'facts>, CssColorOperationError> {
        let actual = slot.as_mut().ok_or(CssColorProtocolError::EmptySlot)?;
        let range = source(&actual.checkpoint.context).consumed;
        let value = surgeist_color::Computation::take(&mut actual.numerical)
            .map_err(|e| numerical_error(e, range))?;
        let mut actual = slot.take().expect("preflight kept wrapper");
        actual.checkpoint.value = value;
        Ok(CssUsedColorCheckpoint {
            checkpoint: actual.checkpoint,
        })
    }
    pub fn recover(
        slot: &mut Option<Self>,
    ) -> Result<CssUsedColorCheckpoint<'css, 'facts>, CssColorProtocolError> {
        let actual = slot.as_mut().ok_or(CssColorProtocolError::EmptySlot)?;
        surgeist_color::Computation::recover(&mut actual.numerical)
            .map_err(CssColorProtocolError::NumericalProtocol)?;
        Ok(CssUsedColorCheckpoint {
            checkpoint: slot
                .take()
                .expect("shared preflight proved Some")
                .checkpoint,
        })
    }
}
pub struct CssColorTransformAction<'step, 'css, 'num, 'facts> {
    operation: &'step mut CssColorTransformation<'css, 'num, 'facts>,
}
impl CssColorTransformAction<'_, '_, '_, '_> {
    pub fn description(&self) -> CssColorActionDescription {
        self.operation.action()
    }
    pub fn layout(&self) -> CssColorActionLayout {
        let d = self.description();
        layout(
            d,
            CssColorInputKind::Utf8,
            match d {
                CssColorActionDescription::Compute(k) => k,
                _ => CssColorKernel::Transition,
            },
        )
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn compute(self) -> Result<(), CssColorOperationError> {
        let range = source(&self.operation.checkpoint.context).consumed;
        self.operation
            .numerical
            .as_mut()
            .expect("owned computation")
            .next_action()
            .compute()
            .map_err(|e| numerical_error(e, range))
    }
}
impl<'css, 'facts> CssUsedColorCheckpoint<'css, 'facts> {
    pub fn source(&self) -> CssColorSourceDescriptor {
        source(self.checkpoint.context())
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn transform<'num>(
        slot: &mut Option<Self>,
        numerical_owner: &'num mut surgeist_color::OperationOwner,
        transformation: Transformation,
        banks: CssColorBankLengths,
    ) -> Result<CssColorTransformation<'css, 'num, 'facts>, CssColorOperationError> {
        let checkpoint = slot.as_ref().ok_or(CssColorProtocolError::EmptySlot)?;
        banks.empty()?;
        let context = checkpoint.checkpoint.context();
        let next = context.owner.prepare_advance(context.phase)?;
        let numerical = surgeist_color::Computation::new(
            numerical_owner,
            surgeist_color::NumericalRequest::Transform {
                value: checkpoint.value(),
                transformation,
            },
        )
        .map_err(CssColorProtocolError::NumericalProtocol)?;
        context.owner.commit_phase(context.phase, next);
        let mut checkpoint = slot
            .take()
            .expect("shared preflight proved Some")
            .checkpoint;
        checkpoint.context.phase = next;
        Ok(CssColorTransformation {
            checkpoint,
            numerical: Some(numerical),
        })
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn serialize<'num>(
        slot: &mut Option<Self>,
        numerical_owner: &'num mut surgeist_color::OperationOwner,
        request: SerializationRequest,
        banks: CssColorBankLengths,
        numerical_digits: usize,
    ) -> Result<CssColorSerialization<'css, 'num, 'facts>, CssColorOperationError> {
        let checkpoint = slot.as_ref().ok_or(CssColorProtocolError::EmptySlot)?;
        banks.empty()?;
        if numerical_digits != 0 {
            return Err(CssColorProtocolError::NumericalProtocol(
                surgeist_color::ProtocolError::StorageNotEmpty,
            )
            .into());
        }
        let context = checkpoint.checkpoint.context();
        let next = context.owner.prepare_advance(context.phase)?;
        let numerical =
            surgeist_color::Serialization::new(numerical_owner, checkpoint.value(), request)
                .map_err(CssColorProtocolError::NumericalProtocol)?;
        context.owner.commit_phase(context.phase, next);
        let mut checkpoint = slot
            .take()
            .expect("shared preflight proved Some")
            .checkpoint;
        checkpoint.context.phase = next;
        Ok(CssColorSerialization {
            checkpoint,
            numerical: Some(numerical),
        })
    }
}
pub struct CssColorSerialization<'css, 'num, 'facts> {
    checkpoint: core::UsedCheckpoint<'css, 'facts>,
    numerical: Option<surgeist_color::Serialization<'num>>,
}
impl<'css, 'num, 'facts> CssColorSerialization<'css, 'num, 'facts> {
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    pub const fn action_bytes() -> usize {
        std::mem::size_of::<CssColorSerializationAction<'_, '_, '_, '_>>()
    }
    pub fn action(&self) -> CssColorActionDescription {
        numerical_description(
            self.numerical
                .as_ref()
                .expect("owned serialization")
                .action(),
        )
    }
    pub fn next_action(&mut self) -> CssColorSerializationAction<'_, 'css, 'num, 'facts> {
        CssColorSerializationAction { operation: self }
    }
    pub fn abandon(&mut self) {
        self.numerical
            .as_mut()
            .expect("owned serialization")
            .abandon();
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn take(
        slot: &mut Option<Self>,
    ) -> Result<CssColorSerializationReceipt<'css, 'facts>, CssColorOperationError> {
        let actual = slot.as_mut().ok_or(CssColorProtocolError::EmptySlot)?;
        let range = source(&actual.checkpoint.context).consumed;
        let receipt = surgeist_color::Serialization::take(&mut actual.numerical)
            .map_err(|e| numerical_serialization_error(e, range))?;
        let actual = slot.take().expect("shared preflight proved Some");
        Ok(CssColorSerializationReceipt {
            checkpoint: actual.checkpoint,
            length: receipt.length(),
        })
    }
    pub fn recover(
        slot: &mut Option<Self>,
    ) -> Result<CssUsedColorCheckpoint<'css, 'facts>, CssColorProtocolError> {
        let actual = slot.as_mut().ok_or(CssColorProtocolError::EmptySlot)?;
        surgeist_color::Serialization::recover(&mut actual.numerical)
            .map_err(CssColorProtocolError::NumericalProtocol)?;
        Ok(CssUsedColorCheckpoint {
            checkpoint: slot
                .take()
                .expect("shared preflight proved Some")
                .checkpoint,
        })
    }
}
pub struct CssColorSerializationAction<'step, 'css, 'num, 'facts> {
    operation: &'step mut CssColorSerialization<'css, 'num, 'facts>,
}
impl<'num> CssColorSerializationAction<'_, '_, 'num, '_> {
    pub fn description(&self) -> CssColorActionDescription {
        self.operation.action()
    }
    pub fn layout(&mut self) -> CssColorActionLayout {
        let mut result = layout(
            self.description(),
            CssColorInputKind::Utf8,
            CssColorKernel::Transition,
        );
        let numerical = self
            .operation
            .numerical
            .as_mut()
            .expect("owned serialization")
            .next_action()
            .layout();
        result.numerical_read_bytes = numerical.read_bytes;
        result.numerical_write_bytes = numerical.write_bytes;
        result.output_unit_bytes = numerical.output_bytes;
        if let CssColorActionDescription::Compute(k) = self.description() {
            result.kernel = k;
        }
        result
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn compute(self) -> Result<(), CssColorOperationError> {
        let range = source(&self.operation.checkpoint.context).consumed;
        self.operation
            .numerical
            .as_mut()
            .expect("owned serialization")
            .next_action()
            .compute()
            .map_err(|e| numerical_serialization_error(e, range))
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn read_numerical_digit(
        self,
        index: usize,
        slot: &surgeist_color::DigitSlot<'_>,
    ) -> Result<(), CssColorOperationError> {
        let context = &self.operation.checkpoint.context;
        if context.owner.current_phase() != context.phase {
            return Err(CssColorProtocolError::ForeignSlot.into());
        }
        self.operation
            .numerical
            .as_mut()
            .expect("owned serialization")
            .next_action()
            .read_digit(index, slot)
            .map_err(|e| CssColorProtocolError::NumericalProtocol(e).into())
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn numerical_record(
        &mut self,
    ) -> Result<surgeist_color::DigitSlot<'num>, CssColorOperationError> {
        let context = &self.operation.checkpoint.context;
        if context.owner.current_phase() != context.phase {
            return Err(CssColorProtocolError::ForeignSlot.into());
        }
        self.operation
            .numerical
            .as_mut()
            .expect("owned serialization")
            .next_action()
            .record()
            .map_err(|e| CssColorProtocolError::NumericalProtocol(e).into())
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn numerical_written(
        self,
        index: usize,
        slot: &surgeist_color::DigitSlot<'_>,
    ) -> Result<(), CssColorOperationError> {
        let context = &self.operation.checkpoint.context;
        if context.owner.current_phase() != context.phase {
            return Err(CssColorProtocolError::ForeignSlot.into());
        }
        self.operation
            .numerical
            .as_mut()
            .expect("owned serialization")
            .next_action()
            .written(index, slot)
            .map_err(|e| CssColorProtocolError::NumericalProtocol(e).into())
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn emitted(self, unit: u8) -> Result<(), CssColorOperationError> {
        self.operation
            .numerical
            .as_mut()
            .expect("owned serialization")
            .next_action()
            .emitted(unit)
            .map_err(|e| CssColorProtocolError::NumericalProtocol(e).into())
    }
}
pub struct CssColorSerializationReceipt<'css, 'facts> {
    checkpoint: core::UsedCheckpoint<'css, 'facts>,
    length: usize,
}
impl CssColorSerializationReceipt<'_, '_> {
    pub fn value(&self) -> Color {
        self.checkpoint.value()
    }
    pub const fn length(&self) -> usize {
        self.length
    }
    pub fn source(&self) -> CssColorSourceDescriptor {
        source(&self.checkpoint.context)
    }
    pub fn standard_failure(&self) -> Option<CssColorStandardFailure> {
        self.checkpoint.standard_failure.map(failure)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact};
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
    #[expect(
        clippy::result_large_err,
        reason = "The controlled source fixture propagates the common owned path and origin error inline without allocation or cloning"
    )]
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
                CssColorActionDescription::ReadSlot { .. } => {
                    assert_eq!(actual.slot_read_bytes, CssColorSlot::record_bytes())
                }
                CssColorActionDescription::AppendSlot { .. }
                | CssColorActionDescription::UpdateSlot { .. } => {
                    assert_eq!(actual.slot_read_bytes, CssColorSlot::record_bytes());
                    assert_eq!(actual.slot_write_bytes, CssColorSlot::record_bytes());
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
        let outcome = CssColorOperation::take(&mut Some(operation))?;
        // This ordinary owner clears its vectors. It does not simulate real
        // HTML capacity/ticket cleanup or claim foreign-controller admission.
        banks.iter_mut().for_each(Vec::clear);
        drop(banks);
        Ok((outcome, folds))
    }
    fn serialize(
        mut serializer: CssColorSerialization<'_, '_, '_>,
    ) -> (
        String,
        Color,
        Option<CssColorSourceDescriptor>,
        Option<CssColorStandardFailure>,
    ) {
        let mut digits = Vec::new();
        let mut output = Vec::new();
        loop {
            let mut action = serializer.next_action();
            match action.description() {
                CssColorActionDescription::Compute(_) => action.compute().unwrap(),
                CssColorActionDescription::ReadNumericalDigit { index } => {
                    action.read_numerical_digit(index, &digits[index]).unwrap()
                }
                CssColorActionDescription::AppendNumericalDigit { index } => {
                    assert_eq!(index, digits.len());
                    digits.push(action.numerical_record().unwrap());
                    action.numerical_written(index, &digits[index]).unwrap();
                }
                CssColorActionDescription::UpdateNumericalDigit { index } => {
                    digits[index] = action.numerical_record().unwrap();
                    action.numerical_written(index, &digits[index]).unwrap();
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
        let receipt = CssColorSerialization::take(&mut Some(serializer)).unwrap();
        digits.clear();
        drop(digits);
        (
            String::from_utf8(output).unwrap(),
            receipt.value(),
            Some(receipt.source()),
            receipt.standard_failure(),
        )
    }
    #[test]
    fn refused_parse_take_and_external_numerical_owner_reuse_preserve_source() {
        let facts = facts();
        let mut css_owner = CssColorOperationOwner::new();
        {
            let mut operation = Some(
                CssColorOperation::parse_used(
                    &mut css_owner,
                    CssColorInputKind::Utf8,
                    3,
                    CssColorGrammarRequest::WholeColor,
                    &facts,
                    facts.identity(),
                )
                .unwrap(),
            );
            let pending = operation.as_ref().unwrap().action();
            assert!(matches!(
                CssColorOperation::take(&mut operation),
                Err(CssColorProtocolError::Terminal)
            ));
            assert_eq!(operation.as_ref().unwrap().action(), pending);
            operation.as_mut().unwrap().abandon();
            assert!(matches!(
                CssColorOperation::take(&mut operation),
                Err(CssColorProtocolError::Terminal)
            ));
            assert!(operation.is_some());
        }
        let (outcome, _) = parse(&mut css_owner, "red", &facts).unwrap();
        let CssColorParseOutcome::Used(checkpoint) = outcome else {
            panic!("red")
        };
        let source = checkpoint.source();
        let mut numerical_owner = surgeist_color::OperationOwner::new();
        let mut slot = Some(checkpoint);
        assert_eq!(
            CssUsedColorCheckpoint::serialize(
                &mut slot,
                &mut numerical_owner,
                SerializationRequest::Css,
                CssColorBankLengths::new(0, 0, 0),
                1
            )
            .err(),
            Some(CssColorOperationError::Protocol(
                CssColorProtocolError::NumericalProtocol(
                    surgeist_color::ProtocolError::StorageNotEmpty
                )
            ))
        );
        assert_eq!(slot.as_ref().unwrap().source(), source);
        let mut transform = Some(
            CssUsedColorCheckpoint::transform(
                &mut slot,
                &mut numerical_owner,
                Transformation::ForceOpaqueAlpha,
                CssColorBankLengths::new(0, 0, 0),
            )
            .unwrap(),
        );
        assert!(CssColorTransformation::take(&mut transform).is_err());
        assert!(transform.is_some());
        transform.as_mut().unwrap().next_action().compute().unwrap();
        let checkpoint = CssColorTransformation::take(&mut transform).unwrap();
        assert_eq!(checkpoint.source(), source);
        let mut slot = Some(checkpoint);
        let serializer = CssUsedColorCheckpoint::serialize(
            &mut slot,
            &mut numerical_owner,
            SerializationRequest::HtmlCompatible,
            CssColorBankLengths::new(0, 0, 0),
            0,
        )
        .unwrap();
        let (output, _, receipt_source, _) = serialize(serializer);
        assert_eq!(output, "#ff0000");
        assert_eq!(receipt_source, Some(source));
        // Source metadata survives after the numerical owner borrow has ended.
        let request = surgeist_color::NumericalRequest::Construct {
            space: crate::ColorSpace::Lab,
            channels: [None; 3],
            alpha: None,
        };
        let mut native =
            Some(surgeist_color::Computation::new(&mut numerical_owner, request).unwrap());
        native.as_mut().unwrap().next_action().compute().unwrap();
        assert_eq!(
            surgeist_color::Computation::take(&mut native)
                .unwrap()
                .alpha(),
            None
        );
        assert_eq!(receipt_source, Some(source));
    }

    #[test]
    fn public_guard_drives_actual_banks_and_exposes_three_folds_per_argument() {
        let mut numerical_owner_0 = surgeist_color::OperationOwner::new();

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
            &mut numerical_owner_0,
            SerializationRequest::Css,
            CssColorBankLengths::new(0, 0, 0),
            0,
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
        let mut numerical_owner_2 = surgeist_color::OperationOwner::new();
        let mut numerical_owner_1 = surgeist_color::OperationOwner::new();
        let mut numerical_owner_0 = surgeist_color::OperationOwner::new();

        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let (outcome, _) = parse(&mut owner, "rgb(1deg 0 0)", &facts).unwrap();
        let CssColorParseOutcome::StandardFailure(failed) = outcome else {
            panic!("degree is not an RGB scalar")
        };
        let expected = failed.failure();
        let checkpoint = failed.fallback(Color::opaque_black());
        let mut slot = Some(checkpoint);
        assert_eq!(
            CssUsedColorCheckpoint::transform(
                &mut slot,
                &mut numerical_owner_0,
                Transformation::ForceOpaqueAlpha,
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
            &mut numerical_owner_1,
            Transformation::ForceOpaqueAlpha,
            CssColorBankLengths::new(0, 0, 0),
        )
        .unwrap();
        assert!(slot.is_none());
        assert_eq!(
            transform.next_action().description(),
            CssColorActionDescription::Compute(CssColorKernel::Numerical(
                surgeist_color::Kernel::Transform(Transformation::ForceOpaqueAlpha)
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
                    &mut numerical_owner_2,
                    SerializationRequest::HtmlCompatible,
                    CssColorBankLengths::new(0, 0, 0),
                    0
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
        assert!(CssColorOperation::take(&mut Some(second)).is_err());
        first.abandon();
        assert!(CssColorOperation::take(&mut Some(first)).is_err());
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
            CssColorSlot::record_bytes(),
            CssColorOperation::register_bytes(),
            CssColorOperation::action_bytes(),
            std::mem::size_of::<CssUsedColorCheckpoint<'_, '_>>(),
            std::mem::size_of::<Option<CssUsedColorCheckpoint<'_, '_>>>(),
            CssColorTransformation::register_bytes(),
            std::mem::size_of::<Option<CssColorTransformation<'_, '_, '_>>>(),
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
        let CssColorParseOutcome::Used(value) =
            CssColorOperation::take(&mut Some(operation)).unwrap()
        else {
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
        let mut numerical_owner_1 = surgeist_color::OperationOwner::new();
        let mut numerical_owner_0 = surgeist_color::OperationOwner::new();

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
                &mut numerical_owner_0,
                SerializationRequest::Css,
                CssColorBankLengths::new(0, 0, 0),
                0,
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
                &mut numerical_owner_1,
                SerializationRequest::Css,
                CssColorBankLengths::new(0, 0, 0),
                0,
            )
            .ok()
            .unwrap();
            assert_eq!(serialize(serializer).0, expected, "{text}");
        }
    }

    #[test]
    fn every_real_bank_refuses_wrong_replies_and_retired_slots_cannot_cross_phase() {
        let mut numerical_owner_0 = surgeist_color::OperationOwner::new();

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
        let CssColorParseOutcome::Used(checkpoint) =
            CssColorOperation::take(&mut Some(operation)).unwrap()
        else {
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
            &mut numerical_owner_0,
            SerializationRequest::Css,
            CssColorBankLengths::new(0, 0, 0),
            0,
        )
        .ok()
        .unwrap();
        let mut foreign_owner = surgeist_color::OperationOwner::new();
        let foreign_digit = {
            let mut f = surgeist_color::format::FiniteFormatter::new(
                &mut foreign_owner,
                1.0,
                surgeist_color::format::RoundingMode::AwayFromZero,
            )
            .unwrap();
            loop {
                let mut a = f.next_action();
                match a.description() {
                    surgeist_color::ActionDescription::Compute(_) => a.compute().unwrap(),
                    surgeist_color::ActionDescription::AppendDigit { .. } => {
                        break a.record().unwrap();
                    }
                    other => panic!("unexpected {other:?}"),
                }
            }
        };
        let mut digits = Vec::new();
        let mut output = Vec::new();
        let mut refused = false;
        loop {
            let pending = serializer.action();
            let mut action = serializer.next_action();
            match pending {
                CssColorActionDescription::Compute(_) => action.compute().unwrap(),
                CssColorActionDescription::ReadNumericalDigit { index } => {
                    if !refused {
                        assert_eq!(
                            action.read_numerical_digit(index, &foreign_digit),
                            Err(CssColorOperationError::Protocol(
                                CssColorProtocolError::NumericalProtocol(
                                    surgeist_color::ProtocolError::ForeignSlot
                                )
                            ))
                        );
                        assert_eq!(serializer.action(), pending);
                        serializer
                            .next_action()
                            .read_numerical_digit(index, &digits[index])
                            .unwrap();
                        refused = true;
                    } else {
                        action.read_numerical_digit(index, &digits[index]).unwrap();
                    }
                }
                CssColorActionDescription::AppendNumericalDigit { index } => {
                    assert_eq!(index, digits.len());
                    digits.push(action.numerical_record().unwrap());
                    action.numerical_written(index, &digits[index]).unwrap();
                }
                CssColorActionDescription::UpdateNumericalDigit { index } => {
                    digits[index] = action.numerical_record().unwrap();
                    action.numerical_written(index, &digits[index]).unwrap();
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
        assert_eq!(
            CssColorSerialization::take(&mut Some(serializer))
                .unwrap()
                .length(),
            output.len()
        );
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
        let mut numerical_owner = surgeist_color::OperationOwner::new();
        let mut transform = CssUsedColorCheckpoint::transform(
            &mut slot,
            &mut numerical_owner,
            Transformation::QuantizeSrgb,
            CssColorBankLengths::new(0, 0, 0),
        )
        .unwrap();
        transform.next_action().compute().unwrap();
        transform.abandon();
        let recovered = CssColorTransformation::recover(&mut Some(transform)).unwrap();
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
        let mut numerical_owner_7 = surgeist_color::OperationOwner::new();
        let mut numerical_owner_6 = surgeist_color::OperationOwner::new();
        let mut numerical_owner_5 = surgeist_color::OperationOwner::new();
        let mut numerical_owner_4 = surgeist_color::OperationOwner::new();
        let mut numerical_owner_3 = surgeist_color::OperationOwner::new();
        let mut numerical_owner_2 = surgeist_color::OperationOwner::new();
        let mut numerical_owner_1 = surgeist_color::OperationOwner::new();
        let mut numerical_owner_0 = surgeist_color::OperationOwner::new();

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
                &mut numerical_owner_0,
                Transformation::ForceOpaqueAlpha,
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
            CssUsedColorCheckpoint::serialize(
                &mut slot,
                &mut numerical_owner_1,
                SerializationRequest::Css,
                banks,
                0
            )
            .err(),
            Some(CssColorOperationError::Protocol(
                CssColorProtocolError::StorageNotEmpty
            ))
        );
        assert_eq!(slot.as_ref().unwrap().source(), source);
        assert_eq!(slot.as_ref().unwrap().value(), original);
        let transformation = CssUsedColorCheckpoint::transform(
            &mut slot,
            &mut numerical_owner_2,
            Transformation::ForceOpaqueAlpha,
            CssColorBankLengths::new(0, 0, 0),
        )
        .unwrap();
        assert!(slot.is_none());
        assert_eq!(
            CssUsedColorCheckpoint::transform(
                &mut slot,
                &mut numerical_owner_3,
                Transformation::ForceOpaqueAlpha,
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
                CssColorProtocolError::NumericalProtocol(surgeist_color::ProtocolError::NotReady)
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
        // Computation remains unpublished until take; abandonment recovers
        // the previous committed floating checkpoint and its source context.
        let mut transform = CssUsedColorCheckpoint::transform(
            &mut slot,
            &mut numerical_owner_4,
            Transformation::QuantizeSrgb,
            CssColorBankLengths::new(0, 0, 0),
        )
        .unwrap();
        transform.next_action().compute().unwrap();
        transform.abandon();
        let mut transform = Some(transform);
        assert_eq!(
            CssColorTransformation::take(&mut transform).err(),
            Some(CssColorOperationError::Protocol(
                CssColorProtocolError::NumericalProtocol(surgeist_color::ProtocolError::Terminated)
            ))
        );
        let checkpoint = CssColorTransformation::recover(&mut transform).unwrap();
        assert_eq!(checkpoint.value().srgb_bytes(), None); // Untaken quantization does not commit byte backing.
        assert_eq!(checkpoint.value().alpha(), Some(1.0));
        assert_eq!(checkpoint.value().alpha_byte(), None);
        assert_eq!(checkpoint.source(), source);
        let mut slot = Some(checkpoint);
        let serializer = CssUsedColorCheckpoint::serialize(
            &mut slot,
            &mut numerical_owner_5,
            SerializationRequest::HtmlCompatible,
            CssColorBankLengths::new(0, 0, 0),
            0,
        )
        .unwrap();
        assert!(slot.is_none());
        assert_eq!(
            CssUsedColorCheckpoint::serialize(
                &mut slot,
                &mut numerical_owner_6,
                SerializationRequest::Css,
                CssColorBankLengths::new(0, 0, 0),
                0
            )
            .err(),
            Some(CssColorOperationError::Protocol(
                CssColorProtocolError::EmptySlot
            ))
        );
        assert_eq!(serialize(serializer).0, "rgb(255, 0, 0)");

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
            &mut numerical_owner_7,
            Transformation::Convert(crate::ColorSpace::Predefined(
                crate::PredefinedColorSpace::XyzD65,
            )),
            CssColorBankLengths::new(0, 0, 0),
        )
        .unwrap();
        assert_eq!(
            transform.next_action().compute(),
            Err(CssColorOperationError::Numerical {
                error: ColorError::Unrepresentable,
                site: CssColorResolutionSite::Source(CssColorInputRange {
                    start: 0,
                    end: "color(srgb-linear 1.7e308 1.7e308 1.7e308)".len()
                })
            })
        );
        transform.abandon();
        let checkpoint = CssColorTransformation::recover(&mut Some(transform)).unwrap();
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
                            CssColorOperation::take(&mut Some(operation)),
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
                        let CssColorParseOutcome::Used(value) =
                            CssColorOperation::take(&mut Some(operation)).unwrap()
                        else {
                            panic!("used checkpoint")
                        };
                        assert_eq!(
                            value.value().space(),
                            crate::ColorSpace::Predefined(crate::PredefinedColorSpace::Srgb)
                        );
                        assert_eq!(value.value().channels(), [Some(1.0), Some(0.0), Some(0.0)]);
                        assert!(
                            !(value.value().serialization_family()
                                == surgeist_color::SerializationFamily::ColorFunction)
                        );
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
