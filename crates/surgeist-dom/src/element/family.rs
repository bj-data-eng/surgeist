//! Actual native family state; cloning this storage preserves snapshots, while
//! DOM single-node cloning selects fresh initialization before copied attrs.
use crate::{Attribute, DocumentString, DomError, QualifiedName};
use std::sync::Arc;
use surgeist_html::bounded::{
    IntegerPrefixScanner, IntegerUnitStepError, NumericMaterializationError,
};
use surgeist_html::{
    HtmlCanvasBitmapDimension, HtmlCanvasNoneState, HtmlFrameNullContent, HtmlMarqueeState,
    HtmlNonNegativeInteger, KnownHtmlAttribute,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NativeFamily {
    Common,
    Fieldset(crate::DomFieldsetState),
    Slot,
    Canvas(Arc<HtmlCanvasNoneState>),
    Marquee(HtmlMarqueeState),
    Frame(HtmlFrameNullContent),
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
            Self::Fieldset(state) => Some(state.message.len()),
            Self::Canvas(state) => units(state.width()).checked_add(units(state.height())),
            Self::Common | Self::Slot | Self::Marquee(_) | Self::Frame(_) => Some(0),
        }
    }

    pub(super) fn after_attribute(
        &self,
        name: &QualifiedName,
        attributes: &[Attribute],
        admit: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        if let Self::Frame(state) = self {
            // Source Frame src set/change/removal processing first requires
            // non-null content. Actual expanded identity is tested, not the
            // qualified spelling used by an ordinary attribute operation.
            if name.namespace().uri().is_none()
                && name.local_name().code_units() == [0x73, 0x72, 0x63]
            {
                admit()?;
                if state.src_change_requires_processing() {
                    unreachable!("stored Frame null content cannot process src attributes")
                }
            }
            return Ok(Self::Frame(*state));
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
        let result = family.after_attribute(&name, &attributes, &mut || {
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
