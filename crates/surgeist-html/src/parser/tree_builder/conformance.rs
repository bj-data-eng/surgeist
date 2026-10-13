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
                super::head::HeadStep::NeedInput | super::head::HeadStep::Boundary(_) => {
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

/// Feeding tests consume the same admitted native projection/comparison owner;
/// their literal expectations do not come from a complete-source parse.
pub(super) fn assert_completed_literal(
    completed: &mut super::end::CompletedStandaloneDocument<'_>,
    dump: &str,
    mode: HtmlDocumentMode,
) {
    let fixture = Fixture {
        ordinal: 0,
        input: "",
        errors: Vec::new(),
        new_errors: None,
        fragment: None,
        scripting: FixtureScripting::Disabled,
        expected: dump,
    };
    let expected = parsed_expected(&fixture, mode).unwrap();
    let mut view = completed.native_view().unwrap();
    let mut projection = MeteredProjection::new().unwrap();
    let result = projection
        .build(&mut view)
        .and_then(|()| projection.compare(&expected, &mut view));
    projection.release(&mut view).unwrap();
    assert!(result.is_ok(), "completed literal comparison: {result:?}");
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

#[cfg(test)]
mod whole_source_comparisons {
    use super::*;
    use crate::construction::parser_session::{
        StringLocation,
        evidence::{
            DiagnosticContext, DiagnosticKind, DiagnosticLocation, DiagnosticLocationRole,
            DiagnosticRecord, DiagnosticSeverity, DiagnosticToken,
        },
        provenance::{
            AssociationRelation, LexicalDisposition, LexicalKind, LexicalOccurrence, LexicalPart,
            NativeAssociation, NativeAttributeField, NativeDestination,
        },
        report::MetadataStep,
    };
    use crate::parser::{
        control::ParseUsage,
        tokenizer::{
            states::State,
            token::{DiagnosticCode, DiagnosticPhase, OriginKind, TextOrigin},
        },
        tree_builder::{
            StandaloneDocumentDriver,
            head::HeadStep,
            report::ImmutableParseReport,
            seal::{OrdinaryInitializationOwner, SealedOrdinaryConstruction},
        },
    };
    use crate::{DocumentString, HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};
    use std::time::{Duration, Instant};

    const PREFIX: &str = "<!doctype html><html><head></head><body>";
    const SUFFIX: &str = "</body></html>";
    const WORK: [WorkCategory; 8] = [
        WorkCategory::InputUnitRead,
        WorkCategory::InspectByte,
        WorkCategory::CopyByte,
        WorkCategory::StateStep,
        WorkCategory::RecordVisit,
        WorkCategory::RecordWrite,
        WorkCategory::ReserveByte,
        WorkCategory::ProviderCall,
    ];

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Origin {
        node: usize,
        output: (usize, usize),
        source: (usize, usize),
        kind: OriginKind,
    }
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Error {
        kind: DiagnosticKind,
        context: DiagnosticContext,
        role: DiagnosticLocationRole,
        token: Option<DiagnosticToken>,
        anchor: &'static str,
        branch: &'static str,
        source: (usize, usize),
    }
    struct Literal {
        label: &'static str,
        raw: Vec<u16>,
        expected: Projection,
        origins: Vec<Origin>,
        errors: Vec<Error>,
        attributes: Vec<((usize, usize), LexicalDisposition)>,
    }
    struct Observation {
        projection: Projection,
        phases: Vec<(&'static str, ParseUsage, Duration)>,
    }
    fn append(p: &mut Projection, parent: usize, kind: Kind) -> usize {
        let index = p.nodes.len();
        p.nodes.push(Node {
            kind,
            parent: Some(parent),
            children: Vec::new(),
            template: None,
            shadow: None,
            owner: None,
        });
        p.nodes[parent].children.push(index);
        index
    }
    fn element(local: &str, attrs: Vec<Attr>) -> Kind {
        Kind::Element(
            Name {
                namespace: Some(units("http://www.w3.org/1999/xhtml")),
                prefix: None,
                local: units(local),
            },
            attrs,
        )
    }
    fn document() -> Projection {
        let mut p = Projection {
            nodes: vec![Node {
                kind: Kind::Document(HtmlDocumentMode::NoQuirks),
                parent: None,
                children: Vec::new(),
                template: None,
                shadow: None,
                owner: None,
            }],
        };
        append(
            &mut p,
            0,
            Kind::Doctype {
                name: units("html"),
                public: Some(Vec::new()),
                system: Some(Vec::new()),
            },
        );
        append(&mut p, 0, element("html", Vec::new()));
        append(&mut p, 2, element("head", Vec::new()));
        append(&mut p, 2, element("body", Vec::new()));
        p
    }
    fn literal(
        label: &'static str,
        body: Vec<u16>,
        expected: Projection,
        origins: Vec<Origin>,
        errors: Vec<Error>,
        attributes: Vec<((usize, usize), LexicalDisposition)>,
    ) -> Literal {
        let mut raw = units(PREFIX);
        raw.extend(body);
        raw.extend(units(SUFFIX));
        Literal {
            label,
            raw,
            expected,
            origins,
            errors,
            attributes,
        }
    }
    fn plain(n: usize) -> Literal {
        let payload = units(&"Aé😀".repeat(n));
        let mut expected = document();
        let p = append(&mut expected, 4, element("p", Vec::new()));
        if !payload.is_empty() {
            append(&mut expected, p, Kind::Text(payload.clone()));
        }
        let mut origins = Vec::new();
        for i in 0..n {
            for (a, b) in [(0, 1), (1, 2), (2, 4)] {
                origins.push(Origin {
                    node: 6,
                    output: (4 * i + a, 4 * i + b),
                    source: (PREFIX.len() + 3 + 4 * i + a, PREFIX.len() + 3 + 4 * i + b),
                    kind: OriginKind::Literal,
                });
            }
        }
        let mut body = units("<p>");
        body.extend(payload);
        body.extend(units("</p>"));
        literal("plain", body, expected, origins, Vec::new(), Vec::new())
    }
    fn references(n: usize, crlf: bool) -> Literal {
        let spelling = if crlf {
            "A\r\n&amp;&#128512;&NotEqualTilde;"
        } else {
            "A\n&amp;&#128512;&NotEqualTilde;"
        };
        let mut expected = document();
        let p = append(&mut expected, 4, element("p", Vec::new()));
        if n != 0 {
            append(
                &mut expected,
                p,
                Kind::Text([65, 10, 38, 0xd83d, 0xde00, 0x2242, 0x338].repeat(n)),
            );
        }
        let mut origins = Vec::new();
        let newline = if crlf { 2 } else { 1 };
        for i in 0..n {
            let s = PREFIX.len() + 3 + i * spelling.len();
            let o = 7 * i;
            for (out, src, kind) in [
                ((o, o + 1), (s, s + 1), OriginKind::Literal),
                (
                    (o + 1, o + 2),
                    (s + 1, s + 1 + newline),
                    if crlf {
                        OriginKind::Newline
                    } else {
                        OriginKind::Literal
                    },
                ),
                (
                    (o + 2, o + 3),
                    (s + 1 + newline, s + 6 + newline),
                    OriginKind::CharacterReference,
                ),
                (
                    (o + 3, o + 5),
                    (s + 6 + newline, s + 15 + newline),
                    OriginKind::CharacterReference,
                ),
                (
                    (o + 5, o + 6),
                    (s + 15 + newline, s + 30 + newline),
                    OriginKind::CharacterReference,
                ),
                (
                    (o + 6, o + 7),
                    (s + 15 + newline, s + 30 + newline),
                    OriginKind::CharacterReference,
                ),
            ] {
                origins.push(Origin {
                    node: 6,
                    output: out,
                    source: src,
                    kind,
                });
            }
        }
        literal(
            if crlf {
                "CRLF-references"
            } else {
                "LF-references"
            },
            units(&format!("<p>{}</p>", spelling.repeat(n))),
            expected,
            origins,
            Vec::new(),
            Vec::new(),
        )
    }
    fn attributes(n: usize) -> Literal {
        let mut body = String::from("<p");
        let mut attrs = Vec::new();
        let mut errors = Vec::new();
        let mut spans = Vec::new();
        for duplicate in [false, true] {
            for i in 0..n {
                let name = format!("sharedprefix{i:04}");
                let spelling = if duplicate {
                    name.to_ascii_uppercase()
                } else {
                    name.clone()
                };
                let value = if duplicate { "discard" } else { "first" };
                body.push(' ');
                let start = PREFIX.len() + body.len();
                body.push_str(&format!("{spelling}='{value}'"));
                spans.push((
                    (start, PREFIX.len() + body.len()),
                    if duplicate {
                        LexicalDisposition::DuplicateAttribute
                    } else {
                        LexicalDisposition::StoredAttribute
                    },
                ));
                if duplicate {
                    errors.push(Error {
                        kind: DiagnosticKind::Tokenizer(DiagnosticCode::DuplicateAttribute),
                        context: DiagnosticContext::Tokenizer {
                            phase: DiagnosticPhase::Tokenizer,
                            state: State::AttributeName,
                        },
                        role: DiagnosticLocationRole::InputFrontier,
                        token: None,
                        anchor: "parse-error-duplicate-attribute",
                        branch: "duplicate-attribute",
                        source: (start + name.len(), start + name.len() + 1),
                    });
                } else {
                    attrs.push(Attr {
                        name: Name {
                            namespace: None,
                            prefix: None,
                            local: units(&name),
                        },
                        value: units(value),
                    });
                }
            }
        }
        body.push_str("></p>");
        let mut expected = document();
        append(&mut expected, 4, element("p", attrs));
        literal(
            "attributes",
            units(&body),
            expected,
            Vec::new(),
            errors,
            spans,
        )
    }
    fn siblings(n: usize) -> Literal {
        let mut expected = document();
        let mut origins = Vec::new();
        for i in 0..n {
            let p = append(&mut expected, 4, element("p", Vec::new()));
            append(&mut expected, p, Kind::Text(vec![88]));
            let s = PREFIX.len() + 8 * i + 3;
            origins.push(Origin {
                node: 6 + 2 * i,
                output: (0, 1),
                source: (s, s + 1),
                kind: OriginKind::Literal,
            });
        }
        literal(
            "siblings",
            units(&"<p>X</p>".repeat(n)),
            expected,
            origins,
            Vec::new(),
            Vec::new(),
        )
    }

    // Pre-observation fixture envelope, not a parser default or a success promise.
    // K bounds original units + expected records. Each scalar/attribute can own
    // token, lexical, cause, history and final rows; 512K leaves room for these
    // simultaneous owners and the metered observer. Its node/edge comparison is
    // quadratic, so the work envelope includes K² and compiled record bytes.
    fn limits(l: &Literal, source_len: usize) -> HtmlParseLimits {
        let k = (l.raw.len() + l.expected.nodes.len() + l.errors.len() + l.attributes.len() + 64)
            as u64;
        let layout = (size_of::<NativeAssociation>()
            + size_of::<TextOrigin>()
            + size_of::<LexicalOccurrence>()
            + size_of::<LexicalPart>()
            + size_of::<DiagnosticRecord>()
            + size_of::<DiagnosticLocation>()
            + size_of::<MeteredNode>()
            + size_of::<MeteredAttr>()
            + size_of::<Edge>()
            + size_of::<Pending>()
            + size_of::<[usize; 64]>()) as u64;
        let capacity = 128 * k * layout;
        HtmlParseLimits::builder()
            .source_units(source_len as u64)
            .buffered_utf16_units(256 * k)
            .token_attributes(64 * k)
            .auxiliary_entries(512 * k)
            .managed_nodes(64 * k)
            .managed_edges(64 * k)
            .managed_attributes(64 * k)
            .managed_utf16_units(256 * k)
            .source_map_entries(512 * k)
            .source_map_payload_units(256 * k)
            .diagnostics(64 * k)
            .diagnostic_payload_units(256 * k)
            .context_effects(0)
            .open_elements(64)
            .tree_depth(64)
            .managed_capacity_bytes(capacity)
            .peak_transient_bytes(2 * capacity)
            .atomic_layout_bytes(capacity)
            .work_credits(512 * k * k * layout)
            .finish()
            .unwrap()
    }
    fn source(l: &Literal, utf8: bool) -> HtmlSource {
        if utf8 {
            HtmlSource::utf8(String::from_utf16(&l.raw).expect("scalar-only UTF8 family"))
        } else {
            HtmlSource::utf16(DocumentString::from_utf16(l.raw.clone()))
        }
    }
    fn offset(l: &Literal, utf8: bool, position: usize) -> usize {
        if utf8 {
            String::from_utf16(&l.raw[..position])
                .expect("scalar boundary")
                .len()
        } else {
            position
        }
    }
    fn check_range(source: &HtmlSource, range: &crate::HtmlSourceRange, expected: (usize, usize)) {
        assert_eq!(range, &source.range(expected.0, expected.1).unwrap());
    }
    fn same_origins(actual: &[Origin], expected: &[Origin]) -> bool {
        let mut actual = actual.to_vec();
        let mut expected = expected.to_vec();
        actual.sort_by_key(|o| (o.node, o.source, o.output));
        expected.sort_by_key(|o| (o.node, o.source, o.output));
        actual == expected
    }
    fn same_fields(actual: &[(usize, u8, Origin)], expected: &[(usize, u8, Origin)]) -> bool {
        let mut actual = actual.to_vec();
        let mut expected = expected.to_vec();
        actual.sort_by_key(|(slot, field, o)| (o.node, *slot, *field, o.source, o.output));
        expected.sort_by_key(|(slot, field, o)| (o.node, *slot, *field, o.source, o.output));
        actual == expected
    }
    #[test]
    fn field_oracle_rejects_sibling_and_equal_attribute_destination_swaps() {
        let literal = siblings(2);
        let expected = literal.origins;
        assert_eq!(expected.iter().map(|o| o.node).collect::<Vec<_>>(), [6, 8]);
        assert!(same_origins(&expected, &expected));
        let mut swapped = expected.clone();
        swapped[0].node = 8;
        swapped[1].node = 6;
        assert_eq!(
            swapped
                .iter()
                .map(|o| (o.output, o.source, o.kind))
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|o| (o.output, o.source, o.kind))
                .collect::<Vec<_>>()
        );
        assert!(!same_origins(&swapped, &expected));

        let literal = attributes(2);
        let Kind::Element(_, values) = &literal.expected.nodes[5].kind else {
            unreachable!()
        };
        assert_eq!(values[0].value, units("first"));
        assert_eq!(values[1].value, units("first"));
        let starts: Vec<_> = literal
            .attributes
            .iter()
            .filter(|(_, disposition)| *disposition == LexicalDisposition::StoredAttribute)
            .map(|(span, _)| span.0)
            .collect();
        let expected: Vec<_> = values
            .iter()
            .enumerate()
            .map(|(slot, value)| {
                let start = starts[slot] + value.name.local.len() + 2;
                (
                    slot,
                    2,
                    Origin {
                        node: 5,
                        output: (0, 1),
                        source: (start, start + 1),
                        kind: OriginKind::Literal,
                    },
                )
            })
            .collect();
        assert!(same_fields(&expected, &expected));
        let mut swapped = expected.clone();
        swapped[0].0 = 1;
        swapped[1].0 = 0;
        assert_eq!(
            swapped
                .iter()
                .map(|(_, field, o)| (*field, o.output, o.source, o.kind))
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|(_, field, o)| (*field, o.output, o.source, o.kind))
                .collect::<Vec<_>>()
        );
        assert!(!same_fields(&swapped, &expected));
    }
    fn evidence(
        l: &Literal,
        utf8: bool,
        report: &ImmutableParseReport,
        positions: &[(bool, usize, usize)],
    ) {
        let source = report.source();
        let diagnostics = report.diagnostics();
        assert_eq!(diagnostics.len(), l.errors.len());
        for (i, expected) in l.errors.iter().enumerate() {
            let (record, location) = diagnostics.retained_record(i).unwrap();
            assert_eq!(record.kind, expected.kind);
            assert_eq!(record.context, expected.context);
            assert_eq!(record.severity, DiagnosticSeverity::Recoverable);
            assert_eq!(location.role, expected.role);
            assert_eq!(location.token, expected.token);
            assert_eq!(record.rule.anchor, expected.anchor);
            assert_eq!(record.rule.branch, expected.branch);
            check_range(
                source,
                &location.source,
                (
                    offset(l, utf8, expected.source.0),
                    offset(l, utf8, expected.source.1),
                ),
            );
        }
        let associations = report.history().associations().unwrap();
        let position = |attribute, owner| {
            positions
                .iter()
                .find(|(is_attribute, actual, _)| *is_attribute == attribute && *actual == owner)
                .expect("field owner has a same-owner qualified structural position")
                .2
        };
        let mut origins = Vec::new();
        let mut attributes = Vec::new();
        let mut pi = Vec::new();
        let mut eof = 0;
        for i in 0..associations.len() {
            let row = associations.row(i).unwrap();
            if let Some(range) = &row.source {
                check_range(source, range, (range.start(), range.end()));
            }
            if row.relation == AssociationRelation::Output
                && matches!(
                    row.destination,
                    NativeDestination::Field(StringLocation::Text(_))
                )
            {
                assert!(row.group.is_some());
                let range = row
                    .source
                    .as_ref()
                    .expect("all selected text is source-bearing");
                origins.push(Origin {
                    node: match row.destination {
                        NativeDestination::Field(StringLocation::Text(owner)) => {
                            position(false, owner)
                        }
                        _ => unreachable!(),
                    },
                    output: (row.start, row.end),
                    source: (range.start(), range.end()),
                    kind: row.kind.expect("selected output origin"),
                });
            }
            if row.relation == AssociationRelation::Output {
                let field = match row.destination {
                    NativeDestination::Field(StringLocation::AttributeName(owner, slot)) => {
                        Some((position(true, owner), slot, 1, &mut attributes))
                    }
                    NativeDestination::Field(StringLocation::AttributeValue(owner, slot)) => {
                        Some((position(true, owner), slot, 2, &mut attributes))
                    }
                    NativeDestination::Field(StringLocation::ProcessingInstructionTarget(
                        owner,
                    )) => Some((position(false, owner), 0, 1, &mut pi)),
                    NativeDestination::Field(StringLocation::ProcessingInstructionData(owner)) => {
                        Some((position(false, owner), 0, 2, &mut pi))
                    }
                    _ => None,
                };
                if let Some((node, slot, field, rows)) = field {
                    assert!(row.group.is_some());
                    let range = row.source.as_ref().expect("selected authored field origin");
                    rows.push((
                        slot,
                        field,
                        Origin {
                            node,
                            output: (row.start, row.end),
                            source: (range.start(), range.end()),
                            kind: row.kind.unwrap(),
                        },
                    ));
                }
            }
            if row.destination == NativeDestination::LexicalEnd {
                check_range(
                    source,
                    row.source.as_ref().unwrap(),
                    (source.len(), source.len()),
                );
                eof += 1;
            }
        }
        let expected: Vec<_> = l
            .origins
            .iter()
            .map(|origin| Origin {
                node: origin.node,
                output: origin.output,
                source: (
                    offset(l, utf8, origin.source.0),
                    offset(l, utf8, origin.source.1),
                ),
                kind: origin.kind,
            })
            .collect();
        assert!(
            same_origins(&origins, &expected),
            "qualified text origins: {origins:?} vs {expected:?}"
        );
        assert_eq!(eof, 1);
        let mut actual = Vec::new();
        let mut i = 0;
        while let Some(occurrence) = associations.lexical_occurrence(i) {
            assert_ne!(occurrence.disposition, LexicalDisposition::Draft);
            check_range(
                source,
                &occurrence.source,
                (occurrence.source.start(), occurrence.source.end()),
            );
            if occurrence.kind == LexicalKind::Attribute {
                actual.push((
                    (occurrence.source.start(), occurrence.source.end()),
                    occurrence.disposition,
                ));
            }
            i += 1;
        }
        let expected: Vec<_> = l
            .attributes
            .iter()
            .map(|(span, d)| ((offset(l, utf8, span.0), offset(l, utf8, span.1)), *d))
            .collect();
        assert_eq!(actual, expected);
        let authored: Vec<_> = l
            .expected
            .nodes
            .iter()
            .enumerate()
            .flat_map(|(node, record)| match &record.kind {
                Kind::Element(_, attrs) => attrs
                    .iter()
                    .enumerate()
                    .map(move |(slot, attr)| (node, slot, attr))
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            })
            .collect();
        let stored: Vec<_> = l
            .attributes
            .iter()
            .filter(|(_, disposition)| *disposition == LexicalDisposition::StoredAttribute)
            .collect();
        assert_eq!(authored.len(), stored.len());
        let mut expected_attributes = Vec::new();
        for ((node, slot, attribute), (span, _)) in authored.into_iter().zip(stored) {
            for (field, raw_start, length) in [
                (1, span.0, attribute.name.local.len()),
                (
                    2,
                    span.0 + attribute.name.local.len() + 2,
                    attribute.value.len(),
                ),
            ] {
                for i in 0..length {
                    expected_attributes.push((
                        slot,
                        field,
                        Origin {
                            node,
                            output: (i, i + 1),
                            source: (
                                offset(l, utf8, raw_start + i),
                                offset(l, utf8, raw_start + i + 1),
                            ),
                            kind: OriginKind::Literal,
                        },
                    ));
                }
            }
        }
        assert!(
            same_fields(&attributes, &expected_attributes),
            "qualified attribute origins: {attributes:?} vs {expected_attributes:?}"
        );
        let mut expected_pi = Vec::new();
        for (node, _) in l
            .expected
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, record)| matches!(record.kind, Kind::Pi { .. }))
        {
            let start = usize::from(l.raw[0] == 0xfeff);
            for (field, raw_start, length) in [(1, start + 2, 6), (2, start + 9, 4)] {
                for i in 0..length {
                    expected_pi.push((
                        0,
                        field,
                        Origin {
                            node,
                            output: (i, i + 1),
                            source: (
                                offset(l, utf8, raw_start + i),
                                offset(l, utf8, raw_start + i + 1),
                            ),
                            kind: OriginKind::Literal,
                        },
                    ));
                }
            }
        }
        pi.sort_by_key(|(slot, field, o)| (o.node, *slot, *field, o.source, o.output));
        expected_pi.sort_by_key(|(slot, field, o)| (o.node, *slot, *field, o.source, o.output));
        assert_eq!(pi, expected_pi);
        eprintln!(
            "{} published-evidence utf8={utf8} d1-rows={} lexical-occurrences={i} diagnostics={} expected-nodes={} native-text-origins={}",
            l.label,
            associations.len(),
            diagnostics.len(),
            l.expected.nodes.len(),
            l.origins.len()
        );
        let document = report.document().unwrap();
        let mut metadata = report.metadata();
        let mut created = Vec::new();
        loop {
            match metadata.advance() {
                MetadataStep::Complete => break,
                MetadataStep::Skipped => {}
                MetadataStep::Record(record) => {
                    assert!(document.node(&record.node().id()).is_ok());
                    // Ordinary creation intentionally has no captured-registry
                    // creation metadata; absence is not a fabricated state.
                    assert!(record.creation().is_none());
                    if let Some(
                        crate::construction::parser_session::report::MetadataDocument::Owned(owner),
                    ) = record.node_document()
                    {
                        assert_eq!(owner.id(), document.root().id());
                        let node = record.node();
                        if let HtmlNodeKind::Element(element) = node.kind() {
                            created.push(element.name().local_name().code_units().to_vec());
                        }
                    }
                }
            }
        }
        let expected: Vec<_> = l
            .expected
            .nodes
            .iter()
            .filter_map(|node| match &node.kind {
                Kind::Element(name, _) => Some(name.local.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(created, expected);
    }
    /// Bind actual storage owners through the existing checked Node-row lender.
    /// The returned scalar pairs are test-owned observations, like phase snapshots;
    /// traversal handles/scratch and every lender lookup use the original control.
    fn destination_positions(
        completed: &mut super::super::end::CompletedStandaloneDocument<'_>,
    ) -> Vec<(bool, usize, usize)> {
        let mut ids = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        let mut pending = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        {
            let mut view = completed.native_view().unwrap();
            let root = view.root().unwrap();
            pending.try_push(view.control(), root).unwrap();
            while let Some(node) = pending.try_pop(view.control()).unwrap() {
                assert!(view.template_contents(&node).unwrap().is_none());
                assert!(view.shadow(&node).unwrap().is_none());
                let count = view.child_count(&node).unwrap();
                for i in (0..count).rev() {
                    let child = view.child_at(&node, i).unwrap().unwrap();
                    pending.try_push(view.control(), child).unwrap();
                }
                ids.try_push(view.control(), node).unwrap();
            }
            pending.release(view.control()).unwrap();
        }
        let mut positions = Vec::new();
        for position in 0..ids.len() {
            let node = {
                let mut view = completed.native_view().unwrap();
                id_at(&mut ids, position, view.control()).unwrap()
            };
            let mut rows = completed.native_associations().unwrap();
            for i in 0..rows.len() {
                let destination = rows.row(i).unwrap().unwrap().0.destination;
                let matched = match destination {
                    NativeDestination::Node(owner) if rows.node_for(i, &node).unwrap() => {
                        Some((false, owner))
                    }
                    NativeDestination::Field(StringLocation::AttributeName(container, slot))
                        if rows
                            .attribute_for(i, &node, slot, NativeAttributeField::Name)
                            .unwrap() =>
                    {
                        Some((true, container))
                    }
                    NativeDestination::Field(StringLocation::AttributeValue(container, slot))
                        if rows
                            .attribute_for(i, &node, slot, NativeAttributeField::Value)
                            .unwrap() =>
                    {
                        Some((true, container))
                    }
                    _ => None,
                };
                if let Some((attribute, owner)) = matched
                    && !positions.contains(&(attribute, owner, position))
                {
                    assert!(!positions.iter().any(|(prior_attribute, previous, _)| {
                        *prior_attribute == attribute && *previous == owner
                    }));
                    positions.push((attribute, owner, position));
                }
            }
        }
        let mut view = completed.native_view().unwrap();
        ids.release(view.control()).unwrap();
        positions
    }
    struct SealObservation {
        owner: SealedOrdinaryConstruction,
        phases: Vec<(&'static str, ParseUsage, Duration)>,
        positions: Vec<(bool, usize, usize)>,
    }
    fn sealed(
        l: &Literal,
        utf8: bool,
        observe: bool,
        override_work: Option<u64>,
    ) -> SealObservation {
        let source = source(l, utf8);
        let mut caps = limits(l, source.len());
        if let Some(work) = override_work {
            caps = caps.into_builder().work_credits(work).finish().unwrap();
        }
        let mut owner =
            OrdinaryInitializationOwner::new(HtmlDocumentParseRequest::offline(source, caps));
        let mut phases = Vec::new();
        let positions;
        let mut clock = Instant::now();
        let parts = {
            let mut attempt = owner.begin().unwrap();
            let mut driver = Some(attempt.take_driver().unwrap());
            phases.push((
                "initialization",
                driver.as_ref().unwrap().usage().clone(),
                clock.elapsed(),
            ));
            clock = Instant::now();
            loop {
                match driver.as_mut().unwrap().step().unwrap() {
                    HeadStep::Consumed => {}
                    HeadStep::Complete => break,
                    HeadStep::NeedInput => panic!("sealed ordinary source requested more input"),
                    HeadStep::Boundary(b) => panic!("unexpected ordinary boundary {b:?}"),
                }
            }
            phases.push((
                "complete",
                driver.as_ref().unwrap().usage().clone(),
                clock.elapsed(),
            ));
            clock = Instant::now();
            let mut completed = StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
            completed.prepare_native_associations().unwrap();
            phases.push((
                "associations",
                completed.usage().unwrap().clone(),
                clock.elapsed(),
            ));
            clock = Instant::now();
            if observe {
                let mut view = completed.native_view().unwrap();
                let mut p = MeteredProjection::new().unwrap();
                p.build(&mut view).unwrap();
                phases.push(("native-observer", view.usage().clone(), clock.elapsed()));
                clock = Instant::now();
                let compared = p.compare(&l.expected, &mut view);
                p.release(&mut view).unwrap();
                compared.unwrap();
                phases.push(("expected-comparison", view.usage().clone(), clock.elapsed()));
                clock = Instant::now();
            }
            positions = destination_positions(&mut completed);
            phases.push((
                "destination-mapping",
                completed.usage().unwrap().clone(),
                clock.elapsed(),
            ));
            clock = Instant::now();
            let parts = completed.freeze_parts().unwrap();
            phases.push(("frozen", parts.final_usage().clone(), clock.elapsed()));
            clock = Instant::now();
            parts
        };
        let sealed = SealedOrdinaryConstruction::take(
            &mut Some(owner.take_request().unwrap()),
            &mut Some(parts),
        )
        .unwrap();
        phases.push(("sealed", sealed.retained_usage().clone(), clock.elapsed()));
        SealObservation {
            owner: sealed,
            phases,
            positions,
        }
    }
    fn run(l: &Literal, utf8: bool) -> Observation {
        let SealObservation {
            owner,
            mut phases,
            positions,
        } = sealed(l, utf8, true, None);
        let mut input = Some(owner);
        let original = input.as_ref().unwrap().source().identity().clone();
        let clock = Instant::now();
        let cancelled_after_publication = input.as_ref().unwrap().request.cancellation().clone();
        let report = ImmutableParseReport::take_document(&mut input).unwrap();
        assert!(input.is_none());
        phases.push(("report", report.usage().clone(), clock.elapsed()));
        assert_eq!(report.source().identity(), &original);
        if utf8 {
            assert_eq!(
                report.source().coordinates(),
                crate::HtmlSourceCoordinates::Utf8Bytes
            );
            assert_eq!(
                report.source().as_utf8().unwrap(),
                String::from_utf16(&l.raw).unwrap()
            );
        } else {
            assert_eq!(
                report.source().coordinates(),
                crate::HtmlSourceCoordinates::Utf16CodeUnits
            );
            assert_eq!(report.source().as_utf16().unwrap().code_units(), l.raw);
        }
        assert_eq!(report.limits().source_units(), report.source().len() as u64);
        assert_eq!(
            report
                .usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            report.source().len() as u64
        );
        assert!(report.fragment().is_none());
        assert!(report.context().is_none());
        assert!(report.effects().is_none());
        assert_eq!(
            report.requested_scripting_mode(),
            crate::HtmlParserScriptingMode::Disabled
        );
        assert_eq!(
            report.effective_scripting_mode(),
            crate::HtmlParserScriptingMode::Disabled
        );
        assert!(!report.allow_declarative_shadow_roots());
        let state = report.document_state().unwrap();
        assert_eq!(
            state.readiness,
            crate::construction::parser_session::ConstructionReadiness::Complete
        );
        assert_eq!(
            state.completion,
            crate::construction::parser_session::ConstructionCompletion::Complete
        );
        assert!(state.ready_for_post_load_tasks);
        cancelled_after_publication.cancel(); // Immutable lenders retain no live execution authority.
        evidence(l, utf8, &report, &positions);
        let clock = Instant::now();
        let projection = project(report.document().unwrap()).unwrap();
        assert_eq!(projection, l.expected);
        eprintln!("immutable-test-comparison {:?}", clock.elapsed());
        let mut previous = [0; 8];
        for (name, usage, elapsed) in &phases {
            let counts = WORK.map(|category| usage.work(category));
            assert_eq!(
                counts.iter().sum::<u64>(),
                usage.resource(HtmlParseResource::WorkCredits).current
            );
            let delta = std::array::from_fn::<_, 8, _>(|i| counts[i] - previous[i]);
            previous = counts;
            eprintln!(
                "{} {name} utf8={utf8} source={} work={delta:?} capacity={}/{} elapsed={elapsed:?}",
                l.label,
                report.source().len(),
                usage.actual_capacity_bytes,
                usage.actual_capacity_high_water
            );
            assert_eq!(
                usage.resource(HtmlParseResource::SourceUnits).current,
                report.source().len() as u64
            );
        }
        // Report usage is immutable publication evidence, not a post-drop ledger.
        // An identical fresh run exercises actual owning cleanup separately.
        drop(report);
        let SealObservation {
            owner: mut cleanup, ..
        } = sealed(l, utf8, false, None);
        cleanup.release().unwrap();
        super::super::tests::assert_released(&cleanup.parts.control);
        Observation { projection, phases }
    }
    fn repeat(l: &Literal, utf8: bool) -> Projection {
        let first = run(l, utf8);
        let second = run(l, utf8);
        assert_eq!(first.projection, second.projection);
        for ((phase, a, _), (other, b, _)) in first.phases.iter().zip(&second.phases) {
            assert_eq!(phase, other);
            assert_eq!(WORK.map(|w| a.work(w)), WORK.map(|w| b.work(w)));
            for resource in crate::settings::RESOURCES {
                if !matches!(
                    resource,
                    HtmlParseResource::ManagedCapacityBytes
                        | HtmlParseResource::PeakTransientBytes
                        | HtmlParseResource::AtomicLayoutBytes
                ) {
                    assert_eq!(a.resource(resource), b.resource(resource));
                }
            }
        }
        first.projection
    }
    #[test]
    fn literal_utf8_utf16_families_keep_exact_native_source_and_publication_contracts() {
        for n in [0, 1, 8, 32] {
            for l in [plain(n), attributes(n), siblings(n)] {
                assert_eq!(repeat(&l, true), repeat(&l, false));
            }
            let crlf = references(n, true);
            let lf = references(n, false);
            let expected = repeat(&crlf, true);
            assert_eq!(expected, repeat(&crlf, false));
            assert_eq!(expected, repeat(&lf, true));
            assert_eq!(expected, repeat(&lf, false));
        }
    }
    #[test]
    fn literal_utf16_surrogate_pair_crlf_and_reference_keep_distinct_source_origins() {
        let payload = [
            0xd800, 13, 10, 0xd83d, 0xde00, 38, 35, 49, 50, 56, 53, 49, 50, 59,
        ];
        let start = PREFIX.len() + 3;
        let mut expected = document();
        let p = append(&mut expected, 4, element("p", Vec::new()));
        append(
            &mut expected,
            p,
            Kind::Text(vec![0xd800, 10, 0xd83d, 0xde00, 0xd83d, 0xde00]),
        );
        let origins = [
            ((0, 1), (0, 1), OriginKind::Literal),
            ((1, 2), (1, 3), OriginKind::Newline),
            ((2, 4), (3, 5), OriginKind::Literal),
            ((4, 6), (5, 14), OriginKind::CharacterReference),
        ]
        .into_iter()
        .map(|(output, source, kind)| Origin {
            node: 6,
            output,
            source: (start + source.0, start + source.1),
            kind,
        })
        .collect();
        let mut body = units("<p>");
        body.extend(payload);
        body.extend(units("</p>"));
        let l = literal(
            "UTF16-surrogate",
            body,
            expected,
            origins,
            vec![Error {
                kind: DiagnosticKind::Tokenizer(DiagnosticCode::SurrogateInInputStream),
                context: DiagnosticContext::Tokenizer {
                    phase: DiagnosticPhase::InputStream,
                    state: State::Data,
                },
                role: DiagnosticLocationRole::InputFrontier,
                token: None,
                anchor: "parse-error-surrogate-in-input-stream",
                branch: "surrogate-in-input-stream",
                source: (start, start + 1),
            }],
            Vec::new(),
        );
        repeat(&l, false);
    }

    #[test]
    fn literal_initial_pi_and_leading_feff_preserve_source_defined_mode_and_errors() {
        use crate::parser::tree_builder::{
            head::{HeadCondition, InsertionMode},
            prologue::{PrologueCondition, PrologueMode},
        };
        // Initial PI is inserted without leaving Initial. Literal FEFF is not
        // ASCII whitespace/BOM decoding; it takes Initial's anything-else path.
        for feff in [false, true] {
            let raw = if feff {
                "\u{feff}<?Target data><!doctype html><p>X</p>"
            } else {
                "<?Target data><!doctype html><p>X</p>"
            };
            let mut expected = Projection {
                nodes: vec![Node {
                    kind: Kind::Document(if feff {
                        HtmlDocumentMode::Quirks
                    } else {
                        HtmlDocumentMode::NoQuirks
                    }),
                    parent: None,
                    children: Vec::new(),
                    template: None,
                    shadow: None,
                    owner: None,
                }],
            };
            if feff {
                let html = append(&mut expected, 0, element("html", Vec::new()));
                append(&mut expected, html, element("head", Vec::new()));
                let body = append(&mut expected, html, element("body", Vec::new()));
                append(&mut expected, body, Kind::Text(vec![0xfeff]));
                append(
                    &mut expected,
                    body,
                    Kind::Pi {
                        target: units("Target"),
                        data: units("data"),
                    },
                );
                let p = append(&mut expected, body, element("p", Vec::new()));
                append(&mut expected, p, Kind::Text(vec![88]));
            } else {
                append(
                    &mut expected,
                    0,
                    Kind::Pi {
                        target: units("Target"),
                        data: units("data"),
                    },
                );
                append(
                    &mut expected,
                    0,
                    Kind::Doctype {
                        name: units("html"),
                        public: Some(Vec::new()),
                        system: Some(Vec::new()),
                    },
                );
                let html = append(&mut expected, 0, element("html", Vec::new()));
                append(&mut expected, html, element("head", Vec::new()));
                let body = append(&mut expected, html, element("body", Vec::new()));
                let p = append(&mut expected, body, element("p", Vec::new()));
                append(&mut expected, p, Kind::Text(vec![88]));
            }
            let pi = "<?Target data>".len();
            let f = usize::from(feff);
            let x = f + pi + "<!doctype html><p>".len();
            let mut origins = vec![Origin {
                node: 7,
                output: (0, 1),
                source: (x, x + 1),
                kind: OriginKind::Literal,
            }];
            let errors = if feff {
                origins.insert(
                    0,
                    Origin {
                        node: 4,
                        output: (0, 1),
                        source: (0, 1),
                        kind: OriginKind::Literal,
                    },
                );
                vec![
                    Error {
                        kind: DiagnosticKind::Prologue(PrologueCondition::MissingDoctype),
                        context: DiagnosticContext::Prologue(PrologueMode::Initial),
                        role: DiagnosticLocationRole::TokenEnvelope,
                        token: Some(DiagnosticToken::Text),
                        anchor: "the-initial-insertion-mode",
                        branch: "anything-else/missing-doctype",
                        source: (0, 1),
                    },
                    Error {
                        kind: DiagnosticKind::Tree(HeadCondition::UnexpectedDoctype),
                        context: DiagnosticContext::Tree(InsertionMode::InBody),
                        role: DiagnosticLocationRole::TokenEnvelope,
                        token: Some(DiagnosticToken::Doctype),
                        anchor: "parsing-main-inbody",
                        branch: "doctype/ignore",
                        source: (1 + pi, 1 + pi + "<!doctype html>".len()),
                    },
                ]
            } else {
                Vec::new()
            };
            let l = Literal {
                label: if feff {
                    "Initial-FEFF-PI"
                } else {
                    "Initial-PI"
                },
                raw: units(raw),
                expected,
                origins,
                errors,
                attributes: Vec::new(),
            };
            assert_eq!(repeat(&l, true), repeat(&l, false));
        }
    }

    #[test]
    fn attribute_lender_qualifies_actual_owner_slot_variant_and_foreign_refusal() {
        use crate::construction::parser_session::{ParserArenaSession, RootKind};
        let mut expected = document();
        let attrs = ["x", "y"]
            .into_iter()
            .map(|name| Attr {
                name: Name {
                    namespace: None,
                    prefix: None,
                    local: units(name),
                },
                value: units("first"),
            })
            .collect();
        append(&mut expected, 4, element("p", attrs));
        let l = literal(
            "attribute-lender",
            units("<p x='first' y='first'></p>"),
            expected,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let request = HtmlDocumentParseRequest::offline(source(&l, false), limits(&l, l.raw.len()));
        let cancel = request.cancellation().clone();
        let mut initialization = OrdinaryInitializationOwner::new(request);
        let mut attempt = initialization.begin().unwrap();
        let mut driver = Some(attempt.take_driver().unwrap());
        while driver.as_mut().unwrap().step().unwrap() != HeadStep::Complete {}
        let mut completed = StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
        completed.prepare_native_associations().unwrap();
        let (body, element) = {
            let mut view = completed.native_view().unwrap();
            let root = view.root().unwrap();
            let html = view.child_at(&root, 1).unwrap().unwrap();
            let body = view.child_at(&html, 1).unwrap().unwrap();
            let element = view.child_at(&body, 0).unwrap().unwrap();
            assert_eq!(view.attribute_count(&element).unwrap(), 2);
            (body, element)
        };
        let mut foreign_control =
            ParseControl::new(limits(&l, l.raw.len()), crate::HtmlParseCancellation::new());
        let mut foreign_session = ParserArenaSession::new(
            &mut foreign_control,
            RootKind::Document(HtmlDocumentMode::NoQuirks),
        )
        .unwrap();
        let foreign_node = foreign_session.root(&mut foreign_control).unwrap();
        let mut checked = [0; 2];
        {
            let mut rows = completed.native_associations().unwrap();
            for i in 0..rows.len() {
                let (slot, field) = match rows.row(i).unwrap().unwrap().0.destination {
                    NativeDestination::Field(StringLocation::AttributeName(_, slot)) => {
                        (slot, NativeAttributeField::Name)
                    }
                    NativeDestination::Field(StringLocation::AttributeValue(_, slot)) => {
                        (slot, NativeAttributeField::Value)
                    }
                    _ => continue,
                };
                let opposite = if field == NativeAttributeField::Name {
                    NativeAttributeField::Value
                } else {
                    NativeAttributeField::Name
                };
                assert!(rows.attribute_for(i, &element, slot, field).unwrap());
                assert!(!rows.attribute_for(i, &body, slot, field).unwrap());
                assert!(!rows.attribute_for(i, &element, 1 - slot, field).unwrap());
                assert!(!rows.attribute_for(i, &element, 2, field).unwrap());
                assert!(!rows.attribute_for(i, &element, slot, opposite).unwrap());
                assert!(matches!(
                    rows.attribute_for(i, &foreign_node, slot, field),
                    Err(SessionError::ForeignNode)
                ));
                checked[usize::from(field == NativeAttributeField::Value)] += 1;
            }
            assert_eq!(checked, [2, 10]); // two one-unit names and two literal "first" values
            cancel.cancel();
            assert!(matches!(
                rows.attribute_for(0, &element, 0, NativeAttributeField::Name),
                Err(SessionError::Stop(ParseStop::Cancelled))
            ));
        }
        let first = completed.at_stop().unwrap().clone();
        drop(body);
        drop(element);
        completed.cleanup_failure().unwrap();
        zero_storage(completed.after_cleanup().unwrap());
        assert_eq!(completed.at_stop(), Some(&first));
        completed.release().unwrap();
        foreign_session.release(&mut foreign_control).unwrap();
        super::super::tests::assert_released(&foreign_control);
    }
    fn zero_storage(usage: &ParseUsage) {
        for resource in [
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::TokenAttributes,
            HtmlParseResource::AuxiliaryEntries,
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::SourceMapEntries,
            HtmlParseResource::SourceMapPayloadUnits,
            HtmlParseResource::Diagnostics,
            HtmlParseResource::DiagnosticPayloadUnits,
            HtmlParseResource::ContextEffects,
            HtmlParseResource::OpenElements,
            HtmlParseResource::ManagedCapacityBytes,
            HtmlParseResource::PeakTransientBytes,
        ] {
            assert_eq!(usage.resource(resource).current, 0, "{resource:?}");
        }
    }
    #[test]
    fn original_request_pre_cancel_retains_unadmitted_source_and_canonical_snapshot() {
        use crate::parser::tree_builder::StandaloneError;
        let l = plain(1);
        let original = source(&l, false);
        let identity = original.identity().clone();
        let request = HtmlDocumentParseRequest::offline(original, limits(&l, l.raw.len()));
        request.cancellation().cancel();
        let mut owner = OrdinaryInitializationOwner::new(request);
        {
            let mut attempt = owner.begin().unwrap();
            assert!(matches!(
                attempt.take_driver(),
                Err(StandaloneError::Stop(ParseStop::Cancelled))
            ));
            let first = attempt.at_stop().unwrap().clone();
            assert_eq!(first.resource(HtmlParseResource::SourceUnits).current, 0);
            assert_eq!(first.resource(HtmlParseResource::WorkCredits).current, 0);
            attempt.retain_failure().unwrap();
        }
        assert_eq!(owner.source().unwrap().identity(), &identity);
        assert_eq!(
            owner.source().unwrap().as_utf16().unwrap().code_units(),
            l.raw
        );
        assert_eq!(owner.source_admitted(), Some(false));
        let first = owner.at_stop().unwrap().clone();
        assert_eq!(owner.after_cleanup(), Some(&first));
        assert!(owner.diagnostics().is_none());
        owner.release_failure().unwrap();
        zero_storage(owner.retained_usage().unwrap());
        assert_eq!(owner.at_stop(), Some(&first));
    }
    #[test]
    fn between_real_driver_steps_cancel_keeps_duplicate_prefix_source_and_first_stop() {
        let l = attributes(1);
        let original = source(&l, true);
        let identity = original.identity().clone();
        let request = HtmlDocumentParseRequest::offline(original, limits(&l, l.raw.len()));
        let cancel = request.cancellation().clone();
        let mut owner = OrdinaryInitializationOwner::new(request);
        {
            let mut attempt = owner.begin().unwrap();
            let mut driver = Some(attempt.take_driver().unwrap());
            // Stop only after the real retained duplicate diagnostic is committed
            // by an actual step; no poll hook/callback or private mutation.
            let mut reached = false;
            for _ in 0..l.raw.len() + 16 {
                assert_eq!(driver.as_mut().unwrap().step().unwrap(), HeadStep::Consumed);
                if let Some((_, session, _)) = driver.as_mut().unwrap().standalone_parts()
                    && session.retained_evidence().unwrap().diagnostics().len() == 1
                {
                    reached = true;
                    break;
                }
            }
            assert!(reached);
            cancel.cancel();
            assert!(driver.as_mut().unwrap().step().is_err());
            let c = &driver.as_ref().unwrap().control;
            assert_eq!(c.stop(), Some(ParseStop::Cancelled));
            let first = c.first_stop_usage().unwrap().clone();
            assert_eq!(
                first.resource(HtmlParseResource::SourceUnits).current,
                l.raw.len() as u64
            );
            let mut failed = StandaloneDocumentDriver::take_failed(&mut driver).unwrap();
            assert!(driver.is_none());
            assert_eq!(failed.at_stop(), &first);
            assert_eq!(failed.source().identity(), &identity);
            assert_eq!(failed.stop(), ParseStop::Cancelled);
            let diagnostics = failed.diagnostics().unwrap();
            assert_eq!(diagnostics.len(), 1);
            let (record, location) = diagnostics.retained_record(0).unwrap();
            assert_eq!(record.kind, l.errors[0].kind);
            assert_eq!(record.context, l.errors[0].context);
            assert_eq!(location.role, l.errors[0].role);
            assert_eq!(location.token, l.errors[0].token);
            check_range(failed.source(), &location.source, l.errors[0].source);
            failed.release().unwrap();
            zero_storage(failed.retained_usage());
            assert_eq!(failed.at_stop(), &first);
            assert_eq!(failed.stop(), ParseStop::Cancelled);
        }
        assert_eq!(owner.take_request().unwrap().source().identity(), &identity);
    }
    #[test]
    fn final_report_exact_and_one_short_admission_preserve_original_complete_owner() {
        let l = attributes(1);
        let SealObservation {
            owner, positions, ..
        } = sealed(&l, true, false, None);
        let mut measured = Some(owner);
        let report = ImmutableParseReport::take_document(&mut measured).unwrap();
        evidence(&l, true, &report, &positions);
        assert_eq!(project(report.document().unwrap()).unwrap(), l.expected);
        let exact = report
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        drop(report);
        // Independent boundary: the final operation admits five record writes
        // together after fixed compiled-layout copying. A measured completed run
        // supplies its cumulative prefix, not the family admission envelope.
        for work in [exact - 1, exact] {
            let SealObservation {
                owner, positions, ..
            } = sealed(&l, true, false, Some(work));
            let mut input = Some(owner);
            let identity = input.as_ref().unwrap().source().identity().clone();
            let result = ImmutableParseReport::take_document(&mut input);
            if work == exact {
                let report = result.unwrap();
                assert!(input.is_none());
                assert_eq!(
                    report
                        .usage()
                        .resource(HtmlParseResource::WorkCredits)
                        .current,
                    exact
                );
                evidence(&l, true, &report, &positions);
                assert_eq!(report.source().identity(), &identity);
            } else {
                assert!(
                    matches!(result,Err(SessionError::Stop(ParseStop::LimitExceeded {
                    resource:HtmlParseResource::WorkCredits,current,requested_delta:5,..})) if current==exact-5)
                );
                let kept = input.as_mut().unwrap();
                assert_eq!(kept.source().identity(), &identity);
                assert_eq!(project(kept.document().unwrap()).unwrap(), l.expected);
                assert_eq!(kept.evidence().unwrap().diagnostics().len(), 1);
                let first = kept.parts.control.first_stop_usage().unwrap().clone();
                let stop = kept.parts.control.stop();
                kept.parts.cleanup_failure().unwrap();
                assert_eq!(kept.parts.control.first_stop_usage(), Some(&first));
                assert!(kept.parts.arena.is_none());
                assert_eq!(kept.parts.diagnostics().unwrap().len(), 1);
                kept.parts.release().unwrap();
                zero_storage(kept.retained_usage());
                assert_eq!(kept.parts.control.stop(), stop);
                assert_eq!(kept.parts.control.first_stop_usage(), Some(&first));
            }
        }
    }

    #[test]
    fn completed_reference_lineage_uses_contributed_extents_not_atom_times_history() {
        use crate::construction::parser_session::provenance::{ConstructionOperation, GroupBody};
        // Five actual Text insertions per literal repetition, but six exact
        // direct origin atoms: the last entity has two output atoms in one leaf.
        // Data lineage has one qualified contribution per insertion. Each of
        // the subsequent insertions has one explicit semantic merge trigger.
        // Token predecessor links remain fixed history, not field coverage.
        let mut observations = Vec::new();
        for n in [1usize, 8, 32] {
            let literal = references(n, true);
            let SealObservation {
                mut owner,
                positions,
                phases,
            } = sealed(&literal, false, true, None);
            assert_eq!(owner.source().as_utf16().unwrap().code_units(), literal.raw);
            assert_eq!(
                owner
                    .retained_usage()
                    .resource(HtmlParseResource::SourceUnits)
                    .current,
                literal.raw.len() as u64
            );
            assert_eq!(owner.document().unwrap().mode(), HtmlDocumentMode::NoQuirks);
            let evidence = owner.evidence().unwrap();
            assert_eq!(evidence.diagnostics().len(), 0);
            let view = evidence.associations().unwrap();
            let mut origins = Vec::new();
            let mut lineage = Vec::new();
            let mut root = None;
            let mut anchors = 0;
            let mut cause_rows = Vec::new();
            for i in 0..view.len() {
                let row = view.row(i).unwrap();
                if row.relation == AssociationRelation::Cause {
                    assert!(row.destination == NativeDestination::Cause(row.cause));
                    assert!(row.group.is_none());
                    assert_eq!((row.start, row.end), (0, 0));
                    cause_rows.push(row.cause);
                }
                if row.destination == NativeDestination::LexicalEnd {
                    anchors += 1;
                    let range = row.source.as_ref().unwrap();
                    check_range(
                        owner.source(),
                        range,
                        (literal.raw.len(), literal.raw.len()),
                    );
                }
                let NativeDestination::Field(StringLocation::Text(node)) = row.destination else {
                    continue;
                };
                // Actual Node-row resolution performed by the same-owner
                // completed lender, not a session-index == tree-ordinal guess.
                assert!(positions.contains(&(false, node, 6)));
                let range = row.source.as_ref().unwrap();
                check_range(owner.source(), range, (range.start(), range.end()));
                if row.relation == AssociationRelation::Output {
                    let group = row.group.unwrap();
                    assert!(root.is_none_or(|previous| previous == group));
                    root = Some(group);
                    origins.push(Origin {
                        node: 6,
                        output: (row.start, row.end),
                        source: (range.start(), range.end()),
                        kind: row.kind.unwrap(),
                    });
                } else {
                    lineage.push((
                        row.relation,
                        row.start,
                        row.end,
                        (range.start(), range.end()),
                        row.group,
                        row.cause,
                    ));
                }
            }
            assert_eq!(origins, literal.origins);
            assert_eq!(origins.len(), 6 * n);
            assert_eq!(anchors, 1);
            // Every real fixed cause survives once, including source-less
            // merge/retirement and contextual Token predecessor links.
            let mut causes = 0;
            while let Some(cause) = view.cause(causes) {
                if let Some(source) = &cause.source {
                    check_range(owner.source(), source, (source.start(), source.end()));
                }
                if cause.operation == ConstructionOperation::Token {
                    assert!(cause.trigger.is_some());
                }
                causes += 1;
            }
            assert_eq!(cause_rows, (0..causes).collect::<Vec<_>>());
            let mut leaves = Vec::new();
            let mut merged = 0;
            let mut index = root.unwrap();
            loop {
                let group = view.group(index).unwrap();
                let cause = view.cause(group.cause).unwrap();
                match group.body {
                    GroupBody::Concat {
                        original,
                        incoming,
                        offset,
                    } => {
                        assert_eq!(cause.operation, ConstructionOperation::Merge);
                        assert!(cause.source.is_none());
                        assert_eq!(
                            (cause.original, cause.incoming),
                            (Some(original), Some(incoming))
                        );
                        let left = view.group(original).unwrap();
                        let right = view.group(incoming).unwrap();
                        assert_eq!(left.units, offset);
                        assert_eq!(offset + right.units, group.units);
                        assert!(original < index && incoming < index);
                        assert!(matches!(right.body, GroupBody::Leaf { .. }));
                        let leaf_cause = view.cause(right.cause).unwrap();
                        assert_eq!(leaf_cause.operation, ConstructionOperation::Token);
                        assert_eq!(cause.trigger, Some(right.cause));
                        let range = leaf_cause.source.as_ref().unwrap();
                        leaves.push((
                            incoming,
                            right.cause,
                            offset,
                            offset + right.units,
                            (range.start(), range.end()),
                        ));
                        let retired = view.cause(left.retirement.unwrap()).unwrap();
                        assert_eq!(retired.operation, ConstructionOperation::Retirement);
                        assert_eq!(retired.original, Some(original));
                        merged += 1;
                        index = original;
                    }
                    GroupBody::Leaf { .. } => {
                        assert_eq!(cause.operation, ConstructionOperation::Token);
                        let range = cause.source.as_ref().unwrap();
                        leaves.push((
                            index,
                            group.cause,
                            0,
                            group.units,
                            (range.start(), range.end()),
                        ));
                        break;
                    }
                    GroupBody::Alias { .. } => panic!("literal append has no semantic copy/reset"),
                }
            }
            leaves.reverse();
            assert_eq!(leaves.len(), 5 * n);
            assert_eq!(merged, 5 * n - 1);
            let expected_leaf_extents: Vec<_> = (0..n)
                .flat_map(|i| {
                    let source = PREFIX.len() + 3 + 32 * i;
                    let output = 7 * i;
                    [
                        ((output, output + 1), (source, source + 1)),
                        ((output + 1, output + 2), (source + 1, source + 3)),
                        ((output + 2, output + 3), (source + 3, source + 8)),
                        ((output + 3, output + 5), (source + 8, source + 17)),
                        ((output + 5, output + 7), (source + 17, source + 32)),
                    ]
                })
                .collect();
            assert_eq!(
                leaves
                    .iter()
                    .map(|(_, _, a, b, source)| ((*a, *b), *source))
                    .collect::<Vec<_>>(),
                expected_leaf_extents
            );
            let actual_usage = owner.retained_usage().clone();
            let association_phase = phases
                .iter()
                .find(|(name, _, _)| *name == "associations")
                .unwrap();
            let complete_phase = phases
                .iter()
                .find(|(name, _, _)| *name == "complete")
                .unwrap();
            eprintln!(
                "reference spatial oracle N={n} direct={} leaves={} merges={} field-lineage={} complete-rows={} association-delta={:?} capacity={}/{}",
                origins.len(),
                leaves.len(),
                merged,
                lineage.len(),
                view.len(),
                WORK.map(
                    |category| association_phase.1.work(category) - complete_phase.1.work(category)
                ),
                actual_usage.actual_capacity_bytes,
                actual_usage.actual_capacity_high_water
            );
            // No borrowed handle/backing escapes. Release this same observed
            // owner before asserting the intentionally missing applicability.
            owner.release().unwrap();
            super::super::tests::assert_released(&owner.parts.control);
            eprintln!("reference spatial oracle N={n} actual owning cleanup zero");
            observations.push((n, leaves, lineage));
        }
        for (n, leaves, lineage) in observations {
            let expected_data: Vec<_> = leaves
                .iter()
                .enumerate()
                .map(|(i, (group, cause, a, b, source))| {
                    (
                        if i == 0 {
                            AssociationRelation::Original
                        } else {
                            AssociationRelation::Incoming
                        },
                        *a,
                        *b,
                        *source,
                        Some(*group),
                        *cause,
                    )
                })
                .collect();
            let expected_triggers: Vec<_> = leaves
                .iter()
                .skip(1)
                .map(|(_, cause, a, b, source)| {
                    (AssociationRelation::Trigger, *a, *b, *source, None, *cause)
                })
                .collect();
            assert_eq!(
                lineage.len(),
                10 * n - 1,
                "five contributed leaf relations plus all actual merge triggers, not atom × ancestor closure"
            );
            for expected in expected_data.iter().chain(&expected_triggers) {
                assert_eq!(
                    lineage.iter().filter(|row| *row == expected).count(),
                    1,
                    "each qualified source/cause/role/group applies at its literal contributed interval"
                );
            }
        }
    }
}
