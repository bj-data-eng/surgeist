//! Page-specific getters and coherent adopted Blink wrappers under one request budget.
use crate::declaration::{
    CssDeclarationBlockError, CssImportance, CssSpecifiedDeclarationBlock,
    CssSpecifiedDeclarationEntry,
};
use crate::rule::page::CssPageDescriptorValueRef as V;
use crate::rule::page::{
    CssMarginRule, CssPageDeclaration, CssPageDeclarationBlock, CssPageDescriptorValue,
    CssPageOrientation, CssPageOutputOrientation, CssPageRule, CssPageSelectorList,
    CssPageSizeValue,
};
use crate::serialization::CssSerializationError as Error;
use crate::serialization::CssSerializationLimits as Limits;
use crate::serialization::cssom_rule_serialization::RuleCssomSource;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::values::common::CssAutoOr;
type Result<T> = std::result::Result<T, Error>;
impl CssPageSelectorList {
    pub fn serialize_cssom(&self, limits: Limits) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.append_page_selectors(self)?;
        Ok(writer.css)
    }
}
impl CssPageDescriptorValue {
    pub(crate) fn append_value(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        match self.view() {
            V::Global(v) => crate::declaration::declaration_serialization::append_global(v, writer),
            V::Pending(_) => crate::declaration::declaration_serialization::append_retained_value(
                self.components(),
                writer,
            ),
            V::PageOrientation(v) => writer.append(match v {
                CssPageOutputOrientation::Upright => "upright",
                CssPageOutputOrientation::RotateLeft => "rotate-left",
                CssPageOutputOrientation::RotateRight => "rotate-right",
            }),
            V::Marks(v) => writer.append(match (v.crop(), v.cross()) {
                (false, false) => "none",
                (true, false) => "crop",
                (false, true) => "cross",
                (true, true) => "crop cross",
            }),
            V::Bleed(v) => match v {
                CssAutoOr::Auto => writer.append("auto"),
                CssAutoOr::Value(v) => v.append_specified(&mut writer.context, &mut writer.css),
            },
            V::Size(v) => match v {
                CssPageSizeValue::Auto => writer.append("auto"),
                CssPageSizeValue::Dimensions(a, b) => {
                    a.append_specified(&mut writer.context, &mut writer.css)?;
                    if let Some(b) = b {
                        writer.append(" ")?;
                        b.append_specified(&mut writer.context, &mut writer.css)?;
                    }
                    Ok(())
                }
                CssPageSizeValue::Named(value) => {
                    let size = value.size();
                    let orientation = value.orientation();
                    if let Some(size) = size {
                        writer.node()?;
                        writer.append(size.css_name())?;
                    }
                    if let Some(orientation) = orientation {
                        if size.is_some() {
                            writer.append(" ")?;
                        }
                        writer.node()?;
                        writer.append(match orientation {
                            CssPageOrientation::Portrait => "portrait",
                            CssPageOrientation::Landscape => "landscape",
                        })?;
                    }
                    Ok(())
                }
            },
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssPageDescriptorValue
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_value(writer)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(
    CssPageDescriptorValue
);
impl SpecifiedRuleWriter {
    fn append_page_selectors(&mut self, list: &CssPageSelectorList) -> Result<()> {
        self.node()?;
        for (index, selector) in list.selectors().iter().enumerate() {
            self.node()?;
            if index != 0 {
                self.append(", ")?;
            }
            if let Some(name) = selector.name() {
                self.node()?;
                self.append_identifier(name.as_str())?;
            }
            for pseudo in selector.pseudos() {
                self.node()?;
                self.append(":")?;
                self.append(pseudo.as_str())?;
            }
        }
        Ok(())
    }
    pub(crate) fn append_page_compact(&mut self, rule: &CssPageRule) -> Result<()> {
        self.append("@page")?;
        if !rule.selectors().is_empty() {
            self.append(" ")?;
        }
        self.append_page_selectors(rule.selectors())?;
        self.append(" { ")?;
        for (index, entry) in rule.declarations().occurrences().iter().enumerate() {
            if index != 0 {
                self.append(" ")?;
            }
            match entry {
                CssPageDeclaration::Property(v) => {
                    crate::declaration::declaration_serialization::append_authored_declaration(
                        v.body(),
                        v.value_components(),
                        v.importance(),
                        self,
                    )?
                }
                CssPageDeclaration::Descriptor(v) => {
                    self.node()?;
                    self.append(v.value().kind().as_str())?;
                    self.append(": ")?;
                    v.value().append_value(self)?;
                    if v.importance() == CssImportance::Important {
                        self.append(" !important")?;
                    }
                    self.append(";")?;
                }
            }
        }
        if !rule.declarations().is_empty() {
            self.append(" ")?;
        }
        for margin in rule.margin_rules() {
            self.node()?;
            self.append("@")?;
            self.append(margin.name().as_str())?;
            self.append(" { ")?;
            self.append_authored_declaration_list(margin.declarations().properties())?;
            if !margin.declarations().is_empty() {
                self.append(" ")?;
            }
            self.append("} ")?;
        }
        self.append("}")
    }
    pub(crate) fn append_page_header(&mut self, selectors: &CssPageSelectorList) -> Result<()> {
        self.append("@page ")?;
        self.append_page_selectors(selectors)?;
        if !selectors.is_empty() {
            self.append(" ")?;
        }
        self.append("{ ")
    }
    pub(crate) fn append_page_cssom(
        &mut self,
        rule: &CssPageRule,
        margin_index: &mut Option<usize>,
    ) -> std::result::Result<(), RuleCssomSource> {
        self.append_page_header(rule.selectors())?;
        let has_output = self.append_page_block(rule.declarations())?;
        if has_output {
            self.append(" ")?;
        }
        for (index, margin) in rule.margin_rules().iter().enumerate() {
            *margin_index = Some(index);
            self.check_graph_child_depth()?;
            self.append_margin_cssom(margin)?;
            *margin_index = None;
            self.append(" ")?;
        }
        self.append("}")?;
        Ok(())
    }
    fn append_margin_cssom(
        &mut self,
        margin: &CssMarginRule,
    ) -> std::result::Result<(), RuleCssomSource> {
        self.node()?;
        self.append("@")?;
        self.append(margin.name().as_str())?;
        self.append(" { ")?;
        let block = CssSpecifiedDeclarationBlock::from_margin_sources(
            margin.declarations().properties().iter(),
            self,
        )?;
        block.append_cssom(self)?;
        if !block.entries().is_empty() {
            self.append(" ")?;
        }
        self.append("}")?;
        Ok(())
    }
    fn append_page_block(
        &mut self,
        block: &CssPageDeclarationBlock,
    ) -> std::result::Result<bool, RuleCssomSource> {
        self.node()?;
        let projection =
            CssSpecifiedDeclarationBlock::from_page_sources(block.properties().iter(), self)?;
        let effective = [
            crate::rule::page::CssPageDescriptorKind::Size,
            crate::rule::page::CssPageDescriptorKind::PageOrientation,
            crate::rule::page::CssPageDescriptorKind::Marks,
            crate::rule::page::CssPageDescriptorKind::Bleed,
        ]
        .map(|kind| block.effective_descriptor(kind));
        let mut projected = projection.entries().iter().peekable();
        let mut output = false;
        let mut property_ordinal = 0;
        let mut segment = Vec::<CssSpecifiedDeclarationEntry>::new();
        for entry in block.occurrences() {
            match entry {
                CssPageDeclaration::Property(_) => {
                    while projected
                        .peek()
                        .is_some_and(|v| v.authored_ordinal() == property_ordinal)
                    {
                        let value = projected.next().expect("peeked terminal");
                        segment.try_reserve(1).map_err(|_| {
                            Error::new(
                                crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                            )
                        })?;
                        segment.push(value.clone());
                    }
                    property_ordinal += 1;
                }
                CssPageDeclaration::Descriptor(value) => {
                    self.node()?;
                    self.without_output(|writer| value.value().append_value(writer))?;
                    let selected = effective.iter().flatten().any(|v| std::ptr::eq(*v, value));
                    if !selected {
                        continue;
                    }
                    if !segment.is_empty() {
                        if output {
                            self.append(" ")?;
                        }
                        projection.append_cssom_entries(&segment, self)?;
                        segment.clear();
                        output = true;
                    }
                    if output {
                        self.append(" ")?;
                    }
                    self.node()?;
                    self.append(value.value().kind().as_str())?;
                    self.append(": ")?;
                    value.value().append_value(self)?;
                    if value.importance() == CssImportance::Important {
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
            projection.append_cssom_entries(&segment, self)?;
            output = true;
        }
        Ok(output)
    }
}
impl CssMarginRule {
    pub fn serialize_cssom(
        &self,
        limits: Limits,
    ) -> std::result::Result<String, crate::serialization::CssRuleSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.append_margin_cssom(self).map_err(|error| {
            crate::serialization::CssRuleSerializationError::new(error, Vec::new(), None)
        })?;
        Ok(writer.css)
    }
}
impl CssPageDeclarationBlock {
    pub fn serialize_cssom(
        &self,
        limits: Limits,
    ) -> std::result::Result<String, crate::serialization::CssRuleSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.append_page_block(self).map_err(|error| {
            crate::serialization::CssRuleSerializationError::new(error, Vec::new(), None)
        })?;
        Ok(writer.css)
    }
    pub fn try_specified_properties(
        &self,
        limits: Limits,
    ) -> std::result::Result<CssSpecifiedDeclarationBlock, CssDeclarationBlockError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        CssSpecifiedDeclarationBlock::from_page_sources(self.properties().iter(), &mut writer)
    }
}
