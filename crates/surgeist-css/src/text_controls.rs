//! Authored Text 4 hyphenation, justification and character spacing.
//!
//! These models preserve specified choices and omissions. Language, glyph,
//! layout, computed integer rounding and contextual spacing remain downstream.

use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueRef, CssIntegerLiteral,
    CssIntegerValue, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    CssValueOrigin, CssValueTokenRef, specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

/// Authored automatic, manual or disabled hyphenation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHyphens {
    None,
    Manual,
    Auto,
}

/// A retained CSS string used as an authored hyphenation character.
///
/// The original component, including browser recovery provenance, is retained.
/// Programmatic strings use the shared checked string construction contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssHyphenateString {
    component: Box<CssComponentValue>,
}
impl CssHyphenateString {
    /// Accepts exactly one string token, preserving its original origin.
    #[must_use]
    pub fn try_from_component(component: CssComponentValue) -> Option<Self> {
        matches!(
            component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::String(_))
        )
        .then(|| Self {
            component: Box::new(component),
        })
    }
    /// Constructs a decoded string through the shared token owner; NUL is rejected.
    pub fn try_new(value: impl Into<String>) -> std::result::Result<Self, CssComponentValueError> {
        Ok(Self {
            component: Box::new(CssComponentValue::try_string(value)?),
        })
    }
    /// Borrows the original checked string component.
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    /// Borrows decoded content without truncating typographic units.
    pub fn as_str(&self) -> &str {
        match self.component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::String(value)) => value,
            _ => unreachable!("checked hyphenation string"),
        }
    }
    /// Borrows the original parsed or programmatic token origin.
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
}
/// A symbolic UA-selected character or an authored CSS string.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHyphenateCharacter {
    Auto,
    String(CssHyphenateString),
}

/// A nonnegative ordinary integer or deferred Integer-root function math.
///
/// This shared hyphenation payload retains existing integer spelling, equality,
/// origins and arithmetic policy. Function results are not clamped or rounded.
#[derive(Clone, Debug, PartialEq)]
pub struct CssHyphenateLimitInteger {
    value: CssIntegerValue,
}
impl CssHyphenateLimitInteger {
    /// Rejects negative or noninteger bare roots while retaining function math.
    #[must_use]
    pub fn try_new(value: CssIntegerValue) -> Option<Self> {
        let value = match value {
            CssIntegerValue::Literal(literal) => {
                if literal.is_negative() {
                    return None;
                }
                CssIntegerValue::Literal(literal)
            }
            CssIntegerValue::Calculation(calculation) => {
                let root =
                    crate::specified_numeric::significant_root(calculation.components()).ok()?;
                match root.view() {
                    CssComponentValueRef::Token(_) => {
                        let literal = CssIntegerLiteral::try_from_component(root.clone()).ok()?;
                        if literal.is_negative() {
                            return None;
                        }
                        CssIntegerValue::Literal(literal)
                    }
                    CssComponentValueRef::Function(_) => CssIntegerValue::Calculation(calculation),
                    _ => return None,
                }
            }
        };
        Some(Self { value })
    }
    /// Borrows the admitted exact literal or symbolic function root.
    pub const fn value(&self) -> &CssIntegerValue {
        &self.value
    }
    /// Borrows the existing integer root's original origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            CssIntegerValue::Literal(value) => value.origin(),
            CssIntegerValue::Calculation(value) => value.origin(),
        }
    }
}
/// One authored hyphenation count, with an unresolved `auto` alternative.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssHyphenateLimitCharsComponent {
    Auto,
    Integer(CssHyphenateLimitInteger),
}
/// One to three authored slots; an after slot cannot exist without a before slot.
#[derive(Clone, Debug, PartialEq)]
pub struct CssHyphenateLimitChars {
    total: CssHyphenateLimitCharsComponent,
    before: Option<CssHyphenateLimitCharsComponent>,
    after: Option<CssHyphenateLimitCharsComponent>,
}
impl CssHyphenateLimitChars {
    /// Retains authored arity and rejects an after slot without a before slot.
    #[must_use]
    pub fn try_new(
        total: CssHyphenateLimitCharsComponent,
        before: Option<CssHyphenateLimitCharsComponent>,
        after: Option<CssHyphenateLimitCharsComponent>,
    ) -> Option<Self> {
        (after.is_none() || before.is_some()).then_some(Self {
            total,
            before,
            after,
        })
    }
    /// Constructs the intrinsic authored single `auto` initial.
    #[must_use]
    pub const fn auto() -> Self {
        Self {
            total: CssHyphenateLimitCharsComponent::Auto,
            before: None,
            after: None,
        }
    }
    /// Borrows the mandatory authored total count.
    pub const fn total(&self) -> &CssHyphenateLimitCharsComponent {
        &self.total
    }
    /// Borrows the explicitly authored before count, if present.
    pub const fn before(&self) -> Option<&CssHyphenateLimitCharsComponent> {
        self.before.as_ref()
    }
    /// Borrows the explicitly authored after count, if present.
    pub const fn after(&self) -> Option<&CssHyphenateLimitCharsComponent> {
        self.after.as_ref()
    }
    /// Returns authored slot count, without inserting effective defaults.
    pub const fn authored_len(&self) -> usize {
        1 + self.before.is_some() as usize + self.after.is_some() as usize
    }
    /// Supplies omission defaults symbolically, without rounding or choosing UA counts.
    /// Explicit children are cloned with their original provenance.
    pub fn effective_components(&self) -> [CssHyphenateLimitCharsComponent; 3] {
        let before = self
            .before
            .clone()
            .unwrap_or(CssHyphenateLimitCharsComponent::Auto);
        let after = self.after.clone().unwrap_or_else(|| before.clone());
        [self.total.clone(), before, after]
    }
}
/// An authored maximum line count or an unlimited count.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssHyphenateLimitLines {
    NoLimit,
    Integer(CssHyphenateLimitInteger),
}
/// Authored restrictions on hyphenation of the last line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHyphenateLimitLast {
    None,
    Always,
    Column,
    Page,
    Spread,
}

