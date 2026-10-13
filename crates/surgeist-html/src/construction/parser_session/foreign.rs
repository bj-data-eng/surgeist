// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from html5ever/src/tree_builder/mod.rs 1694–1841 at
// d7232d746d3112f08926d169591576bd12fe2fdf. HTML877d governs the tables.
// Owned construction transfer uses Surgeist's existing arena and registry.

use super::*;

const XLINK: &str = "http://www.w3.org/1999/xlink";
const XML: &str = "http://www.w3.org/XML/1998/namespace";
const XMLNS: &str = "http://www.w3.org/2000/xmlns/";

fn lookup(
    c: &mut ParseControl,
    name: &DocumentString,
    table: &'static [(&str, &str)],
) -> Result<Option<(&'static str, usize)>, SessionError> {
    for &(original, adjusted) in table {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if exact_ascii(c, Some(name), original)? {
            return Ok(Some((adjusted, 0)));
        }
    }
    Ok(None)
}

/// Owns replacement backing and maps while the original lexical backing/maps
/// remain available to the registry. Prefix removal rebases output positions;
/// original ranges are copied verbatim rather than manufacturing a source union.
fn adjusted_text(
    c: &mut ParseControl,
    original: &Text,
    replacement: &str,
    skip: usize,
) -> Result<Text, SessionError> {
    let mut value = Utf16Buffer::new(c)?;
    let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
    let result = (|| {
        for byte in replacement.bytes() {
            c.charge(WorkCategory::InspectByte, 1)?;
            value.push_unit(u16::from(byte), c)?;
        }
        for origin in original.origins.as_slice() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(
                WorkCategory::InspectByte,
                (2 * mem::size_of::<usize>()) as u64,
            )?;
            if origin.end <= skip {
                continue;
            }
            c.charge(WorkCategory::ProviderCall, 1)?;
            payload_write::<TextOrigin>(c, 1)?;
            origins.try_push(
                c,
                TextOrigin {
                    start: origin.start.saturating_sub(skip),
                    end: origin.end - skip,
                    source: origin.source.clone(),
                    kind: crate::parser::tokenizer::token::OriginKind::RecoverySyntax,
                },
            )?;
        }
        payload_write::<Text>(c, 1)
    })();
    if let Err(error) = result {
        value.release(c)?;
        origins.release(c)?;
        return Err(error);
    }
    match value.freeze(c) {
        Ok(value) => Ok(Text { value, origins }),
        Err(stop) => {
            origins.release(c)?;
            Err(stop.into())
        }
    }
}

