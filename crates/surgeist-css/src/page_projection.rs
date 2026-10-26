//! Immutable selected Page state and borrowed edited rule views.
use crate::cssom_rule_serialization::RuleCssomSource;
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssDeclarationBlockError, CssImportance, CssMarginBox, CssPageDeclaration,
    CssPageDeclarationBlock, CssPageDescriptor, CssPageDescriptorKind, CssPageSelectorList,
    CssRuleSerializationError, CssSerializationError, CssSerializationLimits as Limits,
    CssSpecifiedDeclarationBlock, CssSpecifiedDeclarationEntry,
    CssSpecifiedValueSerializationErrorKind,
};

/// One selected Page declaration. Property terminals retain their actual source
/// occurrence; descriptors retain their original named or raw-value origin.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssSpecifiedPageDeclarationEntry {
    Property(CssSpecifiedDeclarationEntry),
    Descriptor(CssPageDescriptor),
}
/// Exact ordered selected Page state. It never expands a retained shorthand again.
#[derive(Clone, Debug)]
pub struct CssSpecifiedPageDeclarationBlock {
    entries: Vec<CssSpecifiedPageDeclarationEntry>,
    properties: CssSpecifiedDeclarationBlock,
}
/// Why selected Page state or its specified text cannot be completed.
///
/// A property-block or serialization failure is exposed by its accessor and is the
/// error's source; [`Self::is_resource_limit`] groups the resource refusals.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageProjectionError {
    kind: PageProjectionKind,
}
#[derive(Clone, Debug, PartialEq)]
enum PageProjectionKind {
    Properties(CssDeclarationBlockError),
    DuplicateDescriptor { kind: CssPageDescriptorKind },
    Resource(CssSerializationError),
}
impl CssPageProjectionError {
    pub(crate) const fn duplicate(kind: CssPageDescriptorKind) -> Self {
        Self {
            kind: PageProjectionKind::DuplicateDescriptor { kind },
        }
    }
    /// Returns the selected property block's failure.
    #[must_use]
    pub const fn properties(&self) -> Option<&CssDeclarationBlockError> {
        match &self.kind {
            PageProjectionKind::Properties(error) => Some(error),
            _ => None,
        }
    }
    /// Returns the descriptor kind that occurs more than once in the selected state.
    #[must_use]
    pub const fn duplicate_descriptor(&self) -> Option<CssPageDescriptorKind> {
        match self.kind {
            PageProjectionKind::DuplicateDescriptor { kind } => Some(kind),
            _ => None,
        }
    }
    /// Returns the descriptor or punctuation serialization failure.
    #[must_use]
    pub const fn resource(&self) -> Option<&CssSerializationError> {
        match &self.kind {
            PageProjectionKind::Resource(error) => Some(error),
            _ => None,
        }
    }
    /// Returns whether a resource limit refused the projection: a serialization resource
    /// limit, or a property block whose serialization cause is one.
    #[must_use]
    pub fn is_resource_limit(&self) -> bool {
        match &self.kind {
            PageProjectionKind::Resource(error) => error.is_resource_limit(),
            PageProjectionKind::Properties(error) => error.is_resource_limit(),
            PageProjectionKind::DuplicateDescriptor { .. } => false,
        }
    }
}
impl std::fmt::Display for CssPageProjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            PageProjectionKind::Properties(v) => v.fmt(f),
            PageProjectionKind::DuplicateDescriptor { kind } => {
                write!(f, "duplicate selected {} descriptor", kind.as_str())
            }
            PageProjectionKind::Resource(v) => v.fmt(f),
        }
    }
}
impl std::error::Error for CssPageProjectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            PageProjectionKind::Properties(v) => Some(v),
            PageProjectionKind::Resource(v) => Some(v),
            PageProjectionKind::DuplicateDescriptor { .. } => None,
        }
    }
}
impl From<CssDeclarationBlockError> for CssPageProjectionError {
    fn from(v: CssDeclarationBlockError) -> Self {
        Self {
            kind: PageProjectionKind::Properties(v),
        }
    }
}
impl From<CssSerializationError> for CssPageProjectionError {
    fn from(v: CssSerializationError) -> Self {
        Self {
            kind: PageProjectionKind::Resource(v),
        }
    }
}
fn reserve<T>(items: &mut Vec<T>, count: usize) -> Result<(), CssSerializationError> {
    items.try_reserve(count).map_err(|_| {
        CssSerializationError::new(CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
    })
}
impl CssSpecifiedPageDeclarationBlock {
    pub fn try_from_entries(
        entries: &[CssSpecifiedPageDeclarationEntry],
        limits: Limits,
    ) -> Result<Self, CssPageProjectionError> {
        Self::from_entries(entries, &mut SpecifiedRuleWriter::new(limits))
    }
    fn from_entries(
        entries: &[CssSpecifiedPageDeclarationEntry],
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<Self, CssPageProjectionError> {
        writer.node()?;
        let mut properties = Vec::new();
        let mut kinds = Vec::new();
        for entry in entries {
            match entry {
                CssSpecifiedPageDeclarationEntry::Property(v) => {
                    reserve(&mut properties, 1)?;
                    properties.push(v.clone());
                }
                CssSpecifiedPageDeclarationEntry::Descriptor(v) => {
                    writer.node()?;
                    let kind = v.value().kind();
                    if kinds.contains(&kind) {
                        return Err(CssPageProjectionError::duplicate(kind));
                    }
                    reserve(&mut kinds, 1)?;
                    kinds.push(kind);
                    writer.without_output(|w| v.value().append_value(w))?;
                }
            }
        }
        let properties = CssSpecifiedDeclarationBlock::from_page_entries(&properties, writer)?;
        let mut selected = Vec::new();
        reserve(&mut selected, entries.len())?;
        selected.extend_from_slice(entries);
        Ok(Self {
            entries: selected,
            properties,
        })
    }
    #[must_use]
    pub fn entries(&self) -> &[CssSpecifiedPageDeclarationEntry] {
        &self.entries
    }
    pub fn serialize_cssom(&self, limits: Limits) -> Result<String, CssRuleSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer
            .append_selected_page_block(self)
            .map_err(|v| CssRuleSerializationError::new(v, Vec::new(), None))?;
        Ok(writer.css)
    }
}
impl CssPageDeclarationBlock {
    pub fn try_specified(
        &self,
        limits: Limits,
    ) -> Result<CssSpecifiedPageDeclarationBlock, CssPageProjectionError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        let properties =
            CssSpecifiedDeclarationBlock::from_page_sources(self.properties().iter(), &mut writer)?;
        let effective = [
            CssPageDescriptorKind::Size,
            CssPageDescriptorKind::PageOrientation,
            CssPageDescriptorKind::Marks,
            CssPageDescriptorKind::Bleed,
        ]
        .map(|kind| self.effective_descriptor(kind));
        let mut projected = properties.entries().iter().peekable();
        let mut entries = Vec::new();
        let mut ordinal = 0;
        for value in self.occurrences() {
            match value {
                CssPageDeclaration::Property(_) => {
                    while projected
                        .peek()
                        .is_some_and(|v| v.authored_ordinal() == ordinal)
                    {
                        let entry = projected.next().expect("peeked terminal");
                        reserve(&mut entries, 1)?;
                        entries.push(CssSpecifiedPageDeclarationEntry::Property(entry.clone()));
                    }
                    ordinal += 1;
                }
                CssPageDeclaration::Descriptor(v) => {
                    writer.node()?;
                    writer.without_output(|w| v.value().append_value(w))?;
                    if effective
                        .iter()
                        .flatten()
                        .any(|selected| std::ptr::eq(*selected, v))
                    {
                        reserve(&mut entries, 1)?;
                        entries.push(CssSpecifiedPageDeclarationEntry::Descriptor(v.clone()));
                    }
                }
            }
        }
        Ok(CssSpecifiedPageDeclarationBlock {
            entries,
            properties,
        })
    }
}
/// Borrowed canonical margin child using an already checked selected property block.
#[derive(Clone, Copy, Debug)]
pub struct CssMarginRuleView<'a> {
    name: CssMarginBox,
    declarations: &'a CssSpecifiedDeclarationBlock,
}
impl<'a> CssMarginRuleView<'a> {
    pub fn try_new(
        name: CssMarginBox,
        declarations: &'a CssSpecifiedDeclarationBlock,
    ) -> Option<Self> {
        declarations
            .is_margin()
            .then_some(Self { name, declarations })
    }
    #[must_use]
    pub const fn name(self) -> CssMarginBox {
        self.name
    }
    #[must_use]
    pub const fn declarations(self) -> &'a CssSpecifiedDeclarationBlock {
        self.declarations
    }
    pub fn serialize_cssom(self, limits: Limits) -> Result<String, CssRuleSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer
            .append_selected_margin(self)
            .map_err(|v| CssRuleSerializationError::new(v, Vec::new(), None))?;
        Ok(writer.css)
    }
}
/// Borrowed edited Page getter state, independent of a live CSSOM object.
#[derive(Clone, Copy, Debug)]
pub struct CssPageRuleView<'a> {
    selectors: &'a CssPageSelectorList,
    declarations: &'a CssSpecifiedPageDeclarationBlock,
    margins: &'a [CssMarginRuleView<'a>],
}
impl<'a> CssPageRuleView<'a> {
    #[must_use]
    pub const fn new(
        selectors: &'a CssPageSelectorList,
        declarations: &'a CssSpecifiedPageDeclarationBlock,
        margins: &'a [CssMarginRuleView<'a>],
    ) -> Self {
        Self {
            selectors,
            declarations,
            margins,
        }
    }
    #[must_use]
    pub const fn selectors(self) -> &'a CssPageSelectorList {
        self.selectors
    }
    #[must_use]
    pub const fn declarations(self) -> &'a CssSpecifiedPageDeclarationBlock {
        self.declarations
    }
    #[must_use]
    pub const fn margin_rules(self) -> &'a [CssMarginRuleView<'a>] {
        self.margins
    }
    pub fn serialize_cssom(self, limits: Limits) -> Result<String, CssRuleSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        let mut child = None;
        let result = writer
            .node()
            .map_err(RuleCssomSource::from)
            .and_then(|()| writer.append_selected_page(self, &mut child));
        result.map_err(|v| CssRuleSerializationError::new(v, child.into_iter().collect(), None))?;
        Ok(writer.css)
    }
}
impl SpecifiedRuleWriter {
    /// The graph or standalone view owns the logical Page charge.
    pub(crate) fn append_selected_page(
        &mut self,
        page: CssPageRuleView<'_>,
        child: &mut Option<usize>,
    ) -> Result<(), RuleCssomSource> {
        self.append_page_header(page.selectors)?;
        if self.append_selected_page_block(page.declarations)? {
            self.append(" ")?;
        }
        for (index, margin) in page.margins.iter().enumerate() {
            *child = Some(index);
            self.check_graph_child_depth()?;
            self.append_selected_margin(*margin)?;
            *child = None;
            self.append(" ")?;
        }
        self.append("}")?;
        Ok(())
    }
    pub(crate) fn append_selected_page_block(
        &mut self,
        block: &CssSpecifiedPageDeclarationBlock,
    ) -> Result<bool, RuleCssomSource> {
        // Validate selected semantic/source state cumulatively without source expansion.
        let checked = CssSpecifiedPageDeclarationBlock::from_entries(&block.entries, self)
            .map_err(|v| match v.kind {
                PageProjectionKind::Properties(v) => RuleCssomSource::DeclarationBlock(v),
                PageProjectionKind::Resource(v) => RuleCssomSource::from(v),
                PageProjectionKind::DuplicateDescriptor { .. } => {
                    unreachable!("private checked Page state")
                }
            })?;
        let mut segment = Vec::new();
        let mut output = false;
        for entry in &checked.entries {
            match entry {
                CssSpecifiedPageDeclarationEntry::Property(v) => {
                    reserve(&mut segment, 1)?;
                    segment.push(v.clone());
                }
                CssSpecifiedPageDeclarationEntry::Descriptor(v) => {
                    if !segment.is_empty() {
                        if output {
                            self.append(" ")?;
                        }
                        checked.properties.append_cssom_entries(&segment, self)?;
                        segment.clear();
                        output = true;
                    }
                    if output {
                        self.append(" ")?;
                    }
                    self.append(v.value().kind().as_str())?;
                    self.append(": ")?;
                    v.value().append_value(self)?;
                    if v.importance() == CssImportance::Important {
                        self.append(" !important")?;
                    }
                    self.append(";")?;
                    output = true;
                }
            }
        }
        if !segment.is_empty() {
            if output {
                self.append(" ")?;
            }
            checked.properties.append_cssom_entries(&segment, self)?;
            output = true;
        }
        Ok(output)
    }
    fn append_selected_margin(
        &mut self,
        margin: CssMarginRuleView<'_>,
    ) -> Result<(), RuleCssomSource> {
        self.node()?;
        self.append("@")?;
        self.append(margin.name.as_str())?;
        self.append(" { ")?;
        let block =
            CssSpecifiedDeclarationBlock::from_margin_entries(margin.declarations.entries(), self)?;
        block.append_cssom(self)?;
        if !block.entries().is_empty() {
            self.append(" ")?;
        }
        self.append("}")?;
        Ok(())
    }
}
