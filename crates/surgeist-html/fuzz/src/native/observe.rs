//! Finite public-native lending and independent recipe expectations.
use super::*;
use crate::captured::{Fault, Plan, Verdict, check};
use surgeist_html::construction::HtmlConstructionResult;

pub(super) struct Expected {
    op: Option<usize>,
    parent: Option<usize>,
    depth: usize,
    owner: Option<(usize, bool)>,
    children: Vec<usize>,
    template: Option<usize>,
    shadow: Option<(usize, u8)>,
    fragment: bool,
}
impl Expected {
    fn new(op: Option<usize>, parent: Option<usize>, depth: usize, fragment: bool) -> Result<Self> {
        let mut children = Vec::new();
        reserve(&mut children, 64)?;
        Ok(Self {
            op,
            parent,
            depth,
            owner: None,
            children,
            template: None,
            shadow: None,
            fragment,
        })
    }
}
enum Builder {
    Document(HtmlDocumentBuilder),
    Fragment(HtmlFragmentBuilder),
}
enum Frozen {
    Document(HtmlDocument),
    Fragment(HtmlFragment),
}
macro_rules! both {
    ($self:expr,$b:ident,$body:expr) => {
        match $self {
            Builder::Document($b) => $body,
            Builder::Fragment($b) => $body,
        }
    };
}
impl Builder {
    fn new(c: &NativeCase) -> std::result::Result<Self, HtmlConstructionError> {
        let [n, e, a, u, d] = c.caps;
        let limits = HtmlConstructionLimits::new(n, e, a, u, d);
        if c.root == 0 {
            HtmlFragment::builder(limits).map(Self::Fragment)
        } else {
            HtmlDocument::builder(
                limits,
                [
                    HtmlDocumentMode::NoQuirks,
                    HtmlDocumentMode::LimitedQuirks,
                    HtmlDocumentMode::Quirks,
                ][usize::from(c.root - 1)],
            )
            .map(Self::Document)
        }
    }
    fn root(&self) -> HtmlBuilderNode {
        both!(self, b, b.root())
    }
    fn usage(&self) -> HtmlConstructionUsage {
        both!(self, b, b.usage())
    }
    fn finish(self) -> std::result::Result<Frozen, HtmlConstructionError> {
        match self {
            Self::Document(b) => b.finish().map(Frozen::Document),
            Self::Fragment(b) => b.finish().map(Frozen::Fragment),
        }
    }
    fn apply(
        &mut self,
        o: &Op,
        p: &HtmlBuilderNode,
    ) -> std::result::Result<Option<HtmlBuilderNode>, Rejected> {
        match o {
            Op::Element(_, n, attrs) => {
                let mut raw = Vec::new();
                reserve(&mut raw, attrs.len()).map_err(|_| Rejected::Consumer)?;
                for a in attrs {
                    raw.push(a.native());
                }
                let attrs = OrderedAttributes::new(raw).map_err(|_| Rejected::Duplicate)?;
                both!(self, b, b.append_element(p, n.native(), attrs))
                    .map(Some)
                    .map_err(Rejected::Construction)
            }
            Op::Text(_, v) => both!(
                self,
                b,
                b.append_text(p, DocumentString::from_utf16(v.clone()))
            )
            .map(Some)
            .map_err(Rejected::Construction),
            Op::Comment(_, v) => both!(
                self,
                b,
                b.append_comment(p, DocumentString::from_utf16(v.clone()))
            )
            .map(Some)
            .map_err(Rejected::Construction),
            Op::Pi(_, t, d) => {
                let value = HtmlAuthoredProcessingInstruction::new(
                    DocumentString::from_utf16(t.clone()),
                    DocumentString::from_utf16(d.clone()),
                )
                .map_err(Rejected::Pi)?;
                both!(self, b, b.append_processing_instruction(p, value))
                    .map(Some)
                    .map_err(Rejected::Construction)
            }
            Op::Doctype(_, n, public, system) => {
                let value = HtmlDoctype::new(
                    DocumentString::from_utf16(n.clone()),
                    public
                        .as_ref()
                        .map(|v| DocumentString::from_utf16(v.clone())),
                    system
                        .as_ref()
                        .map(|v| DocumentString::from_utf16(v.clone())),
                );
                both!(self, b, b.append_doctype(p, value))
                    .map(Some)
                    .map_err(Rejected::Construction)
            }
            Op::Contents(_) => {
                both!(self, b, b.template_contents(p)).map_err(Rejected::Construction)
            }
            Op::Shadow(_, bits) => {
                both!(self, b, b.attach_declarative_shadow(p, properties(*bits)))
                    .map(Some)
                    .map_err(Rejected::Construction)
            }
        }
    }
}
fn properties(bits: u8) -> HtmlShadowRootProperties {
    HtmlShadowRootProperties::new(
        if bits & 1 == 0 {
            HtmlShadowRootMode::Open
        } else {
            HtmlShadowRootMode::Closed
        },
        if bits & 2 == 0 {
            HtmlSlotAssignment::Named
        } else {
            HtmlSlotAssignment::Manual
        },
        bits & 4 != 0,
        bits & 8 != 0,
        bits & 16 != 0,
    )
}
#[derive(Debug)]
enum Rejected {
    Construction(HtmlConstructionError),
    Pi(HtmlProcessingInstructionError),
    Duplicate,
    Consumer,
}
/// The oracle key of a construction refusal, by the owner's cause. Every cause the fuzz
/// recipes do not provoke shares one key.
fn construction_key(error: &HtmlConstructionError) -> &'static str {
    match error.cause() {
        HtmlConstructionCause::ForeignBuilder => "foreign-builder",
        HtmlConstructionCause::ParentCannotHaveChildren => "leaf-parent",
        HtmlConstructionCause::TemplateContentsRequired => "template-parent",
        HtmlConstructionCause::ShadowHostMustBeElement => "shadow-host",
        HtmlConstructionCause::ShadowAlreadyAttached => "shadow-duplicate",
        HtmlConstructionCause::LimitExceeded { .. } => "construction-limit",
        HtmlConstructionCause::RepresentationExhausted => "construction-representation",
        HtmlConstructionCause::AllocationFailed => "construction-allocation",
        _ => "other-construction",
    }
}
/// Whether a refusal is a resource refusal of the environment rather than of
/// a limit: an allocation that failed or a count that cannot be represented.
fn is_environment_refusal(error: &HtmlConstructionError) -> bool {
    matches!(
        error.cause(),
        HtmlConstructionCause::AllocationFailed | HtmlConstructionCause::RepresentationExhausted
    )
}
/// Whether a refusal is exactly the limit failure of this resource, configured
/// limit and attempted total.
fn is_limit(
    error: &HtmlConstructionError,
    resource: HtmlConstructionResource,
    limit: usize,
    attempted: usize,
) -> bool {
    error.cause()
        == HtmlConstructionCause::LimitExceeded {
            resource,
            limit,
            attempted,
        }
}
impl Rejected {
    fn key(&self) -> &'static str {
        match self {
            Self::Construction(error) => construction_key(error),
            Self::Pi(HtmlProcessingInstructionError::InvalidTarget) => "pi-target",
            Self::Pi(HtmlProcessingInstructionError::ReservedTarget) => "pi-reserved",
            Self::Pi(HtmlProcessingInstructionError::DataContainsGreaterThan) => "pi-greater",
            Self::Pi(HtmlProcessingInstructionError::DataEndsWithQuestionMark) => "pi-question",
            // The error enum is open to later violations; one this oracle does
            // not name is a mismatch against every prediction.
            Self::Pi(_) => "other-pi",
            Self::Duplicate => "attribute-duplicate",
            Self::Consumer => "consumer-allocation",
        }
    }
}
impl Frozen {
    fn root(&self) -> HtmlNode<'_> {
        match self {
            Self::Document(d) => d.root(),
            Self::Fragment(f) => f.root(),
        }
    }
    fn node<'a>(&'a self, id: &HtmlNodeId) -> std::result::Result<HtmlNode<'a>, ForeignHtmlResult> {
        match self {
            Self::Document(d) => d.node(id),
            Self::Fragment(f) => f.node(id),
        }
    }
    fn clone_view(&self) -> Self {
        match self {
            Self::Document(d) => Self::Document(d.clone()),
            Self::Fragment(f) => Self::Fragment(f.clone()),
        }
    }
}
fn name_matches(actual: &QualifiedName, expected: &Name) -> bool {
    let (u, p, l) = expected.values();
    actual
        .namespace()
        .uri()
        .map_or(&[][..], DocumentString::code_units)
        == u
        && actual.prefix().map(DocumentString::code_units) == p
        && actual.local_name().code_units() == l
}
fn payload(actual: &HtmlNodeKind, expected: &Expected, c: &NativeCase) -> bool {
    if expected.fragment {
        return matches!(actual, HtmlNodeKind::Fragment);
    }
    let Some(i) = expected.op else {
        return match actual {
            HtmlNodeKind::Document(m) => {
                c.root != 0
                    && *m
                        == [
                            HtmlDocumentMode::NoQuirks,
                            HtmlDocumentMode::LimitedQuirks,
                            HtmlDocumentMode::Quirks,
                        ][usize::from(c.root - 1)]
            }
            HtmlNodeKind::Fragment => c.root == 0,
            _ => false,
        };
    };
    match (&c.ops[i], actual) {
        (Op::Element(_, n, a), HtmlNodeKind::Element(e)) => {
            name_matches(e.name(), n)
                && e.attributes().len() == a.len()
                && e.attributes().iter().zip(a).all(|(actual, expected)| {
                    let p = expected.values();
                    actual
                        .name()
                        .namespace()
                        .uri()
                        .map_or(&[][..], DocumentString::code_units)
                        == p.uri
                        && actual.name().prefix().map(DocumentString::code_units) == p.prefix
                        && actual.name().local_name().code_units() == p.local
                        && actual.value().code_units() == p.value
                })
        }
        (Op::Text(_, v), HtmlNodeKind::Text(a)) | (Op::Comment(_, v), HtmlNodeKind::Comment(a)) => {
            a.code_units() == v
        }
        (Op::Pi(_, t, d), HtmlNodeKind::ProcessingInstruction(a)) => {
            a.target().code_units() == t && a.data().code_units() == d
        }
        (Op::Doctype(_, n, p, s), HtmlNodeKind::Doctype(a)) => {
            a.name().code_units() == n
                && a.public_id().map(DocumentString::code_units) == p.as_deref()
                && a.system_id().map(DocumentString::code_units) == s.as_deref()
        }
        _ => false,
    }
}
fn assign(ids: &mut [Option<HtmlNodeId>], index: usize, id: HtmlNodeId) -> bool {
    if ids
        .iter()
        .enumerate()
        .any(|(i, existing)| i != index && existing.as_ref() == Some(&id))
    {
        return false;
    }
    if let Some(old) = &ids[index] {
        return *old == id;
    }
    ids[index] = Some(id);
    true
}
fn inspect(
    f: &Frozen,
    expected: &[Expected],
    c: &NativeCase,
) -> std::result::Result<(), &'static str> {
    let mut ids = Vec::new();
    reserve(&mut ids, 64).map_err(|_| "observation-capacity")?;
    ids.resize_with(expected.len(), || None);
    ids[0] = Some(f.root().id());
    for (i, e) in expected.iter().enumerate() {
        let id = *ids[i].as_ref().ok_or("missing-recipe-node")?;
        let n = f.node(&id).map_err(|_| "foreign-native-node")?;
        if !payload(n.kind(), e, c) || n.parent() != e.parent.and_then(|p| ids[p]) {
            return Err("native-payload-or-parent");
        }
        let owner = e.owner.map(|(p, shadow)| {
            if shadow {
                HtmlFragmentOwner::ShadowHost(*ids[p].as_ref().expect("earlier owner"))
            } else {
                HtmlFragmentOwner::Template(*ids[p].as_ref().expect("earlier owner"))
            }
        });
        if n.fragment_owner() != owner {
            return Err("native-owner");
        }
        if n.children().len() != e.children.len() {
            return Err("native-child-count");
        }
        for (child, index) in n.children().zip(&e.children) {
            if !assign(&mut ids, *index, child) {
                return Err("native-alias");
            }
        }
        match (e.template, n.template_contents()) {
            (None, None) => {}
            (Some(index), Some(id)) => {
                if !assign(&mut ids, index, id) {
                    return Err("template-alias");
                }
            }
            _ => return Err("native-template"),
        }
        match (e.shadow, n.declarative_shadow_root()) {
            (None, None) => {}
            (Some((index, bits)), Some(shadow)) => {
                if shadow.host() != id
                    || shadow.properties() != properties(bits)
                    || !assign(&mut ids, index, shadow.contents())
                {
                    return Err("native-shadow");
                }
            }
            _ => return Err("native-shadow"),
        }
    }
    let clone = f.clone_view();
    if clone.root().id() != f.root().id() {
        return Err("cloned-identity");
    }
    for id in ids.iter().flatten() {
        if clone.node(id).map_err(|_| "clone-lending")?.kind()
            != f.node(id).map_err(|_| "original-lending")?.kind()
        {
            return Err("clone-payload");
        }
    }
    let other = HtmlFragment::builder(HtmlConstructionLimits::new(1, 0, 0, 0, 0))
        .map_err(|_| "foreign-probe-allocation")?
        .finish()
        .map_err(|_| "foreign-probe-allocation")?;
    if other.root().id() == f.root().id()
        || !matches!(other.node(&f.root().id()), Err(ForeignHtmlResult))
        || f.node(&other.root().id()).is_ok()
    {
        return Err("foreign-result-identity");
    }
    Ok(())
}
/// What the public contract requires of one append, stated from the recipe and
/// the documented admission order (foreign handle, parent kind, then each
/// resource in declaration order), never from the builder's reply.
#[derive(Debug, PartialEq)]
enum Prediction {
    Reject(&'static str),
    Limit {
        resource: HtmlConstructionResource,
        limit: usize,
        attempted: usize,
    },
    Accept,
}
struct Totals {
    nodes: usize,
    edges: usize,
    attributes: usize,
    units: usize,
    depth: usize,
}
fn duplicate_attribute(attrs: &[Attr]) -> bool {
    // Expanded name: namespace and local name; the prefix is not identity.
    attrs.iter().enumerate().any(|(i, a)| {
        let x = a.values();
        attrs[..i].iter().any(|b| {
            let y = b.values();
            x.uri == y.uri && x.local == y.local
        })
    })
}
fn instruction_error(target: &[u16], data: &[u16]) -> Option<&'static str> {
    // HtmlAuthoredProcessingInstruction: ASCII name start, ASCII name rest,
    // reserved XML spellings, then data with no `>` and no final `?`.
    let alpha = |u: u16| matches!(u, 65..=90 | 97..=122);
    if !target.first().is_some_and(|u| alpha(*u) || *u == 95)
        || !target
            .iter()
            .all(|u| alpha(*u) || matches!(*u, 48..=57 | 45 | 95))
    {
        return Some("pi-target");
    }
    let lower: Vec<u8> = target
        .iter()
        .map(|u| u8::try_from(*u).unwrap_or(0).to_ascii_lowercase())
        .collect();
    if lower == b"xml" || lower == b"xml-stylesheet" {
        return Some("pi-reserved");
    }
    if data.contains(&62) {
        return Some("pi-greater");
    }
    if data.last() == Some(&63) {
        return Some("pi-question");
    }
    None
}
fn predict(o: &Op, parent: Option<&Expected>, c: &NativeCase, totals: &Totals) -> Prediction {
    match o {
        Op::Element(_, _, attrs) if duplicate_attribute(attrs) => {
            return Prediction::Reject("attribute-duplicate");
        }
        Op::Pi(_, t, d) => {
            if let Some(key) = instruction_error(t, d) {
                return Prediction::Reject(key);
            }
        }
        _ => {}
    }
    let Some(parent) = parent else {
        return Prediction::Reject("foreign-builder");
    };
    let leaf = parent
        .op
        .is_some_and(|i| !matches!(c.ops[i], Op::Element(..)));
    let (nodes, attributes, units, depth) = match o {
        Op::Contents(_) => return Prediction::Accept,
        Op::Shadow(..) => {
            if parent.op.is_none() || leaf {
                return Prediction::Reject("shadow-host");
            }
            if parent.shadow.is_some() {
                return Prediction::Reject("shadow-duplicate");
            }
            (1, 0, 0, parent.depth + 1)
        }
        _ => {
            if leaf {
                return Prediction::Reject("leaf-parent");
            }
            if parent.template.is_some() {
                return Prediction::Reject("template-parent");
            }
            match o {
                Op::Element(_, name, attrs) => {
                    let t = usize::from(name.template());
                    (1 + t, attrs.len(), o.units(), parent.depth + 1 + t)
                }
                _ => (1, 0, o.units(), parent.depth + 1),
            }
        }
    };
    for (resource, limit, attempted) in [
        (
            HtmlConstructionResource::Nodes,
            c.caps[0],
            totals.nodes + nodes,
        ),
        (
            HtmlConstructionResource::Edges,
            c.caps[1],
            totals.edges + nodes,
        ),
        (
            HtmlConstructionResource::Attributes,
            c.caps[2],
            totals.attributes + attributes,
        ),
        (
            HtmlConstructionResource::Utf16Units,
            c.caps[3],
            totals.units + units,
        ),
        (
            HtmlConstructionResource::Depth,
            c.caps[4],
            totals.depth.max(depth),
        ),
    ] {
        if attempted > limit {
            return Prediction::Limit {
                resource,
                limit,
                attempted,
            };
        }
    }
    Prediction::Accept
}
pub(super) fn evaluate(t: &NativeTarget, c: &NativeCase) -> Result<CaseOutcome> {
    t.credit(64, 8192, 64)?;
    let mut b = match Builder::new(c) {
        Ok(b) => b,
        Err(e) if c.caps[0] == 0 && is_limit(&e, HtmlConstructionResource::Nodes, 0, 1) => {
            return CaseOutcome::expected_rejection(
                vec![t.descriptor.properties()[2].clone()],
                "native root node denied",
            );
        }
        Err(e) if is_environment_refusal(&e) => {
            return CaseOutcome::expected_rejection(
                vec![t.descriptor.properties()[2].clone()],
                "typed native root resource refusal",
            );
        }
        Err(e) => return t.failure("native-root", format!("{e:?}")),
    };
    let foreign = HtmlFragment::builder(HtmlConstructionLimits::new(1, 0, 0, 0, 0))
        .map_err(|_| resource("foreign builder probe"))?;
    let foreign_handle = foreign.root();
    let mut handles = Vec::new();
    reserve(&mut handles, 65)?;
    handles.push(Some(b.root()));
    let mut slots = Vec::new();
    reserve(&mut slots, 65)?;
    slots.push(Some(0));
    let mut expected = Vec::new();
    reserve(&mut expected, 64)?;
    expected.push(Expected::new(None, None, 0, c.root == 0)?);
    let mut units = 0;
    let mut attrs = 0;
    let mut depth = 0;
    let mut resource_refusal = false;
    for (i, o) in c.ops.iter().enumerate() {
        let p = o.parent();
        let (parent, parent_slot) = if p == 255 {
            (&foreign_handle, None)
        } else if let (Some(h), Some(s)) = (&handles[usize::from(p)], slots[usize::from(p)]) {
            (h, Some(s))
        } else {
            // The named handle was refused or does not exist (an ordinary
            // element has no template contents). This dependency chain stops;
            // later independent commands still run.
            handles.push(None);
            slots.push(None);
            continue;
        };
        let before = b.usage();
        let totals = Totals {
            nodes: expected.len(),
            edges: expected.len() - 1,
            attributes: attrs,
            units,
            depth,
        };
        if [
            before.nodes(),
            before.edges(),
            before.attributes(),
            before.utf16_units(),
            before.depth(),
        ] != [
            totals.nodes,
            totals.edges,
            totals.attributes,
            totals.units,
            totals.depth,
        ] {
            return t.failure("native-admitted-usage", "recipe count mismatch");
        }
        let prediction = predict(o, parent_slot.map(|s| &expected[s]), c, &totals);
        let outcome = b.apply(o, parent);
        if let Err(ref e) = outcome {
            if b.usage() != before {
                return t.failure("rejected-append-mutated", e.key());
            }
            if matches!(e, Rejected::Consumer) {
                return Err(resource("native attribute preparation"));
            }
            if let Some(id) = c.literal
                && seeds::rejection(id, i) != Some(e.key())
            {
                return t.failure("literal-rejection", format!("{id}/{i}: {}", e.key()));
            }
            match (&prediction, e) {
                (Prediction::Reject(key), e) if *key == e.key() => {}
                (
                    Prediction::Limit {
                        resource,
                        limit,
                        attempted,
                    },
                    Rejected::Construction(error),
                ) if is_limit(error, *resource, *limit, *attempted) => resource_refusal = true,
                (Prediction::Accept, Rejected::Construction(error))
                    if is_environment_refusal(error) =>
                {
                    resource_refusal = true
                }
                _ => {
                    return t.failure(
                        "native-rejection-mismatch",
                        format!("predicted {prediction:?}; builder refused {}", e.key()),
                    );
                }
            }
            handles.push(None);
            slots.push(None);
            continue;
        }
        if prediction != Prediction::Accept {
            return t.failure(
                "native-missing-rejection",
                format!("predicted {prediction:?}; builder accepted command {i}"),
            );
        }
        if let Some(id) = c.literal
            && seeds::rejection(id, i).is_some()
        {
            return t.failure("missing-literal-rejection", format!("{id}/{i}"));
        }
        let Ok(handle) = outcome else {
            return t.failure("native-outcome", "outcome changed after inspection");
        };
        let parent_index = parent_slot.ok_or_else(|| invalid("foreign success"))?;
        let index = match o {
            Op::Contents(_) => expected[parent_index].template,
            Op::Shadow(_, bits) => {
                let index = expected.len();
                let d = expected[parent_index].depth + 1;
                let mut e = Expected::new(None, None, d, true)?;
                e.owner = Some((parent_index, true));
                expected.push(e);
                expected[parent_index].shadow = Some((index, *bits));
                depth = depth.max(d);
                Some(index)
            }
            _ => {
                let index = expected.len();
                let d = expected[parent_index].depth + 1;
                expected.push(Expected::new(Some(i), Some(parent_index), d, false)?);
                expected[parent_index].children.push(index);
                units += o.units();
                depth = depth.max(d);
                if let Op::Element(_, name, a) = o {
                    attrs += a.len();
                    if name.template() {
                        let f = expected.len();
                        let mut e = Expected::new(None, None, d + 1, true)?;
                        e.owner = Some((index, false));
                        expected.push(e);
                        expected[index].template = Some(f);
                        depth = depth.max(d + 1);
                    }
                }
                Some(index)
            }
        };
        if handle.is_some() != index.is_some() {
            return t.failure("native-handle-recipe", "optional contents mismatch");
        }
        handles.push(handle);
        slots.push(index);
        let usage = b.usage();
        if [
            usage.nodes(),
            usage.edges(),
            usage.attributes(),
            usage.utf16_units(),
            usage.depth(),
        ] != [expected.len(), expected.len() - 1, attrs, units, depth]
        {
            return t.failure("native-admitted-usage", "recipe count mismatch");
        }
    }
    let Ok(frozen) = b.finish() else {
        return t.failure(
            "native-finish",
            "publication refused a recipe the appends admitted",
        );
    };
    drop(foreign); // Borrowed foreign handles cannot change frozen identity.
    if let Err(key) = inspect(&frozen, &expected, c) {
        if matches!(key, "observation-capacity" | "foreign-probe-allocation") {
            return Err(resource("native observation allocation"));
        }
        return t.failure(key, "public frozen native recipe mismatch");
    }
    // The frozen result enters the semantic capture adapter under facts and at
    // most one fault chosen by the recipe's shape, with the adapter's contract
    // (mirror, fact qualification, atomic refusal) checked by the shared owner.
    let result = match &frozen {
        Frozen::Document(d) => HtmlConstructionResult::Document(d.clone()),
        Frozen::Fragment(f) => HtmlConstructionResult::Fragment(f.clone()),
    };
    let plan = Plan {
        scripting: (c.ops.len() % 3) as u8,
        registry: (usize::from(c.root) % 3) as u8,
        is_value: ((c.ops.len() / 3) % 3) as u8,
        shadow: (c.ops.len() % 8) as u8,
        fault: Fault::ALL[(c.ops.len() + usize::from(c.root)) % Fault::ALL.len()],
        pick: c.ops.len() as u8,
    };
    let mut bounded = false;
    match check(&result, &plan)? {
        Verdict::Violation { key, detail, .. } => return t.capture_failure(key, detail),
        Verdict::ResourceStop => bounded = true,
        Verdict::Verified => {}
    }
    if c.literal.is_some() || !(resource_refusal || bounded) {
        CaseOutcome::evaluated(t.properties(c.literal.is_some()))
    } else {
        CaseOutcome::expected_rejection(
            t.properties(false),
            "verified native resource refusal and retained prefix",
        )
    }
}
