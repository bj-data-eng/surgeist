//! Completed construction enters the existing canonical Draft once.
use super::*;
use crate::parsed_installation::DomParsedHtmlCapturedBinding;
use crate::{
    DomParsedDocumentEnvironment, DomParsedHtmlBinding, DomParsedHtmlInstallError as Error,
    DomParsedHtmlInstallLimits as Limits, DomParsedHtmlInstallResource as Resource,
    DomParsedHtmlInstallation,
};
use surgeist_html::{HtmlDocument, HtmlFragment, HtmlNode, HtmlNodeId, HtmlNodeKind};
use surgeist_html_parser::{
    HtmlParseBaseState, HtmlParseCompletion, HtmlParseDestination, HtmlParseMetadata,
    HtmlParseMetadataDocument, HtmlParseMetadataStep, HtmlParseReadiness, HtmlParseReport,
};
mod state;
#[cfg(test)]
mod tests;

struct Control<'a> {
    limits: Limits,
    work: u64,
    bytes: usize,
    peak_bytes: usize,
    cancel: &'a AtomicBool,
}
impl Control<'_> {
    fn step(&mut self, count: usize) -> Result<(), Error> {
        self.check()?;
        self.work = self
            .work
            .checked_add(u64::try_from(count).map_err(|_| Error::RepresentationExhausted)?)
            .ok_or(Error::RepresentationExhausted)?;
        self.limit(Resource::Work, self.work, self.limits.work)
    }
    fn check(&self) -> Result<(), Error> {
        if self.cancel.load(Ordering::Acquire) {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
    fn limit(&self, resource: Resource, actual: u64, limit: u64) -> Result<(), Error> {
        if actual > limit {
            Err(Error::ResourceLimit { resource, limit })
        } else {
            Ok(())
        }
    }
    fn retain(&mut self, bytes: usize) -> Result<(), Error> {
        self.check()?;
        let next = self
            .bytes
            .checked_add(bytes)
            .ok_or(Error::RepresentationExhausted)?;
        self.limit(
            Resource::ScratchBytes,
            next as u64,
            self.limits.scratch_bytes as u64,
        )?;
        self.bytes = next;
        self.peak_bytes = self.peak_bytes.max(next);
        Ok(())
    }
    fn reserve<T>(&mut self, values: &mut Vec<T>, additional: usize) -> Result<(), Error> {
        self.check()?;
        let need = values
            .len()
            .checked_add(additional)
            .ok_or(Error::RepresentationExhausted)?;
        if need <= values.capacity() {
            return Ok(());
        }
        let old = values.capacity();
        // Geometric requests do not depend on the selected quota. The existing
        // allocation stays live while a separate replacement is formed, so the
        // whole new backing, not merely its net increase, needs admission.
        let chosen = need.max(old.checked_mul(2).ok_or(Error::RepresentationExhausted)?);
        let chosen_bytes = chosen
            .checked_mul(size_of::<T>())
            .ok_or(Error::RepresentationExhausted)?;
        self.limit(
            Resource::ScratchBytes,
            self.bytes
                .checked_add(chosen_bytes)
                .ok_or(Error::RepresentationExhausted)? as u64,
            self.limits.scratch_bytes as u64,
        )?;
        self.step(
            values
                .len()
                .checked_mul(size_of::<T>())
                .ok_or(Error::RepresentationExhausted)?,
        )?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(chosen)
            .map_err(|_| Error::AllocationFailed)?;
        let replacement_bytes = replacement
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(Error::RepresentationExhausted)?;
        let peak = self
            .bytes
            .checked_add(replacement_bytes)
            .ok_or(Error::RepresentationExhausted)?;
        self.limit(
            Resource::ScratchBytes,
            peak as u64,
            self.limits.scratch_bytes as u64,
        )?;
        let next = self
            .bytes
            .checked_sub(
                old.checked_mul(size_of::<T>())
                    .ok_or(Error::RepresentationExhausted)?,
            )
            .and_then(|bytes| bytes.checked_add(replacement_bytes))
            .ok_or(Error::RepresentationExhausted)?;
        self.peak_bytes = self.peak_bytes.max(peak);
        self.check()?;
        // Copy work was prepaid before allocating. No fallible operation or
        // callback remains between moving the values and releasing the old bank.
        replacement.append(values);
        *values = replacement;
        self.bytes = next;
        Ok(())
    }
    fn string(&mut self, value: &DocumentString) -> Result<(), Error> {
        self.step(value.len())
    }
    fn name(&mut self, value: &crate::QualifiedName) -> Result<(), Error> {
        self.step(crate::names::name_units(value).ok_or(Error::RepresentationExhausted)?)
    }
}
impl From<DomError> for Error {
    fn from(error: DomError) -> Self {
        if error == DomError::Cancelled {
            Self::Cancelled
        } else {
            Self::Target(error)
        }
    }
}

struct Row<'a> {
    source: HtmlNode<'a>,
    key: Option<Key>,
}
/// Qualification of the actual sealed completed construction producer.
/// It is not derivable from a node's origin, browsing-context or scripting flag.
/// Ordinary offline collection or completed Disabled/Null fragment admission
/// establishes the producer domain; destination Documents are independently
/// qualified before any retained SVG state is constructed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ParsedSvgQualification {
    _sealed: (),
}
struct PreparedInstallation {
    prepared: Prepared,
    report: Arc<HtmlParseReport>,
    source_root: HtmlNodeId,
    document: DocumentId,
    root: NodeId,
    context_basis: Option<crate::DomDocumentStamp>,
    captured_bindings: Vec<DomParsedHtmlCapturedBinding>,
    bindings: Vec<DomParsedHtmlBinding>,
    #[cfg(test)]
    admitted_work: u64,
    #[cfg(test)]
    admitted_scratch_bytes: usize,
}
enum Arena<'a> {
    Document(&'a HtmlDocument),
    Fragment(&'a HtmlFragment),
}
impl<'a> Arena<'a> {
    fn root(&self) -> HtmlNode<'a> {
        match self {
            Self::Document(value) => value.root(),
            Self::Fragment(value) => value.root(),
        }
    }
    fn node(&self, id: &HtmlNodeId) -> Result<HtmlNode<'a>, surgeist_html::ForeignHtmlResult> {
        match self {
            Self::Document(value) => value.node(id),
            Self::Fragment(value) => value.node(id),
        }
    }
}
struct Source<'a> {
    arena: Arena<'a>,
    rows: Vec<Row<'a>>,
    metadata: Vec<HtmlParseMetadata<'a>>,
    capture: Option<&'a crate::DomHtmlCapture>,
    captured_bindings: Vec<DomParsedHtmlCapturedBinding>,
    bootstrap: [Option<HtmlNodeId>; 2],
    shadow: Vec<ParsedShadow>,
    pub(super) svg_qualification: Option<ParsedSvgQualification>,
}
struct ParsedShadow {
    subject: HtmlNodeId,
    properties: crate::HtmlShadowRootProperties,
    keep_registry_null: bool,
    declarative: bool,
    available_to_internals: bool,
}
impl<'a> Source<'a> {
    fn index(&self, id: &HtmlNodeId, c: &mut Control<'_>) -> Result<Option<usize>, Error> {
        for (i, row) in self.rows.iter().enumerate() {
            c.step(1)?;
            if row.source.id() == *id {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }
    fn add(&mut self, id: HtmlNodeId, c: &mut Control<'_>) -> Result<(), Error> {
        if self.index(&id, c)?.is_some() {
            return Ok(());
        }
        c.limit(
            Resource::MappedNodes,
            self.rows
                .len()
                .checked_add(self.captured_bindings.len())
                .ok_or(Error::RepresentationExhausted)?
                .checked_add(1)
                .ok_or(Error::RepresentationExhausted)? as u64,
            c.limits.mapped_nodes as u64,
        )?;
        let node = self
            .arena
            .node(&id)
            .map_err(|error| Error::SourceLookup { subject: id, error })?;
        c.reserve(&mut self.rows, 1)?;
        self.rows.push(Row {
            source: node,
            key: None,
        });
        Ok(())
    }
    fn add_document(
        &mut self,
        value: HtmlParseMetadataDocument<'a>,
        c: &mut Control<'_>,
    ) -> Result<(), Error> {
        self.add_document_ref(&value, c)
    }
    fn add_document_ref(
        &mut self,
        value: &HtmlParseMetadataDocument<'a>,
        c: &mut Control<'_>,
    ) -> Result<(), Error> {
        match value {
            HtmlParseMetadataDocument::Owned(node) => self.add(node.id(), c),
            HtmlParseMetadataDocument::Captured(node) => {
                self.add_captured(node.id(), c).map(|_| ())
            }
        }
    }
    fn key(&self, id: &HtmlNodeId, c: &mut Control<'_>) -> Result<Key, Error> {
        self.index(id, c)?
            .and_then(|i| self.rows[i].key)
            .ok_or_else(|| Error::MissingMetadata {
                subject: id.clone(),
            })
    }
    fn document_key(
        &self,
        value: HtmlParseMetadataDocument<'_>,
        c: &mut Control<'_>,
    ) -> Result<Key, Error> {
        self.document_ref_key(&value, c)
    }
    fn document_ref_key(
        &self,
        value: &HtmlParseMetadataDocument<'_>,
        c: &mut Control<'_>,
    ) -> Result<Key, Error> {
        match value {
            HtmlParseMetadataDocument::Owned(node) => self.key(&node.id(), c),
            HtmlParseMetadataDocument::Captured(node) => self.captured_key(&node.id(), c),
        }
    }
    fn add_captured(
        &mut self,
        subject: surgeist_html_semantics::HtmlCaptureSubjectId,
        c: &mut Control<'_>,
    ) -> Result<Key, Error> {
        for binding in &self.captured_bindings {
            c.step(1)?;
            if binding.subject == subject {
                return Ok(Key::Existing(binding.node.slot));
            }
        }
        let capture = self.capture.ok_or(Error::NotCompletedOrdinaryReport)?;
        c.step(1)?;
        let node = capture.node(&subject).map_err(|_| Error::ForeignContext)?;
        c.limit(
            Resource::MappedNodes,
            self.rows
                .len()
                .checked_add(self.captured_bindings.len())
                .and_then(|n| n.checked_add(1))
                .ok_or(Error::RepresentationExhausted)? as u64,
            c.limits.mapped_nodes as u64,
        )?;
        c.reserve(&mut self.captured_bindings, 1)?;
        self.captured_bindings
            .push(DomParsedHtmlCapturedBinding { subject, node });
        let captured = capture
            .capture()
            .node(self.captured_bindings.last().expect("just added").subject())
            .map_err(|_| Error::ForeignContext)?;
        let document = captured.node_document();
        if document.subject() != self.captured_bindings.last().expect("just added").subject() {
            self.add_captured(document.subject().clone(), c)?;
        }
        Ok(Key::Existing(node.slot))
    }
    fn captured_key(
        &self,
        subject: &surgeist_html_semantics::HtmlCaptureSubjectId,
        c: &mut Control<'_>,
    ) -> Result<Key, Error> {
        for binding in &self.captured_bindings {
            c.step(1)?;
            if binding.subject == *subject {
                return Ok(Key::Existing(binding.node.slot));
            }
        }
        Err(Error::CapturedStateUnavailable)
    }
    fn bootstrap_only(&self, id: &HtmlNodeId, c: &mut Control<'_>) -> Result<bool, Error> {
        for value in self.bootstrap.iter().flatten() {
            c.step(1)?;
            if value == id {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn observation<T>(
        &self,
        id: &HtmlNodeId,
        c: &mut Control<'_>,
        read: impl Fn(&HtmlParseMetadata<'a>) -> Option<T>,
    ) -> Result<Option<T>, Error> {
        let mut observed = None;
        for record in &self.metadata {
            c.step(1)?;
            if record.node().id() == *id
                && let Some(value) = read(record)
            {
                if observed.is_some() {
                    return Err(Error::UnsupportedState {
                        subject: id.clone(),
                    });
                }
                observed = Some(value);
            }
        }
        Ok(observed)
    }
}

/// Validate only actual historical Captured parse Documents against the current
/// canonical selection. Owned auxiliary/inert Documents are source constructions,
/// not aliases of the target; their original producer qualification stays intact.
fn qualify_captured_hyperlinks(
    source: &Source<'_>,
    draft: &Draft,
    c: &mut Control<'_>,
) -> Result<(), Error> {
    for record in &source.metadata {
        c.step(1)?;
        let Some(link) = record.hyperlink() else {
            continue;
        };
        c.step(1)?;
        let HtmlParseMetadataDocument::Captured(document) = link.document() else {
            continue;
        };
        let surgeist_html_parser::HtmlParseHyperlinkState::Observed(snapshot) = link.state() else {
            // Actual absent href needs no parse evidence. The family transfer
            // separately preserves the original typed unavailable refusal.
            continue;
        };
        let key = source.captured_key(&document.id(), c)?;
        let mut stopped = None;
        let current = draft.encoding_url_environment_with(key, &mut || match c.step(1) {
            Ok(()) => Ok(()),
            Err(error) => {
                stopped = Some(error);
                Err(DomError::Cancelled)
            }
        });
        if let Some(error) = stopped {
            return Err(error);
        }
        let (base, encoding, revision) = current.map_err(|_| Error::CurrentPolicyUnavailable)?;
        c.step(1)?;
        if snapshot.base() != base
            || snapshot.encoding() != encoding
            || snapshot.revision() != revision
        {
            return Err(Error::CurrentPolicyUnavailable);
        }
        let facts = draft
            .node(key)?
            .encoding_url_document
            .as_ref()
            .and_then(|document| document.facts.as_ref())
            .ok_or(Error::CurrentPolicyUnavailable)?;
        let outcome = facts
            .lookup_units_with(
                snapshot.raw().code_units(),
                base,
                encoding,
                revision,
                || c.step(1),
            )
            .map_err(|error| match error {
                surgeist_html_semantics::HtmlEncodingUrlLookupError::Facts(_) => {
                    Error::CurrentPolicyUnavailable
                }
                surgeist_html_semantics::HtmlEncodingUrlLookupError::Interrupted(error) => error,
            })?;
        c.step(1)?;
        if outcome != snapshot.outcome() {
            return Err(Error::CurrentPolicyUnavailable);
        }
    }
    Ok(())
}

fn collect<'a>(report: &'a HtmlParseReport, c: &mut Control<'_>) -> Result<Source<'a>, Error> {
    let document = report.document().ok_or(Error::NotCompletedOrdinaryReport)?;
    let state = report
        .document_state()
        .ok_or(Error::NotCompletedOrdinaryReport)?;
    if state.completion() != HtmlParseCompletion::Complete
        || state.readiness() != HtmlParseReadiness::Complete
        || !state.ready_for_post_load_tasks()
        || report.context().is_some()
        || report.effects().is_some()
        || report.allow_declarative_shadow_roots()
        || report.effective_scripting_mode() != surgeist_html::HtmlParserScriptingMode::Disabled
        || report.image_support() != surgeist_html_semantics::HtmlImageSupport::Disabled
    {
        return Err(Error::NotCompletedOrdinaryReport);
    }
    let mut source = Source {
        arena: Arena::Document(document),
        rows: Vec::new(),
        metadata: Vec::new(),
        svg_qualification: Some(ParsedSvgQualification { _sealed: () }),
        capture: None,
        captured_bindings: Vec::new(),
        bootstrap: [None, None],
        shadow: Vec::new(),
    };
    source.add(document.root().id(), c)?;
    collect_rows(report, source, c)
}

fn collect_fragment<'a>(
    report: &'a HtmlParseReport,
    capture: &'a crate::DomHtmlCapture,
    c: &mut Control<'_>,
) -> Result<Source<'a>, Error> {
    use surgeist_html_parser::{HtmlParseEffect, HtmlParseEffectHost, HtmlParseRegistry};
    let fragment = report.fragment().ok_or(Error::NotCompletedFragmentReport)?;
    let context = report.context().ok_or(Error::NotCompletedFragmentReport)?;
    let parser = report
        .fragment_parser_document()
        .ok_or(Error::NotCompletedFragmentReport)?;
    let state = parser.completion();
    c.step(1)?;
    if state.completion() != HtmlParseCompletion::Complete
        || state.readiness() != HtmlParseReadiness::Complete
        || !state.ready_for_post_load_tasks()
        || report.document().is_some()
    {
        return Err(Error::NotCompletedFragmentReport);
    }
    if context.capture().identity() != capture.capture().identity() {
        return Err(Error::ForeignContext);
    }
    // The unused captured Base bank remains charged historical request input.
    // Actual influencing URL snapshots are validated against canonical owners.
    if report.effective_scripting_mode() != surgeist_html::HtmlParserScriptingMode::Disabled
        || !matches!(
            context.root_registry(),
            surgeist_html_semantics::HtmlCapturedRegistry::Null
        )
    {
        return Err(Error::CurrentPolicyUnavailable);
    }
    let mut source = Source {
        arena: Arena::Fragment(fragment),
        rows: Vec::new(),
        metadata: Vec::new(),
        capture: Some(capture),
        captured_bindings: Vec::new(),
        bootstrap: [
            Some(parser.document().id()),
            parser.document().children().next(),
        ],
        shadow: Vec::new(),
        svg_qualification: None,
    };
    source.add(fragment.root().id(), c)?;
    let target = match context.target() {
        surgeist_html_semantics::HtmlFragmentTarget::Element(value) => value.subject(),
        surgeist_html_semantics::HtmlFragmentTarget::HostedFragment(value) => value.subject(),
    };
    for subject in [
        target,
        context.context_element().subject(),
        context.target_document().subject(),
        context.context_document().subject(),
    ] {
        source.add_captured(subject.clone(), c)?;
    }
    if let Some(form) = context.nearest_form() {
        source.add_captured(form.subject().clone(), c)?;
    }
    let effects = report.effects().ok_or(Error::NotCompletedFragmentReport)?;
    for index in 0..effects.registry_slots() {
        c.step(1)?;
        if let Some(binding) = effects.registry(index) {
            if !matches!(binding.registry(), HtmlParseRegistry::Null) {
                return Err(Error::CurrentPolicyUnavailable);
            }
            if !source.bootstrap_only(&binding.node().id(), c)? {
                source.add(binding.node().id(), c)?;
            }
        }
    }
    for index in 0..effects.len() {
        c.step(1)?;
        match effects
            .effect(index)
            .ok_or(Error::NotCompletedFragmentReport)?
        {
            HtmlParseEffect::TemplateCache { document, inert } => {
                source.add_captured(document.id(), c)?;
                source.add(inert.id(), c)?;
            }
            HtmlParseEffect::AttachShadow {
                host,
                template,
                contents,
                properties,
                keep_registry_null,
                declarative,
                available_to_internals,
                ..
            } => {
                match host {
                    HtmlParseEffectHost::Owned(node) => source.add(node.id(), c)?,
                    HtmlParseEffectHost::Captured(node) => {
                        source.add_captured(node.subject().clone(), c)?;
                    }
                }
                source.add(template.id(), c)?;
                source.add(contents.id(), c)?;
                c.reserve(&mut source.shadow, 1)?;
                source.shadow.push(ParsedShadow {
                    subject: contents.id(),
                    properties: *properties,
                    keep_registry_null,
                    declarative,
                    available_to_internals,
                });
            }
        }
    }
    collect_rows(report, source, c)
}

fn collect_rows<'a>(
    report: &'a HtmlParseReport,
    mut source: Source<'a>,
    c: &mut Control<'_>,
) -> Result<Source<'a>, Error> {
    let mut cursor = report.metadata();
    loop {
        // Skipped slots and the terminating observation are actual work too.
        c.step(1)?;
        let record = match cursor.advance() {
            HtmlParseMetadataStep::Record(record) => record,
            HtmlParseMetadataStep::Skipped => continue,
            HtmlParseMetadataStep::Complete => break,
        };
        // The auxiliary parser Document/html are installed only if an actual
        // retained semantic reference reaches them, not merely as bootstrap.
        if source.bootstrap_only(&record.node().id(), c)? {
            c.reserve(&mut source.metadata, 1)?;
            source.metadata.push(record);
            continue;
        }
        source.add(record.node().id(), c)?;
        if let Some(value) = record.node_document() {
            source.add_document(value, c)?;
        }
        if let Some((parser, preparation)) = record.script_documents() {
            if let Some(value) = parser {
                source.add_document(value, c)?;
            }
            if let Some(value) = preparation {
                source.add_document(value, c)?;
            }
        }
        if let Some(template) = record.template() {
            source.add(template.contents().id(), c)?;
            for value in [template.target(), template.start(), template.end()]
                .into_iter()
                .flatten()
            {
                source.add(value.id(), c)?;
            }
        }
        if let Some((_, _, Some(inert))) = record.document_facts() {
            source.add(inert.id(), c)?;
        }
        if let Some(Some(value)) = record.option_nearest_select() {
            source.add(value.id(), c)?;
        }
        if let Some(Some(value)) = record.form_owner() {
            source.add(value.id(), c)?;
        }
        if let Some(base) = record.base_state() {
            match base {
                HtmlParseBaseState::Unfrozen => {}
                HtmlParseBaseState::Frozen(base) => source.add(base.document().id(), c)?,
                HtmlParseBaseState::Unavailable { previous, .. } => {
                    // Retain a real historical reference without certifying it
                    // as current. Family admission rejects this unavailable state.
                    if let Some(base) = previous {
                        source.add(base.document().id(), c)?;
                    }
                }
            }
        }
        if let Some(Some(value)) = record.document_first_base() {
            source.add(value.id(), c)?;
        }
        if let Some(Some(value)) = record.link_parser_document() {
            source.add_document(value, c)?;
        }
        if let Some(hyperlink) = record.hyperlink() {
            source.add_document_ref(hyperlink.document(), c)?;
        }
        if let Some(image) = record.image() {
            source.add(image.dimension_attribute_source().id(), c)?;
        }
        if let Some(style) = record.style()
            && let Some(document) = style.creating_parser_document()
        {
            source.add_document_ref(document, c)?;
        }
        c.reserve(&mut source.metadata, 1)?;
        source.metadata.push(record);
    }
    let associations = report.associations();
    for index in 0..associations.len() {
        c.step(1)?;
        let row = associations
            .row(index)
            .expect("qualified association index");
        match row.destination() {
            HtmlParseDestination::Node(node)
            | HtmlParseDestination::Field { node, .. }
            | HtmlParseDestination::Raw { node, .. } => {
                if !source.bootstrap_only(&node.id(), c)? {
                    source.add(node.id(), c)?;
                }
            }
            HtmlParseDestination::Lexical { .. }
            | HtmlParseDestination::Omission(_)
            | HtmlParseDestination::LexicalEnd
            | HtmlParseDestination::Cause(_) => {}
        }
    }
    // Grow only from actual retained subjects and current relationships. Retired
    // lexical/cause subjects are preserved by the report, never invented NodeIds.
    let mut next = 0;
    while next < source.rows.len() {
        c.step(1)?;
        let node = source.rows[next].source;
        if let Some(parent) = node.parent() {
            source.add(parent, c)?;
        }
        for child in node.children() {
            c.step(1)?;
            source.add(child, c)?;
        }
        if let Some(contents) = node.template_contents() {
            source.add(contents, c)?;
        }
        if node.declarative_shadow_root().is_some() && source.capture.is_none() {
            return Err(Error::NotCompletedOrdinaryReport);
        }
        next += 1;
    }
    Ok(source)
}

fn apply_fragment_effects(
    report: &HtmlParseReport,
    source: &Source<'_>,
    draft: &mut Draft,
    limits: DomLimits,
    c: &mut Control<'_>,
) -> Result<(), Error> {
    use surgeist_html_parser::{HtmlParseEffect, HtmlParseEffectHost};
    let effects = report.effects().ok_or(Error::NotCompletedFragmentReport)?;
    for index in 0..effects.len() {
        c.step(1)?;
        match effects
            .effect(index)
            .ok_or(Error::NotCompletedFragmentReport)?
        {
            HtmlParseEffect::TemplateCache { document, inert } => {
                let document = source.captured_key(&document.id(), c)?;
                let inert = source.key(&inert.id(), c)?;
                if !matches!(draft.node(document)?.kind, DomNodeKind::Document(_))
                    || draft.node(document)?.template_document
                        != Some(crate::template::TemplateDocument::Ordinary { inert: None })
                    || draft.node(inert)?.template_document
                        != Some(crate::template::TemplateDocument::InertTemplate)
                {
                    return Err(Error::CapturedStateUnavailable);
                }
                draft.node_mut(document).template_document =
                    Some(crate::template::TemplateDocument::Ordinary { inert: Some(inert) });
            }
            HtmlParseEffect::AttachShadow {
                host,
                template,
                contents,
                ..
            } => {
                let host = match host {
                    HtmlParseEffectHost::Owned(node) => source.key(&node.id(), c)?,
                    HtmlParseEffectHost::Captured(value) => {
                        source.captured_key(value.subject(), c)?
                    }
                };
                let template_key = source.key(&template.id(), c)?;
                let contents_key = source.key(&contents.id(), c)?;
                if template.template_contents_kind()
                    != Some(surgeist_html::HtmlTemplateContentsKind::ShadowReference)
                    || draft.node(template_key)?.template_contents != Some(contents_key)
                {
                    return Err(Error::UnsupportedState {
                        subject: template.id(),
                    });
                }
                draft.attach_parsed_shadow(host, contents_key, limits, c.cancel)?;
            }
        }
    }
    Ok(())
}

fn qualify_svg_destination(metadata: &DomDocumentMetadata) -> Result<(), Error> {
    let environment = metadata.document_environment();
    if metadata.custom_element_registry() != crate::DomRegistryBinding::Null
        || environment.browsing_context() != surgeist_html::HtmlBrowsingContextPresence::Absent
        || environment.has_ever_had_navigable()
    {
        return Err(Error::CurrentPolicyUnavailable);
    }
    Ok(())
}

impl Dom {
    /// Transfers a completed ordinary offline report to a new canonical
    /// Document. It materializes retained state directly; no fresh-native
    /// attribute, insertion, cloning, resource or callback hook is replayed.
    pub fn install_parsed_html_document(
        &mut self,
        basis: DomStamp,
        report: Arc<HtmlParseReport>,
        environment: DomParsedDocumentEnvironment,
        limits: Limits,
        cancelled: &AtomicBool,
    ) -> Result<DomParsedHtmlInstallation, Error> {
        self.require_source_idle()?;
        let prepared =
            self.prepare_parsed_html_installation(basis, report, environment, limits, cancelled)?;
        self.publish_parsed_html_installation(prepared, cancelled)
    }

    /// Materializes a completed contextual report as a detached fragment in
    /// its actual target Document, applying only its ordered retained effects.
    /// The original capture is borrowed for exact identity/current-basis joins;
    /// its historical canonical snapshot is not retained by the result.
    ///
    /// This entry admits the current canonical closed policy domain. Actual
    /// URL observations must match their canonical-bound authority. An unused
    /// captured Base bank remains fully charged source history and grants no
    /// authority. Reached missing family observations remain explicit failures.
    /// No CSSOM, resource, script or callback work executes.
    pub fn install_parsed_html_fragment(
        &mut self,
        report: Arc<HtmlParseReport>,
        capture: &crate::DomHtmlCapture,
        limits: Limits,
        cancelled: &AtomicBool,
    ) -> Result<DomParsedHtmlInstallation, Error> {
        self.require_source_idle()?;
        let prepared = self.prepare_parsed_html(
            capture.basis().store(),
            report,
            None,
            Some(capture),
            limits,
            cancelled,
        )?;
        self.publish_parsed_html_installation(prepared, cancelled)
    }

    fn prepare_parsed_html_installation(
        &self,
        basis: DomStamp,
        report: Arc<HtmlParseReport>,
        environment: DomParsedDocumentEnvironment,
        limits: Limits,
        cancelled: &AtomicBool,
    ) -> Result<PreparedInstallation, Error> {
        self.prepare_parsed_html(basis, report, Some(environment), None, limits, cancelled)
    }

    fn prepare_parsed_html(
        &self,
        basis: DomStamp,
        report: Arc<HtmlParseReport>,
        environment: Option<DomParsedDocumentEnvironment>,
        capture: Option<&crate::DomHtmlCapture>,
        limits: Limits,
        cancelled: &AtomicBool,
    ) -> Result<PreparedInstallation, Error> {
        let mut c = Control {
            limits,
            work: 0,
            bytes: 0,
            peak_bytes: 0,
            cancel: cancelled,
        };
        c.check()?;
        if basis.owner() != self.owner {
            return Err(Error::Target(DomError::ForeignOwner));
        }
        let actual = self.snapshot().stamp();
        if basis != actual {
            return Err(Error::StaleDestination {
                expected: basis,
                actual,
            });
        }
        if let Some(capture) = capture {
            c.step(1)?;
            let context = report.context().ok_or(Error::NotCompletedFragmentReport)?;
            if context.capture().identity() != capture.capture().identity() {
                return Err(Error::ForeignContext);
            }
        }
        state::admit_history(&report, environment.as_ref(), &mut c)?;
        let mut source = match capture {
            Some(capture) => collect_fragment(&report, capture, &mut c)?,
            None => collect(&report, &mut c)?,
        };
        if capture.is_some() {
            self.qualify_contextual_policy(&source, &mut c)?;
            // collect_fragment qualified the original completed descriptor,
            // exact capture and sealed Disabled/Null construction domain.
            // Its controller has no acquisition, execution or notification
            // recipients. Current captured policy is a separate canonical join;
            // new auxiliary/inert Documents are checked below before use.
            source.svg_qualification = Some(ParsedSvgQualification { _sealed: () });
        }
        let root_id = source.arena.root().id();
        let mut draft = Draft::new(self, Arc::new(BatchIdentity), cancelled)?;
        if capture.is_some() {
            qualify_captured_hyperlinks(&source, &draft, &mut c)?;
        }
        // Documents precede all members, including actual separately retained
        // inert owners. No native head/body skeleton or arbitrary target exists.
        for i in 0..source.rows.len() {
            c.step(1)?;
            let node = source.rows[i].source;
            let HtmlNodeKind::Document(mode) = node.kind() else {
                continue;
            };
            let design_mode_enabled = node
                .document_design_mode_enabled()
                .ok_or_else(|| Error::MissingMetadata { subject: node.id() })?;
            let (class, inert, _) = source
                .observation(&node.id(), &mut c, |r| r.document_facts())?
                .ok_or_else(|| Error::MissingMetadata { subject: node.id() })?;
            let class = match class {
                surgeist_html_semantics::HtmlDocumentClass::Html => crate::DomDocumentClass::Html,
                surgeist_html_semantics::HtmlDocumentClass::NonHtml => crate::DomDocumentClass::Xml,
                surgeist_html_semantics::HtmlDocumentClass::Unavailable => {
                    return Err(Error::MissingMetadata { subject: node.id() });
                }
            };
            let mut metadata = if let Some(environment) = environment.as_ref() {
                if node.id() == root_id {
                    if inert {
                        return Err(Error::UnsupportedState { subject: node.id() });
                    }
                    DomDocumentMetadata::from_parsed(class, *mode, environment, design_mode_enabled)
                } else {
                    if !inert || *mode != surgeist_html::HtmlDocumentMode::NoQuirks {
                        return Err(Error::UnsupportedState { subject: node.id() });
                    }
                    DomDocumentMetadata::parsed_template_owner(
                        class,
                        environment.input,
                        design_mode_enabled,
                    )
                }
            } else if report
                .fragment_parser_document()
                .is_some_and(|descriptor| descriptor.document().id() == node.id())
            {
                DomDocumentMetadata::from_fragment_parser(
                    &report
                        .fragment_parser_document()
                        .expect("selected descriptor"),
                    self.document_environment.clone(),
                    design_mode_enabled,
                )
            } else {
                if !inert || *mode != surgeist_html::HtmlDocumentMode::NoQuirks {
                    return Err(Error::UnsupportedState { subject: node.id() });
                }
                DomDocumentMetadata::parsed_template_owner(
                    class,
                    self.document_environment.input_platform(),
                    design_mode_enabled,
                )
            };
            state::transfer_document_meta(&source, node, &mut metadata, &mut c)?;
            if capture.is_some() {
                qualify_svg_destination(&metadata)?;
            }
            let interaction = state::document_interaction(&source, node, &mut c)?;
            let own = Key::Created(draft.created.len());
            let key =
                draft.allocate(DomNodeKind::Document(metadata), own, self.limits, cancelled)?;
            // Only this genuinely new owned Document receives its original
            // qualified history. Captured current Documents are never reset.
            draft.node_mut(key).interactive_document = Some(interaction);
            source.rows[i].key = Some(key);
        }
        for i in 0..source.rows.len() {
            c.step(1)?;
            let node = source.rows[i].source;
            if source.rows[i].key.is_some() {
                continue;
            }
            let document = source
                .observation(&node.id(), &mut c, |r| r.node_document())?
                .ok_or_else(|| Error::MissingMetadata { subject: node.id() })?;
            let document = source.document_key(document, &mut c)?;
            let DomNodeKind::Document(document_metadata) = &draft.node(document)?.kind else {
                return Err(Error::UnsupportedState { subject: node.id() });
            };
            // Existing captured Documents retain their own current platform.
            // Only newly created auxiliary Documents take the store selection.
            let input = document_metadata.document_environment().input_platform();
            let kind = state::kind(&source, node, input, &mut c)?;
            source.rows[i].key = Some(draft.allocate(kind, document, self.limits, cancelled)?);
        }
        for row in &source.rows {
            c.step(1)?;
            let key = row.key.expect("complete private allocation pass");
            let parent = row
                .source
                .parent()
                .map(|id| source.key(&id, &mut c))
                .transpose()?;
            let mut children = Vec::new();
            c.reserve(&mut children, row.source.children().len())?;
            for child in row.source.children() {
                c.step(1)?;
                children.push(source.key(&child, &mut c)?);
            }
            let destination = draft.node_mut(key);
            destination.parent = parent;
            destination.children = children;
        }
        state::metadata(&source, &mut draft, self.limits, &mut c)?;
        if capture.is_some() {
            apply_fragment_effects(&report, &source, &mut draft, self.limits, &mut c)?;
            qualify_captured_hyperlinks(&source, &draft, &mut c)?;
        }
        let root = source.key(&root_id, &mut c)?;
        // The final IDs and complete returned bindings are prepared before the
        // one canonical publication; all subsequent operations are moves.
        c.step(source.rows.len())?;
        c.retain(
            source
                .rows
                .len()
                .checked_mul(size_of::<NodeId>())
                .ok_or(Error::RepresentationExhausted)?,
        )?;
        let mut bindings = Vec::new();
        c.reserve(&mut bindings, source.rows.len())?;
        let (prepared, mapped) = draft.finish_with_mapping(
            self,
            cancelled,
            source
                .rows
                .iter()
                .map(|row| row.key.expect("allocated source row")),
        )?;
        // finish owns the actual vector; account its observed backing rather
        // than replacing it with an uncharged collector.
        if mapped.capacity() > source.rows.len() {
            c.retain(
                (mapped.capacity() - source.rows.len())
                    .checked_mul(size_of::<NodeId>())
                    .ok_or(Error::RepresentationExhausted)?,
            )?;
        }
        let root_index = source.index(&root_id, &mut c)?.expect("collected root");
        let document = if let Some(capture) = capture {
            let context = report.context().expect("qualified contextual report");
            DocumentId(
                capture
                    .node(context.target_document().subject())
                    .map_err(|_| Error::ForeignContext)?,
            )
        } else {
            DocumentId(mapped[root_index])
        };
        let root_id_canonical = mapped[root_index];
        for (row, node) in source.rows.iter().zip(mapped) {
            c.step(1)?;
            bindings.push(DomParsedHtmlBinding {
                source: row.source.id(),
                node,
            });
        }
        debug_assert_eq!(root, source.rows[root_index].key.expect("allocated root"));
        let captured_bindings = std::mem::take(&mut source.captured_bindings);
        drop(source);
        c.check()?;
        Ok(PreparedInstallation {
            prepared,
            report,
            source_root: root_id,
            document,
            root: root_id_canonical,
            context_basis: capture.map(crate::DomHtmlCapture::basis),
            captured_bindings,
            bindings,
            #[cfg(test)]
            admitted_work: c.work,
            #[cfg(test)]
            admitted_scratch_bytes: c.peak_bytes,
        })
    }
    fn qualify_contextual_policy(
        &self,
        source: &Source<'_>,
        c: &mut Control<'_>,
    ) -> Result<(), Error> {
        // Current State is held by this operation. Constructor/association
        // transitions preserve this environment; no immutable external stamp is
        // used as authority for URL, CSP, image or host-service currentness.
        for binding in &source.captured_bindings {
            c.step(1)?;
            let node = self.state.node(self.owner, binding.node)?;
            if let DomNodeKind::Document(metadata) = &node.kind {
                qualify_svg_destination(metadata)?;
                if metadata.meta_state().is_none()
                    || metadata.policy_container().referrer_policy().is_none()
                {
                    return Err(Error::CurrentPolicyUnavailable);
                }
            }
        }
        Ok(())
    }
    fn publish_parsed_html_installation(
        &mut self,
        pending: PreparedInstallation,
        cancelled: &AtomicBool,
    ) -> Result<DomParsedHtmlInstallation, Error> {
        let commit = self.publish(pending.prepared, cancelled)?;
        Ok(DomParsedHtmlInstallation {
            report: pending.report,
            source_root: pending.source_root,
            document: pending.document,
            root: pending.root,
            context_basis: pending.context_basis,
            captured_bindings: pending.captured_bindings,
            snapshot: self.snapshot(),
            changes: commit.changes.clone(),
            bindings: pending.bindings,
        })
    }
}
