use crate::declarations::EventInit;
use crate::{DomString, Error, IntegerKind, IntegerMode, IntegerValue, UsvString, Value};

use super::{
    Conversion, Converted, Descriptor, Enumeration, Failure, IntegerType, Work, declarations, types,
};

/// The CSP3 disposition of a security policy violation event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecurityPolicyViolationEventDisposition {
    /// The policy is enforced.
    Enforce,
    /// The policy reports violations.
    Report,
}

impl SecurityPolicyViolationEventDisposition {
    fn from_code_units(units: &[u16]) -> Option<Self> {
        match units {
            [101, 110, 102, 111, 114, 99, 101] => Some(Self::Enforce),
            [114, 101, 112, 111, 114, 116] => Some(Self::Report),
            _ => None,
        }
    }
}

/// Converted `SecurityPolicyViolationEventInit` with its declared defaults.
///
/// Each string retains its conversion buffer reservation.
pub struct SecurityPolicyViolationEventOptions {
    /// Inherited bubbling flag.
    pub bubbles: bool,
    /// Inherited cancelability flag.
    pub cancelable: bool,
    /// Inherited composed flag.
    pub composed: bool,
    /// URI of the blocked resource.
    pub blocked_uri: Converted<UsvString>,
    /// Column in the source file.
    pub column_number: u32,
    /// Policy disposition.
    pub disposition: SecurityPolicyViolationEventDisposition,
    /// URI of the protected document.
    pub document_uri: Converted<UsvString>,
    /// Directive whose enforcement or reporting produced the violation.
    pub effective_directive: Converted<DomString>,
    /// Line in the source file.
    pub line_number: u32,
    /// Serialized policy.
    pub original_policy: Converted<DomString>,
    /// Referrer URI.
    pub referrer: Converted<UsvString>,
    /// Source sample.
    pub sample: Converted<DomString>,
    /// Source file URI.
    pub source_file: Converted<UsvString>,
    /// HTTP status code.
    pub status_code: u16,
    /// Violated directive.
    pub violated_directive: Converted<DomString>,
}

const BLOCKED_URI: &[u16] = &[98, 108, 111, 99, 107, 101, 100, 85, 82, 73];
const COLUMN_NUMBER: &[u16] = &[99, 111, 108, 117, 109, 110, 78, 117, 109, 98, 101, 114];
const DISPOSITION: &[u16] = &[100, 105, 115, 112, 111, 115, 105, 116, 105, 111, 110];
const DOCUMENT_URI: &[u16] = &[100, 111, 99, 117, 109, 101, 110, 116, 85, 82, 73];
const EFFECTIVE_DIRECTIVE: &[u16] = &[
    101, 102, 102, 101, 99, 116, 105, 118, 101, 68, 105, 114, 101, 99, 116, 105, 118, 101,
];
const LINE_NUMBER: &[u16] = &[108, 105, 110, 101, 78, 117, 109, 98, 101, 114];
const ORIGINAL_POLICY: &[u16] = &[
    111, 114, 105, 103, 105, 110, 97, 108, 80, 111, 108, 105, 99, 121,
];
const REFERRER: &[u16] = &[114, 101, 102, 101, 114, 114, 101, 114];
const SAMPLE: &[u16] = &[115, 97, 109, 112, 108, 101];
const SOURCE_FILE: &[u16] = &[115, 111, 117, 114, 99, 101, 70, 105, 108, 101];
const STATUS_CODE: &[u16] = &[115, 116, 97, 116, 117, 115, 67, 111, 100, 101];
const VIOLATED_DIRECTIVE: &[u16] = &[
    118, 105, 111, 108, 97, 116, 101, 100, 68, 105, 114, 101, 99, 116, 105, 118, 101,
];

const DISPOSITION_TYPE: Enumeration<SecurityPolicyViolationEventDisposition> = Enumeration::new(
    "SecurityPolicyViolationEventDisposition",
    SecurityPolicyViolationEventDisposition::from_code_units,
);

