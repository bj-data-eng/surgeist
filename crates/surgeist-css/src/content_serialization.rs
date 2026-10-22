//! Bounded canonical specified text for symbolic Generated Content 3 values.

use crate::content_values::*;
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn ident(writer: &mut SpecifiedRuleWriter, value: &str) -> Result<()> {
    writer.node()?;
    writer.append_identifier(value)
}

fn string(writer: &mut SpecifiedRuleWriter, value: &str) -> Result<()> {
    writer.node()?;
    writer.append_string(value)
}

fn mode_text(mode: CssTargetTextMode) -> &'static str {
    match mode {
        CssTargetTextMode::Content => "content",
        CssTargetTextMode::Before => "before",
        CssTargetTextMode::After => "after",
        CssTargetTextMode::FirstLetter => "first-letter",
    }
}

fn name_mode_text(mode: CssNamedStringMode) -> &'static str {
    match mode {
        CssNamedStringMode::First => "first",
        CssNamedStringMode::Start => "start",
        CssNamedStringMode::Last => "last",
        CssNamedStringMode::FirstExcept => "first-except",
    }
}

fn content_mode_text(mode: CssContentReferenceMode) -> &'static str {
    match mode {
        CssContentReferenceMode::Text => "text",
        CssContentReferenceMode::Before => "before",
        CssContentReferenceMode::After => "after",
        CssContentReferenceMode::FirstLetter => "first-letter",
        CssContentReferenceMode::Marker => "marker",
    }
}

fn target(writer: &mut SpecifiedRuleWriter, value: &CssContentTarget) -> Result<()> {
    match value {
        CssContentTarget::String(value) => string(writer, value.as_str()),
        CssContentTarget::Url(value) => value.append_specified(writer),
    }
}

pub(crate) fn style(writer: &mut SpecifiedRuleWriter, value: &CssCounterStyleValue) -> Result<()> {
    match value {
        CssCounterStyleValue::Named(name) => ident(writer, name.as_str()),
        CssCounterStyleValue::Symbols(symbols) => {
            writer.node()?;
            writer.append("symbols(")?;
            if let Some(system) = symbols.system() {
                if system == CssSymbolsSystem::Symbolic {
                    writer.node()?;
                } else {
                    writer.keyword(match system {
                        CssSymbolsSystem::Cyclic => "cyclic",
                        CssSymbolsSystem::Numeric => "numeric",
                        CssSymbolsSystem::Alphabetic => "alphabetic",
                        CssSymbolsSystem::Symbolic => unreachable!(),
                        CssSymbolsSystem::Fixed => "fixed",
                    })?;
                    writer.append(" ")?;
                }
            }
            for (index, symbol) in symbols.symbols().iter().enumerate() {
                if index != 0 {
                    writer.append(" ")?;
                }
                match symbol {
                    CssCounterSymbolValue::String(value) => string(writer, value.as_str())?,
                    CssCounterSymbolValue::Image(value) => value.append_specified(writer)?,
                }
            }
            writer.append(")")
        }
    }
}

impl CssCounterStyleValue {
    /// Serializes one checked named or `symbols()` counter style without resolving counters.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Applies one cumulative input, projection, and CSS byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        style(writer, self)?;
        Ok(())
    }
}

/// Writes an optional trailing argument, omitting one equal to its default.
///
/// CSSOM "serialize a CSS value" step 2 omits components whose omission does
/// not change the meaning. The explicit default is still traversed and
/// charged as one node, so omission never refunds work.
fn optional_argument<T>(
    writer: &mut SpecifiedRuleWriter,
    value: Option<T>,
    is_default: impl FnOnce(&T) -> bool,
    write: impl FnOnce(&mut SpecifiedRuleWriter, T) -> Result<()>,
) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if is_default(&value) {
        return writer.node();
    }
    writer.append(", ")?;
    write(writer, value)
}

/// The counter style argument of the Generated Content 3 counter functions
/// defaults to `decimal`.
fn optional_counter_style(
    writer: &mut SpecifiedRuleWriter,
    value: Option<&CssCounterStyleValue>,
) -> Result<()> {
    optional_argument(
        writer,
        value,
        |value| matches!(value.named(), Some(name) if name.as_str() == "decimal"),
        style,
    )
}

fn counter(writer: &mut SpecifiedRuleWriter, value: &CssContentCounter) -> Result<()> {
    writer.node()?;
    writer.append("counter(")?;
    ident(writer, value.name().as_str())?;
    optional_counter_style(writer, value.style())?;
    writer.append(")")
}

fn counters(writer: &mut SpecifiedRuleWriter, value: &CssContentCounters) -> Result<()> {
    writer.node()?;
    writer.append("counters(")?;
    ident(writer, value.name().as_str())?;
    writer.append(", ")?;
    string(writer, value.separator().as_str())?;
    optional_counter_style(writer, value.style())?;
    writer.append(")")
}

