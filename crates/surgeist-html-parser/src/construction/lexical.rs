//! Actual consumed markup, retained before normal recovery disposes its drafts.
//! Original source pieces are independent of normalized native field backing.

use super::*;

/// Qualified handle to one actual occurrence. The flat owner alone stores indices.
pub(crate) struct LexicalRef {
    index: usize,
    issuer: Weak<evidence::EvidenceIdentity>,
    control: ControlIdentity,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseLexicalKind {
    Markup,
    StartTag,
    EndTag,
    Attribute,
    Comment,
    Doctype,
    ProcessingInstruction,
    Cdata,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseLexicalDisposition {
    Draft,
    Emitted,
    StoredAttribute,
    AlreadyPresentAttribute,
    DuplicateAttribute,
    EndTagAttribute,
    Abandoned,
    LiteralRecovery,
    Applied,
    Ignored,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseLexicalRole {
    Name,
    Value,
    PublicIdentifier,
    SystemIdentifier,
    Target,
    Data,
    Equals,
    OpeningQuote(crate::parser::tokenizer::states::HtmlTokenizerQuote),
    ClosingQuote(crate::parser::tokenizer::states::HtmlTokenizerQuote),
    Syntax,
    Whitespace,
    EndAnchor,
}
pub(crate) struct LexicalOccurrence {
    pub(crate) kind: HtmlParseLexicalKind,
    pub(crate) disposition: HtmlParseLexicalDisposition,
    pub(crate) source: crate::HtmlSourceRange,
    pub(crate) parent: Option<usize>,
    cause: usize,
    data_units: usize,
}
#[cfg(test)]
impl LexicalOccurrence {
    /// Names a cause the evidence does not hold.
    pub(crate) fn corrupt_cause_for_test(&mut self) {
        self.cause = usize::MAX;
    }
}
impl LexicalOccurrence {
    pub(crate) fn cause(&self) -> usize {
        self.cause
    }
    pub(crate) fn data_units(&self) -> usize {
        self.data_units
    }
}
pub(crate) struct LexicalPart {
    pub(crate) occurrence: usize,
    pub(crate) role: HtmlParseLexicalRole,
    pub(crate) source: crate::HtmlSourceRange,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) kind: Option<HtmlParseOriginKind>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseOmissionKind {
    LeadingWhitespace,
    InitialLf,
    NullInBody,
    NullInTable,
    FramesetCharacter,
    IgnoredText,
}
pub(crate) struct LexicalOmission {
    pub(crate) source: crate::HtmlSourceRange,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) origin: HtmlParseOriginKind,
    pub(crate) kind: HtmlParseOmissionKind,
    pub(crate) rule: HtmlParserRule,
    cause: usize,
}
impl LexicalOmission {
    pub(crate) fn cause(&self) -> usize {
        self.cause
    }
}
pub(super) struct LexicalStorage {
    pub(super) occurrences: OwnedVec<LexicalOccurrence>,
    pub(super) parts: OwnedVec<LexicalPart>,
    pub(super) omissions: OwnedVec<LexicalOmission>,
    pub(super) end: Option<crate::HtmlSourceRange>,
}
impl LexicalStorage {
    pub(super) fn new(c: &mut ParseControl) -> Result<Self, HtmlParseStop> {
        let mut occurrences = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        let mut parts = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        let mut omissions = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        occurrences.require_owner(c)?;
        parts.require_owner(c)?;
        omissions.require_owner(c)?;
        Ok(Self {
            occurrences,
            parts,
            omissions,
            end: None,
        })
    }
    pub(super) fn extents(&self, c: &mut ParseControl) -> Result<[(u64, u64); 3], HtmlParseStop> {
        self.occurrences.require_disposal_owner(c)?;
        self.parts.require_disposal_owner(c)?;
        self.omissions.require_disposal_owner(c)?;
        fn bytes<T>(values: &OwnedVec<T>, c: &mut ParseControl) -> Result<u64, HtmlParseStop> {
            values
                .capacity()
                .checked_mul(mem::size_of::<T>())
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::CapacityMismatch))
        }
        Ok([
            (self.occurrences.len() as u64, bytes(&self.occurrences, c)?),
            (self.parts.len() as u64, bytes(&self.parts, c)?),
            (self.omissions.len() as u64, bytes(&self.omissions, c)?),
        ])
    }
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        self.occurrences.release(c)?;
        self.parts.release(c)?;
        self.omissions.release(c)
    }
}
impl ParseEvidence {
    /// Whether the lexical records are complete: the parse did not select
    /// them, or the tokenizer reached the end of the input.
    pub(crate) fn lexical_complete(&self) -> bool {
        !self.records_lexical() || self.native.lexical.end.is_some()
    }
    /// Only the tokenizer's actual sealed-and-exhausted EOF transition supplies
    /// this checked point. A partial frontier is never lexical completion.
    pub(crate) fn lexical_eof(
        &mut self,
        c: &mut ParseControl,
        point: &crate::HtmlSourceRange,
    ) -> Result<(), HtmlParseStop> {
        if !self.records_lexical() {
            return Ok(());
        }
        self.native.lexical.occurrences.require_owner(c)?;
        c.require_source_range(point)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if point.start() != point.end()
            || self.native.lexical.end.is_some()
            || self
                .native
                .source
                .as_ref()
                .is_some_and(|source| point.end() != source.len())
        {
            return Err(c.fail_invariant(HtmlParseInvariant::TokenizerState));
        }
        c.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<Option<crate::HtmlSourceRange>>() as u64,
        )?;
        c.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        self.native.lexical.end = Some(point.clone());
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn lexical_observations(
        &self,
    ) -> (&[LexicalOccurrence], &[LexicalPart], &[LexicalOmission]) {
        (
            self.native.lexical.occurrences.as_slice(),
            self.native.lexical.parts.as_slice(),
            self.native.lexical.omissions.as_slice(),
        )
    }
    pub(crate) fn retain_omission(
        &mut self,
        c: &mut ParseControl,
        origins: &[HtmlParseTextOrigin],
        kind: HtmlParseOmissionKind,
        rule: HtmlParserRule,
    ) -> Result<(), SessionError> {
        if !self.records_lexical() {
            return Ok(());
        }
        let cause = self.field_cause(c)?;
        for origin in origins {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            c.require_source_range(&origin.source)?;
            payload_write::<LexicalOmission>(c, 1)?;
            c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            self.native.lexical.omissions.reserve_geometric(c, 1)?;
            self.native.lexical.omissions.try_push(
                c,
                LexicalOmission {
                    source: origin.source.clone(),
                    start: origin.start,
                    end: origin.end,
                    origin: origin.kind,
                    kind,
                    rule,
                    cause,
                },
            )?;
        }
        Ok(())
    }
    pub(crate) fn lexical_begin(
        &mut self,
        c: &mut ParseControl,
        source: &crate::HtmlSourceRange,
        kind: HtmlParseLexicalKind,
        parent: Option<&LexicalRef>,
    ) -> Result<Option<LexicalRef>, HtmlParseStop> {
        if !self.records_lexical() {
            return Ok(None);
        }
        c.require_source_range(source)?;
        let parent = parent
            .map(|parent| self.require_lexical(c, parent))
            .transpose()?;
        let cause = self
            .append_cause(c, ConstructionCause::Lexical { source: None })
            .map_err(|error| match error {
                SessionError::Stop(stop) => stop,
                _ => c.fail_invariant(HtmlParseInvariant::RegistryRow),
            })?;
        let index = self.native.lexical.occurrences.len();
        payload_write::<LexicalOccurrence>(c, 1).map_err(|error| match error {
            SessionError::Stop(stop) => stop,
            _ => c.fail_invariant(HtmlParseInvariant::RegistryRow),
        })?;
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        self.native.lexical.occurrences.reserve_geometric(c, 1)?;
        self.native.lexical.occurrences.try_push(
            c,
            LexicalOccurrence {
                kind,
                disposition: HtmlParseLexicalDisposition::Draft,
                source: source.clone(),
                parent,
                cause,
                data_units: 0,
            },
        )?;
        c.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<LexicalRef>() as u64,
        )?;
        c.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        Ok(Some(LexicalRef {
            index,
            issuer: Arc::downgrade(&self.identity),
            control: c.identity(),
        }))
    }
    fn require_lexical(
        &self,
        c: &mut ParseControl,
        reference: &LexicalRef,
    ) -> Result<usize, HtmlParseStop> {
        c.require_identity(&reference.control)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            mem::size_of::<Weak<evidence::EvidenceIdentity>>() as u64,
        )?;
        if reference.issuer.as_ptr() != Arc::as_ptr(&self.identity) {
            return Err(c.fail_invariant(HtmlParseInvariant::ForeignEvidence));
        }
        if self
            .native
            .lexical
            .occurrences
            .as_slice()
            .get(reference.index)
            .is_none()
        {
            return Err(c.fail_invariant(HtmlParseInvariant::RegistryRow));
        }
        Ok(reference.index)
    }
    /// The occurrence a validated reference names. A position past the
    /// occurrences cannot follow `require_lexical`; it is reported as the same
    /// foreign-evidence stop.
    fn lexical_occurrence(
        &self,
        c: &mut ParseControl,
        index: usize,
    ) -> Result<&LexicalOccurrence, HtmlParseStop> {
        self.native
            .lexical
            .occurrences
            .as_slice()
            .get(index)
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::RegistryRow))
    }
    pub(crate) fn lexical_data_offset(
        &self,
        c: &mut ParseControl,
        reference: &LexicalRef,
    ) -> Result<usize, HtmlParseStop> {
        let index = self.require_lexical(c, reference)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            mem::size_of::<usize>() as u64,
        )?;
        Ok(self.lexical_occurrence(c, index)?.data_units)
    }
    pub(crate) fn lexical_data_commit(
        &mut self,
        c: &mut ParseControl,
        reference: &LexicalRef,
        old: usize,
        new: usize,
    ) -> Result<(), HtmlParseStop> {
        let index = self.require_lexical(c, reference)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let occurrence = self.lexical_occurrence(c, index)?;
        if occurrence.disposition != HtmlParseLexicalDisposition::Draft
            || occurrence.data_units != old
            || new < old
        {
            return Err(c.fail_invariant(HtmlParseInvariant::TokenizerState));
        }
        c.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<usize>() as u64,
        )?;
        c.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
        self.native
            .lexical
            .occurrences
            .get_mut(index, c)?
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::RegistryRow))?
            .data_units = new;
        Ok(())
    }
    pub(crate) fn lexical_part(
        &mut self,
        c: &mut ParseControl,
        occurrence: &LexicalRef,
        role: HtmlParseLexicalRole,
        source: &crate::HtmlSourceRange,
        output: std::ops::Range<usize>,
        kind: Option<HtmlParseOriginKind>,
    ) -> Result<(), HtmlParseStop> {
        let occurrence = self.require_lexical(c, occurrence)?;
        c.require_source_range(source)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let record = self
            .native
            .lexical
            .occurrences
            .as_slice()
            .get(occurrence)
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::RegistryRow))?;
        if record.disposition != HtmlParseLexicalDisposition::Draft
            || source.source() != record.source.source()
            || source.coordinates() != record.source.coordinates()
            || output.start > output.end
        {
            return Err(c.fail_invariant(HtmlParseInvariant::TokenizerState));
        }
        if let Some(last) = self.native.lexical.parts.as_slice().last() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            c.charge(
                HtmlParseWorkCategory::InspectByte,
                mem::size_of::<LexicalPart>() as u64,
            )?;
            if last.occurrence == occurrence
                && last.role == role
                && last.source == *source
                && last.start == output.start
                && last.end == output.end
                && last.kind == kind
            {
                return Ok(());
            }
        }
        c.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<LexicalPart>() as u64,
        )?;
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        self.native.lexical.parts.reserve_geometric(c, 1)?;
        self.native.lexical.parts.try_push(
            c,
            LexicalPart {
                occurrence,
                role,
                source: source.clone(),
                start: output.start,
                end: output.end,
                kind,
            },
        )
    }
    pub(crate) fn lexical_finish(
        &mut self,
        c: &mut ParseControl,
        occurrence: &LexicalRef,
        source: &crate::HtmlSourceRange,
        kind: HtmlParseLexicalKind,
        disposition: HtmlParseLexicalDisposition,
    ) -> Result<(), HtmlParseStop> {
        let occurrence = self.require_lexical(c, occurrence)?;
        c.require_source_range(source)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let old = self
            .native
            .lexical
            .occurrences
            .as_slice()
            .get(occurrence)
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::RegistryRow))?;
        if old.source.source() != source.source()
            || old.source.start() != source.start()
            || old.source.end() > source.end()
            || old.disposition != HtmlParseLexicalDisposition::Draft
        {
            return Err(c.fail_invariant(HtmlParseInvariant::TokenizerState));
        }
        let cause_index = old.cause;
        // Acquire both actual mutable slots before any content update. A stop
        // during either admission leaves both retained draft records unchanged.
        let cause = self
            .native
            .causes
            .get_mut(cause_index, c)?
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::RegistryRow))?;
        let record = self
            .native
            .lexical
            .occurrences
            .get_mut(occurrence, c)?
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::RegistryRow))?;
        // Only a lexical cause records a classified range; any other cause here
        // is a registry defect, not a cause to relabel.
        let ConstructionCause::Lexical { source: classified } = cause else {
            return Err(c.fail_invariant(HtmlParseInvariant::RegistryRow));
        };
        c.charge(
            HtmlParseWorkCategory::CopyByte,
            const {
                (mem::size_of::<Option<crate::HtmlSourceRange>>()
                    + mem::size_of::<crate::HtmlSourceRange>()
                    + mem::size_of::<HtmlParseLexicalKind>()
                    + mem::size_of::<HtmlParseLexicalDisposition>()) as u64
            },
        )?;
        c.charge(HtmlParseWorkCategory::RecordWrite, 2)?;
        c.charge(HtmlParseWorkCategory::ProviderCall, 2)?;
        *classified = Some(source.clone());
        record.source = source.clone();
        record.kind = kind;
        record.disposition = disposition;
        Ok(())
    }
    /// The actual emitted handle follows token ownership and reprocessing.
    /// A source-free construction fixture has no lexical occurrence to classify.
    /// The real add-missing-attributes producer discards this exact field.
    pub(crate) fn lexical_attribute_ignored(
        &mut self,
        c: &mut ParseControl,
        reference: Option<&LexicalRef>,
    ) -> Result<(), HtmlParseStop> {
        // Generic source-unbound construction accepts controlled token fields,
        // but owns no complete lexical report. Only the source-bound parser
        // evidence owner can classify a handle issued by that actual lexer.
        if self.native.source.is_none() {
            return Ok(());
        }
        let Some(reference) = reference else {
            return Ok(());
        };
        let index = self.require_lexical(c, reference)?;
        let record = self.lexical_occurrence(c, index)?;
        if record.kind != HtmlParseLexicalKind::Attribute
            || record.disposition != HtmlParseLexicalDisposition::StoredAttribute
        {
            return Err(c.fail_invariant(HtmlParseInvariant::TokenizerState));
        }
        c.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
        c.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<HtmlParseLexicalDisposition>() as u64,
        )?;
        self.native
            .lexical
            .occurrences
            .get_mut(index, c)?
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::RegistryRow))?
            .disposition = HtmlParseLexicalDisposition::AlreadyPresentAttribute;
        Ok(())
    }
    pub(crate) fn lexical_disposition(
        &mut self,
        c: &mut ParseControl,
        reference: Option<&LexicalRef>,
        ignored: bool,
    ) -> Result<(), HtmlParseStop> {
        let Some(reference) = reference else {
            return Ok(());
        };
        let index = self.require_lexical(c, reference)?;
        let record = self.lexical_occurrence(c, index)?;
        if record.disposition == HtmlParseLexicalDisposition::Emitted {
            c.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
            c.charge(
                HtmlParseWorkCategory::CopyByte,
                mem::size_of::<HtmlParseLexicalDisposition>() as u64,
            )?;
            self.native
                .lexical
                .occurrences
                .get_mut(index, c)?
                .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::RegistryRow))?
                .disposition = if ignored {
                HtmlParseLexicalDisposition::Ignored
            } else {
                HtmlParseLexicalDisposition::Applied
            };
        }
        Ok(())
    }
}

