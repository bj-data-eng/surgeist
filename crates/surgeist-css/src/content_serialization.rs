//! Bounded canonical specified text for symbolic Generated Content 3 values.

use crate::CssSerializationError;
use crate::content_values::*;
use crate::specified_rule_serialization::SpecifiedRuleWriter;

type Result<T> = std::result::Result<T, CssSerializationError>;

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
        CssContentTarget::String(value) => writer.string(value.as_str()),
        CssContentTarget::Url(value) => value.append_specified(writer),
    }
}

pub(crate) fn style(writer: &mut SpecifiedRuleWriter, value: &CssCounterStyleValue) -> Result<()> {
    match value {
        CssCounterStyleValue::Named(name) => writer.identifier(name.as_str()),
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
                    CssCounterSymbolValue::String(value) => writer.string(value.as_str())?,
                    CssCounterSymbolValue::Image(value) => value.append_specified(writer)?,
                }
            }
            writer.append(")")
        }
    }
}

impl crate::specified_rule_serialization::SpecifiedProvider for CssCounterStyleValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        style(writer, self)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssCounterStyleValue);

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
    writer.identifier(value.name().as_str())?;
    optional_counter_style(writer, value.style())?;
    writer.append(")")
}

fn counters(writer: &mut SpecifiedRuleWriter, value: &CssContentCounters) -> Result<()> {
    writer.node()?;
    writer.append("counters(")?;
    writer.identifier(value.name().as_str())?;
    writer.append(", ")?;
    writer.string(value.separator().as_str())?;
    optional_counter_style(writer, value.style())?;
    writer.append(")")
}

fn item(writer: &mut SpecifiedRuleWriter, value: &CssContentValueItem) -> Result<()> {
    match value {
        CssContentValueItem::String(value) => writer.string(value.as_str()),
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
                CssLeaderValue::Dotted => writer.string(".")?,
                CssLeaderValue::Solid => writer.string("_")?,
                CssLeaderValue::Space => writer.string(" ")?,
                CssLeaderValue::String(value) => writer.string(value.as_str())?,
            }
            writer.append(")")
        }
        CssContentValueItem::TargetCounter(value) => {
            writer.node()?;
            writer.append("target-counter(")?;
            target(writer, value.target())?;
            writer.append(", ")?;
            writer.identifier(value.name().as_str())?;
            optional_counter_style(writer, value.style())?;
            writer.append(")")
        }
        CssContentValueItem::TargetCounters(value) => {
            writer.node()?;
            writer.append("target-counters(")?;
            target(writer, value.target())?;
            writer.append(", ")?;
            writer.identifier(value.name().as_str())?;
            writer.append(", ")?;
            writer.string(value.separator().as_str())?;
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
            writer.identifier(value.name().as_str())?;
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
            CssContentAlternativeItem::String(value) => writer.string(value.as_str())?,
            CssContentAlternativeItem::Counter(value) => counter(writer, value)?,
            CssContentAlternativeItem::Counters(value) => counters(writer, value)?,
        }
    }
    Ok(())
}

impl crate::specified_rule_serialization::SpecifiedProvider for CssContentValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
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
crate::specified_rule_serialization::specified_serialization!(CssContentValue);

impl crate::specified_rule_serialization::SpecifiedProvider for CssContentList {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.space_separated(self.items(), item)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssContentList);

impl crate::specified_rule_serialization::SpecifiedProvider for CssStringSet {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        let Some(entries) = self.entries() else {
            return writer.keyword("none");
        };
        writer.node()?;
        for (index, entry) in entries.iter().enumerate() {
            if index != 0 {
                writer.append(", ")?;
            }
            writer.node()?;
            writer.identifier(entry.name().as_str())?;
            for value in entry.strings() {
                writer.append(" ")?;
                writer.string(value.as_str())?;
            }
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssStringSet);

impl crate::specified_rule_serialization::SpecifiedProvider for CssBookmarkLevel {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self.level() {
            None => writer.keyword("none"),
            Some(level) => level.append_to_rule_writer(writer),
        }
    }
}
crate::specified_rule_serialization::specified_serialization!(CssBookmarkLevel);

impl crate::specified_rule_serialization::SpecifiedProvider for CssBookmarkState {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.keyword(match self {
            Self::Open => "open",
            Self::Closed => "closed",
        })
    }
}
crate::specified_rule_serialization::specified_serialization!(CssBookmarkState);
