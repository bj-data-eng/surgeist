//! CSS owns source payload traversal and accounting; this adapter supplies the
//! actual retained canonical tables, including detached objects and edited data.
use crate::{model::State, *};
use surgeist_css::rule::CssImportTarget;
use surgeist_css::source::{
    CssRetainedSources, CssSourceRetention, CssSourceRetentionError, CssSourceRetentionResource,
};

impl CssRetainedSources for CssomAuthoredRule {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Rule(value) => sources.visit(value),
            Self::Keyframe(value) => sources.visit(value),
            Self::Margin(value) => sources.visit(value),
            Self::RawNestedDeclarations(value) => sources.visit(value),
            Self::SupportsTestChild(value) => sources.visit(value),
        }
    }
}
impl CssRetainedSources for CssomGroupPrelude {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Supports(value) => sources.visit(value),
            Self::Container(value) => sources.visit(value),
            Self::Scope { start, end } => {
                sources.visit(start)?;
                sources.visit(end)
            }
            Self::When(value) => sources.visit(value),
            Self::Else(value) => sources.visit(value),
            Self::Media(_) | Self::Layer(_) => Ok(()), // list identity / decoded strings
        }
    }
}
impl CssRetainedSources for CssomRuleData {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::NamedSupports { name, body, .. } => {
                sources.visit(name.component())?;
                sources.visit(body)
            }
            Self::SupportsTestChild(value) => sources.visit(value),
            Self::Style { selectors, .. } => sources.visit(selectors),
            Self::Group { prelude, .. } => sources.visit(prelude),
            Self::Page { selectors, .. } => sources.visit(selectors),
            Self::Keyframe { selectors, .. } => sources.visit(selectors),
            Self::Import {
                target,
                supports,
                origin,
                ..
            } => {
                sources.visit(origin)?;
                if let CssImportTarget::Url(value) = target {
                    sources.visit(value.url())?;
                }
                if let Some(value) = supports {
                    sources.visit(value.condition())?;
                }
                Ok(())
            }
            Self::FontFeatureValues {
                family_input,
                font_display,
                ..
            } => {
                if let Some(value) = family_input {
                    sources.visit(value.value())?;
                    sources.visit(value.diagnostics())?;
                }
                sources.visit(font_display.as_slice())
            }
            Self::CustomMedia { name, .. } => sources.visit(name.component()),
            Self::Leaf { current } => sources.visit(current),
            Self::NestedDeclarations { .. }
            | Self::Margin { .. }
            | Self::Keyframes { .. }
            | Self::CounterStyle { .. }
            | Self::FontFace { .. } => Ok(()), // references / decoded names
        }
    }
}
impl<T: CssRetainedSources, E: CssRetainedSources> CssRetainedSources for CssomProjection<T, E> {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Available(value) => sources.visit(value),
            Self::Unavailable(value) => sources.visit(value),
        }
    }
}
impl CssRetainedSources for CssomPropertyOccurrences {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Ordinary(value) | Self::RawContents(value) => sources.visit(value),
            Self::Keyframe(value) => sources.visit(value),
            Self::Margin(value) => sources.visit(value),
        }
    }
}
impl CssRetainedSources for CssomBlockData {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Properties {
                authored, selected, ..
            } => {
                sources.visit(authored)?;
                sources.visit(selected)
            }
            Self::Page { authored, selected } => {
                sources.visit(authored)?;
                sources.visit(selected)
            }
            Self::CounterStyle(value) => sources.visit(value.as_ref()),
            Self::FontFace(value) => sources.visit(value),
        }
    }
}
impl State {
    pub(crate) fn check_source_limits(&self, limits: &CssomLimits) -> Result<(), CssomError> {
        self.source_traversal_steps(limits).map(|_| ())
    }
    fn source_traversal_steps(&self, limits: &CssomLimits) -> Result<usize, CssomError> {
        let mut sources = CssSourceRetention::new(
            limits.max_retained_source_bytes,
            limits.max_source_traversal_steps,
            limits.max_source_traversal_depth,
        );
        let result = (|| {
            let mut pending = Vec::new();
            sources.charge_step()?;
            reserve(&mut pending, limits.max_source_traversal_steps)?;
            pending.push((self, 0usize));
            let mut seen = Vec::<&State>::new();
            while let Some((state, depth)) = pending.pop() {
                sources.charge_step()?;
                let mut duplicate = false;
                for old in &seen {
                    sources.charge_step()?;
                    if std::ptr::eq(*old, state) {
                        duplicate = true;
                        break;
                    }
                }
                if duplicate {
                    continue;
                }
                reserve(&mut seen, limits.max_source_traversal_steps)?;
                seen.push(state);
                sources.at_depth(depth, |sources| {
                    enqueue(
                        state.context.as_ref(),
                        &mut pending,
                        sources,
                        depth,
                        limits.max_source_traversal_steps,
                    )?;
                    for sheet in state.sheets.values() {
                        sources.charge_step()?;
                        sources.visit(sheet.authored.as_ref())?;
                        sources.visit(sheet.diagnostics.as_slice())?;
                        sources.visit(&sheet.inputs.media)?;
                    }
                    for rule in state.rules.values() {
                        sources.charge_step()?;
                        if let Some(context) = rule.selector_inputs.as_deref() {
                            enqueue(
                                context,
                                &mut pending,
                                sources,
                                depth,
                                limits.max_source_traversal_steps,
                            )?;
                        }
                        sources.visit(rule.authored.as_ref())?;
                        sources.visit(rule.data.as_ref())?;
                        sources.visit(&rule.selector_input)?;
                        sources.visit(rule.admission_diagnostics.as_slice())?;
                    }
                    for block in state.blocks.values() {
                        sources.charge_step()?;
                        if let Some(context) = block.inline_context() {
                            enqueue(
                                context.context(),
                                &mut pending,
                                sources,
                                depth,
                                limits.max_source_traversal_steps,
                            )?;
                        }
                        sources.visit(block.data.as_ref())?;
                        sources.visit(block.counter_edits.as_slice())?;
                        if let Some(value) = &block.replacement_input {
                            if let Some(source) = value.parse_source() {
                                sources.source(source.source())?;
                            }
                            sources.visit(value.diagnostics())?;
                        }
                        if let Some(value) = &block.selected_font_face {
                            sources.visit(value.as_ref())?;
                        }
                    }
                    for media in state.media.values() {
                        sources.charge_step()?;
                        sources.visit(media)?;
                    }
                    for map in state.maps.values() {
                        sources.charge_step()?;
                        sources.visit(map.authored.as_slice())?;
                        for entry in &map.entries {
                            sources.charge_step()?;
                            if let CssomFeatureValue::Authored(value) = &entry.value {
                                sources.visit(value)?;
                            }
                        }
                    }
                    Ok(())
                })?;
            }
            Ok(sources.steps())
        })();
        result.map_err(|error: CssSourceRetentionError| {
            assert!(
                !error.is_caller_admission(),
                "standalone Source collector has no caller admission predicate"
            );
            match error.exhausted_limit() {
                Some(CssSourceRetentionResource::RetainedBytes) => CssomError::Limit {
                    resource: "retained parse source bytes",
                    maximum: limits.max_retained_source_bytes,
                },
                Some(CssSourceRetentionResource::Depth) => CssomError::Limit {
                    resource: "retained source traversal depth",
                    maximum: limits.max_source_traversal_depth,
                },
                // An allocation refusal or exhausted collector storage.
                Some(CssSourceRetentionResource::Storage) | None => CssomError::Limit {
                    resource: "retained source traversal storage",
                    maximum: limits.max_source_traversal_steps,
                },
                Some(_) => CssomError::Limit {
                    resource: "retained source traversal steps",
                    maximum: limits.max_source_traversal_steps,
                },
            }
        })
    }
}

