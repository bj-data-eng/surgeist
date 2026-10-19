//! Exact completed metadata adaptation; no native mutation-hook replay.
use super::*;
use crate::element::NativeFamily;
use surgeist_html::{
    HtmlElementClass as Requirement, HtmlInputImageHistory, HtmlInputSelection,
    HtmlNativeFormAssociationKind as Form, KnownHtmlElement as Element,
};

fn required<T>(value: Option<T>, node: HtmlNode<'_>) -> Result<T, Error> {
    value.ok_or_else(|| Error::MissingMetadata { subject: node.id() })
}

fn missing_meta(value: surgeist_html_parser::HtmlParseMetaMissing<'_>) -> Error {
    Error::MetaStateUnavailable {
        document: value.document().id(),
        subject: value.subject().id(),
        reason: value.kind().clone(),
    }
}

fn missing_dialog(value: &surgeist_html_parser::HtmlParseDialogMissing<'_>) -> Error {
    Error::DialogStateUnavailable {
        subject: value.subject().id(),
        action: value.action(),
    }
}

pub(super) fn document_interaction(
    source: &Source<'_>,
    node: HtmlNode<'_>,
    c: &mut Control<'_>,
) -> Result<crate::dialog::DocumentInteractiveState<Key>, Error> {
    let view = required(
        source.observation(&node.id(), c, |r| r.document_interaction())?,
        node,
    )?;
    c.step(1)?;
    if view.document().id() != node.id() {
        return Err(Error::UnsupportedState { subject: node.id() });
    }
    crate::dialog::DocumentInteractiveState::from_parsed(view.disposition())
        .map_err(|missing| missing_dialog(&missing))
}

/// Qualify original per-Document processing before canonical allocation. The
/// destination constructor independently establishes its current association
/// environment; the source's offline profile never supplies that witness.
pub(super) fn transfer_document_meta<'a>(
    source: &Source<'a>,
    node: HtmlNode<'a>,
    metadata: &mut DomDocumentMetadata,
    c: &mut Control<'_>,
) -> Result<(), Error> {
    use surgeist_html_parser::{
        HtmlParseMetaDisposition, HtmlParseMetaDomain, HtmlParseMetaProfile, HtmlParseMetaRefresh,
    };
    let view = required(
        source.observation(&node.id(), c, |r| r.document_meta())?,
        node,
    )?;
    c.step(1)?;
    if view.document().id() != node.id() {
        return Err(Error::UnsupportedState { subject: node.id() });
    }
    match view.profile() {
        HtmlParseMetaProfile::OfflineDoNothingNoUi => {}
    }
    if let Some(missing) = view.first_missing() {
        return Err(missing_meta(missing));
    }
    for domain in [
        HtmlParseMetaDomain::Refresh,
        HtmlParseMetaDomain::PreferredStyle,
        HtmlParseMetaDomain::Csp,
        HtmlParseMetaDomain::ThemeColor,
        HtmlParseMetaDomain::ColorScheme,
    ] {
        c.step(1)?;
        match view.disposition(domain) {
            HtmlParseMetaDisposition::Complete => {}
            HtmlParseMetaDisposition::Unavailable(missing) => return Err(missing_meta(missing)),
        }
    }
    c.step(1)?;
    let refresh = match view.refresh() {
        HtmlParseMetaRefresh::Known(value) => value,
        HtmlParseMetaRefresh::Unavailable(missing) => return Err(missing_meta(missing)),
    };
    let language = view.pragma_language();
    if let Some(language) = language {
        c.string(language)?;
    }
    metadata.establish_parsed_meta_history(language.cloned(), view.referrer_policy(), refresh);
    Ok(())
}