impl ParserArenaSession {
    /// Only actual parser lexical payloads enter foreign creation. Qualification
    /// precedes any borrowed-token transfer; HTML form/reset metadata is absent.
    pub(crate) fn allocate_foreign_tag_for_creation(
        &mut self,
        c: &mut ParseControl,
        input: &mut Option<Tag>,
        intended_parent: &ParserNode,
        namespace: ForeignNamespace,
        token_source: &crate::HtmlSourceRange,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<ParserNode, SessionError> {
        capability.validate_session(c, self)?;
        let storage = self.active()?;
        let parent = storage.index(c, intended_parent)?;
        storage.container(c, parent)?;
        if storage.associated_document_parent(c, parent)? {
            return Err(SessionError::AssociationMove);
        }
        c.require_source_range(token_source)?;
        let tag = input.as_mut().ok_or(SessionError::Consumed)?;
        SessionStorage::check_html_tag(c, tag)?;
        check_formatting_tag(c, tag, token_source)?;
        if namespace == ForeignNamespace::Context {
            c.charge(WorkCategory::RecordVisit, 1)?;
            storage
                .fragment
                .as_ref()
                .ok_or(SessionError::ContextUnavailable)?;
        }
        let document = storage.node_document_ref(c, parent)?;
        let mut prepared =
            self.active_mut()?
                .prepare_document_allocation(c, Some(document), false)?;
        let mut is_value = None;
        if self.active()?.fragment.is_some() {
            for attribute in tag.attributes.as_slice() {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if exact_ascii(c, Some(attribute.name.value.text()), "is")? {
                    is_value = Some(&attribute.value);
                    break;
                }
            }
        }
        self.active_mut()?.prepare_creation_registry(
            c,
            parent,
            tag.name.value.text(),
            is_value,
            false,
            &mut prepared,
        )?;
        let svg = if namespace == ForeignNamespace::Svg
            && exact_ascii(c, Some(tag.name.value.text()), "script")?
        {
            Some(
                self.active_mut()?
                    .retain_svg_slot(c, SvgScriptState::fresh())?,
            )
        } else {
            None
        };
        let tag = take_payload(input, c)?;
        self.active_mut()?.allocate_foreign_payload(
            c,
            tag.name,
            namespace,
            tag.attributes,
            prepared,
            svg,
        )
    }
    pub(crate) fn element_attributes<'a>(
        &'a self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<&'a OrderedAttributes>, SessionError> {
        let storage = self.active()?;
        let index = storage.index(c, node)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        Ok(match &storage.records[index].kind {
            HtmlNodeKind::Element(value) => Some(value.attributes()),
            _ => None,
        })
    }
    pub(crate) fn replace_foreign_nulls(
        &mut self,
        c: &mut ParseControl,
        input: &mut Option<Text>,
    ) -> Result<(), SessionError> {
        self.active()?.require(c)?;
        let original = input.as_mut().ok_or(SessionError::Consumed)?;
        SessionStorage::check_text(c, original)?;
        check_text_source(c, original)?;
        let mut value = Utf16Buffer::new(c)?;
        let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        let result = (|| {
            for &unit in original.value.text().code_units() {
                c.charge(WorkCategory::InspectByte, 2)?;
                if unit != 0 {
                    return Err(SessionError::WrongToken);
                }
                value.push_unit(0xfffd, c)?;
            }
            for origin in original.origins.as_slice() {
                c.charge(WorkCategory::RecordVisit, 1)?;
                c.charge(WorkCategory::ProviderCall, 1)?;
                payload_write::<TextOrigin>(c, 1)?;
                origins.try_push(
                    c,
                    TextOrigin {
                        start: origin.start,
                        end: origin.end,
                        source: origin.source.clone(),
                        kind: crate::parser::tokenizer::token::OriginKind::NullReplacement,
                    },
                )?;
            }
            payload_write::<Option<Text>>(c, 2)
        })();
        if let Err(error) = result {
            value.release(c)?;
            origins.release(c)?;
            return Err(error);
        }
        let value = match value.freeze(c) {
            Ok(value) => value,
            Err(stop) => {
                origins.release(c)?;
                return Err(stop.into());
            }
        };
        let original = input
            .replace(Text { value, origins })
            .expect("qualified original");
        original.release(c)?;
        Ok(())
    }
}

impl SessionStorage {
    fn register_foreign_text(
        &mut self,
        c: &mut ParseControl,
        original: Text,
        replacement: Option<(&str, usize)>,
    ) -> Result<(DocumentString, usize), SessionError> {
        let Some((replacement, skip)) = replacement else {
            return self.register_text(c, original);
        };
        let adjusted = match adjusted_text(c, &original, replacement, skip) {
            Ok(value) => value,
            Err(error) => {
                original.release(c)?;
                return Err(error);
            }
        };
        let original = match self.register_text(c, original) {
            Ok((value, slot)) => {
                drop(value);
                match &self.payloads.values[slot] {
                    RetainedPayload::String {
                        origins: Some(group),
                        ..
                    } => match self
                        .evidence
                        .as_ref()
                        .ok_or(SessionError::WrongToken)
                        .and_then(|evidence| evidence.copy_group_reference(c, group))
                    {
                        Ok(original) => Some(original),
                        Err(error) => {
                            adjusted.release(c)?;
                            return Err(error);
                        }
                    },
                    _ => None,
                }
            }
            Err(error) => {
                adjusted.release(c)?;
                return Err(error);
            }
        };
        self.register_field_copy(
            c,
            provenance::FieldCopy {
                value: adjusted.value,
                origins: adjusted.origins,
                original,
            },
        )
    }
    fn foreign_attribute(
        &mut self,
        c: &mut ParseControl,
        original: Text,
        foreign: ForeignNamespace,
    ) -> Result<(QualifiedName, usize, Option<usize>, Option<usize>), SessionError> {
        let classified = (|| {
            let adjustment = match foreign {
                ForeignNamespace::Svg => lookup(c, original.value.text(), SVG_ATTRIBUTES)?,
                ForeignNamespace::Context => None,
                ForeignNamespace::MathMl => {
                    if exact_ascii(c, Some(original.value.text()), "definitionurl")? {
                        Some(("definitionURL", 0))
                    } else {
                        None
                    }
                }
            };
            for &(input, prefix, local, namespace) in &[
                ("xlink:actuate", Some("xlink"), "actuate", XLINK),
                ("xlink:arcrole", Some("xlink"), "arcrole", XLINK),
                ("xlink:href", Some("xlink"), "href", XLINK),
                ("xlink:role", Some("xlink"), "role", XLINK),
                ("xlink:show", Some("xlink"), "show", XLINK),
                ("xlink:title", Some("xlink"), "title", XLINK),
                ("xlink:type", Some("xlink"), "type", XLINK),
                ("xml:lang", Some("xml"), "lang", XML),
                ("xml:space", Some("xml"), "space", XML),
                ("xmlns", None, "xmlns", XMLNS),
                ("xmlns:xlink", Some("xmlns"), "xlink", XMLNS),
            ] {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if exact_ascii(c, Some(original.value.text()), input)? {
                    return Ok((
                        if prefix.is_some() {
                            Some((local, input.len() - local.len()))
                        } else {
                            None
                        },
                        Some(namespace),
                        prefix,
                    ));
                }
            }
            Ok::<_, SessionError>((adjustment, None, None))
        })();
        let (adjustment, namespace, prefix) = match classified {
            Ok(value) => value,
            Err(error) => {
                original.release(c)?;
                return Err(error);
            }
        };
        let (local, ticket) = self.register_foreign_text(c, original, adjustment)?;
        let (namespace, namespace_ticket) = match namespace {
            Some(uri) => {
                let value = literal_string(c, uri.as_bytes())?;
                let (value, ticket) = self.register_string(c, value, None)?;
                (Namespace::from_uri(value), Some(ticket))
            }
            None => (Namespace::none(), None),
        };
        let (prefix, prefix_ticket) = match prefix {
            Some(prefix) => {
                let value = literal_string(c, prefix.as_bytes())?;
                let (value, ticket) = self.register_string(c, value, None)?;
                (Some(value), Some(ticket))
            }
            None => (None, None),
        };
        Ok((
            QualifiedName::new(namespace, prefix, local),
            ticket,
            namespace_ticket,
            prefix_ticket,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenizer::{states::State, tests::LexerFixture, token::Event};

    fn lexical<'a>(
        source: &'a crate::HtmlSource,
        c: &mut ParseControl,
    ) -> (LexerFixture<'a>, Option<Tag>, crate::HtmlSourceRange) {
        let mut lexer = LexerFixture::new(source, State::Data, None, c).unwrap();
        loop {
            match lexer.next(c, false).unwrap().unwrap() {
                Event::Token(event) => {
                    let token = event.into_token(c).unwrap();
                    let TokenKind::Tag(tag) = token.kind else {
                        panic!("actual lexical start");
                    };
                    if tag.name.value.text() == &DocumentString::from("head") {
                        let mut ignored = Some(TokenKind::Tag(tag));
                        release_token_kind(&mut ignored, c).unwrap();
                        continue;
                    }
                    return (lexer, Some(tag), token.source);
                }
                event => event.release(c).unwrap(),
            }
        }
    }
    fn zero(c: &ParseControl) {
        for resource in [
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::TokenAttributes,
            HtmlParseResource::AuxiliaryEntries,
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::SourceMapEntries,
            HtmlParseResource::ManagedCapacityBytes,
            HtmlParseResource::PeakTransientBytes,
        ] {
            assert_eq!(c.usage().resource(resource).current, 0, "{resource:?}");
        }
    }

    #[test]
    fn foreign_adjustment_coexisting_units_maps_and_managed_units_have_literal_exact_caps() {
        for resource in [
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::SourceMapEntries,
            HtmlParseResource::ManagedUtf16Units,
        ] {
            // Fresh implicit html owns 4+28 units. The real lowercase token
            // owns 14, its case-adjusted replacement owns 14, SVG URI owns26.
            // Both original/replacement maps coexist (14+14), plus one actual
            // canonical MissingDoctype diagnostic location; their buffers
            // also coexist with the actual tokenizer last-start name14 (42).
            // The shorter head fixture keeps factory peak below that boundary.
            // Neither source nor old ticket is
            // aliased or silently refunded to make the new allocation fit.
            let exact = match resource {
                HtmlParseResource::ManagedUtf16Units => 86,
                HtmlParseResource::BufferedUtf16Units => 42,
                // The 28 retained original/replacement origin records coexist
                // with five field groups, three request/prologue causes, one
                // replacement cause, and
                // the one canonical diagnostic location. Actual lexical
                // retention adds eight head rows in the standalone owner and
                // 26 head/lineargradient rows in the independent lexer: 38+34, plus two empty nonce groups.
                _ => 74,
            };
            for cap in [exact - 1, exact] {
                let builder = super::super::tests::limits().into_builder();
                let configured = match resource {
                    HtmlParseResource::BufferedUtf16Units => builder.buffered_utf16_units(cap),
                    HtmlParseResource::SourceMapEntries => builder.source_map_entries(cap),
                    _ => builder.managed_utf16_units(cap),
                }
                .finish()
                .unwrap();
                let request = crate::HtmlDocumentParseRequest::offline(
                    crate::HtmlSource::utf8("<head><lineargradient>"),
                    configured,
                );
                let (mut c, mut session, capability) =
                    super::super::tests::fresh_script_owner(&request);
                assert_eq!(session.retained_evidence().unwrap().len(), 1);
                let root = session.root(&mut c).unwrap();
                let (lexer, mut input, range) = lexical(request.source(), &mut c);
                let result = session.allocate_foreign_tag_for_creation(
                    &mut c,
                    &mut input,
                    &root,
                    ForeignNamespace::Svg,
                    &range,
                    &capability,
                );
                if cap == exact {
                    let node = result.unwrap();
                    let name = session.element_name(&mut c, &node).unwrap().unwrap();
                    assert_eq!(name.local_name(), &DocumentString::from("linearGradient"));
                    assert_eq!(name.namespace(), &Namespace::svg());
                    assert!(input.is_none());
                } else {
                    let SessionError::Stop(first) = result.unwrap_err() else {
                        panic!("actual cap stop");
                    };
                    assert!(
                        matches!(first, ParseStop::LimitExceeded { resource: stopped, .. } if stopped == resource)
                    );
                    assert_eq!(c.stop(), Some(first));
                    assert!(
                        matches!(session.freeze(&mut c), Err(SessionError::Stop(stop)) if stop == first)
                    );
                    assert_eq!(session.active().unwrap().records.len(), 2);
                }
                let mut pending = input.map(TokenKind::Tag);
                if pending.is_some() {
                    release_token_kind(&mut pending, &mut c).unwrap();
                }
                lexer.release(&mut c).unwrap();
                session.release(&mut c).unwrap();
                zero(&c);
            }
        }
    }

    #[test]
    fn foreign_creation_alias_foreign_source_parent_and_cancel_preflight_preserve_actual_tag() {
        for case in 0..4 {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8("<svg viewbox=x>"),
                super::super::tests::limits(),
            );
            let (mut c, mut session, capability) =
                super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let (lexer, mut input, range) = lexical(request.source(), &mut c);
            let alias = if case == 0 {
                Some(input.as_ref().unwrap().name.value.text().clone())
            } else {
                None
            };
            let other_source = crate::HtmlSource::utf8("<svg viewbox=x>");
            let mut other =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let other_root = other.root(&mut c).unwrap();
            if case == 3 {
                request.cancellation().cancel();
            }
            let supplied_range = if case == 1 {
                other_source.range(0, other_source.len()).unwrap()
            } else {
                range
            };
            let result = session.allocate_foreign_tag_for_creation(
                &mut c,
                &mut input,
                if case == 2 { &other_root } else { &root },
                ForeignNamespace::Svg,
                &supplied_range,
                &capability,
            );
            match case {
                0 => assert!(matches!(result, Err(SessionError::AliasedPayload))),
                2 => assert!(matches!(result, Err(SessionError::ForeignNode))),
                _ => assert!(matches!(result, Err(SessionError::Stop(_)))),
            }
            assert!(input.is_some());
            assert_eq!(session.active().unwrap().records.len(), 2);
            drop(alias);
            let mut pending = input.map(TokenKind::Tag);
            release_token_kind(&mut pending, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            other.release(&mut c).unwrap();
            zero(&c);
        }
    }

    #[test]
    fn foreign_adjusted_fields_keep_original_ranges_and_distinct_native_backings() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<svg viewbox=x xlink:href='#y'>"),
            super::super::tests::limits(),
        );
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let (lexer, mut input, range) = lexical(request.source(), &mut c);
        let original_view = input.as_ref().unwrap().attributes.as_slice()[0]
            .name
            .origins
            .as_slice()[0]
            .source
            .clone();
        let original_href = input.as_ref().unwrap().attributes.as_slice()[1]
            .name
            .origins
            .as_slice()[6]
            .source
            .clone();
        let node = session
            .allocate_foreign_tag_for_creation(
                &mut c,
                &mut input,
                &root,
                ForeignNamespace::Svg,
                &range,
                &capability,
            )
            .unwrap();
        let storage = session.active().unwrap();
        let mut original = None;
        let mut adjusted = None;
        let mut href = None;
        for entry in &storage.payloads.values {
            if let RetainedPayload::String {
                value,
                origins: Some(maps),
                location,
                ..
            } = entry
            {
                if value == &DocumentString::from("viewbox") {
                    original = Some((
                        value.backing().as_ptr(),
                        storage.observed_origins(maps, &mut c).unwrap()[0]
                            .source
                            .clone(),
                        storage.observed_origins(maps, &mut c).unwrap()[0].kind,
                    ));
                }
                if matches!(location, StringLocation::AttributeName(_, 0)) {
                    adjusted = Some((
                        value.backing().as_ptr(),
                        storage.observed_origins(maps, &mut c).unwrap()[0]
                            .source
                            .clone(),
                        storage.observed_origins(maps, &mut c).unwrap()[0].kind,
                    ));
                }
                if matches!(location, StringLocation::AttributeName(_, 1)) {
                    href = Some((
                        storage.observed_origins(maps, &mut c).unwrap().len(),
                        storage.observed_origins(maps, &mut c).unwrap()[0].start,
                        storage.observed_origins(maps, &mut c).unwrap()[0].end,
                        storage.observed_origins(maps, &mut c).unwrap()[0]
                            .source
                            .clone(),
                    ));
                }
            }
        }
        let original = original.unwrap();
        let adjusted = adjusted.unwrap();
        assert_ne!(original.0, adjusted.0);
        assert_eq!(original.1, original_view);
        assert_eq!(adjusted.1, original_view);
        assert_ne!(
            original.2,
            crate::parser::tokenizer::token::OriginKind::RecoverySyntax
        );
        assert_eq!(
            adjusted.2,
            crate::parser::tokenizer::token::OriginKind::RecoverySyntax
        );
        assert_eq!(href.unwrap(), (4, 0, 1, original_href));
        session
            .insert(&mut c, &root, &node, None, Some(&capability))
            .unwrap();
        lexer.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let element = frozen.document().unwrap().root().children().last().unwrap();
        let native = frozen.document().unwrap().node(&element).unwrap();
        let HtmlNodeKind::Element(native) = native.kind() else {
            panic!("actual foreign element");
        };
        assert_eq!(
            native.attributes().as_slice()[0].name().local_name(),
            &DocumentString::from("viewBox")
        );
        assert_eq!(
            native.attributes().as_slice()[1].name().prefix(),
            Some(&DocumentString::from("xlink"))
        );
        drop(element);
        frozen.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn foreign_creation_stopped_prefixes_keep_all_adjusted_field_owners_for_cleanup() {
        // A finite sweep targets the new creation operation, after a real
        // admitted lexical fixture. The independent oracle is registry/native
        // alias coherence, sticky refusal and zero storage after disposal.
        for allowance in (0..20_000).step_by(137) {
            let configured = super::super::tests::limits()
                .into_builder()
                .work_credits(100_000_000)
                .finish()
                .unwrap();
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8("<svg viewbox=x xlink:href='#y'>"),
                configured,
            );
            let (mut c, mut session, capability) =
                super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let (lexer, mut input, range) = lexical(request.source(), &mut c);
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                request.limits().work_credits() - used - allowance,
            )
            .unwrap();
            let result = session.allocate_foreign_tag_for_creation(
                &mut c,
                &mut input,
                &root,
                ForeignNamespace::Svg,
                &range,
                &capability,
            );
            if let Err(SessionError::Stop(first)) = result {
                assert!(matches!(
                    first,
                    ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }
                ));
                assert_eq!(c.stop(), Some(first));
                assert!(
                    matches!(session.freeze(&mut c), Err(SessionError::Stop(stop)) if stop == first)
                );
                session
                    .active()
                    .unwrap()
                    .payloads
                    .check_aliases(&session.active().unwrap().records, &mut c)
                    .unwrap();
            } else {
                result.unwrap();
            }
            let mut pending = input.map(TokenKind::Tag);
            if pending.is_some() {
                release_token_kind(&mut pending, &mut c).unwrap();
            }
            lexer.release(&mut c).unwrap();
            // Active disposal can itself exhaust the deliberate allowance;
            // a terminal retry completes the same actual owner, never parsing.
            if let Err(SessionError::Stop(_)) = session.release(&mut c) {
                session.release(&mut c).unwrap();
            }
            zero(&c);
        }
    }

    #[test]
    fn foreign_clone_stopped_qualified_attribute_prefixes_keep_each_ticket_owned() {
        let mut stopped_after_element = false;
        let mut completed = false;
        // Keep all original sampled stop cuts. The first actual full-request
        // clone establishes its cost on the compiled retained evidence layout;
        // append exact and one-short late cuts rather than guess a larger cap.
        let mut cases = vec![None];
        cases.extend((0..200_000).step_by(997).map(Some));
        let mut case = 0;
        while case < cases.len() {
            let allowance = cases[case];
            case += 1;
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8("<svg xlink:href='#y' xml:lang=en>"),
                super::super::tests::limits()
                    .into_builder()
                    .work_credits(100_000_000)
                    .finish()
                    .unwrap(),
            );
            let (mut c, mut session, capability) =
                super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let (lexer, mut input, range) = lexical(request.source(), &mut c);
            let original = session
                .allocate_foreign_tag_for_creation(
                    &mut c,
                    &mut input,
                    &root,
                    ForeignNamespace::Svg,
                    &range,
                    &capability,
                )
                .unwrap();
            session
                .insert(&mut c, &root, &original, None, Some(&capability))
                .unwrap();
            session
                .active()
                .unwrap()
                .qualify_clone_subtree(&mut c, root.index)
                .unwrap();
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            if let Some(allowance) = allowance {
                c.charge(
                    WorkCategory::StateStep,
                    request.limits().work_credits() - used - allowance,
                )
                .unwrap();
            }
            let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let result = session.active_mut().unwrap().clone_node(
                &mut c,
                original.index,
                DocumentRef::Owned(0),
            );
            match result {
                Ok(index) => {
                    let cost = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
                    if let Some(allowance) = allowance {
                        assert!(allowance >= cost);
                    } else {
                        assert!(cost > 0);
                        cases.push(Some(cost));
                        cases.push(Some(cost - 1));
                    }
                    completed = true;
                    assert_ne!(index, original.index);
                    let storage = session.active().unwrap();
                    let HtmlNodeKind::Element(cloned) = &storage.records[index].kind else {
                        panic!("native clone");
                    };
                    assert_eq!(cloned.name().namespace(), &Namespace::svg());
                    let attrs = cloned.attributes().as_slice();
                    assert_eq!(attrs.len(), 2);
                    assert_eq!(
                        attrs[0].name().namespace().uri(),
                        Some(&DocumentString::from(XLINK))
                    );
                    assert_eq!(
                        attrs[0].name().prefix(),
                        Some(&DocumentString::from("xlink"))
                    );
                    assert_eq!(attrs[0].name().local_name(), &DocumentString::from("href"));
                    assert_eq!(attrs[0].value(), &DocumentString::from("#y"));
                    assert_eq!(
                        attrs[1].name().namespace().uri(),
                        Some(&DocumentString::from(XML))
                    );
                    assert_eq!(attrs[1].name().prefix(), Some(&DocumentString::from("xml")));
                    assert_eq!(attrs[1].name().local_name(), &DocumentString::from("lang"));
                    assert_eq!(attrs[1].value(), &DocumentString::from("en"));
                }
                Err(SessionError::Stop(first)) => {
                    assert_eq!(c.stop(), Some(first));
                    assert!(matches!(
                        first,
                        ParseStop::LimitExceeded {
                            resource: HtmlParseResource::WorkCredits,
                            ..
                        }
                    ));
                    stopped_after_element |= session.active().unwrap().records.len() > 3;
                    assert!(
                        matches!(session.freeze(&mut c), Err(SessionError::Stop(stop)) if stop == first)
                    );
                }
                Err(error) => panic!("qualified clone failure: {error:?}"),
            }
            match session
                .active()
                .unwrap()
                .payloads
                .check_aliases(&session.active().unwrap().records, &mut c)
            {
                Ok(()) => {}
                Err(SessionError::Stop(first)) => {
                    // The exact clone cut can stop at the following cleanup
                    // qualification. Terminal cleanup rechecks the actual aliases
                    // without additional work and preserves that first stop.
                    assert_eq!(c.stop(), Some(first));
                    session
                        .active()
                        .unwrap()
                        .payloads
                        .check_aliases(&session.active().unwrap().records, &mut c)
                        .unwrap();
                }
                Err(error) => panic!("actual clone aliases: {error:?}"),
            }
            lexer.release(&mut c).unwrap();
            if let Err(SessionError::Stop(_)) = session.release(&mut c) {
                session.release(&mut c).unwrap();
            }
            zero(&c);
        }
        assert!(stopped_after_element);
        assert!(completed);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ForeignNamespace {
    MathMl,
    Svg,
    Context,
}
impl ForeignNamespace {
    pub(crate) fn uri(self) -> Option<&'static str> {
        match self {
            Self::MathMl => Some("http://www.w3.org/1998/Math/MathML"),
            Self::Svg => Some("http://www.w3.org/2000/svg"),
            Self::Context => None,
        }
    }
    fn element_adjustment(
        self,
        c: &mut ParseControl,
        name: &DocumentString,
    ) -> Result<Option<(&'static str, usize)>, SessionError> {
        if self == Self::Svg {
            lookup(c, name, SVG_NAMES)
        } else {
            Ok(None)
        }
    }
}
const SVG_NAMES: &[(&str, &str)] = &[
    ("altglyph", "altGlyph"),
    ("altglyphdef", "altGlyphDef"),
    ("altglyphitem", "altGlyphItem"),
    ("animatecolor", "animateColor"),
    ("animatemotion", "animateMotion"),
    ("animatetransform", "animateTransform"),
    ("clippath", "clipPath"),
    ("feblend", "feBlend"),
    ("fecolormatrix", "feColorMatrix"),
    ("fecomponenttransfer", "feComponentTransfer"),
    ("fecomposite", "feComposite"),
    ("feconvolvematrix", "feConvolveMatrix"),
    ("fediffuselighting", "feDiffuseLighting"),
    ("fedisplacementmap", "feDisplacementMap"),
    ("fedistantlight", "feDistantLight"),
    ("fedropshadow", "feDropShadow"),
    ("feflood", "feFlood"),
    ("fefunca", "feFuncA"),
    ("fefuncb", "feFuncB"),
    ("fefuncg", "feFuncG"),
    ("fefuncr", "feFuncR"),
    ("fegaussianblur", "feGaussianBlur"),
    ("feimage", "feImage"),
    ("femerge", "feMerge"),
    ("femergenode", "feMergeNode"),
    ("femorphology", "feMorphology"),
    ("feoffset", "feOffset"),
    ("fepointlight", "fePointLight"),
    ("fespecularlighting", "feSpecularLighting"),
    ("fespotlight", "feSpotLight"),
    ("fetile", "feTile"),
    ("feturbulence", "feTurbulence"),
    ("foreignobject", "foreignObject"),
    ("glyphref", "glyphRef"),
    ("lineargradient", "linearGradient"),
    ("radialgradient", "radialGradient"),
    ("textpath", "textPath"),
];
const SVG_ATTRIBUTES: &[(&str, &str)] = &[
    ("attributename", "attributeName"),
    ("attributetype", "attributeType"),
    ("basefrequency", "baseFrequency"),
    ("baseprofile", "baseProfile"),
    ("calcmode", "calcMode"),
    ("clippathunits", "clipPathUnits"),
    ("diffuseconstant", "diffuseConstant"),
    ("edgemode", "edgeMode"),
    ("filterunits", "filterUnits"),
    ("glyphref", "glyphRef"),
    ("gradienttransform", "gradientTransform"),
    ("gradientunits", "gradientUnits"),
    ("kernelmatrix", "kernelMatrix"),
    ("kernelunitlength", "kernelUnitLength"),
    ("keypoints", "keyPoints"),
    ("keysplines", "keySplines"),
    ("keytimes", "keyTimes"),
    ("lengthadjust", "lengthAdjust"),
    ("limitingconeangle", "limitingConeAngle"),
    ("markerheight", "markerHeight"),
    ("markerunits", "markerUnits"),
    ("markerwidth", "markerWidth"),
    ("maskcontentunits", "maskContentUnits"),
    ("maskunits", "maskUnits"),
    ("numoctaves", "numOctaves"),
    ("pathlength", "pathLength"),
    ("patterncontentunits", "patternContentUnits"),
    ("patterntransform", "patternTransform"),
    ("patternunits", "patternUnits"),
    ("pointsatx", "pointsAtX"),
    ("pointsaty", "pointsAtY"),
    ("pointsatz", "pointsAtZ"),
    ("preservealpha", "preserveAlpha"),
    ("preserveaspectratio", "preserveAspectRatio"),
    ("primitiveunits", "primitiveUnits"),
    ("refx", "refX"),
    ("refy", "refY"),
    ("repeatcount", "repeatCount"),
    ("repeatdur", "repeatDur"),
    ("requiredextensions", "requiredExtensions"),
    ("requiredfeatures", "requiredFeatures"),
    ("specularconstant", "specularConstant"),
    ("specularexponent", "specularExponent"),
    ("spreadmethod", "spreadMethod"),
    ("startoffset", "startOffset"),
    ("stddeviation", "stdDeviation"),
    ("stitchtiles", "stitchTiles"),
    ("surfacescale", "surfaceScale"),
    ("systemlanguage", "systemLanguage"),
    ("tablevalues", "tableValues"),
    ("targetx", "targetX"),
    ("targety", "targetY"),
    ("textlength", "textLength"),
    ("viewbox", "viewBox"),
    ("viewtarget", "viewTarget"),
    ("xchannelselector", "xChannelSelector"),
    ("ychannelselector", "yChannelSelector"),
    ("zoomandpan", "zoomAndPan"),
];

