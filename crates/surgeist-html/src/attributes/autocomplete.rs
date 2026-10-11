//! Control autocomplete grammar and literal pinned intrinsic token processing.
//! Form-owner defaults, autofill data, heuristics and credential mediation are
//! separate contextual/effect owners. WHATWG attribution is in the crate notice.

use super::*;
use crate::{HtmlSpaceSeparatedTokens, HtmlSpaceToken, HtmlTokenError};
use std::{error::Error, fmt};

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
const WEBAUTHN: &str = "webauthn";

/// Atomic scan/copy/lookup envelope, not machine instructions or parser polling.
/// The semantic owner separately admits expanded-name recognition. Two shared
/// split passes and their boundary visits use four input-length passes; token
/// copy/validation two; per-token reserve/push visits two; section transform and
/// validation two; IDL length/copy visits two. Vocabulary comparisons inspect at
/// most keyword length plus a length check: three field-table searches, two each
/// of setting/mode/contact, and three credential checks. The fixed remainder
/// covers section prefixes, at most five IDL and three scope handles, their
/// bounded prepends/reservations, default/context observations and empty input.
pub(crate) fn autocomplete_materialization_work_bound(input_code_units: usize) -> Option<usize> {
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
pub(crate) struct AutocompleteMaterializationLimits {
    pub(crate) allocated_bytes: usize,
    pub(crate) nested_output_records: usize,
    pub(crate) scratch_records: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AutocompleteMaterializationUsage {
    pub(crate) allocated_bytes: usize,
    pub(crate) nested_output_records: usize,
    pub(crate) peak_scratch_records: usize,
}

#[derive(Debug)]
pub(crate) struct AutocompleteMaterialized {
    pub(crate) report: HtmlAutocompleteReport,
    pub(crate) usage: AutocompleteMaterializationUsage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AutocompleteMaterializationError {
    RawAdmission(HtmlValueAdmissionError),
    BytesLimit { limit: usize, required: usize },
    OutputLimit { limit: usize, required: usize },
    ScratchLimit { limit: usize, required: usize },
    Allocation { requested_bytes: usize },
    Representation,
}

#[derive(Debug)]
pub(crate) struct AutocompleteMaterializationFailure {
    pub(crate) kind: AutocompleteMaterializationError,
    pub(crate) usage: AutocompleteMaterializationUsage,
}

#[derive(Clone, Copy)]
enum StoragePurpose {
    Output,
    Scratch,
}

/// One reservation owner used by both interpreter front doors. Standalone
/// retains its ordinary infallible allocation contract, not a resource cap.
struct Materialization {
    limits: Option<AutocompleteMaterializationLimits>,
    usage: AutocompleteMaterializationUsage,
    scratch: usize,
}

impl Materialization {
    fn new(limits: Option<AutocompleteMaterializationLimits>) -> Self {
        Self {
            limits,
            usage: AutocompleteMaterializationUsage::default(),
            scratch: 0,
        }
    }

    fn reserve<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
        purpose: StoragePurpose,
    ) -> Result<(), AutocompleteMaterializationError> {
        use AutocompleteMaterializationError as E;
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
        if self.limits.is_some() {
            values
                .try_reserve_exact(needed - values.len())
                .map_err(|_| E::Allocation {
                    requested_bytes: bytes,
                })?;
        } else {
            values.reserve_exact(needed - values.len());
        }
        self.observe_growth(
            values.capacity() - before,
            std::mem::size_of::<T>(),
            purpose,
        )
    }

    fn check_growth(
        &self,
        growth: usize,
        bytes: usize,
        purpose: StoragePurpose,
    ) -> Result<(), AutocompleteMaterializationError> {
        use AutocompleteMaterializationError as E;
        let bytes = self
            .usage
            .allocated_bytes
            .checked_add(bytes)
            .ok_or(E::Representation)?;
        let records = match purpose {
            StoragePurpose::Output => self.usage.nested_output_records,
            StoragePurpose::Scratch => self.scratch,
        }
        .checked_add(growth)
        .ok_or(E::Representation)?;
        if let Some(limits) = self.limits {
            if bytes > limits.allocated_bytes {
                return Err(E::BytesLimit {
                    limit: limits.allocated_bytes,
                    required: bytes,
                });
            }
            match purpose {
                StoragePurpose::Output if records > limits.nested_output_records => {
                    return Err(E::OutputLimit {
                        limit: limits.nested_output_records,
                        required: records,
                    });
                }
                StoragePurpose::Scratch if records > limits.scratch_records => {
                    return Err(E::ScratchLimit {
                        limit: limits.scratch_records,
                        required: records,
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn observe_growth(
        &mut self,
        growth: usize,
        item_bytes: usize,
        purpose: StoragePurpose,
    ) -> Result<(), AutocompleteMaterializationError> {
        use AutocompleteMaterializationError as E;
        self.usage.allocated_bytes = growth
            .checked_mul(item_bytes)
            .and_then(|bytes| self.usage.allocated_bytes.checked_add(bytes))
            .ok_or(E::Representation)?;
        match purpose {
            StoragePurpose::Output => {
                self.usage.nested_output_records = self
                    .usage
                    .nested_output_records
                    .checked_add(growth)
                    .ok_or(E::Representation)?
            }
            StoragePurpose::Scratch => {
                self.scratch = self.scratch.checked_add(growth).ok_or(E::Representation)?;
                self.usage.peak_scratch_records = self.usage.peak_scratch_records.max(self.scratch);
            }
        }
        // Observation precedes the rejection, including allocator overshoot.
        self.check_growth(0, 0, purpose)
    }

    fn units(
        &mut self,
        units: &[u16],
        lowercase: bool,
    ) -> Result<DocumentString, AutocompleteMaterializationError> {
        let mut output = Vec::new();
        self.reserve(&mut output, units.len(), StoragePurpose::Output)?;
        output.extend(units.iter().map(|&unit| {
            if lowercase && (0x41..=0x5a).contains(&unit) {
                unit + 0x20
            } else {
                unit
            }
        }));
        Ok(DocumentString::from_utf16(output))
    }

    fn keyword(
        &mut self,
        keyword: &'static str,
    ) -> Result<DocumentString, AutocompleteMaterializationError> {
        let mut output = Vec::new();
        self.reserve(&mut output, keyword.len(), StoragePurpose::Output)?;
        output.extend(keyword.bytes().map(u16::from));
        Ok(DocumentString::from_utf16(output))
    }

    fn tokens(
        &mut self,
        input: &DocumentString,
    ) -> Result<HtmlSpaceSeparatedTokens, AutocompleteMaterializationError> {
        let slices = crate::tokens::space_token_slices(input);
        let mut tokens = Vec::new();
        self.reserve(&mut tokens, slices.clone().count(), StoragePurpose::Output)?;
        for units in slices {
            let value = self.units(units, false)?;
            tokens.push(
                HtmlSpaceToken::new(value).expect("shared split yields a nonempty space token"),
            );
        }
        Ok(HtmlSpaceSeparatedTokens::from_vec(tokens))
    }

    fn prepend(
        &mut self,
        tokens: &mut Vec<HtmlSpaceToken>,
        token: HtmlSpaceToken,
        purpose: StoragePurpose,
    ) -> Result<(), AutocompleteMaterializationError> {
        self.reserve(tokens, 1, purpose)?;
        tokens.insert(0, token);
        Ok(())
    }

    fn idl(
        &mut self,
        tokens: &[HtmlSpaceToken],
    ) -> Result<DocumentString, AutocompleteMaterializationError> {
        let units = tokens
            .iter()
            .try_fold(tokens.len().saturating_sub(1), |length, token| {
                length.checked_add(token.value().len())
            })
            .ok_or(AutocompleteMaterializationError::Representation)?;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofillFieldCategory {
    Normal,
    Contact,
}

/// The author control group in HTML's defining autofill-field table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofillControlGroup {
    Text,
    Multiline,
    Password,
    Url,
    Username,
    Tel,
    Numeric,
    Month,
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
    ($($variant:ident => ($keyword:literal, $category:ident, $group:ident)),+ $(,)?) => {
        /// Native detail-field vocabulary, excluding on/off and credential fields.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum HtmlAutofillField { $($variant),+ }
        impl HtmlAutofillField {
            const ALL: &'static [Self] = &[$(Self::$variant),+];
            pub fn canonical_keyword(self) -> &'static str {
                match self { $(Self::$variant => $keyword),+ }
            }
            pub fn category(self) -> HtmlAutofillFieldCategory {
                match self { $(Self::$variant => HtmlAutofillFieldCategory::$category),+ }
            }
            pub fn control_group(self) -> HtmlAutofillControlGroup {
                match self { $(Self::$variant => HtmlAutofillControlGroup::$group),+ }
            }
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
    Name => ("name", Normal, Text),
    HonorificPrefix => ("honorific-prefix", Normal, Text),
    GivenName => ("given-name", Normal, Text),
    AdditionalName => ("additional-name", Normal, Text),
    FamilyName => ("family-name", Normal, Text),
    HonorificSuffix => ("honorific-suffix", Normal, Text),
    Nickname => ("nickname", Normal, Text),
    OrganizationTitle => ("organization-title", Normal, Text),
    Username => ("username", Normal, Username),
    NewPassword => ("new-password", Normal, Password),
    CurrentPassword => ("current-password", Normal, Password),
    OneTimeCode => ("one-time-code", Normal, Password),
    Organization => ("organization", Normal, Text),
    StreetAddress => ("street-address", Normal, Multiline),
    AddressLine1 => ("address-line1", Normal, Text),
    AddressLine2 => ("address-line2", Normal, Text),
    AddressLine3 => ("address-line3", Normal, Text),
    AddressLevel4 => ("address-level4", Normal, Text),
    AddressLevel3 => ("address-level3", Normal, Text),
    AddressLevel2 => ("address-level2", Normal, Text),
    AddressLevel1 => ("address-level1", Normal, Text),
    Country => ("country", Normal, Text),
    CountryName => ("country-name", Normal, Text),
    PostalCode => ("postal-code", Normal, Text),
    CcName => ("cc-name", Normal, Text),
    CcGivenName => ("cc-given-name", Normal, Text),
    CcAdditionalName => ("cc-additional-name", Normal, Text),
    CcFamilyName => ("cc-family-name", Normal, Text),
    CcNumber => ("cc-number", Normal, Text),
    CcExp => ("cc-exp", Normal, Month),
    CcExpMonth => ("cc-exp-month", Normal, Numeric),
    CcExpYear => ("cc-exp-year", Normal, Numeric),
    CcCsc => ("cc-csc", Normal, Text),
    CcType => ("cc-type", Normal, Text),
    TransactionCurrency => ("transaction-currency", Normal, Text),
    TransactionAmount => ("transaction-amount", Normal, Numeric),
    Language => ("language", Normal, Text),
    Bday => ("bday", Normal, Date),
    BdayDay => ("bday-day", Normal, Numeric),
    BdayMonth => ("bday-month", Normal, Numeric),
    BdayYear => ("bday-year", Normal, Numeric),
    Sex => ("sex", Normal, Text),
    Url => ("url", Normal, Url),
    Photo => ("photo", Normal, Url),
    Tel => ("tel", Contact, Tel),
    TelCountryCode => ("tel-country-code", Contact, Text),
    TelNational => ("tel-national", Contact, Text),
    TelAreaCode => ("tel-area-code", Contact, Text),
    TelLocal => ("tel-local", Contact, Text),
    TelLocalPrefix => ("tel-local-prefix", Contact, Text),
    TelLocalSuffix => ("tel-local-suffix", Contact, Text),
    TelExtension => ("tel-extension", Contact, Text),
    Email => ("email", Contact, Username),
    Impp => ("impp", Contact, Url),
}

macro_rules! hint_type {
    ($name:ident, [$($variant:ident => $keyword:literal),+]) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name { $($variant),+ }
        impl $name {
            const ALL: &'static [Self] = &[$(Self::$variant),+];
            pub fn canonical_keyword(self) -> &'static str {
                match self { $(Self::$variant => $keyword),+ }
            }
            fn from_keyword(value: &DocumentString) -> Option<Self> {
                Self::ALL.iter().copied().find(|hint| ascii_keyword(value, hint.canonical_keyword()))
            }
        }
    };
}

hint_type!(HtmlAutocompleteMode, [Shipping => "shipping", Billing => "billing"]);
hint_type!(HtmlAutocompleteContactHint, [Home => "home", Work => "work", Mobile => "mobile", Fax => "fax", Pager => "pager"]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteCredential {
    Webauthn,
}
impl HtmlAutocompleteCredential {
    pub fn canonical_keyword(self) -> &'static str {
        WEBAUTHN
    }
}

/// A grammar-valid section token; suffix text remains exact, including UTF-16.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAutocompleteSection {
    token: HtmlSpaceToken,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteSectionError {
    Token(HtmlTokenError),
    MissingSectionPrefix,
}
impl fmt::Display for HtmlAutocompleteSectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid autocomplete section: {self:?}")
    }
}
impl Error for HtmlAutocompleteSectionError {}

fn section_prefix(value: &DocumentString) -> bool {
    value.code_units().get(..8).is_some_and(|units| {
        units.iter().zip(b"section-").all(|(&unit, byte)| {
            u8::try_from(unit).is_ok_and(|unit| unit.eq_ignore_ascii_case(byte))
        })
    })
}

impl HtmlAutocompleteSection {
    /// Requires the section- prefix and one space token, with no suffix grammar.
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlAutocompleteSectionError> {
        let token = HtmlSpaceToken::new(value).map_err(HtmlAutocompleteSectionError::Token)?;
        if !section_prefix(token.value()) {
            return Err(HtmlAutocompleteSectionError::MissingSectionPrefix);
        }
        Ok(Self { token })
    }
    pub fn value(&self) -> &DocumentString {
        self.token.value()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlAutocompleteContactFieldError {
    field: HtmlAutofillField,
}
impl HtmlAutocompleteContactFieldError {
    pub fn field(self) -> HtmlAutofillField {
        self.field
    }
}
impl fmt::Display for HtmlAutocompleteContactFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "autocomplete field {:?} does not admit a contact hint",
            self.field
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
    pub fn new(field: HtmlAutofillField) -> Self {
        Self {
            section: None,
            mode: None,
            contact: None,
            field,
            webauthn: false,
        }
    }
    pub fn with_section(mut self, section: HtmlAutocompleteSection) -> Self {
        self.section = Some(section);
        self
    }
    pub fn with_mode(mut self, mode: HtmlAutocompleteMode) -> Self {
        self.mode = Some(mode);
        self
    }
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
    pub fn with_webauthn(mut self) -> Self {
        self.webauthn = true;
        self
    }
    pub fn section(&self) -> Option<&HtmlAutocompleteSection> {
        self.section.as_ref()
    }
    pub fn mode(&self) -> Option<HtmlAutocompleteMode> {
        self.mode
    }
    pub fn contact_hint(&self) -> Option<HtmlAutocompleteContactHint> {
        self.contact
    }
    pub fn field(&self) -> HtmlAutofillField {
        self.field
    }
    pub fn has_webauthn(&self) -> bool {
        self.webauthn
    }
    pub fn into_attribute(self) -> Attribute {
        let mut tokens = Vec::new();
        if let Some(section) = self.section {
            tokens.push(section.token);
        }
        if let Some(mode) = self.mode {
            tokens.push(keyword_token(mode.canonical_keyword()));
        }
        if let Some(contact) = self.contact {
            tokens.push(keyword_token(contact.canonical_keyword()));
        }
        tokens.push(keyword_token(self.field.canonical_keyword()));
        if self.webauthn {
            tokens.push(keyword_token(WEBAUTHN));
        }
        Attribute::new(
            KnownHtmlAttribute::Autocomplete.qualified_name(),
            HtmlSpaceSeparatedTokens::new(tokens).as_document_string(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteValue {
    Setting(HtmlFormAutocomplete),
    Details(HtmlAutocompleteDetails),
}
impl HtmlAutocompleteValue {
    pub fn into_attribute(self) -> Attribute {
        match self {
            Self::Setting(setting) => setting.into_attribute(),
            Self::Details(details) => details.into_attribute(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteSyntaxError {
    EmptyTokens,
    MissingDetailField,
    UnrecognizedDetailField { token_index: usize },
    ContactHintRequiresContactField { field: HtmlAutofillField },
    UnexpectedToken { token_index: usize },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteSyntaxState {
    Missing,
    /// Ordered expectation syntax only; mantle/control authorship is separate.
    Valid(HtmlAutocompleteValue),
    Invalid(HtmlAutocompleteSyntaxError),
    Unapplied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteMantle {
    Expectation,
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
    pub fn prohibited_input_state(self) -> Option<HtmlInputType> {
        self.prohibited_input_state
    }
    pub fn anchor_setting(self) -> Option<HtmlFormAutocomplete> {
        self.anchor_setting
    }
    pub fn inappropriate_field(self) -> Option<HtmlAutofillField> {
        self.inappropriate_field
    }
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteContextAuthorState {
    NotRequired,
    Unchecked,
    /// No selected context violation; this does not certify lexical conformance.
    Conforming,
    Nonconforming(HtmlAutocompleteContextViolations),
    Unapplied,
}

/// Processing permits the credential field identity separately from author fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofillFieldIdentity {
    Setting(HtmlFormAutocomplete),
    Detail(HtmlAutofillField),
    Credential(HtmlAutocompleteCredential),
}

/// Assignment produced by the selected processing branch, independent of any
/// previous live credential state. Settings return without assigning this
/// property; successful detail processing assigns null or a credential.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteCredentialOutcome {
    NotAssigned,
    Null,
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
    pub fn field_identity(&self) -> HtmlAutofillFieldIdentity {
        self.field_identity
    }
    pub fn field_name(&self) -> &DocumentString {
        &self.field
    }
    /// Mode and contact are the complete typed hint set; section is scope only.
    pub fn mode_hint(&self) -> Option<HtmlAutocompleteMode> {
        self.mode
    }
    pub fn contact_hint(&self) -> Option<HtmlAutocompleteContactHint> {
        self.contact
    }
    pub fn scope(&self) -> &HtmlSpaceSeparatedTokens {
        &self.scope
    }
    pub fn credential_type(&self) -> HtmlAutocompleteCredentialOutcome {
        self.credential
    }
    pub fn idl_value(&self) -> &DocumentString {
        &self.idl_value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteDefaultReason {
    MissingAttribute,
    EmptyTokens,
    UnrecognizedFinalToken,
    SettingWithExtraTokens,
    SettingOnAnchor,
    InvalidCredentialPrefix,
    ExtraLeadingTokens,
    UnrecognizedLeadingToken,
}
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
    pub fn reason(&self) -> HtmlAutocompleteDefaultReason {
        self.reason
    }
    pub fn field(&self) -> HtmlAutocompleteDefaultField {
        self.field
    }
    pub fn idl_value(&self) -> &DocumentString {
        &self.idl_value
    }
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteProcessingState {
    Processed(HtmlAutocompleteProcessed),
    Defaulted(HtmlAutocompleteDefault),
    /// Actual input state is needed for both processing domain and mantle.
    InputContextRequired,
    /// Source gives autofill processing only to input states admitting autocomplete.
    InapplicableInputState(HtmlInputType),
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
    pub fn element(&self) -> &QualifiedName {
        self.input.element()
    }
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.input.raw_attribute()
    }
    pub fn raw(&self) -> Option<&DocumentString> {
        self.input.raw()
    }
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.input.observation()
    }
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.input.applicability()
    }
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.input.attribute()
    }
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.input.inventory_rows()
    }
    pub fn source_clause(&self) -> &'static str {
        self.input.source_clause()
    }
    pub fn processing_source_clause(&self) -> &'static str {
        PROCESSING_SOURCE
    }
    pub fn context_source_clause(&self) -> Option<&'static str> {
        self.input.input_state().map(input_state_source)
    }
    pub fn input_code_units(&self) -> usize {
        self.input.input_code_units()
    }
    pub fn input_state(&self) -> Option<HtmlInputType> {
        self.input.input_state()
    }
    pub fn tokens(&self) -> Option<&HtmlSpaceSeparatedTokens> {
        self.tokens.as_ref()
    }
    pub fn syntax(&self) -> &HtmlAutocompleteSyntaxState {
        &self.syntax
    }
    pub fn mantle(&self) -> Option<HtmlAutocompleteMantle> {
        self.mantle
    }
    pub fn context_author(&self) -> HtmlAutocompleteContextAuthorState {
        self.context_author
    }
    pub fn processing(&self) -> &HtmlAutocompleteProcessingState {
        &self.processing
    }
}

fn keyword_token(keyword: &'static str) -> HtmlSpaceToken {
    HtmlSpaceToken::new(keyword).expect("declared autocomplete keyword is one nonempty space token")
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
    raw: Option<&Attribute>,
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
) -> Result<HtmlAutocompleteProcessingState, AutocompleteMaterializationError> {
    // All temporary IDL handle storage is local to process_inner, including its
    // failure/default paths. Drop it before releasing current scratch accounting.
    let result = process_inner(tokens, mantle, storage);
    storage.scratch = 0;
    result
}

fn process_inner(
    tokens: Option<&HtmlSpaceSeparatedTokens>,
    mantle: HtmlAutocompleteMantle,
    storage: &mut Materialization,
) -> Result<HtmlAutocompleteProcessingState, AutocompleteMaterializationError> {
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
            let token = HtmlSpaceToken::new(storage.keyword(hint.canonical_keyword())?)
                .expect("declared hint is a nonempty space token");
            storage.prepend(&mut output.scope, token.clone(), StoragePurpose::Output)?;
            storage.prepend(&mut output.idl, token, StoragePurpose::Scratch)?;
            if index == 0 {
                return output.finish(storage);
            }
            index -= 1;
        }
        if let Some(hint) = HtmlAutocompleteMode::from_keyword(tokens[index].value()) {
            output.mode = Some(hint);
            let token = HtmlSpaceToken::new(storage.keyword(hint.canonical_keyword())?)
                .expect("declared hint is a nonempty space token");
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
        let section = HtmlSpaceToken::new(storage.units(tokens[0].value().code_units(), true)?)
            .expect("ASCII lowercase preserves a nonempty space token");
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
    ) -> Result<HtmlAutocompleteProcessingState, AutocompleteMaterializationError> {
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAutocompleteReport, HtmlValueAdmissionError> {
        self.control_autocomplete_with_state(element, raw, None)
    }
    /// Actual input state selects mantle, author group and processing domain.
    /// This does not sanitize data, infer form association or mediate credentials.
    pub fn control_autocomplete_for_input(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        state: HtmlInputType,
    ) -> Result<HtmlAutocompleteReport, HtmlValueAdmissionError> {
        self.control_autocomplete_with_state(element, raw, Some(state))
    }
    fn control_autocomplete_with_state(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        state: Option<HtmlInputType>,
    ) -> Result<HtmlAutocompleteReport, HtmlValueAdmissionError> {
        let result =
            self.materialize_autocomplete(element, raw, state, &mut Materialization::new(None));
        match result {
            Ok(report) => Ok(report),
            Err(AutocompleteMaterializationError::RawAdmission(error)) => Err(error),
            // Standalone has no allocation-limit result contract. Its ordinary
            // reserve policy already panics/aborts on representation/allocation
            // failure, as the previous infallible materialization did.
            Err(error) => panic!("standalone autocomplete storage exhausted: {error:?}"),
        }
    }

    pub(crate) fn control_autocomplete_controlled(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        state: Option<HtmlInputType>,
        limits: AutocompleteMaterializationLimits,
    ) -> Result<AutocompleteMaterialized, AutocompleteMaterializationFailure> {
        let mut storage = Materialization::new(Some(limits));
        let result = self.materialize_autocomplete(element, raw, state, &mut storage);
        debug_assert_eq!(storage.scratch, 0);
        result
            .map(|report| AutocompleteMaterialized {
                report,
                usage: storage.usage,
            })
            .map_err(|kind| AutocompleteMaterializationFailure {
                kind,
                usage: storage.usage,
            })
    }

    fn materialize_autocomplete(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        state: Option<HtmlInputType>,
        storage: &mut Materialization,
    ) -> Result<HtmlAutocompleteReport, AutocompleteMaterializationError> {
        self.admit_attribute_raw(raw.map(Attribute::value))
            .map_err(AutocompleteMaterializationError::RawAdmission)?;
        let mut input = report(RULE, element, raw);
        if let Some(state) = state {
            input.supplied_context = SuppliedContext::Input(state);
        }
        let known = KnownHtmlElement::from_name(element);
        if input.applicability == HtmlAttributeApplicability::Applicable
            && state.is_some()
            && known != Some(KnownHtmlElement::Input)
        {
            input.applicability = HtmlAttributeApplicability::DifferentContext;
        }
        if input.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(HtmlAutocompleteReport {
                input,
                tokens: None,
                syntax: HtmlAutocompleteSyntaxState::Unapplied,
                mantle: None,
                context_author: HtmlAutocompleteContextAuthorState::Unapplied,
                processing: HtmlAutocompleteProcessingState::Unapplied,
            });
        }
        let known = known.expect("autocomplete applicability proves a known HTML control");
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

    fn limits() -> AutocompleteMaterializationLimits {
        AutocompleteMaterializationLimits {
            allocated_bytes: usize::MAX,
            nested_output_records: usize::MAX,
            scratch_records: usize::MAX,
        }
    }
    fn raw(value: &str) -> Attribute {
        Attribute::new(KnownHtmlAttribute::Autocomplete.qualified_name(), value)
    }
    fn controlled(
        value: &str,
        limits: AutocompleteMaterializationLimits,
    ) -> Result<AutocompleteMaterialized, AutocompleteMaterializationFailure> {
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
        let exact = AutocompleteMaterializationLimits {
            allocated_bytes: initial.usage.allocated_bytes,
            nested_output_records: initial.usage.nested_output_records,
            scratch_records: initial.usage.peak_scratch_records,
        };
        let repeated = controlled(value, exact).unwrap();
        assert_eq!(repeated.usage, initial.usage);
        assert_eq!(repeated.report, initial.report);
        let error = controlled(
            value,
            AutocompleteMaterializationLimits {
                allocated_bytes: exact.allocated_bytes - 1,
                ..exact
            },
        )
        .unwrap_err();
        assert_eq!(
            error.kind,
            AutocompleteMaterializationError::BytesLimit {
                limit: exact.allocated_bytes - 1,
                required: exact.allocated_bytes
            }
        );
        assert!(error.usage.allocated_bytes > 0);
        let error = controlled(
            value,
            AutocompleteMaterializationLimits {
                nested_output_records: exact.nested_output_records - 1,
                ..exact
            },
        )
        .unwrap_err();
        assert_eq!(
            error.kind,
            AutocompleteMaterializationError::OutputLimit {
                limit: exact.nested_output_records - 1,
                required: exact.nested_output_records
            }
        );
        let error = controlled(
            value,
            AutocompleteMaterializationLimits {
                scratch_records: exact.scratch_records - 1,
                ..exact
            },
        )
        .unwrap_err();
        assert_eq!(
            error.kind,
            AutocompleteMaterializationError::ScratchLimit {
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
            AutocompleteMaterializationLimits {
                scratch_records: 0,
                ..limits()
            },
        )
        .unwrap_err();
        assert_eq!(
            error.kind,
            AutocompleteMaterializationError::ScratchLimit {
                limit: 0,
                required: 1
            }
        );
        assert!(error.usage.allocated_bytes >= std::mem::size_of::<HtmlSpaceToken>() + 8);
        assert!(error.usage.nested_output_records >= 5);
        assert_eq!(error.usage.peak_scratch_records, 0);
        let default = controlled("unknown name", limits()).unwrap();
        assert!(matches!(
            default.report.processing(),
            HtmlAutocompleteProcessingState::Defaulted(_)
        ));
        assert!(default.usage.nested_output_records >= 13);
        assert!(default.usage.peak_scratch_records >= 1);
        assert!(default.usage.allocated_bytes >= 3 * std::mem::size_of::<HtmlSpaceToken>() + 22);
        let mut storage = Materialization::new(Some(limits()));
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
        let mut storage = Materialization::new(Some(AutocompleteMaterializationLimits {
            allocated_bytes: 8,
            nested_output_records: 4,
            scratch_records: 4,
        }));
        storage.check_growth(4, 8, StoragePurpose::Output).unwrap();
        assert_eq!(
            storage.observe_growth(5, 2, StoragePurpose::Output),
            Err(AutocompleteMaterializationError::BytesLimit {
                limit: 8,
                required: 10
            })
        );
        assert_eq!(storage.usage.allocated_bytes, 10);
        assert_eq!(storage.usage.nested_output_records, 5);
        let mut storage = Materialization::new(Some(AutocompleteMaterializationLimits {
            scratch_records: 4,
            ..limits()
        }));
        assert_eq!(
            storage.observe_growth(5, 8, StoragePurpose::Scratch),
            Err(AutocompleteMaterializationError::ScratchLimit {
                limit: 4,
                required: 5
            })
        );
        assert_eq!(storage.usage.allocated_bytes, 40);
        assert_eq!(storage.usage.peak_scratch_records, 5);
    }

    #[test]
    fn admission_precedes_irrelevant_context_and_zero_storage_is_valid_when_unused() {
        let zero = AutocompleteMaterializationLimits {
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
            AutocompleteMaterializationError::RawAdmission(_)
        ));
        assert_eq!(error.usage, AutocompleteMaterializationUsage::default());
        let result = HtmlValueInterpreter::new(4)
            .control_autocomplete_controlled(&foreign, Some(&raw), None, zero)
            .unwrap();
        assert_eq!(
            result.report.applicability(),
            HtmlAttributeApplicability::ForeignElementNamespace
        );
        assert_eq!(result.usage, AutocompleteMaterializationUsage::default());
        let result = HtmlValueInterpreter::new(0)
            .control_autocomplete_controlled(
                &KnownHtmlElement::Input.qualified_name(),
                None,
                Some(HtmlInputType::Hidden),
                zero,
            )
            .unwrap();
        assert!(matches!(
            result.report.processing(),
            HtmlAutocompleteProcessingState::Defaulted(_)
        ));
        assert_eq!(result.usage, AutocompleteMaterializationUsage::default());
    }

    #[test]
    fn representation_and_work_envelope_are_checked_before_growth() {
        let mut storage = Materialization::new(Some(limits()));
        assert_eq!(
            storage.reserve(&mut Vec::<u16>::new(), usize::MAX, StoragePurpose::Output),
            Err(AutocompleteMaterializationError::Representation)
        );
        assert_eq!(storage.usage, AutocompleteMaterializationUsage::default());
        assert!(autocomplete_materialization_work_bound(0).unwrap() > 0);
        assert_eq!(
            autocomplete_materialization_work_bound(1).unwrap()
                - autocomplete_materialization_work_bound(0).unwrap(),
            12
        );
        assert_eq!(autocomplete_materialization_work_bound(usize::MAX), None);
    }
}
