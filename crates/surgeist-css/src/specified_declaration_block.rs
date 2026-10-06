//! Immutable CSSOM specified-order projection of authored declaration occurrences.

use std::{collections::HashMap, fmt, sync::OnceLock};

use crate::{
    CssComponentValueRef, CssComponentValues, CssContributionValueRef, CssContributions,
    CssCustomDeclaration, CssDeclaration, CssDeclarationList, CssExpansion, CssExpansionError,
    CssGlobalKeyword, CssImportance, CssKeyframeDeclarationList, CssKnownProperty,
    CssLonghandContribution, CssLonghandProperty, CssPropertyKindRef, CssPropertyNameRef,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssSvgGlyphOrientationVerticalContribution,
    CssUniversalReset,
    declaration_serialization::{append_global, append_retained_value},
    expansion::{MappingLogic, SPECIFIED_TERMINALS},
    specified_rule_serialization::SpecifiedRuleWriter,
};

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
}
impl Domain {
    fn admits(self, property: CssKnownProperty) -> bool {
        self == Self::Ordinary || crate::parser::keyframe_property_admitted(property)
    }
    fn admits_terminal(self, property: Terminal) -> bool {
        match property {
            Terminal::Known(property) => self.admits(property.known_property()),
            Terminal::SvgGlyphOrientationVertical => true,
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
        self.source.importance()
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

#[derive(Debug)]
struct Candidate {
    property: CssKnownProperty,
    members: Footprint,
    mode: Option<crate::CssBoxSideKind>,
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

fn candidates() -> &'static [Candidate] {
    static CANDIDATES: OnceLock<Vec<Candidate>> = OnceLock::new();
    CANDIDATES.get_or_init(|| {
        let mut result = Vec::new();
        for &property in CssKnownProperty::all() {
            match property.metadata().expect("schema metadata").kind() {
                CssPropertyKindRef::Shorthand(meta) if !meta.is_legacy() => {
                    result.push(Candidate {
                        property,
                        members: Footprint::Canonical(meta.members()),
                        mode: None,
                    })
                }
                CssPropertyKindRef::FourSideShorthand(meta) => {
                    for mode in [
                        crate::CssBoxSideKind::Physical,
                        crate::CssBoxSideKind::Logical,
                    ] {
                        result.push(Candidate {
                            property,
                            members: Footprint::Canonical(meta.members(mode)),
                            mode: Some(mode),
                        });
                    }
                }
                CssPropertyKindRef::UniversalReset(_) => result.push(Candidate {
                    property,
                    members: Footprint::Supported(universe()),
                    mode: None,
                }),
                _ => {}
            }
        }
        // CSSOM's printed stable preference transformations, in their stated order.
        result.sort_by_key(|c| c.property.canonical_name());
        result.sort_by_key(|c| c.property.canonical_name().starts_with('-'));
        result.sort_by_key(|c| {
            let name = c.property.canonical_name();
            name.starts_with('-') && !name.starts_with("-webkit-")
        });
        result.sort_by_key(|c| std::cmp::Reverse(c.members.len()));
        result
    })
}

fn candidates_for_member(property: Terminal) -> &'static [&'static Candidate] {
    static INDEX: OnceLock<HashMap<Terminal, Vec<&'static Candidate>>> = OnceLock::new();
    INDEX
        .get_or_init(|| {
            let mut index: HashMap<_, Vec<_>> = HashMap::new();
            for candidate in candidates() {
                for member in candidate.members.iter() {
                    index.entry(member).or_default().push(candidate);
                }
            }
            index
        })
        .get(&property)
        .map_or(&[], Vec::as_slice)
}
fn candidates_for_name(property: CssKnownProperty) -> &'static [&'static Candidate] {
    static INDEX: OnceLock<HashMap<CssKnownProperty, Vec<&'static Candidate>>> = OnceLock::new();
    INDEX
        .get_or_init(|| {
            let mut index: HashMap<_, Vec<_>> = HashMap::new();
            for candidate in candidates() {
                index.entry(candidate.property).or_default().push(candidate);
            }
            index
        })
        .get(&property)
        .map_or(&[], Vec::as_slice)
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
                if domain == Domain::Keyframe {
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

    pub fn serialize_cssom(&self) -> Result<String> {
        self.serialize_cssom_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_cssom_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_cssom(&mut writer)?;
        Ok(writer.css)
    }
    /// Reads a terminal or exact requested shorthand; absence/nonrepresentability is None.
    pub fn property_value(&self, name: CssPropertyNameRef<'_>) -> Result<Option<String>> {
        self.property_value_with_limits(name, CssSpecifiedValueSerializationLimits::default())
    }
    pub fn property_value_with_limits(
        &self,
        name: CssPropertyNameRef<'_>,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Option<String>> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.node()?;
        if let Some(entry) = self
            .entries
            .iter()
            .find(|entry| entry.property_name() == name)
        {
            entry.append_value(&mut writer)?;
            return Ok(Some(writer.css));
        }
        if let CssPropertyNameRef::Known(property) = name
            && self.domain.admits(property)
        {
            let mut serialized = Vec::new();
            reserve(&mut serialized, self.entries.len())?;
            serialized.resize(self.entries.len(), false);
            for &candidate in candidates_for_name(property) {
                let inverse = self.inverse(candidate, &serialized, &mut writer).map_err(
                    |error| match self.entries.iter().find(|entry| {
                        entry
                            .property
                            .is_some_and(|p| candidate.members.contains(&p))
                    }) {
                        Some(entry) => entry.at(error),
                        None => error,
                    },
                )?;
                if let Some(value) = inverse {
                    value.append(&mut writer).map_err(|error| {
                        match self.entries.iter().find(|entry| {
                            entry
                                .property
                                .is_some_and(|p| candidate.members.contains(&p))
                        }) {
                            Some(entry) => entry.at(error.into()),
                            None => error.into(),
                        }
                    })?;
                    return Ok(Some(writer.css));
                }
            }
        }
        Ok(None)
    }

    pub(crate) fn append_cssom(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        let mut serialized = Vec::new();
        reserve(&mut serialized, self.entries.len())?;
        serialized.resize(self.entries.len(), false);
        let mut emitted = false;
        for (index, entry) in self.entries.iter().enumerate() {
            if serialized[index] {
                continue;
            }
            let mut work = || -> Result<()> {
                let mut found = None;
                if let Some(property) = entry.property {
                    for &candidate in candidates_for_member(property) {
                        if let Some(value) = self.inverse(candidate, &serialized, writer)? {
                            found = Some((candidate, value));
                            break;
                        }
                    }
                }
                if emitted {
                    writer.append(" ")?;
                }
                writer.node()?;
                writer.node()?; // Emitted occurrence and semantic name.
                if let Some((candidate, value)) = found {
                    writer.append(candidate.property.canonical_name())?;
                    writer.append(": ")?;
                    value.append(writer)?;
                    for (i, member) in self.entries.iter().enumerate() {
                        if member
                            .property
                            .is_some_and(|p| candidate.members.contains(&p))
                        {
                            serialized[i] = true;
                        }
                    }
                } else {
                    match entry.property_name() {
                        CssPropertyNameRef::Known(property) => {
                            writer.append(property.canonical_name())?
                        }
                        CssPropertyNameRef::SvgGlyphOrientationVertical => writer.append(
                            crate::CssSvgGlyphOrientationVerticalDeclaration::metadata().name(),
                        )?,
                        CssPropertyNameRef::Custom(name) => {
                            writer.append_identifier(name.as_str())?
                        }
                    }
                    writer.append(": ")?;
                    entry.append_value(writer)?;
                    serialized[index] = true;
                }
                if entry.importance() == CssImportance::Important {
                    writer.append(" !important")?;
                }
                writer.append(";")?;
                emitted = true;
                Ok(())
            };
            work().map_err(|e| entry.at(e))?;
        }
        Ok(())
    }

    fn inverse(
        &self,
        candidate: &Candidate,
        serialized: &[bool],
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<Option<Inverse>> {
        writer.context.charge_projection(1)?;
        if !self.domain.admits(candidate.property) {
            return Ok(None);
        }
        let mut members = Vec::new();
        reserve(&mut members, candidate.members.len())?;
        let mut absent = false;
        for property in candidate.members.iter() {
            // Filtered keyframe All uses the same finite universe on both sides.
            if !self.domain.admits_terminal(property) {
                continue;
            }
            let present = self
                .entries
                .iter()
                .enumerate()
                .find(|(_, entry)| entry.property == Some(property));
            writer
                .context
                .charge_projection(1)
                .map_err(|error| match present {
                    Some((_, entry)) => entry.at(error.into()),
                    None => error.into(),
                })?;
            if let Some((index, entry)) = present {
                if serialized[index] {
                    absent = true;
                } else {
                    members.push((index, entry));
                }
            } else {
                absent = true;
            }
        }
        if absent || members.is_empty() {
            return Ok(None);
        }
        members.sort_unstable_by_key(|(index, _)| *index);
        let importance = members[0].1.importance();
        let mut mixed = false;
        for (_, entry) in &members {
            writer
                .context
                .charge_projection(1)
                .map_err(|error| entry.at(error.into()))?;
            mixed |= entry.importance() != importance;
        }
        if mixed {
            return Ok(None);
        }
        let first = members.iter().map(|m| m.0).min().expect("members");
        let last = members.iter().map(|m| m.0).max().expect("members");
        let mut interference = false;
        for entry in self.entries.get(first + 1..last).unwrap_or(&[]) {
            writer
                .context
                .charge_projection(1)
                .map_err(|error| entry.at(error.into()))?;
            if let Some(property) = entry.property
                && !candidate.members.contains(&property)
            {
                let (group, mapping) = property.mapping();
                interference |= group.is_some()
                    && members.iter().any(|(_, m)| {
                        let (other_group, other_mapping) =
                            m.property.expect("known member").mapping();
                        group == other_group && mapping != other_mapping
                    });
            }
        }
        if interference {
            return Ok(None);
        }
        writer.context.charge_projection(1)?; // Enter representability probe.
        let mut values = Vec::new();
        reserve(&mut values, members.len())?;
        values.extend(members.iter().map(|(_, e)| *e));
        let global = values[0].completed().and_then(|v| match v {
            CssContributionValueRef::Global(k) => Some(k),
            _ => None,
        });
        if let Some(keyword) = global {
            // Whole-value CSS-wide mode-switch grammar always selects physical members.
            if candidate.mode == Some(crate::CssBoxSideKind::Logical) {
                return Ok(None);
            }
            writer.context.charge_projection(1 + values.len())?;
            writer.context.charge_projection(1 + values.len())?;
            for entry in &values {
                writer
                    .context
                    .charge_projection(3)
                    .map_err(|e| entry.at(e.into()))?;
                if entry.completed() != Some(CssContributionValueRef::Global(keyword)) {
                    return Ok(None);
                }
            }
            return Ok(Some(Inverse::Global(keyword)));
        }
        if let EntryValue::Pending { direct: false } = values[0].value {
            let source = values[0].source();
            if !source
                .known()
                .is_some_and(|known| known.grammar() == candidate.property.grammar())
            {
                return Ok(None);
            }
            writer.context.charge_projection(1)?;
            // The retained component owner counts the hypothetical child tree;
            // no supplied value is parsed, expanded or copied into a second source.
            writer.visit_semantic(|writer| {
                append_retained_value(source.value_components(), writer)
            })?;
            writer.context.charge_projection(1 + values.len())?;
            for entry in &values {
                writer
                    .context
                    .charge_projection(3)
                    .map_err(|e| entry.at(e.into()))?;
                if !matches!(entry.value, EntryValue::Pending { direct: false })
                    || !entry.source().same_occurrence(source)
                {
                    return Ok(None);
                }
            }
            return Ok(Some(Inverse::Pending(source.clone())));
        }
        if candidate.property == CssKnownProperty::All {
            return Ok(None);
        }
        let Some(value) = super::specified_inverse::reconstruct(
            candidate.property,
            candidate.mode,
            &values,
            writer,
        )?
        else {
            return Ok(None);
        };
        let generated_count = candidate.members.len();
        writer.context.charge_projection(1 + generated_count)?; // Check expansion and every terminal before allocation.
        let check = value.projected(generated_count)?;
        for check_value in &check {
            if !self.domain.admits(check_value.property()) {
                continue;
            }
            let Some(entry) = values.iter().find(|entry| {
                entry.property_name() == CssPropertyNameRef::Known(check_value.property())
            }) else {
                return Ok(None);
            };
            writer
                .context
                .charge_projection(1)
                .map_err(|error| entry.at(error.into()))?; // Comparison initiation.
            // Exact typed equality; semantic traversals are charged by the inverse owner.
            if let Some(value) = entry.completed() {
                super::specified_inverse::visit_value(value, writer)
                    .map_err(|error| entry.at(error.into()))?;
            } else {
                return Ok(None);
            }
            super::specified_inverse::visit_value(check_value.view(), writer)
                .map_err(|error| entry.at(error.into()))?;
            let Some(original) = entry.completed() else {
                return Ok(None);
            };
            if !super::specified_inverse::values_equal(
                original,
                check_value.view(),
                &mut writer.context,
            )
            .map_err(|error| entry.at(error.into()))?
            {
                return Ok(None);
            }
        }
        let mut current = None;
        let (_, css) = writer
            .capture_value(|writer| {
                super::specified_inverse::append_reconstructed(
                    &value,
                    candidate.property,
                    candidate.mode,
                    &values,
                    writer,
                    &mut current,
                )
            })
            .map_err(|error| match current {
                Some(entry) => entry.at(error.into()),
                None => error.into(),
            })?;
        if css.is_empty() && !writer.context.output_suppressed() {
            return Ok(None);
        }
        Ok(Some(Inverse::Ordinary(css)))
    }
}

enum Inverse {
    Global(CssGlobalKeyword),
    Pending(CssDeclaration),
    Ordinary(String),
}
impl Inverse {
    fn append(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> std::result::Result<(), CssSpecifiedValueSerializationError> {
        match self {
            Self::Global(keyword) => append_global(*keyword, writer),
            Self::Pending(source) => append_retained_value(source.value_components(), writer),
            Self::Ordinary(value) => writer.append(value),
        }
    }
}

impl SpecifiedRuleWriter {
    /// Meaningful composed rule caller: construction and output share this request.
    pub(crate) fn append_cssom_declaration_list(
        &mut self,
        declarations: &CssDeclarationList,
    ) -> Result<()> {
        CssSpecifiedDeclarationBlock::from_sources(declarations.iter(), Domain::Ordinary, self)?
            .append_cssom(self)
    }
    pub(crate) fn append_cssom_keyframe_declaration_list(
        &mut self,
        declarations: &CssKeyframeDeclarationList,
    ) -> Result<()> {
        CssSpecifiedDeclarationBlock::from_sources(
            declarations.iter().map(|d| d.source()),
            Domain::Keyframe,
            self,
        )?
        .append_cssom(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(source: &str) -> CssSpecifiedDeclarationBlock {
        let report = crate::parse_style_attribute(source);
        assert!(report.is_clean());
        CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap()
    }
    fn kind(error: CssDeclarationBlockError) -> CssSpecifiedValueSerializationErrorKind {
        let CssDeclarationBlockErrorKind::Serialization(error) = error.kind else {
            panic!("resource error")
        };
        error.kind()
    }

    #[test]
    fn private_probe_has_independent_success_missing_and_mixed_tariffs() {
        let candidate = candidates_for_name(CssKnownProperty::Pause)[0];
        // Successful None pair: P18 intrinsic probe + I2/P2 actual pair
        // provider (pair + before leaf), bounded temporary "none" = four bytes.
        let full = block("pause-before:none;pause-after:none");
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(2, 20, 4));
        let Some(Inverse::Ordinary(value)) = full
            .inverse(candidate, &[false, false], &mut writer)
            .unwrap()
        else {
            panic!("finite inverse")
        };
        assert_eq!(value, "none");
        assert!(writer.css.is_empty());
        for (limits, expected) in [
            (
                CssSpecifiedValueSerializationLimits::new(1, 20, 4),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(2, 19, 4),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(2, 20, 3),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(
                kind(
                    full.inverse(candidate, &[false, false], &mut writer)
                        .err()
                        .unwrap()
                ),
                expected
            );
            assert!(writer.css.is_empty());
        }
        for (source, serialized, tariff) in [
            ("pause-before:none", vec![false], 3),
            (
                "pause-before:none!important;pause-after:none",
                vec![false, false],
                5,
            ),
        ] {
            let value = block(source);
            let mut writer =
                SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(0, tariff, 0));
            assert!(
                value
                    .inverse(candidate, &serialized, &mut writer)
                    .unwrap()
                    .is_none()
            );
            let mut writer = SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(
                0,
                tariff - 1,
                0,
            ));
            assert_eq!(
                kind(
                    value
                        .inverse(candidate, &serialized, &mut writer)
                        .err()
                        .unwrap()
                ),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            );
        }
    }

    #[test]
    fn pending_probe_requires_genuine_same_original_even_for_equal_text() {
        let report = crate::parse_style_attribute("pause:var(--P);pause:var(--P)");
        assert!(report.is_clean());
        let list = report.syntax();
        let mut value = CssSpecifiedDeclarationBlock::try_from_declarations(list).unwrap();
        assert!(!list[0].same_occurrence(&list[1]));
        value.entries[0].source = list[0].clone();
        value.entries[0].authored_ordinal = 0;
        let candidate = candidates_for_name(CssKnownProperty::Pause)[0];
        let mut writer = SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::default());
        assert!(
            value
                .inverse(candidate, &[false, false], &mut writer)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            value.serialize_cssom().unwrap(),
            "pause-before: ; pause-after: ;"
        );
        assert!(value.entries[0].source.same_occurrence(&list[0]));
        assert!(value.entries[1].source.same_occurrence(&list[1]));
    }

    #[test]
    fn private_math_probe_charges_every_real_generated_arena_node() {
        let value = block("pause-before:calc(1s + 2s);pause-after:calc(1s + 2s)");
        let candidate = candidates_for_name(CssKnownProperty::Pause)[0];
        // Complete intrinsic work P66 includes three four-node comparison
        // work lists; owning pair I5/P4 adds the actual final numeric arena.
        // Each math visit retains Calc + Sum + two leaves and arena nodes3.
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(5, 70, 8));
        let Some(Inverse::Ordinary(css)) = value
            .inverse(candidate, &[false, false], &mut writer)
            .unwrap()
        else {
            panic!("finite math inverse")
        };
        assert_eq!(css, "calc(3s)");
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(5, 69, 8));
        assert_eq!(
            kind(
                value
                    .inverse(candidate, &[false, false], &mut writer)
                    .err()
                    .unwrap()
            ),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
        assert!(writer.css.is_empty());
    }

    #[test]
    fn private_math_probe_admits_shortening_and_both_verification_work_lists() {
        let value = block("pause-before:calc(1s + 2s);pause-after:calc(1s + 2s)");
        let candidate = candidates_for_name(CssKnownProperty::Pause)[0];
        let sources: Vec<_> = value
            .entries
            .iter()
            .map(|entry| entry.source().clone())
            .collect();
        // Eligibility and probe entry P6. Hypothetical aggregate1 + two calc
        // visits (four source nodes + three generated arena nodes each) + one
        // four-node shortening comparison = P19. Check expansion P3. Each
        // verification costs initiation1 + original7 + generated7 + four
        // comparison slots = P19, twice. Final pair provider I5/P4 and bounded
        // temporary "calc(3s)" B8. Total P6+19+3+38+4 = P70.
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(5, 70, 8));
        let Some(Inverse::Ordinary(css)) = value
            .inverse(candidate, &[false, false], &mut writer)
            .unwrap()
        else {
            panic!("finite math inverse")
        };
        assert_eq!(css, "calc(3s)");
        assert!(writer.css.is_empty());
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(5, 69, 8));
        assert_eq!(
            kind(
                value
                    .inverse(candidate, &[false, false], &mut writer)
                    .err()
                    .unwrap()
            ),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
        assert!(writer.css.is_empty());
        let mut retry =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(5, 70, 8));
        let Some(Inverse::Ordinary(css)) = value
            .inverse(candidate, &[false, false], &mut retry)
            .unwrap()
        else {
            panic!("adequate retry")
        };
        assert_eq!(css, "calc(3s)");
        assert!(retry.css.is_empty());
        for (entry, source) in value.entries.iter().zip(&sources) {
            assert!(entry.source().same_occurrence(source));
        }
    }

