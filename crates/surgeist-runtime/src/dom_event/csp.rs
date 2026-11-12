//! Typed payload shared by native and authored SecurityPolicyViolationEvent.

use super::*;
use crate::trusted_types_csp::{Directive, PolicyDelivery};

const POLICY_RESOURCE: &[u16] = &ascii(b"trusted-types-policy");
const SINK_RESOURCE: &[u16] = &ascii(b"trusted-types-sink");
const POLICY_DIRECTIVE: &[u16] = &ascii(b"trusted-types");
const SINK_DIRECTIVE: &[u16] = &ascii(b"require-trusted-types-for");

#[derive(Clone, Copy)]
pub(crate) enum CspEventText {
    DocumentUri,
    Referrer,
    BlockedUri,
    EffectiveDirective,
    ViolatedDirective,
    OriginalPolicy,
    SourceFile,
    Sample,
}

const fn ascii<const N: usize>(bytes: &[u8; N]) -> [u16; N] {
    let mut result = [0; N];
    let mut i = 0;
    while i < N {
        result[i] = bytes[i] as u16;
        i += 1;
    }
    result
}

pub(crate) struct AuthoredCspViolationInit<'a> {
    pub(crate) event: DomEventInit,
    pub(crate) text: [&'a [u16]; 8],
    pub(crate) report_only: bool,
    pub(crate) status_code: u16,
    pub(crate) line_number: u32,
    pub(crate) column_number: u32,
}

pub(crate) struct CspEventData {
    text: [EventText; 8],
    report: Option<Rc<QueuedViolation>>,
    pub(crate) report_only: bool,
    pub(crate) status_code: u16,
    pub(crate) line_number: u32,
    pub(crate) column_number: u32,
    _backing: EventAllocation,
}

impl CspEventData {
    pub(crate) fn text(&self, member: CspEventText) -> &[u16] {
        if let Some(queued) = &self.report {
            let report = queued.report();
            match member {
                CspEventText::BlockedUri => {
                    return match report.directive() {
                        Directive::TrustedTypes => POLICY_RESOURCE,
                        Directive::RequireTrustedTypesFor => SINK_RESOURCE,
                    };
                }
                CspEventText::EffectiveDirective | CspEventText::ViolatedDirective => {
                    return match report.directive() {
                        Directive::TrustedTypes => POLICY_DIRECTIVE,
                        Directive::RequireTrustedTypesFor => SINK_DIRECTIVE,
                    };
                }
                CspEventText::OriginalPolicy => return report.original_policy(),
                CspEventText::Sample => return report.sample(),
                CspEventText::DocumentUri | CspEventText::Referrer | CspEventText::SourceFile => {}
            }
        }
        self.text[member as usize].code_units()
    }

    pub(super) fn authored(
        dom: &mut DomEventDom<'_>,
        storage: &EventStorage,
        init: &AuthoredCspViolationInit<'_>,
        cancel: &AtomicBool,
    ) -> Result<Rc<Self>, DomEventDispatchError> {
        let (text, backing) = Self::retain_text(dom, storage, init.text, cancel)?;
        Ok(Rc::new(Self {
            text,
            report: None,
            report_only: init.report_only,
            status_code: init.status_code,
            line_number: init.line_number,
            column_number: init.column_number,
            _backing: backing,
        }))
    }

    pub(super) fn native(
        dom: &mut DomEventDom<'_>,
        storage: &EventStorage,
        document: surgeist_dom::DocumentId,
        report: Rc<QueuedViolation>,
        cancel: &AtomicBool,
    ) -> Result<Rc<Self>, DomEventDispatchError> {
        let snapshot = dom.snapshot();
        let document = snapshot
            .node_with(document.node(), || storage.step(1, cancel))?
            .as_document()
            .ok_or(DomError::WrongNodeKind)?;
        let metadata = document.metadata();
        // The hosted initial profile has no HTTP response. A later navigation
        // producer must supply response status and qualified URL stripping.
        if !matches!(
            metadata.creation_cause(),
            surgeist_dom::DomDocumentCreationCause::HostedInitialAboutBlank { .. }
        ) {
            return Err(DomError::HostedSourceUnavailable.into());
        }
        let mut text = [&[][..]; 8];
        text[CspEventText::DocumentUri as usize] =
            strip_non_http(metadata.url().code_units(), storage, cancel)?;
        if let Some(referrer) = metadata
            .hosted_referrer()
            .ok_or(DomError::HostedSourceUnavailable)?
        {
            text[CspEventText::Referrer as usize] =
                strip_non_http(referrer.code_units(), storage, cancel)?;
        }
        let (text, backing) = Self::retain_text(dom, storage, text, cancel)?;
        let report_only = report.report().delivery() == PolicyDelivery::ReportOnlyHeader;
        Ok(Rc::new(Self {
            text,
            report: Some(report),
            report_only,
            status_code: 0,
            line_number: 0,
            column_number: 0,
            _backing: backing,
        }))
    }

    fn retain_text(
        dom: &mut DomEventDom<'_>,
        storage: &EventStorage,
        input: [&[u16]; 8],
        cancel: &AtomicBool,
    ) -> Result<([EventText; 8], EventAllocation), DomEventDispatchError> {
        let bytes = size_of::<Self>() + 2 * size_of::<usize>();
        let backing = storage.reserve(dom, bytes, bytes, 0, cancel)?;
        let mut text = [const { EventText::Static(&[]) }; 8];
        for (retained, units) in text.iter_mut().zip(input) {
            if !units.is_empty() {
                *retained = EventText::retain(storage, dom, units, cancel)?;
            }
        }
        Ok((text, backing))
    }
}

fn strip_non_http<'a>(
    url: &'a [u16],
    storage: &EventStorage,
    cancel: &AtomicBool,
) -> Result<&'a [u16], DomEventDispatchError> {
    for (index, unit) in url.iter().enumerate() {
        storage.step(1, cancel)?;
        if *unit == u16::from(b':') {
            let scheme = &url[..index];
            if scheme == ascii(b"http") || scheme == ascii(b"https") {
                break;
            }
            return Ok(scheme);
        }
    }
    Err(DomError::HostedSourceUnavailable.into())
}