/// Authored justification method; `distribute` computes to inter-character downstream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextJustifyBase {
    Auto,
    None,
    InterWord,
    InterCharacter,
    Ruby,
    Distribute,
}
/// Nonempty authored justification roles, retaining an omitted base.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssTextJustify {
    base: Option<CssTextJustifyBase>,
    no_compress: bool,
}
impl CssTextJustify {
    /// Rejects the empty aggregate; a modifier alone is valid.
    #[must_use]
    pub const fn try_new(base: Option<CssTextJustifyBase>, no_compress: bool) -> Option<Self> {
        if base.is_some() || no_compress {
            Some(Self { base, no_compress })
        } else {
            None
        }
    }
    /// Constructs intrinsic `auto` with no compression modifier.
    #[must_use]
    pub const fn auto() -> Self {
        Self {
            base: Some(CssTextJustifyBase::Auto),
            no_compress: false,
        }
    }
    /// Returns the explicitly authored base, including legacy `distribute`.
    pub const fn base(&self) -> Option<CssTextJustifyBase> {
        self.base
    }
    /// Reports explicitly authored `no-compress`.
    pub const fn no_compress(&self) -> bool {
        self.no_compress
    }
}
/// Noninherited alignment of the text group within its block container.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextGroupAlign {
    None,
    Start,
    End,
    Left,
    Right,
    Center,
}