impl SessionStorage {
    fn allocate_foreign_payload(
        &mut self,
        control: &mut ParseControl,
        name: Text,
        foreign: ForeignNamespace,
        mut attributes: OwnedVec<crate::parser::tokenizer::token::Attribute>,
        prepared: Option<template::PreparedDocuments>,
        svg: Option<usize>,
    ) -> Result<ParserNode, SessionError> {
        let result = (|| {
            let replacement = match foreign.element_adjustment(control, name.value.text()) {
                Ok(value) => value,
                Err(error) => {
                    name.release(control)?;
                    return Err(error);
                }
            };
            let (name, name_ticket) = self.register_foreign_text(control, name, replacement)?;
            // The selected HTML namespace is a defining literal, copied through
            // the same admitted exact buffer as token-created units.
            let (namespace, namespace_ticket) = if let Some(uri) = foreign.uri() {
                let namespace = literal_string(control, uri.as_bytes())?;
                let (value, ticket) = self.register_string(control, namespace, None)?;
                (Namespace::from_uri(value), Some(ticket))
            } else {
                self.copy_context_namespace(control)?
            };
            control.charge(WorkCategory::RecordVisit, 2)?;
            let mut units = name
                .len()
                .checked_add(namespace.uri().map_or(0, DocumentString::len))
                .ok_or_else(|| {
                    control
                        .representation::<()>(HtmlParseResource::ManagedUtf16Units)
                        .unwrap_err()
                })?;
            let (values, allocation) = control.new_vector()?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            let original_len = attributes.len();
            if let Err(error) = self.prepare_payload_slot(control) {
                release_vector(control, values, allocation)?;
                return Err(error);
            }
            let container = self.payloads.values.len();
            self.payloads.values.push(RetainedPayload::Attributes(Some(
                AttributeStorage::Building {
                    values,
                    allocation,
                    original_len,
                    reversed_pairs: 0,
                },
            )));
            while let Some(attribute) = attributes.try_pop(control)? {
                let adjusted = self.foreign_attribute(control, attribute.name, foreign);
                let (qualified, local_ticket, namespace_ticket, prefix_ticket) = match adjusted {
                    Ok(value) => value,
                    Err(error) => {
                        attribute.value.release(control)?;
                        return Err(error);
                    }
                };
                let (value, value_ticket) = self.register_text(control, attribute.value)?;
                control.charge(WorkCategory::RecordVisit, 2)?;
                units = units
                    .checked_add(qualified.local_name().len())
                    .and_then(|sum| {
                        sum.checked_add(qualified.namespace().uri().map_or(0, DocumentString::len))
                    })
                    .and_then(|sum| {
                        sum.checked_add(qualified.prefix().map_or(0, DocumentString::len))
                    })
                    .and_then(|sum| sum.checked_add(value.len()))
                    .ok_or_else(|| {
                        control
                            .representation::<()>(HtmlParseResource::ManagedUtf16Units)
                            .unwrap_err()
                    })?;
                let RetainedPayload::Attributes(Some(AttributeStorage::Building {
                    values,
                    allocation,
                    ..
                })) = &mut self.payloads.values[container]
                else {
                    unreachable!()
                };
                control.reserve_vector(values, allocation, 1)?;
                payload_write::<Attribute>(control, 1)?;
                payload_write::<[StringLocation; 2]>(control, 2)?;
                if namespace_ticket.is_some() {
                    payload_write::<StringLocation>(control, 1)?;
                }
                if prefix_ticket.is_some() {
                    payload_write::<StringLocation>(control, 1)?;
                }
                control.acquire(HtmlParseResource::ManagedAttributes, 1)?;
                let position = attributes.len();
                values.push(Attribute::new(qualified, value));
                self.bind_string(
                    local_ticket,
                    StringLocation::AttributeName(container, position),
                );
                self.bind_string(
                    value_ticket,
                    StringLocation::AttributeValue(container, position),
                );
                if let Some(ticket) = namespace_ticket {
                    self.bind_string(
                        ticket,
                        StringLocation::AttributeNamespace(container, position),
                    );
                }
                if let Some(ticket) = prefix_ticket {
                    self.bind_string(ticket, StringLocation::AttributePrefix(container, position));
                }
            }
            self.reverse_attributes(control, container)?;
            payload_write::<Option<AttributeStorage>>(control, 2)?;
            let RetainedPayload::Attributes(storage) = &mut self.payloads.values[container] else {
                unreachable!()
            };
            let AttributeStorage::Building {
                values,
                allocation,
                original_len,
                reversed_pairs,
            } = storage.take().expect("attribute builder phase")
            else {
                unreachable!()
            };
            let (backing, shared) = match control.share_owned(values) {
                Ok(value) => value,
                Err((stop, values)) => {
                    *storage = Some(AttributeStorage::Building {
                        values,
                        allocation,
                        original_len,
                        reversed_pairs,
                    });
                    return Err(stop.into());
                }
            };
            let ordered = match OrderedAttributes::from_parser_storage(backing, control) {
                Ok(values) => values,
                Err((error, backing)) => {
                    // Preserve the admitted shared wrapper in the owning phase
                    // even when intrinsic duplicate checking or work stops.
                    let values = control.release_shared(backing, shared).unwrap_or_else(|_| {
                        panic!("unpublished attribute builder is uniquely owned")
                    });
                    *storage = Some(AttributeStorage::Building {
                        values,
                        allocation,
                        original_len,
                        reversed_pairs,
                    });
                    return Err(match error {
                        ParserAttributeError::Stop(stop) => stop.into(),
                        ParserAttributeError::Duplicate { .. } => SessionError::DuplicateAttribute,
                    });
                }
            };
            *storage = Some(AttributeStorage::Frozen {
                values: ordered,
                allocation,
                shared,
                element: None,
            });
            control.charge(WorkCategory::ProviderCall, 1)?;
            control.charge(
                WorkCategory::CopyByte,
                mem::size_of::<OrderedAttributes>() as u64,
            )?;
            let RetainedPayload::Attributes(Some(AttributeStorage::Frozen { values, .. })) =
                &self.payloads.values[container]
            else {
                unreachable!()
            };
            let attributes = values.clone();
            let count = attributes.len();
            let template = false;
            // Preadmit the actual native-alias bindings and both usage writes.
            // Successful allocation and binding have no fallible alias gap.
            payload_write::<StringLocation>(control, 1 + u64::from(namespace_ticket.is_some()))?;
            payload_write::<Option<usize>>(control, 1)?;
            payload_write::<[usize; 2]>(control, 2)?;
            if svg.is_some() {
                payload_write::<Option<usize>>(control, 1)?;
            }
            let node = self.allocate_prepared_owned(
                control,
                HtmlNodeKind::Element(HtmlElement {
                    link_flags: None,
                    name: QualifiedName::new(namespace, None, name),
                    attributes,
                }),
                template,
                0,
                0,
                prepared,
            )?;
            if let Some(slot) = svg {
                self.bind_svg_slot(slot, node.index);
            }
            self.bind_string(name_ticket, StringLocation::ElementName(node.index));
            if let Some(ticket) = namespace_ticket {
                self.bind_string(ticket, StringLocation::Namespace(node.index));
            }
            let RetainedPayload::Attributes(Some(AttributeStorage::Frozen { element, .. })) =
                &mut self.payloads.values[container]
            else {
                unreachable!()
            };
            *element = Some(node.index);
            self.usage.attributes += count;
            self.usage.utf16_units += units;
            self.initial_nonce_attributes(control, node.index)?;
            Ok(node)
        })();
        release_token_attributes(attributes, control)?;
        result
    }
}
