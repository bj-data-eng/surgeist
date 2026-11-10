//! Lexical parts are captured at actual state transitions, never reparsed.

use super::*;
use token::{HtmlParseTextOrigin, Text};

impl Tokenizer<'_> {
    pub(super) fn retain_lexical_text(
        evidence: &mut ParseEvidence,
        c: &mut ParseControl,
        occurrence: &LexicalRef,
        role: HtmlParseLexicalRole,
        origins: &[HtmlParseTextOrigin],
    ) -> Result<(), HtmlParseStop> {
        for origin in origins {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        role: HtmlParseLexicalRole,
        text: &Text,
    ) -> Result<(), HtmlParseStop> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
    ) -> Result<Option<LexicalRef>, HtmlParseStop> {
        let Some(occurrence) = self.lexical.as_ref() else {
            return Ok(None);
        };
        let (family, disposition) = match kind {
            TokenKind::Tag(tag) => {
                self.lexical_text(
                    evidence,
                    c,
                    occurrence,
                    HtmlParseLexicalRole::Name,
                    &tag.name,
                )?;
                (
                    if tag.kind == TagKind::Start {
                        HtmlParseLexicalKind::StartTag
                    } else {
                        HtmlParseLexicalKind::EndTag
                    },
                    HtmlParseLexicalDisposition::Emitted,
                )
            }
            TokenKind::Comment(text) => {
                self.lexical_text(evidence, c, occurrence, HtmlParseLexicalRole::Data, text)?;
                (
                    HtmlParseLexicalKind::Comment,
                    HtmlParseLexicalDisposition::Emitted,
                )
            }
            TokenKind::Doctype(doctype) => {
                for (text, role) in [
                    (&doctype.name, HtmlParseLexicalRole::Name),
                    (&doctype.public_id, HtmlParseLexicalRole::PublicIdentifier),
                    (&doctype.system_id, HtmlParseLexicalRole::SystemIdentifier),
                ] {
                    if let Some(text) = text {
                        self.lexical_text(evidence, c, occurrence, role, text)?;
                    }
                }
                (
                    HtmlParseLexicalKind::Doctype,
                    HtmlParseLexicalDisposition::Emitted,
                )
            }
            TokenKind::ProcessingInstruction { target, data } => {
                self.lexical_text(
                    evidence,
                    c,
                    occurrence,
                    HtmlParseLexicalRole::Target,
                    target,
                )?;
                self.lexical_text(evidence, c, occurrence, HtmlParseLexicalRole::Data, data)?;
                (
                    HtmlParseLexicalKind::ProcessingInstruction,
                    HtmlParseLexicalDisposition::Emitted,
                )
            }
            TokenKind::Text(text) => {
                // CDATA emits ordinary Text while its delimiter owner remains
                // open; only the actual closing transition ends that envelope.
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let offset = evidence.lexical_data_offset(c, occurrence)?;
                let end = offset
                    .checked_add(text.value.text().len())
                    .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
                for origin in text.origins.as_slice() {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    let start = offset
                        .checked_add(origin.start)
                        .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
                    let end = offset
                        .checked_add(origin.end)
                        .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
                    evidence.lexical_part(
                        c,
                        occurrence,
                        HtmlParseLexicalRole::Data,
                        &origin.source,
                        start..end,
                        Some(origin.kind),
                    )?;
                }
                evidence.lexical_data_commit(c, occurrence, offset, end)?;
                if source.start() != self.start {
                    return Ok(None);
                }
                (
                    HtmlParseLexicalKind::Markup,
                    HtmlParseLexicalDisposition::LiteralRecovery,
                )
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
    ) -> Result<(), HtmlParseStop> {
        let Some(occurrence) = self.lexical.as_ref() else {
            return Ok(());
        };
        let source = self.range(start, self.current.end(), c)?;
        evidence.lexical_part(
            c,
            occurrence,
            HtmlParseLexicalRole::Syntax,
            &source,
            0..0,
            None,
        )
    }
    pub(super) fn retain_lexical_syntax(
        &self,
        point: u32,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        use states::{HtmlTokenizerQuote, HtmlTokenizerState::*};
        let Some(markup) = self.lexical.as_ref() else {
            return Ok(());
        };
        let attribute = self
            .tag
            .as_ref()
            .and_then(|tag| tag.attribute.as_ref())
            .and_then(|attribute| attribute.lexical.as_ref());
        let field = attribute.unwrap_or(markup);
        let role = match self.state {
            TagOpen if matches!(point, 0x21 | 0x2f | 0x3f) => Some(HtmlParseLexicalRole::Syntax),
            EndTagOpen | TagName | BeforeAttributeName | AfterAttributeName
            | AfterAttributeValue
                if matches!(point, 0x2f | 0x3e) =>
            {
                Some(HtmlParseLexicalRole::Syntax)
            }
            AttributeName | AfterAttributeName if point == 0x3d => {
                Some(HtmlParseLexicalRole::Equals)
            }
            BeforeAttributeValue if point == 0x22 => Some(HtmlParseLexicalRole::OpeningQuote(
                HtmlTokenizerQuote::Double,
            )),
            BeforeAttributeValue if point == 0x27 => Some(HtmlParseLexicalRole::OpeningQuote(
                HtmlTokenizerQuote::Single,
            )),
            AttributeValue(HtmlTokenizerQuote::Double) if point == 0x22 => Some(
                HtmlParseLexicalRole::ClosingQuote(HtmlTokenizerQuote::Double),
            ),
            AttributeValue(HtmlTokenizerQuote::Single) if point == 0x27 => Some(
                HtmlParseLexicalRole::ClosingQuote(HtmlTokenizerQuote::Single),
            ),
            SelfClosing
            | BeforeAttributeValue
            | AttributeName
            | AttributeValue(HtmlTokenizerQuote::Unquoted)
                if point == 0x3e =>
            {
                Some(HtmlParseLexicalRole::Syntax)
            }
            TagName
            | BeforeAttributeName
            | AttributeName
            | AfterAttributeName
            | BeforeAttributeValue
            | AfterAttributeValue
            | AttributeValue(HtmlTokenizerQuote::Unquoted)
                if space(point) =>
            {
                Some(HtmlParseLexicalRole::Whitespace)
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
                Some(HtmlParseLexicalRole::Whitespace)
            }
            AfterDoctypeKeyword(_)
            | BeforeDoctypeIdentifier(_)
            | AfterDoctypeIdentifier(states::HtmlTokenizerIdentifier::Public)
            | BetweenDoctypeIdentifiers
                if point == 0x22 =>
            {
                Some(HtmlParseLexicalRole::OpeningQuote(
                    HtmlTokenizerQuote::Double,
                ))
            }
            AfterDoctypeKeyword(_)
            | BeforeDoctypeIdentifier(_)
            | AfterDoctypeIdentifier(states::HtmlTokenizerIdentifier::Public)
            | BetweenDoctypeIdentifiers
                if point == 0x27 =>
            {
                Some(HtmlParseLexicalRole::OpeningQuote(
                    HtmlTokenizerQuote::Single,
                ))
            }
            DoctypeIdentifier(_, HtmlTokenizerQuote::Double) if point == 0x22 => Some(
                HtmlParseLexicalRole::ClosingQuote(HtmlTokenizerQuote::Double),
            ),
            DoctypeIdentifier(_, HtmlTokenizerQuote::Single) if point == 0x27 => Some(
                HtmlParseLexicalRole::ClosingQuote(HtmlTokenizerQuote::Single),
            ),
            DoctypeName
            | AfterDoctypeName
            | AfterDoctypeKeyword(_)
            | BeforeDoctypeIdentifier(_)
            | AfterDoctypeIdentifier(_)
            | BetweenDoctypeIdentifiers
            | BogusDoctype
                if point == 0x3e =>
            {
                Some(HtmlParseLexicalRole::Syntax)
            }
            _ => None,
        };
        if let Some(role) = role {
            let occurrence = if matches!(
                role,
                HtmlParseLexicalRole::Equals
                    | HtmlParseLexicalRole::OpeningQuote(_)
                    | HtmlParseLexicalRole::ClosingQuote(_)
            ) {
                field
            } else {
                markup
            };
            evidence.lexical_part(c, occurrence, role, &self.current, 0..0, None)?;
            if matches!(
                role,
                HtmlParseLexicalRole::OpeningQuote(_) | HtmlParseLexicalRole::ClosingQuote(_)
            ) {
                let opening = matches!(role, HtmlParseLexicalRole::OpeningQuote(_));
                let position = if opening {
                    self.current.end()
                } else {
                    self.current.start()
                };
                let empty = self.range(position, position, c)?;
                let field_role = match self.state {
                    AfterDoctypeKeyword(states::HtmlTokenizerIdentifier::Public)
                    | BeforeDoctypeIdentifier(states::HtmlTokenizerIdentifier::Public)
                    | DoctypeIdentifier(states::HtmlTokenizerIdentifier::Public, _) => {
                        HtmlParseLexicalRole::PublicIdentifier
                    }
                    AfterDoctypeKeyword(states::HtmlTokenizerIdentifier::System)
                    | BeforeDoctypeIdentifier(states::HtmlTokenizerIdentifier::System)
                    | DoctypeIdentifier(states::HtmlTokenizerIdentifier::System, _)
                    | AfterDoctypeIdentifier(states::HtmlTokenizerIdentifier::Public)
                    | BetweenDoctypeIdentifiers => HtmlParseLexicalRole::SystemIdentifier,
                    _ => HtmlParseLexicalRole::Value,
                };
                let offset = if opening {
                    0
                } else {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    c.charge(
                        HtmlParseWorkCategory::InspectByte,
                        size_of::<usize>() as u64,
                    )?;
                    let text = match field_role {
                        HtmlParseLexicalRole::PublicIdentifier => self
                            .doctype
                            .as_ref()
                            .and_then(|doctype| doctype.public.as_ref()),
                        HtmlParseLexicalRole::SystemIdentifier => self
                            .doctype
                            .as_ref()
                            .and_then(|doctype| doctype.system.as_ref()),
                        _ => self
                            .tag
                            .as_ref()
                            .and_then(|tag| tag.attribute.as_ref())
                            .map(|attribute| &attribute.value),
                    }
                    .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
                    text.value.len()
                };
                evidence.lexical_part(c, occurrence, field_role, &empty, offset..offset, None)?;
                if !opening {
                    // Actual closing quote anchors the retained field's exact
                    // output end, including zero for an empty field.
                    evidence.lexical_part(
                        c,
                        occurrence,
                        HtmlParseLexicalRole::EndAnchor,
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
    ) -> Result<(), HtmlParseStop> {
        let Some(occurrence) = self.lexical.as_ref() else {
            return Ok(());
        };
        if let Some(tag) = &self.tag {
            Self::retain_lexical_text(
                evidence,
                c,
                occurrence,
                HtmlParseLexicalRole::Name,
                tag.name.origins.as_slice(),
            )?;
            if let Some(attribute) = &tag.attribute {
                // An open markup occurrence and its attributes are recorded
                // together, so an attribute of one has an occurrence too.
                let attribute_lexical = attribute
                    .lexical
                    .as_ref()
                    .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
                Self::retain_lexical_text(
                    evidence,
                    c,
                    attribute_lexical,
                    HtmlParseLexicalRole::Name,
                    attribute.name.origins.as_slice(),
                )?;
                Self::retain_lexical_text(
                    evidence,
                    c,
                    attribute_lexical,
                    HtmlParseLexicalRole::Value,
                    attribute.value.origins.as_slice(),
                )?;
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                c.charge(
                    HtmlParseWorkCategory::InspectByte,
                    size_of::<usize>() as u64,
                )?;
                let end = attribute.value.value.len();
                evidence.lexical_part(
                    c,
                    attribute_lexical,
                    HtmlParseLexicalRole::Value,
                    &self.current,
                    end..end,
                    None,
                )?;
                evidence.lexical_part(
                    c,
                    attribute_lexical,
                    HtmlParseLexicalRole::EndAnchor,
                    &self.current,
                    end..end,
                    None,
                )?;
                let source = self.range(attribute.start, self.current.end(), c)?;
                evidence.lexical_finish(
                    c,
                    attribute_lexical,
                    &source,
                    HtmlParseLexicalKind::Attribute,
                    HtmlParseLexicalDisposition::Abandoned,
                )?;
            }
        }
        if matches!(
            self.state,
            states::HtmlTokenizerState::PiOpen | states::HtmlTokenizerState::PiTarget
        ) {
            Self::retain_lexical_text(
                evidence,
                c,
                occurrence,
                HtmlParseLexicalRole::Target,
                self.temporary.origins.as_slice(),
            )?;
        }
        for (text, role) in [
            (&self.pi_target, HtmlParseLexicalRole::Target),
            (&self.pi_data, HtmlParseLexicalRole::Data),
        ] {
            if let Some(text) = text {
                Self::retain_lexical_text(evidence, c, occurrence, role, text.origins.as_slice())?;
            }
        }
        let source = self.range(self.start, self.current.end(), c)?;
        evidence.lexical_part(
            c,
            occurrence,
            HtmlParseLexicalRole::EndAnchor,
            &self.current,
            0..0,
            None,
        )?;
        let family = if let Some(tag) = &self.tag {
            if tag.kind == TagKind::Start {
                HtmlParseLexicalKind::StartTag
            } else {
                HtmlParseLexicalKind::EndTag
            }
        } else {
            match self.state {
                states::HtmlTokenizerState::PiOpen
                | states::HtmlTokenizerState::PiTarget
                | states::HtmlTokenizerState::AfterPiTarget
                | states::HtmlTokenizerState::PiData
                | states::HtmlTokenizerState::PiQuestionable => {
                    HtmlParseLexicalKind::ProcessingInstruction
                }
                states::HtmlTokenizerState::Cdata
                | states::HtmlTokenizerState::CdataBracket
                | states::HtmlTokenizerState::CdataEnd => HtmlParseLexicalKind::Cdata,
                _ => HtmlParseLexicalKind::Markup,
            }
        };
        evidence.lexical_finish(
            c,
            occurrence,
            &source,
            family,
            HtmlParseLexicalDisposition::Abandoned,
        )?;
        admit_record_write(size_of::<Option<LexicalRef>>(), 1, c)?;
        self.lexical = None;
        Ok(())
    }
}
