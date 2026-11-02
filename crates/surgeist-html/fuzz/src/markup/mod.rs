//! Balanced-markup recipes with an independent tree oracle, over the public
//! document and fragment parse front doors. Free-form token soup shares the
//! same parse contracts without a tree oracle.
mod codec;
mod compare;
mod generate;
mod model;
mod observe;
mod seeds;
#[cfg(test)]
mod tests;
mod tree;

use crate::captured::Plan;
use crate::{UnicodeSource, error};
use model::*;
use surgeist_fuzz::{
    generation::{Budget, BudgetCharge},
    registry::FuzzManifest,
    *,
};

pub(crate) const MAX_SEGMENTS: usize = 64;
pub(crate) const MAX_SEGMENT_UNITS: usize = 16;
pub(crate) const MAX_SOURCE_UNITS: usize = 4096;
/// Closed token vocabulary of the free-form lane: tags that exercise table,
/// select, foreign, template, raw-text, formatting and malformed-markup
/// recovery, plus tokenizer-state openers and closers.
pub(crate) const SOUP: &[&str] = &[
    "<p>",
    "</p>",
    "<div>",
    "</div>",
    "<b>",
    "</b>",
    "<i>",
    "</i>",
    "<a href=x>",
    "</a>",
    "<table>",
    "</table>",
    "<tr>",
    "<td>",
    "<th>",
    "<tbody>",
    "<caption>",
    "<colgroup><col>",
    "<select>",
    "<option>",
    "</select>",
    "<svg>",
    "</svg>",
    "<math>",
    "</math>",
    "<mi>",
    "<mtext>",
    "<annotation-xml encoding=text/html>",
    "<foreignObject>",
    "</foreignObject>",
    "<g>",
    "<template>",
    "</template>",
    "<template shadowrootmode=open>",
    "<!--",
    "-->",
    "<!x>",
    "<![CDATA[",
    "]]>",
    "&amp;",
    "&#x",
    "&nosuchref;",
    "&",
    "<",
    ">",
    "</",
    "<br/>",
    "<form>",
    "<input type=hidden>",
    "<frameset>",
    "<noscript>",
    "<textarea>",
    "</textarea>",
    "<title>",
    "<style>",
    "<plaintext>",
    "<button>",
    "<h1>",
    "<li>",
    "<nobr>",
    "<ruby>",
    "=",
    "\"",
    "'",
    " ",
    "\n",
    "x",
    "<?pi?>",
    "<?pi>",
    "</br>",
    "<img src=x>",
    "<script>",
    "<body>",
    "<html>",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mode {
    Document { doctype: u8, shell: u8 },
    Fragment { context: u8, shadow: bool },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Segment {
    Token(u8),
    Units(Vec<u16>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Body {
    Exact(Vec<Item>),
    Soup(Vec<Segment>),
}

/// A complete, checked replay input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkupCase {
    pub(crate) mode: Mode,
    pub(crate) utf16: bool,
    pub(crate) body: Body,
    pub(crate) caps: [u64; 19],
    pub(crate) cancelled: bool,
    /// External facts and the injected fault of a capture-import case.
    pub(crate) plan: Option<Plan>,
}
pub(crate) use MarkupCase as Case;

impl Case {
    pub(crate) fn new(
        mode: Mode,
        utf16: bool,
        body: Body,
        caps: [u64; 19],
        cancelled: bool,
        plan: Option<Plan>,
    ) -> Result<Self> {
        let case = Self {
            mode,
            utf16,
            body,
            caps,
            cancelled,
            plan,
        };
        case.validate()?;
        Ok(case)
    }
    pub fn caps(&self) -> [u64; 19] {
        self.caps
    }
    /// Reconfigures the parser caps; each stays within the documented envelope.
    pub fn with_caps(mut self, caps: [u64; 19]) -> Result<Self> {
        self.caps = caps;
        self.validate()?;
        Ok(self)
    }
    pub(crate) fn shadow_permitted(&self) -> bool {
        matches!(self.mode, Mode::Fragment { shadow: true, .. })
    }
    pub(crate) fn root_class(&self) -> Option<Class> {
        match self.mode {
            Mode::Document { .. } => Some(Class::Flow),
            Mode::Fragment { context, .. } => CONTEXTS.get(usize::from(context))?.class,
        }
    }
    /// The exact source units: the serialized recipe or the token soup. A soup
    /// document keeps the selected prologue and has no shell.
    pub(crate) fn units(&self) -> Vec<u16> {
        match (&self.body, self.mode) {
            (Body::Exact(items), Mode::Document { doctype, shell }) => {
                tree::serialize_document(doctype, shell, items)
            }
            (Body::Exact(items), Mode::Fragment { .. }) => tree::serialize_fragment(items),
            (Body::Soup(segments), mode) => {
                let mut out = Vec::new();
                if let Mode::Document { doctype, .. } = mode
                    && let Some(Some(d)) = DOCTYPES.get(usize::from(doctype))
                {
                    out.extend(d.source.encode_utf16());
                }
                for s in segments {
                    match s {
                        Segment::Token(t) => {
                            out.extend(SOUP[usize::from(*t) % SOUP.len()].encode_utf16());
                        }
                        Segment::Units(u) => out.extend(u),
                    }
                }
                out
            }
        }
    }
    pub(crate) fn source(&self) -> UnicodeSource {
        let units = self.units();
        if self.utf16 {
            UnicodeSource::Utf16(units)
        } else {
            UnicodeSource::Utf8(String::from_utf16_lossy(&units))
        }
    }
    pub(crate) fn validate(&self) -> Result<()> {
        if self.plan.is_some_and(|p| !p.valid()) {
            return Err(error("capture fact tag"));
        }
        if self
            .caps
            .iter()
            .zip(crate::codec::CAPS)
            .any(|(a, b)| *a > b)
        {
            return Err(error("cap envelope"));
        }
        crate::codec::limits(self.caps)?;
        match self.mode {
            Mode::Document { doctype, shell } => {
                if usize::from(doctype) >= DOCTYPES.len() || usize::from(shell) >= SHELLS.len() {
                    return Err(error("document prologue or shell"));
                }
            }
            Mode::Fragment { context, .. } => {
                if usize::from(context) >= CONTEXTS.len() {
                    return Err(error("fragment context"));
                }
            }
        }
        match &self.body {
            Body::Exact(items) => {
                let class = self
                    .root_class()
                    .ok_or_else(|| error("context has no balanced recipe form"))?;
                let mut counts = Counts {
                    items: 0,
                    pieces: 0,
                    units: 0,
                    depth: 0,
                };
                check_items(items, class, false, true, 0, &mut counts).map_err(error)?;
            }
            Body::Soup(segments) => {
                if segments.len() > MAX_SEGMENTS
                    || segments.iter().any(|s| match s {
                        Segment::Token(t) => usize::from(*t) >= SOUP.len(),
                        Segment::Units(u) => u.len() > MAX_SEGMENT_UNITS,
                    })
                {
                    return Err(error("soup segment"));
                }
            }
        }
        let units = self.units();
        if units.len() > MAX_SOURCE_UNITS {
            return Err(error("source unit ceiling"));
        }
        if !self.utf16 && String::from_utf16(&units).is_err() {
            return Err(error("UTF-8 source with an isolated surrogate"));
        }
        Ok(())
    }
}

/// HTML-owned concrete shared-framework consumer for parse markup recipes.
pub struct MarkupTarget {
    pub(crate) descriptor: TargetDescriptor,
    /// Whether this is the parse-then-capture-import target.
    pub(crate) capture: bool,
}
impl MarkupTarget {
    pub fn parse() -> Self {
        Self::by_id("surgeist-html::markup-parse", false)
    }
    /// Parses a recipe, then imports the result into a semantic capture under
    /// supplied external facts and at most one injected fault.
    pub fn capture() -> Self {
        Self::by_id("surgeist-html::capture-import", true)
    }
    fn by_id(id: &str, capture: bool) -> Self {
        let manifest: FuzzManifest =
            toml::from_str(include_str!("../../fuzz.toml")).expect("checked owner descriptor");
        Self {
            descriptor: manifest
                .targets
                .into_iter()
                .find(|d| d.id().as_str() == id)
                .expect("registered target"),
            capture,
        }
    }
    /// One of the handwritten recipes whose canonical files are the seeds.
    pub fn seed(&self, id: u8) -> Result<MarkupCase> {
        seeds::case(id, self.capture)
    }
    pub(crate) fn credit(&self, case: &Case, encoded: usize) -> Result<()> {
        let mut counts = Counts {
            items: 0,
            pieces: 0,
            units: 0,
            depth: 0,
        };
        if let Body::Exact(items) = &case.body {
            let class = case.root_class().unwrap_or(Class::Flow);
            check_items(items, class, false, true, 0, &mut counts).map_err(error)?;
        }
        // Two source copies, the expected tree and observation slots are
        // conservatively charged per unit; node slots per recipe item.
        let units = case.units().len() as u64;
        Budget::new(self.descriptor.limits()).credit(BudgetCharge {
            bytes: 2 * encoded as u64 + units * 8 + counts.items as u64 * 256 + 2048,
            nodes: counts.items as u64 + 4,
            edges: counts.items as u64 + 2,
            depth: counts.depth as u64 + 2,
            text_units: units * 2,
            operations: 8,
        })
    }
}
impl CaseTarget for MarkupTarget {
    type Case = MarkupCase;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<MarkupCase> {
        if input.len() as u64 > self.descriptor.limits().bytes() {
            return Err(error("byte admission"));
        }
        let case = codec::decode(input, self.capture)?;
        self.credit(&case, input.len())?;
        Ok(case)
    }
    fn encode(&self, case: &MarkupCase) -> Result<Vec<u8>> {
        if case.plan.is_some() != self.capture {
            return Err(error("case belongs to the other markup target"));
        }
        let bytes = codec::encode(case)?;
        if bytes.len() as u64 > self.descriptor.limits().bytes() {
            return Err(error("encoded byte limit"));
        }
        self.credit(case, bytes.len())?;
        Ok(bytes)
    }
    fn evaluate(&self, case: &MarkupCase) -> Result<CaseOutcome> {
        case.validate()?;
        if case.plan.is_some() != self.capture {
            return Err(error("case belongs to the other markup target"));
        }
        observe::evaluate_parse(self, case)
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<MarkupCase>> {
        if input.len() as u64 > self.descriptor.limits().bytes() {
            return Err(error("entropy byte admission"));
        }
        match generate::synthesize(input, self.capture) {
            Some(case) => {
                match codec::encode(&case).and_then(|bytes| self.credit(&case, bytes.len())) {
                    Ok(()) => Ok(Generation::Generated(case)),
                    Err(e) if e.kind() == FuzzErrorKind::InvalidCase => Ok(Generation::Rejected(
                        GenerationRejection::new("markup recipe exceeds the consumer envelope")?,
                    )),
                    Err(e) => Err(e),
                }
            }
            None => Ok(Generation::Rejected(GenerationRejection::new(
                "markup recipe exceeds the consumer envelope",
            )?)),
        }
    }
    fn reduction_candidates(
        &self,
        case: &MarkupCase,
        emit: &mut dyn FnMut(MarkupCase) -> bool,
    ) -> Result<()> {
        for candidate in generate::candidates(case) {
            if candidate.validate().is_err() {
                continue;
            }
            if !emit(candidate) {
                break;
            }
        }
        Ok(())
    }
}
