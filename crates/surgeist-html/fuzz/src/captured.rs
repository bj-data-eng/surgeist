//! Capture import of an immutable construction result through the public
//! semantics adapter, with the expected outcome stated before the call.
//!
//! The adapter's documented contract: external facts are supplied per node and
//! never inferred from topology; a refused import returns the original builder,
//! source and facts, with the builder's retained usage unchanged; a completed
//! import yields one subject per reachable node, whose kind, payload, ordinary
//! parent and ordered children, template contents, shadow root and host mirror
//! the source and whose supplied facts are retained as given.
//!
//! One fault at most is injected per plan. The refusal each fault must produce
//! follows from that contract, not from running the adapter.
use std::collections::HashMap;
use surgeist_fuzz::{FuzzError, FuzzErrorKind, Result};
use surgeist_html::construction::HtmlConstructionResult;
use surgeist_html::{
    construction::{
        HtmlConstructionLimits, HtmlDocumentMode, HtmlFragment, HtmlFragmentOwner, HtmlNodeId,
        HtmlNodeKind, HtmlShadowRootProperties,
    },
    values::DocumentString,
};
#[cfg(test)]
use surgeist_html::{values::OrderedAttributes, vocabulary::KnownHtmlElement};
use surgeist_html_semantics::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Fault {
    None,
    Revision,
    MissingFacts,
    DuplicateFacts,
    RoleMismatch,
    ForeignNode,
    ForeignRegistry,
    Cancelled,
    MappingLimit,
    ScratchLimit,
    WorkLimit,
    SubjectLimit,
}
impl Fault {
    pub(crate) const ALL: [Fault; 12] = [
        Self::None,
        Self::Revision,
        Self::MissingFacts,
        Self::DuplicateFacts,
        Self::RoleMismatch,
        Self::ForeignNode,
        Self::ForeignRegistry,
        Self::Cancelled,
        Self::MappingLimit,
        Self::ScratchLimit,
        Self::WorkLimit,
        Self::SubjectLimit,
    ];
    pub(crate) fn from_tag(tag: u8) -> Option<Self> {
        Self::ALL.get(usize::from(tag)).copied()
    }
    pub(crate) fn tag(self) -> u8 {
        Self::ALL
            .iter()
            .position(|f| *f == self)
            .map_or(0, |i| i as u8)
    }
}

/// External facts and the one injected fault.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Plan {
    /// 0 disabled, 1 enabled, 2 unavailable.
    pub(crate) scripting: u8,
    /// 0 null, 1 unavailable, 2 one shared complete registry.
    pub(crate) registry: u8,
    /// 0 absent, 1 unavailable, 2 a value.
    pub(crate) is_value: u8,
    /// Bit 0 declarative, bit 1 available to internals, bit 2 keep registry null.
    pub(crate) shadow: u8,
    pub(crate) fault: Fault,
    /// Selects the node (or limit) the fault applies to.
    pub(crate) pick: u8,
}
impl Plan {
    pub(crate) const DEFAULT: Plan = Plan {
        scripting: 0,
        registry: 0,
        is_value: 0,
        shadow: 0,
        fault: Fault::None,
        pick: 0,
    };
    pub(crate) fn valid(&self) -> bool {
        self.scripting < 3 && self.registry < 3 && self.is_value < 3 && self.shadow < 8
    }
}

pub(crate) enum Verdict {
    /// Every mirror, qualification and refusal check held.
    Verified,
    /// A bounded stop of the adapter (a work allowance), with atomic refusal checked.
    ResourceStop,
    Violation {
        /// 0 mirror, 1 fact qualification, 2 atomic refusal.
        property: usize,
        key: &'static str,
        detail: String,
    },
}
fn violation(property: usize, key: &'static str, detail: impl Into<String>) -> Result<Verdict> {
    Ok(Verdict::Violation {
        property,
        key,
        detail: detail.into(),
    })
}
fn harness(
    operation: &'static str,
    detail: &'static str,
    e: impl std::error::Error + Send + Sync + 'static,
) -> FuzzError {
    FuzzError::with_source(FuzzErrorKind::Harness, operation, detail, e)
}

