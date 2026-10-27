//! Actual CSS-owned UTF16 color-scheme ingress under the original Source ledger.
use super::*;
use std::cell::Cell;
use surgeist_css::source::{CssRetainedPayloadError, CssUtf16InputError, CssUtf16ValueAdmission};
use surgeist_css::values::color_adjust::CssColorScheme;
use surgeist_css::{CssParser, CssParserContext, CssParserMode};

pub(super) struct Candidate {
    pub selected: crate::meta::SelectedColorScheme,
    pub formation: Option<source::formation::Formation>,
}
pub(super) fn payload_error(error: CssRetainedPayloadError<DomError>) -> DomError {
    match error.into_admission() {
        Ok(error) => error,
        // The measurement's only other refusal is a capacity overflow.
        Err(_) => DomError::RepresentationExhausted,
    }
}
fn input_error(error: CssUtf16InputError<DomError>) -> DomError {
    match error.into_interruption() {
        Ok(error) => error,
        Err(refusal) => {
            if let Some(resource) = refusal.exhausted_limit() {
                DomError::Meta(Failure::ColorSchemeLimit(resource))
            } else if refusal.is_allocation_failure() {
                DomError::AllocationFailed
            } else {
                // A capacity overflow.
                DomError::RepresentationExhausted
            }
        }
    }
}
pub(super) fn parse(
    draft: &Draft,
    document: Key,
    content: &DocumentString,
    limits: DomLimits,
    cancel: &AtomicBool,
) -> Result<Candidate, DomError> {
    let limits = limits
        .meta_color_scheme
        .ok_or(DomError::Meta(Failure::ColorSchemeProcessingUnavailable))?;
    let formation = source::formation::Formation::new(draft, cancel)?;
    let live = Cell::new(source::storage::Payload::default());
    let context = CssParserContext::new(match draft.document_metadata(document)?.mode() {
        surgeist_html::HtmlDocumentMode::Quirks => CssParserMode::Quirks,
        _ => CssParserMode::Standards,
    });
    let report = CssColorScheme::parse_utf16(
        &CssParser::new(context, Default::default()),
        content.shared_code_units().clone(),
        limits,
        |event| {
            draft.meta_step(cancel)?;
            if let CssUtf16ValueAdmission::Live(next) = event {
                let old = live.get();
                let next = source::storage::Payload {
                    bytes: next.bytes(),
                    units: next.utf16_units(),
                };
                let delta = source::storage::Payload {
                    bytes: next
                        .bytes
                        .checked_sub(old.bytes)
                        .ok_or(DomError::Meta(Failure::CssProcessingProtocol))?,
                    units: next
                        .units
                        .checked_sub(old.units)
                        .ok_or(DomError::Meta(Failure::CssProcessingProtocol))?,
                };
                if let Some(formation) = &formation {
                    formation.retain_payload(delta, cancel)?;
                }
                live.set(next);
            }
            Ok(())
        },
        |remaining| {
            let old = live.get();
            let remaining = source::storage::Payload {
                bytes: remaining.bytes(),
                units: remaining.utf16_units(),
            };
            if let Some(formation) = &formation {
                formation.release_payload(source::storage::Payload {
                    bytes: old
                        .bytes
                        .checked_sub(remaining.bytes)
                        .expect("CSS releases actual live bytes"),
                    units: old
                        .units
                        .checked_sub(remaining.units)
                        .expect("CSS releases actual live units"),
                });
            }
            live.set(remaining);
        },
    )
    .map_err(input_error)?;
    let usage = report.retained_storage();
    debug_assert_eq!(live.get().bytes, usage.bytes());
    debug_assert_eq!(live.get().units, usage.utf16_units());
    // The destination Arc body is allocated while the returned inline report
    // is still live. End that extra header debit only after the actual move.
    if let Some(formation) = &formation {
        formation.retain_bytes(
            size_of::<surgeist_css::values::color_adjust::CssColorSchemeParseReport>(),
            cancel,
        )?;
    }
    let report = Arc::new(report);
    if let Some(formation) = &formation {
        formation.release_payload(source::storage::Payload {
            bytes: size_of::<surgeist_css::values::color_adjust::CssColorSchemeParseReport>(),
            units: 0,
        });
    }
    Ok(Candidate {
        selected: crate::meta::SelectedColorScheme {
            report,
            bytes: usage.bytes(),
            units: usage.utf16_units(),
        },
        formation,
    })
}
