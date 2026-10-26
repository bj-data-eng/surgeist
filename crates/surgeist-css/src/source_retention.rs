//! Retained source admission over actual CSS payloads, independent of formatting.
use crate::*;

/// A finite resource of [`CssSourceRetention`] that can refuse a traversal.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssSourceRetentionResource {
    /// Nesting depth of retained payload owners.
    Depth,
    /// Payload observations.
    Visits,
    /// Retained source bytes.
    RetainedBytes,
    /// Collector bank storage.
    Storage,
}
impl CssSourceRetentionResource {
    /// Returns the resource's stable description, as error messages name it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Depth => "source depth",
            Self::Visits => "source visits",
            Self::RetainedBytes => "retained source bytes",
            Self::Storage => "source storage",
        }
    }
}

/// Finite failure while enumerating or retaining shared source backing.
///
/// A [`CssRetainedSources`] implementation outside CSS returns the first refusal it
/// receives unchanged, or builds one with [`Self::limit`], [`Self::allocation`] or
/// [`Self::caller_admission`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssSourceRetentionError {
    kind: SourceRetentionErrorKind,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceRetentionErrorKind {
    Limit(CssSourceRetentionResource),
    Allocation,
    CallerAdmission,
}
impl CssSourceRetentionError {
    /// A refusal because `resource` is exhausted.
    #[must_use]
    pub const fn limit(resource: CssSourceRetentionResource) -> Self {
        Self {
            kind: SourceRetentionErrorKind::Limit(resource),
        }
    }
    /// A refusal because storage could not be allocated.
    #[must_use]
    pub const fn allocation() -> Self {
        Self {
            kind: SourceRetentionErrorKind::Allocation,
        }
    }
    /// A refusal by the borrowed caller admission predicate; the collector is terminal.
    #[must_use]
    pub const fn caller_admission() -> Self {
        Self {
            kind: SourceRetentionErrorKind::CallerAdmission,
        }
    }
    /// Returns the exhausted resource of a limit refusal.
    #[must_use]
    pub const fn exhausted_limit(&self) -> Option<CssSourceRetentionResource> {
        match self.kind {
            SourceRetentionErrorKind::Limit(resource) => Some(resource),
            _ => None,
        }
    }
    crate::error::kind_predicates! { kind;
        /// Returns whether storage could not be allocated.
        is_allocation_failure => SourceRetentionErrorKind::Allocation,
        /// Returns whether the borrowed caller admission predicate refused.
        is_caller_admission => SourceRetentionErrorKind::CallerAdmission,
    }
}
impl std::fmt::Display for CssSourceRetentionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            SourceRetentionErrorKind::Limit(resource) => {
                write!(
                    f,
                    "CSS source retention {} limit exceeded",
                    resource.as_str()
                )
            }
            SourceRetentionErrorKind::Allocation => {
                f.write_str("CSS source retention allocation failed")
            }
            SourceRetentionErrorKind::CallerAdmission => {
                f.write_str("CSS source retention was refused by the caller's admission")
            }
        }
    }
}
impl std::error::Error for CssSourceRetentionError {}

/// Actual collector work or simultaneous temporary storage, separate from source payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssSourceRetentionAction {
    /// Admit the next observation before its operands are read.
    Step,
    /// Complete live collector metadata and source-reference bank bytes.
    /// During replacement this includes both old and requested/actual new banks.
    Scratch { live_bytes: usize },
}

struct Admission<'a> {
    callback: Option<&'a mut dyn FnMut(CssSourceRetentionAction) -> bool>,
    refused: bool,
}
impl Admission<'_> {
    fn check(&self) -> Result<(), CssSourceRetentionError> {
        if self.refused {
            Err(CssSourceRetentionError::caller_admission())
        } else {
            Ok(())
        }
    }
    fn action(&mut self, action: CssSourceRetentionAction) -> Result<(), CssSourceRetentionError> {
        self.check()?;
        if let Some(callback) = &mut self.callback
            && !callback(action)
        {
            self.refused = true;
            return Err(CssSourceRetentionError::caller_admission());
        }
        Ok(())
    }
}

