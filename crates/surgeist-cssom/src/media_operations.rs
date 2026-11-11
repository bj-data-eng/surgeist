//! Source MediaList operations over the shared live collection owner.
use crate::parser_resources::check_resource_diagnostics;
use crate::sheet_operations::{check_cancellation, sheet_utf16_error, sheet_utf16_limits};
use crate::*;
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_css::component::CssValueOrigin;
use surgeist_css::diagnostic::CssRecoveryDiagnostic;
use surgeist_css::query::media::{CssMediaQuery, CssMediaQueryList};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::source::{
    CssParsedOrigin, CssUtf16InputError, CssUtf16Resource, same_provenance,
};
use surgeist_css::{CssParseReport, CssParser, CssParserContext};

/// A source mutation's disposition and original recovered parser diagnostics.
/// Publication is reported separately by the enclosing batch commit.
#[derive(Clone, Debug)]
pub struct CssomMediaEdit {
    changed: bool,
    diagnostics: Vec<CssRecoveryDiagnostic>,
}
impl CssomMediaEdit {
    pub(crate) fn into_diagnostics(self) -> Vec<CssRecoveryDiagnostic> {
        self.diagnostics
    }
    pub const fn changed(&self) -> bool {
        self.changed
    }
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.diagnostics
    }
}

impl CssomSnapshot {
    /// Canonical CSSOM text of the associated current collection.
    pub(crate) fn media_text(
        &self,
        id: &CssomMediaListId,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        self.media(id)?
            .serialize_cssom(limits)
            .map(|text| text.as_css().to_owned())
            .map_err(CssomError::Media)
    }
    pub(crate) fn media_length(&self, id: &CssomMediaListId) -> Result<usize, CssomError> {
        Ok(self.media(id)?.queries().len())
    }
    /// Returns null out of range, independently of the output allowance.
    pub(crate) fn media_item(
        &self,
        id: &CssomMediaListId,
        index: usize,
        limits: CssSerializationLimits,
    ) -> Result<Option<String>, CssomError> {
        self.media(id)?
            .queries()
            .get(index)
            .map(|query| {
                query
                    .serialize_cssom(limits)
                    .map(|text| text.as_css().to_owned())
                    .map_err(CssomError::Media)
            })
            .transpose()
    }
}

