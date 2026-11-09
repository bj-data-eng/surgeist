//! Control autocomplete grammar and literal pinned intrinsic token processing.
//! Form-owner defaults, autofill data, heuristics and credential mediation are
//! separate contextual/effect owners. WHATWG attribution is in the crate notice.

use super::*;
use crate::internal::accounting::{
    HtmlCapScope, HtmlChange, HtmlLimits, HtmlResource, HtmlTariffs, HtmlWork, autocomplete_limits,
    wide,
};
use crate::{
    internal::microsyntax::tokens::DeclaredKeyword,
    internal::microsyntax::tokens::HtmlSpaceSeparatedTokens,
    internal::microsyntax::tokens::HtmlSpaceToken, internal::microsyntax::tokens::HtmlTokenError,
    internal::microsyntax::tokens::declared_keyword,
};
use std::{error::Error, fmt};
use surgeist_ledger::{Change, Error as LedgerError, Operation, RefusalReason, Scope};

const RULE: RuleMetadata = RuleMetadata {
    attribute: KnownHtmlAttribute::Autocomplete,
    rows: &[14],
    source: source!("attr-fe-autocomplete"),
    context: Context::Elements(&[
        KnownHtmlElement::Input,
        KnownHtmlElement::Select,
        KnownHtmlElement::Textarea,
    ]),
    vocabulary_source: None,
};
const PROCESSING_SOURCE: &str = source!("autofill-processing-model");
const WEBAUTHN_KEYWORD: DeclaredKeyword = declared_keyword!("webauthn");
const WEBAUTHN: &str = WEBAUTHN_KEYWORD.as_str();

/// Atomic scan/copy/lookup envelope, not machine instructions or parser polling.
/// The semantic owner separately admits expanded-name recognition. Two shared
/// split passes and their boundary visits use four input-length passes; token
/// copy/validation two; per-token reserve/push visits two; section transform and
/// validation two; IDL length/copy visits two. Vocabulary comparisons inspect at
/// most keyword length plus a length check: three field-table searches, two each
/// of setting/mode/contact, and three credential checks. The fixed remainder
/// covers section prefixes, at most five IDL and three scope handles, their
/// bounded prepends/reservations, default/context observations and empty input.
pub fn autocomplete_materialization_work_bound(input_code_units: usize) -> Option<usize> {
    let fields = HtmlAutofillField::ALL
        .iter()
        .try_fold(0usize, |sum, field| {
            sum.checked_add(field.canonical_keyword().len() + 1)
        })?;
    let modes = HtmlAutocompleteMode::ALL
        .iter()
        .try_fold(0usize, |sum, hint| {
            sum.checked_add(hint.canonical_keyword().len() + 1)
        })?;
    let contacts = HtmlAutocompleteContactHint::ALL
        .iter()
        .try_fold(0usize, |sum, hint| {
            sum.checked_add(hint.canonical_keyword().len() + 1)
        })?;
    let settings = FORM_AUTOCOMPLETE
        .keywords
        .iter()
        .try_fold(0usize, |sum, keyword| {
            sum.checked_add(keyword.spelling.len() + 1)
        })?;
    input_code_units
        .checked_mul(12)?
        .checked_add(fields.checked_mul(3)?)?
        .checked_add(modes.checked_mul(2)?)?
        .checked_add(contacts.checked_mul(2)?)?
        .checked_add(settings.checked_mul(2)?)?
        .checked_add((WEBAUTHN.len() + 1).checked_mul(3)?)?
        .checked_add(128)
}

/// Remaining session allowances. Fixed Arc wrappers and allocator internals are
/// excluded; every variable Vec capacity is observed before its next write.
#[derive(Clone, Copy, Debug)]
pub struct HtmlAutocompleteMaterializationLimits {
    pub(crate) allocated_bytes: usize,
    pub(crate) nested_output_records: usize,
    pub(crate) scratch_records: usize,
}

/// Storage a materialization actually took, in the same units as
/// [`HtmlAutocompleteMaterializationLimits`].
///
/// Growth is observed after each reservation and before it can be rejected, so the figures include
/// any allocator overshoot.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlAutocompleteMaterializationUsage {
    pub(crate) allocated_bytes: usize,
    pub(crate) nested_output_records: usize,
    pub(crate) peak_scratch_records: usize,
}

/// A successfully materialized report with the storage it took.
///
/// Take the pair apart with `into_parts`, or read each half through `report` and `usage`.
#[derive(Debug)]
pub struct HtmlAutocompleteMaterialized {
    pub(crate) report: HtmlAutocompleteReport,
    pub(crate) usage: HtmlAutocompleteMaterializationUsage,
}

/// Why a controlled materialization stopped. Every variant ends the operation and discards its
/// partial storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteMaterializationError {
    /// The raw value was refused by the interpreter's per-value UTF-16 admission cap before any
    /// storage was taken. The cause is returned from `source`.
    RawAdmission(HtmlValueAdmissionError),
    /// Growth would take the total allocated bytes past the byte allowance.
    BytesLimit {
        /// The byte allowance the session was given.
        limit: usize,
        /// The total allocated bytes the growth would have reached.
        required: usize,
    },
    /// Growth would take the nested output records past the output allowance.
    OutputLimit {
        /// The output record allowance the session was given.
        limit: usize,
        /// The total output records the growth would have reached.
        required: usize,
    },
    /// Growth would take the live scratch records past the scratch allowance.
    ScratchLimit {
        /// The scratch record allowance the session was given.
        limit: usize,
        /// The total live scratch records the growth would have reached.
        required: usize,
    },
    /// The allocator refused to reserve the storage.
    Allocation {
        /// The number of bytes of growth that the allocator was asked for.
        requested_bytes: usize,
    },
    /// A size computation overflowed `usize` or exceeded `isize::MAX` bytes, so the request cannot
    /// be represented on this platform.
    Representation,
    /// The ledger scope that enforces the allowances reported a state its own sequencing rules
    /// out.
    LedgerState,
}

/// A refused materialization with the storage it had taken when it stopped.
///
/// The refusal carries recoverable values, so it is a carrier: `into_parts` hands back the cause
/// and the usage.
#[derive(Debug)]
pub struct HtmlAutocompleteMaterializationFailure {
    pub(crate) kind: HtmlAutocompleteMaterializationError,
    pub(crate) usage: HtmlAutocompleteMaterializationUsage,
}

impl HtmlAutocompleteMaterializationLimits {
    /// Builds a set of allowances. A zero allowance admits no growth of that kind.
    pub const fn new(
        allocated_bytes: usize,
        nested_output_records: usize,
        scratch_records: usize,
    ) -> Self {
        Self {
            allocated_bytes,
            nested_output_records,
            scratch_records,
        }
    }
    /// Returns the remaining byte allowance for every variable-size buffer the materialization
    /// grows.
    pub const fn allocated_bytes(self) -> usize {
        self.allocated_bytes
    }
    /// Returns the remaining allowance of records in the output's nested buffers.
    pub const fn nested_output_records(self) -> usize {
        self.nested_output_records
    }
    /// Returns the remaining allowance of live records in temporary buffers that do not escape.
    pub const fn scratch_records(self) -> usize {
        self.scratch_records
    }
}
impl HtmlAutocompleteMaterializationUsage {
    /// Returns the bytes of buffer growth observed so far, output and scratch together.
    pub const fn allocated_bytes(self) -> usize {
        self.allocated_bytes
    }
    /// Returns the records of growth observed in the output's nested buffers.
    pub const fn nested_output_records(self) -> usize {
        self.nested_output_records
    }
    /// Returns the largest number of scratch records that were live at once.
    pub const fn peak_scratch_records(self) -> usize {
        self.peak_scratch_records
    }
}
impl HtmlAutocompleteMaterialized {
    /// Returns the materialized report.
    pub fn report(&self) -> &HtmlAutocompleteReport {
        &self.report
    }
    /// Returns the storage the materialization took.
    pub const fn usage(&self) -> HtmlAutocompleteMaterializationUsage {
        self.usage
    }
    /// Splits the result into the report and its usage.
    pub fn into_parts(self) -> (HtmlAutocompleteReport, HtmlAutocompleteMaterializationUsage) {
        (self.report, self.usage)
    }
}
impl HtmlAutocompleteMaterializationFailure {
    /// Returns why the materialization stopped.
    pub const fn kind(&self) -> HtmlAutocompleteMaterializationError {
        self.kind
    }
    /// Returns the storage taken before the materialization stopped, including growth that was
    /// rejected after being observed.
    pub const fn usage(&self) -> HtmlAutocompleteMaterializationUsage {
        self.usage
    }
    /// Splits the failure into its cause and usage.
    pub fn into_parts(
        self,
    ) -> (
        HtmlAutocompleteMaterializationError,
        HtmlAutocompleteMaterializationUsage,
    ) {
        (self.kind, self.usage)
    }
}
impl fmt::Display for HtmlAutocompleteMaterializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawAdmission(_) => {
                f.write_str("autocomplete value was refused by raw-value admission")
            }
            Self::BytesLimit { limit, required } => write!(
                f,
                "autocomplete materialization needs {required} allocated bytes, exceeding the limit of {limit}"
            ),
            Self::OutputLimit { limit, required } => write!(
                f,
                "autocomplete materialization needs {required} nested output records, exceeding the limit of {limit}"
            ),
            Self::ScratchLimit { limit, required } => write!(
                f,
                "autocomplete materialization needs {required} scratch records, exceeding the limit of {limit}"
            ),
            Self::Allocation { requested_bytes } => write!(
                f,
                "allocating {requested_bytes} bytes for autocomplete materialization failed"
            ),
            Self::Representation => f.write_str(
                "autocomplete materialization size cannot be represented on this platform",
            ),
            Self::LedgerState => f.write_str(
                "the ledger scope that enforces the allowances reported an impossible state",
            ),
        }
    }
}
impl From<HtmlValueAdmissionError> for HtmlAutocompleteMaterializationError {
    fn from(error: HtmlValueAdmissionError) -> Self {
        Self::RawAdmission(error)
    }
}
impl Error for HtmlAutocompleteMaterializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RawAdmission(error) => Some(error),
            _ => None,
        }
    }
}
impl fmt::Display for HtmlAutocompleteMaterializationFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("autocomplete materialization failed")
    }
}
impl Error for HtmlAutocompleteMaterializationFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.kind)
    }
}

#[derive(Clone, Copy)]
enum StoragePurpose {
    Output,
    Scratch,
}

/// One reservation owner used by both interpreter front doors. Both reserve with the fallible
/// allocator and report a refusal; standalone sets no resource cap, so the allocator and the
/// platform are its only limits.
///
/// The allowances are the caps of one ledger scope. A second, uncapped scope records what was
/// actually taken, so growth the allocator made beyond a cap is observed before it is refused.
/// The typed usage is read back from that second scope.
struct Materialization {
    operation: Operation<HtmlWork, HtmlResource>,
    /// The scope that carries the allowances.
    caps: Scope,
    /// The scope that records the observed growth.
    observed: Scope,
    usage: HtmlAutocompleteMaterializationUsage,
    /// The live scratch records read from the observed scope.
    scratch: usize,
}

