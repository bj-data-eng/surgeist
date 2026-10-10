//! Bounded specified text for represented Counter Styles 3 rules.

use crate::{
    CssCounterStyleDescriptorKind, CssCounterStyleDescriptorRef as Descriptor,
    CssCounterStyleDescriptorValue, CssCounterStyleDescriptorValueRef as Value,
    CssCounterStyleRange, CssCounterStyleRangeBound, CssCounterStyleRule, CssCounterStyleSpeakAs,
    CssCounterStyleSystem, CssCounterSymbol, CssSpecifiedValueSerializationError as Error,
    CssSpecifiedValueSerializationLimits as Limits,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, Error>;

impl CssCounterStyleDescriptorKind {
    const fn index(self) -> usize {
        match self {
            Self::System => 0,
            Self::Negative => 1,
            Self::Prefix => 2,
            Self::Suffix => 3,
            Self::Range => 4,
            Self::Pad => 5,
            Self::Fallback => 6,
            Self::Symbols => 7,
            Self::AdditiveSymbols => 8,
            Self::SpeakAs => 9,
        }
    }
}

fn node(writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)
}

fn keyword(writer: &mut SpecifiedRuleWriter, value: &str) -> Result<()> {
    node(writer)?;
    writer.append(value)
}

fn name(writer: &mut SpecifiedRuleWriter, value: &str) -> Result<()> {
    node(writer)?;
    writer.append_identifier(value)
}

fn symbol(writer: &mut SpecifiedRuleWriter, value: &CssCounterSymbol) -> Result<()> {
    match value {
        CssCounterSymbol::String(value) => {
            node(writer)?;
            writer.append_string(value.as_str())
        }
        CssCounterSymbol::Ident(value) => name(writer, value.as_str()),
        CssCounterSymbol::Image(value) => value.append_specified(writer),
    }
}

fn bound(writer: &mut SpecifiedRuleWriter, value: &CssCounterStyleRangeBound) -> Result<()> {
    match value {
        CssCounterStyleRangeBound::Infinite => keyword(writer, "infinite"),
        CssCounterStyleRangeBound::Integer(value) => {
            value.append_specified(&mut writer.context, &mut writer.css)
        }
    }
}

// Counter Styles 3 §3 chooses the last valid descriptor. Neither that snapshot's
// §9.2 nor pinned CSSOM1 defines complete counter-style cssText ordering. The
// project selects §3's descriptor section order, retaining symbols before
// additive-symbols within §3.8:
// https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/#the-counter-style-rule
// https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-rule
fn slot(value: Descriptor<'_>) -> (usize, &'static str) {
    match value {
        Descriptor::System(_) => (0, "system"),
        Descriptor::Negative(_) => (1, "negative"),
        Descriptor::Prefix(_) => (2, "prefix"),
        Descriptor::Suffix(_) => (3, "suffix"),
        Descriptor::Range(_) => (4, "range"),
        Descriptor::Pad(_) => (5, "pad"),
        Descriptor::Fallback(_) => (6, "fallback"),
        Descriptor::Symbols(_) => (7, "symbols"),
        Descriptor::AdditiveSymbols(_) => (8, "additive-symbols"),
        Descriptor::SpeakAs(_) => (9, "speak-as"),
    }
}