/// Independent view of the source, taken only from its public node views.
struct Info {
    id: HtmlNodeId,
    kind: HtmlNodeKind,
    parent: Option<usize>,
    children: Vec<usize>,
    template: Option<usize>,
    shadow: Option<(usize, HtmlShadowRootProperties)>,
    /// The element a fragment belongs to, and whether it is a shadow root.
    owner: Option<(usize, bool)>,
}
fn closure(result: &HtmlConstructionResult) -> Result<Vec<Info>> {
    let root = match result {
        HtmlConstructionResult::Document(d) => d.root().id(),
        HtmlConstructionResult::Fragment(f) => f.root().id(),
    };
    let mut order = vec![root];
    let mut index: HashMap<HtmlNodeId, usize> = HashMap::new();
    let mut at = 0;
    while at < order.len() {
        let id = order[at];
        index.insert(id, at);
        at += 1;
        let node = result
            .node(&id)
            .map_err(|e| harness("capture closure", "foreign source node", e))?;
        let mut next: Vec<HtmlNodeId> = node.children().collect();
        next.extend(node.template_contents());
        next.extend(node.declarative_shadow_root().map(|s| s.contents()));
        for id in next {
            if order.len() > 100_000 {
                return Err(FuzzError::new(
                    FuzzErrorKind::Resource,
                    "capture closure",
                    "observation node ceiling",
                ));
            }
            order.push(id);
        }
    }
    let lookup = |id: &HtmlNodeId| index.get(id).copied();
    order
        .iter()
        .map(|id| {
            let node = result
                .node(id)
                .map_err(|e| harness("capture closure", "foreign source node", e))?;
            Ok(Info {
                id: *id,
                kind: node.kind().clone(),
                parent: node.parent().as_ref().and_then(lookup),
                children: node.children().filter_map(|c| lookup(&c)).collect(),
                template: node.template_contents().as_ref().and_then(lookup),
                shadow: node
                    .declarative_shadow_root()
                    .and_then(|s| lookup(&s.contents()).map(|i| (i, s.properties()))),
                owner: match node.fragment_owner() {
                    Some(HtmlFragmentOwner::ShadowHost(h)) => lookup(&h).map(|i| (i, true)),
                    Some(HtmlFragmentOwner::Template(h)) => lookup(&h).map(|i| (i, false)),
                    None => None,
                },
            })
        })
        .collect()
}

fn scripting(tag: u8) -> HtmlScriptingStatus {
    match tag {
        0 => HtmlScriptingStatus::Disabled,
        1 => HtmlScriptingStatus::Enabled,
        _ => HtmlScriptingStatus::Unavailable,
    }
}
fn is_value(tag: u8) -> HtmlCapturedIsValue {
    match tag {
        0 => HtmlCapturedIsValue::Absent,
        1 => HtmlCapturedIsValue::Unavailable,
        _ => HtmlCapturedIsValue::Value(DocumentString::from("v")),
    }
}
struct Inputs<'a> {
    plan: &'a Plan,
    registry: Option<HtmlCaptureBuilderRegistry>,
    document: Option<HtmlCaptureBuilderSubject>,
}
impl Inputs<'_> {
    fn registry(&self) -> HtmlCaptureRegistryInput {
        match (self.plan.registry, &self.registry) {
            (0, _) => HtmlCaptureRegistryInput::Null,
            (1, _) => HtmlCaptureRegistryInput::Unavailable,
            (_, Some(handle)) => HtmlCaptureRegistryInput::Registry(handle.clone()),
            (_, None) => HtmlCaptureRegistryInput::Unavailable,
        }
    }
    fn rows(
        &self,
        infos: &[Info],
        revision: HtmlHostFactsRevision,
    ) -> HtmlConstructionCaptureFacts {
        let rows = infos
            .iter()
            .map(|info| {
                let document = match &self.document {
                    Some(d) => HtmlConstructionCaptureDocument::Existing(d.clone()),
                    None => HtmlConstructionCaptureDocument::ResultDocument,
                };
                HtmlConstructionCaptureNodeFacts::new(
                    info.id,
                    document,
                    matches!(info.kind, HtmlNodeKind::Document(_)).then(|| {
                        HtmlConstructionCaptureDocumentFacts::new(
                            scripting(self.plan.scripting),
                            self.registry(),
                        )
                    }),
                    matches!(info.kind, HtmlNodeKind::Element(_)).then(|| {
                        HtmlConstructionCaptureElementFacts::new(
                            is_value(self.plan.is_value),
                            self.registry(),
                        )
                    }),
                    matches!(info.owner, Some((_, true))).then(|| {
                        HtmlConstructionCaptureShadowFacts::new(
                            self.registry(),
                            self.plan.shadow & 1 != 0,
                            self.plan.shadow & 2 != 0,
                            self.plan.shadow & 4 != 0,
                        )
                    }),
                )
            })
            .collect::<Vec<_>>();
        HtmlConstructionCaptureFacts::new(revision, rows)
    }
}