impl CssomBatch {
    /// Replaces the contents through CSS-owned list recovery; empty input clears.
    /// No sheet security/modification guard or owner-attribute write is implied.
    /// An interruption aborts staging. Composed publishers check the same
    /// signal at their final publication fence.
    pub fn set_media_text<'a>(
        &mut self,
        id: &CssomMediaListId,
        source: impl Into<CssomSheetText<'a>>,
        cancel: &AtomicBool,
    ) -> Result<CssomMediaEdit, CssomError> {
        let edited = self.apply(|this| -> Result<_, CssomError> {
            let current = this.staged.media(id)?;
            check_cancellation(cancel)?;
            let source = source.into();
            let exact = matches!(&source, CssomSheetText::Utf16(_));
            let empty = match &source {
                CssomSheetText::Utf8(source) => source.is_empty(),
                CssomSheetText::Utf16(source) => source.is_empty(),
            };
            let (mut value, diagnostics, work) = if empty {
                (CssMediaQueryList::new(Vec::new()), Vec::new(), 0)
            } else {
                let (value, diagnostics, work) =
                    parse_media(source, &this.limits, CssParserContext::default(), cancel)?;
                value
                    .serialize_cssom(this.limits.css)
                    .map_err(CssomError::Media)?;
                (value, diagnostics, work)
            };
            let source_changed = if exact {
                let mut remaining = sheet_utf16_limits(&this.limits)
                    .max_work
                    .saturating_sub(work);
                let changed = exact_source_changed(current, &value, &mut remaining, cancel)?;
                // Returning to the original exact input reuses its source owner.
                // The prepared commit can then recognize a net no-op by identity.
                if changed == Some(true)
                    && let Some(base) = this.base.media.get(id)
                    && exact_media_source_differs(Some(current), Some(base))
                    && exact_source_changed(base, &value, &mut remaining, cancel)? == Some(false)
                {
                    value = base.clone();
                }
                changed
            } else {
                None
            };
            Ok(CssomMediaEdit {
                changed: this.replace_media(id, value, false, source_changed)?,
                diagnostics,
            })
        })?;
        self.finish_cancellable_staging(edited, cancel)
    }
    /// Appends a single recovered query unless its canonical bytes already exist.
    pub fn append_medium<'a>(
        &mut self,
        id: &CssomMediaListId,
        source: impl Into<CssomSheetText<'a>>,
        cancel: &AtomicBool,
    ) -> Result<CssomMediaEdit, CssomError> {
        self.edit_medium(id, source, false, cancel)
    }
    /// Removes every canonical match; a valid absent candidate is NotFound.
    /// Empty/multiple-query input is a successful no-op.
    pub fn delete_medium<'a>(
        &mut self,
        id: &CssomMediaListId,
        source: impl Into<CssomSheetText<'a>>,
        cancel: &AtomicBool,
    ) -> Result<CssomMediaEdit, CssomError> {
        self.edit_medium(id, source, true, cancel)
    }
    fn edit_medium<'a>(
        &mut self,
        id: &CssomMediaListId,
        source: impl Into<CssomSheetText<'a>>,
        remove: bool,
        cancel: &AtomicBool,
    ) -> Result<CssomMediaEdit, CssomError> {
        let edited = self.apply(|this| {
            let current = this.staged.media(id)?;
            check_cancellation(cancel)?;
            let parser = CssParser::default();
            let (report, work) = match source.into() {
                CssomSheetText::Utf8(source) => {
                    check_input(source, &this.limits)?;
                    (CssMediaQuery::parse_medium(&parser, source), 0)
                }
                CssomSheetText::Utf16(source) => CssMediaQuery::parse_medium_utf16_with_work(
                    &parser,
                    source,
                    sheet_utf16_limits(&this.limits),
                    || check_cancellation(cancel),
                )
                .map_err(sheet_utf16_error)?,
            };
            let (candidate, diagnostics) = recovered(report)?;
            let Some(mut candidate) = candidate else {
                return Ok(CssomMediaEdit {
                    changed: false,
                    diagnostics,
                });
            };
            // CSS owns cumulative source/projection/output accounting for the
            // candidate and every scanned member. No token spelling/evaluation
            // or separately reset pair allowance supplies equality.
            let scan_len = current
                .queries()
                .len()
                .checked_add(1)
                .ok_or(CssomError::Limit {
                    resource: "media scan capacity",
                    maximum: this.limits.max_entries,
                })?;
            let mut scan = Vec::new();
            scan.try_reserve(scan_len).map_err(|_| CssomError::Limit {
                resource: "media scan capacity",
                maximum: this.limits.max_entries,
            })?;
            scan.push(candidate.clone());
            scan.extend_from_slice(current.queries());
            let texts = CssMediaQueryList::new(scan)
                .serialize_cssom_members(this.limits.css)
                .map_err(CssomError::Media)?;
            let candidate_text = texts[0].as_css();
            let found = texts[1..]
                .iter()
                .any(|text| text.as_css() == candidate_text);
            if !remove && found {
                return Ok(CssomMediaEdit {
                    changed: false,
                    diagnostics,
                });
            }
            if remove && !found {
                return Err(CssomError::Source(CssomException::NotFound));
            }
            let mut selected = current.queries().to_vec();
            if remove {
                let mut index = 0;
                selected.retain(|_| {
                    let keep = texts[index + 1].as_css() != candidate_text;
                    index += 1;
                    keep
                });
            } else {
                // A previous edit can remove a base member, and later deletes
                // can shift this append into any earlier position. Reuse a
                // matching original now while this append owns work and polling.
                if let Some(base) = this
                    .base
                    .media
                    .get(id)
                    .filter(|base| !std::ptr::eq(*base, current))
                    && let Some(original) = retained_base_member(
                        base,
                        &candidate,
                        sheet_utf16_limits(&this.limits)
                            .max_work
                            .saturating_sub(work),
                        cancel,
                    )?
                {
                    candidate = original.clone();
                }
                selected.push(candidate);
            }
            Ok(CssomMediaEdit {
                changed: this.set_media(id, CssMediaQueryList::new(selected))?,
                diagnostics,
            })
        })?;
        self.finish_cancellable_staging(edited, cancel)
    }
}

pub(crate) fn parse_media(
    source: CssomSheetText<'_>,
    limits: &CssomLimits,
    context: CssParserContext,
    cancel: &AtomicBool,
) -> Result<(CssMediaQueryList, Vec<CssRecoveryDiagnostic>, usize), CssomError> {
    let parser = CssParser::new(context, Default::default());
    let (report, work) = match source {
        CssomSheetText::Utf8(source) => {
            check_input(source, limits)?;
            (CssMediaQueryList::parse(&parser, source), 0)
        }
        CssomSheetText::Utf16(source) => CssMediaQueryList::parse_utf16_with_work(
            &parser,
            source,
            sheet_utf16_limits(limits),
            || check_cancellation(cancel),
        )
        .map_err(sheet_utf16_error)?,
    };
    let (value, diagnostics) = recovered(report)?;
    Ok((value, diagnostics, work))
}

fn exact_source_changed(
    current: &CssMediaQueryList,
    next: &CssMediaQueryList,
    remaining_work: &mut usize,
    cancel: &AtomicBool,
) -> Result<Option<bool>, CssomError> {
    let ((current_context, current), (next_context, next)) =
        match (exact_media_source(current), exact_media_source(next)) {
            (Some(current), Some(next)) => (current, next),
            (None, None) => return Ok(None),
            _ => return Ok(Some(true)),
        };
    if current_context != next_context {
        return Ok(Some(true));
    }
    same_exact_units(current, next, remaining_work, cancel).map(|same| Some(!same))
}