    #[test]
    fn grid_area_prefix_search_pays_for_each_reached_integer_comparison() {
        let value = block(
            "grid-row-start:calc(1 + 2);grid-column-start:calc(1 + 2);grid-row-end:calc(1 + 2);grid-column-end:calc(1 + 2)",
        );
        let candidate = candidates_for_name(CssKnownProperty::GridArea)[0];
        let sources: Vec<_> = value
            .entries
            .iter()
            .map(|entry| entry.source().clone())
            .collect();
        // Eligibility P12. Hypothetical aggregate1 + four numeric visits7 =
        // P29. Prefix0 stops at Auto versus Indexed; prefixes1/2/3 reach 1/2/3
        // matching four-node integer graphs, P24. Check expansion P5. Four
        // member checks cost initiation1 + original7 + generated7 + equality4
        // = P76. The actual GridArea provider has no aggregate visit; its four
        // integer calculations cost I16/P12 and its value owns B37. Total
        // P12+29+24+5+76+12 = P158.
        let exact = CssSpecifiedValueSerializationLimits::new(16, 158, 37);
        let mut writer = SpecifiedRuleWriter::new(exact);
        let Some(Inverse::Ordinary(css)) =
            value.inverse(candidate, &[false; 4], &mut writer).unwrap()
        else {
            panic!("four explicit grid lines")
        };
        assert_eq!(css, "calc(3) / calc(3) / calc(3) / calc(3)");
        assert!(writer.css.is_empty());
        for (limits, expected) in [
            (
                CssSpecifiedValueSerializationLimits::new(15, 158, 37),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(16, 157, 37),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(16, 158, 36),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(
                kind(
                    value
                        .inverse(candidate, &[false; 4], &mut writer)
                        .err()
                        .unwrap()
                ),
                expected
            );
            assert!(writer.css.is_empty());
            let mut retry = SpecifiedRuleWriter::new(exact);
            let Some(Inverse::Ordinary(css)) =
                value.inverse(candidate, &[false; 4], &mut retry).unwrap()
            else {
                panic!("adequate grid retry")
            };
            assert_eq!(css, "calc(3) / calc(3) / calc(3) / calc(3)");
            assert!(retry.css.is_empty());
        }
        for (entry, source) in value.entries.iter().zip(&sources) {
            assert!(entry.source().same_occurrence(source));
        }
    }

    #[test]
    fn borrowed_grid_prefixes_keep_name_auto_and_span_partners_and_actual_error_sources() {
        let candidate = candidates_for_name(CssKnownProperty::GridArea)[0];
        for (source, expected) in [
            ("grid-area:header", "header"),
            ("grid-area:auto", "auto"),
            ("grid-area:span 2 / auto / auto / auto", "span 2"),
            (
                "grid-area:span calc(1 + 2) / auto / auto / auto",
                "span calc(3)",
            ),
            (
                "grid-area:header / other / header / other",
                "header / other",
            ),
            ("grid-area:1 / auto / 2 / auto", "1 / auto / 2"),
            ("grid-area:span 2 / span 3 / auto / auto", "span 2 / span 3"),
        ] {
            let value = block(source);
            let mut writer =
                SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::default());
            let Some(Inverse::Ordinary(css)) =
                value.inverse(candidate, &[false; 4], &mut writer).unwrap()
            else {
                panic!("borrowed grid partner: {source}")
            };
            assert_eq!(css, expected, "{source}");
            assert!(writer.css.is_empty());
        }
        let report = crate::parse_style_attribute(
            "grid-row-start:calc(1 + 2);grid-column-start:calc(1 + 2);grid-row-end:calc(1 + 2);grid-column-end:calc(1 + 2)",
        );
        assert!(report.is_clean());
        let list = report.syntax();
        let value = CssSpecifiedDeclarationBlock::try_from_declarations(list).unwrap();
        // Prefix comparison starts after eligibility12+hypothetical29=P41.
        // P42 reaches first C root; P50 reaches prefix2 E root; P62 reaches
        // prefix3 F root. Each next child admission fails at that actual member.
        for (projections, ordinal) in [(42, 1), (50, 2), (62, 3)] {
            let mut writer = SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(
                0,
                projections,
                0,
            ));
            let error = value
                .inverse(candidate, &[false; 4], &mut writer)
                .err()
                .unwrap();
            assert!(error.declaration().unwrap().same_occurrence(&list[ordinal]));
            assert_eq!(error.authored_ordinal(), Some(ordinal));
            assert_eq!(error.member_ordinal(), Some(0));
            assert_eq!(
                kind(error),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            );
            assert!(writer.css.is_empty());
        }
    }

    #[test]
    fn retained_unequal_time_pairs_admit_the_emission_comparison_work() {
        // The unequal second remains Some after the four-node shortening
        // comparison. The existing aggregate emitter compares that retained
        // value again; its four work-list slots are part of this new consumer.
        // P6 eligibility + P19 hypothetical/shortening + P3 check expansion +
        // P38 verification + P11 final (aggregate1, two numeric arenas3, and
        // the retained comparison4) = P77. Final visits I1+4+4 = I9, B17.
        let exact = CssSpecifiedValueSerializationLimits::new(9, 77, 17);
        let mut failures = Vec::new();
        for (property, source) in [
            (
                CssKnownProperty::Pause,
                "pause-before:calc(1s + 2s);pause-after:calc(1s + 3s)",
            ),
            (
                CssKnownProperty::Rest,
                "rest-before:calc(1s + 2s);rest-after:calc(1s + 3s)",
            ),
            (
                CssKnownProperty::InterestDelay,
                "interest-delay-start:calc(1s + 2s);interest-delay-end:calc(1s + 3s)",
            ),
        ] {
            let value = block(source);
            let sources: Vec<_> = value
                .entries
                .iter()
                .map(|entry| entry.source().clone())
                .collect();
            let candidate = candidates_for_name(property)[0];
            let mut writer = SpecifiedRuleWriter::new(exact);
            let Some(Inverse::Ordinary(css)) =
                value.inverse(candidate, &[false; 2], &mut writer).unwrap()
            else {
                panic!("retained second time value: {source}")
            };
            assert_eq!(css, "calc(3s) calc(4s)");
            assert!(writer.css.is_empty());
            let mut writer =
                SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(9, 76, 17));
            match value.inverse(candidate, &[false; 2], &mut writer) {
                Err(error) => {
                    if kind(error) != CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit {
                        failures.push(format!(
                            "{source}: one-under P76 returned a different resource error"
                        ));
                    }
                }
                _ => failures.push(format!(
                    "{source}: one-under P76 did not reject projection work"
                )),
            }
            assert!(writer.css.is_empty());
            let mut retry = SpecifiedRuleWriter::new(exact);
            let Some(Inverse::Ordinary(css)) =
                value.inverse(candidate, &[false; 2], &mut retry).unwrap()
            else {
                panic!("adequate retained-pair retry")
            };
            assert_eq!(css, "calc(3s) calc(4s)");
            assert!(retry.css.is_empty());
            for (entry, source) in value.entries.iter().zip(&sources) {
                assert!(entry.source().same_occurrence(source));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn private_color_probe_charges_the_actual_exact_coefficient_allocations() {
        let report = crate::parse_style_attribute("color:color(srgb 1 0 0)");
        assert!(report.is_clean());
        let crate::CssKnownPropertyValueRef::Color(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("color payload")
        };
        // One Color and three direct channel visits = P4. The nonzero channel
        // actually materializes its one-limb coefficient, multiplication and
        // exact-text clone = P3. Zero channels own no coefficient limbs.
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(0, 7, 0));
        writer
            .visit_semantic(|writer| {
                value
                    .value()
                    .append_specified(&mut writer.context, &mut writer.css)
            })
            .unwrap();
        assert!(writer.css.is_empty());
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(0, 6, 0));
        assert_eq!(
            writer
                .visit_semantic(|writer| value
                    .value()
                    .append_specified(&mut writer.context, &mut writer.css))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
        assert!(writer.css.is_empty());
    }
}