impl ParseEvidence {
    pub(super) fn project_lexical(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        if let Some(point) = &self.native.lexical.end {
            c.require_source_range(point)?;
            payload_write::<NativeAssociation>(c, 1)?;
            c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            let row = NativeAssociation {
                group: None,
                relation: HtmlParseAssociationRelation::LexicalAnchor,
                destination: NativeDestination::LexicalEnd,
                start: 0,
                end: 0,
                source: Some(point.clone()),
                kind: None,
                cause: self.native.current.ok_or(SessionError::WrongToken)?,
            };
            self.native
                .projection
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .reserve_geometric(c, 1)?;
            self.native
                .projection
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .try_push(c, row)?;
        }
        for (index, occurrence) in self
            .native
            .lexical
            .occurrences
            .as_slice()
            .iter()
            .enumerate()
        {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if matches!(
                occurrence.disposition,
                HtmlParseLexicalDisposition::Draft | HtmlParseLexicalDisposition::Emitted
            ) {
                return Err(SessionError::WrongToken);
            }
            c.require_source_range(&occurrence.source)?;
            payload_write::<NativeAssociation>(c, 1)?;
            c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            let row = NativeAssociation {
                group: None,
                relation: HtmlParseAssociationRelation::LexicalEnvelope,
                destination: NativeDestination::Lexical {
                    occurrence: index,
                    part: None,
                },
                start: 0,
                end: 0,
                source: Some(occurrence.source.clone()),
                kind: None,
                cause: occurrence.cause,
            };
            self.native
                .projection
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .reserve_geometric(c, 1)?;
            self.native
                .projection
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .try_push(c, row)?;
        }
        for (index, part) in self.native.lexical.parts.as_slice().iter().enumerate() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            let occurrence = self
                .native
                .lexical
                .occurrences
                .as_slice()
                .get(part.occurrence)
                .ok_or(SessionError::WrongToken)?;
            if part.source.source() != occurrence.source.source()
                || part.source.start() < occurrence.source.start()
                || part.source.end() > occurrence.source.end()
            {
                return Err(SessionError::WrongToken);
            }
            c.require_source_range(&part.source)?;
            payload_write::<NativeAssociation>(c, 1)?;
            c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            let row = NativeAssociation {
                group: None,
                relation: HtmlParseAssociationRelation::LexicalPart,
                destination: NativeDestination::Lexical {
                    occurrence: part.occurrence,
                    part: Some(index),
                },
                start: part.start,
                end: part.end,
                source: Some(part.source.clone()),
                kind: part.kind,
                cause: occurrence.cause,
            };
            self.native
                .projection
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .reserve_geometric(c, 1)?;
            self.native
                .projection
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .try_push(c, row)?;
        }
        for (index, omission) in self.native.lexical.omissions.as_slice().iter().enumerate() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            c.require_source_range(&omission.source)?;
            payload_write::<NativeAssociation>(c, 1)?;
            c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            let row = NativeAssociation {
                group: None,
                relation: HtmlParseAssociationRelation::Omission,
                destination: NativeDestination::Omission(index),
                start: omission.start,
                end: omission.end,
                source: Some(omission.source.clone()),
                kind: Some(omission.origin),
                cause: omission.cause,
            };
            self.native
                .projection
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .reserve_geometric(c, 1)?;
            self.native
                .projection
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .try_push(c, row)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HtmlParseCancellation, HtmlParseResource, HtmlSource};

    #[test]
    fn lexical_eof_requires_exact_final_point_owner_and_nonterminal_single_write() {
        for case in 0..4 {
            let source = HtmlSource::utf8("x");
            let cancellation = HtmlParseCancellation::new();
            let mut c =
                ParseControl::new(crate::construction::tests::limits(), cancellation.clone());
            c.admit_source(&source).unwrap();
            let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
            evidence.initialize_native_source(&mut c, &source).unwrap();
            let point = source.range(1, 1).unwrap();
            if case == 0 {
                let mut foreign = ParseControl::new(
                    crate::construction::tests::limits(),
                    HtmlParseCancellation::new(),
                );
                foreign.admit_source(&source).unwrap();
                assert_eq!(
                    evidence.lexical_eof(&mut foreign, &point),
                    Err(HtmlParseStop::InternalInvariant(
                        HtmlParseInvariant::ForeignControl
                    ))
                );
                assert!(evidence.native.lexical.end.is_none());
                crate::parser::tree_builder::tests::assert_released(&foreign);
            } else {
                let expected = match case {
                    1 => HtmlParseStop::InternalInvariant(HtmlParseInvariant::TokenizerState),
                    2 => {
                        cancellation.cancel();
                        HtmlParseStop::Cancelled
                    }
                    3 => {
                        evidence.lexical_eof(&mut c, &point).unwrap();
                        HtmlParseStop::InternalInvariant(HtmlParseInvariant::TokenizerState)
                    }
                    _ => unreachable!(),
                };
                let candidate = if case == 1 {
                    source.range(0, 0).unwrap()
                } else {
                    point
                };
                assert_eq!(evidence.lexical_eof(&mut c, &candidate), Err(expected));
                if case == 3 {
                    assert_eq!(evidence.native.lexical.end.as_ref().unwrap().start(), 1);
                } else {
                    assert!(evidence.native.lexical.end.is_none());
                }
                assert_eq!(
                    evidence.lexical_eof(&mut c, &source.range(1, 1).unwrap()),
                    Err(expected)
                );
            }
            assert!(evidence.associations().is_err());
            evidence.release(&mut c).unwrap();
            crate::parser::tree_builder::tests::assert_released(&c);
        }
    }

    /// A handle of the right issuer that names an occurrence the evidence does
    /// not hold is a row that internal state names and cannot find; a part
    /// whose output range is reversed contradicts the sequencing of the
    /// tokenizer that records it. Neither is foreign evidence.
    #[test]
    fn lexical_row_and_sequencing_faults_are_not_foreign_evidence() {
        for reversed_range in [false, true] {
            let source = HtmlSource::utf8("<x>");
            let mut c = ParseControl::new(
                crate::construction::tests::limits(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
            let mut reference = evidence
                .lexical_begin(
                    &mut c,
                    &source.range(0, 1).unwrap(),
                    HtmlParseLexicalKind::Markup,
                    None,
                )
                .unwrap()
                .unwrap();
            let expected = if reversed_range {
                let reversed = std::ops::Range { start: 1, end: 0 };
                let stop = evidence
                    .lexical_part(
                        &mut c,
                        &reference,
                        HtmlParseLexicalRole::Syntax,
                        &source.range(0, 1).unwrap(),
                        reversed,
                        None,
                    )
                    .unwrap_err();
                (stop, HtmlParseInvariant::TokenizerState)
            } else {
                let valid = reference.index;
                reference.index = evidence.native.lexical.occurrences.len();
                let stop = evidence.lexical_data_offset(&mut c, &reference);
                reference.index = valid;
                (stop.unwrap_err(), HtmlParseInvariant::RegistryRow)
            };
            assert_eq!(expected.0, HtmlParseStop::InternalInvariant(expected.1));
            assert_eq!(c.stop(), Some(expected.0));
            drop(reference);
            evidence.release(&mut c).unwrap();
            crate::parser::tree_builder::tests::assert_released(&c);
        }
    }

    #[test]
    fn qualified_lexical_handles_reject_foreign_issuer_and_terminal_updates_without_prefix_mutation()
     {
        let source = HtmlSource::utf8("<x>");
        let mut c = ParseControl::new(
            crate::construction::tests::limits(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut first = ParseEvidence::prepare(&mut c).unwrap();
        let mut second = ParseEvidence::prepare(&mut c).unwrap();
        let reference = first
            .lexical_begin(
                &mut c,
                &source.range(0, 1).unwrap(),
                HtmlParseLexicalKind::Markup,
                None,
            )
            .unwrap()
            .unwrap();
        let stop = second
            .lexical_part(
                &mut c,
                &reference,
                HtmlParseLexicalRole::Syntax,
                &source.range(0, 1).unwrap(),
                0..0,
                None,
            )
            .unwrap_err();
        assert_eq!(
            stop,
            HtmlParseStop::InternalInvariant(HtmlParseInvariant::ForeignEvidence)
        );
        assert!(second.native.lexical.parts.as_slice().is_empty());
        assert_eq!(
            first.native.lexical.occurrences.as_slice()[0].disposition,
            HtmlParseLexicalDisposition::Draft
        );
        assert_eq!(
            first.lexical_finish(
                &mut c,
                &reference,
                &source.range(0, 3).unwrap(),
                HtmlParseLexicalKind::StartTag,
                HtmlParseLexicalDisposition::Emitted
            ),
            Err(stop)
        );
        drop(reference);
        first.release(&mut c).unwrap();
        second.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
    }

    #[test]
    fn lexical_part_actual_capacity_and_cancellation_preserve_prefix_and_cleanup_without_completion()
     {
        for cancelled in [false, true] {
            let source = HtmlSource::utf8("<x>");
            let cancellation = HtmlParseCancellation::new();
            // Occurrence + actual source cause fill the two source-map slots.
            let limits = crate::construction::tests::limits()
                .into_builder()
                .source_map_entries(2)
                .finish()
                .unwrap();
            let mut c = ParseControl::new(limits, cancellation.clone());
            c.admit_source(&source).unwrap();
            let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
            let reference = evidence
                .lexical_begin(
                    &mut c,
                    &source.range(0, 1).unwrap(),
                    HtmlParseLexicalKind::Markup,
                    None,
                )
                .unwrap()
                .unwrap();
            if cancelled {
                cancellation.cancel();
            }
            let stop = evidence
                .lexical_part(
                    &mut c,
                    &reference,
                    HtmlParseLexicalRole::Syntax,
                    &source.range(0, 1).unwrap(),
                    0..0,
                    None,
                )
                .unwrap_err();
            if cancelled {
                assert_eq!(stop, HtmlParseStop::Cancelled);
            } else {
                assert!(matches!(
                    stop,
                    HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::SourceMapEntries,
                        limit: 2,
                        current: 2,
                        requested_delta: 1
                    }
                ));
            }
            assert!(evidence.native.lexical.parts.as_slice().is_empty());
            assert_eq!(
                evidence.native.lexical.occurrences.as_slice()[0].disposition,
                HtmlParseLexicalDisposition::Draft
            );
            drop(reference);
            evidence.release(&mut c).unwrap();
            assert_eq!(c.stop(), Some(stop));
            crate::parser::tree_builder::tests::assert_released(&c);
        }
    }
}