/// Admitted adapter usage never exceeds its allowance.
fn within(usage: HtmlConstructionCaptureUsage, limits: HtmlConstructionCaptureLimits) -> bool {
    usage.mapping_entries() <= limits.mapping_entries()
        && usage.scratch_entries() <= limits.scratch_entries()
        && usage.work() <= limits.work()
}
fn limits_for(n: usize) -> HtmlConstructionCaptureLimits {
    HtmlConstructionCaptureLimits::new(n + 8, 1 << 24, 1 << 40)
}
fn builder<'a>(
    plan: &'a Plan,
    n: usize,
    fragment: bool,
    revision: &HtmlHostFactsRevision,
) -> Result<(HtmlCaptureBuilder, Inputs<'a>)> {
    let existing = usize::from(fragment);
    let subjects = if plan.fault == Fault::SubjectLimit {
        existing + n - 1
    } else {
        existing + n + 8
    };
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(
            subjects,
            8 * n + 64,
            4 * n + 64,
            1 << 22,
            4096,
            8,
            8,
            1 << 24,
        ),
        revision.clone(),
    );
    let mut inputs = Inputs {
        plan,
        registry: None,
        document: None,
    };
    if plan.registry == 2 {
        inputs.registry = Some(
            b.add_registry(Some(Vec::new()))
                .map_err(|e| harness("capture builder", "registry", e))?,
        );
    }
    if fragment {
        inputs.document = Some(
            b.add_document(
                HtmlDocumentMode::NoQuirks,
                scripting(plan.scripting),
                inputs.registry(),
            )
            .map_err(|e| harness("capture builder", "entry document", e))?,
        );
    }
    Ok((b, inputs))
}

#[expect(
    clippy::result_large_err,
    reason = "the adapter returns its failure by value to hand back the original inputs"
)]
fn run(
    b: HtmlCaptureBuilder,
    result: &HtmlConstructionResult,
    facts: HtmlConstructionCaptureFacts,
    limits: HtmlConstructionCaptureLimits,
    cancel: HtmlSemanticCancellation,
) -> std::result::Result<HtmlConstructionCaptureImport, HtmlConstructionCaptureFailure> {
    match result {
        HtmlConstructionResult::Document(d) => {
            HtmlConstructionCaptureAdapter::document(b, d.clone(), facts, limits, cancel)
        }
        HtmlConstructionResult::Fragment(f) => {
            HtmlConstructionCaptureAdapter::fragment(b, f.clone(), facts, limits, cancel)
        }
    }
}

/// The subject, relation and payload snapshot of one capture, by source ordinal.
fn snapshot(
    captured: &HtmlConstructionCapturedResult,
    infos: &[Info],
) -> std::result::Result<Vec<String>, &'static str> {
    let capture = captured.capture();
    let ids = infos
        .iter()
        .map(|i| captured.mapping().subject(&i.id, capture))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| "subject-missing")?;
    let ordinal = |id: &HtmlCaptureSubjectId| ids.iter().position(|x| x == id);
    ids.iter()
        .map(|id| {
            let node = capture.node(id).map_err(|_| "subject-lending")?;
            let children = match node.children() {
                HtmlCapturedChildren::Complete(c) => {
                    Some(c.map(|c| ordinal(&c)).collect::<Vec<_>>())
                }
                HtmlCapturedChildren::Unavailable => None,
            };
            let parent = match node.parent() {
                HtmlCapturedParent::Subject(p) => format!("{:?}", ordinal(&p)),
                other => format!("{other:?}"),
            };
            let host = node.host().map(|h| match h {
                HtmlCapturedHost::Element(e) => format!("{:?}", ordinal(e.subject())),
                other => format!("{other:?}"),
            });
            let shadow = node.shadow_root().map(|s| match s {
                HtmlCapturedShadowRoot::Present(f) => format!("{:?}", ordinal(f.subject())),
                other => format!("{other:?}"),
            });
            let template = node.template_contents().map(|t| match t {
                HtmlCapturedTemplateContents::Present(f) => format!("{:?}", ordinal(f.subject())),
                other => format!("{other:?}"),
            });
            Ok(format!(
                "{:?}|{parent}|{children:?}|{host:?}|{shadow:?}|{template:?}|{:?}|{:?}",
                node.kind(),
                node.lookup_registry(),
                node.template_contents_kind(),
            ))
        })
        .collect()
}