fn item(writer: &mut SpecifiedRuleWriter, value: &CssContentValueItem) -> Result<()> {
    match value {
        CssContentValueItem::String(value) => string(writer, value.as_str()),
        CssContentValueItem::Image(value) => value.append_specified(writer),
        CssContentValueItem::Contents => writer.keyword("contents"),
        CssContentValueItem::OpenQuote => writer.keyword("open-quote"),
        CssContentValueItem::CloseQuote => writer.keyword("close-quote"),
        CssContentValueItem::NoOpenQuote => writer.keyword("no-open-quote"),
        CssContentValueItem::NoCloseQuote => writer.keyword("no-close-quote"),
        CssContentValueItem::Counter(value) => counter(writer, value),
        CssContentValueItem::Counters(value) => counters(writer, value),
        CssContentValueItem::Leader(value) => {
            writer.node()?;
            writer.append("leader(")?;
            match value {
                CssLeaderValue::Dotted => string(writer, ".")?,
                CssLeaderValue::Solid => string(writer, "_")?,
                CssLeaderValue::Space => string(writer, " ")?,
                CssLeaderValue::String(value) => string(writer, value.as_str())?,
            }
            writer.append(")")
        }
        CssContentValueItem::TargetCounter(value) => {
            writer.node()?;
            writer.append("target-counter(")?;
            target(writer, value.target())?;
            writer.append(", ")?;
            ident(writer, value.name().as_str())?;
            optional_counter_style(writer, value.style())?;
            writer.append(")")
        }
        CssContentValueItem::TargetCounters(value) => {
            writer.node()?;
            writer.append("target-counters(")?;
            target(writer, value.target())?;
            writer.append(", ")?;
            ident(writer, value.name().as_str())?;
            writer.append(", ")?;
            string(writer, value.separator().as_str())?;
            optional_counter_style(writer, value.style())?;
            writer.append(")")
        }
        CssContentValueItem::TargetText(value) => {
            writer.node()?;
            writer.append("target-text(")?;
            target(writer, value.target())?;
            // Generated Content 3: the `target-text()` mode defaults to `content`.
            optional_argument(
                writer,
                value.mode(),
                |mode| *mode == CssTargetTextMode::Content,
                |writer, mode| writer.keyword(mode_text(mode)),
            )?;
            writer.append(")")
        }
        CssContentValueItem::NamedString(value) => {
            writer.node()?;
            writer.append("string(")?;
            ident(writer, value.name().as_str())?;
            optional_argument(
                writer,
                value.mode(),
                |mode| *mode == CssNamedStringMode::First,
                |writer, mode| writer.keyword(name_mode_text(mode)),
            )?;
            writer.append(")")
        }
        CssContentValueItem::Content(mode) => {
            writer.node()?;
            writer.append("content(")?;
            // The only argument has no preceding comma.
            if let Some(mode) = mode {
                if *mode == CssContentReferenceMode::Text {
                    writer.node()?;
                } else {
                    writer.keyword(content_mode_text(*mode))?;
                }
            }
            writer.append(")")
        }
    }
}

fn alternative(writer: &mut SpecifiedRuleWriter, value: &CssContentAlternative) -> Result<()> {
    writer.node()?;
    for (index, item_value) in value.items().iter().enumerate() {
        if index != 0 {
            writer.append(" ")?;
        }
        match item_value {
            CssContentAlternativeItem::String(value) => string(writer, value.as_str())?,
            CssContentAlternativeItem::Counter(value) => counter(writer, value)?,
            CssContentAlternativeItem::Counters(value) => counters(writer, value)?,
        }
    }
    Ok(())
}

impl CssContentValue {
    /// Serializes current authored content without resolving counters, targets, or generated boxes.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Applies one cumulative resource budget across every retained branch.
    /// A generated body and an alternative each charge one aggregate node;
    /// functions and `symbols()` charge one aggregate plus their checked leaves.
    /// Image and URL children use their own established node accounting in this writer.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        match self {
            CssContentValue::Normal => writer.keyword("normal")?,
            CssContentValue::None => writer.keyword("none")?,
            CssContentValue::Generated(value) => {
                value.list().append_to_rule_writer(writer)?;
                if let Some(value) = value.alternative() {
                    writer.append(" / ")?;
                    alternative(writer, value)?;
                }
            }
        }
        Ok(())
    }
}

impl CssContentList {
    /// Serializes an ordered content list without resolving its context.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Applies one cumulative input, projection and byte budget to all children.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.space_separated(self.items(), item)?;
        Ok(())
    }
}

impl CssStringSet {
    /// Serializes separate named string assignments without concatenation.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Applies one cumulative input, projection and byte budget to all children.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        let Some(entries) = self.entries() else {
            return writer.keyword("none");
        };
        writer.node()?;
        for (index, entry) in entries.iter().enumerate() {
            if index != 0 {
                writer.append(", ")?;
            }
            writer.node()?;
            ident(writer, entry.name().as_str())?;
            for value in entry.strings() {
                writer.append(" ")?;
                string(writer, value.as_str())?;
            }
        }
        Ok(())
    }
}

impl CssBookmarkLevel {
    /// Serializes the exact level or its symbolic integer calculation.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Applies one cumulative input, projection and byte budget to all children.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self.level() {
            None => writer.keyword("none"),
            Some(level) => level.append_to_rule_writer(writer),
        }
    }
}

impl CssBookmarkState {
    /// Serializes the authored bookmark state without toggling a subtree.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Applies one cumulative input, projection and byte budget to all children.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.keyword(match self {
            Self::Open => "open",
            Self::Closed => "closed",
        })
    }
}
