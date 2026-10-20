//! Actual native family state; cloning this storage preserves snapshots, while
//! DOM single-node cloning selects fresh initialization before copied attrs.
use super::ElementFormation;
use crate::store::payload::Payload as Materialized;
use crate::store::source::{formation::Formation, storage::Payload};
#[cfg(test)]
use crate::{Attribute, QualifiedName};
use crate::{DocumentString, DomError};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use surgeist_html::bounded::{
    IntegerPrefixScanner, IntegerUnitStepError, NumericMaterializationError,
};
use surgeist_html::{
    HtmlAttributeInput, HtmlCanvasBitmapDimension, HtmlCanvasNoneState, HtmlIframeInitialState,
    HtmlLoading, HtmlMarqueeState, HtmlNameInput, HtmlNonNegativeInteger, HtmlValueInterpreter,
    KnownHtmlAttribute,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct IframeState {
    pub(super) fullscreen: bool,
    pub(super) initial: HtmlIframeInitialState,
    pub(super) sandbox: surgeist_html::HtmlSandboxFlags,
}
impl IframeState {
    pub(crate) fn fresh() -> Self {
        Self {
            fullscreen: false,
            initial: HtmlIframeInitialState::fresh(),
            sandbox: surgeist_html::HtmlSandboxFlags::empty(),
        }
    }
}

pub(super) struct AttributeMutation<'a, N: HtmlNameInput + ?Sized, A: HtmlAttributeInput> {
    pub element_name: &'a N,
    pub name: &'a N,
    pub previous: &'a [A],
    pub attributes: &'a [A],
    pub content: Option<surgeist_html::HtmlChildNavigablePresence>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NativeFamily {
    Common,
    Details,
    Dialog(Arc<Materialized<crate::dialog::DomDialogState>>),
    Meta,
    Style(Arc<Materialized<crate::DomStyleState>>),
    Base,
    Link,
    Img,
    Picture,
    Source,
    Hyperlink,
    Template,
    Form(surgeist_html::HtmlFormNativeState),
    Fieldset(Arc<Materialized<crate::DomFieldsetState>>),
    Button(Arc<Materialized<crate::DomButtonState>>),
    Input(Arc<Materialized<crate::DomInputState>>),
    Textarea(Arc<Materialized<crate::DomTextareaState>>),
    Output(Arc<Materialized<crate::DomOutputState>>),
    Select(Arc<Materialized<crate::DomSelectState>>),
    Option(crate::DomOptionState),
    Optgroup,
    Selectedcontent(bool),
    Slot,
    Script,
    SvgScript(crate::svg_script::SvgScriptState),
    Canvas(Arc<Materialized<HtmlCanvasNoneState>>),
    Marquee(HtmlMarqueeState),
    Frame,
    Iframe(IframeState),
}

impl NativeFamily {
    pub(super) fn units(&self) -> Option<usize> {
        fn units(value: HtmlCanvasBitmapDimension<'_>) -> usize {
            match value {
                HtmlCanvasBitmapDimension::Default(_) => 0,
                HtmlCanvasBitmapDimension::Parsed(value) => value.decimal_digits().len(),
            }
        }
        match self {
            Self::Dialog(state) => state.value.units(),
            Self::Fieldset(state) => Some(state.value.control.message.len()),
            Self::Button(state) => Some(state.value.control.message.len()),
            Self::Input(state) => state.value.units(),
            Self::Textarea(state) => state.value.units(),
            Self::Output(state) => state.value.units(),
            Self::Select(state) => Some(state.value.control.message.len()),
            Self::Canvas(state) => {
                units(state.value.width()).checked_add(units(state.value.height()))
            }
            Self::Form(_)
            | Self::Common
            | Self::Details
            | Self::Meta
            | Self::Style(_)
            | Self::Base
            | Self::Link
            | Self::Img
            | Self::Picture
            | Self::Source
            | Self::Hyperlink
            | Self::Template
            | Self::Option(_)
            | Self::Optgroup
            | Self::Selectedcontent(_)
            | Self::Slot
            | Self::Script
            | Self::SvgScript(_)
            | Self::Marquee(_)
            | Self::Frame
            | Self::Iframe(_) => Some(0),
        }
    }

    pub(super) fn after_attribute<N: HtmlNameInput + ?Sized, A: HtmlAttributeInput>(
        &self,
        change: AttributeMutation<'_, N, A>,
        admit: &mut impl FnMut() -> Result<(), DomError>,
        formation: &mut ElementFormation<'_>,
    ) -> Result<Self, DomError> {
        let budget = formation.budget;
        let cancel = formation.cancel;
        let AttributeMutation {
            element_name,
            name,
            previous,
            attributes,
            content,
        } = change;
        if matches!(self, Self::Frame | Self::Iframe(_)) {
            // One actual child-navigable hook boundary. The common src guard
            // checks content before Iframe's additional srcdoc-absence condition.
            if attribute_is(name, KnownHtmlAttribute::Src, budget, cancel)? {
                admit()?;
                if content
                    .expect("actual container relation")
                    .attribute_change_requires_processing()
                {
                    return Err(DomError::HostedSourceUnavailable);
                }
            }

            if let Self::Iframe(state) = self {
                if attribute_is(name, KnownHtmlAttribute::Srcdoc, budget, cancel)? {
                    admit()?;
                    if content
                        .expect("actual container relation")
                        .attribute_change_requires_processing()
                    {
                        return Err(DomError::HostedSourceUnavailable);
                    }
                } else if attribute_is(name, KnownHtmlAttribute::Sandbox, budget, cancel)? {
                    admit()?;
                    if content
                        .expect("actual container relation")
                        .attribute_change_requires_processing()
                    {
                        let mut raw = None;
                        for attribute in attributes {
                            admit()?;
                            if attribute_is(
                                attribute.name(),
                                KnownHtmlAttribute::Sandbox,
                                budget,
                                cancel,
                            )? {
                                raw = Some(attribute.value());
                                break;
                            }
                        }
                        let sandbox = match raw {
                            Some(raw) => surgeist_html::HtmlSandboxFlags::parse_with(raw, admit)?,
                            None => surgeist_html::HtmlSandboxFlags::empty(),
                        };
                        return Self::Iframe(IframeState { sandbox, ..*state })
                            .shared(budget, cancel);
                    }
                } else if attribute_is(name, KnownHtmlAttribute::Loading, budget, cancel)? {
                    let old = loading_state(element_name, previous, admit, budget, cancel)?;
                    let current = loading_state(element_name, attributes, admit, budget, cancel)?;
                    admit()?;
                    if surgeist_html::iframe_loading_became_eager(old, current)
                        && !state.initial.lazy_load_resumption_is_null()
                    {
                        // A resource-capable representation must clear before
                        // invoking actual steps. It cannot enter this phase.
                        unreachable!("native Iframe resumption is actually null")
                    }
                }
            }
            return self.shared(budget, cancel);
        }
        if !matches!(self, Self::Canvas(_)) || dimension(name, budget, cancel)?.is_none() {
            return self.shared(budget, cancel);
        }
        Self::canvas_from_attributes(attributes, admit, formation)
    }
    fn shared(&self, _budget: Option<&Formation>, cancel: &AtomicBool) -> Result<Self, DomError> {
        crate::store::check_cancel(cancel)?;
        // These are aliases of the same materializations. Their original
        // identities and child custody survive; no body/string is formed here.
        Ok(self.clone())
    }
    /// Pure owned Canvas dimension interpretation shared with actual retained
    /// materialization; this performs no attribute hooks or bitmap reset.
    pub(crate) fn canvas_from_attributes<A: HtmlAttributeInput>(
        attributes: &[A],
        admit: &mut impl FnMut() -> Result<(), DomError>,
        formation: &mut ElementFormation<'_>,
    ) -> Result<Self, DomError> {
        let budget = formation.budget;
        let cancel = formation.cancel;
        let mut width = None;
        let mut height = None;
        for attribute in attributes {
            admit()?;
            let target = match dimension(attribute.name(), budget, cancel)? {
                Some(Dimension::Width) => &mut width,
                Some(Dimension::Height) => &mut height,
                None => continue,
            };
            let parsed = parse_dimension(attribute.value(), admit, budget, cancel)?;
            let old_bytes = target
                .as_ref()
                .map_or(0, HtmlNonNegativeInteger::retained_buffer_bytes);
            *target = parsed;
            if let Some(budget) = budget {
                budget.release_payload(Payload {
                    bytes: old_bytes,
                    units: old_bytes,
                });
            }
        }
        if let Some(budget) = budget {
            // The owner documents at most three canonical digits per default
            // normalization. It consumes the parsed integers without copying.
            for _ in 0..6 {
                budget.step(cancel)?;
            }
        }
        let before = width
            .as_ref()
            .map_or(0, HtmlNonNegativeInteger::retained_buffer_bytes)
            .checked_add(
                height
                    .as_ref()
                    .map_or(0, HtmlNonNegativeInteger::retained_buffer_bytes),
            )
            .ok_or(DomError::RepresentationExhausted)?;
        let state = HtmlCanvasNoneState::from_parsed_attributes(width, height);
        if let Some(budget) = budget {
            let mut retained = 0usize;
            for dimension in [state.width(), state.height()] {
                if let HtmlCanvasBitmapDimension::Parsed(value) = dimension {
                    retained = retained
                        .checked_add(value.retained_buffer_bytes())
                        .ok_or(DomError::RepresentationExhausted)?;
                }
            }
            let released = before
                .checked_sub(retained)
                .expect("normalization only releases parsed inputs");
            budget.release_payload(Payload {
                bytes: released,
                units: released,
            });
        }
        let mut extra = Payload::default();
        for dimension in [state.width(), state.height()] {
            if let HtmlCanvasBitmapDimension::Parsed(value) = dimension {
                extra.add_bytes(value.retained_buffer_bytes())?;
                extra.add_units(value.retained_buffer_bytes())?;
            }
        }
        let original = if budget.is_some() {
            extra
        } else {
            Payload::default()
        };
        Ok(Self::Canvas(
            formation.native(extra, original, || Ok(state))?,
        ))
    }
}

fn attribute_is(
    name: &(impl HtmlNameInput + ?Sized),
    attribute: KnownHtmlAttribute,
    budget: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<bool, DomError> {
    if let Some(budget) = budget {
        budget.step(cancel)?;
    }
    if name.namespace_uri().is_some() {
        return Ok(false);
    }
    let mut expected = attribute.local_name().encode_utf16();
    for unit in name.local_name().code_units() {
        if let Some(budget) = budget {
            budget.step(cancel)?;
        }
        if Some(*unit) != expected.next() {
            return Ok(false);
        }
    }
    Ok(expected.next().is_none())
}

fn loading_state<N: HtmlNameInput + ?Sized, A: HtmlAttributeInput>(
    element: &N,
    attributes: &[A],
    admit: &mut impl FnMut() -> Result<(), DomError>,
    budget: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<HtmlLoading, DomError> {
    let mut raw = None;
    for attribute in attributes {
        admit()?;
        if attribute_is(
            attribute.name(),
            KnownHtmlAttribute::Loading,
            budget,
            cancel,
        )? {
            raw = Some(attribute);
            break;
        }
    }
    admit()?;
    // Raw storage has already passed DOM admission. The HTML rule compares two
    // fixed keywords (at most five units each), with no unbounded materialization.
    // It owns missing/invalid defaults. Exact raw length admits either operand
    // without inventing a separate host fact or policy limit.
    let mut report_payload = Payload::default();
    if let Some(budget) = budget {
        report_payload.name(element)?;
        if let Some(raw) = raw {
            report_payload.name(raw.name())?;
            report_payload.string(raw.value())?;
        }
        budget.retain_payload(report_payload, cancel)?;
        // The actual LOADING rule first checks attribute/namespace, performs
        // one closed KnownHtmlElement lookup and compares its two keywords.
        // Preadmit the complete public-vocabulary lookup (early match reduces
        // it), both namespace comparisons, and both raw keyword comparisons.
        budget.step(cancel)?;
        for _ in 0..2 {
            for _ in 0..=element.namespace_uri().map_or(0, DocumentString::len) {
                budget.step(cancel)?;
            }
        }
        for known in surgeist_html::KnownHtmlElement::ALL {
            budget.step(cancel)?;
            for _ in 0..=known.local_name().len() {
                budget.step(cancel)?;
            }
        }
        if let Some(raw) = raw {
            for _ in 0..=raw.name().local_name().len() {
                budget.step(cancel)?;
            }
            for _ in [HtmlLoading::Lazy, HtmlLoading::Eager] {
                budget.step(cancel)?;
                for _ in 0..=raw.value().len() {
                    budget.step(cancel)?;
                }
            }
        }
    }
    let report = HtmlValueInterpreter::new(raw.map_or(0, |a| a.value().len()))
        .loading(element, raw)
        .expect("actual admitted attribute fits its exact raw-unit cap");
    let value = *report
        .intrinsic()
        .expect("actual HTML Iframe loading rule is applicable");
    drop(report);
    if let Some(budget) = budget {
        budget.release_payload(report_payload);
    }
    Ok(value)
}

enum Dimension {
    Width,
    Height,
}
fn dimension(
    name: &(impl HtmlNameInput + ?Sized),
    budget: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<Option<Dimension>, DomError> {
    if attribute_is(name, KnownHtmlAttribute::Width, budget, cancel)? {
        Ok(Some(Dimension::Width))
    } else if attribute_is(name, KnownHtmlAttribute::Height, budget, cancel)? {
        Ok(Some(Dimension::Height))
    } else {
        Ok(None)
    }
}

fn parse_dimension(
    raw: &DocumentString,
    admit: &mut impl FnMut() -> Result<(), DomError>,
    budget: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<Option<HtmlNonNegativeInteger>, DomError> {
    let mut scanner = IntegerPrefixScanner::new(true);
    for unit in raw.code_units() {
        admit()?;
        if !scanner.feed(*unit) {
            break;
        }
    }
    admit()?;
    let Ok(prefix) = scanner.finish() else {
        return Ok(None);
    };
    if let Some(budget) = budget {
        let visits = prefix
            .end()
            .checked_sub(prefix.start())
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| n.checked_add(2))
            .ok_or(DomError::RepresentationExhausted)?;
        for _ in 0..visits {
            budget.step(cancel)?;
        }
    }
    let Ok(digits) = prefix.try_nonnegative_digits(raw.code_units()) else {
        return Ok(None);
    };
    admit()?;
    let work = HtmlNonNegativeInteger::from_borrowed_digits_work_bound(digits.len())
        .ok_or(DomError::RepresentationExhausted)?;
    if let Some(budget) = budget {
        for _ in 0..work {
            budget.step(cancel)?;
        }
        // The original bounded numeric materializer owns the byte capacity
        // limit; no private String layout or numeric grammar is reproduced.
        budget.retain_payload(
            Payload {
                bytes: digits.len(),
                units: digits.len(),
            },
            cancel,
        )?;
    }
    // Scratch is bounded by the admitted raw prefix, independently of the
    // final digit charge (explicit defaults normalize to inline values).
    let materialized =
        HtmlNonNegativeInteger::try_from_borrowed_digits(digits, digits.len(), digits.len())
            .map_err(|failure| match failure.kind() {
                NumericMaterializationError::Boundary(
                    IntegerUnitStepError::RepresentationExhausted,
                ) => DomError::RepresentationExhausted,
                _ => DomError::AllocationFailed,
            })?;
    let (value, bytes) = materialized.into_parts();
    if let Some(budget) = budget {
        let released = digits
            .len()
            .checked_sub(bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        budget.release_payload(Payload {
            bytes: released,
            units: released,
        });
    }
    admit()?;
    Ok(Some(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    fn registry() -> crate::store::bank::CustodyRegistry {
        crate::store::bank::CustodyRegistry::new(&crate::store::custody::Ledger::default(), |_| {
            Ok(())
        })
        .unwrap()
    }
    fn canvas() -> NativeFamily {
        let registry = registry();
        let cancel = AtomicBool::new(false);
        let mut admit = |_| Ok(());
        let mut formation = ElementFormation::new(&registry, None, &cancel, &mut admit);
        NativeFamily::Canvas(
            formation
                .native(Payload::default(), Payload::default(), || {
                    Ok(HtmlCanvasNoneState::fresh())
                })
                .unwrap(),
        )
    }
    #[test]
    fn iframe_sandbox_mutation_updates_only_the_actual_nonnull_content_slot() {
        use surgeist_html::{HtmlChildNavigablePresence as Presence, HtmlSandboxFlag};
        let cancel = AtomicBool::new(false);
        let registry = registry();
        let element = QualifiedName::new(crate::Namespace::html(), None, "iframe");
        let name = QualifiedName::new(crate::Namespace::none(), None, "sandbox");
        let update = |family: &NativeFamily, attributes: &[Attribute], content| {
            family
                .after_attribute(
                    AttributeMutation {
                        element_name: &element,
                        name: &name,
                        previous: &[],
                        attributes,
                        content: Some(content),
                    },
                    &mut || crate::store::check_cancel(&cancel),
                    &mut ElementFormation::new(&registry, None, &cancel, &mut |_| Ok(())),
                )
                .unwrap()
        };
        let fresh = NativeFamily::Iframe(IframeState::fresh());
        let empty = [Attribute::new(name.clone(), "")];
        assert_eq!(update(&fresh, &empty, Presence::Null), fresh);
        let restricted = update(&fresh, &empty, Presence::Present);
        let NativeFamily::Iframe(restricted_state) = restricted else {
            unreachable!()
        };
        assert!(restricted_state.sandbox.contains(HtmlSandboxFlag::Origin));
        assert_eq!(restricted_state.initial, HtmlIframeInitialState::fresh());
        let same_origin = [Attribute::new(name.clone(), "allow-same-origin")];
        let allowed = update(&restricted, &same_origin, Presence::Present);
        let NativeFamily::Iframe(allowed_state) = allowed else {
            unreachable!()
        };
        assert!(!allowed_state.sandbox.contains(HtmlSandboxFlag::Origin));
        assert!(restricted_state.sandbox.contains(HtmlSandboxFlag::Origin));
        assert_eq!(update(&restricted, &[], Presence::Null), restricted);
        assert_eq!(update(&restricted, &[], Presence::Present), fresh);
        assert_eq!(
            update(&NativeFamily::Frame, &empty, Presence::Present),
            NativeFamily::Frame
        );
    }
    #[test]
    fn cancelled_iframe_sandbox_parse_preserves_the_original_family() {
        let family = NativeFamily::Iframe(IframeState::fresh());
        let name = QualifiedName::new(crate::Namespace::none(), None, "sandbox");
        let attributes = [Attribute::new(
            name.clone(),
            "allow-same-origin allow-scripts",
        )];
        let cancel = AtomicBool::new(false);
        let registry = registry();
        let element = QualifiedName::new(crate::Namespace::html(), None, "iframe");
        let mut reads = 0;
        let result = family.after_attribute(
            AttributeMutation {
                element_name: &element,
                name: &name,
                previous: &[],
                attributes: &attributes,
                content: Some(surgeist_html::HtmlChildNavigablePresence::Present),
            },
            &mut || {
                reads += 1;
                if reads == 8 {
                    cancel.store(true, Ordering::Release);
                }
                crate::store::check_cancel(&cancel)
            },
            &mut ElementFormation::new(&registry, None, &cancel, &mut |_| Ok(())),
        );
        assert_eq!(result, Err(DomError::Cancelled));
        assert_eq!(reads, 8);
        assert_eq!(family, NativeFamily::Iframe(IframeState::fresh()));
    }
    #[test]
    fn cancellation_during_actual_dimension_scan_stops_without_mutating_family() {
        let family = canvas();
        let name = QualifiedName::new(crate::Namespace::none(), None, "width");
        let attributes = [Attribute::new(name.clone(), "123456789")];
        let cancel = AtomicBool::new(false);
        let registry = registry();
        let mut reads = 0;
        let element = QualifiedName::new(crate::Namespace::html(), None, "canvas");
        let result = family.after_attribute(
            AttributeMutation {
                element_name: &element,
                name: &name,
                previous: &[],
                attributes: &attributes,
                content: None,
            },
            &mut || {
                reads += 1;
                if reads == 5 {
                    cancel.store(true, Ordering::Release);
                }
                crate::store::check_cancel(&cancel)
            },
            &mut ElementFormation::new(&registry, None, &cancel, &mut |_| Ok(())),
        );
        assert_eq!(result, Err(DomError::Cancelled));
        assert_eq!(reads, 5);
        assert_eq!(family.units(), Some(0));
        assert_eq!(family, canvas());
    }
}
