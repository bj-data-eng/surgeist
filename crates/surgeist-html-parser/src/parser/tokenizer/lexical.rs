//! Lexical parts are captured at actual state transitions, never reparsed.

use super::*;
use token::{Text, TextOrigin};

impl Tokenizer<'_> {
    pub(super) fn retain_lexical_text(
        evidence: &mut ParseEvidence,
        c: &mut ParseControl,
        occurrence: &LexicalRef,
        role: LexicalRole,
        origins: &[TextOrigin],
    ) -> Result<(), ParseStop> {
        for origin in origins {
            c.charge(WorkCategory::RecordVisit, 1)?;
            evidence.lexical_part(
                c,
                occurrence,
                role,
                &origin.source,
                origin.start..origin.end,
                Some(origin.kind),
            )?;
        }
        Ok(())
    }
    fn lexical_text(
        &self,
        evidence: &mut ParseEvidence,
        c: &mut ParseControl,
        occurrence: &LexicalRef,
        role: LexicalRole,
        text: &Text,
    ) -> Result<(), ParseStop> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if text.value.text().is_empty() {
            let anchor = self.range(self.current.start(), self.current.start(), c)?;
            evidence.lexical_part(c, occurrence, role, &anchor, 0..0, None)?;
        }
        Self::retain_lexical_text(evidence, c, occurrence, role, text.origins.as_slice())
    }
    pub(super) fn retain_emitted_lexical(
        &mut self,
        kind: &TokenKind,
        source: &HtmlSourceRange,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<Option<LexicalRef>, ParseStop> {
        let Some(occurrence) = self.lexical.as_ref() else {
            return Ok(None);
        };
        let (family, disposition) = match kind {
            TokenKind::Tag(tag) => {
                self.lexical_text(evidence, c, occurrence, LexicalRole::Name, &tag.name)?;
                (
                    if tag.kind == TagKind::Start {
                        LexicalKind::StartTag
                    } else {
                        LexicalKind::EndTag
                    },
                    LexicalDisposition::Emitted,
                )
            }
            TokenKind::Comment(text) => {
                self.lexical_text(evidence, c, occurrence, LexicalRole::Data, text)?;
                (LexicalKind::Comment, LexicalDisposition::Emitted)
            }
            TokenKind::Doctype(doctype) => {
                for (text, role) in [
                    (&doctype.name, LexicalRole::Name),
                    (&doctype.public_id, LexicalRole::PublicIdentifier),
                    (&doctype.system_id, LexicalRole::SystemIdentifier),
                ] {
                    if let Some(text) = text {
                        self.lexical_text(evidence, c, occurrence, role, text)?;
                    }
                }
                (LexicalKind::Doctype, LexicalDisposition::Emitted)
            }
            TokenKind::ProcessingInstruction { target, data } => {
                self.lexical_text(evidence, c, occurrence, LexicalRole::Target, target)?;
                self.lexical_text(evidence, c, occurrence, LexicalRole::Data, data)?;
                (
                    LexicalKind::ProcessingInstruction,
                    LexicalDisposition::Emitted,
                )
            }
            TokenKind::Text(text) => {
                // CDATA emits ordinary Text while its delimiter owner remains
                // open; only the actual closing transition ends that envelope.
                c.charge(WorkCategory::RecordVisit, 1)?;
                let offset = evidence.lexical_data_offset(c, occurrence)?;
                let end = offset
                    .checked_add(text.value.text().len())
                    .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
                for origin in text.origins.as_slice() {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let start = offset
                        .checked_add(origin.start)
                        .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
                    let end = offset
                        .checked_add(origin.end)
                        .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
                    evidence.lexical_part(
                        c,
                        occurrence,
                        LexicalRole::Data,
                        &origin.source,
                        start..end,
                        Some(origin.kind),
                    )?;
                }
                evidence.lexical_data_commit(c, occurrence, offset, end)?;
                if source.start() != self.start {
                    return Ok(None);
                }
                (LexicalKind::Markup, LexicalDisposition::LiteralRecovery)
            }
            TokenKind::Eof => return Ok(None),
        };
        evidence.lexical_finish(c, occurrence, source, family, disposition)?;
        admit_record_write(size_of::<Option<LexicalRef>>(), 1, c)?;
        Ok(self.lexical.take())
    }
    // Lookbehind punctuation is still potential data until the owning state
    // proves a close. Use its actual retained start and consumed endpoint;
    // neither fallback nor EOF calls this producer.
    pub(super) fn retain_lexical_close(
        &self,
        start: usize,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        let Some(occurrence) = self.lexical.as_ref() else {
            return Ok(());
        };
        let source = self.range(start, self.current.end(), c)?;
        evidence.lexical_part(c, occurrence, LexicalRole::Syntax, &source, 0..0, None)
    }
    pub(super) fn retain_lexical_syntax(
        &self,
        point: u32,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        use states::{Quote, State::*};
        let Some(markup) = self.lexical.as_ref() else {
            return Ok(());
        };
        let attribute = self
            .tag
            .as_ref()
            .and_then(|tag| tag.attribute.as_ref())
            .map(|attribute| &attribute.lexical);
        let field = attribute.unwrap_or(markup);
        let role = match self.state {
            TagOpen if matches!(point, 0x21 | 0x2f | 0x3f) => Some(LexicalRole::Syntax),
            EndTagOpen | TagName | BeforeAttributeName | AfterAttributeName
            | AfterAttributeValue
                if matches!(point, 0x2f | 0x3e) =>
            {
                Some(LexicalRole::Syntax)
            }
            AttributeName | AfterAttributeName if point == 0x3d => Some(LexicalRole::Equals),
            BeforeAttributeValue if point == 0x22 => Some(LexicalRole::OpeningQuote(Quote::Double)),
            BeforeAttributeValue if point == 0x27 => Some(LexicalRole::OpeningQuote(Quote::Single)),
            AttributeValue(Quote::Double) if point == 0x22 => {
                Some(LexicalRole::ClosingQuote(Quote::Double))
            }
            AttributeValue(Quote::Single) if point == 0x27 => {
                Some(LexicalRole::ClosingQuote(Quote::Single))
            }
            SelfClosing
            | BeforeAttributeValue
            | AttributeName
            | AttributeValue(Quote::Unquoted)
                if point == 0x3e =>
            {
                Some(LexicalRole::Syntax)
            }
            TagName
            | BeforeAttributeName
            | AttributeName
            | AfterAttributeName
            | BeforeAttributeValue
            | AfterAttributeValue
            | AttributeValue(Quote::Unquoted)
                if space(point) =>
            {
                Some(LexicalRole::Whitespace)
            }
            Doctype
            | BeforeDoctypeName
            | AfterDoctypeName
            | AfterDoctypeKeyword(_)
            | BeforeDoctypeIdentifier(_)
            | AfterDoctypeIdentifier(_)
            | BetweenDoctypeIdentifiers
            | AfterPiTarget
                if space(point) =>
            {
                Some(LexicalRole::Whitespace)
            }
            AfterDoctypeKeyword(_)
            | BeforeDoctypeIdentifier(_)
            | AfterDoctypeIdentifier(states::Identifier::Public)
            | BetweenDoctypeIdentifiers
                if point == 0x22 =>
            {
                Some(LexicalRole::OpeningQuote(Quote::Double))
            }
            AfterDoctypeKeyword(_)
            | BeforeDoctypeIdentifier(_)
            | AfterDoctypeIdentifier(states::Identifier::Public)
            | BetweenDoctypeIdentifiers
                if point == 0x27 =>
            {
                Some(LexicalRole::OpeningQuote(Quote::Single))
            }
            DoctypeIdentifier(_, Quote::Double) if point == 0x22 => {
                Some(LexicalRole::ClosingQuote(Quote::Double))
            }
            DoctypeIdentifier(_, Quote::Single) if point == 0x27 => {
                Some(LexicalRole::ClosingQuote(Quote::Single))
            }
            DoctypeName
            | AfterDoctypeName
            | AfterDoctypeKeyword(_)
            | BeforeDoctypeIdentifier(_)
            | AfterDoctypeIdentifier(_)
            | BetweenDoctypeIdentifiers
            | BogusDoctype
                if point == 0x3e =>
            {
                Some(LexicalRole::Syntax)
            }
            _ => None,
        };
        if let Some(role) = role {
            let occurrence = if matches!(
                role,
                LexicalRole::Equals | LexicalRole::OpeningQuote(_) | LexicalRole::ClosingQuote(_)
            ) {
                field
            } else {
                markup
            };
            evidence.lexical_part(c, occurrence, role, &self.current, 0..0, None)?;
            if matches!(
                role,
                LexicalRole::OpeningQuote(_) | LexicalRole::ClosingQuote(_)
            ) {
                let opening = matches!(role, LexicalRole::OpeningQuote(_));
                let position = if opening {
                    self.current.end()
                } else {
                    self.current.start()
                };
                let empty = self.range(position, position, c)?;
                let field_role = match self.state {
                    AfterDoctypeKeyword(states::Identifier::Public)
                    | BeforeDoctypeIdentifier(states::Identifier::Public)
                    | DoctypeIdentifier(states::Identifier::Public, _) => {
                        LexicalRole::PublicIdentifier
                    }
                    AfterDoctypeKeyword(states::Identifier::System)
                    | BeforeDoctypeIdentifier(states::Identifier::System)
                    | DoctypeIdentifier(states::Identifier::System, _)
                    | AfterDoctypeIdentifier(states::Identifier::Public)
                    | BetweenDoctypeIdentifiers => LexicalRole::SystemIdentifier,
                    _ => LexicalRole::Value,
                };
                let offset = if opening {
                    0
                } else {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    c.charge(WorkCategory::InspectByte, size_of::<usize>() as u64)?;
                    let text = match field_role {
                        LexicalRole::PublicIdentifier => self
                            .doctype
                            .as_ref()
                            .and_then(|doctype| doctype.public.as_ref()),
                        LexicalRole::SystemIdentifier => self
                            .doctype
                            .as_ref()
                            .and_then(|doctype| doctype.system.as_ref()),
                        _ => self
                            .tag
                            .as_ref()
                            .and_then(|tag| tag.attribute.as_ref())
                            .map(|attribute| &attribute.value),
                    }
                    .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
                    text.value.len()
                };
                evidence.lexical_part(c, occurrence, field_role, &empty, offset..offset, None)?;
                if !opening {
                    // Actual closing quote anchors the retained field's exact
                    // output end, including zero for a genuinely empty field.
                    evidence.lexical_part(
                        c,
                        occurrence,
                        LexicalRole::EndAnchor,
                        &empty,
                        offset..offset,
                        None,
                    )?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn abandon_lexical(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        let Some(occurrence) = self.lexical.as_ref() else {
            return Ok(());
        };
        if let Some(tag) = &self.tag {
            Self::retain_lexical_text(
                evidence,
                c,
                occurrence,
                LexicalRole::Name,
                tag.name.origins.as_slice(),
            )?;
            if let Some(attribute) = &tag.attribute {
                Self::retain_lexical_text(
                    evidence,
                    c,
                    &attribute.lexical,
                    LexicalRole::Name,
                    attribute.name.origins.as_slice(),
                )?;
                Self::retain_lexical_text(
                    evidence,
                    c,
                    &attribute.lexical,
                    LexicalRole::Value,
                    attribute.value.origins.as_slice(),
                )?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                c.charge(WorkCategory::InspectByte, size_of::<usize>() as u64)?;
                let end = attribute.value.value.len();
                evidence.lexical_part(
                    c,
                    &attribute.lexical,
                    LexicalRole::Value,
                    &self.current,
                    end..end,
                    None,
                )?;
                evidence.lexical_part(
                    c,
                    &attribute.lexical,
                    LexicalRole::EndAnchor,
                    &self.current,
                    end..end,
                    None,
                )?;
                let source = self.range(attribute.start, self.current.end(), c)?;
                evidence.lexical_finish(
                    c,
                    &attribute.lexical,
                    &source,
                    LexicalKind::Attribute,
                    LexicalDisposition::Abandoned,
                )?;
            }
        }
        if matches!(self.state, states::State::PiOpen | states::State::PiTarget) {
            Self::retain_lexical_text(
                evidence,
                c,
                occurrence,
                LexicalRole::Target,
                self.temporary.origins.as_slice(),
            )?;
        }
        for (text, role) in [
            (&self.pi_target, LexicalRole::Target),
            (&self.pi_data, LexicalRole::Data),
        ] {
            if let Some(text) = text {
                Self::retain_lexical_text(evidence, c, occurrence, role, text.origins.as_slice())?;
            }
        }
        let source = self.range(self.start, self.current.end(), c)?;
        evidence.lexical_part(
            c,
            occurrence,
            LexicalRole::EndAnchor,
            &self.current,
            0..0,
            None,
        )?;
        let family = if let Some(tag) = &self.tag {
            if tag.kind == TagKind::Start {
                LexicalKind::StartTag
            } else {
                LexicalKind::EndTag
            }
        } else {
            match self.state {
                states::State::PiOpen
                | states::State::PiTarget
                | states::State::AfterPiTarget
                | states::State::PiData
                | states::State::PiQuestionable => LexicalKind::ProcessingInstruction,
                states::State::Cdata | states::State::CdataBracket | states::State::CdataEnd => {
                    LexicalKind::Cdata
                }
                _ => LexicalKind::Markup,
            }
        };
        evidence.lexical_finish(
            c,
            occurrence,
            &source,
            family,
            LexicalDisposition::Abandoned,
        )?;
        admit_record_write(size_of::<Option<LexicalRef>>(), 1, c)?;
        self.lexical = None;
        Ok(())
    }
}
