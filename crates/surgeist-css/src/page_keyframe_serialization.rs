//! Authored page and keyframe payload composition under the shared rule budget.
//!
//! The graph owner charges each logical rule before entering these bridges.
//! Page selectors and keyframe names cost one node each. Every keyframe block,
//! selector-list aggregate and selector costs one node; declaration lists use
//! the shared authored occurrence costs. The blocks slice adds no wrapper node.
//! Each of these costs applies to both input and projection work, including
//! suppressed output. Punctuation and endpoint spelling cost only bytes.
//! Calculated selectors also spend their shared percentage provider's input and
//! projection work without resetting the caller's cumulative limits.

use crate::{
    CssImportance, CssKeyframeBlock, CssKeyframeSelector, CssKeyframesName, CssKeyframesRule,
    CssPageRule, CssPageSelector, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind,
    declaration_serialization::append_authored_declaration,
    numeric_formatting::format_projected_number, specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

impl SpecifiedRuleWriter {
    /// Appends one page payload after the graph has charged its logical rule.
    pub(crate) fn page(&mut self, rule: &CssPageRule) -> Result<()> {
        self.append("@page")?;
        if let Some(selector) = rule.selector() {
            self.node()?;
            self.append(match selector {
                CssPageSelector::Left => " :left",
                CssPageSelector::Right => " :right",
                CssPageSelector::First => " :first",
            })?;
        }
        self.append(" { ")?;
        self.append_authored_declaration_list(rule.declarations())?;
        if !rule.declarations().is_empty() {
            self.append(" ")?;
        }
        self.append("}")
    }

    /// Appends one keyframes payload after the graph charges its logical rule.
    /// Endpoint equivalences follow Animations 1 §3, preserving every authored
    /// selector and block even when finite numeric output rounds to the same text.
    pub(crate) fn keyframes(&mut self, rule: &CssKeyframesRule) -> Result<()> {
        self.append("@keyframes ")?;
        self.node()?;
        match rule.name() {
            CssKeyframesName::Ident(name) => self.append_identifier(name.as_str())?,
            CssKeyframesName::String(name) => self.append_string(name.as_str())?,
        }
        self.append(" { ")?;
        for (index, block) in rule.blocks().iter().enumerate() {
            if index != 0 {
                self.append(" ")?;
            }
            self.keyframe_block(block)?;
        }
        if !rule.blocks().is_empty() {
            self.append(" ")?;
        }
        self.append("}")
    }

    fn keyframe_block(&mut self, block: &CssKeyframeBlock) -> Result<()> {
        self.node()?;
        self.node()?; // Selector-list aggregate.
        for (index, selector) in block.selectors().selectors().iter().enumerate() {
            if index != 0 {
                self.append(", ")?;
            }
            self.keyframe_selector(selector)?;
        }
        self.append(" { ")?;
        self.node()?; // Declaration-list aggregate, including an empty list.
        for (index, declaration) in block.declarations().iter().enumerate() {
            if index != 0 {
                self.append(" ")?;
            }
            append_authored_declaration(
                declaration.body(),
                declaration.value_components(),
                CssImportance::Normal,
                self,
            )?;
        }
        if !block.declarations().is_empty() {
            self.append(" ")?;
        }
        self.append("}")
    }

    fn keyframe_selector(&mut self, selector: &CssKeyframeSelector) -> Result<()> {
        self.node()?;
        match selector {
            CssKeyframeSelector::From => self.append("0%"),
            CssKeyframeSelector::To => self.append("100%"),
            CssKeyframeSelector::Percent(percent) => {
                if let Some(calculation) = percent.specified_calculation() {
                    return calculation.append_to_rule_writer(self);
                }
                if self.context.output_suppressed() {
                    return Ok(());
                }
                let limit = self
                    .context
                    .remaining_bytes()
                    .checked_sub(1)
                    .ok_or_else(|| {
                        CssSpecifiedValueSerializationError::new(
                            CssSpecifiedValueSerializationErrorKind::ByteLimit,
                        )
                    })?;
                // CSSOM 1 number/percentage output uses the common finite-bit
                // millionth formatter. The retained binary64 literal remains unchanged.
                let number = format_projected_number(
                    percent.literal_value().expect("checked literal selector"),
                    limit,
                )?;
                self.append(&number)?;
                self.append("%")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CssRule, CssSpecifiedValueSerializationLimits as Limits, parse_sheet};
    use CssSpecifiedValueSerializationErrorKind as Kind;

    fn append_rule(writer: &mut SpecifiedRuleWriter, rule: &CssRule) -> Result<()> {
        // The real graph owns this single logical-rule charge.
        writer.node()?;
        match rule {
            CssRule::Page(rule) => writer.page(rule),
            CssRule::Keyframes(rule) => writer.keyframes(rule),
            _ => panic!("page or keyframes fixture"),
        }
    }

    fn exact_budget(source: &str, expected: &str, nodes: usize) {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let before = report.clone();
        let [rule] = report.syntax().rules() else {
            panic!("one rule")
        };
        let mut writer = SpecifiedRuleWriter::new(Limits::new(nodes, nodes, expected.len()));
        append_rule(&mut writer, rule).unwrap();
        assert_eq!(writer.css, expected);
        for (limits, kind) in [
            (
                Limits::new(nodes - 1, nodes, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(nodes, nodes - 1, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(nodes, nodes, expected.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(append_rule(&mut writer, rule).unwrap_err().kind(), kind);
            assert_eq!(report, before);
        }
        assert_eq!(report, before);
    }

    #[test]
    fn empty_page_and_each_optional_selector_have_exact_independent_costs() {
        // Logical rule 1 + declaration-list aggregate 1; present selector 1.
        exact_budget("@page{}", "@page { }", 2);
        for (source, expected) in [
            ("@page :LEFT{}", "@page :left { }"),
            ("@page :right{}", "@page :right { }"),
            ("@page :first{}", "@page :first { }"),
        ] {
            exact_budget(source, expected, 3);
        }
    }

    #[test]
    fn page_duplicates_reuse_margin_keyword_and_importance_costs() {
        // Rule/selector/list 3 + two (occurrence/name/margin keyword) triples.
        exact_budget(
            "@page :left{margin-top:auto!important;margin-top:auto}",
            "@page :left { margin-top: auto !important; margin-top: auto; }",
            9,
        );
    }

    #[test]
    fn empty_keyframes_names_and_blocks_have_no_slice_wrapper_cost() {
        // Rule/name 2; each block/list/selector/declaration-list adds 4.
        exact_budget("@keyframes k{}", "@keyframes k { }", 2);
        exact_budget("@keyframes k{from{}}", "@keyframes k { 0% { } }", 6);
        exact_budget(
            "@keyframes k{from{}to{}}",
            "@keyframes k { 0% { } 100% { } }",
            10,
        );
        exact_budget("@keyframes \"日本\"{}", "@keyframes \"日本\" { }", 2);
    }

    #[test]
    fn rounded_selector_collisions_and_duplicate_occurrences_keep_order_and_work() {
        // Rule/name 2 + first block/list/three selectors/declaration list 6
        // + second block/list/selector/declaration list 4 + two declarations 6.
        exact_budget(
            "@keyframes k{0.0000001%,0.0000002%,from{}to{opacity:initial;opacity:inherit}}",
            "@keyframes k { 0%, 0%, 0% { } 100% { opacity: initial; opacity: inherit; } }",
            18,
        );
        exact_budget(
            "@keyframes k{0.0000011%{}0.0000012%{}}",
            "@keyframes k { 0.000001% { } 0.000001% { } }",
            10,
        );
    }

    #[test]
    fn suppressed_keyframes_visit_later_blocks_and_restore_nested_output_modes() {
        let report = parse_sheet("@keyframes k{0.0000011%{}to{opacity:initial}}");
        assert!(report.is_clean());
        let before = report.clone();
        let rule = &report.syntax().rules()[0];
        let mut writer = SpecifiedRuleWriter::new(Limits::new(13, 13, 2));
        writer.append("[").unwrap();
        writer
            .without_output(|writer| {
                writer.without_output(|writer| append_rule(writer, rule))?;
                assert!(writer.context.output_suppressed());
                Ok(())
            })
            .unwrap();
        assert!(!writer.context.output_suppressed());
        writer.append("]").unwrap();
        assert_eq!(writer.css, "[]");
        assert_eq!(writer.node().unwrap_err().kind(), Kind::InputNodeLimit);
        assert_eq!(report, before);

        for (limits, kind) in [
            (Limits::new(12, 13, 1), Kind::InputNodeLimit),
            (Limits::new(13, 12, 1), Kind::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            let error = writer
                .without_output(|writer| writer.without_output(|writer| append_rule(writer, rule)))
                .unwrap_err();
            assert_eq!(error.kind(), kind);
            assert!(!writer.context.output_suppressed());
            writer.append("x").unwrap();
            assert_eq!(writer.css, "x");
            assert_eq!(report, before);
        }
    }

    #[test]
    fn suppressed_calculated_selectors_share_work_and_restore_output_after_failure() {
        let report = parse_sheet("@keyframes k{calc(120%){}calc(-10%){} }");
        assert!(report.is_clean());
        let before = report.clone();
        let rule = &report.syntax().rules()[0];
        // Rule/name 2 + two block/list/selector/declaration-list groups of 4.
        // Each calculation visits its function and leaf, projecting one leaf.
        let mut writer = SpecifiedRuleWriter::new(Limits::new(14, 12, 2));
        writer.append("[").unwrap();
        writer
            .without_output(|writer| {
                writer.without_output(|writer| append_rule(writer, rule))?;
                assert!(writer.context.output_suppressed());
                Ok(())
            })
            .unwrap();
        assert!(!writer.context.output_suppressed());
        writer.append("]").unwrap();
        assert_eq!(writer.css, "[]");
        assert_eq!(writer.node().unwrap_err().kind(), Kind::InputNodeLimit);

        for (limits, kind) in [
            (Limits::new(13, 12, 1), Kind::InputNodeLimit),
            (Limits::new(14, 11, 1), Kind::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            let error = writer
                .without_output(|writer| writer.without_output(|writer| append_rule(writer, rule)))
                .unwrap_err();
            assert_eq!(error.kind(), kind);
            assert!(!writer.context.output_suppressed());
            writer.append("x").unwrap();
            assert_eq!(writer.css, "x");
            assert_eq!(report, before);
        }
        assert_eq!(report, before);
    }
}