/// An explicitly authored autospace insertion policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAutospaceMode {
    Insert,
    Replace,
}
/// Nonempty authored character-class spacing flags and optional insertion mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssAutospaceValues {
    ideograph_alpha: bool,
    ideograph_numeric: bool,
    punctuation: bool,
    mode: Option<CssAutospaceMode>,
}
impl CssAutospaceValues {
    /// Admits unique flags or a mode alone; rejects the all-absent state.
    #[must_use]
    pub const fn try_new(
        ideograph_alpha: bool,
        ideograph_numeric: bool,
        punctuation: bool,
        mode: Option<CssAutospaceMode>,
    ) -> Option<Self> {
        if ideograph_alpha || ideograph_numeric || punctuation || mode.is_some() {
            Some(Self {
                ideograph_alpha,
                ideograph_numeric,
                punctuation,
                mode,
            })
        } else {
            None
        }
    }
    /// Reports explicitly authored ideograph-to-alphabet spacing.
    pub const fn ideograph_alpha(&self) -> bool {
        self.ideograph_alpha
    }
    /// Reports explicitly authored ideograph-to-number spacing.
    pub const fn ideograph_numeric(&self) -> bool {
        self.ideograph_numeric
    }
    /// Reports explicitly authored punctuation spacing.
    pub const fn punctuation(&self) -> bool {
        self.punctuation
    }
    /// Returns authored insertion mode; omission remains observable.
    pub const fn mode(&self) -> Option<CssAutospaceMode> {
        self.mode
    }
    /// Supplies the symbolic insertion default without executing text spacing.
    pub const fn effective_mode(&self) -> CssAutospaceMode {
        match self.mode {
            Some(mode) => mode,
            None => CssAutospaceMode::Insert,
        }
    }
}
/// The `<autospace>` constituent grammar, excluding whole-longhand normal/auto.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAutospace {
    NoAutospace,
    Spacing(CssAutospaceValues),
}
/// The whole authored autospace longhand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextAutospace {
    Normal,
    Auto,
    Autospace(CssAutospace),
}
/// The six-token `<spacing-trim>` constituent, excluding whole-longhand `auto`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSpacingTrim {
    SpaceAll,
    Normal,
    SpaceFirst,
    TrimStart,
    TrimBoth,
    TrimAll,
}
/// Whole authored punctuation trimming, including contextual `auto`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextSpacingTrim {
    Trim(CssSpacingTrim),
    Auto,
}
/// Nonempty shorthand constituents; whole-member omissions remain explicit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssTextSpacingValues {
    trim: Option<CssSpacingTrim>,
    autospace: Option<CssAutospace>,
}
impl CssTextSpacingValues {
    /// Rejects empty input; constituent types exclude longhand-only alternatives.
    #[must_use]
    pub const fn try_new(
        trim: Option<CssSpacingTrim>,
        autospace: Option<CssAutospace>,
    ) -> Option<Self> {
        if trim.is_some() || autospace.is_some() {
            Some(Self { trim, autospace })
        } else {
            None
        }
    }
    /// Returns the authored trim constituent, if present.
    pub const fn trim(&self) -> Option<CssSpacingTrim> {
        self.trim
    }
    /// Returns the complete authored autospace constituent, if present.
    pub const fn autospace(&self) -> Option<CssAutospace> {
        self.autospace
    }
}
/// Authored spacing shorthand, before intrinsic projection or glyph execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextSpacing {
    None,
    Auto,
    Components(CssTextSpacingValues),
}
/// The mutually exclusive end-hanging role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHangingPunctuationEnd {
    ForceEnd,
    AllowEnd,
}
/// Nonempty first/end/last hanging roles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssHangingPunctuationValues {
    first: bool,
    end: Option<CssHangingPunctuationEnd>,
    last: bool,
}
impl CssHangingPunctuationValues {
    /// Rejects the all-absent state; the end enum excludes force/allow conflicts.
    #[must_use]
    pub const fn try_new(
        first: bool,
        end: Option<CssHangingPunctuationEnd>,
        last: bool,
    ) -> Option<Self> {
        if first || end.is_some() || last {
            Some(Self { first, end, last })
        } else {
            None
        }
    }
    /// Reports explicitly authored first-character hanging.
    pub const fn first(&self) -> bool {
        self.first
    }
    /// Returns the explicitly authored exclusive end role.
    pub const fn end(&self) -> Option<CssHangingPunctuationEnd> {
        self.end
    }
    /// Reports explicitly authored last-character hanging.
    pub const fn last(&self) -> bool {
        self.last
    }
}
/// Whole authored hanging-punctuation grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHangingPunctuation {
    None,
    Hang(CssHangingPunctuationValues),
}

