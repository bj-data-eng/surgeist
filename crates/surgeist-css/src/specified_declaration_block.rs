//! Immutable CSSOM specified-order projection of authored declaration occurrences.

use std::{collections::HashMap, fmt, sync::OnceLock};

use crate::{
    CssComponentValueRef, CssComponentValues, CssContributionValueRef, CssContributions,
    CssCustomDeclaration, CssDeclaration, CssDeclarationList, CssExpansion, CssExpansionError,
    CssImportance, CssKeyframeDeclarationList, CssKnownProperty, CssLonghandContribution,
    CssLonghandProperty, CssPropertyKindRef, CssPropertyNameRef,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssSvgGlyphOrientationVerticalContribution,
    CssUniversalReset,
    declaration_serialization::{append_global, append_retained_value},
    expansion::{MappingLogic, SPECIFIED_TERMINALS},
    specified_rule_serialization::SpecifiedRuleWriter,
};

mod cssom;

type Result<T> = std::result::Result<T, CssDeclarationBlockError>;

/// Why an input expansion is not admitted by the keyframe grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssInvalidKeyframeSourceReason {
    Importance,
    Property,
}

/// An atomic declaration-block failure, distinct from parser recovery.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssDeclarationBlockErrorKind {
    Serialization(CssSpecifiedValueSerializationError),
    Expansion(CssExpansionError),
    /// A selected state must contain each terminal identity at most once.
    DuplicateTerminal,
    /// A selected source or terminal is outside Page/margin property grammar.
    InvalidPageSource,
    /// Exactly the eight substitution-dependent Logical 1 mode-switch families.
    PendingFootprintUndetermined {
        property: CssKnownProperty,
    },
    InvalidKeyframeSource {
        reason: CssInvalidKeyframeSourceReason,
    },
    /// A surviving authored environment requirement has no standalone specified value.
    UnresolvedEnvironment {
        property: CssKnownProperty,
    },
}

/// Typed cause and genuine original source; no partial block or text is returned.
#[derive(Clone, Debug)]
pub struct CssDeclarationBlockError {
    kind: CssDeclarationBlockErrorKind,
    declaration: Option<CssDeclaration>,
    authored_ordinal: Option<usize>,
    member_ordinal: Option<usize>,
}