impl StoragePurpose {
    fn resource(self) -> HtmlResource {
        match self {
            Self::Output => HtmlResource::NestedOutputRecords,
            Self::Scratch => HtmlResource::ScratchRecords,
        }
    }
}

impl Materialization {
    fn new(
        limits: Option<HtmlAutocompleteMaterializationLimits>,
    ) -> Result<Self, HtmlAutocompleteMaterializationError> {
        use HtmlAutocompleteMaterializationError as E;
        let allocation = || E::Allocation {
            requested_bytes: std::mem::size_of::<Operation<HtmlWork, HtmlResource>>(),
        };
        let mut operation = Operation::new(
            HtmlCapScope::Autocomplete.name(),
            HtmlLimits::unlimited(),
            3,
        )
        .map_err(|_| allocation())?;
        let root = operation.root();
        let caps = operation
            .child(
                root,
                HtmlCapScope::Autocomplete.name(),
                limits.map_or_else(HtmlLimits::unlimited, autocomplete_limits),
                HtmlTariffs::identity(),
            )
            .map_err(|_| allocation())?;
        let observed = operation
            .child(
                root,
                "html-autocomplete-observed",
                HtmlLimits::unlimited(),
                HtmlTariffs::identity(),
            )
            .map_err(|_| allocation())?;
        Ok(Self {
            operation,
            caps,
            observed,
            usage: HtmlAutocompleteMaterializationUsage::default(),
            scratch: 0,
        })
    }

    /// The materialization error of a ledger error: a refused allowance keeps its limit and the
    /// total the growth would have reached.
    fn refused(error: LedgerError<HtmlWork, HtmlResource>) -> HtmlAutocompleteMaterializationError {
        use HtmlAutocompleteMaterializationError as E;
        let LedgerError::Refused(refusal) = error else {
            return E::LedgerState;
        };
        let (RefusalReason::Limit { limit }, Change::Acquire(resource, amount)) =
            (refusal.reason, refusal.change)
        else {
            return match refusal.reason {
                RefusalReason::Overflow => E::Representation,
                _ => E::LedgerState,
            };
        };
        let (Ok(limit), Some(required)) = (
            usize::try_from(limit),
            refusal
                .current
                .checked_add(amount)
                .and_then(|required| usize::try_from(required).ok()),
        ) else {
            return E::Representation;
        };
        match resource {
            HtmlResource::AllocatedBytes => E::BytesLimit { limit, required },
            HtmlResource::NestedOutputRecords => E::OutputLimit { limit, required },
            HtmlResource::ScratchRecords => E::ScratchLimit { limit, required },
            _ => E::LedgerState,
        }
    }

    /// The growth of one purpose as one batch: bytes first, then records.
    fn growth_batch(growth: usize, bytes: usize, purpose: StoragePurpose) -> [HtmlChange; 2] {
        [
            Change::Acquire(HtmlResource::AllocatedBytes, wide(bytes)),
            Change::Acquire(purpose.resource(), wide(growth)),
        ]
    }

    /// Reads the typed usage and the live scratch from the observed scope.
    fn read(&mut self) -> Result<(), HtmlAutocompleteMaterializationError> {
        use HtmlAutocompleteMaterializationError as E;
        let usage = self
            .operation
            .usage(self.observed)
            .map_err(|_| E::LedgerState)?;
        let read = |value: u64| usize::try_from(value).map_err(|_| E::Representation);
        self.usage = HtmlAutocompleteMaterializationUsage {
            allocated_bytes: read(usage.live(HtmlResource::AllocatedBytes))?,
            nested_output_records: read(usage.live(HtmlResource::NestedOutputRecords))?,
            peak_scratch_records: read(usage.peak(HtmlResource::ScratchRecords))?,
        };
        self.scratch = read(usage.live(HtmlResource::ScratchRecords))?;
        Ok(())
    }

    /// Gives back the live scratch records, which are local to one processing step.
    fn release_scratch(&mut self) -> Result<(), HtmlAutocompleteMaterializationError> {
        let release = [Change::Release(
            HtmlResource::ScratchRecords,
            wide(self.scratch),
        )];
        for scope in [self.observed, self.caps] {
            self.operation
                .apply_batch(scope, &release)
                .map_err(Self::refused)?;
        }
        self.read()
    }

    fn reserve<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
        purpose: StoragePurpose,
    ) -> Result<(), HtmlAutocompleteMaterializationError> {
        use HtmlAutocompleteMaterializationError as E;
        let needed = values
            .len()
            .checked_add(additional)
            .ok_or(E::Representation)?;
        if needed <= values.capacity() {
            return Ok(());
        }
        let growth = needed - values.capacity();
        let bytes = growth
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(E::Representation)?;
        if needed
            .checked_mul(std::mem::size_of::<T>())
            .is_none_or(|bytes| bytes > isize::MAX as usize)
        {
            return Err(E::Representation);
        }
        self.check_growth(growth, bytes, purpose)?;
        let before = values.capacity();
        #[cfg(test)]
        if crate::internal::amortized::fault::refuses_next_reservation() {
            return Err(E::Allocation {
                requested_bytes: bytes,
            });
        }
        values
            .try_reserve_exact(needed - values.len())
            .map_err(|_| E::Allocation {
                requested_bytes: bytes,
            })?;
        self.observe_growth(
            values.capacity() - before,
            std::mem::size_of::<T>(),
            purpose,
        )
    }

    fn check_growth(
        &mut self,
        growth: usize,
        bytes: usize,
        purpose: StoragePurpose,
    ) -> Result<(), HtmlAutocompleteMaterializationError> {
        self.operation
            .check_batch(self.caps, &Self::growth_batch(growth, bytes, purpose))
            .map_err(Self::refused)
    }

    /// Observes the growth the allocator actually made, before the allowances can refuse it: the
    /// observed scope takes it unconditionally, then the capped scope admits or refuses it.
    fn observe_growth(
        &mut self,
        growth: usize,
        item_bytes: usize,
        purpose: StoragePurpose,
    ) -> Result<(), HtmlAutocompleteMaterializationError> {
        use HtmlAutocompleteMaterializationError as E;
        let bytes = growth.checked_mul(item_bytes).ok_or(E::Representation)?;
        let batch = Self::growth_batch(growth, bytes, purpose);
        self.operation
            .apply_batch(self.observed, &batch)
            .map_err(Self::refused)?;
        self.read()?;
        self.operation
            .apply_batch(self.caps, &batch)
            .map_err(Self::refused)
    }

    fn keyword(
        &mut self,
        keyword: &'static str,
    ) -> Result<DocumentString, HtmlAutocompleteMaterializationError> {
        let mut output = Vec::new();
        self.reserve(&mut output, keyword.len(), StoragePurpose::Output)?;
        output.extend(keyword.bytes().map(u16::from));
        Ok(DocumentString::from_utf16(output))
    }

    fn tokens(
        &mut self,
        input: &DocumentString,
    ) -> Result<HtmlSpaceSeparatedTokens, HtmlAutocompleteMaterializationError> {
        let split = crate::internal::microsyntax::tokens::space_tokens(input);
        let mut tokens = Vec::new();
        self.reserve(&mut tokens, split.clone().count(), StoragePurpose::Output)?;
        for token in split {
            tokens.push(token.copy(false, |units, count| {
                self.reserve(units, count, StoragePurpose::Output)
            })?);
        }
        Ok(HtmlSpaceSeparatedTokens::from_vec(tokens))
    }

    fn prepend(
        &mut self,
        tokens: &mut Vec<HtmlSpaceToken>,
        token: HtmlSpaceToken,
        purpose: StoragePurpose,
    ) -> Result<(), HtmlAutocompleteMaterializationError> {
        self.reserve(tokens, 1, purpose)?;
        tokens.insert(0, token);
        Ok(())
    }

    fn idl(
        &mut self,
        tokens: &[HtmlSpaceToken],
    ) -> Result<DocumentString, HtmlAutocompleteMaterializationError> {
        let units = tokens
            .iter()
            .try_fold(tokens.len().saturating_sub(1), |length, token| {
                length.checked_add(token.value().len())
            })
            .ok_or(HtmlAutocompleteMaterializationError::Representation)?;
        let mut output = Vec::new();
        self.reserve(&mut output, units, StoragePurpose::Output)?;
        for (index, token) in tokens.iter().enumerate() {
            if index != 0 {
                output.push(0x20);
            }
            output.extend_from_slice(token.value().code_units());
        }
        Ok(DocumentString::from_utf16(output))
    }
}

/// Whether an autofill field is a normal field or a contact field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofillFieldCategory {
    /// A field that does not admit a contact hint.
    Normal,
    /// A telephone, email or instant-messaging field, which alone admits a contact hint such as
    /// `home`.
    Contact,
}

/// The author control group in HTML's defining autofill-field table. The table admits `textarea`
/// and `select` elements in every group, so a group only limits which `input` states are admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofillControlGroup {
    /// Admits `input` controls in the Hidden, Text and Search states.
    Text,
    /// Admits `input` controls in the Hidden state only, for text that keeps newlines. A Text or
    /// Search `input` cannot hold newlines, so it is not admitted.
    Multiline,
    /// Admits the `input` states of the Text group and the Password state.
    Password,
    /// Admits the `input` states of the Text group and the URL state.
    Url,
    /// Admits the `input` states of the Text group and the Email state.
    Username,
    /// Admits the `input` states of the Text group and the Telephone state.
    Tel,
    /// Admits the `input` states of the Text group and the Number state.
    Numeric,
    /// Admits the `input` states of the Text group and the Month state.
    Month,
    /// Admits the `input` states of the Text group and the Date state.
    Date,
}

impl HtmlAutofillControlGroup {
    fn allows_input(self, state: HtmlInputType) -> bool {
        use HtmlInputType::{Hidden, Search, Text};
        if state == Hidden {
            return true;
        }
        if self == Self::Multiline {
            return false;
        }
        if matches!(state, Text | Search) {
            return true;
        }
        matches!(
            (self, state),
            (Self::Password, HtmlInputType::Password)
                | (Self::Url, HtmlInputType::Url)
                | (Self::Username, HtmlInputType::Email)
                | (Self::Tel, HtmlInputType::Telephone)
                | (Self::Numeric, HtmlInputType::Number)
                | (Self::Month, HtmlInputType::Month)
                | (Self::Date, HtmlInputType::Date)
        )
    }
}

