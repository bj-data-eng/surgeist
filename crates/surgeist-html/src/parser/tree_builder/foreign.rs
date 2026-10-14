// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from tree_builder/mod.rs 1605–1692, 1837–1883 and rules.rs
// 1608–1688 at d7232d746d3112f08926d169591576bd12fe2fdf.
// HTML877d controls PI and annotation-xml integration/breakout differences.

use super::{
    KernelError,
    head::{HeadCondition, HeadError, HeadState, HeadStep},
    prologue::Payload,
    tag_sets, write,
};
use crate::{
    DocumentString,
    bounded::{
        self, HtmlIntegrationPointComparison, HtmlIntegrationPointRule, HtmlMathmlTextPointTest,
    },
    construction::parser_session::{
        BodyTextKind, BodyTextRuns, ForeignNamespace, ParserNode, partition_body_text,
    },
    parser::{
        control::{ParseControl, StandaloneUnsupported, WorkCategory},
        tokenizer::token::{TagKind, Text},
    },
};
use std::mem::size_of;

const HTML: &str = "http://www.w3.org/1999/xhtml";
const MATHML: &str = "http://www.w3.org/1998/Math/MathML";
const SVG: &str = "http://www.w3.org/2000/svg";

pub(super) fn insensitive(
    value: &DocumentString,
    literal: &'static str,
    c: &mut ParseControl,
) -> Result<bool, HeadError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    if value.len() != literal.len() {
        return Ok(false);
    }
    for (index, byte) in literal.bytes().enumerate() {
        c.charge(WorkCategory::InspectByte, 3)?;
        let unit = value.code_units()[index];
        let lower = if (65..=90).contains(&unit) {
            unit + 32
        } else {
            unit
        };
        if lower != u16::from(byte) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn integration_comparison(
    value: &DocumentString,
    literal: &'static str,
    comparison: HtmlIntegrationPointComparison,
    c: &mut ParseControl,
) -> Result<bool, HeadError> {
    match comparison {
        HtmlIntegrationPointComparison::Exact => Ok(tag_sets::ascii_eq(value, literal, c)?),
        HtmlIntegrationPointComparison::AsciiCaseInsensitive => insensitive(value, literal, c),
    }
}

fn integration_rule(
    name: &crate::QualifiedName,
    c: &mut ParseControl,
) -> Result<HtmlIntegrationPointRule, HeadError> {
    // Borrow the namespace, its URI and the local name before comparisons.
    c.charge(WorkCategory::RecordVisit, 3)?;
    bounded::html_integration_point_rule(
        name.namespace().uri(),
        name.local_name(),
        |value, literal, comparison| integration_comparison(value, literal, comparison, c),
    )
}

fn integration_encoding(
    attributes: &crate::OrderedAttributes,
    c: &mut ParseControl,
) -> Result<bool, HeadError> {
    for attribute in attributes.as_slice() {
        // Visit the attribute and borrow its name, namespace, URI, local name
        // and value. Variable UTF-16 inspections remain comparison-owned.
        c.charge(WorkCategory::RecordVisit, 6)?;
        let name = attribute.name();
        if let Some(result) = bounded::html_integration_point_encoding_attribute(
            name.namespace().uri(),
            name.local_name(),
            attribute.value(),
            |value, literal, comparison| integration_comparison(value, literal, comparison, c),
        )? {
            return Ok(result);
        }
    }
    Ok(false)
}

fn mathml_text_point(name: &crate::QualifiedName, c: &mut ParseControl) -> Result<bool, HeadError> {
    Ok(bounded::mathml_text_integration_point(|test| match test {
        HtmlMathmlTextPointTest::Namespace(expected) => tag_sets::namespace(name, expected, c),
        HtmlMathmlTextPointTest::Local(choices) => tag_sets::local_in(name, choices, c),
    })?)
}

impl HeadState<'_> {
    pub(super) fn adjusted_current_name<'a>(
        &'a self,
        c: &mut ParseControl,
    ) -> Result<&'a crate::QualifiedName, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.before.kernel.stack()?.len() == 1
            && let Some(context) = self.before.kernel.session.fragment_context(c)?
        {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let node = context
                .capture()
                .node(context.context_element().subject())
                .map_err(|_| {
                    crate::construction::parser_session::SessionError::ContextUnavailable
                })?;
            let crate::HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(KernelError::NotElement.into());
            };
            return Ok(element.name());
        }
        let current = self.before.kernel.current(c)?;
        self.before
            .kernel
            .session
            .element_name(c, &current)?
            .ok_or_else(|| KernelError::NotElement.into())
    }
    fn adjusted_current_attributes<'a>(
        &'a self,
        c: &mut ParseControl,
    ) -> Result<&'a crate::OrderedAttributes, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.before.kernel.stack()?.len() == 1
            && let Some(context) = self.before.kernel.session.fragment_context(c)?
        {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let node = context
                .capture()
                .node(context.context_element().subject())
                .map_err(|_| {
                    crate::construction::parser_session::SessionError::ContextUnavailable
                })?;
            let crate::HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(KernelError::NotElement.into());
            };
            return Ok(element.attributes());
        }
        let current = self.before.kernel.current(c)?;
        self.before
            .kernel
            .session
            .element_attributes(c, &current)?
            .ok_or_else(|| KernelError::NotElement.into())
    }
    pub(super) fn foreign_namespace(
        &self,
        c: &mut ParseControl,
    ) -> Result<Option<ForeignNamespace>, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.before.kernel.stack()?.len() == 0 {
            return Ok(None);
        }
        let name = self.adjusted_current_name(c)?;
        if tag_sets::namespace(name, HTML, c)? {
            return Ok(None);
        }
        if tag_sets::namespace(name, MATHML, c)? {
            return Ok(Some(ForeignNamespace::MathMl));
        }
        if tag_sets::namespace(name, SVG, c)? {
            return Ok(Some(ForeignNamespace::Svg));
        }
        let context = self
            .before
            .kernel
            .session
            .fragment_context(c)?
            .ok_or_else(|| c.stop_unsupported_standalone(StandaloneUnsupported::Provenance))?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let node = context
            .capture()
            .node(context.context_element().subject())
            .map_err(|_| super::SessionError::ContextUnavailable)?;
        let crate::HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Err(KernelError::NotElement.into());
        };
        let equal = match (name.namespace().uri(), element.name().namespace().uri()) {
            (None, None) => true,
            (Some(left), Some(right)) => tag_sets::equal_units(left, right, c)?,
            _ => false,
        };
        if !equal {
            return Err(c
                .stop_unsupported_standalone(StandaloneUnsupported::Provenance)
                .into());
        }
        Ok(Some(ForeignNamespace::Context))
    }
    fn math_text_point(&self, node: &ParserNode, c: &mut ParseControl) -> Result<bool, HeadError> {
        let name = self
            .before
            .kernel
            .session
            .element_name(c, node)?
            .ok_or(KernelError::EmptyStack)?;
        mathml_text_point(name, c)
    }
    fn integration_point(
        &self,
        node: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<bool, HeadError> {
        let name = self
            .before
            .kernel
            .session
            .element_name(c, node)?
            .ok_or(KernelError::EmptyStack)?;
        match integration_rule(name, c)? {
            HtmlIntegrationPointRule::Never => Ok(false),
            HtmlIntegrationPointRule::Always => Ok(true),
            HtmlIntegrationPointRule::EncodingAttribute => {
                let attrs = self
                    .before
                    .kernel
                    .session
                    .element_attributes(c, node)?
                    .ok_or(KernelError::EmptyStack)?;
                integration_encoding(attrs, c)
            }
        }
    }
    pub(super) fn foreign_dispatch(&self, c: &mut ParseControl) -> Result<bool, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let payload = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload;
        if matches!(payload, Payload::Eof) || self.foreign_namespace(c)?.is_none() {
            return Ok(false);
        }
        let kind = self.kind(c)?;
        let text = matches!(payload, Payload::Text(_));
        let name = self.adjusted_current_name(c)?;
        if mathml_text_point(name, c)?
            && (text
                || (matches!(kind, Some((TagKind::Start, _)))
                    && self.token_in(&["mglyph", "malignmark"], c)?.is_none()))
        {
            return Ok(false);
        }
        if tag_sets::namespace(name, MATHML, c)?
            && tag_sets::local_in(name, &["annotation-xml"], c)?
            && matches!(kind, Some((TagKind::Start, _)))
            && self.token_in(&["svg"], c)?.is_some()
        {
            return Ok(false);
        }
        if self.adjusted_integration_point(c)?
            && (text || matches!(kind, Some((TagKind::Start, _))))
        {
            return Ok(false);
        }
        Ok(true)
    }
    fn adjusted_integration_point(&self, c: &mut ParseControl) -> Result<bool, HeadError> {
        let name = self.adjusted_current_name(c)?;
        match integration_rule(name, c)? {
            HtmlIntegrationPointRule::Never => Ok(false),
            HtmlIntegrationPointRule::Always => Ok(true),
            HtmlIntegrationPointRule::EncodingAttribute => {
                integration_encoding(self.adjusted_current_attributes(c)?, c)
            }
        }
    }
    pub(super) fn insert_foreign(
        &mut self,
        namespace: ForeignNamespace,
        close: bool,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        self.admit_hidden_capability(c)?;
        let svg_script =
            namespace == ForeignNamespace::Svg && self.token_in(&["script"], c)?.is_some();
        let place = self.before.kernel.appropriate_place(None, c)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let pending = self.pending.as_mut().ok_or(HeadError::MissingInput)?;
        let Payload::Tag(input) = &mut pending.payload else {
            return Err(HeadError::MissingInput);
        };
        let capability = &self
            .script
            .as_ref()
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?
            .capability;
        let node = self
            .before
            .kernel
            .session
            .allocate_foreign_tag_for_creation(
                c,
                input,
                &place.parent,
                namespace,
                &pending.source,
                capability,
            )?;
        self.foreign_xmlns_errors(&node, namespace, c)?;
        let location = self.before.kernel.adjusted_location(None, c)?;
        self.before.kernel.insert_element_at(
            &node,
            &location,
            c,
            self.script.as_ref().map(|s| &s.capability),
        )?;
        self.before.kernel.push(&node, c)?;
        if close && svg_script {
            return self.svg_end(&node, c);
        }
        if close {
            self.before
                .kernel
                .pop(c, self.script.as_ref().map(|s| &s.capability))?
                .ok_or(KernelError::EmptyStack)?;
        }
        self.finish(c)
    }
    fn foreign_xmlns_errors(
        &mut self,
        node: &ParserNode,
        namespace: ForeignNamespace,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        // At most the two source-defined attributes can produce these errors.
        // Scan separately so each diagnostic precedes the next actual read.
        for (local, expected) in [
            ("xmlns", namespace.uri()),
            ("xlink", Some("http://www.w3.org/1999/xlink")),
        ] {
            let mut mismatch = false;
            let attrs = self
                .before
                .kernel
                .session
                .element_attributes(c, node)?
                .ok_or(KernelError::EmptyStack)?;
            for attribute in attrs.as_slice() {
                c.charge(WorkCategory::RecordVisit, 2)?;
                if tag_sets::namespace(attribute.name(), "http://www.w3.org/2000/xmlns/", c)?
                    && tag_sets::ascii_eq(attribute.name().local_name(), local, c)?
                    && !(if let Some(expected) = expected {
                        tag_sets::ascii_eq(attribute.value(), expected, c)?
                    } else {
                        let name = self
                            .before
                            .kernel
                            .session
                            .element_name(c, node)?
                            .ok_or(KernelError::NotElement)?;
                        match name.namespace().uri() {
                            Some(uri) => tag_sets::equal_units(attribute.value(), uri, c)?,
                            None => false,
                        }
                    })
                {
                    mismatch = true;
                    break;
                }
            }
            if mismatch {
                self.emit(
                    HeadCondition::ForeignNamespaceMismatch,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inforeign",
                        branch: "foreign-start/xmlns-namespace-mismatch",
                    },
                    c,
                )?;
            }
        }
        Ok(())
    }
    fn foreign_characters(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.body_runs.is_none() {
            let Payload::Text(input) = &mut self
                .pending
                .as_mut()
                .ok_or(HeadError::MissingInput)?
                .payload
            else {
                return Err(HeadError::MissingInput);
            };
            write(size_of::<Option<BodyTextRuns>>(), 1, c)?;
            self.body_runs = Some(partition_body_text(c, input)?);
        }
        loop {
            c.charge(WorkCategory::StateStep, 1)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            if self.body_run.is_none() {
                write(
                    size_of::<Option<crate::construction::parser_session::TextRun>>(),
                    1,
                    c,
                )?;
                self.body_run = self
                    .body_runs
                    .as_mut()
                    .expect("retained runs")
                    .take_next(c)?;
            }
            c.charge(WorkCategory::RecordVisit, 1)?;
            let Some(run) = self.body_run.as_ref() else {
                break;
            };
            c.charge(WorkCategory::InspectByte, size_of::<BodyTextKind>() as u64)?;
            let kind = run.kind;
            if kind == BodyTextKind::Null {
                let mode = self.mode(c)?;
                for origin in self
                    .body_run
                    .as_ref()
                    .expect("retained run")
                    .text
                    .origins
                    .as_slice()
                {
                    c.charge(WorkCategory::RecordVisit, 1)?;

                    c.charge(WorkCategory::ProviderCall, 1)?;
                    self.before.kernel.session.evidence(c)?.append(c, crate::construction::parser_session::evidence::DiagnosticKind::Tree(HeadCondition::NullInForeign), crate::construction::parser_session::evidence::DiagnosticContext::Tree(mode), crate::construction::parser_session::evidence::DiagnosticRule { anchor: "parsing-main-inforeign", branch: "null-character/replace" }, crate::construction::parser_session::evidence::DiagnosticLocationInput { source: &origin.source, role: crate::construction::parser_session::evidence::DiagnosticLocationRole::TextOrigin, token: Some(crate::construction::parser_session::evidence::DiagnosticToken::Text) })?;
                }
            }
            write(
                size_of::<Option<crate::construction::parser_session::TextRun>>()
                    + size_of::<Option<Text>>(),
                2,
                c,
            )?;
            self.whitespace_pending = self.body_run.take().map(|run| run.text);
            if kind == BodyTextKind::Null {
                self.before
                    .kernel
                    .session
                    .replace_foreign_nulls(c, &mut self.whitespace_pending)?;
            }
            self.before.kernel.insert_text(
                &mut self.whitespace_pending,
                None,
                c,
                self.script.as_ref().map(|s| &s.capability),
            )?;
            if kind == BodyTextKind::Other {
                self.frameset(false, c)?;
            }
        }
        self.body_runs.as_mut().expect("retained runs").release(c)?;
        write(size_of::<Option<BodyTextRuns>>(), 1, c)?;
        self.body_runs = None;
        self.finish(c)
    }
    /// None delegates/reprocesses the SAME token in the current HTML mode;
    /// no new token or dispatcher override is fabricated.
    pub(super) fn foreign_rules(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<Option<HeadStep>, HeadError> {
        self.record_trigger(
            crate::construction::parser_session::provenance::ParserRule::Foreign,
            c,
        )?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        match &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        {
            Payload::Text(_) => return Ok(Some(self.foreign_characters(c)?)),
            Payload::Comment(_) | Payload::Pi(_) => return Ok(Some(self.ordinary_node(c)?)),
            Payload::Doctype(_) => {
                self.emit(
                    HeadCondition::UnexpectedDoctype,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inforeign",
                        branch: "foreign-doctype/ignore",
                    },
                    c,
                )?;
                return Ok(Some(self.discard(c)?));
            }
            Payload::Eof => return Ok(None),
            Payload::Tag(_) => {}
        }
        let (kind, close) = self.kind(c)?.ok_or(HeadError::MissingInput)?;
        let breakout = if kind == TagKind::Start {
            self.token_in(
                &[
                    "b",
                    "big",
                    "blockquote",
                    "body",
                    "br",
                    "center",
                    "code",
                    "dd",
                    "div",
                    "dl",
                    "dt",
                    "em",
                    "embed",
                    "h1",
                    "h2",
                    "h3",
                    "h4",
                    "h5",
                    "h6",
                    "head",
                    "hr",
                    "i",
                    "img",
                    "li",
                    "listing",
                    "menu",
                    "meta",
                    "nobr",
                    "ol",
                    "p",
                    "pre",
                    "ruby",
                    "s",
                    "small",
                    "span",
                    "strong",
                    "strike",
                    "sub",
                    "sup",
                    "table",
                    "tt",
                    "u",
                    "ul",
                    "var",
                ],
                c,
            )?
            .is_some()
                || (self.token_in(&["font"], c)?.is_some() && self.foreign_font_breakout(c)?)
        } else {
            self.token_in(&["br", "p"], c)?.is_some()
        };
        if breakout {
            self.emit(
                HeadCondition::UnexpectedForeignToken,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inforeign",
                    branch: if kind == TagKind::Start {
                        "foreign-start/html-breakout"
                    } else {
                        "foreign-end/html-breakout"
                    },
                },
                c,
            )?;
            loop {
                c.charge(WorkCategory::StateStep, 1)?;
                let current = self.before.kernel.current(c)?;
                let current_name = self
                    .before
                    .kernel
                    .session
                    .element_name(c, &current)?
                    .ok_or(KernelError::NotElement)?;
                // The breakout loop tests the actual current node, not the
                // adjusted context substituted by overarching dispatch.
                if tag_sets::namespace(current_name, HTML, c)?
                    || self.math_text_point(&current, c)?
                    || self.integration_point(&current, c)?
                {
                    break;
                }
                self.before
                    .kernel
                    .pop(c, self.script.as_ref().map(|s| &s.capability))?
                    .ok_or(KernelError::EmptyStack)?;
            }
            return Ok(None);
        }
        if kind == TagKind::Start {
            let namespace = self.foreign_namespace(c)?.ok_or(KernelError::EmptyStack)?;
            return Ok(Some(self.insert_foreign(namespace, close, c)?));
        }
        let current = self.before.kernel.current(c)?;
        if self.foreign_namespace(c)? == Some(ForeignNamespace::Svg)
            && self.token_in(&["script"], c)?.is_some()
        {
            let name = self
                .before
                .kernel
                .session
                .element_name(c, &current)?
                .ok_or(KernelError::EmptyStack)?;
            if tag_sets::local_in(name, &["script"], c)? {
                return Ok(Some(self.svg_end(&current, c)?));
            }
        }
        if !self.foreign_end_matches(&current, c)? {
            self.emit(
                HeadCondition::UnexpectedForeignToken,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inforeign",
                    branch: "foreign-end/current-local-name-mismatch",
                },
                c,
            )?;
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        let mut index = self.before.kernel.stack()?.len() - 1;
        loop {
            c.charge(WorkCategory::StateStep, 1)?;
            if index == 0 {
                return Ok(Some(self.discard(c)?));
            }
            let node = self.before.kernel.at(index, c)?;
            if self.foreign_end_matches(&node, c)? {
                loop {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if self.before.kernel.stack()?.len() <= index {
                        break;
                    }
                    self.before
                        .kernel
                        .pop(c, self.script.as_ref().map(|s| &s.capability))?
                        .ok_or(KernelError::EmptyStack)?;
                }
                return Ok(Some(self.discard(c)?));
            }
            index -= 1;
            let node = self.before.kernel.at(index, c)?;
            let name = self
                .before
                .kernel
                .session
                .element_name(c, &node)?
                .ok_or(KernelError::EmptyStack)?;
            if tag_sets::namespace(name, HTML, c)? {
                return Ok(None);
            }
        }
    }
    fn foreign_font_breakout(&self, c: &mut ParseControl) -> Result<bool, HeadError> {
        let Payload::Tag(input) = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Ok(false);
        };
        for attribute in input
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .attributes
            .as_slice()
        {
            c.charge(WorkCategory::RecordVisit, 1)?;
            for name in ["color", "face", "size"] {
                if tag_sets::ascii_eq(attribute.name.value.text(), name, c)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    fn foreign_end_matches(
        &self,
        node: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<bool, HeadError> {
        let name = self
            .before
            .kernel
            .session
            .element_name(c, node)?
            .ok_or(KernelError::EmptyStack)?
            .local_name();
        let Payload::Tag(input) = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Err(HeadError::MissingInput);
        };
        let token = input
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .name
            .value
            .text();
        c.charge(WorkCategory::RecordVisit, 2)?;
        if name.len() != token.len() {
            return Ok(false);
        }
        for index in 0..name.len() {
            c.charge(WorkCategory::InspectByte, 4)?;
            let unit = name.code_units()[index];
            let lower = if (65..=90).contains(&unit) {
                unit + 32
            } else {
                unit
            };
            if lower != token.code_units()[index] {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Attribute, HtmlParseCancellation, HtmlParseResource, Namespace, OrderedAttributes,
        QualifiedName,
    };

    fn encoding(value: &str) -> Attribute {
        Attribute::new(
            QualifiedName::new(Namespace::none(), Some("p".into()), "encoding"),
            value,
        )
    }

    fn control(work: u64, cancellation: HtmlParseCancellation) -> ParseControl {
        ParseControl::new(
            super::super::tests::limits(100)
                .into_builder()
                .work_credits(work)
                .finish()
                .unwrap(),
            cancellation,
        )
    }

    #[test]
    fn mathml_text_group_retains_exact_header_unit_and_refusal_costs() {
        let name = QualifiedName::new(Namespace::mathml(), Some("p".into()), "mtext");
        // Namespace: two field visits and one comparison header, 34 pairs of
        // UTF-16/literal bytes. Local group: one header, five choice/comparison
        // headers and five final pairs. All categories have unit weight.
        let mut exact = control(131, HtmlParseCancellation::new());
        assert!(mathml_text_point(&name, &mut exact).unwrap());
        assert_eq!(exact.usage().work(WorkCategory::RecordVisit), 14);
        assert_eq!(exact.usage().work(WorkCategory::InspectByte), 117);
        assert_eq!(exact.stop(), None);

        let cancellation = HtmlParseCancellation::new();
        let mut short = control(130, cancellation.clone());
        assert!(mathml_text_point(&name, &mut short).is_err());
        let first = short.stop();
        assert!(matches!(
            first,
            Some(crate::HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })
        ));
        let usage = short.usage().clone();
        cancellation.cancel();
        assert!(mathml_text_point(&name, &mut short).is_err());
        assert_eq!(short.stop(), first);
        assert_eq!(short.usage(), &usage);

        let mut foreign = control(3, HtmlParseCancellation::new());
        assert!(
            !mathml_text_point(
                &QualifiedName::new(Namespace::html(), None, "mtext"),
                &mut foreign,
            )
            .unwrap()
        );
        assert_eq!(foreign.usage().work(WorkCategory::RecordVisit), 3);
        assert_eq!(foreign.usage().work(WorkCategory::InspectByte), 0);

        let cancellation = HtmlParseCancellation::new();
        let mut cancelled = control(131, cancellation.clone());
        cancellation.cancel();
        assert!(mathml_text_point(&name, &mut cancelled).is_err());
        assert_eq!(cancelled.stop(), Some(crate::HtmlParseStop::Cancelled));
        assert_eq!(cancelled.usage().work(WorkCategory::RecordVisit), 0);
    }

    #[test]
    fn unique_encoding_value_stops_before_unrelated_attribute_records() {
        for (value, expected) in [("text/html", true), ("text/xml", false)] {
            let attributes = OrderedAttributes::new(std::iter::once(encoding(value)).chain(
                (0..1000).map(|index| {
                    Attribute::new(
                        QualifiedName::new(Namespace::none(), None, format!("other-{index}")),
                        "irrelevant",
                    )
                }),
            ))
            .unwrap();
            // Inspecting all 1,001 attribute records cannot fit this budget.
            let mut control = control(1000, HtmlParseCancellation::new());
            assert_eq!(
                integration_encoding(&attributes, &mut control).unwrap(),
                expected
            );
            assert_eq!(control.stop(), None);
        }
    }

    #[test]
    fn encoding_comparison_exact_and_one_short_preserve_first_stop_and_cancellation() {
        let attributes = OrderedAttributes::new([encoding("ApPlIcAtIoN/XhTmL+XmL")]).unwrap();
        let mut measured = control(1000, HtmlParseCancellation::new());
        assert!(integration_encoding(&attributes, &mut measured).unwrap());
        let required = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let mut exact = control(required, HtmlParseCancellation::new());
        assert!(integration_encoding(&attributes, &mut exact).unwrap());
        assert_eq!(exact.stop(), None);

        let cancellation = HtmlParseCancellation::new();
        let mut short = control(required - 1, cancellation.clone());
        assert!(integration_encoding(&attributes, &mut short).is_err());
        let first = short.stop();
        assert!(matches!(
            first,
            Some(crate::HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })
        ));
        let work = short
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        cancellation.cancel();
        assert!(integration_encoding(&attributes, &mut short).is_err());
        assert_eq!(short.stop(), first);
        assert_eq!(
            short
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            work
        );

        let cancellation = HtmlParseCancellation::new();
        let mut cancelled = control(required, cancellation.clone());
        cancellation.cancel();
        assert!(integration_encoding(&attributes, &mut cancelled).is_err());
        assert_eq!(cancelled.stop(), Some(crate::HtmlParseStop::Cancelled));
        assert_eq!(
            cancelled
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            0
        );
    }
}