/// Refusal while observing CSS-owned retained payloads.
///
/// The caller's admission refusal is returned unchanged through
/// [`Self::into_admission`]; consumers that measure their own payloads build
/// refusals with [`Self::admission`] and [`Self::capacity_overflow`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssRetainedPayloadError<E> {
    kind: RetainedPayloadErrorKind<E>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RetainedPayloadErrorKind<E> {
    Admission(E),
    CapacityOverflow,
}
impl<E> CssRetainedPayloadError<E> {
    /// The caller refused the next actual observation before its operands were read.
    #[must_use]
    pub const fn admission(error: E) -> Self {
        Self {
            kind: RetainedPayloadErrorKind::Admission(error),
        }
    }
    /// The measured byte sum cannot be represented by `usize`.
    #[must_use]
    pub const fn capacity_overflow() -> Self {
        Self {
            kind: RetainedPayloadErrorKind::CapacityOverflow,
        }
    }
    /// Returns the caller's admission refusal, or this error when the byte sum overflowed.
    ///
    /// # Errors
    ///
    /// Returns `self` unchanged when it is a capacity overflow.
    pub fn into_admission(self) -> Result<E, Self> {
        match self.kind {
            RetainedPayloadErrorKind::Admission(error) => Ok(error),
            kind @ RetainedPayloadErrorKind::CapacityOverflow => Err(Self { kind }),
        }
    }
    /// Returns whether the measured byte sum cannot be represented by `usize`.
    #[must_use]
    pub const fn is_capacity_overflow(&self) -> bool {
        matches!(self.kind, RetainedPayloadErrorKind::CapacityOverflow)
    }
}
impl<E: std::fmt::Display> std::fmt::Display for CssRetainedPayloadError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            RetainedPayloadErrorKind::Admission(error) => {
                write!(f, "CSS retained payload admission: {error}")
            }
            RetainedPayloadErrorKind::CapacityOverflow => {
                f.write_str("CSS retained payload capacity overflow")
            }
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for CssRetainedPayloadError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            RetainedPayloadErrorKind::Admission(error) => Some(error),
            RetainedPayloadErrorKind::CapacityOverflow => None,
        }
    }
}

/// CSS-owned source traversal. Implemented for retained CSS payload owners.
pub trait CssRetainedSources {
    #[doc(hidden)]
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError>;
}