macro_rules! fields {
    ($($(#[$vmeta:meta])* $variant:ident => ($keyword:literal, $category:ident, $group:ident)),+ $(,)?) => {
        /// Native detail-field vocabulary, excluding on/off and credential fields.
        ///
        /// Each variant is one autofill field name from the pinned autofill field table. The
        /// canonical keyword is its lowercase spelling, and an author may write it with any ASCII
        /// case. The enum is closed by that table; a new edition of the standard is a deliberate
        /// breaking change.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum HtmlAutofillField { $($(#[$vmeta])* $variant),+ }
        impl HtmlAutofillField {
            const ALL: &'static [Self] = &[$(Self::$variant),+];
            /// Returns the field name in the lowercase spelling of the pinned table.
            pub fn canonical_keyword(self) -> &'static str {
                self.declared().as_str()
            }
            /// The field name, checked while compiling to be one space token.
            fn declared(self) -> DeclaredKeyword {
                match self { $(Self::$variant => declared_keyword!($keyword)),+ }
            }
            /// Returns whether the field is a normal field or a contact field, which alone admits a
            /// contact hint such as `home` or `work`.
            pub fn category(self) -> HtmlAutofillFieldCategory {
                match self { $(Self::$variant => HtmlAutofillFieldCategory::$category),+ }
            }
            /// Returns the group of controls that the table permits to carry this field.
            pub fn control_group(self) -> HtmlAutofillControlGroup {
                match self { $(Self::$variant => HtmlAutofillControlGroup::$group),+ }
            }
            /// Returns the URL of the field's definition in the pinned edition.
            pub fn source_clause(self) -> &'static str {
                match self { $(Self::$variant => concat!(
                    "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fe-autocomplete-",
                    $keyword)),+ }
            }
            fn from_keyword(value: &DocumentString) -> Option<Self> {
                Self::ALL.iter().copied().find(|field| ascii_keyword(value, field.canonical_keyword()))
            }
        }
    };
}

// One source-backed definition owns keyword, category and author control group.
fields! {
    /// Full name.
    Name => ("name", Normal, Text),
    /// Prefix or title, such as "Mr.", "Ms." or "Dr.".
    HonorificPrefix => ("honorific-prefix", Normal, Text),
    /// Given name, also known in some Western cultures as the first name.
    GivenName => ("given-name", Normal, Text),
    /// Additional names, also known in some Western cultures as middle names.
    AdditionalName => ("additional-name", Normal, Text),
    /// Family name, also known in some Western cultures as the last name or surname.
    FamilyName => ("family-name", Normal, Text),
    /// Suffix, such as "Jr.", "B.Sc." or "II".
    HonorificSuffix => ("honorific-suffix", Normal, Text),
    /// Nickname, screen name or handle: a typically short name used instead of the full name.
    Nickname => ("nickname", Normal, Text),
    /// Job title, such as "Software Engineer".
    OrganizationTitle => ("organization-title", Normal, Text),
    /// A username.
    Username => ("username", Normal, Username),
    /// A new password, for example when creating an account or changing a password.
    NewPassword => ("new-password", Normal, Password),
    /// The current password for the account that the username field identifies.
    CurrentPassword => ("current-password", Normal, Password),
    /// A one-time code used for verifying the user's identity.
    OneTimeCode => ("one-time-code", Normal, Password),
    /// Company name corresponding to the person, address or contact information in the other fields
    /// associated with this field.
    Organization => ("organization", Normal, Text),
    /// Street address over multiple lines, with newlines preserved.
    StreetAddress => ("street-address", Normal, Multiline),
    /// Street address, one line per field: the first line.
    AddressLine1 => ("address-line1", Normal, Text),
    /// Street address, one line per field: the second line.
    AddressLine2 => ("address-line2", Normal, Text),
    /// Street address, one line per field: the third line.
    AddressLine3 => ("address-line3", Normal, Text),
    /// The most fine-grained administrative level, in addresses with four administrative levels.
    AddressLevel4 => ("address-level4", Normal, Text),
    /// The third administrative level, in addresses with three or more administrative levels.
    AddressLevel3 => ("address-level3", Normal, Text),
    /// The second administrative level, in addresses with two or more administrative levels; in
    /// countries with two levels this is typically the city, town or village.
    AddressLevel2 => ("address-level2", Normal, Text),
    /// The broadest administrative level in the address, such as the state in the US or the canton
    /// in Switzerland.
    AddressLevel1 => ("address-level1", Normal, Text),
    /// Country code, a valid ISO 3166-1 alpha-2 code.
    Country => ("country", Normal, Text),
    /// Country name.
    CountryName => ("country-name", Normal, Text),
    /// Postal code, post code, ZIP code or CEDEX code.
    PostalCode => ("postal-code", Normal, Text),
    /// Full name as given on the payment instrument.
    CcName => ("cc-name", Normal, Text),
    /// Given name as given on the payment instrument.
    CcGivenName => ("cc-given-name", Normal, Text),
    /// Additional names given on the payment instrument.
    CcAdditionalName => ("cc-additional-name", Normal, Text),
    /// Family name given on the payment instrument.
    CcFamilyName => ("cc-family-name", Normal, Text),
    /// Code identifying the payment instrument, such as the credit card number; ASCII digits.
    CcNumber => ("cc-number", Normal, Text),
    /// Expiration date of the payment instrument, as a valid month string.
    CcExp => ("cc-exp", Normal, Month),
    /// Month component of the expiration date of the payment instrument, an integer from 1 to 12.
    CcExpMonth => ("cc-exp-month", Normal, Numeric),
    /// Year component of the expiration date of the payment instrument, an integer greater than
    /// zero.
    CcExpYear => ("cc-exp-year", Normal, Numeric),
    /// Security code for the payment instrument, also known as the CSC, CVC or CVV; ASCII digits.
    CcCsc => ("cc-csc", Normal, Text),
    /// Type of payment instrument.
    CcType => ("cc-type", Normal, Text),
    /// The currency that the user would prefer the transaction to use, an ISO 4217 currency code.
    TransactionCurrency => ("transaction-currency", Normal, Text),
    /// The amount that the user would like for the transaction, a valid floating-point number.
    TransactionAmount => ("transaction-amount", Normal, Numeric),
    /// Preferred language, a valid BCP 47 language tag.
    Language => ("language", Normal, Text),
    /// Birthday, as a valid date string.
    Bday => ("bday", Normal, Date),
    /// Day component of the birthday, an integer from 1 to 31.
    BdayDay => ("bday-day", Normal, Numeric),
    /// Month component of the birthday, an integer from 1 to 12.
    BdayMonth => ("bday-month", Normal, Numeric),
    /// Year component of the birthday, an integer greater than zero.
    BdayYear => ("bday-year", Normal, Numeric),
    /// Gender identity.
    Sex => ("sex", Normal, Text),
    /// Home page or other web page corresponding to the company, person, address or contact
    /// information in the other fields associated with this field.
    Url => ("url", Normal, Url),
    /// Photograph, icon or other image corresponding to the company, person, address or contact
    /// information in the other fields associated with this field.
    Photo => ("photo", Normal, Url),
    /// Full telephone number including the country code, a contact field.
    Tel => ("tel", Contact, Tel),
    /// Country code component of the telephone number, a contact field.
    TelCountryCode => ("tel-country-code", Contact, Text),
    /// Telephone number without the country code component, a contact field.
    TelNational => ("tel-national", Contact, Text),
    /// Area code component of the telephone number, a contact field.
    TelAreaCode => ("tel-area-code", Contact, Text),
    /// Telephone number without the country code and area code components, a contact field.
    TelLocal => ("tel-local", Contact, Text),
    /// First part of the component of the telephone number that follows the area code, when that
    /// component is split into two, a contact field.
    TelLocalPrefix => ("tel-local-prefix", Contact, Text),
    /// Second part of the component of the telephone number that follows the area code, when that
    /// component is split into two, a contact field.
    TelLocalSuffix => ("tel-local-suffix", Contact, Text),
    /// Telephone number internal extension code, a contact field.
    TelExtension => ("tel-extension", Contact, Text),
    /// Email address, a contact field.
    Email => ("email", Contact, Username),
    /// URL representing an instant messaging protocol endpoint, a contact field.
    Impp => ("impp", Contact, Url),
}

macro_rules! hint_type {
    ($(#[$meta:meta])* $name:ident, [$($(#[$vmeta:meta])* $variant:ident => $keyword:literal),+]) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name { $($(#[$vmeta])* $variant),+ }
        impl $name {
            const ALL: &'static [Self] = &[$(Self::$variant),+];
            /// Returns the keyword in its lowercase spelling.
            pub fn canonical_keyword(self) -> &'static str {
                self.declared().as_str()
            }
            /// The keyword, checked while compiling to be one space token.
            fn declared(self) -> DeclaredKeyword {
                match self { $(Self::$variant => declared_keyword!($keyword)),+ }
            }
            fn from_keyword(value: &DocumentString) -> Option<Self> {
                Self::ALL.iter().copied().find(|hint| ascii_keyword(value, hint.canonical_keyword()))
            }
        }
    };
}

hint_type!(
    /// The mode token of an `autocomplete` value, which says which kind of address or contact
    /// information the user agent is to look at, shipping or billing. It is a typed hint and
    /// joins the scope.
    HtmlAutocompleteMode,
    [
        /// The `shipping` token: the address or contact information is the shipping kind.
        Shipping => "shipping",
        /// The `billing` token: the address or contact information is the billing kind.
        Billing => "billing"
    ]
);
hint_type!(
    /// The contact hint token of an `autocomplete` value, which says which of the person's
    /// telephone or messaging contacts a contact field expects. Only a contact field admits it.
    HtmlAutocompleteContactHint,
    [
        /// The `home` token: the contact is the person's home contact.
        Home => "home",
        /// The `work` token: the contact is the person's work contact.
        Work => "work",
        /// The `mobile` token: the contact is the person's mobile contact.
        Mobile => "mobile",
        /// The `fax` token: the contact is the person's fax line.
        Fax => "fax",
        /// The `pager` token: the contact is the person's pager.
        Pager => "pager"
    ]
);

/// A non-autofill credential type that an `autocomplete` value can request. The enum is closed by
/// the pinned standard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteCredential {
    /// The `webauthn` token: selecting a public key credential resolves a pending conditional
    /// mediation request instead of autofilling the field.
    Webauthn,
}
impl HtmlAutocompleteCredential {
    /// Returns the credential token in its lowercase spelling, `webauthn`.
    pub fn canonical_keyword(self) -> &'static str {
        WEBAUTHN
    }
}

/// A grammar-valid section token; suffix text remains exact, including UTF-16.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAutocompleteSection {
    token: HtmlSpaceToken,
}
/// Why a string is not an `autocomplete` section token. The enum is `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlAutocompleteSectionError {
    /// The string is not a valid space token. The grammar error is returned from `source`.
    Token(HtmlTokenError),
    /// The token does not begin with `section-` (ASCII case-insensitive).
    MissingSectionPrefix,
}
impl fmt::Display for HtmlAutocompleteSectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Token(_) => "autocomplete section is not a valid space-separated token",
            Self::MissingSectionPrefix => "autocomplete section does not start with \"section-\"",
        })
    }
}
impl From<HtmlTokenError> for HtmlAutocompleteSectionError {
    fn from(error: HtmlTokenError) -> Self {
        Self::Token(error)
    }
}
impl Error for HtmlAutocompleteSectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Token(error) => Some(error),
            Self::MissingSectionPrefix => None,
        }
    }
}

