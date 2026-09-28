//! Bounded canonical specified text for symbolic Generated Content 3 values.

use crate::content_values::*;
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn charge(writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)
}

fn keyword(writer: &mut SpecifiedRuleWriter, value: &str) -> Result<()> {
    charge(writer)?;
    writer.append(value)
}

fn ident(writer: &mut SpecifiedRuleWriter, value: &str) -> Result<()> {
    charge(writer)?;
    writer.append_identifier(value)
}

fn string(writer: &mut SpecifiedRuleWriter, value: &str) -> Result<()> {
    charge(writer)?;
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

fn style(writer: &mut SpecifiedRuleWriter, value: &CssCounterStyleValue) -> Result<()> {
    match value {
        CssCounterStyleValue::Named(name) => ident(writer, name.as_str()),
        CssCounterStyleValue::Symbols(symbols) => {
            charge(writer)?;
            writer.append("symbols(")?;
            if let Some(system) = symbols.system() {
                if system == CssSymbolsSystem::Symbolic {
                    charge(writer)?;
                } else {
                    keyword(
                        writer,
                        match system {
                            CssSymbolsSystem::Cyclic => "cyclic",
                            CssSymbolsSystem::Numeric => "numeric",
                            CssSymbolsSystem::Alphabetic => "alphabetic",
                            CssSymbolsSystem::Symbolic => unreachable!(),
                            CssSymbolsSystem::Fixed => "fixed",
                        },
                    )?;
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
        let mut writer = SpecifiedRuleWriter::new(limits);
        style(&mut writer, self)?;
        Ok(writer.css)
    }
}

fn counter(writer: &mut SpecifiedRuleWriter, value: &CssContentCounter) -> Result<()> {
    charge(writer)?;
    writer.append("counter(")?;
    ident(writer, value.name().as_str())?;
    if let Some(style_value) = value.style() {
        if matches!(style_value.named(), Some(name) if name.as_str() == "decimal") {
            // The explicit default is traversed even though CSSOM omits it.
            charge(writer)?;
        } else {
            writer.append(", ")?;
            style(writer, style_value)?;
        }
    }
    writer.append(")")
}

fn counters(writer: &mut SpecifiedRuleWriter, value: &CssContentCounters) -> Result<()> {
    charge(writer)?;
    writer.append("counters(")?;
    ident(writer, value.name().as_str())?;
    writer.append(", ")?;
    string(writer, value.separator().as_str())?;
    if let Some(style_value) = value.style() {
        if matches!(style_value.named(), Some(name) if name.as_str() == "decimal") {
            charge(writer)?;
        } else {
            writer.append(", ")?;
            style(writer, style_value)?;
        }
    }
    writer.append(")")
}

fn item(writer: &mut SpecifiedRuleWriter, value: &CssContentValueItem) -> Result<()> {
    match value {
        CssContentValueItem::String(value) => string(writer, value.as_str()),
        CssContentValueItem::Image(value) => value.append_specified(writer),
        CssContentValueItem::Contents => keyword(writer, "contents"),
        CssContentValueItem::OpenQuote => keyword(writer, "open-quote"),
        CssContentValueItem::CloseQuote => keyword(writer, "close-quote"),
        CssContentValueItem::NoOpenQuote => keyword(writer, "no-open-quote"),
        CssContentValueItem::NoCloseQuote => keyword(writer, "no-close-quote"),
        CssContentValueItem::Counter(value) => counter(writer, value),
        CssContentValueItem::Counters(value) => counters(writer, value),
        CssContentValueItem::Leader(value) => {
            charge(writer)?;
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
            charge(writer)?;
            writer.append("target-counter(")?;
            target(writer, value.target())?;
            writer.append(", ")?;
            ident(writer, value.name().as_str())?;
            if let Some(style_value) = value.style() {
                writer.append(", ")?;
                style(writer, style_value)?;
            }
            writer.append(")")
        }
        CssContentValueItem::TargetCounters(value) => {
            charge(writer)?;
            writer.append("target-counters(")?;
            target(writer, value.target())?;
            writer.append(", ")?;
            ident(writer, value.name().as_str())?;
            writer.append(", ")?;
            string(writer, value.separator().as_str())?;
            if let Some(style_value) = value.style() {
                writer.append(", ")?;
                style(writer, style_value)?;
            }
            writer.append(")")
        }
        CssContentValueItem::TargetText(value) => {
            charge(writer)?;
            writer.append("target-text(")?;
            target(writer, value.target())?;
            if let Some(mode) = value.mode() {
                writer.append(", ")?;
                keyword(writer, mode_text(mode))?;
            }
            writer.append(")")
        }
        CssContentValueItem::NamedString(value) => {
            charge(writer)?;
            writer.append("string(")?;
            ident(writer, value.name().as_str())?;
            if let Some(mode) = value.mode() {
                if mode == CssNamedStringMode::First {
                    charge(writer)?;
                } else {
                    writer.append(", ")?;
                    keyword(writer, name_mode_text(mode))?;
                }
            }
            writer.append(")")
        }
        CssContentValueItem::Content(mode) => {
            charge(writer)?;
            writer.append("content(")?;
            if let Some(mode) = mode {
                if *mode == CssContentReferenceMode::Text {
                    charge(writer)?;
                } else {
                    keyword(writer, content_mode_text(*mode))?;
                }
            }
            writer.append(")")
        }
    }
}

fn alternative(writer: &mut SpecifiedRuleWriter, value: &CssContentAlternative) -> Result<()> {
    charge(writer)?;
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
        let mut writer = SpecifiedRuleWriter::new(limits);
        match self {
            CssContentValue::Normal => keyword(&mut writer, "normal")?,
            CssContentValue::None => keyword(&mut writer, "none")?,
            CssContentValue::Generated(value) => {
                charge(&mut writer)?;
                for (index, item_value) in value.items().iter().enumerate() {
                    if index != 0 {
                        writer.append(" ")?;
                    }
                    item(&mut writer, item_value)?;
                }
                if let Some(value) = value.alternative() {
                    writer.append(" / ")?;
                    alternative(&mut writer, value)?;
                }
            }
        }
        Ok(writer.css)
    }
}