/// One cumulative admission, deduplicated by actual immutable snapshot identity.
/// It does not count references held by callers or inspect Arc strong counts.
///
/// The borrowed predicate may capture thread-local owner state. Consequently the
/// collector is neither Send nor Sync, including values constructed with new.
/// It also makes no UnwindSafe or RefUnwindSafe guarantee. Source snapshots stay
/// independently shareable; this collector's lifetime/auto-trait contract does
/// not change their immutable ownership.
pub struct CssSourceRetention<'admit> {
    sources: Vec<CssSourceSnapshot>,
    usage: CssSourceStorageUsage,
    bytes: usize,
    visits: usize,
    maximum_visits: usize,
    depth: usize,
    maximum_depth: usize,
    admission: Admission<'admit>,
}
impl<'admit> CssSourceRetention<'admit> {
    #[must_use]
    pub const fn new(bytes: usize, maximum_visits: usize, maximum_depth: usize) -> Self {
        Self {
            sources: Vec::new(),
            usage: CssSourceStorageUsage(0),
            bytes,
            visits: 0,
            maximum_visits,
            depth: 0,
            maximum_depth,
            admission: Admission {
                callback: None,
                refused: false,
            },
        }
    }
    /// Construct a collector borrowing the caller's finite admission policy.
    ///
    /// The first Scratch action admits the collector's fixed metadata before
    /// formation. Later reports include actual bank capacity and replacement
    /// overlap. Keep that admission live until this collector has been dropped.
    /// A false predicate result is terminal, including when a visitor swallows
    /// the returned error. The predicate may retain its original typed error in
    /// caller-owned storage; no error allocation or erasure is required.
    pub fn with_admission(
        bytes: usize,
        maximum_visits: usize,
        maximum_depth: usize,
        admit: &'admit mut dyn FnMut(CssSourceRetentionAction) -> bool,
    ) -> Result<Self, CssSourceRetentionError> {
        if !admit(CssSourceRetentionAction::Scratch {
            live_bytes: std::mem::size_of::<Self>(),
        }) {
            return Err(CssSourceRetentionError::caller_admission());
        }
        let mut collector = Self::new(bytes, maximum_visits, maximum_depth);
        collector.admission.callback = Some(admit);
        Ok(collector)
    }
    pub fn visit<T: CssRetainedSources + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), CssSourceRetentionError> {
        self.charge_step()?;
        if self.depth >= self.maximum_depth {
            return Err(CssSourceRetentionError::limit(
                CssSourceRetentionResource::Depth,
            ));
        }
        self.depth += 1;
        let result = value.visit_retained_sources(self);
        self.depth -= 1;
        self.admission.check()?;
        result
    }
    /// Charges one actual consumer traversal action into this same work bank.
    pub fn charge_step(&mut self) -> Result<(), CssSourceRetentionError> {
        self.admission.action(CssSourceRetentionAction::Step)?;
        self.visits = self
            .visits
            .checked_add(1)
            .filter(|count| *count <= self.maximum_visits)
            .ok_or(CssSourceRetentionError::limit(
                CssSourceRetentionResource::Visits,
            ))?;
        Ok(())
    }
    pub fn source(&mut self, source: &CssSourceSnapshot) -> Result<(), CssSourceRetentionError> {
        self.charge_step()?;
        for index in 0..self.sources.len() {
            self.charge_step()?;
            if self.sources[index].same_snapshot(source) {
                return Ok(());
            }
        }
        let usage = self
            .usage
            .checked_add(source.storage_usage())
            .filter(|value| value.validate(self.bytes))
            .ok_or(CssSourceRetentionError::limit(
                CssSourceRetentionResource::RetainedBytes,
            ))?;
        if self.sources.len() == self.sources.capacity() {
            let required =
                self.sources
                    .len()
                    .checked_add(1)
                    .ok_or(CssSourceRetentionError::limit(
                        CssSourceRetentionResource::Storage,
                    ))?;
            let capacity = self
                .sources
                .capacity()
                .saturating_mul(2)
                .max(required)
                .min(self.maximum_visits);
            self.reserve_sources(capacity)?;
        }
        self.sources.push(source.clone());
        self.usage = usage;
        Ok(())
    }
    fn reserve_sources(&mut self, capacity: usize) -> Result<(), CssSourceRetentionError> {
        let scratch = |new_capacity: usize| {
            self.sources
                .capacity()
                .checked_add(new_capacity)
                .and_then(|count| count.checked_mul(std::mem::size_of::<CssSourceSnapshot>()))
                .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
                .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<CssSourceSnapshot>>()))
                .ok_or(CssSourceRetentionError::limit(
                    CssSourceRetentionResource::Storage,
                ))
        };
        self.admission.action(CssSourceRetentionAction::Scratch {
            live_bytes: scratch(capacity)?,
        })?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(capacity)
            .map_err(|_| CssSourceRetentionError::allocation())?;
        // An allocator may grant more capacity than requested. Admit that actual
        // simultaneous extent before copying/using it; refusal drops it while
        // the caller still retains the requested reservation and the old bank.
        self.admission.action(CssSourceRetentionAction::Scratch {
            live_bytes: scratch(replacement.capacity())?,
        })?;
        if replacement.capacity() > self.maximum_visits {
            return Err(CssSourceRetentionError::limit(
                CssSourceRetentionResource::Storage,
            ));
        }
        replacement.append(&mut self.sources);
        self.sources = replacement;
        Ok(())
    }

    /// Complete source payload in this retained snapshot-identity domain.
    ///
    /// Includes private source payload headers and backing, and each exact shared
    /// original UTF16 Vec header/backing once, even across distinct snapshots.
    /// Excludes this collector's own bank, opaque Arc control blocks and allocator
    /// overhead. Existing backing-only usage/quota and visit tariffs are unchanged.
    /// The caller admits each source observation and original-identity comparison
    /// before operands are read. No allocation or source-text comparison occurs.
    /// This read-only observation has its own explicit admission budget,
    /// independently of the optional traversal predicate and its terminal state.
    /// Use complete_payload_bytes_admitted to spend the stored predicate instead.
    pub fn complete_payload_bytes_with<E>(
        &self,
        mut admit: impl FnMut() -> Result<(), E>,
    ) -> Result<usize, CssRetainedPayloadError<E>> {
        complete_payload_bytes(&self.sources, &mut admit)
    }

    /// Measure complete source payload through this collector's same admission
    /// predicate, without advancing the traversal steps() counter. Each action
    /// precedes its observation/comparison. Scratch and source payload stay
    /// separate; a prior caller refusal prevents all observations/callbacks.
    pub fn complete_payload_bytes_admitted(
        &mut self,
    ) -> Result<usize, CssRetainedPayloadError<CssSourceRetentionError>> {
        let Self {
            sources, admission, ..
        } = self;
        complete_payload_bytes(sources, &mut || {
            admission.action(CssSourceRetentionAction::Step)
        })
    }

    #[must_use]
    pub const fn usage(&self) -> CssSourceStorageUsage {
        self.usage
    }

    #[must_use]
    pub const fn steps(&self) -> usize {
        self.visits
    }

    /// Visits payloads beneath an actual consumer-owned representation depth.
    /// This preserves accumulated work/source identity and restores only the
    /// enclosing depth on return, including failure.
    pub fn at_depth<T>(
        &mut self,
        depth: usize,
        visit: impl FnOnce(&mut Self) -> Result<T, CssSourceRetentionError>,
    ) -> Result<T, CssSourceRetentionError> {
        self.admission.check()?;
        if depth < self.depth || depth > self.maximum_depth {
            return Err(CssSourceRetentionError::limit(
                CssSourceRetentionResource::Depth,
            ));
        }
        let previous = self.depth;
        self.depth = depth;
        let result = visit(self);
        self.depth = previous;
        self.admission.check()?;
        result
    }
}
fn complete_payload_bytes<E>(
    sources: &[CssSourceSnapshot],
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<usize, CssRetainedPayloadError<E>> {
    admit().map_err(CssRetainedPayloadError::admission)?;
    let mut total = 0usize;
    for index in 0..sources.len() {
        admit().map_err(CssRetainedPayloadError::admission)?;
        let source = &sources[index];
        let mut include_original = true;
        if let Some(original) = source.original_utf16() {
            for previous in sources.iter().take(index) {
                admit().map_err(CssRetainedPayloadError::admission)?;
                if previous
                    .original_utf16()
                    .is_some_and(|previous| std::sync::Arc::ptr_eq(previous, original))
                {
                    include_original = false;
                    break;
                }
            }
        }
        admit().map_err(CssRetainedPayloadError::admission)?;
        total = total
            .checked_add(
                source
                    .payload_bytes(include_original)
                    .ok_or_else(CssRetainedPayloadError::capacity_overflow)?,
            )
            .ok_or_else(CssRetainedPayloadError::capacity_overflow)?;
    }
    Ok(total)
}

impl<T: CssRetainedSources> CssRetainedSources for [T] {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        for value in self {
            sources.visit(value)?;
        }
        Ok(())
    }
}
impl<T: CssRetainedSources> CssRetainedSources for Option<T> {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let Some(value) = self {
            sources.visit(value)?;
        }
        Ok(())
    }
}
impl<T: CssRetainedSources + ?Sized> CssRetainedSources for &T {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(*self)
    }
}
impl<T: CssRetainedSources + ?Sized> CssRetainedSources for Box<T> {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(self.as_ref())
    }
}
impl CssRetainedSources for CssParsedOrigin {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        sources.source(self.source())
    }
}
impl CssRetainedSources for CssValueOrigin {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Parsed(origin) => sources.visit(origin),
            Self::ImplicitClosure { opening, at } => {
                sources.visit(opening)?;
                sources.visit(at)
            }
            Self::Programmatic | Self::UnretainedInput { .. } => Ok(()),
        }
    }
}
impl CssRetainedSources for CssRecoveryDiagnostic {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let Some(value) = self.source() {
            sources.source(value)?;
        }
        sources.visit(self.error())
    }
}
impl CssRetainedSources for CssError {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let ErrorKind::InvalidComponentValue(value) = self.kind() {
            sources.visit(value.as_ref())?;
        }
        Ok(())
    }
}
impl CssRetainedSources for CssComponentValueError {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(self.origin())
    }
}
impl CssRetainedSources for CssComponentValue {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(self.origin())?;
        match self.view() {
            CssComponentValueRef::Function(value) => {
                sources.visit(value.values())?;
                sources.visit(value.closing_origin())?;
            }
            CssComponentValueRef::Block(value) => {
                sources.visit(value.values())?;
                sources.visit(value.closing_origin())?;
            }
            CssComponentValueRef::Token(_) | CssComponentValueRef::Comment(_) => {
                if let Some(origin) = self.implicit_termination_origin() {
                    sources.visit(origin)?;
                }
            }
        }
        Ok(())
    }
}
impl CssRetainedSources for CssComponentValues {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(self.items())
    }
}
impl CssRetainedSources for CssDeclaration {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let Some(value) = self.parsed_name() {
            sources.visit(value)?;
        }
        if let Some(value) = self.parsed_value() {
            sources.visit(value)?;
        }
        // The private payload constructors derive the semantic body from these
        // exact components; the semantic body cannot import an independent origin.
        sources.visit(self.value_components())
    }
}
impl CssRetainedSources for CssDeclarationList {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        for value in self.iter() {
            sources.visit(value)?;
        }
        Ok(())
    }
}
impl CssRetainedSources for CssSpecifiedDeclarationBlock {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        for value in self.entries() {
            sources.visit(value)?;
        }
        Ok(())
    }
}
impl CssRetainedSources for CssSheet {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let Some(value) = self.parse_source() {
            sources.source(value.source())?;
        }
        sources.visit(self.rules())
    }
}

