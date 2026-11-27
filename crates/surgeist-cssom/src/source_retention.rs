//! CSS owns source payload traversal and accounting; this adapter supplies the
//! actual retained canonical tables, including detached objects and edited data.
//!
//! The retained sources of a state are accounted record by record: each record
//! is measured alone, and what it retains is added to, or taken from, a
//! ledger of the distinct snapshots the state retains. A [`Measurer`] walks one
//! record at a time and collects the change it makes to that ledger.
use crate::part::Part;
use crate::*;
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::rule::CssImportTarget;
use surgeist_css::source::{
    CssRetainedSources, CssSourceBounds, CssSourceDelta, CssSourceMeter, CssSourceRetention,
    CssSourceRetentionError, CssSourceRetentionResource,
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

/// What a walk of one record does: it charges steps and reports the payloads
/// the record retains.
///
/// A collector runs it directly. [`Account`] runs it against the measurements
/// of the payload parts a record holds, which are kept while the parts are shared.
pub(crate) trait Walk {
    /// Charges one step of the walk's own traversal.
    fn charge(&mut self) -> Result<(), CssSourceRetentionError>;
    /// Visits a payload the record holds.
    fn visit<T: CssRetainedSources + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), CssSourceRetentionError>;
    /// Visits a payload part the record shares, with `visit`.
    fn part<T>(
        &mut self,
        part: &Part<T>,
        visit: impl Fn(&mut CssSourceRetention<'_>, &T) -> Result<(), CssSourceRetentionError>,
    ) -> Result<(), CssSourceRetentionError>;
}

impl Walk for CssSourceRetention<'_> {
    fn charge(&mut self) -> Result<(), CssSourceRetentionError> {
        self.charge_step()
    }
    fn visit<T: CssRetainedSources + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), CssSourceRetentionError> {
        CssSourceRetention::visit(self, value)
    }
    fn part<T>(
        &mut self,
        part: &Part<T>,
        visit: impl Fn(&mut CssSourceRetention<'_>, &T) -> Result<(), CssSourceRetentionError>,
    ) -> Result<(), CssSourceRetentionError> {
        visit(self, part)
    }
}

/// A record whose retained sources a walk reaches.
pub(crate) trait Walkable {
    fn walk<W: Walk>(&self, walker: &mut W) -> Result<(), CssSourceRetentionError>;
}
impl Walkable for CssomSheet {
    fn walk<W: Walk>(&self, walker: &mut W) -> Result<(), CssSourceRetentionError> {
        walk_sheet(walker, self)
    }
}
impl Walkable for CssomRule {
    fn walk<W: Walk>(&self, walker: &mut W) -> Result<(), CssSourceRetentionError> {
        walk_rule(walker, self)
    }
}
impl Walkable for CssomBlock {
    fn walk<W: Walk>(&self, walker: &mut W) -> Result<(), CssSourceRetentionError> {
        walk_block(walker, self)
    }
}
impl Walkable for CssMediaQueryList {
    fn walk<W: Walk>(&self, walker: &mut W) -> Result<(), CssSourceRetentionError> {
        walk_media(walker, self)
    }
}
impl Walkable for CssomFeatureMap {
    fn walk<W: Walk>(&self, walker: &mut W) -> Result<(), CssSourceRetentionError> {
        walk_map(walker, self)
    }
}
/// A media list a batch has displaced, which is only visited.
pub(crate) struct Displaced<'a>(pub(crate) &'a CssMediaQueryList);
impl Walkable for Displaced<'_> {
    fn walk<W: Walk>(&self, walker: &mut W) -> Result<(), CssSourceRetentionError> {
        walker.visit(self.0)
    }
}

/// Whether a walk adds what a record retains to the ledger, or takes it out.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Sign {
    Add,
    Remove,
}

/// What the walk of one record measured.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Walked {
    /// The steps the record charges: its own, and those of the payloads it holds.
    pub(crate) steps: usize,
    /// The deepest nesting below the record.
    pub(crate) reach: usize,
}