impl CssCounterStyleDescriptorValue {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self.view() {
            Value::System(value) => match value {
                CssCounterStyleSystem::Cyclic => keyword(writer, "cyclic"),
                CssCounterStyleSystem::Numeric => keyword(writer, "numeric"),
                CssCounterStyleSystem::Alphabetic => keyword(writer, "alphabetic"),
                CssCounterStyleSystem::Symbolic => keyword(writer, "symbolic"),
                CssCounterStyleSystem::Additive => keyword(writer, "additive"),
                CssCounterStyleSystem::Fixed(value) => {
                    keyword(writer, "fixed")?;
                    if let Some(first) = value.first_symbol_value() {
                        // §3.1.2's omitted starting value is exactly 1. CSSOM §6.7.2
                        // omits optional components when their meaning is unchanged.
                        if matches!(first, crate::CssIntegerValue::Literal(literal)
                            if crate::integer_value::exact_i32(literal.numeric().representation()) == Some(1))
                        {
                            writer.without_output(|writer| {
                                first.append_specified(&mut writer.context, &mut writer.css)
                            })?;
                        } else {
                            writer.append(" ")?;
                            first.append_specified(&mut writer.context, &mut writer.css)?;
                        }
                    }
                    Ok(())
                }
                CssCounterStyleSystem::Extends(value) => {
                    keyword(writer, "extends")?;
                    writer.append(" ")?;
                    name(writer, value.as_str())
                }
            },
            Value::Negative(value) => {
                node(writer)?;
                symbol(writer, value.prefix())?;
                if let Some(suffix) = value.suffix() {
                    // §3.2's optional suffix adds nothing when it is the empty
                    // string; traverse it even when CSSOM omission removes bytes.
                    if matches!(suffix, CssCounterSymbol::String(value) if value.as_str().is_empty())
                    {
                        writer.without_output(|writer| symbol(writer, suffix))?;
                    } else {
                        writer.append(" ")?;
                        symbol(writer, suffix)?;
                    }
                }
                Ok(())
            }
            Value::Prefix(value) | Value::Suffix(value) => symbol(writer, value),
            Value::Range(value) => match value {
                CssCounterStyleRange::Auto => keyword(writer, "auto"),
                CssCounterStyleRange::Ranges(value) => {
                    node(writer)?;
                    for (index, interval) in value.ranges().iter().enumerate() {
                        node(writer)?;
                        if index != 0 {
                            writer.append(", ")?;
                        }
                        // Both bounds are required even when equal (§3.5).
                        bound(writer, interval.lower())?;
                        writer.append(" ")?;
                        bound(writer, interval.upper())?;
                    }
                    Ok(())
                }
            },
            Value::Pad(value) => {
                node(writer)?;
                value
                    .minimum_length()
                    .append_specified(&mut writer.context, &mut writer.css)?;
                writer.append(" ")?;
                symbol(writer, value.symbol())
            }
            Value::Fallback(value) => name(writer, value.as_str()),
            Value::Symbols(value) => {
                node(writer)?;
                for (index, value) in value.symbols().iter().enumerate() {
                    if index != 0 {
                        writer.append(" ")?;
                    }
                    symbol(writer, value)?;
                }
                Ok(())
            }
            Value::AdditiveSymbols(value) => {
                node(writer)?;
                for (index, tuple) in value.tuples().iter().enumerate() {
                    node(writer)?;
                    if index != 0 {
                        writer.append(", ")?;
                    }
                    // §3.8's double-ampersand grammar takes its canonical integer-first order.
                    tuple
                        .weight()
                        .append_specified(&mut writer.context, &mut writer.css)?;
                    writer.append(" ")?;
                    symbol(writer, tuple.symbol())?;
                }
                Ok(())
            }
            Value::SpeakAs(value) => match value {
                CssCounterStyleSpeakAs::Auto => keyword(writer, "auto"),
                CssCounterStyleSpeakAs::Bullets => keyword(writer, "bullets"),
                CssCounterStyleSpeakAs::Numbers => keyword(writer, "numbers"),
                CssCounterStyleSpeakAs::Words => keyword(writer, "words"),
                CssCounterStyleSpeakAs::SpellOut => keyword(writer, "spell-out"),
                CssCounterStyleSpeakAs::CounterStyle(value) => name(writer, value.as_str()),
            },
            Value::Pending(_) => crate::pending_serialization::append_pending_specified(
                self.components(),
                &mut writer.context,
                &mut writer.css,
            ),
        }
    }
    /// Serializes the value alone with the same provider and cumulative budget as a rule.
    pub fn to_specified_css(&self) -> Result<String> {
        self.to_specified_css_with_limits(Limits::default())
    }
    pub fn to_specified_css_with_limits(&self, limits: Limits) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }
}
fn descriptor(writer: &mut SpecifiedRuleWriter, value: Descriptor<'_>) -> Result<()> {
    match value {
        Descriptor::System(value) => value.value().append_to_rule_writer(writer),
        Descriptor::Negative(value) => value.value().append_to_rule_writer(writer),
        Descriptor::Prefix(value) => value.value().append_to_rule_writer(writer),
        Descriptor::Suffix(value) => value.value().append_to_rule_writer(writer),
        Descriptor::Range(value) => value.value().append_to_rule_writer(writer),
        Descriptor::Pad(value) => value.value().append_to_rule_writer(writer),
        Descriptor::Fallback(value) => value.value().append_to_rule_writer(writer),
        Descriptor::Symbols(value) => value.value().append_to_rule_writer(writer),
        Descriptor::AdditiveSymbols(value) => value.value().append_to_rule_writer(writer),
        Descriptor::SpeakAs(value) => value.value().append_to_rule_writer(writer),
    }
}

