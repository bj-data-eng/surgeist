//! Borrowed authored rule graph traversal under one monotonic context.
//!
//! Each logical rule charges one input and projection node: published leaf
//! writers retain their existing charge, and new graph arms charge before
//! scheduling children. Rust enum carriers and child slices are transparent.
//! Slice continuations retain only the remaining children, so graph width does
//! not allocate a pending event for every sibling before checking work limits.

use crate::cssom_rule_serialization::{
    CssRuleCssomFormat, CssRuleCssomKind, CssRuleCssomSerializationError, RuleCssomSource,
};
use crate::edited_rule::{
    CssEditedGroupPreludeRef, CssEditedRuleView, EditedRuleKind, EditedStyleSelectorsRef,
};
use crate::specified_rule_serialization::{SpecifiedRuleSerializationSource, SpecifiedRuleWriter};
use crate::{
    CssDeclarationList, CssLayerName, CssLayerNameList, CssMediaQueryList, CssRule, CssScopeRule,
    CssScopedRule, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
};

type Result<T> = std::result::Result<T, RuleCssomSource>;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Format {
    Compact,
    Cssom,
}

struct GraphFailure {
    source: RuleCssomSource,
    path: Vec<usize>,
    keyframe_block_index: Option<usize>,
}
type ValueResult<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

#[derive(Clone, Copy)]
enum EditedChildStart {
    NewLine,
    ExistingLine,
}

#[derive(Clone, Copy)]
enum SupportsChildren<'a> {
    Ordinary(&'a [CssRule]),
    Scoped(&'a [CssScopedRule]),
    Edited(&'a [CssEditedRuleView<'a>]),
}

impl<'a> SupportsChildren<'a> {
    fn next(self, index: usize) -> Option<(Event<'a>, Self)> {
        match self {
            Self::Ordinary(rules) => rules
                .split_first()
                .map(|(first, rest)| (Event::Ordinary(first, Some(index)), Self::Ordinary(rest))),
            Self::Scoped(rules) => rules
                .split_first()
                .map(|(first, rest)| (Event::Scoped(first, Some(index)), Self::Scoped(rest))),
            Self::Edited(rules) => rules
                .split_first()
                .map(|(first, rest)| (Event::Edited(*first, Some(index)), Self::Edited(rest))),
        }
    }
}