fn section_prefix(value: &DocumentString) -> bool {
    value.code_units().get(..8).is_some_and(|units| {
        units
            .iter()
            .zip(b"section-")
            .all(|(&unit, &byte)| ascii_case_insensitive_unit(unit, byte))
    })
}

impl HtmlAutocompleteSection {
    /// Requires the section- prefix and one space token, with no suffix grammar.
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlAutocompleteSectionError> {
        let token = HtmlSpaceToken::new(value)?;
        if !section_prefix(token.value()) {
            return Err(HtmlAutocompleteSectionError::MissingSectionPrefix);
        }
        Ok(Self { token })
    }
    /// Returns the exact token text, including its `section-` prefix and its case.
    pub fn value(&self) -> &DocumentString {
        self.token.value()
    }
}

/// A contact hint was attached to a field that is not a contact field. The accessor names the
/// field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlAutocompleteContactFieldError {
    field: HtmlAutofillField,
}
impl HtmlAutocompleteContactFieldError {
    /// Returns the field that rejected the hint.
    pub fn field(self) -> HtmlAutofillField {
        self.field
    }
}
impl fmt::Display for HtmlAutocompleteContactFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "autocomplete field \"{}\" does not admit a contact hint",
            self.field.canonical_keyword()
        )
    }
}
impl Error for HtmlAutocompleteContactFieldError {}

/// Native ordered detail syntax. Actual element/control suitability is separate.
/// A real detail field is required; bare webauthn cannot be constructed here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAutocompleteDetails {
    section: Option<HtmlAutocompleteSection>,
    mode: Option<HtmlAutocompleteMode>,
    contact: Option<HtmlAutocompleteContactHint>,
    field: HtmlAutofillField,
    webauthn: bool,
}
impl HtmlAutocompleteDetails {
    /// Starts a value from its required detail field, with no section, mode, contact hint or
    /// `webauthn` token.
    pub fn new(field: HtmlAutofillField) -> Self {
        Self {
            section: None,
            mode: None,
            contact: None,
            field,
            webauthn: false,
        }
    }
    /// Sets the section token, which comes first in the ordered value.
    pub fn with_section(mut self, section: HtmlAutocompleteSection) -> Self {
        self.section = Some(section);
        self
    }
    /// Sets the shipping or billing mode token.
    pub fn with_mode(mut self, mode: HtmlAutocompleteMode) -> Self {
        self.mode = Some(mode);
        self
    }
    /// Sets the contact hint. Fails with the field when the field is not a contact field.
    pub fn with_contact_hint(
        mut self,
        hint: HtmlAutocompleteContactHint,
    ) -> Result<Self, HtmlAutocompleteContactFieldError> {
        if self.field.category() != HtmlAutofillFieldCategory::Contact {
            return Err(HtmlAutocompleteContactFieldError { field: self.field });
        }
        self.contact = Some(hint);
        Ok(self)
    }
    /// Appends the `webauthn` credential token after the field.
    pub fn with_webauthn(mut self) -> Self {
        self.webauthn = true;
        self
    }
    /// Returns the section token, if one was set.
    pub fn section(&self) -> Option<&HtmlAutocompleteSection> {
        self.section.as_ref()
    }
    /// Returns the shipping or billing mode, if one was set.
    pub fn mode(&self) -> Option<HtmlAutocompleteMode> {
        self.mode
    }
    /// Returns the contact hint, if one was set. It is only ever set for a contact field.
    pub fn contact_hint(&self) -> Option<HtmlAutocompleteContactHint> {
        self.contact
    }
    /// Returns the detail field, which is always present.
    pub fn field(&self) -> HtmlAutofillField {
        self.field
    }
    /// Returns whether the value ends with the `webauthn` token.
    pub fn has_webauthn(&self) -> bool {
        self.webauthn
    }
    /// Builds the no-namespace `autocomplete` attribute. Tokens appear in the order section, mode,
    /// contact hint, field, `webauthn`, joined by one U+0020 each, with canonical lowercase
    /// keywords and the section as supplied.
    pub fn into_attribute(self) -> Attribute {
        let mut tokens = Vec::new();
        if let Some(section) = self.section {
            tokens.push(section.token);
        }
        if let Some(mode) = self.mode {
            tokens.push(keyword_token(mode.declared()));
        }
        if let Some(contact) = self.contact {
            tokens.push(keyword_token(contact.declared()));
        }
        tokens.push(keyword_token(self.field.declared()));
        if self.webauthn {
            tokens.push(keyword_token(WEBAUTHN_KEYWORD));
        }
        Attribute::new(
            KnownHtmlAttribute::Autocomplete.qualified_name(),
            HtmlSpaceSeparatedTokens::new(tokens).as_document_string(),
        )
    }
}

/// A natively constructed `autocomplete` value: either the on/off setting or ordered detail tokens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteValue {
    /// The single `on` or `off` token.
    Setting(HtmlFormAutocomplete),
    /// Autofill detail tokens that name a field.
    Details(HtmlAutocompleteDetails),
}
impl HtmlAutocompleteValue {
    /// Builds the no-namespace `autocomplete` attribute in its canonical spelling.
    pub fn into_attribute(self) -> Attribute {
        match self {
            Self::Setting(setting) => setting.into_attribute(),
            Self::Details(details) => details.into_attribute(),
        }
    }
}

/// Why an `autocomplete` value is not valid ordered expectation syntax. The enum is
/// `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlAutocompleteSyntaxError {
    /// The value has no tokens, because it is empty or only ASCII whitespace.
    EmptyTokens,
    /// Only a section, mode or contact hint is present, so no field token follows.
    MissingDetailField,
    /// The token where the detail field belongs is not a recognised autofill field name.
    UnrecognizedDetailField {
        /// Zero-based index of the offending token.
        token_index: usize,
    },
    /// A contact hint precedes a field that is not a contact field.
    ContactHintRequiresContactField {
        /// The recognised field that cannot take a contact hint.
        field: HtmlAutofillField,
    },
    /// A token remains after a complete, ordered value.
    UnexpectedToken {
        /// Zero-based index of the first leftover token.
        token_index: usize,
    },
}
impl fmt::Display for HtmlAutocompleteSyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTokens => f.write_str("autocomplete value has no tokens"),
            Self::MissingDetailField => f.write_str("autocomplete value has no detail field"),
            Self::UnrecognizedDetailField { token_index } => write!(
                f,
                "autocomplete token at index {token_index} is not a recognized detail field"
            ),
            Self::ContactHintRequiresContactField { field } => write!(
                f,
                "autocomplete contact hint requires a contact field, but \"{}\" is not one",
                field.canonical_keyword()
            ),
            Self::UnexpectedToken { token_index } => write!(
                f,
                "autocomplete token at index {token_index} is not expected at that position"
            ),
        }
    }
}
impl Error for HtmlAutocompleteSyntaxError {}
/// The ordered expectation syntax judgment of an `autocomplete` value, independent of the element,
/// the input state and the mantle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteSyntaxState {
    /// The attribute is absent.
    Missing,
    /// Ordered expectation syntax only; mantle/control authorship is separate.
    Valid(HtmlAutocompleteValue),
    /// The value violates the ordered expectation syntax; the payload says how.
    Invalid(HtmlAutocompleteSyntaxError),
    /// The rule does not apply to these names, so no syntax was examined.
    Unapplied,
}

/// The role the attribute plays, which the actual element and input state decide. The role decides
/// which values are valid and how processing treats them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteMantle {
    /// The autofill expectation mantle: the attribute describes what input the user is expected to
    /// enter. It applies to every element except a Hidden `input`.
    Expectation,
    /// The autofill anchor mantle: the attribute describes the meaning of the given value. It
    /// applies to an `input` in the Hidden state.
    Anchor,
}

/// Independent context violations; a forbidden input state can also have an
/// inappropriate field. No violation overwrites a different author observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlAutocompleteContextViolations {
    prohibited_input_state: Option<HtmlInputType>,
    anchor_setting: Option<HtmlFormAutocomplete>,
    inappropriate_field: Option<HtmlAutofillField>,
    webauthn_on_select: bool,
}
impl HtmlAutocompleteContextViolations {
    /// Returns the supplied input state if the attribute is not allowed on it (Checkbox, Radio,
    /// File Upload or a button state).
    pub fn prohibited_input_state(self) -> Option<HtmlInputType> {
        self.prohibited_input_state
    }
    /// Returns the `on` or `off` setting if it is written on a Hidden `input`, where only detail
    /// tokens are allowed.
    pub fn anchor_setting(self) -> Option<HtmlFormAutocomplete> {
        self.anchor_setting
    }
    /// Returns the detail field if the supplied input state is not in the field's control group.
    pub fn inappropriate_field(self) -> Option<HtmlAutofillField> {
        self.inappropriate_field
    }
    /// Returns whether the value ends with `webauthn` on a `select` element, which does not admit
    /// it.
    pub fn webauthn_on_select(self) -> bool {
        self.webauthn_on_select
    }
    fn is_empty(self) -> bool {
        self.prohibited_input_state.is_none()
            && self.anchor_setting.is_none()
            && self.inappropriate_field.is_none()
            && !self.webauthn_on_select
    }
}
/// Independent author requirements that depend on the actual element and input state.
///
/// They never replace the syntax judgment: a value can be lexically valid and still violate its
/// context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteContextAuthorState {
    /// The attribute is absent, so no context requirement applies.
    NotRequired,
    /// The element is an `input` and no actual input state was supplied, so context cannot be
    /// judged.
    Unchecked,
    /// No selected context violation; this does not certify lexical conformance.
    Conforming,
    /// At least one context requirement fails; the payload lists each independently.
    Nonconforming(HtmlAutocompleteContextViolations),
    /// The rule does not apply to these names, so no context was judged.
    Unapplied,
}

/// Processing permits the credential field identity separately from author fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofillFieldIdentity {
    /// The final token is the `on` or `off` setting.
    Setting(HtmlFormAutocomplete),
    /// The final token is an autofill detail field.
    Detail(HtmlAutofillField),
    /// The final token is a credential type, `webauthn`.
    Credential(HtmlAutocompleteCredential),
}

/// Assignment produced by the selected processing branch, independent of any
/// previous live credential state. Settings return without assigning this
/// property; successful detail processing assigns null or a credential.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteCredentialOutcome {
    /// The branch assigns no credential type, which is what a setting does.
    NotAssigned,
    /// The branch assigns a null credential type, which a detail field does.
    Null,
    /// The branch assigns the given credential type.
    Credential(HtmlAutocompleteCredential),
}

