//! Checked authored container construction and lexical serialization.
use crate::component_values::CssCanonicalBuilder;
use crate::*;

fn construct<T: Send>(
    deep: bool,
    action: impl FnOnce() -> Result<T, CssContainerConstructionError> + Send,
) -> Result<T, CssContainerConstructionError> {
    if !deep {
        return action();
    }
    // Grammar recursion is bounded by the component ceiling. Keep the owned
    // closure on the caller until the larger-stack worker actually starts.
    let mut action = Some(action);
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("surgeist-css-container".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, || {
                action.take().expect("single construction worker")()
            })
            .map_err(|_| CssContainerConstructionError::WorkerUnavailable {
                origin: CssValueOrigin::Programmatic,
            })?;
        match worker.join() {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}

impl CssContainerCondition {
    /// Admits complete component syntax with the same grammar as parsed conditions.
    /// Explicit tokens, operators and grouping retain their supplied origins.
    pub fn try_from_components(
        values: CssComponentValues,
    ) -> Result<Self, CssContainerConstructionError> {
        Self::try_from_components_with_limits(values, CssComponentValueLimits::default())
    }
    /// Checks nesting, component count and serialized bytes before classification.
    /// Trusted parser-produced EOF closures retain their original provenance.
    pub fn try_from_components_with_limits(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssContainerConstructionError> {
        construct(values.nesting_depth() >= 32, || {
            crate::parser::construct_container_condition(values, limits)
        })
    }
    /// Classifies one lexical enclosure without reparsing, preserving trusted
    /// parser-produced EOF closures. Recognized branches precede opaque fallback.
    pub fn try_from_enclosed(
        enclosed: CssGeneralEnclosed,
    ) -> Result<Self, CssContainerConstructionError> {
        construct(
            crate::media::component_depth(enclosed.component()) >= 32,
            || crate::parser::container_condition_from_enclosed(enclosed),
        )
    }
    /// Serializes authored lexical spelling, operators, trivia and every grouping
    /// pair. Delimiters implied by parser recovery are emitted explicitly.
    pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> {
        self.serialize_with_limit(usize::MAX)
    }
    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        out.push_components(self.components())?;
        out.finish()
    }
}

impl CssContainerPrelude {
    /// Admits a complete, nonempty comma list with the stylesheet grammar.
    pub fn try_from_components(
        values: CssComponentValues,
    ) -> Result<Self, CssContainerConstructionError> {
        Self::try_from_components_with_limits(values, CssComponentValueLimits::default())
    }
    /// Bounds the complete input before entry admission, retaining supplied origins.
    pub fn try_from_components_with_limits(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssContainerConstructionError> {
        construct(values.nesting_depth() >= 32, || {
            crate::parser::construct_container_prelude(values, limits)
        })
    }
    /// Emits the complete authored prelude, including separators and grouping.
    pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> {
        self.serialize_with_limit(usize::MAX)
    }
    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        out.push_components(self.components())?;
        out.finish()
    }
}

impl CssContainerQueryEntry {
    /// Emits only this entry, preserving its name, query and supplied origins.
    pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> {
        self.serialize_with_limit(usize::MAX)
    }
    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        out.push_components(self.components())?;
        out.finish()
    }
}