macro_rules! provider {
    ($ty:ty, $value:ident, $writer:ident => $body:expr) => {
        impl $ty {
            /// Emits canonical specified syntax without contextual computation.
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
            }
            /// Emits atomically under shared work and UTF-8 byte budgets.
            /// Finite/string leaves cost one visit each; carriers are transparent.
            pub fn serialize_specified_with_limits(&self, limits: CssSpecifiedValueSerializationLimits) -> Result<String> {
                let mut writer = SpecifiedRuleWriter::new(limits);
                self.append_to_rule_writer(&mut writer)?;
                Ok(writer.css)
            }
            pub(crate) fn append_to_rule_writer(&self, $writer: &mut SpecifiedRuleWriter) -> Result<()> {
                let $value = self;
                $body
            }
        }
    };
}
macro_rules! keywords {
    ($ty:ty, $($variant:ident => $text:literal),+ $(,)?) => {
        provider!($ty, value, writer => writer.keyword(match value { $(Self::$variant => $text),+ }));
    };
}
keywords!(CssHyphens, None => "none", Manual => "manual", Auto => "auto");
keywords!(CssHyphenateLimitLast, None => "none", Always => "always", Column => "column", Page => "page", Spread => "spread");
keywords!(CssTextJustifyBase, Auto => "auto", None => "none", InterWord => "inter-word", InterCharacter => "inter-character", Ruby => "ruby", Distribute => "distribute");
keywords!(CssTextGroupAlign, None => "none", Start => "start", End => "end", Left => "left", Right => "right", Center => "center");
keywords!(CssAutospaceMode, Insert => "insert", Replace => "replace");
keywords!(CssSpacingTrim, SpaceAll => "space-all", Normal => "normal", SpaceFirst => "space-first", TrimStart => "trim-start", TrimBoth => "trim-both", TrimAll => "trim-all");
keywords!(CssHangingPunctuationEnd, ForceEnd => "force-end", AllowEnd => "allow-end");
provider!(CssHyphenateString, value, writer => { writer.node()?; writer.append_string(value.as_str()) });
provider!(CssHyphenateCharacter, value, writer => match value { Self::Auto => writer.keyword("auto"), Self::String(value) => value.append_to_rule_writer(writer) });
provider!(CssHyphenateLimitInteger, value, writer => value.value.append_to_rule_writer(writer));
provider!(CssHyphenateLimitCharsComponent, value, writer => match value { Self::Auto => writer.keyword("auto"), Self::Integer(value) => value.append_to_rule_writer(writer) });
provider!(CssHyphenateLimitChars, value, writer => {
    value.total.append_to_rule_writer(writer)?;
    if let Some(before) = &value.before { writer.append(" ")?; before.append_to_rule_writer(writer)?; }
    if let Some(after) = &value.after { writer.append(" ")?; after.append_to_rule_writer(writer)?; }
    Ok(())
});
provider!(CssHyphenateLimitLines, value, writer => match value { Self::NoLimit => writer.keyword("no-limit"), Self::Integer(value) => value.append_to_rule_writer(writer) });
provider!(CssTextJustify, value, writer => {
    if let Some(base) = value.base { base.append_to_rule_writer(writer)?; }
    if value.no_compress { if value.base.is_some() { writer.append(" ")?; } writer.keyword("no-compress")?; }
    Ok(())
});
provider!(CssAutospaceValues, value, writer => {
    let mut separated = false;
    for (present, text) in [(value.ideograph_alpha,"ideograph-alpha"),(value.ideograph_numeric,"ideograph-numeric"),(value.punctuation,"punctuation")] {
        if present { if separated { writer.append(" ")?; } writer.keyword(text)?; separated = true; }
    }
    if let Some(mode) = value.mode { if separated { writer.append(" ")?; } mode.append_to_rule_writer(writer)?; }
    Ok(())
});
provider!(CssAutospace, value, writer => match value { Self::NoAutospace => writer.keyword("no-autospace"), Self::Spacing(value) => value.append_to_rule_writer(writer) });
provider!(CssTextAutospace, value, writer => match value { Self::Normal => writer.keyword("normal"), Self::Auto => writer.keyword("auto"), Self::Autospace(value) => value.append_to_rule_writer(writer) });
provider!(CssTextSpacingTrim, value, writer => match value { Self::Auto => writer.keyword("auto"), Self::Trim(value) => value.append_to_rule_writer(writer) });
provider!(CssTextSpacingValues, value, writer => {
    if let Some(trim) = value.trim { writer.source_member(0, |writer| trim.append_to_rule_writer(writer))?; }
    if let Some(auto) = value.autospace { writer.source_member(1, |writer| {
        if value.trim.is_some() { writer.append(" ")?; }
        auto.append_to_rule_writer(writer)
    })?; }
    Ok(())
});
provider!(CssTextSpacing, value, writer => match value { Self::None => writer.keyword("none"), Self::Auto => writer.keyword("auto"), Self::Components(value) => value.append_to_rule_writer(writer) });
provider!(CssHangingPunctuationValues, value, writer => {
    if value.first { writer.keyword("first")?; }
    if let Some(end) = value.end { if value.first { writer.append(" ")?; } end.append_to_rule_writer(writer)?; }
    if value.last { if value.first || value.end.is_some() { writer.append(" ")?; } writer.keyword("last")?; }
    Ok(())
});
provider!(CssHangingPunctuation, value, writer => match value { Self::None => writer.keyword("none"), Self::Hang(value) => value.append_to_rule_writer(writer) });

#[cfg(test)]
mod provider_tests {
    use super::*;
    use crate::CssSpecifiedValueSerializationErrorKind as Kind;
    use crate::CssSpecifiedValueSerializationLimits as Limits;
    use crate::{CssKnownPropertyValueRef, parse_style_attribute};

