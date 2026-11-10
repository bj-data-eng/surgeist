//! Bounded observations through immutable public owners, never private parser copies.
use crate::{Lane, ParseKind, UnicodeCase, UnicodeSource, UnicodeTarget, error};
use std::collections::HashSet;
use surgeist_fuzz::*;
use surgeist_html::{
    construction::{
        ForeignHtmlResult, HtmlDocumentMode, HtmlFragmentOwner, HtmlNode, HtmlNodeId, HtmlNodeKind,
        HtmlShadowRootProperties,
    },
    parsing::HtmlParserScriptingMode,
    values::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName},
};
use surgeist_html_parser::{
    DetailSet, HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseAssociationRelation,
    HtmlParseCancellation, HtmlParseCause, HtmlParseConstructionOperation, HtmlParseDestination,
    HtmlParseDetailUnavailable, HtmlParseDiagnosticContext, HtmlParseDiagnosticKind,
    HtmlParseDiagnosticLocationRole, HtmlParseDiagnosticRule, HtmlParseDiagnosticSeverity,
    HtmlParseDiagnostics, HtmlParseFailure, HtmlParseFailureReason, HtmlParseField,
    HtmlParseLexicalDisposition, HtmlParseLexicalKind, HtmlParseLexicalRole, HtmlParseLimits,
    HtmlParseOmissionKind, HtmlParseOriginKind, HtmlParseRawField, HtmlParseReport,
    HtmlParseRequestError, HtmlParseResource, HtmlParseStop, HtmlParseTokenFamily,
    HtmlParseUnsupportedState, HtmlParserRule, HtmlPrologueDiagnostic, HtmlSource, HtmlSourceRange,
    HtmlTokenizerDiagnostic, HtmlTokenizerDiagnosticPhase, HtmlTokenizerState,
};
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureError, HtmlCaptureFact, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCaptureTemplateCacheInput, HtmlCapturedIsValue,
    HtmlDocumentClass, HtmlFragmentContext, HtmlImageSupport, HtmlScriptingStatus,
    HtmlTemplateDocumentRole,
};
const MAX_NODES: usize = 100_000;
const MAX_ROWS: usize = 100_000;

