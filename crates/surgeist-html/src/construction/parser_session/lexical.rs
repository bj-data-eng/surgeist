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
pub enum LexicalKind {
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
pub enum LexicalDisposition {
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
pub enum LexicalRole {
    Name,
    Value,
    PublicIdentifier,
    SystemIdentifier,
    Target,
    Data,
    Equals,
    OpeningQuote(crate::parser::tokenizer::states::Quote),
    ClosingQuote(crate::parser::tokenizer::states::Quote),
    Syntax,
    Whitespace,
    EndAnchor,
}
pub(crate) struct LexicalOccurrence {
    pub(crate) kind: LexicalKind,
    pub(crate) disposition: LexicalDisposition,
    pub(crate) source: crate::HtmlSourceRange,
    pub(crate) parent: Option<usize>,
    cause: usize,
    data_units: usize,
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
    pub(crate) role: LexicalRole,
    pub(crate) source: crate::HtmlSourceRange,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) kind: Option<OriginKind>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmissionKind {
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
    pub(crate) origin: OriginKind,
    pub(crate) kind: OmissionKind,
    pub(crate) rule: ParserRule,
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
    pub(super) fn new(c: &mut ParseControl) -> Result<Self, ParseStop> {
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
    pub(super) fn extents(&self, c: &mut ParseControl) -> Result<[(u64, u64); 3], ParseStop> {
        self.occurrences.require_disposal_owner(c)?;
        self.parts.require_disposal_owner(c)?;
        self.omissions.require_disposal_owner(c)?;
        fn bytes<T>(values: &OwnedVec<T>, c: &mut ParseControl) -> Result<u64, ParseStop> {
            values
                .capacity()
                .checked_mul(mem::size_of::<T>())
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or_else(|| c.fail_invariant(Invariant::ReleaseUnderflow))
        }
        Ok([
            (self.occurrences.len() as u64, bytes(&self.occurrences, c)?),
            (self.parts.len() as u64, bytes(&self.parts, c)?),
            (self.omissions.len() as u64, bytes(&self.omissions, c)?),
        ])
    }
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.occurrences.release(c)?;
        self.parts.release(c)?;
        self.omissions.release(c)
    }
}
impl ParseEvidence {
    /// Only the tokenizer's actual sealed-and-exhausted EOF transition supplies
    /// this checked point. A partial frontier is never lexical completion.
    pub(crate) fn lexical_eof(
        &mut self,
        c: &mut ParseControl,
        point: &crate::HtmlSourceRange,
    ) -> Result<(), ParseStop> {
        self.native.lexical.occurrences.require_owner(c)?;
        c.require_source_range(point)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if point.start() != point.end()
            || self.native.lexical.end.is_some()
            || self
                .native
                .source
                .as_ref()
                .is_some_and(|source| point.end() != source.len())
        {
            return Err(c.fail_invariant(Invariant::ForeignEvidence));
        }
        c.charge(
            WorkCategory::CopyByte,
            mem::size_of::<Option<crate::HtmlSourceRange>>() as u64,
        )?;
        c.charge(WorkCategory::RecordWrite, 1)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
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
        origins: &[TextOrigin],
        kind: OmissionKind,
        rule: ParserRule,
    ) -> Result<(), SessionError> {
        let cause = self.field_cause(c)?;
        for origin in origins {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.require_source_range(&origin.source)?;
            payload_write::<LexicalOmission>(c, 1)?;
            c.charge(WorkCategory::ProviderCall, 1)?;
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
        kind: LexicalKind,
        parent: Option<&LexicalRef>,
    ) -> Result<LexicalRef, ParseStop> {
        c.require_source_range(source)?;
        let parent = parent
            .map(|parent| self.require_lexical(c, parent))
            .transpose()?;
        let cause = self
            .append_cause(
                c,
                ConstructionCause {
                    operation: ConstructionOperation::Lexical,
                    source: None,
                    captured: None,
                    rule: None,
                    family: None,
                    original_cause: None,
                    retired_destination: None,
                    original_node: None,
                    intended_parent: None,
                    original: None,
                    incoming: None,
                    trigger: None,
                    computation: None,
                },
            )
            .map_err(|error| match error {
                SessionError::Stop(stop) => stop,
                _ => c.fail_invariant(Invariant::ForeignEvidence),
            })?;
        let index = self.native.lexical.occurrences.len();
        payload_write::<LexicalOccurrence>(c, 1).map_err(|error| match error {
            SessionError::Stop(stop) => stop,
            _ => c.fail_invariant(Invariant::ForeignEvidence),
        })?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        self.native.lexical.occurrences.reserve_geometric(c, 1)?;
        self.native.lexical.occurrences.try_push(
            c,
            LexicalOccurrence {
                kind,
                disposition: LexicalDisposition::Draft,
                source: source.clone(),
                parent,
                cause,
                data_units: 0,
            },
        )?;
        c.charge(WorkCategory::CopyByte, mem::size_of::<LexicalRef>() as u64)?;
        c.charge(WorkCategory::RecordWrite, 1)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        Ok(LexicalRef {
            index,
            issuer: Arc::downgrade(&self.identity),
            control: c.identity(),
        })
    }
    fn require_lexical(
        &self,
        c: &mut ParseControl,
        reference: &LexicalRef,
    ) -> Result<usize, ParseStop> {
        c.require_identity(&reference.control)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(
            WorkCategory::InspectByte,
            mem::size_of::<Weak<evidence::EvidenceIdentity>>() as u64,
        )?;
        if reference.issuer.as_ptr() != Arc::as_ptr(&self.identity)
            || self
                .native
                .lexical
                .occurrences
                .as_slice()
                .get(reference.index)
                .is_none()
        {
            return Err(c.fail_invariant(Invariant::ForeignEvidence));
        }
        Ok(reference.index)
    }
    pub(crate) fn lexical_data_offset(
        &self,
        c: &mut ParseControl,
        reference: &LexicalRef,
    ) -> Result<usize, ParseStop> {
        let index = self.require_lexical(c, reference)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(WorkCategory::InspectByte, mem::size_of::<usize>() as u64)?;
        Ok(self.native.lexical.occurrences.as_slice()[index].data_units)
    }
    pub(crate) fn lexical_data_commit(
        &mut self,
        c: &mut ParseControl,
        reference: &LexicalRef,
        old: usize,
        new: usize,
    ) -> Result<(), ParseStop> {
        let index = self.require_lexical(c, reference)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let occurrence = &self.native.lexical.occurrences.as_slice()[index];
        if occurrence.disposition != LexicalDisposition::Draft
            || occurrence.data_units != old
            || new < old
        {
            return Err(c.fail_invariant(Invariant::ForeignEvidence));
        }
        c.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
        c.charge(WorkCategory::RecordWrite, 1)?;
        self.native
            .lexical
            .occurrences
            .get_mut(index, c)?
            .expect("qualified data occurrence")
            .data_units = new;
        Ok(())
    }
    pub(crate) fn lexical_part(
        &mut self,
        c: &mut ParseControl,
        occurrence: &LexicalRef,
        role: LexicalRole,
        source: &crate::HtmlSourceRange,
        output: std::ops::Range<usize>,
        kind: Option<OriginKind>,
    ) -> Result<(), ParseStop> {
        let occurrence = self.require_lexical(c, occurrence)?;
        c.require_source_range(source)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let record = self
            .native
            .lexical
            .occurrences
            .as_slice()
            .get(occurrence)
            .ok_or_else(|| c.fail_invariant(Invariant::ForeignEvidence))?;
        if record.disposition != LexicalDisposition::Draft
            || source.source() != record.source.source()
            || source.coordinates() != record.source.coordinates()
            || output.start > output.end
        {
            return Err(c.fail_invariant(Invariant::ForeignEvidence));
        }
        if let Some(last) = self.native.lexical.parts.as_slice().last() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(
                WorkCategory::InspectByte,
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
        c.charge(WorkCategory::CopyByte, mem::size_of::<LexicalPart>() as u64)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
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
        kind: LexicalKind,
        disposition: LexicalDisposition,
    ) -> Result<(), ParseStop> {
        let occurrence = self.require_lexical(c, occurrence)?;
        c.require_source_range(source)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let old = self
            .native
            .lexical
            .occurrences
            .as_slice()
            .get(occurrence)
            .ok_or_else(|| c.fail_invariant(Invariant::ForeignEvidence))?;
        if old.source.source() != source.source()
            || old.source.start() != source.start()
            || old.source.end() > source.end()
            || old.disposition != LexicalDisposition::Draft
        {
            return Err(c.fail_invariant(Invariant::ForeignEvidence));
        }
        let cause_index = old.cause;
        // Acquire both actual mutable slots before any content update. A stop
        // during either admission leaves both retained draft records unchanged.
        let cause = self
            .native
            .causes
            .get_mut(cause_index, c)?
            .ok_or_else(|| c.fail_invariant(Invariant::ForeignEvidence))?;
        let record = self
            .native
            .lexical
            .occurrences
            .get_mut(occurrence, c)?
            .ok_or_else(|| c.fail_invariant(Invariant::ForeignEvidence))?;
        c.charge(
            WorkCategory::CopyByte,
            (mem::size_of::<Option<crate::HtmlSourceRange>>()
                + mem::size_of::<crate::HtmlSourceRange>()
                + mem::size_of::<LexicalKind>()
                + mem::size_of::<LexicalDisposition>()) as u64,
        )?;
        c.charge(WorkCategory::RecordWrite, 2)?;
        c.charge(WorkCategory::ProviderCall, 2)?;
        cause.source = Some(source.clone());
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
    ) -> Result<(), ParseStop> {
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
        let record = &self.native.lexical.occurrences.as_slice()[index];
        if record.kind != LexicalKind::Attribute
            || record.disposition != LexicalDisposition::StoredAttribute
        {
            return Err(c.fail_invariant(Invariant::ForeignEvidence));
        }
        c.charge(WorkCategory::RecordWrite, 1)?;
        c.charge(
            WorkCategory::CopyByte,
            mem::size_of::<LexicalDisposition>() as u64,
        )?;
        self.native
            .lexical
            .occurrences
            .get_mut(index, c)?
            .expect("qualified attribute")
            .disposition = LexicalDisposition::AlreadyPresentAttribute;
        Ok(())
    }
    pub(crate) fn lexical_disposition(
        &mut self,
        c: &mut ParseControl,
        reference: Option<&LexicalRef>,
        ignored: bool,
    ) -> Result<(), ParseStop> {
        let Some(reference) = reference else {
            return Ok(());
        };
        let index = self.require_lexical(c, reference)?;
        let record = &self.native.lexical.occurrences.as_slice()[index];
        if record.disposition == LexicalDisposition::Emitted {
            c.charge(WorkCategory::RecordWrite, 1)?;
            c.charge(
                WorkCategory::CopyByte,
                mem::size_of::<LexicalDisposition>() as u64,
            )?;
            self.native
                .lexical
                .occurrences
                .get_mut(index, c)?
                .expect("qualified occurrence")
                .disposition = if ignored {
                LexicalDisposition::Ignored
            } else {
                LexicalDisposition::Applied
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
            c.charge(WorkCategory::ProviderCall, 1)?;
            let row = NativeAssociation {
                group: None,
                relation: AssociationRelation::LexicalAnchor,
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
        for index in 0..self.native.lexical.occurrences.len() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let occurrence = &self.native.lexical.occurrences.as_slice()[index];
            if matches!(
                occurrence.disposition,
                LexicalDisposition::Draft | LexicalDisposition::Emitted
            ) {
                return Err(SessionError::WrongToken);
            }
            c.require_source_range(&occurrence.source)?;
            payload_write::<NativeAssociation>(c, 1)?;
            c.charge(WorkCategory::ProviderCall, 1)?;
            let row = NativeAssociation {
                group: None,
                relation: AssociationRelation::LexicalEnvelope,
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
        for index in 0..self.native.lexical.parts.len() {
            c.charge(WorkCategory::RecordVisit, 2)?;
            let part = &self.native.lexical.parts.as_slice()[index];
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
            c.charge(WorkCategory::ProviderCall, 1)?;
            let row = NativeAssociation {
                group: None,
                relation: AssociationRelation::LexicalPart,
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
        for index in 0..self.native.lexical.omissions.len() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let omission = &self.native.lexical.omissions.as_slice()[index];
            c.require_source_range(&omission.source)?;
            payload_write::<NativeAssociation>(c, 1)?;
            c.charge(WorkCategory::ProviderCall, 1)?;
            let row = NativeAssociation {
                group: None,
                relation: AssociationRelation::Omission,
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
            let mut c = ParseControl::new(
                crate::construction::parser_session::tests::limits(),
                cancellation.clone(),
            );
            c.admit_source(&source).unwrap();
            let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
            evidence.initialize_native_source(&mut c, &source).unwrap();
            let point = source.range(1, 1).unwrap();
            if case == 0 {
                let mut foreign = ParseControl::new(
                    crate::construction::parser_session::tests::limits(),
                    HtmlParseCancellation::new(),
                );
                foreign.admit_source(&source).unwrap();
                assert_eq!(
                    evidence.lexical_eof(&mut foreign, &point),
                    Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
                );
                assert!(evidence.native.lexical.end.is_none());
                crate::parser::tree_builder::tests::assert_released(&foreign);
            } else {
                let expected = match case {
                    1 => ParseStop::InternalInvariant(Invariant::ForeignEvidence),
                    2 => {
                        cancellation.cancel();
                        ParseStop::Cancelled
                    }
                    3 => {
                        evidence.lexical_eof(&mut c, &point).unwrap();
                        ParseStop::InternalInvariant(Invariant::ForeignEvidence)
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

    #[test]
    fn qualified_lexical_handles_reject_foreign_issuer_and_terminal_updates_without_prefix_mutation()
     {
        let source = HtmlSource::utf8("<x>");
        let mut c = ParseControl::new(
            crate::construction::parser_session::tests::limits(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut first = ParseEvidence::prepare(&mut c).unwrap();
        let mut second = ParseEvidence::prepare(&mut c).unwrap();
        let reference = first
            .lexical_begin(
                &mut c,
                &source.range(0, 1).unwrap(),
                LexicalKind::Markup,
                None,
            )
            .unwrap();
        let stop = second
            .lexical_part(
                &mut c,
                &reference,
                LexicalRole::Syntax,
                &source.range(0, 1).unwrap(),
                0..0,
                None,
            )
            .unwrap_err();
        assert_eq!(
            stop,
            ParseStop::InternalInvariant(Invariant::ForeignEvidence)
        );
        assert!(second.native.lexical.parts.as_slice().is_empty());
        assert_eq!(
            first.native.lexical.occurrences.as_slice()[0].disposition,
            LexicalDisposition::Draft
        );
        assert_eq!(
            first.lexical_finish(
                &mut c,
                &reference,
                &source.range(0, 3).unwrap(),
                LexicalKind::StartTag,
                LexicalDisposition::Emitted
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
            let limits = crate::construction::parser_session::tests::limits()
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
                    LexicalKind::Markup,
                    None,
                )
                .unwrap();
            if cancelled {
                cancellation.cancel();
            }
            let stop = evidence
                .lexical_part(
                    &mut c,
                    &reference,
                    LexicalRole::Syntax,
                    &source.range(0, 1).unwrap(),
                    0..0,
                    None,
                )
                .unwrap_err();
            if cancelled {
                assert_eq!(stop, ParseStop::Cancelled);
            } else {
                assert!(matches!(
                    stop,
                    ParseStop::LimitExceeded {
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
                LexicalDisposition::Draft
            );
            drop(reference);
            evidence.release(&mut c).unwrap();
            assert_eq!(c.stop(), Some(stop));
            crate::parser::tree_builder::tests::assert_released(&c);
        }
    }
}