pub(super) fn admit_history(
    report: &HtmlParseReport,
    environment: Option<&DomParsedDocumentEnvironment>,
    c: &mut Control<'_>,
) -> Result<(), Error> {
    c.step(1)?;
    let source = report.source();
    let source_bytes = if let Some(utf16) = source.as_utf16() {
        utf16
            .shared_code_units()
            .capacity()
            .checked_mul(size_of::<u16>())
            .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
            .ok_or(Error::RepresentationExhausted)?
    } else {
        source.len()
    };
    // The URL owner's actual payload and this exact encoding-label backing
    // remain reachable through the installed Document. Sharing does not erase
    // their spare backing admission.
    let environment_bytes = if let Some(environment) = environment {
        environment
            .url
            .retained_storage_usage()
            .retained_bytes()
            .checked_add(size_of::<Vec<u16>>())
            .and_then(|n| {
                n.checked_add(
                    environment
                        .encoding_label
                        .shared_code_units()
                        .capacity()
                        .checked_mul(size_of::<u16>())?,
                )
            })
            .ok_or(Error::RepresentationExhausted)?
    } else {
        0
    };
    let mut bytes = report
        .retained_construction_bytes()
        .checked_add(size_of::<HtmlParseReport>() as u64)
        .ok_or(Error::RepresentationExhausted)?
        .checked_add(u64::try_from(source_bytes).map_err(|_| Error::RepresentationExhausted)?)
        .and_then(|v| v.checked_add(u64::try_from(environment_bytes).ok()?))
        .ok_or(Error::RepresentationExhausted)?;
    c.limit(Resource::HistoricalBytes, bytes, c.limits.historical_bytes)?;
    if let Some(context) = report.context() {
        let capture = context
            .capture()
            .retained_storage_usage_with(|| c.step(1))
            .map_err(|error| match error {
                surgeist_html_semantics::HtmlCaptureStorageError::Interrupted(error) => error,
                surgeist_html_semantics::HtmlCaptureStorageError::RepresentationExhausted => {
                    Error::RepresentationExhausted
                }
            })?;
        bytes = bytes
            .checked_add(
                u64::try_from(capture.retained_bytes())
                    .map_err(|_| Error::RepresentationExhausted)?,
            )
            .ok_or(Error::RepresentationExhausted)?;
        c.limit(Resource::HistoricalBytes, bytes, c.limits.historical_bytes)?;
    }
    c.step(1)?;
    if let Some(facts) = report.captured_base_facts() {
        // This separately lent graph is retained request history, not an
        // interpretation input or a substitute canonical policy binding. Its
        // owner receipt includes the out-of-line header and all spare backing;
        // the report header above includes only the Arc handle.
        bytes = bytes
            .checked_add(
                u64::try_from(facts.retained_storage_usage().retained_bytes())
                    .map_err(|_| Error::RepresentationExhausted)?,
            )
            .ok_or(Error::RepresentationExhausted)?;
        c.limit(Resource::HistoricalBytes, bytes, c.limits.historical_bytes)?;
    }
    let root_facts = report.document_base_facts();
    if let Some(facts) = root_facts {
        let document = facts.document();
        if !environment.is_some_and(|environment| {
            document.document_url() == Some(&environment.url)
                && document.encoding() == Some(&environment.encoding)
        }) {
            return Err(Error::EnvironmentMismatch);
        }
        // This owner-issued receipt includes private observation fallback
        // records and actual spare UTF-16/vector backing; formation's logical
        // string extent is deliberately not used as a physical byte measure.
        bytes = bytes
            .checked_add(
                u64::try_from(facts.retained_storage_usage().retained_bytes())
                    .map_err(|_| Error::RepresentationExhausted)?,
            )
            .ok_or(Error::RepresentationExhausted)?;
        c.limit(Resource::HistoricalBytes, bytes, c.limits.historical_bytes)?;
    }
    let mut metadata = report.metadata();
    loop {
        c.step(1)?;
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if let Some(facts) = record.document_base_facts()
                    && !root_facts.is_some_and(|root| root.same_owner(facts))
                {
                    bytes = bytes
                        .checked_add(
                            u64::try_from(facts.retained_storage_usage().retained_bytes())
                                .map_err(|_| Error::RepresentationExhausted)?,
                        )
                        .ok_or(Error::RepresentationExhausted)?;
                    c.limit(Resource::HistoricalBytes, bytes, c.limits.historical_bytes)?;
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    c.limit(Resource::HistoricalBytes, bytes, c.limits.historical_bytes)?;
    if let Some(environment) = environment {
        c.string(&environment.encoding_label)?;
        c.string(environment.url.serialization())?;
    }
    Ok(())
}

pub(super) fn metadata(
    source: &Source<'_>,
    draft: &mut Draft,
    limits: DomLimits,
    c: &mut Control<'_>,
) -> Result<(), Error> {
    for record in &source.metadata {
        c.step(1)?;
        let node = record.node();
        if source.index(&node.id(), c)?.is_none() {
            continue;
        }
        let key = source.key(&node.id(), c)?;
        if let Some(document) = record.node_document() {
            let document = source.document_key(document, c)?;
            if draft.node(key)?.document != document {
                return Err(Error::UnsupportedState { subject: node.id() });
            }
        }
        if let Some((_, inert, cache)) = record.document_facts() {
            let cache = cache.map(|value| source.key(&value.id(), c)).transpose()?;
            let value = Some(if inert {
                crate::template::TemplateDocument::InertTemplate
            } else {
                crate::template::TemplateDocument::Ordinary { inert: cache }
            });
            draft.with_node_fields(key, c.cancel, |fields| fields.template_document = value)?;
        }
        if let Some(template) = record.template() {
            if !matches!(
                node.template_contents_kind(),
                Some(
                    surgeist_html::HtmlTemplateContentsKind::Owned
                        | surgeist_html::HtmlTemplateContentsKind::ShadowReference
                )
            ) {
                return Err(Error::UnsupportedState { subject: node.id() });
            }
            let owned = node.template_contents_kind()
                == Some(surgeist_html::HtmlTemplateContentsKind::Owned);
            if owned
                && template.contents().fragment_owner()
                    != Some(&surgeist_html::HtmlFragmentOwner::Template(node.id()))
            {
                return Err(Error::UnsupportedState { subject: node.id() });
            }
            let contents = source.key(&template.contents().id(), c)?;
            let patch = crate::template::TemplatePatch {
                target: template
                    .target()
                    .map(|value| source.key(&value.id(), c))
                    .transpose()?,
                start: template
                    .start()
                    .map(|value| source.key(&value.id(), c))
                    .transpose()?,
                end: template
                    .end()
                    .map(|value| source.key(&value.id(), c))
                    .transpose()?,
            };
            let value = Some(contents);
            draft.with_node_fields(key, c.cancel, |fields| fields.template_contents = value)?;
            let value = Some(patch);
            draft.with_node_fields(key, c.cancel, |fields| fields.template_patch = value)?;
            if owned {
                draft.with_node_fields(contents, c.cancel, |fields| fields.host = Some(key))?;
            }
        }
        if let Some(Some(select)) = record.option_nearest_select() {
            let value = Some(source.key(&select.id(), c)?);
            draft.with_node_fields(key, c.cancel, |fields| fields.option_select = value)?;
        }
        if let Some(owner) = record.form_owner() {
            let owner = owner.map(|value| source.key(&value.id(), c)).transpose()?;
            let parser_inserted = required(record.form_parser_inserted(), node)?;
            let value = owner;
            draft.with_node_fields(key, c.cancel, |fields| fields.control_owner = value)?;
            draft.with_element_fields(key, c.cancel, |element| {
                if let Some(control) = element.native_control_mut() {
                    control.parser_inserted = parser_inserted;
                } else if !element.is_img() {
                    return Err(Error::UnsupportedState { subject: node.id() });
                }
                Ok(())
            })??;
        }
        if let Some(script) = record.script_state() {
            let (parser, preparation) = required(record.script_documents(), node)?;
            let value = Some(crate::script::ScriptState {
                flags: surgeist_html::HtmlScriptFlags::from_retained(
                    script.force_async(),
                    script.already_started(),
                    script.external_file(),
                    script.ready_to_be_parser_executed(),
                    script.delaying_load(),
                ),
                kind: script.script_type(),
                parser_document: parser
                    .map(|value| source.document_key(value, c))
                    .transpose()?,
                preparation_document: preparation
                    .map(|value| source.document_key(value, c))
                    .transpose()?,
                result: script.result(),
                ready: script.steps_when_result_ready(),
            });
            draft.with_node_fields(key, c.cancel, |fields| fields.script = value)?;
        }
        if let Some(base) = record.base_state() {
            let frozen = match base {
                HtmlParseBaseState::Unfrozen => None,
                HtmlParseBaseState::Frozen(base) => {
                    c.string(base.value().serialization())?;
                    Some(crate::base::Frozen {
                        value: base.value().clone(),
                        document: source.key(&base.document().id(), c)?,
                        revision: base.revision().clone(),
                        // Report preparation excludes a Pending response. False
                        // preserves Returned; true is completed parser response,
                        // not native delivery.
                        response: if base.responded() {
                            crate::base::Response::ParsedCompleted
                        } else {
                            crate::base::Response::Returned
                        },
                    })
                }
                HtmlParseBaseState::Unavailable { .. } => {
                    return Err(Error::BaseStateUnavailable { subject: node.id() });
                }
            };
            let old = draft
                .node(key)?
                .base
                .as_ref()
                .map_or(0, |state| state.units());
            let next = crate::base::BaseState { frozen };
            replace_units(draft, old, next.units(), limits)?;
            let value = Some(next);
            draft.with_node_fields(key, c.cancel, |fields| fields.base = value)?;
        }
        if let Some(facts) = record.document_base_facts() {
            let old = draft
                .node(key)?
                .base_document
                .as_ref()
                .map_or(0, |state| state.units());
            let first = required(record.document_first_base(), node)?
                .map(|node| source.key(&node.id(), c))
                .transpose()?;
            let next = crate::base::BaseDocument {
                facts: Some(facts.clone()),
                first,
                render_blocking: Vec::new(),
            };
            replace_units(draft, old, next.units(), limits)?;
            let value = Some(next);
            draft.with_node_fields(key, c.cancel, |fields| fields.base_document = value)?;
        }
        if let Some(flags) = record.link_flags() {
            let parser = required(record.link_parser_document(), node)?
                .map(|value| source.document_key(value, c))
                .transpose()?;
            let value = Some(crate::base::LinkState {
                flags,
                parser_document: parser,
                sheet: None,
            });
            draft.with_node_fields(key, c.cancel, |fields| fields.link = value)?;
        }
        if let Some(hyperlink) = record.hyperlink() {
            let document = source.document_ref_key(hyperlink.document(), c)?;
            let parsed = match hyperlink.state() {
                surgeist_html_parser::HtmlParseHyperlinkState::AbsentHref => None,
                surgeist_html_parser::HtmlParseHyperlinkState::Observed(value) => {
                    c.string(value.raw())?;
                    Some(crate::hyperlink::Parsed {
                        document,
                        value: (*value).clone(),
                    })
                }
                surgeist_html_parser::HtmlParseHyperlinkState::Unavailable(reason) => {
                    return Err(Error::HyperlinkUnavailable {
                        subject: node.id(),
                        reason: (*reason).clone(),
                    });
                }
            };
            let old = draft
                .node(key)?
                .hyperlink
                .as_ref()
                .map_or(0, |state| state.units());
            let next = crate::hyperlink::HyperlinkState { parsed };
            replace_units(draft, old, next.units(), limits)?;
            let value = Some(next);
            draft.with_node_fields(key, c.cancel, |fields| fields.hyperlink = value)?;
        }
        if let Some(image) = record.image() {
            let latest = image.latest_started();
            let waiting = match image.update() {
                surgeist_html_parser::HtmlParseImageUpdate::Waiting { generation, flags }
                    if generation == latest && generation != 0 =>
                {
                    Some(crate::image::Waiting {
                        invocation: generation,
                        flags,
                    })
                }
                surgeist_html_parser::HtmlParseImageUpdate::Waiting { .. } => {
                    return Err(Error::UnsupportedState { subject: node.id() });
                }
                surgeist_html_parser::HtmlParseImageUpdate::UnavailableActivity { .. } => {
                    return Err(Error::ImageActivityUnavailable { subject: node.id() });
                }
            };
            let parser_inserted = required(
                source.observation(&node.id(), c, |r| r.form_parser_inserted())?,
                node,
            )?;
            let value = Some(crate::image::ImageState {
                native: image.state().clone(),
                dimension_source: source.key(&image.dimension_attribute_source().id(), c)?,
                parser_inserted,
                high_water: latest,
                waiting,
            });
            draft.with_node_fields(key, c.cancel, |fields| fields.image = value)?;
        }
        if let Some(style) = record.style() {
            let document = style.creating_parser_document();
            if style.state().created_by_document_parser() != document.is_some() {
                return Err(Error::UnsupportedState { subject: node.id() });
            }
            let value = document
                .map(|document| match document {
                    HtmlParseMetadataDocument::Owned(node) => source.key(&node.id(), c),
                    HtmlParseMetadataDocument::Captured(_) => {
                        Err(Error::NotCompletedOrdinaryReport)
                    }
                })
                .transpose()?;
            draft.with_node_fields(key, c.cancel, |fields| fields.style_parser_document = value)?;
        }
    }
    c.step(1)?;
    admit(
        draft.counts.base_observations,
        limits.base_observations,
        DomResource::BaseObservations,
    )?;
    Ok(())
}

fn replace_units(
    draft: &mut Draft,
    old: usize,
    new: usize,
    limits: DomLimits,
) -> Result<(), Error> {
    let units = draft
        .units
        .checked_sub(old)
        .and_then(|v| v.checked_add(new))
        .ok_or(Error::RepresentationExhausted)?;
    admit(units, limits.units, DomResource::StringCodeUnits)?;
    draft.units = units;
    Ok(())
}

pub(super) fn allocate(
    source: &Source<'_>,
    node: HtmlNode<'_>,
    draft: &mut Draft,
    document: Key,
    input: crate::DomNativeInputPlatform,
    limits: DomLimits,
    c: &mut Control<'_>,
) -> Result<Key, Error> {
    let mut attributes_custody = None;
    let mut attribute_scratch = 0usize;
    #[cfg(test)]
    let mut written = crate::store::CopyObservation::default();
    c.step(1)?;
    let kind = match node.kind() {
        HtmlNodeKind::Document(_) => return Err(Error::UnsupportedState { subject: node.id() }),
        HtmlNodeKind::Fragment => {
            if let Some(properties) = node.shadow_properties() {
                let mut selected = None;
                for shadow in &source.shadow {
                    c.step(1)?;
                    if shadow.subject == node.id() {
                        if selected.is_some() || shadow.properties != properties {
                            return Err(Error::UnsupportedState { subject: node.id() });
                        }
                        selected = Some(shadow);
                    }
                }
                let shadow =
                    selected.ok_or_else(|| Error::MissingMetadata { subject: node.id() })?;
                DomNodeKind::ShadowRoot(crate::DomShadowRootState::from_parsed(
                    shadow.properties,
                    shadow.keep_registry_null,
                    shadow.declarative,
                    shadow.available_to_internals,
                ))
            } else {
                DomNodeKind::DocumentFragment
            }
        }
        HtmlNodeKind::Text(value) => {
            c.string(value)?;
            DomNodeKind::Text(value.clone())
        }
        HtmlNodeKind::Comment(value) => {
            c.string(value)?;
            DomNodeKind::Comment(value.clone())
        }
        HtmlNodeKind::Doctype(value) => {
            c.string(value.name())?;
            if let Some(value) = value.public_id() {
                c.string(value)?;
            }
            if let Some(value) = value.system_id() {
                c.string(value)?;
            }
            DomNodeKind::DocumentType(DomDocumentType::from_parsed(
                value.name().clone(),
                value.public_id().cloned().unwrap_or_default(),
                value.system_id().cloned().unwrap_or_default(),
            ))
        }
        HtmlNodeKind::ProcessingInstruction(value) => {
            c.string(value.target())?;
            c.string(value.data())?;
            let count = required(
                source.observation(&node.id(), c, |r| r.pi_attribute_count())?,
                node,
            )?;
            let mut pairs = Vec::new();
            c.reserve(&mut pairs, count)?;
            for index in 0..count {
                let (name, value) = required(
                    source.observation(&node.id(), c, |r| r.pi_attribute(index))?,
                    node,
                )?;
                c.string(name)?;
                c.string(value)?;
                pairs.push((name.clone(), value.clone()));
                #[cfg(test)]
                written.pushed::<(DocumentString, DocumentString)>();
            }
            DomNodeKind::ProcessingInstruction(DomProcessingInstruction::from_parsed(
                value.target().clone(),
                value.data().clone(),
                pairs,
            ))
        }
        HtmlNodeKind::Element(element) => {
            c.name(element.name())?;
            for attribute in element.attributes().iter() {
                c.step(1)?;
                c.name(attribute.name())?;
                c.string(attribute.value())?;
            }
            let (custom_state, saved_is) =
                required(source.observation(&node.id(), c, |r| r.creation())?, node)?;
            if let Some(value) = saved_is {
                c.string(value)?;
            }
            let class =
                surgeist_html::bounded::element_classification_with(element.name(), |_| c.step(1))?;
            let common = required(
                source.observation(&node.id(), c, |r| r.common_state())?,
                node,
            )?;
            if common.scope() != class.common_scope() {
                return Err(Error::UnsupportedState { subject: node.id() });
            }
            let nonce = source.observation(&node.id(), c, |r| r.cryptographic_nonce())?;
            if class.has_nonce() != nonce.is_some() {
                return Err(Error::MissingMetadata { subject: node.id() });
            }
            if let Some(value) = nonce {
                c.string(value)?;
            }
            let family = family(source, node, class, input, c)?;
            let name = draft.bind_name(element.name(), None, c.cancel)?;
            let capacity = element.attributes().len();
            let body = if capacity == 0 {
                0
            } else {
                size_of::<payload::Payload<crate::name::AttributeValues>>()
            };
            c.retain(body)?;
            let mut entries = draft.attribute_input(capacity, None, c.cancel, |values| {
                c.reserve(values, capacity)
            })?;
            for attribute in element.attributes().iter() {
                c.step(size_of::<crate::DomAttribute>())?;
                let name = draft.bind_name(attribute.name(), None, c.cancel)?;
                entries.entries.push(crate::DomAttribute::from_parts(
                    name,
                    attribute.value().clone(),
                ));
                #[cfg(test)]
                written.pushed::<crate::DomAttribute>();
            }
            if entries.entries.capacity() != 0 {
                attribute_scratch = entries
                    .entries
                    .capacity()
                    .checked_mul(size_of::<crate::DomAttribute>())
                    .and_then(|bytes| bytes.checked_add(body))
                    .ok_or(Error::RepresentationExhausted)?;
            }
            let attributes = draft.form_attributes(entries, None, c.cancel)?;
            attributes_custody = attributes.custody;
            DomNodeKind::Element(DomElement::from_parsed(
                name,
                attributes.value,
                custom_state,
                saved_is.cloned(),
                nonce.cloned(),
                family,
                common.map(|reference| match reference {}),
            ))
        }
    };
    let key = draft.allocate(kind, document, limits, c.cancel)?;
    // The exact new array/target has transferred into its controlled Slot.
    c.bytes = c
        .bytes
        .checked_sub(attribute_scratch)
        .expect("transferred attribute scratch");
    drop(attributes_custody);
    Ok(key)
}

fn family(
    source: &Source<'_>,
    node: HtmlNode<'_>,
    requirement: Requirement,
    input: crate::DomNativeInputPlatform,
    c: &mut Control<'_>,
) -> Result<NativeFamily, Error> {
    let parser_inserted = if let Requirement::FormAssociated(kind) = requirement {
        if required(source.observation(&node.id(), c, |r| r.form_kind())?, node)? != kind {
            return Err(Error::UnsupportedState { subject: node.id() });
        }
        required(
            source.observation(&node.id(), c, |r| r.form_parser_inserted())?,
            node,
        )?
    } else {
        false
    };
    let control = || crate::control::NativeControlState {
        parser_inserted,
        message: DocumentString::default(),
    };
    Ok(match requirement {
        Requirement::GenericElement
        | Requirement::CommonHtml
        | Requirement::CommonSvg
        | Requirement::CommonMathml => NativeFamily::Common,
        // The sealed offline construction owner exposes no reset/submission,
        // custom-error, reflected-target setter, reaction or activation ingress.
        // Those unchanged initial slots are source-profile invariants; the
        // actual parser flags and associations below are retained separately.
        Requirement::KnownHtml(Element::Form) => {
            NativeFamily::Form(surgeist_html::HtmlFormNativeState::fresh())
        }
        Requirement::KnownHtml(Element::Template) => {
            required(source.observation(&node.id(), c, |r| r.template())?, node)?;
            NativeFamily::Template
        }
        Requirement::KnownHtml(Element::Dialog) => {
            let view = required(source.observation(&node.id(), c, |r| r.dialog())?, node)?;
            c.step(1)?;
            let disposition = view.disposition();
            if let surgeist_html_parser::HtmlParseDialogDisposition::Unavailable(missing) =
                &disposition
            {
                return Err(missing_dialog(missing));
            }
            // Qualify before retaining the shared value; its actual canonical
            // occurrence is admitted by the existing string/kind allocation.
            c.string(view.return_value())?;
            NativeFamily::Dialog(Arc::new(
                crate::dialog::DomDialogState::from_parsed(
                    view.return_value().clone(),
                    disposition,
                )
                .map_err(|missing| missing_dialog(&missing))?,
            ))
        }
        Requirement::KnownHtml(Element::Details) => NativeFamily::Details,
        Requirement::KnownHtml(Element::Meta) => NativeFamily::Meta,
        Requirement::KnownHtml(Element::Base) => {
            match required(source.observation(&node.id(), c, |r| r.base_state())?, node)? {
                HtmlParseBaseState::Unfrozen | HtmlParseBaseState::Frozen(_) => {}
                HtmlParseBaseState::Unavailable { .. } => {
                    return Err(Error::BaseStateUnavailable { subject: node.id() });
                }
            }
            NativeFamily::Base
        }
        Requirement::KnownHtml(Element::Link) => {
            required(source.observation(&node.id(), c, |r| r.link_flags())?, node)?;
            required(
                source.observation(&node.id(), c, |r| r.link_parser_document())?,
                node,
            )?;
            NativeFamily::Link
        }
        Requirement::KnownHtml(Element::A | Element::Area) => {
            required(source.observation(&node.id(), c, |r| r.hyperlink())?, node)?;
            NativeFamily::Hyperlink
        }
        Requirement::FormAssociated(Form::Image) => {
            required(source.observation(&node.id(), c, |r| r.image())?, node)?;
            NativeFamily::Img
        }
        Requirement::KnownHtml(Element::Script) => {
            required(
                source.observation(&node.id(), c, |r| r.script_state())?,
                node,
            )?;
            required(
                source.observation(&node.id(), c, |r| r.script_documents())?,
                node,
            )?;
            NativeFamily::Script
        }
        Requirement::KnownHtml(Element::Style) => {
            let style = required(source.observation(&node.id(), c, |r| r.style())?, node)?;
            let surgeist_html_parser::HtmlParseStyleAssociation::Pending = style.association();
            NativeFamily::Style(Arc::new(crate::DomStyleState::from_parsed(style.state())))
        }
        Requirement::KnownHtml(Element::Slot) => NativeFamily::Slot,
        Requirement::KnownHtml(Element::Picture) => NativeFamily::Picture,
        Requirement::KnownHtml(Element::Source) => NativeFamily::Source,
        // Ordinary offline and its inert owners have no browsing context or
        // navigable. The Frame/Iframe connection guard cannot create a child
        // navigable, and construction exposes neither a resumption nor an
        // activation/navigation ingress. This is the actual selected profile,
        // not a null-content observation used to infer previous history.
        Requirement::HtmlFrame => NativeFamily::Frame,
        Requirement::KnownHtml(Element::Iframe) => {
            NativeFamily::Iframe(crate::element::IframeState::fresh())
        }
        // No parser operation invokes Marquee's start/stop API or animation
        // resources; its initial on/index scalars remain unchanged.
        Requirement::HtmlMarquee => NativeFamily::Marquee(surgeist_html::HtmlMarqueeState::fresh()),
        Requirement::KnownHtml(Element::Canvas) => {
            let HtmlNodeKind::Element(element) = node.kind() else {
                unreachable!("issued element requirement")
            };
            // The producer exposes no context/drawing/bitmap-resource ingress.
            // Reuse the owner interpretation of this actual immutable width /
            // height, without applying an attribute transition or reset hook.
            let mut stopped = None;
            let cancel = c.cancel;
            let value = NativeFamily::canvas_from_attributes(
                element.attributes().as_slice(),
                &mut || match c.step(1) {
                    Ok(()) => Ok(()),
                    Err(error) => {
                        stopped = Some(error);
                        Err(DomError::Cancelled)
                    }
                },
                None,
                cancel,
            );
            // Preserve the adapter's first typed admission failure; the private
            // owner callback's interruption is not a different cancellation.
            if let Some(error) = stopped {
                return Err(error);
            }
            value?
        }
        Requirement::KnownHtml(Element::Optgroup) => NativeFamily::Optgroup,
        Requirement::KnownHtml(Element::Option) => {
            required(
                source.observation(&node.id(), c, |r| r.option_nearest_select())?,
                node,
            )?;
            NativeFamily::Option(crate::DomOptionState {
                flags: required(
                    source.observation(&node.id(), c, |r| r.option_flags().copied())?,
                    node,
                )?,
            })
        }
        Requirement::KnownHtml(Element::Selectedcontent) => {
            NativeFamily::Selectedcontent(required(
                source.observation(&node.id(), c, |r| r.selectedcontent_disabled())?,
                node,
            )?)
        }
        Requirement::FormAssociated(Form::Button) => {
            NativeFamily::Button(crate::DomButtonState { control: control() })
        }
        Requirement::FormAssociated(Form::Fieldset) => {
            NativeFamily::Fieldset(crate::DomFieldsetState { control: control() })
        }
        Requirement::FormAssociated(Form::Output) => {
            let surgeist_html_parser::HtmlParseOutputOverride::Null = required(
                source.observation(&node.id(), c, |r| r.output_default_override())?,
                node,
            )?;
            NativeFamily::Output(Arc::new(crate::DomOutputState {
                control: control(),
                default_override: None,
            }))
        }
        Requirement::FormAssociated(Form::Select) => {
            NativeFamily::Select(Arc::new(crate::DomSelectState {
                control: control(),
                flags: required(
                    source.observation(&node.id(), c, |r| r.select_flags().copied())?,
                    node,
                )?,
            }))
        }
        Requirement::FormAssociated(Form::Textarea) => {
            let raw = required(
                source.observation(&node.id(), c, |r| r.textarea_raw_value())?,
                node,
            )?;
            c.string(raw)?;
            NativeFamily::Textarea(Arc::new(crate::DomTextareaState {
                control: control(),
                flags: required(
                    source.observation(&node.id(), c, |r| r.textarea_flags().copied())?,
                    node,
                )?,
                raw: raw.clone(),
                selection: HtmlInputSelection::initial(input.none_direction()),
            }))
        }
        Requirement::FormAssociated(Form::Input) => {
            let flags = required(
                source.observation(&node.id(), c, |r| r.input_flags().copied())?,
                node,
            )?;
            let raw = required(
                source.observation(&node.id(), c, |r| r.input_raw_value())?,
                node,
            )?;
            c.string(raw)?;
            let cursor = required(
                source.observation(&node.id(), c, |r| r.input_cursor())?,
                node,
            )?;
            let files = if flags.input_type() == surgeist_html::HtmlInputType::File {
                required(
                    source.observation(&node.id(), c, |r| r.input_file_selection())?,
                    node,
                )?
            } else {
                // No selected-File/resource ingress exists in this closed
                // producer; entering/leaving File cannot install resources.
                surgeist_html::HtmlInputFileSelection::Empty
            };
            let coordinate = if flags.input_type() == surgeist_html::HtmlInputType::Image {
                required(
                    source.observation(&node.id(), c, |r| r.input_image_coordinate())?,
                    node,
                )?
            } else {
                // The producer has no pointer/coordinate setter. A prior Image
                // history still retains its actual initial (0,0) coordinate.
                surgeist_html::HtmlInputImageCoordinate::initial()
            };
            let image = match source.observation(&node.id(), c, |r| r.input_image_source())? {
                None => HtmlInputImageHistory::NeverEntered,
                Some(surgeist_html_parser::HtmlInputImageSource::Absent) => {
                    HtmlInputImageHistory::Last {
                        source: None,
                        coordinate,
                    }
                }
                Some(surgeist_html_parser::HtmlInputImageSource::Present(value)) => {
                    c.string(value)?;
                    HtmlInputImageHistory::Last {
                        source: Some(value.clone()),
                        coordinate,
                    }
                }
            };
            // The parser owns the exact cursor but no selected range/direction
            // API. Installation establishes the initial direction from the
            // explicitly supplied target platform; it does not import a range.
            let direction = HtmlInputSelection::initial(input.none_direction()).direction();
            NativeFamily::Input(Arc::new(crate::DomInputState {
                control: control(),
                flags,
                current: raw.clone(),
                selection: HtmlInputSelection::cursor(cursor, direction),
                files,
                image,
                email_count: source.observation(&node.id(), c, |r| r.input_email_value_count())?,
            }))
        }
        Requirement::SvgScript => {
            let processed = required(
                source.observation(&node.id(), c, |r| r.svg_script_processed())?,
                node,
            )?;
            let origin = required(
                source.observation(&node.id(), c, |r| r.svg_script_origin())?,
                node,
            )?;
            let flags = surgeist_html::SvgScriptFlags::from_retained(processed);
            NativeFamily::SvgScript(crate::svg_script::SvgScriptState::from_parsed(
                flags,
                match origin {
                    surgeist_html_parser::HtmlParseSvgScriptOrigin::Parser => {
                        crate::DomSvgScriptOrigin::Parser
                    }
                    surgeist_html_parser::HtmlParseSvgScriptOrigin::Clone => {
                        crate::DomSvgScriptOrigin::Clone
                    }
                },
                source
                    .svg_qualification
                    .as_ref()
                    .ok_or(Error::CurrentPolicyUnavailable)?,
            ))
        }
        // Classification alone cannot establish a remaining family's actual
        // state or processing disposition. These are outstanding installation
        // contributions, rather than another native initialization cause.
        class => {
            return Err(Error::FamilyStateUnavailable {
                subject: node.id(),
                class,
            });
        }
    })
}