impl CssCounterStyleRule {
    /// Serializes the literal rule name as a CSS identifier without resolving it.
    pub fn name_specified_css(&self) -> Result<String> {
        self.name_specified_css_with_limits(Limits::default())
    }

    /// Serializes the name as one input and projection leaf under the byte limit.
    pub fn name_specified_css_with_limits(&self, limits: Limits) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        name(&mut writer, self.name().as_str())?;
        Ok(writer.css)
    }

    /// Serializes a present effective descriptor, or returns an empty string if omitted.
    ///
    /// Omitted fields never synthesize their implicit initial values. Descriptor
    /// references remain symbolic; the rule's authored occurrences remain intact.
    pub fn descriptor_specified_css(&self, kind: CssCounterStyleDescriptorKind) -> Result<String> {
        self.descriptor_specified_css_with_limits(kind, Limits::default())
    }

    /// Applies one cumulative budget to the requested descriptor's occurrences.
    ///
    /// Every retained occurrence of this kind and its value consume nodes,
    /// including suppressed earlier values. Other fields are outside this
    /// projection. An omitted field returns an empty string with zero node cost.
    pub fn descriptor_specified_css_with_limits(
        &self,
        kind: CssCounterStyleDescriptorKind,
        limits: Limits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        let mut effective = None;
        for value in self.descriptors().occurrences() {
            if slot(value).0 == kind.index() {
                node(&mut writer)?;
                if let Some(previous) = effective {
                    writer.without_output(|writer| descriptor(writer, previous))?;
                }
                effective = Some(value);
            }
        }
        if let Some(value) = effective {
            descriptor(&mut writer, value)?;
        }
        Ok(writer.css)
    }

    /// Serializes present effective descriptors without resolving or executing the style.
    ///
    /// Output follows the selected Counter Styles descriptor section order.
    /// Authored occurrences, omitted defaults, and source origins remain unchanged.
    pub fn to_specified_css(&self) -> Result<String> {
        self.to_specified_css_with_limits(Limits::default())
    }

    /// Applies one cumulative input, projection, and final CSS byte budget.
    ///
    /// The rule, name, every descriptor occurrence, typed aggregate, and leaf
    /// each charge one input and projection node. Range intervals and additive
    /// tuples are aggregates; image children use their owner's node accounting.
    /// Suppressed duplicates and optional components retain their full node cost
    /// while consuming no final bytes. Failure returns no partial CSS.
    pub fn to_specified_css_with_limits(&self, limits: Limits) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        node(writer)?;
        let mut effective = [None; 10];
        for (index, value) in self.descriptors().occurrences().enumerate() {
            node(writer)?;
            effective[slot(value).0] = Some((index, value));
        }
        for (index, value) in self.descriptors().occurrences().enumerate() {
            if effective[slot(value).0].is_some_and(|(last, _)| last != index) {
                writer.without_output(|writer| descriptor(writer, value))?;
            }
        }
        writer.append("@counter-style ")?;
        name(writer, self.name().as_str())?;
        writer.append(" {")?;
        for (_, value) in effective.into_iter().flatten() {
            writer.append(" ")?;
            writer.append(slot(value).1)?;
            writer.append(": ")?;
            descriptor(writer, value)?;
            writer.append(";")?;
        }
        writer.append(" }")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CssSpecifiedValueSerializationErrorKind as Kind;

    fn parsed(source: &str) -> CssCounterStyleRule {
        let report = crate::parse_sheet(source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let [crate::CssRule::CounterStyle(rule)] = report.syntax().rules() else {
            panic!("expected counter-style rule")
        };
        rule.clone()
    }

    #[test]
    fn existing_prefix_and_sibling_rules_share_node_and_byte_limits() {
        let rule = parsed("@counter-style x { symbols: a; }");
        let expected = "prefix:@counter-style x { symbols: a; }@counter-style x { symbols: a; }";
        let mut writer = SpecifiedRuleWriter::new(Limits::new(10, 10, expected.len()));
        writer.append("prefix:").unwrap();
        rule.append_to_rule_writer(&mut writer).unwrap();
        rule.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, expected);

        for (limits, kind) in [
            (Limits::new(9, 10, expected.len()), Kind::InputNodeLimit),
            (
                Limits::new(10, 9, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (Limits::new(10, 10, expected.len() - 1), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            writer.append("prefix:").unwrap();
            rule.append_to_rule_writer(&mut writer).unwrap();
            assert_eq!(
                rule.append_to_rule_writer(&mut writer).unwrap_err().kind(),
                kind
            );
        }
    }

    #[test]
    fn nested_suppression_restores_enclosing_mode_and_spends_no_bytes() {
        let rule = parsed(
            "@counter-style x { system: fixed 1; system: fixed +0001; negative: '-' ''; symbols: a; }",
        );
        // Fifteen nodes include the ineffective system and both omitted integers.
        let mut writer = SpecifiedRuleWriter::new(Limits::new(15, 15, 1));
        writer
            .without_output(|writer| {
                rule.append_to_rule_writer(writer)?;
                assert!(writer.context.output_suppressed());
                Ok(())
            })
            .unwrap();
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        writer.append("!").unwrap();
        assert_eq!(writer.css, "!");
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
    }

    #[test]
    fn failure_inside_nested_suppression_restores_mode_without_refunding_nodes() {
        let rule = parsed(
            "@counter-style x { system: fixed 1; system: fixed +0001; negative: '-' ''; symbols: a; }",
        );
        let mut writer = SpecifiedRuleWriter::new(Limits::new(14, 15, 1));
        let error = writer
            .without_output(|writer| rule.append_to_rule_writer(writer))
            .unwrap_err();
        assert_eq!(error.kind(), Kind::InputNodeLimit);
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        writer.append("!").unwrap();
        assert_eq!(writer.css, "!");
    }

    #[test]
    fn internally_constructed_typed_descriptors_use_the_same_semantic_providers() {
        use crate::syntax::{CssCounterStyleDescriptor, CssCounterStyleDescriptors};
        let position = parsed("@counter-style x { symbols: a; }").position();
        let rule = CssCounterStyleRule::new(
            crate::CssCounterStyleName::try_new("Constructed").unwrap(),
            CssCounterStyleDescriptors::from_occurrences(vec![
                CssCounterStyleDescriptor::System(crate::CssDescriptorOccurrence::new(
                    CssCounterStyleDescriptorValue::try_from_components(
                        CssCounterStyleDescriptorKind::System,
                        crate::CssComponentValues::try_new(vec![
                            crate::CssComponentValue::try_ident("extends").unwrap(),
                            crate::CssComponentValue::try_token(" ").unwrap(),
                            crate::CssComponentValue::try_ident("Unknown").unwrap(),
                        ])
                        .unwrap(),
                    )
                    .unwrap(),
                    position,
                )),
                CssCounterStyleDescriptor::Pad(crate::CssDescriptorOccurrence::new(
                    CssCounterStyleDescriptorValue::try_from_components(
                        CssCounterStyleDescriptorKind::Pad,
                        crate::CssComponentValues::try_new(vec![
                            crate::CssComponentValue::try_number("3").unwrap(),
                            crate::CssComponentValue::try_token(" ").unwrap(),
                            crate::CssComponentValue::try_token("\"_\"").unwrap(),
                        ])
                        .unwrap(),
                    )
                    .unwrap(),
                    position,
                )),
            ]),
            position,
        );
        let before = rule.clone();
        assert!(rule.parsed_name().is_none());
        let system = rule.descriptors().system().unwrap();
        assert!(system.parsed_name().is_none());
        assert!(system.parsed_value().is_none());
        assert!(system.value_components().is_none());
        assert_eq!(
            rule.to_specified_css().unwrap(),
            "@counter-style Constructed { system: extends Unknown; pad: 3 \"_\"; }"
        );
        assert_eq!(rule, before);
    }
}