/// Literal pinned successful processing, with exact field/IDL token casing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAutocompleteProcessed {
    field_identity: HtmlAutofillFieldIdentity,
    field: DocumentString,
    mode: Option<HtmlAutocompleteMode>,
    contact: Option<HtmlAutocompleteContactHint>,
    scope: HtmlSpaceSeparatedTokens,
    credential: HtmlAutocompleteCredentialOutcome,
    idl_value: DocumentString,
}
impl HtmlAutocompleteProcessed {
    /// Returns what the final token identifies: a setting, a detail field or a credential.
    pub fn field_identity(&self) -> HtmlAutofillFieldIdentity {
        self.field_identity
    }
    /// Returns the final token's exact spelling: the detail field, or the credential token
    /// `webauthn` when one follows the detail field, or the canonical `on` or `off` for a setting.
    pub fn field_name(&self) -> &DocumentString {
        &self.field
    }
    /// Mode and contact are the complete typed hint set; section is scope only.
    pub fn mode_hint(&self) -> Option<HtmlAutocompleteMode> {
        self.mode
    }
    /// Returns the contact hint, which is only ever produced for a contact field.
    pub fn contact_hint(&self) -> Option<HtmlAutocompleteContactHint> {
        self.contact
    }
    /// Returns the autofill scope: the lowercased `section-` token, the mode and the contact hint,
    /// in that order, for those that are present.
    pub fn scope(&self) -> &HtmlSpaceSeparatedTokens {
        &self.scope
    }
    /// Returns the credential type that the processing branch assigned.
    pub fn credential_type(&self) -> HtmlAutocompleteCredentialOutcome {
        self.credential
    }
    /// Returns the IDL-exposed autofill value: the lowercased section, mode and contact hint tokens
    /// that are present, then the detail field when a `webauthn` token follows it, then the final
    /// token, each separated by one U+0020.
    pub fn idl_value(&self) -> &DocumentString {
        &self.idl_value
    }
}

/// Which step of the processing algorithm sent an `autocomplete` value to its default branch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteDefaultReason {
    /// The element has no `autocomplete` attribute.
    MissingAttribute,
    /// The value splits into no tokens.
    EmptyTokens,
    /// The last token is not a setting, a detail field or a credential type.
    UnrecognizedFinalToken,
    /// The last token is `on` or `off` but other tokens accompany it.
    SettingWithExtraTokens,
    /// The last token is `on` or `off` on an element that wears the anchor mantle.
    SettingOnAnchor,
    /// The last token is `webauthn` and the token before it is not a detail field.
    InvalidCredentialPrefix,
    /// More than one token remains before the field, hints and credential. Only a single `section-`
    /// token may precede them.
    ExtraLeadingTokens,
    /// The one remaining leading token does not begin with `section-` (ASCII case-insensitive).
    UnrecognizedLeadingToken,
}
/// What the field name is on the default branch. The standard sets it for an anchor and leaves it
/// to the form owner for an expectation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteDefaultField {
    /// The source defines an empty field on the anchor default branch.
    Empty,
    /// Only actual current form owner's On/Off, or actual null owner, resolves it.
    FormOwnerRequired,
}
/// Default branch has empty IDL/scope/hints and no produced credential. Its
/// expectation field remains unresolved until qualified owner integration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAutocompleteDefault {
    reason: HtmlAutocompleteDefaultReason,
    field: HtmlAutocompleteDefaultField,
    idl_value: DocumentString,
    scope: HtmlSpaceSeparatedTokens,
}
impl HtmlAutocompleteDefault {
    /// Returns why the default branch was taken.
    pub fn reason(&self) -> HtmlAutocompleteDefaultReason {
        self.reason
    }
    /// Returns whether the field name is empty or still needs the form owner.
    pub fn field(&self) -> HtmlAutocompleteDefaultField {
        self.field
    }
    /// Returns the IDL-exposed value, which is always empty on this branch.
    pub fn idl_value(&self) -> &DocumentString {
        &self.idl_value
    }
    /// Returns the autofill scope, which is always empty on this branch.
    pub fn scope(&self) -> &HtmlSpaceSeparatedTokens {
        &self.scope
    }
    /// The default branch defines an empty hint set.
    pub fn mode_hint(&self) -> Option<HtmlAutocompleteMode> {
        None
    }
    /// The default branch defines an empty hint set.
    pub fn contact_hint(&self) -> Option<HtmlAutocompleteContactHint> {
        None
    }
}
/// The outcome of the autofill processing algorithm for one `autocomplete` value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteProcessingState {
    /// The value was processed successfully.
    Processed(HtmlAutocompleteProcessed),
    /// The algorithm took its default branch; the payload says why.
    Defaulted(HtmlAutocompleteDefault),
    /// Actual input state is needed for both processing domain and mantle.
    InputContextRequired,
    /// Source gives autofill processing only to input states admitting autocomplete.
    InapplicableInputState(HtmlInputType),
    /// The rule does not apply to these names, so nothing was processed.
    Unapplied,
}

/// Exact raw envelope, ordered syntax, author context and independent processing.
/// Standalone input-unit admission is not parser work/allocation accounting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAutocompleteReport {
    input: HtmlAttributeReport<()>,
    tokens: Option<HtmlSpaceSeparatedTokens>,
    syntax: HtmlAutocompleteSyntaxState,
    mantle: Option<HtmlAutocompleteMantle>,
    context_author: HtmlAutocompleteContextAuthorState,
    processing: HtmlAutocompleteProcessingState,
}
impl HtmlAutocompleteReport {
    /// Returns the exact element name, including any stored prefix.
    pub fn element(&self) -> &QualifiedName {
        self.input.element()
    }
    /// Returns the exact supplied attribute name, prefix and value, if present.
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.input.raw_attribute()
    }
    /// Returns the exact supplied raw value, preserving absence versus empty.
    pub fn raw(&self) -> Option<&DocumentString> {
        self.input.raw()
    }
    /// Returns the exact presence of the supplied attribute.
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.input.observation()
    }
    /// Returns whether the `autocomplete` rule covers the supplied element and attribute names.
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.input.applicability()
    }
    /// Returns the selected standard attribute identity, which is always `autocomplete`.
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.input.attribute()
    }
    /// Returns the original index row IDs that this defining rule covers.
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.input.inventory_rows()
    }
    /// Returns the URL of the defining clause of the `autocomplete` attribute in the pinned
    /// edition.
    pub fn source_clause(&self) -> &'static str {
        self.input.source_clause()
    }
    /// Returns the URL of the autofill processing model clause that defines the processing result.
    pub fn processing_source_clause(&self) -> &'static str {
        PROCESSING_SOURCE
    }
    /// Returns the defining clause of the supplied input type state, or `None` when no state was
    /// supplied.
    pub fn context_source_clause(&self) -> Option<&'static str> {
        self.input.input_state().map(input_state_source)
    }
    /// Returns the number of UTF-16 code units of the raw value admitted before name or value
    /// processing, or zero when absent.
    pub fn input_code_units(&self) -> usize {
        self.input.input_code_units()
    }
    /// Returns the supplied actual input type state, without inferring one.
    pub fn input_state(&self) -> Option<HtmlInputType> {
        self.input.input_state()
    }
    /// Returns the value split on ASCII whitespace with each token's exact case, or `None` when the
    /// attribute is absent or the rule does not apply.
    pub fn tokens(&self) -> Option<&HtmlSpaceSeparatedTokens> {
        self.tokens.as_ref()
    }
    /// Returns the ordered expectation syntax judgment of the tokens.
    pub fn syntax(&self) -> &HtmlAutocompleteSyntaxState {
        &self.syntax
    }
    /// Returns the mantle the actual element and input state select. It is `None` when the rule
    /// does not apply or when an `input` was interpreted without its input state.
    pub fn mantle(&self) -> Option<HtmlAutocompleteMantle> {
        self.mantle
    }
    /// Returns the independent context requirements of the element and input state.
    pub fn context_author(&self) -> HtmlAutocompleteContextAuthorState {
        self.context_author
    }
    /// Returns the outcome of the autofill processing algorithm.
    pub fn processing(&self) -> &HtmlAutocompleteProcessingState {
        &self.processing
    }
}

/// The control an applicable verdict names, or `None` with the verdict no longer applicable. A
/// verdict is only `Applicable` together with its known element. The rule covers `input`, `select`
/// and `textarea` alone (HTML 877d), so an element that is none of them is a different element, as
/// `applies` reports for any other; an element that does not fit the supplied input state is a
/// different context.
fn applicable_control(
    applicability: &mut HtmlAttributeApplicability,
    known: Option<KnownHtmlElement>,
    state: Option<HtmlInputType>,
) -> Option<KnownHtmlElement> {
    if *applicability != HtmlAttributeApplicability::Applicable {
        return None;
    }
    match known {
        None => {
            *applicability = HtmlAttributeApplicability::DifferentElement;
            None
        }
        Some(known) if state.is_none() || known == KnownHtmlElement::Input => Some(known),
        Some(_) => {
            *applicability = HtmlAttributeApplicability::DifferentContext;
            None
        }
    }
}

fn keyword_token(keyword: DeclaredKeyword) -> HtmlSpaceToken {
    HtmlSpaceToken::declared(keyword)
}

fn setting(value: &DocumentString) -> Option<HtmlFormAutocomplete> {
    FORM_AUTOCOMPLETE
        .keywords
        .iter()
        .find(|keyword| ascii_keyword(value, keyword.spelling))
        .map(|keyword| keyword.state)
}

fn syntax(tokens: &HtmlSpaceSeparatedTokens) -> HtmlAutocompleteSyntaxState {
    match parse_details(tokens.tokens()) {
        Ok(value) => HtmlAutocompleteSyntaxState::Valid(value),
        Err(error) => HtmlAutocompleteSyntaxState::Invalid(error),
    }
}
fn parse_details(
    tokens: &[HtmlSpaceToken],
) -> Result<HtmlAutocompleteValue, HtmlAutocompleteSyntaxError> {
    if tokens.is_empty() {
        return Err(HtmlAutocompleteSyntaxError::EmptyTokens);
    }
    if tokens.len() == 1
        && let Some(setting) = setting(tokens[0].value())
    {
        return Ok(HtmlAutocompleteValue::Setting(setting));
    }
    let mut index = 0;
    let section = if section_prefix(tokens[0].value()) {
        index += 1;
        Some(HtmlAutocompleteSection {
            token: tokens[0].clone(),
        })
    } else {
        None
    };
    let mode = tokens
        .get(index)
        .and_then(|token| HtmlAutocompleteMode::from_keyword(token.value()));
    if mode.is_some() {
        index += 1;
    }
    let contact = tokens
        .get(index)
        .and_then(|token| HtmlAutocompleteContactHint::from_keyword(token.value()));
    if contact.is_some() {
        index += 1;
    }
    let field_token = tokens
        .get(index)
        .ok_or(HtmlAutocompleteSyntaxError::MissingDetailField)?;
    let field = HtmlAutofillField::from_keyword(field_token.value())
        .ok_or(HtmlAutocompleteSyntaxError::UnrecognizedDetailField { token_index: index })?;
    if contact.is_some() && field.category() != HtmlAutofillFieldCategory::Contact {
        return Err(HtmlAutocompleteSyntaxError::ContactHintRequiresContactField { field });
    }
    index += 1;
    let webauthn = tokens
        .get(index)
        .is_some_and(|token| ascii_keyword(token.value(), WEBAUTHN));
    if webauthn {
        index += 1;
    }
    if index != tokens.len() {
        return Err(HtmlAutocompleteSyntaxError::UnexpectedToken { token_index: index });
    }
    Ok(HtmlAutocompleteValue::Details(HtmlAutocompleteDetails {
        section,
        mode,
        contact,
        field,
        webauthn,
    }))
}

