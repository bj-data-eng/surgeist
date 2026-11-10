//! Bounded native authoring recipes; no parser or live-tree substitute.
mod codec;
mod generate;
mod observe;
mod seeds;

use std::mem::size_of;
use surgeist_fuzz::{
    generation::{Budget, BudgetCharge},
    registry::FuzzManifest,
    *,
};
use surgeist_html::{
    attributes::HtmlHidden,
    construction::{
        ForeignHtmlResult, HtmlAuthoredProcessingInstruction, HtmlBuilderNode,
        HtmlConstructionCause, HtmlConstructionError, HtmlConstructionLimits,
        HtmlConstructionResource, HtmlConstructionUsage, HtmlDoctype, HtmlDocument,
        HtmlDocumentBuilder, HtmlDocumentMode, HtmlFragment, HtmlFragmentBuilder,
        HtmlFragmentOwner, HtmlNode, HtmlNodeId, HtmlNodeKind, HtmlProcessingInstructionError,
        HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
    },
    values::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName},
    vocabulary::KnownHtmlElement,
};

pub(super) const CAPS: [usize; 5] = [64, 128, 128, 8192, 8];
pub(super) const MAX_BYTES: usize = 65536;
// Header 47; at most 12 framing bytes per operation and per attribute,
// plus two bytes for each admitted unit. This also covers keyword tags.
pub(super) const ENCODED_CAPACITY: usize = 47 + 64 * 12 + 128 * 12 + 8192 * 2;
const WRAPPER_ALLOWANCE_PER_NODE: usize = 576;
pub(super) fn invalid(detail: &'static str) -> FuzzError {
    FuzzError::new(FuzzErrorKind::InvalidCase, "HTML native recipe", detail)
}
pub(super) fn resource(detail: &'static str) -> FuzzError {
    FuzzError::new(FuzzErrorKind::Resource, "HTML native recipe", detail)
}
pub(super) fn reserve<T>(v: &mut Vec<T>, n: usize) -> Result<()> {
    v.try_reserve_exact(n)
        .map_err(|_| resource("fixture allocation"))?;
    if v.capacity() > n {
        return Err(resource("fixture capacity exceeded precredit"));
    }
    Ok(())
}