fn reserve<T>(values: &mut Vec<T>, maximum: usize) -> Result<(), CssSourceRetentionError> {
    if values.len() >= maximum {
        return Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::Storage,
        ));
    }
    if values.len() == values.capacity() {
        let required = values
            .len()
            .checked_add(1)
            .ok_or(CssSourceRetentionError::limit(
                CssSourceRetentionResource::Storage,
            ))?;
        let capacity = values
            .capacity()
            .saturating_mul(2)
            .max(required)
            .min(maximum);
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|_| CssSourceRetentionError::allocation())?;
    }
    if values.capacity() > maximum {
        return Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::Storage,
        ));
    }
    Ok(())
}

fn enqueue<'a>(
    context: &'a CssomContext,
    pending: &mut Vec<(&'a State, usize)>,
    sources: &mut CssSourceRetention,
    depth: usize,
    maximum: usize,
) -> Result<(), CssSourceRetentionError> {
    sources.charge_step()?;
    for linked in &context.linked {
        sources.charge_step()?;
        reserve(pending, maximum)?;
        pending.push((
            linked.snapshot.state.as_ref(),
            depth.checked_add(1).ok_or(CssSourceRetentionError::limit(
                CssSourceRetentionResource::Depth,
            ))?,
        ));
    }
    Ok(())
}