fn input_applies(state: HtmlInputType) -> bool {
    !matches!(
        state,
        HtmlInputType::Checkbox
            | HtmlInputType::Radio
            | HtmlInputType::File
            | HtmlInputType::Submit
            | HtmlInputType::Image
            | HtmlInputType::Reset
            | HtmlInputType::Button
    )
}

fn context_author(
    element: KnownHtmlElement,
    raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    state: Option<HtmlInputType>,
    syntax: &HtmlAutocompleteSyntaxState,
    tokens: Option<&HtmlSpaceSeparatedTokens>,
) -> HtmlAutocompleteContextAuthorState {
    if raw.is_none() {
        return HtmlAutocompleteContextAuthorState::NotRequired;
    }
    if element == KnownHtmlElement::Input && state.is_none() {
        return HtmlAutocompleteContextAuthorState::Unchecked;
    }
    let mut violations = HtmlAutocompleteContextViolations {
        prohibited_input_state: state.filter(|&state| !input_applies(state)),
        anchor_setting: None,
        inappropriate_field: None,
        webauthn_on_select: element == KnownHtmlElement::Select
            && tokens
                .and_then(|tokens| tokens.tokens().last())
                .is_some_and(|token| ascii_keyword(token.value(), WEBAUTHN)),
    };
    match syntax {
        HtmlAutocompleteSyntaxState::Valid(HtmlAutocompleteValue::Setting(setting)) => {
            if state == Some(HtmlInputType::Hidden) {
                violations.anchor_setting = Some(*setting);
            }
        }
        HtmlAutocompleteSyntaxState::Valid(HtmlAutocompleteValue::Details(details))
            if state.is_some_and(|state| !details.field.control_group().allows_input(state)) =>
        {
            violations.inappropriate_field = Some(details.field);
        }
        _ => {}
    }
    if violations.is_empty() {
        HtmlAutocompleteContextAuthorState::Conforming
    } else {
        HtmlAutocompleteContextAuthorState::Nonconforming(violations)
    }
}

fn default_state(
    mantle: HtmlAutocompleteMantle,
    reason: HtmlAutocompleteDefaultReason,
) -> HtmlAutocompleteProcessingState {
    HtmlAutocompleteProcessingState::Defaulted(HtmlAutocompleteDefault {
        reason,
        field: if mantle == HtmlAutocompleteMantle::Anchor {
            HtmlAutocompleteDefaultField::Empty
        } else {
            HtmlAutocompleteDefaultField::FormOwnerRequired
        },
        idl_value: DocumentString::from(""),
        scope: HtmlSpaceSeparatedTokens::new([]),
    })
}

fn identity(value: &DocumentString) -> Option<HtmlAutofillFieldIdentity> {
    if let Some(setting) = setting(value) {
        return Some(HtmlAutofillFieldIdentity::Setting(setting));
    }
    if let Some(field) = HtmlAutofillField::from_keyword(value) {
        return Some(HtmlAutofillFieldIdentity::Detail(field));
    }
    ascii_keyword(value, WEBAUTHN).then_some(HtmlAutofillFieldIdentity::Credential(
        HtmlAutocompleteCredential::Webauthn,
    ))
}

fn process(
    tokens: Option<&HtmlSpaceSeparatedTokens>,
    mantle: HtmlAutocompleteMantle,
    storage: &mut Materialization,
) -> Result<HtmlAutocompleteProcessingState, HtmlAutocompleteMaterializationError> {
    // All temporary IDL handle storage is local to process_inner, including its
    // failure/default paths. Drop it before releasing current scratch accounting.
    let result = process_inner(tokens, mantle, storage);
    let released = storage.release_scratch();
    let state = result?;
    released.map(|()| state)
}

fn process_inner(
    tokens: Option<&HtmlSpaceSeparatedTokens>,
    mantle: HtmlAutocompleteMantle,
    storage: &mut Materialization,
) -> Result<HtmlAutocompleteProcessingState, HtmlAutocompleteMaterializationError> {
    use HtmlAutocompleteDefaultReason as D;
    let Some(tokens) = tokens else {
        return Ok(default_state(mantle, D::MissingAttribute));
    };
    let tokens = tokens.tokens();
    let Some(last) = tokens.last() else {
        return Ok(default_state(mantle, D::EmptyTokens));
    };
    let Some(field_identity) = identity(last.value()) else {
        return Ok(default_state(mantle, D::UnrecognizedFinalToken));
    };
    if let HtmlAutofillFieldIdentity::Setting(setting) = field_identity {
        if tokens.len() != 1 {
            return Ok(default_state(mantle, D::SettingWithExtraTokens));
        }
        if mantle == HtmlAutocompleteMantle::Anchor {
            return Ok(default_state(mantle, D::SettingOnAnchor));
        }
        let field = storage.keyword(setting.canonical_keyword())?;
        return Ok(HtmlAutocompleteProcessingState::Processed(
            HtmlAutocompleteProcessed {
                field_identity,
                idl_value: field.clone(),
                field,
                mode: None,
                contact: None,
                scope: HtmlSpaceSeparatedTokens::from_vec(Vec::new()),
                credential: HtmlAutocompleteCredentialOutcome::NotAssigned,
            },
        ));
    }
    let mut index = tokens.len() - 1;
    let mut detail = match field_identity {
        HtmlAutofillFieldIdentity::Detail(field) => Some(field),
        _ => None,
    };
    let mut output = ProcessingOutput {
        field_identity,
        field: last.value().clone(),
        mode: None,
        contact: None,
        scope: Vec::new(),
        credential: HtmlAutocompleteCredentialOutcome::Null,
        idl: Vec::new(),
    };
    storage.prepend(&mut output.idl, last.clone(), StoragePurpose::Scratch)?;
    if matches!(field_identity, HtmlAutofillFieldIdentity::Credential(_)) {
        output.credential =
            HtmlAutocompleteCredentialOutcome::Credential(HtmlAutocompleteCredential::Webauthn);
        if index != 0 {
            index -= 1;
            detail = HtmlAutofillField::from_keyword(tokens[index].value());
            if detail.is_none() {
                return Ok(default_state(mantle, D::InvalidCredentialPrefix));
            }
            storage.prepend(
                &mut output.idl,
                tokens[index].clone(),
                StoragePurpose::Scratch,
            )?;
        }
        // Literal selected pin: field remains the original final credential
        // token. Category changes for contact processing, not field identity.
    }
    if index != 0 {
        index -= 1;
        if detail.is_some_and(|field| field.category() == HtmlAutofillFieldCategory::Contact)
            && let Some(hint) = HtmlAutocompleteContactHint::from_keyword(tokens[index].value())
        {
            output.contact = Some(hint);
            let token = hint
                .declared()
                .copy(|units, count| storage.reserve(units, count, StoragePurpose::Output))?;
            storage.prepend(&mut output.scope, token.clone(), StoragePurpose::Output)?;
            storage.prepend(&mut output.idl, token, StoragePurpose::Scratch)?;
            if index == 0 {
                return output.finish(storage);
            }
            index -= 1;
        }
        if let Some(hint) = HtmlAutocompleteMode::from_keyword(tokens[index].value()) {
            output.mode = Some(hint);
            let token = hint
                .declared()
                .copy(|units, count| storage.reserve(units, count, StoragePurpose::Output))?;
            storage.prepend(&mut output.scope, token.clone(), StoragePurpose::Output)?;
            storage.prepend(&mut output.idl, token, StoragePurpose::Scratch)?;
            if index == 0 {
                return output.finish(storage);
            }
            index -= 1;
        }
        if index != 0 {
            return Ok(default_state(mantle, D::ExtraLeadingTokens));
        }
        if !section_prefix(tokens[0].value()) {
            return Ok(default_state(mantle, D::UnrecognizedLeadingToken));
        }
        let section = tokens[0].units().copy(true, |units, count| {
            storage.reserve(units, count, StoragePurpose::Output)
        })?;
        storage.prepend(&mut output.scope, section.clone(), StoragePurpose::Output)?;
        storage.prepend(&mut output.idl, section, StoragePurpose::Scratch)?;
    }
    output.finish(storage)
}

struct ProcessingOutput {
    field_identity: HtmlAutofillFieldIdentity,
    field: DocumentString,
    mode: Option<HtmlAutocompleteMode>,
    contact: Option<HtmlAutocompleteContactHint>,
    scope: Vec<HtmlSpaceToken>,
    credential: HtmlAutocompleteCredentialOutcome,
    idl: Vec<HtmlSpaceToken>,
}
impl ProcessingOutput {
    fn finish(
        self,
        storage: &mut Materialization,
    ) -> Result<HtmlAutocompleteProcessingState, HtmlAutocompleteMaterializationError> {
        let idl_value = storage.idl(&self.idl)?;
        Ok(HtmlAutocompleteProcessingState::Processed(
            HtmlAutocompleteProcessed {
                field_identity: self.field_identity,
                field: self.field,
                mode: self.mode,
                contact: self.contact,
                scope: HtmlSpaceSeparatedTokens::from_vec(self.scope),
                credential: self.credential,
                idl_value,
            },
        ))
    }
}

