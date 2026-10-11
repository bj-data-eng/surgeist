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
) -> HtmlAutocompleteProcessingState {
    use HtmlAutocompleteDefaultReason as D;
    let Some(tokens) = tokens else {
        return default_state(mantle, D::MissingAttribute);
    };
    let tokens = tokens.tokens();
    let Some(last) = tokens.last() else {
        return default_state(mantle, D::EmptyTokens);
    };
    let Some(field_identity) = identity(last.value()) else {
        return default_state(mantle, D::UnrecognizedFinalToken);
    };
    if let HtmlAutofillFieldIdentity::Setting(setting) = field_identity {
        if tokens.len() != 1 {
            return default_state(mantle, D::SettingWithExtraTokens);
        }
        if mantle == HtmlAutocompleteMantle::Anchor {
            return default_state(mantle, D::SettingOnAnchor);
        }
        let field = DocumentString::from(setting.canonical_keyword());
        return HtmlAutocompleteProcessingState::Processed(HtmlAutocompleteProcessed {
            field_identity,
            idl_value: field.clone(),
            field,
            mode: None,
            contact: None,
            scope: HtmlSpaceSeparatedTokens::new([]),
            credential: HtmlAutocompleteCredentialOutcome::NotAssigned,
        });
    }
    let mut index = tokens.len() - 1;
    let mut credential = HtmlAutocompleteCredentialOutcome::Null;
    let mut detail = match field_identity {
        HtmlAutofillFieldIdentity::Detail(field) => Some(field),
        _ => None,
    };
    let mut idl = vec![last.clone()];
    if matches!(field_identity, HtmlAutofillFieldIdentity::Credential(_)) {
        credential =
            HtmlAutocompleteCredentialOutcome::Credential(HtmlAutocompleteCredential::Webauthn);
        if index != 0 {
            index -= 1;
            detail = HtmlAutofillField::from_keyword(tokens[index].value());
            if detail.is_none() {
                return default_state(mantle, D::InvalidCredentialPrefix);
            }
            idl.insert(0, tokens[index].clone());
        }
        // Literal selected pin: `field` remains the original final credential
        // token. Category changes for contact processing, not field identity.
    }
    let mut scope = Vec::new();
    let mut contact = None;
    let mut mode = None;
    if index != 0 {
        index -= 1;
        if detail.is_some_and(|field| field.category() == HtmlAutofillFieldCategory::Contact)
            && let Some(hint) = HtmlAutocompleteContactHint::from_keyword(tokens[index].value())
        {
            contact = Some(hint);
            let token = keyword_token(hint.canonical_keyword());
            scope.insert(0, token.clone());
            idl.insert(0, token);
            if index == 0 {
                return processed(field_identity, last, mode, contact, scope, credential, idl);
            }
            index -= 1;
        }
        if let Some(hint) = HtmlAutocompleteMode::from_keyword(tokens[index].value()) {
            mode = Some(hint);
            let token = keyword_token(hint.canonical_keyword());
            scope.insert(0, token.clone());
            idl.insert(0, token);
            if index == 0 {
                return processed(field_identity, last, mode, contact, scope, credential, idl);
            }
            index -= 1;
        }
        if index != 0 {
            return default_state(mantle, D::ExtraLeadingTokens);
        }
        if !section_prefix(tokens[0].value()) {
            return default_state(mantle, D::UnrecognizedLeadingToken);
        }
        let section = DocumentString::from_utf16(
            tokens[0]
                .value()
                .code_units()
                .iter()
                .map(|&unit| {
                    if (u16::from(b'A')..=u16::from(b'Z')).contains(&unit) {
                        unit + u16::from(b'a' - b'A')
                    } else {
                        unit
                    }
                })
                .collect::<Vec<_>>(),
        );
        let section =
            HtmlSpaceToken::new(section).expect("ASCII lowercase preserves a nonempty space token");
        scope.insert(0, section.clone());
        idl.insert(0, section);
    }
    processed(field_identity, last, mode, contact, scope, credential, idl)
}

fn processed(
    field_identity: HtmlAutofillFieldIdentity,
    last: &HtmlSpaceToken,
    mode: Option<HtmlAutocompleteMode>,
    contact: Option<HtmlAutocompleteContactHint>,
    scope: Vec<HtmlSpaceToken>,
    credential: HtmlAutocompleteCredentialOutcome,
    idl: Vec<HtmlSpaceToken>,
) -> HtmlAutocompleteProcessingState {
    HtmlAutocompleteProcessingState::Processed(HtmlAutocompleteProcessed {
        field_identity,
        field: last.value().clone(),
        mode,
        contact,
        scope: HtmlSpaceSeparatedTokens::new(scope),
        credential,
        idl_value: HtmlSpaceSeparatedTokens::new(idl).as_document_string(),
    })
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
        self.admit_attribute_raw(raw.map(Attribute::value))?;
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
        let tokens = raw.map(|raw| crate::tokens::split_space_tokens(raw.value()));
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
            (_, Some(mantle)) => process(tokens.as_ref(), mantle),
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
