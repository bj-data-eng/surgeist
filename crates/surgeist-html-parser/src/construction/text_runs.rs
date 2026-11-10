//! Body text partitioned into exact runs, and the splits that stage them.

use super::*;

/// Splits only a leading run of the five HTML ASCII whitespace code points.
/// The returned prefix and retained suffix have real owned backing and exact
/// origin ranges. No source-coordinate range is guessed from output offsets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BodyTextKind {
    Null,
    Whitespace,
    Other,
}

pub(crate) struct TextRun {
    pub(crate) kind: BodyTextKind,
    pub(crate) text: Text,
}

/// Actual lexical owners in original order. Consumed slots remain admitted
/// until collection disposal; no mutable vector escapes.
pub(crate) struct BodyTextRuns {
    pub(super) entries: Option<OwnedVec<Option<TextRun>>>,
    next: usize,
}

impl BodyTextRuns {
    pub(crate) fn take_next(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<TextRun>, SessionError> {
        let entries = self.entries.as_mut().ok_or(SessionError::Consumed)?;
        entries.require_owner(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if self.next == entries.len() {
            return Ok(None);
        }
        payload_write::<usize>(control, 1)?;
        payload_write::<Option<TextRun>>(control, 1)?;
        let next = self
            .next
            .checked_add(1)
            .ok_or_else(|| latch_registry_row(control))?;
        let Some(slot) = entries.get_mut(self.next, control)? else {
            return Err(latch_registry_row(control));
        };
        // A slot already transferred leaves the cursor where it was.
        let Some(value) = slot.take() else {
            return Err(latch_registry_row(control));
        };
        self.next = next;
        Ok(Some(value))
    }
    /// Preflight precedes disposal. Once it begins, newly latched work or
    /// cancellation stops cannot strand later nested owners.
    pub(crate) fn release(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(entries) = self.entries.as_ref() else {
            return Ok(());
        };
        entries.require_disposal_owner(control)?;
        for entry in entries.as_slice() {
            cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(run) = entry {
                check_disposal_text(control, &run.text)?;
            }
        }
        if control.stop().is_none() {
            payload_write::<Option<OwnedVec<Option<TextRun>>>>(control, 1)?;
        }
        let Some(mut entries) = self.entries.take() else {
            return Ok(());
        };
        let mut failure = None;
        loop {
            let entry = if control.stop().is_some() {
                entries.cleanup_pop(control)?
            } else {
                match entries.try_pop(control) {
                    Ok(entry) => entry,
                    Err(stop) => {
                        failure = Some(stop);
                        entries.cleanup_pop(control)?
                    }
                }
            };
            let Some(entry) = entry else { break };
            if let Some(run) = entry {
                run.text.release(control)?;
            }
        }
        entries.release(control)?;
        match failure {
            Some(stop) => Err(stop.into()),
            None => Ok(()),
        }
    }
}

struct TextRunDraft {
    kind: BodyTextKind,
    value: Utf16Buffer,
    origins: OwnedVec<HtmlParseTextOrigin>,
}

impl TextRunDraft {
    fn new(kind: BodyTextKind, control: &mut ParseControl) -> Result<Self, SessionError> {
        Ok(Self {
            kind,
            value: Utf16Buffer::new(control)?,
            origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
        })
    }
    fn release(self, control: &mut ParseControl) -> Result<(), SessionError> {
        self.value.release(control)?;
        self.origins.release(control)?;
        Ok(())
    }
    fn freeze(self, control: &mut ParseControl) -> Result<TextRun, SessionError> {
        let Self {
            kind,
            value,
            origins,
        } = self;
        if let Err(error) = payload_write::<TextRun>(control, 2) {
            value.release(control)?;
            origins.release(control)?;
            return Err(error);
        }
        match value.freeze(control) {
            Ok(value) => Ok(TextRun {
                kind,
                text: Text { value, origins },
            }),
            Err(stop) => {
                origins.release(control)?;
                Err(stop.into())
            }
        }
    }
}

#[derive(Clone, Copy)]
enum TextPartition {
    Body,
    FirstLf,
}

fn body_text_kind(unit: u16) -> BodyTextKind {
    match unit {
        0 => BodyTextKind::Null,
        unit if surgeist_html::bounded::lexical::is_ascii_whitespace(unit) => {
            BodyTextKind::Whitespace
        }
        _ => BodyTextKind::Other,
    }
}

pub(super) fn discard_runs(
    runs: &mut BodyTextRuns,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    match runs.release(control) {
        Err(_) if control.stop().is_some() && runs.entries.is_some() => runs.release(control),
        Err(_) if control.stop().is_some() && runs.entries.is_none() => Ok(()),
        result => result,
    }
}

fn push_text_run(
    entries: &mut OwnedVec<Option<TextRun>>,
    draft: TextRunDraft,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    let run = draft.freeze(control)?;
    if let Err((stop, run)) = entries.try_push_recover_geometric(control, Some(run)) {
        // The refused push hands back the run it was given.
        if let Some(run) = run {
            run.text.release(control)?;
        }
        return Err(stop.into());
    }
    Ok(())
}

/// One origin traversal and one classification read per code unit. Every exact
/// origin is copied to one admitted owner, never repeatedly to suffixes.
/// An origin spanning a requested cut is refused rather than guessed/sliced.
fn stage_text_runs(
    text: &Text,
    partition: TextPartition,
    control: &mut ParseControl,
) -> Result<BodyTextRuns, SessionError> {
    let mut runs = BodyTextRuns {
        entries: Some(OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?),
        next: 0,
    };
    let mut draft: Option<TextRunDraft> = None;
    let result = (|| {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let units = text.value.text().code_units();
        let len = units.len();
        if !control.detail().records_graph() {
            // No origin splits the text, so each maximal run of one kind is a
            // run of its own.
            let mut index = 0;
            while index < len {
                let mut kind = None;
                let mut end = index;
                while let Some(&unit) = units.get(end) {
                    control.charge_unit_inspection()?;
                    let current = match partition {
                        TextPartition::Body => body_text_kind(unit),
                        TextPartition::FirstLf if end == 0 && unit == 10 => {
                            BodyTextKind::Whitespace
                        }
                        TextPartition::FirstLf => BodyTextKind::Other,
                    };
                    if kind.is_some_and(|old| old != current) {
                        break;
                    }
                    kind = Some(current);
                    end = end
                        .checked_add(1)
                        .ok_or_else(|| latch_registry_row(control))?;
                }
                let Some(kind) = kind else {
                    return Err(latch_registry_row(control));
                };
                let Some(entries) = runs.entries.as_mut() else {
                    return Err(latch_registry_row(control));
                };
                if let Some(previous) = draft.take_if(|draft| draft.kind != kind) {
                    push_text_run(entries, previous, control)?;
                }
                let current = match &mut draft {
                    Some(current) => current,
                    empty => empty.insert(TextRunDraft::new(kind, control)?),
                };
                let Some(run_units) = units.get(index..end) else {
                    return Err(latch_registry_row(control));
                };
                current.value.extend_units(run_units, control)?;
                index = end;
            }
            if let Some(last) = draft.take() {
                let Some(entries) = runs.entries.as_mut() else {
                    return Err(latch_registry_row(control));
                };
                push_text_run(entries, last, control)?;
            }
            return Ok(());
        }
        let mut covered = 0;
        for origin in text.origins.as_slice() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            control.require_source_range(&origin.source)?;
            if origin.start != covered || origin.end <= origin.start || origin.end > len {
                return Err(SessionError::OriginSplitUnavailable);
            }
            let mut kind = None;
            for index in origin.start..origin.end {
                control.charge_unit_inspection()?;
                let Some(&unit) = units.get(index) else {
                    return Err(latch_registry_row(control));
                };
                let current = match partition {
                    TextPartition::Body => body_text_kind(unit),
                    TextPartition::FirstLf if index == 0 && unit == 10 => BodyTextKind::Whitespace,
                    TextPartition::FirstLf => BodyTextKind::Other,
                };
                if kind.is_some_and(|old| old != current) {
                    return Err(SessionError::OriginSplitUnavailable);
                }
                kind = Some(current);
            }
            let Some(kind) = kind else {
                return Err(latch_registry_row(control));
            };
            let Some(entries) = runs.entries.as_mut() else {
                return Err(latch_registry_row(control));
            };
            if let Some(previous) = draft.take_if(|draft| draft.kind != kind) {
                push_text_run(entries, previous, control)?;
            }
            let current = match &mut draft {
                Some(current) => current,
                empty => empty.insert(TextRunDraft::new(kind, control)?),
            };
            let start = current.value.len();
            let Some(origin_units) = units.get(origin.start..origin.end) else {
                return Err(latch_registry_row(control));
            };
            current.value.extend_units(origin_units, control)?;
            control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            payload_write::<crate::HtmlSourceRange>(control, 1)?;
            current.origins.try_push_geometric(
                control,
                HtmlParseTextOrigin {
                    start,
                    end: current.value.len(),
                    source: origin.source.clone(),
                    kind: origin.kind,
                },
            )?;
            covered = origin.end;
        }
        if covered != len {
            return Err(SessionError::OriginSplitUnavailable);
        }
        if draft.is_some() {
            let Some(entries) = runs.entries.as_mut() else {
                return Err(latch_registry_row(control));
            };
            if let Some(last) = draft.take() {
                push_text_run(entries, last, control)?;
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        if let Some(draft) = draft.take() {
            draft.release(control)?;
        }
        discard_runs(&mut runs, control)?;
        return Err(error);
    }
    Ok(runs)
}

pub(crate) fn partition_body_text(
    control: &mut ParseControl,
    input: &mut Option<Text>,
) -> Result<BodyTextRuns, SessionError> {
    let text = input.as_mut().ok_or(SessionError::Consumed)?;
    SessionStorage::check_text(control, text)?;
    let mut runs = stage_text_runs(text, TextPartition::Body, control)?;
    let original = match take_payload(input, control) {
        Ok(original) => original,
        Err(error) => {
            discard_runs(&mut runs, control)?;
            return Err(error);
        }
    };
    original.release(control)?;
    Ok(runs)
}

/// Source-required first LF omission, retaining its complete original origin
/// (including CRLF preprocessing) and every later unit in the suffix.
pub(crate) fn split_first_lf(
    control: &mut ParseControl,
    input: &mut Option<Text>,
) -> Result<Option<Text>, SessionError> {
    let text = input.as_mut().ok_or(SessionError::Consumed)?;
    SessionStorage::check_text(control, text)?;
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    if text.value.text().is_empty() {
        return Ok(None);
    }
    control.charge_unit_inspection()?;
    if text.value.text().code_units().first() != Some(&10) {
        return Ok(None);
    }
    if text.value.text().len() == 1 {
        check_text_source(control, text)?;
        return Ok(Some(take_payload(input, control)?));
    }
    let mut runs = stage_text_runs(text, TextPartition::FirstLf, control)?;
    let prefix = match runs.take_next(control) {
        Ok(Some(run)) => run.text,
        // A text longer than one unit that starts with LF stages two runs.
        Ok(None) => {
            discard_runs(&mut runs, control)?;
            return Err(latch_registry_row(control));
        }
        Err(error) => {
            discard_runs(&mut runs, control)?;
            return Err(error);
        }
    };
    let suffix = match runs.take_next(control) {
        Ok(Some(run)) => run.text,
        Ok(None) => {
            prefix.release(control)?;
            discard_runs(&mut runs, control)?;
            return Err(latch_registry_row(control));
        }
        Err(error) => {
            prefix.release(control)?;
            discard_runs(&mut runs, control)?;
            return Err(error);
        }
    };
    if let Err(error) = payload_write::<Option<Text>>(control, 2) {
        prefix.release(control)?;
        suffix.release(control)?;
        discard_runs(&mut runs, control)?;
        return Err(error);
    }
    // Scratch disposal is still fallible while input remains in its caller.
    if let Err(error) = runs.release(control) {
        prefix.release(control)?;
        suffix.release(control)?;
        discard_runs(&mut runs, control)?;
        return Err(error);
    }
    // The original is still in the caller's slot; the suffix replaces it.
    let Some(original) = input.replace(suffix) else {
        return Ok(Some(prefix));
    };
    original.release(control)?;
    Ok(Some(prefix))
}

pub(crate) fn split_leading_ascii_whitespace(
    control: &mut ParseControl,
    input: &mut Option<Text>,
) -> Result<Option<Text>, SessionError> {
    let text = input.as_mut().ok_or(SessionError::Consumed)?;
    SessionStorage::check_text(control, text)?;
    let mut cut = 0usize;
    for &unit in text.value.text().code_units() {
        control.charge_unit_inspection()?;
        if !surgeist_html::bounded::lexical::is_ascii_whitespace(unit) {
            break;
        }
        // The run of whitespace cannot outgrow the text it is counted in.
        cut = cut.saturating_add(1);
    }
    if cut == 0 {
        return Ok(None);
    }
    if cut == text.value.text().len() {
        return Ok(Some(take_payload(input, control)?));
    }
    // Without a graph the text keeps no origins to cover or to split.
    if control.detail().records_graph() {
        let mut covered = 0;
        for origin in text.origins.as_slice() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if origin.start != covered
                || origin.end <= origin.start
                || origin.end > text.value.text().len()
            {
                return Err(SessionError::OriginSplitUnavailable);
            }
            if origin.start < cut && cut < origin.end {
                return Err(SessionError::OriginSplitUnavailable);
            }
            covered = origin.end;
        }
        if covered != text.value.text().len() {
            return Err(SessionError::OriginSplitUnavailable);
        }
    }
    let prefix = slice_text(text, 0, cut, control)?;
    let suffix = match slice_text(text, cut, text.value.text().len(), control) {
        Ok(text) => text,
        Err(error) => {
            prefix.release(control)?;
            return Err(error);
        }
    };
    if let Err(error) = payload_write::<Option<Text>>(control, 2) {
        prefix.release(control)?;
        suffix.release(control)?;
        return Err(error);
    }
    // The original is still in the caller's slot; the suffix replaces it.
    let Some(original) = input.replace(suffix) else {
        return Ok(Some(prefix));
    };
    original.release(control)?;
    Ok(Some(prefix))
}

fn slice_text(
    text: &Text,
    start: usize,
    end: usize,
    control: &mut ParseControl,
) -> Result<Text, SessionError> {
    let mut buffer = Utf16Buffer::new(control)?;
    let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
    let result = (|| {
        let Some(units) = text.value.text().code_units().get(start..end) else {
            return Err(latch_registry_row(control));
        };
        buffer.extend_units(units, control)?;
        for origin in text.origins.as_slice() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if origin.start < start || origin.end > end {
                continue;
            }
            control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            payload_write::<crate::HtmlSourceRange>(control, 1)?;
            let (Some(origin_start), Some(origin_end)) = (
                origin.start.checked_sub(start),
                origin.end.checked_sub(start),
            ) else {
                return Err(latch_registry_row(control));
            };
            origins.try_push_geometric(
                control,
                HtmlParseTextOrigin {
                    start: origin_start,
                    end: origin_end,
                    source: origin.source.clone(),
                    kind: origin.kind,
                },
            )?;
        }
        payload_write::<Text>(control, 2)?;
        Ok::<_, SessionError>(())
    })();
    if let Err(error) = result {
        buffer.release(control)?;
        origins.release(control)?;
        return Err(error);
    }
    let value = match buffer.freeze(control) {
        Ok(value) => value,
        Err(stop) => {
            origins.release(control)?;
            return Err(stop.into());
        }
    };
    Ok(Text { value, origins })
}
