//! Actual native family state; cloning this storage preserves snapshots, while
//! DOM single-node cloning selects fresh initialization before copied attrs.
use crate::{Attribute, DocumentString, DomError, QualifiedName};
use std::sync::Arc;
use surgeist_html::bounded::{
    IntegerPrefixScanner, IntegerUnitStepError, NumericMaterializationError,
};
use surgeist_html::{
    HtmlCanvasBitmapDimension, HtmlCanvasNoneState, HtmlFrameNullContent, HtmlIframeNativeState,
    HtmlLoading, HtmlMarqueeState, HtmlNonNegativeInteger, HtmlValueInterpreter,
    KnownHtmlAttribute,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NativeFamily {
    Common,
    Template(surgeist_html::HtmlTemplateFreshInsertionState),
    Form(surgeist_html::HtmlFormNativeState),
    Fieldset(crate::DomFieldsetState),
    Button(crate::DomButtonState),
    Slot,
    Script,
    Canvas(Arc<HtmlCanvasNoneState>),
    Marquee(HtmlMarqueeState),
    Frame(HtmlFrameNullContent),
    Iframe(HtmlIframeNativeState),
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
            Self::Fieldset(state) => Some(state.control.message.len()),
            Self::Button(state) => Some(state.control.message.len()),
            Self::Canvas(state) => units(state.width()).checked_add(units(state.height())),
            Self::Form(_)
            | Self::Common
            | Self::Template(_)
            | Self::Slot
            | Self::Script
            | Self::Marquee(_)
            | Self::Frame(_)
            | Self::Iframe(_) => Some(0),
        }
    }

    pub(super) fn after_attribute(
        &self,
        element_name: &QualifiedName,
        name: &QualifiedName,
        previous: &[Attribute],
        attributes: &[Attribute],
        removed: bool,
        admit: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        if matches!(self, Self::Frame(_) | Self::Iframe(_)) {
            // One actual child-navigable hook boundary. The common src guard
            // checks content before Iframe's additional srcdoc-absence condition.
            if attribute_is(name, KnownHtmlAttribute::Src) {
                admit()?;
                let process = match self {
                    Self::Frame(state) => state.src_change_requires_processing(),
                    Self::Iframe(state) => state.src_change_requires_processing(),
                    _ => unreachable!("native navigable-container family"),
                };
                if process {
                    unreachable!("stored native null content cannot process src attributes")
                }
            }
            if let Self::Iframe(state) = self {
                if attribute_is(name, KnownHtmlAttribute::Srcdoc) {
                    admit()?;
                    if state.srcdoc_change_requires_processing() {
                        unreachable!("stored native null content cannot process srcdoc")
                    }
                } else if attribute_is(name, KnownHtmlAttribute::Sandbox) {
                    admit()?;
                    let process = if removed {
                        state.sandbox_removal_requires_clearing()
                    } else {
                        state.sandbox_change_requires_processing()
                    };
                    if process {
                        unreachable!("stored native unprocessed sandbox has no live policy write")
                    }
                } else if attribute_is(name, KnownHtmlAttribute::Loading) {
                    let old = loading_state(element_name, previous, admit)?;
                    let current = loading_state(element_name, attributes, admit)?;
                    admit()?;
                    if surgeist_html::iframe_loading_became_eager(old, current)
                        && !state.lazy_load_resumption_is_null()
                    {
                        // A resource-capable representation must clear before
                        // invoking actual steps. It cannot enter this phase.
                        unreachable!("native Iframe resumption is actually null")
                    }
                }
            }
            return Ok(self.clone());
        }
        if !matches!(self, Self::Canvas(_)) || dimension(name).is_none() {
            return Ok(self.clone());
        }
        // Read the complete actual candidate attributes. Recomputing both
        // dimensions through shared grammar avoids cloning unbounded Strings.
        let mut width = None;
        let mut height = None;
        for attribute in attributes {
            admit()?;
            match dimension(attribute.name()) {
                Some(Dimension::Width) => width = parse_dimension(attribute.value(), admit)?,
                Some(Dimension::Height) => height = parse_dimension(attribute.value(), admit)?,
                None => {}
            }
        }
        Ok(Self::Canvas(Arc::new(
            HtmlCanvasNoneState::from_parsed_attributes(width, height),
        )))
    }
}