/// Property slots shared by every parse target: 0 source preservation,
/// 1 native report, 2 diagnostic provenance, 3 terminal ownership and 4
/// immutability after cancellation. Later slots belong to the target.
pub(crate) struct Props<'a>(pub(crate) &'a TargetDescriptor);
impl Props<'_> {
    pub(crate) fn id(&self, property: usize) -> PropertyId {
        self.0.properties()[property].clone()
    }
    pub(crate) fn ids(&self, properties: &[usize]) -> Vec<PropertyId> {
        properties.iter().map(|i| self.id(*i)).collect()
    }
    pub(crate) fn failure(
        &self,
        property: usize,
        key: &str,
        detail: impl Into<String>,
    ) -> Result<CaseOutcome> {
        CaseOutcome::failure(
            FailureKey::new(
                self.id(property),
                if key.ends_with("-limit") {
                    FailureClass::Resource
                } else {
                    FailureClass::PropertyViolation
                },
                Some(key.into()),
            )?,
            detail,
        )
    }
}
fn source(case: &UnicodeCase) -> HtmlSource {
    match &case.source {
        UnicodeSource::Utf8(s) => HtmlSource::utf8(s.clone()),
        UnicodeSource::Utf16(s) => HtmlSource::utf16(DocumentString::from_utf16(s.clone())),
    }
}
pub(crate) fn context(missing: bool) -> Result<HtmlFragmentContext> {
    context_for(Namespace::html(), "div", missing)
}
/// A captured context element of the given namespace and local name in an
/// ordinary no-quirks HTML document with disabled (or, when `missing`,
/// unavailable) scripting and a null registry.
pub(crate) fn context_for(
    namespace: Namespace,
    local: &str,
    missing: bool,
) -> Result<HtmlFragmentContext> {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(2, 4, 0, 128, 8, 0, 0, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            if missing {
                HtmlScriptingStatus::Unavailable
            } else {
                HtmlScriptingStatus::Disabled
            },
            HtmlCaptureRegistryInput::Null,
        )
        .map_err(|e| {
            FuzzError::with_source(
                FuzzErrorKind::Harness,
                "build fixed fragment capture",
                "capture document",
                e,
            )
        })?;
    b.establish_document_template_facts(
        &d,
        HtmlDocumentClass::Html,
        HtmlTemplateDocumentRole::Ordinary,
        HtmlCaptureTemplateCacheInput::Absent,
    )
    .map_err(|e| {
        FuzzError::with_source(
            FuzzErrorKind::Harness,
            "build fixed fragment capture",
            "capture template facts",
            e,
        )
    })?;
    // Image execution support is independently explicit for the parser-created
    // temporary document; target-document facts are not substituted for it.
    b.establish_parser_creation_image_support(&d, HtmlImageSupport::Disabled)
        .map_err(|e| {
            FuzzError::with_source(
                FuzzErrorKind::Harness,
                "build fixed fragment capture",
                "creation image fact",
                e,
            )
        })?;
    let e = b
        .add_element(
            &d,
            QualifiedName::new(namespace, None, local),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .map_err(|e| {
            FuzzError::with_source(
                FuzzErrorKind::Harness,
                "build fixed fragment capture",
                "capture div",
                e,
            )
        })?;
    b.establish_root(&e).map_err(|e| {
        FuzzError::with_source(
            FuzzErrorKind::Harness,
            "build fixed fragment capture",
            "capture root",
            e,
        )
    })?;
    b.establish_no_shadow_root(&e).map_err(|e| {
        FuzzError::with_source(
            FuzzErrorKind::Harness,
            "build fixed fragment capture",
            "capture no shadow",
            e,
        )
    })?;
    let c = b.finish().map_err(|e| {
        FuzzError::with_source(
            FuzzErrorKind::Harness,
            "build fixed fragment capture",
            "capture finish",
            e,
        )
    })?;
    HtmlFragmentContext::element(
        &c,
        &c.subject_id(&e).map_err(|e| {
            FuzzError::with_source(
                FuzzErrorKind::Harness,
                "build fixed fragment capture",
                "capture subject",
                e,
            )
        })?,
    )
    .map_err(|e| {
        FuzzError::with_source(
            FuzzErrorKind::Harness,
            "build fixed fragment capture",
            "fragment context",
            e,
        )
    })
}
/// An owned snapshot of an element's name, attributes and link history.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativeElement {
    name: QualifiedName,
    attributes: OrderedAttributes,
    link_flags: Option<surgeist_html::elements::link::HtmlLinkFlags>,
}
impl NativeElement {
    pub(crate) fn name(&self) -> &QualifiedName {
        &self.name
    }
    pub(crate) fn attributes(&self) -> &OrderedAttributes {
        &self.attributes
    }
}
/// An owned snapshot of a published node's content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NativeKind {
    Document(HtmlDocumentMode),
    Fragment,
    Element(NativeElement),
    Text(DocumentString),
    Comment(DocumentString),
    ProcessingInstruction(surgeist_html::construction::HtmlProcessingInstruction),
    Doctype(surgeist_html::construction::HtmlDoctype),
}
impl From<HtmlNodeKind<'_>> for NativeKind {
    fn from(kind: HtmlNodeKind<'_>) -> Self {
        match kind {
            HtmlNodeKind::Document(mode) => Self::Document(mode),
            HtmlNodeKind::Fragment => Self::Fragment,
            HtmlNodeKind::Element(element) => Self::Element(NativeElement {
                name: element.name().clone(),
                attributes: element.attributes().to_ordered(),
                link_flags: element.link_flags(),
            }),
            HtmlNodeKind::Text(value) => Self::Text(value),
            HtmlNodeKind::Comment(value) => Self::Comment(value),
            HtmlNodeKind::ProcessingInstruction(value) => Self::ProcessingInstruction(value),
            HtmlNodeKind::Doctype(value) => Self::Doctype(value),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativeNode {
    pub(crate) id: HtmlNodeId,
    pub(crate) kind: NativeKind,
    pub(crate) parent: Option<HtmlNodeId>,
    pub(crate) children: Vec<HtmlNodeId>,
    pub(crate) template: Option<HtmlNodeId>,
    pub(crate) shadow: Option<(HtmlNodeId, HtmlShadowRootProperties)>,
    pub(crate) owner: Option<HtmlFragmentOwner>,
}
pub(crate) fn node<'a>(
    report: &'a HtmlParseReport,
    id: &HtmlNodeId,
) -> std::result::Result<HtmlNode<'a>, ForeignHtmlResult> {
    match report.document() {
        Some(d) => d.node(id),
        None => report
            .fragment()
            .expect("qualified report fragment")
            .node(id),
    }
}
pub(crate) fn native(
    report: &HtmlParseReport,
) -> std::result::Result<Vec<NativeNode>, &'static str> {
    let root = if let Some(d) = report.document() {
        d.root()
    } else if let Some(f) = report.fragment() {
        f.root()
    } else {
        return Err("missing-native-root");
    };
    let mut stack = vec![(root.id(), None, None)];
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    while let Some((id, parent, owner)) = stack.pop() {
        if seen.len() >= MAX_NODES {
            return Err("native-node-limit");
        }
        if !seen.insert(id) {
            return Err("native-revisit");
        }
        let n = node(report, &id).map_err(|_| "foreign-native-id")?;
        if n.parent() != parent || n.fragment_owner() != owner {
            return Err("native-parent-owner");
        }
        if n.children().len() > MAX_NODES.saturating_sub(stack.len()) {
            return Err("native-child-limit");
        }
        let children: Vec<_> = n.children().collect();
        for child in children.iter().rev() {
            stack.push((*child, Some(id), None));
        }
        let template = n.template_contents();
        if let Some(ref f) = template {
            stack.push((*f, None, Some(HtmlFragmentOwner::Template(id))));
        }
        let shadow = n
            .declarative_shadow_root()
            .map(|s| (s.contents(), s.properties()));
        if let Some((ref f, _)) = shadow {
            stack.push((*f, None, Some(HtmlFragmentOwner::ShadowHost(id))));
        }
        if stack.len() > MAX_NODES {
            return Err("native-stack-limit");
        }
        out.push(NativeNode {
            id,
            kind: n.kind().into(),
            parent,
            children,
            template,
            shadow,
            owner,
        });
    }
    Ok(out)
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Diagnostic {
    kind: HtmlParseDiagnosticKind,
    context: HtmlParseDiagnosticContext,
    severity: HtmlParseDiagnosticSeverity,
    rule: HtmlParseDiagnosticRule,
    range: HtmlSourceRange,
    role: HtmlParseDiagnosticLocationRole,
    token: Option<HtmlParseTokenFamily>,
}
fn range(source: &HtmlSource, range: &HtmlSourceRange) -> bool {
    source
        .range(range.start(), range.end())
        .is_ok_and(|r| r == *range)
}
/// The retained diagnostics of a result and how many the `Diagnostics` cap
/// dropped. The cap bounds retention, so the dropped count is part of what a
/// published result must keep immutable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObservedDiagnostics {
    records: Vec<Diagnostic>,
    dropped: u64,
}
/// Whether `observed` obeys the configured `Diagnostics` cap: no more than the
/// cap is retained, and a dropped diagnostic means the cap was filled.
fn within_diagnostics_cap(observed: &ObservedDiagnostics, cap: u64) -> bool {
    let retained = observed.records.len() as u64;
    retained <= cap && (observed.dropped == 0 || retained == cap)
}
pub(crate) fn diagnostics(
    source: &HtmlSource,
    d: HtmlParseDiagnostics<'_>,
) -> std::result::Result<ObservedDiagnostics, &'static str> {
    if d.len() > MAX_ROWS {
        return Err("diagnostic-limit");
    }
    let mut out = Vec::with_capacity(d.len());
    for i in 0..d.len() {
        let (r, l) = d
            .retained_record(i)
            .map_err(|_| "diagnostic-unresolved")?
            .ok_or("diagnostic-lending")?;
        if !range(source, l.source_range()) {
            return Err("diagnostic-source-range");
        }
        out.push(Diagnostic {
            kind: r.kind(),
            context: r.context(),
            severity: r.severity(),
            rule: r.rule(),
            range: l.source_range().clone(),
            role: l.role(),
            token: l.token(),
        });
    }
    Ok(ObservedDiagnostics {
        records: out,
        dropped: d.dropped(),
    })
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Cause {
    source: Option<HtmlSourceRange>,
    operation: HtmlParseConstructionOperation,
    rule: Option<HtmlParserRule>,
    token: Option<HtmlParseTokenFamily>,
}
fn cause(source: &HtmlSource, c: HtmlParseCause<'_>) -> std::result::Result<Cause, &'static str> {
    if c.source().is_some_and(|r| !range(source, r)) {
        return Err("cause-source-range");
    }
    Ok(Cause {
        source: c.source().cloned(),
        operation: c.operation(),
        rule: c.rule(),
        token: c.token(),
    })
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Part {
    source: HtmlSourceRange,
    start: usize,
    end: usize,
    role: HtmlParseLexicalRole,
    origin: Option<HtmlParseOriginKind>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum Destination {
    Node(HtmlNodeId),
    Field(HtmlNodeId, HtmlParseField),
    Raw(HtmlNodeId, HtmlParseRawField),
    Lexical {
        source: HtmlSourceRange,
        kind: HtmlParseLexicalKind,
        disposition: HtmlParseLexicalDisposition,
        units: usize,
        part: Option<Part>,
    },
    Omission {
        source: HtmlSourceRange,
        start: usize,
        end: usize,
        kind: HtmlParseOmissionKind,
        rule: HtmlParserRule,
        origin: HtmlParseOriginKind,
    },
    End,
    Cause(Cause),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Association {
    source: Option<HtmlSourceRange>,
    start: usize,
    end: usize,
    origin: Option<HtmlParseOriginKind>,
    relation: HtmlParseAssociationRelation,
    cause: Cause,
    destination: Destination,
    group_units: Option<usize>,
}
fn field_units(n: HtmlNode<'_>, field: HtmlParseField) -> Option<usize> {
    match (n.kind(), field) {
        (HtmlNodeKind::Text(s), HtmlParseField::Text)
        | (HtmlNodeKind::Comment(s), HtmlParseField::Comment) => Some(s.len()),
        (HtmlNodeKind::ProcessingInstruction(p), HtmlParseField::ProcessingInstructionTarget) => {
            Some(p.target().len())
        }
        (HtmlNodeKind::ProcessingInstruction(p), HtmlParseField::ProcessingInstructionData) => {
            Some(p.data().len())
        }
        (HtmlNodeKind::Doctype(d), HtmlParseField::DoctypeName) => Some(d.name().len()),
        (HtmlNodeKind::Doctype(d), HtmlParseField::DoctypePublic) => {
            d.public_id().map(DocumentString::len)
        }
        (HtmlNodeKind::Doctype(d), HtmlParseField::DoctypeSystem) => {
            d.system_id().map(DocumentString::len)
        }
        (HtmlNodeKind::Element(e), HtmlParseField::ElementName) => {
            Some(e.name().local_name().len())
        }
        (HtmlNodeKind::Element(e), HtmlParseField::Namespace) => {
            Some(e.name().namespace().uri().map_or(0, DocumentString::len))
        }
        (HtmlNodeKind::Element(e), HtmlParseField::AttributeName(i)) => {
            e.attributes().get(i).map(|a| a.name().local_name().len())
        }
        (HtmlNodeKind::Element(e), HtmlParseField::AttributeValue(i)) => {
            e.attributes().get(i).map(|a| a.value().len())
        }
        (HtmlNodeKind::Element(e), HtmlParseField::AttributeNamespace(i)) => e
            .attributes()
            .get(i)
            .map(|a| a.name().namespace().uri().map_or(0, DocumentString::len)),
        (HtmlNodeKind::Element(e), HtmlParseField::AttributePrefix(i)) => e
            .attributes()
            .get(i)
            .map(|a| a.name().prefix().map_or(0, DocumentString::len)),
        // PI attributes are exposed by metadata rather than this native record.
        (
            _,
            HtmlParseField::ProcessingInstructionAttributeName(_)
            | HtmlParseField::ProcessingInstructionAttributeValue(_),
        ) => None,
        _ => None,
    }
}
/// The refusal of a build whose parser does not provide detail.
pub(crate) fn detail_error(error: HtmlParseDetailUnavailable) -> FuzzError {
    FuzzError::with_source(
        FuzzErrorKind::Harness,
        "select parse detail",
        "the detail feature of the parser",
        error,
    )
}
/// The diagnostics of a report whose request selected them.
fn report_diagnostics(
    report: &HtmlParseReport,
) -> std::result::Result<HtmlParseDiagnostics<'_>, &'static str> {
    report
        .detail()
        .and_then(|detail| detail.diagnostics())
        .ok_or("diagnostics-not-selected")
}
pub(crate) fn associations(
    report: &HtmlParseReport,
) -> std::result::Result<Vec<Association>, &'static str> {
    let detail = report.detail().ok_or("detail-not-selected")?;
    let rows = detail.associations().ok_or("associations-not-selected")?;
    if rows.len() > MAX_ROWS {
        return Err("association-limit");
    }
    let mut out = Vec::with_capacity(rows.len());
    for i in 0..rows.len() {
        let r = rows
            .row(i)
            .map_err(|_| "association-unresolved")?
            .ok_or("association-lending")?;
        if r.source().is_some_and(|s| !range(report.source(), s)) || r.start() > r.end() {
            return Err("association-source-interval");
        }
        let destination = match r.destination() {
            HtmlParseDestination::Node(n) => Destination::Node(n.id()),
            HtmlParseDestination::Field { node, field } => {
                if r.relation() == HtmlParseAssociationRelation::Output
                    && field_units(node, field).is_some_and(|len| r.end() > len)
                {
                    return Err("native-field-interval");
                }
                Destination::Field(node.id(), field)
            }
            HtmlParseDestination::Raw { node, kind } => Destination::Raw(node.id(), kind),
            HtmlParseDestination::Lexical { occurrence, part } => {
                if !range(report.source(), occurrence.source())
                    || part
                        .as_ref()
                        .is_some_and(|p| !range(report.source(), p.source()) || p.start() > p.end())
                {
                    return Err("lexical-source-interval");
                }
                Destination::Lexical {
                    source: occurrence.source().clone(),
                    kind: occurrence.kind(),
                    disposition: occurrence.disposition(),
                    units: occurrence.data_units(),
                    part: part.map(|p| Part {
                        source: p.source().clone(),
                        start: p.start(),
                        end: p.end(),
                        role: p.role(),
                        origin: p.origin_kind(),
                    }),
                }
            }
            HtmlParseDestination::Omission(o) => {
                if !range(report.source(), o.source()) || o.start() > o.end() {
                    return Err("omission-source-interval");
                }
                Destination::Omission {
                    source: o.source().clone(),
                    start: o.start(),
                    end: o.end(),
                    kind: o.kind(),
                    rule: o.rule(),
                    origin: o.origin_kind(),
                }
            }
            HtmlParseDestination::LexicalEnd => Destination::End,
            HtmlParseDestination::Cause(c) => Destination::Cause(cause(report.source(), c)?),
        };
        let history = detail.history().ok_or("history-not-selected")?;
        let group_units = r
            .group()
            .map(|g| {
                history
                    .header(g)
                    .map(|h| h.units)
                    .map_err(|_| "history-group-lending")
            })
            .transpose()?;
        out.push(Association {
            source: r.source().cloned(),
            start: r.start(),
            end: r.end(),
            origin: r.origin_kind(),
            relation: r.relation(),
            cause: cause(report.source(), r.cause())?,
            destination,
            group_units,
        });
    }
    Ok(out)
}
pub(crate) fn same_source(
    text: &UnicodeSource,
    actual: &HtmlSource,
    original: &HtmlSource,
) -> bool {
    actual.identity() == original.identity()
        && match text {
            UnicodeSource::Utf8(s) => actual.as_utf8() == Some(s.as_str()),
            UnicodeSource::Utf16(s) => actual.as_utf16().is_some_and(|a| a.code_units() == s),
        }
}
fn check_literal(id: u8, nodes: &[NativeNode]) -> bool {
    let elements = |name: &str| {
        nodes.iter().filter(move|n|matches!(&n.kind,NativeKind::Element(e) if e.name().local_name()==&DocumentString::from(name))).collect::<Vec<_>>()
    };
    let text = |n: &NativeNode, expected: &[u16]| {
        n.children.len() == 1
            && nodes.iter().any(|child| {
                n.children.contains(&child.id)
                    && matches!(&child.kind,NativeKind::Text(t) if t.code_units()==expected)
            })
    };
    let p = elements("p");
    match id {
        0=>p.len()==1 && text(p[0],&[65,38,66,10,67]) && matches!(&p[0].kind,NativeKind::Element(e) if e.attributes().as_slice()==[Attribute::new(QualifiedName::new(Namespace::none(),None,"id"),"x")]),
        1=>p.len()==1 && text(p[0],&[65,0xd800,10,66]) && nodes.iter().any(|n|matches!(&n.kind,NativeKind::Text(t) if t.code_units()==[0xfeff])),
        2=>p.len()==1 && text(p[0],&[65,66]),
        3=>{let a=elements("a");a.len()==1 && matches!(&a[0].kind,NativeKind::Element(e) if e.name().namespace()==&Namespace::svg() && e.attributes().as_slice()==[
            Attribute::new(QualifiedName::new(Namespace::from_uri("http://www.w3.org/1999/xlink"),Some("xlink".into()),"href"),"x"),
            Attribute::new(QualifiedName::new(Namespace::none(),None,"viewBox"),"0 0 1 1")])},
        4=>{let t=elements("template");t.len()==1 && p.len()==2 && p.iter().any(|n|text(n,&[84]) && n.parent==t[0].template) && p.iter().any(|n|text(n,&[90]) && n.parent!=t[0].template)},
        // HTML 877d 13.2.5.73-76: both `<?target data?>` and `<?target data>` yield target "target" and data "data".
        5|8=>p.len()==1 && text(p[0],&[80]) && nodes.iter().filter(|n|matches!(&n.kind,NativeKind::ProcessingInstruction(i) if i.target()==&DocumentString::from("target") && i.data()==&DocumentString::from("data"))).count()==1,
        6=>p.len()==2 && text(p[0],&[65]) && text(p[1],&[66]) && p[0].parent==p[1].parent,
        7=>{let t=elements("table");let x=nodes.iter().position(|n|matches!(&n.kind,NativeKind::Text(s) if s.code_units()==[88]));t.len()==1 && x.is_some_and(|i|nodes[i].parent==t[0].parent && i<nodes.iter().position(|n|n.id==t[0].id).expect("existing table"))},
        _=>false,
    }
}
fn literal_evidence(
    id: u8,
    report: &HtmlParseReport,
    rows: &[Association],
    diag: &[Diagnostic],
) -> bool {
    if id == 0 {
        for (start, end, a, b, origin) in [
            (1, 2, 14, 19, HtmlParseOriginKind::CharacterReference),
            (3, 4, 20, 22, HtmlParseOriginKind::Newline),
        ] {
            let expected = report.source().range(a, b).expect("fixed literal range");
            if !rows.iter().any(|r| {
                r.relation == HtmlParseAssociationRelation::Output
                    && r.start == start
                    && r.end == end
                    && r.source.as_ref() == Some(&expected)
                    && r.origin == Some(origin)
                    && matches!(r.destination, Destination::Field(_, HtmlParseField::Text))
            }) {
                return false;
            }
        }
    }
    if id == 1 {
        let expected = report
            .source()
            .range(5, 6)
            .expect("surrogate literal range");
        let surrogate = diag.iter().find(|d| {
            d.kind
                == HtmlParseDiagnosticKind::Tokenizer(
                    HtmlTokenizerDiagnostic::SurrogateInInputStream,
                )
        });
        if !surrogate.is_some_and(|d| {
            d.range == expected
                && d.context
                    == HtmlParseDiagnosticContext::Tokenizer {
                        phase: HtmlTokenizerDiagnosticPhase::InputStream,
                        state: HtmlTokenizerState::Data,
                    }
                && d.role == HtmlParseDiagnosticLocationRole::InputFrontier
                && d.token.is_none()
                && d.severity == HtmlParseDiagnosticSeverity::Recoverable
                && d.rule.anchor() == "parse-error-surrogate-in-input-stream"
                && d.rule.branch() == "surrogate-in-input-stream"
        }) {
            return false;
        }
        if report.document().is_some()
            && !(diag.len() == 2
                && diag[0].kind
                    == HtmlParseDiagnosticKind::Prologue(HtmlPrologueDiagnostic::MissingDoctype)
                && diag[0].range == report.source().range(0, 1).expect("BOM range")
                && diag[1].range == expected)
        {
            return false;
        }
    }
    true
}

/// `HtmlParseResource` in the order of the twenty configured caps.
const RESOURCE_ORDER: [HtmlParseResource; 20] = [
    HtmlParseResource::SourceUnits,
    HtmlParseResource::BufferedUtf16Units,
    HtmlParseResource::TokenAttributes,
    HtmlParseResource::AuxiliaryEntries,
    HtmlParseResource::ManagedNodes,
    HtmlParseResource::ManagedEdges,
    HtmlParseResource::ManagedAttributes,
    HtmlParseResource::ManagedUtf16Units,
    HtmlParseResource::SourceMapEntries,
    HtmlParseResource::SourceMapPayloadUnits,
    HtmlParseResource::Diagnostics,
    HtmlParseResource::DiagnosticPayloadUnits,
    HtmlParseResource::ContextEffects,
    HtmlParseResource::OpenElements,
    HtmlParseResource::ActiveFormattingEntries,
    HtmlParseResource::TreeDepth,
    HtmlParseResource::ManagedCapacityBytes,
    HtmlParseResource::PeakTransientBytes,
    HtmlParseResource::AtomicLayoutBytes,
    HtmlParseResource::WorkCredits,
];

/// The index of the `Diagnostics` cap in the configured caps: its position in
/// `RESOURCE_ORDER`, so a reordered resource list cannot silently point at
/// another cap. A list without it fails to build.
const DIAGNOSTICS_CAP: usize = {
    let mut index = 0;
    while index < RESOURCE_ORDER.len() {
        if RESOURCE_ORDER[index] as usize == HtmlParseResource::Diagnostics as usize {
            break;
        }
        index += 1;
    }
    assert!(index < RESOURCE_ORDER.len());
    index
};

/// The request-side facts a parse result must preserve.
pub(crate) struct Spec<'a> {
    pub(crate) fragment: bool,
    pub(crate) text: &'a UnicodeSource,
    pub(crate) original: &'a HtmlSource,
    pub(crate) limits: &'a HtmlParseLimits,
    /// The configured caps in `HtmlParseResource` order.
    pub(crate) caps: [u64; 20],
    pub(crate) requested: HtmlParserScriptingMode,
    pub(crate) capture: Option<&'a surgeist_html_semantics::HtmlCaptureId>,
    /// Declarative shadow permission selected for the fragment request.
    pub(crate) shadow_allowed: bool,
}
/// Typed observations of one published result, taken once and kept for the
/// later immutability comparison.
pub(crate) struct Verified {
    pub(crate) nodes: Vec<NativeNode>,
    pub(crate) diagnostics: ObservedDiagnostics,
    pub(crate) rows: Vec<Association>,
}
/// How a failed parse is classified when it is not an expected bounded stop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Expect {
    /// No refusal is acceptable (an independent oracle expects success).
    Never,
    /// Resource, allocation and representation stops are product outcomes;
    /// known unsupported states are unsupported, anything else is a failure.
    General,
    /// An independent oracle expects a tree. Only an allocation or
    /// representation refusal, or a limit stop on a resource the case lowered
    /// that reports the configured cap and an attempted total above it, is a
    /// product outcome; an unsupported
    /// state or any other stop is a failure.
    Balanced,
    PreCancelled,
    SourceDenied,
    WorkDenied,
}
/// Source binding, native traversal, typed diagnostic and D1 intervals of a
/// published report.
pub(crate) fn verify_report(
    props: &Props<'_>,
    spec: &Spec<'_>,
    report: &HtmlParseReport,
) -> Result<std::result::Result<Verified, CaseOutcome>> {
    if !same_source(spec.text, report.source(), spec.original)
        || report.limits() != spec.limits
        || report.requested_scripting_mode() != spec.requested
        || report.effective_scripting_mode() != HtmlParserScriptingMode::Disabled
        || report.allow_declarative_shadow_roots() != spec.shadow_allowed
        || report.image_support() != HtmlImageSupport::Disabled
        || report.document().is_some() == spec.fragment
        || report.fragment().is_some() != spec.fragment
        || report.context().map(|c| c.capture().identity()) != spec.capture
    {
        return props
            .failure(
                0,
                "source-binding",
                "successful report changed request binding",
            )
            .map(Err);
    }
    let nodes = match native(report) {
        Ok(n) => n,
        Err(key) => return props.failure(1, key, "native ownership/traversal").map(Err),
    };
    let diagnostics = match report_diagnostics(report)
        .and_then(|selected| diagnostics(report.source(), selected))
    {
        Ok(d) => d,
        Err(key) => return props.failure(2, key, "typed diagnostic range").map(Err),
    };
    if !within_diagnostics_cap(&diagnostics, spec.caps[DIAGNOSTICS_CAP]) {
        return props
            .failure(
                2,
                "diagnostic-cap-retention",
                "retained and dropped diagnostics disagree with the configured cap",
            )
            .map(Err);
    }
    let rows = match associations(report) {
        Ok(r) => r,
        Err(key) => return props.failure(2, key, "native D1 range").map(Err),
    };
    Ok(Ok(Verified {
        nodes,
        diagnostics,
        rows,
    }))
}
/// Late cancellation never changes a published result.
pub(crate) fn verify_immutable(
    props: &Props<'_>,
    report: &HtmlParseReport,
    seen: &Verified,
    cancel: &HtmlParseCancellation,
) -> Result<Option<CaseOutcome>> {
    let usage = report.usage().clone();
    cancel.cancel();
    if report.usage() != &usage
        || native(report).as_ref() != Ok(&seen.nodes)
        || report_diagnostics(report)
            .and_then(|selected| diagnostics(report.source(), selected))
            .as_ref()
            != Ok(&seen.diagnostics)
        || associations(report).as_ref() != Ok(&seen.rows)
    {
        return props
            .failure(
                4,
                "published-report-mutated",
                "cancellation changed immutable published values",
            )
            .map(Some);
    }
    Ok(None)
}
/// Ownership, first stop, cleanup and expected-bound classification of a
/// failed parse.
pub(crate) fn verify_failure(
    props: &Props<'_>,
    spec: &Spec<'_>,
    f: &HtmlParseFailure,
    cancel: &HtmlParseCancellation,
    expect: Expect,
) -> Result<CaseOutcome> {
    if !same_source(spec.text, f.source(), spec.original)
        || f.limits() != spec.limits
        || f.requested_scripting_mode() != spec.requested
        || f.allow_declarative_shadow_roots() != spec.shadow_allowed
        || f.context().map(|c| c.capture().identity()) != spec.capture
    {
        return props.failure(
            0,
            "failed-source-binding",
            "failure changed original request",
        );
    }
    if f.cleanup_error().is_some() {
        return props.failure(3, "cleanup-error", "typed cleanup engine error");
    }
    if let Some(d) = f.diagnostics() {
        match diagnostics(f.source(), d) {
            Err(key) => return props.failure(2, key, "failure diagnostic range"),
            Ok(observed) if !within_diagnostics_cap(&observed, spec.caps[DIAGNOSTICS_CAP]) => {
                return props.failure(
                    2,
                    "diagnostic-cap-retention",
                    "retained and dropped diagnostics disagree with the configured cap",
                );
            }
            Ok(_) => {}
        }
    }
    // The `Diagnostics` cap bounds retention only: no parse stops on it.
    if let HtmlParseFailureReason::Stop(HtmlParseStop::LimitExceeded {
        resource: HtmlParseResource::Diagnostics,
        ..
    }) = f.reason()
    {
        return props.failure(
            3,
            "diagnostic-cap-terminal",
            "the diagnostics cap stopped a parse instead of bounding retention",
        );
    }
    let failure_diags = f
        .diagnostics()
        .map(|d| diagnostics(f.source(), d))
        .transpose()
        .map_err(error)?;
    let admission = f.source_admitted();
    let reason = f.reason();
    let first = f.first_stop();
    let at = f.at_stop().cloned();
    let after = f.after_cleanup().cloned();
    let retained = f.retained_usage();
    if let Some(ref usage) = after {
        for resource in [
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::TokenAttributes,
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::OpenElements,
        ] {
            if usage.resource(resource).current() != 0 {
                return props.failure(
                    3,
                    "live-storage-after-cleanup",
                    "disposable storage remains owned",
                );
            }
        }
    }
    cancel.cancel();
    if f.diagnostics()
        .map(|d| diagnostics(f.source(), d))
        .transpose()
        .map_err(error)?
        != failure_diags
        || f.source_admitted() != admission
        || f.reason() != reason
        || f.first_stop() != first
        || f.at_stop() != at.as_ref()
        || f.after_cleanup() != after.as_ref()
        || f.retained_usage() != retained
    {
        return props.failure(
            4,
            "first-stop-mutated",
            "late cancel changed retained terminal evidence",
        );
    }
    if let HtmlParseFailureReason::Stop(stop) = reason
        && first != Some(stop)
    {
        return props.failure(
            3,
            "stop-disagrees",
            "canonical first stop differs from terminal reason",
        );
    }
    let expected = match expect {
        Expect::PreCancelled => reason == HtmlParseFailureReason::Stop(HtmlParseStop::Cancelled),
        Expect::SourceDenied => {
            matches!(
                reason,
                HtmlParseFailureReason::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::SourceUnits,
                    ..
                })
            ) && f.source_admitted() == Some(false)
        }
        Expect::WorkDenied => matches!(
            reason,
            HtmlParseFailureReason::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })
        ),
        Expect::Never => false,
        Expect::Balanced => match reason {
            HtmlParseFailureReason::Stop(
                HtmlParseStop::RepresentationExhausted { .. }
                | HtmlParseStop::AllocationFailed { .. },
            ) => true,
            HtmlParseFailureReason::Stop(HtmlParseStop::LimitExceeded {
                resource,
                limit,
                current,
                requested_delta,
            }) => {
                let index = RESOURCE_ORDER.iter().position(|r| *r == resource);
                let configured = index.and_then(|i| spec.caps.get(i));
                // A stop is a product outcome only where the case lowered the
                // resource below the documented envelope: a stop on an unlowered
                // cap contradicts an oracle that expects the tree.
                let lowered = index
                    .and_then(|i| crate::codec::CAPS.get(i))
                    .is_some_and(|default| *default != limit);
                if configured != Some(&limit)
                    || current.saturating_add(requested_delta) <= limit
                    || !lowered
                {
                    return props.failure(
                        3,
                        "stop-limit-mismatch",
                        format!("{reason:?} against configured caps {:?}", spec.caps),
                    );
                }
                true
            }
            _ => false,
        },
        Expect::General => matches!(
            reason,
            HtmlParseFailureReason::Stop(
                HtmlParseStop::LimitExceeded { .. }
                    | HtmlParseStop::RepresentationExhausted { .. }
                    | HtmlParseStop::AllocationFailed { .. }
            )
        ),
    };
    if expected {
        return CaseOutcome::expected_rejection(props.ids(&[0, 3, 4]), format!("{reason:?}"));
    }
    if expect == Expect::General
        && matches!(
            reason,
            HtmlParseFailureReason::UnsupportedConstruction(_)
                | HtmlParseFailureReason::UnsupportedContextReactions
                | HtmlParseFailureReason::MissingContextFact(_)
                | HtmlParseFailureReason::Stop(HtmlParseStop::UnsupportedStandaloneState(
                    HtmlParseUnsupportedState::ScriptDocument
                        | HtmlParseUnsupportedState::ScriptPause
                        | HtmlParseUnsupportedState::PendingScript
                        | HtmlParseUnsupportedState::DetailsNotification
                        | HtmlParseUnsupportedState::ImageResource
                ))
        )
    {
        return CaseOutcome::unsupported(format!("{reason:?}"));
    }
    props.failure(
        3,
        "unexpected-terminal",
        format!("{reason:?}; first={first:?}"),
    )
}
pub(crate) fn evaluate(target: &UnicodeTarget, case: &UnicodeCase) -> Result<CaseOutcome> {
    let props = Props(&target.descriptor);
    let original = source(case);
    let limits = case.limits()?;
    let requested = match case.kind {
        ParseKind::Document => HtmlParserScriptingMode::Disabled,
        ParseKind::Fragment => HtmlParserScriptingMode::Inert,
    };
    let mut capture = None;
    let (parsed, cancel) = match case.kind {
        ParseKind::Document => {
            let request = HtmlDocumentParseRequest::offline(original.clone(), limits)
                .with_detail(DetailSet::ALL)
                .map_err(detail_error)?;
            let cancel = request.cancellation().clone();
            if case.lane == Lane::PreCancelled {
                cancel.cancel();
            }
            (request.parse(), cancel)
        }
        ParseKind::Fragment => {
            let cx = context(case.lane == Lane::MissingScripting)?;
            capture = Some(cx.capture().identity().clone());
            let request = HtmlFragmentParseRequest::new(original.clone(), cx, limits);
            if case.lane == Lane::MissingScripting {
                return match request {
                    Err(HtmlParseRequestError::Capture(HtmlCaptureError::MissingFact {
                        fact: HtmlCaptureFact::ScriptingStatus,
                        ..
                    })) => CaseOutcome::expected_rejection(
                        vec![props.id(3)],
                        "explicit captured scripting fact absent",
                    ),
                    _ => props.failure(
                        3,
                        "missing-context-fact",
                        "request failed to retain typed context rejection",
                    ),
                };
            }
            let request = request.map_err(|e| {
                FuzzError::with_source(
                    FuzzErrorKind::Harness,
                    "build fixed fragment request",
                    "qualified capture",
                    e,
                )
            })?;
            let request = request.with_detail(DetailSet::ALL).map_err(detail_error)?;
            let cancel = request.cancellation().clone();
            if case.lane == Lane::PreCancelled {
                cancel.cancel();
            }
            (request.parse(), cancel)
        }
    };
    let spec = Spec {
        fragment: case.kind == ParseKind::Fragment,
        text: &case.source,
        original: &original,
        limits: &limits,
        caps: case.caps,
        requested,
        capture: capture.as_ref(),
        shadow_allowed: false,
    };
    match parsed {
        Ok(report) => {
            if !matches!(case.lane, Lane::General | Lane::Literal(_)) {
                return props.failure(
                    3,
                    "missing-terminal-refusal",
                    "explicit stopped lane completed",
                );
            }
            let seen = match verify_report(&props, &spec, &report)? {
                Ok(v) => v,
                Err(outcome) => return Ok(outcome),
            };
            if let Lane::Literal(id) = case.lane
                && (!check_literal(id, &seen.nodes)
                    || !literal_evidence(id, &report, &seen.rows, &seen.diagnostics.records))
            {
                return props.failure(
                    5,
                    "literal-semantics",
                    "source-derived literal observation disagrees",
                );
            }
            if let Some(outcome) = verify_immutable(&props, &report, &seen, &cancel)? {
                return Ok(outcome);
            }
            let mut checked = vec![0, 1, 2, 4];
            if matches!(case.lane, Lane::Literal(_)) {
                checked.push(5);
            }
            CaseOutcome::evaluated(props.ids(&checked))
        }
        Err(f) => {
            let expect = match case.lane {
                Lane::PreCancelled => Expect::PreCancelled,
                Lane::SourceDenied => Expect::SourceDenied,
                Lane::WorkDenied => Expect::WorkDenied,
                Lane::Literal(_) | Lane::MissingScripting => Expect::Never,
                Lane::General => Expect::General,
            };
            verify_failure(&props, &spec, &f, &cancel, expect)
        }
    }
}

#[cfg(test)]
mod tests;