/// A walk of one record: the steps and nesting of the record are summed from the
/// measurements of its parts and of the payloads it holds directly, and what it
/// retains goes into `delta`, or comes out of it.
///
/// Each payload is measured under the limits of one payload: the steps of the
/// traversal and its depth. A payload that needs more steps than the limit allows
/// is measured again without that bound, and the walk goes on without it, so that
/// the refusal can name the first limit of the fixed order the record exceeds: the
/// depth is refused before the retained bytes, and those before the steps. The
/// walk of a refused record is bounded by the size of what the record retains.
struct Account<'a> {
    delta: &'a mut CssSourceDelta,
    meter: &'a mut CssSourceMeter,
    /// Measures the payloads that need more steps than the limit allows.
    lifted_meter: &'a mut CssSourceMeter,
    /// The bounds a payload is measured under: no bound on bytes, which the ledger answers to.
    bounds: CssSourceBounds,
    lifted_bounds: CssSourceBounds,
    sign: Sign,
    steps: usize,
    reach: usize,
    /// The steps this walk measured itself, which a kept measurement did not.
    work: usize,
    /// Whether the steps limit has been exceeded and the walk goes on without it.
    lifted: bool,
    /// The refusal that lifted the steps limit.
    refusal: Option<CssSourceRetentionError>,
}

impl Account<'_> {
    /// Whether `error` is a refusal for steps that the walk can go on after.
    fn liftable(&self, error: &CssSourceRetentionError) -> bool {
        self.sign == Sign::Add
            && matches!(
                error.exhausted_limit(),
                Some(CssSourceRetentionResource::Visits | CssSourceRetentionResource::Storage)
            )
    }
    /// Notes the refusal for steps that the walk goes on after.
    fn lift(&mut self, error: CssSourceRetentionError) {
        self.lifted = true;
        self.refusal.get_or_insert(error);
    }
    /// Refuses the walk once it passes the steps or the depth the limits allow a record.
    fn within(&mut self) -> Result<(), CssSourceRetentionError> {
        if self.sign == Sign::Remove {
            return Ok(());
        }
        if self.reach > self.bounds.depth() {
            return Err(CssSourceRetentionError::limit(
                CssSourceRetentionResource::Depth,
            ));
        }
        if !self.lifted && self.steps > self.bounds.steps() {
            self.lift(CssSourceRetentionError::limit(
                CssSourceRetentionResource::Visits,
            ));
        }
        Ok(())
    }
    /// The refusal the walk ends with: none, or the steps it went on after.
    fn finish(
        self,
        walked: Result<(), CssSourceRetentionError>,
    ) -> Result<(), CssSourceRetentionError> {
        walked?;
        self.refusal.map_or(Ok(()), Err)
    }
}

impl Walk for Account<'_> {
    fn charge(&mut self) -> Result<(), CssSourceRetentionError> {
        self.steps = self.steps.saturating_add(1);
        self.work = self.work.saturating_add(1);
        self.within()
    }
    fn visit<T: CssRetainedSources + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), CssSourceRetentionError> {
        let mut measured = if self.lifted {
            self.lifted_meter.measure(0, |sources| sources.visit(value))
        } else {
            self.meter.measure(0, |sources| sources.visit(value))
        };
        let mut meter: &CssSourceMeter = if self.lifted {
            self.lifted_meter
        } else {
            self.meter
        };
        if let Err(error) = measured
            && self.liftable(&error)
        {
            self.lift(error);
            measured = self.lifted_meter.measure(0, |sources| sources.visit(value));
            meter = self.lifted_meter;
        }
        measured?;
        self.steps = self.steps.saturating_add(meter.steps());
        self.work = self.work.saturating_add(meter.steps());
        self.reach = self.reach.max(meter.depth());
        match self.sign {
            Sign::Add => self.delta.add_metered(meter)?,
            Sign::Remove => self.delta.remove_metered(meter)?,
        }
        self.within()
    }
    fn part<T>(
        &mut self,
        part: &Part<T>,
        visit: impl Fn(&mut CssSourceRetention<'_>, &T) -> Result<(), CssSourceRetentionError>,
    ) -> Result<(), CssSourceRetentionError> {
        let measurement = match part.measurement(self.bounds, &visit) {
            Err(error) if self.liftable(&error) => {
                self.lift(error);
                part.measurement(self.lifted_bounds, &visit)?
            }
            other => other?,
        };
        self.steps = self.steps.saturating_add(measurement.steps());
        self.reach = self.reach.max(measurement.depth());
        match self.sign {
            Sign::Add => self.delta.add(measurement)?,
            Sign::Remove => self.delta.remove(measurement)?,
        }
        self.within()
    }
}