macro_rules! via {
    ($ty:ty, $value:ident, $($field:expr),+ $(,)?) => {
        impl CssRetainedSources for $ty {
            fn visit_retained_sources(&$value, sources: &mut CssSourceRetention) -> Result<(), CssSourceRetentionError> {
                $(sources.visit($field)?;)+
                Ok(())
            }
        }
    };
}
via!(
    CssStyleRule,
    self,
    self.selectors(),
    self.declarations(),
    self.rules()
);
via!(CssNestedDeclarationsRule, self, self.declarations());
via!(CssMediaRule, self, self.query(), self.rules());
via!(CssSupportsRule, self, self.condition(), self.rules());
via!(CssContainerRule, self, self.prelude(), self.rules());
via!(CssLayerBlockRule, self, self.rules());
via!(
    CssCounterStyleDescriptorValue,
    self,
    &self.origin().cloned(),
    self.components()
);
via!(
    CssFontFeatureDisplayValue,
    self,
    &self.origin().cloned(),
    self.components()
);
via!(
    CssFontFeatureValue,
    self,
    &self.origin().cloned(),
    self.components()
);
via!(CssFontPaletteDescriptorValue, self, self.components());
via!(CssColorProfileDescriptorValue, self, self.components());
via!(CssFontPaletteDescriptor, self, self.value());
via!(CssColorProfileDescriptor, self, self.value());
via!(CssFontPaletteValuesRule, self, self.descriptors());
via!(CssColorProfileRule, self, self.descriptors());
via!(CssKeyframesRule, self, self.blocks());
via!(
    CssKeyframeBlock,
    self,
    self.selectors(),
    self.declarations()
);
via!(CssKeyframeSelectorList, self, self.selectors());
via!(CssStyleSelectorList, self, self.selectors());
via!(CssScopeSelectorList, self, self.selectors());
via!(CssRuleList, self, self.rules());
via!(CssFontFaceRule, self, self.descriptors());
via!(CssFontFaceDescriptor, self, self.value());
via!(CssPendingFontFaceDescriptorValue, self, self.components());
via!(CssKeyframeDeclaration, self, self.source());
via!(CssKeyframeDeclarationList, self, self.as_slice());
via!(CssStyleSelector, self, self.selector());
via!(CssRelativeSelector, self, self.selector());
via!(CssPageSelector, self, &self.origin());
via!(CssPageSelectorList, self, &self.origin(), self.selectors());
via!(
    CssPageDescriptorValue,
    self,
    self.origin(),
    self.components()
);
via!(CssMarginDeclarationBlock, self, self.properties());
via!(
    CssMarginRule,
    self,
    &self.detached_origin(),
    self.declarations()
);
via!(
    CssPageRule,
    self,
    self.selectors(),
    self.declarations(),
    self.margin_rules()
);
via!(
    CssCounterStyleRule,
    self,
    &self.parsed_name(),
    self.descriptors()
);
via!(
    CssFontFeatureValueDefinition,
    self,
    &self.parsed_name(),
    self.value()
);
via!(CssFontFeatureValueBlock, self, self.definitions());
via!(
    CssFontFeatureDisplayOccurrence,
    self,
    &self.parsed_name(),
    self.value()
);
via!(CssFontFeatureValuesRule, self, self.items());
via!(CssWhenRule, self, self.condition(), self.rules());
via!(CssElseRule, self, &self.condition(), self.rules());