    fn emit(value: CssKnownPropertyValueRef<'_>, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match value {
            CssKnownPropertyValueRef::Hyphens(v) => v.value().append_to_rule_writer(writer),
            CssKnownPropertyValueRef::HyphenateCharacter(v) => {
                v.value().append_to_rule_writer(writer)
            }
            CssKnownPropertyValueRef::HyphenateLimitZone(v) => {
                v.value().append_to_rule_writer(writer)
            }
            CssKnownPropertyValueRef::HyphenateLimitChars(v) => {
                v.value().append_to_rule_writer(writer)
            }
            CssKnownPropertyValueRef::HyphenateLimitLines(v) => {
                v.value().append_to_rule_writer(writer)
            }
            CssKnownPropertyValueRef::HyphenateLimitLast(v) => {
                v.value().append_to_rule_writer(writer)
            }
            CssKnownPropertyValueRef::TextJustify(v) => v.value().append_to_rule_writer(writer),
            CssKnownPropertyValueRef::TextGroupAlign(v) => v.value().append_to_rule_writer(writer),
            CssKnownPropertyValueRef::LinePadding(v) => v.value().append_to_rule_writer(writer),
            CssKnownPropertyValueRef::TextAutospace(v) => v.value().append_to_rule_writer(writer),
            CssKnownPropertyValueRef::TextSpacingTrim(v) => v.value().append_to_rule_writer(writer),
            CssKnownPropertyValueRef::TextSpacing(v) => v.value().append_to_rule_writer(writer),
            CssKnownPropertyValueRef::HangingPunctuation(v) => {
                v.value().append_to_rule_writer(writer)
            }
            _ => unreachable!("selected provider fixture"),
        }
    }
    const CASES: [(&str, &str, usize); 13] = [
        ("hyphens", "manual", 1),
        ("hyphenate-character", "\"😀\"", 1),
        ("hyphenate-limit-zone", "-2px", 1),
        ("hyphenate-limit-chars", "8 auto 3", 3),
        ("hyphenate-limit-lines", "2", 1),
        ("hyphenate-limit-last", "column", 1),
        ("text-justify", "distribute no-compress", 2),
        ("text-group-align", "center", 1),
        ("line-padding", "-2px", 1),
        ("text-autospace", "ideograph-alpha punctuation replace", 3),
        ("text-spacing-trim", "trim-both", 1),
        ("text-spacing", "trim-both ideograph-alpha replace", 3),
        ("hanging-punctuation", "first force-end last", 3),
    ];
    #[test]
    fn private_providers_compose_exact_work_and_utf8_bytes_under_one_context() {
        for (name, text, nodes) in CASES {
            let report = parse_style_attribute(&format!("{name}:{text}"));
            assert!(report.is_clean());
            let source = &report.syntax()[0];
            let before = source.clone();
            let value = source.known().unwrap().property_value().unwrap();
            let expected = format!("{text}émanual");
            for (limits, failure) in [
                (Limits::new(nodes + 1, nodes + 1, expected.len()), None),
                (
                    Limits::new(nodes, nodes + 1, expected.len()),
                    Some(Kind::InputNodeLimit),
                ),
                (
                    Limits::new(nodes + 1, nodes, expected.len()),
                    Some(Kind::ProjectionNodeLimit),
                ),
                (
                    Limits::new(nodes + 1, nodes + 1, expected.len() - 1),
                    Some(Kind::ByteLimit),
                ),
            ] {
                for _ in 0..2 {
                    let mut writer = SpecifiedRuleWriter::new(limits);
                    emit(value, &mut writer).unwrap();
                    writer.append("é").unwrap();
                    let result = CssHyphens::Manual.append_to_rule_writer(&mut writer);
                    if let Some(kind) = failure {
                        assert_eq!(result.unwrap_err().kind(), kind);
                        assert_eq!(writer.css, format!("{text}é"));
                    } else {
                        result.unwrap();
                        assert_eq!(writer.css, expected);
                    }
                    assert_eq!(source, &before);
                }
            }
        }
    }
    #[test]
    fn nested_suppression_charges_each_child_and_restores_emission_after_success_or_failure() {
        for (name, text, nodes) in CASES {
            let report = parse_style_attribute(&format!("{name}:{text}"));
            assert!(report.is_clean());
            let source = &report.syntax()[0];
            let before = source.clone();
            let value = source.known().unwrap().property_value().unwrap();
            // Suppressed children consume work but no quote/separator bytes.
            let mut writer = SpecifiedRuleWriter::new(Limits::new(nodes + 1, nodes + 1, 8));
            writer
                .without_output(|writer| writer.without_output(|writer| emit(value, writer)))
                .unwrap();
            assert!(writer.css.is_empty());
            assert!(!writer.context.output_suppressed());
            writer.append("é").unwrap();
            CssHyphens::Manual
                .append_to_rule_writer(&mut writer)
                .unwrap();
            assert_eq!(writer.css, "émanual");
            for (limits, kind) in [
                (Limits::new(nodes - 1, usize::MAX, 1), Kind::InputNodeLimit),
                (
                    Limits::new(usize::MAX, nodes - 1, 1),
                    Kind::ProjectionNodeLimit,
                ),
            ] {
                for _ in 0..2 {
                    let mut writer = SpecifiedRuleWriter::new(limits);
                    let error = writer
                        .without_output(|writer| {
                            writer.without_output(|writer| emit(value, writer))
                        })
                        .unwrap_err();
                    assert_eq!(error.kind(), kind);
                    assert!(!writer.context.output_suppressed());
                    assert!(writer.css.is_empty());
                    writer.append("x").unwrap();
                    assert_eq!(writer.css, "x");
                    assert_eq!(source, &before);
                }
            }
        }
    }
    #[test]
    fn symbolic_numeric_children_share_work_with_siblings_and_ignore_suppressed_byte_limits() {
        for (name, input, canonical, input_nodes, projection_nodes) in [
            ("hyphenate-limit-zone", "calc(1px + 2px)", "calc(3px)", 4, 3),
            ("line-padding", "calc(-2px + -3px)", "calc(-5px)", 4, 3),
            ("hyphenate-limit-lines", "calc(-3 / 2)", "calc(-1.5)", 4, 4),
            (
                "hyphenate-limit-chars",
                "calc(-3 / 2) auto",
                "calc(-1.5) auto",
                5,
                5,
            ),
        ] {
            let report = parse_style_attribute(&format!("{name}:{input}"));
            assert!(report.is_clean());
            let source = &report.syntax()[0];
            let before = source.clone();
            let value = source.known().unwrap().property_value().unwrap();
            let expected = format!("{canonical}émanual");
            for (limits, failure) in [
                (
                    Limits::new(input_nodes + 1, projection_nodes + 1, expected.len()),
                    None,
                ),
                (
                    Limits::new(input_nodes, projection_nodes + 1, expected.len()),
                    Some(Kind::InputNodeLimit),
                ),
                (
                    Limits::new(input_nodes + 1, projection_nodes, expected.len()),
                    Some(Kind::ProjectionNodeLimit),
                ),
                (
                    Limits::new(input_nodes + 1, projection_nodes + 1, expected.len() - 1),
                    Some(Kind::ByteLimit),
                ),
            ] {
                let mut writer = SpecifiedRuleWriter::new(limits);
                emit(value, &mut writer).unwrap();
                writer.append("é").unwrap();
                let result = CssHyphens::Manual.append_to_rule_writer(&mut writer);
                match failure {
                    Some(kind) => {
                        assert_eq!(result.unwrap_err().kind(), kind);
                        assert_eq!(writer.css, format!("{canonical}é"));
                    }
                    None => {
                        result.unwrap();
                        assert_eq!(writer.css, expected);
                    }
                }
            }
            let mut writer =
                SpecifiedRuleWriter::new(Limits::new(input_nodes + 1, projection_nodes + 1, 6));
            writer
                .without_output(|writer| writer.without_output(|writer| emit(value, writer)))
                .unwrap();
            assert!(writer.css.is_empty());
            assert!(!writer.context.output_suppressed());
            CssHyphens::Manual
                .append_to_rule_writer(&mut writer)
                .unwrap();
            assert_eq!(writer.css, "manual");
            for (limits, kind) in [
                (
                    Limits::new(input_nodes - 1, usize::MAX, 1),
                    Kind::InputNodeLimit,
                ),
                (
                    Limits::new(usize::MAX, projection_nodes - 1, 1),
                    Kind::ProjectionNodeLimit,
                ),
            ] {
                let mut writer = SpecifiedRuleWriter::new(limits);
                let error = writer
                    .without_output(|writer| writer.without_output(|writer| emit(value, writer)))
                    .unwrap_err();
                assert_eq!(error.kind(), kind);
                assert!(!writer.context.output_suppressed());
                assert!(writer.css.is_empty());
                writer.append("x").unwrap();
                assert_eq!(writer.css, "x");
            }
            assert_eq!(source, &before);
        }
    }
}