/// Measures the records a settle touches one after another on one reused
/// collector, and collects the change they make to the ledger of retained snapshots.
///
/// A record is measured from its kept parts and from the payloads it holds
/// directly. The change to the ledger is the net of everything added and removed, so
/// a record replaced by one that retains the same snapshots leaves it unchanged.
pub(crate) struct Measurer {
    meter: CssSourceMeter,
    lifted_meter: CssSourceMeter,
    delta: CssSourceDelta,
    bounds: CssSourceBounds,
    lifted_bounds: CssSourceBounds,
    work: usize,
}

impl Measurer {
    pub(crate) fn new(limits: &CssomLimits) -> Self {
        Self::bounded(
            limits.max_source_traversal_steps,
            limits.max_source_traversal_depth,
        )
    }

    /// A measurer for a record the state admitted, which no limit bounds.
    pub(crate) fn unbounded() -> Self {
        Self::bounded(usize::MAX, usize::MAX)
    }

    fn bounded(steps: usize, depth: usize) -> Self {
        let bounds = CssSourceBounds::new(usize::MAX, steps, depth);
        let lifted_bounds = CssSourceBounds::new(usize::MAX, usize::MAX, depth);
        Self {
            meter: CssSourceMeter::new(bounds),
            lifted_meter: CssSourceMeter::new(lifted_bounds),
            delta: CssSourceDelta::new(),
            bounds,
            lifted_bounds,
            work: 0,
        }
    }

    /// Walks `record`, adding what it retains to the change to the ledger or taking it out.
    ///
    /// # Errors
    ///
    /// The refusal of the walk: a payload that needs more nesting than a record may, or
    /// more steps. Where a record exceeds both, the nesting is the refusal.
    pub(crate) fn walk(
        &mut self,
        sign: Sign,
        record: &impl Walkable,
    ) -> Result<Walked, CssSourceRetentionError> {
        let mut account = Account {
            delta: &mut self.delta,
            meter: &mut self.meter,
            lifted_meter: &mut self.lifted_meter,
            bounds: self.bounds,
            lifted_bounds: self.lifted_bounds,
            sign,
            steps: 0,
            reach: 0,
            work: 0,
            lifted: false,
            refusal: None,
        };
        let walked = record.walk(&mut account);
        let (steps, reach, work) = (account.steps, account.reach, account.work);
        let walked = account.finish(walked);
        self.work = self.work.saturating_add(work);
        walked.map(|()| Walked { steps, reach })
    }

    /// The change to the ledger the records walked so far make.
    pub(crate) fn delta(&mut self) -> &mut CssSourceDelta {
        &mut self.delta
    }

    /// The steps the walks measured themselves, which a kept measurement did not.
    pub(crate) const fn work(&self) -> usize {
        self.work
    }
}

/// The resources of the refusals that report an exhausted source collector.
pub(crate) const BYTES_RESOURCE: &str = "retained parse source bytes";
pub(crate) const DEPTH_RESOURCE: &str = "retained source traversal depth";
pub(crate) const STORAGE_RESOURCE: &str = "retained source traversal storage";
pub(crate) const STEPS_RESOURCE: &str = "retained source traversal steps";