impl<T: CssRetainedSources> CssRetainedSources for CssDescriptorOccurrence<T> {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let Some(value) = self.parsed_name() {
            sources.visit(value)?;
        }
        if let Some(value) = self.parsed_value() {
            sources.visit(value)?;
        }
        if let Some(value) = self.value_components() {
            sources.visit(value)?;
        }
        sources.visit(self.value())
    }
}
impl CssRetainedSources for CssCounterStyleDescriptors {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        for value in self.occurrences() {
            use CssCounterStyleDescriptorRef::*;
            match value {
                System(value)
                | Negative(value)
                | Symbols(value)
                | Prefix(value)
                | Suffix(value)
                | Range(value)
                | Pad(value)
                | Fallback(value)
                | AdditiveSymbols(value)
                | SpeakAs(value) => sources.visit(value)?,
            }
        }
        Ok(())
    }
}
impl CssRetainedSources for CssFontFaceDescriptors {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        for value in self.occurrences() {
            sources.visit(value)?;
        }
        Ok(())
    }
}
impl CssRetainedSources for CssKeyframeSelector {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let Self::Percent(value) = self
            && let Some(value) = value.calculation()
        {
            sources.visit(value.components())?;
        }
        Ok(())
    }
}
impl CssRetainedSources for CssFontFeatureValuesItem {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Block(value) => sources.visit(value),
            Self::FontDisplay(value) => sources.visit(value),
        }
    }
}
via!(CssPageDeclarationBlock, self, self.occurrences());
impl CssRetainedSources for CssPageDeclaration {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Property(value) => sources.visit(value),
            Self::Descriptor(value) => sources.visit(value),
        }
    }
}
impl CssRetainedSources for CssPageDescriptor {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let Some(value) = self.occurrence() {
            sources.visit(value)?;
        }
        sources.visit(self.value())
    }
}
impl CssRetainedSources for CssImportRule {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(&self.syntax.at_keyword)?;
        sources.visit(&self.syntax.prelude.target)?;
        sources.visit(&self.syntax.prelude.layer)?;
        sources.visit(&self.syntax.prelude.supports)?;
        sources.visit(&self.syntax.prelude.semicolon)?;
        if let CssImportTarget::Url(value) = self.target() {
            sources.visit(value.url())?;
        }
        if let Some(value) = self.supports() {
            sources.visit(value.condition())?;
        }
        if let Some(value) = self.media() {
            sources.visit(value)?;
        }
        Ok(())
    }
}
impl CssRetainedSources for CssMediaQueryList {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let Some(source) = self.parse_source() {
            sources.source(source.source())?;
        }
        sources.visit(self.queries())?;
        sources.visit(self.comma_origins.as_slice())
    }
}
impl CssRetainedSources for CssMediaQuery {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(self.origin())?;
        match self {
            Self::Condition(value) => sources.visit(value),
            Self::Typed(value) => {
                sources.visit(&value.syntax.modifier)?;
                sources.visit(&value.syntax.media_type)?;
                sources.visit(&value.syntax.conjunction)?;
                if let Some(value) = value.condition() {
                    sources.visit(value)?;
                }
                Ok(())
            }
            Self::Never(_) => Ok(()),
        }
    }
}
impl CssRetainedSources for CssAuthoredFontFaceDescriptorValue {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Ordinary(value) => sources.visit(value),
            Self::Pending(value) => sources.visit(value),
        }
    }
}
impl CssRetainedSources for CssFontFaceDescriptorValue {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::FontFamily(_) | Self::FontDisplay(_) | Self::UnicodeRange(_) => Ok(()),
            Self::FontNamedInstance(value) => {
                if let CssFontNamedInstance::String(value) = value {
                    sources.visit(value.component())?;
                }
                Ok(())
            }
            Self::FontLanguageOverride(value) => {
                if let CssFontLanguageOverride::String(value) = value {
                    sources.visit(value.component())?;
                }
                Ok(())
            }
            Self::Src(value) => {
                for value in value.sources() {
                    sources.charge_step()?;
                    if let CssFontFaceSource::Url(value) = value {
                        sources.visit(value.url())?;
                    }
                }
                Ok(())
            }
            Self::FontWeight(value) => {
                if let CssFontFaceWeight::Range { start, end } = value {
                    sources.visit(start)?;
                    sources.visit(end)?;
                }
                Ok(())
            }
            Self::FontStyle(value) => {
                if let CssFontFaceStyle::Oblique { range: Some(value) } = value {
                    sources.visit(value.start())?;
                    if let Some(value) = value.end() {
                        sources.visit(value)?;
                    }
                }
                Ok(())
            }
            Self::FontWidth(value) => {
                if let CssFontFaceWidth::Range { start, end } = value {
                    sources.visit(start)?;
                    sources.visit(end)?;
                }
                Ok(())
            }
            Self::FontFeatureSettings(value) => {
                if let CssAuthoredFontFeatureSettings::Features(values) = value {
                    for value in values.features() {
                        sources.charge_step()?;
                        if let CssAuthoredFontFeatureValue::Index(value) = value.value() {
                            sources.visit(value)?;
                        }
                    }
                }
                Ok(())
            }
            Self::FontVariationSettings(value) => {
                if let CssFontVariationSettings::Variations(values) = value {
                    for value in values.variations() {
                        sources.visit(value.value())?;
                    }
                }
                Ok(())
            }
            Self::AscentOverride(value)
            | Self::DescentOverride(value)
            | Self::LineGapOverride(value) => {
                if let CssFontMetricOverride::Percentage(value) = value {
                    sources.visit(value)?;
                }
                Ok(())
            }
        }
    }
}
impl CssRetainedSources for CssAbsoluteFontWeight {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Number(value) => sources.visit(value),
            Self::Normal | Self::Bold => Ok(()),
        }
    }
}
impl CssRetainedSources for CssFontWidth {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Percentage(value) => sources.visit(value),
            Self::Keyword(_) => Ok(()),
        }
    }
}
impl CssRetainedSources for CssUrl {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        for value in self.modifiers() {
            sources.charge_step()?;
            match value {
                CssUrlModifier::Function(value) => sources.visit(value.argument_components())?,
                CssUrlModifier::Ident(_) => {}
            }
        }
        Ok(())
    }
}
impl<T: crate::CssNumericRoot, R: crate::CssRange<T>> CssRetainedSources
    for crate::CssNumeric<T, R>
{
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let Some(value) = self.literal() {
            sources.visit(value.component())?;
        }
        if let Some(value) = self.calculation() {
            sources.visit(value.components())?;
        }
        Ok(())
    }
}
via!(CssSpecifiedFontFaceDeclarationBlock, self, self.entries());
via!(CssSpecifiedPageDeclarationBlock, self, self.entries());
via!(
    CssSerializationError,
    self,
    &self.origin(),
    &self.component_error()
);
via!(CssFontFeatureValuesFamilyList, self, self.origin());
impl CssRetainedSources for CssSpecifiedPageDeclarationEntry {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Property(value) => sources.visit(value),
            Self::Descriptor(value) => sources.visit(value),
        }
    }
}
impl CssRetainedSources for CssDeclarationBlockError {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let Some(value) = self.declaration() {
            sources.visit(value)?;
        }
        if let Some(value) = self.serialization() {
            sources.visit(value)?;
        }
        if let Some(value) = self
            .expansion()
            .and_then(CssExpansionError::replacement_error)
        {
            sources.visit(&value.origin())?;
        }
        Ok(())
    }
}
impl CssRetainedSources for CssSerializedOrigin {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Token(value) => sources.visit(value),
            Self::Separator { before, after } => {
                sources.visit(before)?;
                sources.visit(after)
            }
            Self::End(value) => sources.visit(value),
        }
    }
}
impl CssRetainedSources for CssPageProjectionError {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let Some(value) = self.properties() {
            sources.visit(value)?;
        }
        if let Some(value) = self.resource() {
            sources.visit(value)?;
        }
        Ok(())
    }
}
impl CssRetainedSources for CssFontFaceDeclarationBlockError {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        match self.serialization() {
            Some(value) => sources.visit(value),
            None => Ok(()),
        }
    }
}