impl HtmlValueInterpreter {
    /// Interprets control autocomplete with known select/textarea context.
    /// Input without actual state retains syntax but requires input context for
    /// processing. Default expectation outcomes request actual form-owner facts.
    pub fn control_autocomplete(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlAutocompleteReport, HtmlValueInterpretationError> {
        self.control_autocomplete_with_state(element, raw, None)
    }
    /// Actual input state selects mantle, author group and processing domain.
    /// This does not sanitize data, infer form association or mediate credentials.
    pub fn control_autocomplete_for_input(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        state: HtmlInputType,
    ) -> Result<HtmlAutocompleteReport, HtmlValueInterpretationError> {
        self.control_autocomplete_with_state(element, raw, Some(state))
    }
    fn control_autocomplete_with_state(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        state: Option<HtmlInputType>,
    ) -> Result<HtmlAutocompleteReport, HtmlValueInterpretationError> {
        let interpretation = |error| match error {
            HtmlAutocompleteMaterializationError::RawAdmission(error) => error.into(),
            HtmlAutocompleteMaterializationError::Allocation { .. } => {
                HtmlValueInterpretationError::AllocationFailed
            }
            HtmlAutocompleteMaterializationError::Representation => {
                HtmlValueInterpretationError::RepresentationExhausted
            }
            // Allowances belong to the controlled operation; this one sets none, so a limit or a
            // ledger state that no valid use produces breaks the protocol.
            HtmlAutocompleteMaterializationError::BytesLimit { .. }
            | HtmlAutocompleteMaterializationError::OutputLimit { .. }
            | HtmlAutocompleteMaterializationError::ScratchLimit { .. }
            | HtmlAutocompleteMaterializationError::LedgerState => {
                HtmlValueInterpretationError::ProtocolViolated
            }
        };
        let mut storage = Materialization::new(None).map_err(interpretation)?;
        self.materialize_autocomplete(element, raw, state, &mut storage)
            .map_err(interpretation)
    }

    /// Materializes the actual attribute's grammar and processing observations
    /// under caller-selected remaining allowances. Admit the work bound before
    /// calling; this atomic operation adds no controller or cancellation poll.
    /// Both outcomes retain cumulative observed capacities, including discarded
    /// intermediates. All operands remain borrowed on refusal.
    pub fn control_autocomplete_controlled(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        state: Option<HtmlInputType>,
        limits: HtmlAutocompleteMaterializationLimits,
    ) -> Result<HtmlAutocompleteMaterialized, HtmlAutocompleteMaterializationFailure> {
        let mut storage = Materialization::new(Some(limits)).map_err(|kind| {
            HtmlAutocompleteMaterializationFailure {
                kind,
                usage: HtmlAutocompleteMaterializationUsage::default(),
            }
        })?;
        let result = self.materialize_autocomplete(element, raw, state, &mut storage);
        debug_assert_eq!(storage.scratch, 0);
        result
            .map(|report| HtmlAutocompleteMaterialized {
                report,
                usage: storage.usage,
            })
            .map_err(|kind| HtmlAutocompleteMaterializationFailure {
                kind,
                usage: storage.usage,
            })
    }

    fn materialize_autocomplete(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        state: Option<HtmlInputType>,
        storage: &mut Materialization,
    ) -> Result<HtmlAutocompleteReport, HtmlAutocompleteMaterializationError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let mut input = report(RULE, element, raw);
        if let Some(state) = state {
            input.supplied_context = SuppliedContext::Input(state);
        }
        // The applicable verdict is kept only with the known control it names. The rule covers no
        // unknown element, so one is written back as a different element, and a supplied input
        // state fits an `input` alone, so another control is a different context; the report
        // then agrees with what follows.
        let Some(known) = applicable_control(
            &mut input.applicability,
            KnownHtmlElement::from_name(element),
            state,
        ) else {
            return Ok(HtmlAutocompleteReport {
                input,
                tokens: None,
                syntax: HtmlAutocompleteSyntaxState::Unapplied,
                mantle: None,
                context_author: HtmlAutocompleteContextAuthorState::Unapplied,
                processing: HtmlAutocompleteProcessingState::Unapplied,
            });
        };
        let tokens = raw.map(|raw| storage.tokens(raw.value())).transpose()?;
        let syntax = tokens
            .as_ref()
            .map_or(HtmlAutocompleteSyntaxState::Missing, syntax);
        let mantle = match (known, state) {
            (KnownHtmlElement::Input, None) => None,
            (KnownHtmlElement::Input, Some(HtmlInputType::Hidden)) => {
                Some(HtmlAutocompleteMantle::Anchor)
            }
            _ => Some(HtmlAutocompleteMantle::Expectation),
        };
        let context_author = context_author(known, raw, state, &syntax, tokens.as_ref());
        let processing = match (state, mantle) {
            (Some(state), _) if !input_applies(state) => {
                HtmlAutocompleteProcessingState::InapplicableInputState(state)
            }
            (_, None) => HtmlAutocompleteProcessingState::InputContextRequired,
            (_, Some(mantle)) => process(tokens.as_ref(), mantle, storage)?,
        };
        Ok(HtmlAutocompleteReport {
            input,
            tokens,
            syntax,
            mantle,
            context_author,
            processing,
        })
    }
}

#[cfg(test)]
mod controlled_tests {
    use super::*;

    /// An applicable verdict survives only with the known control it names, and every report that
    /// is returned unapplied says so: the verdict never stays applicable without its element.
    #[test]
    fn an_applicable_verdict_is_kept_only_with_its_known_control() {
        use HtmlAttributeApplicability as A;
        let cases = [
            (
                A::Applicable,
                Some(KnownHtmlElement::Select),
                None,
                Some(KnownHtmlElement::Select),
                A::Applicable,
            ),
            (
                A::Applicable,
                Some(KnownHtmlElement::Input),
                Some(HtmlInputType::Text),
                Some(KnownHtmlElement::Input),
                A::Applicable,
            ),
            (
                A::Applicable,
                Some(KnownHtmlElement::Select),
                Some(HtmlInputType::Text),
                None,
                A::DifferentContext,
            ),
            (A::Applicable, None, None, None, A::DifferentElement),
            (
                A::Applicable,
                None,
                Some(HtmlInputType::Text),
                None,
                A::DifferentElement,
            ),
            (
                A::DifferentElement,
                Some(KnownHtmlElement::Input),
                None,
                None,
                A::DifferentElement,
            ),
            (
                A::ForeignElementNamespace,
                None,
                None,
                None,
                A::ForeignElementNamespace,
            ),
        ];
        for (verdict, known, state, control, expected) in cases {
            let mut verdict = verdict;
            assert_eq!(applicable_control(&mut verdict, known, state), control);
            assert_eq!(verdict, expected);
        }
    }

    fn limits() -> HtmlAutocompleteMaterializationLimits {
        HtmlAutocompleteMaterializationLimits {
            allocated_bytes: usize::MAX,
            nested_output_records: usize::MAX,
            scratch_records: usize::MAX,
        }
    }
    fn raw(value: &str) -> Attribute {
        Attribute::new(KnownHtmlAttribute::Autocomplete.qualified_name(), value)
    }
    fn assert_default(
        state: &HtmlAutocompleteProcessingState,
        reason: HtmlAutocompleteDefaultReason,
        field: HtmlAutocompleteDefaultField,
    ) {
        let HtmlAutocompleteProcessingState::Defaulted(default) = state else {
            panic!("expected the default branch, got {state:?}");
        };
        assert_eq!(default.reason(), reason);
        assert_eq!(default.field(), field);
        assert_eq!(default.idl_value(), &DocumentString::from(""));
        assert_eq!(default.scope(), &HtmlSpaceSeparatedTokens::new([]));
    }
    fn controlled(
        value: &str,
        limits: HtmlAutocompleteMaterializationLimits,
    ) -> Result<HtmlAutocompleteMaterialized, HtmlAutocompleteMaterializationFailure> {
        HtmlValueInterpreter::new(200).control_autocomplete_controlled(
            &KnownHtmlElement::Textarea.qualified_name(),
            Some(&raw(value)),
            None,
            limits,
        )
    }

    #[test]
    fn controlled_producer_preserves_exact_published_observations() {
        let interpreter = HtmlValueInterpreter::new(200);
        for value in [
            "",
            "ON",
            "off",
            "NAME",
            "webauthn",
            "USERNAME WEBAUTHN",
            "SECTION-A SHIPPING HOME TEL WEBAUTHN",
            "unknown name",
            "tel home",
            "on webauthn",
            "\u{a0}name",
        ] {
            let raw = raw(value);
            for (element, state) in [
                (KnownHtmlElement::Textarea, None),
                (KnownHtmlElement::Select, None),
                (KnownHtmlElement::Input, None),
                (KnownHtmlElement::Input, Some(HtmlInputType::Hidden)),
                (KnownHtmlElement::Input, Some(HtmlInputType::Checkbox)),
            ] {
                let name = element.qualified_name();
                let result = interpreter
                    .control_autocomplete_controlled(&name, Some(&raw), state, limits())
                    .unwrap();
                let standalone = if let Some(state) = state {
                    interpreter.control_autocomplete_for_input(&name, Some(&raw), state)
                } else {
                    interpreter.control_autocomplete(&name, Some(&raw))
                }
                .unwrap();
                assert_eq!(result.report, standalone, "{value}, {state:?}");
            }
        }
    }

    #[test]
    fn controlled_utf16_materialization_preserves_surrogates_and_field_case() {
        let mut original = b"SECTION-"
            .iter()
            .copied()
            .map(u16::from)
            .collect::<Vec<_>>();
        original.extend([0xd800, 0x20, 0x4e, 0x41, 0x4d, 0x45]);
        let raw = Attribute::new(
            KnownHtmlAttribute::Autocomplete.qualified_name(),
            DocumentString::from_utf16(original.clone()),
        );
        let result = HtmlValueInterpreter::new(14)
            .control_autocomplete_controlled(
                &KnownHtmlElement::Textarea.qualified_name(),
                Some(&raw),
                None,
                limits(),
            )
            .unwrap();
        assert_eq!(result.report.raw().unwrap().code_units(), original);
        let HtmlAutocompleteProcessingState::Processed(value) = result.report.processing() else {
            panic!("source section/detail processing succeeds")
        };
        assert_eq!(value.field_name(), &DocumentString::from("NAME"));
        assert_eq!(
            value.credential_type(),
            HtmlAutocompleteCredentialOutcome::Null
        );
        let mut normalized = b"section-"
            .iter()
            .copied()
            .map(u16::from)
            .collect::<Vec<_>>();
        normalized.extend([0xd800, 0x20, 0x4e, 0x41, 0x4d, 0x45]);
        assert_eq!(value.idl_value().code_units(), normalized);
        assert_eq!(
            value.scope().tokens()[0].value().code_units(),
            &normalized[..9]
        );
    }

    #[test]
    fn exact_observed_allowances_succeed_and_each_one_short_rejects() {
        let value = "SECTION-A SHIPPING HOME TEL WEBAUTHN";
        let initial = controlled(value, limits()).unwrap();
        // Five source tokens and 32 raw units; three scope handles, 21 normalized
        // units and 36 final IDL units. IDL needs five simultaneous handles.
        assert!(initial.usage.nested_output_records >= 97);
        assert!(initial.usage.peak_scratch_records >= 5);
        let exact = HtmlAutocompleteMaterializationLimits {
            allocated_bytes: initial.usage.allocated_bytes,
            nested_output_records: initial.usage.nested_output_records,
            scratch_records: initial.usage.peak_scratch_records,
        };
        let repeated = controlled(value, exact).unwrap();
        assert_eq!(repeated.usage, initial.usage);
        assert_eq!(repeated.report, initial.report);
        let error = controlled(
            value,
            HtmlAutocompleteMaterializationLimits {
                allocated_bytes: exact.allocated_bytes - 1,
                ..exact
            },
        )
        .unwrap_err();
        assert_eq!(
            error.kind,
            HtmlAutocompleteMaterializationError::BytesLimit {
                limit: exact.allocated_bytes - 1,
                required: exact.allocated_bytes
            }
        );
        assert!(error.usage.allocated_bytes > 0);
        let error = controlled(
            value,
            HtmlAutocompleteMaterializationLimits {
                nested_output_records: exact.nested_output_records - 1,
                ..exact
            },
        )
        .unwrap_err();
        assert_eq!(
            error.kind,
            HtmlAutocompleteMaterializationError::OutputLimit {
                limit: exact.nested_output_records - 1,
                required: exact.nested_output_records
            }
        );
        let error = controlled(
            value,
            HtmlAutocompleteMaterializationLimits {
                scratch_records: exact.scratch_records - 1,
                ..exact
            },
        )
        .unwrap_err();
        assert_eq!(
            error.kind,
            HtmlAutocompleteMaterializationError::ScratchLimit {
                limit: exact.scratch_records - 1,
                required: exact.scratch_records
            }
        );
        assert!(error.usage.peak_scratch_records > 0);
    }