fn charge_comparison(remaining_work: &mut usize, cancel: &AtomicBool) -> Result<(), CssomError> {
    check_cancellation(cancel)?;
    *remaining_work = remaining_work
        .checked_sub(1)
        .ok_or_else(|| CssomError::Utf16Input(CssUtf16InputError::limit(CssUtf16Resource::Work)))?;
    Ok(())
}

fn same_exact_units(
    left: &Arc<Vec<u16>>,
    right: &Arc<Vec<u16>>,
    remaining_work: &mut usize,
    cancel: &AtomicBool,
) -> Result<bool, CssomError> {
    if Arc::ptr_eq(left, right) {
        return Ok(true);
    }
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.iter().zip(right.iter()) {
        charge_comparison(remaining_work, cancel)?;
        if left != right {
            return Ok(false);
        }
    }
    Ok(true)
}

fn exact_member_origin(query: &CssMediaQuery) -> Option<(&CssParsedOrigin, &Arc<Vec<u16>>)> {
    let CssValueOrigin::Parsed(origin) = query.origin() else {
        return None;
    };
    Some((origin, origin.source().original_utf16()?))
}

fn retained_base_member<'a>(
    base: &'a CssMediaQueryList,
    candidate: &CssMediaQuery,
    mut remaining_work: usize,
    cancel: &AtomicBool,
) -> Result<Option<&'a CssMediaQuery>, CssomError> {
    let Some((origin, input)) = exact_member_origin(candidate) else {
        return Ok(None);
    };
    for member in base.queries() {
        charge_comparison(&mut remaining_work, cancel)?;
        if let Some((previous, previous_input)) = exact_member_origin(member)
            && previous.span() == origin.span()
            && same_exact_units(previous_input, input, &mut remaining_work, cancel)?
            && same_provenance(member, candidate)
        {
            return Ok(Some(member));
        }
    }
    Ok(None)
}

fn exact_media_source(list: &CssMediaQueryList) -> Option<(CssParserContext, &Arc<Vec<u16>>)> {
    let parsed = list.parse_source()?;
    Some((parsed.context(), parsed.source().original_utf16()?))
}

pub(crate) fn exact_media_source_differs(
    left: Option<&CssMediaQueryList>,
    right: Option<&CssMediaQueryList>,
) -> bool {
    match (
        left.and_then(exact_media_source),
        right.and_then(exact_media_source),
    ) {
        (None, None) => false,
        (Some((left_context, left)), Some((right_context, right))) => {
            left_context != right_context || !Arc::ptr_eq(left, right)
        }
        _ => true,
    }
}

fn check_input(source: &str, limits: &CssomLimits) -> Result<(), CssomError> {
    if source.len() > limits.max_input_bytes {
        return Err(CssomError::Limit {
            resource: "parse input bytes",
            maximum: limits.max_input_bytes,
        });
    }
    Ok(())
}
fn recovered<T>(report: CssParseReport<T>) -> Result<(T, Vec<CssRecoveryDiagnostic>), CssomError> {
    check_resource_diagnostics(report.diagnostics())?;
    Ok(report.into_parts())
}

impl CssomBatch {
    /// Product typed media replacement uses the existing checked Media Queries carrier.
    pub fn set_media(
        &mut self,
        id: &CssomMediaListId,
        value: CssMediaQueryList,
    ) -> Result<bool, CssomError> {
        self.set_media_with_source(id, value, false)
    }

    pub(crate) fn set_media_with_source(
        &mut self,
        id: &CssomMediaListId,
        value: CssMediaQueryList,
        retain_input: bool,
    ) -> Result<bool, CssomError> {
        self.replace_media(id, value, retain_input, None)
    }

    fn replace_media(
        &mut self,
        id: &CssomMediaListId,
        value: CssMediaQueryList,
        retain_input: bool,
        source_changed: Option<bool>,
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            this.staged.media(id)?;
            let changed =
                source_changed.unwrap_or_else(|| !same_provenance(&this.staged.media[id], &value));
            if !changed && !retain_input {
                return Ok(false);
            }
            let sheets = this
                .staged
                .sheets
                .iter()
                .filter(|(_, sheet)| &sheet.media == id)
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>();
            for sheet in sheets {
                this.staged
                    .sheets
                    .get_mut(&sheet)
                    .expect("selected sheet")
                    .inputs
                    .media = value.clone();
            }
            this.staged.media.insert(id.clone(), value);
            if changed {
                this.affect(CssomObjectId::MediaList(id.clone()), &[CssomChange::Media]);
            }
            Ok(changed)
        })
    }
}