/// Charges the context of a record, and each input it links.
fn charge_context<W: Walk>(
    sources: &mut W,
    context: &CssomContext,
) -> Result<(), CssSourceRetentionError> {
    sources.charge()?;
    for _ in &context.linked {
        sources.charge()?;
    }
    Ok(())
}

/// Charges and visits the sources of one sheet record.
pub(crate) fn walk_sheet<W: Walk>(
    sources: &mut W,
    sheet: &CssomSheet,
) -> Result<(), CssSourceRetentionError> {
    let CssomSheet {
        inputs,
        rules: _,
        media: _,
        owner_rule: _,
        namespaces: _,
        authored,
        diagnostics,
    } = sheet;
    let CssomSheetInputs {
        version: _,
        base_url: _,
        location: _,
        constructor_document: _,
        owner: _,
        title: _,
        alternate: _,
        disabled: _,
        origin_clean: _,
        constructed: _,
        disallow_modification: _,
        media,
    } = inputs;
    sources.charge()?;
    sources.part(authored, |sources, value| sources.visit(value))?;
    sources.part(diagnostics, |sources, value| {
        sources.visit(value.as_slice())
    })?;
    sources.visit(media)
}

/// Charges and visits the sources of one rule record.
pub(crate) fn walk_rule<W: Walk>(
    sources: &mut W,
    rule: &CssomRule,
) -> Result<(), CssSourceRetentionError> {
    let CssomRule {
        parent: _,
        data,
        authored,
        insertion_origin,
        selector_input,
        selector_inputs,
        selector_namespaces: _,
        admission_diagnostics,
    } = rule;
    sources.charge()?;
    if let Some(context) = selector_inputs.as_deref() {
        charge_context(sources, context)?;
    }
    sources.part(authored, |sources, value| sources.visit(value))?;
    sources.part(data, |sources, value| sources.visit(value))?;
    if let Some(origin) = insertion_origin {
        sources.visit(origin)?;
    }
    sources.visit(selector_input)?;
    sources.part(admission_diagnostics, |sources, value| {
        sources.visit(value.as_slice())
    })
}

/// Charges and visits the sources of one block record.
pub(crate) fn walk_block<W: Walk>(
    sources: &mut W,
    block: &CssomBlock,
) -> Result<(), CssSourceRetentionError> {
    let CssomBlock {
        parent: _,
        binding,
        flags: _,
        data,
        admission_inputs: _,
        counter_edits,
        replacement_input,
        selected_font_face,
    } = block;
    sources.charge()?;
    if let crate::inline_context::BlockBinding::Inline(context) = binding {
        charge_context(sources, context.context())?;
    }
    sources.part(data, |sources, value| sources.visit(value))?;
    sources.part(counter_edits, |sources, value| {
        sources.visit(value.as_slice())
    })?;
    if let Some(part) = replacement_input {
        sources.part(part, |sources, value| {
            if let Some(source) = value.parse_source() {
                sources.source(source.source())?;
            }
            sources.visit(value.diagnostics())
        })?;
    }
    if let Some(part) = selected_font_face {
        sources.part(part, |sources, value| sources.visit(value))?;
    }
    Ok(())
}

/// Charges and visits the sources of one media list.
pub(crate) fn walk_media<W: Walk>(
    sources: &mut W,
    media: &CssMediaQueryList,
) -> Result<(), CssSourceRetentionError> {
    sources.charge()?;
    sources.visit(media)
}

/// Charges and visits the sources of one feature map.
pub(crate) fn walk_map<W: Walk>(
    sources: &mut W,
    map: &CssomFeatureMap,
) -> Result<(), CssSourceRetentionError> {
    let CssomFeatureMap {
        kind: _,
        entries,
        authored,
    } = map;
    sources.charge()?;
    sources.part(authored, |sources, value| sources.visit(value.as_slice()))?;
    for entry in entries {
        let CssomFeatureEntry { key: _, value } = entry;
        sources.charge()?;
        if let CssomFeatureValue::Authored(value) = value {
            sources.visit(value)?;
        }
    }
    Ok(())
}