    #[test]
    fn failed_and_discarded_intermediates_retain_observed_usage() {
        let error = controlled(
            "name",
            HtmlAutocompleteMaterializationLimits {
                scratch_records: 0,
                ..limits()
            },
        )
        .unwrap_err();
        assert_eq!(
            error.kind,
            HtmlAutocompleteMaterializationError::ScratchLimit {
                limit: 0,
                required: 1
            }
        );
        // The declared contract (limits type docs) only says that every variable
        // Vec capacity is observed before its next write, so the failed run must
        // show some observed bytes and output records; no layout-dependent
        // floor is asserted because `size_of` and allocator growth are not
        // part of that contract.
        assert!(error.usage.allocated_bytes > 0);
        assert!(error.usage.nested_output_records > 0);
        assert_eq!(error.usage.peak_scratch_records, 0);
        let default = controlled("unknown name", limits()).unwrap();
        // Autofill processing model: the last token "name" is a field name, so
        // the leading token must be a section- token; "unknown" is not, and a
        // textarea's default expectation field needs its form owner.
        assert_default(
            default.report.processing(),
            HtmlAutocompleteDefaultReason::UnrecognizedLeadingToken,
            HtmlAutocompleteDefaultField::FormOwnerRequired,
        );
        assert!(default.usage.nested_output_records > 0);
        assert!(default.usage.peak_scratch_records > 0);
        assert!(default.usage.allocated_bytes > 0);
        let mut storage = Materialization::new(Some(limits())).unwrap();
        let tokens = storage
            .tokens(&DocumentString::from("unknown name"))
            .unwrap();
        process(
            Some(&tokens),
            HtmlAutocompleteMantle::Expectation,
            &mut storage,
        )
        .unwrap();
        assert_eq!(storage.scratch, 0);
    }

    #[test]
    fn allocator_overshoot_is_observed_before_resource_rejection() {
        let mut storage = Materialization::new(Some(HtmlAutocompleteMaterializationLimits {
            allocated_bytes: 8,
            nested_output_records: 4,
            scratch_records: 4,
        }))
        .unwrap();
        storage.check_growth(4, 8, StoragePurpose::Output).unwrap();
        assert_eq!(
            storage.observe_growth(5, 2, StoragePurpose::Output),
            Err(HtmlAutocompleteMaterializationError::BytesLimit {
                limit: 8,
                required: 10
            })
        );
        assert_eq!(storage.usage.allocated_bytes, 10);
        assert_eq!(storage.usage.nested_output_records, 5);
        let mut storage = Materialization::new(Some(HtmlAutocompleteMaterializationLimits {
            scratch_records: 4,
            ..limits()
        }))
        .unwrap();
        assert_eq!(
            storage.observe_growth(5, 8, StoragePurpose::Scratch),
            Err(HtmlAutocompleteMaterializationError::ScratchLimit {
                limit: 4,
                required: 5
            })
        );
        assert_eq!(storage.usage.allocated_bytes, 40);
        assert_eq!(storage.usage.peak_scratch_records, 5);
    }

    /// Growth the allocator made past the scratch allowance is recorded by the observed scope and
    /// refused by the capped scope, so only the observed scope holds it. Releasing the scratch
    /// afterwards must still leave both scopes and the live scratch at zero.
    #[test]
    fn releasing_scratch_after_a_refused_overshoot_leaves_both_scopes_at_zero() {
        let mut storage = Materialization::new(Some(HtmlAutocompleteMaterializationLimits {
            scratch_records: 4,
            ..limits()
        }))
        .unwrap();
        assert_eq!(
            storage.observe_growth(5, 8, StoragePurpose::Scratch),
            Err(HtmlAutocompleteMaterializationError::ScratchLimit {
                limit: 4,
                required: 5
            })
        );
        assert_eq!(storage.scratch, 5);
        assert_eq!(storage.release_scratch(), Ok(()));
        assert_eq!(storage.scratch, 0);
        for scope in [storage.observed, storage.caps] {
            let live = storage
                .operation
                .usage(scope)
                .unwrap()
                .live(HtmlResource::ScratchRecords);
            assert_eq!(live, 0);
        }
        // The largest scratch reached stays reported.
        assert_eq!(storage.usage.peak_scratch_records, 5);
    }

    /// A processing step that stops on a refused overshoot still ends with no live scratch, so the
    /// front door's closing check holds.
    #[test]
    fn processing_after_a_refused_overshoot_ends_with_no_live_scratch() {
        let mut storage = Materialization::new(Some(HtmlAutocompleteMaterializationLimits {
            scratch_records: 4,
            ..limits()
        }))
        .unwrap();
        let tokens = storage
            .tokens(&DocumentString::from("unknown name"))
            .unwrap();
        assert!(
            storage
                .observe_growth(5, 8, StoragePurpose::Scratch)
                .is_err()
        );
        let outcome = process(
            Some(&tokens),
            HtmlAutocompleteMantle::Expectation,
            &mut storage,
        );
        assert!(matches!(
            outcome,
            Err(HtmlAutocompleteMaterializationError::ScratchLimit { .. })
        ));
        assert_eq!(storage.scratch, 0);
    }

    #[test]
    fn admission_precedes_irrelevant_context_and_zero_storage_is_valid_when_unused() {
        let zero = HtmlAutocompleteMaterializationLimits {
            allocated_bytes: 0,
            nested_output_records: 0,
            scratch_records: 0,
        };
        let raw = raw("name");
        let foreign = QualifiedName::new(Namespace::svg(), None, "input");
        let error = HtmlValueInterpreter::new(3)
            .control_autocomplete_controlled(&foreign, Some(&raw), None, zero)
            .unwrap_err();
        assert!(matches!(
            error.kind,
            HtmlAutocompleteMaterializationError::RawAdmission(_)
        ));
        assert_eq!(error.usage, HtmlAutocompleteMaterializationUsage::default());
        let result = HtmlValueInterpreter::new(4)
            .control_autocomplete_controlled(&foreign, Some(&raw), None, zero)
            .unwrap();
        assert_eq!(
            result.report.applicability(),
            HtmlAttributeApplicability::ForeignElementNamespace
        );
        assert_eq!(
            result.usage,
            HtmlAutocompleteMaterializationUsage::default()
        );
        let result = HtmlValueInterpreter::new(0)
            .control_autocomplete_controlled(
                &KnownHtmlElement::Input.qualified_name(),
                None::<&Attribute>,
                Some(HtmlInputType::Hidden),
                zero,
            )
            .unwrap();
        // A Hidden input wears the autofill anchor mantle (forms capture line
        // 11553), whose default field is the empty one rather than a form-owner
        // dependent expectation.
        assert_default(
            result.report.processing(),
            HtmlAutocompleteDefaultReason::MissingAttribute,
            HtmlAutocompleteDefaultField::Empty,
        );
        assert_eq!(
            result.usage,
            HtmlAutocompleteMaterializationUsage::default()
        );
    }

    #[test]
    fn representation_and_work_envelope_are_checked_before_growth() {
        let mut storage = Materialization::new(Some(limits())).unwrap();
        assert_eq!(
            storage.reserve(&mut Vec::<u16>::new(), usize::MAX, StoragePurpose::Output),
            Err(HtmlAutocompleteMaterializationError::Representation)
        );
        assert_eq!(
            storage.usage,
            HtmlAutocompleteMaterializationUsage::default()
        );
        assert!(autocomplete_materialization_work_bound(0).unwrap() > 0);
        assert_eq!(
            autocomplete_materialization_work_bound(1).unwrap()
                - autocomplete_materialization_work_bound(0).unwrap(),
            12
        );
        assert_eq!(autocomplete_materialization_work_bound(usize::MAX), None);
    }
}

#[cfg(test)]
mod ordinary_allocation_tests {
    use super::*;
    use crate::internal::amortized::fault;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    fn raw(value: &str) -> Attribute {
        Attribute::new(KnownHtmlAttribute::Autocomplete.qualified_name(), value)
    }

    /// A refused allocation is an expected condition of a fallible allocator, so the standalone
    /// operation reports it as its error. It must neither abort the caller by panicking nor claim
    /// a report for storage it could not reserve.
    #[test]
    fn a_refused_reservation_is_an_error_and_never_a_panic() {
        let interpreter = HtmlValueInterpreter::new(200);
        let name = KnownHtmlElement::Textarea.qualified_name();
        let attribute = raw("section-a shipping home tel webauthn");
        let counting = fault::count_only();
        let baseline = interpreter
            .control_autocomplete(&name, Some(&attribute))
            .unwrap();
        let requests = fault::requests();
        drop(counting);
        assert!(requests >= 2, "{requests} reservation requests");
        for refuse_from in 1..=requests {
            let _armed = fault::arm(refuse_from);
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                interpreter.control_autocomplete(&name, Some(&attribute))
            }));
            let verdict = match &outcome {
                Err(_) => "panicked",
                Ok(Ok(_)) => "reported success",
                Ok(Err(HtmlValueInterpretationError::AllocationFailed)) => {
                    "returned the allocation error"
                }
                Ok(Err(_)) => "returned another error",
            };
            assert_eq!(
                verdict, "returned the allocation error",
                "reservation {refuse_from} of {requests} refused"
            );
        }
        let _armed = fault::arm(requests + 1);
        let report = interpreter
            .control_autocomplete(&name, Some(&attribute))
            .unwrap();
        assert_eq!(report, baseline);
    }

    /// The input-state entry point shares the storage policy, so it reports the same refusal.
    #[test]
    fn a_refused_reservation_for_an_input_is_an_error_and_never_a_panic() {
        let interpreter = HtmlValueInterpreter::new(200);
        let name = KnownHtmlElement::Input.qualified_name();
        let attribute = raw("section-a shipping home tel webauthn");
        let counting = fault::count_only();
        let baseline = interpreter
            .control_autocomplete_for_input(&name, Some(&attribute), HtmlInputType::Text)
            .unwrap();
        let requests = fault::requests();
        drop(counting);
        assert!(requests >= 2, "{requests} reservation requests");
        for refuse_from in 1..=requests {
            let _armed = fault::arm(refuse_from);
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                interpreter.control_autocomplete_for_input(
                    &name,
                    Some(&attribute),
                    HtmlInputType::Text,
                )
            }));
            let verdict = match &outcome {
                Err(_) => "panicked",
                Ok(Ok(_)) => "reported success",
                Ok(Err(HtmlValueInterpretationError::AllocationFailed)) => {
                    "returned the allocation error"
                }
                Ok(Err(_)) => "returned another error",
            };
            assert_eq!(
                verdict, "returned the allocation error",
                "reservation {refuse_from} of {requests} refused"
            );
        }
        let _armed = fault::arm(requests + 1);
        let report = interpreter
            .control_autocomplete_for_input(&name, Some(&attribute), HtmlInputType::Text)
            .unwrap();
        assert_eq!(report, baseline);
    }
}
