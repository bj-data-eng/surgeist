//! Selected html5lib f994590f expected data, interpreted independently of the
//! native parser. HTML 877d remains the language authority. The .dat grammar's
//! diagnostics are retained, not asserted as HTML diagnostic conformance.
//! Expected/native-literal setup allocations are outside the parser ledger;
//! actual completed projection and comparison use its sole control.

use crate::{
    HtmlDocument, HtmlDocumentMode, HtmlFragmentOwner, HtmlNodeId, HtmlNodeKind,
    HtmlShadowRootProperties, QualifiedName,
};

const MAX_FIXTURE_BYTES: usize = 100_000;
const MAX_PROJECTED_NODES: usize = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FixtureScripting {
    Both,
    Disabled,
    Enabled,
}

#[derive(Debug, Eq, PartialEq)]
struct Fixture<'a> {
    ordinal: usize,
    input: &'a str,
    errors: Vec<&'a str>,
    new_errors: Option<Vec<&'a str>>,
    fragment: Option<&'a str>,
    scripting: FixtureScripting,
    expected: &'a str,
}

#[derive(Debug, Eq, PartialEq)]
enum FixtureError {
    Size,
    FinalLineFeed,
    DataMarker,
    ErrorsMarker,
    DuplicateOrUnorderedField,
    FragmentContext,
    DocumentMarker,
    ExpectedRecord,
    Indentation,
    Name,
    Attribute,
    Doctype,
    Limit,
}