impl CssDeclarationBlockError {
    #[must_use]
    pub const fn kind(&self) -> &CssDeclarationBlockErrorKind {
        &self.kind
    }
    #[must_use]
    pub const fn declaration(&self) -> Option<&CssDeclaration> {
        self.declaration.as_ref()
    }
    /// Zero-based input occurrence or supplied expansion-group ordinal.
    #[must_use]
    pub const fn authored_ordinal(&self) -> Option<usize> {
        self.authored_ordinal
    }
    /// Zero-based generated member ordinal, absent before a member exists.
    #[must_use]
    pub const fn member_ordinal(&self) -> Option<usize> {
        self.member_ordinal
    }
    fn new(kind: CssDeclarationBlockErrorKind) -> Self {
        Self {
            kind,
            declaration: None,
            authored_ordinal: None,
            member_ordinal: None,
        }
    }
    fn at(mut self, source: &CssDeclaration, authored: usize, member: Option<usize>) -> Self {
        if self.declaration.is_none() {
            self.declaration = Some(source.clone());
            self.authored_ordinal = Some(authored);
            self.member_ordinal = member;
        }
        self
    }
}
impl From<CssSpecifiedValueSerializationError> for CssDeclarationBlockError {
    fn from(error: CssSpecifiedValueSerializationError) -> Self {
        Self::new(CssDeclarationBlockErrorKind::Serialization(error))
    }
}
impl From<CssExpansionError> for CssDeclarationBlockError {
    fn from(error: CssExpansionError) -> Self {
        Self::new(CssDeclarationBlockErrorKind::Expansion(error))
    }
}
impl fmt::Display for CssDeclarationBlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            CssDeclarationBlockErrorKind::InvalidPageSource => {
                f.write_str("property is not admitted in Page/margin domain")
            }
            CssDeclarationBlockErrorKind::DuplicateTerminal => {
                f.write_str("duplicate selected terminal")
            }
            CssDeclarationBlockErrorKind::Serialization(e) => e.fmt(f),
            CssDeclarationBlockErrorKind::Expansion(e) => e.fmt(f),
            CssDeclarationBlockErrorKind::PendingFootprintUndetermined { property } => write!(
                f,
                "pending {} has no selected physical/logical footprint",
                property.canonical_name()
            ),
            CssDeclarationBlockErrorKind::InvalidKeyframeSource { reason } => {
                write!(f, "keyframe source is not admitted: {reason:?}")
            }
            CssDeclarationBlockErrorKind::UnresolvedEnvironment { property } => write!(
                f,
                "{} requires an authored environment",
                property.canonical_name()
            ),
        }
    }
}
impl std::error::Error for CssDeclarationBlockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            CssDeclarationBlockErrorKind::Serialization(e) => Some(e),
            CssDeclarationBlockErrorKind::Expansion(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Domain {
    Ordinary,
    Keyframe,
    Page,
    Margin,
}
impl Domain {
    fn admits(self, property: CssKnownProperty) -> bool {
        match self {
            Self::Ordinary => true,
            Self::Keyframe => crate::parser::keyframe_property_admitted(property),
            Self::Page | Self::Margin => crate::parser::is_page_margin_property(property),
        }
    }
    fn admits_terminal(self, property: Terminal) -> bool {
        match property {
            Terminal::Known(property) if matches!(self, Self::Page | Self::Margin) => {
                static TERMINALS: OnceLock<Vec<CssLonghandProperty>> = OnceLock::new();
                TERMINALS
                    .get_or_init(|| {
                        let mut terminals = Vec::new();
                        for &property in CssKnownProperty::all() {
                            if !crate::parser::is_page_margin_property(property) {
                                continue;
                            }
                            match property.metadata().expect("shared schema metadata").kind() {
                                CssPropertyKindRef::Longhand(meta) => {
                                    terminals.push(meta.property())
                                }
                                CssPropertyKindRef::Shorthand(meta) => {
                                    terminals.extend_from_slice(meta.members())
                                }
                                CssPropertyKindRef::FourSideShorthand(meta) => {
                                    terminals.extend_from_slice(
                                        meta.members(crate::CssBoxSideKind::Physical),
                                    );
                                    terminals.extend_from_slice(
                                        meta.members(crate::CssBoxSideKind::Logical),
                                    );
                                }
                                CssPropertyKindRef::UniversalReset(_) => {}
                            }
                        }
                        terminals.sort_by_key(|p| p.known_property().canonical_name());
                        terminals.dedup();
                        terminals
                    })
                    .contains(&property)
            }
            Terminal::Known(property) => self.admits(property.known_property()),
            Terminal::SvgGlyphOrientationVertical => {
                matches!(self, Self::Ordinary | Self::Keyframe)
            }
        }
    }
    fn admit_source(self, source: &CssDeclaration, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        if self == Self::Keyframe {
            writer.context.charge_projection(1)?;
            let reason = if source.importance() != CssImportance::Normal {
                Some(CssInvalidKeyframeSourceReason::Importance)
            } else if source
                .known()
                .is_some_and(|known| !self.admits(known.property()))
            {
                Some(CssInvalidKeyframeSourceReason::Property)
            } else {
                None
            };
            if let Some(reason) = reason {
                return Err(CssDeclarationBlockError::new(
                    CssDeclarationBlockErrorKind::InvalidKeyframeSource { reason },
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Terminal {
    Known(CssLonghandProperty),
    SvgGlyphOrientationVertical,
}
impl Terminal {
    fn property_name(self) -> CssPropertyNameRef<'static> {
        match self {
            Self::Known(property) => CssPropertyNameRef::Known(property.known_property()),
            Self::SvgGlyphOrientationVertical => {
                crate::CssSvgGlyphOrientationVerticalDeclaration::metadata().property()
            }
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Known(property) => property.known_property().canonical_name(),
            Self::SvgGlyphOrientationVertical => {
                crate::CssSvgGlyphOrientationVerticalDeclaration::metadata().name()
            }
        }
    }
    fn mapping(self) -> (Option<&'static str>, MappingLogic) {
        match self {
            Self::Known(property) => property.mapping(),
            Self::SvgGlyphOrientationVertical => (None, MappingLogic::Neutral),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Footprint {
    Canonical(&'static [CssLonghandProperty]),
    Supported(&'static [Terminal]),
}
impl Footprint {
    fn len(self) -> usize {
        match self {
            Self::Canonical(values) => values.len(),
            Self::Supported(values) => values.len(),
        }
    }
    fn iter(self) -> impl Iterator<Item = Terminal> {
        let (canonical, supported): (&[CssLonghandProperty], &[Terminal]) = match self {
            Self::Canonical(values) => (values, &[]),
            Self::Supported(values) => (&[], values),
        };
        canonical
            .iter()
            .copied()
            .map(Terminal::Known)
            .chain(supported.iter().copied())
    }
    fn contains(self, property: &Terminal) -> bool {
        self.iter().any(|value| value == *property)
    }
}

fn svg_footprint() -> &'static [Terminal] {
    static FOOTPRINT: OnceLock<Vec<Terminal>> = OnceLock::new();
    FOOTPRINT.get_or_init(|| {
        let metadata = crate::CssSvgGlyphOrientationVerticalDeclaration::metadata();
        metadata
            .settable_members()
            .iter()
            .chain(metadata.reset_only_members())
            .map(|name| match name {
                CssPropertyNameRef::SvgGlyphOrientationVertical => {
                    Terminal::SvgGlyphOrientationVertical
                }
                CssPropertyNameRef::Known(_) | CssPropertyNameRef::Custom(_) => {
                    unreachable!("independent SVG terminal metadata")
                }
            })
            .collect()
    })
}

#[derive(Clone, Debug)]
enum EntryValue {
    Completed(CssLonghandContribution),
    SvgGlyphOrientationVertical(CssSvgGlyphOrientationVerticalContribution),
    Universal(CssUniversalReset),
    Custom,
    Pending { direct: bool },
}

/// A borrowed specified terminal value, retaining authored phase distinctions.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssSpecifiedDeclarationValueRef<'a> {
    Completed(CssContributionValueRef<'a>),
    /// The independent SVG terminal retains its actual completed contribution.
    SvgGlyphOrientationVertical(&'a CssSvgGlyphOrientationVerticalContribution),
    Custom(&'a CssCustomDeclaration),
    UnresolvedLonghand(&'a CssComponentValues),
    /// A fixed-footprint pending shorthand member; its individual value is empty.
    PendingShorthand(&'a CssDeclaration),
}

/// One terminal winner at its original expanded position. Construction is private.
#[derive(Clone, Debug)]
pub struct CssSpecifiedDeclarationEntry {
    property: Option<Terminal>,
    source: CssDeclaration,
    importance: CssImportance,
    authored_ordinal: usize,
    member_ordinal: usize,
    value: EntryValue,
}
impl CssSpecifiedDeclarationEntry {
    #[must_use]
    pub fn property_name(&self) -> CssPropertyNameRef<'_> {
        match self.property {
            Some(property) => property.property_name(),
            None => self.source.property_name(),
        }
    }
    #[must_use]
    pub fn importance(&self) -> CssImportance {
        self.importance
    }
    /// Selects this terminal's priority without changing sibling terminals or the
    /// original authored occurrence used for provenance and pending identity.
    #[must_use]
    pub fn with_importance(&self, importance: CssImportance) -> Self {
        let mut entry = self.clone();
        entry.importance = importance;
        entry
    }
    #[must_use]
    pub const fn source(&self) -> &CssDeclaration {
        &self.source
    }
    #[must_use]
    pub const fn authored_ordinal(&self) -> usize {
        self.authored_ordinal
    }
    #[must_use]
    pub const fn member_ordinal(&self) -> usize {
        self.member_ordinal
    }
    #[must_use]
    pub fn value(&self) -> CssSpecifiedDeclarationValueRef<'_> {
        match &self.value {
            EntryValue::Completed(c) => CssSpecifiedDeclarationValueRef::Completed(c.value()),
            EntryValue::SvgGlyphOrientationVertical(c) => match c.global() {
                Some(keyword) => CssSpecifiedDeclarationValueRef::Completed(
                    CssContributionValueRef::Global(keyword),
                ),
                None => CssSpecifiedDeclarationValueRef::SvgGlyphOrientationVertical(c),
            },
            EntryValue::Universal(reset) => CssSpecifiedDeclarationValueRef::Completed(
                CssContributionValueRef::Global(reset.keyword()),
            ),
            EntryValue::Custom => CssSpecifiedDeclarationValueRef::Custom(
                self.source.custom().expect("custom entry source"),
            ),
            EntryValue::Pending { direct: true } => {
                CssSpecifiedDeclarationValueRef::UnresolvedLonghand(self.source.value_components())
            }
            EntryValue::Pending { direct: false } => {
                CssSpecifiedDeclarationValueRef::PendingShorthand(&self.source)
            }
        }
    }
    fn completed(&self) -> Option<CssContributionValueRef<'_>> {
        match self.value() {
            CssSpecifiedDeclarationValueRef::Completed(v) => Some(v),
            _ => None,
        }
    }
    pub(crate) fn at(&self, error: CssDeclarationBlockError) -> CssDeclarationBlockError {
        error.at(
            &self.source,
            self.authored_ordinal,
            Some(self.member_ordinal),
        )
    }
    fn append_value(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match &self.value {
            EntryValue::SvgGlyphOrientationVertical(c) => match c.global() {
                Some(keyword) => append_global(keyword, writer),
                None => c
                    .ordinary_value()
                    .expect("completed SVG ordinary value")
                    .append_to_rule_writer(writer),
            }
            .map_err(Into::into),
            EntryValue::Completed(c) => match c.value() {
                CssContributionValueRef::Ordinary(value) => {
                    super::specified_inverse::append_longhand(value, writer).map_err(Into::into)
                }
                CssContributionValueRef::Global(keyword) => {
                    append_global(keyword, writer).map_err(Into::into)
                }
                _ => Err(CssDeclarationBlockError::new(
                    CssDeclarationBlockErrorKind::UnresolvedEnvironment {
                        property: c.property(),
                    },
                )),
            },
            EntryValue::Universal(reset) => {
                append_global(reset.keyword(), writer).map_err(Into::into)
            }
            EntryValue::Custom | EntryValue::Pending { direct: true } => {
                append_retained_value(self.source.value_components(), writer).map_err(Into::into)
            }
            EntryValue::Pending { direct: false } => Ok(()),
        }
        .map_err(|error| self.at(error))
    }
}

/// Immutable CSSOM specified-order snapshot, separate from the authored occurrence list.
/// It does not apply selector specificity, cascade origins, writing modes or substitution.
#[derive(Clone, Debug)]
pub struct CssSpecifiedDeclarationBlock {
    entries: Vec<CssSpecifiedDeclarationEntry>,
    domain: Domain,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Key<'a> {
    Known(CssKnownProperty),
    SvgGlyphOrientationVertical,
    Custom(&'a crate::CssCustomPropertyName),
}
impl<'a> Key<'a> {
    fn from_entry(entry: &'a CssSpecifiedDeclarationEntry) -> Self {
        match entry.property_name() {
            CssPropertyNameRef::Known(property) => Self::Known(property),
            CssPropertyNameRef::SvgGlyphOrientationVertical => Self::SvgGlyphOrientationVertical,
            CssPropertyNameRef::Custom(name) => Self::Custom(name),
        }
    }
}

fn universe() -> &'static [Terminal] {
    static UNIVERSE: OnceLock<Vec<Terminal>> = OnceLock::new();
    UNIVERSE.get_or_init(|| {
        let mut properties: Vec<_> = SPECIFIED_TERMINALS
            .iter()
            .copied()
            .filter(|p| {
                !matches!(
                    p.known_property(),
                    CssKnownProperty::Direction | CssKnownProperty::UnicodeBidi
                )
            })
            .map(Terminal::Known)
            .collect();
        properties.extend_from_slice(svg_footprint());
        properties.sort_by_key(|p| {
            let bucket = match p.mapping().1 {
                MappingLogic::Logical => 0,
                MappingLogic::Neutral => 1,
                MappingLogic::Physical => 2,
            };
            (bucket, p.name())
        });
        properties
    })
}

fn footprint(source: &CssDeclaration) -> Result<Footprint> {
    if source.svg_glyph_orientation_vertical().is_some() {
        return Ok(Footprint::Supported(svg_footprint()));
    }
    let Some(known) = source.known() else {
        return Ok(Footprint::Canonical(&[]));
    };
    let meta = known.grammar().metadata().map_err(|_| capacity())?;
    match meta.kind() {
        CssPropertyKindRef::Longhand(meta) => {
            static TERMINALS: OnceLock<
                HashMap<CssLonghandProperty, &'static [CssLonghandProperty]>,
            > = OnceLock::new();
            Ok(Footprint::Canonical(
                TERMINALS
                    .get_or_init(|| {
                        SPECIFIED_TERMINALS
                            .iter()
                            .enumerate()
                            .map(|(index, &property)| {
                                (property, &SPECIFIED_TERMINALS[index..index + 1])
                            })
                            .collect()
                    })
                    .get(&meta.property())
                    .copied()
                    .expect("schema terminal"),
            ))
        }
        CssPropertyKindRef::Shorthand(meta) => Ok(Footprint::Canonical(meta.members())),
        CssPropertyKindRef::FourSideShorthand(_) => Err(CssDeclarationBlockError::new(
            CssDeclarationBlockErrorKind::PendingFootprintUndetermined {
                property: known.property(),
            },
        )),
        CssPropertyKindRef::UniversalReset(_) => Ok(Footprint::Supported(universe())),
    }
}

fn capacity() -> CssDeclarationBlockError {
    CssSpecifiedValueSerializationError::new(
        CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
    )
    .into()
}
fn reserve<T>(values: &mut Vec<T>, count: usize) -> Result<()> {
    values.try_reserve(count).map_err(|_| capacity())
}
fn visit_components(values: &CssComponentValues, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    let mut stack = Vec::new();
    reserve(&mut stack, 1)?;
    stack.push(values.items().iter());
    while let Some(frame) = stack.last_mut() {
        let Some(value) = frame.next() else {
            stack.pop();
            continue;
        };
        writer.context.charge_input(1)?;
        let children = match value.view() {
            CssComponentValueRef::Function(function) => Some(function.values()),
            CssComponentValueRef::Block(block) => Some(block.values()),
            _ => None,
        };
        if let Some(children) = children {
            // Closed component values already prove the owning 256-depth bound.
            reserve(&mut stack, 1)?;
            stack.push(children.items().iter());
        }
    }
    Ok(())
}

fn expansion_source(expansion: &CssExpansion) -> &CssDeclaration {
    match expansion {
        CssExpansion::Pending(pending) => pending.source(),
        CssExpansion::Contributions(CssContributions::Longhands(values)) => values
            .items()
            .first()
            .expect("closed expansion has members")
            .source(),
        CssExpansion::Contributions(CssContributions::UniversalReset(reset)) => reset.source(),
        CssExpansion::Contributions(CssContributions::Custom(custom)) => custom.source(),
        CssExpansion::Contributions(CssContributions::SvgGlyphOrientationVertical(value)) => {
            value.source()
        }
    }
}
fn group_count(expansion: &CssExpansion) -> Result<usize> {
    match expansion {
        CssExpansion::Pending(pending) => Ok(footprint(pending.source())?.len()),
        CssExpansion::Contributions(CssContributions::Longhands(values)) => {
            Ok(values.items().len())
        }
        CssExpansion::Contributions(CssContributions::UniversalReset(_)) => Ok(universe().len()),
        CssExpansion::Contributions(CssContributions::Custom(_)) => Ok(1),
        CssExpansion::Contributions(CssContributions::SvgGlyphOrientationVertical(_)) => Ok(1),
    }
}

impl CssSpecifiedDeclarationBlock {
    pub fn try_from_declarations(declarations: &CssDeclarationList) -> Result<Self> {
        Self::try_from_declarations_with_limits(
            declarations,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    /// Builds without emitting CSS bytes; its byte budget is unspent.
    pub fn try_from_declarations_with_limits(
        declarations: &CssDeclarationList,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        Self::from_sources(declarations.iter(), Domain::Ordinary, &mut writer)
    }
    pub fn try_from_keyframe_declarations(
        declarations: &CssKeyframeDeclarationList,
    ) -> Result<Self> {
        Self::try_from_keyframe_declarations_with_limits(
            declarations,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    /// Builds the closed keyframe-domain snapshot from genuine parser-retained sources.
    pub fn try_from_keyframe_declarations_with_limits(
        declarations: &CssKeyframeDeclarationList,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        Self::from_sources(
            declarations.iter().map(|d| d.source()),
            Domain::Keyframe,
            &mut writer,
        )
    }
    pub fn try_from_expansions(expansions: &[CssExpansion]) -> Result<Self> {
        Self::try_from_expansions_with_limits(
            expansions,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn try_from_expansions_with_limits(
        expansions: &[CssExpansion],
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        Self::from_expansions(expansions, Domain::Ordinary, &mut writer)
    }
    pub fn try_from_keyframe_expansions(expansions: &[CssExpansion]) -> Result<Self> {
        Self::try_from_keyframe_expansions_with_limits(
            expansions,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    /// Checks Normal importance and source admission before projecting completed replacements.
    pub fn try_from_keyframe_expansions_with_limits(
        expansions: &[CssExpansion],
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        Self::from_expansions(expansions, Domain::Keyframe, &mut writer)
    }
    /// Rebuilds the exact ordered selected terminal state without expanding its
    /// original shorthand sources. Duplicate identities are rejected atomically.
    /// Empty selection is valid; pending occurrence identity and source attribution
    /// remain the originals, independently of selected priority and order.
    pub fn try_from_entries(entries: &[CssSpecifiedDeclarationEntry]) -> Result<Self> {
        Self::try_from_entries_with_limits(entries, CssSpecifiedValueSerializationLimits::default())
    }
    /// Validates all selected input and projection work under one cumulative
    /// budget. No CSS bytes are emitted or values serialized and reparsed.
    pub fn try_from_entries_with_limits(
        entries: &[CssSpecifiedDeclarationEntry],
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self> {
        Self::from_selected_entries(entries, &mut SpecifiedRuleWriter::new(limits))
    }
    pub fn try_from_keyframe_entries(entries: &[CssSpecifiedDeclarationEntry]) -> Result<Self> {
        Self::try_from_keyframe_entries_with_limits(
            entries,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    /// Rejects terminal identities or selected priorities outside keyframe grammar.
    pub fn try_from_keyframe_entries_with_limits(
        entries: &[CssSpecifiedDeclarationEntry],
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self> {
        Self::from_entries(
            entries,
            Domain::Keyframe,
            &mut SpecifiedRuleWriter::new(limits),
        )
    }
    pub(crate) fn from_selected_entries(
        entries: &[CssSpecifiedDeclarationEntry],
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<Self> {
        Self::from_entries(entries, Domain::Ordinary, writer)
    }
    fn from_entries(
        entries: &[CssSpecifiedDeclarationEntry],
        domain: Domain,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<Self> {
        writer.node()?;
        let mut identities = HashMap::new();
        for entry in entries {
            let mut work = || -> Result<()> {
                Self::visit_source(entry.source(), Domain::Ordinary, writer)?;
                writer.context.charge_projection(1)?;
                if domain == Domain::Keyframe {
                    writer.context.charge_projection(1)?;
                    let reason = if entry.importance() != CssImportance::Normal {
                        Some(CssInvalidKeyframeSourceReason::Importance)
                    } else if entry.property.is_some_and(|p| !domain.admits_terminal(p)) {
                        Some(CssInvalidKeyframeSourceReason::Property)
                    } else {
                        None
                    };
                    if let Some(reason) = reason {
                        return Err(CssDeclarationBlockError::new(
                            CssDeclarationBlockErrorKind::InvalidKeyframeSource { reason },
                        ));
                    }
                }
                if matches!(domain, Domain::Page | Domain::Margin) {
                    writer.context.charge_projection(1)?;
                    let source = entry.source();
                    if entry.property.is_some_and(|p| !domain.admits_terminal(p))
                        || source
                            .known()
                            .is_some_and(|v| !crate::parser::is_page_margin_property(v.property()))
                        || source.svg_glyph_orientation_vertical().is_some()
                        || crate::parser::page_declaration_violation(
                            source.body(),
                            source.value_components(),
                        )
                        .is_some()
                    {
                        return Err(CssDeclarationBlockError::new(
                            CssDeclarationBlockErrorKind::InvalidPageSource,
                        ));
                    }
                }
                identities.try_reserve(1).map_err(|_| capacity())?;
                if identities.insert(Key::from_entry(entry), ()).is_some() {
                    return Err(CssDeclarationBlockError::new(
                        CssDeclarationBlockErrorKind::DuplicateTerminal,
                    ));
                }
                // Traverse the retained completed replacement, never the source
                // shorthand expansion, under the same semantic projection budget.
                writer.visit_semantic(|writer| match &entry.value {
                    EntryValue::Completed(c) => match c.value() {
                        CssContributionValueRef::Ordinary(value) => {
                            super::specified_inverse::append_longhand(value, writer)
                        }
                        CssContributionValueRef::Global(keyword) => append_global(keyword, writer),
                        _ => Ok(()),
                    },
                    EntryValue::SvgGlyphOrientationVertical(c) => match c.global() {
                        Some(keyword) => append_global(keyword, writer),
                        None => c
                            .ordinary_value()
                            .expect("completed SVG")
                            .append_to_rule_writer(writer),
                    },
                    EntryValue::Universal(reset) => append_global(reset.keyword(), writer),
                    EntryValue::Custom | EntryValue::Pending { .. } => Ok(()),
                })?;
                Ok(())
            };
            work().map_err(|error| entry.at(error))?;
        }
        let mut selected = Vec::new();
        reserve(&mut selected, entries.len())?;
        selected.extend_from_slice(entries);
        Ok(Self {
            entries: selected,
            domain,
        })
    }
    pub fn try_from_page_entries(entries: &[CssSpecifiedDeclarationEntry]) -> Result<Self> {
        Self::try_from_page_entries_with_limits(
            entries,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn try_from_page_entries_with_limits(
        entries: &[CssSpecifiedDeclarationEntry],
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self> {
        Self::from_page_entries(entries, &mut SpecifiedRuleWriter::new(limits))
    }
    pub(crate) fn from_page_entries(
        entries: &[CssSpecifiedDeclarationEntry],
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<Self> {
        Self::from_entries(entries, Domain::Page, writer)
    }
    pub fn try_from_margin_entries(entries: &[CssSpecifiedDeclarationEntry]) -> Result<Self> {
        Self::try_from_margin_entries_with_limits(
            entries,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn try_from_margin_entries_with_limits(
        entries: &[CssSpecifiedDeclarationEntry],
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self> {
        Self::from_margin_entries(entries, &mut SpecifiedRuleWriter::new(limits))
    }
    pub(crate) fn from_margin_entries(
        entries: &[CssSpecifiedDeclarationEntry],
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<Self> {
        Self::from_entries(entries, Domain::Margin, writer)
    }
    #[must_use]
    pub fn is_page(&self) -> bool {
        self.domain == Domain::Page
    }
    #[must_use]
    pub fn is_margin(&self) -> bool {
        self.domain == Domain::Margin
    }
    /// Whether this snapshot was checked in the keyframe declaration domain.
    #[must_use]
    pub fn is_keyframe(&self) -> bool {
        self.domain == Domain::Keyframe
    }
    pub(crate) fn is_ordinary(&self) -> bool {
        self.domain == Domain::Ordinary
    }
    #[must_use]
    pub fn entries(&self) -> &[CssSpecifiedDeclarationEntry] {
        &self.entries
    }

    fn visit_source(
        source: &CssDeclaration,
        domain: Domain,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<()> {
        writer.context.charge_input(2)?; // Occurrence and semantic name.
        visit_components(source.value_components(), writer)?;
        domain.admit_source(source, writer)
    }
    pub(crate) fn from_page_sources<'a>(
        sources: impl Iterator<Item = &'a CssDeclaration> + Clone,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<Self> {
        Self::from_sources(sources, Domain::Page, writer)
    }
    pub(crate) fn from_margin_sources<'a>(
        sources: impl Iterator<Item = &'a CssDeclaration> + Clone,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<Self> {
        Self::from_sources(sources, Domain::Margin, writer)
    }
    fn from_sources<'a>(
        sources: impl Iterator<Item = &'a CssDeclaration> + Clone,
        domain: Domain,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<Self> {
        writer.node()?; // Input aggregate.
        let mut count = 0usize;
        for (ordinal, source) in sources.clone().enumerate() {
            let mut work = || -> Result<usize> {
                Self::visit_source(source, domain, writer)?;
                let n = if source.custom().is_some() {
                    1
                } else if source
                    .known()
                    .is_some_and(|known| known.substitution_dependent().is_some())
                {
                    footprint(source)?.len()
                } else if source
                    .known()
                    .is_some_and(|known| known.property() == CssKnownProperty::All)
                {
                    universe().len()
                } else {
                    crate::expansion::expansion_member_count(source)?
                };
                writer.context.charge_projection(n)?;
                Ok(n)
            };
            let n = work().map_err(|e| e.at(source, ordinal, None))?;
            count = count.checked_add(n).ok_or_else(capacity)?;
        }
        let mut entries = Vec::new();
        reserve(&mut entries, count)?;
        for (ordinal, source) in sources.enumerate() {
            let expansion = crate::expand_declaration(source)
                .map_err(|e| CssDeclarationBlockError::from(e).at(source, ordinal, None))?;
            Self::append_group(&mut entries, &expansion, ordinal, domain, writer)?;
        }
        Self::winners(entries, domain, writer)
    }
    fn from_expansions(
        expansions: &[CssExpansion],
        domain: Domain,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<Self> {
        writer.node()?;
        let mut count = 0usize;
        for (ordinal, expansion) in expansions.iter().enumerate() {
            let source = expansion_source(expansion);
            let mut work = || -> Result<usize> {
                Self::visit_source(source, domain, writer)?;
                let n = group_count(expansion)?;
                writer.context.charge_projection(n)?;
                Ok(n)
            };
            let n = work().map_err(|e| e.at(source, ordinal, None))?;
            count = count.checked_add(n).ok_or_else(capacity)?;
        }
        let mut entries = Vec::new();
        reserve(&mut entries, count)?;
        for (ordinal, expansion) in expansions.iter().enumerate() {
            Self::append_group(&mut entries, expansion, ordinal, domain, writer)?;
        }
        Self::winners(entries, domain, writer)
    }
    fn append_group(
        entries: &mut Vec<CssSpecifiedDeclarationEntry>,
        expansion: &CssExpansion,
        ordinal: usize,
        domain: Domain,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<()> {
        let source = expansion_source(expansion);
        let mut append =
            |property: Option<Terminal>, member_ordinal: usize, value: EntryValue| -> Result<()> {
                if domain != Domain::Ordinary {
                    writer.context.charge_projection(1).map_err(|e| {
                        CssDeclarationBlockError::from(e).at(source, ordinal, Some(member_ordinal))
                    })?;
                    if property.is_some_and(|p| !domain.admits_terminal(p)) {
                        return Ok(());
                    }
                }
                entries.push(CssSpecifiedDeclarationEntry {
                    property,
                    source: source.clone(),
                    importance: source.importance(),
                    authored_ordinal: ordinal,
                    member_ordinal,
                    value,
                });
                Ok(())
            };
        match expansion {
            CssExpansion::Pending(_) => {
                let direct = if source.svg_glyph_orientation_vertical().is_some() {
                    true
                } else {
                    matches!(
                        source
                            .known()
                            .expect("known pending")
                            .grammar()
                            .metadata()
                            .map_err(|_| capacity())?
                            .kind(),
                        CssPropertyKindRef::Longhand(_)
                    )
                };
                for (member, property) in footprint(source)?.iter().enumerate() {
                    append(Some(property), member, EntryValue::Pending { direct })?;
                }
            }
            CssExpansion::Contributions(CssContributions::Custom(_)) => {
                append(None, 0, EntryValue::Custom)?
            }
            CssExpansion::Contributions(CssContributions::SvgGlyphOrientationVertical(value)) => {
                append(
                    Some(Terminal::SvgGlyphOrientationVertical),
                    0,
                    EntryValue::SvgGlyphOrientationVertical(value.clone()),
                )?;
            }
            CssExpansion::Contributions(CssContributions::UniversalReset(reset)) => {
                for (member, &property) in universe().iter().enumerate() {
                    append(Some(property), member, EntryValue::Universal(reset.clone()))?;
                }
            }
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                for (member, contribution) in values.items().iter().enumerate() {
                    let CssPropertyKindRef::Longhand(meta) = contribution
                        .property()
                        .metadata()
                        .map_err(|_| capacity())?
                        .kind()
                    else {
                        unreachable!("closed terminal contribution")
                    };
                    append(
                        Some(Terminal::Known(meta.property())),
                        member,
                        EntryValue::Completed(contribution.clone()),
                    )?;
                }
            }
        }
        Ok(())
    }
    fn winners(
        entries: Vec<CssSpecifiedDeclarationEntry>,
        domain: Domain,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<Self> {
        let mut index: HashMap<Key<'_>, usize> = HashMap::new();
        index.try_reserve(entries.len()).map_err(|_| capacity())?;
        let mut retained = Vec::new();
        reserve(&mut retained, entries.len())?;
        retained.resize(entries.len(), false);
        for (i, entry) in entries.iter().enumerate() {
            writer
                .context
                .charge_projection(1)
                .map_err(|e| entry.at(e.into()))?;
            let key = Key::from_entry(entry);
            let earlier = index.get(&key).copied();
            if earlier.is_none_or(|earlier| {
                entries[earlier].importance() != CssImportance::Important
                    || entry.importance() == CssImportance::Important
            }) {
                if let Some(earlier) = earlier {
                    retained[earlier] = false;
                }
                retained[i] = true;
                index.insert(key, i);
            }
        }
        let mut winners = Vec::new();
        reserve(&mut winners, index.len())?;
        drop(index);
        for (entry, retained) in entries.into_iter().zip(retained) {
            if retained {
                winners.push(entry);
            }
        }
        Ok(Self {
            entries: winners,
            domain,
        })
    }
}
