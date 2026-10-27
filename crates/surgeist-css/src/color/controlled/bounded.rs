use super::CssColorKernel;
use super::protocol::ColorProtocolError;
use crate::color::bounded::{ColorErrorKind, source};
use crate::color::controlled::{CssColorBank, CssColorOperationOwner, CssColorSlot};
use crate::color::operation as core;
use crate::color::{
    CssColorError, CssColorFactDefect, CssColorGrammarRequest, CssColorInputKind,
    CssColorInputRange, CssColorSourceDescriptor, CssColorStandardFailure,
    CssColorStandardFailureKind, CssColorUnavailable, CssColorUnsupported,
};
use crate::color::{CssColorFactIdentity, CssColorFacts};
use surgeist_color::Color;
use surgeist_color::Transformation;

/// Classifies a canonical-core failure as a reached semantic failure, or
/// returns the protocol inconsistency that a defect in the driver produced.
pub(crate) fn classify(error: core::Error) -> Result<CssColorError, ColorProtocolError> {
    classify_kind(error).map(CssColorError::from)
}

fn classify_kind(error: core::Error) -> Result<ColorErrorKind, ColorProtocolError> {
    use crate::color::component_value::Error as V;
    use crate::color::device::DeviceError as D;
    use crate::color::interpolate::Error as I;
    use crate::color::palette::PaletteError as P;
    use crate::color::profile::ComponentNameError as N;
    use crate::color::profile::ProfileError as H;
    use crate::color::profile_conversion::Error as C;
    use crate::color::resolution::Error as R;
    match error {
        core::Error::Protocol(ColorProtocolError::Malformed { offset }) => {
            Ok(ColorErrorKind::MalformedInput { offset })
        }
        core::Error::Protocol(error) => Err(error),
        core::Error::Resolution(error) => Ok(match error {
            R::Protocol(error)
            | R::Component(V::Protocol(error))
            | R::Profile(H::Protocol(error))
            | R::ProfileComponent(N::Protocol(error))
            | R::Conversion(C::Protocol(error)) => return Err(error),
            R::MissingScheme => ColorErrorKind::Unavailable(CssColorUnavailable::Scheme),
            R::ForeignFacts
            | R::Palette(P::ForeignFacts)
            | R::Profile(H::ForeignFacts)
            | R::ProfileComponent(N::ForeignFacts)
            | R::Conversion(C::ForeignFacts)
            | R::Device(D::ForeignFacts) => ColorErrorKind::Facts(CssColorFactDefect::Foreign),
            R::Palette(P::MissingCurrent) => {
                ColorErrorKind::Unavailable(CssColorUnavailable::CurrentColor)
            }
            R::Palette(P::MissingPalette(keyword)) => {
                ColorErrorKind::Unavailable(CssColorUnavailable::Palette(keyword))
            }
            R::Palette(P::Duplicate(keyword)) => {
                ColorErrorKind::Facts(CssColorFactDefect::DuplicatePalette(keyword))
            }
            R::Profile(H::Missing) | R::ProfileComponent(N::Unavailable) => {
                ColorErrorKind::Unavailable(CssColorUnavailable::Profile)
            }
            R::Profile(H::Duplicate { first, second }) => {
                ColorErrorKind::Facts(CssColorFactDefect::DuplicateProfile { first, second })
            }
            R::Profile(H::InvalidMetadata)
            | R::ProfileComponent(N::InvalidMetadata)
            | R::Conversion(C::InvalidMetadata) => {
                ColorErrorKind::Facts(CssColorFactDefect::InvalidProfileMetadata)
            }
            R::Conversion(C::Unavailable) => {
                ColorErrorKind::Unavailable(CssColorUnavailable::ProfileConversion)
            }
            R::Conversion(C::Unsupported) => {
                ColorErrorKind::Unsupported(CssColorUnsupported::ProfileConversion)
            }
            R::ReverseProfileUnsupported | R::Component(V::ReverseCustomOrigin { .. }) => {
                ColorErrorKind::Unsupported(CssColorUnsupported::ReverseProfile)
            }
            R::Conversion(C::DuplicateConversion) => {
                ColorErrorKind::Facts(CssColorFactDefect::DuplicateConversion)
            }
            R::Device(D::Unavailable) => ColorErrorKind::Unavailable(CssColorUnavailable::Device),
            R::Device(D::ConversionMismatch | D::InvalidFacts) => {
                ColorErrorKind::Facts(CssColorFactDefect::InvalidDevice)
            }
            R::CustomInterpolationUnsupported => {
                ColorErrorKind::Unsupported(CssColorUnsupported::CustomInterpolation)
            }
            R::Component(V::UnitBasis { unit, .. }) => {
                ColorErrorKind::Unsupported(CssColorUnsupported::ContextualUnit(unit))
            }
            // The canonical core converts these standard-invalid branches
            // into a standard-failure outcome before any error leaves it.
            R::Channel(crate::color::channels::Error::Grammar) => {
                panic!("the core converts an invalid resolved channel into a standard failure")
            }
            R::Interpolation(I::InvalidWeight) => {
                panic!("the core converts an invalid mix weight into a standard failure")
            }
            R::Conversion(C::InvalidColor) => {
                panic!("the core converts an invalid profile color into a standard failure")
            }
            R::ProfileComponent(N::UnknownChannel | N::InvalidProfile) => {
                panic!("the core converts an invalid profile component into a standard failure")
            }
            R::Interpolation(I::Overflow) => {
                panic!("the core reports a mix counter overflow as a protocol overflow")
            }
            // Grammar and syntax validation precede resolution.
            R::Interpolation(I::Empty) => {
                panic!("a grammar-checked color-mix() has at least one component")
            }
            R::Profile(H::InvalidName) => {
                panic!("a grammar-checked custom profile name is a dashed identifier")
            }
            R::Component(V::Channel(_)) => {
                panic!("a grammar-checked relative channel keyword names a channel of its space")
            }
            R::Component(V::MissingOrigin) => {
                panic!("a relative channel is resolved only after its origin is bound")
            }
            // The core issues only the pending action of a live sub-machine.
            R::Palette(P::WrongAction) | R::Device(D::WrongAction) => {
                panic!("the core completes only the fact read its sub-machine requested")
            }
            R::Palette(P::Stopped)
            | R::Profile(H::Stopped)
            | R::ProfileComponent(N::Stopped)
            | R::Conversion(C::Stopped)
            | R::Device(D::Stopped) => {
                panic!("the core never advances a stopped resolution sub-machine")
            }
        }),
    }
}
/// A reached semantic failure; a protocol inconsistency is a defect in the
/// code driving the parse and panics (decision 5 of #1353).
#[track_caller]
fn reached(error: core::Error) -> CssColorError {
    match classify(error) {
        Ok(error) => error,
        Err(defect) => defect.raise(),
    }
}
/// One actual fixed fact observation, not a hidden array lookup/scan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorFactRead {
    /// The property facts, which supply the current color.
    Properties,
    /// The number of entries in the system color palette.
    PaletteLength,
    /// One system color palette entry.
    PaletteEntry {
        /// Zero-based index of the palette entry.
        index: usize,
        /// Whether the entry is compared as an earlier entry in a duplicate check,
        /// rather than read as the lookup candidate.
        earlier: bool,
    },
    /// The used color scheme.
    Scheme,
    /// The number of custom color profile bindings.
    ProfilesLength,
    /// The header of one profile binding.
    ProfileHeader {
        /// Zero-based index of the profile binding.
        binding: usize,
    },
    /// One byte of a profile binding's name.
    ProfileNameByte {
        /// Zero-based index of the profile binding.
        binding: usize,
        /// Zero-based index of the byte within the profile name.
        index: usize,
    },
    /// The header of the profile binding already selected for the color.
    SelectedProfileHeader,
    /// The header of one component name of the selected profile.
    ComponentNameHeader {
        /// Zero-based position of the name in the profile's ordered component names.
        ordinal: usize,
    },
    /// One byte of a component name of the selected profile.
    ComponentNameByte {
        /// Zero-based position of the name in the profile's ordered component names.
        ordinal: usize,
        /// Zero-based index of the byte within the component name.
        index: usize,
    },
    /// The number of profile conversions.
    ConversionLength,
    /// The header of one profile conversion.
    ConversionHeader {
        /// Zero-based index of the profile conversion.
        ordinal: usize,
    },
    /// One input channel of a resolved profile conversion.
    ConversionChannel {
        /// Zero-based index of the profile conversion.
        conversion: usize,
        /// Zero-based index of the channel within the conversion.
        ordinal: usize,
    },
    /// The `device-cmyk()` processing fact.
    DeviceHeader,
    /// One input channel of the resolved `device-cmyk()` processing record.
    DeviceChannel {
        /// Zero-based index of the channel.
        index: usize,
    },
}
/// The canonical core's pending work, classified once for every driver: the
/// public parse and the crate-private authored protocol share it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Pending {
    ReadInput {
        offset: usize,
    },
    ReadSlot {
        bank: CssColorBank,
        index: usize,
    },
    AppendSlot {
        bank: CssColorBank,
        index: usize,
    },
    UpdateSlot {
        bank: CssColorBank,
        index: usize,
    },
    ReadFact(CssColorFactRead),
    Compute(CssColorKernel),
    /// A used value or a standard failure is ready.
    Complete,
    Stopped,
}
pub(crate) fn pending(cursor: &core::ColorOperation<'_, '_>) -> Pending {
    use crate::color::device::DeviceAction as D;
    use crate::color::palette::PaletteAction as P;
    use crate::color::profile::ComponentNameAction as N;
    use crate::color::profile::ProfileAction as H;
    use crate::color::profile_conversion::ConversionAction as C;
    use crate::color::profile_conversion::InputAction as I;
    use crate::color::resolution::Action as R;
    use core::{Action as A, StorageAction as S};
    if let Some(storage) = cursor.action().storage() {
        return match storage {
            S::Read { bank, index } => Pending::ReadSlot { bank, index },
            S::Append { bank, index } => Pending::AppendSlot { bank, index },
            S::Replace { bank, index } => Pending::UpdateSlot { bank, index },
        };
    }
    let fact = match cursor.action() {
        A::Lexical(super::protocol::ActionDescription::ReadInput { offset }) => {
            return Pending::ReadInput { offset };
        }
        A::UsedReady | A::StandardFailed => return Pending::Complete,
        A::Stopped => return Pending::Stopped,
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
    fact.or_else(|| cursor.additional_fact_read())
        .map_or_else(|| Pending::Compute(cursor.kernel()), Pending::ReadFact)
}
/// Actual fixed records/units which a fact action can inspect. Zero entries
/// are absent observations. These are compiled record layouts, not lengths of
/// referenced names, channels or arrays; those units have separate actions.
impl CssColorFactRead {
    /// Returns the sizes in bytes of the fixed records this observation can inspect, as
    /// `[identity, record, nested record]`; a zero entry means no such observation.
    #[must_use]
    pub fn record_layouts(self) -> [usize; 3] {
        use std::mem::size_of;
        let identity = size_of::<CssColorFactIdentity>();
        let profile = size_of::<crate::color::CssColorProfileBinding<'_>>();
        let conversion = size_of::<crate::color::CssColorProfileConversion<'_>>();
        match self {
            Self::Properties => [
                identity,
                size_of::<crate::color::CssColorPropertyFacts>(),
                0,
            ],
            Self::Scheme => [identity, size_of::<crate::color::CssColorSchemeFact>(), 0],
            Self::PaletteLength | Self::ProfilesLength => [identity, size_of::<usize>(), 0],
            Self::PaletteEntry { .. } => {
                [identity, size_of::<crate::color::CssColorPaletteEntry>(), 0]
            }
            Self::ProfileHeader { .. } | Self::SelectedProfileHeader => [identity, profile, 0],
            Self::ProfileNameByte { .. } => [identity, profile, size_of::<u8>()],
            Self::ComponentNameHeader { .. } => [
                identity,
                profile,
                size_of::<crate::color::CssColorProfileComponentName>(),
            ],
            Self::ComponentNameByte { .. } => [
                identity,
                profile,
                size_of::<crate::color::CssColorProfileComponentName>() + size_of::<u8>(),
            ],
            Self::ConversionLength => [identity, profile, size_of::<usize>()],
            Self::ConversionHeader { .. } => [identity, conversion, 0],
            Self::ConversionChannel { .. } => [identity, conversion, size_of::<f64>()],
            Self::DeviceHeader => [
                identity,
                size_of::<crate::color::CssDeviceColorFact<'_>>(),
                0,
            ],
            Self::DeviceChannel { .. } => [
                identity,
                size_of::<crate::color::CssDeviceColorFact<'_>>(),
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
    /// Size in bytes of the original input unit the action reads; zero when it reads none.
    pub input_unit_bytes: usize,
    /// Size in bytes of the bank record the action reads back; zero when it reads none.
    pub slot_read_bytes: usize,
    /// Size in bytes of the bank record the action writes; zero when it writes none.
    pub slot_write_bytes: usize,
    /// Sizes in bytes of the fixed fact records the action inspects, as returned by `CssColorFactRead::record_layouts`.
    pub fact_records: [usize; 3],
    /// Size in bytes of the output the numerical computation writes; zero for other actions.
    pub output_unit_bytes: usize,
    /// Size in bytes the numerical computation reads; zero for other actions.
    pub numerical_read_bytes: usize,
    /// Size in bytes the numerical computation writes; zero for other actions.
    pub numerical_write_bytes: usize,
    /// The kernel the action executes.
    pub kernel: CssColorKernel,
}
pub(crate) fn layout(
    pending: Pending,
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
    match pending {
        Pending::ReadInput { .. } => {
            layout.input_unit_bytes = match kind {
                CssColorInputKind::Utf8 => size_of::<u8>(),
                CssColorInputKind::Utf16 => size_of::<u16>(),
            }
        }
        Pending::ReadSlot { .. } => layout.slot_read_bytes = CssColorSlot::record_bytes(),
        Pending::AppendSlot { .. } | Pending::UpdateSlot { .. } => {
            // Generated opaque payload plus actual stored acknowledgement.
            layout.slot_write_bytes = CssColorSlot::record_bytes();
            layout.slot_read_bytes = CssColorSlot::record_bytes();
        }
        Pending::ReadFact(fact) => layout.fact_records = fact.record_layouts(),
        Pending::Compute(_) | Pending::Complete | Pending::Stopped => {}
    }
    layout
}
/// Actual fixed initialization observations and continuation layout. The
/// original consumer owns admission and any actual backing allocation/copy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorInitializationLayout {
    /// Size in bytes of the operation owner anchor.
    pub owner_bytes: usize,
    /// Size in bytes of the fixed parse registers.
    pub register_bytes: usize,
    /// Size in bytes of one fact-set identity.
    pub fact_identity_bytes: usize,
}
/// The pending parse of one color. It never contains a completed outcome.
///
/// [`step`](Self::step) consumes the parse and yields either its single next
/// action or its outcome; the action's one completion method returns the
/// parse. A semantic failure ([`CssColorError`]) consumes it, and dropping a
/// parse or an action abandons it. Caller slots stay caller-owned and are
/// released by the caller; nothing executes abandoned work or refunds storage.
/// The outcome no longer borrows the owner, so the owner can serve another
/// parse once the caller has dropped the earlier parse's records.
///
/// ```
/// use surgeist_css::color::controlled::{CssColorBank, CssColorOperationOwner, CssColorParse, CssColorParseAction, CssColorParseOutcome, CssColorParseStep};
/// use surgeist_css::color::{CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest, CssColorInputKind, CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact};
/// let facts = CssColorFacts::new(
///     CssColorFactIdentity::new(1, 1),
///     CssColorPropertyFacts::Unavailable,
///     CssColorSchemeFact::Unavailable,
///     &[],
///     &[],
///     &[],
///     CssDeviceColorFact::Unknown,
/// );
/// let ordinal = |bank| match bank {
///     CssColorBank::Algorithm => 0,
///     CssColorBank::Arithmetic => 1,
///     CssColorBank::Evaluation => 2,
/// };
/// let input = b"red";
/// let mut owner = CssColorOperationOwner::new();
/// let mut banks: [Vec<_>; 3] = Default::default();
/// let mut parse = CssColorParse::start(
///     &mut owner,
///     CssColorInputKind::Utf8,
///     input.len(),
///     CssColorGrammarRequest::WholeColor,
///     &facts,
///     facts.identity(),
/// )?;
/// let outcome = loop {
///     let action = match parse.step() {
///         CssColorParseStep::Complete(outcome) => break outcome,
///         CssColorParseStep::Pending(action) => action,
///     };
///     // A consumer admits `action.layout()` here; refusal drops the action.
///     parse = match action {
///         CssColorParseAction::ReadInput(read) => {
///             let unit = u16::from(input[read.offset()]);
///             read.read(unit)?
///         }
///         CssColorParseAction::ReadSlot(read) => {
///             let slot = &banks[ordinal(read.bank())][read.index()];
///             read.read(slot)?
///         }
///         CssColorParseAction::AppendSlot(write) => {
///             let target = &mut banks[ordinal(write.bank())];
///             let (record, receipt) = write.record();
///             target.push(record);
///             receipt.stored(&target[target.len() - 1])
///         }
///         CssColorParseAction::UpdateSlot(write) => {
///             let index = write.index();
///             let target = &mut banks[ordinal(write.bank())];
///             let (record, receipt) = write.record();
///             target[index] = record;
///             receipt.stored(&target[index])
///         }
///         CssColorParseAction::ReadFact(read) => read.read()?,
///         CssColorParseAction::Compute(compute) => compute.compute()?,
///     };
/// };
/// // Dropping the records frees the owner while the outcome lives on.
/// drop(banks);
/// let next = CssColorParse::start(
///     &mut owner,
///     CssColorInputKind::Utf8,
///     0,
///     CssColorGrammarRequest::WholeColor,
///     &facts,
///     facts.identity(),
/// )?;
/// drop(next);
/// let CssColorParseOutcome::Used(checkpoint) = outcome else {
///     panic!("red is a used color")
/// };
/// assert_eq!(checkpoint.value().srgb_bytes(), Some([255, 0, 0]));
/// # Ok::<(), surgeist_css::color::CssColorError>(())
/// ```
///
/// Misuse that the typestate rejects at compile time (#1353, 5.6):
///
/// A parse is consumed by its step (CF1):
/// ```compile_fail,E0382
/// fn step_twice(parse: surgeist_css::color::controlled::CssColorParse<'_, '_>) {
///     let _ = parse.step();
///     let _ = parse.step();
/// }
/// ```
/// A checkpoint is consumed by its transformation (CF2):
/// ```compile_fail,E0382
/// use surgeist_color::{OperationOwner, Transformation};
/// fn reuse(checkpoint: surgeist_css::color::controlled::CssUsedColorCheckpoint<'_>, owner: &mut OperationOwner) {
///     let _ = checkpoint.transform(owner, Transformation::ForceOpaqueAlpha);
///     let _ = checkpoint.value();
/// }
/// ```
/// No outcome is taken from a pending parse (CF3):
/// ```compile_fail,E0599
/// use surgeist_css::color::controlled::CssColorParse;
/// fn take(parse: CssColorParse<'_, '_>) {
///     let _ = CssColorParse::take(&mut Some(parse));
/// }
/// ```
/// An abandoned parse cannot step (CF4):
/// ```compile_fail,E0382
/// fn abandoned(parse: surgeist_css::color::controlled::CssColorParse<'_, '_>) {
///     drop(parse);
///     let _ = parse.step();
/// }
/// ```
/// Each action exposes only its own completion (CF5):
/// ```compile_fail,E0599
/// fn wrong_completion(compute: surgeist_css::color::controlled::CssColorCompute<'_, '_>) {
///     let _ = compute.read(0);
/// }
/// ```
/// One write mints one record (CF6):
/// ```compile_fail,E0382
/// fn mint_twice(write: surgeist_css::color::controlled::CssColorWriteSlot<'_, '_>) {
///     let _ = write.record();
///     let _ = write.record();
/// }
/// ```
/// A transformation computes once (CF7):
/// ```compile_fail,E0382
/// fn compute_twice(transform: surgeist_css::color::controlled::CssColorTransform<'_, '_>) {
///     let _ = transform.compute();
///     let _ = transform.compute();
/// }
/// ```
/// Only a used checkpoint transforms (CF8):
/// ```compile_fail,E0599
/// use surgeist_color::{OperationOwner, Transformation};
/// fn failed(checkpoint: surgeist_css::color::controlled::CssColorFailureCheckpoint<'_>, owner: &mut OperationOwner) {
///     let _ = checkpoint.transform(owner, Transformation::ForceOpaqueAlpha);
/// }
/// ```
/// A record keeps its owner from starting another parse (CF9):
/// ```compile_fail,E0499
/// use surgeist_css::color::{CssColorFacts, CssColorGrammarRequest, CssColorInputKind};
/// use surgeist_css::color::controlled::{CssColorOperationOwner, CssColorParse, CssColorParseAction, CssColorParseStep};
/// fn second<'f>(owner: &mut CssColorOperationOwner, facts: &'f CssColorFacts<'f>) {
///     let first = CssColorParse::start(
///         owner,
///         CssColorInputKind::Utf8,
///         1,
///         CssColorGrammarRequest::WholeColor,
///         facts,
///         facts.identity(),
///     )
///     .unwrap();
///     let CssColorParseStep::Pending(CssColorParseAction::AppendSlot(write)) = first.step() else {
///         return;
///     };
///     let (slot, receipt) = write.record();
///     drop(receipt);
///     let _second = CssColorParse::start(
///         owner,
///         CssColorInputKind::Utf8,
///         1,
///         CssColorGrammarRequest::WholeColor,
///         facts,
///         facts.identity(),
///     );
///     drop(slot);
/// }
/// ```
/// Only CSS mints a record (CF10):
/// ```compile_fail,E0451
/// let _ = surgeist_css::color::controlled::CssColorSlot { index: 0 };
/// ```
/// A record cannot outlive its owner (CF16):
/// ```compile_fail,E0597
/// use surgeist_css::color::{CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest, CssColorInputKind, CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact};
/// use surgeist_css::color::controlled::{CssColorOperationOwner, CssColorParse, CssColorParseAction, CssColorParseStep};
/// let facts = CssColorFacts::new(
///     CssColorFactIdentity::new(1, 1),
///     CssColorPropertyFacts::Unavailable,
///     CssColorSchemeFact::Unavailable,
///     &[],
///     &[],
///     &[],
///     CssDeviceColorFact::Unknown,
/// );
/// let slot = {
///     let mut owner = CssColorOperationOwner::new();
///     let parse = CssColorParse::start(
///         &mut owner,
///         CssColorInputKind::Utf8,
///         1,
///         CssColorGrammarRequest::WholeColor,
///         &facts,
///         facts.identity(),
///     )
///     .unwrap();
///     let CssColorParseStep::Pending(CssColorParseAction::ReadInput(read)) = parse.step() else {
///         panic!("a parse reads its input first")
///     };
///     let parse = read.read(u16::from(b'r')).unwrap();
///     let CssColorParseStep::Pending(CssColorParseAction::AppendSlot(write)) = parse.step() else {
///         panic!("a read unit is stored")
///     };
///     write.record().0
/// };
/// let _ = slot.bank();
/// ```
pub struct CssColorParse<'owner, 'facts> {
    cursor: core::ColorOperation<'owner, 'facts>,
}
impl<'owner, 'facts> CssColorParse<'owner, 'facts> {
    /// Fixed metadata admission precedes this call; no array/input is scanned.
    ///
    /// # Errors
    ///
    /// [`CssColorFactDefect::Foreign`] when `facts` is not the envelope named
    /// by `expected`.
    ///
    /// # Panics
    ///
    /// If `owner` has already begun `usize::MAX` parses; no input reaches
    /// that count.
    pub fn start(
        owner: &'owner mut CssColorOperationOwner,
        kind: CssColorInputKind,
        length: usize,
        request: CssColorGrammarRequest,
        facts: &'facts CssColorFacts<'facts>,
        expected: CssColorFactIdentity,
    ) -> Result<Self, CssColorError> {
        if facts.identity != expected {
            return Err(ColorErrorKind::Facts(CssColorFactDefect::Foreign).into());
        }
        match core::ColorOperation::start(
            owner,
            kind.native(),
            length,
            request.native(),
            facts,
            expected,
        ) {
            Ok(cursor) => Ok(Self { cursor }),
            Err(defect) => defect.raise(),
        }
    }
    /// Fixed owner/header reads for initialization, separate from the
    /// continuation's own compiled register allocation. No array is traversed.
    #[must_use]
    pub const fn initialization_layout() -> CssColorInitializationLayout {
        CssColorInitializationLayout {
            owner_bytes: CssColorOperationOwner::layout_bytes(),
            register_bytes: Self::register_bytes(),
            fact_identity_bytes: std::mem::size_of::<CssColorFactIdentity>(),
        }
    }
    /// Actual compiled fixed continuation layout, admitted before initialization.
    #[must_use]
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    /// Actual compiled layout of one outstanding action, which owns the parse.
    #[must_use]
    pub const fn action_bytes() -> usize {
        std::mem::size_of::<CssColorParseAction<'_, '_>>()
    }
    /// The exact acknowledged logical extent; capacity remains caller-owned.
    #[must_use]
    pub const fn bank_length(&self, bank: CssColorBank) -> usize {
        self.cursor.bank_length(bank)
    }
    /// Consumes the parse and yields its next action or its outcome.
    pub fn step(self) -> CssColorParseStep<'owner, 'facts> {
        let pending = pending(&self.cursor);
        let parse = self;
        CssColorParseStep::Pending(match pending {
            Pending::ReadInput { offset } => {
                CssColorParseAction::ReadInput(CssColorReadInput { parse, offset })
            }
            Pending::ReadSlot { bank, index } => {
                CssColorParseAction::ReadSlot(CssColorReadSlot { parse, bank, index })
            }
            Pending::AppendSlot { bank, index } => {
                CssColorParseAction::AppendSlot(CssColorWriteSlot { parse, bank, index })
            }
            Pending::UpdateSlot { bank, index } => {
                CssColorParseAction::UpdateSlot(CssColorWriteSlot { parse, bank, index })
            }
            Pending::ReadFact(fact) => {
                CssColorParseAction::ReadFact(CssColorReadFact { parse, fact })
            }
            Pending::Compute(kernel) => {
                CssColorParseAction::Compute(CssColorCompute { parse, kernel })
            }
            Pending::Complete => {
                return CssColorParseStep::Complete(match parse.cursor.take() {
                    Ok(core::ParseOutcome::Used(checkpoint)) => {
                        CssColorParseOutcome::Used(CssUsedColorCheckpoint { checkpoint })
                    }
                    Ok(core::ParseOutcome::StandardFailure(checkpoint)) => {
                        CssColorParseOutcome::StandardFailure(CssColorFailureCheckpoint {
                            checkpoint,
                        })
                    }
                    Err(defect) => defect.raise(),
                });
            }
            Pending::Stopped => ColorProtocolError::Terminal.raise(),
        })
    }
    fn layout(&self, pending: Pending) -> CssColorActionLayout {
        layout(
            pending,
            CssColorInputKind::from_native(self.cursor.input_kind()),
            self.cursor.kernel(),
        )
    }
    /// Panics unless the parse minted `slot` for exactly `bank` and `index`.
    #[track_caller]
    fn require_record(
        &self,
        slot: &CssColorSlot<'_>,
        bank: CssColorBank,
        index: usize,
        verb: &str,
    ) {
        let minted = self
            .cursor
            .qualifier()
            .is_some_and(|(owner, phase)| slot.minted_by(owner, phase));
        assert!(minted, "the color record belongs to another parse");
        let (actual_bank, actual_index) = slot.position();
        assert!(
            (actual_bank, actual_index) == (bank, index),
            "the color record minted for {actual_bank:?} index {actual_index} was {verb} {bank:?} index {index}"
        );
    }
}
/// The parse's next action, or its completed outcome.
#[expect(
    clippy::large_enum_variant,
    reason = "A pending action owns the fixed-register parse inline, admitted as action_bytes; boxing it introduces an unadmitted heap allocation"
)]
pub enum CssColorParseStep<'owner, 'facts> {
    /// The parse waits on this action.
    Pending(CssColorParseAction<'owner, 'facts>),
    /// The parse finished with this outcome.
    Complete(CssColorParseOutcome<'facts>),
}
/// Exactly one finite action; the guard owns the parse until its completion.
pub enum CssColorParseAction<'owner, 'facts> {
    /// Read one original input unit.
    ReadInput(CssColorReadInput<'owner, 'facts>),
    /// Read back one record stored in a bank.
    ReadSlot(CssColorReadSlot<'owner, 'facts>),
    /// Append a new record at the end of a bank.
    AppendSlot(CssColorWriteSlot<'owner, 'facts>),
    /// Replace a record already stored in a bank.
    UpdateSlot(CssColorWriteSlot<'owner, 'facts>),
    /// Observe one fixed fact of the supplied envelope.
    ReadFact(CssColorReadFact<'owner, 'facts>),
    /// Execute one fixed-register kernel transition.
    Compute(CssColorCompute<'owner, 'facts>),
}
impl<'owner, 'facts> CssColorParseAction<'owner, 'facts> {
    /// The payload this action observes or writes, admitted before completion.
    #[must_use]
    pub fn layout(&self) -> CssColorActionLayout {
        match self {
            Self::ReadInput(read) => read.parse.layout(Pending::ReadInput {
                offset: read.offset,
            }),
            Self::ReadSlot(read) => read.parse.layout(Pending::ReadSlot {
                bank: read.bank,
                index: read.index,
            }),
            Self::AppendSlot(write) => write.parse.layout(Pending::AppendSlot {
                bank: write.bank,
                index: write.index,
            }),
            Self::UpdateSlot(write) => write.parse.layout(Pending::UpdateSlot {
                bank: write.bank,
                index: write.index,
            }),
            Self::ReadFact(read) => read.parse.layout(Pending::ReadFact(read.fact)),
            Self::Compute(compute) => compute.parse.layout(Pending::Compute(compute.kernel)),
        }
    }
    /// Declines the action without effect (consumer refusal); the parse stays pending.
    #[must_use]
    pub fn into_parse(self) -> CssColorParse<'owner, 'facts> {
        match self {
            Self::ReadInput(CssColorReadInput { parse, .. })
            | Self::ReadSlot(CssColorReadSlot { parse, .. })
            | Self::AppendSlot(CssColorWriteSlot { parse, .. })
            | Self::UpdateSlot(CssColorWriteSlot { parse, .. })
            | Self::ReadFact(CssColorReadFact { parse, .. })
            | Self::Compute(CssColorCompute { parse, .. }) => parse,
        }
    }
}
/// One original input unit read at a fixed offset.
pub struct CssColorReadInput<'owner, 'facts> {
    parse: CssColorParse<'owner, 'facts>,
    offset: usize,
}
impl<'owner, 'facts> CssColorReadInput<'owner, 'facts> {
    /// The original unit offset to read; it is below the declared length.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }
    /// Supplies the original unit at [`offset`](Self::offset).
    ///
    /// # Errors
    ///
    /// A [`CssColorError::malformed_input_offset`] error when the unit proves the input
    /// malformed for its declared encoding.
    pub fn read(self, unit: u16) -> Result<CssColorParse<'owner, 'facts>, CssColorError> {
        let mut parse = self.parse;
        match parse.cursor.read_input(self.offset, unit) {
            Ok(()) => Ok(parse),
            Err(error) => Err(reached(error)),
        }
    }
}
/// One caller-owned record read back from a fixed bank position.
pub struct CssColorReadSlot<'owner, 'facts> {
    parse: CssColorParse<'owner, 'facts>,
    bank: CssColorBank,
    index: usize,
}
impl<'owner, 'facts> CssColorReadSlot<'owner, 'facts> {
    /// Returns the bank holding the record to read back.
    #[must_use]
    pub const fn bank(&self) -> CssColorBank {
        self.bank
    }
    /// Returns the position of the record within its bank.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }
    /// Lends the record stored at [`bank`](Self::bank) and [`index`](Self::index).
    ///
    /// # Errors
    ///
    /// A reached unavailable, unsupported or malformed-fact failure.
    ///
    /// # Panics
    ///
    /// If `slot` is not the record this parse stored at `bank`/`index`.
    #[track_caller]
    pub fn read(
        self,
        slot: &CssColorSlot<'_>,
    ) -> Result<CssColorParse<'owner, 'facts>, CssColorError> {
        self.parse
            .require_record(slot, self.bank, self.index, "read as");
        let mut parse = self.parse;
        match parse.cursor.read_slot(self.bank, self.index, slot) {
            Ok(()) => Ok(parse),
            Err(error) => Err(reached(error)),
        }
    }
}
/// One record to append at, or replace in, a fixed bank position.
pub struct CssColorWriteSlot<'owner, 'facts> {
    parse: CssColorParse<'owner, 'facts>,
    bank: CssColorBank,
    index: usize,
}
impl<'owner, 'facts> CssColorWriteSlot<'owner, 'facts> {
    /// Returns the bank the record is written to.
    #[must_use]
    pub const fn bank(&self) -> CssColorBank {
        self.bank
    }
    /// Returns the position the record is stored at; for an append it is the bank's current length.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }
    /// Mints the record to store at [`bank`](Self::bank)/[`index`](Self::index).
    ///
    /// Generate only after the write/reserve admission. Advancing still
    /// requires the receipt's acknowledgement of the actual storage.
    #[must_use]
    pub fn record(self) -> (CssColorSlot<'owner>, CssColorSlotReceipt<'owner, 'facts>) {
        let mut parse = self.parse;
        let record = match parse.cursor.pending_record(self.bank, self.index) {
            Ok(record) => record,
            Err(error) => match classify(error) {
                Err(defect) => defect.raise(),
                Ok(error) => panic!("minting a color record reached {error}"),
            },
        };
        (
            record,
            CssColorSlotReceipt {
                parse,
                bank: self.bank,
                index: self.index,
            },
        )
    }
}
/// The pending acknowledgement of one minted record's storage.
pub struct CssColorSlotReceipt<'owner, 'facts> {
    parse: CssColorParse<'owner, 'facts>,
    bank: CssColorBank,
    index: usize,
}
impl<'owner, 'facts> CssColorSlotReceipt<'owner, 'facts> {
    /// Acknowledges that `stored` now holds the minted record at its position.
    ///
    /// # Panics
    ///
    /// If `stored` is not the record minted by the matching
    /// [`record`](CssColorWriteSlot::record).
    #[track_caller]
    pub fn stored(self, stored: &CssColorSlot<'_>) -> CssColorParse<'owner, 'facts> {
        self.parse
            .require_record(stored, self.bank, self.index, "stored as");
        let mut parse = self.parse;
        if let Err(error) = parse.cursor.written(self.bank, self.index, stored) {
            match classify(error) {
                Err(defect) => defect.raise(),
                Ok(error) => panic!("storing a color record reached {error}"),
            }
        }
        parse
    }
    /// Storage refused: the minted record is discarded and the parse waits at
    /// the same write.
    #[must_use]
    pub fn into_parse(self) -> CssColorParse<'owner, 'facts> {
        self.parse
    }
}
/// One fixed observation of the supplied fact envelope.
pub struct CssColorReadFact<'owner, 'facts> {
    parse: CssColorParse<'owner, 'facts>,
    fact: CssColorFactRead,
}
impl<'owner, 'facts> CssColorReadFact<'owner, 'facts> {
    /// Returns the fixed fact observation the parse is waiting on.
    #[must_use]
    pub const fn fact(&self) -> CssColorFactRead {
        self.fact
    }
    /// Observes [`fact`](Self::fact) in the parse's own envelope.
    ///
    /// # Errors
    ///
    /// A reached unavailable, unsupported or malformed-fact failure.
    pub fn read(self) -> Result<CssColorParse<'owner, 'facts>, CssColorError> {
        let mut parse = self.parse;
        match parse.cursor.compute() {
            Ok(()) => Ok(parse),
            Err(error) => Err(reached(error)),
        }
    }
}
/// One fixed-register kernel transition.
pub struct CssColorCompute<'owner, 'facts> {
    parse: CssColorParse<'owner, 'facts>,
    kernel: CssColorKernel,
}
impl<'owner, 'facts> CssColorCompute<'owner, 'facts> {
    /// Returns the fixed kernel this action executes.
    #[must_use]
    pub const fn kernel(&self) -> CssColorKernel {
        self.kernel
    }
    /// Executes [`kernel`](Self::kernel).
    ///
    /// # Errors
    ///
    /// A reached unavailable, unsupported, malformed-fact or malformed-input
    /// failure.
    pub fn compute(self) -> Result<CssColorParse<'owner, 'facts>, CssColorError> {
        let mut parse = self.parse;
        match parse.cursor.compute() {
            Ok(()) => Ok(parse),
            Err(error) => Err(reached(error)),
        }
    }
}
/// Standard failure is the only parse result that admits an explicit fallback.
pub enum CssColorParseOutcome<'facts> {
    /// The input parsed to a used color.
    Used(CssUsedColorCheckpoint<'facts>),
    /// The input is standard-invalid; the checkpoint admits an explicit fallback.
    StandardFailure(CssColorFailureCheckpoint<'facts>),
}
/// Fixed used value plus original source/fact qualification, no recipe backing.
pub struct CssUsedColorCheckpoint<'facts> {
    pub(in crate::color) checkpoint: core::UsedCheckpoint<'facts>,
}
/// Actual standard-invalid branch, distinct from reached capability failure.
pub struct CssColorFailureCheckpoint<'facts> {
    checkpoint: core::FailureCheckpoint<'facts>,
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
impl<'facts> CssUsedColorCheckpoint<'facts> {
    /// Returns the used color value.
    #[must_use]
    pub const fn value(&self) -> Color {
        self.checkpoint.value()
    }
    /// Returns the original input range the color consumed.
    #[must_use]
    pub fn consumed(&self) -> CssColorInputRange {
        let range = self.checkpoint.context().consumed;
        CssColorInputRange {
            start: range.start,
            end: range.end,
        }
    }
    /// Returns the descriptor of the original input.
    #[must_use]
    pub fn source(&self) -> CssColorSourceDescriptor {
        source(self.checkpoint.context())
    }
    /// Returns the identity of the fact set the parse expected.
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
    /// Returns the standard failure an explicit fallback replaced, or `None` when the color parsed directly.
    #[must_use]
    pub fn standard_failure(&self) -> Option<CssColorStandardFailure> {
        self.checkpoint.standard_failure().map(failure)
    }
    /// Starts one admitted, total numerical transformation over `owner`.
    ///
    /// # Panics
    ///
    /// If `owner` has already begun `usize::MAX` operations.
    #[must_use]
    pub fn transform<'num>(
        self,
        owner: &'num mut surgeist_color::OperationOwner,
        transformation: Transformation,
    ) -> CssColorTransform<'num, 'facts> {
        let computation = surgeist_color::Computation::new(
            owner,
            surgeist_color::NumericalRequest::Transform {
                value: self.value(),
                transformation,
            },
        );
        CssColorTransform {
            checkpoint: self.checkpoint,
            computation,
        }
    }
}
impl<'facts> CssColorFailureCheckpoint<'facts> {
    /// Returns the standard failure: its reason and original input range.
    #[must_use]
    pub fn failure(&self) -> CssColorStandardFailure {
        failure(self.checkpoint.failure())
    }
    /// Consumer policy supplies the already checked value; no unavailable,
    /// unsupported, malformed-fact or resource failure has this transition.
    #[must_use]
    pub fn fallback(self, value: Color) -> CssUsedColorCheckpoint<'facts> {
        CssUsedColorCheckpoint {
            checkpoint: self.checkpoint.fallback(value),
        }
    }
}