fn mirror(
    captured: &HtmlConstructionCapturedResult,
    infos: &[Info],
    plan: &Plan,
    fragment: bool,
    registry: &Option<HtmlCaptureBuilderRegistry>,
) -> std::result::Result<(), (usize, &'static str, String)> {
    let capture = captured.capture();
    let registry = registry
        .as_ref()
        .map(|h| capture.registry_id(h))
        .transpose()
        .map_err(|e| (1, "registry-handle", format!("{e:?}")))?;
    let subject = |i: usize| {
        captured
            .mapping()
            .subject(&infos[i].id, capture)
            .map_err(|e| (0, "subject-missing", format!("{e:?}")))
    };
    let expected_registry = match (plan.registry, &registry) {
        (0, _) => HtmlCapturedRegistry::Null,
        (1, _) => HtmlCapturedRegistry::Unavailable,
        (_, Some(id)) => HtmlCapturedRegistry::Registry(id.clone()),
        (_, None) => HtmlCapturedRegistry::Unavailable,
    };
    let usage = capture.usage().subjects();
    let wanted = infos.len() + usize::from(fragment);
    if usage != wanted {
        return Err((0, "subject-count", format!("{usage} != {wanted}")));
    }
    let root = subject(0)?;
    for (i, info) in infos.iter().enumerate() {
        let id = subject(i)?;
        let node = capture
            .node(&id)
            .map_err(|e| (0, "subject-lending", format!("{e:?}")))?;
        let mut local_registry = HtmlCapturedRegistry::Null;
        match (&info.kind, node.kind()) {
            (HtmlNodeKind::Document(mode), HtmlCapturedNodeKind::Document(d)) => {
                if d.mode() != *mode || d.scripting() != scripting(plan.scripting) {
                    return Err((1, "document-fact", format!("{d:?}")));
                }
                local_registry = expected_registry.clone();
            }
            (HtmlNodeKind::Element(e), HtmlCapturedNodeKind::Element(c)) => {
                if c.name() != e.name() || c.attributes() != e.attributes() {
                    return Err((0, "element-payload", format!("{i}")));
                }
                if c.is_value() != &is_value(plan.is_value) {
                    return Err((1, "element-fact", format!("{:?}", c.is_value())));
                }
                local_registry = expected_registry.clone();
            }
            (HtmlNodeKind::Fragment, HtmlCapturedNodeKind::Fragment(f)) => {
                match (matches!(info.owner, Some((_, true))), f.kind()) {
                    (false, HtmlCapturedFragmentKind::Plain) => {}
                    (true, HtmlCapturedFragmentKind::ShadowRoot(p)) => {
                        let wanted = infos
                            .iter()
                            .find_map(|h| h.shadow.filter(|(c, _)| *c == i).map(|(_, p)| p));
                        if wanted != Some(p.properties())
                            || p.declarative() != (plan.shadow & 1 != 0)
                            || p.available_to_internals() != (plan.shadow & 2 != 0)
                            || p.keep_registry_null() != (plan.shadow & 4 != 0)
                        {
                            return Err((1, "shadow-fact", format!("{p:?}")));
                        }
                        local_registry = expected_registry.clone();
                    }
                    _ => return Err((0, "fragment-kind", format!("{i}"))),
                }
            }
            (HtmlNodeKind::Text(a), HtmlCapturedNodeKind::Text(b))
            | (HtmlNodeKind::Comment(a), HtmlCapturedNodeKind::Comment(b)) => {
                if a != b {
                    return Err((0, "text-payload", format!("{i}")));
                }
            }
            (
                HtmlNodeKind::ProcessingInstruction(a),
                HtmlCapturedNodeKind::ProcessingInstruction(b),
            ) => {
                if a.target() != b.target() || a.data() != b.data() {
                    return Err((0, "instruction-payload", format!("{i}")));
                }
            }
            (HtmlNodeKind::Doctype(a), HtmlCapturedNodeKind::Doctype(b)) => {
                if a != b {
                    return Err((0, "doctype-payload", format!("{i}")));
                }
            }
            _ => return Err((0, "kind", format!("{i}"))),
        }
        if node.lookup_registry() != local_registry {
            return Err((
                1,
                "registry-fact",
                format!("{i}: {:?}", node.lookup_registry()),
            ));
        }
        // Document membership: the result document, or the supplied entry document.
        let member = node.node_document();
        if fragment {
            if capture.node(member.subject()).is_err() || member.subject() == &root {
                return Err((0, "node-document", format!("{i}")));
            }
        } else if member.subject() != &root {
            return Err((0, "node-document", format!("{i}")));
        }
        let expected_parent = match info.parent {
            Some(p) => HtmlCapturedParent::Subject(subject(p)?),
            None => HtmlCapturedParent::Root,
        };
        let container = matches!(
            info.kind,
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_)
        );
        if node.parent() != expected_parent {
            return Err((0, "parent", format!("{i}: {:?}", node.parent())));
        }
        if container {
            let want = info
                .children
                .iter()
                .map(|c| subject(*c))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            match node.children() {
                HtmlCapturedChildren::Complete(c) if c.clone().collect::<Vec<_>>() == want => {}
                _ => return Err((0, "children", format!("{i}"))),
            }
        }
        if let HtmlNodeKind::Element(_) = info.kind {
            match (info.template, node.template_contents()) {
                (Some(t), Some(HtmlCapturedTemplateContents::Present(f)))
                    if f.subject() == &subject(t)? => {}
                (None, None) => {}
                (_, found) => return Err((0, "template-contents", format!("{i}: {found:?}"))),
            }
            match (info.shadow, node.shadow_root()) {
                (Some((c, _)), Some(HtmlCapturedShadowRoot::Present(f)))
                    if f.subject() == &subject(c)? => {}
                (Some(_), found) => return Err((0, "shadow-root", format!("{i}: {found:?}"))),
                (None, Some(HtmlCapturedShadowRoot::Present(_))) => {
                    return Err((0, "shadow-root", format!("{i}: invented")));
                }
                (None, _) => {}
            }
        }
        if let HtmlNodeKind::Fragment = info.kind {
            // A shadow root's host and a template contents' host are the owning
            // element; a root fragment has none.
            match (info.owner, node.host()) {
                (Some((h, _)), Some(HtmlCapturedHost::Element(e)))
                    if e.subject() == &subject(h)? => {}
                (Some(_), found) => return Err((0, "fragment-host", format!("{i}: {found:?}"))),
                (None, Some(HtmlCapturedHost::Element(_))) => {
                    return Err((0, "fragment-host", format!("{i}: invented")));
                }
                (None, _) => {}
            }
        }
    }
    Ok(())
}