const UNSIGNED_LONG: IntegerType = IntegerType::new(IntegerKind::UnsignedLong, IntegerMode::Wrap);
const UNSIGNED_SHORT: IntegerType = IntegerType::new(IntegerKind::UnsignedShort, IntegerMode::Wrap);

impl<'scope> Conversion<'_, 'scope> {
    /// Convert the optional CSP3 event constructor dictionary. Inherited
    /// `EventInit` members precede the derived members in lexicographic order.
    pub fn security_policy_violation_event_init(
        &mut self,
        input: Option<&Value<'scope>>,
    ) -> Result<SecurityPolicyViolationEventOptions, Failure> {
        self.work(Work::Argument, 1).map_err(Failure)?;
        let input = match input {
            Some(value) if self.dictionary("SecurityPolicyViolationEventInit", value)? => {
                Some(value)
            }
            _ => None,
        };
        let flags = match input {
            Some(value) => *self.value(&declarations::event_init(), value)?.get(),
            None => EventInit::default(),
        };
        let blocked_uri = self.violation_usv_string(input, BLOCKED_URI)?;
        let column_number = self.violation_unsigned_long(input, COLUMN_NUMBER)?;
        let disposition = *self
            .violation_member(
                input,
                DISPOSITION,
                DISPOSITION_TYPE.value_type(),
                SecurityPolicyViolationEventDisposition::Enforce,
            )?
            .get();
        let document_uri = self.violation_usv_string(input, DOCUMENT_URI)?;
        let effective_directive = self.violation_dom_string(input, EFFECTIVE_DIRECTIVE)?;
        let line_number = self.violation_unsigned_long(input, LINE_NUMBER)?;
        let original_policy = self.violation_dom_string(input, ORIGINAL_POLICY)?;
        let referrer = self.violation_usv_string(input, REFERRER)?;
        let sample = self.violation_dom_string(input, SAMPLE)?;
        let source_file = self.violation_usv_string(input, SOURCE_FILE)?;
        let status_code = self.violation_member(
            input,
            STATUS_CODE,
            &UNSIGNED_SHORT,
            IntegerValue::UnsignedShort(0),
        )?;
        let IntegerValue::UnsignedShort(status_code) = *status_code.get() else {
            return Err(self.host(Error::BackendUnavailable));
        };
        let violated_directive = self.violation_dom_string(input, VIOLATED_DIRECTIVE)?;
        Ok(SecurityPolicyViolationEventOptions {
            bubbles: flags.bubbles,
            cancelable: flags.cancelable,
            composed: flags.composed,
            blocked_uri,
            column_number,
            disposition,
            document_uri,
            effective_directive,
            line_number,
            original_policy,
            referrer,
            sample,
            source_file,
            status_code,
            violated_directive,
        })
    }

    fn violation_member<D: Descriptor>(
        &mut self,
        input: Option<&Value<'scope>>,
        key: &'static [u16],
        ty: &D,
        default: D::Output,
    ) -> Result<Converted<D::Output>, Failure> {
        if let Some(input) = input
            && let Some(value) = self.member(input, true, key)?
        {
            return self.value(ty, &value);
        }
        Ok(self.plain(default))
    }

    fn violation_unsigned_long(
        &mut self,
        input: Option<&Value<'scope>>,
        key: &'static [u16],
    ) -> Result<u32, Failure> {
        let value =
            self.violation_member(input, key, &UNSIGNED_LONG, IntegerValue::UnsignedLong(0))?;
        match *value.get() {
            IntegerValue::UnsignedLong(value) => Ok(value),
            _ => Err(self.host(Error::BackendUnavailable)),
        }
    }

    fn violation_dom_string(
        &mut self,
        input: Option<&Value<'scope>>,
        key: &'static [u16],
    ) -> Result<Converted<DomString>, Failure> {
        self.violation_member(input, key, &types::DOM_STRING, DomString::new(Vec::new()))
    }

    fn violation_usv_string(
        &mut self,
        input: Option<&Value<'scope>>,
        key: &'static [u16],
    ) -> Result<Converted<UsvString>, Failure> {
        self.violation_member(
            input,
            key,
            &types::USV_STRING,
            UsvString::normalize(Vec::new()),
        )
    }
}