/// A complete UTF-8 .dat file, not a synthesized concatenation of cases. The
/// upstream separator is two LFs; section payloads are never trimmed. Exactly
/// the grammar's final data LF is removed; a preceding CR or LF remains input.
fn decode(file: &str) -> Result<Vec<Fixture<'_>>, FixtureError> {
    if file.len() > MAX_FIXTURE_BYTES {
        return Err(FixtureError::Size);
    }
    let file = file.strip_suffix('\n').ok_or(FixtureError::FinalLineFeed)?;
    let mut fixtures = Vec::new();
    for (index, block) in file.split("\n\n#data\n").enumerate() {
        let block = if index == 0 {
            block
                .strip_prefix("#data\n")
                .ok_or(FixtureError::DataMarker)?
        } else {
            block
        };
        let (input, remainder) = block
            .split_once("\n#errors\n")
            .ok_or(FixtureError::ErrorsMarker)?;
        let (metadata, expected) = if let Some(parts) = remainder.split_once("#document\n") {
            parts
        } else if let Some(metadata) = remainder.strip_suffix("#document") {
            (metadata, "")
        } else {
            return Err(FixtureError::DocumentMarker);
        };
        if !metadata.is_empty() && !metadata.ends_with('\n') {
            return Err(FixtureError::DocumentMarker);
        }
        let mut errors = Vec::new();
        let mut new_errors = None;
        let mut fragment = None;
        let mut scripting = FixtureScripting::Both;
        let mut lines = metadata.split_terminator('\n');
        let mut phase = 0;
        while let Some(line) = lines.next() {
            match line {
                "#new-errors" if phase == 0 => {
                    phase = 1;
                    new_errors = Some(Vec::new());
                }
                "#document-fragment" if phase <= 1 && fragment.is_none() => {
                    phase = 2;
                    let context = lines.next().ok_or(FixtureError::FragmentContext)?;
                    if context.is_empty() || context.starts_with('#') {
                        return Err(FixtureError::FragmentContext);
                    }
                    fragment = Some(context);
                }
                "#script-off" | "#script-on" if phase <= 2 => {
                    phase = 3;
                    scripting = if line == "#script-off" {
                        FixtureScripting::Disabled
                    } else {
                        FixtureScripting::Enabled
                    };
                }
                line if line.is_empty() || line.starts_with('#') || phase >= 2 => {
                    return Err(FixtureError::DuplicateOrUnorderedField);
                }
                line if phase == 0 => errors.push(line),
                line => new_errors.as_mut().expect("new-errors phase").push(line),
            }
        }
        if expected.is_empty() && fragment.is_none() {
            return Err(FixtureError::ExpectedRecord);
        }
        fixtures.push(Fixture {
            ordinal: index + 1,
            input,
            errors,
            new_errors,
            fragment,
            scripting,
            expected,
        });
    }
    Ok(fixtures)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Name {
    namespace: Option<Vec<u16>>,
    prefix: Option<Vec<u16>>,
    local: Vec<u16>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Attr {
    name: Name,
    value: Vec<u16>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum Kind {
    Document(HtmlDocumentMode),
    Fragment,
    Element(Name, Vec<Attr>),
    Text(Vec<u16>),
    Comment(Vec<u16>),
    Pi {
        target: Vec<u16>,
        data: Vec<u16>,
    },
    Doctype {
        name: Vec<u16>,
        public: Option<Vec<u16>>,
        system: Option<Vec<u16>>,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Owner {
    Template(usize),
    Shadow(usize),
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Node {
    kind: Kind,
    parent: Option<usize>,
    children: Vec<usize>,
    template: Option<usize>,
    shadow: Option<(usize, HtmlShadowRootProperties)>,
    owner: Option<Owner>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Projection {
    nodes: Vec<Node>,
}
#[derive(Debug, Eq, PartialEq)]
enum ProjectionError {
    ForeignIdentity,
    RepeatedIdentity,
    Parent,
    Association,
    Limit,
}
fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
fn native_name(name: &QualifiedName) -> Name {
    Name {
        namespace: name.namespace().uri().map(|s| s.code_units().to_vec()),
        prefix: name.prefix().map(|s| s.code_units().to_vec()),
        local: name.local_name().code_units().to_vec(),
    }
}
fn native_kind(kind: &HtmlNodeKind) -> Kind {
    match kind {
        HtmlNodeKind::Document(mode) => Kind::Document(*mode),
        HtmlNodeKind::Fragment => Kind::Fragment,
        HtmlNodeKind::Element(element) => Kind::Element(
            native_name(element.name()),
            element
                .attributes()
                .as_slice()
                .iter()
                .map(|attr| Attr {
                    name: native_name(attr.name()),
                    value: attr.value().code_units().to_vec(),
                })
                .collect(),
        ),
        HtmlNodeKind::Text(text) => Kind::Text(text.code_units().to_vec()),
        HtmlNodeKind::Comment(text) => Kind::Comment(text.code_units().to_vec()),
        HtmlNodeKind::ProcessingInstruction(pi) => Kind::Pi {
            target: pi.target().code_units().to_vec(),
            data: pi.data().code_units().to_vec(),
        },
        HtmlNodeKind::Doctype(d) => Kind::Doctype {
            name: d.name().code_units().to_vec(),
            public: d.public_id().map(|s| s.code_units().to_vec()),
            system: d.system_id().map(|s| s.code_units().to_vec()),
        },
    }
}

/// Copies every native field and checks real backlinks/result-local identities.
/// Associated template/shadow fragments are separate roots, not child edges.
/// No native handles or shared payload aliases survive the returned projection.
fn project(document: &HtmlDocument) -> Result<Projection, ProjectionError> {
    let mut result = Projection { nodes: Vec::new() };
    let mut ids: Vec<HtmlNodeId> = Vec::new();
    let mut pending = vec![(document.root().id(), None, None)];
    while let Some((id, parent, owner)) = pending.pop() {
        if result.nodes.len() >= MAX_PROJECTED_NODES {
            return Err(ProjectionError::Limit);
        }
        if ids.contains(&id) {
            return Err(ProjectionError::RepeatedIdentity);
        }
        let view = document
            .node(&id)
            .map_err(|_| ProjectionError::ForeignIdentity)?;
        if view.parent() != parent.map(|i: usize| ids[i].clone()) {
            return Err(ProjectionError::Parent);
        }
        match (view.fragment_owner(), owner) {
            (None, None) => {}
            (Some(HtmlFragmentOwner::Template(actual)), Some(Owner::Template(i)))
            | (Some(HtmlFragmentOwner::ShadowHost(actual)), Some(Owner::Shadow(i)))
                if actual == &ids[i] => {}
            _ => return Err(ProjectionError::Association),
        }
        let index = result.nodes.len();
        if let Some(i) = parent {
            result.nodes[i].children.push(index);
        }
        match owner {
            Some(Owner::Template(i)) => result.nodes[i].template = Some(index),
            Some(Owner::Shadow(i)) => {
                let association = document
                    .node(&ids[i])
                    .map_err(|_| ProjectionError::ForeignIdentity)?
                    .declarative_shadow_root()
                    .ok_or(ProjectionError::Association)?;
                if association.host() != &ids[i] || association.contents() != &id {
                    return Err(ProjectionError::Association);
                }
                result.nodes[i].shadow = Some((index, association.properties()));
            }
            None => {}
        }
        result.nodes.push(Node {
            kind: native_kind(view.kind()),
            parent,
            children: Vec::new(),
            template: None,
            shadow: None,
            owner,
        });
        ids.push(id);
        if let Some(shadow) = view.declarative_shadow_root() {
            pending.push((shadow.contents().clone(), None, Some(Owner::Shadow(index))));
        }
        if let Some(template) = view.template_contents() {
            pending.push((template, None, Some(Owner::Template(index))));
        }
        for child in view.children().rev() {
            pending.push((child, Some(index), None));
        }
    }
    Ok(result)
}

fn dump_name(raw: &str, element: bool) -> Result<Name, FixtureError> {
    let (namespace, prefix, local) = if let Some((designator, local)) = raw.split_once(' ') {
        let (uri, prefix) = match designator {
            "svg" if element => ("http://www.w3.org/2000/svg", None),
            "math" if element => ("http://www.w3.org/1998/Math/MathML", None),
            "xlink" if !element => ("http://www.w3.org/1999/xlink", Some("xlink")),
            "xml" if !element => ("http://www.w3.org/XML/1998/namespace", Some("xml")),
            "xmlns" if !element => (
                "http://www.w3.org/2000/xmlns/",
                (local != "xmlns").then_some("xmlns"),
            ),
            _ => return Err(FixtureError::Name),
        };
        (Some(units(uri)), prefix.map(units), local)
    } else {
        (
            element.then(|| units("http://www.w3.org/1999/xhtml")),
            None,
            raw,
        )
    };
    if local.is_empty() || local.contains(' ') {
        return Err(FixtureError::Name);
    }
    Ok(Name {
        namespace,
        prefix,
        local: units(local),
    })
}

fn attribute_key(name: &Name) -> Vec<u16> {
    let designator = match name.namespace.as_deref() {
        None => "",
        Some(uri) if uri == units("http://www.w3.org/1999/xlink") => "xlink ",
        Some(uri) if uri == units("http://www.w3.org/XML/1998/namespace") => "xml ",
        Some(uri) if uri == units("http://www.w3.org/2000/xmlns/") => "xmlns ",
        _ => unreachable!("decoder emits only fixture attribute namespace designators"),
    };
    let mut key = units(designator);
    key.extend_from_slice(&name.local);
    key
}

fn dump_kind(raw: &str) -> Result<Kind, FixtureError> {
    if let Some(raw) = raw
        .strip_prefix("<!DOCTYPE ")
        .and_then(|s| s.strip_suffix('>'))
    {
        let (name, public, system) = if let Some((name, tail)) = raw.split_once(" \"") {
            let tail = tail.strip_suffix('"').ok_or(FixtureError::Doctype)?;
            let (public, system) = tail.split_once("\" \"").ok_or(FixtureError::Doctype)?;
            (name, Some(units(public)), Some(units(system)))
        } else {
            (raw, None, None)
        };
        return Ok(Kind::Doctype {
            name: units(name),
            public,
            system,
        });
    }
    if let Some(raw) = raw
        .strip_prefix("<!-- ")
        .and_then(|s| s.strip_suffix(" -->"))
    {
        return Ok(Kind::Comment(units(raw)));
    }
    if let Some(raw) = raw.strip_prefix("<?").and_then(|s| s.strip_suffix('>')) {
        let (target, data) = raw.split_once(' ').ok_or(FixtureError::ExpectedRecord)?;
        return Ok(Kind::Pi {
            target: units(target),
            data: units(data),
        });
    }
    if let Some(raw) = raw.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return Ok(Kind::Text(units(raw)));
    }
    if let Some(raw) = raw.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
        return Ok(Kind::Element(dump_name(raw, true)?, Vec::new()));
    }
    Err(FixtureError::ExpectedRecord)
}

/// The upstream format does not carry document mode or distinguish null from
/// empty doctype identifiers. Those require explicit governing-source supplements
/// at the selected case binding, and independent native observation tests below.
fn expected(fixture: &Fixture<'_>, mode: HtmlDocumentMode) -> Result<Projection, FixtureError> {
    let root = if fixture.fragment.is_some() {
        Kind::Fragment
    } else {
        Kind::Document(mode)
    };
    let mut result = Projection {
        nodes: vec![Node {
            kind: root,
            parent: None,
            children: Vec::new(),
            template: None,
            shadow: None,
            owner: None,
        }],
    };
    let mut path = vec![0];
    let mut lines = fixture.expected.split_terminator('\n').peekable();
    while let Some(line) = lines.next() {
        let raw = line
            .strip_prefix("| ")
            .ok_or(FixtureError::ExpectedRecord)?;
        let spaces = raw.len() - raw.trim_start_matches(' ').len();
        if spaces % 2 != 0 {
            return Err(FixtureError::Indentation);
        }
        let depth = spaces / 2;
        if depth >= path.len() {
            return Err(FixtureError::Indentation);
        }
        let mut record = raw[spaces..].to_owned();
        let closing = if record.starts_with("<!-- ") {
            Some(" -->")
        } else if record.starts_with('"') || (!record.starts_with('<') && record.contains("=\"")) {
            Some("\"")
        } else {
            None
        };
        if let Some(closing) = closing {
            while !record.ends_with(closing) || (record == "\"" && closing == "\"") {
                record.push('\n');
                record.push_str(lines.next().ok_or(FixtureError::ExpectedRecord)?);
            }
        }
        let parent = path[depth];
        if !record.starts_with(['<', '"'])
            && let Some((raw_name, value)) = record.split_once("=\"")
        {
            if !result.nodes[parent].children.is_empty() || result.nodes[parent].template.is_some()
            {
                return Err(FixtureError::Attribute);
            }
            let Kind::Element(_, attrs) = &mut result.nodes[parent].kind else {
                return Err(FixtureError::Attribute);
            };
            let value = value.strip_suffix('"').ok_or(FixtureError::Attribute)?;
            let attr = Attr {
                name: dump_name(raw_name, false)?,
                value: units(value),
            };
            if attrs
                .iter()
                .any(|a| a.name.namespace == attr.name.namespace && a.name.local == attr.name.local)
                || attrs.last().is_some_and(|previous| {
                    attribute_key(&previous.name) >= attribute_key(&attr.name)
                })
            {
                return Err(FixtureError::Attribute);
            }
            attrs.push(attr);
            continue;
        }
        if result.nodes.len() >= MAX_PROJECTED_NODES {
            return Err(FixtureError::Limit);
        }
        let index = result.nodes.len();
        let (kind, parent, owner) = if record == "content" {
            let Kind::Element(name, _) = &result.nodes[parent].kind else {
                return Err(FixtureError::ExpectedRecord);
            };
            if name != &dump_name("template", true)? || result.nodes[parent].template.is_some() {
                return Err(FixtureError::ExpectedRecord);
            }
            result.nodes[parent].template = Some(index);
            (Kind::Fragment, None, Some(Owner::Template(parent)))
        } else {
            result.nodes[parent].children.push(index);
            (dump_kind(&record)?, Some(parent), None)
        };
        result.nodes.push(Node {
            kind,
            parent,
            children: Vec::new(),
            template: None,
            shadow: None,
            owner,
        });
        path.truncate(depth + 1);
        path.push(index);
    }
    Ok(result)
}

/// Governing HTML initial-mode DOM creation uses empty strings for missing
/// token IDs (877d raw96343–96347). This explicit source supplement fills the
/// .dat dump's omitted-ID ambiguity without changing the native observation.
fn parsed_expected(
    fixture: &Fixture<'_>,
    mode: HtmlDocumentMode,
) -> Result<Projection, FixtureError> {
    let mut result = expected(fixture, mode)?;
    for node in &mut result.nodes {
        if let Kind::Doctype { public, system, .. } = &mut node.kind {
            if public.is_none() {
                *public = Some(Vec::new());
            }
            if system.is_none() {
                *system = Some(Vec::new());
            }
        }
    }
    Ok(result)
}

// The expected decoder uses ordinary bounded test allocations. Actual completed
// projection is a different owner: all storage below uses the borrowed parser
// control and is explicitly released before the completed driver is released.
use crate::HtmlParseResource;
use crate::construction::parser_session::native_view::{
    NativeFragmentOwner, NativeKind, NativeName, NativeString,
};
use crate::construction::parser_session::{NativeView, ParserNode, SessionError};
use crate::parser::control::{OwnedVec, ParseControl, ParseStop, WorkCategory};
use std::mem::size_of;

#[derive(Clone, Copy)]
struct Span {
    start: usize,
    len: usize,
}
#[derive(Clone, Copy)]
struct MeteredName {
    namespace: Option<Span>,
    prefix: Option<Span>,
    local: Span,
}
#[derive(Clone, Copy)]
struct MeteredAttr {
    name: MeteredName,
    value: Span,
}
#[derive(Clone, Copy)]
enum MeteredKind {
    Document(HtmlDocumentMode),
    Fragment,
    Element(MeteredName, Span),
    Text(Span),
    Comment(Span),
    Pi {
        target: Span,
        data: Span,
    },
    Doctype {
        name: Span,
        public: Option<Span>,
        system: Option<Span>,
    },
}
#[derive(Clone, Copy)]
struct MeteredNode {
    kind: MeteredKind,
    parent: Option<usize>,
    template: Option<usize>,
    shadow: Option<(usize, HtmlShadowRootProperties)>,
    owner: Option<Owner>,
}
#[derive(Clone, Copy)]
struct Edge {
    parent: usize,
    child: usize,
}
struct Pending {
    node: ParserNode,
    parent: Option<usize>,
    owner: Option<Owner>,
}
struct MeteredProjection {
    nodes: OwnedVec<MeteredNode>,
    attrs: OwnedVec<MeteredAttr>,
    edges: OwnedVec<Edge>,
    units: OwnedVec<u16>,
}
#[derive(Debug)]
enum AdapterError {
    Stop(ParseStop),
    Session(SessionError),
    Structure(ProjectionError),
    Mismatch,
}
impl From<ParseStop> for AdapterError {
    fn from(e: ParseStop) -> Self {
        Self::Stop(e)
    }
}
impl From<SessionError> for AdapterError {
    fn from(e: SessionError) -> Self {
        Self::Session(e)
    }
}

/// Fixed POD output reads are admitted before inspection/copy. This does not
/// bulk-prepay a variable traversal; loops below poll per edge/record/code unit.
fn charges(c: &mut ParseControl, values: &[(WorkCategory, u64)]) -> Result<(), ParseStop> {
    for (kind, units) in values {
        c.charge(*kind, *units)?;
    }
    Ok(())
}
fn read<T: Copy>(v: &mut OwnedVec<T>, i: usize, c: &mut ParseControl) -> Result<T, AdapterError> {
    v.require_owner(c)?;
    c.admit_atomic_layout(size_of::<T>())?;
    charges(
        c,
        &[
            (WorkCategory::RecordVisit, 1),
            (WorkCategory::InspectByte, size_of::<T>() as u64),
            (WorkCategory::CopyByte, size_of::<T>() as u64),
        ],
    )?;
    v.as_slice()
        .get(i)
        .copied()
        .ok_or(AdapterError::Structure(ProjectionError::Limit))
}
fn id_at(
    v: &mut OwnedVec<ParserNode>,
    i: usize,
    c: &mut ParseControl,
) -> Result<ParserNode, AdapterError> {
    v.require_owner(c)?;
    charges(
        c,
        &[
            (WorkCategory::RecordVisit, 1),
            (WorkCategory::InspectByte, size_of::<ParserNode>() as u64),
            (WorkCategory::CopyByte, size_of::<ParserNode>() as u64),
            (WorkCategory::ProviderCall, 1),
        ],
    )?;
    v.as_slice()
        .get(i)
        .cloned()
        .ok_or(AdapterError::Structure(ProjectionError::Limit))
}
impl MeteredProjection {
    fn new() -> Result<Self, ParseStop> {
        Ok(Self {
            nodes: OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
            attrs: OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
            edges: OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
            units: OwnedVec::new(HtmlParseResource::BufferedUtf16Units)?,
        })
    }
    fn release(self, view: &mut NativeView<'_>) -> Result<(), ParseStop> {
        // No nested allocations in any record, and release is also valid after
        // a first stop. Never refund work or high-water observations.
        self.nodes.release(view.control())?;
        self.attrs.release(view.control())?;
        self.edges.release(view.control())?;
        self.units.release(view.control())
    }
    fn string(
        &mut self,
        s: &NativeString<'_>,
        view: &mut NativeView<'_>,
    ) -> Result<Span, AdapterError> {
        view.control().charge(WorkCategory::RecordVisit, 1)?;
        let span = Span {
            start: self.units.len(),
            len: s.len(),
        };
        for i in 0..span.len {
            let unit = view
                .unit(s, i)?
                .ok_or(AdapterError::Structure(ProjectionError::Limit))?;
            self.units.try_push(view.control(), unit)?;
        }
        Ok(span)
    }
    fn optional_string(
        &mut self,
        s: Option<&NativeString<'_>>,
        v: &mut NativeView<'_>,
    ) -> Result<Option<Span>, AdapterError> {
        s.map(|s| self.string(s, v)).transpose()
    }
    fn name(
        &mut self,
        n: &NativeName<'_>,
        v: &mut NativeView<'_>,
    ) -> Result<MeteredName, AdapterError> {
        Ok(MeteredName {
            namespace: self.optional_string(n.namespace.as_ref(), v)?,
            prefix: self.optional_string(n.prefix.as_ref(), v)?,
            local: self.string(&n.local, v)?,
        })
    }
    fn kind(
        &mut self,
        node: &ParserNode,
        view: &mut NativeView<'_>,
    ) -> Result<MeteredKind, AdapterError> {
        Ok(match view.kind(node)? {
            NativeKind::Document(mode) => MeteredKind::Document(mode),
            NativeKind::Fragment => MeteredKind::Fragment,
            NativeKind::Element(name) => {
                let name = self.name(&name, view)?;
                let count = view.attribute_count(node)?;
                let span = Span {
                    start: self.attrs.len(),
                    len: count,
                };
                for i in 0..count {
                    let attr = view
                        .attribute_at(node, i)?
                        .ok_or(AdapterError::Structure(ProjectionError::Limit))?;
                    let name = self.name(&attr.name, view)?;
                    let value = self.string(&attr.value, view)?;
                    self.attrs
                        .try_push(view.control(), MeteredAttr { name, value })?;
                }
                MeteredKind::Element(name, span)
            }
            NativeKind::Text(s) => MeteredKind::Text(self.string(&s, view)?),
            NativeKind::Comment(s) => MeteredKind::Comment(self.string(&s, view)?),
            NativeKind::ProcessingInstruction { target, data } => MeteredKind::Pi {
                target: self.string(&target, view)?,
                data: self.string(&data, view)?,
            },
            NativeKind::Doctype {
                name,
                public_id,
                system_id,
            } => MeteredKind::Doctype {
                name: self.string(&name, view)?,
                public: self.optional_string(public_id.as_ref(), view)?,
                system: self.optional_string(system_id.as_ref(), view)?,
            },
        })
    }
    fn build(&mut self, view: &mut NativeView<'_>) -> Result<(), AdapterError> {
        let mut ids = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let mut pending = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let outcome = (|| {
            let root = view.root()?;
            pending.try_push(
                view.control(),
                Pending {
                    node: root,
                    parent: None,
                    owner: None,
                },
            )?;
            while let Some(frame) = pending.try_pop(view.control())? {
                let index = self.nodes.len();
                for i in 0..ids.len() {
                    let previous = id_at(&mut ids, i, view.control())?;
                    if view.same_node(&previous, &frame.node)? {
                        return Err(AdapterError::Structure(ProjectionError::RepeatedIdentity));
                    }
                }
                let actual_parent = view.parent(&frame.node)?;
                match (actual_parent, frame.parent) {
                    (None, None) => {}
                    (Some(actual), Some(i)) => {
                        let expected = id_at(&mut ids, i, view.control())?;
                        if !view.same_node(&actual, &expected)? {
                            return Err(AdapterError::Structure(ProjectionError::Parent));
                        }
                    }
                    _ => return Err(AdapterError::Structure(ProjectionError::Parent)),
                }
                let actual_owner = view.fragment_owner(&frame.node)?;
                match (actual_owner, frame.owner) {
                    (None, None) => {}
                    (Some(NativeFragmentOwner::Template(actual)), Some(Owner::Template(i)))
                    | (Some(NativeFragmentOwner::ShadowHost(actual)), Some(Owner::Shadow(i))) => {
                        let expected = id_at(&mut ids, i, view.control())?;
                        if !view.same_node(&actual, &expected)? {
                            return Err(AdapterError::Structure(ProjectionError::Association));
                        }
                    }
                    _ => return Err(AdapterError::Structure(ProjectionError::Association)),
                }
                if let Some(parent) = frame.parent {
                    self.edges.try_push(
                        view.control(),
                        Edge {
                            parent,
                            child: index,
                        },
                    )?;
                }
                let kind = self.kind(&frame.node, view)?;
                self.nodes.try_push(
                    view.control(),
                    MeteredNode {
                        kind,
                        parent: frame.parent,
                        template: None,
                        shadow: None,
                        owner: frame.owner,
                    },
                )?;
                match frame.owner {
                    Some(Owner::Template(i)) => {
                        charges(
                            view.control(),
                            &[
                                (WorkCategory::RecordWrite, 1),
                                (WorkCategory::CopyByte, size_of::<Option<usize>>() as u64),
                            ],
                        )?;
                        self.nodes
                            .get_mut(i, view.control())?
                            .ok_or(AdapterError::Structure(ProjectionError::Association))?
                            .template = Some(index);
                    }
                    Some(Owner::Shadow(i)) => {
                        let host = id_at(&mut ids, i, view.control())?;
                        let shadow = view
                            .shadow(&host)?
                            .ok_or(AdapterError::Structure(ProjectionError::Association))?;
                        if !view.same_node(&shadow.host, &host)?
                            || !view.same_node(&shadow.contents, &frame.node)?
                        {
                            return Err(AdapterError::Structure(ProjectionError::Association));
                        }
                        charges(
                            view.control(),
                            &[
                                (WorkCategory::RecordWrite, 1),
                                (
                                    WorkCategory::CopyByte,
                                    size_of::<Option<(usize, HtmlShadowRootProperties)>>() as u64,
                                ),
                            ],
                        )?;
                        self.nodes
                            .get_mut(i, view.control())?
                            .ok_or(AdapterError::Structure(ProjectionError::Association))?
                            .shadow = Some((index, shadow.properties));
                    }
                    None => {}
                }
                if let Some(shadow) = view.shadow(&frame.node)? {
                    if !view.same_node(&shadow.host, &frame.node)? {
                        return Err(AdapterError::Structure(ProjectionError::Association));
                    }
                    pending.try_push(
                        view.control(),
                        Pending {
                            node: shadow.contents,
                            parent: None,
                            owner: Some(Owner::Shadow(index)),
                        },
                    )?;
                }
                if let Some(template) = view.template_contents(&frame.node)? {
                    pending.try_push(
                        view.control(),
                        Pending {
                            node: template,
                            parent: None,
                            owner: Some(Owner::Template(index)),
                        },
                    )?;
                }
                let count = view.child_count(&frame.node)?;
                for i in (0..count).rev() {
                    let child = view
                        .child_at(&frame.node, i)?
                        .ok_or(AdapterError::Structure(ProjectionError::Parent))?;
                    pending.try_push(
                        view.control(),
                        Pending {
                            node: child,
                            parent: Some(index),
                            owner: None,
                        },
                    )?;
                }
                ids.try_push(view.control(), frame.node)?;
            }
            Ok(())
        })();
        ids.release(view.control())?;
        pending.release(view.control())?;
        outcome
    }
    fn equal_units(
        &mut self,
        actual: Span,
        expected: &[u16],
        c: &mut ParseControl,
    ) -> Result<bool, AdapterError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if actual.len != expected.len() {
            return Ok(false);
        }
        for (offset, expected) in expected.iter().enumerate() {
            c.charge(WorkCategory::InspectByte, 2)?;
            if read(&mut self.units, actual.start + offset, c)? != *expected {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn equal_optional(
        &mut self,
        actual: Option<Span>,
        expected: Option<&Vec<u16>>,
        c: &mut ParseControl,
    ) -> Result<bool, AdapterError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        match (actual, expected) {
            (None, None) => Ok(true),
            (Some(a), Some(e)) => self.equal_units(a, e, c),
            _ => Ok(false),
        }
    }
    fn equal_name(
        &mut self,
        actual: MeteredName,
        expected: &Name,
        c: &mut ParseControl,
    ) -> Result<bool, AdapterError> {
        Ok(
            self.equal_optional(actual.namespace, expected.namespace.as_ref(), c)?
                && self.equal_optional(actual.prefix, expected.prefix.as_ref(), c)?
                && self.equal_units(actual.local, &expected.local, c)?,
        )
    }
    fn equal_kind(
        &mut self,
        actual: MeteredKind,
        expected: &Kind,
        c: &mut ParseControl,
    ) -> Result<bool, AdapterError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        match (actual, expected) {
            (MeteredKind::Document(a), Kind::Document(e)) => Ok(a == *e),
            (MeteredKind::Fragment, Kind::Fragment) => Ok(true),
            (MeteredKind::Text(a), Kind::Text(e)) | (MeteredKind::Comment(a), Kind::Comment(e)) => {
                self.equal_units(a, e, c)
            }
            (
                MeteredKind::Pi { target, data },
                Kind::Pi {
                    target: et,
                    data: ed,
                },
            ) => Ok(self.equal_units(target, et, c)? && self.equal_units(data, ed, c)?),
            (
                MeteredKind::Doctype {
                    name,
                    public,
                    system,
                },
                Kind::Doctype {
                    name: en,
                    public: ep,
                    system: es,
                },
            ) => Ok(self.equal_units(name, en, c)?
                && self.equal_optional(public, ep.as_ref(), c)?
                && self.equal_optional(system, es.as_ref(), c)?),
            (MeteredKind::Element(name, attrs), Kind::Element(en, ea)) => {
                if !self.equal_name(name, en, c)? || attrs.len != ea.len() {
                    return Ok(false);
                }
                // Corpus attributes are sorted by names. The native projection
                // retains actual order; only this declared comparison is a
                // permutation by the complete exact expanded name/prefix/value.
                for expected in ea {
                    let mut found = false;
                    for i in 0..attrs.len {
                        let actual = read(&mut self.attrs, attrs.start + i, c)?;
                        if self.equal_name(actual.name, &expected.name, c)?
                            && self.equal_units(actual.value, &expected.value, c)?
                        {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    fn compare(
        &mut self,
        expected: &Projection,
        view: &mut NativeView<'_>,
    ) -> Result<(), AdapterError> {
        let c = view.control();
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.nodes.len() != expected.nodes.len() {
            return Err(AdapterError::Mismatch);
        }
        for (i, expected) in expected.nodes.iter().enumerate() {
            let actual = read(&mut self.nodes, i, c)?;
            c.admit_atomic_layout(size_of::<Node>())?;
            c.charge(WorkCategory::InspectByte, size_of::<Node>() as u64)?;
            if actual.parent != expected.parent
                || actual.template != expected.template
                || actual.shadow != expected.shadow
                || actual.owner != expected.owner
                || !self.equal_kind(actual.kind, &expected.kind, c)?
            {
                return Err(AdapterError::Mismatch);
            }
            let mut child_index = 0;
            for e in 0..self.edges.len() {
                let edge = read(&mut self.edges, e, c)?;
                if edge.parent == i {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if expected.children.get(child_index) != Some(&edge.child) {
                        return Err(AdapterError::Mismatch);
                    }
                    child_index += 1;
                }
            }
            if child_index != expected.children.len() {
                return Err(AdapterError::Mismatch);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum CaseError {
    Parser(super::StandaloneError),
    Observation(AdapterError),
    Cleanup(ParseStop),
}
impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stop(e) => write!(f, "control stop: {e:?}"),
            Self::Session(e) => write!(f, "native observation: {e:?}"),
            Self::Structure(e) => write!(f, "structure: {e:?}"),
            Self::Mismatch => f.write_str("independent expected projection differs"),
        }
    }
}
impl std::fmt::Display for CaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parser(e) => write!(f, "actual driver: {e:?}"),
            Self::Observation(e) => write!(f, "{e}"),
            Self::Cleanup(e) => write!(f, "projection cleanup: {e:?}"),
        }
    }
}

/// Actual request→driver→qualified whole completed owner→checked borrow. No
/// token/diagnostic disposal or session extraction is used as observation.
fn check(
    request: &crate::HtmlDocumentParseRequest,
    expected: &Projection,
) -> Result<(), CaseError> {
    let mut entry = super::entry::OrdinaryDocumentEntry::new(request);
    let driver = match entry.take_driver() {
        Ok(driver) => driver,
        Err(error) => {
            entry.release_failure().map_err(CaseError::Parser)?;
            return Err(CaseError::Parser(error));
        }
    };
    let mut driver = Some(driver);
    let parsed = (|| {
        loop {
            match driver.as_mut().expect("actual driver owned").step()? {
                super::head::HeadStep::Consumed => {}
                super::head::HeadStep::Complete => break,
                super::head::HeadStep::Boundary(_) => {
                    return Err(super::StandaloneError::Head(
                        super::head::HeadError::IncompleteCompletion,
                    ));
                }
            }
        }
        super::StandaloneDocumentDriver::take_completed(&mut driver)
    })();
    let mut completed = match parsed {
        Ok(owner) => owner,
        Err(error) => {
            driver
                .as_mut()
                .expect("failed admission retains driver")
                .release()
                .map_err(CaseError::Parser)?;
            return Err(CaseError::Parser(error));
        }
    };
    let observed = (|| {
        let mut view = completed.native_view().map_err(CaseError::Parser)?;
        let mut projection = MeteredProjection::new().map_err(CaseError::Cleanup)?;
        let result = projection
            .build(&mut view)
            .and_then(|()| projection.compare(expected, &mut view));
        projection.release(&mut view).map_err(CaseError::Cleanup)?;
        result.map_err(CaseError::Observation)
    })();
    completed.release().map_err(CaseError::Parser)?;
    observed
}

const DATA: &[(&str, &str)] = &[
    (
        "adoption02.dat",
        include_str!("../../../tests/fixtures/html5lib-f994590f/tree-construction/adoption02.dat"),
    ),
    (
        "namespace-sensitivity.dat",
        include_str!(
            "../../../tests/fixtures/html5lib-f994590f/tree-construction/namespace-sensitivity.dat"
        ),
    ),
    (
        "quirks01.dat",
        include_str!("../../../tests/fixtures/html5lib-f994590f/tree-construction/quirks01.dat"),
    ),
    (
        "tests14.dat",
        include_str!("../../../tests/fixtures/html5lib-f994590f/tree-construction/tests14.dat"),
    ),
    (
        "tests24.dat",
        include_str!("../../../tests/fixtures/html5lib-f994590f/tree-construction/tests24.dat"),
    ),
    (
        "tables01.dat",
        include_str!("../../../tests/fixtures/html5lib-f994590f/tree-construction/tables01.dat"),
    ),
    (
        "comments01.dat",
        include_str!("../../../tests/fixtures/html5lib-f994590f/tree-construction/comments01.dat"),
    ),
    (
        "template.dat",
        include_str!("../../../tests/fixtures/html5lib-f994590f/tree-construction/template.dat"),
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Attribute, DocumentString, HtmlAuthoredProcessingInstruction, HtmlConstructionLimits,
        HtmlDoctype, Namespace, OrderedAttributes,
    };
    fn limits() -> HtmlConstructionLimits {
        HtmlConstructionLimits::new(100, 100, 100, 100_000, 100)
    }

    #[test]
    fn decoder_preserves_input_metadata_and_one_grammar_lf() {
        let decoded = decode("#data\n A\r\n\n#errors\n(1,1): one\n#new-errors\nnew detail\n#script-off\n#document\n| \" A\\r\"\n").unwrap();
        assert_eq!(decoded[0].input, " A\r\n");
        assert_eq!(decoded[0].errors, ["(1,1): one"]);
        assert_eq!(decoded[0].new_errors.as_ref().unwrap(), &["new detail"]);
        assert_eq!(decoded[0].scripting, FixtureScripting::Disabled);
        assert_eq!(decoded[0].ordinal, 1);
    }

    #[test]
    fn decoder_rejects_missing_duplicate_and_unordered_fields() {
        for raw in [
            "#data\nx\n#errors\n#document\n| \"x\"",
            "#data\nx\n#document\n| \"x\"\n",
            "#data\nx\n#errors\n#errors\n#document\n| \"x\"\n",
            "#data\nx\n#errors\n#script-off\n#script-on\n#document\n| \"x\"\n",
            "#data\nx\n#errors\n#document-fragment\n#document\n| \"x\"\n",
        ] {
            assert!(decode(raw).is_err(), "{raw:?}");
        }
    }

    #[test]
    fn decoder_retains_fragments_and_script_enabled_without_claiming_runnability() {
        let f = decode(
            "#data\nx\n#errors\n#document-fragment\nsvg title\n#script-on\n#document\n| \"x\"\n",
        )
        .unwrap();
        assert_eq!(f[0].fragment, Some("svg title"));
        assert_eq!(f[0].scripting, FixtureScripting::Enabled);
        assert_eq!(
            expected(&f[0], HtmlDocumentMode::NoQuirks).unwrap().nodes[0].kind,
            Kind::Fragment
        );
    }

    #[test]
    fn expected_decoder_preserves_multiline_text_comment_and_namespaced_attributes() {
        let f = decode("#data\nx\n#errors\n#document\n| <svg svg>\n|   xlink href=\"a\nb\"\n|   xlink:href=\"plain\"\n|   \"first\n\nlast\"\n|   <!-- a\nb -->\n").unwrap();
        let p = expected(&f[0], HtmlDocumentMode::NoQuirks).unwrap();
        let Kind::Element(name, attrs) = &p.nodes[1].kind else {
            panic!()
        };
        assert_eq!(name.namespace, Some(units("http://www.w3.org/2000/svg")));
        assert_eq!(attrs[0].name.prefix, Some(units("xlink")));
        assert_eq!(attrs[0].name.local, units("href"));
        assert_eq!(attrs[0].value, units("a\nb"));
        assert_eq!(attrs[1].name.namespace, None);
        assert_eq!(attrs[1].name.local, units("xlink:href"));
        assert_eq!(p.nodes[2].kind, Kind::Text(units("first\n\nlast")));
        assert_eq!(p.nodes[3].kind, Kind::Comment(units("a\nb")));
    }

    #[test]
    fn expected_decoder_rejects_invalid_depth_duplicate_names_and_fake_content_marker() {
        for tree in [
            "|   <p>",
            "| <p>\n|   id=\"1\"\n|   id=\"2\"",
            "| <svg template>\n|   content",
            "| <p>\n|  <b>",
        ] {
            let raw = format!("#data\nx\n#errors\n#document\n{tree}\n");
            let f = decode(&raw).unwrap();
            assert!(expected(&f[0], HtmlDocumentMode::NoQuirks).is_err());
        }
    }

    #[test]
    fn all_complete_retained_files_decode_with_stable_ordinals_and_raw_sections() {
        let mut total = 0;
        for (path, raw) in DATA {
            let cases = decode(raw).unwrap_or_else(|e| panic!("{path}: {e:?}"));
            assert!(!cases.is_empty());
            for (i, f) in cases.iter().enumerate() {
                assert_eq!(f.ordinal, i + 1);
                expected(f, HtmlDocumentMode::NoQuirks)
                    .unwrap_or_else(|e| panic!("{path}/{}: {e:?}", f.ordinal));
            }
            total += cases.len();
        }
        assert_eq!(total, 169);
    }

    #[test]
    fn native_projection_observes_modes_null_empty_names_prefixes_attrs_pi_utf16_and_backlinks() {
        let mut b = HtmlDocument::builder(limits(), HtmlDocumentMode::LimitedQuirks).unwrap();
        let root = b.root();
        b.append_doctype(
            &root,
            HtmlDoctype::new("HTML", None, Some(DocumentString::default())),
        )
        .unwrap();
        let attrs = OrderedAttributes::new([
            Attribute::new(QualifiedName::new(Namespace::none(), None, "z"), "first"),
            Attribute::new(
                QualifiedName::new(
                    Namespace::from_uri("urn:attr"),
                    Some(DocumentString::default()),
                    "a",
                ),
                DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]),
            ),
        ])
        .unwrap();
        let element = b
            .append_element(
                &root,
                QualifiedName::new(
                    Namespace::from_uri("urn:element"),
                    Some("p".into()),
                    "Mixed",
                ),
                attrs,
            )
            .unwrap();
        b.append_text(
            &element,
            DocumentString::from_utf16(vec![0xd83d, 0xde00, 0xd800, 13, 10, 0xdc00]),
        )
        .unwrap();
        b.append_comment(&root, " A\nB ").unwrap();
        b.append_processing_instruction(
            &root,
            HtmlAuthoredProcessingInstruction::new("Target", " raw\n").unwrap(),
        )
        .unwrap();
        let p = project(&b.finish()).unwrap();
        assert_eq!(p.nodes.len(), 6);
        assert_eq!(
            p.nodes[0].kind,
            Kind::Document(HtmlDocumentMode::LimitedQuirks)
        );
        assert_eq!(p.nodes[0].children, [1, 2, 4, 5]);
        assert_eq!(
            p.nodes[1].kind,
            Kind::Doctype {
                name: units("HTML"),
                public: None,
                system: Some(Vec::new())
            }
        );
        let Kind::Element(name, attrs) = &p.nodes[2].kind else {
            panic!()
        };
        assert_eq!(name.namespace, Some(units("urn:element")));
        assert_eq!(name.prefix, Some(units("p")));
        assert_eq!(name.local, units("Mixed"));
        assert_eq!(attrs[0].name.local, units("z"));
        assert_eq!(attrs[1].name.prefix, Some(Vec::new()));
        assert_eq!(attrs[1].value, [0xd800, 0, 0xdc00]);
        assert_eq!(p.nodes[3].parent, Some(2));
        assert_eq!(
            p.nodes[3].kind,
            Kind::Text(vec![0xd83d, 0xde00, 0xd800, 13, 10, 0xdc00])
        );
        assert_eq!(p.nodes[4].kind, Kind::Comment(units(" A\nB ")));
        assert_eq!(
            p.nodes[5].kind,
            Kind::Pi {
                target: units("Target"),
                data: units(" raw\n")
            }
        );
    }

    #[test]
    fn native_projection_preserves_separate_template_contents_and_distinct_equal_siblings() {
        let mut b = HtmlDocument::builder(limits(), HtmlDocumentMode::NoQuirks).unwrap();
        let root = b.root();
        let template = b
            .append_element(
                &root,
                QualifiedName::new(Namespace::html(), None, "template"),
                OrderedAttributes::default(),
            )
            .unwrap();
        let contents = b.template_contents(&template).unwrap().unwrap();
        b.append_text(&contents, "same").unwrap();
        b.append_text(&root, "same").unwrap();
        let p = project(&b.finish()).unwrap();
        assert_eq!(p.nodes[0].children, [1, 4]);
        assert_eq!(p.nodes[1].children, []);
        assert_eq!(p.nodes[1].template, Some(2));
        assert_eq!(p.nodes[2].kind, Kind::Fragment);
        assert_eq!(p.nodes[2].parent, None);
        assert_eq!(p.nodes[2].owner, Some(Owner::Template(1)));
        assert_eq!(p.nodes[2].children, [3]);
        assert_eq!(p.nodes[3].parent, Some(2));
        assert_eq!(p.nodes[3].kind, p.nodes[4].kind);
        assert_ne!(p.nodes[3].parent, p.nodes[4].parent);
    }
    #[test]
    fn comment_quotes_are_payload_not_an_attribute_or_text_delimiter() {
        let f = decode("#data\nx\n#errors\n#document\n| <!-- ?xml version=\"1.0\" -->\n").unwrap();
        let p = expected(&f[0], HtmlDocumentMode::Quirks).unwrap();
        assert_eq!(
            p.nodes[1].kind,
            Kind::Comment(units("?xml version=\"1.0\""))
        );
    }

    #[test]
    fn native_projection_retains_all_shadow_properties_and_reverse_owner() {
        let mut b = HtmlDocument::builder(limits(), HtmlDocumentMode::NoQuirks).unwrap();
        let host = b
            .append_element(
                &b.root(),
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
            )
            .unwrap();
        let props = crate::HtmlShadowRootProperties::new(
            crate::HtmlShadowRootMode::Closed,
            crate::HtmlSlotAssignment::Manual,
            true,
            false,
            true,
        );
        let contents = b.attach_declarative_shadow(&host, props).unwrap();
        b.append_text(&host, "light").unwrap();
        b.append_text(&contents, "shadow").unwrap();
        let p = project(&b.finish()).unwrap();
        assert_eq!(p.nodes[1].children, [2]);
        assert_eq!(p.nodes[1].shadow, Some((3, props)));
        assert_eq!(p.nodes[3].kind, Kind::Fragment);
        assert_eq!(p.nodes[3].parent, None);
        assert_eq!(p.nodes[3].owner, Some(Owner::Shadow(1)));
        assert_eq!(p.nodes[3].children, [4]);
        assert_eq!(p.nodes[4].parent, Some(3));
        assert_eq!(p.nodes[4].kind, Kind::Text(units("shadow")));
    }
    fn request(raw: &str) -> crate::HtmlDocumentParseRequest {
        crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(raw),
            super::super::tests::limits(1_000),
        )
    }

    #[test]
    fn selected_pinned_ordinary_disabled_structures_match_after_real_completion() {
        // These stable ordinals select structural expected data, not full-case
        // diagnostic conformance. No fragment or Enabled replay is implied.
        let selected: &[(&str, usize, HtmlDocumentMode)] = &[
            ("adoption02.dat", 1, HtmlDocumentMode::Quirks),
            ("namespace-sensitivity.dat", 1, HtmlDocumentMode::Quirks),
            ("quirks01.dat", 1, HtmlDocumentMode::LimitedQuirks),
            ("tests14.dat", 1, HtmlDocumentMode::NoQuirks),
            ("tests14.dat", 6, HtmlDocumentMode::NoQuirks),
            ("tests24.dat", 7, HtmlDocumentMode::NoQuirks),
            ("tables01.dat", 1, HtmlDocumentMode::Quirks),
            ("tables01.dat", 4, HtmlDocumentMode::Quirks),
            ("comments01.dat", 1, HtmlDocumentMode::Quirks),
            ("template.dat", 1, HtmlDocumentMode::Quirks),
            // Complete the four already-bundled lexical/prologue families.
            // Modes come from HTML877d initial rules, never the actual output.
            ("tests14.dat", 2, HtmlDocumentMode::NoQuirks),
            ("tests14.dat", 3, HtmlDocumentMode::NoQuirks),
            ("tests14.dat", 4, HtmlDocumentMode::NoQuirks),
            ("tests14.dat", 5, HtmlDocumentMode::NoQuirks),
            ("tests14.dat", 7, HtmlDocumentMode::NoQuirks),
            ("tests24.dat", 1, HtmlDocumentMode::NoQuirks),
            ("tests24.dat", 2, HtmlDocumentMode::NoQuirks),
            ("tests24.dat", 3, HtmlDocumentMode::NoQuirks),
            ("tests24.dat", 4, HtmlDocumentMode::NoQuirks),
            ("tests24.dat", 5, HtmlDocumentMode::NoQuirks),
            ("tests24.dat", 6, HtmlDocumentMode::NoQuirks),
            ("tests24.dat", 8, HtmlDocumentMode::NoQuirks),
            ("comments01.dat", 2, HtmlDocumentMode::Quirks),
            ("comments01.dat", 3, HtmlDocumentMode::Quirks),
            ("comments01.dat", 4, HtmlDocumentMode::Quirks),
            ("comments01.dat", 5, HtmlDocumentMode::Quirks),
            ("comments01.dat", 6, HtmlDocumentMode::Quirks),
            ("comments01.dat", 7, HtmlDocumentMode::Quirks),
            ("comments01.dat", 8, HtmlDocumentMode::Quirks),
            ("comments01.dat", 9, HtmlDocumentMode::Quirks),
            ("comments01.dat", 10, HtmlDocumentMode::Quirks),
            ("comments01.dat", 11, HtmlDocumentMode::Quirks),
            // Reserved xml is bogus-comment recovery even in the newer PI edition.
            ("comments01.dat", 12, HtmlDocumentMode::Quirks),
            ("comments01.dat", 13, HtmlDocumentMode::Quirks),
            ("comments01.dat", 14, HtmlDocumentMode::Quirks),
            ("comments01.dat", 15, HtmlDocumentMode::Quirks),
            ("comments01.dat", 16, HtmlDocumentMode::Quirks),
            ("quirks01.dat", 2, HtmlDocumentMode::Quirks),
            ("quirks01.dat", 3, HtmlDocumentMode::Quirks),
            ("quirks01.dat", 4, HtmlDocumentMode::Quirks),
            ("tables01.dat", 2, HtmlDocumentMode::Quirks),
            ("tables01.dat", 3, HtmlDocumentMode::Quirks),
            ("tables01.dat", 5, HtmlDocumentMode::Quirks),
            ("tables01.dat", 6, HtmlDocumentMode::Quirks),
            ("tables01.dat", 7, HtmlDocumentMode::Quirks),
            ("tables01.dat", 8, HtmlDocumentMode::Quirks),
            ("tables01.dat", 9, HtmlDocumentMode::Quirks),
            ("tables01.dat", 10, HtmlDocumentMode::Quirks),
            ("tables01.dat", 11, HtmlDocumentMode::Quirks),
            ("tables01.dat", 12, HtmlDocumentMode::Quirks),
            ("tables01.dat", 13, HtmlDocumentMode::Quirks),
            ("tables01.dat", 14, HtmlDocumentMode::Quirks),
            ("tables01.dat", 15, HtmlDocumentMode::Quirks),
            ("tables01.dat", 16, HtmlDocumentMode::Quirks),
            ("tables01.dat", 17, HtmlDocumentMode::Quirks),
            ("tables01.dat", 18, HtmlDocumentMode::Quirks),
            ("tables01.dat", 19, HtmlDocumentMode::Quirks),
        ];
        for (path, ordinal, mode) in selected {
            let (_, raw) = DATA.iter().find(|(p, _)| p == path).unwrap();
            let fixtures = decode(raw).unwrap();
            let fixture = &fixtures[ordinal - 1];
            assert!(fixture.fragment.is_none());
            assert_ne!(fixture.scripting, FixtureScripting::Enabled);
            let expected = parsed_expected(fixture, *mode).unwrap();
            check(&request(fixture.input), &expected)
                .unwrap_or_else(|e| panic!("{path}/{ordinal}: {e}"));
        }
    }

    #[test]
    fn initial_pi_preserves_later_doctype_and_no_quirks_selection() {
        // HTML877d raw96334–96347: an initial PI does not leave Initial;
        // the following doctype is created with empty missing identifiers.
        let f = decode("#data\n<?Target data><!doctype html><p>X\n#errors\n#document\n| <?Target data>\n| <!DOCTYPE html>\n| <html>\n|   <head>\n|   <body>\n|     <p>\n|       \"X\"\n").unwrap();
        let e = parsed_expected(&f[0], HtmlDocumentMode::NoQuirks).unwrap();
        check(&request(f[0].input), &e).unwrap();
    }

    #[test]
    fn completed_current_html_pi_and_utf16_normalization_use_literal_source_expectations() {
        let source = crate::HtmlSource::utf16(DocumentString::from_utf16(vec![
            60, 63, 84, 97, 114, 103, 101, 116, 32, 100, 97, 116, 97, 62, 60, 112, 62, 13, 10,
            0xd800, 38, 35, 49, 50, 56, 53, 49, 50, 59, 0,
        ]));
        let request =
            crate::HtmlDocumentParseRequest::offline(source, super::super::tests::limits(1_000));
        let f = decode("#data\nx\n#errors\n#document\n| <?Target data>\n| <html>\n|   <head>\n|   <body>\n|     <p>\n|       \"placeholder\"\n").unwrap();
        let mut e = expected(&f[0], HtmlDocumentMode::Quirks).unwrap();
        e.nodes[6].kind = Kind::Text(vec![10, 0xd800, 0xd83d, 0xde00]);
        check(&request, &e).unwrap();
    }

    #[test]
    fn completed_doctype_creator_empty_strings_and_attributes_remain_observable() {
        let fixture = decode("#data\nx\n#errors\n#document\n| <!DOCTYPE html>\n| <html>\n|   <head>\n|   <body>\n|     <p>\n|       a=\"second\"\n|       z=\"first\"\n").unwrap();
        let mut e = parsed_expected(&fixture[0], HtmlDocumentMode::NoQuirks).unwrap();
        // Source supplement disambiguates the corpus dump's collapsed optional
        // identifiers; no native null/empty field is erased before comparison.
        e.nodes[1].kind = Kind::Doctype {
            name: units("html"),
            public: Some(Vec::new()),
            system: Some(Vec::new()),
        };
        check(
            &request("<!doctype html PUBLIC \"\"><p z=first a=second>"),
            &e,
        )
        .unwrap();
    }

    #[test]
    fn real_mismatch_is_not_promoted_to_match_and_cleanup_preserves_completed_owner() {
        let fixture = decode(
            "#data\nx\n#errors\n#document\n| <html>\n|   <head>\n|   <body>\n|     \"wrong\"\n",
        )
        .unwrap();
        let e = expected(&fixture[0], HtmlDocumentMode::Quirks).unwrap();
        assert!(matches!(
            check(&request("right"), &e),
            Err(CaseError::Observation(AdapterError::Mismatch))
        ));
    }

    #[test]
    fn metered_projection_cancel_and_work_failure_release_all_consumer_storage() {
        for cancel in [false, true] {
            let request = request("<template><p>payload</template>");
            let mut driver = Some(crate::parser::tree_builder::entry::initialized_driver(
                &request,
            ));
            loop {
                match driver.as_mut().unwrap().step().unwrap() {
                    super::super::head::HeadStep::Consumed => {}
                    super::super::head::HeadStep::Complete => break,
                    other => panic!("unexpected {other:?}"),
                }
            }
            let mut completed =
                super::super::StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
            {
                let mut view = completed.native_view().unwrap();
                let before = view
                    .usage()
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .current;
                if cancel {
                    request.cancellation().cancel();
                } else {
                    let left = request.limits().work_credits()
                        - view
                            .usage()
                            .resource(HtmlParseResource::WorkCredits)
                            .current;
                    view.control()
                        .charge(WorkCategory::StateStep, left - 1)
                        .unwrap();
                }
                let mut projection = MeteredProjection::new().unwrap();
                assert!(projection.build(&mut view).is_err());
                let stop = view.control().stop().unwrap();
                projection.release(&mut view).unwrap();
                assert_eq!(
                    view.usage()
                        .resource(HtmlParseResource::ManagedCapacityBytes)
                        .current,
                    before
                );
                assert_eq!(view.control().stop(), Some(stop));
            }
            completed.release().unwrap();
        }
    }
    fn completed<'a>(
        request: &'a crate::HtmlDocumentParseRequest,
    ) -> super::super::end::CompletedStandaloneDocument<'a> {
        let mut driver = Some(crate::parser::tree_builder::entry::initialized_driver(
            request,
        ));
        loop {
            match driver.as_mut().unwrap().step().unwrap() {
                super::super::head::HeadStep::Consumed => {}
                super::super::head::HeadStep::Complete => break,
                other => panic!("unexpected {other:?}"),
            }
        }
        super::super::StandaloneDocumentDriver::take_completed(&mut driver).unwrap()
    }

    #[test]
    fn completed_projection_preserves_native_attribute_order_and_successful_reuse() {
        let request = request("<p z=first a=second>");
        let mut completed = completed(&request);
        {
            let mut view = completed.native_view().unwrap();
            let before_capacity = view
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current;
            let before_units = view
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current;
            let before_aux = view
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current;
            let initial_work = view
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
            for _ in 0..2 {
                let mut output = MeteredProjection::new().unwrap();
                output.build(&mut view).unwrap();
                assert!(
                    view.usage()
                        .resource(HtmlParseResource::ManagedCapacityBytes)
                        .current
                        > before_capacity
                );
                assert!(
                    view.usage()
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .current
                        > before_units
                );
                let node = read(&mut output.nodes, 4, view.control()).unwrap();
                let MeteredKind::Element(_, attrs) = node.kind else {
                    panic!("p")
                };
                assert_eq!(attrs.len, 2);
                let first = read(&mut output.attrs, attrs.start, view.control()).unwrap();
                let second = read(&mut output.attrs, attrs.start + 1, view.control()).unwrap();
                assert!(
                    output
                        .equal_units(first.name.local, &[122], view.control())
                        .unwrap()
                );
                assert!(
                    output
                        .equal_units(first.value, &[102, 105, 114, 115, 116], view.control())
                        .unwrap()
                );
                assert!(
                    output
                        .equal_units(second.name.local, &[97], view.control())
                        .unwrap()
                );
                output.release(&mut view).unwrap();
                assert_eq!(
                    view.usage()
                        .resource(HtmlParseResource::ManagedCapacityBytes)
                        .current,
                    before_capacity
                );
                assert_eq!(
                    view.usage()
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .current,
                    before_units
                );
                assert_eq!(
                    view.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    before_aux
                );
                assert!(view.control().stop().is_none());
            }
            assert!(
                view.usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current
                    > initial_work
            );
            assert!(
                view.usage()
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .high_water
                    > before_capacity
            );
        }
        completed.release().unwrap();
    }

    #[test]
    fn actual_reserved_capacity_limit_preserves_first_stop_and_releases_projection_tickets() {
        let limits = super::super::tests::limits(1_000)
            .into_builder()
            .managed_capacity_bytes(1_000_000)
            .finish()
            .unwrap();
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8("<p>text"), limits);
        let mut completed = completed(&request);
        {
            let mut view = completed.native_view().unwrap();
            let baseline = view
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current;
            let (mut held, mut ticket) = view.control().new_vector::<u8>().unwrap();
            // A real uninitialized allocation with its original capacity ticket;
            // this is not a proxy ledger increment or a fake allocator failure.
            let remaining = limits.managed_capacity_bytes() - baseline;
            view.control()
                .reserve_vector(&mut held, &mut ticket, (remaining - 1) as usize)
                .unwrap();
            let held_capacity = view
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current;
            let mut output = MeteredProjection::new().unwrap();
            assert!(matches!(
                output.build(&mut view),
                Err(AdapterError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedCapacityBytes,
                    ..
                }))
            ));
            let first = view.control().stop().unwrap();
            output.release(&mut view).unwrap();
            assert_eq!(
                view.usage()
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .current,
                held_capacity
            );
            assert!(matches!(view.root(), Err(SessionError::Stop(stop)) if stop == first));
            view.control().release_vector(held, ticket).unwrap();
            assert_eq!(
                view.usage()
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .current,
                baseline
            );
            assert_eq!(view.control().stop(), Some(first));
        }
        completed.release().unwrap();
    }
    #[test]
    fn empty_fragment_dump_is_valid_metadata_but_never_an_ordinary_completion_case() {
        let f = decode("#data\n\n#errors\n#document-fragment\ndiv\n#document\n").unwrap();
        assert_eq!(f[0].input, "");
        assert_eq!(f[0].expected, "");
        assert_eq!(f[0].fragment, Some("div"));
        assert_eq!(
            expected(&f[0], HtmlDocumentMode::NoQuirks)
                .unwrap()
                .nodes
                .len(),
            1
        );
    }

    #[test]
    fn expected_attributes_must_follow_upstream_utf16_name_order_before_children() {
        for tree in [
            "| <p>\n|   z=\"first\"\n|   a=\"second\"",
            "| <p>\n|   <b>\n|   id=\"late\"",
        ] {
            let raw = format!("#data\nx\n#errors\n#document\n{tree}\n");
            assert_eq!(
                expected(&decode(&raw).unwrap()[0], HtmlDocumentMode::NoQuirks),
                Err(FixtureError::Attribute)
            );
        }
    }
}