enum Event<'a> {
    Edited(CssEditedRuleView<'a>, Option<usize>),
    EditedChildren(&'a [CssEditedRuleView<'a>], usize, bool, EditedChildStart),
    Ordinary(&'a CssRule, Option<usize>),
    Scoped(&'a CssScopedRule, Option<usize>),
    OrdinaryChildren(&'a [CssRule], usize, bool),
    ScopedChildren(&'a [CssScopedRule], usize, bool),
    SupportsChildren(SupportsChildren<'a>, usize),
    SupportsChildText(usize),
    Declarations(&'a CssDeclarationList),
    Text(&'static str),
    EndRule(bool),
    StyleAncestor(bool),
}

fn push<'a>(work: &mut Vec<Event<'a>>, event: Event<'a>) -> ValueResult<()> {
    work.try_reserve(1).map_err(|_| {
        CssSpecifiedValueSerializationError::new(
            CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
        )
    })?;
    work.push(event);
    Ok(())
}

impl SpecifiedRuleWriter {
    /// One borrowed loop visits both rule enums without constructing a second
    /// graph, recursively walking groups, or resetting any provider budget.
    pub(crate) fn append_rule_graph(
        &mut self,
        rule: &CssRule,
    ) -> std::result::Result<(), SpecifiedRuleSerializationSource> {
        self.append_graph(
            Event::Ordinary(rule, None),
            Format::Compact,
            Vec::new(),
            None,
        )
        .map_err(|error| match error.source {
            RuleCssomSource::Provider(source) => source,
            _ => unreachable!("compact traversal uses only compact providers"),
        })
    }

    pub(crate) fn append_cssom_rule_graph(
        &mut self,
        rule: &CssRule,
        path: Vec<usize>,
        namespaces: Option<&crate::CssNamespaceContext>,
    ) -> std::result::Result<(), CssRuleCssomSerializationError> {
        self.append_graph(Event::Ordinary(rule, None), Format::Cssom, path, namespaces)
            .map_err(|failure| {
                CssRuleCssomSerializationError::new(
                    failure.source,
                    failure.path,
                    failure.keyframe_block_index,
                )
            })
    }

    pub(crate) fn append_edited_rule_graph(
        &mut self,
        rule: CssEditedRuleView<'_>,
        cssom: bool,
    ) -> std::result::Result<(), CssRuleCssomSerializationError> {
        self.append_graph(
            Event::Edited(rule, None),
            if cssom {
                Format::Cssom
            } else {
                Format::Compact
            },
            Vec::new(),
            None,
        )
        .map_err(|failure| {
            CssRuleCssomSerializationError::new(
                failure.source,
                failure.path,
                failure.keyframe_block_index,
            )
        })
    }

    fn append_graph(
        &mut self,
        first: Event<'_>,
        format: Format,
        mut path: Vec<usize>,
        namespaces: Option<&crate::CssNamespaceContext>,
    ) -> std::result::Result<(), GraphFailure> {
        let mut work = Vec::new();
        let mut keyframe_block_index = None;
        let mut margin_rule_index = None;
        let mut style_ancestor = false;
        let result = (|| -> Result<()> {
            push(&mut work, first)?;
            while let Some(event) = work.pop() {
                match event {
                    Event::SupportsChildren(children, index) => {
                        if let Some((first, rest)) = children.next(index) {
                            push(&mut work, Event::SupportsChildren(rest, index + 1))?;
                            push(&mut work, Event::SupportsChildText(self.css.len()))?;
                            push(&mut work, first)?;
                        }
                    }
                    Event::SupportsChildText(start) => {
                        let end = self.css.len();
                        if end != start {
                            // The selected grouping helper prefixes only nonempty
                            // complete child text. Visit once, then relocate the
                            // checked prefix in the same output; append has already
                            // charged its bytes and reserved the required capacity.
                            self.append("\n  ")?;
                            self.css.truncate(end);
                            self.css.insert_str(start, "\n  ");
                        }
                    }
                    Event::EditedChildren(rules, index, separator, start) => {
                        if let Some((first, rest)) = rules.split_first() {
                            if let EditedRuleKind::NestedDeclarations(block) = first.kind
                                && block.entries().is_empty()
                            {
                                // Visit the omitted child with its normal provider
                                // work and exact error path, without a separator.
                                push(
                                    &mut work,
                                    Event::EditedChildren(rest, index + 1, separator, start),
                                )?;
                                push(&mut work, Event::Edited(*first, Some(index)))?;
                                continue;
                            }
                            push(
                                &mut work,
                                Event::EditedChildren(
                                    rest,
                                    index + 1,
                                    true,
                                    EditedChildStart::NewLine,
                                ),
                            )?;
                            if format == Format::Cssom {
                                if matches!(start, EditedChildStart::NewLine) {
                                    self.append("\n")?;
                                }
                                self.append("  ")?;
                            } else if separator {
                                self.append(" ")?;
                            }
                            push(&mut work, Event::Edited(*first, Some(index)))?;
                        }
                    }
                    Event::Edited(view, index) => {
                        if let EditedRuleKind::Parsed(rule) = view.kind {
                            push(&mut work, Event::Ordinary(rule, index))?;
                            continue;
                        }
                        if let EditedRuleKind::ParsedScoped(rule) = view.kind {
                            push(&mut work, Event::Scoped(rule, index))?;
                            continue;
                        }
                        if format == Format::Cssom
                            && let Some(index) = index
                        {
                            path.try_reserve(1).map_err(|_| {
                                CssSpecifiedValueSerializationError::new(
                                    CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                                )
                            })?;
                            path.push(index);
                        }
                        if format == Format::Cssom {
                            push(&mut work, Event::EndRule(index.is_some()))?;
                        }
                        self.node()?;
                        match view.kind {
                            EditedRuleKind::Parsed(_) | EditedRuleKind::ParsedScoped(_) => {
                                unreachable!("parsed event forwarded")
                            }
                            EditedRuleKind::Page(page) => {
                                self.append_selected_page(page, &mut margin_rule_index)?;
                            }
                            EditedRuleKind::Keyframes(keyframes) => {
                                self.append_selected_keyframes(
                                    keyframes,
                                    &mut keyframe_block_index,
                                )?;
                            }
                            EditedRuleKind::Import(import) => {
                                if format == Format::Cssom {
                                    import
                                        .rule()
                                        .append_cssom_with_media(self, Some(import.media()))?;
                                } else {
                                    import.rule().append_specified_with_media(
                                        &mut self.context,
                                        &mut self.css,
                                        Some(import.media()),
                                    )?;
                                }
                            }
                            EditedRuleKind::NestedDeclarations(block) => {
                                block.append_cssom(self)?
                            }
                            EditedRuleKind::Style(selectors, declarations, children) => {
                                match selectors {
                                    EditedStyleSelectorsRef::Ordinary(selectors) => {
                                        if format == Format::Cssom {
                                            self.cssom_style_selectors(
                                                selectors,
                                                style_ancestor,
                                                namespaces,
                                            )?;
                                        } else {
                                            self.style_selectors(selectors)?;
                                        }
                                    }
                                    EditedStyleSelectorsRef::Scoped(selectors) => {
                                        if format == Format::Cssom {
                                            self.cssom_scoped_style_selectors(
                                                selectors,
                                                style_ancestor,
                                                namespaces,
                                            )?;
                                        } else {
                                            self.scoped_style_selectors(selectors)?;
                                        }
                                    }
                                }
                                push(&mut work, Event::StyleAncestor(style_ancestor))?;
                                style_ancestor = true;
                                self.style_payload_block(
                                    &mut work,
                                    declarations.entries().is_empty(),
                                    (!children.is_empty()).then_some(Event::EditedChildren(
                                        children,
                                        0,
                                        true,
                                        EditedChildStart::NewLine,
                                    )),
                                    format,
                                    true,
                                    |writer| declarations.append_cssom(writer).map_err(Into::into),
                                )?;
                            }
                            EditedRuleKind::Group(prelude, children) => {
                                match prelude {
                                    CssEditedGroupPreludeRef::Media(list) => {
                                        self.media_prelude(list, format)?
                                    }
                                    CssEditedGroupPreludeRef::Supports(condition) => {
                                        self.supports_prelude(condition)?;
                                    }
                                    CssEditedGroupPreludeRef::Container(prelude) => {
                                        if format == Format::Cssom {
                                            return Err(RuleCssomSource::FormatUnavailable(
                                                CssRuleCssomFormat::Container,
                                            ));
                                        }
                                        self.append("@container ")?;
                                        prelude
                                            .append_specified(&mut self.context, &mut self.css)?;
                                    }
                                    CssEditedGroupPreludeRef::Layer(name) => {
                                        if format == Format::Cssom {
                                            return Err(RuleCssomSource::FormatUnavailable(
                                                CssRuleCssomFormat::LayerBlock,
                                            ));
                                        }
                                        self.layer_prelude(name)?;
                                    }
                                }
                                if format == Format::Cssom
                                    && matches!(prelude, CssEditedGroupPreludeRef::Supports(_))
                                {
                                    self.supports_block(
                                        &mut work,
                                        SupportsChildren::Edited(children),
                                    )?;
                                } else if format == Format::Cssom {
                                    // Media requires an initial and final LF even when
                                    // its retained children all serialize to empty text.
                                    self.append(" {\n")?;
                                    push(&mut work, Event::Text("\n}"))?;
                                    if !children.is_empty() {
                                        push(
                                            &mut work,
                                            Event::EditedChildren(
                                                children,
                                                0,
                                                false,
                                                EditedChildStart::ExistingLine,
                                            ),
                                        )?;
                                    }
                                } else {
                                    self.append(" {")?;
                                    push(&mut work, Event::Text(" }"))?;
                                    if !children.is_empty() {
                                        push(
                                            &mut work,
                                            Event::EditedChildren(
                                                children,
                                                0,
                                                true,
                                                EditedChildStart::NewLine,
                                            ),
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                    Event::StyleAncestor(value) => style_ancestor = value,
                    Event::Text(text) => self.append(text)?,
                    Event::EndRule(pop) => {
                        if pop {
                            path.pop();
                        }
                    }
                    Event::Declarations(declarations) => {
                        if format == Format::Cssom {
                            self.append_cssom_declaration_list(declarations)?;
                        } else {
                            self.append_authored_declaration_list(declarations)?;
                        }
                    }
                    Event::OrdinaryChildren(rules, index, separator) => {
                        if let Some((first, rest)) = rules.split_first() {
                            if format == Format::Cssom {
                                self.append("\n  ")?;
                            } else if separator {
                                self.append(" ")?;
                            }
                            push(&mut work, Event::OrdinaryChildren(rest, index + 1, true))?;
                            push(&mut work, Event::Ordinary(first, Some(index)))?;
                        }
                    }
                    Event::ScopedChildren(rules, index, separator) => {
                        if let Some((first, rest)) = rules.split_first() {
                            if format == Format::Cssom {
                                self.append("\n  ")?;
                            } else if separator {
                                self.append(" ")?;
                            }
                            push(&mut work, Event::ScopedChildren(rest, index + 1, true))?;
                            push(&mut work, Event::Scoped(first, Some(index)))?;
                        }
                    }
                    Event::Ordinary(rule, index) => {
                        if format == Format::Cssom
                            && let Some(index) = index
                        {
                            path.try_reserve(1).map_err(|_| {
                                CssSpecifiedValueSerializationError::new(
                                    CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                                )
                            })?;
                            path.push(index);
                        }
                        if format == Format::Cssom {
                            push(&mut work, Event::EndRule(index.is_some()))?;
                        }
                        match rule {
                            CssRule::Namespace(rule) => self.namespace(rule)?,
                            CssRule::CounterStyle(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::CounterStyle,
                                    ));
                                }
                                rule.append_to_rule_writer(self)?
                            }
                            CssRule::FontFace(rule) => self.font_face(rule)?,
                            CssRule::FontFeatureValues(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::FontFeatureValues,
                                    ));
                                }
                                self.font_features(rule)?
                            }
                            CssRule::FontPaletteValues(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::FontPaletteValues,
                                    ));
                                }
                                self.palette(rule)?
                            }
                            CssRule::ColorProfile(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::ColorProfile,
                                    ));
                                }
                                self.color_profile(rule)?
                            }
                            CssRule::SupportsCondition(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::SupportsCondition,
                                    ));
                                }
                                self.named_supports_rule(rule)?
                            }
                            CssRule::NestedDeclarations(rule) => {
                                self.node()?;
                                push(&mut work, Event::Declarations(rule.declarations()))?;
                            }
                            CssRule::Page(rule) => {
                                self.node()?;
                                if format == Format::Cssom {
                                    self.append_page_cssom(rule, &mut margin_rule_index)?;
                                } else {
                                    self.page(rule)?;
                                }
                            }
                            CssRule::Keyframes(rule) => {
                                self.node()?;
                                if format == Format::Cssom {
                                    self.keyframes_cssom(rule, &mut keyframe_block_index)?;
                                } else {
                                    self.keyframes(rule)?;
                                }
                            }
                            CssRule::CustomMedia(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::CustomMedia,
                                    ));
                                }
                                self.node()?;
                                rule.append_specified(&mut self.context, &mut self.css)?;
                            }
                            CssRule::Import(rule) => {
                                self.node()?;
                                if format == Format::Cssom {
                                    rule.append_cssom(self)?;
                                } else {
                                    rule.append_specified(&mut self.context, &mut self.css)?;
                                }
                            }
                            CssRule::LayerStatement(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::LayerStatement,
                                    ));
                                }
                                self.node()?;
                                self.append("@layer ")?;
                                self.layer_names(rule.names())?;
                                self.append(";")?;
                            }
                            CssRule::LayerBlock(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::LayerBlock,
                                    ));
                                }
                                self.node()?;
                                self.layer_prelude(rule.name())?;
                                self.open_block(
                                    &mut work,
                                    format,
                                    (!rule.rules().is_empty()).then_some(Event::OrdinaryChildren(
                                        rule.rules(),
                                        0,
                                        false,
                                    )),
                                )?;
                            }
                            CssRule::Style(rule) => {
                                self.node()?;
                                if format == Format::Cssom {
                                    self.cssom_style_selectors(
                                        rule.selectors(),
                                        style_ancestor,
                                        namespaces,
                                    )?;
                                } else {
                                    self.style_selectors(rule.selectors())?;
                                }
                                push(&mut work, Event::StyleAncestor(style_ancestor))?;
                                style_ancestor = true;
                                self.style_block(
                                    &mut work,
                                    rule.declarations(),
                                    rule.rules(),
                                    format,
                                )?;
                            }
                            CssRule::Media(rule) => {
                                self.node()?;
                                self.media_prelude(rule.query(), format)?;
                                self.open_block(
                                    &mut work,
                                    format,
                                    (!rule.rules().is_empty()).then_some(Event::OrdinaryChildren(
                                        rule.rules(),
                                        0,
                                        false,
                                    )),
                                )?;
                            }
                            CssRule::Supports(rule) => {
                                self.node()?;
                                self.supports_prelude(rule.condition())?;
                                if format == Format::Cssom {
                                    self.supports_block(
                                        &mut work,
                                        SupportsChildren::Ordinary(rule.rules()),
                                    )?;
                                } else {
                                    self.open_block(
                                        &mut work,
                                        format,
                                        (!rule.rules().is_empty()).then_some(
                                            Event::OrdinaryChildren(rule.rules(), 0, false),
                                        ),
                                    )?;
                                }
                            }
                            CssRule::Container(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::Container,
                                    ));
                                }
                                self.node()?;
                                self.append("@container ")?;
                                rule.prelude()
                                    .append_specified(&mut self.context, &mut self.css)?;
                                self.open_block(
                                    &mut work,
                                    format,
                                    (!rule.rules().is_empty()).then_some(Event::OrdinaryChildren(
                                        rule.rules(),
                                        0,
                                        false,
                                    )),
                                )?;
                            }
                            CssRule::When(rule) => {
                                self.node()?;
                                if format == Format::Cssom {
                                    return Err(RuleCssomSource::SourceUndefined(
                                        CssRuleCssomKind::When,
                                    ));
                                }
                                self.append("@when ")?;
                                rule.condition()
                                    .append_specified(&mut self.context, &mut self.css)?;
                                self.open_block(
                                    &mut work,
                                    format,
                                    (!rule.rules().is_empty()).then_some(Event::OrdinaryChildren(
                                        rule.rules(),
                                        0,
                                        false,
                                    )),
                                )?;
                            }
                            CssRule::Else(rule) => {
                                self.node()?;
                                if format == Format::Cssom {
                                    return Err(RuleCssomSource::SourceUndefined(
                                        CssRuleCssomKind::Else,
                                    ));
                                }
                                self.append("@else")?;
                                if let Some(condition) = rule.condition() {
                                    self.append(" ")?;
                                    condition.append_specified(&mut self.context, &mut self.css)?;
                                }
                                self.open_block(
                                    &mut work,
                                    format,
                                    (!rule.rules().is_empty()).then_some(Event::OrdinaryChildren(
                                        rule.rules(),
                                        0,
                                        false,
                                    )),
                                )?;
                            }
                            CssRule::Scope(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::Scope,
                                    ));
                                }
                                self.scope_block(&mut work, rule, format)?
                            }
                        }
                    }
                    Event::Scoped(rule, index) => {
                        if format == Format::Cssom
                            && let Some(index) = index
                        {
                            path.try_reserve(1).map_err(|_| {
                                CssSpecifiedValueSerializationError::new(
                                    CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                                )
                            })?;
                            path.push(index);
                        }
                        if format == Format::Cssom {
                            push(&mut work, Event::EndRule(index.is_some()))?;
                        }
                        match rule {
                            CssScopedRule::CounterStyle(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::CounterStyle,
                                    ));
                                }
                                rule.append_to_rule_writer(self)?
                            }
                            CssScopedRule::FontFace(rule) => self.font_face(rule)?,
                            CssScopedRule::FontFeatureValues(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::FontFeatureValues,
                                    ));
                                }
                                self.font_features(rule)?
                            }
                            CssScopedRule::FontPaletteValues(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::FontPaletteValues,
                                    ));
                                }
                                self.palette(rule)?
                            }
                            CssScopedRule::ColorProfile(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::ColorProfile,
                                    ));
                                }
                                self.color_profile(rule)?
                            }
                            CssScopedRule::SupportsCondition(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::SupportsCondition,
                                    ));
                                }
                                self.named_supports_rule(rule)?
                            }
                            CssScopedRule::NestedDeclarations(rule) => {
                                self.node()?;
                                push(&mut work, Event::Declarations(rule.declarations()))?;
                            }
                            CssScopedRule::Page(rule) => {
                                self.node()?;
                                if format == Format::Cssom {
                                    self.append_page_cssom(rule, &mut margin_rule_index)?;
                                } else {
                                    self.page(rule)?;
                                }
                            }
                            CssScopedRule::Keyframes(rule) => {
                                self.node()?;
                                if format == Format::Cssom {
                                    self.keyframes_cssom(rule, &mut keyframe_block_index)?;
                                } else {
                                    self.keyframes(rule)?;
                                }
                            }
                            CssScopedRule::CustomMedia(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::CustomMedia,
                                    ));
                                }
                                self.node()?;
                                rule.append_specified(&mut self.context, &mut self.css)?;
                            }
                            CssScopedRule::LayerStatement(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::LayerStatement,
                                    ));
                                }
                                self.node()?;
                                self.append("@layer ")?;
                                self.layer_names(rule.names())?;
                                self.append(";")?;
                            }
                            CssScopedRule::LayerBlock(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::LayerBlock,
                                    ));
                                }
                                self.node()?;
                                self.layer_prelude(rule.name())?;
                                self.open_block(
                                    &mut work,
                                    format,
                                    (!rule.rules().rules().is_empty()).then_some(
                                        Event::ScopedChildren(rule.rules().rules(), 0, false),
                                    ),
                                )?;
                            }
                            CssScopedRule::Style(rule) => {
                                self.node()?;
                                if format == Format::Cssom {
                                    self.cssom_scoped_style_selectors(
                                        rule.selectors(),
                                        style_ancestor,
                                        namespaces,
                                    )?;
                                } else {
                                    self.scoped_style_selectors(rule.selectors())?;
                                }
                                // Scoped style children have ordinary nesting semantics.
                                push(&mut work, Event::StyleAncestor(style_ancestor))?;
                                style_ancestor = true;
                                self.style_block(
                                    &mut work,
                                    rule.declarations(),
                                    rule.rules(),
                                    format,
                                )?;
                            }
                            CssScopedRule::Media(rule) => {
                                self.node()?;
                                self.media_prelude(rule.query(), format)?;
                                self.open_block(
                                    &mut work,
                                    format,
                                    (!rule.rules().rules().is_empty()).then_some(
                                        Event::ScopedChildren(rule.rules().rules(), 0, false),
                                    ),
                                )?;
                            }
                            CssScopedRule::Supports(rule) => {
                                self.node()?;
                                self.supports_prelude(rule.condition())?;
                                if format == Format::Cssom {
                                    self.supports_block(
                                        &mut work,
                                        SupportsChildren::Scoped(rule.rules().rules()),
                                    )?;
                                } else {
                                    self.open_block(
                                        &mut work,
                                        format,
                                        (!rule.rules().rules().is_empty()).then_some(
                                            Event::ScopedChildren(rule.rules().rules(), 0, false),
                                        ),
                                    )?;
                                }
                            }
                            CssScopedRule::Container(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::Container,
                                    ));
                                }
                                self.node()?;
                                self.append("@container ")?;
                                rule.prelude()
                                    .append_specified(&mut self.context, &mut self.css)?;
                                self.open_block(
                                    &mut work,
                                    format,
                                    (!rule.rules().rules().is_empty()).then_some(
                                        Event::ScopedChildren(rule.rules().rules(), 0, false),
                                    ),
                                )?;
                            }
                            CssScopedRule::When(rule) => {
                                self.node()?;
                                if format == Format::Cssom {
                                    return Err(RuleCssomSource::SourceUndefined(
                                        CssRuleCssomKind::When,
                                    ));
                                }
                                self.append("@when ")?;
                                rule.condition()
                                    .append_specified(&mut self.context, &mut self.css)?;
                                self.open_block(
                                    &mut work,
                                    format,
                                    (!rule.rules().rules().is_empty()).then_some(
                                        Event::ScopedChildren(rule.rules().rules(), 0, false),
                                    ),
                                )?;
                            }
                            CssScopedRule::Else(rule) => {
                                self.node()?;
                                if format == Format::Cssom {
                                    return Err(RuleCssomSource::SourceUndefined(
                                        CssRuleCssomKind::Else,
                                    ));
                                }
                                self.append("@else")?;
                                if let Some(condition) = rule.condition() {
                                    self.append(" ")?;
                                    condition.append_specified(&mut self.context, &mut self.css)?;
                                }
                                self.open_block(
                                    &mut work,
                                    format,
                                    (!rule.rules().rules().is_empty()).then_some(
                                        Event::ScopedChildren(rule.rules().rules(), 0, false),
                                    ),
                                )?;
                            }
                            CssScopedRule::Scope(rule) => {
                                if format == Format::Cssom {
                                    self.node()?;
                                    return Err(RuleCssomSource::FormatUnavailable(
                                        CssRuleCssomFormat::Scope,
                                    ));
                                }
                                self.scope_block(&mut work, rule, format)?
                            }
                        }
                    }
                }
            }
            Ok(())
        })();
        result.map_err(|mut source| {
            if let Some(index) = margin_rule_index {
                if path.try_reserve(1).is_err() {
                    source = crate::CssSpecifiedValueSerializationError::new(
                        crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    )
                    .into();
                } else {
                    path.push(index);
                }
            }
            GraphFailure {
                source,
                path,
                keyframe_block_index,
            }
        })
    }

    fn supports_prelude(&mut self, condition: &crate::CssSupportsCondition) -> Result<()> {
        self.append("@supports ")?;
        condition
            .append_specified(&mut self.context, &mut self.css)
            .map_err(Into::into)
    }

    fn supports_block<'a>(
        &mut self,
        work: &mut Vec<Event<'a>>,
        children: SupportsChildren<'a>,
    ) -> ValueResult<()> {
        self.append(" {")?;
        push(work, Event::Text("\n}"))?;
        push(work, Event::SupportsChildren(children, 0))
    }

    fn open_block<'a>(
        &mut self,
        work: &mut Vec<Event<'a>>,
        format: Format,
        child: Option<Event<'a>>,
    ) -> ValueResult<()> {
        self.append(" {")?;
        push(
            work,
            Event::Text(if format == Format::Cssom { "\n}" } else { " }" }),
        )?;
        if let Some(child) = child {
            if format == Format::Compact {
                self.append(" ")?;
            }
            push(work, child)?;
        } else if format == Format::Cssom {
            self.append("\n")?;
        }
        Ok(())
    }

    /// The leading declaration list charges its shared aggregate even when
    /// empty. Later runs remain logical rules at their original child position.
    fn style_block<'a>(
        &mut self,
        work: &mut Vec<Event<'a>>,
        declarations: &'a CssDeclarationList,
        children: &'a [CssRule],
        format: Format,
    ) -> Result<()> {
        self.style_payload_block(
            work,
            declarations.is_empty(),
            (!children.is_empty()).then_some(Event::OrdinaryChildren(
                children,
                0,
                !declarations.is_empty(),
            )),
            format,
            false,
            |writer| {
                if format == Format::Cssom {
                    writer
                        .append_cssom_declaration_list(declarations)
                        .map_err(Into::into)
                } else {
                    writer
                        .append_authored_declaration_list(declarations)
                        .map_err(Into::into)
                }
            },
        )
    }

    fn style_payload_block<'a>(
        &mut self,
        work: &mut Vec<Event<'a>>,
        empty: bool,
        child: Option<Event<'a>>,
        format: Format,
        child_starts_compact_body: bool,
        append_declarations: impl FnOnce(&mut Self) -> Result<()>,
    ) -> Result<()> {
        self.append(" {")?;
        if format == Format::Cssom {
            if !empty {
                self.append(if child.is_some() { "\n  " } else { " " })?;
            }
            append_declarations(self)?;
            if let Some(child) = child {
                push(work, Event::Text("\n}"))?;
                push(work, child)?;
            } else {
                self.append(" }")?;
            }
        } else {
            if !empty || (child.is_some() && !child_starts_compact_body) {
                self.append(" ")?;
            }
            append_declarations(self)?;
            push(work, Event::Text(" }"))?;
            if let Some(child) = child {
                push(work, child)?;
            }
        }
        Ok(())
    }

    fn scope_block<'a>(
        &mut self,
        work: &mut Vec<Event<'a>>,
        rule: &'a CssScopeRule,
        format: Format,
    ) -> ValueResult<()> {
        self.node()?;
        self.append("@scope")?;
        if let Some(root) = rule.root() {
            self.append(" (")?;
            self.scope_selectors(root)?;
            self.append(")")?;
        }
        if let Some(limit) = rule.limit() {
            self.append(" to (")?;
            self.scope_selectors(limit)?;
            self.append(")")?;
        }
        self.open_block(
            work,
            format,
            (!rule.rules().rules().is_empty()).then_some(Event::ScopedChildren(
                rule.rules().rules(),
                0,
                false,
            )),
        )
    }

    fn media_prelude(&mut self, list: &CssMediaQueryList, format: Format) -> Result<()> {
        self.append("@media")?;
        if format == Format::Cssom || !list.queries().is_empty() {
            self.append(" ")?;
        }
        let captured = list.capture_cssom(&mut self.context)?;
        self.append(captured.as_css())?;
        Ok(())
    }

    fn layer_prelude(&mut self, name: Option<&CssLayerName>) -> ValueResult<()> {
        self.append("@layer")?;
        if let Some(name) = name {
            self.append(" ")?;
            self.layer_name(name)?;
        }
        Ok(())
    }

    /// A name charges one aggregate and each retained identifier component
    /// once. The component slice adds no extra charge.
    fn layer_name(&mut self, name: &CssLayerName) -> ValueResult<()> {
        self.node()?;
        for (index, component) in name.components().iter().enumerate() {
            if index != 0 {
                self.append(".")?;
            }
            self.node()?;
            self.append_identifier(component)?;
        }
        Ok(())
    }

    /// A layer-name list adds one aggregate before its ordered names.
    fn layer_names(&mut self, names: &CssLayerNameList) -> ValueResult<()> {
        self.node()?;
        for (index, name) in names.names().iter().enumerate() {
            if index != 0 {
                self.append(", ")?;
            }
            self.layer_name(name)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CssRule, CssSpecifiedRuleSerializationErrorKind as RuleKind,
        CssSpecifiedValueSerializationErrorKind as Kind,
        CssSpecifiedValueSerializationLimits as Limits, parse_sheet,
    };

    fn exact_nodes(source: &str, expected: &str, nodes: usize) {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let before = report.clone();
        assert_eq!(
            report
                .syntax()
                .to_specified_css_with_limits(Limits::new(nodes, nodes, expected.len()))
                .unwrap(),
            expected,
        );
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
            let error = report
                .syntax()
                .to_specified_css_with_limits(limits)
                .unwrap_err();
            assert_eq!(error.kind(), RuleKind::Resource(kind));
            assert_eq!(error.rule_index(), Some(0));
            assert_eq!(report, before);
        }
        assert_eq!(report, before);
    }

    #[test]
    fn empty_media_lists_keep_one_prelude_space_and_their_aggregate_cost() {
        // Sheet, media rule, and empty query-list aggregate each cost one.
        exact_nodes("@media{}", "@media { }", 3);
        // A scope adds one logical rule, without an enum/list carrier charge.
        exact_nodes("@scope{@media{}}", "@scope { @media { } }", 4);
    }

    #[test]
    fn layer_names_charge_lists_names_and_identifier_components_once() {
        // Sheet/rule 2 + name aggregate 1 + identifier component 1.
        exact_nodes("@layer base{}", "@layer base { }", 4);
        // Sheet/rule/list 3 + two names 2 + three components 3.
        exact_nodes("@layer a.b,c;", "@layer a.b, c;", 8);
        // Anonymous blocks have no name or name-list aggregate.
        exact_nodes("@layer{}", "@layer { }", 2);
    }

    #[test]
    fn recovered_charset_only_sheet_charges_its_empty_retained_aggregate() {
        let report = parse_sheet("@charset \"windows-1252\";");
        assert!(!report.is_clean());
        let [diagnostic] = report.diagnostics() else {
            panic!("one unknown charset")
        };
        assert_eq!(
            diagnostic.error().code(),
            crate::CssErrorCode::UnknownAtRule
        );
        assert_eq!(diagnostic.action(), crate::CssRecoveryAction::DropAtRule);
        let before = report.clone();
        assert_eq!(
            report
                .syntax()
                .to_specified_css_with_limits(Limits::new(1, 1, 0))
                .unwrap(),
            "",
        );
        for (limits, kind) in [
            (Limits::new(0, 1, 0), Kind::InputNodeLimit),
            (Limits::new(1, 0, 0), Kind::ProjectionNodeLimit),
        ] {
            let error = report
                .syntax()
                .to_specified_css_with_limits(limits)
                .unwrap_err();
            assert_eq!(error.kind(), RuleKind::Resource(kind));
            assert_eq!(error.rule_index(), None);
        }
        assert_eq!(report, before);
        assert!(report.syntax().rules().is_empty());
    }

    #[test]
    fn final_group_delimiters_report_the_enclosing_rule_without_returning_partial_text() {
        let report = parse_sheet("@namespace n 'u';@scope{@layer{@font-face{}}}");
        assert!(report.is_clean());
        let before = report.clone();
        let expected = "@namespace n url(\"u\");\n@scope { @layer { @font-face { } } }";
        let error = report
            .syntax()
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len() - 1))
            .unwrap_err();
        assert_eq!(error.kind(), RuleKind::Resource(Kind::ByteLimit));
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(report, before);
        assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
        assert!(matches!(report.syntax().rules()[1], CssRule::Scope(_)));
    }
}
