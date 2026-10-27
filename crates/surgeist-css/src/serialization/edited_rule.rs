//! Immutable borrowed rule-graph inputs for CSSOM-owned edits. No live identity is stored.
use crate::declaration::CssSpecifiedDeclarationBlock;
use crate::query::CssCondition;
use crate::query::container::CssContainerPrelude;
use crate::query::media::CssMediaQueryList;
use crate::query::supports::CssSupportsLeaf;
use crate::rule::{CssLayerName, CssRule};
use crate::selector::CssStyleSelectorList;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::serialization::{CssRuleSerializationError, CssSerializationLimits};

/// A CSS-owned, already checked group prelude. Evaluation stays with its consumer.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssGroupPreludeRef<'a> {
    /// A `@media` prelude.
    Media(&'a CssMediaQueryList),
    /// A `@supports` prelude.
    Supports(&'a CssCondition<CssSupportsLeaf>),
    /// A `@container` prelude.
    Container(&'a CssContainerPrelude),
    /// A `@layer` block prelude; `None` for an anonymous layer.
    Layer(Option<&'a CssLayerName>),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum EditedRuleKind<'a> {
    Parsed(&'a CssRule),
    Style(
        &'a CssStyleSelectorList,
        &'a CssSpecifiedDeclarationBlock,
        &'a [CssRuleGraphInput<'a>],
    ),
    Group(CssGroupPreludeRef<'a>, &'a [CssRuleGraphInput<'a>]),
    NestedDeclarations(&'a CssSpecifiedDeclarationBlock),
    Page(crate::rule::page::CssPageRuleView<'a>),
    Keyframes(crate::rule::keyframes::CssKeyframesRuleView<'a>),
    Import(crate::rule::CssImportRuleView<'a>),
    FontFace(&'a crate::rule::font_face::CssSpecifiedFontFaceDeclarationBlock),
    NamedSupports(
        &'a crate::query::supports::CssSupportsConditionName,
        &'a crate::query::supports::CssSupportsTestBody,
    ),
    SupportsTestChild(crate::query::supports::CssSupportsTestChildRef<'a>),
}

/// Borrowed current rule-graph input, admitted by one final CSSOM writer request.
///
/// Node assembly only stores already typed CSS payloads and child slices. It does
/// not visit descendants, serialize, allocate or reset a native work allowance.
/// Declaration-domain coupling and resource/capability validation occur lazily
/// in the final owning graph traversal, with the actual failing child path.
/// Original payload origins are retained; edited wrappers have no fabricated
/// authored occurrence. Live membership, buffer allocation and placement belong
/// to the consumer.
#[derive(Clone, Copy, Debug)]
pub struct CssRuleGraphInput<'a> {
    pub(crate) kind: EditedRuleKind<'a>,
}
impl<'a> CssRuleGraphInput<'a> {
    /// Wraps a parsed rule, with its whole subtree, unchanged.
    #[must_use]
    pub const fn from_rule(rule: &'a CssRule) -> Self {
        Self {
            kind: EditedRuleKind::Parsed(rule),
        }
    }
    /// The selected block must be Ordinary-domain when the request is admitted. The
    /// selectors keep their symbolic anchor domain: nesting anchors of a style rule,
    /// or scope anchors of a scope body.
    #[must_use]
    pub const fn style(
        selectors: &'a CssStyleSelectorList,
        declarations: &'a CssSpecifiedDeclarationBlock,
        children: &'a [Self],
    ) -> Self {
        Self {
            kind: EditedRuleKind::Style(selectors, declarations, children),
        }
    }
    /// Builds a group rule from its prelude and its child inputs, in order.
    #[must_use]
    pub const fn group(prelude: CssGroupPreludeRef<'a>, children: &'a [Self]) -> Self {
        Self {
            kind: EditedRuleKind::Group(prelude, children),
        }
    }
    /// Empty Ordinary-domain blocks remain real ordered children, with no text.
    #[must_use]
    pub const fn nested_declarations(declarations: &'a CssSpecifiedDeclarationBlock) -> Self {
        Self {
            kind: EditedRuleKind::NestedDeclarations(declarations),
        }
    }
    /// Builds a `@page` rule from its checked view.
    #[must_use]
    pub const fn page(page: crate::rule::page::CssPageRuleView<'a>) -> Self {
        Self {
            kind: EditedRuleKind::Page(page),
        }
    }
    /// Borrows exact current selected descriptor order, independent of the
    /// original whole-rule writer's fixed effective-kind order.
    #[must_use]
    pub const fn font_face(
        declarations: &'a crate::rule::font_face::CssSpecifiedFontFaceDeclarationBlock,
    ) -> Self {
        Self {
            kind: EditedRuleKind::FontFace(declarations),
        }
    }
    /// Children already carry the owning Keyframe-domain coupling proof.
    #[must_use]
    pub const fn keyframes(
        name: &'a crate::rule::keyframes::CssKeyframesName,
        rules: &'a [crate::rule::keyframes::CssKeyframeRuleView<'a>],
    ) -> Self {
        Self {
            kind: EditedRuleKind::Keyframes(
                crate::rule::keyframes::CssKeyframesRuleView::from_checked_parts(name, rules),
            ),
        }
    }
    /// Borrows original checked clauses and supplied current rule-owned media.
    #[must_use]
    pub const fn import(
        rule: &'a crate::rule::CssImportRule,
        media: &'a CssMediaQueryList,
    ) -> Self {
        Self {
            kind: EditedRuleKind::Import(crate::rule::CssImportRuleView::from_checked_parts(
                rule, media,
            )),
        }
    }
    /// Selects Surgeist #644 canonical named-support bytes over a current body.
    /// This explicit opt-in does not change the standard parsed rule serializer.
    pub const fn named_supports(
        name: &'a crate::query::supports::CssSupportsConditionName,
        current_body: &'a crate::query::supports::CssSupportsTestBody,
    ) -> Self {
        Self {
            kind: EditedRuleKind::NamedSupports(name, current_body),
        }
    }
    /// Inert test child; it is never admitted as an ordinary rendering rule.
    pub const fn supports_test_child(
        child: crate::query::supports::CssSupportsTestChildRef<'a>,
    ) -> Self {
        Self {
            kind: EditedRuleKind::SupportsTestChild(child),
        }
    }
    /// Serializes the literal CSSOM text of this graph in one native traversal.
    ///
    /// Supply actual namespace bindings and external Style ancestry; grouping
    /// descendants inherit that ancestry and siblings restore it. One writer
    /// owns all input, projection and output work under `limits.values`.
    ///
    /// Root depth is zero; ordinary/test children add one, body/run containers
    /// are transparent; named-test component enclosures add depth beneath their
    /// leaf, and `limits.max_depth` bounds that structural depth. Other
    /// semantic-value writers retain their selected value quotas. A root-one
    /// consumer must admit its root then subtract one (or its actual selected
    /// subtree depth) before passing the remaining native allowance.
    ///
    /// # Errors
    ///
    /// Returns [`CssRuleSerializationError`] when a limit is exhausted or a rule
    /// has no literal CSSOM format. Failure returns no partial text, mutates no
    /// input and allows a fresh retry.
    pub fn serialize_cssom(
        self,
        namespaces: &crate::selector::CssNamespaceContext,
        ancestor: crate::rule::CssStyleAncestor,
        limits: CssRuleSerializationLimits,
    ) -> Result<String, CssRuleSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits.values);
        writer.max_graph_depth = Some(limits.max_depth);
        writer.append_rule_graph_input(self, namespaces, ancestor)?;
        Ok(writer.css)
    }

    /// Serializes this graph as canonical specified CSS, in one native traversal.
    ///
    /// `limits.values` bounds input nodes, projection nodes and output UTF-8 bytes
    /// cumulatively across the whole graph, and `limits.max_depth` bounds its
    /// structural depth as for [`Self::serialize_cssom`]. Wrappers built from payloads
    /// have no source position; borrowed parsed rules, selectors and queries keep
    /// their actual origins.
    ///
    /// # Errors
    ///
    /// Returns [`CssRuleSerializationError`], with its rule path, when a limit is
    /// exhausted or a value cannot be represented; no partial text is returned.
    pub fn serialize_specified(
        self,
        limits: CssRuleSerializationLimits,
    ) -> Result<String, CssRuleSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits.values);
        writer.max_graph_depth = Some(limits.max_depth);
        writer.append_rule_graph_input_specified(self)?;
        Ok(writer.css)
    }
}

/// Limits for the explicitly structural-depth-bounded native graph operation.
/// Root depth is zero. Rule-child edges include native Page margins and Keyframe
/// blocks; named-support candidate/component nesting also contributes depth.
/// Existing value-only serialization limits remain independent domain requests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssRuleSerializationLimits {
    /// Limits for each value serialization (declarations and preludes) within the graph operation.
    pub values: CssSerializationLimits,
    /// Maximum structural depth of the rule graph; the root has depth zero.
    pub max_depth: usize,
}
impl Default for CssRuleSerializationLimits {
    fn default() -> Self {
        Self {
            values: CssSerializationLimits::default(),
            max_depth: 256,
        }
    }
}