impl CssRetainedSources for CssScopeRule {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        if let Some(value) = self.root() {
            sources.visit(value)?;
        }
        if let Some(value) = self.limit() {
            sources.visit(value)?;
        }
        sources.visit(self.rules())
    }
}
impl CssRetainedSources for CssRule {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::CustomMedia(value) => sources.visit(value),
            Self::Import(value) => sources.visit(value),
            // These owners contain decoded Strings and positions, no source handles.
            Self::Namespace(_) | Self::LayerStatement(_) => Ok(()),
            Self::CounterStyle(value) => sources.visit(value),
            Self::Page(value) => sources.visit(value),
            Self::LayerBlock(value) => sources.visit(value),
            Self::FontFace(value) => sources.visit(value),
            Self::FontFeatureValues(value) => sources.visit(value),
            Self::FontPaletteValues(value) => sources.visit(value),
            Self::ColorProfile(value) => sources.visit(value),
            Self::Keyframes(value) => sources.visit(value),
            Self::Style(value) => sources.visit(value),
            Self::NestedDeclarations(value) => sources.visit(value),
            Self::Media(value) => sources.visit(value),
            Self::Supports(value) => sources.visit(value),
            Self::SupportsCondition(value) => sources.visit(value),
            Self::Container(value) => sources.visit(value),
            Self::When(value) => sources.visit(value),
            Self::Else(value) => sources.visit(value),
            Self::Scope(value) => sources.visit(value),
        }
    }
}
#[cfg(test)]
mod admission_tests;