/// One pending total numerical transformation of a used checkpoint.
///
/// It wraps [`surgeist_color::Computation`], so it has no error. The
/// checkpoint's value can only come from CSS resolution, an explicit fallback
/// on a standard failure, or a CSS-issued transformation.
pub struct CssColorTransform<'num, 'facts> {
    checkpoint: core::UsedCheckpoint<'facts>,
    computation: surgeist_color::Computation<'num>,
}
impl<'facts> CssColorTransform<'_, 'facts> {
    /// Actual compiled continuation layout, admitted before construction.
    #[must_use]
    pub const fn register_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    /// The fixed kernel [`compute`](Self::compute) executes.
    #[must_use]
    pub const fn kernel(&self) -> surgeist_color::Kernel {
        self.computation.kernel()
    }
    /// The payload the computation observes, admitted before it executes.
    #[must_use]
    pub const fn layout(&self) -> CssColorActionLayout {
        let numerical = self.computation.layout();
        CssColorActionLayout {
            input_unit_bytes: 0,
            slot_read_bytes: 0,
            slot_write_bytes: 0,
            fact_records: [0; 3],
            output_unit_bytes: numerical.output_bytes,
            numerical_read_bytes: numerical.read_bytes,
            numerical_write_bytes: numerical.write_bytes,
            kernel: CssColorKernel::Numerical(self.computation.kernel()),
        }
    }
    /// Executes the kernel; the transformed value becomes the checkpoint's value.
    #[must_use]
    pub fn compute(self) -> CssUsedColorCheckpoint<'facts> {
        let mut checkpoint = self.checkpoint;
        checkpoint.value = self.computation.compute();
        CssUsedColorCheckpoint { checkpoint }
    }
    /// Declines the transformation; the original checkpoint is returned unchanged.
    #[must_use]
    pub fn into_checkpoint(self) -> CssUsedColorCheckpoint<'facts> {
        CssUsedColorCheckpoint {
            checkpoint: self.checkpoint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact};
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
    fn start<'owner, 'facts>(
        owner: &'owner mut CssColorOperationOwner,
        kind: CssColorInputKind,
        length: usize,
        facts: &'facts CssColorFacts<'facts>,
    ) -> CssColorParse<'owner, 'facts> {
        CssColorParse::start(
            owner,
            kind,
            length,
            CssColorGrammarRequest::WholeColor,
            facts,
            facts.identity(),
        )
        .unwrap()
    }
    /// Services every action with ordinary vectors; `observe` sees each
    /// pending action before its completion.
    fn drive<'owner, 'facts>(
        mut parse: CssColorParse<'owner, 'facts>,
        input: &[u16],
        banks: &mut [Vec<CssColorSlot<'owner>>; 3],
        mut observe: impl FnMut(&CssColorParseAction<'owner, 'facts>),
    ) -> Result<CssColorParseOutcome<'facts>, CssColorError> {
        loop {
            let action = match parse.step() {
                CssColorParseStep::Complete(outcome) => return Ok(outcome),
                CssColorParseStep::Pending(action) => action,
            };
            observe(&action);
            parse = match action {
                CssColorParseAction::ReadInput(read) => {
                    let unit = input[read.offset()];
                    read.read(unit)?
                }
                CssColorParseAction::ReadSlot(read) => {
                    let slot = &banks[bank(read.bank())][read.index()];
                    read.read(slot)?
                }
                CssColorParseAction::AppendSlot(write) => {
                    let target = &mut banks[bank(write.bank())];
                    assert_eq!(write.index(), target.len());
                    let (record, receipt) = write.record();
                    target.push(record);
                    receipt.stored(&target[target.len() - 1])
                }
                CssColorParseAction::UpdateSlot(write) => {
                    let index = write.index();
                    let target = &mut banks[bank(write.bank())];
                    let (record, receipt) = write.record();
                    target[index] = record;
                    receipt.stored(&target[index])
                }
                CssColorParseAction::ReadFact(read) => read.read()?,
                CssColorParseAction::Compute(compute) => compute.compute()?,
            };
        }
    }
    fn units(input: &str) -> Vec<u16> {
        input.bytes().map(u16::from).collect()
    }
    fn parse<'facts>(
        owner: &mut CssColorOperationOwner,
        input: &str,
        facts: &'facts CssColorFacts<'facts>,
    ) -> Result<(CssColorParseOutcome<'facts>, usize), CssColorError> {
        let parse = start(owner, CssColorInputKind::Utf8, input.len(), facts);
        let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
        let mut folds = 0;
        let outcome = drive(parse, &units(input), &mut banks, |action| {
            let actual = action.layout();
            match action {
                CssColorParseAction::ReadInput(_) => {
                    assert_eq!(actual.input_unit_bytes, std::mem::size_of::<u8>());
                }
                CssColorParseAction::ReadSlot(_) => {
                    assert_eq!(actual.slot_read_bytes, CssColorSlot::record_bytes());
                }
                CssColorParseAction::AppendSlot(_) | CssColorParseAction::UpdateSlot(_) => {
                    assert_eq!(actual.slot_read_bytes, CssColorSlot::record_bytes());
                    assert_eq!(actual.slot_write_bytes, CssColorSlot::record_bytes());
                }
                CssColorParseAction::ReadFact(read) => {
                    assert_eq!(actual.fact_records, read.fact().record_layouts());
                }
                CssColorParseAction::Compute(compute) => {
                    assert_eq!(actual.kernel, compute.kernel());
                    folds += compute.kernel().variadic_fold_count();
                }
            }
        });
        // This ordinary owner clears its vectors. It does not simulate real
        // HTML capacity/ticket cleanup or claim foreign-controller admission.
        banks.iter_mut().for_each(Vec::clear);
        drop(banks);
        Ok((outcome?, folds))
    }
    /// Serializes a checkpoint's value as consumers do, through Color's own
    /// serialization, and keeps the checkpoint's source and failure.
    fn serialize(
        checkpoint: CssUsedColorCheckpoint<'_>,
        owner: &mut surgeist_color::OperationOwner,
        request: surgeist_color::SerializationRequest,
    ) -> (
        String,
        Color,
        Option<CssColorSourceDescriptor>,
        Option<CssColorStandardFailure>,
    ) {
        use surgeist_color::{SerializationAction as A, SerializationStep};
        let mut digits = Vec::new();
        let mut output = Vec::new();
        let mut serialization =
            surgeist_color::Serialization::new(owner, checkpoint.value(), request);
        let receipt = loop {
            let action = match serialization.step() {
                SerializationStep::Complete(receipt) => break receipt,
                SerializationStep::Pending(action) => action,
            };
            serialization = match action {
                A::Compute(compute) => compute.compute(),
                A::ReadDigit(read) => {
                    let slot = &digits[read.index()];
                    read.read(slot)
                }
                A::AppendDigit(write) => {
                    assert_eq!(write.index(), digits.len());
                    let (record, receipt) = write.record();
                    digits.push(record);
                    receipt.stored(&digits[digits.len() - 1])
                }
                A::ReplaceDigit(write) => {
                    let index = write.index();
                    let (record, receipt) = write.record();
                    digits[index] = record;
                    receipt.stored(&digits[index])
                }
                A::Emit(emit) => {
                    output.push(emit.unit());
                    emit.emitted()
                }
            };
        };
        assert_eq!(receipt.length(), output.len());
        drop(digits);
        (
            String::from_utf8(output).unwrap(),
            receipt.value(),
            Some(checkpoint.source()),
            checkpoint.standard_failure(),
        )
    }
    #[test]
    fn a_declined_action_keeps_the_parse_pending_and_owner_reuse_preserves_source() {
        let facts = facts();
        let mut css_owner = CssColorOperationOwner::new();
        {
            let parse = start(&mut css_owner, CssColorInputKind::Utf8, 3, &facts);
            let CssColorParseStep::Pending(action) = parse.step() else {
                panic!("a parse reads its input first")
            };
            let pending = action.layout();
            // Declining performs no observation or transition.
            let parse = action.into_parse();
            let CssColorParseStep::Pending(CssColorParseAction::ReadInput(read)) = parse.step()
            else {
                panic!("the same read remains pending")
            };
            assert_eq!(read.offset(), 0);
            assert_eq!(
                CssColorParseAction::ReadInput(read).layout(),
                pending,
                "declining changes nothing"
            );
        }
        let (outcome, _) = parse(&mut css_owner, "red", &facts).unwrap();
        let CssColorParseOutcome::Used(checkpoint) = outcome else {
            panic!("red")
        };
        let source = checkpoint.source();
        let mut numerical_owner = surgeist_color::OperationOwner::new();
        let transform =
            checkpoint.transform(&mut numerical_owner, Transformation::ForceOpaqueAlpha);
        let checkpoint = transform.compute();
        assert_eq!(checkpoint.source(), source);
        let (output, _, receipt_source, _) = serialize(
            checkpoint,
            &mut numerical_owner,
            surgeist_color::SerializationRequest::HtmlCompatible,
        );
        assert_eq!(output, "#ff0000");
        assert_eq!(receipt_source, Some(source));
        // Source metadata survives after the numerical owner borrow has ended.
        let request = surgeist_color::NumericalRequest::Construct {
            space: surgeist_color::ColorSpace::Lab,
            channels: [None; 3],
            alpha: None,
        };
        assert_eq!(
            surgeist_color::Computation::new(&mut numerical_owner, request)
                .compute()
                .alpha(),
            None
        );
        assert_eq!(receipt_source, Some(source));
    }

    #[test]
    fn public_guard_drives_actual_banks_and_exposes_three_folds_per_argument() {
        let mut numerical_owner = surgeist_color::OperationOwner::new();
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
        assert_eq!(
            serialize(
                checkpoint,
                &mut numerical_owner,
                surgeist_color::SerializationRequest::Css
            ),
            ("rgb(1, 0, 0)".to_owned(), value, Some(source), None)
        );
    }
    #[test]
    fn public_checkpoint_transformation_retains_the_fallback_reason() {
        let mut numerical_owner_2 = surgeist_color::OperationOwner::new();
        let mut numerical_owner_1 = surgeist_color::OperationOwner::new();
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let (outcome, _) = parse(&mut owner, "rgb(1deg 0 0)", &facts).unwrap();
        let CssColorParseOutcome::StandardFailure(failed) = outcome else {
            panic!("degree is not an RGB scalar")
        };
        let expected = failed.failure();
        let checkpoint = failed.fallback(Color::opaque_black());
        assert_eq!(checkpoint.standard_failure(), Some(expected));
        let transform =
            checkpoint.transform(&mut numerical_owner_1, Transformation::ForceOpaqueAlpha);
        let kernel = surgeist_color::Kernel::Transform(Transformation::ForceOpaqueAlpha);
        assert_eq!(transform.kernel(), kernel);
        assert_eq!(transform.layout().kernel, CssColorKernel::Numerical(kernel));
        let checkpoint = transform.compute();
        let source = checkpoint.source();
        let value = checkpoint.value();
        assert_eq!(
            serialize(
                checkpoint,
                &mut numerical_owner_2,
                surgeist_color::SerializationRequest::HtmlCompatible
            ),
            ("#000000".to_owned(), value, Some(source), Some(expected))
        );
    }
    #[test]
    fn reached_profile_metadata_precedes_distinct_forward_mix_and_reverse_capabilities() {
        use crate::color::{
            CssColorProfileAvailability as A, CssColorProfileBinding, CssColorProfileComponentName,
            CssColorProfileConversion, CssColorProfileName,
        };
        use crate::rule::color_profile::CssColorProfileRenderingIntent;
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
            assert_eq!(error.unsupported(), Some(expected), "{input}");
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
        assert_eq!(
            parse(&mut owner, "color-mix(in --p, red, blue)", &facts)
                .err()
                .and_then(|error| error.unavailable()),
            Some(CssColorUnavailable::ProfileConversion)
        );
    }
    #[test]
    fn refused_storage_waits_at_the_same_write_and_records_name_their_bank() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let parse = start(&mut owner, CssColorInputKind::Utf8, 1, &facts);
        let CssColorParseStep::Pending(CssColorParseAction::ReadInput(read)) = parse.step() else {
            panic!("a parse reads its input first")
        };
        let parse = read.read(u16::from(b'r')).unwrap();
        let CssColorParseStep::Pending(CssColorParseAction::AppendSlot(write)) = parse.step()
        else {
            panic!("a read unit is stored")
        };
        assert_eq!((write.bank(), write.index()), (CssColorBank::Algorithm, 0));
        let (refused, receipt) = write.record();
        assert_eq!(refused.bank(), CssColorBank::Algorithm);
        // Storage refused: the minted record stays unstored.
        let parse = receipt.into_parse();
        assert_eq!(parse.bank_length(CssColorBank::Algorithm), 0);
        let CssColorParseStep::Pending(CssColorParseAction::AppendSlot(write)) = parse.step()
        else {
            panic!("the same write remains pending")
        };
        assert_eq!((write.bank(), write.index()), (CssColorBank::Algorithm, 0));
        let (record, receipt) = write.record();
        let bank = [record];
        let parse = receipt.stored(&bank[0]);
        assert_eq!(parse.bank_length(CssColorBank::Algorithm), 1);
    }
    #[test]
    fn fixed_initialization_and_utf16_observation_are_exposed_before_action() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let layout = CssColorParse::initialization_layout();
        assert_eq!(
            layout.owner_bytes,
            std::mem::size_of::<CssColorOperationOwner>()
        );
        assert_eq!(layout.owner_bytes, CssColorOperationOwner::layout_bytes());
        assert_eq!(
            layout.register_bytes,
            std::mem::size_of::<CssColorParse<'_, '_>>()
        );
        assert_eq!(
            layout.fact_identity_bytes,
            std::mem::size_of::<CssColorFactIdentity>()
        );
        assert_eq!(
            CssColorParse::action_bytes(),
            std::mem::size_of::<CssColorParseAction<'_, '_>>()
        );
        eprintln!(
            "compiled CSS color owner={} slot={} parse={} parse_action={} checkpoint={} transform={}",
            std::mem::size_of::<CssColorOperationOwner>(),
            CssColorSlot::record_bytes(),
            CssColorParse::register_bytes(),
            CssColorParse::action_bytes(),
            std::mem::size_of::<CssUsedColorCheckpoint<'_>>(),
            CssColorTransform::register_bytes(),
        );
        let expected = CssColorFactIdentity::new(90, 91);
        let refused = CssColorParse::start(
            &mut owner,
            CssColorInputKind::Utf16,
            3,
            CssColorGrammarRequest::WholeColor,
            &facts,
            expected,
        )
        .err();
        assert_eq!(
            refused.and_then(|error| error.fact_defect()),
            Some(CssColorFactDefect::Foreign)
        );
        assert_eq!(owner.current_phase(), 0);
        let parse = start(&mut owner, CssColorInputKind::Utf16, 3, &facts);
        let CssColorParseStep::Pending(action) = parse.step() else {
            panic!("pending input")
        };
        let observed = action.layout();
        assert_eq!(observed.input_unit_bytes, std::mem::size_of::<u16>());
        assert_eq!(observed.slot_read_bytes, 0);
        assert_eq!(observed.slot_write_bytes, 0);
        assert_eq!(observed.fact_records, [0; 3]);
        let CssColorParseAction::ReadInput(read) = action else {
            panic!("the first action reads input")
        };
        assert_eq!(read.offset(), 0);
    }

    #[test]
    fn completed_intrinsic_kernels_distinguish_contrast_and_each_mix_fold() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let text = "color-mix(in srgb, contrast-color(black) 25%, black 25%, white 50%)";
        let parse = start(&mut owner, CssColorInputKind::Utf8, text.len(), &facts);
        let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
        let mut contrast = 0;
        let mut interpolation = 0;
        let outcome = drive(parse, &units(text), &mut banks, |action| {
            if let CssColorParseAction::Compute(compute) = action {
                assert_eq!(action.layout().kernel, compute.kernel());
                contrast += usize::from(compute.kernel() == CssColorKernel::Contrast);
                interpolation += usize::from(compute.kernel() == CssColorKernel::Interpolation);
            }
        })
        .unwrap();
        let CssColorParseOutcome::Used(value) = outcome else {
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
            // Color5 serialization converts present cylindrical results into
            // modern color(srgb ...); absolute hsl/hwb remain Color4 rgb(...).
            assert_eq!(
                serialize(
                    value,
                    &mut numerical_owner_0,
                    surgeist_color::SerializationRequest::Css
                )
                .0,
                "color(srgb 1 0 0)",
                "{text}"
            );
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
            assert_eq!(
                serialize(
                    value,
                    &mut numerical_owner_1,
                    surgeist_color::SerializationRequest::Css
                )
                .0,
                expected,
                "{text}"
            );
        }
    }

    #[test]
    fn an_arithmetic_parse_reads_every_real_bank() {
        let mut numerical_owner = surgeist_color::OperationOwner::new();
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let text = "rgb(calc(1 + 2) 0 0)";
        let parse = start(&mut owner, CssColorInputKind::Utf8, text.len(), &facts);
        let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
        let mut read_banks = [false; 3];
        let outcome = drive(parse, &units(text), &mut banks, |action| {
            if let CssColorParseAction::ReadSlot(read) = action {
                read_banks[bank(read.bank())] = true;
            }
        })
        .unwrap();
        let CssColorParseOutcome::Used(checkpoint) = outcome else {
            panic!("valid intrinsic arithmetic")
        };
        assert_eq!(read_banks, [true; 3]);
        assert_eq!(
            checkpoint.value().channels(),
            [Some(3.0 / 255.0), Some(0.0), Some(0.0)]
        );
        // The parse records stay retained while the used value serializes;
        // Color's serialization never observes CSS banks.
        assert_eq!(
            serialize(
                checkpoint,
                &mut numerical_owner,
                surgeist_color::SerializationRequest::Css
            )
            .0,
            "rgb(3, 0, 0)"
        );
        banks.iter_mut().for_each(Vec::clear);
    }

    #[test]
    fn a_declined_transformation_returns_the_original_checkpoint() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let (outcome, _) = parse(&mut owner, "rgb(100% 0 0 / .5)", &facts).unwrap();
        let CssColorParseOutcome::Used(checkpoint) = outcome else {
            panic!("valid resolved color")
        };
        let original = checkpoint.value();
        let source = checkpoint.source();
        assert_eq!(original.srgb_bytes(), None);
        let mut numerical_owner = surgeist_color::OperationOwner::new();
        let transform = checkpoint.transform(&mut numerical_owner, Transformation::QuantizeSrgb);
        let declined = transform.into_checkpoint();
        // Only computing commits a transformation; declining keeps the input.
        assert!(declined.value().identical(original));
        assert_eq!(declined.value().srgb_bytes(), None);
        assert_eq!(declined.value().alpha(), Some(0.5));
        assert_eq!(declined.source(), source);
        assert!(std::ptr::eq(declined.facts(), &facts));
    }

    #[test]
    fn transformations_commit_computed_values_and_keep_the_source_context() {
        let mut numerical_owner = surgeist_color::OperationOwner::new();
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let (outcome, _) = parse(&mut owner, "rgba(255 0 0 / .5)", &facts).unwrap();
        let CssColorParseOutcome::Used(checkpoint) = outcome else {
            panic!("valid resolved color")
        };
        let source = checkpoint.source();
        assert!(std::ptr::eq(checkpoint.facts(), &facts));
        let checkpoint = checkpoint
            .transform(&mut numerical_owner, Transformation::ForceOpaqueAlpha)
            .compute();
        assert_eq!(checkpoint.value().alpha(), Some(1.0));
        assert_eq!(checkpoint.source(), source);
        let checkpoint = checkpoint
            .transform(&mut numerical_owner, Transformation::QuantizeSrgb)
            .into_checkpoint();
        // A declined quantization does not commit byte backing.
        assert_eq!(checkpoint.value().srgb_bytes(), None);
        assert_eq!(checkpoint.value().alpha(), Some(1.0));
        assert_eq!(checkpoint.value().alpha_byte(), None);
        assert_eq!(checkpoint.source(), source);
        assert_eq!(
            serialize(
                checkpoint,
                &mut numerical_owner,
                surgeist_color::SerializationRequest::HtmlCompatible
            )
            .0,
            "rgb(255, 0, 0)"
        );

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
        // XYZ Z is 1.089 times the shared channel, beyond binary64: the
        // conversion takes the closest finite value instead of failing.
        let checkpoint = checkpoint
            .transform(
                &mut numerical_owner,
                Transformation::Convert(surgeist_color::ColorSpace::Predefined(
                    surgeist_color::PredefinedColorSpace::XyzD65,
                )),
            )
            .compute();
        assert_eq!(checkpoint.value().channels()[2], Some(f64::MAX));
        assert!(!checkpoint.value().identical(before));
        assert_eq!(checkpoint.source(), source);
        assert_eq!(checkpoint.standard_failure(), None);
    }

    #[test]
    fn final_cylindrical_conversion_is_advertised_pending_and_abandonable() {
        let facts = facts();
        for abandon in [false, true] {
            let mut owner = CssColorOperationOwner::new();
            let text = "hsl(0 100% 50%)";
            let input = units(text);
            let mut parse = start(&mut owner, CssColorInputKind::Utf8, text.len(), &facts);
            let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
            let mut conversion = false;
            loop {
                let action = match parse.step() {
                    CssColorParseStep::Complete(CssColorParseOutcome::Used(value)) => {
                        assert_eq!(
                            value.value().space(),
                            surgeist_color::ColorSpace::Predefined(
                                surgeist_color::PredefinedColorSpace::Srgb
                            )
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
                    CssColorParseStep::Complete(_) => panic!("used checkpoint"),
                    CssColorParseStep::Pending(action) => action,
                };
                if let CssColorParseAction::Compute(compute) = &action
                    && compute.kernel() == CssColorKernel::ColorConversion
                {
                    conversion = true;
                    let layout = action.layout();
                    assert_eq!(layout.kernel, CssColorKernel::ColorConversion);
                    assert_eq!(layout.input_unit_bytes, 0);
                    assert_eq!(layout.slot_read_bytes, 0);
                    assert_eq!(layout.slot_write_bytes, 0);
                    assert_eq!(layout.output_unit_bytes, 0);
                    assert_eq!(layout.fact_records, [0; 3]);
                    if abandon {
                        // The caller withholds admission and abandons the action
                        // with its parse; the caller-owned banks keep their
                        // records until the caller releases them.
                        let lengths = banks.each_ref().map(Vec::len);
                        assert!(lengths[0] > 0);
                        let _abandoned = action;
                        assert_eq!(banks.each_ref().map(Vec::len), lengths);
                        break;
                    }
                    // Declining instead keeps the conversion pending.
                    let CssColorParseStep::Pending(CssColorParseAction::Compute(again)) =
                        action.into_parse().step()
                    else {
                        panic!("the conversion remains pending")
                    };
                    assert_eq!(again.kernel(), CssColorKernel::ColorConversion);
                    parse = again.compute().unwrap();
                    continue;
                }
                parse = match action {
                    CssColorParseAction::ReadInput(read) => {
                        let unit = input[read.offset()];
                        read.read(unit).unwrap()
                    }
                    CssColorParseAction::ReadSlot(read) => {
                        let slot = &banks[bank(read.bank())][read.index()];
                        read.read(slot).unwrap()
                    }
                    CssColorParseAction::AppendSlot(write) => {
                        let target = &mut banks[bank(write.bank())];
                        let (record, receipt) = write.record();
                        target.push(record);
                        receipt.stored(&target[target.len() - 1])
                    }
                    CssColorParseAction::UpdateSlot(write) => {
                        let index = write.index();
                        let target = &mut banks[bank(write.bank())];
                        let (record, receipt) = write.record();
                        target[index] = record;
                        receipt.stored(&target[index])
                    }
                    CssColorParseAction::ReadFact(read) => read.read().unwrap(),
                    CssColorParseAction::Compute(compute) => compute.compute().unwrap(),
                };
            }
            banks.iter_mut().for_each(Vec::clear);
            drop(banks);
            assert!(
                conversion,
                "actual root conversion must be admitted before Used"
            );
        }
    }

    /// Services `parse` until it requests a slot read, keeping its records.
    fn until_slot_read<'owner, 'facts>(
        mut parse: CssColorParse<'owner, 'facts>,
        input: &[u8],
        banks: &mut [Vec<CssColorSlot<'owner>>; 3],
    ) -> CssColorReadSlot<'owner, 'facts> {
        loop {
            let CssColorParseStep::Pending(action) = parse.step() else {
                panic!("a lexical parse reads its scalars back")
            };
            parse = match action {
                CssColorParseAction::ReadInput(read) => {
                    let unit = u16::from(input[read.offset()]);
                    read.read(unit).unwrap()
                }
                CssColorParseAction::ReadSlot(read) => return read,
                CssColorParseAction::AppendSlot(write) => {
                    let target = &mut banks[bank(write.bank())];
                    let (record, receipt) = write.record();
                    target.push(record);
                    receipt.stored(&target[target.len() - 1])
                }
                CssColorParseAction::Compute(compute) => compute.compute().unwrap(),
                _ => panic!("unexpected lexical action"),
            };
        }
    }
    #[test]
    #[should_panic(expected = "belongs to another parse")]
    fn reading_a_slot_minted_by_another_parse_panics() {
        // P1 (#1353, 5.6): a record of another parse is a driver defect.
        let facts = facts();
        let mut first_owner = CssColorOperationOwner::new();
        let mut second_owner = CssColorOperationOwner::new();
        let first = start(&mut first_owner, CssColorInputKind::Utf8, 3, &facts);
        let second = start(&mut second_owner, CssColorInputKind::Utf8, 3, &facts);
        let mut first_banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
        let mut second_banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
        let _ = until_slot_read(first, b"red", &mut first_banks);
        let read = until_slot_read(second, b"red", &mut second_banks);
        let foreign = &first_banks[bank(read.bank())][read.index()];
        let _ = read.read(foreign);
    }
    #[test]
    #[should_panic(expected = "was stored as Algorithm index 1")]
    fn storing_a_record_at_the_wrong_index_panics() {
        // P2 (#1353, 5.6): acknowledging the record of index 0 as the storage
        // of the record minted for index 1 is a driver defect.
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let mut parse = start(&mut owner, CssColorInputKind::Utf8, 3, &facts);
        let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
        loop {
            let CssColorParseStep::Pending(action) = parse.step() else {
                panic!("the input is stored before completion")
            };
            parse = match action {
                CssColorParseAction::ReadInput(read) => {
                    let unit = u16::from(b"red"[read.offset()]);
                    read.read(unit).unwrap()
                }
                CssColorParseAction::AppendSlot(write) => {
                    let index = write.index();
                    let target = &mut banks[bank(write.bank())];
                    let (record, receipt) = write.record();
                    target.push(record);
                    let stored = &target[index.saturating_sub(1)];
                    receipt.stored(stored)
                }
                CssColorParseAction::Compute(compute) => compute.compute().unwrap(),
                _ => panic!("unexpected lexical action"),
            };
        }
    }
}