/// Imports `result` under `plan`, checks the expected outcome and, after a
/// refusal, that the returned inputs complete a fault-free retry.
pub(crate) fn check(result: &HtmlConstructionResult, plan: &Plan) -> Result<Verdict> {
    let infos = closure(result)?;
    let n = infos.len();
    let fragment = result.fragment().is_some();
    let revision = HtmlHostFactsRevision::fresh();
    let (b, inputs) = builder(plan, n, fragment, &revision)?;
    let before = b.usage();
    let mut facts = inputs.rows(&infos, revision.clone());
    let mut limits = limits_for(n);
    let cancel = HtmlSemanticCancellation::new();
    let foreign_registry = if plan.fault == Fault::ForeignRegistry {
        let mut other = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(4, 4, 4, 64, 4, 4, 4, 64),
            HtmlHostFactsRevision::fresh(),
        );
        Some(
            other
                .add_registry(Some(Vec::new()))
                .map_err(|e| harness("capture builder", "foreign registry", e))?,
        )
    } else {
        None
    };
    let pick = usize::from(plan.pick);
    let mut inapplicable = false;
    let expected: Option<HtmlConstructionCaptureError> = match plan.fault {
        Fault::None => None,
        Fault::Revision => {
            facts = HtmlConstructionCaptureFacts::new(
                HtmlHostFactsRevision::fresh(),
                facts.nodes().to_vec(),
            );
            Some(HtmlConstructionCaptureError::HostRevisionMismatch)
        }
        Fault::MissingFacts => {
            let mut rows = facts.nodes().to_vec();
            rows.remove(pick % n);
            facts = HtmlConstructionCaptureFacts::new(revision.clone(), rows);
            Some(HtmlConstructionCaptureError::MissingNodeDocument)
        }
        Fault::DuplicateFacts => {
            let mut rows = facts.nodes().to_vec();
            rows.push(rows[pick % n].clone());
            facts = HtmlConstructionCaptureFacts::new(revision.clone(), rows);
            Some(HtmlConstructionCaptureError::DuplicateNodeFacts)
        }
        Fault::RoleMismatch => {
            let mut rows = facts.nodes().to_vec();
            let target = (0..n)
                .map(|k| (k + pick) % n)
                .find(|k| !matches!(infos[*k].kind, HtmlNodeKind::Document(_)));
            if let Some(k) = target {
                let old = &rows[k];
                rows[k] = HtmlConstructionCaptureNodeFacts::new(
                    *old.node(),
                    old.document().clone(),
                    Some(HtmlConstructionCaptureDocumentFacts::new(
                        scripting(plan.scripting),
                        HtmlCaptureRegistryInput::Null,
                    )),
                    old.element_facts().cloned(),
                    old.shadow_facts().cloned(),
                );
                facts = HtmlConstructionCaptureFacts::new(revision.clone(), rows);
                Some(HtmlConstructionCaptureError::WrongFactRole)
            } else {
                inapplicable = true;
                None
            }
        }
        Fault::ForeignNode => {
            let foreign = HtmlFragment::builder(HtmlConstructionLimits::new(1, 0, 0, 0, 0))
                .map_err(|e| harness("capture fault", "foreign source", e))?
                .finish()
                .map_err(|e| harness("capture fault", "foreign source", e))?;
            let mut rows = facts.nodes().to_vec();
            rows.push(HtmlConstructionCaptureNodeFacts::new(
                foreign.root().id(),
                match &inputs.document {
                    Some(d) => HtmlConstructionCaptureDocument::Existing(d.clone()),
                    None => HtmlConstructionCaptureDocument::ResultDocument,
                },
                None,
                None,
                None,
            ));
            facts = HtmlConstructionCaptureFacts::new(revision.clone(), rows);
            Some(HtmlConstructionCaptureError::ForeignResult)
        }
        Fault::ForeignRegistry => {
            let foreign = foreign_registry
                .clone()
                .map(HtmlCaptureRegistryInput::Registry)
                .unwrap_or(HtmlCaptureRegistryInput::Unavailable);
            let mut rows = facts.nodes().to_vec();
            let candidates: Vec<usize> = (0..n)
                .filter(|k| {
                    rows[*k].document_facts().is_some()
                        || rows[*k].element_facts().is_some()
                        || rows[*k].shadow_facts().is_some()
                })
                .collect();
            if candidates.is_empty() {
                inapplicable = true;
                None
            } else {
                let k = candidates[pick % candidates.len()];
                let old = &rows[k];
                rows[k] = HtmlConstructionCaptureNodeFacts::new(
                    *old.node(),
                    old.document().clone(),
                    old.document_facts().map(|d| {
                        HtmlConstructionCaptureDocumentFacts::new(d.scripting(), foreign.clone())
                    }),
                    old.element_facts().map(|e| {
                        HtmlConstructionCaptureElementFacts::new(
                            e.is_value().clone(),
                            foreign.clone(),
                        )
                    }),
                    old.shadow_facts().map(|s| {
                        HtmlConstructionCaptureShadowFacts::new(
                            foreign.clone(),
                            s.declarative(),
                            s.available_to_internals(),
                            s.keep_registry_null(),
                        )
                    }),
                );
                facts = HtmlConstructionCaptureFacts::new(revision.clone(), rows);
                Some(HtmlConstructionCaptureError::ForeignBuilder)
            }
        }
        Fault::Cancelled => {
            cancel.cancel();
            Some(HtmlConstructionCaptureError::Cancelled)
        }
        Fault::MappingLimit => {
            limits =
                HtmlConstructionCaptureLimits::new(n - 1, limits.scratch_entries(), limits.work());
            Some(HtmlConstructionCaptureError::LimitExceeded)
        }
        Fault::ScratchLimit => {
            limits = HtmlConstructionCaptureLimits::new(limits.mapping_entries(), 0, limits.work());
            Some(HtmlConstructionCaptureError::LimitExceeded)
        }
        Fault::WorkLimit => {
            limits = HtmlConstructionCaptureLimits::new(
                limits.mapping_entries(),
                limits.scratch_entries(),
                pick * 4,
            );
            None
        }
        Fault::SubjectLimit => None,
    };
    let row_count = facts.nodes().len();
    let injected = plan.fault != Fault::None && !inapplicable;
    let outcome = run(b, result, facts, limits, cancel);
    match outcome {
        Ok(import) => {
            if injected && plan.fault != Fault::WorkLimit {
                return violation(2, "fault-not-refused", format!("{:?} imported", plan.fault));
            }
            if import.usage().mapping_entries() != n {
                return violation(0, "mapping-count", format!("{:?}", import.usage()));
            }
            let imported = import.usage();
            if !within(imported, limits) {
                return violation(2, "usage-over-limit", format!("{imported:?}"));
            }
            let finished = match import.finish() {
                Ok(f) => f,
                // Finishing spends the same work allowance as the traversal, so
                // a tight allowance can stop here; the refusal keeps the import.
                Err(f)
                    if plan.fault == Fault::WorkLimit
                        && *f.reason() == HtmlConstructionCaptureError::LimitExceeded =>
                {
                    if f.import().usage().work() > limits.work()
                        || f.import().usage().work() < imported.work()
                        || f.import().mapping().root() != &infos[0].id
                    {
                        return violation(
                            2,
                            "finish-refusal-state",
                            format!("{:?}", f.import().usage()),
                        );
                    }
                    return Ok(Verdict::ResourceStop);
                }
                Err(f) => return violation(2, "finish-refused", format!("{:?}", f.reason())),
            };
            if let Err((property, key, detail)) =
                mirror(&finished, &infos, plan, fragment, &inputs.registry)
            {
                return Ok(Verdict::Violation {
                    property,
                    key,
                    detail,
                });
            }
            // The source result is untouched and deterministic re-import into a
            // fresh builder reproduces the same observations while the first
            // capture stays as it was.
            let first = match snapshot(&finished, &infos) {
                Ok(s) => s,
                Err(key) => return violation(0, key, "snapshot"),
            };
            if finished.source().len() != result.len() {
                return violation(0, "source-changed", "result record count");
            }
            let again = Plan {
                fault: Fault::None,
                ..*plan
            };
            let (b2, inputs2) = builder(&again, n, fragment, &revision)?;
            let facts2 = inputs2.rows(&infos, revision.clone());
            let second = run(
                b2,
                result,
                facts2,
                limits_for(n),
                HtmlSemanticCancellation::new(),
            );
            let Ok(second) = second else {
                return violation(0, "reimport-refused", "fault-free re-import");
            };
            if !within(second.usage(), limits_for(n)) {
                return violation(2, "usage-over-limit", format!("{:?}", second.usage()));
            }
            let Ok(second) = second.finish() else {
                return violation(0, "reimport-refused", "fault-free re-import finish");
            };
            if snapshot(&second, &infos).as_ref() != Ok(&first)
                || snapshot(&finished, &infos).as_ref() != Ok(&first)
            {
                return violation(
                    0,
                    "capture-not-deterministic-or-mutated",
                    "second import changed or differed from the first",
                );
            }
            Ok(Verdict::Verified)
        }
        Err(failure) => {
            let reason = failure.reason().clone();
            let resource = plan.fault == Fault::WorkLimit
                && reason == HtmlConstructionCaptureError::LimitExceeded;
            let subject_limit = plan.fault == Fault::SubjectLimit
                && matches!(
                    reason,
                    HtmlConstructionCaptureError::Capture(HtmlCaptureError::LimitExceeded { .. })
                );
            if !(resource || subject_limit || (injected && expected.as_ref() == Some(&reason))) {
                return violation(
                    2,
                    "refusal-mismatch",
                    format!("{:?} gave {reason:?}", plan.fault),
                );
            }
            // Atomic refusal: the same builder, source and facts, nothing retained.
            if failure.builder().usage() != before {
                return violation(2, "builder-usage-changed", format!("{reason:?}"));
            }
            if failure.facts().nodes().len() != row_count {
                return violation(2, "facts-not-returned", format!("{reason:?}"));
            }
            let admitted = failure.usage();
            if !within(admitted, limits) {
                return violation(2, "usage-over-limit", format!("{admitted:?}"));
            }
            let source_ok = failure.source().len() == result.len();
            if !source_ok {
                return violation(2, "source-not-returned", format!("{reason:?}"));
            }
            if plan.fault == Fault::SubjectLimit {
                // The builder's own retention limit is fixed, so no retry can succeed.
                return Ok(Verdict::Verified);
            }
            // The returned builder completes a fault-free import of the returned source.
            let (_, builder, source, _, _) = failure.into_parts();
            let retry_facts = inputs.rows(&infos, revision.clone());
            let retried = run(
                builder,
                &source,
                retry_facts,
                limits_for(n),
                HtmlSemanticCancellation::new(),
            );
            let retried = match retried {
                Ok(r) => r,
                Err(f) => {
                    return violation(
                        2,
                        "retry-refused",
                        format!("after {reason:?}: {:?}", f.reason()),
                    );
                }
            };
            if !within(retried.usage(), limits_for(n)) {
                return violation(2, "usage-over-limit", format!("{:?}", retried.usage()));
            }
            let Ok(finished) = retried.finish() else {
                return violation(2, "retry-refused", format!("finish after {reason:?}"));
            };
            if let Err((property, key, detail)) =
                mirror(&finished, &infos, plan, fragment, &inputs.registry)
            {
                return Ok(Verdict::Violation {
                    property,
                    key,
                    detail: format!("after retry: {detail}"),
                });
            }
            if resource {
                Ok(Verdict::ResourceStop)
            } else {
                Ok(Verdict::Verified)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn native_fragment() -> HtmlConstructionResult {
        let mut b = HtmlFragment::builder(HtmlConstructionLimits::new(8, 8, 4, 64, 8)).unwrap();
        let root = b.root();
        let div = b
            .append_element(
                &root,
                KnownHtmlElement::Div.qualified_name(),
                OrderedAttributes::default(),
            )
            .unwrap();
        b.append_text(&div, "x").unwrap();
        HtmlConstructionResult::Fragment(b.finish().unwrap())
    }

    fn imported(
        result: &HtmlConstructionResult,
        plan: &Plan,
    ) -> (HtmlConstructionCapturedResult, Vec<Info>) {
        let infos = closure(result).unwrap();
        let revision = HtmlHostFactsRevision::fresh();
        let (b, inputs) = builder(plan, infos.len(), true, &revision).unwrap();
        let facts = inputs.rows(&infos, revision);
        let import = run(
            b,
            result,
            facts,
            limits_for(infos.len()),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        (import.finish().unwrap(), infos)
    }

    #[test]
    fn a_clean_import_mirrors_topology_and_a_wrong_expectation_is_reported() {
        let result = native_fragment();
        let plan = Plan {
            scripting: 1,
            is_value: 2,
            ..Plan::DEFAULT
        };
        let (captured, infos) = imported(&result, &plan);
        assert_eq!(infos.len(), 3);
        assert_eq!(mirror(&captured, &infos, &plan, true, &None), Ok(()));
        // The supplied facts are what the capture retains: expecting different
        // external facts, or a different entry document, is a reported violation.
        let wrong_element = Plan {
            is_value: 0,
            ..plan
        };
        assert_eq!(
            mirror(&captured, &infos, &wrong_element, true, &None).map_err(|e| e.1),
            Err("element-fact")
        );
        let wrong_registry = Plan {
            registry: 1,
            ..plan
        };
        assert_eq!(
            mirror(&captured, &infos, &wrong_registry, true, &None).map_err(|e| e.1),
            Err("registry-fact")
        );
        assert_eq!(
            mirror(&captured, &infos, &plan, false, &None).map_err(|e| e.1),
            Err("subject-count")
        );
    }

    #[test]
    fn every_fault_has_the_refusal_the_adapter_contract_requires() {
        let result = native_fragment();
        for fault in Fault::ALL {
            let plan = Plan {
                registry: 2,
                fault,
                pick: 1,
                ..Plan::DEFAULT
            };
            match check(&result, &plan).unwrap() {
                Verdict::Verified => {}
                Verdict::ResourceStop => assert_eq!(fault, Fault::WorkLimit),
                Verdict::Violation { key, detail, .. } => panic!("{fault:?}: {key}: {detail}"),
            }
        }
    }
}