fn attribute_is(name: &QualifiedName, attribute: KnownHtmlAttribute) -> bool {
    name.namespace().uri().is_none()
        && name
            .local_name()
            .code_units()
            .iter()
            .copied()
            .eq(attribute.local_name().encode_utf16())
}

fn loading_state(
    element: &QualifiedName,
    attributes: &[Attribute],
    admit: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<HtmlLoading, DomError> {
    let mut raw = None;
    for attribute in attributes {
        admit()?;
        if attribute_is(attribute.name(), KnownHtmlAttribute::Loading) {
            raw = Some(attribute);
            break;
        }
    }
    admit()?;
    // Raw storage has already passed DOM admission. The HTML rule compares two
    // fixed keywords (at most five units each), with no unbounded materialization.
    // It owns missing/invalid defaults. Exact raw length admits either operand
    // without inventing a separate host fact or policy limit.
    let report = HtmlValueInterpreter::new(raw.map_or(0, |a| a.value().len()))
        .loading(element, raw)
        .expect("actual admitted attribute fits its exact raw-unit cap");
    Ok(*report
        .intrinsic()
        .expect("actual HTML Iframe loading rule is applicable"))
}

enum Dimension {
    Width,
    Height,
}
fn dimension(name: &QualifiedName) -> Option<Dimension> {
    if name.namespace().uri().is_some() {
        return None;
    }
    let matches = |attribute: KnownHtmlAttribute| {
        name.local_name()
            .code_units()
            .iter()
            .copied()
            .eq(attribute.local_name().encode_utf16())
    };
    if matches(KnownHtmlAttribute::Width) {
        Some(Dimension::Width)
    } else if matches(KnownHtmlAttribute::Height) {
        Some(Dimension::Height)
    } else {
        None
    }
}

fn parse_dimension(
    raw: &DocumentString,
    admit: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<Option<HtmlNonNegativeInteger>, DomError> {
    let mut scanner = IntegerPrefixScanner::new(true);
    for &unit in raw.code_units() {
        admit()?;
        if !scanner.feed(unit) {
            break;
        }
    }
    admit()?;
    let Ok(prefix) = scanner.finish() else {
        return Ok(None);
    };
    let Ok(digits) = prefix.try_nonnegative_digits(raw.code_units()) else {
        return Ok(None);
    };
    admit()?;
    HtmlNonNegativeInteger::from_borrowed_digits_work_bound(digits.len())
        .ok_or(DomError::RepresentationExhausted)?;
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
    admit()?;
    Ok(Some(materialized.into_parts().0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    #[test]
    fn cancellation_during_actual_dimension_scan_stops_without_mutating_family() {
        let family = NativeFamily::Canvas(Arc::new(HtmlCanvasNoneState::fresh()));
        let name = QualifiedName::new(crate::Namespace::none(), None, "width");
        let attributes = [Attribute::new(name.clone(), "123456789")];
        let cancel = AtomicBool::new(false);
        let mut reads = 0;
        let element = QualifiedName::new(crate::Namespace::html(), None, "canvas");
        let result = family.after_attribute(&element, &name, &[], &attributes, false, &mut || {
            reads += 1;
            if reads == 5 {
                cancel.store(true, Ordering::Release);
            }
            crate::store::check_cancel(&cancel)
        });
        assert_eq!(result, Err(DomError::Cancelled));
        assert_eq!(reads, 5);
        assert_eq!(family.units(), Some(0));
        assert_eq!(
            family,
            NativeFamily::Canvas(Arc::new(HtmlCanvasNoneState::fresh()))
        );
    }
}