/// Complete owned replay input, constructed only by checked recipes/decoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeCase {
    root: u8,
    literal: Option<u8>,
    caps: [usize; 5],
    ops: Vec<Op>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum Name {
    Known(u8),
    Raw {
        uri: Vec<u16>,
        prefix: Option<Vec<u16>>,
        local: Vec<u16>,
    },
}
impl Name {
    fn values(&self) -> (&[u16], Option<&[u16]>, &[u16]) {
        match self {
            Self::Known(n) => (
                seeds::HTML,
                None,
                match n {
                    0 => &[100, 105, 118],
                    1 => &[97],
                    2 => &[116, 101, 109, 112, 108, 97, 116, 101],
                    3 => &[115, 112, 97, 110],
                    _ => unreachable!("checked name"),
                },
            ),
            Self::Raw { uri, prefix, local } => (uri, prefix.as_deref(), local),
        }
    }
    fn native(&self) -> QualifiedName {
        match self {
            Self::Known(n) => [
                KnownHtmlElement::Div,
                KnownHtmlElement::A,
                KnownHtmlElement::Template,
                KnownHtmlElement::Span,
            ][usize::from(*n)]
            .qualified_name(),
            Self::Raw { uri, prefix, local } => QualifiedName::new(
                Namespace::from_uri(DocumentString::from_utf16(uri.clone())),
                prefix
                    .as_ref()
                    .map(|p| DocumentString::from_utf16(p.clone())),
                DocumentString::from_utf16(local.clone()),
            ),
        }
    }
    fn template(&self) -> bool {
        let (u, _, l) = self.values();
        u == seeds::HTML && l == [116, 101, 109, 112, 108, 97, 116, 101]
    }
    fn units(&self) -> usize {
        let (u, p, l) = self.values();
        u.len() + p.map_or(0, <[u16]>::len) + l.len()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum Attr {
    Raw(Name, Vec<u16>),
    Hidden(u8),
}
struct AttributeParts<'a> {
    uri: &'a [u16],
    prefix: Option<&'a [u16]>,
    local: &'a [u16],
    value: &'a [u16],
}
impl Attr {
    fn values(&self) -> AttributeParts<'_> {
        match self {
            Self::Raw(n, v) => {
                let (u, p, l) = n.values();
                AttributeParts {
                    uri: u,
                    prefix: p,
                    local: l,
                    value: v,
                }
            }
            Self::Hidden(k) => AttributeParts {
                uri: &[],
                prefix: None,
                local: &[104, 105, 100, 100, 101, 110],
                value: if *k == 0 {
                    &[104, 105, 100, 100, 101, 110]
                } else {
                    &[117, 110, 116, 105, 108, 45, 102, 111, 117, 110, 100]
                },
            },
        }
    }
    fn native(&self) -> Attribute {
        match self {
            Self::Raw(n, v) => Attribute::new(n.native(), DocumentString::from_utf16(v.clone())),
            Self::Hidden(k) => if *k == 0 {
                HtmlHidden::Hidden
            } else {
                HtmlHidden::UntilFound
            }
            .into_attribute(),
        }
    }
    fn units(&self) -> usize {
        let p = self.values();
        p.uri.len() + p.prefix.map_or(0, <[u16]>::len) + p.local.len() + p.value.len()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum Op {
    Element(u8, Name, Vec<Attr>),
    Text(u8, Vec<u16>),
    Comment(u8, Vec<u16>),
    Pi(u8, Vec<u16>, Vec<u16>),
    Doctype(u8, Vec<u16>, Option<Vec<u16>>, Option<Vec<u16>>),
    Contents(u8),
    Shadow(u8, u8),
}
impl Op {
    fn parent(&self) -> u8 {
        match self {
            Self::Element(p, ..)
            | Self::Text(p, ..)
            | Self::Comment(p, ..)
            | Self::Pi(p, ..)
            | Self::Doctype(p, ..)
            | Self::Contents(p)
            | Self::Shadow(p, _) => *p,
        }
    }
    /// Renumbers a reference to a handle slot above a removed command's slot.
    fn renumber_after(&mut self, removed: u8) {
        let parent = match self {
            Self::Element(p, ..)
            | Self::Text(p, ..)
            | Self::Comment(p, ..)
            | Self::Pi(p, ..)
            | Self::Doctype(p, ..)
            | Self::Contents(p)
            | Self::Shadow(p, _) => p,
        };
        if *parent != 255 && *parent > removed {
            *parent -= 1;
        }
    }
    fn units(&self) -> usize {
        match self {
            Self::Element(_, n, a) => n.units() + a.iter().map(Attr::units).sum::<usize>(),
            Self::Text(_, v) | Self::Comment(_, v) => v.len(),
            Self::Pi(_, t, d) => t.len() + d.len(),
            Self::Doctype(_, n, p, s) => {
                n.len() + p.as_ref().map_or(0, Vec::len) + s.as_ref().map_or(0, Vec::len)
            }
            _ => 0,
        }
    }
}
impl NativeCase {
    /// One exact text append, without a literal-oracle marker.
    pub fn text_fragment(units: Vec<u16>) -> Result<Self> {
        if units.len() > 512 {
            return Err(invalid("individual payload ceiling"));
        }
        if units.capacity() > 512 {
            return Err(resource("retained caller payload capacity"));
        }
        Ok(Self {
            root: 0,
            literal: None,
            caps: CAPS,
            ops: vec![Op::Text(0, units)],
        })
    }
    pub fn with_caps(mut self, caps: [usize; 5]) -> Result<Self> {
        if self.literal.is_some() || caps.iter().zip(CAPS).any(|(a, b)| *a > b) {
            return Err(invalid("literal or cap reconfiguration"));
        }
        self.caps = caps;
        Ok(self)
    }
    pub fn caps(&self) -> [usize; 5] {
        self.caps
    }
    pub fn operation_count(&self) -> usize {
        self.ops.len()
    }
}

/// HTML-owned concrete shared-framework consumer over public native builders.
pub struct NativeTarget {
    descriptor: TargetDescriptor,
}
impl Default for NativeTarget {
    fn default() -> Self {
        Self::new()
    }
}
impl NativeTarget {
    pub fn new() -> Self {
        let m: FuzzManifest =
            toml::from_str(include_str!("../../fuzz.toml")).expect("checked descriptor");
        Self {
            descriptor: m
                .targets
                .into_iter()
                .find(|d| d.id().as_str() == "surgeist-html::native-authoring")
                .expect("registered native target"),
        }
    }
    // Two owned recipes (decode/literal comparison or reduction), two encoded
    // buffers (engine canonical plus validation), four unit payload copies
    // (original recipe/reduced candidate/prepared Vec/native backing during an
    // evaluator callback), actual observation slots and a conservative
    // 576-byte wrapper allowance per possible node. Opaque native arena/allocator
    // overhead is not measured here; product limits retain their logical meaning.
    fn credit(&self, nodes: usize, units: usize, ops: usize) -> Result<()> {
        let bytes = 2 * 64 * size_of::<Op>()
            + 2 * 128 * size_of::<Attr>()
            + 64 * size_of::<observe::Expected>()
            + 65 * size_of::<Option<HtmlBuilderNode>>()
            + 65 * size_of::<Option<usize>>()
            + 64 * size_of::<Option<HtmlNodeId>>()
            + 64 * 64 * size_of::<usize>()
            + 8192 * 2 * 4
            + 2 * ENCODED_CAPACITY
            + WRAPPER_ALLOWANCE_PER_NODE * 64;
        Budget::new(self.descriptor.limits()).credit(BudgetCharge {
            bytes: u64::try_from(bytes).map_err(|_| resource("byte credit representation"))?,
            nodes: nodes as u64,
            edges: nodes.saturating_sub(2) as u64,
            depth: nodes.saturating_sub(1).min(8) as u64,
            text_units: units as u64,
            operations: (ops + 32) as u64,
        })
    }
    pub fn literal(&self, id: u8) -> Result<NativeCase> {
        self.credit(64, 8192, 64)?;
        seeds::case(id)
    }
    fn failure(&self, key: &str, detail: impl Into<String>) -> Result<CaseOutcome> {
        let property = match key {
            "rejected-append-mutated"
            | "native-refusal-limit"
            | "native-rejection-mismatch"
            | "native-missing-rejection" => 2,
            "literal-rejection" | "missing-literal-rejection" => 4,
            "native-owner" | "native-child-count" | "native-template" | "native-shadow" => 1,
            "native-alias"
            | "template-alias"
            | "cloned-identity"
            | "foreign-result-identity"
            | "foreign-native-node" => 3,
            _ => 0,
        };
        CaseOutcome::failure(
            FailureKey::new(
                self.descriptor.properties()[property].clone(),
                FailureClass::PropertyViolation,
                Some(key.into()),
            )?,
            detail,
        )
    }
    fn properties(&self, literal: bool) -> Vec<PropertyId> {
        let all = self.descriptor.properties();
        let mut checked = all[..if literal { 5 } else { 4 }].to_vec();
        checked.push(all[5].clone());
        checked
    }
    /// A captured-import violation names the sixth property.
    fn capture_failure(&self, key: &str, detail: impl Into<String>) -> Result<CaseOutcome> {
        CaseOutcome::failure(
            FailureKey::new(
                self.descriptor.properties()[5].clone(),
                FailureClass::PropertyViolation,
                Some(key.into()),
            )?,
            detail,
        )
    }
}
impl CaseTarget for NativeTarget {
    type Case = NativeCase;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<NativeCase> {
        codec::decode(self, input)
    }
    fn encode(&self, c: &NativeCase) -> Result<Vec<u8>> {
        codec::encode(self, c)
    }
    fn evaluate(&self, c: &NativeCase) -> Result<CaseOutcome> {
        codec::validate(self, c)?;
        observe::evaluate(self, c)
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<NativeCase>> {
        if input.len() > MAX_BYTES {
            return Ok(Generation::Rejected(GenerationRejection::new(
                "native entropy byte ceiling",
            )?));
        }
        let choice = input.first().copied().unwrap_or(0);
        if choice % 4 == 0 {
            return self.literal((choice / 4) % 8).map(Generation::Generated);
        }
        self.credit(64, 8192, 64)?;
        if choice % 4 == 1 {
            // One exact text append, preserving every UTF16 unit of the entropy.
            let raw = input.get(1..).unwrap_or_default();
            if raw.len().div_ceil(2) > 512 {
                return Ok(Generation::Rejected(GenerationRejection::new(
                    "native field unit ceiling",
                )?));
            }
            let mut units = Vec::new();
            reserve(&mut units, raw.len().div_ceil(2))?;
            for p in raw.chunks(2) {
                units.push(u16::from_le_bytes([p[0], p.get(1).copied().unwrap_or(0)]));
            }
            return Ok(Generation::Generated(NativeCase {
                root: 0,
                literal: None,
                caps: CAPS,
                ops: vec![Op::Text(0, units)],
            }));
        }
        let case = generate::synthesize(input);
        match codec::validate(self, &case) {
            Ok(()) => Ok(Generation::Generated(case)),
            Err(e) if e.kind() == FuzzErrorKind::InvalidCase => Ok(Generation::Rejected(
                GenerationRejection::new("native recipe exceeds the consumer envelope")?,
            )),
            Err(e) => Err(e),
        }
    }
    fn reduction_candidates(
        &self,
        c: &NativeCase,
        emit: &mut dyn FnMut(NativeCase) -> bool,
    ) -> Result<()> {
        if c.literal.is_some() {
            return Ok(());
        }
        self.credit(64, 8192, 64)?;
        let mut proposals = generate::candidates(c);
        if let [Op::Text(p, v)] = c.ops.as_slice() {
            for n in [0, v.len() / 2, v.len().saturating_sub(1)] {
                if n < v.len() {
                    proposals.push(NativeCase {
                        root: c.root,
                        literal: None,
                        caps: c.caps,
                        ops: vec![Op::Text(*p, v[..n].to_vec())],
                    });
                }
            }
        }
        for reduced in proposals {
            codec::validate(self, &reduced)?;
            if !emit(reduced) {
                return Ok(());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compiled_credit_fits_selected_policy_before_any_case_allocation() {
        NativeTarget::new().credit(64, 8192, 64).unwrap();
        assert_eq!(ENCODED_CAPACITY, 18735);
        println!(
            "native layouts: Op={} Attr={} Expected={} BuilderHandle={} NodeId={} optional-index={} encoded={} wrapper-per-node={}",
            size_of::<Op>(),
            size_of::<Attr>(),
            size_of::<observe::Expected>(),
            size_of::<Option<HtmlBuilderNode>>(),
            size_of::<Option<HtmlNodeId>>(),
            size_of::<Option<usize>>(),
            ENCODED_CAPACITY,
            WRAPPER_ALLOWANCE_PER_NODE
        );
    }
}
